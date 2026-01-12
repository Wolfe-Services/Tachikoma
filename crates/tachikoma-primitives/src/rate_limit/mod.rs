//! Rate limiting for primitive operations.

mod bucket;
mod config;

pub use bucket::TokenBucket;
pub use config::RateLimitConfig;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::error::{PrimitiveError, PrimitiveResult};

/// Rate limiter for primitives.
pub struct RateLimiter {
    /// Per-primitive limiters.
    primitive_limiters: HashMap<String, Arc<Mutex<TokenBucket>>>,
    /// Global limiter.
    global_limiter: Arc<Mutex<TokenBucket>>,
    /// Configuration.
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        let global_limiter = Arc::new(Mutex::new(TokenBucket::new(
            config.global_tokens_per_second,
            config.global_burst_size,
        )));

        Self {
            primitive_limiters: HashMap::new(),
            global_limiter,
            config,
        }
    }

    /// Get or create a limiter for a primitive.
    fn get_primitive_limiter(&mut self, primitive: &str) -> Arc<Mutex<TokenBucket>> {
        if !self.primitive_limiters.contains_key(primitive) {
            let limit = self.config.primitive_limits
                .get(primitive)
                .copied()
                .unwrap_or(self.config.default_tokens_per_second);

            let burst = self.config.primitive_burst
                .get(primitive)
                .copied()
                .unwrap_or(self.config.default_burst_size);

            let bucket = Arc::new(Mutex::new(TokenBucket::new(limit, burst)));
            self.primitive_limiters.insert(primitive.to_string(), bucket);
        }

        self.primitive_limiters.get(primitive).unwrap().clone()
    }

    /// Try to acquire permission for an operation.
    pub async fn try_acquire(&mut self, primitive: &str) -> PrimitiveResult<RateLimitPermit> {
        // Check global limit first
        {
            let mut global = self.global_limiter.lock().await;
            if !global.try_acquire() {
                warn!("Global rate limit exceeded");
                return Err(PrimitiveError::Validation {
                    message: "Global rate limit exceeded".to_string(),
                });
            }
        }

        // Check primitive-specific limit
        let limiter = self.get_primitive_limiter(primitive);
        {
            let mut bucket = limiter.lock().await;
            if !bucket.try_acquire() {
                warn!("Rate limit exceeded for primitive: {}", primitive);
                return Err(PrimitiveError::Validation {
                    message: format!("Rate limit exceeded for {}", primitive),
                });
            }
        }

        debug!("Rate limit permit acquired for {}", primitive);

        Ok(RateLimitPermit {
            primitive: primitive.to_string(),
            acquired_at: std::time::Instant::now(),
        })
    }

    /// Acquire with waiting if limit exceeded.
    pub async fn acquire(&mut self, primitive: &str) -> RateLimitPermit {
        loop {
            match self.try_acquire(primitive).await {
                Ok(permit) => return permit,
                Err(_) => {
                    // Wait and retry
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Acquire with timeout.
    pub async fn acquire_timeout(
        &mut self,
        primitive: &str,
        timeout: Duration,
    ) -> PrimitiveResult<RateLimitPermit> {
        let deadline = std::time::Instant::now() + timeout;

        loop {
            match self.try_acquire(primitive).await {
                Ok(permit) => return Ok(permit),
                Err(_) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(PrimitiveError::Timeout {
                            duration: timeout,
                        });
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    /// Get current status.
    pub async fn status(&self, primitive: &str) -> RateLimitStatus {
        let global = self.global_limiter.lock().await;
        let global_available = global.available_tokens();
        drop(global);

        let primitive_available = if let Some(limiter) = self.primitive_limiters.get(primitive) {
            let bucket = limiter.lock().await;
            bucket.available_tokens()
        } else {
            self.config.default_burst_size
        };

        RateLimitStatus {
            primitive: primitive.to_string(),
            primitive_tokens_available: primitive_available,
            global_tokens_available: global_available,
            primitive_limit: self.config.primitive_limits
                .get(primitive)
                .copied()
                .unwrap_or(self.config.default_tokens_per_second),
            global_limit: self.config.global_tokens_per_second,
        }
    }
}

/// A permit from the rate limiter.
#[derive(Debug)]
pub struct RateLimitPermit {
    pub primitive: String,
    pub acquired_at: std::time::Instant,
}

/// Rate limit status.
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub primitive: String,
    pub primitive_tokens_available: u64,
    pub global_tokens_available: u64,
    pub primitive_limit: u64,
    pub global_limit: u64,
}

impl RateLimitStatus {
    /// Format as headers.
    pub fn to_headers(&self) -> Vec<(String, String)> {
        vec![
            ("X-RateLimit-Limit".to_string(), self.primitive_limit.to_string()),
            ("X-RateLimit-Remaining".to_string(), self.primitive_tokens_available.to_string()),
            ("X-RateLimit-Global-Limit".to_string(), self.global_limit.to_string()),
            ("X-RateLimit-Global-Remaining".to_string(), self.global_tokens_available.to_string()),
        ]
    }
}

/// Shared rate limiter for use across the application.
pub struct SharedRateLimiter {
    inner: Arc<Mutex<RateLimiter>>,
}

impl SharedRateLimiter {
    /// Create a new shared rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiter::new(config))),
        }
    }

    /// Try to acquire a permit.
    pub async fn try_acquire(&self, primitive: &str) -> PrimitiveResult<RateLimitPermit> {
        let mut limiter = self.inner.lock().await;
        limiter.try_acquire(primitive).await
    }

    /// Acquire with waiting.
    pub async fn acquire(&self, primitive: &str) -> RateLimitPermit {
        let mut limiter = self.inner.lock().await;
        limiter.acquire(primitive).await
    }

    /// Get status.
    pub async fn status(&self, primitive: &str) -> RateLimitStatus {
        let limiter = self.inner.lock().await;
        limiter.status(primitive).await
    }
}

impl Clone for SharedRateLimiter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);

        // Should succeed initially
        let permit = limiter.try_acquire("read_file").await;
        assert!(permit.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_exhaustion() {
        let config = RateLimitConfig {
            global_tokens_per_second: 1,
            global_burst_size: 2,
            default_tokens_per_second: 1,
            default_burst_size: 2,
            ..Default::default()
        };

        let mut limiter = RateLimiter::new(config);

        // First two should succeed (burst)
        assert!(limiter.try_acquire("read_file").await.is_ok());
        assert!(limiter.try_acquire("read_file").await.is_ok());

        // Third should fail
        assert!(limiter.try_acquire("read_file").await.is_err());
    }

    #[tokio::test]
    async fn test_shared_limiter() {
        let config = RateLimitConfig::default();
        let limiter = SharedRateLimiter::new(config);

        let limiter1 = limiter.clone();
        let limiter2 = limiter.clone();

        // Both clones share the same limiter
        let permit1 = limiter1.try_acquire("bash").await;
        assert!(permit1.is_ok());

        // Status should show one token used
        let status = limiter2.status("bash").await;
        assert!(status.primitive_tokens_available < status.primitive_limit);
    }
}