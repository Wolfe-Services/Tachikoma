# Spec 366: Authentication Types and Traits

## Phase
17 - Authentication/Authorization

## Spec ID
366

## Status
Planned

## Dependencies
- Spec 001: Project Setup
- Spec 010: Error Handling

## Estimated Context
~10%

---

## Objective

Define the core authentication types, traits, and abstractions that form the foundation of the Tachikoma authentication system. This includes user identity types, authentication results, credential types, and the traits that different authentication providers must implement.

---

## Acceptance Criteria

- [ ] Define `UserId` type with strong typing
- [ ] Define `AuthCredentials` enum for various credential types
- [ ] Define `AuthIdentity` struct representing authenticated users
- [ ] Create `AuthProvider` trait for pluggable authentication
- [ ] Define `AuthResult` type alias with proper error handling
- [ ] Create `AuthContext` for carrying auth state through requests
- [ ] Implement serialization for all public types
- [ ] Document all types with proper rustdoc

---

## Implementation Details

### Core Types

```rust
// src/auth/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

/// Strongly-typed user identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(Uuid);

impl UserId {
    /// Create a new random user ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parse from string
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Strongly-typed session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of credentials that can be used for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthCredentials {
    /// Username and password authentication
    Password {
        username: String,
        password: SecretString,
    },
    /// JWT bearer token
    BearerToken {
        token: SecretString,
    },
    /// API key authentication
    ApiKey {
        key_id: String,
        key_secret: SecretString,
    },
    /// OAuth2 authorization code
    OAuth2Code {
        provider: String,
        code: String,
        redirect_uri: String,
    },
    /// OAuth2 access token
    OAuth2Token {
        provider: String,
        access_token: SecretString,
    },
    /// Refresh token for token renewal
    RefreshToken {
        token: SecretString,
    },
    /// Multi-factor authentication code
    MfaCode {
        user_id: UserId,
        code: String,
        mfa_type: MfaType,
    },
}

/// Secret string that doesn't expose its contents in debug output
#[derive(Clone, Serialize, Deserialize)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString([REDACTED])")
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Types of multi-factor authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaType {
    /// Time-based One-Time Password (TOTP)
    Totp,
    /// SMS verification code
    Sms,
    /// Email verification code
    Email,
    /// Hardware security key (WebAuthn)
    SecurityKey,
    /// Backup recovery code
    BackupCode,
}

/// Represents an authenticated user identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthIdentity {
    /// Unique user identifier
    pub user_id: UserId,
    /// Username or email
    pub username: String,
    /// Display name
    pub display_name: Option<String>,
    /// Email address (verified)
    pub email: Option<String>,
    /// Whether email is verified
    pub email_verified: bool,
    /// User roles
    pub roles: HashSet<String>,
    /// User permissions (computed from roles + direct grants)
    pub permissions: HashSet<String>,
    /// Authentication method used
    pub auth_method: AuthMethod,
    /// When authentication occurred
    pub authenticated_at: DateTime<Utc>,
    /// Session ID if applicable
    pub session_id: Option<SessionId>,
    /// Additional claims/metadata
    pub claims: serde_json::Value,
}

impl AuthIdentity {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.roles.contains(*r))
    }

    /// Check if user has all specified permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.permissions.contains(*p))
    }

    /// Check if user is an admin
    pub fn is_admin(&self) -> bool {
        self.has_role("admin") || self.has_role("super_admin")
    }
}

/// Method used for authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Password,
    ApiKey,
    OAuth2,
    Jwt,
    Session,
    Mfa,
}

/// Authentication context carried through requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The authenticated identity (if any)
    pub identity: Option<AuthIdentity>,
    /// The raw token/credentials used
    pub credentials: Option<AuthCredentials>,
    /// Request metadata
    pub metadata: AuthMetadata,
}

impl AuthContext {
    /// Create an anonymous (unauthenticated) context
    pub fn anonymous(metadata: AuthMetadata) -> Self {
        Self {
            identity: None,
            credentials: None,
            metadata,
        }
    }

    /// Create an authenticated context
    pub fn authenticated(identity: AuthIdentity, metadata: AuthMetadata) -> Self {
        Self {
            identity: Some(identity),
            credentials: None,
            metadata,
        }
    }

    /// Check if the context is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.identity.is_some()
    }

    /// Get the user ID if authenticated
    pub fn user_id(&self) -> Option<UserId> {
        self.identity.as_ref().map(|i| i.user_id)
    }

    /// Require authentication, returning error if not authenticated
    pub fn require_auth(&self) -> Result<&AuthIdentity, AuthError> {
        self.identity.as_ref().ok_or(AuthError::NotAuthenticated)
    }
}

/// Metadata about the authentication request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthMetadata {
    /// Client IP address
    pub ip_address: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Geographic location (if resolved)
    pub geo_location: Option<GeoLocation>,
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Authentication errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User account is locked")]
    AccountLocked,

    #[error("User account is disabled")]
    AccountDisabled,

    #[error("Password expired")]
    PasswordExpired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Role required: {0}")]
    RoleRequired(String),

    #[error("MFA required")]
    MfaRequired,

    #[error("MFA code invalid")]
    MfaInvalid,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session invalid")]
    SessionInvalid,

    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;
```

### Authentication Provider Trait

```rust
// src/auth/provider.rs

use async_trait::async_trait;
use crate::auth::types::*;

/// Trait for authentication providers
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Provider name for identification
    fn name(&self) -> &str;

    /// Authenticate with the given credentials
    async fn authenticate(
        &self,
        credentials: &AuthCredentials,
        metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity>;

    /// Validate an existing identity (e.g., check if still valid)
    async fn validate(&self, identity: &AuthIdentity) -> AuthResult<bool>;

    /// Revoke authentication (logout)
    async fn revoke(&self, identity: &AuthIdentity) -> AuthResult<()>;

    /// Check if this provider can handle the given credentials
    fn supports(&self, credentials: &AuthCredentials) -> bool;
}

/// Composite authentication provider that delegates to multiple providers
pub struct CompositeAuthProvider {
    providers: Vec<Box<dyn AuthProvider>>,
}

impl CompositeAuthProvider {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn add_provider(&mut self, provider: impl AuthProvider + 'static) {
        self.providers.push(Box::new(provider));
    }

    pub fn with_provider(mut self, provider: impl AuthProvider + 'static) -> Self {
        self.add_provider(provider);
        self
    }
}

impl Default for CompositeAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for CompositeAuthProvider {
    fn name(&self) -> &str {
        "composite"
    }

    async fn authenticate(
        &self,
        credentials: &AuthCredentials,
        metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity> {
        for provider in &self.providers {
            if provider.supports(credentials) {
                return provider.authenticate(credentials, metadata).await;
            }
        }
        Err(AuthError::InvalidCredentials)
    }

    async fn validate(&self, identity: &AuthIdentity) -> AuthResult<bool> {
        for provider in &self.providers {
            if let Ok(valid) = provider.validate(identity).await {
                return Ok(valid);
            }
        }
        Ok(false)
    }

    async fn revoke(&self, identity: &AuthIdentity) -> AuthResult<()> {
        for provider in &self.providers {
            let _ = provider.revoke(identity).await;
        }
        Ok(())
    }

    fn supports(&self, credentials: &AuthCredentials) -> bool {
        self.providers.iter().any(|p| p.supports(credentials))
    }
}

/// Trait for user storage/repository
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find user by ID
    async fn find_by_id(&self, id: UserId) -> AuthResult<Option<User>>;

    /// Find user by username
    async fn find_by_username(&self, username: &str) -> AuthResult<Option<User>>;

    /// Find user by email
    async fn find_by_email(&self, email: &str) -> AuthResult<Option<User>>;

    /// Create a new user
    async fn create(&self, user: &User) -> AuthResult<()>;

    /// Update an existing user
    async fn update(&self, user: &User) -> AuthResult<()>;

    /// Delete a user
    async fn delete(&self, id: UserId) -> AuthResult<()>;

    /// Update password hash
    async fn update_password(&self, id: UserId, password_hash: &str) -> AuthResult<()>;

    /// Get user roles
    async fn get_roles(&self, id: UserId) -> AuthResult<HashSet<String>>;

    /// Set user roles
    async fn set_roles(&self, id: UserId, roles: HashSet<String>) -> AuthResult<()>;
}

/// User entity for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub password_hash: Option<String>,
    pub roles: HashSet<String>,
    pub enabled: bool,
    pub locked: bool,
    pub locked_until: Option<DateTime<Utc>>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            username: username.into(),
            email: None,
            email_verified: false,
            display_name: None,
            password_hash: None,
            roles: HashSet::new(),
            enabled: true,
            locked: false,
            locked_until: None,
            password_changed_at: None,
            last_login_at: None,
            failed_login_attempts: 0,
            mfa_enabled: false,
            mfa_secret: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Convert to AuthIdentity
    pub fn to_identity(&self, auth_method: AuthMethod, session_id: Option<SessionId>) -> AuthIdentity {
        AuthIdentity {
            user_id: self.id,
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            email_verified: self.email_verified,
            roles: self.roles.clone(),
            permissions: HashSet::new(), // Computed separately
            auth_method,
            authenticated_at: Utc::now(),
            session_id,
            claims: serde_json::Value::Null,
        }
    }
}
```

### Module Organization

```rust
// src/auth/mod.rs

pub mod types;
pub mod provider;
pub mod config;
pub mod local;
pub mod session;
pub mod tokens;
pub mod middleware;
pub mod guards;
pub mod roles;
pub mod permissions;
pub mod api_keys;
pub mod oauth;
pub mod mfa;
pub mod password;
pub mod recovery;
pub mod audit;
pub mod rate_limit;
pub mod lockout;
pub mod events;

pub use types::*;
pub use provider::*;

/// Prelude for common imports
pub mod prelude {
    pub use super::types::{
        AuthContext, AuthCredentials, AuthError, AuthIdentity,
        AuthMetadata, AuthMethod, AuthResult, MfaType,
        SecretString, SessionId, UserId,
    };
    pub use super::provider::{AuthProvider, UserRepository};
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_user_id_parsing() {
        let id = UserId::new();
        let parsed = UserId::parse(&id.to_string()).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_secret_string_debug() {
        let secret = SecretString::new("password123");
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("password123"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_auth_identity_has_role() {
        let mut identity = create_test_identity();
        identity.roles.insert("admin".to_string());
        identity.roles.insert("user".to_string());

        assert!(identity.has_role("admin"));
        assert!(identity.has_role("user"));
        assert!(!identity.has_role("guest"));
    }

    #[test]
    fn test_auth_identity_has_permission() {
        let mut identity = create_test_identity();
        identity.permissions.insert("read:users".to_string());
        identity.permissions.insert("write:users".to_string());

        assert!(identity.has_permission("read:users"));
        assert!(identity.has_all_permissions(&["read:users", "write:users"]));
        assert!(!identity.has_all_permissions(&["read:users", "delete:users"]));
    }

    #[test]
    fn test_auth_context_anonymous() {
        let ctx = AuthContext::anonymous(AuthMetadata::default());
        assert!(!ctx.is_authenticated());
        assert!(ctx.user_id().is_none());
        assert!(ctx.require_auth().is_err());
    }

    #[test]
    fn test_auth_context_authenticated() {
        let identity = create_test_identity();
        let user_id = identity.user_id;
        let ctx = AuthContext::authenticated(identity, AuthMetadata::default());

        assert!(ctx.is_authenticated());
        assert_eq!(ctx.user_id(), Some(user_id));
        assert!(ctx.require_auth().is_ok());
    }

    fn create_test_identity() -> AuthIdentity {
        AuthIdentity {
            user_id: UserId::new(),
            username: "testuser".to_string(),
            display_name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            roles: HashSet::new(),
            permissions: HashSet::new(),
            auth_method: AuthMethod::Password,
            authenticated_at: Utc::now(),
            session_id: None,
            claims: serde_json::Value::Null,
        }
    }
}
```

---

## Related Specs

- **Spec 367**: Auth Configuration - Uses types defined here
- **Spec 368**: Local User Auth - Implements AuthProvider trait
- **Spec 369**: Session Management - Uses SessionId type
- **Spec 370**: JWT Tokens - Uses AuthIdentity for claims
- **Spec 374**: Role-Based Access - Extends AuthIdentity roles
- **Spec 375**: Permissions - Extends AuthIdentity permissions
