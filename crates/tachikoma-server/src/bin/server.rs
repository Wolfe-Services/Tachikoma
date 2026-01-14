//! Tachikoma Server Binary

use anyhow::Result;
use tachikoma_server::{Server, ServerConfig};
use tracing::info;

#[cfg(feature = "tracing")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    #[cfg(feature = "tracing")]
    {
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Load configuration
    dotenvy::dotenv().ok();
    let config = ServerConfig::from_env()?;

    info!(
        "Starting Tachikoma Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create and run server
    let server = Server::new(config).await?;
    server.run().await?;

    info!("Server shutdown complete");
    Ok(())
}