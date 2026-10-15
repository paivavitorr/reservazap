use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use super::_entities::payments::{self, Entity as Payment, PaymentStatus};

pub struct PaymentModel;

impl PaymentModel {
    /// Registra (ou atualiza) o pagamento de uma cobrança Pix pelo `txid`.
    ///
    /// Idempotente: se o `txid` já existe, apenas transiciona o status quando o
    /// registro ainda está `pending` (pending → paid/late_paid/cancelled).
    #[allow(clippy::too_many_arguments)]
    pub async fn registrar(
        db: &DatabaseConnection,
        appointment_id: Uuid,
        provider: &str,
        txid: &str,
        amount_in_cents: i64,
        status: PaymentStatus,
        paid_at: Option<DateTime<Utc>>,
        raw_payload: Option<String>,
    ) -> Result<payments::Model, sea_orm::DbErr> {
        if let Some(existente) = Payment::find()
            .filter(payments::Column::Txid.eq(txid))
            .one(db)
            .await?
        {
            if existente.status == PaymentStatus::Pending && status != PaymentStatus::Pending {
                Self::atualizar_status(db, txid, status.clone(), paid_at, raw_payload).await?;
                let atualizado = Payment::find()
                    .filter(payments::Column::Txid.eq(txid))
                    .one(db)
                    .await?;
                return Ok(atualizado.unwrap_or(existente));
            }
            return Ok(existente);
        }

        let now = Utc::now().fixed_offset();
        let active = payments::ActiveModel {
            id: Set(Uuid::new_v4()),
            appointment_id: Set(appointment_id),
            provider: Set(provider.to_string()),
            txid: Set(txid.to_string()),
            amount_in_cents: Set(amount_in_cents),
            status: Set(status),
            paid_at: Set(paid_at.map(|dt| dt.fixed_offset())),
            raw_payload: Set(raw_payload),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active.insert(db).await
    }

    /// Transição de status de um pagamento registrado (reconciliação).
    /// Só aplica sobre registros ainda `pending` — pagamentos confirmados ou
    /// já reconciliados não são sobrescritos.
    pub async fn atualizar_status(
        db: &DatabaseConnection,
        txid: &str,
        status: PaymentStatus,
        paid_at: Option<DateTime<Utc>>,
        raw_payload: Option<String>,
    ) -> Result<(), sea_orm::DbErr> {
        let now = Utc::now().fixed_offset();
        let mut active = payments::ActiveModel {
            status: Set(status),
            updated_at: Set(now),
            ..Default::default()
        };
        if let Some(dt) = paid_at {
            active.paid_at = Set(Some(dt.fixed_offset()));
        }
        if let Some(payload) = raw_payload {
            active.raw_payload = Set(Some(payload));
        }
        Payment::update_many()
            .set(active)
            .filter(payments::Column::Txid.eq(txid))
            .filter(payments::Column::Status.eq(PaymentStatus::Pending))
            .exec(db)
            .await?;
        Ok(())
    }
}
