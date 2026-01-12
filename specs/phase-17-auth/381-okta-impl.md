# Spec 381: Okta OAuth Implementation

## Overview
Implement the Okta OAuth/OIDC provider for enterprise SSO authentication.

## Rust Implementation

### Okta OAuth Provider
```rust
// src/auth/oauth/okta/provider.rs

use super::types::*;
use crate::auth::oauth::types::{OAuthProvider, OAuthTokens, OAuthUserInfo, OAuthError};
use reqwest::Client;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use tracing::{debug, info, warn, instrument};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Okta OAuth provider implementation
pub struct OktaProvider {
    config: OktaOAuthConfig,
    client: Client,
    jwks_cache: Arc<RwLock<JwksCache>>,
}

/// Cache for Okta's JWKS
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

impl OktaProvider {
    pub fn new(config: OktaOAuthConfig) -> Self {
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
    #[instrument(skip(self, code, pkce_verifier))]
    pub async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<OktaTokenResponse, OAuthError> {
        let mut form = vec![
            ("grant_type", "authorization_code".to_string()),
            ("client_id", self.config.client_id.clone()),
            ("client_secret", self.config.client_secret.clone()),
            ("code", code.to_string()),
            ("redirect_uri", self.config.redirect_uri.clone()),
        ];

        if let Some(verifier) = pkce_verifier {
            form.push(("code_verifier", verifier.to_string()));
        }

        let response = self.client
            .post(&self.config.token_url())
            .header("Accept", "application/json")
            .form(&form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::CodeExchangeFailed(error_text));
        }

        let token_response: OktaTokenResponse = response.json().await?;

        if let Some(error) = &token_response.error {
            return Err(OAuthError::CodeExchangeFailed(
                token_response.error_description.unwrap_or_else(|| error.clone())
            ));
        }

        debug!("Successfully exchanged code for Okta tokens");
        Ok(token_response)
    }

    /// Get user info from userinfo endpoint
    #[instrument(skip(self, access_token))]
    pub async fn get_userinfo(&self, access_token: &str) -> Result<OktaUserInfo, OAuthError> {
        let response = self.client
            .get(&self.config.userinfo_url())
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::UserInfoFailed(error_text));
        }

        let user_info: OktaUserInfo = response.json().await?;
        debug!("Retrieved Okta user info: {}", user_info.sub);
        Ok(user_info)
    }

    /// Fetch and cache Okta's JWKS
    async fn fetch_jwks(&self) -> Result<(), OAuthError> {
        let response = self.client
            .get(&self.config.jwks_uri())
            .send()
            .await?;

        #[derive(Deserialize)]
        struct Jwks {
            keys: Vec<JwkKey>,
        }

        #[derive(Deserialize)]
        struct JwkKey {
            kid: String,
            kty: String,
            #[serde(default)]
            alg: Option<String>,
            n: Option<String>,
            e: Option<String>,
        }

        let jwks: Jwks = response.json().await?;

        let mut cache = self.jwks_cache.write().await;
        cache.keys.clear();

        for key in jwks.keys {
            if key.kty == "RSA" {
                if let (Some(n), Some(e)) = (&key.n, &key.e) {
                    if let Ok(decoding_key) = self.create_rsa_key(n, e) {
                        cache.keys.insert(key.kid.clone(), decoding_key);
                    }
                }
            }
        }

        cache.fetched_at = Some(chrono::Utc::now());
        debug!("Fetched {} Okta JWKs", cache.keys.len());

        Ok(())
    }

    /// Create RSA DecodingKey from components
    fn create_rsa_key(&self, n: &str, e: &str) -> Result<DecodingKey, OAuthError> {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let n_bytes = URL_SAFE_NO_PAD.decode(n)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid JWK n: {}", e)))?;
        let e_bytes = URL_SAFE_NO_PAD.decode(e)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid JWK e: {}", e)))?;

        DecodingKey::from_rsa_components(&n_bytes, &e_bytes)
            .map_err(|e| OAuthError::ProviderError(format!("Invalid RSA components: {}", e)))
    }

    /// Get decoding key for a specific kid
    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, OAuthError> {
        {
            let cache = self.jwks_cache.read().await;
            if !cache.is_stale() {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        self.fetch_jwks().await?;

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
    ) -> Result<OktaIdTokenClaims, OktaOAuthError> {
        // Decode header to get kid
        let header = decode_header(id_token)
            .map_err(|_| OktaOAuthError::InvalidIdToken)?;

        let kid = header.kid
            .ok_or(OktaOAuthError::InvalidIdToken)?;

        // Get decoding key
        let decoding_key = self.get_decoding_key(&kid).await
            .map_err(OktaOAuthError::OAuth)?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.client_id]);
        validation.set_issuer(&[&self.config.issuer()]);

        // Decode and validate
        let token_data = decode::<OktaIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| {
                warn!("ID token validation failed: {}", e);
                OktaOAuthError::InvalidIdToken
            })?;

        let claims = token_data.claims;

        // Validate nonce
        if let Some(expected_nonce) = nonce {
            if claims.nonce.as_deref() != Some(expected_nonce) {
                return Err(OktaOAuthError::NonceMismatch);
            }
        }

        // Validate groups if configured
        if let Some(allowed_groups) = &self.config.allowed_groups {
            if !claims.has_any_group(allowed_groups) {
                return Err(OktaOAuthError::GroupNotAllowed);
            }
        }

        // Validate email verified
        if self.config.require_verified_email {
            if claims.email_verified != Some(true) {
                return Err(OktaOAuthError::EmailNotVerified);
            }
        }

        Ok(claims)
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OktaTokenResponse, OAuthError> {
        let response = self.client
            .post(&self.config.token_url())
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::CodeExchangeFailed(error_text));
        }

        let token_response: OktaTokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Revoke tokens
    pub async fn revoke_token(&self, token: &str, token_type: &str) -> Result<(), OAuthError> {
        let revoke_url = format!("{}/v1/revoke", self.config.issuer());

        let response = self.client
            .post(&revoke_url)
            .form(&[
                ("token", token),
                ("token_type_hint", token_type),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::ProviderError(format!("Revocation failed: {}", error_text)));
        }

        Ok(())
    }

    /// Introspect token
    pub async fn introspect_token(&self, token: &str) -> Result<TokenIntrospection, OAuthError> {
        let introspect_url = format!("{}/v1/introspect", self.config.issuer());

        let response = self.client
            .post(&introspect_url)
            .form(&[
                ("token", token),
                ("client_id", &self.config.client_id),
                ("client_secret", &self.config.client_secret),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthError::ProviderError(format!("Introspection failed: {}", error_text)));
        }

        let introspection: TokenIntrospection = response.json().await?;
        Ok(introspection)
    }

    pub fn config(&self) -> &OktaOAuthConfig {
        &self.config
    }
}

/// Token introspection response
#[derive(Debug, Clone, Deserialize)]
pub struct TokenIntrospection {
    pub active: bool,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub exp: Option<i64>,
    #[serde(default)]
    pub iat: Option<i64>,
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub aud: Option<String>,
    #[serde(default)]
    pub iss: Option<String>,
    #[serde(default)]
    pub jti: Option<String>,
    #[serde(default)]
    pub uid: Option<String>,
}

use serde::Deserialize;

impl OAuthProvider for OktaProvider {
    fn name(&self) -> &'static str {
        "okta"
    }

    fn authorization_url(&self, state: &str) -> String {
        self.config.authorization_url(state, None, None)
    }

    async fn exchange_code(&self, code: &str) -> Result<OAuthTokens, OAuthError> {
        let response = self.exchange_code(code, None).await?;

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
            username: user_info.preferred_username,
            avatar_url: None,
            raw_data: serde_json::to_value(&user_info).unwrap_or_default(),
        })
    }
}

/// Okta OAuth service
pub struct OktaOAuthService {
    provider: OktaProvider,
    state_store: Box<dyn OktaStateStore + Send + Sync>,
}

#[async_trait::async_trait]
pub trait OktaStateStore {
    async fn store(&self, state: &OktaOAuthState) -> Result<(), OAuthError>;
    async fn get(&self, state: &str) -> Result<Option<OktaOAuthState>, OAuthError>;
    async fn delete(&self, state: &str) -> Result<(), OAuthError>;
}

impl OktaOAuthService {
    pub fn new(
        config: OktaOAuthConfig,
        state_store: Box<dyn OktaStateStore + Send + Sync>,
    ) -> Self {
        Self {
            provider: OktaProvider::new(config),
            state_store,
        }
    }

    /// Start OAuth flow with PKCE
    pub async fn start_flow(&self, redirect_to: Option<String>) -> Result<(String, OktaOAuthState), OAuthError> {
        let state = OktaOAuthState::new(redirect_to, self.provider.config.use_pkce);

        self.state_store.store(&state).await?;

        let auth_url = self.provider.config.authorization_url(
            &state.state,
            Some(&state.nonce),
            state.pkce_challenge().as_deref(),
        );

        info!("Started Okta OAuth flow with state: {}", state.state);
        Ok((auth_url, state))
    }

    /// Handle OAuth callback
    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<OktaAuthResult, OktaOAuthError> {
        // Validate state
        let stored_state = self.state_store.get(state).await
            .map_err(OktaOAuthError::OAuth)?
            .ok_or(OktaOAuthError::OAuth(OAuthError::StateMismatch))?;

        if stored_state.is_expired() {
            return Err(OktaOAuthError::OAuth(OAuthError::StateExpired));
        }

        // Exchange code for tokens (with PKCE if used)
        let tokens = self.provider.exchange_code(
            code,
            stored_state.pkce_verifier.as_deref(),
        ).await.map_err(OktaOAuthError::OAuth)?;

        // Validate ID token
        let claims = self.provider.validate_id_token(
            &tokens.id_token,
            Some(&stored_state.nonce),
        ).await?;

        // Clean up state
        self.state_store.delete(state).await
            .map_err(OktaOAuthError::OAuth)?;

        // Create session info
        let session = OktaSession::from_tokens(&claims, &tokens);

        info!("Okta OAuth completed for user: {}", claims.sub);

        Ok(OktaAuthResult {
            tokens,
            claims,
            session,
            redirect_to: stored_state.redirect_to,
        })
    }

    /// Logout from Okta
    pub fn logout_url(&self, id_token: &str, post_logout_redirect: Option<&str>) -> String {
        self.provider.config.logout_url(Some(id_token), post_logout_redirect)
    }

    pub fn provider(&self) -> &OktaProvider {
        &self.provider
    }
}

/// Okta authentication result
#[derive(Debug, Clone)]
pub struct OktaAuthResult {
    pub tokens: OktaTokenResponse,
    pub claims: OktaIdTokenClaims,
    pub session: OktaSession,
    pub redirect_to: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okta_provider_creation() {
        let config = OktaOAuthConfig::new(
            "dev-123456.okta.com",
            "test-client",
            "test-secret"
        );

        let provider = OktaProvider::new(config);
        assert_eq!(provider.name(), "okta");
    }

    #[test]
    fn test_authorization_url_with_pkce() {
        let config = OktaOAuthConfig::new(
            "dev-123456.okta.com",
            "test-client",
            "test-secret"
        );

        let state = OktaOAuthState::new(None, true);
        let url = config.authorization_url(
            &state.state,
            Some(&state.nonce),
            state.pkce_challenge().as_deref(),
        );

        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
    }
}
```

## Files to Create
- `src/auth/oauth/okta/provider.rs` - Okta OAuth provider
- `src/auth/oauth/okta/service.rs` - Okta OAuth service
