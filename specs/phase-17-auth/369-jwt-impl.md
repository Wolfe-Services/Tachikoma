# Spec 369: JWT Implementation

## Overview
Implement JSON Web Token (JWT) generation and validation for stateless authentication.

## Rust Implementation

### Dependencies
```toml
[dependencies]
jsonwebtoken = "9.2"
```

### JWT Implementation
```rust
// src/auth/jwt.rs

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header,
    TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::auth::types::{UserRole, AuthMethod};

#[derive(Debug, Error)]
pub enum JwtTokenError {
    #[error("Token expired")]
    Expired,

    #[error("Token not yet valid")]
    NotYetValid,

    #[error("Invalid token: {0}")]
    Invalid(String),

    #[error("Missing claim: {0}")]
    MissingClaim(String),

    #[error("Invalid issuer")]
    InvalidIssuer,

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    // Registered claims
    pub sub: String,              // Subject (user ID)
    pub iss: String,              // Issuer
    pub aud: Vec<String>,         // Audience
    pub exp: i64,                 // Expiration time (Unix timestamp)
    pub nbf: i64,                 // Not before (Unix timestamp)
    pub iat: i64,                 // Issued at (Unix timestamp)
    pub jti: String,              // JWT ID (unique identifier)

    // Custom claims
    pub email: Option<String>,
    pub role: String,
    pub permissions: Vec<String>,
    pub session_id: Option<String>,
    pub auth_method: String,
    pub tenant_id: Option<String>,
}

impl Claims {
    pub fn new(
        user_id: &str,
        email: Option<String>,
        role: UserRole,
        permissions: Vec<String>,
        issuer: &str,
        audience: Vec<String>,
        lifetime: Duration,
    ) -> Self {
        let now = Utc::now();

        Self {
            sub: user_id.to_string(),
            iss: issuer.to_string(),
            aud: audience,
            exp: (now + lifetime).timestamp(),
            nbf: now.timestamp(),
            iat: now.timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
            email,
            role: format!("{:?}", role).to_lowercase(),
            permissions,
            session_id: None,
            auth_method: "password".to_string(),
            tenant_id: None,
        }
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_auth_method(mut self, method: AuthMethod) -> Self {
        self.auth_method = format!("{:?}", method).to_lowercase();
        self
    }

    pub fn with_tenant(mut self, tenant_id: &str) -> Self {
        self.tenant_id = Some(tenant_id.to_string());
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(Utc::now)
    }

    pub fn issued_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.iat, 0).unwrap_or_else(Utc::now)
    }

    pub fn parse_role(&self) -> UserRole {
        match self.role.as_str() {
            "guest" => UserRole::Guest,
            "user" => UserRole::User,
            "moderator" => UserRole::Moderator,
            "admin" => UserRole::Admin,
            "superadmin" => UserRole::SuperAdmin,
            _ => UserRole::User,
        }
    }
}

/// JWT Configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Secret key for HS256 (or private key path for RS256)
    pub secret: String,
    /// Public key path for RS256 (None for HS256)
    pub public_key: Option<String>,
    /// Algorithm to use
    pub algorithm: Algorithm,
    /// Issuer
    pub issuer: String,
    /// Audience
    pub audience: Vec<String>,
    /// Token lifetime
    pub lifetime: Duration,
    /// Leeway for time-based validation (seconds)
    pub leeway: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "change-this-secret-in-production".to_string(),
            public_key: None,
            algorithm: Algorithm::HS256,
            issuer: "tachikoma".to_string(),
            audience: vec!["tachikoma".to_string()],
            lifetime: Duration::hours(1),
            leeway: 60,
        }
    }
}

/// JWT Handler
pub struct JwtHandler {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtHandler {
    pub fn new(config: JwtConfig) -> Result<Self, JwtTokenError> {
        let (encoding_key, decoding_key) = match config.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                let secret = config.secret.as_bytes();
                (
                    EncodingKey::from_secret(secret),
                    DecodingKey::from_secret(secret),
                )
            }
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => {
                let private_key = std::fs::read(&config.secret)
                    .map_err(|e| JwtTokenError::Invalid(format!("Cannot read private key: {}", e)))?;
                let public_key_path = config.public_key.as_ref()
                    .ok_or_else(|| JwtTokenError::Invalid("Public key path required for RS algorithms".to_string()))?;
                let public_key = std::fs::read(public_key_path)
                    .map_err(|e| JwtTokenError::Invalid(format!("Cannot read public key: {}", e)))?;

                (
                    EncodingKey::from_rsa_pem(&private_key)
                        .map_err(|e| JwtTokenError::Invalid(format!("Invalid private key: {}", e)))?,
                    DecodingKey::from_rsa_pem(&public_key)
                        .map_err(|e| JwtTokenError::Invalid(format!("Invalid public key: {}", e)))?,
                )
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                let private_key = std::fs::read(&config.secret)
                    .map_err(|e| JwtTokenError::Invalid(format!("Cannot read private key: {}", e)))?;
                let public_key_path = config.public_key.as_ref()
                    .ok_or_else(|| JwtTokenError::Invalid("Public key path required for ES algorithms".to_string()))?;
                let public_key = std::fs::read(public_key_path)
                    .map_err(|e| JwtTokenError::Invalid(format!("Cannot read public key: {}", e)))?;

                (
                    EncodingKey::from_ec_pem(&private_key)
                        .map_err(|e| JwtTokenError::Invalid(format!("Invalid private key: {}", e)))?,
                    DecodingKey::from_ec_pem(&public_key)
                        .map_err(|e| JwtTokenError::Invalid(format!("Invalid public key: {}", e)))?,
                )
            }
            _ => return Err(JwtTokenError::Invalid("Unsupported algorithm".to_string())),
        };

        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&config.audience);
        validation.leeway = config.leeway;
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Encode claims to JWT
    pub fn encode(&self, claims: &Claims) -> Result<String, JwtTokenError> {
        let header = Header::new(self.config.algorithm);
        let token = encode(&header, claims, &self.encoding_key)?;
        Ok(token)
    }

    /// Decode and validate JWT
    pub fn decode(&self, token: &str) -> Result<TokenData<Claims>, JwtTokenError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data)
    }

    /// Validate JWT and return claims
    pub fn validate(&self, token: &str) -> Result<Claims, JwtTokenError> {
        let token_data = self.decode(token)?;
        Ok(token_data.claims)
    }

    /// Create a new access token for a user
    pub fn create_access_token(
        &self,
        user_id: &str,
        email: Option<String>,
        role: UserRole,
        permissions: Vec<String>,
    ) -> Result<String, JwtTokenError> {
        let claims = Claims::new(
            user_id,
            email,
            role,
            permissions,
            &self.config.issuer,
            self.config.audience.clone(),
            self.config.lifetime,
        );

        self.encode(&claims)
    }

    /// Get token expiration time
    pub fn get_expiration(&self) -> DateTime<Utc> {
        Utc::now() + self.config.lifetime
    }

    /// Get configuration
    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

/// Extract JWT from Authorization header
pub fn extract_bearer_token(header: &str) -> Option<&str> {
    if header.starts_with("Bearer ") || header.starts_with("bearer ") {
        Some(&header[7..])
    } else {
        None
    }
}

/// JWT Key pair generator (for RS256/ES256)
pub struct KeyPairGenerator;

impl KeyPairGenerator {
    /// Generate RSA key pair
    pub fn generate_rsa(bits: usize) -> Result<(Vec<u8>, Vec<u8>), JwtTokenError> {
        use rsa::{RsaPrivateKey, pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding}};

        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| JwtTokenError::Invalid(format!("RSA generation failed: {}", e)))?;

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| JwtTokenError::Invalid(format!("Private key encoding failed: {}", e)))?;

        let public_key = private_key.to_public_key();
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| JwtTokenError::Invalid(format!("Public key encoding failed: {}", e)))?;

        Ok((private_pem.as_bytes().to_vec(), public_pem.as_bytes().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-at-least-32-chars".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_jwt_encode_decode() {
        let handler = JwtHandler::new(test_config()).unwrap();

        let claims = Claims::new(
            "user-123",
            Some("test@example.com".to_string()),
            UserRole::User,
            vec!["read".to_string()],
            "tachikoma",
            vec!["tachikoma".to_string()],
            Duration::hours(1),
        );

        let token = handler.encode(&claims).unwrap();
        let decoded = handler.validate(&token).unwrap();

        assert_eq!(decoded.sub, "user-123");
        assert_eq!(decoded.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_claims_expiration() {
        let claims = Claims::new(
            "user",
            None,
            UserRole::User,
            vec![],
            "iss",
            vec!["aud".to_string()],
            Duration::hours(1),
        );

        assert!(!claims.is_expired());

        let expired_claims = Claims {
            exp: (Utc::now() - Duration::hours(1)).timestamp(),
            ..claims
        };

        assert!(expired_claims.is_expired());
    }
}
```

## Files to Create
- `src/auth/jwt.rs` - JWT implementation
