use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::tenants::TenantModel;
use crate::services::booking::{BookingError, BookingService};
use crate::services::pix_provider::MockPixProvider;

#[derive(Debug, Deserialize)]
pub struct BookingIntentPayload {
    pub service_id: Uuid,
    pub professional_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub client_name: String,
    pub client_phone: String,
    pub client_email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BookingIntentResponse {
    pub appointment_id: Uuid,
    pub status: String,
    pub hold_expires_at: DateTime<Utc>,
    pub signal: SignalInfo,
}

#[derive(Debug, Serialize)]
pub struct SignalInfo {
    pub amount_in_cents: i64,
    pub pix: PixInfo,
}

#[derive(Debug, Serialize)]
pub struct PixInfo {
    pub txid: String,
    pub copia_e_cola: String,
    pub qr_code_base64: Option<String>,
    pub expiracao_segundos: u32,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub hold_expires_at: Option<DateTime<Utc>>,
    pub signal_paid_at: Option<DateTime<Utc>>,
}

async fn create_intent(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Json(payload): Json<BookingIntentPayload>,
) -> Result<Json<BookingIntentResponse>> {
    let tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or_else(|| Error::not_found("tenant não encontrado"))?;

    if !tenant.is_active {
        return Err(Error::not_found("tenant inativo"));
    }

    let pix = MockPixProvider;
    let hold_minutes = tenant.booking_hold_minutes;

    let (appointment_id, charge, hold_expires) = BookingService::criar_intencao(
        &ctx.db,
        &pix,
        tenant.id,
        payload.service_id,
        payload.professional_id,
        payload.starts_at,
        payload.client_name,
        payload.client_phone,
        payload.client_email,
        hold_minutes,
    )
    .await
    .map_err(|e| match e {
        BookingError::SlotUnavailable => Error::message("horário indisponível"),
        BookingError::ServiceNotFound => Error::not_found("serviço não encontrado"),
        BookingError::ProfessionalNotFound => Error::not_found("profissional não encontrado"),
        other => Error::string(&other.to_string()),
    })?;

    Ok(Json(BookingIntentResponse {
        appointment_id,
        status: "hold".to_string(),
        hold_expires_at: hold_expires,
        signal: SignalInfo {
            amount_in_cents: charge.valor_centavos,
            pix: PixInfo {
                txid: charge.txid,
                copia_e_cola: charge.copia_e_cola,
                qr_code_base64: charge.qr_code_base64,
                expiracao_segundos: charge.expiracao_segundos,
            },
        },
    }))
}

async fn get_status(
    State(ctx): State<AppContext>,
    Path((slug, appointment_id)): Path<(String, Uuid)>,
) -> Result<Json<StatusResponse>> {
    let _tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or_else(|| Error::not_found("tenant não encontrado"))?;

    let (status, paid_at) = BookingService::status_agendamento(&ctx.db, appointment_id)
        .await
        .map_err(|e| match e {
            BookingError::AppointmentNotFound => Error::not_found("agendamento não encontrado"),
            other => Error::string(&other.to_string()),
        })?;

    Ok(Json(StatusResponse {
        status: format!("{:?}", status).to_lowercase(),
        hold_expires_at: None,
        signal_paid_at: paid_at,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("public")
        .add("/{slug}/bookings/intent", post(create_intent))
        .add("/{slug}/bookings/{appointment_id}/status", get(get_status))
}