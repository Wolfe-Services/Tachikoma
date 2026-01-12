# 318 - API Versioning

**Phase:** 15 - Server
**Spec ID:** 318
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement API versioning strategy supporting URL path versioning, header-based versioning, and graceful deprecation of older API versions.

---

## Acceptance Criteria

- [ ] URL path versioning (/api/v1, /api/v2)
- [ ] Accept-Version header support
- [ ] Version negotiation logic
- [ ] Deprecation headers
- [ ] Version-specific route registration
- [ ] Migration guides generation
- [ ] Backwards compatibility layer

---

## Implementation Details

### 1. Version Types (crates/tachikoma-server/src/versioning/types.rs)

```rust
//! API version types and utilities.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::HeaderValue, request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported API versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    /// Latest stable API version.
    pub const LATEST: Self = Self::V1;

    /// All supported versions in order.
    pub const ALL: &'static [Self] = &[Self::V1, Self::V2];

    /// Check if this version is deprecated.
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::V1)
    }

    /// Get deprecation date if applicable.
    pub fn deprecation_date(&self) -> Option<&'static str> {
        match self {
            Self::V1 => Some("2025-06-01"),
            Self::V2 => None,
        }
    }

    /// Get sunset date (when version will be removed).
    pub fn sunset_date(&self) -> Option<&'static str> {
        match self {
            Self::V1 => Some("2025-12-01"),
            Self::V2 => None,
        }
    }

    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "v1" | "1" | "1.0" => Some(Self::V1),
            "v2" | "2" | "2.0" => Some(Self::V2),
            _ => None,
        }
    }

    /// Get version string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V2 => "v2",
        }
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::LATEST
    }
}

/// Version extractor from request.
#[derive(Debug, Clone, Copy)]
pub struct RequestVersion(pub ApiVersion);

#[async_trait]
impl<S> FromRequestParts<S> for RequestVersion
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // First check Accept-Version header
        if let Some(header) = parts.headers.get("accept-version") {
            if let Ok(value) = header.to_str() {
                if let Some(version) = ApiVersion::parse(value) {
                    return Ok(RequestVersion(version));
                }
            }
        }

        // Fall back to path-based version extraction
        // This assumes the path includes /api/vX/
        let path = parts.uri.path();
        for version in ApiVersion::ALL {
            if path.contains(&format!("/api/{}/", version.as_str())) {
                return Ok(RequestVersion(*version));
            }
        }

        // Default to latest version
        Ok(RequestVersion(ApiVersion::LATEST))
    }
}
```

### 2. Version Middleware (crates/tachikoma-server/src/versioning/middleware.rs)

```rust
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
```

### 3. Version Router Builder (crates/tachikoma-server/src/versioning/router.rs)

```rust
//! Versioned router construction utilities.

use axum::Router;
use super::types::ApiVersion;
use crate::state::AppState;

/// Builder for constructing versioned API routers.
pub struct VersionedRouterBuilder {
    routes: Vec<(ApiVersion, Router<AppState>)>,
}

impl VersionedRouterBuilder {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Add routes for a specific version.
    pub fn version(mut self, version: ApiVersion, router: Router<AppState>) -> Self {
        self.routes.push((version, router));
        self
    }

    /// Build the final router with all versions nested.
    pub fn build(self) -> Router<AppState> {
        let mut root = Router::new();

        for (version, router) in self.routes {
            root = root.nest(&format!("/api/{}", version.as_str()), router);
        }

        root
    }
}

impl Default for VersionedRouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for defining version-specific handlers.
#[macro_export]
macro_rules! version_handler {
    ($name:ident, $v1:expr, $v2:expr) => {
        pub async fn $name(
            version: $crate::versioning::RequestVersion,
            // other extractors...
        ) -> impl axum::response::IntoResponse {
            match version.0 {
                $crate::versioning::ApiVersion::V1 => $v1,
                $crate::versioning::ApiVersion::V2 => $v2,
            }
        }
    };
}
```

### 4. Version Negotiation (crates/tachikoma-server/src/versioning/negotiation.rs)

```rust
//! Version negotiation logic.

use super::types::ApiVersion;

/// Negotiate the best API version based on client preferences.
pub fn negotiate_version(
    accept_header: Option<&str>,
    supported: &[ApiVersion],
) -> ApiVersion {
    if let Some(accept) = accept_header {
        // Parse Accept-Version header (e.g., "v2, v1;q=0.5")
        let preferences = parse_version_preferences(accept);

        for (version, _weight) in preferences {
            if supported.contains(&version) {
                return version;
            }
        }
    }

    // Return latest supported version
    supported.iter().max().copied().unwrap_or(ApiVersion::LATEST)
}

fn parse_version_preferences(header: &str) -> Vec<(ApiVersion, f32)> {
    let mut preferences: Vec<(ApiVersion, f32)> = header
        .split(',')
        .filter_map(|part| {
            let parts: Vec<&str> = part.trim().split(';').collect();
            let version_str = parts.first()?.trim();
            let version = ApiVersion::parse(version_str)?;

            let weight = parts
                .get(1)
                .and_then(|q| {
                    q.trim()
                        .strip_prefix("q=")
                        .and_then(|w| w.parse::<f32>().ok())
                })
                .unwrap_or(1.0);

            Some((version, weight))
        })
        .collect();

    // Sort by weight descending
    preferences.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    preferences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_version_simple() {
        let result = negotiate_version(
            Some("v2"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2);
    }

    #[test]
    fn test_negotiate_version_weighted() {
        let result = negotiate_version(
            Some("v1;q=0.5, v2;q=0.9"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2);
    }

    #[test]
    fn test_negotiate_version_unsupported() {
        let result = negotiate_version(
            Some("v3"),
            &[ApiVersion::V1, ApiVersion::V2],
        );
        assert_eq!(result, ApiVersion::V2); // Latest supported
    }
}
```

---

## Testing Requirements

1. Version parsing works for all formats
2. Header-based version detection works
3. Path-based version detection works
4. Deprecation headers added correctly
5. Version negotiation follows weights
6. Default to latest when unspecified
7. Sunset headers included for deprecated

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [319-request-response.md](319-request-response.md)
- Used by: All API endpoints
