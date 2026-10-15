use sea_orm_migration::prelude::*;

use super::m20261009_000001_create_tenants::Tenants;
use super::m20261009_000002_create_services::Services;
use super::m20261009_000003_create_professionals::Professionals;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // extensão necessária para EXCLUDE constraint com tstzrange
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS btree_gist")
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Appointments::Table)
                    .col(
                        ColumnDef::new(Appointments::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Appointments::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Appointments::ServiceId).uuid().not_null())
                    .col(
                        ColumnDef::new(Appointments::ProfessionalId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Appointments::StartsAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Appointments::EndsAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Appointments::ClientName)
                            .string_len(120)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Appointments::ClientPhone)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Appointments::ClientEmail).string_len(160))
                    .col(
                        ColumnDef::new(Appointments::Status)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Appointments::SignalAmountInCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Appointments::SignalPaidAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Appointments::PixTxid)
                            .string_len(64)
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Appointments::PixPayload).text())
                    .col(
                        ColumnDef::new(Appointments::HoldExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Appointments::CancelledAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Appointments::CancellationReason).text())
                    .col(
                        ColumnDef::new(Appointments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Appointments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Appointments::Table, Appointments::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Appointments::Table, Appointments::ServiceId)
                            .to(Services::Table, Services::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Appointments::Table, Appointments::ProfessionalId)
                            .to(Professionals::Table, Professionals::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // constraint anti-overlap: impede double-booking do mesmo profissional no mesmo horário
        // aplica-se apenas a status ativos (hold/confirmed)
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE appointments
                   ADD CONSTRAINT no_overlap EXCLUDE USING gist (
                       professional_id WITH =,
                       tstzrange(starts_at, ends_at, '[)') WITH &&
                   ) WHERE (status IN ('hold', 'confirmed'))"#,
            )
            .await?;

        // índice para a task de expiração (sweeper)
        manager
            .create_index(
                Index::create()
                    .name("idx_appointments_hold_expiry")
                    .table(Appointments::Table)
                    .col(Appointments::HoldExpiresAt)
                    .and_where(Expr::col(Appointments::Status).eq("hold"))
                    .to_owned(),
            )
            .await?;

        // índice para queries de disponibilidade
        manager
            .create_index(
                Index::create()
                    .name("idx_appointments_professional_time")
                    .table(Appointments::Table)
                    .col(Appointments::ProfessionalId)
                    .col(Appointments::StartsAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Appointments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Appointments {
    Table,
    Id,
    TenantId,
    ServiceId,
    ProfessionalId,
    StartsAt,
    EndsAt,
    ClientName,
    ClientPhone,
    ClientEmail,
    Status,
    SignalAmountInCents,
    SignalPaidAt,
    PixTxid,
    PixPayload,
    HoldExpiresAt,
    CancelledAt,
    CancellationReason,
    CreatedAt,
    UpdatedAt,
}
