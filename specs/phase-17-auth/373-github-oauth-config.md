# Spec 373: GitHub OAuth Configuration

## Overview
Define configuration and types for GitHub OAuth authentication integration.

## Rust Implementation

### GitHub OAuth Types
```rust
// src/auth/oauth/github/types.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// GitHub OAuth configuration
#[derive(Debug, Clone)]
pub struct GitHubOAuthConfig {
    /// GitHub OAuth App Client ID
    pub client_id: String,
    /// GitHub OAuth App Client Secret
    pub client_secret: String,
    /// Redirect URI after authorization
    pub redirect_uri: String,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// GitHub API base URL (for enterprise)
    pub api_base_url: String,
    /// GitHub authorization URL
    pub authorize_url: String,
    /// GitHub token URL
    pub token_url: String,
    /// Allow signup from GitHub OAuth
    pub allow_signup: bool,
    /// Restrict to specific email domains
    pub allowed_email_domains: Option<Vec<String>>,
    /// Require verified email
    pub require_verified_email: bool,
}

impl Default for GitHubOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: "http://localhost:8080/auth/github/callback".to_string(),
            scopes: vec!["user:email".to_string(), "read:user".to_string()],
            api_base_url: "https://api.github.com".to_string(),
            authorize_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            allow_signup: true,
            allowed_email_domains: None,
            require_verified_email: true,
        }
    }
}

impl GitHubOAuthConfig {
    pub fn enterprise(base_url: &str) -> Self {
        Self {
            api_base_url: format!("{}/api/v3", base_url),
            authorize_url: format!("{}/login/oauth/authorize", base_url),
            token_url: format!("{}/login/oauth/access_token", base_url),
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    /// Generate authorization URL with state
    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&scope={}&state={}",
            self.authorize_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state)
        )
    }
}

/// GitHub OAuth token response
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// GitHub user profile
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub html_url: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GitHub email
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

/// OAuth state (stored temporarily during flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub provider: String,
    pub redirect_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl OAuthState {
    pub fn new(provider: &str, redirect_to: Option<String>) -> Self {
        use uuid::Uuid;
        let now = Utc::now();

        Self {
            state: Uuid::new_v4().to_string(),
            provider: provider.to_string(),
            redirect_to,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(10),
            metadata: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Linked OAuth account
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinkedOAuthAccount {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_username: Option<String>,
    pub provider_email: Option<String>,
    pub access_token: Option<String>,  // Encrypted
    pub refresh_token: Option<String>, // Encrypted
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Option<String>,
    pub profile_data: Option<String>,  // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl LinkedOAuthAccount {
    pub fn is_token_expired(&self) -> bool {
        self.token_expires_at.map_or(true, |exp| Utc::now() > exp)
    }
}

/// OAuth callback parameters
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCallback {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

impl OAuthCallback {
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }
}

/// OAuth provider trait
pub trait OAuthProvider {
    /// Get provider name
    fn name(&self) -> &'static str;

    /// Generate authorization URL
    fn authorization_url(&self, state: &str) -> String;

    /// Exchange code for token
    fn exchange_code(&self, code: &str) -> impl std::future::Future<Output = Result<OAuthTokens, OAuthError>> + Send;

    /// Get user info
    fn get_user_info(&self, access_token: &str) -> impl std::future::Future<Output = Result<OAuthUserInfo, OAuthError>> + Send;
}

/// Generic OAuth tokens
#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

/// Generic OAuth user info
#[derive(Debug, Clone)]
pub struct OAuthUserInfo {
    pub provider_id: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub name: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub raw_data: serde_json::Value,
}

/// OAuth error
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("OAuth state mismatch")]
    StateMismatch,

    #[error("OAuth state expired")]
    StateExpired,

    #[error("OAuth code exchange failed: {0}")]
    CodeExchangeFailed(String),

    #[error("Failed to get user info: {0}")]
    UserInfoFailed(String),

    #[error("Email not provided or not verified")]
    EmailNotVerified,

    #[error("Email domain not allowed: {0}")]
    EmailDomainNotAllowed(String),

    #[error("Account already linked")]
    AccountAlreadyLinked,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// OAuth database schema
pub fn oauth_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS oauth_states (
    state TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    redirect_to TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_oauth_states_expires ON oauth_states(expires_at);

CREATE TABLE IF NOT EXISTS linked_oauth_accounts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    provider_username TEXT,
    provider_email TEXT,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TEXT,
    scopes TEXT,
    profile_data TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT,
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX IF NOT EXISTS idx_linked_oauth_user ON linked_oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_linked_oauth_provider ON linked_oauth_accounts(provider, provider_user_id);
"#
}
```

## Files to Create
- `src/auth/oauth/github/types.rs` - GitHub OAuth types
- `src/auth/oauth/github/mod.rs` - Module exports
- `src/auth/oauth/types.rs` - Common OAuth types
