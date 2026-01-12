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
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        message: String,
        value: i32,
    }

    #[test]
    fn test_request_builder_url() {
        let builder = RequestBuilder::new().base_url("https://api.example.com");
        assert_eq!(builder.url("/v1/test"), "https://api.example.com/v1/test");
    }

    #[test]
    fn test_request_builder_url_trailing_slash() {
        let builder = RequestBuilder::new().base_url("https://api.example.com/");
        assert_eq!(builder.url("/v1/test"), "https://api.example.com/v1/test");
    }

    #[test]
    fn test_request_builder_no_base_url() {
        let builder = RequestBuilder::new();
        assert_eq!(builder.url("/v1/test"), "/v1/test");
    }

    #[test]
    fn test_bearer_auth() {
        let builder = RequestBuilder::new().bearer_auth("token123");
        let auth = builder.headers().get(AUTHORIZATION).unwrap();
        assert_eq!(auth.to_str().unwrap(), "Bearer token123");
    }

    #[test]
    fn test_api_key() {
        let builder = RequestBuilder::new().api_key("key123");
        let api_key = builder.headers().get(headers::X_API_KEY).unwrap();
        assert_eq!(api_key.to_str().unwrap(), "key123");
    }

    #[test]
    fn test_json_content() {
        let builder = RequestBuilder::new().json_content();
        let content_type = builder.headers().get(CONTENT_TYPE).unwrap();
        assert_eq!(content_type.to_str().unwrap(), headers::CONTENT_TYPE_JSON);
    }

    #[test]
    fn test_custom_header() {
        let builder = RequestBuilder::new().header("Custom-Header", "custom-value");
        let custom = builder.headers().get("Custom-Header").unwrap();
        assert_eq!(custom.to_str().unwrap(), "custom-value");
    }

    #[test]
    fn test_json_body_serialization() {
        let test_data = TestData {
            message: "hello".to_string(),
            value: 42,
        };
        let json_body = JsonBody(test_data);
        
        let serialized = json_body.to_string().unwrap();
        assert!(serialized.contains("\"message\":\"hello\""));
        assert!(serialized.contains("\"value\":42"));
    }

    #[test]
    fn test_json_body_bytes() {
        let test_data = TestData {
            message: "test".to_string(),
            value: 123,
        };
        let json_body = JsonBody(test_data);
        
        let bytes = json_body.to_bytes().unwrap();
        assert!(!bytes.is_empty());
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["message"], "test");
        assert_eq!(parsed["value"], 123);
    }

    #[test]
    fn test_header_constants() {
        assert_eq!(headers::CONTENT_TYPE_JSON, "application/json");
        assert_eq!(headers::CONTENT_TYPE_SSE, "text/event-stream");
        assert_eq!(headers::X_API_KEY, "x-api-key");
        assert_eq!(headers::ANTHROPIC_VERSION, "anthropic-version");
    }

    #[test]
    fn test_builder_default() {
        let builder1 = RequestBuilder::default();
        let builder2 = RequestBuilder::new();
        
        // Both should have empty headers initially
        assert_eq!(builder1.headers().len(), builder2.headers().len());
    }

    #[test]
    fn test_builder_chaining() {
        let builder = RequestBuilder::new()
            .base_url("https://api.example.com")
            .bearer_auth("token")
            .json_content()
            .header("Custom", "value");
            
        assert_eq!(builder.url("/test"), "https://api.example.com/test");
        assert!(builder.headers().contains_key(AUTHORIZATION));
        assert!(builder.headers().contains_key(CONTENT_TYPE));
        assert!(builder.headers().contains_key("Custom"));
    }
}