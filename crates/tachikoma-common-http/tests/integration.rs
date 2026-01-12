use tachikoma_common_http::{HttpClient, HttpConfig, HttpError};
use std::time::Duration;

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