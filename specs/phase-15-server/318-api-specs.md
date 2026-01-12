# Spec 318: Specs API

## Phase
15 - Server/API Layer

## Spec ID
318

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 313: Route Definitions
- Spec 317: Missions API

## Estimated Context
~12%

---

## Objective

Implement the complete Specs API for Tachikoma, providing CRUD operations for specs, status management, execution triggers, dependency tracking, and conversation handling with full validation and proper response formatting.

---

## Acceptance Criteria

- [ ] Full CRUD operations for specs
- [ ] Status transition workflow with validation
- [ ] Spec execution triggering
- [ ] Dependency graph management
- [ ] Conversation history management
- [ ] File change tracking and application
- [ ] Proper filtering and search
- [ ] Context estimation tracking

---

## Implementation Details

### Request/Response Types

```rust
// src/api/types/specs.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

/// Request to create a new spec
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateSpecRequest {
    /// Phase this spec belongs to
    pub phase_id: Uuid,

    /// Spec identifier (e.g., "311")
    #[validate(length(min = 1, max = 50))]
    pub spec_id: String,

    /// Spec title
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    /// Detailed description/objective
    #[validate(length(max = 10000))]
    pub description: Option<String>,

    /// Acceptance criteria (markdown)
    pub acceptance_criteria: Option<String>,

    /// Implementation details (markdown)
    pub implementation_details: Option<String>,

    /// Testing requirements (markdown)
    pub testing_requirements: Option<String>,

    /// Dependencies on other specs
    pub dependencies: Option<Vec<Uuid>>,

    /// Estimated context percentage
    #[validate(range(min = 0, max = 100))]
    pub estimated_context: Option<f32>,

    /// Tags for organization
    pub tags: Option<Vec<String>>,
}

/// Request to update a spec
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateSpecRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,

    #[validate(length(max = 10000))]
    pub description: Option<String>,

    pub acceptance_criteria: Option<String>,

    pub implementation_details: Option<String>,

    pub testing_requirements: Option<String>,

    pub dependencies: Option<Vec<Uuid>>,

    #[validate(range(min = 0, max = 100))]
    pub estimated_context: Option<f32>,

    pub tags: Option<Vec<String>>,

    /// Version for optimistic locking
    pub version: i64,
}

/// Request to update spec status
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: SpecStatus,
    pub notes: Option<String>,
}

/// Spec response
#[derive(Debug, Clone, Serialize)]
pub struct SpecResponse {
    pub id: Uuid,
    pub phase_id: Uuid,
    pub mission_id: Uuid,
    pub spec_id: String,
    pub title: String,
    pub status: SpecStatus,
    pub description: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub implementation_details: Option<String>,
    pub testing_requirements: Option<String>,
    pub dependencies: Vec<SpecDependency>,
    pub dependents: Vec<SpecDependency>,
    pub estimated_context: Option<f32>,
    pub actual_context: Option<f32>,
    pub tags: Vec<String>,
    pub stats: SpecStats,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecDependency {
    pub spec_id: Uuid,
    pub spec_number: String,
    pub title: String,
    pub status: SpecStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecStats {
    pub message_count: i32,
    pub file_changes: i32,
    pub applied_changes: i32,
    pub execution_count: i32,
    pub last_execution_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpecStatus {
    Planned,
    InProgress,
    InReview,
    Testing,
    Completed,
    Blocked,
    Deferred,
}

/// List response
#[derive(Debug, Clone, Serialize)]
pub struct SpecListResponse {
    pub specs: Vec<SpecSummary>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpecSummary {
    pub id: Uuid,
    pub phase_id: Uuid,
    pub spec_id: String,
    pub title: String,
    pub status: SpecStatus,
    pub estimated_context: Option<f32>,
    pub dependency_count: i32,
    pub message_count: i32,
    pub tags: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation types
#[derive(Debug, Clone, Serialize)]
pub struct ConversationResponse {
    pub spec_id: Uuid,
    pub messages: Vec<MessageResponse>,
    pub total_tokens: i64,
    pub context_usage: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub tokens: Option<i32>,
    pub model: Option<String>,
    pub file_changes: Vec<FileChangeResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AddMessageRequest {
    pub role: MessageRole,

    #[validate(length(min = 1, max = 100000))]
    pub content: String,

    /// Whether to trigger LLM execution after adding
    pub execute: Option<bool>,
}

/// File change types
#[derive(Debug, Clone, Serialize)]
pub struct FileChangeResponse {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_path: String,
    pub change_type: FileChangeType,
    pub original_content: Option<String>,
    pub new_content: String,
    pub status: FileChangeStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    Create,
    Modify,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeStatus {
    Pending,
    Applied,
    Rejected,
    Conflicted,
}

/// Dependency graph response
#[derive(Debug, Clone, Serialize)]
pub struct DependencyGraph {
    pub root_spec_id: Uuid,
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyNode {
    pub spec_id: Uuid,
    pub spec_number: String,
    pub title: String,
    pub status: SpecStatus,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyEdge {
    pub from: Uuid,
    pub to: Uuid,
    pub relationship: DependencyRelationship,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRelationship {
    DependsOn,
    Blocks,
}
```

### Spec Handlers

```rust
// src/server/handlers/specs.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::api::types::specs::*;
use crate::server::error::{ApiError, ApiResult};
use crate::server::state::AppState;

/// List specs with filtering
pub async fn list_specs(
    State(state): State<AppState>,
    Query(params): Query<SpecListParams>,
) -> ApiResult<Json<SpecListResponse>> {
    let storage = state.storage();

    let filter = SpecFilter {
        phase_id: params.phase_id,
        mission_id: params.mission_id,
        status: params.status,
        search: params.search.clone(),
        tag: params.tag.clone(),
    };

    let (specs, total) = storage
        .specs()
        .list(&filter, params.pagination())
        .await?;

    let summaries: Vec<SpecSummary> = futures::future::try_join_all(
        specs.into_iter().map(|s| build_spec_summary(s, storage.clone()))
    ).await?;

    Ok(Json(SpecListResponse {
        specs: summaries,
        pagination: PaginationMeta::new(params.page, params.per_page, total),
    }))
}

/// Create a new spec
pub async fn create_spec(
    State(state): State<AppState>,
    Json(request): Json<CreateSpecRequest>,
) -> ApiResult<(StatusCode, Json<SpecResponse>)> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let storage = state.storage();

    // Verify phase exists
    let phase = storage.phases().get(request.phase_id).await?;

    // Check for duplicate spec_id within mission
    if storage.specs().exists_in_mission(phase.mission_id, &request.spec_id).await? {
        return Err(ApiError::Conflict {
            message: format!("Spec '{}' already exists in this mission", request.spec_id),
        });
    }

    // Validate dependencies exist
    if let Some(ref deps) = request.dependencies {
        for dep_id in deps {
            storage.specs().get(*dep_id).await?;
        }
    }

    let spec = Spec {
        id: Uuid::new_v4(),
        phase_id: request.phase_id,
        spec_id: request.spec_id,
        title: request.title,
        status: SpecStatus::Planned,
        description: request.description,
        acceptance_criteria: request.acceptance_criteria,
        implementation_details: request.implementation_details,
        testing_requirements: request.testing_requirements,
        estimated_context: request.estimated_context,
        actual_context: None,
        tags: request.tags.unwrap_or_default(),
        version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let spec = storage.specs().create(spec).await?;

    // Create dependencies
    if let Some(deps) = request.dependencies {
        for dep_id in deps {
            storage.specs().add_dependency(spec.id, dep_id).await?;
        }
    }

    let response = build_spec_response(spec, storage).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific spec
pub async fn get_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> ApiResult<Json<SpecResponse>> {
    let storage = state.storage();
    let spec = storage.specs().get(spec_id).await?;
    let response = build_spec_response(spec, storage).await?;
    Ok(Json(response))
}

/// Update a spec
pub async fn update_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<UpdateSpecRequest>,
) -> ApiResult<Json<SpecResponse>> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let storage = state.storage();
    let mut spec = storage.specs().get(spec_id).await?;

    // Optimistic locking
    if spec.version != request.version {
        return Err(ApiError::Conflict {
            message: format!(
                "Spec was modified. Expected version {}, found {}",
                request.version, spec.version
            ),
        });
    }

    // Apply updates
    if let Some(title) = request.title {
        spec.title = title;
    }
    if let Some(description) = request.description {
        spec.description = Some(description);
    }
    if let Some(criteria) = request.acceptance_criteria {
        spec.acceptance_criteria = Some(criteria);
    }
    if let Some(details) = request.implementation_details {
        spec.implementation_details = Some(details);
    }
    if let Some(testing) = request.testing_requirements {
        spec.testing_requirements = Some(testing);
    }
    if let Some(context) = request.estimated_context {
        spec.estimated_context = Some(context);
    }
    if let Some(tags) = request.tags {
        spec.tags = tags;
    }

    spec.version += 1;
    spec.updated_at = Utc::now();

    let spec = storage.specs().update(spec).await?;

    // Update dependencies if provided
    if let Some(deps) = request.dependencies {
        storage.specs().clear_dependencies(spec_id).await?;
        for dep_id in deps {
            storage.specs().add_dependency(spec_id, dep_id).await?;
        }
    }

    let response = build_spec_response(spec, storage).await?;
    Ok(Json(response))
}

/// Delete a spec
pub async fn delete_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let storage = state.storage();

    // Check if other specs depend on this one
    let dependents = storage.specs().get_dependents(spec_id).await?;
    if !dependents.is_empty() {
        return Err(ApiError::UnprocessableEntity {
            message: format!(
                "Cannot delete spec: {} other specs depend on it",
                dependents.len()
            ),
        });
    }

    storage.specs().delete(spec_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Update spec status
pub async fn update_status(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<UpdateStatusRequest>,
) -> ApiResult<Json<SpecResponse>> {
    let storage = state.storage();
    let mut spec = storage.specs().get(spec_id).await?;

    // Validate status transition
    validate_status_transition(spec.status, request.status)?;

    // Check dependencies for InProgress transition
    if request.status == SpecStatus::InProgress {
        let deps = storage.specs().get_dependencies(spec_id).await?;
        let blocked: Vec<_> = deps
            .iter()
            .filter(|d| d.status != SpecStatus::Completed)
            .collect();

        if !blocked.is_empty() {
            return Err(ApiError::UnprocessableEntity {
                message: format!(
                    "Cannot start spec: {} dependencies are not completed",
                    blocked.len()
                ),
            });
        }
    }

    spec.status = request.status;
    spec.updated_at = Utc::now();
    spec.version += 1;

    let spec = storage.specs().update(spec).await?;

    // Log status change
    if let Some(notes) = request.notes {
        storage.specs().add_status_history(spec_id, request.status, notes).await?;
    }

    let response = build_spec_response(spec, storage).await?;
    Ok(Json(response))
}

/// Execute a spec (trigger LLM interaction)
pub async fn execute_spec(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<ExecuteSpecRequest>,
) -> ApiResult<Json<ExecutionResponse>> {
    let storage = state.storage();
    let spec = storage.specs().get(spec_id).await?;

    // Verify spec is in a valid state for execution
    if spec.status == SpecStatus::Completed || spec.status == SpecStatus::Deferred {
        return Err(ApiError::UnprocessableEntity {
            message: "Cannot execute a completed or deferred spec".to_string(),
        });
    }

    // Get or select backend
    let backend = if let Some(backend_id) = request.backend_id {
        state.backend_manager().get(backend_id)?
    } else {
        state.backend_manager().get_default()?
    };

    // Build context from spec
    let context = build_execution_context(&spec, storage.clone()).await?;

    // Execute with backend
    let execution = state
        .backend_manager()
        .execute(backend, context, request.options)
        .await?;

    // Store execution result
    storage.specs().record_execution(spec_id, &execution).await?;

    // Update spec status if needed
    if spec.status == SpecStatus::Planned {
        let mut spec = spec;
        spec.status = SpecStatus::InProgress;
        spec.updated_at = Utc::now();
        spec.version += 1;
        storage.specs().update(spec).await?;
    }

    Ok(Json(ExecutionResponse {
        execution_id: execution.id,
        status: execution.status,
        messages_generated: execution.messages.len() as i32,
        file_changes: execution.file_changes.len() as i32,
        tokens_used: execution.tokens_used,
        duration_ms: execution.duration.as_millis() as i64,
    }))
}

/// Get spec dependencies
pub async fn get_dependencies(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> ApiResult<Json<DependencyGraph>> {
    let storage = state.storage();

    // Build full dependency graph
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = std::collections::HashSet::new();

    async fn traverse(
        storage: &Arc<dyn Storage>,
        spec_id: Uuid,
        depth: i32,
        visited: &mut std::collections::HashSet<Uuid>,
        nodes: &mut Vec<DependencyNode>,
        edges: &mut Vec<DependencyEdge>,
    ) -> ApiResult<()> {
        if visited.contains(&spec_id) {
            return Ok(());
        }
        visited.insert(spec_id);

        let spec = storage.specs().get(spec_id).await?;

        nodes.push(DependencyNode {
            spec_id: spec.id,
            spec_number: spec.spec_id.clone(),
            title: spec.title.clone(),
            status: spec.status,
            depth,
        });

        let deps = storage.specs().get_dependencies(spec_id).await?;
        for dep in deps {
            edges.push(DependencyEdge {
                from: spec_id,
                to: dep.id,
                relationship: DependencyRelationship::DependsOn,
            });

            Box::pin(traverse(storage, dep.id, depth + 1, visited, nodes, edges)).await?;
        }

        Ok(())
    }

    traverse(&storage, spec_id, 0, &mut visited, &mut nodes, &mut edges).await?;

    Ok(Json(DependencyGraph {
        root_spec_id: spec_id,
        nodes,
        edges,
    }))
}

/// Get conversation history
pub async fn get_conversation(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
) -> ApiResult<Json<ConversationResponse>> {
    let storage = state.storage();

    let messages = storage.messages().list_for_spec(spec_id).await?;

    let mut total_tokens = 0i64;
    let message_responses: Vec<MessageResponse> = futures::future::try_join_all(
        messages.into_iter().map(|m| async {
            let file_changes = storage.file_changes().list_for_message(m.id).await?;
            total_tokens += m.tokens.unwrap_or(0) as i64;

            Ok::<_, ApiError>(MessageResponse {
                id: m.id,
                role: m.role,
                content: m.content,
                tokens: m.tokens,
                model: m.model,
                file_changes: file_changes.into_iter().map(|fc| fc.into()).collect(),
                created_at: m.created_at,
            })
        })
    ).await?;

    Ok(Json(ConversationResponse {
        spec_id,
        messages: message_responses,
        total_tokens,
        context_usage: calculate_context_usage(total_tokens),
    }))
}

/// Add a message to conversation
pub async fn add_message(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    Json(request): Json<AddMessageRequest>,
) -> ApiResult<(StatusCode, Json<MessageResponse>)> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let storage = state.storage();

    // Verify spec exists
    storage.specs().get(spec_id).await?;

    let message = Message {
        id: Uuid::new_v4(),
        spec_id,
        role: request.role,
        content: request.content,
        tokens: None,
        model: None,
        created_at: Utc::now(),
    };

    let message = storage.messages().create(message).await?;

    // Trigger execution if requested
    if request.execute.unwrap_or(false) {
        // Queue execution asynchronously
        tokio::spawn({
            let state = state.clone();
            let spec_id = spec_id;
            async move {
                if let Err(e) = execute_spec_internal(&state, spec_id).await {
                    tracing::error!(spec_id = %spec_id, error = %e, "Background execution failed");
                }
            }
        });
    }

    Ok((StatusCode::CREATED, Json(MessageResponse {
        id: message.id,
        role: message.role,
        content: message.content,
        tokens: message.tokens,
        model: message.model,
        file_changes: vec![],
        created_at: message.created_at,
    })))
}

/// Apply a file change
pub async fn apply_change(
    State(state): State<AppState>,
    Path((spec_id, change_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<FileChangeResponse>> {
    let storage = state.storage();
    let forge_registry = state.forge_registry();

    let mut change = storage.file_changes().get(change_id).await?;

    // Verify it belongs to the spec
    let message = storage.messages().get(change.message_id).await?;
    if message.spec_id != spec_id {
        return Err(ApiError::not_found("FileChange"));
    }

    // Get the forge for this spec's mission
    let spec = storage.specs().get(spec_id).await?;
    let phase = storage.phases().get(spec.phase_id).await?;
    let mission = storage.missions().get(phase.mission_id).await?;

    let forge = forge_registry
        .read()
        .await
        .get_for_mission(mission.id)
        .ok_or_else(|| ApiError::bad_request("No forge configured for this mission"))?;

    // Apply the change
    match change.change_type {
        FileChangeType::Create | FileChangeType::Modify => {
            forge.write_file(&change.file_path, &change.new_content).await?;
        }
        FileChangeType::Delete => {
            forge.delete_file(&change.file_path).await?;
        }
        FileChangeType::Rename => {
            // Rename handling would need additional fields
            unimplemented!("Rename not yet implemented");
        }
    }

    change.status = FileChangeStatus::Applied;
    let change = storage.file_changes().update(change).await?;

    Ok(Json(change.into()))
}

/// Reject a file change
pub async fn reject_change(
    State(state): State<AppState>,
    Path((spec_id, change_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<FileChangeResponse>> {
    let storage = state.storage();

    let mut change = storage.file_changes().get(change_id).await?;

    // Verify it belongs to the spec
    let message = storage.messages().get(change.message_id).await?;
    if message.spec_id != spec_id {
        return Err(ApiError::not_found("FileChange"));
    }

    change.status = FileChangeStatus::Rejected;
    let change = storage.file_changes().update(change).await?;

    Ok(Json(change.into()))
}

// Helper functions

fn validate_status_transition(from: SpecStatus, to: SpecStatus) -> ApiResult<()> {
    let valid = match (from, to) {
        (SpecStatus::Planned, SpecStatus::InProgress) => true,
        (SpecStatus::Planned, SpecStatus::Blocked) => true,
        (SpecStatus::Planned, SpecStatus::Deferred) => true,
        (SpecStatus::InProgress, SpecStatus::InReview) => true,
        (SpecStatus::InProgress, SpecStatus::Blocked) => true,
        (SpecStatus::InProgress, SpecStatus::Testing) => true,
        (SpecStatus::InReview, SpecStatus::InProgress) => true,
        (SpecStatus::InReview, SpecStatus::Testing) => true,
        (SpecStatus::InReview, SpecStatus::Completed) => true,
        (SpecStatus::Testing, SpecStatus::InProgress) => true,
        (SpecStatus::Testing, SpecStatus::Completed) => true,
        (SpecStatus::Blocked, SpecStatus::Planned) => true,
        (SpecStatus::Blocked, SpecStatus::InProgress) => true,
        (SpecStatus::Deferred, SpecStatus::Planned) => true,
        (same, same2) if same == same2 => true,
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(ApiError::UnprocessableEntity {
            message: format!("Invalid status transition from {:?} to {:?}", from, to),
        })
    }
}

async fn build_spec_response(spec: Spec, storage: &Arc<dyn Storage>) -> ApiResult<SpecResponse> {
    let phase = storage.phases().get(spec.phase_id).await?;
    let dependencies = storage.specs().get_dependencies(spec.id).await?;
    let dependents = storage.specs().get_dependents(spec.id).await?;
    let messages = storage.messages().list_for_spec(spec.id).await?;
    let file_changes = storage.file_changes().list_for_spec(spec.id).await?;
    let executions = storage.specs().get_execution_count(spec.id).await?;
    let last_execution = storage.specs().get_last_execution(spec.id).await?;

    Ok(SpecResponse {
        id: spec.id,
        phase_id: spec.phase_id,
        mission_id: phase.mission_id,
        spec_id: spec.spec_id,
        title: spec.title,
        status: spec.status,
        description: spec.description,
        acceptance_criteria: spec.acceptance_criteria,
        implementation_details: spec.implementation_details,
        testing_requirements: spec.testing_requirements,
        dependencies: dependencies.into_iter().map(|d| d.into()).collect(),
        dependents: dependents.into_iter().map(|d| d.into()).collect(),
        estimated_context: spec.estimated_context,
        actual_context: spec.actual_context,
        tags: spec.tags,
        stats: SpecStats {
            message_count: messages.len() as i32,
            file_changes: file_changes.len() as i32,
            applied_changes: file_changes.iter().filter(|c| c.status == FileChangeStatus::Applied).count() as i32,
            execution_count: executions,
            last_execution_at: last_execution,
        },
        version: spec.version,
        created_at: spec.created_at,
        updated_at: spec.updated_at,
    })
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
    fn test_status_transitions() {
        // Valid transitions
        assert!(validate_status_transition(SpecStatus::Planned, SpecStatus::InProgress).is_ok());
        assert!(validate_status_transition(SpecStatus::InProgress, SpecStatus::InReview).is_ok());
        assert!(validate_status_transition(SpecStatus::Testing, SpecStatus::Completed).is_ok());

        // Invalid transitions
        assert!(validate_status_transition(SpecStatus::Completed, SpecStatus::Planned).is_err());
        assert!(validate_status_transition(SpecStatus::Planned, SpecStatus::Completed).is_err());
    }

    #[tokio::test]
    async fn test_dependency_cycle_detection() {
        let state = create_test_state().await;

        let spec_a = create_test_spec(&state, "A").await;
        let spec_b = create_test_spec(&state, "B").await;

        // A depends on B
        state.storage().specs().add_dependency(spec_a.id, spec_b.id).await.unwrap();

        // B depends on A should fail (cycle)
        let result = state.storage().specs().add_dependency(spec_b.id, spec_a.id).await;
        assert!(result.is_err());
    }
}
```

---

## Related Specs

- **Spec 317**: Missions API
- **Spec 319**: Forge API
- **Spec 325**: WebSocket Streaming
