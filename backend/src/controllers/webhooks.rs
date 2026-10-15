use axum::extract::Query;
use axum::http::HeaderMap;
use axum::routing::post;
use loco_rs::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

use crate::models::appointments::AppointmentModel;
use crate::models::payments::PaymentModel;
use crate::services::booking::{BookingError, BookingService};
use crate::services::notifications;
use crate::services::pix_provider::{self, PixStatus};

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct PixWebhookPayload {
    pub txid: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize)]
pub struct WebhookResponse {
    pub ok: bool,
    pub appointment_status: Option<String>,
}

/// Webhook de confirmação de pagamento Pix (contrato do provedor mock).
///
/// `{ "txid": "...", "status": "paid" }`. Idempotente por `txid` — entregar o
/// mesmo evento duas vezes não duplica confirmação.
async fn pix_webhook(
    State(ctx): State<AppContext>,
    Json(payload): Json<PixWebhookPayload>,
) -> Result<Json<WebhookResponse>> {
    if payload.txid.trim().is_empty() {
        return Err(Error::BadRequest("txid obrigatório".to_string()));
    }

    match payload.status.as_str() {
        "paid" => {
            let confirmado = BookingService::confirmar_por_txid(
                &ctx.db,
                "mock",
                &payload.txid,
                Some(serde_json::to_string(&payload).unwrap_or_default()),
            )
            .await
            .map_err(map_webhook_error)?;
            if !confirmado {
                return Ok(Json(WebhookResponse {
                    ok: false,
                    appointment_status: Some("late_paid".to_string()),
                }));
            }
            notificar_confirmacao(&ctx.db, &payload.txid).await;
            Ok(Json(WebhookResponse {
                ok: true,
                appointment_status: Some("confirmed".to_string()),
            }))
        }
        "cancelled" => {
            PaymentModel::atualizar_status(
                &ctx.db,
                &payload.txid,
                crate::models::_entities::payments::PaymentStatus::Cancelled,
                None,
                None,
            )
            .await?;
            Ok(Json(WebhookResponse {
                ok: true,
                appointment_status: None,
            }))
        }
        _ => Err(Error::BadRequest("status desconhecido".to_string())),
    }
}

/// Webhook do Mercado Pago (`payment.updated`).
///
/// Fluxo seguro: valida a assinatura `x-signature`, **consulta o pagamento no
/// PSP** (não confia no corpo do webhook) e só então confirma — idempotente.
async fn mercado_pago_webhook(
    State(ctx): State<AppContext>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Json<serde_json::Value>,
) -> Result<Json<WebhookResponse>> {
    let txid = query
        .get("data.id")
        .cloned()
        .or_else(|| {
            body.0
                .get("data")
                .and_then(|d| d.get("id"))
                .map(|v| v.to_string().trim_matches('"').to_string())
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::BadRequest("data.id obrigatório".to_string()))?;

    let secret = std::env::var("MERCADOPAGO_WEBHOOK_SECRET").unwrap_or_default();
    let em_producao = std::env::var("LOCO_ENV").unwrap_or_default() == "production";
    if secret.is_empty() {
        if em_producao {
            return Err(Error::Unauthorized(
                "webhook sem secret configurado".to_string(),
            ));
        }
        tracing::warn!("MERCADOPAGO_WEBHOOK_SECRET não configurado — assinatura não validada");
    } else {
        let signature = header_str(&headers, "x-signature").unwrap_or_default();
        let request_id = header_str(&headers, "x-request-id").unwrap_or_default();
        let (ts, v1) = pix_provider::parse_x_signature(&signature)
            .ok_or_else(|| Error::Unauthorized("assinatura ausente".to_string()))?;
        if !pix_provider::assinatura_webhook_valida(&secret, &txid, &request_id, &ts, &v1) {
            return Err(Error::Unauthorized("assinatura inválida".to_string()));
        }
    }

    // confere a situação real no PSP antes de agir
    let pix = pix_provider::from_env();
    let status = pix
        .consultar_cobranca(&txid)
        .await
        .map_err(|e| Error::string(&e.to_string()))?;

    match status {
        PixStatus::Paid => {
            let confirmado = BookingService::confirmar_por_txid(
                &ctx.db,
                "mercadopago",
                &txid,
                Some(body.0.to_string()),
            )
            .await
            .map_err(map_webhook_error)?;
            if confirmado {
                notificar_confirmacao(&ctx.db, &txid).await;
            }
            Ok(Json(WebhookResponse {
                ok: confirmado,
                appointment_status: Some(if confirmado {
                    "confirmed".to_string()
                } else {
                    "late_paid".to_string()
                }),
            }))
        }
        PixStatus::Cancelled => {
            PaymentModel::atualizar_status(
                &ctx.db,
                &txid,
                crate::models::_entities::payments::PaymentStatus::Cancelled,
                None,
                Some(body.0.to_string()),
            )
            .await?;
            Ok(Json(WebhookResponse {
                ok: true,
                appointment_status: None,
            }))
        }
        PixStatus::Expired => {
            PaymentModel::atualizar_status(
                &ctx.db,
                &txid,
                crate::models::_entities::payments::PaymentStatus::Expired,
                None,
                Some(body.0.to_string()),
            )
            .await?;
            Ok(Json(WebhookResponse {
                ok: true,
                appointment_status: None,
            }))
        }
        // ainda pendente ou desconhecido: não faz nada e acalma o retry do PSP
        _ => Ok(Json(WebhookResponse {
            ok: true,
            appointment_status: None,
        })),
    }
}

/// Notifica o cliente sobre a confirmação (best-effort).
async fn notificar_confirmacao(ctx_db: &sea_orm::DatabaseConnection, txid: &str) {
    match AppointmentModel::find_by_txid(ctx_db, txid).await {
        Ok(Some(appt)) => {
            let provider = notifications::from_env();
            notifications::notificar_confirmacao(ctx_db, &appt, &provider).await;
        }
        Ok(None) => {
            tracing::warn!(
                txid,
                "agendamento não encontrado para notificação de confirmação"
            );
        }
        Err(e) => {
            tracing::warn!(txid, error = %e, "falha ao buscar agendamento para notificação");
        }
    }
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn map_webhook_error(e: BookingError) -> Error {
    match e {
        BookingError::AppointmentNotFound => Error::NotFound,
        BookingError::InvalidTransition => {
            Error::BadRequest("agendamento não está mais em aberto".to_string())
        }
        other => Error::string(&other.to_string()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("webhooks")
        .add("/pix", post(pix_webhook))
        .add("/pix/mercadopago", post(mercado_pago_webhook))
}
