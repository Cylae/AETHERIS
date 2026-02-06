use anyhow::Result;
use log::LevelFilter;
use aetheris_core::interface::cli;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    cli::run().await
}
