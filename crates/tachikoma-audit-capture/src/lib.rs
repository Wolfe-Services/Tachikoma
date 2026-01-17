//! Audit event capture middleware for Tachikoma.
//!
//! This crate provides a non-blocking, thread-safe mechanism for capturing
//! audit events throughout the application. It includes:
//!
//! - Async audit event capture
//! - Non-blocking event submission
//! - Batch event collection
//! - Event enrichment (timestamps, correlation IDs)
//! - Thread-safe event queue

mod batch;
mod capture;

pub use batch::{BatchCollector, BatchConfig, EventBatch, batch_processing_loop};
pub use capture::{AuditCapture, AuditContext, CaptureConfig, CapturedEvent};

// Re-export types for convenience
pub use tachikoma_audit_types::{
    AuditAction, AuditActor, AuditCategory, AuditEvent, AuditEventBuilder,
    AuditOutcome, AuditTarget,
};

/// Convenient audit logging macro.
#[macro_export]
macro_rules! audit {
    ($capture:expr, $category:expr, $action:expr) => {
        $capture.record_simple($category, $action, $crate::AuditActor::Unknown)
    };
    ($capture:expr, $category:expr, $action:expr, actor = $actor:expr) => {
        $capture.record_simple($category, $action, $actor)
    };
    ($capture:expr, $category:expr, $action:expr, $($key:ident = $value:expr),+ $(,)?) => {{
        let mut builder = $crate::AuditEvent::builder($category, $action);
        $(
            builder = builder.$key($value);
        )+
        $capture.record(builder.build());
    }};
}