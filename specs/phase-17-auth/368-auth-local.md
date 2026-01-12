# Spec 368: Local User Authentication

## Phase
17 - Authentication/Authorization

## Spec ID
368

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration
- Spec 379: Password Hashing

## Estimated Context
~12%

---

## Objective

Implement local user authentication using username/password credentials. This includes user registration, login, password validation, and integration with the password hashing system. The implementation should support configurable password policies and integrate with the lockout and audit systems.

---

## Acceptance Criteria

- [ ] Implement `LocalAuthProvider` implementing `AuthProvider` trait
- [ ] Support user registration with validation
- [ ] Implement login with password verification
- [ ] Enforce password policies from configuration
- [ ] Track failed login attempts
- [ ] Support password change functionality
- [ ] Integrate with session creation on successful login
- [ ] Emit appropriate authentication events
- [ ] Handle account lockout checking

---

## Implementation Details

### Local Authentication Provider

```rust
// src/auth/local.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::{AuthConfig, PasswordConfig},
    events::{AuthEvent, AuthEventEmitter},
    lockout::LockoutManager,
    password::{PasswordHasher, PasswordValidator},
    provider::{AuthProvider, User, UserRepository},
    types::*,
};

/// Local authentication provider using username/password
pub struct LocalAuthProvider {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<PasswordHasher>,
    password_validator: Arc<PasswordValidator>,
    lockout_manager: Arc<LockoutManager>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: PasswordConfig,
}

impl LocalAuthProvider {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<PasswordHasher>,
        lockout_manager: Arc<LockoutManager>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: PasswordConfig,
    ) -> Self {
        let password_validator = Arc::new(PasswordValidator::new(config.clone()));

        Self {
            user_repository,
            password_hasher,
            password_validator,
            lockout_manager,
            event_emitter,
            config,
        }
    }

    /// Register a new user
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn register(
        &self,
        username: &str,
        email: Option<&str>,
        password: &str,
    ) -> AuthResult<User> {
        // Validate username
        self.validate_username(username)?;

        // Validate password
        self.password_validator.validate(password)?;

        // Check if username already exists
        if let Some(_) = self.user_repository.find_by_username(username).await? {
            return Err(AuthError::InvalidCredentials); // Don't reveal user exists
        }

        // Check if email already exists
        if let Some(email) = email {
            if let Some(_) = self.user_repository.find_by_email(email).await? {
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Hash the password
        let password_hash = self.password_hasher.hash(password).await?;

        // Create user
        let mut user = User::new(username);
        user.email = email.map(String::from);
        user.password_hash = Some(password_hash);
        user.password_changed_at = Some(Utc::now());
        user.roles.insert("user".to_string()); // Default role

        self.user_repository.create(&user).await?;

        info!(user_id = %user.id, "User registered successfully");

        // Emit registration event
        self.event_emitter
            .emit(AuthEvent::UserRegistered {
                user_id: user.id,
                username: username.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(user)
    }

    /// Validate username format
    fn validate_username(&self, username: &str) -> AuthResult<()> {
        if username.len() < 3 {
            return Err(AuthError::InvalidCredentials);
        }
        if username.len() > 64 {
            return Err(AuthError::InvalidCredentials);
        }
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(())
    }

    /// Login with username and password
    #[instrument(skip(self, password), fields(username = %username))]
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity> {
        // Find user
        let user = self
            .user_repository
            .find_by_username(username)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is locked
        if let Err(e) = self.lockout_manager.check_locked(&user).await {
            self.emit_login_failure(&user, metadata, "Account locked").await;
            return Err(e);
        }

        // Check if account is enabled
        if !user.enabled {
            self.emit_login_failure(&user, metadata, "Account disabled").await;
            return Err(AuthError::AccountDisabled);
        }

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AuthError::InvalidCredentials)?;

        match self.password_hasher.verify(password, password_hash).await {
            Ok(true) => {
                // Check password expiration
                if self.is_password_expired(&user) {
                    self.emit_login_failure(&user, metadata, "Password expired").await;
                    return Err(AuthError::PasswordExpired);
                }

                // Reset failed attempts on successful login
                self.lockout_manager.reset_failed_attempts(&user).await?;

                // Update last login
                let mut updated_user = user.clone();
                updated_user.last_login_at = Some(Utc::now());
                self.user_repository.update(&updated_user).await?;

                let identity = user.to_identity(AuthMethod::Password, None);

                // Emit success event
                self.event_emitter
                    .emit(AuthEvent::LoginSuccess {
                        user_id: user.id,
                        auth_method: AuthMethod::Password,
                        ip_address: metadata.ip_address.clone(),
                        user_agent: metadata.user_agent.clone(),
                        timestamp: Utc::now(),
                    })
                    .await;

                info!(user_id = %user.id, "Login successful");
                Ok(identity)
            }
            Ok(false) | Err(_) => {
                // Record failed attempt
                self.lockout_manager.record_failed_attempt(&user).await?;

                self.emit_login_failure(&user, metadata, "Invalid password").await;

                warn!(user_id = %user.id, "Login failed - invalid password");
                Err(AuthError::InvalidCredentials)
            }
        }
    }

    /// Check if user's password has expired
    fn is_password_expired(&self, user: &User) -> bool {
        if self.config.expiration_days == 0 {
            return false;
        }

        if let Some(changed_at) = user.password_changed_at {
            let expiration = changed_at
                + chrono::Duration::days(self.config.expiration_days as i64);
            Utc::now() > expiration
        } else {
            true // No password change date means expired
        }
    }

    /// Change user password
    #[instrument(skip(self, current_password, new_password), fields(user_id = %user_id))]
    pub async fn change_password(
        &self,
        user_id: UserId,
        current_password: &str,
        new_password: &str,
    ) -> AuthResult<()> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Verify current password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or(AuthError::InvalidCredentials)?;

        if !self.password_hasher.verify(current_password, password_hash).await? {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate new password
        self.password_validator.validate(new_password)?;

        // Check password history
        self.check_password_history(user_id, new_password).await?;

        // Hash new password
        let new_hash = self.password_hasher.hash(new_password).await?;

        // Update password
        self.user_repository.update_password(user_id, &new_hash).await?;

        // Emit event
        self.event_emitter
            .emit(AuthEvent::PasswordChanged {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        info!("Password changed successfully");
        Ok(())
    }

    /// Check if new password was used recently
    async fn check_password_history(
        &self,
        _user_id: UserId,
        _new_password: &str,
    ) -> AuthResult<()> {
        // TODO: Implement password history check
        // This would require a password_history table
        Ok(())
    }

    async fn emit_login_failure(&self, user: &User, metadata: &AuthMetadata, reason: &str) {
        self.event_emitter
            .emit(AuthEvent::LoginFailure {
                user_id: Some(user.id),
                username: user.username.clone(),
                reason: reason.to_string(),
                ip_address: metadata.ip_address.clone(),
                user_agent: metadata.user_agent.clone(),
                timestamp: Utc::now(),
            })
            .await;
    }
}

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    fn name(&self) -> &str {
        "local"
    }

    async fn authenticate(
        &self,
        credentials: &AuthCredentials,
        metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity> {
        match credentials {
            AuthCredentials::Password { username, password } => {
                self.login(username, password.expose(), metadata).await
            }
            _ => Err(AuthError::InvalidCredentials),
        }
    }

    async fn validate(&self, identity: &AuthIdentity) -> AuthResult<bool> {
        // Check if user still exists and is enabled
        let user = self
            .user_repository
            .find_by_id(identity.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(user.enabled && !user.locked)
    }

    async fn revoke(&self, _identity: &AuthIdentity) -> AuthResult<()> {
        // Local auth doesn't have tokens to revoke
        // Session revocation is handled by session manager
        Ok(())
    }

    fn supports(&self, credentials: &AuthCredentials) -> bool {
        matches!(credentials, AuthCredentials::Password { .. })
    }
}

/// Service for user management operations
pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<PasswordHasher>,
    password_validator: Arc<PasswordValidator>,
    event_emitter: Arc<dyn AuthEventEmitter>,
}

impl UserService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<PasswordHasher>,
        config: PasswordConfig,
        event_emitter: Arc<dyn AuthEventEmitter>,
    ) -> Self {
        Self {
            user_repository,
            password_hasher,
            password_validator: Arc::new(PasswordValidator::new(config)),
            event_emitter,
        }
    }

    /// Get user by ID
    pub async fn get_user(&self, id: UserId) -> AuthResult<Option<User>> {
        self.user_repository.find_by_id(id).await
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> AuthResult<Option<User>> {
        self.user_repository.find_by_username(username).await
    }

    /// Update user profile
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn update_profile(
        &self,
        user_id: UserId,
        display_name: Option<String>,
        email: Option<String>,
    ) -> AuthResult<User> {
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        if let Some(name) = display_name {
            user.display_name = Some(name);
        }

        if let Some(email) = email {
            // Check if email is already in use
            if let Some(existing) = self.user_repository.find_by_email(&email).await? {
                if existing.id != user_id {
                    return Err(AuthError::InvalidCredentials);
                }
            }
            user.email = Some(email);
            user.email_verified = false; // Require re-verification
        }

        user.updated_at = Utc::now();
        self.user_repository.update(&user).await?;

        Ok(user)
    }

    /// Enable a user account
    pub async fn enable_user(&self, user_id: UserId) -> AuthResult<()> {
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        user.enabled = true;
        user.updated_at = Utc::now();
        self.user_repository.update(&user).await?;

        self.event_emitter
            .emit(AuthEvent::AccountEnabled {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Disable a user account
    pub async fn disable_user(&self, user_id: UserId) -> AuthResult<()> {
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        user.enabled = false;
        user.updated_at = Utc::now();
        self.user_repository.update(&user).await?;

        self.event_emitter
            .emit(AuthEvent::AccountDisabled {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Delete a user account
    pub async fn delete_user(&self, user_id: UserId) -> AuthResult<()> {
        self.user_repository.delete(user_id).await?;

        self.event_emitter
            .emit(AuthEvent::AccountDeleted {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Set admin password (for initial setup or reset)
    pub async fn set_password(&self, user_id: UserId, password: &str) -> AuthResult<()> {
        self.password_validator.validate(password)?;

        let hash = self.password_hasher.hash(password).await?;
        self.user_repository.update_password(user_id, &hash).await?;

        self.event_emitter
            .emit(AuthEvent::PasswordReset {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }
}

/// Registration request
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub display_name: Option<String>,
}

/// Login request
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    /// MFA code if MFA is enabled
    pub mfa_code: Option<String>,
}

/// Login response
#[derive(Debug, Clone, serde::Serialize)]
pub struct LoginResponse {
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub mfa_required: bool,
}

/// Public user information
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserInfo {
    pub id: UserId,
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            roles: user.roles.into_iter().collect(),
        }
    }
}
```

### In-Memory User Repository (for testing)

```rust
// src/auth/repository/memory.rs

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use crate::auth::provider::{User, UserRepository};
use crate::auth::types::*;

/// In-memory user repository for testing
pub struct InMemoryUserRepository {
    users: RwLock<HashMap<UserId, User>>,
    username_index: RwLock<HashMap<String, UserId>>,
    email_index: RwLock<HashMap<String, UserId>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            username_index: RwLock::new(HashMap::new()),
            email_index: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: UserId) -> AuthResult<Option<User>> {
        let users = self.users.read().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;
        Ok(users.get(&id).cloned())
    }

    async fn find_by_username(&self, username: &str) -> AuthResult<Option<User>> {
        let index = self.username_index.read().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if let Some(id) = index.get(username) {
            let users = self.users.read().map_err(|_| {
                AuthError::Internal("Lock poisoned".to_string())
            })?;
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> AuthResult<Option<User>> {
        let index = self.email_index.read().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if let Some(id) = index.get(email) {
            let users = self.users.read().map_err(|_| {
                AuthError::Internal("Lock poisoned".to_string())
            })?;
            Ok(users.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn create(&self, user: &User) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;
        let mut username_index = self.username_index.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;
        let mut email_index = self.email_index.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        users.insert(user.id, user.clone());
        username_index.insert(user.username.clone(), user.id);
        if let Some(ref email) = user.email {
            email_index.insert(email.clone(), user.id);
        }

        Ok(())
    }

    async fn update(&self, user: &User) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if users.contains_key(&user.id) {
            users.insert(user.id, user.clone());
            Ok(())
        } else {
            Err(AuthError::UserNotFound)
        }
    }

    async fn delete(&self, id: UserId) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;
        let mut username_index = self.username_index.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;
        let mut email_index = self.email_index.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if let Some(user) = users.remove(&id) {
            username_index.remove(&user.username);
            if let Some(email) = user.email {
                email_index.remove(&email);
            }
            Ok(())
        } else {
            Err(AuthError::UserNotFound)
        }
    }

    async fn update_password(&self, id: UserId, password_hash: &str) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if let Some(user) = users.get_mut(&id) {
            user.password_hash = Some(password_hash.to_string());
            user.password_changed_at = Some(Utc::now());
            user.updated_at = Utc::now();
            Ok(())
        } else {
            Err(AuthError::UserNotFound)
        }
    }

    async fn get_roles(&self, id: UserId) -> AuthResult<HashSet<String>> {
        let users = self.users.read().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        users
            .get(&id)
            .map(|u| u.roles.clone())
            .ok_or(AuthError::UserNotFound)
    }

    async fn set_roles(&self, id: UserId, roles: HashSet<String>) -> AuthResult<()> {
        let mut users = self.users.write().map_err(|_| {
            AuthError::Internal("Lock poisoned".to_string())
        })?;

        if let Some(user) = users.get_mut(&id) {
            user.roles = roles;
            user.updated_at = Utc::now();
            Ok(())
        } else {
            Err(AuthError::UserNotFound)
        }
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

    async fn setup_provider() -> LocalAuthProvider {
        let repo = Arc::new(InMemoryUserRepository::new());
        let hasher = Arc::new(PasswordHasher::new(PasswordConfig::default()));
        let lockout = Arc::new(LockoutManager::new(LockoutConfig::default()));
        let events = Arc::new(NoOpEventEmitter);

        LocalAuthProvider::new(repo, hasher, lockout, events, PasswordConfig::default())
    }

    #[tokio::test]
    async fn test_register_user() {
        let provider = setup_provider().await;

        let user = provider
            .register("testuser", Some("test@example.com"), "SecureP@ssw0rd!")
            .await
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.password_hash.is_some());
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let provider = setup_provider().await;

        provider
            .register("testuser", None, "SecureP@ssw0rd!")
            .await
            .unwrap();

        let result = provider
            .register("testuser", None, "AnotherP@ss123!")
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_success() {
        let provider = setup_provider().await;

        provider
            .register("testuser", None, "SecureP@ssw0rd!")
            .await
            .unwrap();

        let identity = provider
            .login("testuser", "SecureP@ssw0rd!", &AuthMetadata::default())
            .await
            .unwrap();

        assert_eq!(identity.username, "testuser");
        assert_eq!(identity.auth_method, AuthMethod::Password);
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        let provider = setup_provider().await;

        provider
            .register("testuser", None, "SecureP@ssw0rd!")
            .await
            .unwrap();

        let result = provider
            .login("testuser", "wrongpassword", &AuthMetadata::default())
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let provider = setup_provider().await;

        let result = provider
            .login("nonexistent", "password", &AuthMetadata::default())
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_change_password() {
        let provider = setup_provider().await;

        let user = provider
            .register("testuser", None, "SecureP@ssw0rd!")
            .await
            .unwrap();

        provider
            .change_password(user.id, "SecureP@ssw0rd!", "NewSecureP@ss123!")
            .await
            .unwrap();

        // Old password should fail
        let result = provider
            .login("testuser", "SecureP@ssw0rd!", &AuthMetadata::default())
            .await;
        assert!(result.is_err());

        // New password should work
        let identity = provider
            .login("testuser", "NewSecureP@ss123!", &AuthMetadata::default())
            .await
            .unwrap();
        assert_eq!(identity.username, "testuser");
    }

    struct NoOpEventEmitter;

    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _event: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses core types
- **Spec 367**: Auth Configuration - Uses PasswordConfig
- **Spec 369**: Session Management - Creates sessions on login
- **Spec 379**: Password Hashing - Uses for password operations
- **Spec 381**: Audit Logging - Logs auth events
- **Spec 383**: Account Lockout - Integrates with lockout
