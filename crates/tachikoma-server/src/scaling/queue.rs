//! Request queue management.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Request queue for managing pending requests.
pub struct RequestQueue<T> {
    sender: mpsc::Sender<QueuedRequest<T>>,
    receiver: mpsc::Receiver<QueuedRequest<T>>,
    pending: AtomicUsize,
    max_size: usize,
    timeout: Duration,
}

struct QueuedRequest<T> {
    request: T,
    queued_at: Instant,
}

impl<T: Send + 'static> RequestQueue<T> {
    pub fn new(max_size: usize, timeout: Duration) -> Self {
        let (sender, receiver) = mpsc::channel(max_size);
        Self {
            sender,
            receiver,
            pending: AtomicUsize::new(0),
            max_size,
            timeout,
        }
    }

    /// Enqueue a request.
    pub async fn enqueue(&self, request: T) -> Result<(), QueueError> {
        let pending = self.pending.load(Ordering::SeqCst);
        if pending >= self.max_size {
            warn!(pending = pending, max = self.max_size, "Request queue full");
            return Err(QueueError::Full);
        }

        let queued = QueuedRequest {
            request,
            queued_at: Instant::now(),
        };

        match self.sender.send(queued).await {
            Ok(()) => {
                self.pending.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            Err(_) => Err(QueueError::Closed),
        }
    }

    /// Dequeue a request.
    pub async fn dequeue(&mut self) -> Option<T> {
        loop {
            match self.receiver.recv().await {
                Some(queued) => {
                    self.pending.fetch_sub(1, Ordering::SeqCst);

                    // Check if request has timed out
                    if queued.queued_at.elapsed() > self.timeout {
                        debug!("Dropping timed out request from queue");
                        continue;
                    }

                    return Some(queued.request);
                }
                None => return None,
            }
        }
    }

    /// Get pending count.
    pub fn pending(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    /// Check if queue is full.
    pub fn is_full(&self) -> bool {
        self.pending.load(Ordering::SeqCst) >= self.max_size
    }
}

/// Queue errors.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Queue is full")]
    Full,
    #[error("Queue is closed")]
    Closed,
}