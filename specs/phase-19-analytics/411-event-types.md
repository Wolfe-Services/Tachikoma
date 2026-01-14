# 411 - Analytics Event Types

## Overview

Core event type definitions for the analytics system, supporting PostHog-style event tracking with extensible properties.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/event_types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for an event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Base analytics event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Unique event identifier
    pub id: EventId,
    /// Event type/name
    pub event: String,
    /// Event category
    pub category: EventCategory,
    /// Distinct user identifier
    pub distinct_id: String,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Event properties
    pub properties: EventProperties,
    /// User properties at time of event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_properties: Option<HashMap<String, serde_json::Value>>,
    /// Session identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Environment (production, staging, etc.)
    pub environment: String,
    /// SDK/source that sent the event
    pub source: EventSource,
    /// Server receive timestamp
    pub received_at: DateTime<Utc>,
}

/// Event categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    /// Page or screen views
    Pageview,
    /// User actions (clicks, form submissions)
    Action,
    /// Custom events
    Custom,
    /// System events (errors, performance)
    System,
    /// Feature flag evaluations
    FeatureFlag,
    /// User identification
    Identify,
    /// Group/company identification
    Group,
    /// Revenue/transaction events
    Revenue,
    /// Session lifecycle
    Session,
}

/// Source of the event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    /// SDK name
    pub sdk: String,
    /// SDK version
    pub sdk_version: String,
    /// Platform (web, ios, android, server)
    pub platform: Platform,
    /// Library used (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Web,
    Ios,
    Android,
    Server,
    Mobile,
    Desktop,
    Unknown,
}

/// Event properties container
pub type EventProperties = HashMap<String, serde_json::Value>;

/// Standard pageview event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageviewEvent {
    /// Page URL
    pub url: String,
    /// Page path
    pub path: String,
    /// Page title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Referrer URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// Query parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_params: Option<HashMap<String, String>>,
    /// UTM parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utm: Option<UtmParams>,
    /// Time spent on page (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_on_page_ms: Option<u64>,
}

/// UTM tracking parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmParams {
    pub source: Option<String>,
    pub medium: Option<String>,
    pub campaign: Option<String>,
    pub term: Option<String>,
    pub content: Option<String>,
}

/// User action event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEvent {
    /// Action name
    pub action: String,
    /// Element type (button, link, form)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,
    /// Element identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    /// Element text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_text: Option<String>,
    /// CSS selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Value associated with action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// User identification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyEvent {
    /// User ID in your system
    pub user_id: String,
    /// Anonymous ID (before identification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymous_id: Option<String>,
    /// User traits/properties
    pub traits: HashMap<String, serde_json::Value>,
    /// Whether to merge with existing user
    #[serde(default)]
    pub merge: bool,
}

/// Group identification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEvent {
    /// Group type (company, team, etc.)
    pub group_type: String,
    /// Group ID
    pub group_id: String,
    /// Group properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Revenue/purchase event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueEvent {
    /// Revenue amount
    pub amount: f64,
    /// Currency code (ISO 4217)
    pub currency: String,
    /// Product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<String>,
    /// Order/transaction ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Quantity
    #[serde(default = "default_quantity")]
    pub quantity: u32,
    /// Product category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Product name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

fn default_quantity() -> u32 {
    1
}

/// Feature flag evaluation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlagEvent {
    /// Flag key
    pub flag_key: String,
    /// Flag value
    pub flag_value: serde_json::Value,
    /// Evaluation reason
    pub reason: String,
    /// Whether user is in experiment
    pub in_experiment: bool,
    /// Experiment variant (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error message
    pub message: String,
    /// Error type/name
    pub error_type: String,
    /// Stack trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<String>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// File where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Line number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

/// Performance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEvent {
    /// Performance metric name
    pub metric: String,
    /// Metric value
    pub value: f64,
    /// Unit (ms, bytes, etc.)
    pub unit: String,
    /// Resource URL (for resource timing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_url: Option<String>,
    /// Performance category
    pub category: PerformanceCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceCategory {
    Navigation,
    Resource,
    Paint,
    LongTask,
    WebVital,
    Custom,
}

/// Session event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// Session ID
    pub session_id: String,
    /// Session event type
    pub session_event: SessionEventType,
    /// Session duration (for end events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Page views in session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_views: Option<u32>,
    /// Events in session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEventType {
    Start,
    End,
    Heartbeat,
}

impl AnalyticsEvent {
    /// Create a new analytics event
    pub fn new(event: &str, distinct_id: &str, category: EventCategory) -> Self {
        Self {
            id: EventId::new(),
            event: event.to_string(),
            category,
            distinct_id: distinct_id.to_string(),
            timestamp: Utc::now(),
            properties: HashMap::new(),
            user_properties: None,
            session_id: None,
            environment: "production".to_string(),
            source: EventSource {
                sdk: "tachikoma".to_string(),
                sdk_version: env!("CARGO_PKG_VERSION").to_string(),
                platform: Platform::Server,
                library: None,
            },
            received_at: Utc::now(),
        }
    }

    /// Create a pageview event
    pub fn pageview(distinct_id: &str, pageview: PageviewEvent) -> Self {
        let mut event = Self::new("$pageview", distinct_id, EventCategory::Pageview);
        event.properties.insert("$current_url".to_string(), serde_json::json!(pageview.url));
        event.properties.insert("$pathname".to_string(), serde_json::json!(pageview.path));
        if let Some(title) = pageview.title {
            event.properties.insert("$title".to_string(), serde_json::json!(title));
        }
        if let Some(referrer) = pageview.referrer {
            event.properties.insert("$referrer".to_string(), serde_json::json!(referrer));
        }
        event
    }

    /// Create an action event
    pub fn action(distinct_id: &str, action: ActionEvent) -> Self {
        let mut event = Self::new(&action.action, distinct_id, EventCategory::Action);
        if let Some(element_type) = action.element_type {
            event.properties.insert("$element_type".to_string(), serde_json::json!(element_type));
        }
        if let Some(element_id) = action.element_id {
            event.properties.insert("$element_id".to_string(), serde_json::json!(element_id));
        }
        event
    }

    /// Create an identify event
    pub fn identify(identify: IdentifyEvent) -> Self {
        let mut event = Self::new("$identify", &identify.user_id, EventCategory::Identify);
        event.properties.insert("$user_id".to_string(), serde_json::json!(identify.user_id));
        event.user_properties = Some(identify.traits);
        event
    }

    /// Add a property
    pub fn with_property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.properties.insert(key.to_string(), value.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Set environment
    pub fn with_environment(mut self, env: &str) -> Self {
        self.environment = env.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pageview() {
        let pageview = PageviewEvent {
            url: "https://example.com/page".to_string(),
            path: "/page".to_string(),
            title: Some("Test Page".to_string()),
            referrer: Some("https://google.com".to_string()),
            query_params: None,
            utm: None,
            time_on_page_ms: None,
        };

        let event = AnalyticsEvent::pageview("user-123", pageview);
        assert_eq!(event.event, "$pageview");
        assert_eq!(event.category, EventCategory::Pageview);
    }

    #[test]
    fn test_event_builder() {
        let event = AnalyticsEvent::new("button_click", "user-123", EventCategory::Action)
            .with_property("button_id", "signup")
            .with_property("page", "/landing")
            .with_session("session-456");

        assert_eq!(event.properties.get("button_id"), Some(&serde_json::json!("signup")));
        assert_eq!(event.session_id, Some("session-456".to_string()));
    }
}
```

## Reserved Event Names

| Event Name | Description |
|------------|-------------|
| $pageview | Page view tracking |
| $identify | User identification |
| $group | Group/company identification |
| $set | Set user properties |
| $unset | Remove user properties |
| $create_alias | Create user alias |
| $feature_flag | Feature flag evaluation |
| $session_start | Session started |
| $session_end | Session ended |

## Related Specs

- 412-event-schema.md - Schema validation
- 413-event-capture.md - Event capture API
- 417-user-identification.md - User identification
