//! In-memory cache implementation.

use super::r#trait::{Cache, CacheError, CacheResult, CacheStats};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::interval;
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
    /// Create a new in-memory cache with the specified maximum size.
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

    /// Start a background task to clean up expired entries.
    fn start_cleanup(&self) {
        let entries = self.entries.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(30));
            loop {
                cleanup_interval.tick().await;
                let now = Instant::now();
                let mut expired_keys = Vec::new();

                // Collect expired keys
                for entry in entries.iter() {
                    if entry.value().expires_at <= now {
                        expired_keys.push(entry.key().clone());
                    }
                }

                // Remove expired entries
                for key in expired_keys {
                    entries.remove(&key);
                }
            }
        });
    }

    /// Evict entries if the cache is at capacity.
    fn evict_if_needed(&self) {
        if self.entries.len() >= self.max_size {
            // Simple eviction: remove oldest entry
            // In production, would use LRU or similar
            if let Some(oldest) = self.entries.iter().next() {
                let key = oldest.key().clone();
                drop(oldest);
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
        match self.entries.get(key) {
            Some(entry) => Ok(entry.expires_at > Instant::now()),
            None => Ok(false),
        }
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