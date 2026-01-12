# Spec 312: Server Configuration

## Phase
15 - Server/API Layer

## Spec ID
312

## Status
Planned

## Dependencies
- Spec 311: Server Setup

## Estimated Context
~8%

---

## Objective

Implement comprehensive configuration management for the Tachikoma server, supporting environment variables, configuration files, and runtime configuration with proper validation and defaults.

---

## Acceptance Criteria

- [ ] Configuration loads from environment variables with TACHIKOMA_ prefix
- [ ] Configuration loads from TOML/YAML config files
- [ ] Configuration supports multiple environments (dev, staging, prod)
- [ ] All configuration values have sensible defaults
- [ ] Configuration is validated at startup
- [ ] Sensitive values are redacted in logs
- [ ] Configuration can be reloaded at runtime for select values
- [ ] Configuration documentation is auto-generated

---

## Implementation Details

### Configuration Structure

```rust
// src/server/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Main server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Server binding configuration
    pub server: ServerBindConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// API configuration
    pub api: ApiConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerBindConfig {
    /// Host to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Number of worker threads (0 = auto)
    pub workers: usize,

    /// Request timeout in seconds
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,

    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keep_alive: Duration,

    /// Maximum concurrent connections
    pub max_connections: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Database URL
    #[serde(skip_serializing)]
    pub url: SecretString,

    /// Maximum pool size
    pub max_pool_size: u32,

    /// Minimum pool size
    pub min_pool_size: u32,

    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,

    /// Idle timeout for connections
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,

    /// Run migrations on startup
    pub run_migrations: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ApiConfig {
    /// Base path for API routes
    pub base_path: String,

    /// Maximum request body size in bytes
    pub max_body_size: usize,

    /// Default pagination limit
    pub default_page_size: usize,

    /// Maximum pagination limit
    pub max_page_size: usize,

    /// Enable CORS
    pub cors_enabled: bool,

    /// CORS allowed origins
    pub cors_origins: Vec<String>,

    /// Rate limiting enabled
    pub rate_limit_enabled: bool,

    /// Requests per minute per IP
    pub rate_limit_rpm: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// API key for authentication (optional)
    #[serde(skip_serializing)]
    pub api_key: Option<SecretString>,

    /// Enable TLS
    pub tls_enabled: bool,

    /// TLS certificate path
    pub tls_cert_path: Option<PathBuf>,

    /// TLS key path
    pub tls_key_path: Option<PathBuf>,

    /// Trusted proxy count for X-Forwarded-For
    pub trusted_proxies: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: LogFormat,

    /// Include request/response bodies in logs
    pub log_bodies: bool,

    /// Log file path (None = stdout only)
    pub file_path: Option<PathBuf>,

    /// Enable JSON structured logging
    pub json: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FeatureFlags {
    /// Enable WebSocket support
    pub websockets: bool,

    /// Enable SSE streaming
    pub sse: bool,

    /// Enable metrics endpoint
    pub metrics: bool,

    /// Enable health check endpoint
    pub health_check: bool,

    /// Enable API documentation
    pub api_docs: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Pretty,
    Compact,
    Json,
}

/// Wrapper for sensitive strings that redacts in debug output
#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
```

### Default Implementations

```rust
// src/server/config.rs (continued)

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerBindConfig::default(),
            database: DatabaseConfig::default(),
            api: ApiConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

impl Default for ServerBindConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            workers: 0, // auto-detect
            request_timeout: Duration::from_secs(30),
            keep_alive: Duration::from_secs(75),
            max_connections: 10000,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: SecretString::new("sqlite://tachikoma.db"),
            max_pool_size: 10,
            min_pool_size: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
            run_migrations: true,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_path: "/api".to_string(),
            max_body_size: 10 * 1024 * 1024, // 10MB
            default_page_size: 20,
            max_page_size: 100,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
            rate_limit_enabled: true,
            rate_limit_rpm: 100,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            trusted_proxies: 0,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Pretty,
            log_bodies: false,
            file_path: None,
            json: false,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            websockets: true,
            sse: true,
            metrics: true,
            health_check: true,
            api_docs: true,
        }
    }
}
```

### Configuration Loading

```rust
// src/server/config/loader.rs
use figment::{
    Figment,
    providers::{Env, Format, Toml, Serialized},
};
use std::path::Path;

use super::ServerConfig;

/// Configuration loader with multiple sources
pub struct ConfigLoader {
    figment: Figment,
}

impl ConfigLoader {
    /// Create a new config loader with defaults
    pub fn new() -> Self {
        Self {
            figment: Figment::new()
                .merge(Serialized::defaults(ServerConfig::default())),
        }
    }

    /// Add a TOML config file
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.figment = self.figment.merge(Toml::file(path.as_ref()));
        self
    }

    /// Add environment variables with TACHIKOMA_ prefix
    pub fn with_env(mut self) -> Self {
        self.figment = self.figment.merge(
            Env::prefixed("TACHIKOMA_")
                .split("__")
                .map(|key| key.as_str().replace("__", ".").into())
        );
        self
    }

    /// Load and validate configuration
    pub fn load(self) -> Result<ServerConfig, ConfigError> {
        let config: ServerConfig = self.figment.extract()?;
        config.validate()?;
        Ok(config)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    Load(#[from] figment::Error),

    #[error("Configuration validation failed: {0}")]
    Validation(String),
}

impl ServerConfig {
    /// Load configuration from default sources
    pub fn load() -> Result<Self, ConfigError> {
        ConfigLoader::new()
            .with_file("tachikoma.toml")
            .with_file("config/default.toml")
            .with_env()
            .load()
    }

    /// Load configuration for a specific environment
    pub fn load_for_env(env: &str) -> Result<Self, ConfigError> {
        ConfigLoader::new()
            .with_file("config/default.toml")
            .with_file(format!("config/{}.toml", env))
            .with_env()
            .load()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port
        if self.server.port == 0 {
            return Err(ConfigError::Validation(
                "Server port cannot be 0".to_string()
            ));
        }

        // Validate database URL
        if self.database.url.expose().is_empty() {
            return Err(ConfigError::Validation(
                "Database URL is required".to_string()
            ));
        }

        // Validate TLS configuration
        if self.security.tls_enabled {
            if self.security.tls_cert_path.is_none() {
                return Err(ConfigError::Validation(
                    "TLS certificate path required when TLS is enabled".to_string()
                ));
            }
            if self.security.tls_key_path.is_none() {
                return Err(ConfigError::Validation(
                    "TLS key path required when TLS is enabled".to_string()
                ));
            }
        }

        // Validate pagination
        if self.api.default_page_size > self.api.max_page_size {
            return Err(ConfigError::Validation(
                "Default page size cannot exceed max page size".to_string()
            ));
        }

        Ok(())
    }
}
```

### Environment-Specific Config Example

```toml
# config/default.toml
[server]
host = "127.0.0.1"
port = 3000
workers = 0
request_timeout = "30s"
keep_alive = "75s"

[database]
url = "sqlite://tachikoma.db"
max_pool_size = 10
run_migrations = true

[api]
base_path = "/api"
max_body_size = 10485760
cors_enabled = true
rate_limit_enabled = true
rate_limit_rpm = 100

[logging]
level = "info"
format = "pretty"
json = false

[features]
websockets = true
sse = true
metrics = true
health_check = true
```

```toml
# config/production.toml
[server]
host = "0.0.0.0"
workers = 4

[database]
max_pool_size = 50
run_migrations = false

[api]
cors_origins = ["https://app.tachikoma.io"]
rate_limit_rpm = 60

[logging]
level = "warn"
format = "json"
json = true

[security]
tls_enabled = true
trusted_proxies = 1
```

### Runtime Configuration Updates

```rust
// src/server/config/runtime.rs
use std::sync::Arc;
use tokio::sync::watch;

/// Runtime-configurable settings
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub log_level: LogLevel,
    pub rate_limit_rpm: u32,
    pub maintenance_mode: bool,
}

/// Handle for updating runtime configuration
pub struct RuntimeConfigHandle {
    sender: watch::Sender<RuntimeConfig>,
}

impl RuntimeConfigHandle {
    pub fn new(initial: RuntimeConfig) -> (Self, watch::Receiver<RuntimeConfig>) {
        let (sender, receiver) = watch::channel(initial);
        (Self { sender }, receiver)
    }

    pub fn update<F>(&self, f: F) -> Result<(), watch::error::SendError<RuntimeConfig>>
    where
        F: FnOnce(&mut RuntimeConfig),
    {
        self.sender.send_modify(f);
        Ok(())
    }

    pub fn set_log_level(&self, level: LogLevel) {
        self.sender.send_modify(|config| {
            config.log_level = level;
        });
    }

    pub fn set_maintenance_mode(&self, enabled: bool) {
        self.sender.send_modify(|config| {
            config.maintenance_mode = enabled;
        });
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config_is_valid() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("TACHIKOMA_SERVER__PORT", "8080");
        env::set_var("TACHIKOMA_API__CORS_ENABLED", "false");

        let config = ConfigLoader::new()
            .with_env()
            .load()
            .unwrap();

        assert_eq!(config.server.port, 8080);
        assert!(!config.api.cors_enabled);

        env::remove_var("TACHIKOMA_SERVER__PORT");
        env::remove_var("TACHIKOMA_API__CORS_ENABLED");
    }

    #[test]
    fn test_secret_string_redacted() {
        let secret = SecretString::new("super_secret_password");
        let debug = format!("{:?}", secret);

        assert!(!debug.contains("super_secret_password"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_validation_fails_for_invalid_config() {
        let mut config = ServerConfig::default();
        config.server.port = 0;

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tls_validation() {
        let mut config = ServerConfig::default();
        config.security.tls_enabled = true;

        assert!(config.validate().is_err());

        config.security.tls_cert_path = Some("/path/to/cert".into());
        config.security.tls_key_path = Some("/path/to/key".into());

        assert!(config.validate().is_ok());
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 313**: Route Definitions
- **Spec 334**: Distributed Tracing (logging config)
