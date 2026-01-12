# 449 - Audit API

**Phase:** 20 - Audit System
**Spec ID:** 449
**Status:** Planned
**Dependencies:** All previous audit specs
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement a comprehensive API for the audit system, providing programmatic access to all audit functionality.

---

## Acceptance Criteria

- [ ] Query API endpoints
- [ ] Admin API for management
- [ ] Event submission API
- [ ] Export/report API
- [ ] WebSocket for real-time events

---

## Implementation Details

### 1. API Types (src/api/types.rs)

```rust
//! Audit API types.

use serde::{Deserialize, Serialize};
use crate::{AuditCategory, AuditSeverity, AuditQuery};

/// API response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: ApiError) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// API error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            code: "UNAUTHORIZED".to_string(),
            message: "Authentication required".to_string(),
            details: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: "FORBIDDEN".to_string(),
            message: message.into(),
            details: None,
        }
    }
}

/// Query request.
#[derive(Debug, Clone, Deserialize)]
pub struct QueryRequest {
    #[serde(flatten)]
    pub query: AuditQuery,
}

/// Event submission request.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitEventRequest {
    pub category: AuditCategory,
    pub action: String,
    #[serde(default)]
    pub severity: Option<AuditSeverity>,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub outcome: Option<String>,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// Export request.
#[derive(Debug, Clone, Deserialize)]
pub struct ExportRequest {
    pub format: crate::export::ExportFormat,
    pub query: AuditQuery,
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub include_metadata: bool,
}

/// Retention policy update request.
#[derive(Debug, Clone, Deserialize)]
pub struct RetentionPolicyRequest {
    pub default_retention_days: Option<u32>,
    pub category_overrides: Option<std::collections::HashMap<String, u32>>,
    pub archive_before_delete: Option<bool>,
}

/// Alert rule request.
#[derive(Debug, Clone, Deserialize)]
pub struct AlertRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub conditions: crate::alerting::AlertConditions,
    pub severity: crate::alerting::AlertSeverity,
    pub channels: Vec<crate::alerting::NotificationChannel>,
    pub throttle_seconds: Option<u32>,
}
```

### 2. Query API (src/api/query.rs)

```rust
//! Query API handlers.

use super::types::*;
use crate::{
    AuditQuery, QueryExecutor, AuditSearch, Timeline, TimelineBuilder,
    TimelineConfig, ActivityTracker,
};
use std::sync::Arc;

/// Query API handler.
pub struct QueryApi {
    executor: Arc<QueryExecutor>,
    search: Arc<AuditSearch>,
    timeline: Arc<TimelineBuilder>,
    activity: Arc<ActivityTracker>,
}

impl QueryApi {
    /// Create new query API.
    pub fn new(
        executor: Arc<QueryExecutor>,
        search: Arc<AuditSearch>,
        timeline: Arc<TimelineBuilder>,
        activity: Arc<ActivityTracker>,
    ) -> Self {
        Self { executor, search, timeline, activity }
    }

    /// Query audit events.
    pub fn query(&self, request: QueryRequest) -> ApiResponse<crate::query::QueryPage<crate::executor::AuditEventSummary>> {
        match self.executor.execute(&request.query) {
            Ok(page) => ApiResponse::success(page),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }

    /// Get a single event by ID.
    pub fn get_event(&self, event_id: &str) -> ApiResponse<Option<crate::AuditEvent>> {
        // Implementation would query by ID
        ApiResponse::success(None)
    }

    /// Search audit events.
    pub fn search(&self, query: &str) -> ApiResponse<crate::search::SearchResults> {
        match self.search.search(query) {
            Ok(results) => ApiResponse::success(results),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }

    /// Get timeline data.
    pub fn timeline(&self, config: TimelineConfig) -> ApiResponse<Timeline> {
        match self.timeline.build(&config) {
            Ok(timeline) => ApiResponse::success(timeline),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }

    /// Get user activity summary.
    pub fn user_activity(
        &self,
        user_id: &str,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> ApiResponse<crate::user_activity::UserActivitySummary> {
        match self.activity.user_summary(user_id, start, end) {
            Ok(summary) => ApiResponse::success(summary),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }
}

impl<T> From<ApiResponse<()>> for ApiResponse<T> {
    fn from(resp: ApiResponse<()>) -> Self {
        ApiResponse {
            success: resp.success,
            data: None,
            error: resp.error,
            timestamp: resp.timestamp,
        }
    }
}
```

### 3. Admin API (src/api/admin.rs)

```rust
//! Admin API handlers.

use super::types::*;
use crate::{
    RetentionEnforcer, RetentionPolicy, AlertEngine, AlertRule,
    IntegrityMonitor, ArchiveCreator, ArchiveMetadata,
};
use std::sync::Arc;

/// Admin API handler.
pub struct AdminApi {
    retention: Arc<RetentionEnforcer>,
    alerts: Arc<AlertEngine>,
    integrity: Arc<IntegrityMonitor>,
    archiver: Arc<ArchiveCreator>,
}

impl AdminApi {
    /// Create new admin API.
    pub fn new(
        retention: Arc<RetentionEnforcer>,
        alerts: Arc<AlertEngine>,
        integrity: Arc<IntegrityMonitor>,
        archiver: Arc<ArchiveCreator>,
    ) -> Self {
        Self { retention, alerts, integrity, archiver }
    }

    /// Get retention policy.
    pub fn get_retention_policy(&self) -> ApiResponse<RetentionPolicy> {
        // Would return current policy
        ApiResponse::success(RetentionPolicy::default())
    }

    /// Update retention policy.
    pub fn update_retention_policy(&self, request: RetentionPolicyRequest) -> ApiResponse<()> {
        // Would update policy
        ApiResponse::success(())
    }

    /// Run retention enforcement manually.
    pub async fn enforce_retention(&self) -> ApiResponse<crate::enforcer::EnforcementResult> {
        match self.retention.enforce().await {
            Ok(result) => ApiResponse::success(result),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }

    /// List alert rules.
    pub fn list_alert_rules(&self) -> ApiResponse<Vec<AlertRule>> {
        ApiResponse::success(self.alerts.rules())
    }

    /// Create alert rule.
    pub fn create_alert_rule(&self, request: AlertRuleRequest) -> ApiResponse<AlertRule> {
        let rule = AlertRule {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description,
            enabled: request.enabled,
            conditions: request.conditions,
            severity: request.severity,
            channels: request.channels,
            throttle: request.throttle_seconds.map(|s| crate::alerting::ThrottleConfig {
                min_interval_seconds: s,
                group_by: None,
                max_per_window: None,
                window_seconds: None,
            }),
            tags: Vec::new(),
        };

        self.alerts.add_rule(rule.clone());
        ApiResponse::success(rule)
    }

    /// Delete alert rule.
    pub fn delete_alert_rule(&self, rule_id: &str) -> ApiResponse<()> {
        self.alerts.remove_rule(rule_id);
        ApiResponse::success(())
    }

    /// Run integrity check.
    pub fn check_integrity(&self) -> ApiResponse<crate::integrity_monitor::IntegrityCheck> {
        let result = self.integrity.check();
        ApiResponse::success(result)
    }

    /// Create archive.
    pub fn create_archive(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
        output_path: &std::path::Path,
    ) -> ApiResponse<ArchiveMetadata> {
        match self.archiver.create_archive(start, end, output_path) {
            Ok(metadata) => ApiResponse::success(metadata),
            Err(e) => ApiResponse::error(ApiError::internal(e.to_string())).into(),
        }
    }
}
```

### 4. WebSocket Events (src/api/websocket.rs)

```rust
//! WebSocket real-time event streaming.

use crate::AuditEvent;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;

/// WebSocket message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// New audit event.
    Event { event: AuditEvent },
    /// Alert triggered.
    Alert { alert: crate::alerting::Alert },
    /// Subscription confirmation.
    Subscribed { categories: Vec<String> },
    /// Error message.
    Error { message: String },
    /// Heartbeat.
    Ping,
    Pong,
}

/// Subscription filter.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionFilter {
    pub categories: Option<Vec<crate::AuditCategory>>,
    pub min_severity: Option<crate::AuditSeverity>,
    pub actors: Option<Vec<String>>,
}

impl SubscriptionFilter {
    /// Check if an event matches this filter.
    pub fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(ref cats) = self.categories {
            if !cats.contains(&event.category) {
                return false;
            }
        }

        if let Some(min_sev) = self.min_severity {
            if event.severity < min_sev {
                return false;
            }
        }

        if let Some(ref actors) = self.actors {
            let actor_id = event.actor.identifier();
            if !actors.iter().any(|a| actor_id.contains(a)) {
                return false;
            }
        }

        true
    }
}

/// WebSocket event broadcaster.
pub struct EventBroadcaster {
    sender: broadcast::Sender<WsMessage>,
}

impl EventBroadcaster {
    /// Create a new broadcaster.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Broadcast an event.
    pub fn broadcast_event(&self, event: AuditEvent) {
        let _ = self.sender.send(WsMessage::Event { event });
    }

    /// Broadcast an alert.
    pub fn broadcast_alert(&self, alert: crate::alerting::Alert) {
        let _ = self.sender.send(WsMessage::Alert { alert });
    }

    /// Subscribe to broadcasts.
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.sender.subscribe()
    }
}

/// Handle a WebSocket connection.
pub async fn handle_websocket_connection<S>(
    stream: S,
    broadcaster: Arc<EventBroadcaster>,
) where
    S: futures_util::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + futures_util::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    let (mut ws_sender, mut ws_receiver) = stream.split();
    let mut broadcast_rx = broadcaster.subscribe();
    let mut filter: Option<SubscriptionFilter> = None;

    loop {
        tokio::select! {
            // Handle incoming messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(sub_filter) = serde_json::from_str::<SubscriptionFilter>(&text) {
                            filter = Some(sub_filter);
                            let categories: Vec<String> = filter.as_ref()
                                .and_then(|f| f.categories.as_ref())
                                .map(|c| c.iter().map(|cat| cat.to_string()).collect())
                                .unwrap_or_default();
                            let msg = WsMessage::Subscribed { categories };
                            let _ = ws_sender.send(Message::Text(serde_json::to_string(&msg).unwrap())).await;
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }

            // Handle broadcast messages
            Ok(msg) = broadcast_rx.recv() => {
                let should_send = match &msg {
                    WsMessage::Event { event } => {
                        filter.as_ref().map(|f| f.matches(event)).unwrap_or(true)
                    }
                    WsMessage::Alert { .. } => true,
                    _ => true,
                };

                if should_send {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if ws_sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}
```

---

## Testing Requirements

1. All API endpoints return correct responses
2. Error handling is consistent
3. WebSocket connections are stable
4. Filters apply correctly
5. Authentication is enforced

---

## Related Specs

- Depends on: All previous audit specs
- Next: [450-audit-tests.md](450-audit-tests.md)
