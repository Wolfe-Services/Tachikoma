# Spec 380: Account Recovery

## Phase
17 - Authentication/Authorization

## Spec ID
380

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration
- Spec 379: Password Hashing

## Estimated Context
~9%

---

## Objective

Implement secure account recovery mechanisms for users who have lost access to their accounts. This includes password reset via email, security questions (optional), and account recovery through verified backup methods. The implementation should prevent enumeration attacks and ensure secure token handling.

---

## Acceptance Criteria

- [ ] Implement password reset request flow
- [ ] Generate secure, time-limited reset tokens
- [ ] Send password reset emails (integration point)
- [ ] Implement secure password reset completion
- [ ] Prevent user enumeration in reset flow
- [ ] Support account recovery via backup email
- [ ] Implement rate limiting for recovery requests
- [ ] Track recovery attempts for security

---

## Implementation Details

### Account Recovery System

```rust
// src/auth/recovery.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::AuthConfig,
    events::{AuthEvent, AuthEventEmitter},
    password::PasswordHasher,
    provider::UserRepository,
    types::*,
};

/// Password reset request
#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetRequest {
    /// Email address for reset
    pub email: String,
}

/// Password reset completion
#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetCompletion {
    /// Reset token
    pub token: String,
    /// New password
    pub new_password: String,
    /// Confirm password
    pub confirm_password: String,
}

impl PasswordResetCompletion {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.new_password != self.confirm_password {
            errors.push("Passwords do not match".to_string());
        }

        if self.token.is_empty() {
            errors.push("Reset token is required".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Password reset token data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetToken {
    /// Token ID (stored)
    pub id: String,

    /// User ID
    pub user_id: UserId,

    /// Token hash (for verification)
    pub token_hash: String,

    /// When created
    pub created_at: DateTime<Utc>,

    /// When expires
    pub expires_at: DateTime<Utc>,

    /// IP address of requester
    pub ip_address: Option<String>,

    /// User agent of requester
    pub user_agent: Option<String>,

    /// Whether token has been used
    pub used: bool,

    /// When token was used
    pub used_at: Option<DateTime<Utc>>,
}

impl ResetToken {
    pub fn is_valid(&self) -> bool {
        !self.used && Utc::now() < self.expires_at
    }
}

/// Recovery method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryMethod {
    Email,
    BackupEmail,
    Phone,
    SecurityQuestions,
}

/// Account recovery manager
pub struct RecoveryManager {
    token_storage: Arc<dyn ResetTokenStorage>,
    user_repository: Arc<dyn UserRepository>,
    password_hasher: Arc<PasswordHasher>,
    email_sender: Arc<dyn EmailSender>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: RecoveryConfig,
}

/// Recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Token validity in hours
    pub token_validity_hours: u32,
    /// Maximum reset requests per hour per IP
    pub max_requests_per_hour: u32,
    /// Minimum time between requests for same email
    pub cooldown_minutes: u32,
    /// Whether to use constant-time responses (prevent enumeration)
    pub constant_time_response: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            token_validity_hours: 1,
            max_requests_per_hour: 5,
            cooldown_minutes: 5,
            constant_time_response: true,
        }
    }
}

impl RecoveryManager {
    pub fn new(
        token_storage: Arc<dyn ResetTokenStorage>,
        user_repository: Arc<dyn UserRepository>,
        password_hasher: Arc<PasswordHasher>,
        email_sender: Arc<dyn EmailSender>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: RecoveryConfig,
    ) -> Self {
        Self {
            token_storage,
            user_repository,
            password_hasher,
            email_sender,
            event_emitter,
            config,
        }
    }

    /// Request a password reset
    #[instrument(skip(self), fields(email = %request.email))]
    pub async fn request_reset(
        &self,
        request: &PasswordResetRequest,
        metadata: &AuthMetadata,
    ) -> AuthResult<()> {
        // Always return success to prevent enumeration
        // Actual processing happens regardless of whether user exists

        // Find user by email
        let user = self.user_repository.find_by_email(&request.email).await?;

        if let Some(user) = user {
            // Check cooldown
            if let Some(last_token) = self.token_storage.find_latest_for_user(user.id).await? {
                let cooldown_end = last_token.created_at
                    + Duration::minutes(self.config.cooldown_minutes as i64);
                if Utc::now() < cooldown_end {
                    // Still in cooldown, but don't reveal this to user
                    warn!(user_id = %user.id, "Reset request during cooldown");
                    return Ok(());
                }
            }

            // Generate token
            let raw_token = self.generate_token();
            let token_hash = self.hash_token(&raw_token);

            let reset_token = ResetToken {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: user.id,
                token_hash,
                created_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(self.config.token_validity_hours as i64),
                ip_address: metadata.ip_address.clone(),
                user_agent: metadata.user_agent.clone(),
                used: false,
                used_at: None,
            };

            // Store token
            self.token_storage.create(&reset_token).await?;

            // Send email
            self.email_sender
                .send_password_reset(&user.email.unwrap_or_default(), &raw_token)
                .await?;

            // Emit event
            self.event_emitter
                .emit(AuthEvent::PasswordResetRequested {
                    user_id: user.id,
                    ip_address: metadata.ip_address.clone(),
                    timestamp: Utc::now(),
                })
                .await;

            info!("Password reset email sent");
        } else {
            // User not found, but don't reveal this
            // Simulate delay to prevent timing attacks
            if self.config.constant_time_response {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        Ok(())
    }

    /// Complete password reset
    #[instrument(skip(self, completion))]
    pub async fn complete_reset(
        &self,
        completion: &PasswordResetCompletion,
        metadata: &AuthMetadata,
    ) -> AuthResult<()> {
        // Validate request
        completion.validate().map_err(|_| AuthError::InvalidCredentials)?;

        // Hash the token to find it
        let token_hash = self.hash_token(&completion.token);

        // Find token
        let mut reset_token = self
            .token_storage
            .find_by_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid("Invalid reset token".to_string()))?;

        // Validate token
        if !reset_token.is_valid() {
            if reset_token.used {
                return Err(AuthError::TokenInvalid("Token already used".to_string()));
            }
            return Err(AuthError::TokenExpired);
        }

        // Get user
        let user = self
            .user_repository
            .find_by_id(reset_token.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Validate password policy
        // (This would integrate with PasswordValidator from spec 379)

        // Hash new password
        let password_hash = self.password_hasher.hash(&completion.new_password).await?;

        // Update password
        self.user_repository
            .update_password(user.id, &password_hash)
            .await?;

        // Mark token as used
        reset_token.used = true;
        reset_token.used_at = Some(Utc::now());
        self.token_storage.update(&reset_token).await?;

        // Invalidate all other reset tokens for this user
        self.token_storage.invalidate_user_tokens(user.id).await?;

        // Emit event
        self.event_emitter
            .emit(AuthEvent::PasswordReset {
                user_id: user.id,
                timestamp: Utc::now(),
            })
            .await;

        // Send confirmation email
        if let Some(email) = &user.email {
            self.email_sender.send_password_changed(email).await?;
        }

        info!(user_id = %user.id, "Password reset completed");
        Ok(())
    }

    /// Verify a reset token is valid (for UI purposes)
    pub async fn verify_token(&self, token: &str) -> AuthResult<bool> {
        let token_hash = self.hash_token(token);

        match self.token_storage.find_by_hash(&token_hash).await? {
            Some(reset_token) => Ok(reset_token.is_valid()),
            None => Ok(false),
        }
    }

    /// Generate a secure random token
    fn generate_token(&self) -> String {
        let mut rng = thread_rng();
        (0..64)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect()
    }

    /// Hash a token for storage
    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.token_storage.cleanup_expired().await
    }
}

/// Email sender trait
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Send password reset email
    async fn send_password_reset(&self, email: &str, token: &str) -> AuthResult<()>;

    /// Send password changed confirmation
    async fn send_password_changed(&self, email: &str) -> AuthResult<()>;

    /// Send account recovery email
    async fn send_account_recovery(&self, email: &str, token: &str) -> AuthResult<()>;
}

/// Reset token storage trait
#[async_trait]
pub trait ResetTokenStorage: Send + Sync {
    /// Create a new reset token
    async fn create(&self, token: &ResetToken) -> AuthResult<()>;

    /// Find token by hash
    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<ResetToken>>;

    /// Find latest token for user
    async fn find_latest_for_user(&self, user_id: UserId) -> AuthResult<Option<ResetToken>>;

    /// Update a token
    async fn update(&self, token: &ResetToken) -> AuthResult<()>;

    /// Invalidate all tokens for a user
    async fn invalidate_user_tokens(&self, user_id: UserId) -> AuthResult<()>;

    /// Clean up expired tokens
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory reset token storage
pub struct InMemoryResetTokenStorage {
    tokens: RwLock<HashMap<String, ResetToken>>,
    hash_index: RwLock<HashMap<String, String>>,
    user_index: RwLock<HashMap<UserId, Vec<String>>>,
}

impl InMemoryResetTokenStorage {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
            hash_index: RwLock::new(HashMap::new()),
            user_index: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryResetTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ResetTokenStorage for InMemoryResetTokenStorage {
    async fn create(&self, token: &ResetToken) -> AuthResult<()> {
        let mut tokens = self.tokens.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;

        tokens.insert(token.id.clone(), token.clone());
        hash_index.insert(token.token_hash.clone(), token.id.clone());
        user_index
            .entry(token.user_id)
            .or_insert_with(Vec::new)
            .push(token.id.clone());

        Ok(())
    }

    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<ResetToken>> {
        let hash_index = self.hash_index.read().await;
        let tokens = self.tokens.read().await;

        if let Some(id) = hash_index.get(hash) {
            Ok(tokens.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_latest_for_user(&self, user_id: UserId) -> AuthResult<Option<ResetToken>> {
        let user_index = self.user_index.read().await;
        let tokens = self.tokens.read().await;

        if let Some(ids) = user_index.get(&user_id) {
            let latest = ids
                .iter()
                .filter_map(|id| tokens.get(id))
                .max_by_key(|t| t.created_at);
            Ok(latest.cloned())
        } else {
            Ok(None)
        }
    }

    async fn update(&self, token: &ResetToken) -> AuthResult<()> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.id.clone(), token.clone());
        Ok(())
    }

    async fn invalidate_user_tokens(&self, user_id: UserId) -> AuthResult<()> {
        let user_index = self.user_index.read().await;
        let mut tokens = self.tokens.write().await;

        if let Some(ids) = user_index.get(&user_id) {
            for id in ids {
                if let Some(token) = tokens.get_mut(id) {
                    token.used = true;
                    token.used_at = Some(Utc::now());
                }
            }
        }

        Ok(())
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut tokens = self.tokens.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;

        let expired: Vec<_> = tokens
            .iter()
            .filter(|(_, t)| !t.is_valid())
            .map(|(id, t)| (id.clone(), t.clone()))
            .collect();

        let count = expired.len();

        for (id, token) in expired {
            tokens.remove(&id);
            hash_index.remove(&token.token_hash);
            if let Some(ids) = user_index.get_mut(&token.user_id) {
                ids.retain(|i| i != &id);
            }
        }

        Ok(count)
    }
}

/// Mock email sender for testing
pub struct MockEmailSender {
    sent_emails: RwLock<Vec<SentEmail>>,
}

#[derive(Debug, Clone)]
pub struct SentEmail {
    pub to: String,
    pub email_type: EmailType,
    pub token: Option<String>,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum EmailType {
    PasswordReset,
    PasswordChanged,
    AccountRecovery,
}

impl MockEmailSender {
    pub fn new() -> Self {
        Self {
            sent_emails: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_sent_emails(&self) -> Vec<SentEmail> {
        self.sent_emails.read().await.clone()
    }
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send_password_reset(&self, email: &str, token: &str) -> AuthResult<()> {
        let mut emails = self.sent_emails.write().await;
        emails.push(SentEmail {
            to: email.to_string(),
            email_type: EmailType::PasswordReset,
            token: Some(token.to_string()),
            sent_at: Utc::now(),
        });
        Ok(())
    }

    async fn send_password_changed(&self, email: &str) -> AuthResult<()> {
        let mut emails = self.sent_emails.write().await;
        emails.push(SentEmail {
            to: email.to_string(),
            email_type: EmailType::PasswordChanged,
            token: None,
            sent_at: Utc::now(),
        });
        Ok(())
    }

    async fn send_account_recovery(&self, email: &str, token: &str) -> AuthResult<()> {
        let mut emails = self.sent_emails.write().await;
        emails.push(SentEmail {
            to: email.to_string(),
            email_type: EmailType::AccountRecovery,
            token: Some(token.to_string()),
            sent_at: Utc::now(),
        });
        Ok(())
    }
}

/// Account unlock request (for locked accounts)
#[derive(Debug, Clone, Deserialize)]
pub struct AccountUnlockRequest {
    pub email: String,
    pub verification_code: Option<String>,
}

/// Account recovery response
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryResponse {
    pub success: bool,
    pub message: String,
    /// Methods available for recovery
    pub available_methods: Vec<RecoveryMethod>,
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_manager() -> (RecoveryManager, Arc<MockEmailSender>) {
        let token_storage = Arc::new(InMemoryResetTokenStorage::new());
        let user_repo = Arc::new(MockUserRepository::new());
        let password_hasher = Arc::new(PasswordHasher::new(PasswordConfig::default()));
        let email_sender = Arc::new(MockEmailSender::new());
        let events = Arc::new(NoOpEventEmitter);

        let manager = RecoveryManager::new(
            token_storage,
            user_repo,
            password_hasher,
            email_sender.clone(),
            events,
            RecoveryConfig::default(),
        );

        (manager, email_sender)
    }

    #[tokio::test]
    async fn test_request_reset_existing_user() {
        let (manager, email_sender) = setup_manager().await;

        let request = PasswordResetRequest {
            email: "test@example.com".to_string(),
        };

        manager.request_reset(&request, &AuthMetadata::default()).await.unwrap();

        let emails = email_sender.get_sent_emails().await;
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].to, "test@example.com");
        assert!(matches!(emails[0].email_type, EmailType::PasswordReset));
    }

    #[tokio::test]
    async fn test_request_reset_nonexistent_user() {
        let (manager, email_sender) = setup_manager().await;

        let request = PasswordResetRequest {
            email: "nonexistent@example.com".to_string(),
        };

        // Should not return error (prevent enumeration)
        let result = manager.request_reset(&request, &AuthMetadata::default()).await;
        assert!(result.is_ok());

        // No email should be sent
        let emails = email_sender.get_sent_emails().await;
        assert_eq!(emails.len(), 0);
    }

    #[tokio::test]
    async fn test_complete_reset() {
        let (manager, email_sender) = setup_manager().await;

        // Request reset
        let request = PasswordResetRequest {
            email: "test@example.com".to_string(),
        };
        manager.request_reset(&request, &AuthMetadata::default()).await.unwrap();

        // Get the token from the "email"
        let emails = email_sender.get_sent_emails().await;
        let token = emails[0].token.as_ref().unwrap();

        // Complete reset
        let completion = PasswordResetCompletion {
            token: token.clone(),
            new_password: "NewSecurePassword1!".to_string(),
            confirm_password: "NewSecurePassword1!".to_string(),
        };

        let result = manager.complete_reset(&completion, &AuthMetadata::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_complete_reset_invalid_token() {
        let (manager, _) = setup_manager().await;

        let completion = PasswordResetCompletion {
            token: "invalid_token".to_string(),
            new_password: "NewSecurePassword1!".to_string(),
            confirm_password: "NewSecurePassword1!".to_string(),
        };

        let result = manager.complete_reset(&completion, &AuthMetadata::default()).await;
        assert!(matches!(result, Err(AuthError::TokenInvalid(_))));
    }

    #[tokio::test]
    async fn test_token_reuse_prevention() {
        let (manager, email_sender) = setup_manager().await;

        // Request reset
        let request = PasswordResetRequest {
            email: "test@example.com".to_string(),
        };
        manager.request_reset(&request, &AuthMetadata::default()).await.unwrap();

        let emails = email_sender.get_sent_emails().await;
        let token = emails[0].token.as_ref().unwrap();

        // Complete reset first time
        let completion = PasswordResetCompletion {
            token: token.clone(),
            new_password: "NewSecurePassword1!".to_string(),
            confirm_password: "NewSecurePassword1!".to_string(),
        };
        manager.complete_reset(&completion, &AuthMetadata::default()).await.unwrap();

        // Try to use same token again
        let result = manager.complete_reset(&completion, &AuthMetadata::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_verify_token() {
        let (manager, email_sender) = setup_manager().await;

        // Request reset
        let request = PasswordResetRequest {
            email: "test@example.com".to_string(),
        };
        manager.request_reset(&request, &AuthMetadata::default()).await.unwrap();

        let emails = email_sender.get_sent_emails().await;
        let token = emails[0].token.as_ref().unwrap();

        // Valid token
        assert!(manager.verify_token(token).await.unwrap());

        // Invalid token
        assert!(!manager.verify_token("invalid").await.unwrap());
    }

    #[test]
    fn test_reset_completion_validation() {
        let valid = PasswordResetCompletion {
            token: "token".to_string(),
            new_password: "password".to_string(),
            confirm_password: "password".to_string(),
        };
        assert!(valid.validate().is_ok());

        let mismatched = PasswordResetCompletion {
            token: "token".to_string(),
            new_password: "password1".to_string(),
            confirm_password: "password2".to_string(),
        };
        assert!(mismatched.validate().is_err());
    }

    struct MockUserRepository {
        users: RwLock<HashMap<String, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            let mut users = HashMap::new();
            let mut user = User::new("testuser");
            user.email = Some("test@example.com".to_string());
            users.insert("test@example.com".to_string(), user);
            Self {
                users: RwLock::new(users),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: UserId) -> AuthResult<Option<User>> {
            let users = self.users.read().await;
            Ok(users.values().find(|u| u.id == id).cloned())
        }
        async fn find_by_username(&self, _: &str) -> AuthResult<Option<User>> { Ok(None) }
        async fn find_by_email(&self, email: &str) -> AuthResult<Option<User>> {
            let users = self.users.read().await;
            Ok(users.get(email).cloned())
        }
        async fn create(&self, _: &User) -> AuthResult<()> { Ok(()) }
        async fn update(&self, _: &User) -> AuthResult<()> { Ok(()) }
        async fn delete(&self, _: UserId) -> AuthResult<()> { Ok(()) }
        async fn update_password(&self, _: UserId, _: &str) -> AuthResult<()> { Ok(()) }
        async fn get_roles(&self, _: UserId) -> AuthResult<HashSet<String>> { Ok(HashSet::new()) }
        async fn set_roles(&self, _: UserId, _: HashSet<String>) -> AuthResult<()> { Ok(()) }
    }

    struct NoOpEventEmitter;
    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthError and UserId
- **Spec 367**: Auth Configuration - Recovery settings
- **Spec 379**: Password Hashing - Uses PasswordHasher
- **Spec 381**: Audit Logging - Logs recovery events
- **Spec 384**: Auth Events - Emits recovery events
