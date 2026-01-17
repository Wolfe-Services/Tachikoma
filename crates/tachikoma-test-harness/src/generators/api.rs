//! API-specific generators for endpoints, requests, and responses.

use super::*;

/// Generate API endpoint paths
pub fn api_endpoint() -> String {
    let segments = vec![
        vec!["api", "v1"],
        vec!["users", "sessions", "specs", "missions", "tools"],
        words(1),
    ];

    segments
        .into_iter()
        .map(|options| {
            use rand::Rng;
            options[rand::thread_rng().gen_range(0..options.len())].to_string()
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Generate HTTP status codes
pub fn http_status() -> u16 {
    use rand::Rng;
    let statuses = [200, 201, 400, 401, 403, 404, 422, 500];
    statuses[rand::thread_rng().gen_range(0..statuses.len())]
}

/// Generate API keys with different provider prefixes
pub fn api_key_for_provider(provider: &str) -> String {
    let prefix = match provider {
        "claude" | "anthropic" => "sk-ant",
        "openai" => "sk",
        "github" => "ghp",
        "huggingface" => "hf",
        _ => "api",
    };
    format!("{}-{}", prefix, alphanumeric(48))
}

/// Generate JWT tokens
pub fn jwt_token() -> String {
    let header = base64::encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = base64::encode(&serde_json::json!({
        "sub": uuid(),
        "name": full_name(),
        "iat": timestamp_between(1640995200, 1672531200), // 2022-2023
        "exp": timestamp_between(1672531200, 1704067200)  // 2023-2024
    }).to_string());
    let signature = alphanumeric(43);

    format!("{}.{}.{}", header, payload, signature)
}

/// Generate bearer tokens
pub fn bearer_token() -> String {
    alphanumeric(64)
}

/// Generate request IDs for tracing
pub fn request_id() -> String {
    format!("req_{}", hex_string(32))
}

/// Generate correlation IDs for distributed tracing
pub fn correlation_id() -> String {
    format!("corr_{}", uuid())
}

/// Generate webhook URLs
pub fn webhook_url() -> String {
    format!(
        "https://{}.webhook.site/{}",
        alphanumeric(8),
        uuid()
    )
}

/// Generate API response with error
pub fn error_response(code: u16, message: &str) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "code": code,
            "message": message,
            "timestamp": timestamp_between(1640995200, 1704067200),
            "request_id": request_id()
        }
    })
}

/// Generate successful API response
pub fn success_response(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "data": data,
        "status": "success",
        "timestamp": timestamp_between(1640995200, 1704067200),
        "request_id": request_id()
    })
}

/// Generate paginated response
pub fn paginated_response(items: Vec<serde_json::Value>, page: u32, per_page: u32, total: u32) -> serde_json::Value {
    serde_json::json!({
        "data": items,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "total_pages": (total + per_page - 1) / per_page,
            "has_next": page * per_page < total,
            "has_prev": page > 1
        },
        "status": "success",
        "timestamp": timestamp_between(1640995200, 1704067200)
    })
}

/// Generate rate limit headers
pub fn rate_limit_headers() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut headers = HashMap::new();
    headers.insert("X-RateLimit-Limit".to_string(), "100".to_string());
    headers.insert("X-RateLimit-Remaining".to_string(), rand::thread_rng().gen_range(0..100).to_string());
    headers.insert("X-RateLimit-Reset".to_string(), (chrono::Utc::now().timestamp() + 3600).to_string());
    headers
}

/// Generate User-Agent strings
pub fn user_agent() -> String {
    let browsers = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
        "Tachikoma/1.0 (CLI)",
        "curl/7.68.0"
    ];
    
    use rand::Rng;
    browsers[rand::thread_rng().gen_range(0..browsers.len())].to_string()
}