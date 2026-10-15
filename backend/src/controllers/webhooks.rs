use axum::routing::post;
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::services::booking::{BookingError, BookingService};

#[derive(Debug, Deserialize)]
pub struct PixWebhookPayload {
    pub txid: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize)]
pub struct WebhookResponse {
    pub ok: bool,
    pub appointment_status: Option<String>,
}

/// Webhook de confirmação de pagamento Pix.
///
/// Contrato do provedor mock (Fase 2 substitui pela validação do PSP):
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
            let confirmado = BookingService::confirmar_por_txid(&ctx.db, &payload.txid)
                .await
                .map_err(|e| match e {
                    BookingError::AppointmentNotFound => Error::NotFound,
                    BookingError::InvalidTransition => {
                        Error::BadRequest("agendamento não está mais em aberto".to_string())
                    }
                    other => Error::string(&other.to_string()),
                })?;
            if !confirmado {
                tracing::warn!(
                    txid = %payload.txid,
                    "pagamento recebido com hold vencido — horário não confirmado"
                );
                return Ok(Json(WebhookResponse {
                    ok: false,
                    appointment_status: Some("expired".to_string()),
                }));
            }
            Ok(Json(WebhookResponse {
                ok: true,
                appointment_status: Some("confirmed".to_string()),
            }))
        }
        "cancelled" => Ok(Json(WebhookResponse {
            ok: true,
            appointment_status: None,
        })),
        _ => Err(Error::BadRequest("status desconhecido".to_string())),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("webhooks")
        .add("/pix", post(pix_webhook))
}
