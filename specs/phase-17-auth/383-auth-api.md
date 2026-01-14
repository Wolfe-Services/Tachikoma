# Spec 383: Authentication API

## Overview
Implement REST API endpoints for authentication operations including login, logout, registration, and token management.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Auth API Handlers
```rust
// src/auth/api.rs

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, instrument};

use super::middleware::{AuthUser, MaybeAuthUser};
use super::types::*;
use super::session::*;
use super::jwt::*;
use super::refresh::*;

/// Auth API state
pub struct AuthApiState {
    pub auth_service: Arc<AuthService>,
    pub session_store: Arc<dyn SessionStore + Send + Sync>,
    pub jwt_handler: JwtHandler,
    pub refresh_service: RefreshTokenService,
    pub config: AuthApiConfig,
}

/// API configuration
#[derive(Debug, Clone)]
pub struct AuthApiConfig {
    pub session_cookie_name: String,
    pub session_cookie_secure: bool,
    pub session_cookie_http_only: bool,
    pub session_cookie_same_site: String,
    pub allow_registration: bool,
    pub require_email_verification: bool,
}

impl Default for AuthApiConfig {
    fn default() -> Self {
        Self {
            session_cookie_name: "session".to_string(),
            session_cookie_secure: true,
            session_cookie_http_only: true,
            session_cookie_same_site: "Lax".to_string(),
            allow_registration: true,
            require_email_verification: true,
        }
    }
}

/// Create auth router
pub fn auth_router(state: Arc<AuthApiState>) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/register", post(register))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .route("/me", delete(delete_account))
        .route("/password", post(change_password))
        .route("/sessions", get(list_sessions))
        .route("/sessions/:id", delete(revoke_session))
        .route("/sessions/all", delete(revoke_all_sessions))
        .with_state(state)
}

// === Request/Response Types ===

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub remember_me: bool,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: UserResponse,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub role: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub device: Option<String>,
    pub ip_address: Option<String>,
    pub last_active: String,
    pub created_at: String,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(error: &str, code: &str) -> Self {
        Self {
            error: error.to_string(),
            code: code.to_string(),
            details: None,
        }
    }
}

// === Handlers ===

/// Login with email and password
#[instrument(skip(state, request))]
async fn login(
    State(state): State<Arc<AuthApiState>>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), (StatusCode, Json<ApiError>)> {
    // Authenticate user
    let user = state.auth_service
        .authenticate(&request.email, &request.password)
        .await
        .map_err(|e| {
            warn!("Login failed for {}: {}", request.email, e);
            (StatusCode::UNAUTHORIZED, Json(ApiError::new("Invalid credentials", "INVALID_CREDENTIALS")))
        })?;

    // Get permissions for role
    let permissions = super::permissions::permissions_for_role(user.role)
        .into_iter()
        .collect();

    // Create JWT
    let access_token = state.jwt_handler
        .create_access_token(&user.id, Some(user.email.clone()), user.role, permissions)
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "TOKEN_ERROR")))
        })?;

    let expires_in = state.jwt_handler.config().lifetime.num_seconds();

    // Create refresh token if remember_me
    let refresh_token = if request.remember_me {
        let (token, _) = state.refresh_service
            .create(&user.id, None, None, None)
            .await
            .map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "REFRESH_TOKEN_ERROR")))
            })?;
        Some(token)
    } else {
        None
    };

    // Create session cookie
    let cookie = create_session_cookie(
        &state.config,
        &access_token,
        request.remember_me,
    );

    let jar = jar.add(cookie);

    info!("User {} logged in successfully", user.id);

    Ok((jar, Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: user_to_response(&user),
    })))
}

/// Logout and invalidate session
#[instrument(skip(state, auth_user))]
async fn logout(
    State(state): State<Arc<AuthApiState>>,
    jar: CookieJar,
    auth_user: AuthUser,
) -> Result<(CookieJar, StatusCode), (StatusCode, Json<ApiError>)> {
    // Revoke session if exists
    if let Some(session_id) = &auth_user.0.session_id {
        let _ = state.session_store.delete(session_id).await;
    }

    // Clear cookie
    let cookie = axum_extra::extract::cookie::Cookie::build((&state.config.session_cookie_name, ""))
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    let jar = jar.remove(cookie);

    info!("User {} logged out", auth_user.0.user_id);
    Ok((jar, StatusCode::NO_CONTENT))
}

/// Register new user
#[instrument(skip(state, request))]
async fn register(
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, Json<ApiError>)> {
    if !state.config.allow_registration {
        return Err((StatusCode::FORBIDDEN, Json(ApiError::new("Registration is disabled", "REGISTRATION_DISABLED"))));
    }

    // Create user
    let user = state.auth_service
        .register(&request.email, &request.password, request.name.as_deref())
        .await
        .map_err(|e| match e {
            AuthError::EmailAlreadyExists => {
                (StatusCode::CONFLICT, Json(ApiError::new("Email already registered", "EMAIL_EXISTS")))
            }
            AuthError::WeakPassword(msg) => {
                (StatusCode::BAD_REQUEST, Json(ApiError::new(&msg, "WEAK_PASSWORD")))
            }
            _ => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "REGISTRATION_ERROR")))
            }
        })?;

    let response = if state.config.require_email_verification {
        // Send verification email
        // state.email_service.send_verification(&user.email).await?;

        RegisterResponse {
            user: user_to_response(&user),
            message: "Please check your email to verify your account".to_string(),
            access_token: None,
        }
    } else {
        // Auto-login
        let permissions = super::permissions::permissions_for_role(user.role)
            .into_iter()
            .collect();

        let access_token = state.jwt_handler
            .create_access_token(&user.id, Some(user.email.clone()), user.role, permissions)
            .ok();

        RegisterResponse {
            user: user_to_response(&user),
            message: "Registration successful".to_string(),
            access_token,
        }
    };

    info!("New user registered: {}", user.id);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Refresh access token
#[instrument(skip(state, request))]
async fn refresh_token(
    State(state): State<Arc<AuthApiState>>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, (StatusCode, Json<ApiError>)> {
    let get_user = |user_id: &str| {
        let user_id = user_id.to_string();
        let auth_service = state.auth_service.clone();
        Box::pin(async move {
            auth_service.get_user(&user_id).await
        })
    };

    let result = state.refresh_service
        .refresh(&request.refresh_token, get_user)
        .await
        .map_err(|e| match e {
            RefreshError::Expired => {
                (StatusCode::UNAUTHORIZED, Json(ApiError::new("Refresh token expired", "REFRESH_EXPIRED")))
            }
            RefreshError::Revoked | RefreshError::FamilyCompromised => {
                (StatusCode::UNAUTHORIZED, Json(ApiError::new("Refresh token revoked", "REFRESH_REVOKED")))
            }
            _ => {
                (StatusCode::UNAUTHORIZED, Json(ApiError::new("Invalid refresh token", "INVALID_REFRESH")))
            }
        })?;

    Ok(Json(RefreshResponse {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: result.expires_in,
    }))
}

/// Get current user profile
async fn get_current_user(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiError>)> {
    let user = state.auth_service
        .get_user(&auth_user.0.user_id)
        .await
        .map_err(|_| {
            (StatusCode::NOT_FOUND, Json(ApiError::new("User not found", "USER_NOT_FOUND")))
        })?;

    Ok(Json(user_to_response(&user)))
}

/// Change password
#[instrument(skip(state, auth_user, request))]
async fn change_password(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    state.auth_service
        .change_password(&auth_user.0.user_id, &request.current_password, &request.new_password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, Json(ApiError::new("Current password is incorrect", "INVALID_PASSWORD")))
            }
            AuthError::WeakPassword(msg) => {
                (StatusCode::BAD_REQUEST, Json(ApiError::new(&msg, "WEAK_PASSWORD")))
            }
            _ => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "PASSWORD_ERROR")))
            }
        })?;

    info!("Password changed for user {}", auth_user.0.user_id);
    Ok(StatusCode::NO_CONTENT)
}

/// List user sessions
async fn list_sessions(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
) -> Result<Json<Vec<SessionResponse>>, (StatusCode, Json<ApiError>)> {
    let sessions = state.session_store
        .list_for_user(&auth_user.0.user_id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "SESSION_ERROR")))
        })?;

    let current_session_id = auth_user.0.session_id.as_deref();

    let response: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id.clone(),
            device: s.user_agent,
            ip_address: s.ip_address,
            last_active: s.last_active_at.to_rfc3339(),
            created_at: s.created_at.to_rfc3339(),
            is_current: Some(s.id.as_str()) == current_session_id,
        })
        .collect();

    Ok(Json(response))
}

/// Revoke a specific session
async fn revoke_session(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
    Path(session_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    // Verify session belongs to user
    let session = state.session_store
        .get(&session_id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "SESSION_ERROR")))
        })?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(ApiError::new("Session not found", "SESSION_NOT_FOUND")))
        })?;

    if session.user_id != auth_user.0.user_id {
        return Err((StatusCode::FORBIDDEN, Json(ApiError::new("Cannot revoke other user's session", "FORBIDDEN"))));
    }

    state.session_store.delete(&session_id).await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "SESSION_ERROR")))
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Revoke all sessions except current
async fn revoke_all_sessions(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    let count = state.session_store
        .delete_all_for_user(&auth_user.0.user_id, auth_user.0.session_id.as_deref())
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "SESSION_ERROR")))
        })?;

    Ok(Json(serde_json::json!({
        "revoked": count
    })))
}

/// Delete user account
async fn delete_account(
    State(state): State<Arc<AuthApiState>>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    state.auth_service
        .delete_user(&auth_user.0.user_id)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::new(&e.to_string(), "DELETE_ERROR")))
        })?;

    info!("User {} deleted their account", auth_user.0.user_id);
    Ok(StatusCode::NO_CONTENT)
}

// === Helpers ===

fn user_to_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id.clone(),
        email: user.email.clone(),
        name: user.name.clone(),
        role: format!("{:?}", user.role).to_lowercase(),
        created_at: user.created_at.to_rfc3339(),
        avatar_url: user.avatar_url.clone(),
    }
}

fn create_session_cookie(
    config: &AuthApiConfig,
    token: &str,
    remember: bool,
) -> axum_extra::extract::cookie::Cookie<'static> {
    let mut builder = axum_extra::extract::cookie::Cookie::build((
        config.session_cookie_name.clone(),
        token.to_string(),
    ))
    .path("/")
    .http_only(config.session_cookie_http_only)
    .secure(config.session_cookie_secure);

    if remember {
        builder = builder.max_age(time::Duration::days(30));
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error() {
        let error = ApiError::new("Test error", "TEST_CODE");
        assert_eq!(error.error, "Test error");
        assert_eq!(error.code, "TEST_CODE");
    }
}
```

## Files to Create
- `src/auth/api.rs` - Authentication API handlers
