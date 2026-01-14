//! Rate limit middleware layer.

use super::{
    store::{InMemoryStore, RateLimitStore},
    types::{KeyStrategy, RateLimitConfig},
};
use crate::{error::types::ApiError, middleware::auth::types::AuthUser};
use axum::{
    body::Body,
    extract::Request,
    http::{header, Response},
    response::IntoResponse,
};
use std::{sync::Arc, task::{Context, Poll}, time::Duration};
use tower::{Layer, Service};
use futures::future::BoxFuture;

/// Rate limit layer.
#[derive(Clone)]
pub struct RateLimitLayer {
    store: Arc<dyn RateLimitStore>,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            store: Arc::new(InMemoryStore::new()),
            config: RateLimitConfig {
                max_requests,
                window,
                burst: 0,
                key_strategy: KeyStrategy::Ip,
            },
        }
    }

    pub fn with_store(mut self, store: Arc<dyn RateLimitStore>) -> Self {
        self.store = store;
        self
    }

    pub fn with_config(mut self, config: RateLimitConfig) -> Self {
        self.config = config;
        self
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            store: self.store.clone(),
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    store: Arc<dyn RateLimitStore>,
    config: RateLimitConfig,
}

impl<S> Service<Request> for RateLimitMiddleware<S>
where
    S: Service<Request, Response = Response<Body>, Error = std::convert::Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let store = self.store.clone();
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract rate limit key
            let key = extract_key(&req, &config.key_strategy);

            // Check rate limit
            let result = store.check_and_consume(&key, &config).await;

            if !result.allowed {
                let retry_after = result.retry_after
                    .map(|d| d.as_secs())
                    .unwrap_or(1);

                let error = ApiError::RateLimited { retry_after };
                let mut response = error.into_response();

                // Add rate limit headers to error response
                let headers = response.headers_mut();
                add_rate_limit_headers(headers, &result);

                return Ok(response);
            }

            // Call inner service
            let mut response = inner.call(req).await?;

            // Add rate limit headers to successful response
            let headers = response.headers_mut();
            add_rate_limit_headers(headers, &result);

            Ok(response)
        })
    }
}

fn add_rate_limit_headers(headers: &mut axum::http::HeaderMap, result: &super::store::RateLimitResult) {
    if let Ok(value) = result.limit.to_string().parse() {
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-limit"),
            value,
        );
    }
    
    if let Ok(value) = result.remaining.to_string().parse() {
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-remaining"),
            value,
        );
    }

    let reset_seconds = result.reset_at.elapsed().as_secs();
    if let Ok(value) = reset_seconds.to_string().parse() {
        headers.insert(
            header::HeaderName::from_static("x-ratelimit-reset"),
            value,
        );
    }

    if let Some(retry_after) = result.retry_after {
        if let Ok(value) = retry_after.as_secs().to_string().parse() {
            headers.insert(
                header::HeaderName::from_static("retry-after"),
                value,
            );
        }
    }
}

fn extract_key(req: &Request, strategy: &KeyStrategy) -> String {
    match strategy {
        KeyStrategy::Ip => {
            // Try X-Forwarded-For, then X-Real-IP, then connection IP
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
                .or_else(|| {
                    req.headers()
                        .get("x-real-ip")
                        .and_then(|v| v.to_str().ok())
                        .map(String::from)
                })
                .unwrap_or_else(|| "unknown".to_string())
        }
        KeyStrategy::User => {
            req.extensions()
                .get::<AuthUser>()
                .map(|u| format!("user:{}", u.id))
                .unwrap_or_else(|| "anonymous".to_string())
        }
        KeyStrategy::ApiKey => {
            req.headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(|k| format!("apikey:{}", k))
                .unwrap_or_else(|| "no-key".to_string())
        }
        KeyStrategy::Composite => {
            let ip = extract_key(req, &KeyStrategy::Ip);
            let user = req.extensions()
                .get::<AuthUser>()
                .map(|u| u.id.to_string());

            match user {
                Some(user_id) => format!("{}:{}", ip, user_id),
                None => ip,
            }
        }
    }
}