//! Authentication middleware layer.

use super::{
    jwt::decode_token,
    types::{AuthUser, Claims, TokenType},
};
use crate::error::ApiError;
use axum::{
    body::Body,
    http::{header, Request},
    response::Response,
};
use std::sync::Arc;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;

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

impl<S> Layer<S> for AuthLayer {
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

impl<S> Service<Request<Body>> for AuthMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Response> + Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let jwt_secret = self.jwt_secret.clone();
        let allow_api_keys = self.allow_api_keys;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract token from request
            match extract_token(&req) {
                Ok(token) => {
                    // Decode and validate token
                    match decode_token(&token, &jwt_secret) {
                        Ok(claims) => {
                            // Verify token type and not expired
                            if claims.token_type != TokenType::Access {
                                let error_response = ApiError::InvalidToken.into();
                                return Ok(error_response);
                            }

                            if claims.is_expired() {
                                let error_response = ApiError::TokenExpired.into();
                                return Ok(error_response);
                            }

                            // Create auth user from claims
                            if let Some(auth_user) = AuthUser::from_claims(claims) {
                                // Insert auth user into request extensions
                                req.extensions_mut().insert(auth_user);
                            } else {
                                let error_response = ApiError::InvalidToken.into();
                                return Ok(error_response);
                            }
                        }
                        Err(_) => {
                            let error_response = ApiError::InvalidToken.into();
                            return Ok(error_response);
                        }
                    }
                }
                Err(err) => {
                    let error_response: Response = err.into();
                    return Ok(error_response);
                }
            }

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use uuid::Uuid;

    #[tokio::test]
    async fn test_extract_token_from_bearer_header() {
        let mut req = Request::builder()
            .header("Authorization", "Bearer test_token")
            .body(Body::empty())
            .unwrap();

        let token = extract_token(&req).unwrap();
        assert_eq!(token, "test_token");
    }

    #[tokio::test]
    async fn test_extract_token_from_cookie() {
        let req = Request::builder()
            .header("Cookie", "access_token=test_token; other=value")
            .body(Body::empty())
            .unwrap();

        let token = extract_token(&req).unwrap();
        assert_eq!(token, "test_token");
    }

    #[tokio::test] 
    async fn test_extract_token_missing() {
        let req = Request::builder()
            .body(Body::empty())
            .unwrap();

        let result = extract_token(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }
}