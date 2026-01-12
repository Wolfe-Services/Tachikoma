# 114 - Loop Notifications

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 114
**Status:** Planned
**Dependencies:** 096-loop-runner-core, 113-loop-hooks
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement the notification system for the Ralph Loop - delivering alerts and status updates through various channels to keep users informed about loop progress and issues.

---

## Acceptance Criteria

- [ ] Multiple notification channels (desktop, slack, email, etc.)
- [ ] Configurable notification events
- [ ] Notification templates
- [ ] Rate limiting/debouncing
- [ ] Priority levels
- [ ] Quiet hours support
- [ ] Notification history
- [ ] Acknowledgment tracking

---

## Implementation Details

### 1. Notification Types (src/notifications/types.rs)

```rust
//! Notification type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Events that can trigger notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTrigger {
    /// Loop started.
    LoopStarted,
    /// Loop completed successfully.
    LoopCompleted,
    /// Loop failed/stopped.
    LoopFailed,
    /// Loop paused.
    LoopPaused,
    /// Loop resumed.
    LoopResumed,
    /// Iteration completed.
    IterationComplete,
    /// Tests all passing.
    TestsAllPassing,
    /// Tests failing.
    TestsFailing,
    /// No progress detected.
    NoProgress,
    /// Context rebooted.
    ContextRebooted,
    /// Error occurred.
    Error,
    /// Safety limit reached.
    SafetyLimit,
    /// Mode switched.
    ModeSwitched,
    /// Milestone reached.
    MilestoneReached,
    /// User attention needed.
    AttentionNeeded,
}

/// Priority level for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    /// Low priority, informational.
    Low,
    /// Normal priority.
    Normal,
    /// High priority, important.
    High,
    /// Critical, requires immediate attention.
    Critical,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// A notification to be sent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification ID.
    pub id: String,
    /// Trigger event.
    pub trigger: NotificationTrigger,
    /// Priority level.
    pub priority: NotificationPriority,
    /// Title/summary.
    pub title: String,
    /// Body/details.
    pub body: String,
    /// Additional data.
    pub data: HashMap<String, serde_json::Value>,
    /// When it was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Channels to send to.
    pub channels: Vec<String>,
}

impl Notification {
    /// Create a new notification.
    pub fn new(trigger: NotificationTrigger, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            trigger,
            priority: NotificationPriority::default(),
            title: title.into(),
            body: body.into(),
            data: HashMap::new(),
            created_at: chrono::Utc::now(),
            channels: vec![],
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: NotificationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Add data.
    pub fn with_data<V: Serialize>(mut self, key: &str, value: V) -> Self {
        if let Ok(json) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), json);
        }
        self
    }

    /// Set specific channels.
    pub fn to_channels(mut self, channels: Vec<String>) -> Self {
        self.channels = channels;
        self
    }
}

/// Configuration for notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    /// Enable notifications.
    pub enabled: bool,
    /// Default channels.
    pub default_channels: Vec<String>,
    /// Channels configuration.
    pub channels: Vec<ChannelConfig>,
    /// Triggers and their settings.
    pub triggers: HashMap<NotificationTrigger, TriggerConfig>,
    /// Rate limiting.
    pub rate_limit: RateLimitConfig,
    /// Quiet hours.
    pub quiet_hours: Option<QuietHoursConfig>,
    /// Templates.
    pub templates: HashMap<NotificationTrigger, NotificationTemplate>,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_channels: vec!["desktop".to_string()],
            channels: vec![ChannelConfig::Desktop(DesktopConfig::default())],
            triggers: Self::default_triggers(),
            rate_limit: RateLimitConfig::default(),
            quiet_hours: None,
            templates: HashMap::new(),
        }
    }
}

impl NotificationsConfig {
    fn default_triggers() -> HashMap<NotificationTrigger, TriggerConfig> {
        let mut triggers = HashMap::new();

        triggers.insert(NotificationTrigger::LoopCompleted, TriggerConfig {
            enabled: true,
            priority: NotificationPriority::High,
            channels: None,
        });

        triggers.insert(NotificationTrigger::LoopFailed, TriggerConfig {
            enabled: true,
            priority: NotificationPriority::Critical,
            channels: None,
        });

        triggers.insert(NotificationTrigger::Error, TriggerConfig {
            enabled: true,
            priority: NotificationPriority::High,
            channels: None,
        });

        triggers.insert(NotificationTrigger::SafetyLimit, TriggerConfig {
            enabled: true,
            priority: NotificationPriority::Critical,
            channels: None,
        });

        triggers
    }
}

/// Configuration for a specific trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    /// Enable this trigger.
    pub enabled: bool,
    /// Override priority.
    pub priority: NotificationPriority,
    /// Specific channels (None = use default).
    pub channels: Option<Vec<String>>,
}

/// Channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelConfig {
    /// Desktop notifications.
    Desktop(DesktopConfig),
    /// Slack.
    Slack(SlackConfig),
    /// Discord.
    Discord(DiscordConfig),
    /// Email.
    Email(EmailConfig),
    /// Webhook.
    Webhook(WebhookConfig),
    /// SMS.
    Sms(SmsConfig),
    /// Log file.
    LogFile(LogFileConfig),
}

impl ChannelConfig {
    /// Get channel name.
    pub fn name(&self) -> &str {
        match self {
            Self::Desktop(_) => "desktop",
            Self::Slack(_) => "slack",
            Self::Discord(_) => "discord",
            Self::Email(_) => "email",
            Self::Webhook(c) => &c.name,
            Self::Sms(_) => "sms",
            Self::LogFile(_) => "logfile",
        }
    }
}

/// Desktop notification config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Sound for notifications.
    pub sound: Option<String>,
    /// Show icon.
    pub show_icon: bool,
}

/// Slack config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Webhook URL.
    pub webhook_url: String,
    /// Channel override.
    pub channel: Option<String>,
    /// Bot username.
    pub username: Option<String>,
    /// Icon emoji.
    pub icon_emoji: Option<String>,
}

/// Discord config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Webhook URL.
    pub webhook_url: String,
    /// Bot username.
    pub username: Option<String>,
    /// Avatar URL.
    pub avatar_url: Option<String>,
}

/// Email config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server.
    pub smtp_server: String,
    /// SMTP port.
    pub smtp_port: u16,
    /// Username.
    pub username: String,
    /// Password (encrypted/reference).
    pub password_ref: String,
    /// From address.
    pub from: String,
    /// To addresses.
    pub to: Vec<String>,
}

/// Webhook config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Channel name.
    pub name: String,
    /// URL.
    pub url: String,
    /// HTTP method.
    pub method: String,
    /// Headers.
    pub headers: HashMap<String, String>,
}

/// SMS config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// Provider (twilio, etc.).
    pub provider: String,
    /// Account credentials reference.
    pub credentials_ref: String,
    /// Phone numbers.
    pub to: Vec<String>,
}

/// Log file config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileConfig {
    /// Path to log file.
    pub path: std::path::PathBuf,
    /// Format (json, text).
    pub format: String,
}

/// Rate limiting config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Minimum interval between similar notifications.
    #[serde(with = "humantime_serde")]
    pub min_interval: Duration,
    /// Maximum notifications per hour.
    pub max_per_hour: u32,
    /// Debounce window for rapid events.
    #[serde(with = "humantime_serde")]
    pub debounce_window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            min_interval: Duration::from_secs(60),
            max_per_hour: 20,
            debounce_window: Duration::from_secs(5),
        }
    }
}

/// Quiet hours configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    /// Enable quiet hours.
    pub enabled: bool,
    /// Start time (HH:MM).
    pub start: String,
    /// End time (HH:MM).
    pub end: String,
    /// Days to apply.
    pub days: Option<Vec<u8>>,
    /// Allow critical notifications.
    pub allow_critical: bool,
}

/// Notification template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// Title template.
    pub title: String,
    /// Body template.
    pub body: String,
}

/// Sent notification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// The notification.
    pub notification: Notification,
    /// Channels it was sent to.
    pub sent_to: Vec<String>,
    /// Send results.
    pub results: HashMap<String, SendResult>,
    /// Acknowledged.
    pub acknowledged: bool,
    /// Acknowledged at.
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Result of sending to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub success: bool,
    pub error: Option<String>,
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&humantime::format_duration(*duration).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}
```

### 2. Notification Manager (src/notifications/manager.rs)

```rust
//! Notification management and delivery.

use super::types::{
    ChannelConfig, Notification, NotificationPriority, NotificationRecord,
    NotificationTrigger, NotificationsConfig, SendResult,
};
use crate::error::{LoopError, LoopResult};

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages notification delivery.
pub struct NotificationManager {
    /// Configuration.
    config: RwLock<NotificationsConfig>,
    /// Channel senders.
    channels: RwLock<HashMap<String, Arc<dyn NotificationChannel>>>,
    /// Notification history.
    history: RwLock<VecDeque<NotificationRecord>>,
    /// Rate limiting state.
    rate_state: RwLock<RateLimitState>,
    /// Pending notifications (for debouncing).
    pending: RwLock<HashMap<NotificationTrigger, PendingNotification>>,
}

/// Rate limiting state.
struct RateLimitState {
    last_sent: HashMap<NotificationTrigger, Instant>,
    sent_this_hour: u32,
    hour_start: Instant,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            last_sent: HashMap::new(),
            sent_this_hour: 0,
            hour_start: Instant::now(),
        }
    }
}

/// Pending notification for debouncing.
struct PendingNotification {
    notification: Notification,
    first_seen: Instant,
}

/// Trait for notification channels.
#[async_trait::async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Get channel name.
    fn name(&self) -> &str;

    /// Send notification.
    async fn send(&self, notification: &Notification) -> LoopResult<()>;
}

impl NotificationManager {
    /// Create a new notification manager.
    pub fn new(config: NotificationsConfig) -> Self {
        Self {
            config: RwLock::new(config),
            channels: RwLock::new(HashMap::new()),
            history: RwLock::new(VecDeque::new()),
            rate_state: RwLock::new(RateLimitState::default()),
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize channels.
    pub async fn initialize(&self) -> LoopResult<()> {
        let config = self.config.read().await;

        for channel_config in &config.channels {
            let channel: Arc<dyn NotificationChannel> = match channel_config {
                ChannelConfig::Desktop(c) => Arc::new(DesktopChannel::new(c.clone())),
                ChannelConfig::Slack(c) => Arc::new(SlackChannel::new(c.clone())),
                ChannelConfig::Discord(c) => Arc::new(DiscordChannel::new(c.clone())),
                ChannelConfig::Webhook(c) => Arc::new(WebhookChannel::new(c.clone())),
                ChannelConfig::LogFile(c) => Arc::new(LogFileChannel::new(c.clone())),
                _ => continue,
            };

            self.channels.write().await.insert(channel.name().to_string(), channel);
        }

        Ok(())
    }

    /// Send a notification.
    pub async fn notify(&self, notification: Notification) -> LoopResult<NotificationRecord> {
        let config = self.config.read().await;

        if !config.enabled {
            return Ok(NotificationRecord {
                notification,
                sent_to: vec![],
                results: HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
            });
        }

        // Check quiet hours
        if !self.should_notify_now(&config, &notification).await {
            debug!("Notification suppressed due to quiet hours");
            return Ok(NotificationRecord {
                notification,
                sent_to: vec![],
                results: HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
            });
        }

        // Check rate limit
        if !self.check_rate_limit(&notification).await {
            debug!("Notification suppressed due to rate limit");
            return Ok(NotificationRecord {
                notification,
                sent_to: vec![],
                results: HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
            });
        }

        // Determine channels
        let channels = if notification.channels.is_empty() {
            config.triggers
                .get(&notification.trigger)
                .and_then(|t| t.channels.clone())
                .unwrap_or_else(|| config.default_channels.clone())
        } else {
            notification.channels.clone()
        };

        drop(config); // Release lock

        // Send to channels
        let channel_map = self.channels.read().await;
        let mut results = HashMap::new();
        let mut sent_to = Vec::new();

        for channel_name in &channels {
            if let Some(channel) = channel_map.get(channel_name) {
                let result = channel.send(&notification).await;
                results.insert(channel_name.clone(), SendResult {
                    success: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                    sent_at: chrono::Utc::now(),
                });
                if result.is_ok() {
                    sent_to.push(channel_name.clone());
                }
            }
        }

        // Record
        let record = NotificationRecord {
            notification,
            sent_to,
            results,
            acknowledged: false,
            acknowledged_at: None,
        };

        // Add to history
        let mut history = self.history.write().await;
        history.push_back(record.clone());
        while history.len() > 100 {
            history.pop_front();
        }

        Ok(record)
    }

    /// Check if we should notify based on quiet hours.
    async fn should_notify_now(&self, config: &NotificationsConfig, notification: &Notification) -> bool {
        let quiet = match &config.quiet_hours {
            Some(q) if q.enabled => q,
            _ => return true,
        };

        // Allow critical through
        if quiet.allow_critical && notification.priority == NotificationPriority::Critical {
            return true;
        }

        let now = chrono::Local::now();

        // Check day
        if let Some(days) = &quiet.days {
            let day = now.weekday().num_days_from_sunday() as u8;
            if !days.contains(&day) {
                return true;
            }
        }

        // Check time
        let current_time = now.format("%H:%M").to_string();
        let start = &quiet.start;
        let end = &quiet.end;

        if start <= end {
            // Normal range (e.g., 22:00 - 08:00 doesn't apply here)
            if current_time >= *start && current_time < *end {
                return false;
            }
        } else {
            // Overnight range (e.g., 22:00 - 08:00)
            if current_time >= *start || current_time < *end {
                return false;
            }
        }

        true
    }

    /// Check rate limit.
    async fn check_rate_limit(&self, notification: &Notification) -> bool {
        let config = self.config.read().await;
        let limit = &config.rate_limit;
        drop(config);

        let mut state = self.rate_state.write().await;

        // Reset hourly counter if needed
        if state.hour_start.elapsed() >= std::time::Duration::from_secs(3600) {
            state.sent_this_hour = 0;
            state.hour_start = Instant::now();
        }

        // Check hourly limit
        if state.sent_this_hour >= limit.max_per_hour {
            return false;
        }

        // Check minimum interval for this trigger
        if let Some(last) = state.last_sent.get(&notification.trigger) {
            if last.elapsed() < limit.min_interval {
                return false;
            }
        }

        // Update state
        state.last_sent.insert(notification.trigger, Instant::now());
        state.sent_this_hour += 1;

        true
    }

    /// Create notification from trigger.
    pub async fn from_trigger(
        &self,
        trigger: NotificationTrigger,
        data: HashMap<String, serde_json::Value>,
    ) -> Notification {
        let config = self.config.read().await;

        let (title, body) = if let Some(template) = config.templates.get(&trigger) {
            (
                self.apply_template(&template.title, &data),
                self.apply_template(&template.body, &data),
            )
        } else {
            (
                format!("{:?}", trigger),
                data.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            )
        };

        let priority = config.triggers
            .get(&trigger)
            .map(|t| t.priority)
            .unwrap_or_default();

        drop(config);

        Notification {
            id: uuid::Uuid::new_v4().to_string(),
            trigger,
            priority,
            title,
            body,
            data,
            created_at: chrono::Utc::now(),
            channels: vec![],
        }
    }

    /// Apply template substitution.
    fn apply_template(&self, template: &str, data: &HashMap<String, serde_json::Value>) -> String {
        let mut result = template.to_string();

        for (key, value) in data {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        result
    }

    /// Acknowledge a notification.
    pub async fn acknowledge(&self, id: &str) {
        let mut history = self.history.write().await;
        for record in history.iter_mut() {
            if record.notification.id == id {
                record.acknowledged = true;
                record.acknowledged_at = Some(chrono::Utc::now());
                break;
            }
        }
    }

    /// Get notification history.
    pub async fn get_history(&self) -> Vec<NotificationRecord> {
        self.history.read().await.iter().cloned().collect()
    }

    /// Get unacknowledged notifications.
    pub async fn get_unacknowledged(&self) -> Vec<NotificationRecord> {
        self.history
            .read()
            .await
            .iter()
            .filter(|r| !r.acknowledged)
            .cloned()
            .collect()
    }
}

// Channel implementations
struct DesktopChannel { config: super::types::DesktopConfig }
struct SlackChannel { config: super::types::SlackConfig }
struct DiscordChannel { config: super::types::DiscordConfig }
struct WebhookChannel { config: super::types::WebhookConfig }
struct LogFileChannel { config: super::types::LogFileConfig }

impl DesktopChannel {
    fn new(config: super::types::DesktopConfig) -> Self { Self { config } }
}

impl SlackChannel {
    fn new(config: super::types::SlackConfig) -> Self { Self { config } }
}

impl DiscordChannel {
    fn new(config: super::types::DiscordConfig) -> Self { Self { config } }
}

impl WebhookChannel {
    fn new(config: super::types::WebhookConfig) -> Self { Self { config } }
}

impl LogFileChannel {
    fn new(config: super::types::LogFileConfig) -> Self { Self { config } }
}

#[async_trait::async_trait]
impl NotificationChannel for DesktopChannel {
    fn name(&self) -> &str { "desktop" }

    async fn send(&self, notification: &Notification) -> LoopResult<()> {
        #[cfg(feature = "desktop-notifications")]
        {
            notify_rust::Notification::new()
                .summary(&notification.title)
                .body(&notification.body)
                .show()
                .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationChannel for SlackChannel {
    fn name(&self) -> &str { "slack" }

    async fn send(&self, notification: &Notification) -> LoopResult<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "text": format!("*{}*\n{}", notification.title, notification.body),
            "channel": self.config.channel,
            "username": self.config.username,
            "icon_emoji": self.config.icon_emoji,
        });

        client.post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationChannel for DiscordChannel {
    fn name(&self) -> &str { "discord" }

    async fn send(&self, notification: &Notification) -> LoopResult<()> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "content": format!("**{}**\n{}", notification.title, notification.body),
            "username": self.config.username,
            "avatar_url": self.config.avatar_url,
        });

        client.post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationChannel for WebhookChannel {
    fn name(&self) -> &str { &self.config.name }

    async fn send(&self, notification: &Notification) -> LoopResult<()> {
        let client = reqwest::Client::new();
        let mut request = match self.config.method.to_uppercase().as_str() {
            "POST" => client.post(&self.config.url),
            "PUT" => client.put(&self.config.url),
            _ => client.get(&self.config.url),
        };

        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        request.json(&notification)
            .send()
            .await
            .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl NotificationChannel for LogFileChannel {
    fn name(&self) -> &str { "logfile" }

    async fn send(&self, notification: &Notification) -> LoopResult<()> {
        use tokio::io::AsyncWriteExt;

        let entry = if self.config.format == "json" {
            serde_json::to_string(&notification).unwrap_or_default() + "\n"
        } else {
            format!(
                "{} [{}] {}: {}\n",
                notification.created_at.to_rfc3339(),
                format!("{:?}", notification.priority).to_uppercase(),
                notification.title,
                notification.body
            )
        };

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.path)
            .await
            .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;

        file.write_all(entry.as_bytes())
            .await
            .map_err(|e| LoopError::NotificationFailed { source: e.to_string() })?;

        Ok(())
    }
}

use chrono::Datelike;
```

### 3. Module Root (src/notifications/mod.rs)

```rust
//! Notification system for loop events.

pub mod manager;
pub mod types;

pub use manager::{NotificationChannel, NotificationManager};
pub use types::{
    ChannelConfig, DesktopConfig, DiscordConfig, EmailConfig, LogFileConfig,
    Notification, NotificationPriority, NotificationRecord, NotificationsConfig,
    NotificationTemplate, NotificationTrigger, QuietHoursConfig, RateLimitConfig,
    SendResult, SlackConfig, SmsConfig, WebhookConfig,
};
```

---

## Testing Requirements

1. Notifications send to configured channels
2. Rate limiting prevents spam
3. Quiet hours suppress notifications
4. Critical notifications bypass quiet hours
5. Templates are applied correctly
6. History is maintained
7. Acknowledgment works
8. Multiple channels receive notifications

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Depends on: [113-loop-hooks.md](113-loop-hooks.md)
- Next: [115-loop-tests.md](115-loop-tests.md)
- Related: [111-unattended-mode.md](111-unattended-mode.md)
