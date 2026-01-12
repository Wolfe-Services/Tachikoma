# Spec 408: Analytics Event Collector

## Phase
19 - Analytics/Telemetry

## Spec ID
408

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 407: Analytics Configuration (config management)

## Estimated Context
~11%

---

## Objective

Implement the core event collection system that receives, buffers, samples, and dispatches analytics events throughout the Tachikoma application. The collector is the central hub for all analytics data flow.

---

## Acceptance Criteria

- [ ] Implement thread-safe event collector
- [ ] Create event buffering with configurable limits
- [ ] Implement sampling strategies
- [ ] Support event batching for efficiency
- [ ] Create flush mechanisms (time-based, size-based)
- [ ] Implement back-pressure handling
- [ ] Support multiple event sinks
- [ ] Create global collector instance management

---

## Implementation Details

### Event Collector

```rust
// src/analytics/collector.rs

use crate::analytics::config::{AnalyticsConfig, AnalyticsConfigManager, SamplingConfig};
use crate::analytics::types::{
    AnalyticsEvent, EventBatch, EventCategory, EventId, EventPriority, EventType,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc, RwLock};
use uuid::Uuid;

/// Trait for event sinks that receive collected events
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Process a batch of events
    async fn process(&self, batch: EventBatch) -> Result<(), SinkError>;

    /// Flush any pending data
    async fn flush(&self) -> Result<(), SinkError>;

    /// Get sink identifier
    fn name(&self) -> &str;
}

/// Errors from event sinks
#[derive(Debug, thiserror::Error)]
pub enum SinkError {
    #[error("Sink unavailable: {0}")]
    Unavailable(String),

    #[error("Write failed: {0}")]
    WriteFailed(String),

    #[error("Flush failed: {0}")]
    FlushFailed(String),
}

/// Statistics for the collector
#[derive(Debug, Default)]
pub struct CollectorStats {
    pub events_received: AtomicU64,
    pub events_sampled_out: AtomicU64,
    pub events_processed: AtomicU64,
    pub events_dropped: AtomicU64,
    pub batches_flushed: AtomicU64,
    pub flush_errors: AtomicU64,
}

impl CollectorStats {
    pub fn snapshot(&self) -> CollectorStatsSnapshot {
        CollectorStatsSnapshot {
            events_received: self.events_received.load(Ordering::Relaxed),
            events_sampled_out: self.events_sampled_out.load(Ordering::Relaxed),
            events_processed: self.events_processed.load(Ordering::Relaxed),
            events_dropped: self.events_dropped.load(Ordering::Relaxed),
            batches_flushed: self.batches_flushed.load(Ordering::Relaxed),
            flush_errors: self.flush_errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectorStatsSnapshot {
    pub events_received: u64,
    pub events_sampled_out: u64,
    pub events_processed: u64,
    pub events_dropped: u64,
    pub batches_flushed: u64,
    pub flush_errors: u64,
}

/// Sampling state tracker
struct SamplingState {
    /// Event counts per window
    window_counts: HashMap<String, WindowCounter>,
    /// Random number generator
    rng: rand::rngs::SmallRng,
}

struct WindowCounter {
    count: u32,
    window_start: Instant,
}

impl SamplingState {
    fn new() -> Self {
        use rand::SeedableRng;
        Self {
            window_counts: HashMap::new(),
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }

    fn should_sample(&mut self, event_type: &EventType, config: &SamplingConfig) -> bool {
        use rand::Rng;

        let key = format!("{:?}", event_type);
        let now = Instant::now();

        let counter = self.window_counts.entry(key).or_insert_with(|| WindowCounter {
            count: 0,
            window_start: now,
        });

        // Check if we need to reset the window
        let window_duration = Duration::from_secs(config.window_seconds);
        if now.duration_since(counter.window_start) >= window_duration {
            counter.count = 0;
            counter.window_start = now;
        }

        // Always sample if below minimum
        if counter.count < config.min_per_window {
            counter.count += 1;
            return true;
        }

        // Probabilistic sampling
        let should_sample = self.rng.gen::<f64>() < config.rate;
        if should_sample {
            counter.count += 1;
        }

        should_sample
    }
}

/// Main analytics event collector
pub struct EventCollector {
    /// Configuration manager
    config: Arc<RwLock<AnalyticsConfigManager>>,

    /// Event buffer
    buffer: Arc<RwLock<Vec<AnalyticsEvent>>>,

    /// Registered event sinks
    sinks: Arc<RwLock<Vec<Arc<dyn EventSink>>>>,

    /// Sampling state
    sampling: Arc<RwLock<SamplingState>>,

    /// Collector statistics
    stats: Arc<CollectorStats>,

    /// Current session ID
    session_id: Uuid,

    /// Sequence counter for batches
    batch_sequence: AtomicU64,

    /// Last flush time
    last_flush: Arc<RwLock<Instant>>,

    /// Shutdown flag
    shutdown: Arc<AtomicBool>,

    /// Event sender for async processing
    event_tx: mpsc::Sender<AnalyticsEvent>,

    /// Broadcast channel for real-time event subscribers
    broadcast_tx: broadcast::Sender<AnalyticsEvent>,
}

impl EventCollector {
    /// Create a new event collector
    pub fn new(config: AnalyticsConfigManager) -> Self {
        let (event_tx, event_rx) = mpsc::channel(10000);
        let (broadcast_tx, _) = broadcast::channel(1000);

        let collector = Self {
            config: Arc::new(RwLock::new(config)),
            buffer: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            sinks: Arc::new(RwLock::new(Vec::new())),
            sampling: Arc::new(RwLock::new(SamplingState::new())),
            stats: Arc::new(CollectorStats::default()),
            session_id: Uuid::new_v4(),
            batch_sequence: AtomicU64::new(0),
            last_flush: Arc::new(RwLock::new(Instant::now())),
            shutdown: Arc::new(AtomicBool::new(false)),
            event_tx,
            broadcast_tx,
        };

        // Start background processor
        collector.start_processor(event_rx);

        collector
    }

    /// Start the background event processor
    fn start_processor(&self, mut event_rx: mpsc::Receiver<AnalyticsEvent>) {
        let buffer = Arc::clone(&self.buffer);
        let config = Arc::clone(&self.config);
        let sinks = Arc::clone(&self.sinks);
        let stats = Arc::clone(&self.stats);
        let last_flush = Arc::clone(&self.last_flush);
        let shutdown = Arc::clone(&self.shutdown);
        let batch_sequence = self.batch_sequence.load(Ordering::Relaxed);

        tokio::spawn(async move {
            let mut flush_interval = tokio::time::interval(Duration::from_secs(30));
            let mut batch_seq = batch_sequence;

            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        let mut buf = buffer.write().await;
                        buf.push(event);

                        let cfg = config.read().await;
                        if buf.len() >= cfg.config().collection.buffer_size {
                            drop(cfg);
                            drop(buf);

                            batch_seq += 1;
                            if let Err(e) = Self::flush_buffer(
                                &buffer,
                                &sinks,
                                &stats,
                                &last_flush,
                                batch_seq,
                            ).await {
                                tracing::error!("Flush error: {}", e);
                            }
                        }
                    }

                    _ = flush_interval.tick() => {
                        let buf = buffer.read().await;
                        if !buf.is_empty() {
                            drop(buf);

                            batch_seq += 1;
                            if let Err(e) = Self::flush_buffer(
                                &buffer,
                                &sinks,
                                &stats,
                                &last_flush,
                                batch_seq,
                            ).await {
                                tracing::error!("Periodic flush error: {}", e);
                            }
                        }
                    }

                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
            }

            // Final flush on shutdown
            let _ = Self::flush_buffer(&buffer, &sinks, &stats, &last_flush, batch_seq + 1).await;
        });
    }

    async fn flush_buffer(
        buffer: &Arc<RwLock<Vec<AnalyticsEvent>>>,
        sinks: &Arc<RwLock<Vec<Arc<dyn EventSink>>>>,
        stats: &Arc<CollectorStats>,
        last_flush: &Arc<RwLock<Instant>>,
        sequence: u64,
    ) -> Result<(), CollectorError> {
        let events: Vec<AnalyticsEvent> = {
            let mut buf = buffer.write().await;
            std::mem::take(&mut *buf)
        };

        if events.is_empty() {
            return Ok(());
        }

        let batch = EventBatch::new(events, sequence);
        let batch_len = batch.len();

        let sinks_guard = sinks.read().await;
        for sink in sinks_guard.iter() {
            match sink.process(batch.clone()).await {
                Ok(()) => {}
                Err(e) => {
                    stats.flush_errors.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!("Sink {} error: {}", sink.name(), e);
                }
            }
        }

        stats.events_processed.fetch_add(batch_len as u64, Ordering::Relaxed);
        stats.batches_flushed.fetch_add(1, Ordering::Relaxed);

        *last_flush.write().await = Instant::now();

        Ok(())
    }

    /// Collect a single event
    pub async fn collect(&self, event: AnalyticsEvent) -> Result<(), CollectorError> {
        self.stats.events_received.fetch_add(1, Ordering::Relaxed);

        // Check if collection is enabled
        let config = self.config.read().await;
        if !config.should_collect(&event.event_type, event.priority) {
            return Ok(());
        }

        // Apply sampling
        let sampling_config = config.get_sampling(&event.event_type);
        drop(config);

        {
            let mut sampling = self.sampling.write().await;
            if !sampling.should_sample(&event.event_type, &sampling_config) {
                self.stats.events_sampled_out.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }

        // Enrich with session ID
        let mut event = event;
        if event.session_id.is_none() {
            event.session_id = Some(self.session_id);
        }

        // Broadcast to real-time subscribers
        let _ = self.broadcast_tx.send(event.clone());

        // Send to buffer
        self.event_tx
            .send(event)
            .await
            .map_err(|_| CollectorError::QueueFull)?;

        Ok(())
    }

    /// Collect multiple events
    pub async fn collect_batch(&self, events: Vec<AnalyticsEvent>) -> Result<(), CollectorError> {
        for event in events {
            self.collect(event).await?;
        }
        Ok(())
    }

    /// Register an event sink
    pub async fn register_sink(&self, sink: Arc<dyn EventSink>) {
        let mut sinks = self.sinks.write().await;
        sinks.push(sink);
    }

    /// Unregister an event sink by name
    pub async fn unregister_sink(&self, name: &str) {
        let mut sinks = self.sinks.write().await;
        sinks.retain(|s| s.name() != name);
    }

    /// Force flush all buffered events
    pub async fn flush(&self) -> Result<(), CollectorError> {
        let sequence = self.batch_sequence.fetch_add(1, Ordering::Relaxed);
        Self::flush_buffer(
            &self.buffer,
            &self.sinks,
            &self.stats,
            &self.last_flush,
            sequence,
        )
        .await
    }

    /// Get collector statistics
    pub fn stats(&self) -> CollectorStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get current session ID
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    /// Subscribe to real-time events
    pub fn subscribe(&self) -> broadcast::Receiver<AnalyticsEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Update configuration at runtime
    pub async fn update_config(&self, config: AnalyticsConfigManager) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<(), CollectorError> {
        self.shutdown.store(true, Ordering::Relaxed);

        // Flush remaining events
        self.flush().await?;

        // Flush all sinks
        let sinks = self.sinks.read().await;
        for sink in sinks.iter() {
            if let Err(e) = sink.flush().await {
                tracing::warn!("Sink {} flush error: {}", sink.name(), e);
            }
        }

        Ok(())
    }
}

/// Errors from the collector
#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    #[error("Event queue is full")]
    QueueFull,

    #[error("Collector is shut down")]
    Shutdown,

    #[error("Sink error: {0}")]
    SinkError(#[from] SinkError),
}

/// Global collector instance
static GLOBAL_COLLECTOR: once_cell::sync::OnceCell<Arc<EventCollector>> =
    once_cell::sync::OnceCell::new();

/// Initialize the global collector
pub fn init_global_collector(config: AnalyticsConfigManager) -> Arc<EventCollector> {
    GLOBAL_COLLECTOR
        .get_or_init(|| Arc::new(EventCollector::new(config)))
        .clone()
}

/// Get the global collector
pub fn global_collector() -> Option<Arc<EventCollector>> {
    GLOBAL_COLLECTOR.get().cloned()
}

/// Convenience macro for collecting events
#[macro_export]
macro_rules! track_event {
    ($event_type:expr) => {
        if let Some(collector) = $crate::analytics::collector::global_collector() {
            let event = $crate::analytics::types::EventBuilder::new($event_type).build();
            let _ = collector.collect(event).await;
        }
    };
    ($event_type:expr, $data:expr) => {
        if let Some(collector) = $crate::analytics::collector::global_collector() {
            let event = $crate::analytics::types::EventBuilder::new($event_type)
                .data($data)
                .build();
            let _ = collector.collect(event).await;
        }
    };
}

/// Simple in-memory sink for testing
pub struct MemorySink {
    events: Arc<RwLock<Vec<EventBatch>>>,
}

impl MemorySink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_events(&self) -> Vec<EventBatch> {
        self.events.read().await.clone()
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

impl Default for MemorySink {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventSink for MemorySink {
    async fn process(&self, batch: EventBatch) -> Result<(), SinkError> {
        let mut events = self.events.write().await;
        events.push(batch);
        Ok(())
    }

    async fn flush(&self) -> Result<(), SinkError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// Console sink for debugging
pub struct ConsoleSink {
    prefix: String,
}

impl ConsoleSink {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

#[async_trait]
impl EventSink for ConsoleSink {
    async fn process(&self, batch: EventBatch) -> Result<(), SinkError> {
        for event in &batch.events {
            println!(
                "[{}] {:?}: {:?}",
                self.prefix, event.event_type, event.data
            );
        }
        Ok(())
    }

    async fn flush(&self) -> Result<(), SinkError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::types::{EventBuilder, EventData, UsageEventData};

    #[tokio::test]
    async fn test_collector_creation() {
        let config = AnalyticsConfigManager::new();
        let collector = EventCollector::new(config);

        assert_eq!(collector.stats().events_received, 0);
    }

    #[tokio::test]
    async fn test_event_collection() {
        let config = AnalyticsConfigManager::new();
        let collector = EventCollector::new(config);

        let sink = Arc::new(MemorySink::new());
        collector.register_sink(sink.clone()).await;

        let event = EventBuilder::new(EventType::MissionCreated)
            .usage_data("mission", "create", true)
            .build();

        collector.collect(event).await.unwrap();
        collector.flush().await.unwrap();

        let batches = sink.get_events().await;
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].events.len(), 1);
    }

    #[tokio::test]
    async fn test_session_enrichment() {
        let config = AnalyticsConfigManager::new();
        let collector = EventCollector::new(config);

        let sink = Arc::new(MemorySink::new());
        collector.register_sink(sink.clone()).await;

        let event = EventBuilder::new(EventType::SessionStarted).build();
        assert!(event.session_id.is_none());

        collector.collect(event).await.unwrap();
        collector.flush().await.unwrap();

        let batches = sink.get_events().await;
        let collected_event = &batches[0].events[0];
        assert_eq!(collected_event.session_id, Some(collector.session_id()));
    }

    #[tokio::test]
    async fn test_broadcast_subscription() {
        let config = AnalyticsConfigManager::new();
        let collector = EventCollector::new(config);

        let mut subscriber = collector.subscribe();

        let event = EventBuilder::new(EventType::FeatureUsed)
            .usage_data("test", "action", true)
            .build();
        let event_id = event.id;

        collector.collect(event).await.unwrap();

        // Give time for broadcast
        tokio::time::sleep(Duration::from_millis(50)).await;

        let received = subscriber.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().id, event_id);
    }

    #[tokio::test]
    async fn test_sampling() {
        let mut config = AnalyticsConfigManager::new();
        // This would need proper config setup for sampling

        let collector = EventCollector::new(config);
        let sink = Arc::new(MemorySink::new());
        collector.register_sink(sink.clone()).await;

        // Collect many events
        for _ in 0..100 {
            let event = EventBuilder::new(EventType::ResponseLatency)
                .performance_data("latency", 100.0, "ms")
                .build();
            collector.collect(event).await.unwrap();
        }

        collector.flush().await.unwrap();

        let stats = collector.stats();
        assert_eq!(stats.events_received, 100);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Event collection and buffering
   - Sampling logic accuracy
   - Session ID enrichment
   - Sink registration/unregistration

2. **Integration Tests**
   - End-to-end event flow
   - Multiple concurrent collectors
   - Flush timing behavior
   - Shutdown sequence

3. **Load Tests**
   - High-volume event processing
   - Back-pressure handling
   - Memory usage under load

---

## Related Specs

- Spec 406: Analytics Types
- Spec 407: Analytics Configuration
- Spec 409: Analytics Storage
- Spec 410: Analytics Aggregation
