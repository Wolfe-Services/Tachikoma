# Spec 317: Missions API

## Phase
15 - Server/API Layer

## Spec ID
317

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 313: Route Definitions
- Spec 201: Storage Layer

## Estimated Context
~11%

---

## Objective

Implement the complete Missions API for Tachikoma, providing CRUD operations for missions, phases, and related resources with proper validation, authorization, and response formatting.

---

## Acceptance Criteria

- [ ] Full CRUD operations for missions
- [ ] Nested phase management within missions
- [ ] Mission activation/archival workflows
- [ ] Mission duplication with deep copy
- [ ] Export functionality in multiple formats
- [ ] Proper pagination and filtering
- [ ] Input validation with helpful error messages
- [ ] Optimistic locking for concurrent updates

---

## Implementation Details

### Request/Response Types

```rust
// src/api/types/missions.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

/// Request to create a new mission
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMissionRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be between 1 and 200 characters"))]
    pub name: String,

    #[validate(length(max = 2000, message = "Description cannot exceed 2000 characters"))]
    pub description: Option<String>,

    /// Optional initial phases to create
    pub phases: Option<Vec<CreatePhaseRequest>>,

    /// Template to base the mission on
    pub template_id: Option<Uuid>,

    /// Tags for organization
    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    pub tags: Option<Vec<String>>,

    /// Custom metadata
    pub metadata: Option<serde_json::Value>,
}

/// Request to update a mission
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateMissionRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    pub tags: Option<Vec<String>>,

    pub metadata: Option<serde_json::Value>,

    /// Version for optimistic locking
    pub version: i64,
}

/// Mission response
#[derive(Debug, Clone, Serialize)]
pub struct MissionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: MissionStatus,
    pub phases: Vec<PhaseResponse>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub stats: MissionStats,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissionStats {
    pub total_specs: i32,
    pub completed_specs: i32,
    pub in_progress_specs: i32,
    pub total_phases: i32,
    pub completion_percentage: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
}

/// Phase within a mission
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreatePhaseRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    #[validate(range(min = 0))]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdatePhaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub order: Option<i32>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseResponse {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub order: i32,
    pub spec_count: i32,
    pub completed_specs: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List response with pagination
#[derive(Debug, Clone, Serialize)]
pub struct MissionListResponse {
    pub missions: Vec<MissionSummary>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissionSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: MissionStatus,
    pub phase_count: i32,
    pub spec_count: i32,
    pub completion_percentage: f32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}
```

### Mission Handlers

```rust
// src/server/handlers/missions.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::api::types::missions::*;
use crate::api::types::PaginationParams;
use crate::server::error::{ApiError, ApiResult};
use crate::server::state::AppState;

/// List all missions with pagination and filtering
pub async fn list_missions(
    State(state): State<AppState>,
    Query(params): Query<MissionListParams>,
) -> ApiResult<Json<MissionListResponse>> {
    let storage = state.storage();

    let filter = MissionFilter {
        status: params.status,
        tag: params.tag.clone(),
        search: params.search.clone(),
    };

    let (missions, total) = storage
        .missions()
        .list(&filter, params.pagination())
        .await?;

    let summaries: Vec<MissionSummary> = missions
        .into_iter()
        .map(|m| m.into())
        .collect();

    Ok(Json(MissionListResponse {
        missions: summaries,
        pagination: PaginationMeta {
            page: params.page.unwrap_or(1),
            per_page: params.per_page.unwrap_or(20),
            total,
            total_pages: ((total as f64) / (params.per_page.unwrap_or(20) as f64)).ceil() as u32,
        },
    }))
}

/// Create a new mission
pub async fn create_mission(
    State(state): State<AppState>,
    Json(request): Json<CreateMissionRequest>,
) -> ApiResult<(StatusCode, Json<MissionResponse>)> {
    // Validate request
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let storage = state.storage();

    // Check for template if specified
    let template = if let Some(template_id) = request.template_id {
        Some(storage.missions().get(template_id).await?)
    } else {
        None
    };

    // Create mission
    let mission = Mission {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        status: MissionStatus::Draft,
        tags: request.tags.unwrap_or_default(),
        metadata: request.metadata,
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let mission = storage.missions().create(mission).await?;

    // Create initial phases if provided
    let mut phases = Vec::new();
    if let Some(phase_requests) = request.phases {
        for (i, phase_req) in phase_requests.into_iter().enumerate() {
            let phase = Phase {
                id: Uuid::new_v4(),
                mission_id: mission.id,
                name: phase_req.name,
                description: phase_req.description,
                order: phase_req.order.unwrap_or(i as i32),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            phases.push(storage.phases().create(phase).await?);
        }
    }

    // Build response
    let response = build_mission_response(mission, phases, storage).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific mission
pub async fn get_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> ApiResult<Json<MissionResponse>> {
    let storage = state.storage();

    let mission = storage
        .missions()
        .get(mission_id)
        .await?;

    let phases = storage
        .phases()
        .list_for_mission(mission_id)
        .await?;

    let response = build_mission_response(mission, phases, storage).await?;

    Ok(Json(response))
}

/// Update a mission
pub async fn update_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Json(request): Json<UpdateMissionRequest>,
) -> ApiResult<Json<MissionResponse>> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let storage = state.storage();

    // Get existing mission
    let mut mission = storage.missions().get(mission_id).await?;

    // Check version for optimistic locking
    if mission.version != request.version {
        return Err(ApiError::Conflict {
            message: format!(
                "Mission was modified. Expected version {}, found {}",
                request.version, mission.version
            ),
        });
    }

    // Apply updates
    if let Some(name) = request.name {
        mission.name = name;
    }
    if let Some(description) = request.description {
        mission.description = Some(description);
    }
    if let Some(tags) = request.tags {
        mission.tags = tags;
    }
    if let Some(metadata) = request.metadata {
        mission.metadata = Some(metadata);
    }

    mission.version += 1;
    mission.updated_at = Utc::now();

    let mission = storage.missions().update(mission).await?;

    let phases = storage.phases().list_for_mission(mission_id).await?;
    let response = build_mission_response(mission, phases, storage).await?;

    Ok(Json(response))
}

/// Delete a mission
pub async fn delete_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let storage = state.storage();

    // Check if mission exists
    let mission = storage.missions().get(mission_id).await?;

    // Prevent deletion of active missions
    if mission.status == MissionStatus::Active {
        return Err(ApiError::UnprocessableEntity {
            message: "Cannot delete an active mission. Archive it first.".to_string(),
        });
    }

    // Delete cascade: phases, specs, etc.
    storage.missions().delete(mission_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Activate a mission
pub async fn activate_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> ApiResult<Json<MissionResponse>> {
    let storage = state.storage();

    let mut mission = storage.missions().get(mission_id).await?;

    // Validate transition
    match mission.status {
        MissionStatus::Draft | MissionStatus::Paused => {
            mission.status = MissionStatus::Active;
            mission.updated_at = Utc::now();
            mission.version += 1;
        }
        MissionStatus::Active => {
            // Already active, no-op
        }
        MissionStatus::Completed | MissionStatus::Archived => {
            return Err(ApiError::UnprocessableEntity {
                message: "Cannot activate a completed or archived mission".to_string(),
            });
        }
    }

    let mission = storage.missions().update(mission).await?;
    let phases = storage.phases().list_for_mission(mission_id).await?;
    let response = build_mission_response(mission, phases, storage).await?;

    Ok(Json(response))
}

/// Archive a mission
pub async fn archive_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
) -> ApiResult<Json<MissionResponse>> {
    let storage = state.storage();

    let mut mission = storage.missions().get(mission_id).await?;

    mission.status = MissionStatus::Archived;
    mission.updated_at = Utc::now();
    mission.version += 1;

    let mission = storage.missions().update(mission).await?;
    let phases = storage.phases().list_for_mission(mission_id).await?;
    let response = build_mission_response(mission, phases, storage).await?;

    Ok(Json(response))
}

/// Duplicate a mission
pub async fn duplicate_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Json(request): Json<DuplicateMissionRequest>,
) -> ApiResult<(StatusCode, Json<MissionResponse>)> {
    let storage = state.storage();

    // Get source mission
    let source = storage.missions().get(mission_id).await?;
    let source_phases = storage.phases().list_for_mission(mission_id).await?;

    // Create new mission
    let new_mission = Mission {
        id: Uuid::new_v4(),
        name: request.name.unwrap_or(format!("{} (Copy)", source.name)),
        description: source.description.clone(),
        status: MissionStatus::Draft,
        tags: source.tags.clone(),
        metadata: source.metadata.clone(),
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let new_mission = storage.missions().create(new_mission).await?;

    // Duplicate phases
    let mut new_phases = Vec::new();
    for phase in source_phases {
        let new_phase = Phase {
            id: Uuid::new_v4(),
            mission_id: new_mission.id,
            name: phase.name.clone(),
            description: phase.description.clone(),
            order: phase.order,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        new_phases.push(storage.phases().create(new_phase).await?);

        // Optionally duplicate specs
        if request.include_specs.unwrap_or(false) {
            let specs = storage.specs().list_for_phase(phase.id).await?;
            for spec in specs {
                let new_spec = spec.duplicate_for_phase(new_phases.last().unwrap().id);
                storage.specs().create(new_spec).await?;
            }
        }
    }

    let response = build_mission_response(new_mission, new_phases, storage).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Export a mission
pub async fn export_mission(
    State(state): State<AppState>,
    Path(mission_id): Path<Uuid>,
    Query(params): Query<ExportParams>,
) -> ApiResult<impl IntoResponse> {
    let storage = state.storage();

    let mission = storage.missions().get(mission_id).await?;
    let phases = storage.phases().list_for_mission(mission_id).await?;

    // Gather all specs
    let mut all_specs = Vec::new();
    for phase in &phases {
        let specs = storage.specs().list_for_phase(phase.id).await?;
        all_specs.extend(specs);
    }

    let export = MissionExport {
        mission,
        phases,
        specs: all_specs,
        exported_at: Utc::now(),
        version: "1.0".to_string(),
    };

    match params.format.as_deref().unwrap_or("json") {
        "json" => {
            let json = serde_json::to_string_pretty(&export)?;
            Ok((
                [(header::CONTENT_TYPE, "application/json")],
                [(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}.json\"", mission.name))],
                json,
            ))
        }
        "yaml" => {
            let yaml = serde_yaml::to_string(&export)?;
            Ok((
                [(header::CONTENT_TYPE, "application/x-yaml")],
                [(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}.yaml\"", mission.name))],
                yaml,
            ))
        }
        format => Err(ApiError::bad_request(format!("Unsupported export format: {}", format))),
    }
}

// Helper functions

async fn build_mission_response(
    mission: Mission,
    phases: Vec<Phase>,
    storage: &Arc<dyn Storage>,
) -> ApiResult<MissionResponse> {
    let mut phase_responses = Vec::new();
    let mut total_specs = 0;
    let mut completed_specs = 0;
    let mut in_progress_specs = 0;

    for phase in phases {
        let specs = storage.specs().list_for_phase(phase.id).await?;
        let spec_count = specs.len() as i32;
        let completed = specs.iter().filter(|s| s.status == SpecStatus::Completed).count() as i32;
        let in_progress = specs.iter().filter(|s| s.status == SpecStatus::InProgress).count() as i32;

        total_specs += spec_count;
        completed_specs += completed;
        in_progress_specs += in_progress;

        phase_responses.push(PhaseResponse {
            id: phase.id,
            mission_id: phase.mission_id,
            name: phase.name,
            description: phase.description,
            order: phase.order,
            spec_count,
            completed_specs: completed,
            created_at: phase.created_at,
            updated_at: phase.updated_at,
        });
    }

    let completion_percentage = if total_specs > 0 {
        (completed_specs as f32 / total_specs as f32) * 100.0
    } else {
        0.0
    };

    Ok(MissionResponse {
        id: mission.id,
        name: mission.name,
        description: mission.description,
        status: mission.status,
        phases: phase_responses,
        tags: mission.tags,
        metadata: mission.metadata,
        stats: MissionStats {
            total_specs,
            completed_specs,
            in_progress_specs,
            total_phases: phase_responses.len() as i32,
            completion_percentage,
        },
        version: mission.version,
        created_at: mission.created_at,
        updated_at: mission.updated_at,
    })
}

fn validation_errors_to_field_errors(errors: validator::ValidationErrors) -> Vec<FieldError> {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, errors)| {
            errors.iter().map(|e| FieldError {
                field: field.to_string(),
                message: e.message.clone().map(|m| m.to_string()).unwrap_or_default(),
                code: e.code.to_string().into(),
            })
        })
        .collect()
}
```

### Query Parameters

```rust
// src/api/types/missions.rs (additional)

#[derive(Debug, Clone, Deserialize)]
pub struct MissionListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<MissionStatus>,
    pub tag: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<MissionSortField>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionSortField {
    Name,
    CreatedAt,
    UpdatedAt,
    Status,
}

impl MissionListParams {
    pub fn pagination(&self) -> Pagination {
        Pagination {
            page: self.page.unwrap_or(1),
            per_page: self.per_page.unwrap_or(20).min(100),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DuplicateMissionRequest {
    pub name: Option<String>,
    pub include_specs: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportParams {
    pub format: Option<String>,
    pub include_conversation: Option<bool>,
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mission_validation() {
        let request = CreateMissionRequest {
            name: "".to_string(), // Invalid: empty
            description: None,
            phases: None,
            template_id: None,
            tags: None,
            metadata: None,
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_mission_status_transitions() {
        let mission = Mission {
            status: MissionStatus::Draft,
            ..Default::default()
        };

        // Draft -> Active is valid
        assert!(can_transition(mission.status, MissionStatus::Active));

        // Archived -> Active is invalid
        let archived = Mission {
            status: MissionStatus::Archived,
            ..Default::default()
        };
        assert!(!can_transition(archived.status, MissionStatus::Active));
    }

    #[tokio::test]
    async fn test_list_missions_pagination() {
        let state = create_test_state().await;

        // Create 25 missions
        for i in 0..25 {
            create_test_mission(&state, format!("Mission {}", i)).await;
        }

        let params = MissionListParams {
            page: Some(2),
            per_page: Some(10),
            ..Default::default()
        };

        let result = list_missions(State(state), Query(params)).await.unwrap();

        assert_eq!(result.0.missions.len(), 10);
        assert_eq!(result.0.pagination.page, 2);
        assert_eq!(result.0.pagination.total, 25);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_mission_crud_flow() {
        let app = create_test_app().await;

        // Create
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/missions")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"name": "Test Mission"}"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let body = body_to_json(create_response).await;
        let mission_id = body["id"].as_str().unwrap();

        // Read
        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/missions/{}", mission_id))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        // Update
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/api/v1/missions/{}", mission_id))
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"name": "Updated Mission", "version": 1}"#))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        // Delete
        let delete_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/missions/{}", mission_id))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    }
}
```

---

## Related Specs

- **Spec 313**: Route Definitions
- **Spec 318**: Specs API
- **Spec 328**: Request Validation
- **Spec 330**: Pagination
