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