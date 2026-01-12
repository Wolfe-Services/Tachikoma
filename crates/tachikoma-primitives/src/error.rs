//! Error types for primitives.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during primitive execution.
#[derive(Debug, Error)]
pub enum PrimitiveError {
    /// File not found.
    #[error("file not found: {path}")]
    FileNotFound {
        /// Path to the file that was not found.
        path: PathBuf,
    },

    /// Permission denied.
    #[error("permission denied: {path}")]
    PermissionDenied {
        /// Path to the file that was denied access.
        path: PathBuf,
    },

    /// Path not allowed by configuration.
    #[error("path not allowed: {path}")]
    PathNotAllowed {
        /// Path that is not allowed.
        path: PathBuf,
    },

    /// File too large.
    #[error("file too large: {size} bytes (max: {max})")]
    FileTooLarge {
        /// Actual file size in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        max: usize,
    },

    /// Operation timed out.
    #[error("operation timed out after {duration:?}")]
    Timeout {
        /// Duration after which the operation timed out.
        duration: std::time::Duration,
    },

    /// Command execution failed.
    #[error("command failed with exit code {exit_code}: {message}")]
    CommandFailed {
        /// Exit code of the failed command.
        exit_code: i32,
        /// Error message describing the failure.
        message: String,
    },

    /// Search pattern invalid.
    #[error("invalid search pattern: {pattern}")]
    InvalidPattern {
        /// The invalid pattern string.
        pattern: String,
    },

    /// Edit target not unique.
    #[error("edit target not unique: found {count} matches")]
    NotUnique {
        /// Number of matches found.
        count: usize,
    },

    /// Edit target not found.
    #[error("edit target not found in file")]
    TargetNotFound,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Validation error.
    #[error("validation error: {message}")]
    Validation {
        /// Validation error message.
        message: String,
    },
}

/// Result type alias for primitives.
pub type PrimitiveResult<T> = std::result::Result<T, PrimitiveError>;