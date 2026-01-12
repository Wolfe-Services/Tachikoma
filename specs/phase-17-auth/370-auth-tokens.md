# Spec 370: JWT Token Management

## Phase
17 - Authentication/Authorization

## Spec ID
370

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~12%

---

## Objective

Implement JWT (JSON Web Token) creation, validation, and management for stateless authentication. This includes access tokens for API authentication and the infrastructure for refresh tokens. The implementation should support multiple signing algorithms (HMAC, RSA) and include proper claims validation.

---

## Acceptance Criteria

- [ ] Implement `TokenManager` for JWT operations
- [ ] Support HS256, HS384, HS512 signing algorithms
- [ ] Support RS256, RS384, RS512 signing algorithms
- [ ] Create access tokens with configurable claims
- [ ] Validate tokens with proper error handling
- [ ] Extract and validate standard JWT claims (iss, aud, exp, iat, sub)
- [ ] Support custom claims for roles and permissions
- [ ] Implement token blacklisting for revocation
- [ ] Provide secure token encoding/decoding

---

## Implementation Details

### JWT Token Manager

```rust
// src/auth/tokens.rs

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, instrument};

use crate::auth::{
    config::TokenConfig,
    types::*,
};

/// Standard JWT claims plus custom claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: Vec<String>,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Not before (Unix timestamp)
    pub nbf: i64,

    /// JWT ID (unique identifier)
    pub jti: String,

    /// Token type (access or refresh)
    pub token_type: TokenType,

    /// User roles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,

    /// User permissions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,

    /// Session ID (if linked to a session)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Additional custom claims
    #[serde(flatten)]
    pub custom: serde_json::Map<String, serde_json::Value>,
}

impl TokenClaims {
    /// Get user ID from subject
    pub fn user_id(&self) -> AuthResult<UserId> {
        UserId::parse(&self.sub).map_err(|_| AuthError::TokenInvalid("Invalid subject".to_string()))
    }

    /// Get session ID if present
    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id.as_ref().and_then(|s| {
            s.parse::<uuid::Uuid>().ok().map(SessionId::from)
        })
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Convert to AuthIdentity
    pub fn to_identity(&self) -> AuthResult<AuthIdentity> {
        Ok(AuthIdentity {
            user_id: self.user_id()?,
            username: self.username.clone().unwrap_or_default(),
            display_name: None,
            email: self.email.clone(),
            email_verified: false,
            roles: self.roles.iter().cloned().collect(),
            permissions: self.permissions.iter().cloned().collect(),
            auth_method: AuthMethod::Jwt,
            authenticated_at: DateTime::from_timestamp(self.iat, 0)
                .unwrap_or_else(Utc::now),
            session_id: self.session_id(),
            claims: serde_json::to_value(&self.custom).unwrap_or_default(),
        })
    }
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token pair returned after successful authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Access token for API authentication
    pub access_token: String,

    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiration in seconds
    pub expires_in: u64,

    /// Refresh token expiration in seconds
    pub refresh_expires_in: u64,
}

/// JWT Token Manager
pub struct TokenManager {
    config: TokenConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
    validation: Validation,
    blacklist: Arc<RwLock<TokenBlacklist>>,
}

impl TokenManager {
    /// Create a new token manager with HMAC signing
    pub fn new_hmac(config: TokenConfig) -> AuthResult<Self> {
        let algorithm = match config.algorithm.as_str() {
            "HS256" => Algorithm::HS256,
            "HS384" => Algorithm::HS384,
            "HS512" => Algorithm::HS512,
            alg => {
                return Err(AuthError::ConfigError(format!(
                    "Unsupported HMAC algorithm: {}",
                    alg
                )))
            }
        };

        let encoding_key = EncodingKey::from_secret(config.secret_key.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret_key.as_bytes());

        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&config.audience);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            algorithm,
            validation,
            blacklist: Arc::new(RwLock::new(TokenBlacklist::new())),
        })
    }

    /// Create a new token manager with RSA signing
    pub fn new_rsa(config: TokenConfig) -> AuthResult<Self> {
        let algorithm = match config.algorithm.as_str() {
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            alg => {
                return Err(AuthError::ConfigError(format!(
                    "Unsupported RSA algorithm: {}",
                    alg
                )))
            }
        };

        let private_key_path = config.private_key_path.as_ref().ok_or_else(|| {
            AuthError::ConfigError("RSA private key path required".to_string())
        })?;
        let public_key_path = config.public_key_path.as_ref().ok_or_else(|| {
            AuthError::ConfigError("RSA public key path required".to_string())
        })?;

        let private_key = std::fs::read(private_key_path).map_err(|e| {
            AuthError::ConfigError(format!("Failed to read private key: {}", e))
        })?;
        let public_key = std::fs::read(public_key_path).map_err(|e| {
            AuthError::ConfigError(format!("Failed to read public key: {}", e))
        })?;

        let encoding_key = EncodingKey::from_rsa_pem(&private_key).map_err(|e| {
            AuthError::ConfigError(format!("Invalid private key: {}", e))
        })?;
        let decoding_key = DecodingKey::from_rsa_pem(&public_key).map_err(|e| {
            AuthError::ConfigError(format!("Invalid public key: {}", e))
        })?;

        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&config.audience);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            algorithm,
            validation,
            blacklist: Arc::new(RwLock::new(TokenBlacklist::new())),
        })
    }

    /// Create token manager based on algorithm in config
    pub fn from_config(config: TokenConfig) -> AuthResult<Self> {
        if config.algorithm.starts_with("RS") {
            Self::new_rsa(config)
        } else {
            Self::new_hmac(config)
        }
    }

    /// Generate an access token for the given identity
    #[instrument(skip(self, identity), fields(user_id = %identity.user_id))]
    pub fn create_access_token(&self, identity: &AuthIdentity) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.access_token_lifetime_secs as i64);

        let claims = TokenClaims {
            sub: identity.user_id.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Access,
            roles: if self.config.include_roles {
                identity.roles.iter().cloned().collect()
            } else {
                vec![]
            },
            permissions: if self.config.include_permissions {
                identity.permissions.iter().cloned().collect()
            } else {
                vec![]
            },
            session_id: identity.session_id.map(|s| s.to_string()),
            username: Some(identity.username.clone()),
            email: identity.email.clone(),
            custom: serde_json::Map::new(),
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(|e| {
            AuthError::Internal(format!("Token encoding failed: {}", e))
        })?;

        debug!(jti = %claims.jti, "Access token created");
        Ok(token)
    }

    /// Generate a refresh token
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub fn create_refresh_token(
        &self,
        user_id: UserId,
        session_id: Option<SessionId>,
    ) -> AuthResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.refresh_token_lifetime_secs as i64);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            token_type: TokenType::Refresh,
            roles: vec![],
            permissions: vec![],
            session_id: session_id.map(|s| s.to_string()),
            username: None,
            email: None,
            custom: serde_json::Map::new(),
        };

        let header = Header::new(self.algorithm);
        let token = encode(&header, &claims, &self.encoding_key).map_err(|e| {
            AuthError::Internal(format!("Token encoding failed: {}", e))
        })?;

        debug!(jti = %claims.jti, "Refresh token created");
        Ok(token)
    }

    /// Create a token pair (access + refresh)
    pub fn create_token_pair(
        &self,
        identity: &AuthIdentity,
    ) -> AuthResult<TokenPair> {
        let access_token = self.create_access_token(identity)?;
        let refresh_token = self.create_refresh_token(
            identity.user_id,
            identity.session_id,
        )?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_lifetime_secs,
            refresh_expires_in: self.config.refresh_token_lifetime_secs,
        })
    }

    /// Validate and decode a token
    #[instrument(skip(self, token))]
    pub async fn validate_token(&self, token: &str) -> AuthResult<TokenClaims> {
        // Decode and validate
        let token_data: TokenData<TokenClaims> =
            decode(token, &self.decoding_key, &self.validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        AuthError::TokenExpired
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidToken => {
                        AuthError::TokenInvalid("Invalid token format".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        AuthError::TokenInvalid("Invalid signature".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                        AuthError::TokenInvalid("Invalid issuer".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                        AuthError::TokenInvalid("Invalid audience".to_string())
                    }
                    _ => AuthError::TokenInvalid(format!("Token validation failed: {}", e)),
                }
            })?;

        let claims = token_data.claims;

        // Check blacklist
        if self.is_blacklisted(&claims.jti).await {
            warn!(jti = %claims.jti, "Attempted use of blacklisted token");
            return Err(AuthError::TokenInvalid("Token has been revoked".to_string()));
        }

        Ok(claims)
    }

    /// Validate an access token and return the identity
    pub async fn validate_access_token(&self, token: &str) -> AuthResult<AuthIdentity> {
        let claims = self.validate_token(token).await?;

        if claims.token_type != TokenType::Access {
            return Err(AuthError::TokenInvalid("Not an access token".to_string()));
        }

        claims.to_identity()
    }

    /// Validate a refresh token
    pub async fn validate_refresh_token(&self, token: &str) -> AuthResult<TokenClaims> {
        let claims = self.validate_token(token).await?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::TokenInvalid("Not a refresh token".to_string()));
        }

        Ok(claims)
    }

    /// Blacklist a token (revoke)
    #[instrument(skip(self))]
    pub async fn blacklist_token(&self, jti: &str, exp: i64) {
        let mut blacklist = self.blacklist.write().await;
        blacklist.add(jti.to_string(), exp);
        debug!(jti = %jti, "Token blacklisted");
    }

    /// Check if a token is blacklisted
    async fn is_blacklisted(&self, jti: &str) -> bool {
        let blacklist = self.blacklist.read().await;
        blacklist.contains(jti)
    }

    /// Revoke a token by its string representation
    pub async fn revoke_token(&self, token: &str) -> AuthResult<()> {
        // Try to decode without validation to get claims
        let claims = self.validate_token(token).await?;
        self.blacklist_token(&claims.jti, claims.exp).await;
        Ok(())
    }

    /// Clean up expired entries from blacklist
    pub async fn cleanup_blacklist(&self) -> usize {
        let mut blacklist = self.blacklist.write().await;
        blacklist.cleanup()
    }
}

/// Token blacklist for revoked tokens
struct TokenBlacklist {
    /// Map of JTI to expiration timestamp
    tokens: std::collections::HashMap<String, i64>,
}

impl TokenBlacklist {
    fn new() -> Self {
        Self {
            tokens: std::collections::HashMap::new(),
        }
    }

    fn add(&mut self, jti: String, exp: i64) {
        self.tokens.insert(jti, exp);
    }

    fn contains(&self, jti: &str) -> bool {
        self.tokens.contains_key(jti)
    }

    /// Remove expired entries and return count of removed
    fn cleanup(&mut self) -> usize {
        let now = Utc::now().timestamp();
        let before = self.tokens.len();
        self.tokens.retain(|_, exp| *exp > now);
        before - self.tokens.len()
    }
}

/// JWT Authentication Provider
pub struct JwtAuthProvider {
    token_manager: Arc<TokenManager>,
}

impl JwtAuthProvider {
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self { token_manager }
    }
}

#[async_trait::async_trait]
impl super::provider::AuthProvider for JwtAuthProvider {
    fn name(&self) -> &str {
        "jwt"
    }

    async fn authenticate(
        &self,
        credentials: &AuthCredentials,
        _metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity> {
        match credentials {
            AuthCredentials::BearerToken { token } => {
                self.token_manager
                    .validate_access_token(token.expose())
                    .await
            }
            _ => Err(AuthError::InvalidCredentials),
        }
    }

    async fn validate(&self, identity: &AuthIdentity) -> AuthResult<bool> {
        // JWT is self-contained, if we got here it's valid
        // Additional validation could check user status
        Ok(true)
    }

    async fn revoke(&self, _identity: &AuthIdentity) -> AuthResult<()> {
        // Would need the original token to revoke
        // This is typically handled at the session level
        Ok(())
    }

    fn supports(&self, credentials: &AuthCredentials) -> bool {
        matches!(credentials, AuthCredentials::BearerToken { .. })
    }
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
    if parts.len() == 2 && parts[0].eq_ignore_ascii_case("Bearer") {
        Some(parts[1])
    } else {
        None
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> TokenConfig {
        let mut config = TokenConfig::default();
        config.secret_key = "test-secret-key-that-is-at-least-32-bytes-long".to_string();
        config.access_token_lifetime_secs = 3600;
        config.refresh_token_lifetime_secs = 86400;
        config
    }

    fn create_test_identity() -> AuthIdentity {
        let mut roles = HashSet::new();
        roles.insert("user".to_string());
        roles.insert("admin".to_string());

        AuthIdentity {
            user_id: UserId::new(),
            username: "testuser".to_string(),
            display_name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            roles,
            permissions: HashSet::new(),
            auth_method: AuthMethod::Password,
            authenticated_at: Utc::now(),
            session_id: None,
            claims: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_token_manager_creation() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        assert_eq!(manager.algorithm, Algorithm::HS256);
    }

    #[tokio::test]
    async fn test_create_access_token() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        let identity = create_test_identity();

        let token = manager.create_access_token(&identity).unwrap();
        assert!(!token.is_empty());

        // Token should have 3 parts (header.payload.signature)
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[tokio::test]
    async fn test_validate_access_token() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        let identity = create_test_identity();

        let token = manager.create_access_token(&identity).unwrap();
        let validated = manager.validate_access_token(&token).await.unwrap();

        assert_eq!(validated.user_id, identity.user_id);
        assert_eq!(validated.username, identity.username);
        assert!(validated.roles.contains("user"));
        assert!(validated.roles.contains("admin"));
    }

    #[tokio::test]
    async fn test_create_token_pair() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        let identity = create_test_identity();

        let pair = manager.create_token_pair(&identity).unwrap();

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 3600);
    }

    #[tokio::test]
    async fn test_validate_refresh_token() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        let user_id = UserId::new();

        let token = manager.create_refresh_token(user_id, None).unwrap();
        let claims = manager.validate_refresh_token(&token).await.unwrap();

        assert_eq!(claims.token_type, TokenType::Refresh);
        assert_eq!(claims.user_id().unwrap(), user_id);
    }

    #[tokio::test]
    async fn test_token_blacklist() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();
        let identity = create_test_identity();

        let token = manager.create_access_token(&identity).unwrap();

        // Should validate before blacklisting
        assert!(manager.validate_access_token(&token).await.is_ok());

        // Revoke the token
        manager.revoke_token(&token).await.unwrap();

        // Should fail after blacklisting
        let result = manager.validate_access_token(&token).await;
        assert!(matches!(result, Err(AuthError::TokenInvalid(_))));
    }

    #[tokio::test]
    async fn test_expired_token() {
        let mut config = create_test_config();
        config.access_token_lifetime_secs = 0; // Immediate expiration
        let manager = TokenManager::new_hmac(config).unwrap();
        let identity = create_test_identity();

        let token = manager.create_access_token(&identity).unwrap();

        // Wait a moment for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let result = manager.validate_access_token(&token).await;
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let config = create_test_config();
        let manager = TokenManager::new_hmac(config).unwrap();

        let result = manager.validate_access_token("invalid.token.here").await;
        assert!(matches!(result, Err(AuthError::TokenInvalid(_))));
    }

    #[tokio::test]
    async fn test_wrong_secret() {
        let config1 = create_test_config();
        let manager1 = TokenManager::new_hmac(config1).unwrap();
        let identity = create_test_identity();

        let token = manager1.create_access_token(&identity).unwrap();

        let mut config2 = create_test_config();
        config2.secret_key = "different-secret-key-also-32-bytes-long".to_string();
        let manager2 = TokenManager::new_hmac(config2).unwrap();

        let result = manager2.validate_access_token(&token).await;
        assert!(matches!(result, Err(AuthError::TokenInvalid(_))));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_bearer_token("bearer xyz789"),
            Some("xyz789")
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
        assert_eq!(extract_bearer_token("abc123"), None);
    }

    #[test]
    fn test_token_claims_to_identity() {
        let user_id = UserId::new();
        let claims = TokenClaims {
            sub: user_id.to_string(),
            iss: "test".to_string(),
            aud: vec!["test".to_string()],
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            jti: "test-jti".to_string(),
            token_type: TokenType::Access,
            roles: vec!["admin".to_string()],
            permissions: vec!["read:all".to_string()],
            session_id: None,
            username: Some("testuser".to_string()),
            email: Some("test@example.com".to_string()),
            custom: serde_json::Map::new(),
        };

        let identity = claims.to_identity().unwrap();
        assert_eq!(identity.user_id, user_id);
        assert_eq!(identity.username, "testuser");
        assert!(identity.roles.contains("admin"));
        assert!(identity.permissions.contains("read:all"));
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthIdentity and UserId
- **Spec 367**: Auth Configuration - Uses TokenConfig
- **Spec 371**: Token Refresh - Uses TokenManager for refresh
- **Spec 372**: Auth Middleware - Validates tokens in requests
- **Spec 376**: API Keys - Alternative to JWT for API auth
