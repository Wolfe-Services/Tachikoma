# 065 - Gemini Authentication

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 065
**Status:** Planned
**Dependencies:** 064-gemini-api-client, 017-secret-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement secure API key and service account handling for the Google Gemini backend, including environment variable loading, OAuth2 support for GCP, and proper secret management.

---

## Acceptance Criteria

- [x] Secure API key storage using `Secret<T>`
- [x] Environment variable loading (`GOOGLE_API_KEY`, `GEMINI_API_KEY`)
- [x] GCP service account authentication
- [x] OAuth2 token refresh
- [x] Key validation before use

---

## Implementation Details

### 1. Authentication Types (src/auth/types.rs)

```rust
//! Authentication types for Gemini API.

use serde::{Deserialize, Serialize};
use std::fmt;
use tachikoma_common_config::Secret;

/// API key for Gemini authentication.
#[derive(Clone)]
pub struct GeminiApiKey {
    inner: Secret<String>,
    key_id: String,
}

impl GeminiApiKey {
    /// Create a new API key.
    pub fn new(key: impl Into<String>) -> Result<Self, AuthError> {
        let key = key.into();
        Self::validate(&key)?;

        let key_id = Self::derive_key_id(&key);

        Ok(Self {
            inner: Secret::new(key),
            key_id,
        })
    }

    fn validate(key: &str) -> Result<(), AuthError> {
        if key.is_empty() {
            return Err(AuthError::EmptyKey);
        }

        // Google API keys are typically 39 characters
        if key.len() < 20 {
            return Err(AuthError::InvalidFormat(
                "API key appears too short".to_string(),
            ));
        }

        Ok(())
    }

    fn derive_key_id(key: &str) -> String {
        if key.len() >= 12 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else {
            "***".to_string()
        }
    }

    /// Get the key for use in API calls.
    pub fn expose(&self) -> &str {
        self.inner.expose()
    }

    /// Get the key identifier.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

impl fmt::Debug for GeminiApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeminiApiKey")
            .field("key_id", &self.key_id)
            .finish()
    }
}

/// Service account credentials for GCP.
#[derive(Clone)]
pub struct ServiceAccountCredentials {
    /// Project ID.
    pub project_id: String,
    /// Client email.
    pub client_email: String,
    /// Private key.
    private_key: Secret<String>,
    /// Token endpoint.
    pub token_uri: String,
}

impl ServiceAccountCredentials {
    /// Load from JSON file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, AuthError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AuthError::FileReadError(e.to_string()))?;

        Self::from_json(&content)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, AuthError> {
        #[derive(Deserialize)]
        struct ServiceAccountJson {
            project_id: String,
            client_email: String,
            private_key: String,
            token_uri: String,
        }

        let parsed: ServiceAccountJson = serde_json::from_str(json)
            .map_err(|e| AuthError::InvalidFormat(e.to_string()))?;

        Ok(Self {
            project_id: parsed.project_id,
            client_email: parsed.client_email,
            private_key: Secret::new(parsed.private_key),
            token_uri: parsed.token_uri,
        })
    }

    /// Get the private key for signing.
    pub fn private_key(&self) -> &str {
        self.private_key.expose()
    }
}

impl fmt::Debug for ServiceAccountCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceAccountCredentials")
            .field("project_id", &self.project_id)
            .field("client_email", &self.client_email)
            .finish()
    }
}

/// OAuth2 access token.
#[derive(Clone)]
pub struct AccessToken {
    token: Secret<String>,
    expires_at: std::time::Instant,
}

impl AccessToken {
    /// Create a new access token.
    pub fn new(token: String, expires_in_secs: u64) -> Self {
        Self {
            token: Secret::new(token),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(expires_in_secs.saturating_sub(60)),
        }
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        std::time::Instant::now() >= self.expires_at
    }

    /// Get the token value.
    pub fn expose(&self) -> &str {
        self.token.expose()
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccessToken")
            .field("expired", &self.is_expired())
            .finish()
    }
}

/// Authentication errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("API key is empty")]
    EmptyKey,

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("credentials not found in environment")]
    NotInEnvironment,

    #[error("failed to read credentials: {0}")]
    FileReadError(String),

    #[error("token refresh failed: {0}")]
    TokenRefreshFailed(String),
}
```

### 2. Key Loading (src/auth/loader.rs)

```rust
//! Credential loading for Gemini.

use super::{AuthError, GeminiApiKey, ServiceAccountCredentials};
use tracing::{debug, info};

/// Load API key from environment.
pub fn load_api_key_from_env() -> Result<GeminiApiKey, AuthError> {
    debug!("Loading Gemini API key from environment");

    // Try GOOGLE_API_KEY first, then GEMINI_API_KEY
    let key = std::env::var("GOOGLE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .map_err(|_| AuthError::NotInEnvironment)?;

    GeminiApiKey::new(key)
}

/// Load service account from environment.
pub fn load_service_account_from_env() -> Result<ServiceAccountCredentials, AuthError> {
    debug!("Loading GCP service account from environment");

    // Check for GOOGLE_APPLICATION_CREDENTIALS
    let path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .map_err(|_| AuthError::NotInEnvironment)?;

    ServiceAccountCredentials::from_file(std::path::Path::new(&path))
}

/// Try to load any available credentials.
pub fn load_credentials() -> Result<GeminiCredentials, AuthError> {
    // Try API key first
    if let Ok(api_key) = load_api_key_from_env() {
        info!(key_id = %api_key.key_id(), "Using API key authentication");
        return Ok(GeminiCredentials::ApiKey(api_key));
    }

    // Try service account
    if let Ok(sa) = load_service_account_from_env() {
        info!(
            project = %sa.project_id,
            email = %sa.client_email,
            "Using service account authentication"
        );
        return Ok(GeminiCredentials::ServiceAccount(sa));
    }

    Err(AuthError::NotInEnvironment)
}

/// Loaded credentials.
#[derive(Debug, Clone)]
pub enum GeminiCredentials {
    ApiKey(GeminiApiKey),
    ServiceAccount(ServiceAccountCredentials),
}
```

### 3. Token Manager (src/auth/token.rs)

```rust
//! OAuth2 token management for GCP.

use super::{AccessToken, AuthError, ServiceAccountCredentials};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Token manager for GCP OAuth2.
pub struct TokenManager {
    credentials: ServiceAccountCredentials,
    client: Client,
    token: Arc<RwLock<Option<AccessToken>>>,
}

impl TokenManager {
    /// Create a new token manager.
    pub fn new(credentials: ServiceAccountCredentials) -> Self {
        Self {
            credentials,
            client: Client::new(),
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a valid access token, refreshing if necessary.
    pub async fn get_token(&self) -> Result<String, AuthError> {
        // Check if we have a valid cached token
        {
            let token = self.token.read().await;
            if let Some(t) = &*token {
                if !t.is_expired() {
                    return Ok(t.expose().to_string());
                }
            }
        }

        // Need to refresh
        self.refresh_token().await
    }

    /// Refresh the access token.
    async fn refresh_token(&self) -> Result<String, AuthError> {
        debug!("Refreshing OAuth2 access token");

        let jwt = self.create_jwt()?;

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: u64,
        }

        let response = self
            .client
            .post(&self.credentials.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::TokenRefreshFailed(body));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;

        let token = AccessToken::new(token_response.access_token.clone(), token_response.expires_in);

        info!("Successfully refreshed access token");

        // Cache the token
        {
            let mut cached = self.token.write().await;
            *cached = Some(token);
        }

        Ok(token_response.access_token)
    }

    /// Create a signed JWT for authentication.
    fn create_jwt(&self) -> Result<String, AuthError> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        use serde::Serialize;

        #[derive(Serialize)]
        struct Claims {
            iss: String,
            scope: String,
            aud: String,
            iat: i64,
            exp: i64,
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let claims = Claims {
            iss: self.credentials.client_email.clone(),
            scope: "https://www.googleapis.com/auth/generative-language".to_string(),
            aud: self.credentials.token_uri.clone(),
            iat: now,
            exp: now + 3600,
        };

        // Create JWT header
        let header = serde_json::json!({
            "alg": "RS256",
            "typ": "JWT"
        });

        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());

        let message = format!("{}.{}", header_b64, claims_b64);

        // Sign with RSA private key
        let signature = self.sign_rs256(&message)?;
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

        Ok(format!("{}.{}", message, signature_b64))
    }

    /// Sign data with RS256.
    fn sign_rs256(&self, data: &str) -> Result<Vec<u8>, AuthError> {
        use ring::{rand, signature};

        let key_pem = self.credentials.private_key();

        // Parse PEM private key
        let pem = pem::parse(key_pem)
            .map_err(|e| AuthError::InvalidFormat(format!("Invalid PEM: {}", e)))?;

        let key_pair = signature::RsaKeyPair::from_pkcs8(&pem.contents())
            .map_err(|e| AuthError::InvalidFormat(format!("Invalid key: {}", e)))?;

        let rng = rand::SystemRandom::new();
        let mut signature = vec![0; key_pair.public().modulus_len()];

        key_pair
            .sign(
                &signature::RSA_PKCS1_SHA256,
                &rng,
                data.as_bytes(),
                &mut signature,
            )
            .map_err(|e| AuthError::TokenRefreshFailed(format!("Signing failed: {}", e)))?;

        Ok(signature)
    }
}

impl std::fmt::Debug for TokenManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenManager")
            .field("project_id", &self.credentials.project_id)
            .finish()
    }
}
```

### 4. Authentication Provider (src/auth/provider.rs)

```rust
//! High-level authentication provider for Gemini.

use super::{
    load_credentials, AccessToken, AuthError, GeminiApiKey, GeminiCredentials,
    ServiceAccountCredentials, TokenManager,
};
use std::sync::Arc;
use tracing::debug;

/// Authentication provider for Gemini API.
pub struct GeminiAuthProvider {
    credentials: GeminiCredentials,
    token_manager: Option<Arc<TokenManager>>,
}

impl GeminiAuthProvider {
    /// Create from environment.
    pub fn from_env() -> Result<Self, AuthError> {
        let credentials = load_credentials()?;

        let token_manager = match &credentials {
            GeminiCredentials::ServiceAccount(sa) => {
                Some(Arc::new(TokenManager::new(sa.clone())))
            }
            GeminiCredentials::ApiKey(_) => None,
        };

        Ok(Self {
            credentials,
            token_manager,
        })
    }

    /// Create with API key.
    pub fn with_api_key(key: impl Into<String>) -> Result<Self, AuthError> {
        let api_key = GeminiApiKey::new(key)?;
        Ok(Self {
            credentials: GeminiCredentials::ApiKey(api_key),
            token_manager: None,
        })
    }

    /// Create with service account.
    pub fn with_service_account(credentials: ServiceAccountCredentials) -> Self {
        let token_manager = Arc::new(TokenManager::new(credentials.clone()));
        Self {
            credentials: GeminiCredentials::ServiceAccount(credentials),
            token_manager: Some(token_manager),
        }
    }

    /// Check if using API key authentication.
    pub fn is_api_key(&self) -> bool {
        matches!(self.credentials, GeminiCredentials::ApiKey(_))
    }

    /// Get the API key (if using API key auth).
    pub fn api_key(&self) -> Option<&GeminiApiKey> {
        match &self.credentials {
            GeminiCredentials::ApiKey(key) => Some(key),
            _ => None,
        }
    }

    /// Get an access token (if using service account).
    pub async fn get_access_token(&self) -> Result<Option<String>, AuthError> {
        match &self.token_manager {
            Some(tm) => Ok(Some(tm.get_token().await?)),
            None => Ok(None),
        }
    }

    /// Build the URL with authentication.
    pub fn authenticate_url(&self, base_url: &str) -> String {
        match &self.credentials {
            GeminiCredentials::ApiKey(key) => {
                if base_url.contains('?') {
                    format!("{}&key={}", base_url, key.expose())
                } else {
                    format!("{}?key={}", base_url, key.expose())
                }
            }
            GeminiCredentials::ServiceAccount(_) => {
                // OAuth uses headers, not URL params
                base_url.to_string()
            }
        }
    }

    /// Build authorization header (for service account).
    pub async fn authorization_header(&self) -> Result<Option<String>, AuthError> {
        if let Some(token) = self.get_access_token().await? {
            Ok(Some(format!("Bearer {}", token)))
        } else {
            Ok(None)
        }
    }
}

impl std::fmt::Debug for GeminiAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiAuthProvider")
            .field("auth_type", &if self.is_api_key() { "api_key" } else { "service_account" })
            .finish()
    }
}
```

### 5. Module Exports (src/auth/mod.rs)

```rust
//! Authentication module for Gemini API.

mod loader;
mod provider;
mod token;
mod types;

pub use loader::{load_api_key_from_env, load_credentials, load_service_account_from_env, GeminiCredentials};
pub use provider::GeminiAuthProvider;
pub use token::TokenManager;
pub use types::{AccessToken, AuthError, GeminiApiKey, ServiceAccountCredentials};
```

---

## Testing Requirements

1. API key validation works
2. Service account loading from file works
3. JWT generation produces valid tokens
4. Token refresh happens on expiry
5. URL authentication is correct

---

## Related Specs

- Depends on: [064-gemini-api-client.md](064-gemini-api-client.md)
- Next: [066-gemini-tools.md](066-gemini-tools.md)
