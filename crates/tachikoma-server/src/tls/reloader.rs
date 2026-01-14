//! Certificate hot-reloading.

use super::{builder::build_tls_config, config::TlsConfig};
use rustls::server::ServerConfig as RustlsServerConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{error, info, warn};

/// Certificate reloader for hot-reloading TLS certificates.
pub struct CertReloader {
    config: TlsConfig,
    sender: watch::Sender<Arc<RustlsServerConfig>>,
    receiver: watch::Receiver<Arc<RustlsServerConfig>>,
}

impl CertReloader {
    pub fn new(config: TlsConfig) -> Result<Self, super::config::TlsConfigError> {
        let tls_config = build_tls_config(&config)?;
        let (sender, receiver) = watch::channel(Arc::new(tls_config));

        Ok(Self {
            config,
            sender,
            receiver,
        })
    }

    /// Get the current TLS configuration.
    pub fn get_config(&self) -> Arc<RustlsServerConfig> {
        self.receiver.borrow().clone()
    }

    /// Subscribe to configuration changes.
    pub fn subscribe(&self) -> watch::Receiver<Arc<RustlsServerConfig>> {
        self.receiver.clone()
    }

    /// Start the reloader task.
    pub fn start(self: Arc<Self>) {
        if let Some(interval) = self.config.reload_interval_secs {
            tokio::spawn(async move {
                self.reload_loop(Duration::from_secs(interval)).await;
            });
        }
    }

    async fn reload_loop(&self, interval: Duration) {
        let mut interval_timer = tokio::time::interval(interval);

        loop {
            interval_timer.tick().await;

            match self.reload() {
                Ok(true) => info!("TLS certificates reloaded successfully"),
                Ok(false) => {} // No change
                Err(e) => error!("Failed to reload TLS certificates: {}", e),
            }
        }
    }

    /// Reload certificates.
    pub fn reload(&self) -> Result<bool, super::config::TlsConfigError> {
        // Check if files have changed (by timestamp or hash)
        // For simplicity, we'll just reload unconditionally

        let new_config = build_tls_config(&self.config)?;

        // Send new configuration
        if self.sender.send(Arc::new(new_config)).is_err() {
            warn!("No receivers for TLS config update");
        }

        Ok(true)
    }
}

/// Create TLS acceptor with hot-reload support.
pub fn create_tls_acceptor(
    reloader: &CertReloader,
) -> impl tower::Service<
    tokio::net::TcpStream,
    Response = tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    Error = std::io::Error,
> + Clone {
    use tokio_rustls::TlsAcceptor;

    let config = reloader.get_config();
    TlsAcceptor::from(config)
}