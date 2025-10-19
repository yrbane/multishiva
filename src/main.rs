use anyhow::Result;
use tracing_subscriber;

pub mod cli;
pub mod core;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("🕉️  MultiShiva starting...");

    // Parse CLI arguments
    let _args = cli::parse_args();

    tracing::info!("MultiShiva initialized successfully");

    Ok(())
}
