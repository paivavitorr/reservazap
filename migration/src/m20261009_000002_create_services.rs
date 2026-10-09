use sea_orm_migration::prelude::*;

use super::m20261009_000001_create_tenants::Tenants;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Services::Table)
                    .col(ColumnDef::new(Services::Id).uuid().not_null().primary_key().default(Expr::cust("gen_random_uuid()")))
                    .col(ColumnDef::new(Services::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Services::Name).string_len(120).not_null())
                    .col(ColumnDef::new(Services::Description).text())
                    .col(ColumnDef::new(Services::DurationMinutes).integer().not_null())
                    .col(ColumnDef::new(Services::PriceInCents).big_integer().not_null())
                    .col(ColumnDef::new(Services::SignalPercent).small_integer().not_null().default(30))
                    .col(ColumnDef::new(Services::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Services::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("now()")))
                    .col(ColumnDef::new(Services::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("now()")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Services::Table, Services::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_services_tenant_active")
                    .table(Services::Table)
                    .col(Services::TenantId)
                    .and_where(Expr::col(Services::IsActive).eq(true))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Services::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Services {
    Table,
    Id,
    TenantId,
    Name,
    Description,
    DurationMinutes,
    PriceInCents,
    SignalPercent,
    IsActive,
    CreatedAt,
    UpdatedAt,
}