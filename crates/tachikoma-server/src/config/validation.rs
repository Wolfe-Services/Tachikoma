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
    use crate::config::types::*;

    #[test]
    fn test_invalid_jwt_secret() {
        let mut config = test_config();
        config.auth.jwt_secret = "short".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidJwtSecret)));
    }

    #[test]
    fn test_invalid_database_url() {
        let mut config = test_config();
        config.database.url = "".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidDatabaseUrl)));
    }

    #[test]
    fn test_missing_tls_cert() {
        let mut config = test_config();
        config.server.tls_enabled = true;
        config.server.tls_cert_path = None;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::MissingTlsCert)));
    }

    #[test]
    fn test_invalid_port() {
        let mut config = test_config();
        config.server.port = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidPort(0))));
    }

    #[test]
    fn test_invalid_rate_limit() {
        let mut config = test_config();
        config.rate_limit.enabled = true;
        config.rate_limit.requests_per_window = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidRateLimit)));
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = test_config();
        config.logging.level = "invalid".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| matches!(e, ConfigError::InvalidLogLevel(_))));
    }

    #[test]
    fn test_valid_config() {
        let config = test_config();
        let result = validate_config(&config);
        assert!(result.is_ok());
    }

    fn test_config() -> ServerConfig {
        ServerConfig {
            server: ServerBindConfig {
                host: "localhost".to_string(),
                port: 8080,
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
                request_timeout_secs: 30,
                keepalive_secs: 75,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/test".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout_secs: 10,
                idle_timeout_secs: 600,
                log_queries: false,
            },
            auth: AuthConfig {
                jwt_secret: "a".repeat(32), // Valid 32-character secret
                access_token_expiry_secs: 3600,
                refresh_token_expiry_secs: 604800,
                enable_api_keys: true,
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                requests_per_window: 100,
                window_secs: 60,
                burst: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                log_requests: true,
                exclude_paths: vec!["/health".to_string()],
            },
            cors: CorsConfig {
                allowed_origins: vec![],
                allow_any_origin: false,
                allow_credentials: false,
                max_age_secs: 86400,
            },
            websocket: WebSocketConfig {
                enabled: true,
                ping_interval_secs: 30,
                max_message_size: 65536,
                require_auth: true,
            },
            cache: CacheConfig {
                enabled: false,
                redis_url: None,
                default_ttl_secs: 300,
            },
            features: FeatureFlags {
                metrics: true,
                health: true,
                openapi: false,
            },
        }
    }
}