# 326 - CORS Configuration

**Phase:** 15 - Server
**Spec ID:** 326
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement CORS (Cross-Origin Resource Sharing) configuration with flexible origin policies, credential support, and preflight handling.

---

## Acceptance Criteria

- [x] Configurable allowed origins
- [x] Wildcard and regex origin matching
- [x] Credentials support
- [x] Preflight request handling
- [x] Exposed headers configuration
- [x] Max age caching
- [x] Per-route CORS overrides

---

## Implementation Details

### 1. CORS Configuration (crates/tachikoma-server/src/middleware/cors/config.rs)

```rust
//! CORS configuration types.

use std::collections::HashSet;
use std::time::Duration;

/// CORS configuration.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins.
    pub allowed_origins: AllowedOrigins,
    /// Allowed methods.
    pub allowed_methods: HashSet<String>,
    /// Allowed headers.
    pub allowed_headers: AllowedHeaders,
    /// Exposed headers (accessible to client).
    pub exposed_headers: HashSet<String>,
    /// Allow credentials (cookies, auth headers).
    pub allow_credentials: bool,
    /// Max age for preflight cache.
    pub max_age: Option<Duration>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: AllowedOrigins::default(),
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: AllowedHeaders::default(),
            exposed_headers: HashSet::new(),
            allow_credentials: false,
            max_age: Some(Duration::from_secs(86400)), // 24 hours
        }
    }
}

impl CorsConfig {
    /// Create permissive CORS config (for development).
    pub fn permissive() -> Self {
        Self {
            allowed_origins: AllowedOrigins::Any,
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: AllowedHeaders::Any,
            exposed_headers: HashSet::new(),
            allow_credentials: true,
            max_age: Some(Duration::from_secs(86400)),
        }
    }

    /// Create strict CORS config (for production).
    pub fn strict(origins: Vec<String>) -> Self {
        Self {
            allowed_origins: AllowedOrigins::List(origins.into_iter().collect()),
            allowed_methods: ["GET", "POST", "PUT", "DELETE", "PATCH"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_headers: AllowedHeaders::List(
                ["Content-Type", "Authorization", "X-Request-ID"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            exposed_headers: ["X-Request-ID", "X-RateLimit-Remaining"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allow_credentials: true,
            max_age: Some(Duration::from_secs(3600)), // 1 hour
        }
    }

    /// Check if origin is allowed.
    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.is_allowed(origin)
    }

    /// Check if method is allowed.
    pub fn is_method_allowed(&self, method: &str) -> bool {
        self.allowed_methods.contains(method)
    }
}

/// Allowed origins configuration.
#[derive(Debug, Clone)]
pub enum AllowedOrigins {
    /// Allow any origin.
    Any,
    /// Allow specific origins.
    List(HashSet<String>),
    /// Allow origins matching regex patterns.
    Regex(Vec<regex::Regex>),
}

impl Default for AllowedOrigins {
    fn default() -> Self {
        Self::List(HashSet::new())
    }
}

impl AllowedOrigins {
    /// Check if origin is allowed.
    pub fn is_allowed(&self, origin: &str) -> bool {
        match self {
            Self::Any => true,
            Self::List(origins) => origins.contains(origin),
            Self::Regex(patterns) => patterns.iter().any(|p| p.is_match(origin)),
        }
    }

    /// Create from environment variable (comma-separated).
    pub fn from_env(var: &str) -> Self {
        match std::env::var(var) {
            Ok(value) if value == "*" => Self::Any,
            Ok(value) => Self::List(
                value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            ),
            Err(_) => Self::default(),
        }
    }
}

/// Allowed headers configuration.
#[derive(Debug, Clone)]
pub enum AllowedHeaders {
    /// Allow any headers.
    Any,
    /// Allow specific headers.
    List(HashSet<String>),
}

impl Default for AllowedHeaders {
    fn default() -> Self {
        Self::List(
            ["Content-Type", "Authorization", "Accept", "Origin", "X-Requested-With"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    }
}

impl AllowedHeaders {
    /// Check if header is allowed.
    pub fn is_allowed(&self, header: &str) -> bool {
        match self {
            Self::Any => true,
            Self::List(headers) => {
                let header_lower = header.to_lowercase();
                headers.iter().any(|h| h.to_lowercase() == header_lower)
            }
        }
    }

    /// Get allowed headers as string.
    pub fn to_header_value(&self) -> String {
        match self {
            Self::Any => "*".to_string(),
            Self::List(headers) => headers.iter().cloned().collect::<Vec<_>>().join(", "),
        }
    }
}
```

### 2. CORS Layer (crates/tachikoma-server/src/middleware/cors/layer.rs)

```rust
//! CORS middleware layer.

use super::config::CorsConfig;
use axum::{
    body::Body,
    http::{header, Method, Request, Response, StatusCode},
};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// CORS middleware layer.
#[derive(Clone)]
pub struct CorsLayer {
    config: CorsConfig,
}

impl CorsLayer {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    pub fn permissive() -> Self {
        Self::new(CorsConfig::permissive())
    }
}

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
    config: CorsConfig,
}

impl<S> Service<Request<Body>> for CorsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get origin from request
            let origin = req
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            // Handle preflight request
            if req.method() == Method::OPTIONS {
                return Ok(handle_preflight(&config, origin.as_deref()));
            }

            // Call inner service
            let mut response = inner.call(req).await?;

            // Add CORS headers to response
            add_cors_headers(&mut response, &config, origin.as_deref());

            Ok(response)
        })
    }
}

fn handle_preflight(config: &CorsConfig, origin: Option<&str>) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NO_CONTENT;

    if let Some(origin) = origin {
        if config.is_origin_allowed(origin) {
            add_cors_headers(&mut response, config, Some(origin));

            // Add preflight-specific headers
            let headers = response.headers_mut();

            // Allowed methods
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                config.allowed_methods
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
                    .parse()
                    .unwrap(),
            );

            // Allowed headers
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                config.allowed_headers.to_header_value().parse().unwrap(),
            );

            // Max age
            if let Some(max_age) = config.max_age {
                headers.insert(
                    header::ACCESS_CONTROL_MAX_AGE,
                    max_age.as_secs().to_string().parse().unwrap(),
                );
            }
        }
    }

    response
}

fn add_cors_headers(response: &mut Response<Body>, config: &CorsConfig, origin: Option<&str>) {
    let headers = response.headers_mut();

    // Set origin header
    if let Some(origin) = origin {
        if config.is_origin_allowed(origin) {
            match &config.allowed_origins {
                super::config::AllowedOrigins::Any => {
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                }
                _ => {
                    headers.insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        origin.parse().unwrap(),
                    );
                    // Vary header for caching
                    headers.insert(header::VARY, "Origin".parse().unwrap());
                }
            }
        }
    }

    // Credentials
    if config.allow_credentials {
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "true".parse().unwrap(),
        );
    }

    // Exposed headers
    if !config.exposed_headers.is_empty() {
        headers.insert(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            config.exposed_headers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
                .parse()
                .unwrap(),
        );
    }
}
```

### 3. CORS Builder (crates/tachikoma-server/src/middleware/cors/builder.rs)

```rust
//! CORS configuration builder.

use super::config::{AllowedHeaders, AllowedOrigins, CorsConfig};
use std::collections::HashSet;
use std::time::Duration;

/// Builder for CORS configuration.
pub struct CorsBuilder {
    config: CorsConfig,
}

impl CorsBuilder {
    pub fn new() -> Self {
        Self {
            config: CorsConfig::default(),
        }
    }

    /// Allow any origin.
    pub fn allow_any_origin(mut self) -> Self {
        self.config.allowed_origins = AllowedOrigins::Any;
        self
    }

    /// Allow specific origins.
    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_origins = AllowedOrigins::List(
            origins.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Allow origins matching regex patterns.
    pub fn allow_origin_regex<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let regexes: Vec<regex::Regex> = patterns
            .into_iter()
            .filter_map(|p| regex::Regex::new(p.as_ref()).ok())
            .collect();
        self.config.allowed_origins = AllowedOrigins::Regex(regexes);
        self
    }

    /// Set allowed methods.
    pub fn allow_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_methods = methods.into_iter().map(Into::into).collect();
        self
    }

    /// Allow any headers.
    pub fn allow_any_header(mut self) -> Self {
        self.config.allowed_headers = AllowedHeaders::Any;
        self
    }

    /// Allow specific headers.
    pub fn allow_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_headers = AllowedHeaders::List(
            headers.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Set exposed headers.
    pub fn expose_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.exposed_headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Allow credentials.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.config.allow_credentials = allow;
        self
    }

    /// Set max age for preflight cache.
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.config.max_age = Some(duration);
        self
    }

    /// Build the CORS configuration.
    pub fn build(self) -> CorsConfig {
        self.config
    }
}

impl Default for CorsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = CorsBuilder::new()
            .allow_origins(["https://example.com", "https://app.example.com"])
            .allow_methods(["GET", "POST"])
            .allow_credentials(true)
            .max_age(Duration::from_secs(3600))
            .build();

        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("https://other.com"));
        assert!(config.allow_credentials);
    }
}
```

---

## Testing Requirements

1. Preflight requests handled correctly
2. Origin validation works
3. Credentials header set properly
4. Exposed headers included
5. Max-age caching works
6. Regex origin matching works
7. Vary header set for caching

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [327-health-endpoints.md](327-health-endpoints.md)
- Used by: All API endpoints
