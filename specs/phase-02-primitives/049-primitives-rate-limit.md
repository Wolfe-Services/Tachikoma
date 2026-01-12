# 049 - Primitives Rate Limiting

**Phase:** 2 - Five Primitives
**Spec ID:** 049
**Status:** Planned
**Dependencies:** 046-primitives-trait
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement rate limiting for primitive operations to prevent resource exhaustion and enable fair resource sharing.

---

## Acceptance Criteria

- [x] Per-primitive rate limits
- [x] Global rate limits across all primitives
- [x] Token bucket algorithm implementation
- [x] Configurable limits
- [x] Backpressure support
- [x] Rate limit headers/metadata in responses

---

## Implementation Details

### 1. Rate Limiter Module (src/rate_limit/mod.rs)

```rust
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
```

### 2. Token Bucket (src/rate_limit/bucket.rs)

```rust
//! Token bucket rate limiting algorithm.

use std::time::{Duration, Instant};

/// Token bucket for rate limiting.
pub struct TokenBucket {
    /// Maximum tokens (burst size).
    capacity: u64,
    /// Current token count.
    tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last refill time.
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket.
    pub fn new(tokens_per_second: u64, burst_size: u64) -> Self {
        Self {
            capacity: burst_size,
            tokens: burst_size as f64,
            refill_rate: tokens_per_second as f64,
            last_refill: Instant::now(),
        }
    }

    /// Try to acquire a token.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Try to acquire multiple tokens.
    pub fn try_acquire_n(&mut self, n: u64) -> bool {
        self.refill();

        let n = n as f64;
        if self.tokens >= n {
            self.tokens -= n;
            true
        } else {
            false
        }
    }

    /// Get available tokens.
    pub fn available_tokens(&self) -> u64 {
        self.tokens as u64
    }

    /// Time until next token available.
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            let seconds = needed / self.refill_rate;
            Duration::from_secs_f64(seconds)
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = elapsed.as_secs_f64() * self.refill_rate;

        self.tokens = (self.tokens + new_tokens).min(self.capacity as f64);
        self.last_refill = now;
    }

    /// Reset the bucket to full.
    pub fn reset(&mut self) {
        self.tokens = self.capacity as f64;
        self.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 5);

        // Start with burst capacity
        assert_eq!(bucket.available_tokens(), 5);

        // Use tokens
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert_eq!(bucket.available_tokens(), 3);
    }

    #[test]
    fn test_token_bucket_exhaustion() {
        let mut bucket = TokenBucket::new(1, 2);

        // Use all tokens
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10, 5);

        // Use all tokens
        for _ in 0..5 {
            assert!(bucket.try_acquire());
        }
        assert!(!bucket.try_acquire());

        // Wait for refill (100ms = 1 token at 10/s)
        sleep(Duration::from_millis(120));

        // Should have new token
        assert!(bucket.try_acquire());
    }

    #[test]
    fn test_acquire_multiple() {
        let mut bucket = TokenBucket::new(10, 5);

        assert!(bucket.try_acquire_n(3));
        assert_eq!(bucket.available_tokens(), 2);
        assert!(!bucket.try_acquire_n(3));
    }
}
```

### 3. Rate Limit Configuration (src/rate_limit/config.rs)

```rust
//! Rate limit configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Global tokens per second.
    pub global_tokens_per_second: u64,
    /// Global burst size.
    pub global_burst_size: u64,
    /// Default per-primitive tokens per second.
    pub default_tokens_per_second: u64,
    /// Default per-primitive burst size.
    pub default_burst_size: u64,
    /// Per-primitive rate limits (tokens/second).
    pub primitive_limits: HashMap<String, u64>,
    /// Per-primitive burst sizes.
    pub primitive_burst: HashMap<String, u64>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut primitive_limits = HashMap::new();
        primitive_limits.insert("read_file".to_string(), 100);
        primitive_limits.insert("list_files".to_string(), 50);
        primitive_limits.insert("bash".to_string(), 10);
        primitive_limits.insert("edit_file".to_string(), 20);
        primitive_limits.insert("code_search".to_string(), 30);

        let mut primitive_burst = HashMap::new();
        primitive_burst.insert("read_file".to_string(), 200);
        primitive_burst.insert("list_files".to_string(), 100);
        primitive_burst.insert("bash".to_string(), 20);
        primitive_burst.insert("edit_file".to_string(), 40);
        primitive_burst.insert("code_search".to_string(), 60);

        Self {
            global_tokens_per_second: 200,
            global_burst_size: 500,
            default_tokens_per_second: 50,
            default_burst_size: 100,
            primitive_limits,
            primitive_burst,
        }
    }
}

impl RateLimitConfig {
    /// Create a new config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable rate limiting (for testing).
    pub fn disabled() -> Self {
        Self {
            global_tokens_per_second: u64::MAX,
            global_burst_size: u64::MAX,
            default_tokens_per_second: u64::MAX,
            default_burst_size: u64::MAX,
            primitive_limits: HashMap::new(),
            primitive_burst: HashMap::new(),
        }
    }

    /// Set global limit.
    pub fn global_limit(mut self, tokens_per_second: u64, burst: u64) -> Self {
        self.global_tokens_per_second = tokens_per_second;
        self.global_burst_size = burst;
        self
    }

    /// Set limit for a primitive.
    pub fn primitive_limit(
        mut self,
        primitive: &str,
        tokens_per_second: u64,
        burst: u64,
    ) -> Self {
        self.primitive_limits.insert(primitive.to_string(), tokens_per_second);
        self.primitive_burst.insert(primitive.to_string(), burst);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert!(config.primitive_limits.contains_key("bash"));
        assert!(config.global_tokens_per_second > 0);
    }

    #[test]
    fn test_disabled_config() {
        let config = RateLimitConfig::disabled();
        assert_eq!(config.global_tokens_per_second, u64::MAX);
    }

    #[test]
    fn test_builder() {
        let config = RateLimitConfig::new()
            .global_limit(100, 200)
            .primitive_limit("bash", 5, 10);

        assert_eq!(config.global_tokens_per_second, 100);
        assert_eq!(config.primitive_limits.get("bash"), Some(&5));
    }
}
```

---

## Testing Requirements

1. Token bucket refills correctly over time
2. Burst capacity works as expected
3. Rate limits are enforced per primitive
4. Global rate limits work
5. Timeout acquisition works
6. Status reports correct values
7. Shared limiter is thread-safe

---

## Related Specs

- Depends on: [046-primitives-trait.md](046-primitives-trait.md)
- Next: [050-primitives-tests.md](050-primitives-tests.md)
- Related: [022-http-retry-logic.md](../phase-01-common/022-http-retry-logic.md)
