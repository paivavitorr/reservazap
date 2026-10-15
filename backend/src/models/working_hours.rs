use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use super::_entities::working_hours::{self, Entity as WorkingHours};

pub struct WorkingHoursModel;

impl WorkingHoursModel {
    /// Todas as faixas do tenant que se aplicam a um profissional
    /// (específicas dele + gerais). O merge por weekday é feito no service.
    pub async fn find_do_profissional(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        professional_id: Uuid,
    ) -> Result<Vec<working_hours::Model>, sea_orm::DbErr> {
        WorkingHours::find()
            .filter(working_hours::Column::TenantId.eq(tenant_id))
            .filter(
                Condition::any()
                    .add(working_hours::Column::ProfessionalId.is_null())
                    .add(working_hours::Column::ProfessionalId.eq(professional_id)),
            )
            .order_by_asc(working_hours::Column::Weekday)
            .order_by_asc(working_hours::Column::StartTime)
            .all(db)
            .await
    }
}
