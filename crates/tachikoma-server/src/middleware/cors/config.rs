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