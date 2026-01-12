# Spec 366: Authentication Core Types

## Overview
Define the core types and structures for the authentication system including users, credentials, and authentication context.

## Rust Implementation

### Core Authentication Types
```rust
// src/auth/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Authentication error types
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("User not found")]
    UserNotFound,

    #[error("User disabled")]
    UserDisabled,

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("MFA required")]
    MfaRequired,

    #[error("MFA invalid")]
    MfaInvalid,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// User identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
    pub status: UserStatus,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

impl User {
    pub fn new(email: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            username: None,
            display_name: None,
            avatar_url: None,
            email_verified: false,
            mfa_enabled: false,
            status: UserStatus::Active,
            role: UserRole::User,
            created_at: now,
            updated_at: now,
            last_login_at: None,
            metadata: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn can_login(&self) -> Result<(), AuthError> {
        match self.status {
            UserStatus::Active => Ok(()),
            UserStatus::Disabled => Err(AuthError::UserDisabled),
            UserStatus::Pending => Err(AuthError::EmailNotVerified),
            UserStatus::Locked => Err(AuthError::UserDisabled),
        }
    }
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Pending,
    Active,
    Disabled,
    Locked,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Guest,
    User,
    Moderator,
    Admin,
    SuperAdmin,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl UserRole {
    pub fn has_permission(&self, required: UserRole) -> bool {
        *self >= required
    }
}

/// Authentication method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Password,
    OAuth,
    MagicLink,
    DeviceCode,
    ApiKey,
    Mfa,
}

/// Credential types
#[derive(Debug, Clone)]
pub enum Credential {
    Password {
        email: String,
        password: String,
    },
    OAuth {
        provider: OAuthProvider,
        code: String,
        state: Option<String>,
    },
    MagicLink {
        token: String,
    },
    DeviceCode {
        device_code: String,
    },
    RefreshToken {
        token: String,
    },
    ApiKey {
        key: String,
    },
}

/// OAuth provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthProvider {
    GitHub,
    Google,
    Microsoft,
    Okta,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
            Self::Microsoft => "microsoft",
            Self::Okta => "okta",
        }
    }
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: User,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: Option<String>,
    pub mfa_required: bool,
    pub mfa_token: Option<String>,
}

impl AuthResult {
    pub fn new(user: User, access_token: String, expires_in: i64) -> Self {
        Self {
            user,
            access_token,
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in,
            scope: None,
            mfa_required: false,
            mfa_token: None,
        }
    }

    pub fn with_refresh_token(mut self, token: String) -> Self {
        self.refresh_token = Some(token);
        self
    }

    pub fn with_mfa_required(mut self, mfa_token: String) -> Self {
        self.mfa_required = true;
        self.mfa_token = Some(mfa_token);
        self
    }
}

/// Authentication context (attached to requests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: Option<String>,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub auth_method: AuthMethod,
    pub authenticated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tenant_id: Option<String>,
}

impl AuthContext {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn has_role(&self, required: UserRole) -> bool {
        self.role.has_permission(required)
    }
}

/// Password hash
#[derive(Debug, Clone)]
pub struct PasswordHash {
    pub hash: String,
    pub algorithm: HashAlgorithm,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Argon2id,
    Bcrypt,
    Pbkdf2,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Argon2id
    }
}

/// MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub method: MfaMethod,
    pub secret: Option<String>,  // Encrypted TOTP secret
    pub recovery_codes: Vec<String>,  // Encrypted
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Totp,
    Sms,
    Email,
    WebAuthn,
}

/// Login attempt tracking
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub id: String,
    pub user_id: Option<String>,
    pub email: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

/// API key for service authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub key_prefix: String,  // First 8 chars for identification
    pub key_hash: String,    // Hashed key
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ApiKey {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Utc::now() > exp)
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string()) || self.scopes.contains(&"*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("test@example.com".to_string());
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.status, UserStatus::Active);
        assert!(!user.email_verified);
    }

    #[test]
    fn test_role_permissions() {
        assert!(UserRole::Admin.has_permission(UserRole::User));
        assert!(UserRole::Admin.has_permission(UserRole::Moderator));
        assert!(!UserRole::User.has_permission(UserRole::Admin));
    }

    #[test]
    fn test_auth_context_expiry() {
        let ctx = AuthContext {
            user_id: "user-1".to_string(),
            session_id: None,
            role: UserRole::User,
            permissions: vec![],
            auth_method: AuthMethod::Password,
            authenticated_at: Utc::now(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            tenant_id: None,
        };

        assert!(ctx.is_expired());
    }
}
```

### Permission System
```rust
// src/auth/permissions.rs

use std::collections::HashSet;

/// Standard permissions
pub mod permissions {
    pub const READ_MISSIONS: &str = "missions:read";
    pub const WRITE_MISSIONS: &str = "missions:write";
    pub const DELETE_MISSIONS: &str = "missions:delete";

    pub const READ_SPECS: &str = "specs:read";
    pub const WRITE_SPECS: &str = "specs:write";
    pub const APPROVE_SPECS: &str = "specs:approve";

    pub const READ_FORGE: &str = "forge:read";
    pub const WRITE_FORGE: &str = "forge:write";
    pub const REVIEW_FORGE: &str = "forge:review";

    pub const MANAGE_USERS: &str = "users:manage";
    pub const MANAGE_ROLES: &str = "roles:manage";
    pub const MANAGE_CONFIG: &str = "config:manage";

    pub const VIEW_ANALYTICS: &str = "analytics:view";
    pub const VIEW_AUDIT: &str = "audit:view";

    pub const ADMIN_ALL: &str = "*";
}

/// Role-based permission sets
pub fn permissions_for_role(role: super::UserRole) -> HashSet<String> {
    use super::UserRole;
    use permissions::*;

    let mut perms = HashSet::new();

    match role {
        UserRole::Guest => {
            perms.insert(READ_MISSIONS.to_string());
            perms.insert(READ_SPECS.to_string());
        }
        UserRole::User => {
            perms.insert(READ_MISSIONS.to_string());
            perms.insert(WRITE_MISSIONS.to_string());
            perms.insert(READ_SPECS.to_string());
            perms.insert(WRITE_SPECS.to_string());
            perms.insert(READ_FORGE.to_string());
            perms.insert(WRITE_FORGE.to_string());
        }
        UserRole::Moderator => {
            perms.extend(permissions_for_role(UserRole::User));
            perms.insert(APPROVE_SPECS.to_string());
            perms.insert(REVIEW_FORGE.to_string());
            perms.insert(VIEW_ANALYTICS.to_string());
        }
        UserRole::Admin => {
            perms.extend(permissions_for_role(UserRole::Moderator));
            perms.insert(DELETE_MISSIONS.to_string());
            perms.insert(MANAGE_USERS.to_string());
            perms.insert(MANAGE_CONFIG.to_string());
            perms.insert(VIEW_AUDIT.to_string());
        }
        UserRole::SuperAdmin => {
            perms.insert(ADMIN_ALL.to_string());
        }
    }

    perms
}
```

## Files to Create
- `src/auth/types.rs` - Core authentication types
- `src/auth/permissions.rs` - Permission definitions
- `src/auth/mod.rs` - Module exports
