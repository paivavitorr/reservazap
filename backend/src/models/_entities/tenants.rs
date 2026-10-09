use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub phone: String,
    pub timezone: String,
    pub pix_key: Option<String>,
    pub pix_key_type: Option<String>,
    pub booking_hold_minutes: i32,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::services::Entity")]
    Services,
    #[sea_orm(has_many = "super::professionals::Entity")]
    Professionals,
    #[sea_orm(has_many = "super::appointments::Entity")]
    Appointments,
}

impl Related<super::services::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Services.def()
    }
}

impl Related<super::professionals::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Professionals.def()
    }
}

impl Related<super::appointments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Appointments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_slug(slug: &str) -> Select<Self> {
        use sea_orm::QueryFilter;
        Self::find().filter(Column::Slug.eq(slug))
    }
}