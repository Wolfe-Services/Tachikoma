# 332 - Server Configuration

**Phase:** 15 - Server
**Spec ID:** 332
**Status:** Planned
**Dependencies:** 316-server-crate
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement server configuration management with environment variables, config files, validation, and hot reloading support.

---

## Acceptance Criteria

- [x] Configuration struct definition
- [x] Environment variable loading
- [x] Config file support (TOML)
- [x] Configuration validation
- [x] Default values
- [x] Hot reload support
- [x] Config documentation

---

## Implementation Details

### 1. Config Types (crates/tachikoma-server/src/config/types.rs)

```rust
//! Server configuration types.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// Main server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server binding configuration.
    pub server: ServerBindConfig,
    /// Database configuration.
    pub database: DatabaseConfig,
    /// Authentication configuration.
    pub auth: AuthConfig,
    /// Rate limiting configuration.
    pub rate_limit: RateLimitConfig,
    /// Logging configuration.
    pub logging: LoggingConfig,
    /// CORS configuration.
    pub cors: CorsConfig,
    /// WebSocket configuration.
    pub websocket: WebSocketConfig,
    /// Cache configuration.
    #[serde(default)]
    pub cache: CacheConfig,
    /// Feature flags.
    #[serde(default)]
    pub features: FeatureFlags,
}

/// Server binding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerBindConfig {
    /// Host to bind to.
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to bind to.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Enable TLS.
    #[serde(default)]
    pub tls_enabled: bool,
    /// TLS certificate path.
    pub tls_cert_path: Option<PathBuf>,
    /// TLS key path.
    pub tls_key_path: Option<PathBuf>,
    /// Request timeout.
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    /// Keep-alive timeout.
    #[serde(default = "default_keepalive")]
    pub keepalive_secs: u64,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_request_timeout() -> u64 {
    30
}

fn default_keepalive() -> u64 {
    75
}

impl ServerBindConfig {
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
}

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL.
    pub url: String,
    /// Maximum connections in pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Minimum connections in pool.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// Connection timeout.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
    /// Idle timeout.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    /// Enable query logging.
    #[serde(default)]
    pub log_queries: bool,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_idle_timeout() -> u64 {
    600
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key.
    pub jwt_secret: String,
    /// Access token expiry (seconds).
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry_secs: u64,
    /// Refresh token expiry (seconds).
    #[serde(default = "default_refresh_token_expiry")]
    pub refresh_token_expiry_secs: u64,
    /// Enable API key authentication.
    #[serde(default = "default_true")]
    pub enable_api_keys: bool,
}

fn default_access_token_expiry() -> u64 {
    3600 // 1 hour
}

fn default_refresh_token_expiry() -> u64 {
    604800 // 7 days
}

fn default_true() -> bool {
    true
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Requests per window.
    #[serde(default = "default_rate_limit")]
    pub requests_per_window: u32,
    /// Window size in seconds.
    #[serde(default = "default_rate_window")]
    pub window_secs: u64,
    /// Burst allowance.
    #[serde(default = "default_burst")]
    pub burst: u32,
}

fn default_rate_limit() -> u32 {
    100
}

fn default_rate_window() -> u64 {
    60
}

fn default_burst() -> u32 {
    10
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level.
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log format (json or pretty).
    #[serde(default = "default_log_format")]
    pub format: String,
    /// Enable request logging.
    #[serde(default = "default_true")]
    pub log_requests: bool,
    /// Paths to exclude from logging.
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

/// CORS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Allow any origin.
    #[serde(default)]
    pub allow_any_origin: bool,
    /// Allow credentials.
    #[serde(default)]
    pub allow_credentials: bool,
    /// Max age for preflight cache.
    #[serde(default = "default_cors_max_age")]
    pub max_age_secs: u64,
}

fn default_cors_max_age() -> u64 {
    86400
}

/// WebSocket configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Enable WebSocket support.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Ping interval.
    #[serde(default = "default_ping_interval")]
    pub ping_interval_secs: u64,
    /// Maximum message size.
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,
    /// Require authentication.
    #[serde(default = "default_true")]
    pub require_auth: bool,
}

fn default_ping_interval() -> u64 {
    30
}

fn default_max_message_size() -> usize {
    65536
}

/// Cache configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching.
    #[serde(default)]
    pub enabled: bool,
    /// Redis URL (if using Redis).
    pub redis_url: Option<String>,
    /// Default TTL.
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_secs: u64,
}

fn default_cache_ttl() -> u64 {
    300
}

/// Feature flags.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable metrics endpoint.
    #[serde(default = "default_true")]
    pub metrics: bool,
    /// Enable health endpoints.
    #[serde(default = "default_true")]
    pub health: bool,
    /// Enable OpenAPI documentation.
    #[serde(default)]
    pub openapi: bool,
}
```

### 2. Config Loader (crates/tachikoma-server/src/config/loader.rs)

```rust
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
```

### 3. Config Validation (crates/tachikoma-server/src/config/validation.rs)

```rust
//! Configuration validation.

use super::types::ServerConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid JWT secret: must be at least 32 characters")]
    InvalidJwtSecret,

    #[error("Invalid database URL")]
    InvalidDatabaseUrl,

    #[error("TLS enabled but certificate path not provided")]
    MissingTlsCert,

    #[error("TLS enabled but key path not provided")]
    MissingTlsKey,

    #[error("Invalid port: {0}")]
    InvalidPort(u16),

    #[error("Invalid rate limit configuration")]
    InvalidRateLimit,

    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),
}

/// Validate server configuration.
pub fn validate_config(config: &ServerConfig) -> Result<(), Vec<ConfigError>> {
    let mut errors = Vec::new();

    // Validate JWT secret
    if config.auth.jwt_secret.len() < 32 {
        errors.push(ConfigError::InvalidJwtSecret);
    }

    // Validate database URL
    if config.database.url.is_empty() {
        errors.push(ConfigError::InvalidDatabaseUrl);
    }

    // Validate TLS configuration
    if config.server.tls_enabled {
        if config.server.tls_cert_path.is_none() {
            errors.push(ConfigError::MissingTlsCert);
        }
        if config.server.tls_key_path.is_none() {
            errors.push(ConfigError::MissingTlsKey);
        }
    }

    // Validate port
    if config.server.port == 0 {
        errors.push(ConfigError::InvalidPort(0));
    }

    // Validate rate limit
    if config.rate_limit.enabled && config.rate_limit.requests_per_window == 0 {
        errors.push(ConfigError::InvalidRateLimit);
    }

    // Validate log level
    let valid_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_levels.contains(&config.logging.level.to_lowercase().as_str()) {
        errors.push(ConfigError::InvalidLogLevel(config.logging.level.clone()));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_jwt_secret() {
        let mut config = test_config();
        config.auth.jwt_secret = "short".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidJwtSecret)));
    }

    fn test_config() -> ServerConfig {
        // Return a valid test configuration
        todo!()
    }
}
```

### 4. Default Config (crates/tachikoma-server/src/config/defaults.toml)

```toml
# Default server configuration

[server]
host = "0.0.0.0"
port = 8080
tls_enabled = false
request_timeout_secs = 30
keepalive_secs = 75

[database]
max_connections = 10
min_connections = 1
connect_timeout_secs = 10
idle_timeout_secs = 600
log_queries = false

[auth]
access_token_expiry_secs = 3600
refresh_token_expiry_secs = 604800
enable_api_keys = true

[rate_limit]
enabled = true
requests_per_window = 100
window_secs = 60
burst = 10

[logging]
level = "info"
format = "pretty"
log_requests = true
exclude_paths = ["/health", "/metrics"]

[cors]
allowed_origins = []
allow_any_origin = false
allow_credentials = false
max_age_secs = 86400

[websocket]
enabled = true
ping_interval_secs = 30
max_message_size = 65536
require_auth = true

[cache]
enabled = false
default_ttl_secs = 300

[features]
metrics = true
health = true
openapi = false
```

---

## Testing Requirements

1. Default values applied correctly
2. Environment variables override config
3. Config file loads properly
4. Validation catches invalid configs
5. Sensitive values not logged
6. Hot reload works (if implemented)
7. Missing optional values handled

---

## Related Specs

- Depends on: [316-server-crate.md](316-server-crate.md)
- Next: [333-server-startup.md](333-server-startup.md)
- Used by: Server initialization
