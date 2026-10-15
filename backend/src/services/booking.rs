use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use super::availability::{carregar_expediente, slot_cabe_no_expediente};
use super::pix_provider::{PixCharge, PixProvider};
use crate::models::_entities::appointments::AppointmentStatus;
use crate::models::_entities::professionals;
use crate::models::_entities::services;
use crate::models::_entities::tenants;
use crate::models::appointments::AppointmentModel;

#[derive(Debug, thiserror::Error)]
pub enum BookingError {
    #[error("serviço não encontrado")]
    ServiceNotFound,
    #[error("profissional não encontrado")]
    ProfessionalNotFound,
    #[error("horário indisponível")]
    SlotUnavailable,
    #[error("data ou horário inválido")]
    InvalidDate,
    #[error("dados de agendamento inválidos: {0}")]
    InvalidData(&'static str),
    #[error("falha na cobrança Pix: {0}")]
    PixFailed(String),
    #[error("agendamento não encontrado")]
    AppointmentNotFound,
    #[error("agendamento já expirado")]
    HoldExpired,
    #[error("transição de status inválida")]
    InvalidTransition,
    #[error("erro de disponibilidade: {0}")]
    Availability(#[from] super::availability::AvailabilityError),
    #[error("erro de banco: {0}")]
    Database(#[from] sea_orm::DbErr),
}

/// Detecta violação da constraint de exclusão `no_overlap` (SQLSTATE 23P01).
fn is_overlap_violation(err: &sea_orm::DbErr) -> bool {
    let texto = err.to_string().to_lowercase();
    texto.contains("no_overlap") || texto.contains("23p01") || texto.contains("exclusion")
}

pub struct BookingService;

impl BookingService {
    #[allow(clippy::too_many_arguments)]
    pub async fn criar_intencao(
        db: &DatabaseConnection,
        pix: &dyn PixProvider,
        tenant: &tenants::Model,
        service_id: Uuid,
        professional_id: Uuid,
        starts_at: DateTime<Utc>,
        client_name: String,
        client_phone: String,
        client_email: Option<String>,
    ) -> Result<(Uuid, PixCharge, DateTime<Utc>), BookingError> {
        use sea_orm::EntityTrait;

        validar_cliente(&client_name, &client_phone, client_email.as_deref())?;

        let hold_minutes = tenant.booking_hold_minutes;
        if hold_minutes <= 0 {
            return Err(BookingError::InvalidData("tempo de reserva inválido"));
        }

        let service = services::Entity::find_by_id(service_id)
            .one(db)
            .await?
            .filter(|s| s.tenant_id == tenant.id && s.is_active)
            .ok_or(BookingError::ServiceNotFound)?;

        let professional = professionals::Entity::find_by_id(professional_id)
            .one(db)
            .await?
            .filter(|p| p.tenant_id == tenant.id && p.is_active)
            .ok_or(BookingError::ProfessionalNotFound)?;

        if service.duration_minutes <= 0 {
            return Err(BookingError::InvalidData("duração do serviço inválida"));
        }

        let now = Utc::now();
        if starts_at <= now {
            return Err(BookingError::InvalidDate);
        }

        let ends_at = starts_at + chrono::Duration::minutes(service.duration_minutes as i64);

        // valida contra o expediente configurado
        let tz: chrono_tz::Tz = tenant
            .timezone
            .parse()
            .map_err(|_| BookingError::InvalidData("fuso horário do estabelecimento inválido"))?;
        let expediente = carregar_expediente(db, tenant.id, professional.id).await?;
        if !slot_cabe_no_expediente(&expediente, tz, starts_at, ends_at) {
            return Err(BookingError::InvalidData("horário fora do expediente"));
        }

        let signal_amount = service.signal_amount();

        let charge = pix
            .criar_cobranca(
                signal_amount,
                &format!("Sinal - {}", service.name),
                (hold_minutes * 60) as u32,
            )
            .await
            .map_err(|e| BookingError::PixFailed(e.to_string()))?;

        let resultado = AppointmentModel::create_hold(
            db,
            tenant.id,
            service_id,
            professional.id,
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
        .await;

        let appointment = match resultado {
            Ok(appointment) => appointment,
            Err(e) => {
                if let Err(cancel_err) = pix.cancelar_cobranca(&charge.txid).await {
                    tracing::warn!(
                        txid = %charge.txid,
                        error = %cancel_err,
                        "falha ao cancelar cobrança órfã"
                    );
                }
                return Err(if is_overlap_violation(&e) {
                    BookingError::SlotUnavailable
                } else {
                    BookingError::Database(e)
                });
            }
        };

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

    /// Confirma pelo `txid` do Pix — caminho usado pelos webhooks do PSP.
    /// Idempotente: agendamento já confirmado retorna `true` sem efeito.
    pub async fn confirmar_por_txid(
        db: &DatabaseConnection,
        txid: &str,
    ) -> Result<bool, BookingError> {
        let appt = AppointmentModel::find_by_txid(db, txid)
            .await?
            .ok_or(BookingError::AppointmentNotFound)?;

        match appt.status {
            AppointmentStatus::Confirmed => Ok(true),
            AppointmentStatus::Hold => AppointmentModel::confirm_payment(db, appt.id)
                .await
                .map_err(BookingError::Database),
            _ => Err(BookingError::InvalidTransition),
        }
    }

    pub async fn status_agendamento(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        appointment_id: Uuid,
    ) -> Result<crate::models::_entities::appointments::Model, BookingError> {
        AppointmentModel::find_by_id_for_tenant(db, tenant_id, appointment_id)
            .await?
            .ok_or(BookingError::AppointmentNotFound)
    }
}

fn validar_cliente(
    client_name: &str,
    client_phone: &str,
    client_email: Option<&str>,
) -> Result<(), BookingError> {
    let nome = client_name.trim();
    if nome.len() < 2 || nome.len() > 120 {
        return Err(BookingError::InvalidData(
            "nome deve ter de 2 a 120 caracteres",
        ));
    }

    let telefone: String = client_phone
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if telefone.len() < 10 || telefone.len() > 13 {
        return Err(BookingError::InvalidData("telefone inválido"));
    }

    if let Some(email) = client_email {
        let email = email.trim();
        if !email.is_empty() && (!email.contains('@') || email.len() > 160) {
            return Err(BookingError::InvalidData("e-mail inválido"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cliente_valido_passa() {
        assert!(validar_cliente("Maria Silva", "11987654321", Some("maria@x.com")).is_ok());
    }

    #[test]
    fn cliente_nome_curto_falha() {
        assert!(validar_cliente("M", "11987654321", None).is_err());
    }

    #[test]
    fn cliente_telefone_curto_falha() {
        assert!(validar_cliente("Maria Silva", "1198765", None).is_err());
    }

    #[test]
    fn cliente_email_invalido_falha() {
        assert!(validar_cliente("Maria Silva", "11987654321", Some("sem-arroba")).is_err());
    }

    #[test]
    fn detecta_violacao_de_overlap() {
        let err = sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
            "conflicting key value violates exclusion constraint \"no_overlap\"".to_string(),
        ));
        assert!(is_overlap_violation(&err));

        let outro = sea_orm::DbErr::Exec(sea_orm::RuntimeErr::Internal(
            "connection refused".to_string(),
        ));
        assert!(!is_overlap_violation(&outro));
    }
}
