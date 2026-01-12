# Spec 406: Analytics Event Types

## Phase
19 - Analytics/Telemetry

## Spec ID
406

## Status
Planned

## Dependencies
- Spec 001: Core Architecture (foundational types)
- Spec 101: Event System (event infrastructure)

## Estimated Context
~10%

---

## Objective

Define comprehensive analytics event types for tracking usage patterns, performance metrics, and system behavior in Tachikoma. These types form the foundation for all analytics collection, aggregation, and reporting capabilities.

---

## Acceptance Criteria

- [ ] Define core analytics event enum covering all trackable activities
- [ ] Implement event categorization (usage, performance, error, business)
- [ ] Create event metadata structures with timestamps and context
- [ ] Implement event serialization for storage and transmission
- [ ] Define event priority and sampling levels
- [ ] Create event validation and sanitization utilities
- [ ] Support custom event extensions for plugins
- [ ] Implement event batching structures

---

## Implementation Details

### Core Analytics Types

```rust
// src/analytics/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for analytics events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Category of analytics events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    /// User interaction and feature usage
    Usage,
    /// System performance metrics
    Performance,
    /// Error and exception tracking
    Error,
    /// Business metrics (tokens, costs)
    Business,
    /// Security-related events
    Security,
    /// System lifecycle events
    System,
    /// Custom plugin events
    Custom,
}

/// Priority level for event processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventPriority {
    /// Low priority, can be sampled/dropped
    Low = 0,
    /// Normal priority, standard processing
    Normal = 1,
    /// High priority, always processed
    High = 2,
    /// Critical, immediate processing required
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Sampling configuration for events
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SamplingConfig {
    /// Sampling rate (0.0 to 1.0)
    pub rate: f64,
    /// Minimum events per time window
    pub min_per_window: u32,
    /// Time window in seconds
    pub window_seconds: u64,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            rate: 1.0,
            min_per_window: 1,
            window_seconds: 60,
        }
    }
}

/// Core analytics event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Unique event identifier
    pub id: EventId,
    /// Event category
    pub category: EventCategory,
    /// Specific event type
    pub event_type: EventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Session identifier
    pub session_id: Option<Uuid>,
    /// Event priority
    pub priority: EventPriority,
    /// Event-specific data
    pub data: EventData,
    /// Additional metadata
    pub metadata: EventMetadata,
}

impl AnalyticsEvent {
    pub fn new(event_type: EventType, data: EventData) -> Self {
        Self {
            id: EventId::new(),
            category: event_type.category(),
            event_type,
            timestamp: Utc::now(),
            session_id: None,
            priority: EventPriority::default(),
            data,
            metadata: EventMetadata::default(),
        }
    }

    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Enumeration of all analytics event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Usage events
    SessionStarted,
    SessionEnded,
    MissionCreated,
    MissionCompleted,
    MissionFailed,
    MissionCancelled,
    CommandExecuted,
    FeatureUsed,
    BackendSelected,
    ToolInvoked,

    // Performance events
    ResponseLatency,
    MemoryUsage,
    CpuUsage,
    DiskUsage,
    NetworkLatency,
    CacheHit,
    CacheMiss,

    // Error events
    ErrorOccurred,
    WarningRaised,
    PanicCaught,
    ValidationFailed,
    TimeoutOccurred,
    RetryAttempted,

    // Business events
    TokensConsumed,
    CostIncurred,
    QuotaChecked,
    BudgetAlerted,

    // Security events
    AuthAttempted,
    PermissionDenied,
    SensitiveDataAccessed,

    // System events
    ConfigChanged,
    PluginLoaded,
    PluginUnloaded,
    UpdateAvailable,
    SystemStarted,
    SystemShutdown,

    // Custom events
    Custom(String),
}

impl EventType {
    /// Get the category for this event type
    pub fn category(&self) -> EventCategory {
        match self {
            Self::SessionStarted
            | Self::SessionEnded
            | Self::MissionCreated
            | Self::MissionCompleted
            | Self::MissionFailed
            | Self::MissionCancelled
            | Self::CommandExecuted
            | Self::FeatureUsed
            | Self::BackendSelected
            | Self::ToolInvoked => EventCategory::Usage,

            Self::ResponseLatency
            | Self::MemoryUsage
            | Self::CpuUsage
            | Self::DiskUsage
            | Self::NetworkLatency
            | Self::CacheHit
            | Self::CacheMiss => EventCategory::Performance,

            Self::ErrorOccurred
            | Self::WarningRaised
            | Self::PanicCaught
            | Self::ValidationFailed
            | Self::TimeoutOccurred
            | Self::RetryAttempted => EventCategory::Error,

            Self::TokensConsumed
            | Self::CostIncurred
            | Self::QuotaChecked
            | Self::BudgetAlerted => EventCategory::Business,

            Self::AuthAttempted
            | Self::PermissionDenied
            | Self::SensitiveDataAccessed => EventCategory::Security,

            Self::ConfigChanged
            | Self::PluginLoaded
            | Self::PluginUnloaded
            | Self::UpdateAvailable
            | Self::SystemStarted
            | Self::SystemShutdown => EventCategory::System,

            Self::Custom(_) => EventCategory::Custom,
        }
    }

    /// Get the default sampling config for this event type
    pub fn default_sampling(&self) -> SamplingConfig {
        match self.category() {
            EventCategory::Error | EventCategory::Security => SamplingConfig {
                rate: 1.0,
                min_per_window: 100,
                window_seconds: 60,
            },
            EventCategory::Performance => SamplingConfig {
                rate: 0.1,
                min_per_window: 10,
                window_seconds: 60,
            },
            _ => SamplingConfig::default(),
        }
    }
}

/// Event-specific data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    /// No additional data
    Empty,

    /// Simple key-value pairs
    KeyValue(HashMap<String, serde_json::Value>),

    /// Usage event data
    Usage(UsageEventData),

    /// Performance metrics
    Performance(PerformanceEventData),

    /// Error details
    Error(ErrorEventData),

    /// Business metrics
    Business(BusinessEventData),

    /// Custom structured data
    Custom(serde_json::Value),
}

impl Default for EventData {
    fn default() -> Self {
        Self::Empty
    }
}

/// Data for usage events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEventData {
    /// Feature or component name
    pub feature: String,
    /// Action performed
    pub action: String,
    /// Target of the action
    pub target: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Success indicator
    pub success: bool,
    /// Additional context
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Data for performance events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEventData {
    /// Metric name
    pub metric: String,
    /// Metric value
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Tags for dimensional analysis
    pub tags: HashMap<String, String>,
}

/// Data for error events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEventData {
    /// Error code
    pub code: String,
    /// Error message (sanitized)
    pub message: String,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Stack trace (if available and allowed)
    pub stack_trace: Option<String>,
    /// Component where error occurred
    pub component: String,
    /// Whether error was recovered
    pub recovered: bool,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Data for business events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessEventData {
    /// Metric type
    pub metric_type: BusinessMetricType,
    /// Numeric value
    pub value: f64,
    /// Currency or unit
    pub unit: String,
    /// Associated backend
    pub backend: Option<String>,
    /// Model identifier
    pub model: Option<String>,
}

/// Types of business metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessMetricType {
    InputTokens,
    OutputTokens,
    TotalTokens,
    CostUsd,
    CostCredits,
    QuotaUsed,
    QuotaRemaining,
}

/// Metadata attached to all events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Application version
    pub app_version: Option<String>,
    /// Operating system
    pub os: Option<String>,
    /// Architecture
    pub arch: Option<String>,
    /// Locale
    pub locale: Option<String>,
    /// Timezone offset in minutes
    pub timezone_offset: Option<i32>,
    /// Custom metadata
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    pub fn from_environment() -> Self {
        Self {
            app_version: option_env!("CARGO_PKG_VERSION").map(String::from),
            os: Some(std::env::consts::OS.to_string()),
            arch: Some(std::env::consts::ARCH.to_string()),
            locale: std::env::var("LANG").ok(),
            timezone_offset: None, // Would need chrono-tz
            custom: HashMap::new(),
        }
    }

    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

/// Batch of analytics events for efficient processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    /// Batch identifier
    pub id: Uuid,
    /// Events in the batch
    pub events: Vec<AnalyticsEvent>,
    /// When the batch was created
    pub created_at: DateTime<Utc>,
    /// Sequence number for ordering
    pub sequence: u64,
}

impl EventBatch {
    pub fn new(events: Vec<AnalyticsEvent>, sequence: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            events,
            created_at: Utc::now(),
            sequence,
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Builder for creating analytics events
#[derive(Debug)]
pub struct EventBuilder {
    event_type: EventType,
    priority: EventPriority,
    session_id: Option<Uuid>,
    data: EventData,
    metadata: EventMetadata,
}

impl EventBuilder {
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            priority: EventPriority::default(),
            session_id: None,
            data: EventData::default(),
            metadata: EventMetadata::from_environment(),
        }
    }

    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn data(mut self, data: EventData) -> Self {
        self.data = data;
        self
    }

    pub fn usage_data(mut self, feature: &str, action: &str, success: bool) -> Self {
        self.data = EventData::Usage(UsageEventData {
            feature: feature.to_string(),
            action: action.to_string(),
            target: None,
            duration_ms: None,
            success,
            extra: HashMap::new(),
        });
        self
    }

    pub fn performance_data(mut self, metric: &str, value: f64, unit: &str) -> Self {
        self.data = EventData::Performance(PerformanceEventData {
            metric: metric.to_string(),
            value,
            unit: unit.to_string(),
            tags: HashMap::new(),
        });
        self
    }

    pub fn error_data(
        mut self,
        code: &str,
        message: &str,
        severity: ErrorSeverity,
        component: &str,
    ) -> Self {
        self.data = EventData::Error(ErrorEventData {
            code: code.to_string(),
            message: message.to_string(),
            severity,
            stack_trace: None,
            component: component.to_string(),
            recovered: false,
        });
        self
    }

    pub fn metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn custom_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.custom.insert(key.to_string(), value);
        self
    }

    pub fn build(self) -> AnalyticsEvent {
        AnalyticsEvent {
            id: EventId::new(),
            category: self.event_type.category(),
            event_type: self.event_type,
            timestamp: Utc::now(),
            session_id: self.session_id,
            priority: self.priority,
            data: self.data,
            metadata: self.metadata,
        }
    }
}

/// Validation result for events
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate an analytics event
pub fn validate_event(event: &AnalyticsEvent) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check timestamp is reasonable
    let now = Utc::now();
    let age = now.signed_duration_since(event.timestamp);

    if age.num_days() > 7 {
        warnings.push("Event is more than 7 days old".to_string());
    }

    if age.num_seconds() < 0 {
        errors.push("Event timestamp is in the future".to_string());
    }

    // Validate event-specific data matches type
    match (&event.event_type, &event.data) {
        (EventType::TokensConsumed, EventData::Business(_)) => {}
        (EventType::TokensConsumed, _) => {
            warnings.push("TokensConsumed event should have Business data".to_string());
        }
        (EventType::ErrorOccurred, EventData::Error(_)) => {}
        (EventType::ErrorOccurred, _) => {
            warnings.push("ErrorOccurred event should have Error data".to_string());
        }
        _ => {}
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Sanitize event data to remove sensitive information
pub fn sanitize_event(mut event: AnalyticsEvent) -> AnalyticsEvent {
    // Sanitize error messages
    if let EventData::Error(ref mut error_data) = event.data {
        // Remove potential secrets from error messages
        error_data.message = sanitize_string(&error_data.message);
        if let Some(ref mut trace) = error_data.stack_trace {
            *trace = sanitize_string(trace);
        }
    }

    // Remove sensitive metadata
    event.metadata.custom.remove("api_key");
    event.metadata.custom.remove("token");
    event.metadata.custom.remove("password");
    event.metadata.custom.remove("secret");

    event
}

fn sanitize_string(s: &str) -> String {
    // Pattern to match potential secrets
    let patterns = [
        (r"sk-[a-zA-Z0-9]{20,}", "[REDACTED_API_KEY]"),
        (r"Bearer\s+[a-zA-Z0-9._-]+", "Bearer [REDACTED]"),
        (r"password[\"']?\s*[:=]\s*[\"'][^\"']+[\"']", "password=[REDACTED]"),
    ];

    let mut result = s.to_string();
    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = EventBuilder::new(EventType::MissionCreated)
            .priority(EventPriority::High)
            .usage_data("mission", "create", true)
            .build();

        assert_eq!(event.event_type, EventType::MissionCreated);
        assert_eq!(event.category, EventCategory::Usage);
        assert_eq!(event.priority, EventPriority::High);
    }

    #[test]
    fn test_event_validation() {
        let event = EventBuilder::new(EventType::SessionStarted).build();
        let result = validate_event(&event);
        assert!(result.valid);
    }

    #[test]
    fn test_event_serialization() {
        let event = EventBuilder::new(EventType::TokensConsumed)
            .data(EventData::Business(BusinessEventData {
                metric_type: BusinessMetricType::TotalTokens,
                value: 1500.0,
                unit: "tokens".to_string(),
                backend: Some("anthropic".to_string()),
                model: Some("claude-3-opus".to_string()),
            }))
            .build();

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
    }

    #[test]
    fn test_sanitization() {
        let event = EventBuilder::new(EventType::ErrorOccurred)
            .error_data(
                "API_ERROR",
                "Failed with key sk-abc123def456ghi789jkl012mno345",
                ErrorSeverity::Error,
                "backend",
            )
            .build();

        let sanitized = sanitize_event(event);

        if let EventData::Error(error_data) = sanitized.data {
            assert!(error_data.message.contains("[REDACTED_API_KEY]"));
            assert!(!error_data.message.contains("sk-abc123"));
        } else {
            panic!("Expected Error data");
        }
    }

    #[test]
    fn test_event_batch() {
        let events: Vec<AnalyticsEvent> = (0..10)
            .map(|_| EventBuilder::new(EventType::FeatureUsed).build())
            .collect();

        let batch = EventBatch::new(events, 1);
        assert_eq!(batch.len(), 10);
        assert_eq!(batch.sequence, 1);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Event creation and initialization
   - Event type to category mapping
   - Event serialization/deserialization
   - Validation logic correctness
   - Sanitization effectiveness

2. **Property Tests**
   - Any event type maps to valid category
   - Serialization is reversible
   - Sanitization preserves event structure

3. **Integration Tests**
   - Event batching with real events
   - Metadata extraction from environment

---

## Related Specs

- Spec 407: Analytics Configuration
- Spec 408: Analytics Collector
- Spec 409: Analytics Storage
- Spec 419: Error Tracking
