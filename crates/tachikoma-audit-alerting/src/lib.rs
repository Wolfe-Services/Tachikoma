//! Audit alerting system.
//!
//! Provides real-time alerting based on audit events, enabling immediate
//! notification of security incidents and critical events.

mod alerting;
mod alert_engine;
mod notification;

pub use alerting::*;
pub use alert_engine::{AlertEngine, AlertEngineConfig};
pub use notification::{NotificationDispatcher, NotificationHandler, NotificationError};

// Re-export important types from audit-types
pub use tachikoma_audit_types::{
    AuditAction, AuditActor, AuditCategory, AuditEvent, AuditOutcome, AuditSeverity, AuditTarget,
};