use aetheris_core::interface::cli;
use anyhow::Result;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    cli::run().await
}
