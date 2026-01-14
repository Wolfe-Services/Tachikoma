//! Authentication types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// User email.
    pub email: String,
    /// User roles.
    pub roles: Vec<String>,
    /// Token type (access/refresh).
    pub token_type: TokenType,
    /// Issued at timestamp.
    pub iat: i64,
    /// Expiration timestamp.
    pub exp: i64,
    /// JWT ID (for revocation).
    pub jti: String,
}

/// Token type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

impl Claims {
    /// Create new access token claims.
    pub fn new_access(user_id: Uuid, email: &str, roles: Vec<String>, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles,
            token_type: TokenType::Access,
            iat: now,
            exp: now + expires_in,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Create new refresh token claims.
    pub fn new_refresh(user_id: Uuid, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email: String::new(),
            roles: Vec::new(),
            token_type: TokenType::Refresh,
            iat: now,
            exp: now + expires_in,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Get user ID as UUID.
    pub fn user_id(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.sub).ok()
    }

    /// Check if token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// Authenticated user context.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub claims: Claims,
}

impl AuthUser {
    /// Create from claims.
    pub fn from_claims(claims: Claims) -> Option<Self> {
        let id = claims.user_id()?;
        Some(Self {
            id,
            email: claims.email.clone(),
            roles: claims.roles.clone(),
            claims,
        })
    }

    /// Check if user has admin role.
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&"admin".to_string())
    }
}

/// API key authentication info.
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub key_id: String,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}