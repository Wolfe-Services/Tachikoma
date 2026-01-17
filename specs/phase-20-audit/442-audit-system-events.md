# 442 - Audit System Events

**Phase:** 20 - Audit System
**Spec ID:** 442
**Status:** Planned
**Dependencies:** 431-audit-event-types, 433-audit-capture
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement automatic capture of system-level events including startup, shutdown, errors, configuration changes, and health events.

---

## Acceptance Criteria

- [x] Application lifecycle events
- [x] Configuration change tracking
- [x] Error event capture
- [x] Health/status events
- [x] Resource usage events

---

## Implementation Details

### 1. System Event Types (src/system_events.rs)

```rust
//! System-level audit event capture.

use crate::{
    AuditAction, AuditActor, AuditCapture, AuditCategory, AuditEvent,
    AuditOutcome, AuditSeverity, AuditTarget,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System event recorder.
pub struct SystemEventRecorder {
    capture: AuditCapture,
    component: String,
}

impl SystemEventRecorder {
    /// Create a new system event recorder.
    pub fn new(capture: AuditCapture, component: impl Into<String>) -> Self {
        Self {
            capture,
            component: component.into(),
        }
    }

    /// Record system startup.
    pub fn startup(&self, version: &str, config_hash: &str) {
        let event = AuditEvent::builder(AuditCategory::System, AuditAction::SystemStartup)
            .actor(AuditActor::system(&self.component))
            .severity(AuditSeverity::Info)
            .metadata("version", version)
            .metadata("config_hash", config_hash)
            .metadata("pid", std::process::id())
            .build();
        self.capture.record(event);
    }

    /// Record system shutdown.
    pub fn shutdown(&self, reason: &str, graceful: bool) {
        let severity = if graceful {
            AuditSeverity::Info
        } else {
            AuditSeverity::High
        };

        let event = AuditEvent::builder(AuditCategory::System, AuditAction::SystemShutdown)
            .actor(AuditActor::system(&self.component))
            .severity(severity)
            .metadata("reason", reason)
            .metadata("graceful", graceful)
            .build();
        self.capture.record(event);
    }

    /// Record a system error.
    pub fn error(&self, error_type: &str, message: &str, stack_trace: Option<&str>) {
        let mut builder = AuditEvent::builder(AuditCategory::System, AuditAction::SystemError)
            .actor(AuditActor::system(&self.component))
            .severity(AuditSeverity::High)
            .outcome(AuditOutcome::Failure {
                reason: message.to_string(),
            })
            .metadata("error_type", error_type)
            .metadata("message", message);

        if let Some(trace) = stack_trace {
            builder = builder.metadata("stack_trace", trace);
        }

        self.capture.record(builder.build());
    }

    /// Record a configuration change.
    pub fn config_changed(
        &self,
        config_key: &str,
        old_value: Option<&str>,
        new_value: &str,
        changed_by: Option<&str>,
    ) {
        let mut builder = AuditEvent::builder(AuditCategory::Configuration, AuditAction::ConfigUpdated)
            .actor(match changed_by {
                Some(user) => AuditActor::User {
                    user_id: tachikoma_common_core::UserId::new(),
                    username: Some(user.to_string()),
                    session_id: None,
                },
                None => AuditActor::system(&self.component),
            })
            .severity(AuditSeverity::Medium)
            .target(AuditTarget::new("config", config_key))
            .metadata("key", config_key)
            .metadata("new_value", new_value);

        if let Some(old) = old_value {
            builder = builder.metadata("old_value", old);
        }

        self.capture.record(builder.build());
    }

    /// Record a health check event.
    pub fn health_check(&self, status: HealthStatus, details: HashMap<String, String>) {
        let severity = match status {
            HealthStatus::Healthy => AuditSeverity::Info,
            HealthStatus::Degraded => AuditSeverity::Medium,
            HealthStatus::Unhealthy => AuditSeverity::High,
        };

        let outcome = match status {
            HealthStatus::Healthy => AuditOutcome::Success,
            _ => AuditOutcome::Failure {
                reason: format!("Health status: {:?}", status),
            },
        };

        let mut builder = AuditEvent::builder(AuditCategory::System, AuditAction::Custom("health_check".to_string()))
            .actor(AuditActor::system(&self.component))
            .severity(severity)
            .outcome(outcome)
            .metadata("status", format!("{:?}", status));

        for (key, value) in details {
            builder = builder.metadata(key, value);
        }

        self.capture.record(builder.build());
    }

    /// Record resource usage.
    pub fn resource_usage(&self, metrics: ResourceMetrics) {
        let severity = if metrics.is_critical() {
            AuditSeverity::High
        } else if metrics.is_warning() {
            AuditSeverity::Medium
        } else {
            AuditSeverity::Info
        };

        let event = AuditEvent::builder(AuditCategory::System, AuditAction::Custom("resource_usage".to_string()))
            .actor(AuditActor::system(&self.component))
            .severity(severity)
            .metadata("cpu_percent", metrics.cpu_percent)
            .metadata("memory_percent", metrics.memory_percent)
            .metadata("disk_percent", metrics.disk_percent)
            .metadata("open_files", metrics.open_files)
            .build();
        self.capture.record(event);
    }

    /// Record a backup event.
    pub fn backup(&self, backup_type: &str, size_bytes: u64, success: bool, location: &str) {
        let action = if success {
            AuditAction::BackupCreated
        } else {
            AuditAction::SystemError
        };

        let outcome = if success {
            AuditOutcome::Success
        } else {
            AuditOutcome::Failure {
                reason: "Backup failed".to_string(),
            }
        };

        let event = AuditEvent::builder(AuditCategory::System, action)
            .actor(AuditActor::system(&self.component))
            .severity(if success { AuditSeverity::Info } else { AuditSeverity::High })
            .outcome(outcome)
            .target(AuditTarget::new("backup", location))
            .metadata("backup_type", backup_type)
            .metadata("size_bytes", size_bytes)
            .metadata("location", location)
            .build();
        self.capture.record(event);
    }
}

/// Health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Resource usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub open_files: u32,
}

impl ResourceMetrics {
    /// Check if any metric is at warning level.
    pub fn is_warning(&self) -> bool {
        self.cpu_percent > 70.0
            || self.memory_percent > 70.0
            || self.disk_percent > 70.0
    }

    /// Check if any metric is at critical level.
    pub fn is_critical(&self) -> bool {
        self.cpu_percent > 90.0
            || self.memory_percent > 90.0
            || self.disk_percent > 90.0
    }
}
```

### 2. Lifecycle Hooks (src/lifecycle.rs)

```rust
//! Application lifecycle audit hooks.

use crate::{AuditCapture, system_events::SystemEventRecorder};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

/// Lifecycle event handler.
pub struct LifecycleAudit {
    recorder: Arc<SystemEventRecorder>,
    version: String,
    config_hash: String,
}

impl LifecycleAudit {
    /// Create a new lifecycle audit handler.
    pub fn new(capture: AuditCapture, version: impl Into<String>) -> Self {
        Self {
            recorder: Arc::new(SystemEventRecorder::new(capture, "tachikoma")),
            version: version.into(),
            config_hash: String::new(),
        }
    }

    /// Set the config hash for startup logging.
    pub fn with_config_hash(mut self, hash: impl Into<String>) -> Self {
        self.config_hash = hash.into();
        self
    }

    /// Record application startup.
    pub fn on_startup(&self) {
        info!("Recording application startup");
        self.recorder.startup(&self.version, &self.config_hash);
    }

    /// Record application shutdown.
    pub fn on_shutdown(&self, reason: &str, graceful: bool) {
        info!("Recording application shutdown: {}", reason);
        self.recorder.shutdown(reason, graceful);
    }

    /// Install shutdown signal handlers.
    pub async fn install_signal_handlers(self: Arc<Self>) {
        let recorder = self.clone();

        tokio::spawn(async move {
            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {
                    recorder.on_shutdown("SIGINT received", true);
                }
                _ = terminate => {
                    recorder.on_shutdown("SIGTERM received", true);
                }
            }
        });
    }

    /// Record a panic.
    pub fn on_panic(&self, info: &std::panic::PanicInfo) {
        let message = match info.payload().downcast_ref::<&str>() {
            Some(s) => s.to_string(),
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.clone(),
                None => "Unknown panic".to_string(),
            },
        };

        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));

        self.recorder.error(
            "panic",
            &message,
            location.as_deref(),
        );
    }
}

/// Install panic hook for audit logging.
pub fn install_panic_hook(lifecycle: Arc<LifecycleAudit>) {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        lifecycle.on_panic(info);
        default_hook(info);
    }));
}
```

### 3. Configuration Watcher (src/config_watcher.rs)

```rust
//! Configuration change auditing.

use crate::system_events::SystemEventRecorder;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Configuration file watcher for audit logging.
pub struct ConfigWatcher {
    recorder: Arc<SystemEventRecorder>,
    watcher: RecommendedWatcher,
    rx: mpsc::Receiver<notify::Result<Event>>,
}

impl ConfigWatcher {
    /// Create a new config watcher.
    pub fn new(recorder: Arc<SystemEventRecorder>) -> Result<Self, ConfigWatchError> {
        let (tx, rx) = mpsc::channel(100);

        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.blocking_send(res);
        })?;

        Ok(Self { recorder, watcher, rx })
    }

    /// Watch a configuration file or directory.
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<(), ConfigWatchError> {
        self.watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        Ok(())
    }

    /// Start processing file change events.
    pub async fn start(mut self) {
        while let Some(result) = self.rx.recv().await {
            match result {
                Ok(event) => {
                    self.handle_event(event);
                }
                Err(e) => {
                    warn!("Config watch error: {}", e);
                }
            }
        }
    }

    fn handle_event(&self, event: Event) {
        use notify::EventKind;

        match event.kind {
            EventKind::Modify(_) => {
                for path in &event.paths {
                    if let Some(filename) = path.file_name() {
                        debug!("Config file modified: {:?}", filename);
                        self.recorder.config_changed(
                            &filename.to_string_lossy(),
                            None,
                            "(modified)",
                            None,
                        );
                    }
                }
            }
            EventKind::Create(_) => {
                for path in &event.paths {
                    if let Some(filename) = path.file_name() {
                        debug!("Config file created: {:?}", filename);
                        self.recorder.config_changed(
                            &filename.to_string_lossy(),
                            None,
                            "(created)",
                            None,
                        );
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    if let Some(filename) = path.file_name() {
                        debug!("Config file removed: {:?}", filename);
                        self.recorder.config_changed(
                            &filename.to_string_lossy(),
                            Some("(existed)"),
                            "(deleted)",
                            None,
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

/// Config watch error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigWatchError {
    #[error("notify error: {0}")]
    Notify(#[from] notify::Error),
}
```

---

## Testing Requirements

1. Startup/shutdown events are captured
2. Error events include stack traces
3. Config changes are tracked
4. Health status maps to correct severity
5. Resource metrics thresholds work

---

## Related Specs

- Depends on: [431-audit-event-types.md](431-audit-event-types.md), [433-audit-capture.md](433-audit-capture.md)
- Next: [443-audit-security.md](443-audit-security.md)
