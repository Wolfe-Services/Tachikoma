# Spec 316: CORS Configuration

## Phase
15 - Server/API Layer

## Spec ID
316

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 312: Server Configuration

## Estimated Context
~8%

---

## Objective

Implement comprehensive CORS (Cross-Origin Resource Sharing) configuration for the Tachikoma API, supporting development flexibility while maintaining security in production environments.

---

## Acceptance Criteria

- [ ] CORS is configurable via environment and config files
- [ ] Development mode allows all origins
- [ ] Production mode restricts to specific origins
- [ ] Preflight requests are handled efficiently
- [ ] Credentials can be enabled/disabled per route
- [ ] Custom headers are properly exposed
- [ ] CORS errors provide helpful debugging information

---

## Implementation Details

### CORS Configuration

```rust
// src/server/cors/config.rs
use axum::http::{HeaderName, HeaderValue, Method};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// CORS configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CorsConfig {
    /// Whether CORS is enabled
    pub enabled: bool,

    /// Allowed origins (empty = all origins in dev, none in prod)
    pub allowed_origins: Vec<String>,

    /// Allowed HTTP methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Headers to expose to the browser
    pub exposed_headers: Vec<String>,

    /// Allow credentials (cookies, authorization headers)
    pub allow_credentials: bool,

    /// Max age for preflight cache (seconds)
    pub max_age_secs: u64,

    /// Allow private network access (for local development)
    pub allow_private_network: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec![], // Empty = permissive in dev
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-API-Key".to_string(),
                "X-Request-ID".to_string(),
            ],
            exposed_headers: vec![
                "X-Request-ID".to_string(),
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "X-RateLimit-Reset".to_string(),
            ],
            allow_credentials: true,
            max_age_secs: 86400, // 24 hours
            allow_private_network: false,
        }
    }
}

impl CorsConfig {
    /// Create permissive CORS config for development
    pub fn permissive() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allow_credentials: false, // Can't use * with credentials
            allow_private_network: true,
            ..Default::default()
        }
    }

    /// Create restrictive CORS config for production
    pub fn restrictive(origins: Vec<String>) -> Self {
        Self {
            enabled: true,
            allowed_origins: origins,
            allow_credentials: true,
            allow_private_network: false,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CorsConfigError> {
        // Can't use credentials with wildcard origin
        if self.allow_credentials && self.allowed_origins.contains(&"*".to_string()) {
            return Err(CorsConfigError::CredentialsWithWildcard);
        }

        // Validate origin patterns
        for origin in &self.allowed_origins {
            if origin != "*" && !origin.starts_with("http://") && !origin.starts_with("https://") {
                return Err(CorsConfigError::InvalidOrigin(origin.clone()));
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CorsConfigError {
    #[error("Cannot allow credentials with wildcard origin")]
    CredentialsWithWildcard,

    #[error("Invalid origin format: {0}")]
    InvalidOrigin(String),
}
```

### CORS Layer Builder

```rust
// src/server/cors/layer.rs
use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};
use std::time::Duration;

use super::config::CorsConfig;

/// Build a CorsLayer from configuration
pub fn build_cors_layer(config: &CorsConfig) -> CorsLayer {
    if !config.enabled {
        return CorsLayer::new();
    }

    let mut layer = CorsLayer::new();

    // Configure origins
    if config.allowed_origins.is_empty() || config.allowed_origins.contains(&"*".to_string()) {
        layer = layer.allow_origin(Any);
    } else {
        let origins: Vec<HeaderValue> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        layer = layer.allow_origin(origins);
    }

    // Configure methods
    let methods: Vec<Method> = config
        .allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    layer = layer.allow_methods(methods);

    // Configure headers
    if config.allowed_headers.contains(&"*".to_string()) {
        layer = layer.allow_headers(Any);
    } else {
        let headers: Vec<HeaderName> = config
            .allowed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        layer = layer.allow_headers(headers);
    }

    // Configure exposed headers
    let exposed: Vec<HeaderName> = config
        .exposed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    layer = layer.expose_headers(exposed);

    // Configure credentials
    if config.allow_credentials {
        layer = layer.allow_credentials(true);
    }

    // Configure max age
    layer = layer.max_age(Duration::from_secs(config.max_age_secs));

    // Configure private network access
    if config.allow_private_network {
        layer = layer.allow_private_network(true);
    }

    layer
}
```

### Dynamic CORS Handling

```rust
// src/server/cors/dynamic.rs
use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Dynamic CORS handler for runtime-configurable origins
#[derive(Clone)]
pub struct DynamicCors {
    allowed_origins: Arc<RwLock<HashSet<String>>>,
    allow_credentials: bool,
}

impl DynamicCors {
    pub fn new(origins: Vec<String>, allow_credentials: bool) -> Self {
        Self {
            allowed_origins: Arc::new(RwLock::new(origins.into_iter().collect())),
            allow_credentials,
        }
    }

    /// Add a new allowed origin at runtime
    pub async fn add_origin(&self, origin: String) {
        self.allowed_origins.write().await.insert(origin);
    }

    /// Remove an allowed origin at runtime
    pub async fn remove_origin(&self, origin: &str) {
        self.allowed_origins.write().await.remove(origin);
    }

    /// Check if an origin is allowed
    pub async fn is_origin_allowed(&self, origin: &str) -> bool {
        let origins = self.allowed_origins.read().await;
        origins.contains(origin) || origins.contains("*")
    }
}

/// Middleware for dynamic CORS handling
pub async fn dynamic_cors_middleware(
    State(cors): State<DynamicCors>,
    request: Request,
    next: Next,
) -> Response {
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let method = request.method().clone();

    // Handle preflight
    if method == Method::OPTIONS {
        return handle_preflight(&cors, origin.as_deref()).await;
    }

    // Process request
    let mut response = next.run(request).await;

    // Add CORS headers to response
    if let Some(ref origin) = origin {
        if cors.is_origin_allowed(origin).await {
            add_cors_headers(response.headers_mut(), origin, cors.allow_credentials);
        }
    }

    response
}

async fn handle_preflight(cors: &DynamicCors, origin: Option<&str>) -> Response {
    let Some(origin) = origin else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if !cors.is_origin_allowed(origin).await {
        return StatusCode::FORBIDDEN.into_response();
    }

    let mut response = StatusCode::NO_CONTENT.into_response();
    add_cors_headers(response.headers_mut(), origin, cors.allow_credentials);

    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, OPTIONS"),
    );

    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type, Authorization, X-API-Key, X-Request-ID"),
    );

    response.headers_mut().insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("86400"),
    );

    response
}

fn add_cors_headers(
    headers: &mut axum::http::HeaderMap,
    origin: &str,
    allow_credentials: bool,
) {
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*")),
    );

    if allow_credentials {
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            HeaderValue::from_static("true"),
        );
    }

    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("X-Request-ID, X-RateLimit-Limit, X-RateLimit-Remaining"),
    );
}
```

### Route-Specific CORS

```rust
// src/server/cors/route.rs
use axum::{
    routing::MethodRouter,
    Router,
};
use tower_http::cors::CorsLayer;

use crate::server::state::AppState;
use super::config::CorsConfig;

/// Extension trait for adding route-specific CORS
pub trait CorsRouteExt {
    /// Apply CORS to this route with specific configuration
    fn with_cors(self, config: &CorsConfig) -> Self;

    /// Apply permissive CORS (for development routes)
    fn with_permissive_cors(self) -> Self;

    /// Disable CORS for this route
    fn without_cors(self) -> Self;
}

impl CorsRouteExt for Router<AppState> {
    fn with_cors(self, config: &CorsConfig) -> Self {
        self.layer(super::layer::build_cors_layer(config))
    }

    fn with_permissive_cors(self) -> Self {
        self.layer(super::layer::build_cors_layer(&CorsConfig::permissive()))
    }

    fn without_cors(self) -> Self {
        // No layer applied
        self
    }
}

/// Builder for route-specific CORS configuration
pub struct RouteCorsBuilder {
    config: CorsConfig,
}

impl RouteCorsBuilder {
    pub fn new() -> Self {
        Self {
            config: CorsConfig::default(),
        }
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.config.allowed_origins.push(origin.into());
        self
    }

    pub fn allow_method(mut self, method: impl Into<String>) -> Self {
        self.config.allowed_methods.push(method.into());
        self
    }

    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.config.allow_credentials = allow;
        self
    }

    pub fn build(self) -> CorsLayer {
        super::layer::build_cors_layer(&self.config)
    }
}
```

### CORS Error Responses

```rust
// src/server/cors/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// CORS-specific error response
#[derive(Debug, Serialize)]
pub struct CorsErrorResponse {
    pub error: CorsErrorInfo,
}

#[derive(Debug, Serialize)]
pub struct CorsErrorInfo {
    pub code: &'static str,
    pub message: String,
    pub allowed_origins: Option<Vec<String>>,
}

pub enum CorsError {
    OriginNotAllowed {
        origin: String,
        allowed: Vec<String>,
    },
    InvalidOrigin {
        origin: String,
    },
    CredentialsRequired,
}

impl IntoResponse for CorsError {
    fn into_response(self) -> Response {
        let (status, info) = match self {
            CorsError::OriginNotAllowed { origin, allowed } => (
                StatusCode::FORBIDDEN,
                CorsErrorInfo {
                    code: "CORS_ORIGIN_NOT_ALLOWED",
                    message: format!("Origin '{}' is not allowed", origin),
                    allowed_origins: Some(allowed),
                },
            ),
            CorsError::InvalidOrigin { origin } => (
                StatusCode::BAD_REQUEST,
                CorsErrorInfo {
                    code: "CORS_INVALID_ORIGIN",
                    message: format!("Invalid origin format: '{}'", origin),
                    allowed_origins: None,
                },
            ),
            CorsError::CredentialsRequired => (
                StatusCode::UNAUTHORIZED,
                CorsErrorInfo {
                    code: "CORS_CREDENTIALS_REQUIRED",
                    message: "Credentials are required for this request".to_string(),
                    allowed_origins: None,
                },
            ),
        };

        (status, Json(CorsErrorResponse { error: info })).into_response()
    }
}
```

### Integration with Routes

```rust
// src/server/routes/mod.rs (CORS integration example)
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::server::state::AppState;
use crate::server::cors::{build_cors_layer, CorsConfig};

pub fn api_routes(cors_config: &CorsConfig) -> Router<AppState> {
    let cors_layer = build_cors_layer(cors_config);

    Router::new()
        .nest("/api/v1", v1_routes())
        .layer(cors_layer)
}

/// Routes that need different CORS settings
pub fn public_routes() -> Router<AppState> {
    // More permissive CORS for public endpoints
    let cors = CorsConfig {
        allowed_origins: vec!["*".to_string()],
        allow_credentials: false,
        ..Default::default()
    };

    Router::new()
        .nest("/public", public_handlers())
        .layer(build_cors_layer(&cors))
}

/// Internal routes with restrictive CORS
pub fn internal_routes() -> Router<AppState> {
    let cors = CorsConfig {
        allowed_origins: vec!["https://admin.tachikoma.io".to_string()],
        allow_credentials: true,
        ..Default::default()
    };

    Router::new()
        .nest("/internal", internal_handlers())
        .layer(build_cors_layer(&cors))
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
    use axum::http::{header, Request, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn test_cors_config_validation() {
        // Valid config
        let config = CorsConfig::restrictive(vec!["https://example.com".to_string()]);
        assert!(config.validate().is_ok());

        // Invalid: credentials with wildcard
        let invalid = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allow_credentials: true,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[tokio::test]
    async fn test_preflight_request() {
        let config = CorsConfig::restrictive(vec!["https://example.com".to_string()]);
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(build_cors_layer(&config));

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/test")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert!(response.headers().contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(response.headers().contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
    }

    #[tokio::test]
    async fn test_cors_headers_added() {
        let config = CorsConfig::restrictive(vec!["https://example.com".to_string()]);
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(build_cors_layer(&config));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(header::ORIGIN, "https://example.com")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "https://example.com"
        );
    }

    #[tokio::test]
    async fn test_origin_not_allowed() {
        let config = CorsConfig::restrictive(vec!["https://allowed.com".to_string()]);
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(build_cors_layer(&config));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(header::ORIGIN, "https://notallowed.com")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        // Origin header not present means request was blocked
        assert!(!response.headers().contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    }

    #[tokio::test]
    async fn test_dynamic_cors() {
        let cors = DynamicCors::new(vec!["https://example.com".to_string()], true);

        assert!(cors.is_origin_allowed("https://example.com").await);
        assert!(!cors.is_origin_allowed("https://other.com").await);

        cors.add_origin("https://other.com".to_string()).await;
        assert!(cors.is_origin_allowed("https://other.com").await);

        cors.remove_origin("https://other.com").await;
        assert!(!cors.is_origin_allowed("https://other.com").await);
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 312**: Server Configuration
- **Spec 314**: Middleware Stack
