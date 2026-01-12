# 317 - Axum Router

**Phase:** 15 - Server
**Spec ID:** 317
**Status:** Planned
**Dependencies:** 316-server-crate
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create the Axum router configuration with nested route groups, middleware stacks, and proper separation of API versions and internal endpoints.

---

## Acceptance Criteria

- [ ] Main router construction function
- [ ] API v1 route group
- [ ] Internal routes (health, metrics)
- [ ] Middleware application order
- [ ] Nested routers for resources
- [ ] Fallback handler for 404
- [ ] OpenAPI/Swagger integration ready

---

## Implementation Details

### 1. Routes Module (crates/tachikoma-server/src/routes/mod.rs)

```rust
//! Route configuration for the Tachikoma API server.

mod internal;
mod v1;

use crate::{
    middleware::{auth::AuthLayer, cors::cors_layer, logging::LoggingLayer, rate_limit::RateLimitLayer},
    state::AppState,
};
use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
};
use std::time::Duration;

/// Create the main application router.
pub fn create_router(state: AppState) -> Router {
    // Common middleware stack applied to all routes
    let common_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(
            axum::http::HeaderName::from_static("x-request-id"),
        ))
        .layer(CatchPanicLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
        .layer(LoggingLayer);

    Router::new()
        // API routes
        .nest("/api/v1", v1::router(state.clone()))
        // Internal routes (health, metrics, etc.)
        .nest("/internal", internal::router(state.clone()))
        // WebSocket endpoint
        .route("/ws", get(crate::ws::handler::ws_handler))
        // Root redirect to API docs
        .route("/", get(root_handler))
        // Fallback for unmatched routes
        .fallback(fallback_handler.into_service())
        // Apply common middleware
        .layer(common_middleware)
        // CORS must be outermost
        .layer(cors_layer(&state.config))
        // Attach state
        .with_state(state)
}

async fn root_handler() -> impl IntoResponse {
    axum::response::Redirect::to("/api/v1/docs")
}

async fn fallback_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({
            "error": "not_found",
            "message": "The requested resource was not found"
        })),
    )
}
```

### 2. API v1 Routes (crates/tachikoma-server/src/routes/v1.rs)

```rust
//! API v1 routes.

use crate::{
    handlers::{auth, metrics, missions, specs, forge, config as config_handlers},
    middleware::{auth::AuthLayer, authz::AuthzLayer, rate_limit::RateLimitLayer},
    state::AppState,
};
use axum::{
    middleware,
    routing::{delete, get, post, put, patch},
    Router,
};
use tower::ServiceBuilder;

/// Create the v1 API router.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // Public routes
        .merge(public_routes())
        // Authenticated routes
        .merge(authenticated_routes(state.clone()))
        // API documentation
        .route("/docs", get(api_docs))
        .route("/openapi.json", get(openapi_spec))
}

fn public_routes() -> Router<AppState> {
    Router::new()
        // Authentication
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/forgot-password", post(auth::forgot_password))
        // Rate limited for auth endpoints
        .layer(RateLimitLayer::new(10, Duration::from_secs(60)))
}

fn authenticated_routes(state: AppState) -> Router<AppState> {
    let auth_middleware = ServiceBuilder::new()
        .layer(AuthLayer::new(state.config.jwt_secret.clone()));

    Router::new()
        // User
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/change-password", post(auth::change_password))
        // Missions
        .nest("/missions", mission_routes())
        // Specs
        .nest("/specs", spec_routes())
        // Forge sessions
        .nest("/forge", forge_routes())
        // Configuration
        .nest("/config", config_routes())
        // Metrics (authenticated)
        .route("/metrics/usage", get(metrics::usage))
        .route("/metrics/costs", get(metrics::costs))
        // Apply authentication middleware
        .layer(auth_middleware)
}

fn mission_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(missions::list).post(missions::create))
        .route("/:id", get(missions::get).put(missions::update).delete(missions::delete))
        .route("/:id/start", post(missions::start))
        .route("/:id/pause", post(missions::pause))
        .route("/:id/resume", post(missions::resume))
        .route("/:id/cancel", post(missions::cancel))
        .route("/:id/logs", get(missions::logs))
        .route("/:id/artifacts", get(missions::artifacts))
}

fn spec_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(specs::list).post(specs::create))
        .route("/:id", get(specs::get).put(specs::update).delete(specs::delete))
        .route("/:id/validate", post(specs::validate))
        .route("/:id/preview", get(specs::preview))
        .route("/search", get(specs::search))
}

fn forge_routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(forge::list_sessions).post(forge::create_session))
        .route("/sessions/:id", get(forge::get_session).delete(forge::delete_session))
        .route("/sessions/:id/drafts", get(forge::list_drafts).post(forge::add_draft))
        .route("/sessions/:id/synthesize", post(forge::synthesize))
        .route("/sessions/:id/converge", post(forge::converge))
}

fn config_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(config_handlers::get_config).put(config_handlers::update_config))
        .route("/models", get(config_handlers::list_models))
        .route("/providers", get(config_handlers::list_providers))
        .route("/validate", post(config_handlers::validate_config))
}

async fn api_docs() -> impl axum::response::IntoResponse {
    // Return Swagger UI HTML
    axum::response::Html(include_str!("../../static/swagger.html"))
}

async fn openapi_spec() -> impl axum::response::IntoResponse {
    // Return OpenAPI JSON spec
    axum::Json(include_str!("../../static/openapi.json"))
}

use std::time::Duration;
```

### 3. Internal Routes (crates/tachikoma-server/src/routes/internal.rs)

```rust
//! Internal routes for health checks, metrics, and admin operations.

use crate::{
    handlers::{health, metrics as metrics_handlers, admin},
    state::AppState,
};
use axum::{
    routing::{get, post},
    Router,
};

/// Create the internal routes router.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // Health checks
        .route("/health", get(health::health_check))
        .route("/health/live", get(health::liveness))
        .route("/health/ready", get(health::readiness))
        // Prometheus metrics
        .route("/metrics", get(metrics_handlers::prometheus))
        // Debug/admin endpoints (protected by internal network)
        .nest("/admin", admin_routes())
}

fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/cache/clear", post(admin::clear_cache))
        .route("/connections", get(admin::list_connections))
        .route("/debug/state", get(admin::debug_state))
}
```

### 4. Route Extractors (crates/tachikoma-server/src/routes/extractors.rs)

```rust
//! Custom extractors for route handlers.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Path, Query},
    http::{request::Parts, StatusCode},
};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

/// Extract and validate a UUID path parameter.
pub struct ValidatedId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for ValidatedId
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(id): Path<String> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::BadRequest("Invalid path parameter".into()))?;

        let uuid = Uuid::parse_str(&id)
            .map_err(|_| ApiError::BadRequest("Invalid UUID format".into()))?;

        Ok(ValidatedId(uuid))
    }
}

/// Pagination parameters.
#[derive(Debug, serde::Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl Pagination {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    pub fn limit(&self) -> u32 {
        self.per_page.min(100)
    }
}

/// Sort parameters.
#[derive(Debug, serde::Deserialize)]
pub struct Sort {
    #[serde(default = "default_sort_field")]
    pub sort_by: String,
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

fn default_sort_field() -> String { "created_at".into() }
fn default_sort_order() -> SortOrder { SortOrder::Desc }
```

---

## Testing Requirements

1. All routes resolve correctly
2. Middleware applied in correct order
3. Nested routers work
4. Fallback returns 404
5. CORS headers present
6. Request ID propagated
7. Compression enabled

---

## Related Specs

- Depends on: [316-server-crate.md](316-server-crate.md)
- Next: [318-api-versioning.md](318-api-versioning.md)
- Used by: All handler specs
