//! System-level audit event capture for Tachikoma.
//!
//! This crate provides automatic capture of system-level events including:
//! - Application lifecycle (startup, shutdown)
//! - Configuration changes
//! - Error events with stack traces
//! - Health/status monitoring
//! - Resource usage tracking

mod lifecycle;
mod recorder;
mod config_watcher;
mod health;
mod resources;

pub use lifecycle::{LifecycleAudit, install_panic_hook};
pub use recorder::SystemEventRecorder;
pub use config_watcher::{ConfigWatcher, ConfigWatchError};
pub use health::HealthStatus;
pub use resources::ResourceMetrics;

// Re-export types for convenience
pub use tachikoma_audit_capture::{AuditCapture, AuditContext};
pub use tachikoma_audit_types::{
    AuditAction, AuditActor, AuditCategory, AuditEvent, AuditEventBuilder,
    AuditOutcome, AuditSeverity, AuditTarget,
};