# Spec 367: Authentication Configuration

## Phase
17 - Authentication/Authorization

## Spec ID
367

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits

## Estimated Context
~8%

---

## Objective

Define comprehensive configuration structures for the authentication system. This includes settings for password policies, token lifetimes, session management, OAuth2 providers, rate limiting, and security policies.

---

## Acceptance Criteria

- [ ] Define `AuthConfig` as the root configuration structure
- [ ] Create `PasswordConfig` for password policy settings
- [ ] Create `TokenConfig` for JWT and token settings
- [ ] Create `SessionConfig` for session management settings
- [ ] Create `OAuth2ProviderConfig` for OAuth2 integration
- [ ] Create `SecurityConfig` for security policies
- [ ] Support environment variable overrides
- [ ] Implement configuration validation
- [ ] Provide sensible defaults for all settings

---

## Implementation Details

### Configuration Structures

```rust
// src/auth/config.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Root authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Whether authentication is enabled
    pub enabled: bool,

    /// Password authentication configuration
    pub password: PasswordConfig,

    /// JWT token configuration
    pub tokens: TokenConfig,

    /// Session configuration
    pub session: SessionConfig,

    /// OAuth2 provider configurations
    pub oauth2: OAuth2Config,

    /// API key configuration
    pub api_keys: ApiKeyConfig,

    /// Multi-factor authentication configuration
    pub mfa: MfaConfig,

    /// Security policies
    pub security: SecurityConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Account lockout configuration
    pub lockout: LockoutConfig,

    /// Audit logging configuration
    pub audit: AuditConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            password: PasswordConfig::default(),
            tokens: TokenConfig::default(),
            session: SessionConfig::default(),
            oauth2: OAuth2Config::default(),
            api_keys: ApiKeyConfig::default(),
            mfa: MfaConfig::default(),
            security: SecurityConfig::default(),
            rate_limit: RateLimitConfig::default(),
            lockout: LockoutConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

impl AuthConfig {
    /// Load configuration from environment with prefix
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_env_with_prefix("TACHIKOMA_AUTH")
    }

    /// Load configuration from environment with custom prefix
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load top-level settings
        if let Ok(val) = std::env::var(format!("{}_ENABLED", prefix)) {
            config.enabled = val.parse().unwrap_or(true);
        }

        // Load password config
        config.password = PasswordConfig::from_env_with_prefix(&format!("{}_PASSWORD", prefix))?;

        // Load token config
        config.tokens = TokenConfig::from_env_with_prefix(&format!("{}_TOKEN", prefix))?;

        // Load session config
        config.session = SessionConfig::from_env_with_prefix(&format!("{}_SESSION", prefix))?;

        // Load security config
        config.security = SecurityConfig::from_env_with_prefix(&format!("{}_SECURITY", prefix))?;

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.password.validate()?;
        self.tokens.validate()?;
        self.session.validate()?;
        self.security.validate()?;
        self.rate_limit.validate()?;
        self.lockout.validate()?;
        Ok(())
    }
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordConfig {
    /// Minimum password length
    pub min_length: usize,

    /// Maximum password length
    pub max_length: usize,

    /// Require uppercase letters
    pub require_uppercase: bool,

    /// Require lowercase letters
    pub require_lowercase: bool,

    /// Require digits
    pub require_digit: bool,

    /// Require special characters
    pub require_special: bool,

    /// Special characters that are allowed/required
    pub special_chars: String,

    /// Number of previous passwords to check against
    pub history_count: usize,

    /// Password expiration in days (0 = never expires)
    pub expiration_days: u32,

    /// Days before expiration to warn user
    pub expiration_warning_days: u32,

    /// Argon2 memory cost in KB
    pub argon2_memory_kb: u32,

    /// Argon2 time cost (iterations)
    pub argon2_time_cost: u32,

    /// Argon2 parallelism factor
    pub argon2_parallelism: u32,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            special_chars: "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string(),
            history_count: 5,
            expiration_days: 90,
            expiration_warning_days: 14,
            argon2_memory_kb: 65536, // 64 MB
            argon2_time_cost: 3,
            argon2_parallelism: 4,
        }
    }
}

impl PasswordConfig {
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(val) = std::env::var(format!("{}_MIN_LENGTH", prefix)) {
            config.min_length = val.parse().map_err(|_| {
                ConfigError::InvalidValue("min_length".to_string())
            })?;
        }

        if let Ok(val) = std::env::var(format!("{}_MAX_LENGTH", prefix)) {
            config.max_length = val.parse().map_err(|_| {
                ConfigError::InvalidValue("max_length".to_string())
            })?;
        }

        if let Ok(val) = std::env::var(format!("{}_REQUIRE_UPPERCASE", prefix)) {
            config.require_uppercase = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_REQUIRE_LOWERCASE", prefix)) {
            config.require_lowercase = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_REQUIRE_DIGIT", prefix)) {
            config.require_digit = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_REQUIRE_SPECIAL", prefix)) {
            config.require_special = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_ARGON2_MEMORY_KB", prefix)) {
            config.argon2_memory_kb = val.parse().map_err(|_| {
                ConfigError::InvalidValue("argon2_memory_kb".to_string())
            })?;
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_length < 8 {
            return Err(ConfigError::InvalidValue(
                "min_length must be at least 8".to_string(),
            ));
        }
        if self.max_length < self.min_length {
            return Err(ConfigError::InvalidValue(
                "max_length must be >= min_length".to_string(),
            ));
        }
        if self.argon2_memory_kb < 1024 {
            return Err(ConfigError::InvalidValue(
                "argon2_memory_kb must be at least 1024".to_string(),
            ));
        }
        Ok(())
    }
}

/// JWT and token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TokenConfig {
    /// Secret key for signing tokens (should be loaded from secure source)
    #[serde(skip_serializing)]
    pub secret_key: String,

    /// Token issuer (iss claim)
    pub issuer: String,

    /// Token audience (aud claim)
    pub audience: Vec<String>,

    /// Access token lifetime in seconds
    pub access_token_lifetime_secs: u64,

    /// Refresh token lifetime in seconds
    pub refresh_token_lifetime_secs: u64,

    /// Algorithm for signing (HS256, HS384, HS512, RS256, etc.)
    pub algorithm: String,

    /// RSA public key path (for RS* algorithms)
    pub public_key_path: Option<String>,

    /// RSA private key path (for RS* algorithms)
    pub private_key_path: Option<String>,

    /// Whether to include user roles in token
    pub include_roles: bool,

    /// Whether to include user permissions in token
    pub include_permissions: bool,

    /// Maximum number of active refresh tokens per user
    pub max_refresh_tokens_per_user: usize,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            secret_key: String::new(), // Must be set
            issuer: "tachikoma".to_string(),
            audience: vec!["tachikoma-api".to_string()],
            access_token_lifetime_secs: 900, // 15 minutes
            refresh_token_lifetime_secs: 604800, // 7 days
            algorithm: "HS256".to_string(),
            public_key_path: None,
            private_key_path: None,
            include_roles: true,
            include_permissions: false,
            max_refresh_tokens_per_user: 5,
        }
    }
}

impl TokenConfig {
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(val) = std::env::var(format!("{}_SECRET_KEY", prefix)) {
            config.secret_key = val;
        }

        if let Ok(val) = std::env::var(format!("{}_ISSUER", prefix)) {
            config.issuer = val;
        }

        if let Ok(val) = std::env::var(format!("{}_ACCESS_LIFETIME", prefix)) {
            config.access_token_lifetime_secs = val.parse().map_err(|_| {
                ConfigError::InvalidValue("access_token_lifetime".to_string())
            })?;
        }

        if let Ok(val) = std::env::var(format!("{}_REFRESH_LIFETIME", prefix)) {
            config.refresh_token_lifetime_secs = val.parse().map_err(|_| {
                ConfigError::InvalidValue("refresh_token_lifetime".to_string())
            })?;
        }

        if let Ok(val) = std::env::var(format!("{}_ALGORITHM", prefix)) {
            config.algorithm = val;
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.secret_key.is_empty() && !self.algorithm.starts_with("RS") {
            return Err(ConfigError::MissingRequired("secret_key".to_string()));
        }

        let valid_algorithms = ["HS256", "HS384", "HS512", "RS256", "RS384", "RS512"];
        if !valid_algorithms.contains(&self.algorithm.as_str()) {
            return Err(ConfigError::InvalidValue(format!(
                "algorithm must be one of: {:?}",
                valid_algorithms
            )));
        }

        if self.access_token_lifetime_secs < 60 {
            return Err(ConfigError::InvalidValue(
                "access_token_lifetime must be at least 60 seconds".to_string(),
            ));
        }

        Ok(())
    }

    pub fn access_token_duration(&self) -> Duration {
        Duration::from_secs(self.access_token_lifetime_secs)
    }

    pub fn refresh_token_duration(&self) -> Duration {
        Duration::from_secs(self.refresh_token_lifetime_secs)
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Whether session-based auth is enabled
    pub enabled: bool,

    /// Session storage backend
    pub storage: SessionStorage,

    /// Session lifetime in seconds
    pub lifetime_secs: u64,

    /// Idle timeout in seconds (0 = no idle timeout)
    pub idle_timeout_secs: u64,

    /// Whether to regenerate session ID on login
    pub regenerate_on_login: bool,

    /// Cookie name for session ID
    pub cookie_name: String,

    /// Cookie domain
    pub cookie_domain: Option<String>,

    /// Cookie path
    pub cookie_path: String,

    /// Whether cookie is secure (HTTPS only)
    pub cookie_secure: bool,

    /// Whether cookie is HTTP only
    pub cookie_http_only: bool,

    /// Cookie same-site policy
    pub cookie_same_site: SameSite,

    /// Maximum concurrent sessions per user (0 = unlimited)
    pub max_sessions_per_user: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage: SessionStorage::Memory,
            lifetime_secs: 86400, // 24 hours
            idle_timeout_secs: 1800, // 30 minutes
            regenerate_on_login: true,
            cookie_name: "tachikoma_session".to_string(),
            cookie_domain: None,
            cookie_path: "/".to_string(),
            cookie_secure: true,
            cookie_http_only: true,
            cookie_same_site: SameSite::Strict,
            max_sessions_per_user: 5,
        }
    }
}

impl SessionConfig {
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(val) = std::env::var(format!("{}_ENABLED", prefix)) {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_LIFETIME", prefix)) {
            config.lifetime_secs = val.parse().map_err(|_| {
                ConfigError::InvalidValue("lifetime".to_string())
            })?;
        }

        if let Ok(val) = std::env::var(format!("{}_COOKIE_NAME", prefix)) {
            config.cookie_name = val;
        }

        if let Ok(val) = std::env::var(format!("{}_COOKIE_SECURE", prefix)) {
            config.cookie_secure = val.parse().unwrap_or(true);
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.lifetime_secs < 300 {
            return Err(ConfigError::InvalidValue(
                "session lifetime must be at least 300 seconds".to_string(),
            ));
        }
        Ok(())
    }
}

/// Session storage backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStorage {
    Memory,
    Redis { url: String },
    Database,
}

/// Cookie same-site policy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OAuth2Config {
    /// Whether OAuth2 is enabled
    pub enabled: bool,

    /// Configured OAuth2 providers
    pub providers: HashMap<String, OAuth2ProviderConfig>,
}

/// OAuth2 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ProviderConfig {
    /// Provider name (github, google, etc.)
    pub name: String,

    /// Client ID
    pub client_id: String,

    /// Client secret
    #[serde(skip_serializing)]
    pub client_secret: String,

    /// Authorization endpoint URL
    pub auth_url: String,

    /// Token endpoint URL
    pub token_url: String,

    /// User info endpoint URL (optional)
    pub userinfo_url: Option<String>,

    /// Requested scopes
    pub scopes: Vec<String>,

    /// Redirect URI
    pub redirect_uri: String,

    /// Whether to auto-create users on first OAuth login
    pub auto_create_users: bool,

    /// Default roles for auto-created users
    pub default_roles: Vec<String>,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiKeyConfig {
    /// Whether API key auth is enabled
    pub enabled: bool,

    /// Header name for API key
    pub header_name: String,

    /// Query parameter name for API key (if allowed)
    pub query_param_name: Option<String>,

    /// Prefix required for API keys (e.g., "tk_")
    pub key_prefix: String,

    /// Key length (excluding prefix)
    pub key_length: usize,

    /// Maximum API keys per user
    pub max_keys_per_user: usize,

    /// Default expiration in days (0 = never expires)
    pub default_expiration_days: u32,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header_name: "X-API-Key".to_string(),
            query_param_name: None, // Disabled by default for security
            key_prefix: "tk_".to_string(),
            key_length: 32,
            max_keys_per_user: 10,
            default_expiration_days: 365,
        }
    }
}

/// MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MfaConfig {
    /// Whether MFA is enabled
    pub enabled: bool,

    /// Whether MFA is required for all users
    pub required: bool,

    /// Allowed MFA methods
    pub allowed_methods: Vec<MfaMethod>,

    /// TOTP issuer name
    pub totp_issuer: String,

    /// TOTP digit count (6 or 8)
    pub totp_digits: u32,

    /// TOTP time step in seconds
    pub totp_step_secs: u64,

    /// Number of backup codes to generate
    pub backup_code_count: usize,

    /// Backup code length
    pub backup_code_length: usize,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            required: false,
            allowed_methods: vec![MfaMethod::Totp, MfaMethod::BackupCodes],
            totp_issuer: "Tachikoma".to_string(),
            totp_digits: 6,
            totp_step_secs: 30,
            backup_code_count: 10,
            backup_code_length: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    Sms,
    Email,
    SecurityKey,
    BackupCodes,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Require HTTPS for all auth endpoints
    pub require_https: bool,

    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,

    /// Whether to log authentication events
    pub log_auth_events: bool,

    /// Whether to mask sensitive data in logs
    pub mask_sensitive_data: bool,

    /// IP addresses that are always allowed
    pub ip_whitelist: Vec<String>,

    /// IP addresses that are always blocked
    pub ip_blacklist: Vec<String>,

    /// Minimum TLS version
    pub min_tls_version: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_https: true,
            allowed_origins: vec![],
            log_auth_events: true,
            mask_sensitive_data: true,
            ip_whitelist: vec![],
            ip_blacklist: vec![],
            min_tls_version: "1.2".to_string(),
        }
    }
}

impl SecurityConfig {
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self, ConfigError> {
        let mut config = Self::default();

        if let Ok(val) = std::env::var(format!("{}_REQUIRE_HTTPS", prefix)) {
            config.require_https = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var(format!("{}_LOG_AUTH_EVENTS", prefix)) {
            config.log_auth_events = val.parse().unwrap_or(true);
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let valid_tls_versions = ["1.2", "1.3"];
        if !valid_tls_versions.contains(&self.min_tls_version.as_str()) {
            return Err(ConfigError::InvalidValue(
                "min_tls_version must be 1.2 or 1.3".to_string(),
            ));
        }
        Ok(())
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    pub enabled: bool,

    /// Login attempts per window
    pub login_attempts: u32,

    /// Window size in seconds
    pub window_secs: u64,

    /// Token refresh attempts per window
    pub refresh_attempts: u32,

    /// Password reset attempts per window
    pub password_reset_attempts: u32,

    /// MFA attempts per window
    pub mfa_attempts: u32,

    /// API requests per window (for API key auth)
    pub api_requests: u32,

    /// API window size in seconds
    pub api_window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            login_attempts: 5,
            window_secs: 900, // 15 minutes
            refresh_attempts: 10,
            password_reset_attempts: 3,
            mfa_attempts: 5,
            api_requests: 1000,
            api_window_secs: 60,
        }
    }
}

impl RateLimitConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.login_attempts == 0 {
            return Err(ConfigError::InvalidValue(
                "login_attempts must be > 0".to_string(),
            ));
        }
        if self.window_secs < 60 {
            return Err(ConfigError::InvalidValue(
                "window_secs must be at least 60".to_string(),
            ));
        }
        Ok(())
    }
}

/// Account lockout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LockoutConfig {
    /// Whether account lockout is enabled
    pub enabled: bool,

    /// Number of failed attempts before lockout
    pub max_failed_attempts: u32,

    /// Lockout duration in seconds
    pub lockout_duration_secs: u64,

    /// Whether to reset failed count on successful login
    pub reset_on_success: bool,

    /// Whether to progressively increase lockout duration
    pub progressive_lockout: bool,

    /// Maximum lockout duration (for progressive lockout)
    pub max_lockout_duration_secs: u64,
}

impl Default for LockoutConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_failed_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
            reset_on_success: true,
            progressive_lockout: true,
            max_lockout_duration_secs: 86400, // 24 hours
        }
    }
}

impl LockoutConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.max_failed_attempts == 0 {
            return Err(ConfigError::InvalidValue(
                "max_failed_attempts must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    pub enabled: bool,

    /// Events to log
    pub log_events: Vec<AuditEventType>,

    /// Retention period in days
    pub retention_days: u32,

    /// Whether to include IP addresses
    pub include_ip: bool,

    /// Whether to include user agent
    pub include_user_agent: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_events: vec![
                AuditEventType::LoginSuccess,
                AuditEventType::LoginFailure,
                AuditEventType::Logout,
                AuditEventType::PasswordChange,
                AuditEventType::MfaEnabled,
                AuditEventType::MfaDisabled,
                AuditEventType::AccountLocked,
                AuditEventType::AccountUnlocked,
            ],
            retention_days: 90,
            include_ip: true,
            include_user_agent: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailure,
    Logout,
    PasswordChange,
    PasswordReset,
    MfaEnabled,
    MfaDisabled,
    MfaSuccess,
    MfaFailure,
    AccountLocked,
    AccountUnlocked,
    TokenRefresh,
    ApiKeyCreated,
    ApiKeyRevoked,
    SessionCreated,
    SessionDestroyed,
}

/// Configuration errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.password.min_length, 12);
        assert_eq!(config.tokens.access_token_lifetime_secs, 900);
    }

    #[test]
    fn test_password_config_validation() {
        let mut config = PasswordConfig::default();
        assert!(config.validate().is_ok());

        config.min_length = 4;
        assert!(config.validate().is_err());

        config.min_length = 12;
        config.max_length = 8;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_token_config_validation() {
        let mut config = TokenConfig::default();
        config.secret_key = "test-secret-key".to_string();
        assert!(config.validate().is_ok());

        config.algorithm = "INVALID".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_token_durations() {
        let config = TokenConfig::default();
        assert_eq!(config.access_token_duration(), Duration::from_secs(900));
        assert_eq!(config.refresh_token_duration(), Duration::from_secs(604800));
    }

    #[test]
    fn test_session_config_validation() {
        let mut config = SessionConfig::default();
        assert!(config.validate().is_ok());

        config.lifetime_secs = 60;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rate_limit_config_validation() {
        let mut config = RateLimitConfig::default();
        assert!(config.validate().is_ok());

        config.login_attempts = 0;
        assert!(config.validate().is_err());
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Configuration applies to all auth types
- **Spec 370**: JWT Tokens - Uses TokenConfig
- **Spec 369**: Session Management - Uses SessionConfig
- **Spec 377**: OAuth2 Support - Uses OAuth2Config
- **Spec 378**: MFA - Uses MfaConfig
- **Spec 382**: Rate Limiting - Uses RateLimitConfig
- **Spec 383**: Account Lockout - Uses LockoutConfig
