//! Error handling for the Tachikoma API server.

pub mod context;
pub mod response;
pub mod types;

pub use context::{conflict, not_found, state_conflict, ErrorContext};
pub use types::{ApiError, ApiResult};