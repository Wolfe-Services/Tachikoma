# Spec 385: Authentication Errors

## Overview
Define comprehensive error types and handling for the authentication system.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Auth Error Types
```rust
// src/auth/errors.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Main authentication error type
#[derive(Debug, Error)]
pub enum AuthError {
    // === Credential Errors ===
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("User not found")]
    UserNotFound,

    #[error("Account is disabled")]
    AccountDisabled,

    #[error("Account is locked")]
    AccountLocked,

    #[error("Account pending verification")]
    AccountPendingVerification,

    #[error("Account deleted")]
    AccountDeleted,

    // === Password Errors ===
    #[error("Weak password: {0}")]
    WeakPassword(String),

    #[error("Password recently used")]
    PasswordRecentlyUsed,

    #[error("Password expired")]
    PasswordExpired,

    // === Token Errors ===
    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Token revoked")]
    TokenRevoked,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Refresh token required")]
    RefreshTokenRequired,

    // === Session Errors ===
    #[error("Session expired")]
    SessionExpired,

    #[error("Session invalid")]
    SessionInvalid,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Too many active sessions")]
    TooManySessions,

    // === OAuth Errors ===
    #[error("OAuth state mismatch")]
    OAuthStateMismatch,

    #[error("OAuth state expired")]
    OAuthStateExpired,

    #[error("OAuth code exchange failed: {0}")]
    OAuthCodeExchangeFailed(String),

    #[error("OAuth user info failed: {0}")]
    OAuthUserInfoFailed(String),

    #[error("OAuth account already linked")]
    OAuthAccountAlreadyLinked,

    #[error("OAuth account not linked")]
    OAuthAccountNotLinked,

    #[error("OAuth email not verified")]
    OAuthEmailNotVerified,

    #[error("OAuth email domain not allowed: {0}")]
    OAuthEmailDomainNotAllowed(String),

    #[error("OAuth provider error: {0}")]
    OAuthProviderError(String),

    // === Magic Link Errors ===
    #[error("Magic link expired")]
    MagicLinkExpired,

    #[error("Magic link already used")]
    MagicLinkAlreadyUsed,

    #[error("Magic link invalid")]
    MagicLinkInvalid,

    // === Device Code Errors ===
    #[error("Device code expired")]
    DeviceCodeExpired,

    #[error("Device authorization pending")]
    DeviceAuthorizationPending,

    #[error("Device authorization denied")]
    DeviceAuthorizationDenied,

    #[error("Device code invalid")]
    DeviceCodeInvalid,

    #[error("Polling too fast")]
    DeviceCodeSlowDown,

    // === Permission Errors ===
    #[error("Permission denied")]
    PermissionDenied,

    #[error("Insufficient role")]
    InsufficientRole,

    #[error("Missing permission: {0}")]
    MissingPermission(String),

    // === Rate Limiting ===
    #[error("Too many requests")]
    RateLimited,

    #[error("Too many failed attempts")]
    TooManyFailedAttempts,

    // === MFA Errors ===
    #[error("MFA required")]
    MfaRequired,

    #[error("MFA code invalid")]
    MfaCodeInvalid,

    #[error("MFA code expired")]
    MfaCodeExpired,

    #[error("MFA not configured")]
    MfaNotConfigured,

    // === Tenant Errors ===
    #[error("Tenant not found")]
    TenantNotFound,

    #[error("Tenant access denied")]
    TenantAccessDenied,

    #[error("Tenant disabled")]
    TenantDisabled,

    // === Internal Errors ===
    #[error("Database error: {0}")]
    Database(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Email service error: {0}")]
    EmailService(String),
}

impl AuthError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            Self::WeakPassword(_) |
            Self::PasswordRecentlyUsed |
            Self::DeviceCodeInvalid |
            Self::MagicLinkInvalid => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            Self::InvalidCredentials |
            Self::TokenExpired |
            Self::TokenInvalid |
            Self::TokenRevoked |
            Self::TokenNotFound |
            Self::RefreshTokenRequired |
            Self::SessionExpired |
            Self::SessionInvalid |
            Self::SessionNotFound |
            Self::OAuthStateMismatch |
            Self::OAuthStateExpired |
            Self::MagicLinkExpired |
            Self::MagicLinkAlreadyUsed |
            Self::DeviceCodeExpired |
            Self::DeviceAuthorizationDenied |
            Self::MfaRequired |
            Self::MfaCodeInvalid |
            Self::MfaCodeExpired => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            Self::AccountDisabled |
            Self::AccountLocked |
            Self::AccountPendingVerification |
            Self::AccountDeleted |
            Self::PermissionDenied |
            Self::InsufficientRole |
            Self::MissingPermission(_) |
            Self::TenantAccessDenied |
            Self::TenantDisabled |
            Self::OAuthEmailDomainNotAllowed(_) => StatusCode::FORBIDDEN,

            // 404 Not Found
            Self::UserNotFound |
            Self::OAuthAccountNotLinked |
            Self::TenantNotFound |
            Self::MfaNotConfigured => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::EmailAlreadyExists |
            Self::OAuthAccountAlreadyLinked |
            Self::TooManySessions => StatusCode::CONFLICT,

            // 428 Precondition Required
            Self::DeviceAuthorizationPending => StatusCode::from_u16(428).unwrap(),

            // 429 Too Many Requests
            Self::RateLimited |
            Self::TooManyFailedAttempts |
            Self::DeviceCodeSlowDown => StatusCode::TOO_MANY_REQUESTS,

            // 500 Internal Server Error
            Self::Database(_) |
            Self::Internal(_) |
            Self::Configuration(_) |
            Self::EmailService(_) |
            Self::OAuthCodeExchangeFailed(_) |
            Self::OAuthUserInfoFailed(_) |
            Self::OAuthProviderError(_) |
            Self::PasswordExpired |
            Self::OAuthEmailNotVerified => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::EmailAlreadyExists => "EMAIL_EXISTS",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::AccountDisabled => "ACCOUNT_DISABLED",
            Self::AccountLocked => "ACCOUNT_LOCKED",
            Self::AccountPendingVerification => "ACCOUNT_PENDING",
            Self::AccountDeleted => "ACCOUNT_DELETED",
            Self::WeakPassword(_) => "WEAK_PASSWORD",
            Self::PasswordRecentlyUsed => "PASSWORD_RECENTLY_USED",
            Self::PasswordExpired => "PASSWORD_EXPIRED",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::TokenInvalid => "TOKEN_INVALID",
            Self::TokenRevoked => "TOKEN_REVOKED",
            Self::TokenNotFound => "TOKEN_NOT_FOUND",
            Self::RefreshTokenRequired => "REFRESH_TOKEN_REQUIRED",
            Self::SessionExpired => "SESSION_EXPIRED",
            Self::SessionInvalid => "SESSION_INVALID",
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::TooManySessions => "TOO_MANY_SESSIONS",
            Self::OAuthStateMismatch => "OAUTH_STATE_MISMATCH",
            Self::OAuthStateExpired => "OAUTH_STATE_EXPIRED",
            Self::OAuthCodeExchangeFailed(_) => "OAUTH_CODE_EXCHANGE_FAILED",
            Self::OAuthUserInfoFailed(_) => "OAUTH_USER_INFO_FAILED",
            Self::OAuthAccountAlreadyLinked => "OAUTH_ALREADY_LINKED",
            Self::OAuthAccountNotLinked => "OAUTH_NOT_LINKED",
            Self::OAuthEmailNotVerified => "OAUTH_EMAIL_NOT_VERIFIED",
            Self::OAuthEmailDomainNotAllowed(_) => "OAUTH_EMAIL_DOMAIN_NOT_ALLOWED",
            Self::OAuthProviderError(_) => "OAUTH_PROVIDER_ERROR",
            Self::MagicLinkExpired => "MAGIC_LINK_EXPIRED",
            Self::MagicLinkAlreadyUsed => "MAGIC_LINK_ALREADY_USED",
            Self::MagicLinkInvalid => "MAGIC_LINK_INVALID",
            Self::DeviceCodeExpired => "DEVICE_CODE_EXPIRED",
            Self::DeviceAuthorizationPending => "AUTHORIZATION_PENDING",
            Self::DeviceAuthorizationDenied => "ACCESS_DENIED",
            Self::DeviceCodeInvalid => "INVALID_GRANT",
            Self::DeviceCodeSlowDown => "SLOW_DOWN",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::InsufficientRole => "INSUFFICIENT_ROLE",
            Self::MissingPermission(_) => "MISSING_PERMISSION",
            Self::RateLimited => "RATE_LIMITED",
            Self::TooManyFailedAttempts => "TOO_MANY_ATTEMPTS",
            Self::MfaRequired => "MFA_REQUIRED",
            Self::MfaCodeInvalid => "MFA_CODE_INVALID",
            Self::MfaCodeExpired => "MFA_CODE_EXPIRED",
            Self::MfaNotConfigured => "MFA_NOT_CONFIGURED",
            Self::TenantNotFound => "TENANT_NOT_FOUND",
            Self::TenantAccessDenied => "TENANT_ACCESS_DENIED",
            Self::TenantDisabled => "TENANT_DISABLED",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::EmailService(_) => "EMAIL_SERVICE_ERROR",
        }
    }

    /// Check if error should be logged
    pub fn should_log(&self) -> bool {
        matches!(self,
            Self::Database(_) |
            Self::Internal(_) |
            Self::Configuration(_) |
            Self::EmailService(_) |
            Self::OAuthCodeExchangeFailed(_) |
            Self::OAuthUserInfoFailed(_) |
            Self::OAuthProviderError(_)
        )
    }

    /// Check if error message is safe to expose
    pub fn is_safe_to_expose(&self) -> bool {
        !matches!(self,
            Self::Database(_) |
            Self::Internal(_) |
            Self::Configuration(_)
        )
    }
}

/// API error response
#[derive(Debug, Serialize)]
pub struct AuthErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Log internal errors
        if self.should_log() {
            tracing::error!("Auth error: {}", self);
        }

        // Build response
        let message = if self.is_safe_to_expose() {
            self.to_string()
        } else {
            "An internal error occurred".to_string()
        };

        let mut response = AuthErrorResponse {
            error: message,
            code: self.error_code().to_string(),
            details: None,
            retry_after: None,
        };

        // Add retry-after for rate limiting
        if matches!(self, AuthError::RateLimited | AuthError::DeviceCodeSlowDown) {
            response.retry_after = Some(5);
        }

        (status, Json(response)).into_response()
    }
}

// === Conversions from other error types ===

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::Database(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::TokenInvalid,
            _ => AuthError::TokenInvalid,
        }
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(_: argon2::password_hash::Error) -> Self {
        AuthError::InvalidCredentials
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::OAuthProviderError(err.to_string())
    }
}

/// Result type alias for auth operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Extension trait for adding context to errors
pub trait AuthErrorContext<T> {
    fn context(self, context: &str) -> AuthResult<T>;
}

impl<T, E: std::error::Error> AuthErrorContext<T> for Result<T, E> {
    fn context(self, context: &str) -> AuthResult<T> {
        self.map_err(|e| AuthError::Internal(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(AuthError::InvalidCredentials.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AuthError::AccountDisabled.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AuthError::EmailAlreadyExists.status_code(), StatusCode::CONFLICT);
        assert_eq!(AuthError::RateLimited.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AuthError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AuthError::DeviceAuthorizationPending.error_code(), "AUTHORIZATION_PENDING");
    }

    #[test]
    fn test_error_logging() {
        assert!(AuthError::Database("test".to_string()).should_log());
        assert!(!AuthError::InvalidCredentials.should_log());
    }

    #[test]
    fn test_error_exposure() {
        assert!(AuthError::InvalidCredentials.is_safe_to_expose());
        assert!(!AuthError::Database("test".to_string()).is_safe_to_expose());
    }
}
```

## Files to Create
- `src/auth/errors.rs` - Authentication error types
