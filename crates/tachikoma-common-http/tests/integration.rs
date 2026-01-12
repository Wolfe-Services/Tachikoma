use tachikoma_common_http::{
    HttpClient, HttpConfig, HttpError, RequestBuilder, JsonBody, headers,
    ResponseError, RetryPolicy, RetryableError,
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

#[tokio::test]
async fn test_retry_with_http_errors() {
    // Test retry behavior with different HTTP error types
    let _policy = RetryPolicy {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };

    // Test retryable errors
    let timeout_error = HttpError::Timeout;
    assert!(timeout_error.is_retryable());

    let rate_limit_error = HttpError::RateLimited { retry_after: Some(Duration::from_secs(1)) };
    assert!(rate_limit_error.is_retryable());
    assert_eq!(rate_limit_error.retry_after(), Some(Duration::from_secs(1)));

    let server_error = HttpError::ServerError { status: 500, body: "Internal Server Error".to_string() };
    assert!(server_error.is_retryable());

    let server_error_502 = HttpError::ServerError { status: 502, body: "Bad Gateway".to_string() };
    assert!(server_error_502.is_retryable());

    // Test non-retryable errors
    let client_error = HttpError::ClientError { status: 400, body: "Bad Request".to_string() };
    assert!(!client_error.is_retryable());

    let not_found_error = HttpError::ClientError { status: 404, body: "Not Found".to_string() };
    assert!(!not_found_error.is_retryable());
}

#[tokio::test]
async fn test_retry_policy_configurations() {
    // Test default policy
    let default_policy = RetryPolicy::default();
    assert_eq!(default_policy.max_attempts, 3);
    assert_eq!(default_policy.initial_delay, Duration::from_millis(500));
    assert_eq!(default_policy.max_delay, Duration::from_secs(30));
    assert_eq!(default_policy.multiplier, 2.0);
    assert!(default_policy.jitter);

    // Test no retry policy
    let no_retry = RetryPolicy::no_retry();
    assert_eq!(no_retry.max_attempts, 1);

    // Test aggressive policy
    let aggressive = RetryPolicy::aggressive();
    assert_eq!(aggressive.max_attempts, 5);
    assert_eq!(aggressive.initial_delay, Duration::from_millis(100));
    assert_eq!(aggressive.max_delay, Duration::from_secs(60));
}

#[tokio::test]
async fn test_exponential_backoff_behavior() {
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay: Duration::from_millis(10),
        multiplier: 2.0,
        jitter: false,
        ..Default::default()
    };

    // Test exponential progression
    assert_eq!(policy.delay_for_attempt(0), Duration::ZERO);
    assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(10));
    assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(20));
    assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(40));
    assert_eq!(policy.delay_for_attempt(4), Duration::from_millis(80));
}

#[tokio::test]
async fn test_jitter_prevention_of_thundering_herd() {
    let policy = RetryPolicy {
        initial_delay: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: true,
        ..Default::default()
    };

    // Collect multiple delay calculations for the same attempt
    let mut delays = Vec::new();
    for _ in 0..10 {
        delays.push(policy.delay_for_attempt(1));
    }

    // Verify all delays are within expected range (100ms to 125ms with 25% jitter)
    for delay in &delays {
        assert!(*delay >= Duration::from_millis(100));
        assert!(*delay <= Duration::from_millis(125));
    }

    // Verify there's actually variation (very unlikely all 10 would be exactly the same)
    let first_delay = delays[0];
    let has_variation = delays.iter().any(|&d| d != first_delay);
    assert!(has_variation, "Jitter should introduce variation in delays");
}

#[tokio::test] 
async fn test_rate_limit_header_parsing() {
    // Test rate limit error with retry-after
    let rate_limit_with_delay = HttpError::RateLimited {
        retry_after: Some(Duration::from_secs(30))
    };
    assert_eq!(rate_limit_with_delay.retry_after(), Some(Duration::from_secs(30)));

    // Test rate limit error without retry-after
    let rate_limit_no_delay = HttpError::RateLimited {
        retry_after: None
    };
    assert_eq!(rate_limit_no_delay.retry_after(), None);

    // Test other errors don't have retry_after
    let timeout_error = HttpError::Timeout;
    assert_eq!(timeout_error.retry_after(), None);
}