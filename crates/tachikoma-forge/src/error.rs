//! Error types for Tachikoma Forge.

use thiserror::Error;

/// Result type for Forge operations.
pub type ForgeResult<T> = Result<T, ForgeError>;

/// Error types for Forge operations.
#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("Round error: {0}")]
    Round(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}