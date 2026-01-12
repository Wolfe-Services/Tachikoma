# Spec 371: Token Refresh

## Phase
17 - Authentication/Authorization

## Spec ID
371

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration
- Spec 370: JWT Tokens

## Estimated Context
~9%

---

## Objective

Implement secure token refresh functionality for renewing access tokens without re-authentication. This includes refresh token storage, validation, rotation, and revocation. The implementation should support refresh token families for detecting token theft and provide secure token rotation.

---

## Acceptance Criteria

- [ ] Implement `RefreshTokenManager` for refresh operations
- [ ] Store refresh tokens securely with metadata
- [ ] Implement token rotation (new refresh token on use)
- [ ] Support refresh token families for theft detection
- [ ] Implement refresh token revocation
- [ ] Limit active refresh tokens per user
- [ ] Handle refresh token reuse detection
- [ ] Clean up expired refresh tokens

---

## Implementation Details

### Refresh Token Manager

```rust
// src/auth/refresh.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::TokenConfig,
    events::{AuthEvent, AuthEventEmitter},
    provider::UserRepository,
    tokens::{TokenManager, TokenPair},
    types::*,
};

/// Stored refresh token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// Unique token identifier (JTI)
    pub id: String,

    /// User ID this token belongs to
    pub user_id: UserId,

    /// Token family ID for rotation tracking
    pub family_id: String,

    /// The actual token hash (never store raw tokens)
    pub token_hash: String,

    /// When the token was created
    pub created_at: DateTime<Utc>,

    /// When the token expires
    pub expires_at: DateTime<Utc>,

    /// IP address from token creation
    pub ip_address: Option<String>,

    /// User agent from token creation
    pub user_agent: Option<String>,

    /// Whether this token has been used (for rotation)
    pub used: bool,

    /// Whether this token has been revoked
    pub revoked: bool,

    /// Session ID if linked to a session
    pub session_id: Option<SessionId>,
}

impl RefreshToken {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }
}

/// Refresh token manager handles token lifecycle
pub struct RefreshTokenManager {
    storage: Arc<dyn RefreshTokenStorage>,
    token_manager: Arc<TokenManager>,
    user_repository: Arc<dyn UserRepository>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: TokenConfig,
}

impl RefreshTokenManager {
    pub fn new(
        storage: Arc<dyn RefreshTokenStorage>,
        token_manager: Arc<TokenManager>,
        user_repository: Arc<dyn UserRepository>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: TokenConfig,
    ) -> Self {
        Self {
            storage,
            token_manager,
            user_repository,
            event_emitter,
            config,
        }
    }

    /// Create a new refresh token for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn create_refresh_token(
        &self,
        user_id: UserId,
        session_id: Option<SessionId>,
        metadata: &AuthMetadata,
    ) -> AuthResult<(String, RefreshToken)> {
        // Check token limit
        self.enforce_token_limit(user_id).await?;

        // Generate new token
        let raw_token = self.token_manager.create_refresh_token(user_id, session_id)?;
        let token_hash = self.hash_token(&raw_token);
        let family_id = uuid::Uuid::new_v4().to_string();
        let jti = uuid::Uuid::new_v4().to_string();

        let refresh_token = RefreshToken {
            id: jti,
            user_id,
            family_id,
            token_hash,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.refresh_token_lifetime_secs as i64),
            ip_address: metadata.ip_address.clone(),
            user_agent: metadata.user_agent.clone(),
            used: false,
            revoked: false,
            session_id,
        };

        self.storage.store(&refresh_token).await?;

        info!(token_id = %refresh_token.id, "Refresh token created");

        Ok((raw_token, refresh_token))
    }

    /// Refresh tokens using a refresh token
    #[instrument(skip(self, raw_token))]
    pub async fn refresh(
        &self,
        raw_token: &str,
        metadata: &AuthMetadata,
    ) -> AuthResult<TokenPair> {
        // Validate the JWT refresh token first
        let claims = self.token_manager.validate_refresh_token(raw_token).await?;
        let user_id = claims.user_id()?;
        let token_hash = self.hash_token(raw_token);

        // Find the stored refresh token
        let stored_token = self
            .storage
            .find_by_hash(&token_hash)
            .await?
            .ok_or(AuthError::TokenInvalid("Refresh token not found".to_string()))?;

        // Check token validity
        if !stored_token.is_valid() {
            return Err(AuthError::TokenExpired);
        }

        // Check for token reuse (potential theft)
        if stored_token.used {
            // Token was already used - possible theft!
            // Revoke all tokens in this family
            warn!(
                family_id = %stored_token.family_id,
                user_id = %user_id,
                "Refresh token reuse detected - revoking family"
            );

            self.revoke_family(&stored_token.family_id).await?;

            self.event_emitter
                .emit(AuthEvent::RefreshTokenReuse {
                    user_id,
                    family_id: stored_token.family_id.clone(),
                    ip_address: metadata.ip_address.clone(),
                    timestamp: Utc::now(),
                })
                .await;

            return Err(AuthError::TokenInvalid(
                "Token reuse detected - all tokens revoked".to_string(),
            ));
        }

        // Mark current token as used
        self.storage.mark_used(&stored_token.id).await?;

        // Get user for new token
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Check if user is still valid
        if !user.enabled || user.locked {
            return Err(AuthError::AccountDisabled);
        }

        // Create new token pair with rotation
        let identity = user.to_identity(AuthMethod::Jwt, stored_token.session_id);
        let access_token = self.token_manager.create_access_token(&identity)?;

        // Create new refresh token in same family
        let new_raw_token = self.token_manager.create_refresh_token(user_id, stored_token.session_id)?;
        let new_token_hash = self.hash_token(&new_raw_token);
        let new_jti = uuid::Uuid::new_v4().to_string();

        let new_refresh_token = RefreshToken {
            id: new_jti,
            user_id,
            family_id: stored_token.family_id, // Same family
            token_hash: new_token_hash,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.config.refresh_token_lifetime_secs as i64),
            ip_address: metadata.ip_address.clone(),
            user_agent: metadata.user_agent.clone(),
            used: false,
            revoked: false,
            session_id: stored_token.session_id,
        };

        self.storage.store(&new_refresh_token).await?;

        self.event_emitter
            .emit(AuthEvent::TokenRefreshed {
                user_id,
                old_token_id: stored_token.id.clone(),
                new_token_id: new_refresh_token.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!(
            old_token_id = %stored_token.id,
            new_token_id = %new_refresh_token.id,
            "Token refreshed"
        );

        Ok(TokenPair {
            access_token,
            refresh_token: new_raw_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_lifetime_secs,
            refresh_expires_in: self.config.refresh_token_lifetime_secs,
        })
    }

    /// Revoke a specific refresh token
    #[instrument(skip(self, raw_token))]
    pub async fn revoke_token(&self, raw_token: &str) -> AuthResult<()> {
        let token_hash = self.hash_token(raw_token);

        if let Some(token) = self.storage.find_by_hash(&token_hash).await? {
            self.storage.revoke(&token.id).await?;

            self.event_emitter
                .emit(AuthEvent::RefreshTokenRevoked {
                    user_id: token.user_id,
                    token_id: token.id.clone(),
                    timestamp: Utc::now(),
                })
                .await;

            info!(token_id = %token.id, "Refresh token revoked");
        }

        Ok(())
    }

    /// Revoke all tokens in a family
    #[instrument(skip(self))]
    pub async fn revoke_family(&self, family_id: &str) -> AuthResult<()> {
        self.storage.revoke_family(family_id).await?;
        info!(family_id = %family_id, "Token family revoked");
        Ok(())
    }

    /// Revoke all refresh tokens for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn revoke_user_tokens(&self, user_id: UserId) -> AuthResult<()> {
        self.storage.revoke_user_tokens(user_id).await?;

        self.event_emitter
            .emit(AuthEvent::AllRefreshTokensRevoked {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        info!("All user refresh tokens revoked");
        Ok(())
    }

    /// Get all active refresh tokens for a user
    pub async fn get_user_tokens(&self, user_id: UserId) -> AuthResult<Vec<RefreshToken>> {
        let tokens = self.storage.get_user_tokens(user_id).await?;
        Ok(tokens.into_iter().filter(|t| t.is_valid()).collect())
    }

    /// Enforce token limit per user
    async fn enforce_token_limit(&self, user_id: UserId) -> AuthResult<()> {
        if self.config.max_refresh_tokens_per_user == 0 {
            return Ok(());
        }

        let tokens = self.get_user_tokens(user_id).await?;
        if tokens.len() >= self.config.max_refresh_tokens_per_user {
            // Revoke oldest token
            if let Some(oldest) = tokens.into_iter().min_by_key(|t| t.created_at) {
                self.storage.revoke(&oldest.id).await?;
            }
        }

        Ok(())
    }

    /// Hash a token for storage
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

/// Storage backend trait for refresh tokens
#[async_trait]
pub trait RefreshTokenStorage: Send + Sync {
    /// Store a refresh token
    async fn store(&self, token: &RefreshToken) -> AuthResult<()>;

    /// Find token by hash
    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<RefreshToken>>;

    /// Find token by ID
    async fn find_by_id(&self, id: &str) -> AuthResult<Option<RefreshToken>>;

    /// Mark a token as used
    async fn mark_used(&self, id: &str) -> AuthResult<()>;

    /// Revoke a token
    async fn revoke(&self, id: &str) -> AuthResult<()>;

    /// Revoke all tokens in a family
    async fn revoke_family(&self, family_id: &str) -> AuthResult<()>;

    /// Revoke all tokens for a user
    async fn revoke_user_tokens(&self, user_id: UserId) -> AuthResult<()>;

    /// Get all tokens for a user
    async fn get_user_tokens(&self, user_id: UserId) -> AuthResult<Vec<RefreshToken>>;

    /// Clean up expired tokens
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory refresh token storage
pub struct InMemoryRefreshTokenStorage {
    tokens: RwLock<HashMap<String, RefreshToken>>,
    hash_index: RwLock<HashMap<String, String>>,
    user_index: RwLock<HashMap<UserId, Vec<String>>>,
    family_index: RwLock<HashMap<String, Vec<String>>>,
}

impl InMemoryRefreshTokenStorage {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
            hash_index: RwLock::new(HashMap::new()),
            user_index: RwLock::new(HashMap::new()),
            family_index: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRefreshTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RefreshTokenStorage for InMemoryRefreshTokenStorage {
    async fn store(&self, token: &RefreshToken) -> AuthResult<()> {
        let mut tokens = self.tokens.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;
        let mut family_index = self.family_index.write().await;

        tokens.insert(token.id.clone(), token.clone());
        hash_index.insert(token.token_hash.clone(), token.id.clone());

        user_index
            .entry(token.user_id)
            .or_insert_with(Vec::new)
            .push(token.id.clone());

        family_index
            .entry(token.family_id.clone())
            .or_insert_with(Vec::new)
            .push(token.id.clone());

        Ok(())
    }

    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<RefreshToken>> {
        let hash_index = self.hash_index.read().await;
        let tokens = self.tokens.read().await;

        if let Some(id) = hash_index.get(hash) {
            Ok(tokens.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn find_by_id(&self, id: &str) -> AuthResult<Option<RefreshToken>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(id).cloned())
    }

    async fn mark_used(&self, id: &str) -> AuthResult<()> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(id) {
            token.used = true;
        }
        Ok(())
    }

    async fn revoke(&self, id: &str) -> AuthResult<()> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(id) {
            token.revoked = true;
        }
        Ok(())
    }

    async fn revoke_family(&self, family_id: &str) -> AuthResult<()> {
        let family_index = self.family_index.read().await;
        let mut tokens = self.tokens.write().await;

        if let Some(ids) = family_index.get(family_id) {
            for id in ids {
                if let Some(token) = tokens.get_mut(id) {
                    token.revoked = true;
                }
            }
        }

        Ok(())
    }

    async fn revoke_user_tokens(&self, user_id: UserId) -> AuthResult<()> {
        let user_index = self.user_index.read().await;
        let mut tokens = self.tokens.write().await;

        if let Some(ids) = user_index.get(&user_id) {
            for id in ids {
                if let Some(token) = tokens.get_mut(id) {
                    token.revoked = true;
                }
            }
        }

        Ok(())
    }

    async fn get_user_tokens(&self, user_id: UserId) -> AuthResult<Vec<RefreshToken>> {
        let user_index = self.user_index.read().await;
        let tokens = self.tokens.read().await;

        match user_index.get(&user_id) {
            Some(ids) => Ok(ids
                .iter()
                .filter_map(|id| tokens.get(id).cloned())
                .collect()),
            None => Ok(vec![]),
        }
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut tokens = self.tokens.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;
        let mut family_index = self.family_index.write().await;

        let expired_ids: Vec<_> = tokens
            .iter()
            .filter(|(_, t)| t.is_expired())
            .map(|(id, t)| (id.clone(), t.clone()))
            .collect();

        let count = expired_ids.len();

        for (id, token) in expired_ids {
            tokens.remove(&id);
            hash_index.remove(&token.token_hash);

            if let Some(ids) = user_index.get_mut(&token.user_id) {
                ids.retain(|i| i != &id);
            }

            if let Some(ids) = family_index.get_mut(&token.family_id) {
                ids.retain(|i| i != &id);
            }
        }

        Ok(count)
    }
}

/// Refresh request
#[derive(Debug, Clone, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Token info response
#[derive(Debug, Clone, Serialize)]
pub struct TokenInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub current: bool,
}

impl From<RefreshToken> for TokenInfo {
    fn from(token: RefreshToken) -> Self {
        Self {
            id: token.id,
            created_at: token.created_at,
            expires_at: token.expires_at,
            ip_address: token.ip_address,
            user_agent: token.user_agent,
            current: false,
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

    async fn setup_manager() -> RefreshTokenManager {
        let storage = Arc::new(InMemoryRefreshTokenStorage::new());
        let mut config = TokenConfig::default();
        config.secret_key = "test-secret-key-that-is-at-least-32-bytes".to_string();
        config.refresh_token_lifetime_secs = 86400;
        config.max_refresh_tokens_per_user = 5;

        let token_manager = Arc::new(TokenManager::new_hmac(config.clone()).unwrap());
        let user_repo = Arc::new(MockUserRepository::new());
        let events = Arc::new(NoOpEventEmitter);

        RefreshTokenManager::new(storage, token_manager, user_repo, events, config)
    }

    #[tokio::test]
    async fn test_create_refresh_token() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        let (raw_token, stored) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();

        assert!(!raw_token.is_empty());
        assert_eq!(stored.user_id, user_id);
        assert!(!stored.used);
        assert!(!stored.revoked);
    }

    #[tokio::test]
    async fn test_refresh_token_rotation() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        // Create initial token
        let (raw_token, _) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();

        // Refresh
        let new_pair = manager.refresh(&raw_token, &metadata).await.unwrap();

        // New tokens should be different
        assert_ne!(raw_token, new_pair.refresh_token);
        assert!(!new_pair.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_token_reuse_detection() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        let (raw_token, _) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();

        // First refresh should succeed
        let _ = manager.refresh(&raw_token, &metadata).await.unwrap();

        // Second use of same token should fail (reuse detection)
        let result = manager.refresh(&raw_token, &metadata).await;
        assert!(matches!(result, Err(AuthError::TokenInvalid(_))));
    }

    #[tokio::test]
    async fn test_revoke_token() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        let (raw_token, _) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();

        // Revoke
        manager.revoke_token(&raw_token).await.unwrap();

        // Should fail to use
        let result = manager.refresh(&raw_token, &metadata).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_user_tokens() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        // Create multiple tokens
        let (token1, _) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();
        let (token2, _) = manager
            .create_refresh_token(user_id, None, &metadata)
            .await
            .unwrap();

        // Revoke all
        manager.revoke_user_tokens(user_id).await.unwrap();

        // Both should fail
        assert!(manager.refresh(&token1, &metadata).await.is_err());
        assert!(manager.refresh(&token2, &metadata).await.is_err());
    }

    #[tokio::test]
    async fn test_token_limit_enforcement() {
        let manager = setup_manager().await;
        let user_id = UserId::new();
        let metadata = AuthMetadata::default();

        // Create tokens up to limit (5)
        for _ in 0..6 {
            manager
                .create_refresh_token(user_id, None, &metadata)
                .await
                .unwrap();
        }

        // Should only have 5 valid tokens
        let tokens = manager.get_user_tokens(user_id).await.unwrap();
        assert_eq!(tokens.len(), 5);
    }

    struct MockUserRepository;
    impl MockUserRepository {
        fn new() -> Self { Self }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: UserId) -> AuthResult<Option<User>> {
            let mut user = User::new("testuser");
            user.id = id;
            Ok(Some(user))
        }
        async fn find_by_username(&self, _: &str) -> AuthResult<Option<User>> { Ok(None) }
        async fn find_by_email(&self, _: &str) -> AuthResult<Option<User>> { Ok(None) }
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

- **Spec 366**: Auth Types - Uses core types
- **Spec 367**: Auth Configuration - Uses TokenConfig
- **Spec 370**: JWT Tokens - Uses TokenManager
- **Spec 369**: Session Management - Can be linked to sessions
- **Spec 384**: Auth Events - Emits refresh events
