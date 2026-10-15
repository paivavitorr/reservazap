use super::_entities::professionals::{self, Entity as Professional};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

pub struct ProfessionalModel;

impl ProfessionalModel {
    pub async fn find_by_id_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<professionals::Model>, sea_orm::DbErr> {
        Professional::find_by_id(id)
            .filter(professionals::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
    }

    pub async fn find_by_tenant(
        db: &DatabaseConnection,
        tenant_id: uuid::Uuid,
    ) -> Result<Vec<professionals::Model>, sea_orm::DbErr> {
        Professional::find()
            .filter(professionals::Column::TenantId.eq(tenant_id))
            .filter(professionals::Column::IsActive.eq(true))
            .order_by_asc(professionals::Column::Name)
            .all(db)
            .await
    }
}
