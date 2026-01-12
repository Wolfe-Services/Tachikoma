//! Cancellation support for bash commands.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

/// Cancellation token for bash commands.
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    notify: watch::Sender<bool>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> (Self, CancellationWatcher) {
        let (tx, rx) = watch::channel(false);
        let cancelled = Arc::new(AtomicBool::new(false));

        let token = Self {
            cancelled: cancelled.clone(),
            notify: tx,
        };

        let watcher = CancellationWatcher {
            cancelled,
            notify: rx,
        };

        (token, watcher)
    }

    /// Cancel the operation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let _ = self.notify.send(true);
    }

    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new().0
    }
}

/// Watches for cancellation.
#[derive(Clone)]
pub struct CancellationWatcher {
    cancelled: Arc<AtomicBool>,
    notify: watch::Receiver<bool>,
}

impl CancellationWatcher {
    /// Check if cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Wait for cancellation.
    pub async fn cancelled(&mut self) {
        while !*self.notify.borrow() {
            if self.notify.changed().await.is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cancellation_token() {
        let (token, watcher) = CancellationToken::new();

        assert!(!token.is_cancelled());
        assert!(!watcher.is_cancelled());

        token.cancel();

        assert!(token.is_cancelled());
        assert!(watcher.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_wait() {
        let (token, mut watcher) = CancellationToken::new();

        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            token.cancel();
        });

        watcher.cancelled().await;
        assert!(watcher.is_cancelled());

        handle.await.unwrap();
    }
}