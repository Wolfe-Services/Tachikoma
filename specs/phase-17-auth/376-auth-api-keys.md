# Spec 376: API Key Authentication

## Phase
17 - Authentication/Authorization

## Spec ID
376

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~10%

---

## Objective

Implement API key authentication for programmatic access to the system. API keys provide a simpler authentication method for scripts, CLI tools, and service-to-service communication. The implementation should support key generation, validation, scoping, and revocation.

---

## Acceptance Criteria

- [ ] Define `ApiKey` struct with metadata
- [ ] Implement secure key generation with prefix
- [ ] Create `ApiKeyManager` for key lifecycle
- [ ] Support key scoping with permissions
- [ ] Implement key validation and authentication
- [ ] Support key expiration
- [ ] Implement key revocation
- [ ] Track key usage statistics
- [ ] Limit keys per user

---

## Implementation Details

### API Key Types and Manager

```rust
// src/auth/api_keys.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::ApiKeyConfig,
    events::{AuthEvent, AuthEventEmitter},
    provider::UserRepository,
    types::*,
};

/// API key data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique key identifier
    pub id: String,

    /// Key name/description
    pub name: String,

    /// User who owns this key
    pub user_id: UserId,

    /// Hashed key value (never store raw key)
    pub key_hash: String,

    /// Key prefix for identification (e.g., "tk_live_")
    pub prefix: String,

    /// Last 4 characters of key (for display)
    pub suffix: String,

    /// Permissions/scopes granted to this key
    pub scopes: HashSet<String>,

    /// When the key was created
    pub created_at: DateTime<Utc>,

    /// When the key expires (None = never)
    pub expires_at: Option<DateTime<Utc>>,

    /// When the key was last used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Number of times the key has been used
    pub use_count: u64,

    /// IP address restrictions (empty = no restriction)
    pub allowed_ips: Vec<String>,

    /// Whether the key is active
    pub active: bool,

    /// Whether the key is revoked
    pub revoked: bool,

    /// When the key was revoked
    pub revoked_at: Option<DateTime<Utc>>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ApiKey {
    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }

    /// Check if key is valid
    pub fn is_valid(&self) -> bool {
        self.active && !self.revoked && !self.is_expired()
    }

    /// Check if IP is allowed
    pub fn is_ip_allowed(&self, ip: Option<&str>) -> bool {
        if self.allowed_ips.is_empty() {
            return true;
        }

        match ip {
            Some(ip) => self.allowed_ips.iter().any(|allowed| {
                allowed == ip || allowed == "*"
            }),
            None => false,
        }
    }

    /// Check if key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains("*") || self.scopes.contains(scope)
    }

    /// Get display representation of key
    pub fn display_key(&self) -> String {
        format!("{}...{}", self.prefix, self.suffix)
    }
}

/// Result of key creation including the raw key
#[derive(Debug, Clone)]
pub struct CreatedApiKey {
    /// The API key metadata
    pub key: ApiKey,

    /// The raw key value (only available at creation time)
    pub raw_key: String,
}

/// API key creation request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyRequest {
    /// Key name/description
    pub name: String,

    /// Requested scopes
    pub scopes: Option<Vec<String>>,

    /// Expiration in days (None = use default)
    pub expires_in_days: Option<u32>,

    /// IP restrictions
    pub allowed_ips: Option<Vec<String>>,

    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// API key manager
pub struct ApiKeyManager {
    storage: Arc<dyn ApiKeyStorage>,
    user_repository: Arc<dyn UserRepository>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: ApiKeyConfig,
}

impl ApiKeyManager {
    pub fn new(
        storage: Arc<dyn ApiKeyStorage>,
        user_repository: Arc<dyn UserRepository>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: ApiKeyConfig,
    ) -> Self {
        Self {
            storage,
            user_repository,
            event_emitter,
            config,
        }
    }

    /// Create a new API key
    #[instrument(skip(self, request), fields(user_id = %user_id))]
    pub async fn create_key(
        &self,
        user_id: UserId,
        request: CreateApiKeyRequest,
    ) -> AuthResult<CreatedApiKey> {
        // Check key limit
        let existing = self.storage.get_user_keys(user_id).await?;
        if existing.len() >= self.config.max_keys_per_user {
            return Err(AuthError::ConfigError(format!(
                "Maximum API keys per user ({}) exceeded",
                self.config.max_keys_per_user
            )));
        }

        // Generate raw key
        let raw_key = self.generate_key();
        let key_hash = self.hash_key(&raw_key);

        // Calculate expiration
        let expires_at = request.expires_in_days
            .or(Some(self.config.default_expiration_days))
            .filter(|&days| days > 0)
            .map(|days| Utc::now() + chrono::Duration::days(days as i64));

        let key = ApiKey {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            user_id,
            key_hash,
            prefix: self.config.key_prefix.clone(),
            suffix: raw_key.chars().rev().take(4).collect::<String>().chars().rev().collect(),
            scopes: request.scopes.map(|s| s.into_iter().collect()).unwrap_or_default(),
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            use_count: 0,
            allowed_ips: request.allowed_ips.unwrap_or_default(),
            active: true,
            revoked: false,
            revoked_at: None,
            metadata: request.metadata.unwrap_or_default(),
        };

        self.storage.create(&key).await?;

        self.event_emitter
            .emit(AuthEvent::ApiKeyCreated {
                user_id,
                key_id: key.id.clone(),
                key_name: key.name.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!(key_id = %key.id, "API key created");

        Ok(CreatedApiKey {
            key,
            raw_key: format!("{}{}", self.config.key_prefix, raw_key),
        })
    }

    /// Validate an API key and return identity
    #[instrument(skip(self, raw_key))]
    pub async fn validate_key(&self, raw_key: &str) -> AuthResult<AuthIdentity> {
        // Check prefix
        if !raw_key.starts_with(&self.config.key_prefix) {
            return Err(AuthError::InvalidCredentials);
        }

        // Remove prefix for hashing
        let key_value = &raw_key[self.config.key_prefix.len()..];
        let key_hash = self.hash_key(key_value);

        // Find key by hash
        let mut key = self
            .storage
            .find_by_hash(&key_hash)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Validate key
        if !key.is_valid() {
            if key.revoked {
                warn!(key_id = %key.id, "Attempted use of revoked API key");
            } else if key.is_expired() {
                warn!(key_id = %key.id, "Attempted use of expired API key");
            }
            return Err(AuthError::InvalidCredentials);
        }

        // Get user
        let user = self
            .user_repository
            .find_by_id(key.user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Check if user is valid
        if !user.enabled || user.locked {
            return Err(AuthError::AccountDisabled);
        }

        // Update usage stats
        key.last_used_at = Some(Utc::now());
        key.use_count += 1;
        self.storage.update(&key).await?;

        // Create identity with API key scopes as permissions
        let mut identity = user.to_identity(AuthMethod::ApiKey, None);
        if !key.scopes.is_empty() {
            // Restrict permissions to key scopes
            identity.permissions = key.scopes.clone();
        }

        Ok(identity)
    }

    /// Validate key and check IP
    pub async fn validate_key_with_ip(
        &self,
        raw_key: &str,
        ip: Option<&str>,
    ) -> AuthResult<AuthIdentity> {
        let identity = self.validate_key(raw_key).await?;

        // Find key again to check IP (or cache it)
        let key_value = &raw_key[self.config.key_prefix.len()..];
        let key_hash = self.hash_key(key_value);
        let key = self.storage.find_by_hash(&key_hash).await?.unwrap();

        if !key.is_ip_allowed(ip) {
            warn!(key_id = %key.id, ip = ?ip, "API key used from unauthorized IP");
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(identity)
    }

    /// Revoke an API key
    #[instrument(skip(self), fields(key_id = %key_id, user_id = %user_id))]
    pub async fn revoke_key(&self, user_id: UserId, key_id: &str) -> AuthResult<()> {
        let mut key = self
            .storage
            .get(key_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify ownership
        if key.user_id != user_id {
            return Err(AuthError::InsufficientPermissions);
        }

        key.revoked = true;
        key.revoked_at = Some(Utc::now());
        self.storage.update(&key).await?;

        self.event_emitter
            .emit(AuthEvent::ApiKeyRevoked {
                user_id,
                key_id: key_id.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        info!("API key revoked");
        Ok(())
    }

    /// Get all keys for a user
    pub async fn get_user_keys(&self, user_id: UserId) -> AuthResult<Vec<ApiKey>> {
        self.storage.get_user_keys(user_id).await
    }

    /// Get a specific key by ID (for owner only)
    pub async fn get_key(&self, user_id: UserId, key_id: &str) -> AuthResult<Option<ApiKey>> {
        let key = self.storage.get(key_id).await?;

        match key {
            Some(k) if k.user_id == user_id => Ok(Some(k)),
            Some(_) => Err(AuthError::InsufficientPermissions),
            None => Ok(None),
        }
    }

    /// Update key metadata
    #[instrument(skip(self), fields(key_id = %key_id, user_id = %user_id))]
    pub async fn update_key(
        &self,
        user_id: UserId,
        key_id: &str,
        name: Option<String>,
        scopes: Option<Vec<String>>,
        allowed_ips: Option<Vec<String>>,
    ) -> AuthResult<ApiKey> {
        let mut key = self
            .storage
            .get(key_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        if key.user_id != user_id {
            return Err(AuthError::InsufficientPermissions);
        }

        if let Some(name) = name {
            key.name = name;
        }
        if let Some(scopes) = scopes {
            key.scopes = scopes.into_iter().collect();
        }
        if let Some(ips) = allowed_ips {
            key.allowed_ips = ips;
        }

        self.storage.update(&key).await?;
        info!("API key updated");

        Ok(key)
    }

    /// Generate a random key value
    fn generate_key(&self) -> String {
        let mut rng = thread_rng();
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        (0..self.config.key_length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }

    /// Hash a key for storage
    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Clean up expired keys
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

/// API key authentication provider
pub struct ApiKeyAuthProvider {
    manager: Arc<ApiKeyManager>,
}

impl ApiKeyAuthProvider {
    pub fn new(manager: Arc<ApiKeyManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl crate::auth::provider::AuthProvider for ApiKeyAuthProvider {
    fn name(&self) -> &str {
        "api_key"
    }

    async fn authenticate(
        &self,
        credentials: &AuthCredentials,
        metadata: &AuthMetadata,
    ) -> AuthResult<AuthIdentity> {
        match credentials {
            AuthCredentials::ApiKey { key_id: _, key_secret } => {
                self.manager
                    .validate_key_with_ip(key_secret.expose(), metadata.ip_address.as_deref())
                    .await
            }
            _ => Err(AuthError::InvalidCredentials),
        }
    }

    async fn validate(&self, _identity: &AuthIdentity) -> AuthResult<bool> {
        // API key validation is done on each request
        Ok(true)
    }

    async fn revoke(&self, _identity: &AuthIdentity) -> AuthResult<()> {
        // Key revocation is done through the manager
        Ok(())
    }

    fn supports(&self, credentials: &AuthCredentials) -> bool {
        matches!(credentials, AuthCredentials::ApiKey { .. })
    }
}

/// API key storage trait
#[async_trait]
pub trait ApiKeyStorage: Send + Sync {
    /// Create a new key
    async fn create(&self, key: &ApiKey) -> AuthResult<()>;

    /// Get key by ID
    async fn get(&self, id: &str) -> AuthResult<Option<ApiKey>>;

    /// Find key by hash
    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<ApiKey>>;

    /// Update a key
    async fn update(&self, key: &ApiKey) -> AuthResult<()>;

    /// Delete a key
    async fn delete(&self, id: &str) -> AuthResult<()>;

    /// Get all keys for a user
    async fn get_user_keys(&self, user_id: UserId) -> AuthResult<Vec<ApiKey>>;

    /// Clean up expired keys
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory API key storage
pub struct InMemoryApiKeyStorage {
    keys: RwLock<HashMap<String, ApiKey>>,
    hash_index: RwLock<HashMap<String, String>>,
    user_index: RwLock<HashMap<UserId, Vec<String>>>,
}

impl InMemoryApiKeyStorage {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            hash_index: RwLock::new(HashMap::new()),
            user_index: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryApiKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiKeyStorage for InMemoryApiKeyStorage {
    async fn create(&self, key: &ApiKey) -> AuthResult<()> {
        let mut keys = self.keys.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;

        keys.insert(key.id.clone(), key.clone());
        hash_index.insert(key.key_hash.clone(), key.id.clone());
        user_index
            .entry(key.user_id)
            .or_insert_with(Vec::new)
            .push(key.id.clone());

        Ok(())
    }

    async fn get(&self, id: &str) -> AuthResult<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(id).cloned())
    }

    async fn find_by_hash(&self, hash: &str) -> AuthResult<Option<ApiKey>> {
        let hash_index = self.hash_index.read().await;
        let keys = self.keys.read().await;

        if let Some(id) = hash_index.get(hash) {
            Ok(keys.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    async fn update(&self, key: &ApiKey) -> AuthResult<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id.clone(), key.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> AuthResult<()> {
        let mut keys = self.keys.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;

        if let Some(key) = keys.remove(id) {
            hash_index.remove(&key.key_hash);
            if let Some(ids) = user_index.get_mut(&key.user_id) {
                ids.retain(|i| i != id);
            }
        }

        Ok(())
    }

    async fn get_user_keys(&self, user_id: UserId) -> AuthResult<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        let user_index = self.user_index.read().await;

        let key_ids = user_index.get(&user_id).cloned().unwrap_or_default();
        Ok(key_ids
            .iter()
            .filter_map(|id| keys.get(id).cloned())
            .collect())
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut keys = self.keys.write().await;
        let mut hash_index = self.hash_index.write().await;
        let mut user_index = self.user_index.write().await;

        let expired: Vec<_> = keys
            .iter()
            .filter(|(_, k)| k.is_expired())
            .map(|(id, k)| (id.clone(), k.clone()))
            .collect();

        let count = expired.len();

        for (id, key) in expired {
            keys.remove(&id);
            hash_index.remove(&key.key_hash);
            if let Some(ids) = user_index.get_mut(&key.user_id) {
                ids.retain(|i| i != &id);
            }
        }

        Ok(count)
    }
}

/// API key response (without sensitive data)
#[derive(Debug, Clone, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub suffix: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub active: bool,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            name: key.name,
            prefix: key.prefix,
            suffix: key.suffix,
            scopes: key.scopes.into_iter().collect(),
            created_at: key.created_at,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            active: key.active && !key.revoked,
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

    async fn setup_manager() -> ApiKeyManager {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        let user_repo = Arc::new(MockUserRepository);
        let events = Arc::new(NoOpEventEmitter);
        let config = ApiKeyConfig::default();

        ApiKeyManager::new(storage, user_repo, events, config)
    }

    #[tokio::test]
    async fn test_create_api_key() {
        let manager = setup_manager().await;
        let user_id = UserId::new();

        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            scopes: Some(vec!["read".to_string()]),
            expires_in_days: Some(30),
            allowed_ips: None,
            metadata: None,
        };

        let created = manager.create_key(user_id, request).await.unwrap();

        assert!(created.raw_key.starts_with("tk_"));
        assert_eq!(created.key.name, "Test Key");
        assert!(created.key.scopes.contains("read"));
        assert!(created.key.is_valid());
    }

    #[tokio::test]
    async fn test_validate_api_key() {
        let manager = setup_manager().await;
        let user_id = UserId::new();

        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            scopes: None,
            expires_in_days: None,
            allowed_ips: None,
            metadata: None,
        };

        let created = manager.create_key(user_id, request).await.unwrap();
        let identity = manager.validate_key(&created.raw_key).await.unwrap();

        assert_eq!(identity.user_id, user_id);
        assert_eq!(identity.auth_method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let manager = setup_manager().await;

        let result = manager.validate_key("tk_invalid_key_12345678").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_api_key() {
        let manager = setup_manager().await;
        let user_id = UserId::new();

        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            scopes: None,
            expires_in_days: None,
            allowed_ips: None,
            metadata: None,
        };

        let created = manager.create_key(user_id, request).await.unwrap();
        manager.revoke_key(user_id, &created.key.id).await.unwrap();

        // Should fail validation
        let result = manager.validate_key(&created.raw_key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_limit() {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        let user_repo = Arc::new(MockUserRepository);
        let events = Arc::new(NoOpEventEmitter);
        let mut config = ApiKeyConfig::default();
        config.max_keys_per_user = 2;

        let manager = ApiKeyManager::new(storage, user_repo, events, config);
        let user_id = UserId::new();

        let request = CreateApiKeyRequest {
            name: "Key".to_string(),
            scopes: None,
            expires_in_days: None,
            allowed_ips: None,
            metadata: None,
        };

        // Create 2 keys (should succeed)
        manager.create_key(user_id, request.clone()).await.unwrap();
        manager.create_key(user_id, request.clone()).await.unwrap();

        // Third key should fail
        let result = manager.create_key(user_id, request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_api_key_is_valid() {
        let key = ApiKey {
            id: "test".to_string(),
            name: "Test".to_string(),
            user_id: UserId::new(),
            key_hash: "hash".to_string(),
            prefix: "tk_".to_string(),
            suffix: "1234".to_string(),
            scopes: HashSet::new(),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            use_count: 0,
            allowed_ips: vec![],
            active: true,
            revoked: false,
            revoked_at: None,
            metadata: HashMap::new(),
        };

        assert!(key.is_valid());
    }

    #[test]
    fn test_api_key_ip_restriction() {
        let mut key = ApiKey {
            id: "test".to_string(),
            name: "Test".to_string(),
            user_id: UserId::new(),
            key_hash: "hash".to_string(),
            prefix: "tk_".to_string(),
            suffix: "1234".to_string(),
            scopes: HashSet::new(),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            use_count: 0,
            allowed_ips: vec!["192.168.1.1".to_string()],
            active: true,
            revoked: false,
            revoked_at: None,
            metadata: HashMap::new(),
        };

        assert!(key.is_ip_allowed(Some("192.168.1.1")));
        assert!(!key.is_ip_allowed(Some("10.0.0.1")));
        assert!(!key.is_ip_allowed(None));

        key.allowed_ips = vec![];
        assert!(key.is_ip_allowed(Some("10.0.0.1"))); // No restriction
    }

    struct MockUserRepository;

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

- **Spec 366**: Auth Types - Uses AuthIdentity and AuthCredentials
- **Spec 367**: Auth Configuration - Uses ApiKeyConfig
- **Spec 372**: Auth Middleware - Extracts API keys from requests
- **Spec 381**: Audit Logging - Logs API key events
