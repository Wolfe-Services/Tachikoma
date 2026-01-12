# 021 - HTTP Request/Response Types

**Phase:** 1 - Core Common Crates
**Spec ID:** 021
**Status:** Planned
**Dependencies:** 020-http-client-foundation
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define common request and response types for HTTP APIs including headers, JSON body helpers, and streaming support.

---

## Acceptance Criteria

- [ ] Request builder with headers
- [ ] JSON response parsing
- [ ] Streaming response support
- [ ] Common header constants
- [ ] Content-Type handling

---

## Implementation Details

### 1. Request Types (crates/tachikoma-common-http/src/request.rs)

```rust
//! HTTP request types and builders.

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

/// Common HTTP headers.
pub mod headers {
    pub const CONTENT_TYPE_JSON: &str = "application/json";
    pub const CONTENT_TYPE_SSE: &str = "text/event-stream";
    pub const X_API_KEY: &str = "x-api-key";
    pub const ANTHROPIC_VERSION: &str = "anthropic-version";
}

/// A request builder with common patterns.
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    headers: HeaderMap,
    base_url: Option<String>,
}

impl RequestBuilder {
    /// Create a new request builder.
    pub fn new() -> Self {
        Self {
            headers: HeaderMap::new(),
            base_url: None,
        }
    }

    /// Set the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Add a header.
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            HeaderName::try_from(name.as_ref()),
            HeaderValue::try_from(value.as_ref()),
        ) {
            self.headers.insert(name, value);
        }
        self
    }

    /// Add bearer token authorization.
    pub fn bearer_auth(mut self, token: impl AsRef<str>) -> Self {
        if let Ok(value) = HeaderValue::try_from(format!("Bearer {}", token.as_ref())) {
            self.headers.insert(AUTHORIZATION, value);
        }
        self
    }

    /// Add API key header.
    pub fn api_key(self, key: impl AsRef<str>) -> Self {
        self.header(headers::X_API_KEY, key)
    }

    /// Set content type to JSON.
    pub fn json_content(mut self) -> Self {
        if let Ok(value) = HeaderValue::try_from(headers::CONTENT_TYPE_JSON) {
            self.headers.insert(CONTENT_TYPE, value);
        }
        self
    }

    /// Get the built headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Build the URL.
    pub fn url(&self, path: &str) -> String {
        match &self.base_url {
            Some(base) => format!("{}{}", base.trim_end_matches('/'), path),
            None => path.to_string(),
        }
    }
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON request body wrapper.
#[derive(Debug, Clone)]
pub struct JsonBody<T>(pub T);

impl<T: Serialize> JsonBody<T> {
    /// Serialize to JSON bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self.0)
    }

    /// Serialize to JSON string.
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.0)
    }

    /// Serialize to pretty JSON string.
    pub fn to_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder_url() {
        let builder = RequestBuilder::new().base_url("https://api.example.com");
        assert_eq!(builder.url("/v1/test"), "https://api.example.com/v1/test");
    }

    #[test]
    fn test_bearer_auth() {
        let builder = RequestBuilder::new().bearer_auth("token123");
        let auth = builder.headers().get(AUTHORIZATION).unwrap();
        assert_eq!(auth.to_str().unwrap(), "Bearer token123");
    }
}
```

### 2. Response Types (crates/tachikoma-common-http/src/response.rs)

```rust
//! HTTP response types.

use serde::de::DeserializeOwned;

/// Parse a JSON response.
pub async fn parse_json<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, ResponseError> {
    let status = response.status();
    let bytes = response.bytes().await.map_err(ResponseError::Read)?;

    serde_json::from_slice(&bytes).map_err(|e| ResponseError::Parse {
        status: status.as_u16(),
        body: String::from_utf8_lossy(&bytes).to_string(),
        source: e,
    })
}

/// Response parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("failed to read response body: {0}")]
    Read(#[source] reqwest::Error),

    #[error("failed to parse JSON (status {status}): {source}")]
    Parse {
        status: u16,
        body: String,
        #[source]
        source: serde_json::Error,
    },
}
```

---

## Testing Requirements

1. Request builder constructs correct URLs
2. Headers are properly formatted
3. JSON responses parse correctly
4. Parse errors include response body

---

## Related Specs

- Depends on: [020-http-client-foundation.md](020-http-client-foundation.md)
- Next: [022-http-retry-logic.md](022-http-retry-logic.md)
