//! Shutdown coordination.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use tracing::{info, warn};

/// Shutdown coordinator for graceful shutdown.
#[derive(Clone)]
pub struct ShutdownCoordinator {
    /// Shutdown signal sender.
    sender: broadcast::Sender<()>,
    /// Shutdown initiated flag.
    initiated: Arc<AtomicBool>,
    /// Shutdown complete notifier.
    complete_tx: Arc<watch::Sender<bool>>,
    /// Shutdown complete receiver.
    complete_rx: watch::Receiver<bool>,
    /// Shutdown timeout.
    timeout: Duration,
}

impl ShutdownCoordinator {
    pub fn new(timeout: Duration) -> Self {
        let (sender, _) = broadcast::channel(1);
        let (complete_tx, complete_rx) = watch::channel(false);

        Self {
            sender,
            initiated: Arc::new(AtomicBool::new(false)),
            complete_tx: Arc::new(complete_tx),
            complete_rx,
            timeout,
        }
    }

    /// Subscribe to shutdown signal.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.sender.subscribe()
    }

    /// Check if shutdown has been initiated.
    pub fn is_shutting_down(&self) -> bool {
        self.initiated.load(Ordering::SeqCst)
    }

    /// Initiate shutdown.
    pub fn initiate(&self) {
        if self.initiated.swap(true, Ordering::SeqCst) {
            // Already initiated
            return;
        }

        info!("Initiating graceful shutdown...");
        let _ = self.sender.send(());
    }

    /// Wait for shutdown completion.
    pub async fn wait_for_completion(&self) {
        let mut rx = self.complete_rx.clone();
        let _ = rx.wait_for(|&complete| complete).await;
    }

    /// Mark shutdown as complete.
    pub fn complete(&self) {
        info!("Shutdown complete");
        let _ = self.complete_tx.send(true);
    }

    /// Get shutdown timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}