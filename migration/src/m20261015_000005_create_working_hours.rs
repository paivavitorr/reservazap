use sea_orm_migration::prelude::*;

use super::m20261009_000001_create_tenants::Tenants;
use super::m20261009_000003_create_professionals::Professionals;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WorkingHours::Table)
                    .col(
                        ColumnDef::new(WorkingHours::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(WorkingHours::TenantId).uuid().not_null())
                    .col(ColumnDef::new(WorkingHours::ProfessionalId).uuid().null())
                    .col(
                        ColumnDef::new(WorkingHours::Weekday)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkingHours::StartTime).time().not_null())
                    .col(ColumnDef::new(WorkingHours::EndTime).time().not_null())
                    .col(
                        ColumnDef::new(WorkingHours::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(WorkingHours::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WorkingHours::Table, WorkingHours::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(WorkingHours::Table, WorkingHours::ProfessionalId)
                            .to(Professionals::Table, Professionals::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE working_hours
                   ADD CONSTRAINT working_hours_range_check CHECK (end_time > start_time)"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_working_hours_tenant_weekday")
                    .table(WorkingHours::Table)
                    .col(WorkingHours::TenantId)
                    .col(WorkingHours::Weekday)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_working_hours_professional")
                    .table(WorkingHours::Table)
                    .col(WorkingHours::ProfessionalId)
                    .col(WorkingHours::Weekday)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkingHours::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum WorkingHours {
    Table,
    Id,
    TenantId,
    ProfessionalId,
    Weekday,
    StartTime,
    EndTime,
    CreatedAt,
    UpdatedAt,
}
