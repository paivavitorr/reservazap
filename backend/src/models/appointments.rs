use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::_entities::appointments::{self, AppointmentStatus, Entity as Appointment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lembrete {
    VinteQuatroHoras,
    UmaHora,
}

pub struct AppointmentModel;

impl AppointmentModel {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<appointments::Model>, sea_orm::DbErr> {
        Appointment::find_by_id(id).one(db).await
    }

    pub async fn find_by_id_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<appointments::Model>, sea_orm::DbErr> {
        Appointment::find_by_id(id)
            .filter(appointments::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    pub async fn find_by_txid(
        db: &DatabaseConnection,
        txid: &str,
    ) -> Result<Option<appointments::Model>, sea_orm::DbErr> {
        Appointment::find()
            .filter(appointments::Column::PixTxid.eq(txid))
            .one(db)
            .await
    }

    /// Agendamentos ativos (hold/confirmed) de um profissional que se sobrepõem
    /// ao intervalo `[starts_at, ends_at)`.
    pub async fn find_active_overlapping(
        db: &DatabaseConnection,
        professional_id: Uuid,
        starts_at: chrono::DateTime<Utc>,
        ends_at: chrono::DateTime<Utc>,
    ) -> Result<Vec<appointments::Model>, sea_orm::DbErr> {
        Appointment::find()
            .filter(appointments::Column::ProfessionalId.eq(professional_id))
            .filter(
                appointments::Column::Status
                    .is_in([AppointmentStatus::Hold, AppointmentStatus::Confirmed]),
            )
            .filter(appointments::Column::StartsAt.lt(ends_at.fixed_offset()))
            .filter(appointments::Column::EndsAt.gt(starts_at.fixed_offset()))
            .all(db)
            .await
    }

    pub async fn find_expired_holds(
        db: &DatabaseConnection,
    ) -> Result<Vec<appointments::Model>, sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        Appointment::find()
            .filter(appointments::Column::Status.eq(AppointmentStatus::Hold))
            .filter(appointments::Column::HoldExpiresAt.lt(now))
            .all(db)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_hold(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        service_id: Uuid,
        professional_id: Uuid,
        starts_at: chrono::DateTime<Utc>,
        ends_at: chrono::DateTime<Utc>,
        client_name: String,
        client_phone: String,
        client_email: Option<String>,
        signal_amount: i64,
        hold_minutes: i32,
        pix_txid: String,
        pix_payload: String,
    ) -> Result<appointments::Model, sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        let hold_expires = (now + chrono::Duration::minutes(hold_minutes as i64)).fixed_offset();

        let active = appointments::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            service_id: Set(service_id),
            professional_id: Set(professional_id),
            starts_at: Set(starts_at.fixed_offset()),
            ends_at: Set(ends_at.fixed_offset()),
            client_name: Set(client_name),
            client_phone: Set(client_phone),
            client_email: Set(client_email),
            status: Set(AppointmentStatus::Hold),
            signal_amount_in_cents: Set(signal_amount),
            signal_paid_at: Set(None),
            pix_txid: Set(Some(pix_txid)),
            pix_payload: Set(Some(pix_payload)),
            hold_expires_at: Set(hold_expires),
            cancelled_at: Set(None),
            cancellation_reason: Set(None),
            reminded_24h_at: Set(None),
            reminded_1h_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        active.insert(db).await
    }

    /// Marca um hold vencido como `expired`. Só afeta linhas ainda em `hold`,
    /// então uma confirmação concorrente nunca é sobrescrita. Retorna `false`
    /// quando nada foi expirado (ex.: confirmação chegou primeiro).
    pub async fn mark_expired(db: &DatabaseConnection, id: Uuid) -> Result<bool, sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        let res = Appointment::update_many()
            .set(appointments::ActiveModel {
                status: Set(AppointmentStatus::Expired),
                updated_at: Set(now),
                ..Default::default()
            })
            .filter(appointments::Column::Id.eq(id))
            .filter(appointments::Column::Status.eq(AppointmentStatus::Hold))
            .exec(db)
            .await?;
        Ok(res.rows_affected == 1)
    }

    /// Agendamentos confirmados que ainda não receberam o lembrete do tipo
    /// informado e cujo início está dentro da janela `[inicio, fim)`.
    pub async fn find_aguardando_lembrete(
        db: &DatabaseConnection,
        tipo: Lembrete,
        inicio: chrono::DateTime<Utc>,
        fim: chrono::DateTime<Utc>,
    ) -> Result<Vec<appointments::Model>, sea_orm::DbErr> {
        let coluna = match tipo {
            Lembrete::VinteQuatroHoras => appointments::Column::Reminded24hAt,
            Lembrete::UmaHora => appointments::Column::Reminded1hAt,
        };
        Appointment::find()
            .filter(appointments::Column::Status.eq(AppointmentStatus::Confirmed))
            .filter(appointments::Column::StartsAt.gte(inicio.fixed_offset()))
            .filter(appointments::Column::StartsAt.lt(fim.fixed_offset()))
            .filter(coluna.is_null())
            .all(db)
            .await
    }

    /// Marca o lembrete como enviado (idempotente por agendamento).
    pub async fn marcar_lembrete(
        db: &DatabaseConnection,
        id: Uuid,
        tipo: Lembrete,
    ) -> Result<(), sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        let mut active = appointments::ActiveModel {
            updated_at: Set(now),
            ..Default::default()
        };
        match tipo {
            Lembrete::VinteQuatroHoras => active.reminded_24h_at = Set(Some(now)),
            Lembrete::UmaHora => active.reminded_1h_at = Set(Some(now)),
        }
        Appointment::update_many()
            .set(active)
            .filter(appointments::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(())
    }

    /// Confirma o pagamento do sinal de forma atômica: `hold` → `confirmed`
    /// somente se o hold ainda não venceu. Retorna `false` quando a transição
    /// não se aplica (já confirmado, expirado ou cancelado).
    pub async fn confirm_payment(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<bool, sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        let res = Appointment::update_many()
            .set(appointments::ActiveModel {
                status: Set(AppointmentStatus::Confirmed),
                signal_paid_at: Set(Some(now)),
                updated_at: Set(now),
                ..Default::default()
            })
            .filter(appointments::Column::Id.eq(id))
            .filter(appointments::Column::Status.eq(AppointmentStatus::Hold))
            .filter(appointments::Column::HoldExpiresAt.gt(now))
            .exec(db)
            .await?;
        Ok(res.rows_affected == 1)
    }
}
