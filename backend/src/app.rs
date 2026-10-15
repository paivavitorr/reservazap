use async_trait::async_trait;
use loco_rs::{
    Result,
    app::{AppContext, Hooks},
    bgworker::Queue,
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
};
use std::path::Path;

use crate::{controllers, tasks};
use migration::Migrator;

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        "reservazap"
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .prefix("api")
            .add_route(controllers::bookings::routes())
            .add_route(controllers::catalog::routes())
            .add_route(controllers::webhooks::routes())
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::expire_holds::ExpireHolds);
    }

    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    async fn seed(ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(crate::seed::seed_demo(&ctx.db).await?)
    }
}
