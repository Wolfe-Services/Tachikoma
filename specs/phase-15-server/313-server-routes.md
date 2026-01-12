# Spec 313: Route Definitions

## Phase
15 - Server/API Layer

## Spec ID
313

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 312: Server Configuration

## Estimated Context
~10%

---

## Objective

Define the complete routing structure for the Tachikoma API, organizing endpoints into logical groups with proper versioning, documentation, and type-safe route parameters.

---

## Acceptance Criteria

- [ ] All API routes are versioned under /api/v1
- [ ] Routes are organized by resource type
- [ ] Route parameters are type-safe using extractors
- [ ] OpenAPI documentation is auto-generated
- [ ] Route conflicts are prevented at compile time
- [ ] Nested routers support middleware composition
- [ ] Route metadata supports permissions/scopes

---

## Implementation Details

### Route Organization

```rust
// src/server/routes/mod.rs
pub mod api;
pub mod health;
pub mod websocket;

use axum::Router;
use crate::server::state::AppState;

/// Build all API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1", api::v1_routes())
        .merge(health::health_routes())
}

/// Build WebSocket routes
pub fn websocket_routes() -> Router<AppState> {
    Router::new()
        .nest("/ws", websocket::ws_routes())
}
```

### API Version 1 Routes

```rust
// src/server/routes/api/mod.rs
pub mod missions;
pub mod specs;
pub mod forge;
pub mod backends;
pub mod settings;

use axum::{
    Router,
    routing::{get, post, put, patch, delete},
};
use crate::server::state::AppState;

/// All v1 API routes
pub fn v1_routes() -> Router<AppState> {
    Router::new()
        // Missions
        .nest("/missions", missions::routes())
        // Specs
        .nest("/specs", specs::routes())
        // Forge
        .nest("/forge", forge::routes())
        // Backends
        .nest("/backends", backends::routes())
        // Settings
        .nest("/settings", settings::routes())
}
```

### Mission Routes

```rust
// src/server/routes/api/missions.rs
use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::server::{
    state::AppState,
    handlers::missions as handlers,
    error::ApiError,
};
use crate::api::types::{
    CreateMissionRequest,
    UpdateMissionRequest,
    MissionResponse,
    MissionListResponse,
    PaginationParams,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", get(list_missions).post(create_mission))
        // Single resource routes
        .route(
            "/:mission_id",
            get(get_mission)
                .put(update_mission)
                .delete(delete_mission)
        )
        // Mission phases
        .route("/:mission_id/phases", get(list_phases).post(create_phase))
        .route(
            "/:mission_id/phases/:phase_id",
            get(get_phase).put(update_phase).delete(delete_phase)
        )
        // Mission actions
        .route("/:mission_id/activate", post(activate_mission))
        .route("/:mission_id/archive", post(archive_mission))
        .route("/:mission_id/duplicate", post(duplicate_mission))
        // Mission exports
        .route("/:mission_id/export", get(export_mission))
}

/// List all missions with pagination
#[utoipa::path(
    get,
    path = "/api/v1/missions",
    params(PaginationParams),
    responses(
        (status = 200, description = "List of missions", body = MissionListResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "missions"
)]
async fn list_missions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<MissionListResponse>, ApiError> {
    handlers::list_missions(state, params).await
}

/// Create a new mission
#[utoipa::path(
    post,
    path = "/api/v1/missions",
    request_body = CreateMissionRequest,
    responses(
        (status = 201, description = "Mission created", body = MissionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "missions"
)]
async fn create_mission(
    State(state): State<AppState>,
    Json(request): Json<CreateMissionRequest>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::create_mission(state, request).await
}

/// Get a specific mission
#[utoipa::path(
    get,
    path = "/api/v1/missions/{mission_id}",
    params(
        ("mission_id" = Uuid, Path, description = "Mission UUID")
    ),
    responses(
        (status = 200, description = "Mission details", body = MissionResponse),
        (status = 404, description = "Mission not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "missions"
)]
async fn get_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::get_mission(state, mission_id).await
}

/// Update a mission
async fn update_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Json(request): Json<UpdateMissionRequest>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::update_mission(state, mission_id, request).await
}

/// Delete a mission
async fn delete_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<(), ApiError> {
    handlers::delete_mission(state, mission_id).await
}

// Phase handlers
async fn list_phases(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<Json<Vec<PhaseResponse>>, ApiError> {
    handlers::list_phases(state, mission_id).await
}

async fn create_phase(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Json(request): Json<CreatePhaseRequest>,
) -> Result<Json<PhaseResponse>, ApiError> {
    handlers::create_phase(state, mission_id, request).await
}

async fn get_phase(
    State(state): State<AppState>,
    Path((mission_id, phase_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<PhaseResponse>, ApiError> {
    handlers::get_phase(state, mission_id, phase_id).await
}

async fn update_phase(
    State(state): State<AppState>,
    Path((mission_id, phase_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdatePhaseRequest>,
) -> Result<Json<PhaseResponse>, ApiError> {
    handlers::update_phase(state, mission_id, phase_id, request).await
}

async fn delete_phase(
    State(state): State<AppState>,
    Path((mission_id, phase_id)): Path<(Uuid, Uuid)>,
) -> Result<(), ApiError> {
    handlers::delete_phase(state, mission_id, phase_id).await
}

// Action handlers
async fn activate_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::activate_mission(state, mission_id).await
}

async fn archive_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::archive_mission(state, mission_id).await
}

async fn duplicate_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> Result<Json<MissionResponse>, ApiError> {
    handlers::duplicate_mission(state, mission_id).await
}

async fn export_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Query(params): Query<ExportParams>,
) -> Result<impl IntoResponse, ApiError> {
    handlers::export_mission(state, mission_id, params).await
}
```

### Spec Routes

```rust
// src/server/routes/api/specs.rs
use axum::{
    Router,
    routing::{get, post, put, patch, delete},
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;

use crate::server::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Collection routes
        .route("/", get(list_specs).post(create_spec))
        // Single resource routes
        .route(
            "/:spec_id",
            get(get_spec).put(update_spec).delete(delete_spec)
        )
        // Spec status updates
        .route("/:spec_id/status", patch(update_status))
        // Spec execution
        .route("/:spec_id/execute", post(execute_spec))
        // Spec dependencies
        .route("/:spec_id/dependencies", get(get_dependencies))
        // Spec conversation
        .route("/:spec_id/conversation", get(get_conversation))
        .route("/:spec_id/conversation", post(add_message))
        // Spec file changes
        .route("/:spec_id/changes", get(list_changes))
        .route("/:spec_id/changes/:change_id/apply", post(apply_change))
        .route("/:spec_id/changes/:change_id/reject", post(reject_change))
        // Search within phase
        .route("/search", get(search_specs))
}

async fn list_specs(
    State(state): State<AppState>,
    Query(params): Query<SpecListParams>,
) -> Result<Json<SpecListResponse>, ApiError> {
    handlers::list_specs(state, params).await
}

async fn create_spec(
    State(state): State<AppState>,
    Json(request): Json<CreateSpecRequest>,
) -> Result<Json<SpecResponse>, ApiError> {
    handlers::create_spec(state, request).await
}

async fn get_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> Result<Json<SpecResponse>, ApiError> {
    handlers::get_spec(state, spec_id).await
}

async fn update_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<UpdateSpecRequest>,
) -> Result<Json<SpecResponse>, ApiError> {
    handlers::update_spec(state, spec_id, request).await
}

async fn delete_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> Result<(), ApiError> {
    handlers::delete_spec(state, spec_id).await
}

async fn update_status(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<Json<SpecResponse>, ApiError> {
    handlers::update_status(state, spec_id, request).await
}

async fn execute_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<ExecuteSpecRequest>,
) -> Result<Json<ExecutionResponse>, ApiError> {
    handlers::execute_spec(state, spec_id, request).await
}

async fn get_dependencies(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> Result<Json<DependencyGraph>, ApiError> {
    handlers::get_dependencies(state, spec_id).await
}

async fn get_conversation(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> Result<Json<ConversationResponse>, ApiError> {
    handlers::get_conversation(state, spec_id).await
}

async fn add_message(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<AddMessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    handlers::add_message(state, spec_id, request).await
}

async fn search_specs(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    handlers::search_specs(state, params).await
}
```

### Type-Safe Route Parameters

```rust
// src/server/routes/extractors.rs
use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;
use serde::Deserialize;

/// Validated mission ID extractor
#[derive(Debug, Clone, Copy)]
pub struct MissionId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for MissionId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(id) = Path::<Uuid>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid mission ID format".to_string()))?;

        Ok(MissionId(id))
    }
}

/// Validated spec ID extractor
#[derive(Debug, Clone, Copy)]
pub struct SpecId(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for SpecId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(id) = Path::<Uuid>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid spec ID format".to_string()))?;

        Ok(SpecId(id))
    }
}

/// Composite path extractor for nested resources
#[derive(Debug, Deserialize)]
pub struct MissionSpecPath {
    pub mission_id: Uuid,
    pub spec_id: Uuid,
}

/// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub phase_id: Option<Uuid>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}
```

### OpenAPI Documentation

```rust
// src/server/routes/openapi.rs
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::Router;

use crate::server::state::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Tachikoma API",
        version = "1.0.0",
        description = "API for managing missions, specs, and LLM interactions",
        license(name = "MIT"),
        contact(
            name = "Tachikoma Team",
            url = "https://github.com/tachikoma"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development"),
        (url = "https://api.tachikoma.io", description = "Production")
    ),
    paths(
        // Missions
        crate::server::routes::api::missions::list_missions,
        crate::server::routes::api::missions::create_mission,
        crate::server::routes::api::missions::get_mission,
        // Specs
        crate::server::routes::api::specs::list_specs,
        crate::server::routes::api::specs::create_spec,
        crate::server::routes::api::specs::get_spec,
        // Health
        crate::server::routes::health::health_check,
        crate::server::routes::health::readiness,
    ),
    components(
        schemas(
            MissionResponse,
            CreateMissionRequest,
            SpecResponse,
            CreateSpecRequest,
            PaginationParams,
            HealthResponse,
        )
    ),
    tags(
        (name = "missions", description = "Mission management"),
        (name = "specs", description = "Spec management"),
        (name = "forge", description = "Forge management"),
        (name = "backends", description = "LLM backend management"),
        (name = "health", description = "Health checks"),
    )
)]
pub struct ApiDoc;

/// Build OpenAPI documentation routes
pub fn openapi_routes() -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
```

### Route Metadata for Authorization

```rust
// src/server/routes/metadata.rs
use std::collections::HashSet;

/// Route metadata for authorization
#[derive(Debug, Clone)]
pub struct RouteMetadata {
    /// Required permissions for this route
    pub permissions: HashSet<Permission>,
    /// Whether the route requires authentication
    pub requires_auth: bool,
    /// Rate limit category
    pub rate_limit_category: RateLimitCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    MissionsRead,
    MissionsWrite,
    SpecsRead,
    SpecsWrite,
    SpecsExecute,
    ForgeRead,
    ForgeWrite,
    BackendsRead,
    BackendsWrite,
    SettingsRead,
    SettingsWrite,
    Admin,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum RateLimitCategory {
    #[default]
    Standard,
    Expensive,      // LLM calls
    Bulk,           // Bulk operations
    Unlimited,      // Health checks
}

impl Default for RouteMetadata {
    fn default() -> Self {
        Self {
            permissions: HashSet::new(),
            requires_auth: false,
            rate_limit_category: RateLimitCategory::Standard,
        }
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, Method};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_routes_are_registered() {
        let app = create_test_app();

        // Test GET /api/v1/missions
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/missions")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_nested_routes() {
        let app = create_test_app();

        // Test GET /api/v1/missions/:id/phases
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/missions/00000000-0000-0000-0000-000000000001/phases")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_invalid_uuid_returns_400() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/missions/not-a-uuid")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 317**: Missions API (handlers)
- **Spec 318**: Specs API (handlers)
- **Spec 328**: Request Validation
