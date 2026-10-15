use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "working_hours")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub professional_id: Option<Uuid>,
    #[sea_orm(column_type = "SmallInteger")]
    pub weekday: i16,
    pub start_time: Time,
    pub end_time: Time,
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
        belongs_to = "super::professionals::Entity",
        from = "Column::ProfessionalId",
        to = "super::professionals::Column::Id",
        on_delete = "Cascade"
    )]
    Professional,
}

impl Related<super::tenants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::professionals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Professional.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
