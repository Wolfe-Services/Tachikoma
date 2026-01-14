# 322 - Auth Middleware

**Phase:** 15 - Server
**Spec ID:** 322
**Status:** Planned
**Dependencies:** 321-error-response
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement JWT-based authentication middleware that validates tokens, extracts user claims, and provides user context to handlers.

---

## Acceptance Criteria

- [x] JWT validation middleware
- [x] Token extraction from headers/cookies
- [x] User claims extraction
- [x] Token refresh logic
- [x] API key authentication support
- [x] Request context injection
- [x] Token blacklist checking

---

## Implementation Details

### 1. Auth Types (crates/tachikoma-server/src/middleware/auth/types.rs)

```rust
//! Authentication types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// User email.
    pub email: String,
    /// User roles.
    pub roles: Vec<String>,
    /// Token type (access/refresh).
    pub token_type: TokenType,
    /// Issued at timestamp.
    pub iat: i64,
    /// Expiration timestamp.
    pub exp: i64,
    /// JWT ID (for revocation).
    pub jti: String,
}

/// Token type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

impl Claims {
    /// Create new access token claims.
    pub fn new_access(user_id: Uuid, email: &str, roles: Vec<String>, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles,
            token_type: TokenType::Access,
            iat: now,
            exp: now + expires_in,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Create new refresh token claims.
    pub fn new_refresh(user_id: Uuid, expires_in: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id.to_string(),
            email: String::new(),
            roles: Vec::new(),
            token_type: TokenType::Refresh,
            iat: now,
            exp: now + expires_in,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Get user ID as UUID.
    pub fn user_id(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.sub).ok()
    }

    /// Check if token is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Check if user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// Authenticated user context.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub claims: Claims,
}

impl AuthUser {
    /// Create from claims.
    pub fn from_claims(claims: Claims) -> Option<Self> {
        let id = claims.user_id()?;
        Some(Self {
            id,
            email: claims.email.clone(),
            roles: claims.roles.clone(),
            claims,
        })
    }

    /// Check if user has admin role.
    pub fn is_admin(&self) -> bool {
        self.roles.contains(&"admin".to_string())
    }
}

/// API key authentication info.
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub key_id: String,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}
```

### 2. Auth Layer (crates/tachikoma-server/src/middleware/auth/layer.rs)

```rust
//! Authentication middleware layer.

use super::{
    jwt::{decode_token, TokenDecoder},
    types::{AuthUser, Claims, TokenType},
};
use crate::error::ApiError;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// Authentication layer configuration.
#[derive(Clone)]
pub struct AuthLayer {
    jwt_secret: Arc<String>,
    allow_api_keys: bool,
}

impl AuthLayer {
    /// Create new auth layer.
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret: Arc::new(jwt_secret),
            allow_api_keys: false,
        }
    }

    /// Enable API key authentication.
    pub fn with_api_keys(mut self) -> Self {
        self.allow_api_keys = true;
        self
    }
}

impl<S> tower::Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            jwt_secret: self.jwt_secret.clone(),
            allow_api_keys: self.allow_api_keys,
        }
    }
}

/// Authentication middleware service.
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    jwt_secret: Arc<String>,
    allow_api_keys: bool,
}

impl<S> tower::Service<Request<Body>> for AuthMiddleware<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let jwt_secret = self.jwt_secret.clone();
        let allow_api_keys = self.allow_api_keys;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract token from request
            let token = extract_token(&req)?;

            // Decode and validate token
            let claims = decode_token(&token, &jwt_secret)
                .map_err(|_| ApiError::InvalidToken)?;

            // Verify token type
            if claims.token_type != TokenType::Access {
                return Err(ApiError::InvalidToken.into());
            }

            // Create auth user from claims
            let auth_user = AuthUser::from_claims(claims)
                .ok_or(ApiError::InvalidToken)?;

            // Insert auth user into request extensions
            req.extensions_mut().insert(auth_user);

            // Continue to handler
            inner.call(req).await
        })
    }
}

fn extract_token(req: &Request<Body>) -> Result<String, ApiError> {
    // Try Authorization header first
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        let auth_str = auth_header.to_str().map_err(|_| ApiError::InvalidToken)?;

        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            return Ok(token.to_string());
        }
    }

    // Try cookie as fallback
    if let Some(cookie_header) = req.headers().get(header::COOKIE) {
        let cookie_str = cookie_header.to_str().map_err(|_| ApiError::InvalidToken)?;

        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("access_token=") {
                return Ok(token.to_string());
            }
        }
    }

    Err(ApiError::Unauthorized)
}
```

### 3. JWT Utilities (crates/tachikoma-server/src/middleware/auth/jwt.rs)

```rust
//! JWT encoding and decoding utilities.

use super::types::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

/// Encode claims into a JWT token.
pub fn encode_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Decode and validate a JWT token.
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// Token decoder for use in extractors.
pub struct TokenDecoder {
    secret: String,
}

impl TokenDecoder {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn decode(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode_token(token, &self.secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_encode_decode_roundtrip() {
        let secret = "test_secret_key_32_chars_long!!";
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "test@example.com",
            vec!["user".into()],
            3600,
        );

        let token = encode_token(&claims, secret).unwrap();
        let decoded = decode_token(&token, secret).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
    }
}
```

### 4. Auth Extractor (crates/tachikoma-server/src/middleware/auth/extractor.rs)

```rust
//! Authentication extractors for handlers.

use super::types::AuthUser;
use crate::error::ApiError;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};

/// Extractor for authenticated user (required).
pub struct Auth(pub AuthUser);

#[async_trait]
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(Auth)
            .ok_or(ApiError::Unauthorized)
    }
}

/// Extractor for optional authenticated user.
pub struct MaybeAuth(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(MaybeAuth(parts.extensions.get::<AuthUser>().cloned()))
    }
}

/// Extractor that requires admin role.
pub struct AdminAuth(pub AuthUser);

#[async_trait]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(ApiError::Unauthorized)?;

        if user.is_admin() {
            Ok(AdminAuth(user))
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

/// Extractor that requires specific role.
pub struct RequireRole<const ROLE: &'static str>(pub AuthUser);

#[async_trait]
impl<S, const ROLE: &'static str> FromRequestParts<S> for RequireRole<ROLE>
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(ApiError::Unauthorized)?;

        if user.roles.contains(&ROLE.to_string()) {
            Ok(RequireRole(user))
        } else {
            Err(ApiError::InsufficientPermissions)
        }
    }
}
```

---

## Testing Requirements

1. Valid JWT passes authentication
2. Expired JWT returns 401
3. Invalid JWT returns 401
4. Missing token returns 401
5. Claims extracted correctly
6. Role-based extractors work
7. Cookie fallback works

---

## Related Specs

- Depends on: [321-error-response.md](321-error-response.md)
- Next: [323-authz-middleware.md](323-authz-middleware.md)
- Used by: All protected endpoints
