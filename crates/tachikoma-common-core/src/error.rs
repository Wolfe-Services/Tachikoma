//! Error types for Tachikoma.

use thiserror::Error;

/// The main error type for Tachikoma operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Generic error with custom message.
    #[error("{0}")]
    Generic(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl Error {
    /// Create a new generic error.
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Create a new configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}

/// Result type alias using Tachikoma's Error.
pub type Result<T> = std::result::Result<T, Error>;
