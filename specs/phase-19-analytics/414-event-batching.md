# 414 - Event Batching

## Overview

Client-side and server-side event batching for efficient network utilization and throughput optimization.

## Rust Implementation

```rust
// crates/analytics/src/batching.rs

use crate::event_types::AnalyticsEvent;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::interval;

/// Configuration for event batching
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum events per batch
    pub max_batch_size: usize,
    /// Maximum time to wait before flushing
    pub max_batch_age: Duration,
    /// Maximum bytes per batch (approximate)
    pub max_batch_bytes: usize,
    /// Number of concurrent flush operations
    pub flush_concurrency: usize,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_age: Duration::from_secs(5),
            max_batch_bytes: 1024 * 1024, // 1MB
            flush_concurrency: 4,
            retry_config: RetryConfig::default(),
        }
    }
}

/// Retry configuration for failed batches
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Batch of events ready for processing
#[derive(Debug)]
pub struct EventBatch {
    pub events: Vec<AnalyticsEvent>,
    pub created_at: Instant,
    pub attempt: u32,
}

impl EventBatch {
    pub fn new(events: Vec<AnalyticsEvent>) -> Self {
        Self {
            events,
            created_at: Instant::now(),
            attempt: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.events.len()
    }

    pub fn approximate_bytes(&self) -> usize {
        // Rough estimate: serialize and measure
        self.events.iter()
            .map(|e| serde_json::to_string(e).map(|s| s.len()).unwrap_or(500))
            .sum()
    }
}

/// Event batcher that collects events and flushes them periodically
pub struct EventBatcher {
    config: BatchConfig,
    buffer: Arc<Mutex<Vec<AnalyticsEvent>>>,
    buffer_bytes: Arc<Mutex<usize>>,
    last_flush: Arc<Mutex<Instant>>,
    flush_notify: Arc<Notify>,
    output_tx: mpsc::Sender<EventBatch>,
}

impl EventBatcher {
    pub fn new(config: BatchConfig, output_tx: mpsc::Sender<EventBatch>) -> Self {
        Self {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_bytes: Arc::new(Mutex::new(0)),
            last_flush: Arc::new(Mutex::new(Instant::now())),
            flush_notify: Arc::new(Notify::new()),
            output_tx,
        }
    }

    /// Start the batcher background task
    pub fn start(self: Arc<Self>) {
        let batcher = self.clone();

        // Periodic flush task
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(100));

            loop {
                ticker.tick().await;

                let should_flush = {
                    let last_flush = batcher.last_flush.lock().await;
                    last_flush.elapsed() >= batcher.config.max_batch_age
                };

                if should_flush {
                    batcher.flush().await;
                }
            }
        });

        // Immediate flush listener
        let batcher2 = self.clone();
        tokio::spawn(async move {
            loop {
                batcher2.flush_notify.notified().await;
                batcher2.flush().await;
            }
        });
    }

    /// Add an event to the batch
    pub async fn add(&self, event: AnalyticsEvent) {
        let event_bytes = serde_json::to_string(&event)
            .map(|s| s.len())
            .unwrap_or(500);

        let should_flush = {
            let mut buffer = self.buffer.lock().await;
            let mut buffer_bytes = self.buffer_bytes.lock().await;

            buffer.push(event);
            *buffer_bytes += event_bytes;

            buffer.len() >= self.config.max_batch_size ||
            *buffer_bytes >= self.config.max_batch_bytes
        };

        if should_flush {
            self.flush_notify.notify_one();
        }
    }

    /// Add multiple events
    pub async fn add_many(&self, events: Vec<AnalyticsEvent>) {
        for event in events {
            self.add(event).await;
        }
    }

    /// Flush the current batch
    pub async fn flush(&self) {
        let events = {
            let mut buffer = self.buffer.lock().await;
            let mut buffer_bytes = self.buffer_bytes.lock().await;

            if buffer.is_empty() {
                return;
            }

            let events = std::mem::take(&mut *buffer);
            *buffer_bytes = 0;
            events
        };

        *self.last_flush.lock().await = Instant::now();

        let batch = EventBatch::new(events);

        // Send to output channel (non-blocking)
        let _ = self.output_tx.try_send(batch);
    }

    /// Get current buffer size
    pub async fn buffer_size(&self) -> usize {
        self.buffer.lock().await.len()
    }
}

/// Batch processor that handles persistence with retries
pub struct BatchProcessor {
    config: BatchConfig,
    handler: Arc<dyn BatchHandler>,
}

/// Handler trait for processing batches
#[async_trait::async_trait]
pub trait BatchHandler: Send + Sync {
    async fn process(&self, batch: &EventBatch) -> Result<(), BatchError>;
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rate limited")]
    RateLimited,
}

impl BatchProcessor {
    pub fn new(config: BatchConfig, handler: Arc<dyn BatchHandler>) -> Self {
        Self { config, handler }
    }

    /// Start processing batches from the input channel
    pub async fn run(&self, mut input_rx: mpsc::Receiver<EventBatch>) {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.flush_concurrency));

        while let Some(batch) = input_rx.recv().await {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let handler = self.handler.clone();
            let config = self.config.retry_config.clone();

            tokio::spawn(async move {
                let result = Self::process_with_retry(batch, handler, config).await;
                if let Err(e) = result {
                    tracing::error!("Failed to process batch after retries: {}", e);
                }
                drop(permit);
            });
        }
    }

    async fn process_with_retry(
        mut batch: EventBatch,
        handler: Arc<dyn BatchHandler>,
        config: RetryConfig,
    ) -> Result<(), BatchError> {
        let mut delay = config.initial_delay;

        loop {
            match handler.process(&batch).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    batch.attempt += 1;

                    if batch.attempt >= config.max_retries {
                        return Err(e);
                    }

                    // Check if retryable
                    match &e {
                        BatchError::Validation(_) => return Err(e), // Don't retry
                        BatchError::RateLimited => {
                            // Longer delay for rate limiting
                            delay = config.max_delay;
                        }
                        _ => {
                            // Exponential backoff
                            delay = Duration::from_secs_f64(
                                (delay.as_secs_f64() * config.backoff_multiplier)
                                    .min(config.max_delay.as_secs_f64())
                            );
                        }
                    }

                    tracing::warn!(
                        "Batch processing failed (attempt {}): {}. Retrying in {:?}",
                        batch.attempt,
                        e,
                        delay
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

/// Client-side batching implementation
pub mod client {
    use super::*;
    use std::collections::VecDeque;

    /// Client-side event queue with persistence
    pub struct ClientEventQueue {
        queue: Mutex<VecDeque<AnalyticsEvent>>,
        max_queue_size: usize,
        persistence: Option<Arc<dyn QueuePersistence>>,
    }

    #[async_trait::async_trait]
    pub trait QueuePersistence: Send + Sync {
        async fn save(&self, events: &[AnalyticsEvent]) -> Result<(), String>;
        async fn load(&self) -> Result<Vec<AnalyticsEvent>, String>;
        async fn clear(&self) -> Result<(), String>;
    }

    impl ClientEventQueue {
        pub fn new(max_size: usize, persistence: Option<Arc<dyn QueuePersistence>>) -> Self {
            Self {
                queue: Mutex::new(VecDeque::new()),
                max_queue_size: max_size,
                persistence,
            }
        }

        /// Add event to queue
        pub async fn enqueue(&self, event: AnalyticsEvent) {
            let mut queue = self.queue.lock().await;

            // Drop oldest events if queue is full
            while queue.len() >= self.max_queue_size {
                queue.pop_front();
            }

            queue.push_back(event);
        }

        /// Get events for sending (up to limit)
        pub async fn dequeue(&self, limit: usize) -> Vec<AnalyticsEvent> {
            let mut queue = self.queue.lock().await;
            let count = limit.min(queue.len());

            queue.drain(..count).collect()
        }

        /// Return failed events to the front of the queue
        pub async fn requeue(&self, events: Vec<AnalyticsEvent>) {
            let mut queue = self.queue.lock().await;

            for event in events.into_iter().rev() {
                queue.push_front(event);
            }
        }

        /// Persist queue to storage
        pub async fn persist(&self) -> Result<(), String> {
            if let Some(persistence) = &self.persistence {
                let queue = self.queue.lock().await;
                let events: Vec<_> = queue.iter().cloned().collect();
                persistence.save(&events).await
            } else {
                Ok(())
            }
        }

        /// Load queue from storage
        pub async fn restore(&self) -> Result<(), String> {
            if let Some(persistence) = &self.persistence {
                let events = persistence.load().await?;
                let mut queue = self.queue.lock().await;

                for event in events {
                    if queue.len() < self.max_queue_size {
                        queue.push_back(event);
                    }
                }
            }
            Ok(())
        }

        /// Get queue size
        pub async fn size(&self) -> usize {
            self.queue.lock().await.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler {
        should_fail: std::sync::atomic::AtomicBool,
    }

    #[async_trait::async_trait]
    impl BatchHandler for MockHandler {
        async fn process(&self, _batch: &EventBatch) -> Result<(), BatchError> {
            if self.should_fail.load(std::sync::atomic::Ordering::SeqCst) {
                Err(BatchError::Network("test error".to_string()))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_batcher_flush_on_size() {
        let (tx, mut rx) = mpsc::channel(10);

        let batcher = Arc::new(EventBatcher::new(
            BatchConfig {
                max_batch_size: 5,
                ..Default::default()
            },
            tx,
        ));

        batcher.clone().start();

        // Add 5 events
        for i in 0..5 {
            let event = AnalyticsEvent::new(
                &format!("event_{}", i),
                "user-123",
                crate::event_types::EventCategory::Custom,
            );
            batcher.add(event).await;
        }

        // Should receive a batch
        let batch = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(batch.size(), 5);
    }

    #[tokio::test]
    async fn test_client_queue() {
        let queue = client::ClientEventQueue::new(100, None);

        // Enqueue events
        for i in 0..10 {
            let event = AnalyticsEvent::new(
                &format!("event_{}", i),
                "user-123",
                crate::event_types::EventCategory::Custom,
            );
            queue.enqueue(event).await;
        }

        assert_eq!(queue.size().await, 10);

        // Dequeue some
        let events = queue.dequeue(5).await;
        assert_eq!(events.len(), 5);
        assert_eq!(queue.size().await, 5);

        // Requeue failed
        queue.requeue(events).await;
        assert_eq!(queue.size().await, 10);
    }
}
```

## Batching Strategies

| Trigger | Description |
|---------|-------------|
| Size | Flush when batch reaches max events |
| Time | Flush after max age elapsed |
| Bytes | Flush when batch exceeds size limit |
| Manual | Explicit flush call (page unload, etc.) |

## Related Specs

- 413-event-capture.md - Event capture API
- 415-event-persistence.md - Storage
- 403-flag-sdk-ts.md - Client SDK batching
