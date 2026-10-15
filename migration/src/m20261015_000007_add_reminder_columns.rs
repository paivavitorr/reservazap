use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Appointments::Table)
                    .add_column(
                        ColumnDef::new(Appointments::Reminded24hAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Appointments::Table)
                    .add_column(
                        ColumnDef::new(Appointments::Reminded1hAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Appointments::Table)
                    .drop_column(Appointments::Reminded24hAt)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Appointments::Table)
                    .drop_column(Appointments::Reminded1hAt)
                    .to_owned(),
            )
            .await
    }
}

enum Appointments {
    Table,
    Reminded24hAt,
    Reminded1hAt,
}

impl Iden for Appointments {
    fn unquoted(&self) -> &str {
        match self {
            Self::Table => "appointments",
            Self::Reminded24hAt => "reminded_24h_at",
            Self::Reminded1hAt => "reminded_1h_at",
        }
    }
}
