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