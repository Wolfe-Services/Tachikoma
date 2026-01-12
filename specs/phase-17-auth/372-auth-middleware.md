# Spec 372: Authentication Middleware

## Phase
17 - Authentication/Authorization

## Spec ID
372

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 369: Session Management
- Spec 370: JWT Tokens

## Estimated Context
~10%

---

## Objective

Implement authentication middleware for extracting and validating credentials from HTTP requests. The middleware should support multiple authentication methods (Bearer tokens, sessions, API keys), attach the authenticated identity to the request context, and provide clear error responses for authentication failures.

---

## Acceptance Criteria

- [ ] Create `AuthMiddleware` for HTTP request authentication
- [ ] Support Bearer token extraction from Authorization header
- [ ] Support session cookie extraction
- [ ] Support API key extraction from header or query
- [ ] Attach `AuthContext` to request extensions
- [ ] Provide configurable authentication requirements
- [ ] Return appropriate HTTP status codes for auth failures
- [ ] Support optional authentication (allow anonymous)
- [ ] Log authentication attempts appropriately

---

## Implementation Details

### Authentication Middleware

```rust
// src/auth/middleware.rs

use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header, request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use std::sync::Arc;
use tracing::{debug, warn, instrument};

use crate::auth::{
    api_keys::ApiKeyManager,
    config::{AuthConfig, SessionConfig},
    session::{Session, SessionManager},
    tokens::{extract_bearer_token, TokenManager},
    types::*,
};

/// Authentication middleware state
#[derive(Clone)]
pub struct AuthState {
    pub token_manager: Arc<TokenManager>,
    pub session_manager: Arc<SessionManager>,
    pub api_key_manager: Arc<ApiKeyManager>,
    pub config: AuthConfig,
}

impl AuthState {
    pub fn new(
        token_manager: Arc<TokenManager>,
        session_manager: Arc<SessionManager>,
        api_key_manager: Arc<ApiKeyManager>,
        config: AuthConfig,
    ) -> Self {
        Self {
            token_manager,
            session_manager,
            api_key_manager,
            config,
        }
    }
}

/// Authentication middleware
#[instrument(skip_all)]
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let metadata = extract_metadata(&request);

    // Try different authentication methods in order
    let auth_result = authenticate_request(&state, &request, &metadata).await;

    match auth_result {
        Ok(context) => {
            // Attach auth context to request extensions
            request.extensions_mut().insert(context);
            next.run(request).await
        }
        Err(e) => {
            // For middleware that allows anonymous, create anonymous context
            let context = AuthContext::anonymous(metadata);
            request.extensions_mut().insert(context);
            next.run(request).await
        }
    }
}

/// Strict authentication middleware (rejects anonymous)
#[instrument(skip_all)]
pub async fn require_auth_middleware(
    State(state): State<AuthState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let metadata = extract_metadata(&request);

    match authenticate_request(&state, &request, &metadata).await {
        Ok(context) => {
            request.extensions_mut().insert(context);
            next.run(request).await
        }
        Err(e) => auth_error_response(e),
    }
}

/// Try to authenticate the request using various methods
async fn authenticate_request(
    state: &AuthState,
    request: &Request<Body>,
    metadata: &AuthMetadata,
) -> AuthResult<AuthContext> {
    // Try Bearer token first
    if let Some(identity) = try_bearer_auth(state, request).await? {
        debug!(user_id = %identity.user_id, "Authenticated via Bearer token");
        return Ok(AuthContext::authenticated(identity, metadata.clone()));
    }

    // Try session cookie
    if let Some(identity) = try_session_auth(state, request).await? {
        debug!(user_id = %identity.user_id, "Authenticated via session");
        return Ok(AuthContext::authenticated(identity, metadata.clone()));
    }

    // Try API key
    if let Some(identity) = try_api_key_auth(state, request).await? {
        debug!(user_id = %identity.user_id, "Authenticated via API key");
        return Ok(AuthContext::authenticated(identity, metadata.clone()));
    }

    // No valid authentication found
    Err(AuthError::NotAuthenticated)
}

/// Try to authenticate using Bearer token
async fn try_bearer_auth(
    state: &AuthState,
    request: &Request<Body>,
) -> AuthResult<Option<AuthIdentity>> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) => {
            if let Some(token) = extract_bearer_token(header) {
                match state.token_manager.validate_access_token(token).await {
                    Ok(identity) => Ok(Some(identity)),
                    Err(e) => {
                        debug!(error = %e, "Bearer token validation failed");
                        Err(e)
                    }
                }
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

/// Try to authenticate using session cookie
async fn try_session_auth(
    state: &AuthState,
    request: &Request<Body>,
) -> AuthResult<Option<AuthIdentity>> {
    let cookie_header = request
        .headers()
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok());

    if let Some(cookies) = cookie_header {
        let session_cookie_name = &state.config.session.cookie_name;

        // Parse cookies to find session ID
        if let Some(session_id) = parse_cookie(cookies, session_cookie_name) {
            if let Ok(sid) = session_id.parse::<uuid::Uuid>() {
                let sid = SessionId::from(sid);

                match state.session_manager.validate_session(sid).await {
                    Ok(session) => {
                        // Convert session to identity
                        let identity = session_to_identity(&session);
                        Ok(Some(identity))
                    }
                    Err(e) => {
                        debug!(error = %e, "Session validation failed");
                        Ok(None)
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Try to authenticate using API key
async fn try_api_key_auth(
    state: &AuthState,
    request: &Request<Body>,
) -> AuthResult<Option<AuthIdentity>> {
    if !state.config.api_keys.enabled {
        return Ok(None);
    }

    let header_name = &state.config.api_keys.header_name;

    // Check header first
    let api_key = request
        .headers()
        .get(header_name)
        .and_then(|h| h.to_str().ok());

    // Check query parameter if allowed and header not found
    let api_key = match api_key {
        Some(k) => Some(k.to_string()),
        None => {
            if let Some(ref param_name) = state.config.api_keys.query_param_name {
                extract_query_param(request.uri().query(), param_name)
            } else {
                None
            }
        }
    };

    if let Some(key) = api_key {
        match state.api_key_manager.validate_key(&key).await {
            Ok(identity) => Ok(Some(identity)),
            Err(e) => {
                debug!(error = %e, "API key validation failed");
                Err(e)
            }
        }
    } else {
        Ok(None)
    }
}

/// Extract metadata from request
fn extract_metadata(request: &Request<Body>) -> AuthMetadata {
    let ip_address = request
        .headers()
        .get("X-Forwarded-For")
        .or_else(|| request.headers().get("X-Real-IP"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    let request_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .map(String::from);

    AuthMetadata {
        ip_address,
        user_agent,
        request_id,
        geo_location: None,
    }
}

/// Parse a specific cookie value from cookie header
fn parse_cookie(cookies: &str, name: &str) -> Option<String> {
    cookies
        .split(';')
        .map(|c| c.trim())
        .find(|c| c.starts_with(&format!("{}=", name)))
        .map(|c| c.splitn(2, '=').nth(1).unwrap_or("").to_string())
}

/// Extract query parameter from query string
fn extract_query_param(query: Option<&str>, name: &str) -> Option<String> {
    query.and_then(|q| {
        q.split('&')
            .find(|p| p.starts_with(&format!("{}=", name)))
            .map(|p| p.splitn(2, '=').nth(1).unwrap_or("").to_string())
    })
}

/// Convert session to identity
fn session_to_identity(session: &Session) -> AuthIdentity {
    AuthIdentity {
        user_id: session.user_id,
        username: session.get_data::<String>("username").unwrap_or_default(),
        display_name: session.get_data("display_name"),
        email: session.get_data("email"),
        email_verified: session.get_data("email_verified").unwrap_or(false),
        roles: session.get_data("roles").unwrap_or_default(),
        permissions: session.get_data("permissions").unwrap_or_default(),
        auth_method: AuthMethod::Session,
        authenticated_at: session.created_at,
        session_id: Some(session.id),
        claims: serde_json::Value::Null,
    }
}

/// Convert auth error to HTTP response
fn auth_error_response(error: AuthError) -> Response {
    let (status, message) = match error {
        AuthError::NotAuthenticated => (
            StatusCode::UNAUTHORIZED,
            "Authentication required",
        ),
        AuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            "Invalid credentials",
        ),
        AuthError::TokenExpired => (
            StatusCode::UNAUTHORIZED,
            "Token expired",
        ),
        AuthError::TokenInvalid(_) => (
            StatusCode::UNAUTHORIZED,
            "Invalid token",
        ),
        AuthError::SessionExpired => (
            StatusCode::UNAUTHORIZED,
            "Session expired",
        ),
        AuthError::SessionInvalid => (
            StatusCode::UNAUTHORIZED,
            "Invalid session",
        ),
        AuthError::AccountLocked => (
            StatusCode::FORBIDDEN,
            "Account locked",
        ),
        AuthError::AccountDisabled => (
            StatusCode::FORBIDDEN,
            "Account disabled",
        ),
        AuthError::InsufficientPermissions => (
            StatusCode::FORBIDDEN,
            "Insufficient permissions",
        ),
        AuthError::RateLimitExceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded",
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Authentication error",
        ),
    };

    let body = serde_json::json!({
        "error": message,
        "code": status.as_u16(),
    });

    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// Extractor for AuthContext from request
#[derive(Debug, Clone)]
pub struct Auth(pub AuthContext);

impl Auth {
    /// Get the authenticated identity, or error if not authenticated
    pub fn require(&self) -> Result<&AuthIdentity, AuthError> {
        self.0.require_auth()
    }

    /// Get the user ID if authenticated
    pub fn user_id(&self) -> Option<UserId> {
        self.0.user_id()
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.0.is_authenticated()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .map(Auth)
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Auth middleware not configured",
            ))
    }
}

/// Extractor that requires authentication
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthIdentity);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))?;

        context
            .identity
            .clone()
            .map(RequireAuth)
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))
    }
}

/// Middleware layer builder
pub struct AuthLayer {
    state: AuthState,
    require_auth: bool,
}

impl AuthLayer {
    pub fn new(state: AuthState) -> Self {
        Self {
            state,
            require_auth: false,
        }
    }

    pub fn require_auth(mut self) -> Self {
        self.require_auth = true;
        self
    }
}

/// Router extension for adding auth middleware
pub trait AuthRouterExt {
    fn with_auth(self, state: AuthState) -> Self;
    fn require_auth(self, state: AuthState) -> Self;
}

impl AuthRouterExt for axum::Router {
    fn with_auth(self, state: AuthState) -> Self {
        use axum::middleware;
        self.layer(middleware::from_fn_with_state(state, auth_middleware))
    }

    fn require_auth(self, state: AuthState) -> Self {
        use axum::middleware;
        self.layer(middleware::from_fn_with_state(state, require_auth_middleware))
    }
}

/// Example usage with handlers
#[cfg(feature = "example")]
mod example {
    use super::*;
    use axum::{routing::get, Json, Router};

    async fn public_handler(Auth(auth): Auth) -> impl IntoResponse {
        if let Some(user_id) = auth.user_id() {
            format!("Hello, user {}", user_id)
        } else {
            "Hello, anonymous user".to_string()
        }
    }

    async fn protected_handler(RequireAuth(identity): RequireAuth) -> impl IntoResponse {
        format!("Hello, {}", identity.username)
    }

    async fn admin_handler(RequireAuth(identity): RequireAuth) -> Result<impl IntoResponse, Response> {
        if !identity.has_role("admin") {
            return Err(auth_error_response(AuthError::InsufficientPermissions));
        }
        Ok(format!("Admin area for {}", identity.username))
    }

    fn create_router(state: AuthState) -> Router {
        Router::new()
            .route("/public", get(public_handler))
            .route("/protected", get(protected_handler))
            .route("/admin", get(admin_handler))
            .with_auth(state)
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
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler(Auth(auth): Auth) -> String {
        if auth.is_authenticated() {
            format!("authenticated:{}", auth.user_id().unwrap())
        } else {
            "anonymous".to_string()
        }
    }

    async fn protected_handler(RequireAuth(identity): RequireAuth) -> String {
        format!("user:{}", identity.user_id)
    }

    fn create_test_state() -> AuthState {
        let mut token_config = TokenConfig::default();
        token_config.secret_key = "test-secret-32-bytes-minimum-len".to_string();

        AuthState {
            token_manager: Arc::new(TokenManager::new_hmac(token_config.clone()).unwrap()),
            session_manager: Arc::new(SessionManager::new(
                Arc::new(InMemorySessionStorage::new()),
                SessionConfig::default(),
                Arc::new(NoOpEventEmitter),
            )),
            api_key_manager: Arc::new(ApiKeyManager::new(
                Arc::new(InMemoryApiKeyStorage::new()),
                Arc::new(MockUserRepository),
                ApiKeyConfig::default(),
            )),
            config: AuthConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_anonymous_access() {
        let state = create_test_state();
        let app = Router::new()
            .route("/test", get(test_handler))
            .with_auth(state);

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"anonymous");
    }

    #[tokio::test]
    async fn test_bearer_token_auth() {
        let state = create_test_state();
        let app = Router::new()
            .route("/test", get(test_handler))
            .with_auth(state.clone());

        // Create a valid token
        let identity = AuthIdentity {
            user_id: UserId::new(),
            username: "testuser".to_string(),
            display_name: None,
            email: None,
            email_verified: false,
            roles: HashSet::new(),
            permissions: HashSet::new(),
            auth_method: AuthMethod::Password,
            authenticated_at: Utc::now(),
            session_id: None,
            claims: serde_json::Value::Null,
        };
        let token = state.token_manager.create_access_token(&identity).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.starts_with("authenticated:"));
    }

    #[tokio::test]
    async fn test_protected_route_requires_auth() {
        let state = create_test_state();
        let app = Router::new()
            .route("/protected", get(protected_handler))
            .require_auth(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let state = create_test_state();
        let app = Router::new()
            .route("/protected", get(protected_handler))
            .require_auth(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer invalid.token.here")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_metadata() {
        let request = Request::builder()
            .uri("/test")
            .header("X-Forwarded-For", "192.168.1.1, 10.0.0.1")
            .header("User-Agent", "Test Agent/1.0")
            .header("X-Request-ID", "req-123")
            .body(Body::empty())
            .unwrap();

        let metadata = extract_metadata(&request);

        assert_eq!(metadata.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(metadata.user_agent, Some("Test Agent/1.0".to_string()));
        assert_eq!(metadata.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_parse_cookie() {
        let cookies = "session=abc123; other=value; third=123";
        assert_eq!(parse_cookie(cookies, "session"), Some("abc123".to_string()));
        assert_eq!(parse_cookie(cookies, "other"), Some("value".to_string()));
        assert_eq!(parse_cookie(cookies, "missing"), None);
    }

    #[test]
    fn test_extract_query_param() {
        assert_eq!(
            extract_query_param(Some("api_key=abc123&other=value"), "api_key"),
            Some("abc123".to_string())
        );
        assert_eq!(
            extract_query_param(Some("api_key=abc123"), "missing"),
            None
        );
        assert_eq!(extract_query_param(None, "api_key"), None);
    }

    struct NoOpEventEmitter;
    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthContext and AuthIdentity
- **Spec 369**: Session Management - Validates sessions
- **Spec 370**: JWT Tokens - Validates bearer tokens
- **Spec 373**: Auth Guards - Uses middleware for route protection
- **Spec 376**: API Keys - Validates API keys
