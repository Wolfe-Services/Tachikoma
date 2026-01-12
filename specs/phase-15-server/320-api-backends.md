# Spec 320: Backends API

## Phase
15 - Server/API Layer

## Spec ID
320

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 313: Route Definitions
- Spec 401: Backend Abstraction

## Estimated Context
~10%

---

## Objective

Implement the Backends API for Tachikoma, providing endpoints to manage LLM backend configurations (OpenAI, Anthropic, local models), model selection, and backend health monitoring.

---

## Acceptance Criteria

- [ ] CRUD operations for backend configurations
- [ ] Backend health checking and status monitoring
- [ ] Model listing per backend
- [ ] Default backend selection
- [ ] API key management (secure storage)
- [ ] Usage statistics tracking
- [ ] Backend capability discovery

---

## Implementation Details

### Request/Response Types

```rust
// src/api/types/backends.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use validator::Validate;

/// Backend provider types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendProvider {
    OpenAI,
    Anthropic,
    Ollama,
    LMStudio,
    AzureOpenAI,
    Custom,
}

/// Request to create a backend configuration
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBackendRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,

    pub provider: BackendProvider,

    /// API base URL (optional for known providers)
    pub base_url: Option<String>,

    /// API key (will be stored securely)
    pub api_key: Option<String>,

    /// Organization ID (for OpenAI)
    pub organization_id: Option<String>,

    /// Default model to use
    pub default_model: Option<String>,

    /// Maximum tokens per request
    #[validate(range(min = 1, max = 1000000))]
    pub max_tokens: Option<u32>,

    /// Default temperature
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,

    /// Request timeout in seconds
    #[validate(range(min = 1, max = 3600))]
    pub timeout_secs: Option<u32>,

    /// Set as default backend
    pub is_default: Option<bool>,

    /// Custom headers
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
}

/// Request to update a backend
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateBackendRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: Option<String>,

    pub base_url: Option<String>,

    /// New API key (only set if provided)
    pub api_key: Option<String>,

    pub organization_id: Option<String>,

    pub default_model: Option<String>,

    #[validate(range(min = 1, max = 1000000))]
    pub max_tokens: Option<u32>,

    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,

    #[validate(range(min = 1, max = 3600))]
    pub timeout_secs: Option<u32>,

    pub is_default: Option<bool>,

    pub enabled: Option<bool>,
}

/// Backend response
#[derive(Debug, Clone, Serialize)]
pub struct BackendResponse {
    pub id: Uuid,
    pub name: String,
    pub provider: BackendProvider,
    pub base_url: Option<String>,
    pub organization_id: Option<String>,
    pub default_model: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u32,
    pub is_default: bool,
    pub enabled: bool,
    pub status: BackendStatus,
    pub capabilities: BackendCapabilities,
    pub stats: BackendStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendStatus {
    Online,
    Offline,
    RateLimited,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub embeddings: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub streaming: bool,
    pub json_mode: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendStats {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_cost_usd: f64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Model information
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: BackendProvider,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub input_cost_per_1k: Option<f64>,
    pub output_cost_per_1k: Option<f64>,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub function_calling: bool,
    pub vision: bool,
    pub json_mode: bool,
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct BackendHealthResponse {
    pub backend_id: Uuid,
    pub status: BackendStatus,
    pub latency_ms: i64,
    pub message: Option<String>,
    pub models_available: i32,
    pub rate_limit_remaining: Option<i32>,
    pub checked_at: DateTime<Utc>,
}

/// Test request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct TestBackendRequest {
    pub model: Option<String>,

    #[validate(length(min = 1, max = 1000))]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestBackendResponse {
    pub success: bool,
    pub latency_ms: i64,
    pub model_used: String,
    pub response: Option<String>,
    pub tokens_used: Option<i32>,
    pub error: Option<String>,
}
```

### Backend Handlers

```rust
// src/server/handlers/backends.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::api::types::backends::*;
use crate::server::error::{ApiError, ApiResult};
use crate::server::state::AppState;
use crate::backend::{Backend, BackendConfig, OpenAIBackend, AnthropicBackend, OllamaBackend};

/// List all backends
pub async fn list_backends(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<BackendResponse>>> {
    let manager = state.backend_manager();

    let backends: Vec<BackendResponse> = manager
        .list_all()
        .iter()
        .map(|b| build_backend_response(b))
        .collect();

    Ok(Json(backends))
}

/// Create a new backend
pub async fn create_backend(
    State(state): State<AppState>,
    Json(request): Json<CreateBackendRequest>,
) -> ApiResult<(StatusCode, Json<BackendResponse>)> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let manager = state.backend_manager();

    // Check for duplicate name
    if manager.get_by_name(&request.name).is_some() {
        return Err(ApiError::Conflict {
            message: format!("Backend with name '{}' already exists", request.name),
        });
    }

    let config = BackendConfig {
        id: Uuid::new_v4(),
        name: request.name.clone(),
        provider: request.provider,
        base_url: request.base_url.or_else(|| default_base_url(request.provider)),
        api_key: request.api_key.map(|k| SecretString::new(k)),
        organization_id: request.organization_id,
        default_model: request.default_model.or_else(|| default_model(request.provider)),
        max_tokens: request.max_tokens.unwrap_or(4096),
        temperature: request.temperature.unwrap_or(0.7),
        timeout_secs: request.timeout_secs.unwrap_or(120),
        is_default: request.is_default.unwrap_or(false),
        enabled: true,
        custom_headers: request.custom_headers.unwrap_or_default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Store API key securely
    if let Some(ref api_key) = config.api_key {
        state.storage().secrets().store(
            &format!("backend_{}_api_key", config.id),
            api_key.expose(),
        ).await?;
    }

    // Create backend instance
    let backend = create_backend_instance(&config).await?;

    // If setting as default, unset other defaults
    if config.is_default {
        manager.clear_defaults().await;
    }

    manager.register(backend).await;

    let response = build_backend_response(manager.get(config.id).unwrap());

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a backend by ID
pub async fn get_backend(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
) -> ApiResult<Json<BackendResponse>> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    Ok(Json(build_backend_response(backend)))
}

/// Update a backend
pub async fn update_backend(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
    Json(request): Json<UpdateBackendRequest>,
) -> ApiResult<Json<BackendResponse>> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })?;

    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    let mut config = backend.config().clone();

    // Apply updates
    if let Some(name) = request.name {
        config.name = name;
    }
    if let Some(base_url) = request.base_url {
        config.base_url = Some(base_url);
    }
    if let Some(api_key) = request.api_key {
        config.api_key = Some(SecretString::new(api_key.clone()));
        // Update stored secret
        state.storage().secrets().store(
            &format!("backend_{}_api_key", config.id),
            &api_key,
        ).await?;
    }
    if let Some(org_id) = request.organization_id {
        config.organization_id = Some(org_id);
    }
    if let Some(model) = request.default_model {
        config.default_model = Some(model);
    }
    if let Some(max_tokens) = request.max_tokens {
        config.max_tokens = max_tokens;
    }
    if let Some(temperature) = request.temperature {
        config.temperature = temperature;
    }
    if let Some(timeout) = request.timeout_secs {
        config.timeout_secs = timeout;
    }
    if let Some(enabled) = request.enabled {
        config.enabled = enabled;
    }
    if let Some(is_default) = request.is_default {
        if is_default {
            manager.clear_defaults().await;
        }
        config.is_default = is_default;
    }

    config.updated_at = Utc::now();

    // Recreate backend with new config
    let new_backend = create_backend_instance(&config).await?;
    manager.update(new_backend).await;

    let response = build_backend_response(manager.get(backend_id).unwrap());

    Ok(Json(response))
}

/// Delete a backend
pub async fn delete_backend(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    // Prevent deletion of default backend
    if backend.config().is_default {
        return Err(ApiError::UnprocessableEntity {
            message: "Cannot delete the default backend. Set another backend as default first.".to_string(),
        });
    }

    // Delete stored secret
    state.storage().secrets().delete(
        &format!("backend_{}_api_key", backend_id),
    ).await?;

    manager.remove(backend_id).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Check backend health
pub async fn health_check(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
) -> ApiResult<Json<BackendHealthResponse>> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    let start = std::time::Instant::now();
    let health = backend.health_check().await;
    let latency = start.elapsed();

    let models = backend.list_models().await.unwrap_or_default();

    Ok(Json(BackendHealthResponse {
        backend_id,
        status: health.status,
        latency_ms: latency.as_millis() as i64,
        message: health.message,
        models_available: models.len() as i32,
        rate_limit_remaining: health.rate_limit_remaining,
        checked_at: Utc::now(),
    }))
}

/// List available models for a backend
pub async fn list_models(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ModelInfo>>> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    let models = backend.list_models().await?;

    Ok(Json(models.into_iter().map(|m| m.into()).collect()))
}

/// Test a backend with a simple prompt
pub async fn test_backend(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
    Json(request): Json<TestBackendRequest>,
) -> ApiResult<Json<TestBackendResponse>> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    let prompt = request.prompt.unwrap_or_else(|| "Say 'Hello!' and nothing else.".to_string());
    let model = request.model.or_else(|| backend.config().default_model.clone());

    let start = std::time::Instant::now();

    match backend.complete(&prompt, model.as_deref()).await {
        Ok(response) => {
            Ok(Json(TestBackendResponse {
                success: true,
                latency_ms: start.elapsed().as_millis() as i64,
                model_used: response.model,
                response: Some(response.content),
                tokens_used: Some(response.usage.total_tokens as i32),
                error: None,
            }))
        }
        Err(e) => {
            Ok(Json(TestBackendResponse {
                success: false,
                latency_ms: start.elapsed().as_millis() as i64,
                model_used: model.unwrap_or_default(),
                response: None,
                tokens_used: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

/// Get usage statistics
pub async fn get_usage(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
    Query(params): Query<UsageParams>,
) -> ApiResult<Json<UsageResponse>> {
    let manager = state.backend_manager();
    let storage = state.storage();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    let usage = storage.usage().get_for_backend(
        backend_id,
        params.start_date,
        params.end_date,
    ).await?;

    Ok(Json(usage.into()))
}

/// Set default backend
pub async fn set_default(
    State(state): State<AppState>,
    Path(backend_id): Path<Uuid>,
) -> ApiResult<Json<BackendResponse>> {
    let manager = state.backend_manager();

    let backend = manager
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    // Clear all defaults and set this one
    manager.set_default(backend_id).await;

    let response = build_backend_response(manager.get(backend_id).unwrap());

    Ok(Json(response))
}

// Helper functions

async fn create_backend_instance(config: &BackendConfig) -> ApiResult<Box<dyn Backend>> {
    let backend: Box<dyn Backend> = match config.provider {
        BackendProvider::OpenAI => {
            Box::new(OpenAIBackend::new(config.clone()).await?)
        }
        BackendProvider::Anthropic => {
            Box::new(AnthropicBackend::new(config.clone()).await?)
        }
        BackendProvider::Ollama | BackendProvider::LMStudio => {
            Box::new(OllamaBackend::new(config.clone()).await?)
        }
        BackendProvider::AzureOpenAI => {
            Box::new(AzureOpenAIBackend::new(config.clone()).await?)
        }
        BackendProvider::Custom => {
            Box::new(CustomBackend::new(config.clone()).await?)
        }
    };

    Ok(backend)
}

fn default_base_url(provider: BackendProvider) -> Option<String> {
    match provider {
        BackendProvider::OpenAI => Some("https://api.openai.com/v1".to_string()),
        BackendProvider::Anthropic => Some("https://api.anthropic.com".to_string()),
        BackendProvider::Ollama => Some("http://localhost:11434".to_string()),
        BackendProvider::LMStudio => Some("http://localhost:1234/v1".to_string()),
        _ => None,
    }
}

fn default_model(provider: BackendProvider) -> Option<String> {
    match provider {
        BackendProvider::OpenAI => Some("gpt-4-turbo-preview".to_string()),
        BackendProvider::Anthropic => Some("claude-3-opus-20240229".to_string()),
        BackendProvider::Ollama => Some("llama2".to_string()),
        _ => None,
    }
}

fn build_backend_response(backend: &dyn Backend) -> BackendResponse {
    let config = backend.config();
    let stats = backend.stats();
    let capabilities = backend.capabilities();

    BackendResponse {
        id: config.id,
        name: config.name.clone(),
        provider: config.provider,
        base_url: config.base_url.clone(),
        organization_id: config.organization_id.clone(),
        default_model: config.default_model.clone(),
        max_tokens: config.max_tokens,
        temperature: config.temperature,
        timeout_secs: config.timeout_secs,
        is_default: config.is_default,
        enabled: config.enabled,
        status: backend.status(),
        capabilities: capabilities.into(),
        stats: stats.into(),
        created_at: config.created_at,
        updated_at: config.updated_at,
    }
}
```

### Routes

```rust
// src/server/routes/api/backends.rs
use axum::{
    Router,
    routing::{get, post, put, delete},
};

use crate::server::state::AppState;
use crate::server::handlers::backends as handlers;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_backends).post(handlers::create_backend))
        .route("/:backend_id", get(handlers::get_backend).put(handlers::update_backend).delete(handlers::delete_backend))
        .route("/:backend_id/health", get(handlers::health_check))
        .route("/:backend_id/models", get(handlers::list_models))
        .route("/:backend_id/test", post(handlers::test_backend))
        .route("/:backend_id/usage", get(handlers::get_usage))
        .route("/:backend_id/default", post(handlers::set_default))
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
    fn test_default_urls() {
        assert_eq!(
            default_base_url(BackendProvider::OpenAI),
            Some("https://api.openai.com/v1".to_string())
        );
        assert_eq!(
            default_base_url(BackendProvider::Ollama),
            Some("http://localhost:11434".to_string())
        );
    }

    #[tokio::test]
    async fn test_create_backend_validation() {
        let request = CreateBackendRequest {
            name: "".to_string(), // Invalid: empty
            provider: BackendProvider::OpenAI,
            ..Default::default()
        };

        assert!(request.validate().is_err());
    }
}
```

---

## Related Specs

- **Spec 401**: Backend Abstraction
- **Spec 318**: Specs API (execution)
- **Spec 325**: WebSocket Streaming
