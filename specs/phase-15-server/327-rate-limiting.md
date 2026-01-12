# Spec 327: Rate Limiting

## Phase
15 - Server/API Layer

## Spec ID
327

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 314: Middleware

## Estimated Context
~9%

---

## Objective

Implement comprehensive rate limiting for the Tachikoma API, protecting against abuse while ensuring fair resource allocation across different endpoint categories and client tiers.

---

## Acceptance Criteria

- [ ] Request rate limiting per IP address
- [ ] Configurable limits per endpoint category
- [ ] Rate limit headers in responses
- [ ] Graceful handling of limit exceeded
- [ ] Different tiers for authenticated vs unauthenticated
- [ ] Expensive operations have separate limits
- [ ] Rate limit bypass for internal services

---

## Implementation Details

### Rate Limit Configuration

```rust
// src/server/ratelimit/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Default rate limit (requests per minute)
    pub default_rpm: u32,

    /// Default burst size
    pub default_burst: u32,

    /// Rate limits by category
    pub categories: HashMap<String, CategoryLimit>,

    /// IP whitelist (bypass rate limiting)
    pub whitelist: Vec<String>,

    /// Custom limits by API key
    pub api_key_limits: HashMap<String, u32>,

    /// Storage backend (memory or redis)
    pub storage: RateLimitStorage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CategoryLimit {
    /// Requests per minute
    pub rpm: u32,

    /// Burst size (max concurrent)
    pub burst: u32,

    /// Window duration in seconds
    #[serde(default = "default_window")]
    pub window_secs: u64,
}

fn default_window() -> u64 {
    60
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStorage {
    #[default]
    Memory,
    Redis,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut categories = HashMap::new();

        // Standard API endpoints
        categories.insert("standard".to_string(), CategoryLimit {
            rpm: 100,
            burst: 20,
            window_secs: 60,
        });

        // LLM execution endpoints (expensive)
        categories.insert("execution".to_string(), CategoryLimit {
            rpm: 10,
            burst: 3,
            window_secs: 60,
        });

        // Bulk operations
        categories.insert("bulk".to_string(), CategoryLimit {
            rpm: 5,
            burst: 2,
            window_secs: 60,
        });

        // Health checks (unlimited)
        categories.insert("health".to_string(), CategoryLimit {
            rpm: 1000,
            burst: 100,
            window_secs: 60,
        });

        Self {
            enabled: true,
            default_rpm: 100,
            default_burst: 20,
            categories,
            whitelist: vec!["127.0.0.1".to_string()],
            api_key_limits: HashMap::new(),
            storage: RateLimitStorage::Memory,
        }
    }
}

/// Rate limit category for routes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateLimitCategory {
    Standard,
    Execution,
    Bulk,
    Health,
    Unlimited,
}

impl RateLimitCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RateLimitCategory::Standard => "standard",
            RateLimitCategory::Execution => "execution",
            RateLimitCategory::Bulk => "bulk",
            RateLimitCategory::Health => "health",
            RateLimitCategory::Unlimited => "unlimited",
        }
    }
}
```

### Rate Limiter Implementation

```rust
// src/server/ratelimit/limiter.rs
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::config::{RateLimitConfig, RateLimitCategory};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: u32, rpm: u32) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Instant::now(),
            capacity: capacity as f64,
            refill_rate: rpm as f64 / 60.0, // Convert RPM to per second
        }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_update = now;
    }

    fn remaining(&self) -> u32 {
        self.tokens as u32
    }

    fn reset_time(&self) -> Duration {
        let tokens_needed = self.capacity - self.tokens;
        let seconds_needed = tokens_needed / self.refill_rate;
        Duration::from_secs_f64(seconds_needed)
    }
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: i64,
    pub retry_after: Option<Duration>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request is allowed
    pub async fn check(
        &self,
        key: &str,
        category: RateLimitCategory,
    ) -> RateLimitResult {
        if !self.config.enabled || category == RateLimitCategory::Unlimited {
            return RateLimitResult {
                allowed: true,
                limit: u32::MAX,
                remaining: u32::MAX,
                reset_at: 0,
                retry_after: None,
            };
        }

        let category_config = self.config.categories
            .get(category.as_str())
            .cloned()
            .unwrap_or_else(|| super::config::CategoryLimit {
                rpm: self.config.default_rpm,
                burst: self.config.default_burst,
                window_secs: 60,
            });

        let bucket_key = format!("{}:{}", category.as_str(), key);

        let mut buckets = self.buckets.write().await;
        let bucket = buckets
            .entry(bucket_key)
            .or_insert_with(|| TokenBucket::new(category_config.burst, category_config.rpm));

        let allowed = bucket.try_consume(1);
        let remaining = bucket.remaining();
        let reset_time = bucket.reset_time();

        RateLimitResult {
            allowed,
            limit: category_config.rpm,
            remaining,
            reset_at: (chrono::Utc::now() + chrono::Duration::from_std(reset_time).unwrap()).timestamp(),
            retry_after: if allowed { None } else { Some(reset_time) },
        }
    }

    /// Check if an IP is whitelisted
    pub fn is_whitelisted(&self, ip: &IpAddr) -> bool {
        self.config.whitelist.contains(&ip.to_string())
    }

    /// Get custom limit for API key
    pub fn get_api_key_limit(&self, api_key: &str) -> Option<u32> {
        self.config.api_key_limits.get(api_key).copied()
    }

    /// Clean up expired buckets
    pub async fn cleanup(&self, max_age: Duration) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < max_age
        });
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;
                self.cleanup(Duration::from_secs(3600)).await; // 1 hour max age
            }
        })
    }
}
```

### Rate Limit Middleware

```rust
// src/server/middleware/ratelimit.rs
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::net::SocketAddr;

use crate::server::ratelimit::{RateLimiter, RateLimitCategory, RateLimitResult};
use crate::server::error::ErrorResponse;

/// Rate limit layer
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter>,
    category: RateLimitCategory,
}

impl RateLimitLayer {
    pub fn new(limiter: Arc<RateLimiter>, category: RateLimitCategory) -> Self {
        Self { limiter, category }
    }
}

impl<S> tower::Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
            category: self.category,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<RateLimiter>,
    category: RateLimitCategory,
}

impl<S, B> tower::Service<Request<B>> for RateLimitService<S>
where
    S: tower::Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();
        let limiter = self.limiter.clone();
        let category = self.category;

        Box::pin(async move {
            // Extract client identifier
            let client_key = extract_client_key(&request);

            // Check if whitelisted
            if let Some(ip) = client_key.parse().ok() {
                if limiter.is_whitelisted(&ip) {
                    return inner.call(request).await;
                }
            }

            // Check rate limit
            let result = limiter.check(&client_key, category).await;

            if result.allowed {
                let mut response = inner.call(request).await?;
                add_rate_limit_headers(response.headers_mut(), &result);
                Ok(response)
            } else {
                Ok(rate_limit_exceeded_response(&result))
            }
        })
    }
}

/// Middleware function for rate limiting
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let category = get_route_category(&request);
    let client_key = addr.ip().to_string();

    // Check whitelist
    if limiter.is_whitelisted(&addr.ip()) {
        return next.run(request).await;
    }

    let result = limiter.check(&client_key, category).await;

    if result.allowed {
        let mut response = next.run(request).await;
        add_rate_limit_headers(response.headers_mut(), &result);
        response
    } else {
        rate_limit_exceeded_response(&result)
    }
}

fn extract_client_key<B>(request: &Request<B>) -> String {
    // Try X-Forwarded-For first (for proxied requests)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(first_ip) = value.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }

    // Fall back to connection info (set by extension)
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_route_category<B>(request: &Request<B>) -> RateLimitCategory {
    let path = request.uri().path();

    if path.starts_with("/health") {
        RateLimitCategory::Health
    } else if path.contains("/execute") || path.contains("/stream") {
        RateLimitCategory::Execution
    } else if path.contains("/bulk") || path.contains("/export") || path.contains("/import") {
        RateLimitCategory::Bulk
    } else {
        RateLimitCategory::Standard
    }
}

fn add_rate_limit_headers(headers: &mut HeaderMap, result: &RateLimitResult) {
    headers.insert(
        HeaderName::from_static("x-ratelimit-limit"),
        HeaderValue::from_str(&result.limit.to_string()).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-remaining"),
        HeaderValue::from_str(&result.remaining.to_string()).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("x-ratelimit-reset"),
        HeaderValue::from_str(&result.reset_at.to_string()).unwrap(),
    );
}

fn rate_limit_exceeded_response(result: &RateLimitResult) -> Response {
    let mut headers = HeaderMap::new();
    add_rate_limit_headers(&mut headers, result);

    if let Some(retry_after) = result.retry_after {
        headers.insert(
            HeaderName::from_static("retry-after"),
            HeaderValue::from_str(&retry_after.as_secs().to_string()).unwrap(),
        );
    }

    let body = ErrorResponse::new(
        "RATE_LIMITED",
        format!(
            "Rate limit exceeded. Please retry after {} seconds.",
            result.retry_after.map(|d| d.as_secs()).unwrap_or(60)
        ),
        429,
    );

    let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
    *response.headers_mut() = headers;
    response
}
```

### Route-Specific Rate Limiting

```rust
// src/server/routes/ratelimit.rs
use axum::Router;

use crate::server::state::AppState;
use crate::server::ratelimit::{RateLimiter, RateLimitCategory};
use crate::server::middleware::ratelimit::RateLimitLayer;

/// Extension trait for adding rate limiting to routes
pub trait RateLimitExt {
    fn with_rate_limit(self, limiter: Arc<RateLimiter>, category: RateLimitCategory) -> Self;
}

impl RateLimitExt for Router<AppState> {
    fn with_rate_limit(self, limiter: Arc<RateLimiter>, category: RateLimitCategory) -> Self {
        self.layer(RateLimitLayer::new(limiter, category))
    }
}

/// Apply rate limiting to API routes
pub fn apply_rate_limits(router: Router<AppState>, limiter: Arc<RateLimiter>) -> Router<AppState> {
    router
        // Standard rate limit for most routes
        .layer(RateLimitLayer::new(limiter.clone(), RateLimitCategory::Standard))
}

/// Create execution routes with specific rate limits
pub fn execution_routes(limiter: Arc<RateLimiter>) -> Router<AppState> {
    Router::new()
        .route("/specs/:id/execute", post(execute_spec))
        .route("/specs/:id/stream", get(stream_execution))
        .layer(RateLimitLayer::new(limiter, RateLimitCategory::Execution))
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
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            default_rpm: 10,
            default_burst: 5,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // First 5 requests should be allowed (burst)
        for _ in 0..5 {
            let result = limiter.check("test-key", RateLimitCategory::Standard).await;
            assert!(result.allowed);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            default_rpm: 10,
            default_burst: 2,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // First 2 requests allowed
        limiter.check("test-key", RateLimitCategory::Standard).await;
        limiter.check("test-key", RateLimitCategory::Standard).await;

        // Third request should be blocked
        let result = limiter.check("test-key", RateLimitCategory::Standard).await;
        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_whitelist_bypass() {
        let config = RateLimitConfig {
            whitelist: vec!["192.168.1.1".to_string()],
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        assert!(limiter.is_whitelisted(&"192.168.1.1".parse().unwrap()));
        assert!(!limiter.is_whitelisted(&"192.168.1.2".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_category_specific_limits() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new(config);

        // Health category should have higher limits
        let health_result = limiter.check("key", RateLimitCategory::Health).await;
        assert!(health_result.limit > 100);

        // Execution category should have lower limits
        let exec_result = limiter.check("key", RateLimitCategory::Execution).await;
        assert!(exec_result.limit <= 10);
    }
}
```

---

## Related Specs

- **Spec 314**: Middleware Stack
- **Spec 315**: Error Handling
- **Spec 333**: Prometheus Metrics
