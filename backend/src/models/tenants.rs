use sea_orm::DatabaseConnection;
use super::_entities::tenants::{self, Entity as Tenant};

pub struct TenantModel;

impl TenantModel {
    pub async fn find_by_slug(db: &DatabaseConnection, slug: &str) -> Result<Option<tenants::Model>, sea_orm::DbErr> {
        Tenant::find_by_slug(slug).one(db).await
    }
}