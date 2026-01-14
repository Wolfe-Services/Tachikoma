# Spec 377: Magic Link Types

## Overview
Define types for passwordless magic link authentication.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Magic Link Types
```rust
// src/auth/magic_link/types.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MagicLinkError {
    #[error("Magic link expired")]
    Expired,

    #[error("Magic link not found")]
    NotFound,

    #[error("Magic link already used")]
    AlreadyUsed,

    #[error("Invalid magic link")]
    Invalid,

    #[error("Too many requests")]
    RateLimited,

    #[error("Email sending failed: {0}")]
    EmailFailed(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Magic link token stored in database
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct MagicLinkToken {
    pub id: String,
    pub token_hash: String,
    pub email: String,
    pub user_id: Option<String>,  // None for signup links
    pub purpose: String,  // MagicLinkPurpose as string
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl MagicLinkToken {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() &&
        self.revoked_at.is_none() &&
        Utc::now() < self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn purpose(&self) -> MagicLinkPurpose {
        MagicLinkPurpose::from_str(&self.purpose)
    }
}

/// Purpose of magic link
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MagicLinkPurpose {
    /// Login to existing account
    Login,
    /// Sign up for new account
    Signup,
    /// Verify email address
    EmailVerification,
    /// Reset password (if password auth is also enabled)
    PasswordReset,
    /// Confirm account deletion
    AccountDeletion,
    /// Link additional email
    EmailLink,
}

impl MagicLinkPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Signup => "signup",
            Self::EmailVerification => "email_verification",
            Self::PasswordReset => "password_reset",
            Self::AccountDeletion => "account_deletion",
            Self::EmailLink => "email_link",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "login" => Self::Login,
            "signup" => Self::Signup,
            "email_verification" => Self::EmailVerification,
            "password_reset" => Self::PasswordReset,
            "account_deletion" => Self::AccountDeletion,
            "email_link" => Self::EmailLink,
            _ => Self::Login,
        }
    }

    /// Get default lifetime for this purpose
    pub fn default_lifetime(&self) -> Duration {
        match self {
            Self::Login => Duration::minutes(15),
            Self::Signup => Duration::hours(24),
            Self::EmailVerification => Duration::hours(24),
            Self::PasswordReset => Duration::hours(1),
            Self::AccountDeletion => Duration::hours(1),
            Self::EmailLink => Duration::hours(24),
        }
    }
}

impl Default for MagicLinkPurpose {
    fn default() -> Self {
        Self::Login
    }
}

/// Magic link configuration
#[derive(Debug, Clone)]
pub struct MagicLinkConfig {
    /// Base URL for magic links
    pub base_url: String,
    /// Path for magic link verification
    pub verify_path: String,
    /// Token length in bytes
    pub token_length: usize,
    /// Default token lifetime
    pub default_lifetime: Duration,
    /// Allow signup via magic link
    pub allow_signup: bool,
    /// Rate limit per email (requests per hour)
    pub rate_limit_per_email: u32,
    /// Rate limit per IP (requests per hour)
    pub rate_limit_per_ip: u32,
    /// Custom lifetimes per purpose
    pub lifetimes: Option<std::collections::HashMap<MagicLinkPurpose, Duration>>,
    /// Email sender name
    pub sender_name: String,
    /// Email sender address
    pub sender_email: String,
}

impl Default for MagicLinkConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            verify_path: "/auth/magic-link/verify".to_string(),
            token_length: 32,
            default_lifetime: Duration::minutes(15),
            allow_signup: true,
            rate_limit_per_email: 5,
            rate_limit_per_ip: 20,
            lifetimes: None,
            sender_name: "Tachikoma".to_string(),
            sender_email: "noreply@tachikoma.local".to_string(),
        }
    }
}

impl MagicLinkConfig {
    /// Get lifetime for a specific purpose
    pub fn lifetime_for(&self, purpose: MagicLinkPurpose) -> Duration {
        self.lifetimes
            .as_ref()
            .and_then(|m| m.get(&purpose))
            .copied()
            .unwrap_or_else(|| purpose.default_lifetime())
    }

    /// Generate full verification URL
    pub fn verification_url(&self, token: &str) -> String {
        format!("{}{}{}", self.base_url, self.verify_path, token)
    }
}

/// Request to create a magic link
#[derive(Debug, Clone, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
    #[serde(default)]
    pub purpose: MagicLinkPurpose,
    pub redirect_to: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl MagicLinkRequest {
    pub fn login(email: &str) -> Self {
        Self {
            email: email.to_string(),
            purpose: MagicLinkPurpose::Login,
            redirect_to: None,
            metadata: None,
        }
    }

    pub fn signup(email: &str) -> Self {
        Self {
            email: email.to_string(),
            purpose: MagicLinkPurpose::Signup,
            redirect_to: None,
            metadata: None,
        }
    }

    pub fn with_redirect(mut self, url: &str) -> Self {
        self.redirect_to = Some(url.to_string());
        self
    }
}

/// Response after creating magic link
#[derive(Debug, Clone, Serialize)]
pub struct MagicLinkResponse {
    pub success: bool,
    pub message: String,
    /// Only included in dev mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_url: Option<String>,
}

impl MagicLinkResponse {
    pub fn sent() -> Self {
        Self {
            success: true,
            message: "If an account exists with this email, a magic link has been sent.".to_string(),
            debug_url: None,
        }
    }

    pub fn with_debug_url(mut self, url: &str) -> Self {
        self.debug_url = Some(url.to_string());
        self
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            debug_url: None,
        }
    }
}

/// Magic link verification request
#[derive(Debug, Clone, Deserialize)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

/// Magic link verification result
#[derive(Debug, Clone)]
pub struct MagicLinkVerifyResult {
    pub email: String,
    pub user_id: Option<String>,
    pub purpose: MagicLinkPurpose,
    pub redirect_to: Option<String>,
    pub is_new_user: bool,
}

/// Email template data for magic link
#[derive(Debug, Clone, Serialize)]
pub struct MagicLinkEmailData {
    pub recipient_email: String,
    pub recipient_name: Option<String>,
    pub magic_link_url: String,
    pub purpose: String,
    pub expires_in_minutes: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub app_name: String,
}

/// Rate limit entry
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MagicLinkRateLimit {
    pub key: String,  // email or IP
    pub key_type: String,  // "email" or "ip"
    pub count: i32,
    pub window_start: DateTime<Utc>,
}

/// Magic link database schema
pub fn magic_link_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS magic_link_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    user_id TEXT,
    purpose TEXT NOT NULL DEFAULT 'login',
    redirect_to TEXT,
    metadata TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    used_at TEXT,
    revoked_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_magic_link_hash ON magic_link_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_magic_link_email ON magic_link_tokens(email);
CREATE INDEX IF NOT EXISTS idx_magic_link_expires ON magic_link_tokens(expires_at);

CREATE TABLE IF NOT EXISTS magic_link_rate_limits (
    key TEXT NOT NULL,
    key_type TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    window_start TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (key, key_type)
);

CREATE INDEX IF NOT EXISTS idx_magic_link_rate_window ON magic_link_rate_limits(window_start);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_link_purpose() {
        assert_eq!(MagicLinkPurpose::Login.as_str(), "login");
        assert_eq!(MagicLinkPurpose::from_str("signup"), MagicLinkPurpose::Signup);
    }

    #[test]
    fn test_default_lifetimes() {
        assert!(MagicLinkPurpose::Login.default_lifetime() < MagicLinkPurpose::Signup.default_lifetime());
    }

    #[test]
    fn test_config_verification_url() {
        let config = MagicLinkConfig::default();
        let url = config.verification_url("abc123");
        assert!(url.contains("/auth/magic-link/verify"));
        assert!(url.contains("abc123"));
    }

    #[test]
    fn test_magic_link_request() {
        let req = MagicLinkRequest::login("test@example.com")
            .with_redirect("/dashboard");

        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.purpose, MagicLinkPurpose::Login);
        assert_eq!(req.redirect_to, Some("/dashboard".to_string()));
    }
}
```

## Files to Create
- `src/auth/magic_link/types.rs` - Magic link types
- `src/auth/magic_link/mod.rs` - Module exports
