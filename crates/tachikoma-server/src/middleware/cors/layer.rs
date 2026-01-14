//! CORS middleware layer.

use super::config::CorsConfig;
use axum::{
    body::Body,
    http::{header, Method, Request, Response, StatusCode},
};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// CORS middleware layer.
#[derive(Clone)]
pub struct CorsLayer {
    config: CorsConfig,
}

impl CorsLayer {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    pub fn permissive() -> Self {
        Self::new(CorsConfig::permissive())
    }
}

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CorsMiddleware {
            inner,
            config: self.config.clone(),
        }
    }
}

#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
    config: CorsConfig,
}

impl<S> Service<Request<Body>> for CorsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get origin from request
            let origin = req
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            // Handle preflight request
            if req.method() == Method::OPTIONS {
                return Ok(handle_preflight(&config, origin.as_deref()));
            }

            // Call inner service
            let mut response = inner.call(req).await?;

            // Add CORS headers to response
            add_cors_headers(&mut response, &config, origin.as_deref());

            Ok(response)
        })
    }
}

fn handle_preflight(config: &CorsConfig, origin: Option<&str>) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NO_CONTENT;

    if let Some(origin) = origin {
        if config.is_origin_allowed(origin) {
            add_cors_headers(&mut response, config, Some(origin));

            // Add preflight-specific headers
            let headers = response.headers_mut();

            // Allowed methods
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                config.allowed_methods
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
                    .parse()
                    .unwrap(),
            );

            // Allowed headers
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                config.allowed_headers.to_header_value().parse().unwrap(),
            );

            // Max age
            if let Some(max_age) = config.max_age {
                headers.insert(
                    header::ACCESS_CONTROL_MAX_AGE,
                    max_age.as_secs().to_string().parse().unwrap(),
                );
            }
        }
    }

    response
}

fn add_cors_headers(response: &mut Response<Body>, config: &CorsConfig, origin: Option<&str>) {
    let headers = response.headers_mut();

    // Set origin header
    if let Some(origin) = origin {
        if config.is_origin_allowed(origin) {
            match &config.allowed_origins {
                super::config::AllowedOrigins::Any => {
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
                }
                _ => {
                    headers.insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        origin.parse().unwrap(),
                    );
                    // Vary header for caching
                    headers.insert(header::VARY, "Origin".parse().unwrap());
                }
            }
        }
    }

    // Credentials
    if config.allow_credentials {
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "true".parse().unwrap(),
        );
    }

    // Exposed headers
    if !config.exposed_headers.is_empty() {
        headers.insert(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            config.exposed_headers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
                .parse()
                .unwrap(),
        );
    }
}