# Spec 380: Okta OAuth Configuration

## Overview
Define configuration and types for Okta OAuth/OIDC authentication integration for enterprise SSO.

## Rust Implementation

### Okta OAuth Types
```rust
// src/auth/oauth/okta/types.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Okta OAuth/OIDC configuration
#[derive(Debug, Clone)]
pub struct OktaOAuthConfig {
    /// Okta organization domain (e.g., "your-org.okta.com")
    pub okta_domain: String,
    /// OAuth Client ID
    pub client_id: String,
    /// OAuth Client Secret
    pub client_secret: String,
    /// Redirect URI after authorization
    pub redirect_uri: String,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// Authorization server ID (default or custom)
    pub authorization_server_id: Option<String>,
    /// Allow signup from Okta OAuth
    pub allow_signup: bool,
    /// Require verified email
    pub require_verified_email: bool,
    /// Restrict to specific groups
    pub allowed_groups: Option<Vec<String>>,
    /// Session idle timeout (from Okta policy)
    pub session_idle_timeout: Option<chrono::Duration>,
    /// Use PKCE (Proof Key for Code Exchange)
    pub use_pkce: bool,
}

impl OktaOAuthConfig {
    /// Create new config with minimal required fields
    pub fn new(okta_domain: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            okta_domain: okta_domain.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            redirect_uri: "http://localhost:8080/auth/okta/callback".to_string(),
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "groups".to_string(),
            ],
            authorization_server_id: None, // Use default
            allow_signup: true,
            require_verified_email: true,
            allowed_groups: None,
            session_idle_timeout: None,
            use_pkce: true,
        }
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        !self.okta_domain.is_empty() &&
        !self.client_id.is_empty() &&
        !self.client_secret.is_empty()
    }

    /// Get issuer URL
    pub fn issuer(&self) -> String {
        match &self.authorization_server_id {
            Some(server_id) => format!("https://{}/oauth2/{}", self.okta_domain, server_id),
            None => format!("https://{}/oauth2/default", self.okta_domain),
        }
    }

    /// Get authorization endpoint
    pub fn authorize_url(&self) -> String {
        format!("{}/v1/authorize", self.issuer())
    }

    /// Get token endpoint
    pub fn token_url(&self) -> String {
        format!("{}/v1/token", self.issuer())
    }

    /// Get userinfo endpoint
    pub fn userinfo_url(&self) -> String {
        format!("{}/v1/userinfo", self.issuer())
    }

    /// Get JWKS URI
    pub fn jwks_uri(&self) -> String {
        format!("{}/v1/keys", self.issuer())
    }

    /// Get logout URL
    pub fn logout_url(&self, id_token_hint: Option<&str>, post_logout_redirect: Option<&str>) -> String {
        let mut url = format!("{}/v1/logout", self.issuer());

        let mut params = Vec::new();
        if let Some(token) = id_token_hint {
            params.push(format!("id_token_hint={}", urlencoding::encode(token)));
        }
        if let Some(redirect) = post_logout_redirect {
            params.push(format!("post_logout_redirect_uri={}", urlencoding::encode(redirect)));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }

    /// Generate authorization URL
    pub fn authorization_url(&self, state: &str, nonce: Option<&str>, pkce_challenge: Option<&str>) -> String {
        let scopes = self.scopes.join(" ");
        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.authorize_url(),
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state),
        );

        if let Some(nonce) = nonce {
            url.push_str(&format!("&nonce={}", urlencoding::encode(nonce)));
        }

        if self.use_pkce {
            if let Some(challenge) = pkce_challenge {
                url.push_str(&format!(
                    "&code_challenge={}&code_challenge_method=S256",
                    urlencoding::encode(challenge)
                ));
            }
        }

        url
    }
}

/// Okta token response
#[derive(Debug, Clone, Deserialize)]
pub struct OktaTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    pub id_token: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Okta userinfo response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OktaUserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub preferred_username: Option<String>,
    pub nickname: Option<String>,
    pub groups: Option<Vec<String>>,
    pub locale: Option<String>,
    pub zoneinfo: Option<String>,
    pub updated_at: Option<i64>,
}

/// Okta ID token claims
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OktaIdTokenClaims {
    // Standard OIDC claims
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: Option<i64>,
    pub nonce: Option<String>,
    pub at_hash: Option<String>,
    pub c_hash: Option<String>,

    // Profile claims
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub preferred_username: Option<String>,

    // Okta-specific claims
    pub groups: Option<Vec<String>>,
    pub amr: Option<Vec<String>>,  // Authentication methods
    pub idp: Option<String>,       // Identity provider
    pub ver: Option<i32>,          // Token version
    pub jti: Option<String>,       // JWT ID
}

impl OktaIdTokenClaims {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.exp
    }

    pub fn validate_audience(&self, expected: &str) -> bool {
        self.aud == expected
    }

    pub fn validate_issuer(&self, expected: &str) -> bool {
        self.iss == expected
    }

    pub fn has_group(&self, group: &str) -> bool {
        self.groups.as_ref()
            .map(|groups| groups.iter().any(|g| g == group))
            .unwrap_or(false)
    }

    pub fn has_any_group(&self, allowed: &[String]) -> bool {
        self.groups.as_ref()
            .map(|groups| groups.iter().any(|g| allowed.contains(g)))
            .unwrap_or(false)
    }
}

/// Okta OAuth state with PKCE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaOAuthState {
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: Option<String>,
    pub redirect_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl OktaOAuthState {
    pub fn new(redirect_to: Option<String>, use_pkce: bool) -> Self {
        use uuid::Uuid;
        let now = Utc::now();

        Self {
            state: Uuid::new_v4().to_string(),
            nonce: Uuid::new_v4().to_string(),
            pkce_verifier: if use_pkce {
                Some(Self::generate_pkce_verifier())
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

    /// Generate PKCE verifier (43-128 characters)
    fn generate_pkce_verifier() -> String {
        use rand::Rng;
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Generate PKCE challenge from verifier
    pub fn pkce_challenge(&self) -> Option<String> {
        self.pkce_verifier.as_ref().map(|verifier| {
            use sha2::{Sha256, Digest};
            use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

            let mut hasher = Sha256::new();
            hasher.update(verifier.as_bytes());
            URL_SAFE_NO_PAD.encode(hasher.finalize())
        })
    }
}

/// Okta-specific errors
#[derive(Debug, thiserror::Error)]
pub enum OktaOAuthError {
    #[error("Invalid ID token")]
    InvalidIdToken,

    #[error("ID token expired")]
    IdTokenExpired,

    #[error("Invalid issuer: expected {expected}, got {actual}")]
    InvalidIssuer { expected: String, actual: String },

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("Nonce mismatch")]
    NonceMismatch,

    #[error("User not in allowed group")]
    GroupNotAllowed,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("PKCE verification failed")]
    PkceVerificationFailed,

    #[error("OAuth error: {0}")]
    OAuth(#[from] crate::auth::oauth::types::OAuthError),
}

/// Okta session info (for session management)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaSession {
    pub okta_user_id: String,
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub groups: Vec<String>,
    pub auth_methods: Vec<String>,
    pub idp: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl OktaSession {
    pub fn from_tokens(
        claims: &OktaIdTokenClaims,
        tokens: &OktaTokenResponse,
    ) -> Self {
        Self {
            okta_user_id: claims.sub.clone(),
            id_token: tokens.id_token.clone(),
            access_token: tokens.access_token.clone(),
            refresh_token: tokens.refresh_token.clone(),
            groups: claims.groups.clone().unwrap_or_default(),
            auth_methods: claims.amr.clone().unwrap_or_default(),
            idp: claims.idp.clone(),
            expires_at: DateTime::from_timestamp(claims.exp, 0)
                .unwrap_or_else(Utc::now),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okta_config() {
        let config = OktaOAuthConfig::new(
            "dev-123456.okta.com",
            "client-id",
            "client-secret"
        );

        assert!(config.is_valid());
        assert_eq!(config.issuer(), "https://dev-123456.okta.com/oauth2/default");
    }

    #[test]
    fn test_custom_authorization_server() {
        let mut config = OktaOAuthConfig::new(
            "dev-123456.okta.com",
            "client-id",
            "client-secret"
        );
        config.authorization_server_id = Some("custom-server".to_string());

        assert_eq!(
            config.issuer(),
            "https://dev-123456.okta.com/oauth2/custom-server"
        );
    }

    #[test]
    fn test_pkce_generation() {
        let state = OktaOAuthState::new(None, true);

        assert!(state.pkce_verifier.is_some());
        assert!(state.pkce_challenge().is_some());

        let verifier = state.pkce_verifier.as_ref().unwrap();
        assert!(verifier.len() >= 43);
    }

    #[test]
    fn test_group_validation() {
        let claims = OktaIdTokenClaims {
            iss: "https://test.okta.com".to_string(),
            sub: "user123".to_string(),
            aud: "client-id".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
            auth_time: None,
            nonce: None,
            at_hash: None,
            c_hash: None,
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
            given_name: None,
            family_name: None,
            preferred_username: None,
            groups: Some(vec!["Admins".to_string(), "Users".to_string()]),
            amr: None,
            idp: None,
            ver: None,
            jti: None,
        };

        assert!(claims.has_group("Admins"));
        assert!(!claims.has_group("SuperAdmins"));
        assert!(claims.has_any_group(&["Admins".to_string(), "Guests".to_string()]));
    }
}
```

## Files to Create
- `src/auth/oauth/okta/types.rs` - Okta OAuth types
- `src/auth/oauth/okta/mod.rs` - Module exports
