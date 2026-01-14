# Spec 376: Google OAuth Implementation

## Overview
Implement the Google OAuth provider with OpenID Connect support for authentication.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Google OAuth Provider
```rust
// src/auth/oauth/google/provider.rs

use super::types::*;
use crate::auth::oauth::types::{OAuthProvider, OAuthTokens, OAuthUserInfo, OAuthError};
use reqwest::Client;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use tracing::{debug, info, warn, instrument};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Google OAuth provider implementation with OIDC support
pub struct GoogleProvider {
    config: GoogleOAuthConfig,
    client: Client,
    jwks_cache: Arc<RwLock<JwksCache>>,
}

/// Cache for Google's JWKS
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<chrono::DateTime<chrono::Utc>>,
    ttl: chrono::Duration,
}

impl JwksCache {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
            fetched_at: None,
            ttl: chrono::Duration::hours(1),
        }
    }

    fn is_stale(&self) -> bool {
        match self.fetched_at {
            Some(fetched) => chrono::Utc::now() > fetched + self.ttl,
            None => true,
        }
    }
}

impl GoogleProvider {
    pub fn new(config: GoogleOAuthConfig) -> Self {
        let client = Client::builder()
            .user_agent("Tachikoma/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new())),
        }
    }

    /// Exchange authorization code for tokens
    #[instrument(skip(self, code))]
    pub async fn exchange_code(&self, code: &str) -> Result<GoogleTokenResponse, OAuthError> {
        let response = self.client
            .post(&self.config.token_url)
            .form(&[
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("code", &code.to_string()),
                ("redirect_uri", &self.config.redirect_uri),
                ("grant_type", &"authorization_code".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::CodeExchangeFailed(error_text));
        }

        let token_response: GoogleTokenResponse = response.json().await?;

        if let Some(error) = &token_response.error {
            return Err(OAuthError::CodeExchangeFailed(
                token_response.error_description.unwrap_or_else(|| error.clone())
            ));
        }

        debug!("Successfully exchanged code for Google tokens");
        Ok(token_response)
    }

    /// Get user info from userinfo endpoint
    #[instrument(skip(self, access_token))]
    pub async fn get_userinfo(&self, access_token: &str) -> Result<GoogleUserInfo, OAuthError> {
        let response = self.client
            .get(&self.config.userinfo_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::UserInfoFailed(error_text));
        }

        let user_info: GoogleUserInfo = response.json().await?;
        debug!("Retrieved Google user info: {}", user_info.sub);
        Ok(user_info)
    }

    /// Fetch and cache Google's JWKS
    async fn fetch_jwks(&self) -> Result<(), OAuthError> {
        let response = self.client
            .get(GoogleJwks::JWKS_URL)
            .send()
            .await?;

        let jwks: GoogleJwks = response.json().await?;

        let mut cache = self.jwks_cache.write().await;
        cache.keys.clear();

        for key in jwks.keys {
            if let Ok(decoding_key) = self.jwk_to_decoding_key(&key) {
                cache.keys.insert(key.kid.clone(), decoding_key);
            }
        }

        cache.fetched_at = Some(chrono::Utc::now());
        debug!("Fetched {} Google JWKs", cache.keys.len());

        Ok(())
    }

    /// Convert JWK to DecodingKey
    fn jwk_to_decoding_key(&self, jwk: &GoogleJwk) -> Result<DecodingKey, OAuthError> {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let n = URL_SAFE_NO_PAD.decode(&jwk.n)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid JWK n: {}", e)))?;
        let e = URL_SAFE_NO_PAD.decode(&jwk.e)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid JWK e: {}", e)))?;

        DecodingKey::from_rsa_components(&n, &e)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid RSA components: {}", e)))
    }

    /// Get decoding key for a specific kid
    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, OAuthError> {
        // Check if cache is stale
        {
            let cache = self.jwks_cache.read().await;
            if !cache.is_stale() {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        // Refresh cache
        self.fetch_jwks().await?;

        // Try again
        let cache = self.jwks_cache.read().await;
        cache.keys.get(kid)
            .cloned()
            .ok_or_else(|| OAuthError::ProviderError(format!("JWK not found: {}", kid)))
    }

    /// Validate and decode ID token
    #[instrument(skip(self, id_token, nonce))]
    pub async fn validate_id_token(
        &self,
        id_token: &str,
        nonce: Option<&str>,
    ) -> Result<GoogleIdTokenClaims, GoogleOAuthError> {
        // Decode header to get kid
        let header = decode_header(id_token)
            .map_err(|_| GoogleOAuthError::InvalidIdToken)?;

        let kid = header.kid
            .ok_or(GoogleOAuthError::InvalidIdToken)?;

        // Get decoding key
        let decoding_key = self.get_decoding_key(&kid).await
            .map_err(GoogleOAuthError::OAuth)?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.client_id]);
        validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);

        // Decode and validate
        let token_data = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| {
                warn!("ID token validation failed: {}", e);
                GoogleOAuthError::InvalidIdToken
            })?;

        let claims = token_data.claims;

        // Validate nonce if provided
        if let Some(expected_nonce) = nonce {
            if claims.nonce.as_deref() != Some(expected_nonce) {
                return Err(GoogleOAuthError::NonceMismatch);
            }
        }

        // Validate hosted domain if configured
        if let Some(expected_hd) = &self.config.hosted_domain {
            if claims.hd.as_ref() != Some(expected_hd) {
                return Err(GoogleOAuthError::HostedDomainMismatch {
                    expected: expected_hd.clone(),
                    actual: claims.hd.clone(),
                });
            }
        }

        // Validate email verified
        if self.config.require_verified_email {
            if claims.email_verified != Some(true) {
                return Err(GoogleOAuthError::EmailNotVerified);
            }
        }

        Ok(claims)
    }

    /// Get user info - prefer ID token, fall back to userinfo endpoint
    #[instrument(skip(self, tokens, nonce))]
    pub async fn get_user_info_full(
        &self,
        tokens: &GoogleTokenResponse,
        nonce: Option<&str>,
    ) -> Result<OAuthUserInfo, GoogleOAuthError> {
        // Try ID token first (more secure)
        if let Some(id_token) = &tokens.id_token {
            let claims = self.validate_id_token(id_token, nonce).await?;

            return Ok(OAuthUserInfo {
                provider_id: claims.sub,
                email: claims.email,
                email_verified: claims.email_verified.unwrap_or(false),
                name: claims.name,
                username: None,
                avatar_url: claims.picture,
                raw_data: serde_json::to_value(&claims).unwrap_or_default(),
            });
        }

        // Fall back to userinfo endpoint
        let user_info = self.get_userinfo(&tokens.access_token).await
            .map_err(GoogleOAuthError::OAuth)?;

        if self.config.require_verified_email && user_info.email_verified != Some(true) {
            return Err(GoogleOAuthError::EmailNotVerified);
        }

        // Validate hosted domain
        if let Some(expected_hd) = &self.config.hosted_domain {
            if user_info.hd.as_ref() != Some(expected_hd) {
                return Err(GoogleOAuthError::HostedDomainMismatch {
                    expected: expected_hd.clone(),
                    actual: user_info.hd.clone(),
                });
            }
        }

        Ok(OAuthUserInfo {
            provider_id: user_info.sub,
            email: user_info.email,
            email_verified: user_info.email_verified.unwrap_or(false),
            name: user_info.name,
            username: None,
            avatar_url: user_info.picture,
            raw_data: serde_json::to_value(&user_info).unwrap_or_default(),
        })
    }

    /// Revoke tokens
    pub async fn revoke_token(&self, token: &str) -> Result<(), OAuthError> {
        let response = self.client
            .post("https://oauth2.googleapis.com/revoke")
            .form(&[("token", token)])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::ProviderError(format!("Revocation failed: {}", error_text)));
        }

        Ok(())
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleTokenResponse, OAuthError> {
        let response = self.client
            .post(&self.config.token_url)
            .form(&[
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("refresh_token", &refresh_token.to_string()),
                ("grant_type", &"refresh_token".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::CodeExchangeFailed(error_text));
        }

        let token_response: GoogleTokenResponse = response.json().await?;
        Ok(token_response)
    }

    pub fn config(&self) -> &GoogleOAuthConfig {
        &self.config
    }
}

impl OAuthProvider for GoogleProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    fn authorization_url(&self, state: &str) -> String {
        self.config.authorization_url(state, None)
    }

    async fn exchange_code(&self, code: &str) -> Result<OAuthTokens, OAuthError> {
        let response = self.exchange_code(code).await?;

        Ok(OAuthTokens {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: Some(response.expires_in),
            scope: response.scope,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo, OAuthError> {
        let user_info = self.get_userinfo(access_token).await?;

        Ok(OAuthUserInfo {
            provider_id: user_info.sub,
            email: user_info.email,
            email_verified: user_info.email_verified.unwrap_or(false),
            name: user_info.name,
            username: None,
            avatar_url: user_info.picture,
            raw_data: serde_json::to_value(&user_info).unwrap_or_default(),
        })
    }
}

/// Google OAuth service
pub struct GoogleOAuthService {
    provider: GoogleProvider,
    state_store: Box<dyn GoogleStateStore + Send + Sync>,
}

#[async_trait::async_trait]
pub trait GoogleStateStore {
    async fn store(&self, state: &GoogleOAuthState) -> Result<(), OAuthError>;
    async fn get(&self, state: &str) -> Result<Option<GoogleOAuthState>, OAuthError>;
    async fn delete(&self, state: &str) -> Result<(), OAuthError>;
}

impl GoogleOAuthService {
    pub fn new(
        config: GoogleOAuthConfig,
        state_store: Box<dyn GoogleStateStore + Send + Sync>,
    ) -> Self {
        Self {
            provider: GoogleProvider::new(config),
            state_store,
        }
    }

    /// Start OAuth flow with OIDC nonce
    pub async fn start_flow(&self, redirect_to: Option<String>) -> Result<(String, GoogleOAuthState), OAuthError> {
        let state = GoogleOAuthState::new(redirect_to, true);

        self.state_store.store(&state).await?;

        let auth_url = self.provider.config.authorization_url(&state.state, state.nonce.as_deref());

        info!("Started Google OAuth flow with state: {}", state.state);
        Ok((auth_url, state))
    }

    /// Handle OAuth callback with ID token validation
    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<GoogleAuthResult, GoogleOAuthError> {
        // Validate state
        let stored_state = self.state_store.get(state).await
            .map_err(GoogleOAuthError::OAuth)?
            .ok_or(GoogleOAuthError::OAuth(OAuthError::StateMismatch))?;

        if stored_state.is_expired() {
            return Err(GoogleOAuthError::OAuth(OAuthError::StateExpired));
        }

        // Exchange code for tokens
        let tokens = self.provider.exchange_code(code).await
            .map_err(GoogleOAuthError::OAuth)?;

        // Get user info with ID token validation
        let user_info = self.provider.get_user_info_full(&tokens, stored_state.nonce.as_deref()).await?;

        // Clean up state
        self.state_store.delete(state).await
            .map_err(GoogleOAuthError::OAuth)?;

        info!("Google OAuth completed for user: {}", user_info.provider_id);

        Ok(GoogleAuthResult {
            tokens,
            user_info,
            redirect_to: stored_state.redirect_to,
        })
    }

    pub fn provider(&self) -> &GoogleProvider {
        &self.provider
    }
}

/// Google authentication result
#[derive(Debug, Clone)]
pub struct GoogleAuthResult {
    pub tokens: GoogleTokenResponse,
    pub user_info: OAuthUserInfo,
    pub redirect_to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider_creation() {
        let config = GoogleOAuthConfig {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            ..Default::default()
        };

        let provider = GoogleProvider::new(config);
        assert_eq!(provider.name(), "google");
    }

    #[test]
    fn test_authorization_url_with_nonce() {
        let config = GoogleOAuthConfig {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            hosted_domain: Some("example.com".to_string()),
            ..Default::default()
        };

        let url = config.authorization_url("test-state", Some("test-nonce"));
        assert!(url.contains("nonce=test-nonce"));
        assert!(url.contains("hd=example.com"));
    }
}
```

## Files to Create
- `src/auth/oauth/google/provider.rs` - Google OAuth provider
- `src/auth/oauth/google/service.rs` - Google OAuth service
