# Spec 374: GitHub OAuth Implementation

## Overview
Implement the GitHub OAuth provider for authentication and account linking.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### GitHub OAuth Provider
```rust
// src/auth/oauth/github/provider.rs

use super::types::*;
use crate::auth::oauth::types::{OAuthProvider, OAuthTokens, OAuthUserInfo, OAuthError};
use reqwest::Client;
use tracing::{debug, info, warn, instrument};

/// GitHub OAuth provider implementation
pub struct GitHubProvider {
    config: GitHubOAuthConfig,
    client: Client,
}

impl GitHubProvider {
    pub fn new(config: GitHubOAuthConfig) -> Self {
        let client = Client::builder()
            .user_agent("Tachikoma/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Exchange authorization code for tokens
    #[instrument(skip(self, code))]
    pub async fn exchange_code(&self, code: &str) -> Result<GitHubTokenResponse, OAuthError> {
        let response = self.client
            .post(&self.config.token_url)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("code", &code.to_string()),
                ("redirect_uri", &self.config.redirect_uri),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::CodeExchangeFailed(error_text));
        }

        let token_response: GitHubTokenResponse = response.json().await?;

        if let Some(error) = &token_response.error {
            return Err(OAuthError::CodeExchangeFailed(
                token_response.error_description.unwrap_or_else(|| error.clone())
            ));
        }

        debug!("Successfully exchanged code for GitHub token");
        Ok(token_response)
    }

    /// Get user profile from GitHub
    #[instrument(skip(self, access_token))]
    pub async fn get_user(&self, access_token: &str) -> Result<GitHubUser, OAuthError> {
        let response = self.client
            .get(format!("{}/user", self.config.api_base_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::UserInfoFailed(error_text));
        }

        let user: GitHubUser = response.json().await?;
        debug!("Retrieved GitHub user: {}", user.login);
        Ok(user)
    }

    /// Get user emails from GitHub
    #[instrument(skip(self, access_token))]
    pub async fn get_emails(&self, access_token: &str) -> Result<Vec<GitHubEmail>, OAuthError> {
        let response = self.client
            .get(format!("{}/user/emails", self.config.api_base_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::UserInfoFailed(error_text));
        }

        let emails: Vec<GitHubEmail> = response.json().await?;
        debug!("Retrieved {} GitHub emails", emails.len());
        Ok(emails)
    }

    /// Get primary verified email
    pub async fn get_primary_email(&self, access_token: &str) -> Result<Option<String>, OAuthError> {
        let emails = self.get_emails(access_token).await?;

        // First try to find primary verified email
        if let Some(primary) = emails.iter().find(|e| e.primary && e.verified) {
            return Ok(Some(primary.email.clone()));
        }

        // Fall back to any verified email
        if let Some(verified) = emails.iter().find(|e| e.verified) {
            return Ok(Some(verified.email.clone()));
        }

        Ok(None)
    }

    /// Validate email domain
    pub fn validate_email_domain(&self, email: &str) -> Result<(), OAuthError> {
        if let Some(allowed_domains) = &self.config.allowed_email_domains {
            let domain = email.split('@').last().unwrap_or("");
            if !allowed_domains.iter().any(|d| d == domain) {
                return Err(OAuthError::EmailDomainNotAllowed(domain.to_string()));
            }
        }
        Ok(())
    }

    /// Get full user info with email
    #[instrument(skip(self, access_token))]
    pub async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
        let user = self.get_user(access_token).await?;

        // Get email - try user profile first, then emails endpoint
        let email = if let Some(email) = &user.email {
            Some(email.clone())
        } else {
            self.get_primary_email(access_token).await?
        };

        // Validate email requirements
        if self.config.require_verified_email {
            let email = email.as_ref()
                .ok_or(OAuthError::EmailNotVerified)?;
            self.validate_email_domain(email)?;
        }

        Ok(OAuthUserInfo {
            provider_id: user.id.to_string(),
            email,
            email_verified: true, // GitHub only returns verified emails from /user/emails
            name: user.name,
            username: Some(user.login.clone()),
            avatar_url: user.avatar_url,
            raw_data: serde_json::to_value(&user).unwrap_or_default(),
        })
    }

    /// Get configuration
    pub fn config(&self) -> &GitHubOAuthConfig {
        &self.config
    }
}

impl OAuthProvider for GitHubProvider {
    fn name(&self) -> &'static str {
        "github"
    }

    fn authorization_url(&self, state: &str) -> String {
        self.config.authorization_url(state)
    }

    async fn exchange_code(&self, code: &str) -> Result<OAuthTokens, OAuthError> {
        let response = self.exchange_code(code).await?;

        Ok(OAuthTokens {
            access_token: response.access_token,
            refresh_token: None, // GitHub doesn't provide refresh tokens for OAuth apps
            token_type: response.token_type,
            expires_in: None,
            scope: Some(response.scope),
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
        self.get_user_info(access_token).await
    }
}

/// GitHub OAuth service for managing the flow
pub struct GitHubOAuthService {
    provider: GitHubProvider,
    state_store: Box<dyn OAuthStateStore + Send + Sync>,
}

impl GitHubOAuthService {
    pub fn new(
        config: GitHubOAuthConfig,
        state_store: Box<dyn OAuthStateStore + Send + Sync>,
    ) -> Self {
        Self {
            provider: GitHubProvider::new(config),
            state_store,
        }
    }

    /// Start OAuth flow - generate authorization URL
    #[instrument(skip(self))]
    pub async fn start_flow(&self, redirect_to: Option<String>) -> Result<(String, OAuthState), OAuthError> {
        let state = OAuthState::new("github", redirect_to);

        // Store state
        self.state_store.store(&state).await?;

        let auth_url = self.provider.authorization_url(&state.state);

        info!("Started GitHub OAuth flow with state: {}", state.state);
        Ok((auth_url, state))
    }

    /// Handle OAuth callback
    #[instrument(skip(self, callback))]
    pub async fn handle_callback(
        &self,
        callback: &OAuthCallback,
    ) -> Result<GitHubAuthResult, OAuthError> {
        // Check for OAuth error
        if callback.has_error() {
            let error = callback.error.clone().unwrap_or_default();
            let description = callback.error_description.clone().unwrap_or_default();
            return Err(OAuthError::ProviderError(format!("{}: {}", error, description)));
        }

        // Validate state
        let stored_state = self.state_store.get(&callback.state).await?
            .ok_or(OAuthError::StateMismatch)?;

        if stored_state.is_expired() {
            return Err(OAuthError::StateExpired);
        }

        // Exchange code for token
        let tokens = self.provider.exchange_code(&callback.code).await?;

        // Get user info
        let user_info = self.provider.get_user_info(&tokens.access_token).await?;

        // Clean up state
        self.state_store.delete(&callback.state).await?;

        info!("GitHub OAuth completed for user: {:?}", user_info.username);

        Ok(GitHubAuthResult {
            tokens,
            user_info,
            redirect_to: stored_state.redirect_to,
        })
    }

    /// Get provider
    pub fn provider(&self) -> &GitHubProvider {
        &self.provider
    }
}

/// OAuth state store trait
#[async_trait::async_trait]
pub trait OAuthStateStore {
    async fn store(&self, state: &OAuthState) -> Result<(), OAuthError>;
    async fn get(&self, state: &str) -> Result<Option<OAuthState>, OAuthError>;
    async fn delete(&self, state: &str) -> Result<(), OAuthError>;
}

/// SQLite-backed OAuth state store
pub struct SqliteOAuthStateStore {
    pool: sqlx::sqlite::SqlitePool,
}

impl SqliteOAuthStateStore {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl OAuthStateStore for SqliteOAuthStateStore {
    async fn store(&self, state: &OAuthState) -> Result<(), OAuthError> {
        let metadata = state.metadata.as_ref()
            .map(|m| serde_json::to_string(m).ok())
            .flatten();

        sqlx::query(r#"
            INSERT INTO oauth_states (state, provider, redirect_to, created_at, expires_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&state.state)
        .bind(&state.provider)
        .bind(&state.redirect_to)
        .bind(state.created_at)
        .bind(state.expires_at)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ProviderError(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, state: &str) -> Result<Option<OAuthState>, OAuthError> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, String, String, Option<String>)>(
            "SELECT state, provider, redirect_to, created_at, expires_at, metadata FROM oauth_states WHERE state = ?"
        )
        .bind(state)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ProviderError(e.to_string()))?;

        Ok(row.map(|(state, provider, redirect_to, created_at, expires_at, metadata)| {
            OAuthState {
                state,
                provider,
                redirect_to,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                expires_at: chrono::DateTime::parse_from_rfc3339(&expires_at)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                metadata: metadata.and_then(|m| serde_json::from_str(&m).ok()),
            }
        }))
    }

    async fn delete(&self, state: &str) -> Result<(), OAuthError> {
        sqlx::query("DELETE FROM oauth_states WHERE state = ?")
            .bind(state)
            .execute(&self.pool)
            .await
            .map_err(|e| OAuthError::ProviderError(e.to_string()))?;

        Ok(())
    }
}

/// GitHub authentication result
#[derive(Debug, Clone)]
pub struct GitHubAuthResult {
    pub tokens: GitHubTokenResponse,
    pub user_info: OAuthUserInfo,
    pub redirect_to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config() {
        let config = GitHubOAuthConfig::default();
        assert!(!config.is_valid()); // No client_id/secret

        let config = GitHubOAuthConfig {
            client_id: "test-id".to_string(),
            client_secret: "test-secret".to_string(),
            ..Default::default()
        };
        assert!(config.is_valid());
    }

    #[test]
    fn test_authorization_url() {
        let config = GitHubOAuthConfig {
            client_id: "my-client-id".to_string(),
            ..Default::default()
        };

        let url = config.authorization_url("test-state");
        assert!(url.contains("client_id=my-client-id"));
        assert!(url.contains("state=test-state"));
    }

    #[test]
    fn test_enterprise_config() {
        let config = GitHubOAuthConfig::enterprise("https://github.mycompany.com");
        assert_eq!(config.api_base_url, "https://github.mycompany.com/api/v3");
        assert_eq!(config.authorize_url, "https://github.mycompany.com/login/oauth/authorize");
    }
}
```

## Files to Create
- `src/auth/oauth/github/provider.rs` - GitHub OAuth provider
- `src/auth/oauth/github/service.rs` - OAuth service implementation
