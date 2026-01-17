//! Audit compliance reporting and framework support.

pub mod compliance;
pub mod control_library;
pub mod report_generator;

// GDPR-specific modules
pub mod gdpr;
pub mod dsar_handler;

pub use compliance::*;
pub use control_library::ControlLibrary;
pub use report_generator::{ReportConfig, ReportGenerator, ReportError};

// GDPR exports
pub use dsar_handler::{DsarHandler, AccessResponse, PortableData, GdprError};
pub use gdpr::*;