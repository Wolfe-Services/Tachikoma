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