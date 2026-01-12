# Spec 382: Authentication Middleware

## Overview
Implement authentication middleware for protecting routes and extracting user context.

## Rust Implementation

### Auth Middleware
```rust
// src/auth/middleware.rs

use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{debug, warn, instrument};

use super::jwt::{JwtHandler, Claims, extract_bearer_token};
use super::session::SessionStore;
use super::types::{AuthContext, UserRole, Permission};

/// Authentication state shared across middleware
pub struct AuthState {
    pub jwt_handler: JwtHandler,
    pub session_store: Arc<dyn SessionStore + Send + Sync>,
    pub config: AuthMiddlewareConfig,
}

/// Middleware configuration
#[derive(Debug, Clone)]
pub struct AuthMiddlewareConfig {
    /// Allow unauthenticated requests (sets empty context)
    pub allow_anonymous: bool,
    /// Required role (if any)
    pub required_role: Option<UserRole>,
    /// Required permissions (all must match)
    pub required_permissions: Vec<Permission>,
    /// Skip auth for specific paths
    pub skip_paths: Vec<String>,
    /// Cookie name for session token
    pub session_cookie_name: String,
}

impl Default for AuthMiddlewareConfig {
    fn default() -> Self {
        Self {
            allow_anonymous: false,
            required_role: None,
            required_permissions: vec![],
            skip_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/api/auth/login".to_string(),
                "/api/auth/register".to_string(),
            ],
            session_cookie_name: "session".to_string(),
        }
    }
}

/// Authentication middleware
#[instrument(skip_all)]
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let path = request.uri().path().to_string();

    // Check if path should skip auth
    if state.config.skip_paths.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    // Try to extract auth context
    let auth_context = extract_auth_context(&state, &request).await;

    match auth_context {
        Ok(ctx) => {
            // Validate role if required
            if let Some(required_role) = &state.config.required_role {
                if !ctx.has_role(*required_role) {
                    warn!("Insufficient role for user {}", ctx.user_id);
                    return Err(AuthError::InsufficientRole);
                }
            }

            // Validate permissions if required
            for permission in &state.config.required_permissions {
                if !ctx.has_permission(permission) {
                    warn!("Missing permission {:?} for user {}", permission, ctx.user_id);
                    return Err(AuthError::InsufficientPermissions);
                }
            }

            // Insert context into request extensions
            request.extensions_mut().insert(ctx);
            Ok(next.run(request).await)
        }
        Err(e) => {
            if state.config.allow_anonymous {
                // Insert anonymous context
                request.extensions_mut().insert(AuthContext::anonymous());
                Ok(next.run(request).await)
            } else {
                Err(e)
            }
        }
    }
}

/// Extract authentication context from request
async fn extract_auth_context(
    state: &AuthState,
    request: &Request,
) -> Result<AuthContext, AuthError> {
    // Try Bearer token first
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = extract_bearer_token(auth_str) {
                return validate_jwt(state, token).await;
            }
        }
    }

    // Try session cookie
    if let Some(cookie_header) = request.headers().get(header::COOKIE) {
        if let Ok(cookies) = cookie_header.to_str() {
            if let Some(session_token) = extract_cookie(cookies, &state.config.session_cookie_name) {
                return validate_session(state, session_token).await;
            }
        }
    }

    // Try API key header
    if let Some(api_key) = request.headers().get("X-API-Key") {
        if let Ok(key) = api_key.to_str() {
            return validate_api_key(state, key).await;
        }
    }

    Err(AuthError::MissingCredentials)
}

/// Validate JWT token
async fn validate_jwt(state: &AuthState, token: &str) -> Result<AuthContext, AuthError> {
    let claims = state.jwt_handler.validate(token)
        .map_err(|e| {
            debug!("JWT validation failed: {}", e);
            AuthError::InvalidToken
        })?;

    Ok(AuthContext::from_claims(&claims))
}

/// Validate session token
async fn validate_session(state: &AuthState, token: &str) -> Result<AuthContext, AuthError> {
    let session = state.session_store.get(token).await
        .map_err(|_| AuthError::InvalidSession)?
        .ok_or(AuthError::InvalidSession)?;

    if session.is_expired() {
        return Err(AuthError::SessionExpired);
    }

    // Update session activity
    let _ = state.session_store.touch(token).await;

    Ok(AuthContext {
        user_id: session.user_id,
        email: session.email,
        role: session.role,
        permissions: session.permissions,
        session_id: Some(session.id),
        auth_method: session.auth_method,
        tenant_id: session.tenant_id,
    })
}

/// Validate API key
async fn validate_api_key(_state: &AuthState, _key: &str) -> Result<AuthContext, AuthError> {
    // API key validation would go here
    // For now, return error
    Err(AuthError::InvalidApiKey)
}

/// Extract cookie value by name
fn extract_cookie(cookies: &str, name: &str) -> Option<&str> {
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if let Some((key, value)) = cookie.split_once('=') {
            if key.trim() == name {
                return Some(value.trim());
            }
        }
    }
    None
}

/// Auth middleware error
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication credentials")]
    MissingCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Invalid session")]
    InvalidSession,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Insufficient role")]
    InsufficientRole,

    #[error("Insufficient permissions")]
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingCredentials => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
            AuthError::InvalidSession => (StatusCode::UNAUTHORIZED, "Invalid session"),
            AuthError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired"),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Invalid API key"),
            AuthError::InsufficientRole => (StatusCode::FORBIDDEN, "Insufficient role"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };

        let body = serde_json::json!({
            "error": message,
            "code": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Extractor for authenticated user
#[derive(Debug, Clone)]
pub struct AuthUser(pub AuthContext);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .filter(|ctx| !ctx.is_anonymous())
            .map(AuthUser)
            .ok_or(AuthError::MissingCredentials)
    }
}

/// Extractor for optional authenticated user
#[derive(Debug, Clone)]
pub struct MaybeAuthUser(pub Option<AuthContext>);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for MaybeAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let ctx = parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .filter(|ctx| !ctx.is_anonymous());

        Ok(MaybeAuthUser(ctx))
    }
}

/// Role guard macro for handlers
#[macro_export]
macro_rules! require_role {
    ($ctx:expr, $role:expr) => {
        if !$ctx.has_role($role) {
            return Err($crate::auth::middleware::AuthError::InsufficientRole);
        }
    };
}

/// Permission guard macro for handlers
#[macro_export]
macro_rules! require_permission {
    ($ctx:expr, $permission:expr) => {
        if !$ctx.has_permission(&$permission) {
            return Err($crate::auth::middleware::AuthError::InsufficientPermissions);
        }
    };
}

/// Create auth middleware layer with configuration
pub fn auth_layer(state: Arc<AuthState>) -> axum::middleware::from_fn_with_state<Arc<AuthState>, fn(State<Arc<AuthState>>, Request, Next) -> impl std::future::Future<Output = Result<Response, AuthError>>> {
    axum::middleware::from_fn_with_state(state, auth_middleware)
}

/// Builder for auth middleware
pub struct AuthMiddlewareBuilder {
    config: AuthMiddlewareConfig,
}

impl AuthMiddlewareBuilder {
    pub fn new() -> Self {
        Self {
            config: AuthMiddlewareConfig::default(),
        }
    }

    pub fn allow_anonymous(mut self, allow: bool) -> Self {
        self.config.allow_anonymous = allow;
        self
    }

    pub fn require_role(mut self, role: UserRole) -> Self {
        self.config.required_role = Some(role);
        self
    }

    pub fn require_permission(mut self, permission: Permission) -> Self {
        self.config.required_permissions.push(permission);
        self
    }

    pub fn skip_path(mut self, path: &str) -> Self {
        self.config.skip_paths.push(path.to_string());
        self
    }

    pub fn build(self) -> AuthMiddlewareConfig {
        self.config
    }
}

impl Default for AuthMiddlewareBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cookie() {
        let cookies = "session=abc123; theme=dark; lang=en";

        assert_eq!(extract_cookie(cookies, "session"), Some("abc123"));
        assert_eq!(extract_cookie(cookies, "theme"), Some("dark"));
        assert_eq!(extract_cookie(cookies, "missing"), None);
    }

    #[test]
    fn test_auth_middleware_builder() {
        let config = AuthMiddlewareBuilder::new()
            .allow_anonymous(true)
            .require_role(UserRole::Admin)
            .skip_path("/public")
            .build();

        assert!(config.allow_anonymous);
        assert_eq!(config.required_role, Some(UserRole::Admin));
        assert!(config.skip_paths.contains(&"/public".to_string()));
    }
}
```

## Files to Create
- `src/auth/middleware.rs` - Authentication middleware
