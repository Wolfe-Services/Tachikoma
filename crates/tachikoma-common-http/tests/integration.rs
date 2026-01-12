use tachikoma_common_http::{
    HttpClient, HttpConfig, HttpError, RequestBuilder, JsonBody, headers,
    ResponseError,
};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestPayload {
    message: String,
    value: i32,
}

#[tokio::test]
async fn test_http_client_functionality() {
    // Test client creation with defaults
    let client = HttpClient::new().expect("Failed to create client");
    
    // Test that we can access the inner reqwest client
    let _inner = client.inner();
    
    // Test client with custom config
    let config = HttpConfig {
        connect_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(15),
        user_agent: "test-tachikoma/1.0".to_string(),
        pool_max_idle_per_host: 5,
        gzip: true,
    };
    
    let custom_client = HttpClient::with_config(config).expect("Failed to create custom client");
    let _custom_inner = custom_client.inner();
}

#[tokio::test]
async fn test_request_builder_integration() {
    // Create a request builder with all features
    let builder = RequestBuilder::new()
        .base_url("https://httpbin.org")
        .bearer_auth("test-token")
        .json_content()
        .header("Custom-Header", "test-value");
    
    // Test URL building
    let url = builder.url("/json");
    assert_eq!(url, "https://httpbin.org/json");
    
    // Test headers
    let headers = builder.headers();
    assert!(headers.contains_key(reqwest::header::AUTHORIZATION));
    assert!(headers.contains_key(reqwest::header::CONTENT_TYPE));
    assert!(headers.contains_key("Custom-Header"));
    
    // Test header values
    let auth = headers.get(reqwest::header::AUTHORIZATION).unwrap();
    assert_eq!(auth.to_str().unwrap(), "Bearer test-token");
    
    let content_type = headers.get(reqwest::header::CONTENT_TYPE).unwrap();
    assert_eq!(content_type.to_str().unwrap(), headers::CONTENT_TYPE_JSON);
    
    let custom = headers.get("Custom-Header").unwrap();
    assert_eq!(custom.to_str().unwrap(), "test-value");
}

#[test]
fn test_json_body_functionality() {
    let payload = TestPayload {
        message: "Hello, World!".to_string(),
        value: 42,
    };
    
    let json_body = JsonBody(payload);
    
    // Test serialization methods
    let string_result = json_body.to_string().unwrap();
    assert!(string_result.contains("Hello, World!"));
    assert!(string_result.contains("42"));
    
    let bytes_result = json_body.to_bytes().unwrap();
    assert!(!bytes_result.is_empty());
    
    let pretty_result = json_body.to_string_pretty().unwrap();
    assert!(pretty_result.contains("Hello, World!"));
    assert!(pretty_result.len() > string_result.len()); // Pretty formatting adds whitespace
}

#[test]
fn test_header_constants() {
    // Verify all header constants are properly defined
    assert_eq!(headers::CONTENT_TYPE_JSON, "application/json");
    assert_eq!(headers::CONTENT_TYPE_SSE, "text/event-stream");
    assert_eq!(headers::X_API_KEY, "x-api-key");
    assert_eq!(headers::ANTHROPIC_VERSION, "anthropic-version");
}

#[test]
fn test_response_error_types() {
    // Create a mock JSON error
    let json_error = serde_json::from_str::<TestPayload>("invalid json").unwrap_err();
    
    let parse_error = ResponseError::Parse {
        status: 400,
        body: "invalid json".to_string(),
        source: json_error,
    };
    
    // Test error display
    let error_string = format!("{}", parse_error);
    assert!(error_string.contains("failed to parse JSON"));
    assert!(error_string.contains("status 400"));
    
    // Test error source chain
    assert!(parse_error.source().is_some());
}

#[test]
fn test_api_key_builder() {
    let builder = RequestBuilder::new()
        .api_key("test-api-key-123")
        .header(headers::ANTHROPIC_VERSION, "2023-06-01");
    
    let headers = builder.headers();
    
    // Test API key header
    let api_key = headers.get(headers::X_API_KEY).unwrap();
    assert_eq!(api_key.to_str().unwrap(), "test-api-key-123");
    
    // Test Anthropic version header
    let version = headers.get(headers::ANTHROPIC_VERSION).unwrap();
    assert_eq!(version.to_str().unwrap(), "2023-06-01");
}

#[test]
fn test_default_config_values() {
    let config = HttpConfig::default();
    
    assert_eq!(config.connect_timeout, Duration::from_secs(10));
    assert_eq!(config.request_timeout, Duration::from_secs(30));
    assert!(config.user_agent.starts_with("tachikoma/"));
    assert_eq!(config.pool_max_idle_per_host, 10);
    assert!(config.gzip);
}

#[test]
fn test_error_display() {
    let server_error = HttpError::ServerError {
        status: 500,
        body: "Internal Server Error".to_string(),
    };
    
    let error_string = format!("{}", server_error);
    assert!(error_string.contains("500"));
    assert!(error_string.contains("server error"));
    
    let rate_limit_error = HttpError::RateLimited {
        retry_after: Some(Duration::from_secs(60)),
    };
    
    let rate_limit_string = format!("{}", rate_limit_error);
    assert!(rate_limit_string.contains("rate limited"));
    assert!(rate_limit_string.contains("60"));
    
    let client_error = HttpError::ClientError {
        status: 404,
        body: "Not Found".to_string(),
    };
    
    let client_error_string = format!("{}", client_error);
    assert!(client_error_string.contains("404"));
    assert!(client_error_string.contains("client error"));
}