//! Logging utilities for read_file errors.

use super::error::ReadFileError;
use tracing::{error, warn, info};
use std::path::Path;

/// Log a read_file error with appropriate level and context.
pub fn log_read_error(err: &ReadFileError, operation_id: &str) {
    let code = err.error_code();
    let suggestion = err.recovery_suggestion();

    match err {
        ReadFileError::NotFound { path, .. } => {
            warn!(
                operation_id = %operation_id,
                code = %code,
                path = %path.display(),
                "File not found"
            );
        }
        ReadFileError::PermissionDenied { path, required, .. } => {
            warn!(
                operation_id = %operation_id,
                code = %code,
                path = %path.display(),
                required = %required,
                "Permission denied"
            );
        }
        ReadFileError::TooLarge { path, actual_size, max_size, .. } => {
            info!(
                operation_id = %operation_id,
                code = %code,
                path = %path.display(),
                actual_size = %actual_size,
                max_size = %max_size,
                suggestion = %suggestion,
                "File too large"
            );
        }
        ReadFileError::PathNotAllowed { path, reason, .. } => {
            warn!(
                operation_id = %operation_id,
                code = %code,
                path = %path.display(),
                reason = %reason,
                "Path not allowed by security policy"
            );
        }
        ReadFileError::Io { path, message, .. } => {
            error!(
                operation_id = %operation_id,
                code = %code,
                path = %path.display(),
                message = %message,
                "IO error reading file"
            );
        }
        _ => {
            warn!(
                operation_id = %operation_id,
                code = %code,
                error = %err,
                "Read file error"
            );
        }
    }
}

/// Audit log for successful reads (security).
pub fn log_read_success(path: &Path, bytes_read: usize, operation_id: &str) {
    info!(
        operation_id = %operation_id,
        path = %path.display(),
        bytes_read = %bytes_read,
        "File read successfully"
    );
}