use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(24))")]
pub enum AppointmentStatus {
    #[sea_orm(string_value = "hold")]
    Hold,
    #[sea_orm(string_value = "confirmed")]
    Confirmed,
    #[sea_orm(string_value = "completed")]
    Completed,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "no_show")]
    NoShow,
    #[sea_orm(string_value = "expired")]
    Expired,
}

impl AppointmentStatus {
    pub fn is_hold(&self) -> bool {
        matches!(self, Self::Hold)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Hold | Self::Confirmed)
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Hold, Self::Confirmed)
                | (Self::Hold, Self::Expired)
                | (Self::Hold, Self::Cancelled)
                | (Self::Confirmed, Self::Completed)
                | (Self::Confirmed, Self::Cancelled)
                | (Self::Confirmed, Self::NoShow)
        )
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "appointments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub service_id: Uuid,
    pub professional_id: Uuid,
    pub starts_at: DateTimeWithTimeZone,
    pub ends_at: DateTimeWithTimeZone,
    pub client_name: String,
    pub client_phone: String,
    pub client_email: Option<String>,
    pub status: AppointmentStatus,
    pub signal_amount_in_cents: i64,
    pub signal_paid_at: Option<DateTimeWithTimeZone>,
    pub pix_txid: Option<String>,
    pub pix_payload: Option<String>,
    pub hold_expires_at: DateTimeWithTimeZone,
    pub cancelled_at: Option<DateTimeWithTimeZone>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenants::Entity",
        from = "Column::TenantId",
        to = "super::tenants::Column::Id",
        on_delete = "Cascade"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::services::Entity",
        from = "Column::ServiceId",
        to = "super::services::Column::Id"
    )]
    Service,
    #[sea_orm(
        belongs_to = "super::professionals::Entity",
        from = "Column::ProfessionalId",
        to = "super::professionals::Column::Id"
    )]
    Professional,
}

impl Related<super::tenants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::services::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Service.def()
    }
}

impl Related<super::professionals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Professional.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn is_hold_active(&self) -> bool {
        self.status == AppointmentStatus::Hold
            && self.hold_expires_at > chrono::Utc::now().fixed_offset()
    }
}