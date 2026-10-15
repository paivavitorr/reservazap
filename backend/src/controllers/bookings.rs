use axum::routing::{get, post};
use chrono::{DateTime, NaiveDate, Utc};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::models::_entities::services;
use crate::models::professionals::ProfessionalModel;
use crate::models::tenants::TenantModel;
use crate::services::availability::{self, AvailabilityError};
use crate::services::booking::{BookingError, BookingService};
use crate::services::pix_provider;

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

#[derive(Debug, Deserialize)]
pub struct AvailabilityQuery {
    pub professional_id: Uuid,
    pub service_id: Uuid,
    pub from: Option<String>,
    pub days: Option<i64>,
}

fn map_booking_error(e: BookingError) -> Error {
    match e {
        BookingError::SlotUnavailable => Error::BadRequest("horário indisponível".to_string()),
        BookingError::ServiceNotFound => Error::NotFound,
        BookingError::ProfessionalNotFound => Error::NotFound,
        BookingError::AppointmentNotFound => Error::NotFound,
        BookingError::InvalidDate => Error::BadRequest("data ou horário inválido".to_string()),
        BookingError::InvalidData(msg) => Error::BadRequest(msg.to_string()),
        other => Error::string(&other.to_string()),
    }
}

async fn create_intent(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Json(payload): Json<BookingIntentPayload>,
) -> Result<Json<BookingIntentResponse>> {
    let tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or(Error::NotFound)?;

    if !tenant.is_active {
        return Err(Error::NotFound);
    }

    let pix = pix_provider::from_env();

    let (appointment_id, charge, hold_expires) = BookingService::criar_intencao(
        &ctx.db,
        pix.as_ref(),
        &tenant,
        payload.service_id,
        payload.professional_id,
        payload.starts_at,
        payload.client_name,
        payload.client_phone,
        payload.client_email,
    )
    .await
    .map_err(map_booking_error)?;

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
    let tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or(Error::NotFound)?;

    let appt = BookingService::status_agendamento(&ctx.db, tenant.id, appointment_id)
        .await
        .map_err(map_booking_error)?;

    Ok(Json(StatusResponse {
        status: appt.status.as_str().to_string(),
        hold_expires_at: Some(appt.hold_expires_at.with_timezone(&Utc)),
        signal_paid_at: appt.signal_paid_at.map(|dt| dt.with_timezone(&Utc)),
    }))
}

async fn get_availability(
    State(ctx): State<AppContext>,
    Path(slug): Path<String>,
    Query(query): Query<AvailabilityQuery>,
) -> Result<Json<availability::Disponibilidade>> {
    let tenant = TenantModel::find_by_slug(&ctx.db, &slug)
        .await?
        .ok_or(Error::NotFound)?;

    if !tenant.is_active {
        return Err(Error::NotFound);
    }

    let service = services::Entity::find_by_id(query.service_id)
        .one(&ctx.db)
        .await?
        .filter(|s| s.tenant_id == tenant.id && s.is_active)
        .ok_or(Error::NotFound)?;

    let professional =
        ProfessionalModel::find_by_id_for_tenant(&ctx.db, tenant.id, query.professional_id)
            .await?
            .filter(|p| p.is_active)
            .ok_or(Error::NotFound)?;

    let hoje = chrono::Utc::now()
        .with_timezone(&chrono_tz::Tz::from_str(&tenant.timezone).unwrap_or(chrono_tz::Tz::UTC))
        .date_naive();

    let from = match query.from {
        Some(raw) => NaiveDate::parse_from_str(&raw, "%Y-%m-%d").map_err(|_| {
            Error::BadRequest("parâmetro 'from' inválido (use YYYY-MM-DD)".to_string())
        })?,
        None => hoje,
    };

    let disponivel = availability::disponibilidade(
        &ctx.db,
        &tenant,
        professional.id,
        service.duration_minutes,
        from,
        query.days.unwrap_or(7),
    )
    .await
    .map_err(|e| match e {
        AvailabilityError::TimezoneInvalida => Error::string("fuso horário do tenant inválido"),
        AvailabilityError::DataInvalida => Error::BadRequest("data inválida".to_string()),
        AvailabilityError::Database(inner) => Error::string(&inner.to_string()),
    })?;

    Ok(Json(disponivel))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("public")
        .add("/{slug}/bookings/intent", post(create_intent))
        .add("/{slug}/bookings/{appointment_id}/status", get(get_status))
        .add("/{slug}/availability", get(get_availability))
}
