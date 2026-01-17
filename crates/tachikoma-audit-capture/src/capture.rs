//! Audit event capture mechanism.

use tachikoma_audit_types::{AuditActor, AuditCategory, AuditAction, AuditEvent, AuditEventBuilder};
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