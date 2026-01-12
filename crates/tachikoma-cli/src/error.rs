//! CLI-specific error types.

use thiserror::Error;

use crate::Exit;

/// CLI error type
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(#[from] tachikoma_common_config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl CliError {
    /// Get the appropriate exit code for this error
    pub fn exit_code(&self) -> crate::Exit {
        match self {
            Self::Config(_) => Exit::ConfigError,
            Self::Io(_) => Exit::IoError,
            Self::Network(_) => Exit::NetworkError,
            Self::Validation(_) => Exit::ValidationError,
            _ => Exit::GeneralError,
        }
    }
}