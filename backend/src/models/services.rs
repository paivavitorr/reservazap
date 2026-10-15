use super::_entities::services::{self, Entity as Service};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

pub struct ServiceModel;

impl ServiceModel {
    pub async fn find_by_id_for_tenant(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
        id: uuid::Uuid,
    ) -> Result<Option<services::Model>, sea_orm::DbErr> {
        Service::find_by_id(id)
            .filter(services::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    /// Inclui serviços desativados (uso no painel administrativo).
    pub async fn find_by_tenant_all(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
    ) -> Result<Vec<services::Model>, sea_orm::DbErr> {
        Service::find()
            .filter(services::Column::TenantId.eq(tenant_id))
            .order_by_asc(services::Column::Name)
            .all(db)
            .await
    }

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
