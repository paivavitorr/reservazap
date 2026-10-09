use loco_rs::cli;
use reservazap::app::App;

#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    cli::run_main::<App>().await
}