# 338 - Server Caching

**Phase:** 15 - Server
**Spec ID:** 338
**Status:** Planned
**Dependencies:** 332-server-config
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement caching layer with in-memory and Redis backends, cache invalidation, and cache-aside pattern support.

---

## Acceptance Criteria

- [ ] In-memory cache implementation
- [ ] Redis cache backend
- [ ] Cache-aside pattern helpers
- [ ] TTL support
- [ ] Cache invalidation
- [ ] Cache statistics
- [ ] Response caching middleware

---

## Implementation Details

### 1. Cache Trait (crates/tachikoma-server/src/cache/trait.rs)

```rust
//! Cache trait definition.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Cache operation result.
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache errors.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Key not found")]
    NotFound,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Backend error: {0}")]
    Backend(String),
    #[error("Connection error: {0}")]
    Connection(String),
}

/// Cache backend trait.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from cache.
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>>;

    /// Set a value in cache with TTL.
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()>;

    /// Delete a value from cache.
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Delete all keys matching a pattern.
    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64>;

    /// Check if key exists.
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Get time to live for key.
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;

    /// Clear all cache entries.
    async fn clear(&self) -> CacheResult<()>;

    /// Get cache statistics.
    async fn stats(&self) -> CacheStats;
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}
```

### 2. In-Memory Cache (crates/tachikoma-server/src/cache/memory.rs)

```rust
//! In-memory cache implementation.

use super::r#trait::{Cache, CacheError, CacheResult, CacheStats};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// In-memory cache entry.
struct CacheEntry {
    value: Vec<u8>,
    expires_at: Instant,
}

/// In-memory cache implementation.
pub struct MemoryCache {
    entries: DashMap<String, CacheEntry>,
    max_size: usize,
    stats: CacheStatsInner,
}

struct CacheStatsInner {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl MemoryCache {
    pub fn new(max_size: usize) -> Self {
        let cache = Self {
            entries: DashMap::new(),
            max_size,
            stats: CacheStatsInner {
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
                evictions: AtomicU64::new(0),
            },
        };

        // Start cleanup task
        cache.start_cleanup();

        cache
    }

    fn start_cleanup(&self) {
        // Note: In real implementation, would spawn a background task
        // that periodically removes expired entries
    }

    fn evict_if_needed(&self) {
        if self.entries.len() >= self.max_size {
            // Simple eviction: remove oldest entry
            // In production, use LRU or similar
            if let Some(oldest) = self.entries.iter().next() {
                let key = oldest.key().clone();
                self.entries.remove(&key);
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        match self.entries.get(key) {
            Some(entry) => {
                if entry.expires_at > Instant::now() {
                    self.stats.hits.fetch_add(1, Ordering::Relaxed);
                    let value: T = serde_json::from_slice(&entry.value)
                        .map_err(|e| CacheError::Serialization(e.to_string()))?;
                    debug!(key = key, "Cache hit");
                    Ok(Some(value))
                } else {
                    // Expired
                    drop(entry);
                    self.entries.remove(key);
                    self.stats.misses.fetch_add(1, Ordering::Relaxed);
                    debug!(key = key, "Cache miss (expired)");
                    Ok(None)
                }
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                debug!(key = key, "Cache miss");
                Ok(None)
            }
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()> {
        self.evict_if_needed();

        let serialized = serde_json::to_vec(value)
            .map_err(|e| CacheError::Serialization(e.to_string()))?;

        let entry = CacheEntry {
            value: serialized,
            expires_at: Instant::now() + ttl,
        };

        self.entries.insert(key.to_string(), entry);
        debug!(key = key, ttl_secs = ttl.as_secs(), "Cache set");
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        self.entries.remove(key);
        debug!(key = key, "Cache delete");
        Ok(())
    }

    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64> {
        let mut deleted = 0;
        let pattern = pattern.replace('*', "");

        self.entries.retain(|k, _| {
            if k.contains(&pattern) {
                deleted += 1;
                false
            } else {
                true
            }
        });

        debug!(pattern = pattern, deleted = deleted, "Cache delete pattern");
        Ok(deleted)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        Ok(self.entries.contains_key(key))
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        match self.entries.get(key) {
            Some(entry) => {
                let now = Instant::now();
                if entry.expires_at > now {
                    Ok(Some(entry.expires_at - now))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        self.entries.clear();
        debug!("Cache cleared");
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            size: self.entries.len() as u64,
            evictions: self.stats.evictions.load(Ordering::Relaxed),
        }
    }
}
```

### 3. Redis Cache (crates/tachikoma-server/src/cache/redis.rs)

```rust
//! Redis cache implementation.

use super::r#trait::{Cache, CacheError, CacheResult, CacheStats};
use async_trait::async_trait;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tracing::debug;

/// Redis cache implementation.
pub struct RedisCache {
    client: redis::Client,
    prefix: String,
    stats: CacheStatsInner,
}

struct CacheStatsInner {
    hits: AtomicU64,
    misses: AtomicU64,
}

impl RedisCache {
    pub fn new(url: &str, prefix: &str) -> CacheResult<Self> {
        let client = redis::Client::open(url)
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            prefix: prefix.to_string(),
            stats: CacheStatsInner {
                hits: AtomicU64::new(0),
                misses: AtomicU64::new(0),
            },
        })
    }

    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    async fn get_connection(&self) -> CacheResult<redis::aio::Connection> {
        self.client
            .get_async_connection()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> CacheResult<Option<T>> {
        let mut conn = self.get_connection().await?;
        let full_key = self.key(key);

        let result: Option<String> = conn
            .get(&full_key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        match result {
            Some(data) => {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                let value: T = serde_json::from_str(&data)
                    .map_err(|e| CacheError::Serialization(e.to_string()))?;
                debug!(key = key, "Redis cache hit");
                Ok(Some(value))
            }
            None => {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                debug!(key = key, "Redis cache miss");
                Ok(None)
            }
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;
        let full_key = self.key(key);

        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::Serialization(e.to_string()))?;

        conn.set_ex(&full_key, serialized, ttl.as_secs() as usize)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        debug!(key = key, ttl_secs = ttl.as_secs(), "Redis cache set");
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;
        let full_key = self.key(key);

        conn.del(&full_key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        debug!(key = key, "Redis cache delete");
        Ok(())
    }

    async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64> {
        let mut conn = self.get_connection().await?;
        let full_pattern = self.key(pattern);

        let keys: Vec<String> = conn
            .keys(&full_pattern)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn
            .del(&keys)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        debug!(pattern = pattern, deleted = deleted, "Redis cache delete pattern");
        Ok(deleted)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let mut conn = self.get_connection().await?;
        let full_key = self.key(key);

        let exists: bool = conn
            .exists(&full_key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        Ok(exists)
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let mut conn = self.get_connection().await?;
        let full_key = self.key(key);

        let ttl: i64 = conn
            .ttl(&full_key)
            .await
            .map_err(|e| CacheError::Backend(e.to_string()))?;

        if ttl > 0 {
            Ok(Some(Duration::from_secs(ttl as u64)))
        } else {
            Ok(None)
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        self.delete_pattern("*").await?;
        Ok(())
    }

    async fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.stats.hits.load(Ordering::Relaxed),
            misses: self.stats.misses.load(Ordering::Relaxed),
            size: 0, // Would need DBSIZE command
            evictions: 0,
        }
    }
}
```

### 4. Cache Helpers (crates/tachikoma-server/src/cache/helpers.rs)

```rust
//! Cache helper functions and patterns.

use super::r#trait::{Cache, CacheResult};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::time::Duration;

/// Cache-aside pattern helper.
pub async fn cache_aside<T, F, Fut>(
    cache: &dyn Cache,
    key: &str,
    ttl: Duration,
    fetch: F,
) -> CacheResult<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
    F: FnOnce() -> Fut,
    Fut: Future<Output = CacheResult<T>>,
{
    // Try cache first
    if let Some(value) = cache.get::<T>(key).await? {
        return Ok(value);
    }

    // Fetch from source
    let value = fetch().await?;

    // Store in cache
    cache.set(key, &value, ttl).await?;

    Ok(value)
}

/// Cache key builder for consistent key generation.
pub struct CacheKeyBuilder {
    parts: Vec<String>,
}

impl CacheKeyBuilder {
    pub fn new(prefix: &str) -> Self {
        Self {
            parts: vec![prefix.to_string()],
        }
    }

    pub fn add(mut self, part: impl ToString) -> Self {
        self.parts.push(part.to_string());
        self
    }

    pub fn add_opt(mut self, part: Option<impl ToString>) -> Self {
        if let Some(p) = part {
            self.parts.push(p.to_string());
        }
        self
    }

    pub fn build(self) -> String {
        self.parts.join(":")
    }
}

/// Common cache key prefixes.
pub mod keys {
    use super::CacheKeyBuilder;
    use uuid::Uuid;

    pub fn mission(id: Uuid) -> String {
        CacheKeyBuilder::new("mission").add(id).build()
    }

    pub fn mission_list(page: u32, page_size: u32) -> String {
        CacheKeyBuilder::new("missions")
            .add("list")
            .add(page)
            .add(page_size)
            .build()
    }

    pub fn spec(mission_id: Uuid, spec_id: &str) -> String {
        CacheKeyBuilder::new("spec")
            .add(mission_id)
            .add(spec_id)
            .build()
    }

    pub fn user(id: Uuid) -> String {
        CacheKeyBuilder::new("user").add(id).build()
    }

    pub fn session(token: &str) -> String {
        CacheKeyBuilder::new("session").add(token).build()
    }
}

/// Invalidation patterns.
pub mod invalidate {
    pub fn mission(id: uuid::Uuid) -> String {
        format!("mission:{}*", id)
    }

    pub fn all_missions() -> String {
        "missions:*".to_string()
    }

    pub fn user(id: uuid::Uuid) -> String {
        format!("user:{}*", id)
    }
}
```

---

## Testing Requirements

1. In-memory cache works
2. Redis cache works
3. TTL expiration correct
4. Cache invalidation works
5. Pattern deletion works
6. Stats tracking accurate
7. Cache-aside pattern correct

---

## Related Specs

- Depends on: [332-server-config.md](332-server-config.md)
- Next: [339-db-connection.md](339-db-connection.md)
- Used by: API handlers
