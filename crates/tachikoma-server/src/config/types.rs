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