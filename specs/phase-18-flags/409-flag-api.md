# 409 - Feature Flag API

## Overview

RESTful API specification for feature flag management and evaluation.


## Acceptance Criteria
- [x] Implementation complete per spec

## API Endpoints

### Flag Management

```yaml
openapi: 3.0.3
info:
  title: Feature Flags API
  version: 1.0.0

paths:
  /api/v1/flags:
    get:
      summary: List all flags
      parameters:
        - name: status
          in: query
          schema:
            type: string
            enum: [active, disabled, deprecated, archived]
        - name: tags
          in: query
          schema:
            type: array
            items:
              type: string
        - name: owner
          in: query
          schema:
            type: string
        - name: limit
          in: query
          schema:
            type: integer
            default: 50
        - name: offset
          in: query
          schema:
            type: integer
            default: 0
      responses:
        '200':
          description: List of flags
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/FlagListResponse'

    post:
      summary: Create a new flag
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateFlagRequest'
      responses:
        '201':
          description: Flag created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Flag'
        '400':
          description: Invalid request
        '409':
          description: Flag already exists

  /api/v1/flags/{flagId}:
    get:
      summary: Get a flag by ID
      parameters:
        - name: flagId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Flag details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Flag'
        '404':
          description: Flag not found

    put:
      summary: Update a flag
      parameters:
        - name: flagId
          in: path
          required: true
          schema:
            type: string
        - name: If-Match
          in: header
          schema:
            type: string
          description: ETag for optimistic locking
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateFlagRequest'
      responses:
        '200':
          description: Flag updated
        '404':
          description: Flag not found
        '409':
          description: Version conflict

    delete:
      summary: Delete a flag
      parameters:
        - name: flagId
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: Flag deleted
        '404':
          description: Flag not found

  /api/v1/flags/{flagId}/toggle:
    post:
      summary: Toggle flag status
      parameters:
        - name: flagId
          in: path
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                enabled:
                  type: boolean
      responses:
        '200':
          description: Flag toggled

  /api/v1/flags/{flagId}/rules:
    get:
      summary: Get flag rules
      responses:
        '200':
          description: List of rules

    post:
      summary: Add a rule
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Rule'
      responses:
        '201':
          description: Rule added

  /api/v1/flags/{flagId}/rules/{ruleId}:
    put:
      summary: Update a rule
      responses:
        '200':
          description: Rule updated

    delete:
      summary: Delete a rule
      responses:
        '204':
          description: Rule deleted

  /api/v1/flags/{flagId}/overrides:
    get:
      summary: Get flag overrides
      responses:
        '200':
          description: List of overrides

    post:
      summary: Add an override
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/Override'
      responses:
        '201':
          description: Override added

  /api/v1/flags/{flagId}/audit:
    get:
      summary: Get audit log for a flag
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            default: 50
      responses:
        '200':
          description: Audit entries

components:
  schemas:
    Flag:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        description:
          type: string
        status:
          type: string
          enum: [active, disabled, testing, deprecated, archived]
        valueType:
          type: string
          enum: [boolean, string, number, json, variant]
        defaultValue:
          oneOf:
            - type: boolean
            - type: string
            - type: number
            - type: object
        rules:
          type: array
          items:
            $ref: '#/components/schemas/Rule'
        rollout:
          $ref: '#/components/schemas/Rollout'
        experiment:
          $ref: '#/components/schemas/Experiment'
        metadata:
          $ref: '#/components/schemas/Metadata'
        version:
          type: integer
        createdAt:
          type: string
          format: date-time
        updatedAt:
          type: string
          format: date-time

    CreateFlagRequest:
      type: object
      required:
        - id
        - name
        - valueType
      properties:
        id:
          type: string
          pattern: '^[a-z0-9-]+$'
        name:
          type: string
        description:
          type: string
        valueType:
          type: string
        defaultValue:
          description: Must match valueType
        tags:
          type: array
          items:
            type: string
        owner:
          type: string

    Rule:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        priority:
          type: integer
        conditions:
          type: array
          items:
            $ref: '#/components/schemas/Condition'
        value:
          description: Value to return when rule matches
        enabled:
          type: boolean
          default: true

    Condition:
      type: object
      properties:
        property:
          type: string
        operator:
          type: string
          enum: [equals, not_equals, contains, in, not_in, greater_than, less_than, matches]
        value:
          description: Value to compare against

    Rollout:
      type: object
      properties:
        percentage:
          type: number
          minimum: 0
          maximum: 100
        bucketBy:
          type: string
          default: user_id

    Experiment:
      type: object
      properties:
        name:
          type: string
        variants:
          type: array
          items:
            $ref: '#/components/schemas/Variant'
        bucketBy:
          type: string

    Variant:
      type: object
      properties:
        key:
          type: string
        name:
          type: string
        weight:
          type: number
        value:
          description: Value for this variant

    Override:
      type: object
      properties:
        type:
          type: string
          enum: [user, group, session]
        targetId:
          type: string
        value:
          description: Override value
        expiresAt:
          type: string
          format: date-time

    Metadata:
      type: object
      properties:
        tags:
          type: array
          items:
            type: string
        owner:
          type: string
        documentationUrl:
          type: string
        sunsetDate:
          type: string
          format: date-time
```

## Rust API Implementation

```rust
// crates/flags/src/api.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::definition::FlagDefinition;
use crate::storage::{FlagStorage, QueryOptions, StorageError};
use crate::types::*;

pub struct ApiState {
    pub storage: Arc<dyn FlagStorage>,
    pub audit: Arc<crate::audit::AuditLogger>,
}

pub fn flag_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/flags", get(list_flags).post(create_flag))
        .route("/flags/:id", get(get_flag).put(update_flag).delete(delete_flag))
        .route("/flags/:id/toggle", post(toggle_flag))
        .route("/flags/:id/rules", get(list_rules).post(add_rule))
        .route("/flags/:id/rules/:rule_id", put(update_rule).delete(delete_rule))
        .route("/flags/:id/overrides", get(list_overrides).post(add_override))
        .route("/flags/:id/audit", get(get_audit_log))
        .route("/flags/:id/stats", get(get_flag_stats))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct ListFlagsQuery {
    status: Option<String>,
    tags: Option<String>,
    owner: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn list_flags(
    State(state): State<Arc<ApiState>>,
    Query(query): Query<ListFlagsQuery>,
) -> Result<Json<ListFlagsResponse>, ApiError> {
    let status = query.status.and_then(|s| match s.as_str() {
        "active" => Some(FlagStatus::Active),
        "disabled" => Some(FlagStatus::Disabled),
        "deprecated" => Some(FlagStatus::Deprecated),
        "archived" => Some(FlagStatus::Archived),
        _ => None,
    });

    let tags = query.tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let options = QueryOptions {
        status,
        tags,
        owner: query.owner,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
        ..Default::default()
    };

    let flags = state.storage.list(options.clone()).await?;
    let total = state.storage.count(options).await?;

    Ok(Json(ListFlagsResponse {
        flags: flags.into_iter().map(|f| f.definition).collect(),
        total,
        limit: query.limit.unwrap_or(50),
        offset: query.offset.unwrap_or(0),
    }))
}

#[derive(Serialize)]
struct ListFlagsResponse {
    flags: Vec<FlagDefinition>,
    total: usize,
    limit: usize,
    offset: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    id: String,
    name: String,
    description: Option<String>,
    value_type: String,
    default_value: serde_json::Value,
    tags: Option<Vec<String>>,
    owner: Option<String>,
}

async fn create_flag(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CreateFlagRequest>,
) -> Result<(StatusCode, Json<FlagDefinition>), ApiError> {
    let flag = match req.value_type.as_str() {
        "boolean" => {
            let default = req.default_value.as_bool().unwrap_or(false);
            FlagDefinition::new_boolean(&req.id, &req.name, default)?
        }
        "string" => {
            let default = req.default_value.as_str().unwrap_or("");
            FlagDefinition::new_string(&req.id, &req.name, default)?
        }
        _ => return Err(ApiError::BadRequest("Invalid value type".to_string())),
    };

    let mut flag = flag;
    if let Some(desc) = req.description {
        flag.description = desc;
    }
    if let Some(tags) = req.tags {
        flag.metadata.tags = tags;
    }
    if let Some(owner) = req.owner {
        flag.metadata.owner = Some(owner);
    }

    let stored = state.storage.create(flag).await?;

    Ok((StatusCode::CREATED, Json(stored.definition)))
}

async fn get_flag(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<Json<FlagDefinition>, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(stored.definition))
}

async fn update_flag(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(flag): Json<FlagDefinition>,
) -> Result<Json<FlagDefinition>, ApiError> {
    if flag.id.as_str() != id {
        return Err(ApiError::BadRequest("Flag ID mismatch".to_string()));
    }

    let stored = state.storage.update(flag, None).await?;
    Ok(Json(stored.definition))
}

async fn delete_flag(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let flag_id = FlagId::new(&id);
    state.storage.delete(&flag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    enabled: bool,
}

async fn toggle_flag(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(req): Json<ToggleRequest>,
) -> Result<Json<FlagDefinition>, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    let mut flag = stored.definition;
    flag.status = if req.enabled {
        FlagStatus::Active
    } else {
        FlagStatus::Disabled
    };

    let updated = state.storage.update(flag, None).await?;
    Ok(Json(updated.definition))
}

async fn list_rules(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Rule>>, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(stored.definition.rules))
}

async fn add_rule(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(rule): Json<Rule>,
) -> Result<(StatusCode, Json<Rule>), ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    let mut flag = stored.definition;
    flag.add_rule(rule.clone())?;

    state.storage.update(flag, None).await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

async fn update_rule(
    State(state): State<Arc<ApiState>>,
    Path((id, rule_id)): Path<(String, String)>,
    Json(rule): Json<Rule>,
) -> Result<Json<Rule>, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    let mut flag = stored.definition;
    if let Some(existing) = flag.rules.iter_mut().find(|r| r.id == rule_id) {
        *existing = rule.clone();
    } else {
        return Err(ApiError::NotFound);
    }

    state.storage.update(flag, None).await?;
    Ok(Json(rule))
}

async fn delete_rule(
    State(state): State<Arc<ApiState>>,
    Path((id, rule_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    let mut flag = stored.definition;
    flag.rules.retain(|r| r.id != rule_id);

    state.storage.update(flag, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_overrides(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Result<Json<OverridesResponse>, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(OverridesResponse {
        user_overrides: stored.definition.user_overrides,
        group_overrides: stored.definition.group_overrides,
    }))
}

#[derive(Serialize)]
struct OverridesResponse {
    user_overrides: std::collections::HashMap<String, FlagValue>,
    group_overrides: std::collections::HashMap<String, FlagValue>,
}

async fn add_override(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(req): Json<AddOverrideRequest>,
) -> Result<StatusCode, ApiError> {
    let flag_id = FlagId::new(&id);
    let stored = state.storage.get(&flag_id).await?
        .ok_or(ApiError::NotFound)?;

    let mut flag = stored.definition;

    match req.override_type.as_str() {
        "user" => {
            flag.user_overrides.insert(req.target_id, req.value);
        }
        "group" => {
            flag.group_overrides.insert(req.target_id, req.value);
        }
        _ => return Err(ApiError::BadRequest("Invalid override type".to_string())),
    }

    state.storage.update(flag, None).await?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct AddOverrideRequest {
    override_type: String,
    target_id: String,
    value: FlagValue,
}

async fn get_audit_log(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<crate::audit::AuditEntry>>, ApiError> {
    let entries = state.audit.get_flag_history(&id, query.limit.unwrap_or(50)).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(entries))
}

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}

async fn get_flag_stats(
    State(_state): State<Arc<ApiState>>,
    Path(_id): Path<String>,
) -> Result<Json<FlagStatistics>, ApiError> {
    // Would integrate with analytics
    Ok(Json(FlagStatistics::default()))
}

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    BadRequest(String),
    Conflict(String),
    Internal(String),
}

impl From<StorageError> for ApiError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::NotFound(_) => ApiError::NotFound,
            StorageError::AlreadyExists(_) => ApiError::Conflict("Already exists".to_string()),
            StorageError::VersionConflict { .. } => ApiError::Conflict("Version conflict".to_string()),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

impl From<crate::definition::FlagDefinitionError> for ApiError {
    fn from(err: crate::definition::FlagDefinitionError) -> Self {
        ApiError::BadRequest(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::BadRequest(msg) => return (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Conflict(msg) => return (StatusCode::CONFLICT, msg).into_response(),
            ApiError::Internal(msg) => return (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        };

        (status, message).into_response()
    }
}
```

## SDK Evaluation Endpoint

```rust
// Lightweight evaluation endpoint for SDKs
pub fn sdk_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/sdk/flags", get(sdk_get_all_flags))
        .route("/sdk/evaluate", post(sdk_evaluate))
        .route("/sdk/evaluate/:flag_id", post(sdk_evaluate_single))
        .route("/sdk/track", post(sdk_track))
        .with_state(state)
}

async fn sdk_get_all_flags(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<Vec<FlagDefinition>>, ApiError> {
    let flags = state.storage.list(QueryOptions {
        status: Some(FlagStatus::Active),
        limit: 10000,
        ..Default::default()
    }).await?;

    Ok(Json(flags.into_iter().map(|f| f.definition).collect()))
}
```

## Related Specs

- 392-flag-definition.md - Flag structure
- 401-flag-admin-ui.md - UI integration
- 402-flag-sdk-rust.md - SDK usage
