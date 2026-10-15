use super::_entities::services::{self, Entity as Service};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

pub struct ServiceModel;

impl ServiceModel {
    pub async fn find_by_tenant(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
    ) -> Result<Vec<services::Model>, sea_orm::DbErr> {
        Service::find()
            .filter(services::Column::TenantId.eq(tenant_id))
            .filter(services::Column::IsActive.eq(true))
            .order_by_asc(services::Column::Name)
            .all(db)
            .await
    }
}
