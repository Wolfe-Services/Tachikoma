//! Versioning middleware for adding deprecation headers.

use axum::{
    body::Body,
    http::{header::HeaderName, Request, Response},
    middleware::Next,
};
use super::types::ApiVersion;

/// Header names for version information.
pub const DEPRECATION_HEADER: &str = "deprecation";
pub const SUNSET_HEADER: &str = "sunset";
pub const API_VERSION_HEADER: &str = "x-api-version";

/// Middleware that adds version-related headers to responses.
pub async fn version_headers_middleware(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Extract version from request path
    let path = request.uri().path();
    let version = extract_version_from_path(path);

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Add current API version header
    if let Some(v) = version {
        headers.insert(
            HeaderName::from_static(API_VERSION_HEADER),
            v.as_str().parse().unwrap(),
        );

        // Add deprecation headers if applicable
        if v.is_deprecated() {
            if let Some(date) = v.deprecation_date() {
                headers.insert(
                    HeaderName::from_static(DEPRECATION_HEADER),
                    format!("@{}", date).parse().unwrap(),
                );
            }

            if let Some(date) = v.sunset_date() {
                headers.insert(
                    HeaderName::from_static(SUNSET_HEADER),
                    date.parse().unwrap(),
                );
            }
        }
    }

    response
}

fn extract_version_from_path(path: &str) -> Option<ApiVersion> {
    for version in ApiVersion::ALL {
        if path.contains(&format!("/api/{}/", version.as_str())) {
            return Some(*version);
        }
    }
    None
}