use anyhow::Result;
use multishiva::cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🕉️  MultiShiva starting...");

    // Parse CLI arguments
    let _args = cli::parse_args();

    tracing::info!("MultiShiva initialized successfully");

    Ok(())
}
