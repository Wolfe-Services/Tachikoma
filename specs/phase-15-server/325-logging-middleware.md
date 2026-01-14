# 325 - Logging Middleware

**Phase:** 15 - Server
**Spec ID:** 325
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement request/response logging middleware with structured logging, request correlation, and configurable log levels.

---

## Acceptance Criteria

- [x] Structured request logging
- [x] Response status logging
- [x] Request duration measurement
- [x] Request ID correlation
- [x] Sensitive data redaction
- [x] Configurable log levels per path
- [x] JSON log format option

---

## Implementation Details

### 1. Logging Layer (crates/tachikoma-server/src/middleware/logging/layer.rs)

```rust
//! Request logging middleware.

use axum::{
    body::Body,
    http::{Request, Response},
};
use std::time::Instant;
use tower::{Layer, Service};
use tracing::{info, span, Level, Span};
use uuid::Uuid;

/// Request logging layer.
#[derive(Clone, Default)]
pub struct LoggingLayer {
    config: LoggingConfig,
}

#[derive(Clone, Default)]
pub struct LoggingConfig {
    /// Log request bodies (careful with size).
    pub log_bodies: bool,
    /// Log response bodies.
    pub log_response_bodies: bool,
    /// Paths to exclude from logging.
    pub exclude_paths: Vec<String>,
    /// Headers to redact.
    pub redact_headers: Vec<String>,
}

impl LoggingLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: LoggingConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
    config: LoggingConfig,
}

impl<S> Service<Request<Body>> for LoggingMiddleware<S>
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
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        // Check if path should be excluded
        let path = req.uri().path().to_string();
        if config.exclude_paths.iter().any(|p| path.starts_with(p)) {
            return Box::pin(async move { inner.call(req).await });
        }

        // Extract request info
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();

        // Get or generate request ID
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Get client IP
        let client_ip = req
            .headers()
            .get("x-forwarded-for")
            .or_else(|| req.headers().get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string());

        // Get user agent
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_default();

        let start = Instant::now();

        Box::pin(async move {
            // Create request span
            let span = span!(
                Level::INFO,
                "request",
                request_id = %request_id,
                method = %method,
                path = %uri.path(),
                client_ip = %client_ip,
            );

            let _enter = span.enter();

            // Log request
            info!(
                event = "request_started",
                method = %method,
                uri = %uri,
                version = ?version,
                user_agent = %user_agent,
            );

            // Call inner service
            let response = inner.call(req).await?;

            // Calculate duration
            let duration = start.elapsed();
            let status = response.status();

            // Log response
            info!(
                event = "request_completed",
                status = %status.as_u16(),
                duration_ms = duration.as_millis() as u64,
            );

            Ok(response)
        })
    }
}
```

### 2. Sensitive Data Redaction (crates/tachikoma-server/src/middleware/logging/redaction.rs)

```rust
//! Sensitive data redaction utilities.

use std::collections::HashSet;

/// Headers that should be redacted in logs.
pub const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
];

/// Fields that should be redacted in request bodies.
pub const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "password_confirm",
    "current_password",
    "new_password",
    "token",
    "secret",
    "api_key",
    "credit_card",
    "ssn",
];

/// Redact sensitive headers from a header map.
pub fn redact_headers(
    headers: &axum::http::HeaderMap,
    additional: &[String],
) -> Vec<(String, String)> {
    let sensitive: HashSet<&str> = SENSITIVE_HEADERS
        .iter()
        .copied()
        .chain(additional.iter().map(|s| s.as_str()))
        .collect();

    headers
        .iter()
        .map(|(name, value)| {
            let name_lower = name.as_str().to_lowercase();
            let value_str = if sensitive.contains(name_lower.as_str()) {
                "[REDACTED]".to_string()
            } else {
                value.to_str().unwrap_or("[non-utf8]").to_string()
            };
            (name.as_str().to_string(), value_str)
        })
        .collect()
}

/// Redact sensitive fields from a JSON value.
pub fn redact_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lower = key.to_lowercase();
                if SENSITIVE_FIELDS.iter().any(|f| key_lower.contains(f)) {
                    *val = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    redact_json(val);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr.iter_mut() {
                redact_json(val);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_redact_json() {
        let mut value = json!({
            "email": "user@example.com",
            "password": "secret123",
            "data": {
                "api_key": "key123"
            }
        });

        redact_json(&mut value);

        assert_eq!(value["email"], "user@example.com");
        assert_eq!(value["password"], "[REDACTED]");
        assert_eq!(value["data"]["api_key"], "[REDACTED]");
    }
}
```

### 3. Access Log Format (crates/tachikoma-server/src/middleware/logging/format.rs)

```rust
//! Log formatting utilities.

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Common Log Format entry.
#[derive(Debug, Serialize)]
pub struct AccessLogEntry {
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
    pub client_ip: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status: u16,
    pub duration_ms: u64,
    pub bytes_sent: u64,
    pub user_agent: String,
    pub user_id: Option<String>,
}

impl AccessLogEntry {
    /// Format as Common Log Format string.
    pub fn to_clf(&self) -> String {
        format!(
            "{} - {} [{}] \"{} {}{}\" {} {} \"{}\"",
            self.client_ip,
            self.user_id.as_deref().unwrap_or("-"),
            self.timestamp.format("%d/%b/%Y:%H:%M:%S %z"),
            self.method,
            self.path,
            self.query.as_ref().map(|q| format!("?{}", q)).unwrap_or_default(),
            self.status,
            self.bytes_sent,
            self.user_agent,
        )
    }

    /// Format as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
```

---

## Testing Requirements

1. Request info logged correctly
2. Response status logged
3. Duration measured accurately
4. Sensitive headers redacted
5. JSON body fields redacted
6. Excluded paths not logged
7. Request ID propagated

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [326-cors-config.md](326-cors-config.md)
- Used by: All requests
