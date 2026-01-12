# 445 - Audit Alerting

**Phase:** 20 - Audit System
**Spec ID:** 445
**Status:** Planned
**Dependencies:** 433-audit-capture, 443-audit-security
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement real-time alerting based on audit events, enabling immediate notification of security incidents and critical events.

---

## Acceptance Criteria

- [ ] Alert rule definition
- [ ] Real-time event matching
- [ ] Alert notification channels
- [ ] Alert throttling/deduplication
- [ ] Alert acknowledgment tracking

---

## Implementation Details

### 1. Alert Types (src/alerting.rs)

```rust
//! Audit alerting system.

use crate::{AuditCategory, AuditEvent, AuditSeverity};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alert rule definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique rule identifier.
    pub id: String,
    /// Rule name.
    pub name: String,
    /// Rule description.
    pub description: Option<String>,
    /// Is rule enabled.
    pub enabled: bool,
    /// Conditions that trigger the alert.
    pub conditions: AlertConditions,
    /// Alert severity when triggered.
    pub severity: AlertSeverity,
    /// Notification channels.
    pub channels: Vec<NotificationChannel>,
    /// Throttling configuration.
    pub throttle: Option<ThrottleConfig>,
    /// Tags for organization.
    pub tags: Vec<String>,
}

/// Conditions for triggering an alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConditions {
    /// Match specific categories.
    #[serde(default)]
    pub categories: Vec<AuditCategory>,
    /// Match specific actions.
    #[serde(default)]
    pub actions: Vec<String>,
    /// Minimum audit severity.
    pub min_severity: Option<AuditSeverity>,
    /// Match failure outcomes.
    #[serde(default)]
    pub failures_only: bool,
    /// Match specific actor patterns.
    pub actor_pattern: Option<String>,
    /// Match specific target patterns.
    pub target_pattern: Option<String>,
    /// Custom field matches.
    #[serde(default)]
    pub field_matches: HashMap<String, String>,
    /// Threshold conditions.
    pub threshold: Option<ThresholdCondition>,
}

impl AlertConditions {
    /// Check if an event matches these conditions.
    pub fn matches(&self, event: &AuditEvent) -> bool {
        // Category check
        if !self.categories.is_empty() && !self.categories.contains(&event.category) {
            return false;
        }

        // Action check
        if !self.actions.is_empty() {
            let action_str = format!("{:?}", event.action).to_lowercase();
            if !self.actions.iter().any(|a| action_str.contains(&a.to_lowercase())) {
                return false;
            }
        }

        // Severity check
        if let Some(min_sev) = self.min_severity {
            if event.severity < min_sev {
                return false;
            }
        }

        // Failure check
        if self.failures_only && event.outcome.is_success() {
            return false;
        }

        // Actor pattern check
        if let Some(ref pattern) = self.actor_pattern {
            let actor_id = event.actor.identifier();
            if !actor_id.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        // Target pattern check
        if let Some(ref pattern) = self.target_pattern {
            if let Some(ref target) = event.target {
                if !target.resource_id.to_lowercase().contains(&pattern.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Threshold-based alert condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdCondition {
    /// Number of events to trigger.
    pub count: u32,
    /// Time window for counting.
    pub window_seconds: u32,
    /// Group by field (for per-entity thresholds).
    pub group_by: Option<String>,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert throttling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleConfig {
    /// Minimum time between alerts.
    pub min_interval_seconds: u32,
    /// Group throttling by field.
    pub group_by: Option<String>,
    /// Maximum alerts per window.
    pub max_per_window: Option<u32>,
    /// Window duration in seconds.
    pub window_seconds: Option<u32>,
}

/// Notification channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannel {
    /// In-app notification.
    InApp {
        /// Target users.
        users: Vec<String>,
    },
    /// Email notification.
    Email {
        /// Recipient addresses.
        recipients: Vec<String>,
        /// Email template.
        template: Option<String>,
    },
    /// Webhook notification.
    Webhook {
        /// Webhook URL.
        url: String,
        /// HTTP headers.
        headers: HashMap<String, String>,
        /// Request template.
        template: Option<String>,
    },
    /// Slack notification.
    Slack {
        /// Webhook URL.
        webhook_url: String,
        /// Channel override.
        channel: Option<String>,
    },
}

/// A triggered alert instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert instance ID.
    pub id: String,
    /// Rule that triggered this alert.
    pub rule_id: String,
    /// Alert name (from rule).
    pub name: String,
    /// Alert severity.
    pub severity: AlertSeverity,
    /// When triggered.
    pub triggered_at: DateTime<Utc>,
    /// Triggering event(s).
    pub trigger_events: Vec<String>,
    /// Alert message.
    pub message: String,
    /// Current status.
    pub status: AlertStatus,
    /// When acknowledged (if applicable).
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Who acknowledged (if applicable).
    pub acknowledged_by: Option<String>,
    /// Resolution notes.
    pub resolution_notes: Option<String>,
}

/// Alert status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    /// Alert is active/unacknowledged.
    Active,
    /// Alert has been acknowledged.
    Acknowledged,
    /// Alert has been resolved.
    Resolved,
    /// Alert was suppressed by throttling.
    Suppressed,
}
```

### 2. Alert Engine (src/alert_engine.rs)

```rust
//! Alert processing engine.

use crate::alerting::*;
use crate::AuditEvent;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Alert engine configuration.
#[derive(Debug, Clone)]
pub struct AlertEngineConfig {
    /// Maximum alerts to buffer.
    pub buffer_size: usize,
    /// Default throttle interval.
    pub default_throttle_seconds: u32,
}

impl Default for AlertEngineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            default_throttle_seconds: 60,
        }
    }
}

/// Alert engine for processing events and generating alerts.
pub struct AlertEngine {
    rules: Arc<RwLock<Vec<AlertRule>>>,
    throttle_state: Arc<RwLock<ThrottleState>>,
    alert_sender: mpsc::Sender<Alert>,
    config: AlertEngineConfig,
}

struct ThrottleState {
    last_alert_times: HashMap<String, DateTime<Utc>>,
    window_counts: HashMap<String, Vec<DateTime<Utc>>>,
}

impl ThrottleState {
    fn new() -> Self {
        Self {
            last_alert_times: HashMap::new(),
            window_counts: HashMap::new(),
        }
    }

    fn should_throttle(&mut self, rule_id: &str, group_key: Option<&str>, config: &ThrottleConfig) -> bool {
        let key = match group_key {
            Some(g) => format!("{}:{}", rule_id, g),
            None => rule_id.to_string(),
        };

        let now = Utc::now();

        // Check minimum interval
        if let Some(last) = self.last_alert_times.get(&key) {
            let elapsed = (now - *last).num_seconds() as u32;
            if elapsed < config.min_interval_seconds {
                return true;
            }
        }

        // Check window limit
        if let (Some(max), Some(window)) = (config.max_per_window, config.window_seconds) {
            let counts = self.window_counts.entry(key.clone()).or_insert_with(Vec::new);

            // Clean old entries
            let cutoff = now - chrono::Duration::seconds(window as i64);
            counts.retain(|t| *t > cutoff);

            if counts.len() >= max as usize {
                return true;
            }

            counts.push(now);
        }

        self.last_alert_times.insert(key, now);
        false
    }
}

impl AlertEngine {
    /// Create a new alert engine.
    pub fn new(config: AlertEngineConfig) -> (Self, mpsc::Receiver<Alert>) {
        let (sender, receiver) = mpsc::channel(config.buffer_size);

        let engine = Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            throttle_state: Arc::new(RwLock::new(ThrottleState::new())),
            alert_sender: sender,
            config,
        };

        (engine, receiver)
    }

    /// Add an alert rule.
    pub fn add_rule(&self, rule: AlertRule) {
        self.rules.write().push(rule);
    }

    /// Remove an alert rule.
    pub fn remove_rule(&self, rule_id: &str) {
        self.rules.write().retain(|r| r.id != rule_id);
    }

    /// Get all rules.
    pub fn rules(&self) -> Vec<AlertRule> {
        self.rules.read().clone()
    }

    /// Process an audit event and generate alerts.
    pub async fn process_event(&self, event: &AuditEvent) {
        let rules = self.rules.read().clone();

        for rule in rules {
            if !rule.enabled {
                continue;
            }

            if rule.conditions.matches(event) {
                self.maybe_trigger_alert(&rule, event).await;
            }
        }
    }

    async fn maybe_trigger_alert(&self, rule: &AlertRule, event: &AuditEvent) {
        // Check throttling
        if let Some(ref throttle) = rule.throttle {
            let group_key = throttle.group_by.as_ref().map(|field| {
                match field.as_str() {
                    "actor_id" => event.actor.identifier(),
                    "target_id" => event.target.as_ref()
                        .map(|t| t.resource_id.clone())
                        .unwrap_or_default(),
                    _ => String::new(),
                }
            });

            if self.throttle_state.write().should_throttle(
                &rule.id,
                group_key.as_deref(),
                throttle,
            ) {
                debug!("Alert throttled for rule {}", rule.id);
                return;
            }
        }

        // Create alert
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            name: rule.name.clone(),
            severity: rule.severity,
            triggered_at: Utc::now(),
            trigger_events: vec![event.id.to_string()],
            message: self.format_alert_message(rule, event),
            status: AlertStatus::Active,
            acknowledged_at: None,
            acknowledged_by: None,
            resolution_notes: None,
        };

        info!("Alert triggered: {} (rule: {})", alert.id, rule.name);

        // Send alert
        if let Err(e) = self.alert_sender.send(alert).await {
            warn!("Failed to send alert: {}", e);
        }
    }

    fn format_alert_message(&self, rule: &AlertRule, event: &AuditEvent) -> String {
        format!(
            "[{}] {} - {} action by {} on {}",
            rule.severity.to_string().to_uppercase(),
            rule.name,
            format!("{:?}", event.action),
            event.actor.identifier(),
            event.target.as_ref()
                .map(|t| t.resource_id.as_str())
                .unwrap_or("unknown")
        )
    }
}

impl AlertSeverity {
    fn to_string(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}
```

### 3. Notification Dispatcher (src/notification.rs)

```rust
//! Alert notification dispatch.

use crate::alerting::{Alert, NotificationChannel};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Notification dispatcher error.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("channel error: {0}")]
    Channel(String),
    #[error("template error: {0}")]
    Template(String),
}

/// Trait for notification handlers.
#[async_trait]
pub trait NotificationHandler: Send + Sync {
    /// Send a notification.
    async fn notify(&self, alert: &Alert) -> Result<(), NotificationError>;
}

/// Notification dispatcher.
pub struct NotificationDispatcher {
    handlers: HashMap<String, Box<dyn NotificationHandler>>,
}

impl NotificationDispatcher {
    /// Create a new dispatcher.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler.
    pub fn register(&mut self, name: &str, handler: Box<dyn NotificationHandler>) {
        self.handlers.insert(name.to_string(), handler);
    }

    /// Dispatch notifications for an alert.
    pub async fn dispatch(&self, alert: &Alert, channels: &[NotificationChannel]) {
        for channel in channels {
            let result = match channel {
                NotificationChannel::InApp { users } => {
                    self.notify_in_app(alert, users).await
                }
                NotificationChannel::Email { recipients, template } => {
                    self.notify_email(alert, recipients, template.as_deref()).await
                }
                NotificationChannel::Webhook { url, headers, template } => {
                    self.notify_webhook(alert, url, headers, template.as_deref()).await
                }
                NotificationChannel::Slack { webhook_url, channel } => {
                    self.notify_slack(alert, webhook_url, channel.as_deref()).await
                }
            };

            if let Err(e) = result {
                error!("Notification failed for alert {}: {}", alert.id, e);
            }
        }
    }

    async fn notify_in_app(&self, alert: &Alert, users: &[String]) -> Result<(), NotificationError> {
        debug!("In-app notification for {} users", users.len());
        // Implementation would push to in-app notification system
        Ok(())
    }

    async fn notify_email(
        &self,
        alert: &Alert,
        recipients: &[String],
        _template: Option<&str>,
    ) -> Result<(), NotificationError> {
        debug!("Email notification to {} recipients", recipients.len());
        // Implementation would use email service
        Ok(())
    }

    async fn notify_webhook(
        &self,
        alert: &Alert,
        url: &str,
        headers: &HashMap<String, String>,
        _template: Option<&str>,
    ) -> Result<(), NotificationError> {
        debug!("Webhook notification to {}", url);

        let client = reqwest::Client::new();
        let mut request = client.post(url).json(alert);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        request.send().await
            .map_err(|e| NotificationError::Http(e.to_string()))?;

        Ok(())
    }

    async fn notify_slack(
        &self,
        alert: &Alert,
        webhook_url: &str,
        channel: Option<&str>,
    ) -> Result<(), NotificationError> {
        debug!("Slack notification to {:?}", channel);

        let color = match alert.severity {
            crate::alerting::AlertSeverity::Info => "#36a64f",
            crate::alerting::AlertSeverity::Warning => "#ff9900",
            crate::alerting::AlertSeverity::Critical => "#ff0000",
        };

        let payload = serde_json::json!({
            "channel": channel,
            "attachments": [{
                "color": color,
                "title": &alert.name,
                "text": &alert.message,
                "fields": [
                    {"title": "Severity", "value": format!("{:?}", alert.severity), "short": true},
                    {"title": "Time", "value": alert.triggered_at.to_rfc3339(), "short": true}
                ],
                "footer": "Tachikoma Audit System"
            }]
        });

        reqwest::Client::new()
            .post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| NotificationError::Http(e.to_string()))?;

        Ok(())
    }
}

impl Default for NotificationDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Testing Requirements

1. Alert rules match events correctly
2. Throttling prevents alert floods
3. All notification channels work
4. Alert status transitions are valid
5. Threshold conditions trigger correctly

---

## Related Specs

- Depends on: [433-audit-capture.md](433-audit-capture.md), [443-audit-security.md](443-audit-security.md)
- Next: [446-audit-immutability.md](446-audit-immutability.md)
