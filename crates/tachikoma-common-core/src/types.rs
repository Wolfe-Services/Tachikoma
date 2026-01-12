//! Common types and utilities for Tachikoma.

use serde::{Deserialize, Serialize};

/// Configuration for a Tachikoma instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TachikomaConfig {
    /// Instance name.
    pub name: String,
    /// Version information.
    pub version: String,
    /// Debug mode flag.
    pub debug: bool,
}

impl Default for TachikomaConfig {
    fn default() -> Self {
        Self {
            name: "tachikoma".to_string(),
            version: "0.1.0".to_string(),
            debug: false,
        }
    }
}

/// Common result type for Tachikoma operations.
pub type TachikomaResult<T> = crate::error::Result<T>;
