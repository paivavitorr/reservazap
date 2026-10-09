use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::pix_provider::{PixCharge, PixProvider};
use crate::models::appointments::AppointmentModel;
use crate::models::_entities::appointments::AppointmentStatus;
use crate::models::_entities::services;

#[derive(Debug, thiserror::Error)]
pub enum BookingError {
    #[error("serviço não encontrado")]
    ServiceNotFound,
    #[error("profissional não encontrado")]
    ProfessionalNotFound,
    #[error("horário indisponível")]
    SlotUnavailable,
    #[error("falha na cobrança Pix: {0}")]
    PixFailed(String),
    #[error("agendamento não encontrado")]
    AppointmentNotFound,
    #[error("agendamento já expirado")]
    HoldExpired,
    #[error("transição de status inválida")]
    InvalidTransition,
    #[error("erro de banco: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub struct BookingService;

impl BookingService {
    pub async fn criar_intencao(
        db: &DatabaseConnection,
        pix: &dyn PixProvider,
        tenant_id: Uuid,
        service_id: Uuid,
        professional_id: Uuid,
        starts_at: DateTime<Utc>,
        client_name: String,
        client_phone: String,
        client_email: Option<String>,
        hold_minutes: i32,
    ) -> Result<(Uuid, PixCharge, DateTime<Utc>), BookingError> {
        use sea_orm::EntityTrait;
        let service = services::Entity::find_by_id(service_id)
            .one(db)
            .await?
            .ok_or(BookingError::ServiceNotFound)?;

        let ends_at = starts_at + chrono::Duration::minutes(service.duration_minutes as i64);
        let signal_amount = service.signal_amount();

        let charge = pix
            .criar_cobranca(
                signal_amount,
                &format!("Sinal - {}", service.name),
                (hold_minutes * 60) as u32,
            )
            .await
            .map_err(|e| BookingError::PixFailed(e.to_string()))?;

        let appointment = AppointmentModel::create_hold(
            db,
            tenant_id,
            service_id,
            professional_id,
            starts_at,
            ends_at,
            client_name,
            client_phone,
            client_email,
            signal_amount,
            hold_minutes,
            charge.txid.clone(),
            charge.copia_e_cola.clone(),
        )
        .await
            .map_err(|e| {
                if e.to_string().contains("no_overlap") {
                    BookingError::SlotUnavailable
                } else {
                    BookingError::Database(e)
                }
            })?;

        let hold_expires = appointment.hold_expires_at.with_timezone(&Utc);
        Ok((appointment.id, charge, hold_expires))
    }

    pub async fn confirmar_pagamento(
        db: &DatabaseConnection,
        appointment_id: Uuid,
    ) -> Result<bool, BookingError> {
        AppointmentModel::confirm_payment(db, appointment_id)
            .await
            .map_err(BookingError::Database)
    }

    pub async fn status_agendamento(
        db: &DatabaseConnection,
        appointment_id: Uuid,
    ) -> Result<(AppointmentStatus, Option<DateTime<Utc>>), BookingError> {
        let appt = AppointmentModel::find_by_id(db, appointment_id)
            .await?
            .ok_or(BookingError::AppointmentNotFound)?;

        let paid_at = appt.signal_paid_at.map(|dt| dt.with_timezone(&Utc));
        Ok((appt.status, paid_at))
    }
}