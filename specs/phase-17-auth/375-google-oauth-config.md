# Spec 375: Google OAuth Configuration

## Overview
Define configuration and types for Google OAuth authentication integration.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Google OAuth Types
```rust
// src/auth/oauth/google/types.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Google OAuth configuration
#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    /// Google OAuth Client ID
    pub client_id: String,
    /// Google OAuth Client Secret
    pub client_secret: String,
    /// Redirect URI after authorization
    pub redirect_uri: String,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// Google authorization endpoint
    pub authorize_url: String,
    /// Google token endpoint
    pub token_url: String,
    /// Google userinfo endpoint
    pub userinfo_url: String,
    /// Allow signup from Google OAuth
    pub allow_signup: bool,
    /// Restrict to specific hosted domains (G Suite)
    pub hosted_domain: Option<String>,
    /// Require verified email
    pub require_verified_email: bool,
    /// Prompt type (none, consent, select_account)
    pub prompt: Option<String>,
    /// Access type (online, offline for refresh token)
    pub access_type: String,
}

impl Default for GoogleOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: "http://localhost:8080/auth/google/callback".to_string(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            authorize_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: "https://www.googleapis.com/oauth2/v3/userinfo".to_string(),
            allow_signup: true,
            hosted_domain: None,
            require_verified_email: true,
            prompt: Some("select_account".to_string()),
            access_type: "offline".to_string(),
        }
    }
}

impl GoogleOAuthConfig {
    /// Create config for G Suite domain restriction
    pub fn for_domain(domain: &str) -> Self {
        Self {
            hosted_domain: Some(domain.to_string()),
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    /// Generate authorization URL with state and nonce
    pub fn authorization_url(&self, state: &str, nonce: Option<&str>) -> String {
        let scopes = self.scopes.join(" ");
        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type={}",
            self.authorize_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state),
            urlencoding::encode(&self.access_type),
        );

        if let Some(hd) = &self.hosted_domain {
            url.push_str(&format!("&hd={}", urlencoding::encode(hd)));
        }

        if let Some(prompt) = &self.prompt {
            url.push_str(&format!("&prompt={}", urlencoding::encode(prompt)));
        }

        if let Some(nonce) = nonce {
            url.push_str(&format!("&nonce={}", urlencoding::encode(nonce)));
        }

        url
    }
}

/// Google OAuth token response
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Google user info from userinfo endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleUserInfo {
    pub sub: String,  // Google user ID
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
    pub hd: Option<String>,  // Hosted domain (G Suite)
}

/// Google ID token claims (when using OpenID Connect)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleIdTokenClaims {
    // Standard OIDC claims
    pub iss: String,
    pub azp: String,
    pub aud: String,
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub at_hash: Option<String>,
    pub nonce: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub locale: Option<String>,
    pub iat: i64,
    pub exp: i64,
    // G Suite specific
    pub hd: Option<String>,
}

impl GoogleIdTokenClaims {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }

    pub fn validate_audience(&self, expected: &str) -> bool {
        self.aud == expected
    }

    pub fn validate_issuer(&self) -> bool {
        self.iss == "https://accounts.google.com" || self.iss == "accounts.google.com"
    }
}

/// Google OAuth state with nonce for OIDC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthState {
    pub state: String,
    pub nonce: Option<String>,  // For ID token validation
    pub redirect_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl GoogleOAuthState {
    pub fn new(redirect_to: Option<String>, include_nonce: bool) -> Self {
        use uuid::Uuid;
        let now = Utc::now();

        Self {
            state: Uuid::new_v4().to_string(),
            nonce: if include_nonce {
                Some(Uuid::new_v4().to_string())
            } else {
                None
            },
            redirect_to,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(10),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Google-specific errors
#[derive(Debug, thiserror::Error)]
pub enum GoogleOAuthError {
    #[error("Invalid ID token")]
    InvalidIdToken,

    #[error("ID token expired")]
    IdTokenExpired,

    #[error("Invalid issuer")]
    InvalidIssuer,

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("Nonce mismatch")]
    NonceMismatch,

    #[error("Hosted domain mismatch: expected {expected}, got {actual:?}")]
    HostedDomainMismatch {
        expected: String,
        actual: Option<String>,
    },

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("OAuth error: {0}")]
    OAuth(#[from] crate::auth::oauth::types::OAuthError),
}

/// Google OIDC Discovery document
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOIDCDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub revocation_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

impl GoogleOIDCDiscovery {
    pub const DISCOVERY_URL: &'static str =
        "https://accounts.google.com/.well-known/openid-configuration";
}

/// Google JWK (JSON Web Key) for ID token validation
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleJwk {
    pub kty: String,
    pub alg: String,
    pub kid: String,
    pub n: String,
    pub e: String,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
}

/// Google JWKS (JSON Web Key Set)
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleJwks {
    pub keys: Vec<GoogleJwk>,
}

impl GoogleJwks {
    pub const JWKS_URL: &'static str = "https://www.googleapis.com/oauth2/v3/certs";

    pub fn find_key(&self, kid: &str) -> Option<&GoogleJwk> {
        self.keys.iter().find(|k| k.kid == kid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_config_default() {
        let config = GoogleOAuthConfig::default();
        assert!(!config.is_valid());
        assert!(config.scopes.contains(&"openid".to_string()));
        assert!(config.scopes.contains(&"email".to_string()));
    }

    #[test]
    fn test_authorization_url() {
        let config = GoogleOAuthConfig {
            client_id: "test-client".to_string(),
            client_secret: "secret".to_string(),
            ..Default::default()
        };

        let url = config.authorization_url("test-state", Some("test-nonce"));
        assert!(url.contains("client_id=test-client"));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("nonce=test-nonce"));
        assert!(url.contains("access_type=offline"));
    }

    #[test]
    fn test_domain_restriction() {
        let config = GoogleOAuthConfig::for_domain("example.com");
        assert_eq!(config.hosted_domain, Some("example.com".to_string()));

        let url = config.authorization_url("state", None);
        assert!(url.contains("hd=example.com"));
    }

    #[test]
    fn test_oauth_state() {
        let state = GoogleOAuthState::new(Some("/dashboard".to_string()), true);
        assert!(state.nonce.is_some());
        assert!(!state.is_expired());
    }
}
```

## Files to Create
- `src/auth/oauth/google/types.rs` - Google OAuth types
- `src/auth/oauth/google/mod.rs` - Module exports
