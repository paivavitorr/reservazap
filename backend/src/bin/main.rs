use migration::Migrator;
use reservazap::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    loco_rs::cli::main::<App, Migrator>().await
}