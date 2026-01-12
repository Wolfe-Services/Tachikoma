# 020 - HTTP Client Foundation

**Phase:** 1 - Core Common Crates
**Spec ID:** 020
**Status:** Planned
**Dependencies:** 019-async-runtime
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a configured HTTP client wrapper with sensible defaults, connection pooling, and middleware support for API calls.

---

## Acceptance Criteria

- [ ] reqwest client with configured defaults
- [ ] Connection pooling
- [ ] Timeout configuration
- [ ] User-Agent header
- [ ] Request/response logging (debug)

---

## Implementation Details

### 1. HTTP Client (crates/tachikoma-common-http/src/client.rs)

```rust
//! HTTP client configuration.

use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// HTTP client configuration.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Request timeout.
    pub request_timeout: Duration,
    /// User agent string.
    pub user_agent: String,
    /// Maximum connections per host.
    pub pool_max_idle_per_host: usize,
    /// Enable gzip decompression.
    pub gzip: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            user_agent: format!("tachikoma/{}", env!("CARGO_PKG_VERSION")),
            pool_max_idle_per_host: 10,
            gzip: true,
        }
    }
}

/// Build a configured HTTP client.
pub fn build_client(config: HttpConfig) -> Result<Client, HttpError> {
    let mut builder = ClientBuilder::new()
        .connect_timeout(config.connect_timeout)
        .timeout(config.request_timeout)
        .user_agent(&config.user_agent)
        .pool_max_idle_per_host(config.pool_max_idle_per_host);

    if config.gzip {
        builder = builder.gzip(true);
    }

    builder.build().map_err(HttpError::ClientBuild)
}

/// HTTP errors.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("failed to build HTTP client: {0}")]
    ClientBuild(#[source] reqwest::Error),

    #[error("request failed: {0}")]
    Request(#[source] reqwest::Error),

    #[error("request timed out")]
    Timeout,

    #[error("rate limited (retry after {retry_after:?})")]
    RateLimited { retry_after: Option<Duration> },

    #[error("server error: {status}")]
    ServerError { status: u16, body: String },

    #[error("client error: {status}")]
    ClientError { status: u16, body: String },
}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            HttpError::Timeout
        } else {
            HttpError::Request(e)
        }
    }
}

/// Shared HTTP client for the application.
pub struct HttpClient {
    inner: Client,
}

impl HttpClient {
    /// Create a new HTTP client with default config.
    pub fn new() -> Result<Self, HttpError> {
        Self::with_config(HttpConfig::default())
    }

    /// Create a new HTTP client with custom config.
    pub fn with_config(config: HttpConfig) -> Result<Self, HttpError> {
        let inner = build_client(config)?;
        Ok(Self { inner })
    }

    /// Get the inner reqwest client.
    pub fn inner(&self) -> &Client {
        &self.inner
    }

    /// Make a GET request.
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, HttpError> {
        self.inner.get(url).send().await.map_err(HttpError::from)
    }

    /// Make a POST request with JSON body.
    pub async fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response, HttpError> {
        self.inner
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(HttpError::from)
    }

    /// Check response status and convert errors.
    pub async fn check_response(response: reqwest::Response) -> Result<reqwest::Response, HttpError> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);

            return Err(HttpError::RateLimited { retry_after });
        }

        let body = response.text().await.unwrap_or_default();

        if status.is_server_error() {
            Err(HttpError::ServerError {
                status: status.as_u16(),
                body,
            })
        } else {
            Err(HttpError::ClientError {
                status: status.as_u16(),
                body,
            })
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("failed to create HTTP client")
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-http"
version.workspace = true
edition.workspace = true

[dependencies]
reqwest = { workspace = true, features = ["json", "gzip"] }
serde.workspace = true
thiserror.workspace = true
```

---

## Testing Requirements

1. Client builds with default config
2. Timeout errors are detected
3. Rate limit responses parsed correctly
4. Error responses converted properly

---

## Related Specs

- Depends on: [019-async-runtime.md](019-async-runtime.md)
- Next: [021-http-request-types.md](021-http-request-types.md)
