//! System event recorder.

use crate::{
    AuditAction, AuditActor, AuditCapture, AuditCategory, AuditEvent,
    AuditOutcome, AuditSeverity, AuditTarget, HealthStatus, ResourceMetrics,
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