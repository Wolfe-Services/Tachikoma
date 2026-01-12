# tachikoma-common-http

HTTP client utilities for Tachikoma.

## Features

- Configured reqwest client with sensible defaults
- Connection pooling
- Timeout configuration
- User-Agent header
- Request/response logging (debug)
- Error handling with rate limiting support

## Usage

```rust
use tachikoma_common_http::{HttpClient, HttpConfig};

// Create with defaults
let client = HttpClient::new()?;

// Create with custom config
let config = HttpConfig {
    connect_timeout: Duration::from_secs(5),
    request_timeout: Duration::from_secs(10),
    ..Default::default()
};
let client = HttpClient::with_config(config)?;

// Make requests
let response = client.get("https://api.example.com").await?;
```