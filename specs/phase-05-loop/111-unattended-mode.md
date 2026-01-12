# 111 - Unattended Mode

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 111
**Status:** Planned
**Dependencies:** 096-loop-runner-core, 104-stop-conditions
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement unattended mode for the Ralph Loop - a fully autonomous mode where the loop runs without user intervention, relying on stop conditions, safety limits, and notifications to operate safely.

---

## Acceptance Criteria

- [ ] Autonomous execution without prompts
- [ ] Comprehensive safety limits
- [ ] Automatic error recovery
- [ ] Resource usage limits
- [ ] Time-based constraints
- [ ] Notification on important events
- [ ] Graceful degradation
- [ ] Audit logging for review

---

## Implementation Details

### 1. Unattended Mode Types (src/unattended/types.rs)

```rust
//! Unattended mode type definitions.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for unattended mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnattendedConfig {
    /// Enable unattended mode.
    pub enabled: bool,

    /// Safety limits.
    pub safety: SafetyLimits,

    /// Error recovery settings.
    pub recovery: RecoveryConfig,

    /// Resource limits.
    pub resources: ResourceLimits,

    /// Scheduling constraints.
    pub schedule: ScheduleConfig,

    /// Notification settings.
    pub notifications: NotificationConfig,

    /// Audit settings.
    pub audit: AuditConfig,
}

impl Default for UnattendedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            safety: SafetyLimits::default(),
            recovery: RecoveryConfig::default(),
            resources: ResourceLimits::default(),
            schedule: ScheduleConfig::default(),
            notifications: NotificationConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

/// Safety limits for unattended operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyLimits {
    /// Maximum total iterations.
    pub max_iterations: u32,
    /// Maximum total runtime.
    #[serde(with = "humantime_serde")]
    pub max_runtime: Duration,
    /// Maximum consecutive failures.
    pub max_consecutive_failures: u32,
    /// Maximum reboots.
    pub max_reboots: u32,
    /// Maximum files that can be modified.
    pub max_files_modified: u32,
    /// Maximum lines of code changed.
    pub max_lines_changed: u32,
    /// Protected file patterns (won't be modified).
    pub protected_patterns: Vec<String>,
    /// Require tests to pass before considering complete.
    pub require_passing_tests: bool,
}

impl Default for SafetyLimits {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            max_runtime: Duration::from_secs(3600 * 4), // 4 hours
            max_consecutive_failures: 5,
            max_reboots: 20,
            max_files_modified: 50,
            max_lines_changed: 5000,
            protected_patterns: vec![
                ".env*".to_string(),
                "*.key".to_string(),
                "*.pem".to_string(),
                "secrets/*".to_string(),
            ],
            require_passing_tests: true,
        }
    }
}

/// Error recovery configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Enable automatic recovery.
    pub enabled: bool,
    /// Maximum recovery attempts per error type.
    pub max_attempts: u32,
    /// Delay between recovery attempts.
    #[serde(with = "humantime_serde")]
    pub retry_delay: Duration,
    /// Exponential backoff multiplier.
    pub backoff_multiplier: f64,
    /// Maximum backoff delay.
    #[serde(with = "humantime_serde")]
    pub max_backoff: Duration,
    /// Recovery strategies by error type.
    pub strategies: Vec<RecoveryStrategy>,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 3,
            retry_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(300),
            strategies: vec![
                RecoveryStrategy {
                    error_pattern: "context".to_string(),
                    action: RecoveryAction::Reboot,
                },
                RecoveryStrategy {
                    error_pattern: "timeout".to_string(),
                    action: RecoveryAction::Retry,
                },
                RecoveryStrategy {
                    error_pattern: "rate limit".to_string(),
                    action: RecoveryAction::Wait {
                        duration: Duration::from_secs(60),
                    },
                },
            ],
        }
    }
}

/// A recovery strategy for specific errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Error pattern to match.
    pub error_pattern: String,
    /// Action to take.
    pub action: RecoveryAction,
}

/// Recovery action to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecoveryAction {
    /// Retry the operation.
    Retry,
    /// Wait before retrying.
    Wait {
        #[serde(with = "humantime_serde")]
        duration: Duration,
    },
    /// Reboot the context.
    Reboot,
    /// Skip and continue.
    Skip,
    /// Stop the loop.
    Stop,
}

/// Resource usage limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (bytes, 0 = unlimited).
    pub max_memory_bytes: u64,
    /// Maximum disk usage (bytes, 0 = unlimited).
    pub max_disk_bytes: u64,
    /// Maximum API calls per hour.
    pub max_api_calls_per_hour: u32,
    /// Maximum cost (in cents, 0 = unlimited).
    pub max_cost_cents: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 0,
            max_disk_bytes: 0,
            max_api_calls_per_hour: 1000,
            max_cost_cents: 0,
        }
    }
}

/// Schedule constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Only run during certain hours (24h format).
    pub allowed_hours: Option<(u8, u8)>,
    /// Days of week to run (0 = Sunday).
    pub allowed_days: Option<Vec<u8>>,
    /// Pause during specific time windows.
    pub blackout_windows: Vec<TimeWindow>,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            allowed_hours: None,
            allowed_days: None,
            blackout_windows: vec![],
        }
    }
}

/// A time window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time (HH:MM).
    pub start: String,
    /// End time (HH:MM).
    pub end: String,
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable notifications.
    pub enabled: bool,
    /// Events to notify on.
    pub notify_on: Vec<NotificationEvent>,
    /// Notification channels.
    pub channels: Vec<NotificationChannel>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notify_on: vec![
                NotificationEvent::LoopStarted,
                NotificationEvent::LoopCompleted,
                NotificationEvent::LoopFailed,
                NotificationEvent::SafetyLimitReached,
            ],
            channels: vec![],
        }
    }
}

/// Events that can trigger notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationEvent {
    LoopStarted,
    LoopCompleted,
    LoopFailed,
    LoopPaused,
    SafetyLimitReached,
    ErrorRecovered,
    TestsAllPassing,
    MilestoneReached,
}

/// Notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Desktop notification.
    Desktop,
    /// Log file.
    LogFile { path: String },
    /// Webhook.
    Webhook { url: String },
    /// Email.
    Email { address: String },
    /// Slack.
    Slack { webhook_url: String },
}

/// Audit logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging.
    pub enabled: bool,
    /// Audit log path.
    pub log_path: std::path::PathBuf,
    /// Log all iterations.
    pub log_iterations: bool,
    /// Log all file changes.
    pub log_file_changes: bool,
    /// Log all decisions.
    pub log_decisions: bool,
    /// Maximum log size before rotation.
    pub max_log_size_bytes: u64,
    /// Number of log files to keep.
    pub log_retention_count: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: std::path::PathBuf::from(".ralph/audit.log"),
            log_iterations: true,
            log_file_changes: true,
            log_decisions: true,
            max_log_size_bytes: 10 * 1024 * 1024, // 10MB
            log_retention_count: 5,
        }
    }
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

### 2. Unattended Controller (src/unattended/controller.rs)

```rust
//! Unattended mode controller.

use super::types::{
    AuditConfig, NotificationChannel, NotificationEvent, RecoveryAction, RecoveryConfig,
    SafetyLimits, UnattendedConfig,
};
use crate::error::{LoopError, LoopResult};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Controls unattended mode operation.
pub struct UnattendedController {
    /// Configuration.
    config: RwLock<UnattendedConfig>,
    /// Start time.
    start_time: Instant,
    /// Error counts by type.
    error_counts: RwLock<HashMap<String, u32>>,
    /// Files modified this run.
    files_modified: RwLock<Vec<String>>,
    /// Lines changed this run.
    lines_changed: RwLock<u32>,
    /// Audit logger.
    audit_logger: RwLock<Option<AuditLogger>>,
}

impl UnattendedController {
    /// Create a new unattended controller.
    pub fn new(config: UnattendedConfig) -> Self {
        let audit_logger = if config.audit.enabled {
            AuditLogger::new(&config.audit).ok()
        } else {
            None
        };

        Self {
            config: RwLock::new(config),
            start_time: Instant::now(),
            error_counts: RwLock::new(HashMap::new()),
            files_modified: RwLock::new(Vec::new()),
            lines_changed: RwLock::new(0),
            audit_logger: RwLock::new(audit_logger),
        }
    }

    /// Check if we can continue (safety limits).
    pub async fn can_continue(&self, iteration: u32, reboots: u32, consecutive_failures: u32) -> LoopResult<bool> {
        let config = self.config.read().await;
        let limits = &config.safety;

        // Check iteration limit
        if limits.max_iterations > 0 && iteration >= limits.max_iterations {
            warn!("Reached maximum iterations: {}", limits.max_iterations);
            return Ok(false);
        }

        // Check runtime limit
        if self.start_time.elapsed() >= limits.max_runtime {
            warn!("Reached maximum runtime: {:?}", limits.max_runtime);
            return Ok(false);
        }

        // Check reboot limit
        if limits.max_reboots > 0 && reboots >= limits.max_reboots {
            warn!("Reached maximum reboots: {}", limits.max_reboots);
            return Ok(false);
        }

        // Check consecutive failures
        if consecutive_failures >= limits.max_consecutive_failures {
            warn!("Reached maximum consecutive failures: {}", limits.max_consecutive_failures);
            return Ok(false);
        }

        // Check files modified
        let files_count = self.files_modified.read().await.len() as u32;
        if limits.max_files_modified > 0 && files_count >= limits.max_files_modified {
            warn!("Reached maximum files modified: {}", limits.max_files_modified);
            return Ok(false);
        }

        // Check lines changed
        let lines = *self.lines_changed.read().await;
        if limits.max_lines_changed > 0 && lines >= limits.max_lines_changed {
            warn!("Reached maximum lines changed: {}", limits.max_lines_changed);
            return Ok(false);
        }

        // Check schedule
        if !self.is_allowed_time(&config.schedule).await {
            info!("Outside allowed schedule window");
            return Ok(false);
        }

        Ok(true)
    }

    /// Check if a file is protected.
    pub async fn is_protected_file(&self, path: &str) -> bool {
        let config = self.config.read().await;

        for pattern in &config.safety.protected_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(path))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Record file modification.
    pub async fn record_file_modified(&self, path: &str, lines_changed: u32) {
        let mut files = self.files_modified.write().await;
        if !files.contains(&path.to_string()) {
            files.push(path.to_string());
        }

        *self.lines_changed.write().await += lines_changed;

        // Audit log
        if let Some(logger) = self.audit_logger.write().await.as_mut() {
            logger.log_file_change(path, lines_changed).await;
        }
    }

    /// Handle an error with recovery.
    pub async fn handle_error(&self, error: &str) -> RecoveryAction {
        let config = self.config.read().await;

        if !config.recovery.enabled {
            return RecoveryAction::Stop;
        }

        // Find matching strategy
        for strategy in &config.recovery.strategies {
            if error.to_lowercase().contains(&strategy.error_pattern.to_lowercase()) {
                // Check attempt count
                let mut counts = self.error_counts.write().await;
                let count = counts.entry(strategy.error_pattern.clone()).or_insert(0);
                *count += 1;

                if *count > config.recovery.max_attempts {
                    warn!("Max recovery attempts reached for: {}", strategy.error_pattern);
                    return RecoveryAction::Stop;
                }

                info!("Attempting recovery ({}/{}): {:?}",
                    count, config.recovery.max_attempts, strategy.action);

                return strategy.action.clone();
            }
        }

        // Default: retry with backoff
        RecoveryAction::Retry
    }

    /// Calculate retry delay with backoff.
    pub async fn get_retry_delay(&self, attempt: u32) -> std::time::Duration {
        let config = self.config.read().await;
        let base_delay = config.recovery.retry_delay;
        let multiplier = config.recovery.backoff_multiplier;

        let delay = base_delay.mul_f64(multiplier.powi(attempt as i32 - 1));
        delay.min(config.recovery.max_backoff)
    }

    /// Check if current time is allowed.
    async fn is_allowed_time(&self, schedule: &super::types::ScheduleConfig) -> bool {
        let now = chrono::Local::now();

        // Check allowed hours
        if let Some((start, end)) = schedule.allowed_hours {
            let hour = now.hour() as u8;
            if start <= end {
                // Normal range (e.g., 9-17)
                if hour < start || hour >= end {
                    return false;
                }
            } else {
                // Overnight range (e.g., 22-6)
                if hour < start && hour >= end {
                    return false;
                }
            }
        }

        // Check allowed days
        if let Some(days) = &schedule.allowed_days {
            let day = now.weekday().num_days_from_sunday() as u8;
            if !days.contains(&day) {
                return false;
            }
        }

        // Check blackout windows
        for window in &schedule.blackout_windows {
            if let (Ok(start), Ok(end)) = (
                chrono::NaiveTime::parse_from_str(&window.start, "%H:%M"),
                chrono::NaiveTime::parse_from_str(&window.end, "%H:%M"),
            ) {
                let current = now.time();
                if current >= start && current <= end {
                    return false;
                }
            }
        }

        true
    }

    /// Send notification.
    pub async fn notify(&self, event: NotificationEvent, message: &str) {
        let config = self.config.read().await;

        if !config.notifications.enabled {
            return;
        }

        if !config.notifications.notify_on.contains(&event) {
            return;
        }

        for channel in &config.notifications.channels {
            if let Err(e) = self.send_notification(channel, event, message).await {
                warn!("Failed to send notification: {}", e);
            }
        }
    }

    /// Send to specific channel.
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        event: NotificationEvent,
        message: &str,
    ) -> LoopResult<()> {
        match channel {
            NotificationChannel::Desktop => {
                #[cfg(feature = "desktop-notifications")]
                {
                    notify_rust::Notification::new()
                        .summary(&format!("Ralph Loop: {:?}", event))
                        .body(message)
                        .show()
                        .ok();
                }
            }
            NotificationChannel::LogFile { path } => {
                let line = format!(
                    "{} [{:?}] {}\n",
                    chrono::Utc::now().to_rfc3339(),
                    event,
                    message
                );
                tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .await?
                    .write_all(line.as_bytes())
                    .await?;
            }
            NotificationChannel::Webhook { url } => {
                let client = reqwest::Client::new();
                let payload = serde_json::json!({
                    "event": format!("{:?}", event),
                    "message": message,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                client.post(url).json(&payload).send().await.ok();
            }
            NotificationChannel::Slack { webhook_url } => {
                let client = reqwest::Client::new();
                let payload = serde_json::json!({
                    "text": format!("*{:?}*: {}", event, message),
                });
                client.post(webhook_url).json(&payload).send().await.ok();
            }
            NotificationChannel::Email { address: _ } => {
                // Email would require SMTP configuration
                warn!("Email notifications not implemented");
            }
        }

        Ok(())
    }

    /// Log audit entry.
    pub async fn audit(&self, action: &str, details: &str) {
        if let Some(logger) = self.audit_logger.write().await.as_mut() {
            logger.log(action, details).await;
        }
    }

    /// Get summary of safety status.
    pub async fn safety_summary(&self) -> SafetySummary {
        let config = self.config.read().await;
        let limits = &config.safety;

        SafetySummary {
            runtime_elapsed: self.start_time.elapsed(),
            runtime_limit: limits.max_runtime,
            files_modified: self.files_modified.read().await.len() as u32,
            files_limit: limits.max_files_modified,
            lines_changed: *self.lines_changed.read().await,
            lines_limit: limits.max_lines_changed,
        }
    }
}

/// Summary of safety limit status.
#[derive(Debug, Clone)]
pub struct SafetySummary {
    pub runtime_elapsed: std::time::Duration,
    pub runtime_limit: std::time::Duration,
    pub files_modified: u32,
    pub files_limit: u32,
    pub lines_changed: u32,
    pub lines_limit: u32,
}

/// Audit logger.
struct AuditLogger {
    path: std::path::PathBuf,
}

impl AuditLogger {
    fn new(config: &AuditConfig) -> LoopResult<Self> {
        Ok(Self {
            path: config.log_path.clone(),
        })
    }

    async fn log(&mut self, action: &str, details: &str) {
        let entry = format!(
            "{} [{}] {}\n",
            chrono::Utc::now().to_rfc3339(),
            action,
            details
        );

        if let Ok(mut file) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
        {
            use tokio::io::AsyncWriteExt;
            file.write_all(entry.as_bytes()).await.ok();
        }
    }

    async fn log_file_change(&mut self, path: &str, lines: u32) {
        self.log("FILE_CHANGE", &format!("{} ({} lines)", path, lines)).await;
    }
}

use chrono::Timelike;
use tokio::io::AsyncWriteExt;
```

### 3. Module Root (src/unattended/mod.rs)

```rust
//! Unattended mode for autonomous operation.

pub mod controller;
pub mod types;

pub use controller::{SafetySummary, UnattendedController};
pub use types::{
    AuditConfig, NotificationChannel, NotificationConfig, NotificationEvent,
    RecoveryAction, RecoveryConfig, RecoveryStrategy, ResourceLimits,
    SafetyLimits, ScheduleConfig, TimeWindow, UnattendedConfig,
};
```

---

## Testing Requirements

1. Safety limits are enforced
2. Recovery strategies match patterns
3. Backoff delay calculates correctly
4. Schedule checking works
5. Protected files are blocked
6. Notifications are sent
7. Audit log is written
8. File/line tracking is accurate

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Depends on: [104-stop-conditions.md](104-stop-conditions.md)
- Next: [112-mode-switching.md](112-mode-switching.md)
- Related: [110-attended-mode.md](110-attended-mode.md)
