# Spec 369: Session Management

## Phase
17 - Authentication/Authorization

## Spec ID
369

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~11%

---

## Objective

Implement secure session management for maintaining user authentication state. This includes session creation, validation, refresh, and destruction, with support for multiple storage backends (in-memory, Redis, database). The implementation should handle session fixation prevention, concurrent session limits, and idle timeout.

---

## Acceptance Criteria

- [ ] Implement `SessionManager` for session lifecycle management
- [ ] Create `Session` struct with all necessary metadata
- [ ] Support multiple storage backends (Memory, Redis, Database)
- [ ] Implement session creation with secure ID generation
- [ ] Implement session validation and refresh
- [ ] Support session destruction (logout)
- [ ] Handle idle timeout and absolute timeout
- [ ] Implement concurrent session limiting
- [ ] Support session fixation prevention (ID regeneration)

---

## Implementation Details

### Session Types and Manager

```rust
// src/auth/session.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::SessionConfig,
    events::{AuthEvent, AuthEventEmitter},
    types::*,
};

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: SessionId,

    /// User ID this session belongs to
    pub user_id: UserId,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session was last accessed
    pub last_accessed_at: DateTime<Utc>,

    /// When the session expires (absolute timeout)
    pub expires_at: DateTime<Utc>,

    /// IP address from session creation
    pub ip_address: Option<String>,

    /// User agent from session creation
    pub user_agent: Option<String>,

    /// Additional session data
    pub data: HashMap<String, serde_json::Value>,

    /// Whether this session has been explicitly invalidated
    pub invalidated: bool,
}

impl Session {
    /// Create a new session
    pub fn new(
        user_id: UserId,
        lifetime: Duration,
        metadata: &AuthMetadata,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            user_id,
            created_at: now,
            last_accessed_at: now,
            expires_at: now + lifetime,
            ip_address: metadata.ip_address.clone(),
            user_agent: metadata.user_agent.clone(),
            data: HashMap::new(),
            invalidated: false,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        !self.invalidated && !self.is_expired()
    }

    /// Check if session has been idle too long
    pub fn is_idle(&self, idle_timeout: Duration) -> bool {
        Utc::now() > self.last_accessed_at + idle_timeout
    }

    /// Touch the session (update last accessed time)
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Set session data
    pub fn set_data(&mut self, key: impl Into<String>, value: impl Serialize) {
        if let Ok(json) = serde_json::to_value(value) {
            self.data.insert(key.into(), json);
        }
    }

    /// Get session data
    pub fn get_data<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Session manager handles session lifecycle
pub struct SessionManager {
    storage: Arc<dyn SessionStorage>,
    config: SessionConfig,
    event_emitter: Arc<dyn AuthEventEmitter>,
}

impl SessionManager {
    pub fn new(
        storage: Arc<dyn SessionStorage>,
        config: SessionConfig,
        event_emitter: Arc<dyn AuthEventEmitter>,
    ) -> Self {
        Self {
            storage,
            config,
            event_emitter,
        }
    }

    /// Create a new session for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn create_session(
        &self,
        user_id: UserId,
        metadata: &AuthMetadata,
    ) -> AuthResult<Session> {
        // Check concurrent session limit
        if self.config.max_sessions_per_user > 0 {
            let existing = self.storage.get_user_sessions(user_id).await?;
            if existing.len() >= self.config.max_sessions_per_user {
                // Remove oldest session
                if let Some(oldest) = existing.into_iter().min_by_key(|s| s.created_at) {
                    self.destroy_session(oldest.id).await?;
                }
            }
        }

        let lifetime = Duration::seconds(self.config.lifetime_secs as i64);
        let session = Session::new(user_id, lifetime, metadata);

        self.storage.store(&session).await?;

        self.event_emitter
            .emit(AuthEvent::SessionCreated {
                session_id: session.id,
                user_id,
                ip_address: metadata.ip_address.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!(session_id = %session.id, "Session created");
        Ok(session)
    }

    /// Get a session by ID
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session(&self, session_id: SessionId) -> AuthResult<Option<Session>> {
        let session = self.storage.get(session_id).await?;

        if let Some(ref s) = session {
            if !s.is_valid() {
                return Ok(None);
            }

            // Check idle timeout
            if self.config.idle_timeout_secs > 0 {
                let idle_timeout = Duration::seconds(self.config.idle_timeout_secs as i64);
                if s.is_idle(idle_timeout) {
                    self.destroy_session(session_id).await?;
                    return Ok(None);
                }
            }
        }

        Ok(session)
    }

    /// Validate and touch a session
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn validate_session(&self, session_id: SessionId) -> AuthResult<Session> {
        let mut session = self
            .get_session(session_id)
            .await?
            .ok_or(AuthError::SessionInvalid)?;

        // Update last accessed time
        session.touch();
        self.storage.store(&session).await?;

        Ok(session)
    }

    /// Regenerate session ID (for session fixation prevention)
    #[instrument(skip(self), fields(old_session_id = %session_id))]
    pub async fn regenerate_session(&self, session_id: SessionId) -> AuthResult<Session> {
        let old_session = self
            .get_session(session_id)
            .await?
            .ok_or(AuthError::SessionInvalid)?;

        // Create new session with same data
        let mut new_session = Session {
            id: SessionId::new(),
            user_id: old_session.user_id,
            created_at: old_session.created_at,
            last_accessed_at: Utc::now(),
            expires_at: old_session.expires_at,
            ip_address: old_session.ip_address,
            user_agent: old_session.user_agent,
            data: old_session.data,
            invalidated: false,
        };

        // Store new session
        self.storage.store(&new_session).await?;

        // Destroy old session
        self.storage.delete(session_id).await?;

        info!(
            old_id = %session_id,
            new_id = %new_session.id,
            "Session regenerated"
        );

        Ok(new_session)
    }

    /// Destroy a session (logout)
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn destroy_session(&self, session_id: SessionId) -> AuthResult<()> {
        if let Some(session) = self.storage.get(session_id).await? {
            self.storage.delete(session_id).await?;

            self.event_emitter
                .emit(AuthEvent::SessionDestroyed {
                    session_id,
                    user_id: session.user_id,
                    timestamp: Utc::now(),
                })
                .await;

            info!("Session destroyed");
        }

        Ok(())
    }

    /// Destroy all sessions for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn destroy_user_sessions(&self, user_id: UserId) -> AuthResult<()> {
        let sessions = self.storage.get_user_sessions(user_id).await?;

        for session in sessions {
            self.storage.delete(session.id).await?;
        }

        self.event_emitter
            .emit(AuthEvent::AllSessionsDestroyed {
                user_id,
                timestamp: Utc::now(),
            })
            .await;

        info!("All user sessions destroyed");
        Ok(())
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: UserId) -> AuthResult<Vec<Session>> {
        let sessions = self.storage.get_user_sessions(user_id).await?;
        Ok(sessions.into_iter().filter(|s| s.is_valid()).collect())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

/// Session storage backend trait
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Store a session
    async fn store(&self, session: &Session) -> AuthResult<()>;

    /// Get a session by ID
    async fn get(&self, id: SessionId) -> AuthResult<Option<Session>>;

    /// Delete a session
    async fn delete(&self, id: SessionId) -> AuthResult<()>;

    /// Get all sessions for a user
    async fn get_user_sessions(&self, user_id: UserId) -> AuthResult<Vec<Session>>;

    /// Clean up expired sessions
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory session storage
pub struct InMemorySessionStorage {
    sessions: RwLock<HashMap<SessionId, Session>>,
    user_sessions: RwLock<HashMap<UserId, Vec<SessionId>>>,
}

impl InMemorySessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            user_sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStorage for InMemorySessionStorage {
    async fn store(&self, session: &Session) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        // Add to main storage
        sessions.insert(session.id, session.clone());

        // Add to user index
        user_sessions
            .entry(session.user_id)
            .or_insert_with(Vec::new)
            .push(session.id);

        Ok(())
    }

    async fn get(&self, id: SessionId) -> AuthResult<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&id).cloned())
    }

    async fn delete(&self, id: SessionId) -> AuthResult<()> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        if let Some(session) = sessions.remove(&id) {
            if let Some(ids) = user_sessions.get_mut(&session.user_id) {
                ids.retain(|&sid| sid != id);
            }
        }

        Ok(())
    }

    async fn get_user_sessions(&self, user_id: UserId) -> AuthResult<Vec<Session>> {
        let sessions = self.sessions.read().await;
        let user_sessions = self.user_sessions.read().await;

        let session_ids = user_sessions.get(&user_id);
        match session_ids {
            Some(ids) => Ok(ids
                .iter()
                .filter_map(|id| sessions.get(id).cloned())
                .collect()),
            None => Ok(vec![]),
        }
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut sessions = self.sessions.write().await;
        let mut user_sessions = self.user_sessions.write().await;

        let expired: Vec<_> = sessions
            .iter()
            .filter(|(_, s)| !s.is_valid())
            .map(|(id, s)| (*id, s.user_id))
            .collect();

        let count = expired.len();

        for (id, user_id) in expired {
            sessions.remove(&id);
            if let Some(ids) = user_sessions.get_mut(&user_id) {
                ids.retain(|&sid| sid != id);
            }
        }

        Ok(count)
    }
}

/// Redis session storage
#[cfg(feature = "redis")]
pub struct RedisSessionStorage {
    client: redis::Client,
    prefix: String,
    ttl_secs: u64,
}

#[cfg(feature = "redis")]
impl RedisSessionStorage {
    pub fn new(url: &str, ttl_secs: u64) -> AuthResult<Self> {
        let client = redis::Client::open(url).map_err(|e| {
            AuthError::ConfigError(format!("Redis connection error: {}", e))
        })?;

        Ok(Self {
            client,
            prefix: "session:".to_string(),
            ttl_secs,
        })
    }

    fn session_key(&self, id: SessionId) -> String {
        format!("{}{}", self.prefix, id)
    }

    fn user_sessions_key(&self, user_id: UserId) -> String {
        format!("{}user:{}", self.prefix, user_id)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl SessionStorage for RedisSessionStorage {
    async fn store(&self, session: &Session) -> AuthResult<()> {
        let mut conn = self.client.get_async_connection().await.map_err(|e| {
            AuthError::Internal(format!("Redis error: {}", e))
        })?;

        let session_json = serde_json::to_string(session).map_err(|e| {
            AuthError::Internal(format!("Serialization error: {}", e))
        })?;

        // Store session with TTL
        redis::cmd("SETEX")
            .arg(&self.session_key(session.id))
            .arg(self.ttl_secs)
            .arg(&session_json)
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;

        // Add to user's session set
        redis::cmd("SADD")
            .arg(&self.user_sessions_key(session.user_id))
            .arg(session.id.to_string())
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn get(&self, id: SessionId) -> AuthResult<Option<Session>> {
        let mut conn = self.client.get_async_connection().await.map_err(|e| {
            AuthError::Internal(format!("Redis error: {}", e))
        })?;

        let result: Option<String> = redis::cmd("GET")
            .arg(&self.session_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;

        match result {
            Some(json) => {
                let session: Session = serde_json::from_str(&json).map_err(|e| {
                    AuthError::Internal(format!("Deserialization error: {}", e))
                })?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, id: SessionId) -> AuthResult<()> {
        let mut conn = self.client.get_async_connection().await.map_err(|e| {
            AuthError::Internal(format!("Redis error: {}", e))
        })?;

        // Get session first to get user_id
        if let Some(session) = self.get(id).await? {
            // Remove from user's session set
            redis::cmd("SREM")
                .arg(&self.user_sessions_key(session.user_id))
                .arg(id.to_string())
                .query_async(&mut conn)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;
        }

        // Delete session
        redis::cmd("DEL")
            .arg(&self.session_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;

        Ok(())
    }

    async fn get_user_sessions(&self, user_id: UserId) -> AuthResult<Vec<Session>> {
        let mut conn = self.client.get_async_connection().await.map_err(|e| {
            AuthError::Internal(format!("Redis error: {}", e))
        })?;

        let session_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&self.user_sessions_key(user_id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis error: {}", e)))?;

        let mut sessions = Vec::new();
        for id_str in session_ids {
            if let Ok(id) = id_str.parse::<uuid::Uuid>() {
                if let Some(session) = self.get(SessionId::from(id)).await? {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        // Redis handles expiration automatically with TTL
        Ok(0)
    }
}

/// Session cookie builder
pub struct SessionCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: SameSite,
    pub max_age: Option<i64>,
}

impl SessionCookie {
    pub fn new(session: &Session, config: &SessionConfig) -> Self {
        Self {
            name: config.cookie_name.clone(),
            value: session.id.to_string(),
            domain: config.cookie_domain.clone(),
            path: config.cookie_path.clone(),
            secure: config.cookie_secure,
            http_only: config.cookie_http_only,
            same_site: config.cookie_same_site,
            max_age: Some(config.lifetime_secs as i64),
        }
    }

    /// Create a cookie that clears the session
    pub fn clear(config: &SessionConfig) -> Self {
        Self {
            name: config.cookie_name.clone(),
            value: String::new(),
            domain: config.cookie_domain.clone(),
            path: config.cookie_path.clone(),
            secure: config.cookie_secure,
            http_only: config.cookie_http_only,
            same_site: config.cookie_same_site,
            max_age: Some(0),
        }
    }

    /// Convert to Set-Cookie header value
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        parts.push(format!("Path={}", self.path));

        if self.secure {
            parts.push("Secure".to_string());
        }

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }

        let same_site = match self.same_site {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        };
        parts.push(format!("SameSite={}", same_site));

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age));
        }

        parts.join("; ")
    }
}

use crate::auth::config::SameSite;
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> AuthMetadata {
        AuthMetadata {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            request_id: None,
            geo_location: None,
        }
    }

    #[test]
    fn test_session_creation() {
        let user_id = UserId::new();
        let metadata = create_test_metadata();
        let session = Session::new(user_id, Duration::hours(1), &metadata);

        assert_eq!(session.user_id, user_id);
        assert!(session.is_valid());
        assert!(!session.is_expired());
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_session_expiration() {
        let user_id = UserId::new();
        let metadata = create_test_metadata();
        let mut session = Session::new(user_id, Duration::seconds(-1), &metadata);

        assert!(session.is_expired());
        assert!(!session.is_valid());
    }

    #[test]
    fn test_session_invalidation() {
        let user_id = UserId::new();
        let metadata = create_test_metadata();
        let mut session = Session::new(user_id, Duration::hours(1), &metadata);

        assert!(session.is_valid());
        session.invalidate();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_session_data() {
        let user_id = UserId::new();
        let metadata = create_test_metadata();
        let mut session = Session::new(user_id, Duration::hours(1), &metadata);

        session.set_data("key1", "value1");
        session.set_data("key2", 42i32);

        assert_eq!(session.get_data::<String>("key1"), Some("value1".to_string()));
        assert_eq!(session.get_data::<i32>("key2"), Some(42));
        assert_eq!(session.get_data::<String>("key3"), None);
    }

    #[tokio::test]
    async fn test_session_manager_create() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let config = SessionConfig::default();
        let events = Arc::new(NoOpEventEmitter);
        let manager = SessionManager::new(storage, config, events);

        let user_id = UserId::new();
        let metadata = create_test_metadata();

        let session = manager.create_session(user_id, &metadata).await.unwrap();

        assert_eq!(session.user_id, user_id);
    }

    #[tokio::test]
    async fn test_session_manager_get() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let config = SessionConfig::default();
        let events = Arc::new(NoOpEventEmitter);
        let manager = SessionManager::new(storage, config, events);

        let user_id = UserId::new();
        let metadata = create_test_metadata();

        let session = manager.create_session(user_id, &metadata).await.unwrap();
        let retrieved = manager.get_session(session.id).await.unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn test_session_manager_destroy() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let config = SessionConfig::default();
        let events = Arc::new(NoOpEventEmitter);
        let manager = SessionManager::new(storage, config, events);

        let user_id = UserId::new();
        let metadata = create_test_metadata();

        let session = manager.create_session(user_id, &metadata).await.unwrap();
        manager.destroy_session(session.id).await.unwrap();

        let retrieved = manager.get_session(session.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_session_concurrent_limit() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let mut config = SessionConfig::default();
        config.max_sessions_per_user = 2;
        let events = Arc::new(NoOpEventEmitter);
        let manager = SessionManager::new(storage, config, events);

        let user_id = UserId::new();
        let metadata = create_test_metadata();

        let s1 = manager.create_session(user_id, &metadata).await.unwrap();
        let s2 = manager.create_session(user_id, &metadata).await.unwrap();
        let s3 = manager.create_session(user_id, &metadata).await.unwrap();

        // First session should be removed
        let sessions = manager.get_user_sessions(user_id).await.unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(!sessions.iter().any(|s| s.id == s1.id));
    }

    #[test]
    fn test_session_cookie() {
        let user_id = UserId::new();
        let metadata = create_test_metadata();
        let session = Session::new(user_id, Duration::hours(1), &metadata);
        let config = SessionConfig::default();

        let cookie = SessionCookie::new(&session, &config);
        let header = cookie.to_header_value();

        assert!(header.contains(&config.cookie_name));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("SameSite=Strict"));
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

- **Spec 366**: Auth Types - Uses SessionId and AuthMetadata
- **Spec 367**: Auth Configuration - Uses SessionConfig
- **Spec 368**: Local Auth - Creates sessions on login
- **Spec 372**: Auth Middleware - Validates sessions in requests
- **Spec 381**: Audit Logging - Logs session events
- **Spec 384**: Auth Events - Emits session events
