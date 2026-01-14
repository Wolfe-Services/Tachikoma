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
    /// Create a new Redis cache with the given URL and key prefix.
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

    /// Build a full key with prefix.
    fn key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    /// Get a Redis connection.
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
            size: 0, // Would need DBSIZE command for actual implementation
            evictions: 0,
        }
    }
}