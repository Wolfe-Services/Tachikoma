//! Batch event collection for efficient persistence.

use crate::CapturedEvent;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::debug;

/// Configuration for batch collection.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum events per batch.
    pub max_batch_size: usize,
    /// Maximum time to wait before flushing.
    pub max_batch_age: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_age: Duration::from_secs(1),
        }
    }
}

/// Collected batch of events.
#[derive(Debug)]
pub struct EventBatch {
    pub events: Vec<CapturedEvent>,
    pub collected_at: Instant,
}

impl EventBatch {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            collected_at: Instant::now(),
        }
    }

    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    fn len(&self) -> usize {
        self.events.len()
    }
}

/// Batch collector that aggregates events.
pub struct BatchCollector {
    config: BatchConfig,
    current_batch: EventBatch,
}

impl BatchCollector {
    /// Create a new collector.
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            current_batch: EventBatch::new(),
        }
    }

    /// Add an event to the current batch.
    /// Returns Some(batch) if the batch is ready to flush.
    pub fn add(&mut self, event: CapturedEvent) -> Option<EventBatch> {
        self.current_batch.events.push(event);

        if self.should_flush() {
            Some(self.take_batch())
        } else {
            None
        }
    }

    /// Check if current batch should be flushed.
    pub fn should_flush(&self) -> bool {
        self.current_batch.len() >= self.config.max_batch_size
            || self.current_batch.collected_at.elapsed() >= self.config.max_batch_age
    }

    /// Check if batch is due based on age alone.
    pub fn is_due(&self) -> bool {
        !self.current_batch.is_empty()
            && self.current_batch.collected_at.elapsed() >= self.config.max_batch_age
    }

    /// Take the current batch and reset.
    pub fn take_batch(&mut self) -> EventBatch {
        std::mem::replace(&mut self.current_batch, EventBatch::new())
    }

    /// Check if there are pending events.
    pub fn has_pending(&self) -> bool {
        !self.current_batch.is_empty()
    }
}

/// Async batch processing loop.
pub async fn batch_processing_loop(
    mut receiver: mpsc::Receiver<CapturedEvent>,
    mut batch_sender: mpsc::Sender<EventBatch>,
    config: BatchConfig,
) {
    let mut collector = BatchCollector::new(config.clone());
    let mut interval = tokio::time::interval(config.max_batch_age / 2);

    loop {
        tokio::select! {
            Some(event) = receiver.recv() => {
                if let Some(batch) = collector.add(event) {
                    debug!("Flushing batch of {} events (size limit)", batch.len());
                    if batch_sender.send(batch).await.is_err() {
                        break;
                    }
                }
            }
            _ = interval.tick() => {
                if collector.is_due() {
                    let batch = collector.take_batch();
                    debug!("Flushing batch of {} events (time limit)", batch.len());
                    if batch_sender.send(batch).await.is_err() {
                        break;
                    }
                }
            }
            else => break,
        }
    }

    // Flush remaining events
    if collector.has_pending() {
        let batch = collector.take_batch();
        let _ = batch_sender.send(batch).await;
    }
}