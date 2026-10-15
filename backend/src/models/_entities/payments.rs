use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(24))")]
pub enum PaymentStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "paid")]
    Paid,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "expired")]
    Expired,
    /// Pagamento confirmado depois do vencimento do hold (reconciliação manual).
    #[sea_orm(string_value = "late_paid")]
    LatePaid,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Paid => "paid",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::LatePaid => "late_paid",
        }
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub provider: String,
    pub txid: String,
    pub amount_in_cents: i64,
    pub status: PaymentStatus,
    pub paid_at: Option<DateTimeWithTimeZone>,
    pub raw_payload: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::appointments::Entity",
        from = "Column::AppointmentId",
        to = "super::appointments::Column::Id",
        on_delete = "Cascade"
    )]
    Appointment,
}

impl Related<super::appointments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Appointment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
