# 072 - Backend Rate Limiting

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 072
**Status:** Planned
**Dependencies:** 051-backend-trait, 070-backend-factory
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement rate limiting infrastructure for backends to prevent API throttling, including request tracking, token-based limits, and automatic backoff when rate limits are hit.

---

## Acceptance Criteria

- [ ] Request rate limiting (RPM)
- [ ] Token rate limiting (TPM)
- [ ] Provider-specific limits
- [ ] Automatic backoff on 429 errors
- [ ] Rate limit usage tracking
- [ ] Predictive rate limit avoidance

---

## Implementation Details

### 1. Rate Limit Types (src/rate_limit/types.rs)

```rust
//! Rate limiting types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute.
    pub requests_per_minute: u32,
    /// Maximum tokens per minute.
    pub tokens_per_minute: u32,
    /// Maximum concurrent requests.
    pub max_concurrent: u32,
    /// Burst allowance (extra requests).
    pub burst_allowance: u32,
}

impl RateLimitConfig {
    /// Default limits for Claude.
    pub fn claude_default() -> Self {
        Self {
            requests_per_minute: 50,
            tokens_per_minute: 40_000,
            max_concurrent: 5,
            burst_allowance: 10,
        }
    }

    /// Default limits for OpenAI.
    pub fn openai_default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 90_000,
            max_concurrent: 10,
            burst_allowance: 20,
        }
    }

    /// Default limits for Gemini.
    pub fn gemini_default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 120_000,
            max_concurrent: 10,
            burst_allowance: 15,
        }
    }

    /// No limits (for local backends).
    pub fn unlimited() -> Self {
        Self {
            requests_per_minute: u32::MAX,
            tokens_per_minute: u32::MAX,
            max_concurrent: u32::MAX,
            burst_allowance: 0,
        }
    }
}

/// Current rate limit usage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RateLimitUsage {
    /// Requests made in current window.
    pub requests_used: u32,
    /// Tokens used in current window.
    pub tokens_used: u64,
    /// Current concurrent requests.
    pub concurrent: u32,
    /// Requests remaining.
    pub requests_remaining: Option<u32>,
    /// Tokens remaining.
    pub tokens_remaining: Option<u64>,
    /// Time until reset.
    pub reset_in: Option<Duration>,
}

impl RateLimitUsage {
    /// Check if we're close to the limit.
    pub fn is_near_limit(&self, threshold: f32) -> bool {
        if let Some(remaining) = self.requests_remaining {
            if remaining < 5 {
                return true;
            }
        }

        if let Some(remaining) = self.tokens_remaining {
            if remaining < 1000 {
                return true;
            }
        }

        false
    }

    /// Get request usage percentage.
    pub fn request_usage_percent(&self, limit: u32) -> f32 {
        self.requests_used as f32 / limit as f32 * 100.0
    }

    /// Get token usage percentage.
    pub fn token_usage_percent(&self, limit: u32) -> f32 {
        self.tokens_used as f32 / limit as f32 * 100.0
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub enum RateLimitDecision {
    /// Proceed with the request.
    Allow,
    /// Wait before proceeding.
    Wait(Duration),
    /// Request should be rejected.
    Reject(String),
}

/// Headers from rate limit response.
#[derive(Debug, Clone, Default)]
pub struct RateLimitHeaders {
    /// Requests remaining.
    pub requests_remaining: Option<u32>,
    /// Tokens remaining.
    pub tokens_remaining: Option<u64>,
    /// Reset time.
    pub reset_at: Option<std::time::SystemTime>,
    /// Retry after (for 429 responses).
    pub retry_after: Option<Duration>,
}

impl RateLimitHeaders {
    /// Parse from HTTP response headers.
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let mut parsed = Self::default();

        // Claude headers
        if let Some(val) = headers.get("x-ratelimit-remaining-requests") {
            parsed.requests_remaining = val.to_str().ok().and_then(|v| v.parse().ok());
        }
        if let Some(val) = headers.get("x-ratelimit-remaining-tokens") {
            parsed.tokens_remaining = val.to_str().ok().and_then(|v| v.parse().ok());
        }

        // OpenAI headers
        if let Some(val) = headers.get("x-ratelimit-remaining-requests") {
            parsed.requests_remaining = val.to_str().ok().and_then(|v| v.parse().ok());
        }
        if let Some(val) = headers.get("x-ratelimit-remaining-tokens") {
            parsed.tokens_remaining = val.to_str().ok().and_then(|v| v.parse().ok());
        }

        // Retry-After header
        if let Some(val) = headers.get("retry-after") {
            if let Ok(secs) = val.to_str().unwrap_or("").parse::<u64>() {
                parsed.retry_after = Some(Duration::from_secs(secs));
            }
        }

        parsed
    }
}
```

### 2. Rate Limiter (src/rate_limit/limiter.rs)

```rust
//! Rate limiter implementation.

use super::types::{RateLimitConfig, RateLimitDecision, RateLimitHeaders, RateLimitUsage};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

/// Rate limiter for a backend.
pub struct RateLimiter {
    config: RateLimitConfig,
    /// Request count in current window.
    request_count: AtomicU32,
    /// Token count in current window.
    token_count: AtomicU64,
    /// Window start time.
    window_start: Mutex<Instant>,
    /// Concurrent request semaphore.
    concurrent_sem: Arc<Semaphore>,
    /// Backoff until this time.
    backoff_until: Mutex<Option<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            concurrent_sem: Arc::new(Semaphore::new(config.max_concurrent as usize)),
            config,
            request_count: AtomicU32::new(0),
            token_count: AtomicU64::new(0),
            window_start: Mutex::new(Instant::now()),
            backoff_until: Mutex::new(None),
        }
    }

    /// Check if a request should be allowed.
    pub async fn check(&self, estimated_tokens: u32) -> RateLimitDecision {
        // Check backoff
        if let Some(until) = *self.backoff_until.lock().await {
            if Instant::now() < until {
                let wait = until - Instant::now();
                return RateLimitDecision::Wait(wait);
            }
        }

        // Reset window if needed
        self.maybe_reset_window().await;

        // Check request limit
        let requests = self.request_count.load(Ordering::Relaxed);
        if requests >= self.config.requests_per_minute + self.config.burst_allowance {
            let wait = self.time_until_reset().await;
            return RateLimitDecision::Wait(wait);
        }

        // Check token limit
        let tokens = self.token_count.load(Ordering::Relaxed);
        if tokens + estimated_tokens as u64 > self.config.tokens_per_minute as u64 {
            let wait = self.time_until_reset().await;
            return RateLimitDecision::Wait(wait);
        }

        RateLimitDecision::Allow
    }

    /// Acquire permission for a request.
    pub async fn acquire(&self, estimated_tokens: u32) -> Result<RateLimitPermit, RateLimitDecision> {
        // Check limits
        match self.check(estimated_tokens).await {
            RateLimitDecision::Allow => {}
            decision => return Err(decision),
        }

        // Acquire concurrent permit
        let permit = self
            .concurrent_sem
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| RateLimitDecision::Reject("Semaphore closed".to_string()))?;

        // Increment request count
        self.request_count.fetch_add(1, Ordering::Relaxed);

        Ok(RateLimitPermit {
            limiter: self,
            _permit: permit,
            tokens_used: 0,
        })
    }

    /// Record token usage after a request.
    pub fn record_tokens(&self, tokens: u64) {
        self.token_count.fetch_add(tokens, Ordering::Relaxed);
    }

    /// Set backoff after rate limit error.
    pub async fn set_backoff(&self, duration: Duration) {
        *self.backoff_until.lock().await = Some(Instant::now() + duration);
        warn!(duration_secs = duration.as_secs(), "Rate limit backoff set");
    }

    /// Update from response headers.
    pub async fn update_from_headers(&self, headers: &RateLimitHeaders) {
        if let Some(retry_after) = headers.retry_after {
            self.set_backoff(retry_after).await;
        }
    }

    /// Get current usage.
    pub async fn usage(&self) -> RateLimitUsage {
        RateLimitUsage {
            requests_used: self.request_count.load(Ordering::Relaxed),
            tokens_used: self.token_count.load(Ordering::Relaxed),
            concurrent: (self.config.max_concurrent as usize - self.concurrent_sem.available_permits()) as u32,
            requests_remaining: Some(
                self.config
                    .requests_per_minute
                    .saturating_sub(self.request_count.load(Ordering::Relaxed)),
            ),
            tokens_remaining: Some(
                (self.config.tokens_per_minute as u64)
                    .saturating_sub(self.token_count.load(Ordering::Relaxed)),
            ),
            reset_in: Some(self.time_until_reset().await),
        }
    }

    /// Reset window if a minute has passed.
    async fn maybe_reset_window(&self) {
        let mut window_start = self.window_start.lock().await;
        if window_start.elapsed() >= Duration::from_secs(60) {
            *window_start = Instant::now();
            self.request_count.store(0, Ordering::Relaxed);
            self.token_count.store(0, Ordering::Relaxed);
            debug!("Rate limit window reset");
        }
    }

    /// Time until window resets.
    async fn time_until_reset(&self) -> Duration {
        let window_start = self.window_start.lock().await;
        let elapsed = window_start.elapsed();
        if elapsed >= Duration::from_secs(60) {
            Duration::ZERO
        } else {
            Duration::from_secs(60) - elapsed
        }
    }
}

/// Permit for a rate-limited request.
pub struct RateLimitPermit<'a> {
    limiter: &'a RateLimiter,
    _permit: tokio::sync::OwnedSemaphorePermit,
    tokens_used: u64,
}

impl<'a> RateLimitPermit<'a> {
    /// Record tokens used by this request.
    pub fn record_tokens(&mut self, tokens: u64) {
        self.tokens_used = tokens;
        self.limiter.record_tokens(tokens);
    }
}
```

### 3. Adaptive Rate Limiter (src/rate_limit/adaptive.rs)

```rust
//! Adaptive rate limiting based on response patterns.

use super::limiter::RateLimiter;
use super::types::{RateLimitConfig, RateLimitHeaders};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tracing::{debug, info};

/// Adaptive rate limiter that adjusts based on responses.
pub struct AdaptiveRateLimiter {
    inner: RateLimiter,
    /// Success count.
    successes: AtomicU32,
    /// Rate limit hit count.
    rate_limits: AtomicU32,
    /// Current multiplier (1.0 = normal).
    multiplier: std::sync::atomic::AtomicU64,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            inner: RateLimiter::new(config),
            successes: AtomicU32::new(0),
            rate_limits: AtomicU32::new(0),
            multiplier: std::sync::atomic::AtomicU64::new(f64::to_bits(1.0)),
        }
    }

    /// Get the effective rate limit.
    pub fn effective_rpm(&self) -> u32 {
        let multiplier = f64::from_bits(self.multiplier.load(Ordering::Relaxed));
        (self.inner.config.requests_per_minute as f64 * multiplier) as u32
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        self.successes.fetch_add(1, Ordering::Relaxed);

        // Increase multiplier on sustained success
        let successes = self.successes.load(Ordering::Relaxed);
        if successes >= 100 && successes % 100 == 0 {
            self.increase_limit();
        }
    }

    /// Record a rate limit hit.
    pub async fn record_rate_limit(&self, retry_after: Option<Duration>) {
        self.rate_limits.fetch_add(1, Ordering::Relaxed);
        self.decrease_limit();

        if let Some(duration) = retry_after {
            self.inner.set_backoff(duration).await;
        }
    }

    /// Increase the effective limit.
    fn increase_limit(&self) {
        let current = f64::from_bits(self.multiplier.load(Ordering::Relaxed));
        let new = (current * 1.1).min(1.5); // Max 150% of base
        self.multiplier.store(f64::to_bits(new), Ordering::Relaxed);
        debug!(multiplier = new, "Increased rate limit multiplier");
    }

    /// Decrease the effective limit.
    fn decrease_limit(&self) {
        let current = f64::from_bits(self.multiplier.load(Ordering::Relaxed));
        let new = (current * 0.7).max(0.3); // Min 30% of base
        self.multiplier.store(f64::to_bits(new), Ordering::Relaxed);
        info!(multiplier = new, "Decreased rate limit multiplier");
    }

    /// Get statistics.
    pub fn stats(&self) -> AdaptiveStats {
        AdaptiveStats {
            successes: self.successes.load(Ordering::Relaxed),
            rate_limits: self.rate_limits.load(Ordering::Relaxed),
            multiplier: f64::from_bits(self.multiplier.load(Ordering::Relaxed)),
        }
    }

    /// Get the inner rate limiter.
    pub fn inner(&self) -> &RateLimiter {
        &self.inner
    }
}

/// Adaptive rate limiter statistics.
#[derive(Debug, Clone)]
pub struct AdaptiveStats {
    pub successes: u32,
    pub rate_limits: u32,
    pub multiplier: f64,
}
```

### 4. Module Exports (src/rate_limit/mod.rs)

```rust
//! Rate limiting for backends.

mod adaptive;
mod limiter;
mod types;

pub use adaptive::{AdaptiveRateLimiter, AdaptiveStats};
pub use limiter::{RateLimiter, RateLimitPermit};
pub use types::{
    RateLimitConfig, RateLimitDecision, RateLimitHeaders, RateLimitUsage,
};
```

---

## Testing Requirements

1. Request counting works correctly
2. Token counting tracks usage
3. Window resets after one minute
4. Backoff is applied correctly
5. Adaptive limits adjust appropriately

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [073-backend-tokens.md](073-backend-tokens.md)
