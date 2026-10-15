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
                    .table(Professionals::Table)
                    .col(
                        ColumnDef::new(Professionals::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Professionals::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(Professionals::Name)
                            .string_len(120)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Professionals::Email).string_len(160))
                    .col(ColumnDef::new(Professionals::Phone).string_len(20))
                    .col(
                        ColumnDef::new(Professionals::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Professionals::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Professionals::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Professionals::Table, Professionals::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Professionals::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Professionals {
    Table,
    Id,
    TenantId,
    Name,
    Email,
    Phone,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
