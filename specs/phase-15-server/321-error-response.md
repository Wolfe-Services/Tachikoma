# 321 - Error Response

**Phase:** 15 - Server
**Spec ID:** 321
**Status:** Planned
**Dependencies:** 319-request-response
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Create a comprehensive error handling system with typed errors, proper HTTP status codes, and consistent error response formatting across all endpoints.

---

## Acceptance Criteria

- [x] ApiError enum with variants
- [x] Automatic HTTP status mapping
- [x] IntoResponse implementation
- [x] Error context/chaining
- [x] Error logging integration
- [x] Stack trace in dev mode
- [x] Error codes for client handling

---

## Implementation Details

### 1. Error Types (crates/tachikoma-server/src/error/types.rs)

```rust
//! API error types.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

/// Result type for API operations.
pub type ApiResult<T> = Result<T, ApiError>;

/// API error enum covering all error cases.
#[derive(Debug, Error)]
pub enum ApiError {
    // 400 Bad Request
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation failed")]
    ValidationError(HashMap<String, Vec<String>>),

    #[error("Invalid query parameter: {0}")]
    InvalidQueryParam(String),

    // 401 Unauthorized
    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    // 403 Forbidden
    #[error("Access denied")]
    Forbidden,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Resource access denied")]
    ResourceAccessDenied(String),

    // 404 Not Found
    #[error("{0} not found")]
    NotFound(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound { resource: String, id: String },

    // 409 Conflict
    #[error("Resource already exists: {0}")]
    Conflict(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("State conflict: {0}")]
    StateConflict(String),

    // 422 Unprocessable Entity
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    // 429 Too Many Requests
    #[error("Rate limit exceeded")]
    RateLimited {
        retry_after: u64,
    },

    // 500 Internal Server Error
    #[error("Internal server error")]
    Internal(#[source] anyhow::Error),

    #[error("Database error")]
    Database(#[source] sqlx::Error),

    // 502 Bad Gateway
    #[error("Upstream service error: {0}")]
    UpstreamError(String),

    // 503 Service Unavailable
    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Service temporarily unavailable: {0}")]
    ServiceTemporarilyUnavailable(String),
}

impl ApiError {
    /// Get HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_)
            | Self::ValidationError(_)
            | Self::InvalidQueryParam(_) => StatusCode::BAD_REQUEST,

            Self::Unauthorized
            | Self::InvalidCredentials
            | Self::TokenExpired
            | Self::InvalidToken => StatusCode::UNAUTHORIZED,

            Self::Forbidden
            | Self::InsufficientPermissions
            | Self::ResourceAccessDenied(_) => StatusCode::FORBIDDEN,

            Self::NotFound(_)
            | Self::ResourceNotFound { .. } => StatusCode::NOT_FOUND,

            Self::Conflict(_)
            | Self::DuplicateEntry(_)
            | Self::StateConflict(_) => StatusCode::CONFLICT,

            Self::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,

            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,

            Self::Internal(_)
            | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Self::UpstreamError(_) => StatusCode::BAD_GATEWAY,

            Self::ServiceUnavailable
            | Self::ServiceTemporarilyUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get error code for client handling.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::ValidationError(_) => "validation_error",
            Self::InvalidQueryParam(_) => "invalid_query_param",
            Self::Unauthorized => "unauthorized",
            Self::InvalidCredentials => "invalid_credentials",
            Self::TokenExpired => "token_expired",
            Self::InvalidToken => "invalid_token",
            Self::Forbidden => "forbidden",
            Self::InsufficientPermissions => "insufficient_permissions",
            Self::ResourceAccessDenied(_) => "resource_access_denied",
            Self::NotFound(_) => "not_found",
            Self::ResourceNotFound { .. } => "resource_not_found",
            Self::Conflict(_) => "conflict",
            Self::DuplicateEntry(_) => "duplicate_entry",
            Self::StateConflict(_) => "state_conflict",
            Self::UnprocessableEntity(_) => "unprocessable_entity",
            Self::RateLimited { .. } => "rate_limited",
            Self::Internal(_) => "internal_error",
            Self::Database(_) => "database_error",
            Self::UpstreamError(_) => "upstream_error",
            Self::ServiceUnavailable => "service_unavailable",
            Self::ServiceTemporarilyUnavailable(_) => "service_temporarily_unavailable",
        }
    }

    /// Check if this is a client error (4xx).
    pub fn is_client_error(&self) -> bool {
        self.status_code().is_client_error()
    }

    /// Check if this is a server error (5xx).
    pub fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }
}
```

### 2. Error Response (crates/tachikoma-server/src/error/response.rs)

```rust
//! Error response implementation.

use super::types::ApiError;
use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::{error, warn};

/// Error response body.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<std::collections::HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after: Option<u64>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log based on error type
        if self.is_server_error() {
            error!(
                error = %self,
                code = self.error_code(),
                "Server error occurred"
            );
        } else if matches!(
            self,
            ApiError::Unauthorized | ApiError::InvalidCredentials | ApiError::Forbidden
        ) {
            warn!(
                error = %self,
                code = self.error_code(),
                "Auth error occurred"
            );
        }

        let status = self.status_code();
        let code = self.error_code();

        // Build response body
        let (message, details, fields, retry_after) = match &self {
            ApiError::ValidationError(field_errors) => {
                (self.to_string(), None, Some(field_errors.clone()), None)
            }
            ApiError::RateLimited { retry_after } => {
                (self.to_string(), None, None, Some(*retry_after))
            }
            ApiError::ResourceNotFound { resource, id } => {
                let details = serde_json::json!({
                    "resource": resource,
                    "id": id
                });
                (self.to_string(), Some(details), None, None)
            }
            ApiError::Internal(err) => {
                // Don't expose internal error details in production
                let message = if cfg!(debug_assertions) {
                    format!("{}: {}", self, err)
                } else {
                    "An internal error occurred".to_string()
                };
                (message, None, None, None)
            }
            ApiError::Database(err) => {
                // Don't expose database errors in production
                let message = if cfg!(debug_assertions) {
                    format!("Database error: {}", err)
                } else {
                    "A database error occurred".to_string()
                };
                (message, None, None, None)
            }
            _ => (self.to_string(), None, None, None),
        };

        let body = ErrorResponse {
            success: false,
            error: ErrorBody {
                code,
                message,
                details,
                fields,
                retry_after,
            },
        };

        let mut response = (status, Json(body)).into_response();

        // Add retry-after header for rate limiting
        if let ApiError::RateLimited { retry_after } = self {
            response.headers_mut().insert(
                header::RETRY_AFTER,
                retry_after.to_string().parse().unwrap(),
            );
        }

        response
    }
}

// Conversion implementations
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Record".into()),
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violation
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return ApiError::DuplicateEntry(
                            db_err.message().to_string()
                        );
                    }
                }
                ApiError::Database(err)
            }
            _ => ApiError::Database(err),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::TokenExpired,
            _ => ApiError::InvalidToken,
        }
    }
}
```

### 3. Error Context Extension (crates/tachikoma-server/src/error/context.rs)

```rust
//! Error context utilities.

use super::types::ApiError;

/// Extension trait for adding context to errors.
pub trait ErrorContext<T> {
    /// Add context to an error, converting to ApiError.
    fn context(self, context: impl Into<String>) -> Result<T, ApiError>;

    /// Add context for not found errors.
    fn not_found(self, resource: impl Into<String>) -> Result<T, ApiError>;

    /// Add context for forbidden errors.
    fn forbidden(self, message: impl Into<String>) -> Result<T, ApiError>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> ErrorContext<T> for Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T, ApiError> {
        self.map_err(|e| {
            ApiError::Internal(anyhow::Error::from(e).context(context.into()))
        })
    }

    fn not_found(self, resource: impl Into<String>) -> Result<T, ApiError> {
        self.map_err(|_| ApiError::NotFound(resource.into()))
    }

    fn forbidden(self, message: impl Into<String>) -> Result<T, ApiError> {
        self.map_err(|_| ApiError::ResourceAccessDenied(message.into()))
    }
}

impl<T> ErrorContext<T> for Option<T> {
    fn context(self, context: impl Into<String>) -> Result<T, ApiError> {
        self.ok_or_else(|| ApiError::NotFound(context.into()))
    }

    fn not_found(self, resource: impl Into<String>) -> Result<T, ApiError> {
        self.ok_or_else(|| ApiError::NotFound(resource.into()))
    }

    fn forbidden(self, message: impl Into<String>) -> Result<T, ApiError> {
        self.ok_or_else(|| ApiError::ResourceAccessDenied(message.into()))
    }
}

/// Create a not found error for a specific resource.
pub fn not_found(resource: &str, id: &str) -> ApiError {
    ApiError::ResourceNotFound {
        resource: resource.to_string(),
        id: id.to_string(),
    }
}

/// Create a conflict error.
pub fn conflict(message: impl Into<String>) -> ApiError {
    ApiError::Conflict(message.into())
}

/// Create a state conflict error.
pub fn state_conflict(message: impl Into<String>) -> ApiError {
    ApiError::StateConflict(message.into())
}
```

---

## Testing Requirements

1. All error types map to correct status
2. Error codes are consistent
3. Validation errors include fields
4. Rate limit includes retry-after
5. Internal errors sanitized in prod
6. Database errors handled properly
7. Context propagates correctly

---

## Related Specs

- Depends on: [319-request-response.md](319-request-response.md)
- Next: [322-auth-middleware.md](322-auth-middleware.md)
- Used by: All handlers
