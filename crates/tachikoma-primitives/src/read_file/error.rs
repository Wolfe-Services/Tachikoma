//! Error types specific to read_file operations.

use std::path::PathBuf;
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Errors that can occur when reading a file.
#[derive(Debug, Error)]
pub enum ReadFileError {
    /// File does not exist.
    #[error("file not found: {path}")]
    NotFound {
        /// Path to the file that was not found.
        path: PathBuf,
        /// Suggestion for similar files if any.
        suggestion: Option<String>,
    },

    /// Permission denied.
    #[error("permission denied reading file: {path}")]
    PermissionDenied {
        /// Path to the file that was denied access.
        path: PathBuf,
        /// Required permission.
        required: String,
    },

    /// File is too large.
    #[error("file too large: {path} is {actual_size} bytes (limit: {max_size} bytes)")]
    TooLarge {
        /// Path to the file.
        path: PathBuf,
        /// Actual file size in bytes.
        actual_size: u64,
        /// Maximum allowed size in bytes.
        max_size: usize,
    },

    /// File appears to be binary.
    #[error("file appears to be binary: {path}")]
    BinaryFile {
        /// Path to the binary file.
        path: PathBuf,
        /// Detected mime type if available.
        mime_type: Option<String>,
    },

    /// Invalid line range requested.
    #[error("invalid line range: {start}..{end} (file has {total_lines} lines)")]
    InvalidLineRange {
        /// Start line number.
        start: usize,
        /// End line number.
        end: usize,
        /// Total lines in the file.
        total_lines: usize,
    },

    /// Path is not allowed by security policy.
    #[error("path not allowed by security policy: {path}")]
    PathNotAllowed {
        /// Path that was denied.
        path: PathBuf,
        /// Reason for denial.
        reason: String,
    },

    /// Path is not a file.
    #[error("path is not a file: {path}")]
    NotAFile {
        /// Path that is not a file.
        path: PathBuf,
        /// Actual type (directory, symlink, etc.).
        actual_type: String,
    },

    /// Encoding error.
    #[error("encoding error in file: {path}")]
    EncodingError {
        /// Path to the file with encoding issues.
        path: PathBuf,
        /// Position of encoding error.
        position: Option<usize>,
    },

    /// Generic IO error.
    #[error("IO error reading {path}: {message}")]
    Io {
        /// Path to the file that caused the IO error.
        path: PathBuf,
        /// Error message from the IO operation.
        message: String,
        /// Source IO error.
        #[source]
        source: std::io::Error,
    },
}

impl ReadFileError {
    /// Get a recovery suggestion for this error.
    pub fn recovery_suggestion(&self) -> String {
        match self {
            Self::NotFound { suggestion, .. } => {
                if let Some(s) = suggestion {
                    format!("Did you mean: {}?", s)
                } else {
                    "Check that the file path is correct and the file exists.".to_string()
                }
            }
            Self::PermissionDenied { .. } => {
                "Check file permissions or run with appropriate privileges.".to_string()
            }
            Self::TooLarge { max_size, .. } => {
                format!(
                    "Use line range options to read a portion of the file, \
                     or increase max_size beyond {} bytes.",
                    max_size
                )
            }
            Self::BinaryFile { .. } => {
                "Use a different tool to view binary files, or use read_bytes instead.".to_string()
            }
            Self::InvalidLineRange { total_lines, .. } => {
                format!("Specify a line range within 1..{}", total_lines)
            }
            Self::PathNotAllowed { reason, .. } => {
                format!("Security policy: {}. Contact administrator if access is needed.", reason)
            }
            Self::NotAFile { actual_type, .. } => {
                format!(
                    "The path points to a {}. Use list_files for directories.",
                    actual_type
                )
            }
            Self::EncodingError { .. } => {
                "File may use a different encoding. Try reading as bytes.".to_string()
            }
            Self::Io { .. } => {
                "Retry the operation or check system resources.".to_string()
            }
        }
    }

    /// Get error code for this error.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "READ_FILE_NOT_FOUND",
            Self::PermissionDenied { .. } => "READ_FILE_PERMISSION_DENIED",
            Self::TooLarge { .. } => "READ_FILE_TOO_LARGE",
            Self::BinaryFile { .. } => "READ_FILE_BINARY",
            Self::InvalidLineRange { .. } => "READ_FILE_INVALID_RANGE",
            Self::PathNotAllowed { .. } => "READ_FILE_PATH_NOT_ALLOWED",
            Self::NotAFile { .. } => "READ_FILE_NOT_A_FILE",
            Self::EncodingError { .. } => "READ_FILE_ENCODING_ERROR",
            Self::Io { .. } => "READ_FILE_IO_ERROR",
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Io { .. })
    }
}

/// Serializable error response for API/IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileErrorResponse {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Path that caused the error.
    pub path: Option<String>,
    /// Recovery suggestion.
    pub suggestion: String,
    /// Whether the operation can be retried.
    pub retryable: bool,
}

impl From<&ReadFileError> for ReadFileErrorResponse {
    fn from(err: &ReadFileError) -> Self {
        let path = match err {
            ReadFileError::NotFound { path, .. }
            | ReadFileError::PermissionDenied { path, .. }
            | ReadFileError::TooLarge { path, .. }
            | ReadFileError::BinaryFile { path, .. }
            | ReadFileError::PathNotAllowed { path, .. }
            | ReadFileError::NotAFile { path, .. }
            | ReadFileError::EncodingError { path, .. }
            | ReadFileError::Io { path, .. } => Some(path.display().to_string()),
            ReadFileError::InvalidLineRange { .. } => None,
        };

        Self {
            code: err.error_code().to_string(),
            message: err.to_string(),
            path,
            suggestion: err.recovery_suggestion(),
            retryable: err.is_retryable(),
        }
    }
}

/// Convert from std::io::Error with path context.
pub fn io_error_with_path(err: std::io::Error, path: PathBuf) -> ReadFileError {
    use std::io::ErrorKind;

    match err.kind() {
        ErrorKind::NotFound => ReadFileError::NotFound {
            path,
            suggestion: None,
        },
        ErrorKind::PermissionDenied => ReadFileError::PermissionDenied {
            path,
            required: "read".to_string(),
        },
        _ => ReadFileError::Io {
            path,
            message: err.to_string(),
            source: err,
        },
    }
}