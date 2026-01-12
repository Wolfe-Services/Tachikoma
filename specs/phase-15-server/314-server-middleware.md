# Spec 314: Middleware Stack

## Phase
15 - Server/API Layer

## Spec ID
314

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 312: Server Configuration

## Estimated Context
~10%

---

## Objective

Implement a comprehensive middleware stack for the Tachikoma server, providing cross-cutting concerns like logging, authentication, request ID tracking, timeout handling, and request/response transformation.

---

## Acceptance Criteria

- [ ] Request ID is generated and propagated through all layers
- [ ] Request/response logging captures appropriate detail levels
- [ ] Authentication middleware validates API keys when configured
- [ ] Timeout middleware prevents hanging requests
- [ ] Compression middleware handles gzip/brotli encoding
- [ ] Security headers are applied to all responses
- [ ] Middleware execution order is well-defined and documented

---

## Implementation Details

### Middleware Stack Overview

```rust
// src/server/middleware/mod.rs
pub mod auth;
pub mod compression;
pub mod logging;
pub mod request_id;
pub mod security;
pub mod timeout;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    decompression::RequestDecompressionLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::server::state::AppState;
use crate::server::config::ServerConfig;

/// Build the complete middleware stack
pub fn middleware_stack(config: &ServerConfig) -> ServiceBuilder<...> {
    ServiceBuilder::new()
        // Outermost: Request ID tracking
        .layer(request_id::RequestIdLayer::new())
        // Security headers
        .layer(security::SecurityHeadersLayer::new())
        // Logging (uses request ID)
        .layer(logging::LoggingLayer::new(config.logging.clone()))
        // Timeout
        .layer(TimeoutLayer::new(config.server.request_timeout))
        // Request decompression
        .layer(RequestDecompressionLayer::new())
        // Response compression
        .layer(CompressionLayer::new())
        // Authentication (if configured)
        .option_layer(
            config.security.api_key.as_ref().map(|key| {
                auth::ApiKeyLayer::new(key.clone())
            })
        )
}
```

### Request ID Middleware

```rust
// src/server/middleware/request_id.rs
use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::Span;
use uuid::Uuid;

pub static REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Layer for adding request IDs
#[derive(Clone)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> tower::Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S, B> tower::Service<Request<B>> for RequestIdService<S>
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

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract or generate request ID
            let request_id = request
                .headers()
                .get(&REQUEST_ID_HEADER)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Store in request extensions
            request.extensions_mut().insert(RequestId(request_id.clone()));

            // Add to tracing span
            Span::current().record("request_id", &request_id);

            // Call inner service
            let mut response = inner.call(request).await?;

            // Add to response headers
            response.headers_mut().insert(
                REQUEST_ID_HEADER.clone(),
                HeaderValue::from_str(&request_id).unwrap(),
            );

            Ok(response)
        })
    }
}

/// Request ID extension
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Extractor for request ID
impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Axum extractor implementation
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<RequestId>()
            .cloned()
            .unwrap_or_else(|| RequestId(Uuid::new_v4().to_string())))
    }
}
```

### Authentication Middleware

```rust
// src/server/middleware/auth.rs
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;

use crate::server::config::SecretString;

/// API key authentication layer
#[derive(Clone)]
pub struct ApiKeyLayer {
    api_key: SecretString,
}

impl ApiKeyLayer {
    pub fn new(api_key: SecretString) -> Self {
        Self { api_key }
    }
}

impl<S> tower::Layer<S> for ApiKeyLayer {
    type Service = ApiKeyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ApiKeyService {
            inner,
            api_key: self.api_key.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ApiKeyService<S> {
    inner: S,
    api_key: SecretString,
}

impl<S, B> tower::Service<Request<B>> for ApiKeyService<S>
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
        let api_key = self.api_key.clone();

        Box::pin(async move {
            // Skip auth for health check endpoints
            if request.uri().path().starts_with("/health") {
                return inner.call(request).await;
            }

            // Extract API key from header
            let provided_key = request
                .headers()
                .get("x-api-key")
                .or_else(|| request.headers().get("authorization"))
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_start_matches("Bearer "));

            match provided_key {
                Some(key) if constant_time_compare(key, api_key.expose()) => {
                    inner.call(request).await
                }
                _ => {
                    Ok(AuthError::InvalidApiKey.into_response())
                }
            }
        })
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[derive(Debug)]
pub enum AuthError {
    MissingApiKey,
    InvalidApiKey,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingApiKey => (StatusCode::UNAUTHORIZED, "Missing API key"),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key"),
        };

        (status, message).into_response()
    }
}

/// Middleware function alternative
pub async fn require_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let api_key = headers
        .get("x-api-key")
        .or_else(|| headers.get("authorization"))
        .ok_or(AuthError::MissingApiKey)?;

    // Validate key against stored key from state
    // This is a simplified example; real implementation would check state

    Ok(next.run(request).await)
}
```

### Logging Middleware

```rust
// src/server/middleware/logging.rs
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::Response,
};
use std::time::{Duration, Instant};
use tracing::{info, warn, error, Level, Span};

use crate::server::config::LoggingConfig;
use crate::server::middleware::request_id::RequestId;

/// Enhanced logging layer
#[derive(Clone)]
pub struct LoggingLayer {
    config: LoggingConfig,
}

impl LoggingLayer {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
    config: LoggingConfig,
}

impl<S> tower::Service<Request<Body>> for LoggingService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let start = Instant::now();

            // Extract request details
            let method = request.method().clone();
            let uri = request.uri().clone();
            let version = request.version();
            let request_id = request
                .extensions()
                .get::<RequestId>()
                .map(|r| r.0.clone())
                .unwrap_or_default();

            // Log request
            info!(
                target: "http",
                request_id = %request_id,
                method = %method,
                uri = %uri,
                version = ?version,
                "Request started"
            );

            // Call inner service
            let response = inner.call(request).await?;

            let duration = start.elapsed();
            let status = response.status();

            // Log response with appropriate level
            let log_level = status_to_log_level(status);

            match log_level {
                Level::ERROR => error!(
                    target: "http",
                    request_id = %request_id,
                    method = %method,
                    uri = %uri,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Request completed with error"
                ),
                Level::WARN => warn!(
                    target: "http",
                    request_id = %request_id,
                    method = %method,
                    uri = %uri,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Request completed with warning"
                ),
                _ => info!(
                    target: "http",
                    request_id = %request_id,
                    method = %method,
                    uri = %uri,
                    status = %status.as_u16(),
                    duration_ms = %duration.as_millis(),
                    "Request completed"
                ),
            }

            Ok(response)
        })
    }
}

fn status_to_log_level(status: StatusCode) -> Level {
    if status.is_server_error() {
        Level::ERROR
    } else if status.is_client_error() {
        Level::WARN
    } else {
        Level::INFO
    }
}
```

### Security Headers Middleware

```rust
// src/server/middleware/security.rs
use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Security headers configuration
#[derive(Clone)]
pub struct SecurityHeadersLayer {
    /// Content Security Policy
    csp: Option<String>,
    /// Whether to enable HSTS
    hsts_enabled: bool,
    /// HSTS max age in seconds
    hsts_max_age: u64,
}

impl SecurityHeadersLayer {
    pub fn new() -> Self {
        Self {
            csp: Some("default-src 'self'".to_string()),
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
        }
    }

    pub fn with_csp(mut self, csp: impl Into<String>) -> Self {
        self.csp = Some(csp.into());
        self
    }

    pub fn without_csp(mut self) -> Self {
        self.csp = None;
        self
    }

    pub fn with_hsts(mut self, enabled: bool, max_age: u64) -> Self {
        self.hsts_enabled = enabled;
        self.hsts_max_age = max_age;
        self
    }
}

impl Default for SecurityHeadersLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> tower::Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService {
            inner,
            csp: self.csp.clone(),
            hsts_enabled: self.hsts_enabled,
            hsts_max_age: self.hsts_max_age,
        }
    }
}

#[derive(Clone)]
pub struct SecurityHeadersService<S> {
    inner: S,
    csp: Option<String>,
    hsts_enabled: bool,
    hsts_max_age: u64,
}

impl<S, B> tower::Service<Request<B>> for SecurityHeadersService<S>
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
        let csp = self.csp.clone();
        let hsts_enabled = self.hsts_enabled;
        let hsts_max_age = self.hsts_max_age;

        Box::pin(async move {
            let mut response = inner.call(request).await?;
            let headers = response.headers_mut();

            // X-Content-Type-Options
            headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );

            // X-Frame-Options
            headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );

            // X-XSS-Protection
            headers.insert(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );

            // Content Security Policy
            if let Some(ref csp) = csp {
                headers.insert(
                    HeaderName::from_static("content-security-policy"),
                    HeaderValue::from_str(csp).unwrap(),
                );
            }

            // HSTS
            if hsts_enabled {
                headers.insert(
                    HeaderName::from_static("strict-transport-security"),
                    HeaderValue::from_str(&format!(
                        "max-age={}; includeSubDomains",
                        hsts_max_age
                    )).unwrap(),
                );
            }

            // Referrer Policy
            headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            // Permissions Policy
            headers.insert(
                HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );

            Ok(response)
        })
    }
}

/// Middleware function for adding security headers
pub async fn add_security_headers(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    response
}
```

### Middleware Composition

```rust
// src/server/middleware/compose.rs
use axum::{
    middleware::{self, from_fn},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::server::config::ServerConfig;
use crate::server::state::AppState;

use super::{
    auth::require_auth,
    logging::LoggingLayer,
    request_id::RequestIdLayer,
    security::SecurityHeadersLayer,
};

/// Apply all middleware to a router
pub fn apply_middleware(router: Router<AppState>, config: &ServerConfig) -> Router<AppState> {
    let service_builder = ServiceBuilder::new()
        // Outermost layer - runs first on request, last on response
        .layer(RequestIdLayer::new())
        .layer(SecurityHeadersLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(config.server.request_timeout))
        .layer(CompressionLayer::new());

    router.layer(service_builder)
}

/// Apply authentication middleware to protected routes
pub fn protected_routes(router: Router<AppState>) -> Router<AppState> {
    router.layer(middleware::from_fn(require_auth))
}

/// Create route-specific middleware stack
pub fn route_middleware(router: Router<AppState>, config: &ServerConfig) -> Router<AppState> {
    router
        .layer(middleware::from_fn(|request, next| async move {
            // Custom per-route middleware
            next.run(request).await
        }))
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_request_id_generated() {
        let app = Router::new()
            .route("/test", get(|req_id: RequestId| async move {
                req_id.0
            }))
            .layer(RequestIdLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert!(response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn test_request_id_preserved() {
        let app = Router::new()
            .route("/test", get(|req_id: RequestId| async move {
                req_id.0
            }))
            .layer(RequestIdLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-request-id", "my-custom-id")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get("x-request-id").unwrap(),
            "my-custom-id"
        );
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(SecurityHeadersLayer::new());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert!(response.headers().contains_key("x-content-type-options"));
        assert!(response.headers().contains_key("x-frame-options"));
    }

    #[tokio::test]
    async fn test_auth_middleware_rejects_invalid_key() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(ApiKeyLayer::new(SecretString::new("valid-key")));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-api-key", "invalid-key")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_accepts_valid_key() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(ApiKeyLayer::new(SecretString::new("valid-key")));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("x-api-key", "valid-key")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 316**: CORS Configuration
- **Spec 327**: Rate Limiting
- **Spec 334**: Distributed Tracing
