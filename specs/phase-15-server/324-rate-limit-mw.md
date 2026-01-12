# 324 - Rate Limit Middleware

**Phase:** 15 - Server
**Spec ID:** 324
**Status:** Planned
**Dependencies:** 317-axum-router, 321-error-response
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement rate limiting middleware to protect the API from abuse, with configurable limits per endpoint, user, and IP address.

---

## Acceptance Criteria

- [ ] Token bucket algorithm implementation
- [ ] Per-user rate limiting
- [ ] Per-IP rate limiting
- [ ] Per-endpoint configuration
- [ ] Rate limit headers in responses
- [ ] Redis backend for distributed limiting
- [ ] Sliding window support
- [ ] Burst allowance configuration

---

## Implementation Details

### 1. Rate Limit Types (crates/tachikoma-server/src/middleware/rate_limit/types.rs)

```rust
//! Rate limiting types.

use std::time::Duration;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests allowed.
    pub max_requests: u32,
    /// Time window for the limit.
    pub window: Duration,
    /// Burst allowance (extra requests allowed in short bursts).
    pub burst: u32,
    /// Key extraction strategy.
    pub key_strategy: KeyStrategy,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            burst: 0,
            key_strategy: KeyStrategy::Ip,
        }
    }

    pub fn with_burst(mut self, burst: u32) -> Self {
        self.burst = burst;
        self
    }

    pub fn by_user(mut self) -> Self {
        self.key_strategy = KeyStrategy::User;
        self
    }

    pub fn by_ip(mut self) -> Self {
        self.key_strategy = KeyStrategy::Ip;
        self
    }

    pub fn by_api_key(mut self) -> Self {
        self.key_strategy = KeyStrategy::ApiKey;
        self
    }

    pub fn composite(mut self) -> Self {
        self.key_strategy = KeyStrategy::Composite;
        self
    }
}

/// Strategy for extracting rate limit key.
#[derive(Debug, Clone, Copy)]
pub enum KeyStrategy {
    /// Rate limit by IP address.
    Ip,
    /// Rate limit by authenticated user.
    User,
    /// Rate limit by API key.
    ApiKey,
    /// Composite key (IP + User if authenticated).
    Composite,
}

/// Rate limit state for a key.
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Current token count.
    pub tokens: u32,
    /// Last refill timestamp.
    pub last_refill: std::time::Instant,
    /// Maximum tokens (max_requests + burst).
    pub max_tokens: u32,
    /// Refill rate (tokens per second).
    pub refill_rate: f64,
}

impl RateLimitState {
    pub fn new(config: &RateLimitConfig) -> Self {
        let max_tokens = config.max_requests + config.burst;
        let refill_rate = config.max_requests as f64 / config.window.as_secs_f64();

        Self {
            tokens: max_tokens,
            last_refill: std::time::Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    /// Refill tokens based on elapsed time.
    pub fn refill(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = (elapsed.as_secs_f64() * self.refill_rate) as u32;

        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    /// Try to consume a token. Returns true if successful.
    pub fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get time until next token is available.
    pub fn retry_after(&self) -> Duration {
        if self.tokens > 0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1.0 / self.refill_rate)
        }
    }
}

/// Rate limit info for response headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
    pub retry_after: Option<u64>,
}
```

### 2. Rate Limit Store (crates/tachikoma-server/src/middleware/rate_limit/store.rs)

```rust
//! Rate limit storage backends.

use super::types::{RateLimitConfig, RateLimitState};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

/// Trait for rate limit storage.
#[async_trait]
pub trait RateLimitStore: Send + Sync {
    /// Check if request is allowed and consume a token.
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult;

    /// Get current state for a key.
    async fn get_state(&self, key: &str) -> Option<RateLimitState>;
}

/// Result of rate limit check.
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: std::time::Instant,
    pub retry_after: Option<std::time::Duration>,
}

/// In-memory rate limit store.
pub struct InMemoryStore {
    states: DashMap<String, RateLimitState>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitStore for InMemoryStore {
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult {
        let mut entry = self.states
            .entry(key.to_string())
            .or_insert_with(|| RateLimitState::new(config));

        let state = entry.value_mut();
        let allowed = state.try_consume();

        RateLimitResult {
            allowed,
            limit: state.max_tokens,
            remaining: state.tokens,
            reset_at: state.last_refill + config.window,
            retry_after: if allowed { None } else { Some(state.retry_after()) },
        }
    }

    async fn get_state(&self, key: &str) -> Option<RateLimitState> {
        self.states.get(key).map(|entry| entry.value().clone())
    }
}

/// Redis-backed rate limit store (for distributed systems).
#[cfg(feature = "redis")]
pub struct RedisStore {
    client: redis::Client,
    prefix: String,
}

#[cfg(feature = "redis")]
impl RedisStore {
    pub fn new(redis_url: &str, prefix: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            prefix: prefix.to_string(),
        })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl RateLimitStore for RedisStore {
    async fn check_and_consume(&self, key: &str, config: &RateLimitConfig) -> RateLimitResult {
        // Implement using Redis MULTI/EXEC or Lua script for atomicity
        todo!("Implement Redis rate limiting")
    }

    async fn get_state(&self, key: &str) -> Option<RateLimitState> {
        todo!("Implement Redis state retrieval")
    }
}
```

### 3. Rate Limit Layer (crates/tachikoma-server/src/middleware/rate_limit/layer.rs)

```rust
//! Rate limit middleware layer.

use super::{
    store::{InMemoryStore, RateLimitStore},
    types::{KeyStrategy, RateLimitConfig},
};
use crate::{error::ApiError, middleware::auth::types::AuthUser};
use axum::{
    body::Body,
    http::{header, Request, Response},
};
use std::{sync::Arc, time::Duration};
use tower::{Layer, Service};

/// Rate limit layer.
#[derive(Clone)]
pub struct RateLimitLayer {
    store: Arc<dyn RateLimitStore>,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            store: Arc::new(InMemoryStore::new()),
            config: RateLimitConfig {
                max_requests,
                window,
                burst: 0,
                key_strategy: KeyStrategy::Ip,
            },
        }
    }

    pub fn with_store(mut self, store: Arc<dyn RateLimitStore>) -> Self {
        self.store = store;
        self
    }

    pub fn with_config(mut self, config: RateLimitConfig) -> Self {
        self.config = config;
        self
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            store: self.store.clone(),
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    store: Arc<dyn RateLimitStore>,
    config: RateLimitConfig,
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let store = self.store.clone();
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract rate limit key
            let key = extract_key(&req, &config.key_strategy);

            // Check rate limit
            let result = store.check_and_consume(&key, &config).await;

            if !result.allowed {
                let retry_after = result.retry_after
                    .map(|d| d.as_secs())
                    .unwrap_or(1);

                return Err(ApiError::RateLimited { retry_after }.into());
            }

            // Call inner service
            let mut response = inner.call(req).await?;

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert(
                header::HeaderName::from_static("x-ratelimit-limit"),
                result.limit.to_string().parse().unwrap(),
            );
            headers.insert(
                header::HeaderName::from_static("x-ratelimit-remaining"),
                result.remaining.to_string().parse().unwrap(),
            );
            headers.insert(
                header::HeaderName::from_static("x-ratelimit-reset"),
                result.reset_at.elapsed().as_secs().to_string().parse().unwrap(),
            );

            Ok(response)
        })
    }
}

fn extract_key(req: &Request<Body>, strategy: &KeyStrategy) -> String {
    match strategy {
        KeyStrategy::Ip => {
            // Try X-Forwarded-For, then X-Real-IP, then connection IP
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
                .or_else(|| {
                    req.headers()
                        .get("x-real-ip")
                        .and_then(|v| v.to_str().ok())
                        .map(String::from)
                })
                .unwrap_or_else(|| "unknown".to_string())
        }
        KeyStrategy::User => {
            req.extensions()
                .get::<AuthUser>()
                .map(|u| format!("user:{}", u.id))
                .unwrap_or_else(|| "anonymous".to_string())
        }
        KeyStrategy::ApiKey => {
            req.headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(|k| format!("apikey:{}", k))
                .unwrap_or_else(|| "no-key".to_string())
        }
        KeyStrategy::Composite => {
            let ip = extract_key(req, &KeyStrategy::Ip);
            let user = req.extensions()
                .get::<AuthUser>()
                .map(|u| u.id.to_string());

            match user {
                Some(user_id) => format!("{}:{}", ip, user_id),
                None => ip,
            }
        }
    }
}
```

---

## Testing Requirements

1. Token bucket refills correctly
2. Burst allowance works
3. Rate limit headers included
4. 429 returned when exceeded
5. Different key strategies work
6. Retry-After header correct
7. Distributed store works (Redis)

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [325-logging-middleware.md](325-logging-middleware.md)
- Used by: All public endpoints
