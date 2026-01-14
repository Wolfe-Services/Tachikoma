//! Configuration loading utilities.

use super::types::ServerConfig;
use anyhow::{Context, Result};
use std::path::Path;
use tracing::info;

/// Load configuration from various sources.
pub struct ConfigLoader {
    config_path: Option<String>,
    env_prefix: String,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            config_path: None,
            env_prefix: "TACHIKOMA".to_string(),
        }
    }

    /// Set config file path.
    pub fn with_config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set environment variable prefix.
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Load configuration.
    pub fn load(&self) -> Result<ServerConfig> {
        let mut builder = config::Config::builder();

        // Add default values
        builder = builder.add_source(config::File::from_str(
            include_str!("defaults.toml"),
            config::FileFormat::Toml,
        ));

        // Add config file if specified
        if let Some(path) = &self.config_path {
            if Path::new(path).exists() {
                info!(path = %path, "Loading config file");
                builder = builder.add_source(config::File::with_name(path));
            }
        }

        // Add environment variables
        builder = builder.add_source(
            config::Environment::with_prefix(&self.env_prefix)
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .context("Failed to build configuration")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load configuration from environment.
pub fn load_config() -> Result<ServerConfig> {
    let config_path = std::env::var("CONFIG_PATH").ok();

    let mut loader = ConfigLoader::new();
    if let Some(path) = config_path {
        loader = loader.with_config_path(path);
    }

    loader.load()
}