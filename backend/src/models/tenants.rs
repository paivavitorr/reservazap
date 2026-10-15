use super::_entities::tenants::{self, Entity as Tenant};
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct TenantModel;

impl TenantModel {
    pub async fn find_by_id(
        db: &sea_orm::DatabaseConnection,
        id: uuid::Uuid,
    ) -> Result<Option<tenants::Model>, sea_orm::DbErr> {
        Tenant::find_by_id(id).one(db).await
    }

    pub async fn find_by_slug(
        db: &DatabaseConnection,
        slug: &str,
    ) -> Result<Option<tenants::Model>, sea_orm::DbErr> {
        Tenant::find_by_slug(slug).one(db).await
    }
}
