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