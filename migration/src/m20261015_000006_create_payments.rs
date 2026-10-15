use sea_orm_migration::prelude::*;

use super::m20261009_000004_create_appointments::Appointments;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .col(
                        ColumnDef::new(Payments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Payments::AppointmentId).uuid().not_null())
                    .col(ColumnDef::new(Payments::Provider).string_len(32).not_null())
                    .col(
                        ColumnDef::new(Payments::Txid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Payments::AmountInCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Payments::Status).string_len(24).not_null())
                    .col(ColumnDef::new(Payments::PaidAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Payments::RawPayload).text())
                    .col(
                        ColumnDef::new(Payments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Payments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Payments::Table, Payments::AppointmentId)
                            .to(Appointments::Table, Appointments::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_appointment")
                    .table(Payments::Table)
                    .col(Payments::AppointmentId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Payments {
    Table,
    Id,
    AppointmentId,
    Provider,
    Txid,
    AmountInCents,
    Status,
    PaidAt,
    RawPayload,
    CreatedAt,
    UpdatedAt,
}
