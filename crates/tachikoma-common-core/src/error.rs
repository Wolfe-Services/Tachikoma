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
    /// Configuration file not found.
    pub const CONFIG_NOT_FOUND: Self = Self(1001);
    /// Configuration file parsing error.
    pub const CONFIG_PARSE_ERROR: Self = Self(1002);
    /// Invalid configuration value.
    pub const CONFIG_INVALID_VALUE: Self = Self(1003);

    // File system errors (2xxx)
    /// File not found.
    pub const FILE_NOT_FOUND: Self = Self(2001);
    /// Error reading file.
    pub const FILE_READ_ERROR: Self = Self(2002);
    /// Error writing file.
    pub const FILE_WRITE_ERROR: Self = Self(2003);
    /// Invalid file path.
    pub const PATH_INVALID: Self = Self(2004);

    // Network errors (3xxx)
    /// Network operation timed out.
    pub const NETWORK_TIMEOUT: Self = Self(3001);
    /// Network connection failed.
    pub const NETWORK_CONNECTION: Self = Self(3002);
    /// API request failed.
    pub const API_ERROR: Self = Self(3003);
    /// Rate limited by API.
    pub const RATE_LIMITED: Self = Self(3004);

    // Backend errors (4xxx)
    /// Backend service unavailable.
    pub const BACKEND_UNAVAILABLE: Self = Self(4001);
    /// Backend authentication failed.
    pub const BACKEND_AUTH_FAILED: Self = Self(4002);
    /// Context window redlined.
    pub const CONTEXT_REDLINED: Self = Self(4003);
    /// Tool call execution failed.
    pub const TOOL_CALL_FAILED: Self = Self(4004);

    // Validation errors (5xxx)
    /// Input validation failed.
    pub const VALIDATION_FAILED: Self = Self(5001);
    /// Spec validation failed.
    pub const SPEC_INVALID: Self = Self(5002);
    /// ID parsing failed.
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
    /// Configuration error.
    #[error("configuration error: {message}")]
    Config {
        /// Error code.
        code: ErrorCode,
        /// Error message.
        message: String,
        /// Source error.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// File system error.
    #[error("file system error: {message}")]
    FileSystem {
        /// Error code.
        code: ErrorCode,
        /// Error message.
        message: String,
        /// File path that caused the error.
        path: Option<String>,
        /// Source error.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Network error.
    #[error("network error: {message}")]
    Network {
        /// Error code.
        code: ErrorCode,
        /// Error message.
        message: String,
        /// Source error.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Backend error.
    #[error("backend error: {message}")]
    Backend {
        /// Error code.
        code: ErrorCode,
        /// Error message.
        message: String,
        /// Backend name that caused the error.
        backend: Option<String>,
        /// Source error.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation error.
    #[error("validation error: {message}")]
    Validation {
        /// Error code.
        code: ErrorCode,
        /// Error message.
        message: String,
        /// Field that failed validation.
        field: Option<String>,
    },

    /// Internal error.
    #[error("internal error: {message}")]
    Internal {
        /// Error message.
        message: String,
        /// Source error.
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
    /// Error code string.
    pub code: String,
    /// Error category.
    pub category: ErrorCategory,
    /// Human-readable error message.
    pub message: String,
    /// Additional error details.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_categories() {
        // Test error codes map to correct categories
        assert_eq!(ErrorCode::CONFIG_NOT_FOUND.category(), ErrorCategory::Config);
        assert_eq!(ErrorCode::FILE_NOT_FOUND.category(), ErrorCategory::FileSystem);
        assert_eq!(ErrorCode::NETWORK_TIMEOUT.category(), ErrorCategory::Network);
        assert_eq!(ErrorCode::BACKEND_UNAVAILABLE.category(), ErrorCategory::Backend);
        assert_eq!(ErrorCode::VALIDATION_FAILED.category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::CONFIG_NOT_FOUND.to_string(), "E1001");
        assert_eq!(ErrorCode::FILE_NOT_FOUND.to_string(), "E2001");
        assert_eq!(ErrorCode::NETWORK_TIMEOUT.to_string(), "E3001");
    }

    #[test]
    fn test_error_creation() {
        // Test convenience constructors
        let config_err = Error::config("invalid setting");
        assert_eq!(config_err.code(), ErrorCode::CONFIG_INVALID_VALUE);
        assert_eq!(config_err.category(), ErrorCategory::Config);

        let file_err = Error::file_not_found("/path/to/file.txt");
        assert_eq!(file_err.code(), ErrorCode::FILE_NOT_FOUND);
        assert_eq!(file_err.category(), ErrorCategory::FileSystem);

        let validation_err = Error::validation("field is required");
        assert_eq!(validation_err.code(), ErrorCode::VALIDATION_FAILED);
        assert_eq!(validation_err.category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_error_messages() {
        // Test human-readable error messages
        let config_err = Error::config("missing API key");
        assert!(config_err.to_string().contains("configuration error"));
        assert!(config_err.to_string().contains("missing API key"));

        let file_err = Error::file_not_found("/missing/file.txt");
        assert!(file_err.to_string().contains("file system error"));
        assert!(file_err.to_string().contains("file not found: /missing/file.txt"));
    }

    #[test]
    fn test_error_chain() {
        // Test error context chain support
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        
        let fs_error = Error::FileSystem {
            code: ErrorCode::FILE_READ_ERROR,
            message: "failed to read config file".to_string(),
            path: Some("/etc/config.toml".to_string()),
            source: Some(Box::new(io_error)),
        };

        // Verify source error is preserved
        assert!(fs_error.source().is_some());
        assert!(fs_error.to_string().contains("failed to read config file"));
    }

    #[test]
    fn test_error_serialization() {
        // Test errors serialize correctly for API responses
        let config_err = Error::config("invalid configuration");
        let error_response = ErrorResponse::from(&config_err);

        // Test serialization
        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("config"));
        assert!(json.contains("E1003"));

        // Test round-trip
        let deserialized: ErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "E1003");
        assert_eq!(deserialized.category, ErrorCategory::Config);
    }

    #[test]
    fn test_error_category_serialization() {
        // Test error categories serialize correctly
        let categories = vec![
            ErrorCategory::Config,
            ErrorCategory::FileSystem,
            ErrorCategory::Network,
            ErrorCategory::Backend,
            ErrorCategory::Validation,
            ErrorCategory::Internal,
        ];

        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: ErrorCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(category, deserialized);
        }
    }

    #[test]
    fn test_all_error_variants() {
        // Test all error variants work correctly
        let errors = vec![
            Error::Config {
                code: ErrorCode::CONFIG_NOT_FOUND,
                message: "config not found".to_string(),
                source: None,
            },
            Error::FileSystem {
                code: ErrorCode::FILE_NOT_FOUND,
                message: "file missing".to_string(),
                path: Some("/path".to_string()),
                source: None,
            },
            Error::Network {
                code: ErrorCode::NETWORK_TIMEOUT,
                message: "timeout".to_string(),
                source: None,
            },
            Error::Backend {
                code: ErrorCode::BACKEND_UNAVAILABLE,
                message: "backend down".to_string(),
                backend: Some("openai".to_string()),
                source: None,
            },
            Error::Validation {
                code: ErrorCode::VALIDATION_FAILED,
                message: "invalid input".to_string(),
                field: Some("name".to_string()),
            },
            Error::Internal {
                message: "internal error".to_string(),
                source: None,
            },
        ];

        for err in errors {
            // Test error has valid code and category
            let code = err.code();
            let category = err.category();
            
            // Test error message is not empty
            assert!(!err.to_string().is_empty());
            
            // Test error response conversion
            let response = ErrorResponse::from(&err);
            assert_eq!(response.code, code.to_string());
            assert_eq!(response.category, category);
            assert!(!response.message.is_empty());
        }
    }

    #[test]
    fn test_internal_error_code() {
        let internal_err = Error::Internal {
            message: "something went wrong".to_string(),
            source: None,
        };
        
        assert_eq!(internal_err.code(), ErrorCode(9999));
        assert_eq!(internal_err.category(), ErrorCategory::Internal);
    }
}
