# 012 - Error Types

**Phase:** 1 - Core Common Crates
**Spec ID:** 012
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define a consistent error handling strategy with typed errors, error context, and integration with the `thiserror` crate.

---

## Acceptance Criteria

- [x] Base error type with categories
- [x] Error context chain support
- [x] Serializable error representation
- [x] Error code system
- [x] Human-readable error messages

---

## Implementation Details

### 1. Error Module (src/error.rs)

```rust
//! Error types for Tachikoma.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error category for grouping errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Configuration errors.
    Config,
    /// File system errors.
    FileSystem,
    /// Network/API errors.
    Network,
    /// Backend/model errors.
    Backend,
    /// Validation errors.
    Validation,
    /// Internal errors.
    Internal,
}

/// Unique error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(u16);

impl ErrorCode {
    // Config errors (1xxx)
    pub const CONFIG_NOT_FOUND: Self = Self(1001);
    pub const CONFIG_PARSE_ERROR: Self = Self(1002);
    pub const CONFIG_INVALID_VALUE: Self = Self(1003);

    // File system errors (2xxx)
    pub const FILE_NOT_FOUND: Self = Self(2001);
    pub const FILE_READ_ERROR: Self = Self(2002);
    pub const FILE_WRITE_ERROR: Self = Self(2003);
    pub const PATH_INVALID: Self = Self(2004);

    // Network errors (3xxx)
    pub const NETWORK_TIMEOUT: Self = Self(3001);
    pub const NETWORK_CONNECTION: Self = Self(3002);
    pub const API_ERROR: Self = Self(3003);
    pub const RATE_LIMITED: Self = Self(3004);

    // Backend errors (4xxx)
    pub const BACKEND_UNAVAILABLE: Self = Self(4001);
    pub const BACKEND_AUTH_FAILED: Self = Self(4002);
    pub const CONTEXT_REDLINED: Self = Self(4003);
    pub const TOOL_CALL_FAILED: Self = Self(4004);

    // Validation errors (5xxx)
    pub const VALIDATION_FAILED: Self = Self(5001);
    pub const SPEC_INVALID: Self = Self(5002);
    pub const ID_PARSE_ERROR: Self = Self(5003);

    /// Get error code number.
    pub fn code(&self) -> u16 {
        self.0
    }

    /// Get category from code.
    pub fn category(&self) -> ErrorCategory {
        match self.0 / 1000 {
            1 => ErrorCategory::Config,
            2 => ErrorCategory::FileSystem,
            3 => ErrorCategory::Network,
            4 => ErrorCategory::Backend,
            5 => ErrorCategory::Validation,
            _ => ErrorCategory::Internal,
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

/// Main error type for Tachikoma.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("configuration error: {message}")]
    Config {
        code: ErrorCode,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("file system error: {message}")]
    FileSystem {
        code: ErrorCode,
        message: String,
        path: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("network error: {message}")]
    Network {
        code: ErrorCode,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("backend error: {message}")]
    Backend {
        code: ErrorCode,
        message: String,
        backend: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("validation error: {message}")]
    Validation {
        code: ErrorCode,
        message: String,
        field: Option<String>,
    },

    #[error("internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Get the error code.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Config { code, .. } => *code,
            Self::FileSystem { code, .. } => *code,
            Self::Network { code, .. } => *code,
            Self::Backend { code, .. } => *code,
            Self::Validation { code, .. } => *code,
            Self::Internal { .. } => ErrorCode(9999),
        }
    }

    /// Get the error category.
    pub fn category(&self) -> ErrorCategory {
        self.code().category()
    }

    /// Create a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            code: ErrorCode::CONFIG_INVALID_VALUE,
            message: message.into(),
            source: None,
        }
    }

    /// Create a file not found error.
    pub fn file_not_found(path: impl Into<String>) -> Self {
        let path = path.into();
        Self::FileSystem {
            code: ErrorCode::FILE_NOT_FOUND,
            message: format!("file not found: {}", path),
            path: Some(path),
            source: None,
        }
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            code: ErrorCode::VALIDATION_FAILED,
            message: message.into(),
            field: None,
        }
    }
}

/// Serializable error representation for IPC/API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub category: ErrorCategory,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl From<&Error> for ErrorResponse {
    fn from(err: &Error) -> Self {
        Self {
            code: err.code().to_string(),
            category: err.category(),
            message: err.to_string(),
            details: None,
        }
    }
}

/// Result type alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;
```

---

## Testing Requirements

1. Error codes map to correct categories
2. Errors serialize correctly for API responses
3. Error messages are human-readable
4. Source errors are preserved in chain

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [013-result-utilities.md](013-result-utilities.md)
