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