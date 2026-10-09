use sea_orm_migration::prelude::*;

mod m20261009_000001_create_tenants;
mod m20261009_000002_create_services;
mod m20261009_000003_create_professionals;
mod m20261009_000004_create_appointments;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261009_000001_create_tenants::Migration),
            Box::new(m20261009_000002_create_services::Migration),
            Box::new(m20261009_000003_create_professionals::Migration),
            Box::new(m20261009_000004_create_appointments::Migration),
        ]
    }
}