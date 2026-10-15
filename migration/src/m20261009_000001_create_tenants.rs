use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tenants::Table)
                    .col(
                        ColumnDef::new(Tenants::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(Tenants::Slug)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Tenants::Name).string_len(120).not_null())
                    .col(ColumnDef::new(Tenants::Phone).string_len(20).not_null())
                    .col(
                        ColumnDef::new(Tenants::Timezone)
                            .string_len(64)
                            .not_null()
                            .default("America/Sao_Paulo"),
                    )
                    .col(ColumnDef::new(Tenants::PixKey).string_len(255))
                    .col(ColumnDef::new(Tenants::PixKeyType).string_len(16))
                    .col(
                        ColumnDef::new(Tenants::BookingHoldMinutes)
                            .integer()
                            .not_null()
                            .default(10),
                    )
                    .col(
                        ColumnDef::new(Tenants::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Tenants::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Tenants::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tenants::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tenants {
    Table,
    Id,
    Slug,
    Name,
    Phone,
    Timezone,
    PixKey,
    PixKeyType,
    BookingHoldMinutes,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
