//! API error types.

use axum::http::StatusCode;
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

    #[error("Resource not found")]
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