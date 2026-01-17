# 433 - Audit Capture

**Phase:** 20 - Audit System
**Spec ID:** 433
**Status:** Planned
**Dependencies:** 431-audit-event-types, 432-audit-schema
**Estimated Context:** ~14% of Sonnet window

---

## Objective

Implement the audit event capture mechanism, providing a non-blocking interface for recording audit events throughout the application.

---

## Acceptance Criteria

- [x] Async audit event capture
- [x] Non-blocking event submission
- [x] Batch event collection
- [x] Event enrichment (timestamps, correlation IDs)
- [x] Thread-safe event queue

---

## Implementation Details

### 1. Capture Module (src/capture.rs)

```rust
//! Audit event capture mechanism.

use crate::{AuditActor, AuditCategory, AuditAction, AuditEvent, AuditEventBuilder};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

/// Configuration for audit capture.
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Maximum events to buffer before applying backpressure.
    pub buffer_size: usize,
    /// Enable automatic timestamp enrichment.
    pub auto_timestamp: bool,
    /// Default actor if none specified.
    pub default_actor: Option<AuditActor>,
    /// Correlation ID header name for HTTP contexts.
    pub correlation_header: String,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10_000,
            auto_timestamp: true,
            default_actor: None,
            correlation_header: "X-Correlation-ID".to_string(),
        }
    }
}

/// Handle for submitting audit events.
#[derive(Clone)]
pub struct AuditCapture {
    sender: mpsc::Sender<CapturedEvent>,
    config: Arc<CaptureConfig>,
}

/// Internal captured event with metadata.
#[derive(Debug)]
pub struct CapturedEvent {
    pub event: AuditEvent,
    pub captured_at: std::time::Instant,
}

impl AuditCapture {
    /// Create a new capture handle.
    pub fn new(config: CaptureConfig) -> (Self, mpsc::Receiver<CapturedEvent>) {
        let (sender, receiver) = mpsc::channel(config.buffer_size);
        let capture = Self {
            sender,
            config: Arc::new(config),
        };
        (capture, receiver)
    }

    /// Record an audit event (non-blocking).
    pub fn record(&self, event: AuditEvent) {
        let captured = CapturedEvent {
            event,
            captured_at: std::time::Instant::now(),
        };

        match self.sender.try_send(captured) {
            Ok(()) => debug!("Audit event captured"),
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("Audit buffer full, event may be dropped");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                error!("Audit channel closed");
            }
        }
    }

    /// Record an audit event asynchronously.
    pub async fn record_async(&self, event: AuditEvent) {
        let captured = CapturedEvent {
            event,
            captured_at: std::time::Instant::now(),
        };

        if let Err(e) = self.sender.send(captured).await {
            error!("Failed to send audit event: {}", e);
        }
    }

    /// Create a builder with default actor.
    pub fn builder(&self, category: AuditCategory, action: AuditAction) -> AuditEventBuilder {
        let mut builder = AuditEvent::builder(category, action);
        if let Some(actor) = &self.config.default_actor {
            builder = builder.actor(actor.clone());
        }
        builder
    }

    /// Record a simple event with minimal info.
    pub fn record_simple(
        &self,
        category: AuditCategory,
        action: AuditAction,
        actor: AuditActor,
    ) {
        let event = AuditEvent::builder(category, action)
            .actor(actor)
            .build();
        self.record(event);
    }

    /// Check if the capture channel is healthy.
    pub fn is_healthy(&self) -> bool {
        !self.sender.is_closed()
    }

    /// Get approximate buffer usage.
    pub fn buffer_usage(&self) -> f64 {
        let capacity = self.sender.capacity();
        let max_capacity = self.sender.max_capacity();
        1.0 - (capacity as f64 / max_capacity as f64)
    }
}

/// Context-aware audit capture with automatic enrichment.
#[derive(Clone)]
pub struct AuditContext {
    capture: AuditCapture,
    actor: Option<AuditActor>,
    correlation_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl AuditContext {
    /// Create a new context.
    pub fn new(capture: AuditCapture) -> Self {
        Self {
            capture,
            actor: None,
            correlation_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the actor for this context.
    pub fn with_actor(mut self, actor: AuditActor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the correlation ID.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set the IP address.
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the user agent.
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Create an enriched builder.
    pub fn builder(&self, category: AuditCategory, action: AuditAction) -> AuditEventBuilder {
        let mut builder = AuditEvent::builder(category, action);

        if let Some(actor) = &self.actor {
            builder = builder.actor(actor.clone());
        }
        if let Some(correlation_id) = &self.correlation_id {
            builder = builder.correlation_id(correlation_id.clone());
        }
        if let Some(ip) = &self.ip_address {
            builder = builder.ip_address(ip.clone());
        }
        if let Some(ua) = &self.user_agent {
            builder = builder.user_agent(ua.clone());
        }

        builder
    }

    /// Record an event with automatic enrichment.
    pub fn record(&self, category: AuditCategory, action: AuditAction) {
        let event = self.builder(category, action).build();
        self.capture.record(event);
    }
}

/// Macro for convenient audit logging.
#[macro_export]
macro_rules! audit {
    ($capture:expr, $category:expr, $action:expr) => {
        $capture.record_simple($category, $action, $crate::AuditActor::Unknown)
    };
    ($capture:expr, $category:expr, $action:expr, actor = $actor:expr) => {
        $capture.record_simple($category, $action, $actor)
    };
    ($capture:expr, $category:expr, $action:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let mut builder = $crate::AuditEvent::builder($category, $action);
        $(
            builder = builder.$key($value);
        )+
        $capture.record(builder.build());
    }};
}
```

### 2. Batch Collector (src/batch.rs)

```rust
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
```

---

## Testing Requirements

1. Events are captured without blocking
2. Batch collection respects size and time limits
3. Context enrichment works correctly
4. Buffer backpressure is handled gracefully
5. Audit macro generates correct events

---

## Related Specs

- Depends on: [431-audit-event-types.md](431-audit-event-types.md), [432-audit-schema.md](432-audit-schema.md)
- Next: [434-audit-persistence.md](434-audit-persistence.md)
