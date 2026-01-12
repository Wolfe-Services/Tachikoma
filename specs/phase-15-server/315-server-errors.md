# Spec 315: Error Handling

## Phase
15 - Server/API Layer

## Spec ID
315

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 314: Middleware

## Estimated Context
~9%

---

## Objective

Implement comprehensive error handling for the Tachikoma API, providing consistent error responses, proper HTTP status codes, detailed error information for debugging, and appropriate error sanitization for production environments.

---

## Acceptance Criteria

- [ ] All errors return consistent JSON response format
- [ ] HTTP status codes accurately reflect error types
- [ ] Error codes are unique and documentable
- [ ] Stack traces are included in development, hidden in production
- [ ] Validation errors include field-level details
- [ ] Errors are properly logged with context
- [ ] Error responses include request ID for correlation
- [ ] Custom error types integrate seamlessly with Axum

---

## Implementation Details

### Error Response Structure

```rust
// src/server/error/response.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error information
    pub error: ErrorInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Machine-readable error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// HTTP status code
    #[serde(skip_serializing)]
    pub status: u16,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,

    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Stack trace (development only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorDetails {
    /// Validation errors with field-level information
    Validation(ValidationErrors),

    /// Resource-related errors
    Resource(ResourceError),

    /// Rate limiting information
    RateLimit(RateLimitInfo),

    /// Generic key-value details
    Generic(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub fields: Vec<FieldError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceError {
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: i64,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>, status: u16) -> Self {
        Self {
            error: ErrorInfo {
                code: code.into(),
                message: message.into(),
                status,
                details: None,
                request_id: None,
                trace: None,
            },
        }
    }

    pub fn with_details(mut self, details: ErrorDetails) -> Self {
        self.error.details = Some(details);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.error.request_id = Some(request_id.into());
        self
    }

    pub fn with_trace(mut self, trace: impl Into<String>) -> Self {
        self.error.trace = Some(trace.into());
        self
    }
}
```

### API Error Types

```rust
// src/server/error/types.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::backtrace::Backtrace;
use thiserror::Error;

use super::response::{ErrorDetails, ErrorResponse, FieldError, ResourceError, ValidationErrors};

/// Main API error type
#[derive(Debug, Error)]
pub enum ApiError {
    // 400 Bad Request
    #[error("Invalid request: {message}")]
    BadRequest {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // 400 Validation Error
    #[error("Validation failed")]
    Validation {
        errors: Vec<FieldError>,
    },

    // 401 Unauthorized
    #[error("Authentication required")]
    Unauthorized {
        message: String,
    },

    // 403 Forbidden
    #[error("Permission denied: {message}")]
    Forbidden {
        message: String,
    },

    // 404 Not Found
    #[error("{resource_type} not found")]
    NotFound {
        resource_type: String,
        resource_id: Option<String>,
    },

    // 409 Conflict
    #[error("Resource conflict: {message}")]
    Conflict {
        message: String,
    },

    // 422 Unprocessable Entity
    #[error("Cannot process request: {message}")]
    UnprocessableEntity {
        message: String,
    },

    // 429 Too Many Requests
    #[error("Rate limit exceeded")]
    RateLimited {
        limit: u32,
        remaining: u32,
        reset_at: i64,
    },

    // 500 Internal Server Error
    #[error("Internal server error")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Option<String>,
    },

    // 502 Bad Gateway
    #[error("Upstream service error: {message}")]
    BadGateway {
        message: String,
    },

    // 503 Service Unavailable
    #[error("Service temporarily unavailable")]
    ServiceUnavailable {
        message: String,
        retry_after: Option<u64>,
    },

    // 504 Gateway Timeout
    #[error("Request timed out")]
    Timeout {
        message: String,
    },
}

impl ApiError {
    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
            source: None,
        }
    }

    /// Create a validation error
    pub fn validation(errors: Vec<FieldError>) -> Self {
        Self::Validation { errors }
    }

    /// Create a single field validation error
    pub fn validation_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            errors: vec![FieldError {
                field: field.into(),
                message: message.into(),
                code: None,
            }],
        }
    }

    /// Create a not found error
    pub fn not_found(resource_type: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            resource_id: None,
        }
    }

    /// Create a not found error with ID
    pub fn not_found_with_id(
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            resource_id: Some(resource_id.into()),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
            backtrace: Some(Backtrace::capture().to_string()),
        }
    }

    /// Create an internal error from another error
    pub fn internal_from<E: std::error::Error + Send + Sync + 'static>(error: E) -> Self {
        Self::Internal {
            message: error.to_string(),
            source: Some(Box::new(error)),
            backtrace: Some(Backtrace::capture().to_string()),
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest { .. } => StatusCode::BAD_REQUEST,
            Self::Validation { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadGateway { .. } => StatusCode::BAD_GATEWAY,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout { .. } => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest { .. } => "BAD_REQUEST",
            Self::Validation { .. } => "VALIDATION_ERROR",
            Self::Unauthorized { .. } => "UNAUTHORIZED",
            Self::Forbidden { .. } => "FORBIDDEN",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict { .. } => "CONFLICT",
            Self::UnprocessableEntity { .. } => "UNPROCESSABLE_ENTITY",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::Internal { .. } => "INTERNAL_ERROR",
            Self::BadGateway { .. } => "BAD_GATEWAY",
            Self::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            Self::Timeout { .. } => "TIMEOUT",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let message = self.to_string();

        let mut response = ErrorResponse::new(code, message, status.as_u16());

        // Add details based on error type
        match &self {
            ApiError::Validation { errors } => {
                response = response.with_details(ErrorDetails::Validation(ValidationErrors {
                    fields: errors.clone(),
                }));
            }
            ApiError::NotFound {
                resource_type,
                resource_id,
            } => {
                response = response.with_details(ErrorDetails::Resource(ResourceError {
                    resource_type: resource_type.clone(),
                    resource_id: resource_id.clone(),
                }));
            }
            ApiError::RateLimited {
                limit,
                remaining,
                reset_at,
            } => {
                response = response.with_details(ErrorDetails::RateLimit(
                    super::response::RateLimitInfo {
                        limit: *limit,
                        remaining: *remaining,
                        reset_at: *reset_at,
                    },
                ));
            }
            ApiError::Internal { backtrace, .. } => {
                // Only include trace in development
                #[cfg(debug_assertions)]
                if let Some(trace) = backtrace {
                    response = response.with_trace(trace.clone());
                }
            }
            _ => {}
        }

        // Log the error
        match status {
            s if s.is_server_error() => {
                tracing::error!(
                    error_code = %code,
                    error_message = %self,
                    "Server error"
                );
            }
            _ => {
                tracing::warn!(
                    error_code = %code,
                    error_message = %self,
                    "Client error"
                );
            }
        }

        (status, Json(response)).into_response()
    }
}
```

### Error Conversion Implementations

```rust
// src/server/error/conversions.rs
use super::types::ApiError;

// From storage errors
impl From<crate::storage::StorageError> for ApiError {
    fn from(err: crate::storage::StorageError) -> Self {
        match err {
            crate::storage::StorageError::NotFound { entity, id } => {
                ApiError::not_found_with_id(entity, id)
            }
            crate::storage::StorageError::Conflict { message } => {
                ApiError::Conflict { message }
            }
            crate::storage::StorageError::Connection(e) => {
                ApiError::ServiceUnavailable {
                    message: "Database connection failed".to_string(),
                    retry_after: Some(5),
                }
            }
            e => ApiError::internal_from(e),
        }
    }
}

// From serialization errors
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::bad_request(format!("Invalid JSON: {}", err))
    }
}

// From UUID parse errors
impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        ApiError::bad_request(format!("Invalid UUID: {}", err))
    }
}

// From Axum rejection types
impl From<axum::extract::rejection::JsonRejection> for ApiError {
    fn from(rejection: axum::extract::rejection::JsonRejection) -> Self {
        ApiError::bad_request(rejection.to_string())
    }
}

impl From<axum::extract::rejection::PathRejection> for ApiError {
    fn from(rejection: axum::extract::rejection::PathRejection) -> Self {
        ApiError::bad_request(rejection.to_string())
    }
}

impl From<axum::extract::rejection::QueryRejection> for ApiError {
    fn from(rejection: axum::extract::rejection::QueryRejection) -> Self {
        ApiError::bad_request(rejection.to_string())
    }
}

// From standard I/O errors
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::internal(err.to_string())
    }
}

// Generic anyhow integration
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal {
            message: err.to_string(),
            source: None,
            backtrace: Some(format!("{:?}", err)),
        }
    }
}
```

### Result Type Alias

```rust
// src/server/error/mod.rs
pub mod conversions;
pub mod response;
pub mod types;

pub use response::{ErrorDetails, ErrorResponse, FieldError, ValidationErrors};
pub use types::ApiError;

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;
```

### Error Handler for Panics

```rust
// src/server/error/panic.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::panic::PanicInfo;

use super::response::ErrorResponse;

/// Panic handler that converts panics to 500 errors
pub fn panic_handler(info: &PanicInfo) -> Response {
    let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };

    tracing::error!(
        panic_message = %message,
        location = ?info.location(),
        "Handler panicked"
    );

    let response = ErrorResponse::new(
        "INTERNAL_ERROR",
        "An unexpected error occurred",
        500,
    );

    #[cfg(debug_assertions)]
    let response = response.with_trace(format!(
        "Panic at {:?}: {}",
        info.location(),
        message
    ));

    (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
}
```

### Error Middleware

```rust
// src/server/middleware/error_handler.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::panic::AssertUnwindSafe;
use futures::FutureExt;

use crate::server::error::{ApiError, ErrorResponse};
use crate::server::middleware::request_id::RequestId;

/// Middleware to catch panics and convert to error responses
pub async fn catch_panic(
    request_id: RequestId,
    request: Request,
    next: Next,
) -> Response {
    let result = AssertUnwindSafe(next.run(request))
        .catch_unwind()
        .await;

    match result {
        Ok(response) => response,
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("Unknown panic");

            tracing::error!(
                request_id = %request_id.0,
                panic_message = %message,
                "Handler panicked"
            );

            let mut response = ErrorResponse::new(
                "INTERNAL_ERROR",
                "An unexpected error occurred",
                500,
            ).with_request_id(request_id.0);

            #[cfg(debug_assertions)]
            {
                response = response.with_trace(message.to_string());
            }

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::bad_request("test").status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::not_found("User").status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::internal("test").status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_validation_error_details() {
        let error = ApiError::validation(vec![
            FieldError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
                code: Some("INVALID_FORMAT".to_string()),
            },
        ]);

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse::new("NOT_FOUND", "User not found", 404)
            .with_request_id("req-123");

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("req-123"));
    }

    #[test]
    fn test_not_found_with_resource_details() {
        let error = ApiError::not_found_with_id("Mission", "abc-123");

        match error {
            ApiError::NotFound { resource_type, resource_id } => {
                assert_eq!(resource_type, "Mission");
                assert_eq!(resource_id, Some("abc-123".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[tokio::test]
    async fn test_error_into_response() {
        let error = ApiError::not_found("Mission");
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 328**: Request Validation
- **Spec 329**: Response Types
