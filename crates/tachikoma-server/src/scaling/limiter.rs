//! Connection limiting.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

/// Connection limiter for controlling concurrent connections.
pub struct ConnectionLimiter {
    /// Semaphore for limiting connections.
    semaphore: Arc<Semaphore>,
    /// Current connection count.
    current: AtomicUsize,
    /// Maximum connections.
    max: usize,
}

impl ConnectionLimiter {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            current: AtomicUsize::new(0),
            max: max_connections,
        }
    }

    /// Try to acquire a connection slot.
    pub fn try_acquire(&self) -> Option<ConnectionGuard> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let count = self.current.fetch_add(1, Ordering::SeqCst) + 1;
                debug!(connections = count, "Connection acquired");
                Some(ConnectionGuard {
                    _permit: permit,
                    current: &self.current,
                })
            }
            Err(_) => {
                warn!(
                    max = self.max,
                    "Connection limit reached, rejecting connection"
                );
                None
            }
        }
    }

    /// Acquire a connection slot (blocking).
    pub async fn acquire(&self) -> ConnectionGuard {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        let count = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        debug!(connections = count, "Connection acquired");
        ConnectionGuard {
            _permit: permit,
            current: &self.current,
        }
    }

    /// Get current connection count.
    pub fn current(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }

    /// Get available slots.
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum connections.
    pub fn max(&self) -> usize {
        self.max
    }
}

/// Guard that releases connection slot when dropped.
pub struct ConnectionGuard<'a> {
    _permit: tokio::sync::OwnedSemaphorePermit,
    current: &'a AtomicUsize,
}

impl Drop for ConnectionGuard<'_> {
    fn drop(&mut self) {
        let count = self.current.fetch_sub(1, Ordering::SeqCst) - 1;
        debug!(connections = count, "Connection released");
    }
}