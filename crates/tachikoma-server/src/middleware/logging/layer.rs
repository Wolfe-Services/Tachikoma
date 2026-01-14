//! Request logging middleware.

use axum::{
    body::Body,
    extract::Request,
    http::Response,
};
use std::{sync::Arc, task::{Context, Poll}, time::Instant};
use tower::{Layer, Service};
use tracing::{info, span, Level, Span};
use uuid::Uuid;
use futures::future::BoxFuture;

/// Request logging layer.
#[derive(Clone, Default)]
pub struct LoggingLayer {
    config: LoggingConfig,
}

#[derive(Clone, Default)]
pub struct LoggingConfig {
    /// Log request bodies (careful with size).
    pub log_bodies: bool,
    /// Log response bodies.
    pub log_response_bodies: bool,
    /// Paths to exclude from logging.
    pub exclude_paths: Vec<String>,
    /// Headers to redact.
    pub redact_headers: Vec<String>,
}

impl LoggingLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: LoggingConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct LoggingMiddleware<S> {
    inner: S,
    config: LoggingConfig,
}

impl<S> Service<Request> for LoggingMiddleware<S>
where
    S: Service<Request, Response = Response<Body>, Error = std::convert::Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check if path should be excluded
            let path = req.uri().path().to_string();
            if config.exclude_paths.iter().any(|p| path.starts_with(p)) {
                return inner.call(req).await;
            }

            // Extract request info
            let method = req.method().clone();
            let uri = req.uri().clone();
            let version = req.version();

            // Get or generate request ID
            let request_id = req
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            // Get client IP
            let client_ip = req
                .headers()
                .get("x-forwarded-for")
                .or_else(|| req.headers().get("x-real-ip"))
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(',').next())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Get user agent
            let user_agent = req
                .headers()
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(String::from)
                .unwrap_or_default();

            let start = Instant::now();

            // Create request span
            let span = span!(
                Level::INFO,
                "request",
                request_id = %request_id,
                method = %method,
                path = %uri.path(),
                client_ip = %client_ip,
            );

            let _enter = span.enter();

            // Log request
            info!(
                event = "request_started",
                method = %method,
                uri = %uri,
                version = ?version,
                user_agent = %user_agent,
            );

            // Call inner service
            let response = inner.call(req).await?;

            // Calculate duration
            let duration = start.elapsed();
            let status = response.status();

            // Log response
            info!(
                event = "request_completed",
                status = %status.as_u16(),
                duration_ms = duration.as_millis() as u64,
            );

            Ok(response)
        })
    }
}