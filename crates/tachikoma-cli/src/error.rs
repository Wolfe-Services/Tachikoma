//! CLI error handling and formatting.

use std::fmt;
use std::io;
use std::process::ExitCode;

use thiserror::Error;

pub mod formatter;
pub mod handler;

pub use formatter::ErrorFormatter;
pub use handler::{handle_result, setup_panic_handler, ErrorContext};

/// CLI error type with rich context
#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        hint: Option<String>,
    },

    #[error("{message}")]
    Io {
        message: String,
        #[source]
        source: io::Error,
        path: Option<std::path::PathBuf>,
    },

    #[error("{message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        url: Option<String>,
    },

    #[error("{message}")]
    Validation {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    #[error("{message}")]
    NotFound {
        message: String,
        resource_type: String,
        resource_name: String,
        suggestions: Vec<String>,
    },

    #[error("{message}")]
    Permission {
        message: String,
        path: Option<std::path::PathBuf>,
        required: Option<String>,
    },

    #[error("{message}")]
    Command {
        message: String,
        command: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    Backend {
        message: String,
        backend: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    Tool {
        message: String,
        tool: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("{message}")]
    User {
        message: String,
        hint: Option<String>,
    },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Get the error code for this error
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config { .. } => "E001",
            Self::Io { .. } => "E002",
            Self::Network { .. } => "E003",
            Self::Validation { .. } => "E004",
            Self::NotFound { .. } => "E005",
            Self::Permission { .. } => "E006",
            Self::Command { .. } => "E007",
            Self::Backend { .. } => "E008",
            Self::Tool { .. } => "E009",
            Self::User { .. } => "E010",
            Self::Other(_) => "E999",
        }
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> ExitCode {
        let code = match self {
            Self::Config { .. } => 2,
            Self::Io { .. } => 3,
            Self::Network { .. } => 4,
            Self::Validation { .. } => 5,
            Self::NotFound { .. } => 6,
            Self::Permission { .. } => 7,
            Self::Command { .. } => 8,
            Self::Backend { .. } => 9,
            Self::Tool { .. } => 10,
            Self::User { .. } => 1,
            Self::Other(_) => 1,
        };
        ExitCode::from(code)
    }

    /// Get hint for this error if available
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Config { hint, .. } => hint.as_deref(),
            Self::User { hint, .. } => hint.as_deref(),
            Self::NotFound { suggestions, .. } if !suggestions.is_empty() => {
                Some("See suggestions below")
            }
            _ => None,
        }
    }

    /// Get suggestions for this error
    pub fn suggestions(&self) -> &[String] {
        match self {
            Self::NotFound { suggestions, .. } => suggestions,
            _ => &[],
        }
    }

    /// Create a config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            hint: None,
        }
    }

    /// Create a config error with hint
    pub fn config_with_hint(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
            hint: Some(hint.into()),
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
            path: None,
        }
    }

    /// Create an IO error with path
    pub fn io_with_path(
        message: impl Into<String>,
        source: io::Error,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self::Io {
            message: message.into(),
            source,
            path: Some(path.into()),
        }
    }

    /// Create a not found error
    pub fn not_found(
        resource_type: impl Into<String>,
        resource_name: impl Into<String>,
    ) -> Self {
        let resource_type = resource_type.into();
        let resource_name = resource_name.into();
        Self::NotFound {
            message: format!("{resource_type} not found: {resource_name}"),
            resource_type,
            resource_name,
            suggestions: vec![],
        }
    }

    /// Create a not found error with suggestions
    pub fn not_found_with_suggestions(
        resource_type: impl Into<String>,
        resource_name: impl Into<String>,
        suggestions: Vec<String>,
    ) -> Self {
        let resource_type = resource_type.into();
        let resource_name = resource_name.into();
        Self::NotFound {
            message: format!("{resource_type} not found: {resource_name}"),
            resource_type,
            resource_name,
            suggestions,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            expected: None,
            actual: None,
        }
    }

    /// Create a user error (user did something wrong)
    pub fn user(message: impl Into<String>) -> Self {
        Self::User {
            message: message.into(),
            hint: None,
        }
    }

    /// Create a user error with hint
    pub fn user_with_hint(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::User {
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

// Conversion implementations
impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            source: err,
            path: None,
        }
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        Self::Config {
            message: format!("Invalid TOML: {err}"),
            source: Some(Box::new(err)),
            hint: Some("Check your configuration file syntax".to_string()),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        Self::Validation {
            message: format!("Invalid JSON: {err}"),
            field: None,
            expected: None,
            actual: None,
        }
    }
}

impl From<tachikoma_common_config::ConfigError> for CliError {
    fn from(err: tachikoma_common_config::ConfigError) -> Self {
        Self::Config {
            message: format!("Configuration error: {err}"),
            source: Some(Box::new(err)),
            hint: Some("Check your Tachikoma configuration file".to_string()),
        }
    }
}