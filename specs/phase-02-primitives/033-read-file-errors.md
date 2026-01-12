# 033 - Read File Error Handling

**Phase:** 2 - Five Primitives
**Spec ID:** 033
**Status:** Planned
**Dependencies:** 032-read-file-impl
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive error handling for the `read_file` primitive, including specific error types, helpful error messages, and recovery suggestions.

---

## Acceptance Criteria

- [ ] Specific error types for each failure mode
- [ ] Human-readable error messages with context
- [ ] Error recovery suggestions
- [ ] Proper error logging with tracing
- [ ] Error conversion from std::io::Error
- [ ] Serializable error responses

---

## Implementation Details

### 1. Read File Errors (src/read_file/error.rs)

```rust
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
        path: PathBuf,
        /// Suggestion for similar files if any.
        suggestion: Option<String>,
    },

    /// Permission denied.
    #[error("permission denied reading file: {path}")]
    PermissionDenied {
        path: PathBuf,
        /// Required permission.
        required: String,
    },

    /// File is too large.
    #[error("file too large: {path} is {actual_size} bytes (limit: {max_size} bytes)")]
    TooLarge {
        path: PathBuf,
        actual_size: u64,
        max_size: usize,
    },

    /// File appears to be binary.
    #[error("file appears to be binary: {path}")]
    BinaryFile {
        path: PathBuf,
        /// Detected mime type if available.
        mime_type: Option<String>,
    },

    /// Invalid line range requested.
    #[error("invalid line range: {start}..{end} (file has {total_lines} lines)")]
    InvalidLineRange {
        start: usize,
        end: usize,
        total_lines: usize,
    },

    /// Path is not allowed by security policy.
    #[error("path not allowed by security policy: {path}")]
    PathNotAllowed {
        path: PathBuf,
        /// Reason for denial.
        reason: String,
    },

    /// Path is not a file.
    #[error("path is not a file: {path}")]
    NotAFile {
        path: PathBuf,
        /// Actual type (directory, symlink, etc.).
        actual_type: String,
    },

    /// Encoding error.
    #[error("encoding error in file: {path}")]
    EncodingError {
        path: PathBuf,
        /// Position of encoding error.
        position: Option<usize>,
    },

    /// Generic IO error.
    #[error("IO error reading {path}: {message}")]
    Io {
        path: PathBuf,
        message: String,
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
```

### 2. Error Logging Integration (src/read_file/logging.rs)

```rust
//! Logging utilities for read_file errors.

use super::error::ReadFileError;
use tracing::{error, warn, info, Level};
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
```

### 3. Similar File Suggestion (src/read_file/suggest.rs)

```rust
//! File suggestion utilities for error messages.

use std::path::Path;

/// Find similar file names in the same directory.
pub fn find_similar_files(path: &Path, max_suggestions: usize) -> Vec<String> {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return Vec::new(),
    };

    let parent = match path.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut candidates: Vec<(String, usize)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .filter(|name| name != file_name)
        .map(|name| {
            let distance = levenshtein_distance(file_name, &name);
            (name, distance)
        })
        .filter(|(_, distance)| *distance <= 3) // Only suggest if close enough
        .collect();

    candidates.sort_by_key(|(_, d)| *d);
    candidates.truncate(max_suggestions);

    candidates.into_iter().map(|(name, _)| name).collect()
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("main.rs", "mian.rs"), 2);
        assert_eq!(levenshtein_distance("test", "test"), 0);
    }
}
```

---

## Testing Requirements

1. Each error type can be created and formatted
2. Error codes are unique and consistent
3. Recovery suggestions are helpful and accurate
4. Error responses serialize correctly
5. Similar file suggestion works with typos
6. Logging outputs correct level and fields

---

## Related Specs

- Depends on: [032-read-file-impl.md](032-read-file-impl.md)
- Next: [034-list-files-impl.md](034-list-files-impl.md)
- Related: [048-primitives-audit.md](048-primitives-audit.md)
