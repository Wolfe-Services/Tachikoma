# Spec 382: Authentication Rate Limiting

## Phase
17 - Authentication/Authorization

## Spec ID
382

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~9%

---

## Objective

Implement rate limiting for authentication endpoints to prevent brute force attacks and denial of service. The rate limiter should support multiple strategies (fixed window, sliding window, token bucket) and various key types (IP, user, endpoint).

---

## Acceptance Criteria

- [ ] Implement `RateLimiter` with configurable strategies
- [ ] Support fixed window rate limiting
- [ ] Support sliding window rate limiting
- [ ] Support token bucket algorithm
- [ ] Rate limit by IP address, user ID, or combined
- [ ] Different limits for different auth operations
- [ ] Provide rate limit headers in responses
- [ ] Support Redis backend for distributed limiting

---

## Implementation Details

### Rate Limiting System

```rust
// src/auth/rate_limit.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn, instrument};

use crate::auth::{
    config::RateLimitConfig,
    events::{AuthEvent, AuthEventEmitter},
    types::*,
};

/// Rate limit key for identifying what to limit
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RateLimitKey {
    /// Limit by IP address
    Ip(String),
    /// Limit by user ID
    User(UserId),
    /// Limit by IP and action
    IpAction(String, RateLimitAction),
    /// Limit by user and action
    UserAction(UserId, RateLimitAction),
    /// Limit by IP and user
    IpUser(String, UserId),
    /// Custom key
    Custom(String),
}

impl RateLimitKey {
    pub fn to_string(&self) -> String {
        match self {
            RateLimitKey::Ip(ip) => format!("ip:{}", ip),
            RateLimitKey::User(uid) => format!("user:{}", uid),
            RateLimitKey::IpAction(ip, action) => format!("ip:{}:action:{:?}", ip, action),
            RateLimitKey::UserAction(uid, action) => format!("user:{}:action:{:?}", uid, action),
            RateLimitKey::IpUser(ip, uid) => format!("ip:{}:user:{}", ip, uid),
            RateLimitKey::Custom(key) => format!("custom:{}", key),
        }
    }
}

/// Rate limited actions
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitAction {
    Login,
    TokenRefresh,
    PasswordReset,
    MfaVerify,
    ApiRequest,
    Registration,
}

/// Rate limit result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Number of remaining requests
    pub remaining: u32,
    /// Total limit
    pub limit: u32,
    /// When the limit resets (Unix timestamp)
    pub reset_at: i64,
    /// Retry after seconds (if not allowed)
    pub retry_after: Option<u64>,
}

impl RateLimitResult {
    /// Create an allowed result
    pub fn allowed(remaining: u32, limit: u32, reset_at: DateTime<Utc>) -> Self {
        Self {
            allowed: true,
            remaining,
            limit,
            reset_at: reset_at.timestamp(),
            retry_after: None,
        }
    }

    /// Create a denied result
    pub fn denied(limit: u32, reset_at: DateTime<Utc>) -> Self {
        let retry_after = (reset_at - Utc::now()).num_seconds().max(0) as u64;
        Self {
            allowed: false,
            remaining: 0,
            limit,
            reset_at: reset_at.timestamp(),
            retry_after: Some(retry_after),
        }
    }

    /// Convert to rate limit headers
    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            ("X-RateLimit-Remaining".to_string(), self.remaining.to_string()),
            ("X-RateLimit-Reset".to_string(), self.reset_at.to_string()),
        ];

        if let Some(retry_after) = self.retry_after {
            headers.push(("Retry-After".to_string(), retry_after.to_string()));
        }

        headers
    }
}

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed
    async fn check(&self, key: &RateLimitKey) -> RateLimitResult;

    /// Record a request (increment counter)
    async fn record(&self, key: &RateLimitKey) -> RateLimitResult;

    /// Reset limit for a key
    async fn reset(&self, key: &RateLimitKey);
}

/// Fixed window rate limiter
pub struct FixedWindowRateLimiter {
    storage: Arc<dyn RateLimitStorage>,
    default_limit: u32,
    default_window_secs: u64,
    action_limits: HashMap<RateLimitAction, (u32, u64)>,
}

impl FixedWindowRateLimiter {
    pub fn new(storage: Arc<dyn RateLimitStorage>, config: &RateLimitConfig) -> Self {
        let mut action_limits = HashMap::new();
        action_limits.insert(
            RateLimitAction::Login,
            (config.login_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::TokenRefresh,
            (config.refresh_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::PasswordReset,
            (config.password_reset_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::MfaVerify,
            (config.mfa_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::ApiRequest,
            (config.api_requests, config.api_window_secs),
        );

        Self {
            storage,
            default_limit: config.login_attempts,
            default_window_secs: config.window_secs,
            action_limits,
        }
    }

    fn get_limit_for_key(&self, key: &RateLimitKey) -> (u32, u64) {
        match key {
            RateLimitKey::IpAction(_, action) | RateLimitKey::UserAction(_, action) => {
                self.action_limits
                    .get(action)
                    .copied()
                    .unwrap_or((self.default_limit, self.default_window_secs))
            }
            _ => (self.default_limit, self.default_window_secs),
        }
    }
}

#[async_trait]
impl RateLimiter for FixedWindowRateLimiter {
    async fn check(&self, key: &RateLimitKey) -> RateLimitResult {
        let (limit, window_secs) = self.get_limit_for_key(key);
        let window_start = get_window_start(window_secs);
        let window_end = window_start + Duration::seconds(window_secs as i64);

        let count = self
            .storage
            .get_count(&key.to_string(), window_start)
            .await
            .unwrap_or(0);

        if count >= limit {
            RateLimitResult::denied(limit, window_end)
        } else {
            RateLimitResult::allowed(limit - count, limit, window_end)
        }
    }

    async fn record(&self, key: &RateLimitKey) -> RateLimitResult {
        let (limit, window_secs) = self.get_limit_for_key(key);
        let window_start = get_window_start(window_secs);
        let window_end = window_start + Duration::seconds(window_secs as i64);

        let count = self
            .storage
            .increment(&key.to_string(), window_start, window_secs)
            .await
            .unwrap_or(1);

        if count > limit {
            RateLimitResult::denied(limit, window_end)
        } else {
            RateLimitResult::allowed(limit - count, limit, window_end)
        }
    }

    async fn reset(&self, key: &RateLimitKey) {
        let _ = self.storage.delete(&key.to_string()).await;
    }
}

/// Sliding window rate limiter
pub struct SlidingWindowRateLimiter {
    storage: Arc<dyn SlidingWindowStorage>,
    default_limit: u32,
    default_window_secs: u64,
    action_limits: HashMap<RateLimitAction, (u32, u64)>,
}

impl SlidingWindowRateLimiter {
    pub fn new(storage: Arc<dyn SlidingWindowStorage>, config: &RateLimitConfig) -> Self {
        let mut action_limits = HashMap::new();
        action_limits.insert(
            RateLimitAction::Login,
            (config.login_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::TokenRefresh,
            (config.refresh_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::PasswordReset,
            (config.password_reset_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::MfaVerify,
            (config.mfa_attempts, config.window_secs),
        );
        action_limits.insert(
            RateLimitAction::ApiRequest,
            (config.api_requests, config.api_window_secs),
        );

        Self {
            storage,
            default_limit: config.login_attempts,
            default_window_secs: config.window_secs,
            action_limits,
        }
    }

    fn get_limit_for_key(&self, key: &RateLimitKey) -> (u32, u64) {
        match key {
            RateLimitKey::IpAction(_, action) | RateLimitKey::UserAction(_, action) => {
                self.action_limits
                    .get(action)
                    .copied()
                    .unwrap_or((self.default_limit, self.default_window_secs))
            }
            _ => (self.default_limit, self.default_window_secs),
        }
    }
}

#[async_trait]
impl RateLimiter for SlidingWindowRateLimiter {
    async fn check(&self, key: &RateLimitKey) -> RateLimitResult {
        let (limit, window_secs) = self.get_limit_for_key(key);
        let window_start = Utc::now() - Duration::seconds(window_secs as i64);

        let count = self
            .storage
            .count_in_window(&key.to_string(), window_start)
            .await
            .unwrap_or(0);

        let reset_at = Utc::now() + Duration::seconds(window_secs as i64);

        if count >= limit {
            RateLimitResult::denied(limit, reset_at)
        } else {
            RateLimitResult::allowed(limit - count, limit, reset_at)
        }
    }

    async fn record(&self, key: &RateLimitKey) -> RateLimitResult {
        let (limit, window_secs) = self.get_limit_for_key(key);

        // Add timestamp
        let _ = self.storage.add_timestamp(&key.to_string(), Utc::now()).await;

        // Clean old entries
        let window_start = Utc::now() - Duration::seconds(window_secs as i64);
        let _ = self.storage.remove_before(&key.to_string(), window_start).await;

        // Count current window
        let count = self
            .storage
            .count_in_window(&key.to_string(), window_start)
            .await
            .unwrap_or(1);

        let reset_at = Utc::now() + Duration::seconds(window_secs as i64);

        if count > limit {
            RateLimitResult::denied(limit, reset_at)
        } else {
            RateLimitResult::allowed(limit - count, limit, reset_at)
        }
    }

    async fn reset(&self, key: &RateLimitKey) {
        let _ = self.storage.clear(&key.to_string()).await;
    }
}

/// Token bucket rate limiter
pub struct TokenBucketRateLimiter {
    storage: Arc<dyn TokenBucketStorage>,
    default_capacity: u32,
    default_refill_rate: f64,
    action_config: HashMap<RateLimitAction, (u32, f64)>,
}

impl TokenBucketRateLimiter {
    pub fn new(storage: Arc<dyn TokenBucketStorage>, config: &RateLimitConfig) -> Self {
        let mut action_config = HashMap::new();
        // Convert window-based config to token bucket
        // Capacity = max requests, refill_rate = requests per second
        action_config.insert(
            RateLimitAction::Login,
            (config.login_attempts, config.login_attempts as f64 / config.window_secs as f64),
        );
        action_config.insert(
            RateLimitAction::ApiRequest,
            (config.api_requests, config.api_requests as f64 / config.api_window_secs as f64),
        );

        Self {
            storage,
            default_capacity: config.login_attempts,
            default_refill_rate: config.login_attempts as f64 / config.window_secs as f64,
            action_config,
        }
    }
}

#[async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn check(&self, key: &RateLimitKey) -> RateLimitResult {
        let (capacity, refill_rate) = self.get_config_for_key(key);
        let bucket = self.storage.get_bucket(&key.to_string()).await;

        let tokens = bucket.map(|b| b.available_tokens(capacity, refill_rate))
            .unwrap_or(capacity as f64);

        let reset_at = Utc::now() + Duration::seconds((capacity as f64 / refill_rate) as i64);

        if tokens >= 1.0 {
            RateLimitResult::allowed(tokens as u32, capacity, reset_at)
        } else {
            RateLimitResult::denied(capacity, reset_at)
        }
    }

    async fn record(&self, key: &RateLimitKey) -> RateLimitResult {
        let (capacity, refill_rate) = self.get_config_for_key(key);

        let bucket = self.storage.get_or_create_bucket(&key.to_string(), capacity).await;
        let remaining = bucket.consume(capacity, refill_rate);
        let _ = self.storage.save_bucket(&key.to_string(), &bucket).await;

        let reset_at = Utc::now() + Duration::seconds((capacity as f64 / refill_rate) as i64);

        if remaining >= 0.0 {
            RateLimitResult::allowed(remaining as u32, capacity, reset_at)
        } else {
            RateLimitResult::denied(capacity, reset_at)
        }
    }

    async fn reset(&self, key: &RateLimitKey) {
        let _ = self.storage.delete_bucket(&key.to_string()).await;
    }
}

impl TokenBucketRateLimiter {
    fn get_config_for_key(&self, key: &RateLimitKey) -> (u32, f64) {
        match key {
            RateLimitKey::IpAction(_, action) | RateLimitKey::UserAction(_, action) => {
                self.action_config
                    .get(action)
                    .copied()
                    .unwrap_or((self.default_capacity, self.default_refill_rate))
            }
            _ => (self.default_capacity, self.default_refill_rate),
        }
    }
}

/// Token bucket state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_update: DateTime<Utc>,
}

impl TokenBucket {
    pub fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Utc::now(),
        }
    }

    pub fn available_tokens(&self, capacity: u32, refill_rate: f64) -> f64 {
        let elapsed = (Utc::now() - self.last_update).num_milliseconds() as f64 / 1000.0;
        let refilled = self.tokens + (elapsed * refill_rate);
        refilled.min(capacity as f64)
    }

    pub fn consume(&mut self, capacity: u32, refill_rate: f64) -> f64 {
        let available = self.available_tokens(capacity, refill_rate);
        self.tokens = (available - 1.0).max(0.0);
        self.last_update = Utc::now();
        available - 1.0
    }
}

/// Get the start of the current fixed window
fn get_window_start(window_secs: u64) -> DateTime<Utc> {
    let now = Utc::now().timestamp();
    let window_start = now - (now % window_secs as i64);
    DateTime::from_timestamp(window_start, 0).unwrap()
}

/// Storage trait for fixed window rate limiting
#[async_trait]
pub trait RateLimitStorage: Send + Sync {
    async fn get_count(&self, key: &str, window_start: DateTime<Utc>) -> AuthResult<u32>;
    async fn increment(&self, key: &str, window_start: DateTime<Utc>, ttl_secs: u64) -> AuthResult<u32>;
    async fn delete(&self, key: &str) -> AuthResult<()>;
}

/// Storage trait for sliding window rate limiting
#[async_trait]
pub trait SlidingWindowStorage: Send + Sync {
    async fn add_timestamp(&self, key: &str, timestamp: DateTime<Utc>) -> AuthResult<()>;
    async fn count_in_window(&self, key: &str, window_start: DateTime<Utc>) -> AuthResult<u32>;
    async fn remove_before(&self, key: &str, timestamp: DateTime<Utc>) -> AuthResult<()>;
    async fn clear(&self, key: &str) -> AuthResult<()>;
}

/// Storage trait for token bucket rate limiting
#[async_trait]
pub trait TokenBucketStorage: Send + Sync {
    async fn get_bucket(&self, key: &str) -> Option<TokenBucket>;
    async fn get_or_create_bucket(&self, key: &str, capacity: u32) -> TokenBucket;
    async fn save_bucket(&self, key: &str, bucket: &TokenBucket) -> AuthResult<()>;
    async fn delete_bucket(&self, key: &str) -> AuthResult<()>;
}

/// In-memory fixed window storage
pub struct InMemoryRateLimitStorage {
    counters: RwLock<HashMap<String, (u32, DateTime<Utc>)>>,
}

impl InMemoryRateLimitStorage {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRateLimitStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitStorage for InMemoryRateLimitStorage {
    async fn get_count(&self, key: &str, window_start: DateTime<Utc>) -> AuthResult<u32> {
        let counters = self.counters.read().await;
        if let Some((count, start)) = counters.get(key) {
            if *start == window_start {
                return Ok(*count);
            }
        }
        Ok(0)
    }

    async fn increment(&self, key: &str, window_start: DateTime<Utc>, _ttl_secs: u64) -> AuthResult<u32> {
        let mut counters = self.counters.write().await;
        let entry = counters.entry(key.to_string()).or_insert((0, window_start));

        if entry.1 != window_start {
            *entry = (1, window_start);
        } else {
            entry.0 += 1;
        }

        Ok(entry.0)
    }

    async fn delete(&self, key: &str) -> AuthResult<()> {
        let mut counters = self.counters.write().await;
        counters.remove(key);
        Ok(())
    }
}

/// In-memory sliding window storage
pub struct InMemorySlidingWindowStorage {
    timestamps: RwLock<HashMap<String, Vec<DateTime<Utc>>>>,
}

impl InMemorySlidingWindowStorage {
    pub fn new() -> Self {
        Self {
            timestamps: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SlidingWindowStorage for InMemorySlidingWindowStorage {
    async fn add_timestamp(&self, key: &str, timestamp: DateTime<Utc>) -> AuthResult<()> {
        let mut timestamps = self.timestamps.write().await;
        timestamps.entry(key.to_string()).or_default().push(timestamp);
        Ok(())
    }

    async fn count_in_window(&self, key: &str, window_start: DateTime<Utc>) -> AuthResult<u32> {
        let timestamps = self.timestamps.read().await;
        let count = timestamps
            .get(key)
            .map(|ts| ts.iter().filter(|&&t| t >= window_start).count() as u32)
            .unwrap_or(0);
        Ok(count)
    }

    async fn remove_before(&self, key: &str, timestamp: DateTime<Utc>) -> AuthResult<()> {
        let mut timestamps = self.timestamps.write().await;
        if let Some(ts) = timestamps.get_mut(key) {
            ts.retain(|&t| t >= timestamp);
        }
        Ok(())
    }

    async fn clear(&self, key: &str) -> AuthResult<()> {
        let mut timestamps = self.timestamps.write().await;
        timestamps.remove(key);
        Ok(())
    }
}

/// In-memory token bucket storage
pub struct InMemoryTokenBucketStorage {
    buckets: RwLock<HashMap<String, TokenBucket>>,
}

impl InMemoryTokenBucketStorage {
    pub fn new() -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl TokenBucketStorage for InMemoryTokenBucketStorage {
    async fn get_bucket(&self, key: &str) -> Option<TokenBucket> {
        let buckets = self.buckets.read().await;
        buckets.get(key).cloned()
    }

    async fn get_or_create_bucket(&self, key: &str, capacity: u32) -> TokenBucket {
        let mut buckets = self.buckets.write().await;
        buckets.entry(key.to_string()).or_insert_with(|| TokenBucket::new(capacity)).clone()
    }

    async fn save_bucket(&self, key: &str, bucket: &TokenBucket) -> AuthResult<()> {
        let mut buckets = self.buckets.write().await;
        buckets.insert(key.to_string(), bucket.clone());
        Ok(())
    }

    async fn delete_bucket(&self, key: &str) -> AuthResult<()> {
        let mut buckets = self.buckets.write().await;
        buckets.remove(key);
        Ok(())
    }
}

/// Rate limit manager
pub struct RateLimitManager {
    limiter: Arc<dyn RateLimiter>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: RateLimitConfig,
}

impl RateLimitManager {
    pub fn new(
        limiter: Arc<dyn RateLimiter>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: RateLimitConfig,
    ) -> Self {
        Self {
            limiter,
            event_emitter,
            config,
        }
    }

    /// Check and record rate limit
    #[instrument(skip(self), fields(key = ?key))]
    pub async fn check_rate_limit(&self, key: RateLimitKey) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::allowed(u32::MAX, u32::MAX, Utc::now() + Duration::hours(1));
        }

        let result = self.limiter.record(&key).await;

        if !result.allowed {
            warn!(key = ?key, "Rate limit exceeded");

            self.event_emitter
                .emit(AuthEvent::RateLimitExceeded {
                    key: key.to_string(),
                    timestamp: Utc::now(),
                })
                .await;
        }

        result
    }

    /// Just check without recording
    pub async fn check_only(&self, key: &RateLimitKey) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::allowed(u32::MAX, u32::MAX, Utc::now() + Duration::hours(1));
        }

        self.limiter.check(key).await
    }

    /// Reset rate limit for key
    pub async fn reset(&self, key: &RateLimitKey) {
        self.limiter.reset(key).await;
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

    #[tokio::test]
    async fn test_fixed_window_rate_limiting() {
        let storage = Arc::new(InMemoryRateLimitStorage::new());
        let mut config = RateLimitConfig::default();
        config.login_attempts = 3;
        config.window_secs = 60;

        let limiter = FixedWindowRateLimiter::new(storage, &config);

        let key = RateLimitKey::Ip("192.168.1.1".to_string());

        // First 3 requests should be allowed
        for i in 0..3 {
            let result = limiter.record(&key).await;
            assert!(result.allowed, "Request {} should be allowed", i + 1);
        }

        // Fourth request should be denied
        let result = limiter.record(&key).await;
        assert!(!result.allowed, "Fourth request should be denied");
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_sliding_window_rate_limiting() {
        let storage = Arc::new(InMemorySlidingWindowStorage::new());
        let mut config = RateLimitConfig::default();
        config.login_attempts = 3;
        config.window_secs = 60;

        let limiter = SlidingWindowRateLimiter::new(storage, &config);

        let key = RateLimitKey::IpAction("192.168.1.1".to_string(), RateLimitAction::Login);

        // First 3 requests should be allowed
        for _ in 0..3 {
            let result = limiter.record(&key).await;
            assert!(result.allowed);
        }

        // Fourth request should be denied
        let result = limiter.record(&key).await;
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_token_bucket_rate_limiting() {
        let storage = Arc::new(InMemoryTokenBucketStorage::new());
        let mut config = RateLimitConfig::default();
        config.login_attempts = 5;
        config.window_secs = 60;

        let limiter = TokenBucketRateLimiter::new(storage, &config);

        let key = RateLimitKey::Ip("192.168.1.1".to_string());

        // First 5 requests should be allowed
        for _ in 0..5 {
            let result = limiter.record(&key).await;
            assert!(result.allowed);
        }

        // Sixth request should be denied
        let result = limiter.record(&key).await;
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let storage = Arc::new(InMemoryRateLimitStorage::new());
        let mut config = RateLimitConfig::default();
        config.login_attempts = 2;

        let limiter = FixedWindowRateLimiter::new(storage, &config);

        let key = RateLimitKey::Ip("192.168.1.1".to_string());

        // Use up limit
        limiter.record(&key).await;
        limiter.record(&key).await;
        assert!(!limiter.record(&key).await.allowed);

        // Reset
        limiter.reset(&key).await;

        // Should be allowed again
        let result = limiter.record(&key).await;
        assert!(result.allowed);
    }

    #[test]
    fn test_rate_limit_result_headers() {
        let result = RateLimitResult::allowed(5, 10, Utc::now());
        let headers = result.to_headers();

        assert!(headers.iter().any(|(k, _)| k == "X-RateLimit-Limit"));
        assert!(headers.iter().any(|(k, _)| k == "X-RateLimit-Remaining"));
        assert!(headers.iter().any(|(k, _)| k == "X-RateLimit-Reset"));
    }

    #[test]
    fn test_rate_limit_key_to_string() {
        let ip_key = RateLimitKey::Ip("192.168.1.1".to_string());
        assert!(ip_key.to_string().contains("ip:192.168.1.1"));

        let action_key = RateLimitKey::IpAction("10.0.0.1".to_string(), RateLimitAction::Login);
        assert!(action_key.to_string().contains("Login"));
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10);
        assert_eq!(bucket.tokens, 10.0);

        // Consume one
        let remaining = bucket.consume(10, 1.0);
        assert!(remaining >= 8.0); // Approximately 9 remaining
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

- **Spec 366**: Auth Types - Uses AuthError for rate limit errors
- **Spec 367**: Auth Configuration - Uses RateLimitConfig
- **Spec 372**: Auth Middleware - Applies rate limiting to requests
- **Spec 381**: Audit Logging - Logs rate limit events
- **Spec 383**: Account Lockout - Works with rate limiting
