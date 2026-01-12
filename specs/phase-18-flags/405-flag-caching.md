# 405 - Feature Flag Caching

## Overview

Multi-layer caching strategy for feature flags to minimize latency and reduce load on the flag storage backend.

## Rust Implementation

```rust
// crates/flags/src/cache.rs

use crate::definition::FlagDefinition;
use crate::types::FlagId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache trait for flag storage
#[async_trait]
pub trait FlagCache: Send + Sync {
    /// Get a flag definition from cache
    async fn get(&self, flag_id: &FlagId) -> Option<CachedFlag>;

    /// Get multiple flags from cache
    async fn get_many(&self, flag_ids: &[FlagId]) -> HashMap<FlagId, CachedFlag>;

    /// Set a flag in the cache
    async fn set(&self, flag_id: &FlagId, flag: FlagDefinition, ttl: Option<Duration>);

    /// Set multiple flags in the cache
    async fn set_many(&self, flags: Vec<(FlagId, FlagDefinition)>, ttl: Option<Duration>);

    /// Invalidate a specific flag
    async fn invalidate(&self, flag_id: &FlagId);

    /// Invalidate multiple flags
    async fn invalidate_many(&self, flag_ids: &[FlagId]);

    /// Clear all cached flags
    async fn clear(&self);

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;
}

/// Cached flag with metadata
#[derive(Debug, Clone)]
pub struct CachedFlag {
    pub definition: FlagDefinition,
    pub cached_at: Instant,
    pub expires_at: Option<Instant>,
    pub hit_count: u64,
}

impl CachedFlag {
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Instant::now() > e).unwrap_or(false)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub evictions: u64,
    pub hit_rate: f64,
}

/// In-memory LRU cache implementation
pub struct InMemoryCache {
    cache: RwLock<HashMap<FlagId, CachedEntry>>,
    max_size: usize,
    default_ttl: Duration,
    stats: RwLock<CacheStatsInternal>,
}

struct CachedEntry {
    flag: FlagDefinition,
    cached_at: Instant,
    expires_at: Option<Instant>,
    last_accessed: Instant,
    hit_count: u64,
}

#[derive(Default)]
struct CacheStatsInternal {
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl InMemoryCache {
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            default_ttl,
            stats: RwLock::new(CacheStatsInternal::default()),
        }
    }

    async fn evict_if_needed(&self) {
        let mut cache = self.cache.write().await;

        if cache.len() >= self.max_size {
            // Find and remove least recently used entry
            let lru_key = cache.iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(k, _)| k.clone());

            if let Some(key) = lru_key {
                cache.remove(&key);
                self.stats.write().await.evictions += 1;
            }
        }
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;
                self.cleanup_expired().await;
            }
        });
    }

    async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();

        cache.retain(|_, entry| {
            entry.expires_at.map(|e| e > now).unwrap_or(true)
        });
    }
}

#[async_trait]
impl FlagCache for InMemoryCache {
    async fn get(&self, flag_id: &FlagId) -> Option<CachedFlag> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(flag_id) {
            // Check expiration
            if entry.expires_at.map(|e| Instant::now() > e).unwrap_or(false) {
                cache.remove(flag_id);
                self.stats.write().await.misses += 1;
                return None;
            }

            entry.last_accessed = Instant::now();
            entry.hit_count += 1;
            self.stats.write().await.hits += 1;

            Some(CachedFlag {
                definition: entry.flag.clone(),
                cached_at: entry.cached_at,
                expires_at: entry.expires_at,
                hit_count: entry.hit_count,
            })
        } else {
            self.stats.write().await.misses += 1;
            None
        }
    }

    async fn get_many(&self, flag_ids: &[FlagId]) -> HashMap<FlagId, CachedFlag> {
        let mut result = HashMap::new();

        for flag_id in flag_ids {
            if let Some(cached) = self.get(flag_id).await {
                result.insert(flag_id.clone(), cached);
            }
        }

        result
    }

    async fn set(&self, flag_id: &FlagId, flag: FlagDefinition, ttl: Option<Duration>) {
        self.evict_if_needed().await;

        let now = Instant::now();
        let ttl = ttl.unwrap_or(self.default_ttl);

        let entry = CachedEntry {
            flag,
            cached_at: now,
            expires_at: Some(now + ttl),
            last_accessed: now,
            hit_count: 0,
        };

        self.cache.write().await.insert(flag_id.clone(), entry);
    }

    async fn set_many(&self, flags: Vec<(FlagId, FlagDefinition)>, ttl: Option<Duration>) {
        for (flag_id, flag) in flags {
            self.set(&flag_id, flag, ttl).await;
        }
    }

    async fn invalidate(&self, flag_id: &FlagId) {
        self.cache.write().await.remove(flag_id);
    }

    async fn invalidate_many(&self, flag_ids: &[FlagId]) {
        let mut cache = self.cache.write().await;
        for flag_id in flag_ids {
            cache.remove(flag_id);
        }
    }

    async fn clear(&self) {
        self.cache.write().await.clear();
    }

    async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let stats = self.stats.read().await;

        let total = stats.hits + stats.misses;
        let hit_rate = if total > 0 {
            stats.hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits: stats.hits,
            misses: stats.misses,
            size: cache.len(),
            evictions: stats.evictions,
            hit_rate,
        }
    }
}

/// Redis-based distributed cache
pub struct RedisCache {
    client: redis::Client,
    prefix: String,
    default_ttl: Duration,
}

impl RedisCache {
    pub fn new(redis_url: &str, prefix: &str, default_ttl: Duration) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;

        Ok(Self {
            client,
            prefix: prefix.to_string(),
            default_ttl,
        })
    }

    fn cache_key(&self, flag_id: &FlagId) -> String {
        format!("{}:flag:{}", self.prefix, flag_id.as_str())
    }
}

#[async_trait]
impl FlagCache for RedisCache {
    async fn get(&self, flag_id: &FlagId) -> Option<CachedFlag> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        let key = self.cache_key(flag_id);

        let data: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()?;

        data.and_then(|json| {
            serde_json::from_str::<FlagDefinition>(&json).ok().map(|def| {
                CachedFlag {
                    definition: def,
                    cached_at: Instant::now(),
                    expires_at: None,
                    hit_count: 0,
                }
            })
        })
    }

    async fn get_many(&self, flag_ids: &[FlagId]) -> HashMap<FlagId, CachedFlag> {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return HashMap::new(),
        };

        let keys: Vec<String> = flag_ids.iter().map(|id| self.cache_key(id)).collect();

        let values: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        let mut result = HashMap::new();
        for (flag_id, value) in flag_ids.iter().zip(values) {
            if let Some(json) = value {
                if let Ok(def) = serde_json::from_str::<FlagDefinition>(&json) {
                    result.insert(flag_id.clone(), CachedFlag {
                        definition: def,
                        cached_at: Instant::now(),
                        expires_at: None,
                        hit_count: 0,
                    });
                }
            }
        }

        result
    }

    async fn set(&self, flag_id: &FlagId, flag: FlagDefinition, ttl: Option<Duration>) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return,
        };

        let key = self.cache_key(flag_id);
        let json = match serde_json::to_string(&flag) {
            Ok(j) => j,
            Err(_) => return,
        };

        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs() as usize;

        let _: Result<(), _> = redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(&json)
            .query_async(&mut conn)
            .await;
    }

    async fn set_many(&self, flags: Vec<(FlagId, FlagDefinition)>, ttl: Option<Duration>) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return,
        };

        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs() as usize;
        let mut pipe = redis::pipe();

        for (flag_id, flag) in flags {
            let key = self.cache_key(&flag_id);
            if let Ok(json) = serde_json::to_string(&flag) {
                pipe.cmd("SETEX").arg(&key).arg(ttl_secs).arg(&json);
            }
        }

        let _: Result<(), _> = pipe.query_async(&mut conn).await;
    }

    async fn invalidate(&self, flag_id: &FlagId) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return,
        };

        let key = self.cache_key(flag_id);
        let _: Result<(), _> = redis::cmd("DEL").arg(&key).query_async(&mut conn).await;
    }

    async fn invalidate_many(&self, flag_ids: &[FlagId]) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return,
        };

        let keys: Vec<String> = flag_ids.iter().map(|id| self.cache_key(id)).collect();
        let _: Result<(), _> = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await;
    }

    async fn clear(&self) {
        let mut conn = match self.client.get_async_connection().await {
            Ok(c) => c,
            Err(_) => return,
        };

        let pattern = format!("{}:flag:*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

        if !keys.is_empty() {
            let _: Result<(), _> = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await;
        }
    }

    async fn stats(&self) -> CacheStats {
        // Redis doesn't track these stats directly
        CacheStats::default()
    }
}

/// Multi-layer cache (L1 in-memory + L2 Redis)
pub struct TieredCache {
    l1: Arc<InMemoryCache>,
    l2: Option<Arc<RedisCache>>,
}

impl TieredCache {
    pub fn new(
        l1_size: usize,
        l1_ttl: Duration,
        redis_url: Option<&str>,
        l2_ttl: Duration,
    ) -> Self {
        let l1 = Arc::new(InMemoryCache::new(l1_size, l1_ttl));

        let l2 = redis_url.and_then(|url| {
            RedisCache::new(url, "tachikoma", l2_ttl)
                .ok()
                .map(Arc::new)
        });

        Self { l1, l2 }
    }
}

#[async_trait]
impl FlagCache for TieredCache {
    async fn get(&self, flag_id: &FlagId) -> Option<CachedFlag> {
        // Try L1 first
        if let Some(cached) = self.l1.get(flag_id).await {
            return Some(cached);
        }

        // Try L2
        if let Some(l2) = &self.l2 {
            if let Some(cached) = l2.get(flag_id).await {
                // Promote to L1
                self.l1.set(flag_id, cached.definition.clone(), None).await;
                return Some(cached);
            }
        }

        None
    }

    async fn get_many(&self, flag_ids: &[FlagId]) -> HashMap<FlagId, CachedFlag> {
        let mut result = self.l1.get_many(flag_ids).await;

        // Find missing keys
        let missing: Vec<_> = flag_ids.iter()
            .filter(|id| !result.contains_key(*id))
            .cloned()
            .collect();

        // Try L2 for missing
        if !missing.is_empty() {
            if let Some(l2) = &self.l2 {
                let l2_result = l2.get_many(&missing).await;

                // Promote to L1
                for (id, cached) in &l2_result {
                    self.l1.set(id, cached.definition.clone(), None).await;
                }

                result.extend(l2_result);
            }
        }

        result
    }

    async fn set(&self, flag_id: &FlagId, flag: FlagDefinition, ttl: Option<Duration>) {
        // Set in both layers
        self.l1.set(flag_id, flag.clone(), ttl).await;

        if let Some(l2) = &self.l2 {
            l2.set(flag_id, flag, ttl).await;
        }
    }

    async fn set_many(&self, flags: Vec<(FlagId, FlagDefinition)>, ttl: Option<Duration>) {
        self.l1.set_many(flags.clone(), ttl).await;

        if let Some(l2) = &self.l2 {
            l2.set_many(flags, ttl).await;
        }
    }

    async fn invalidate(&self, flag_id: &FlagId) {
        self.l1.invalidate(flag_id).await;

        if let Some(l2) = &self.l2 {
            l2.invalidate(flag_id).await;
        }
    }

    async fn invalidate_many(&self, flag_ids: &[FlagId]) {
        self.l1.invalidate_many(flag_ids).await;

        if let Some(l2) = &self.l2 {
            l2.invalidate_many(flag_ids).await;
        }
    }

    async fn clear(&self) {
        self.l1.clear().await;

        if let Some(l2) = &self.l2 {
            l2.clear().await;
        }
    }

    async fn stats(&self) -> CacheStats {
        self.l1.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inmemory_cache() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));

        let flag = FlagDefinition::new_boolean("test", "Test", false).unwrap();
        let flag_id = FlagId::new("test");

        cache.set(&flag_id, flag.clone(), None).await;

        let cached = cache.get(&flag_id).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().definition.id, flag.id);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = InMemoryCache::new(100, Duration::from_millis(50));
        let flag = FlagDefinition::new_boolean("test", "Test", false).unwrap();
        let flag_id = FlagId::new("test");

        cache.set(&flag_id, flag, Some(Duration::from_millis(50))).await;

        // Should exist immediately
        assert!(cache.get(&flag_id).await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be expired
        assert!(cache.get(&flag_id).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryCache::new(100, Duration::from_secs(60));
        let flag = FlagDefinition::new_boolean("test", "Test", false).unwrap();
        let flag_id = FlagId::new("test");

        cache.set(&flag_id, flag, None).await;

        // Cache hit
        cache.get(&flag_id).await;
        cache.get(&flag_id).await;

        // Cache miss
        cache.get(&FlagId::new("nonexistent")).await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }
}
```

## Caching Strategy

| Layer | Technology | TTL | Purpose |
|-------|-----------|-----|---------|
| L1 | In-memory | 30s-5m | Hot flags, ultra-low latency |
| L2 | Redis | 5m-30m | Shared across instances |
| L3 | Database | N/A | Source of truth |

## Related Specs

- 393-flag-storage.md - Storage layer
- 404-flag-sync.md - Sync invalidation
- 402-flag-sdk-rust.md - SDK caching
