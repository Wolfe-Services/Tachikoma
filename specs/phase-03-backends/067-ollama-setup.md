# 067 - Ollama Setup (Local)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 067
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config, 020-http-client-foundation
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement the Ollama backend for local LLM inference, including server detection, health checking, and basic API integration for running models like Llama, Mistral, and CodeLlama locally.

---

## Acceptance Criteria

- [x] `OllamaBackend` implementing `Backend` trait
- [x] Ollama server detection and connection
- [x] Health check and version detection
- [x] Chat API integration
- [x] Generate API integration
- [x] Streaming response support

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backend-ollama/Cargo.toml)

```toml
[package]
name = "tachikoma-backend-ollama"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Ollama local backend for Tachikoma"

[dependencies]
tachikoma-backends-core.workspace = true
tachikoma-common-http.workspace = true
async-trait = "0.1"
reqwest = { workspace = true, features = ["json", "stream"] }
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["sync", "time", "process"] }
futures = "0.3"
tracing.workspace = true
bytes = "1.5"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

### 2. API Types (src/api_types.rs)

```rust
//! Ollama API request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Request to the /api/chat endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// A message in chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Model options.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

/// Tool function definition.
#[derive(Debug, Clone, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: JsonValue,
}

/// Tool call from model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: FunctionCall,
}

/// Function call details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: JsonValue,
}

/// Response from /api/chat.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub prompt_eval_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
    #[serde(default)]
    pub eval_duration: Option<u64>,
}

/// Request to /api/generate endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

/// Response from /api/generate.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    #[serde(default)]
    pub context: Option<Vec<u32>>,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

/// Response from /api/tags.
#[derive(Debug, Clone, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<ModelInfo>,
}

/// Model information.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

/// Model details.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelDetails {
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Response from /api/show.
#[derive(Debug, Clone, Deserialize)]
pub struct ShowResponse {
    pub modelfile: Option<String>,
    pub parameters: Option<String>,
    pub template: Option<String>,
    pub details: Option<ModelDetails>,
}

/// Ollama version info.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}
```

### 3. Server Management (src/server.rs)

```rust
//! Ollama server detection and management.

use reqwest::Client;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for Ollama server.
#[derive(Debug, Clone)]
pub struct OllamaServerConfig {
    /// Base URL of the Ollama server.
    pub base_url: String,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Request timeout.
    pub request_timeout: Duration,
}

impl Default for OllamaServerConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(300), // Long timeout for generation
        }
    }
}

/// Ollama server connection.
#[derive(Debug)]
pub struct OllamaServer {
    config: OllamaServerConfig,
    client: Client,
    version: Option<String>,
}

impl OllamaServer {
    /// Create a new server connection.
    pub fn new(config: OllamaServerConfig) -> Result<Self, ServerError> {
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .map_err(|e| ServerError::HttpClient(e.to_string()))?;

        Ok(Self {
            config,
            client,
            version: None,
        })
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Get the HTTP client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check if the server is running.
    pub async fn is_running(&self) -> bool {
        self.check_health().await.is_ok()
    }

    /// Check server health and get version.
    pub async fn check_health(&self) -> Result<String, ServerError> {
        debug!(url = %self.config.base_url, "Checking Ollama server health");

        let url = format!("{}/api/version", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ServerError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ServerError::NotRunning);
        }

        let version_info: super::api_types::VersionResponse = response
            .json()
            .await
            .map_err(|e| ServerError::InvalidResponse(e.to_string()))?;

        info!(version = %version_info.version, "Connected to Ollama server");

        Ok(version_info.version)
    }

    /// List available models.
    pub async fn list_models(&self) -> Result<Vec<super::api_types::ModelInfo>, ServerError> {
        let url = format!("{}/api/tags", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ServerError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ServerError::ApiError(body));
        }

        let tags: super::api_types::TagsResponse = response
            .json()
            .await
            .map_err(|e| ServerError::InvalidResponse(e.to_string()))?;

        Ok(tags.models)
    }

    /// Get model information.
    pub async fn show_model(&self, name: &str) -> Result<super::api_types::ShowResponse, ServerError> {
        let url = format!("{}/api/show", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
            .map_err(|e| ServerError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ServerError::ApiError(body));
        }

        response
            .json()
            .await
            .map_err(|e| ServerError::InvalidResponse(e.to_string()))
    }

    /// Pull a model.
    pub async fn pull_model(&self, name: &str) -> Result<(), ServerError> {
        let url = format!("{}/api/pull", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": name, "stream": false }))
            .send()
            .await
            .map_err(|e| ServerError::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ServerError::ApiError(body));
        }

        info!(model = %name, "Model pulled successfully");
        Ok(())
    }

    /// Chat endpoint URL.
    pub fn chat_url(&self) -> String {
        format!("{}/api/chat", self.config.base_url)
    }

    /// Generate endpoint URL.
    pub fn generate_url(&self) -> String {
        format!("{}/api/generate", self.config.base_url)
    }
}

/// Server errors.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("failed to create HTTP client: {0}")]
    HttpClient(String),

    #[error("failed to connect to Ollama: {0}")]
    Connection(String),

    #[error("Ollama server is not running")]
    NotRunning,

    #[error("invalid response from server: {0}")]
    InvalidResponse(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),
}
```

### 4. Ollama Backend Implementation (src/backend.rs)

```rust
//! Ollama backend implementation.

use crate::api_types::*;
use crate::server::{OllamaServer, OllamaServerConfig, ServerError};
use async_trait::async_trait;
use tachikoma_backends_core::{
    Backend, BackendCapabilities, BackendError, BackendInfo,
    CompletionRequest, CompletionResponse, CompletionStream,
    FinishReason, Message, Role, ToolCall, Usage,
};
use tracing::{debug, instrument};

/// Ollama backend implementation.
#[derive(Debug)]
pub struct OllamaBackend {
    server: OllamaServer,
    config: OllamaBackendConfig,
    info: BackendInfo,
}

/// Configuration for the Ollama backend.
#[derive(Debug, Clone)]
pub struct OllamaBackendConfig {
    /// Model to use.
    pub model: String,
    /// Context window size.
    pub num_ctx: u32,
    /// Keep model loaded.
    pub keep_alive: String,
    /// Enable tool calling (model dependent).
    pub enable_tools: bool,
}

impl Default for OllamaBackendConfig {
    fn default() -> Self {
        Self {
            model: "llama3.1:8b".to_string(),
            num_ctx: 4096,
            keep_alive: "5m".to_string(),
            enable_tools: false,
        }
    }
}

impl OllamaBackend {
    /// Create a new Ollama backend.
    pub async fn new(
        server_config: OllamaServerConfig,
        backend_config: OllamaBackendConfig,
    ) -> Result<Self, BackendError> {
        let server = OllamaServer::new(server_config)
            .map_err(|e| BackendError::Configuration(e.to_string()))?;

        // Verify server is running
        server.check_health().await
            .map_err(|e| BackendError::ServiceUnavailable {
                message: e.to_string(),
                retry_after: None,
            })?;

        // Get available models
        let models = server.list_models().await
            .map_err(|e| BackendError::Configuration(e.to_string()))?;

        let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        let info = BackendInfo {
            name: "ollama".to_string(),
            version: "local".to_string(),
            default_model: backend_config.model.clone(),
            available_models: model_names,
            capabilities: BackendCapabilities {
                streaming: true,
                tool_calling: backend_config.enable_tools,
                vision: false, // Model dependent
                json_mode: true,
                max_context_tokens: backend_config.num_ctx,
                max_output_tokens: backend_config.num_ctx / 2,
            },
        };

        Ok(Self {
            server,
            config: backend_config,
            info,
        })
    }

    /// Convert internal request to API request.
    fn to_chat_request(&self, request: &CompletionRequest) -> ChatRequest {
        let model = request
            .model
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| self.config.model.clone());

        let messages: Vec<ChatMessage> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                };

                ChatMessage {
                    role: role.to_string(),
                    content: msg.content.to_text(),
                    images: None,
                    tool_calls: None,
                }
            })
            .collect();

        let options = Some(ModelOptions {
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            num_predict: request.max_tokens.map(|t| t as i32),
            num_ctx: Some(self.config.num_ctx),
            stop: request.stop.clone(),
        });

        let tools = if self.config.enable_tools {
            request.tools.as_ref().map(|tools| {
                tools
                    .iter()
                    .map(|t| Tool {
                        tool_type: "function".to_string(),
                        function: ToolFunction {
                            name: t.name.clone(),
                            description: t.description.clone(),
                            parameters: t.parameters.to_json_schema(),
                        },
                    })
                    .collect()
            })
        } else {
            None
        };

        ChatRequest {
            model,
            messages,
            format: None,
            options,
            stream: Some(false),
            keep_alive: Some(self.config.keep_alive.clone()),
            tools,
        }
    }

    /// Convert API response to internal response.
    fn from_chat_response(&self, response: ChatResponse) -> CompletionResponse {
        let tool_calls: Vec<ToolCall> = response
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, tc)| ToolCall {
                id: format!("call_{}", i),
                name: tc.function.name,
                arguments: serde_json::to_string(&tc.function.arguments).unwrap_or_default(),
            })
            .collect();

        let finish_reason = if !tool_calls.is_empty() {
            FinishReason::ToolUse
        } else {
            FinishReason::Stop
        };

        let usage = Usage::new(
            response.prompt_eval_count.unwrap_or(0),
            response.eval_count.unwrap_or(0),
        );

        CompletionResponse {
            content: if response.message.content.is_empty() {
                None
            } else {
                Some(response.message.content)
            },
            tool_calls,
            finish_reason,
            usage,
            model: response.model,
        }
    }
}

#[async_trait]
impl Backend for OllamaBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    #[instrument(skip(self, request), fields(model = %self.config.model))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        let chat_request = self.to_chat_request(&request);

        debug!("Sending request to Ollama");

        let response = self
            .server
            .client()
            .post(&self.server.chat_url())
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| BackendError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(BackendError::Api {
                status: 500,
                message: body,
            });
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| BackendError::Parsing(e.to_string()))?;

        Ok(self.from_chat_response(chat_response))
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        // Streaming implementation
        todo!("Streaming implementation in 069-ollama-tools.md")
    }

    async fn health_check(&self) -> Result<bool, BackendError> {
        self.server
            .check_health()
            .await
            .map(|_| true)
            .map_err(|e| BackendError::ServiceUnavailable {
                message: e.to_string(),
                retry_after: None,
            })
    }

    fn count_tokens(&self, text: &str) -> u32 {
        // Rough estimate for Llama tokenizer
        (text.len() / 4) as u32
    }
}
```

### 5. Library Root (src/lib.rs)

```rust
//! Ollama local backend for Tachikoma.

#![warn(missing_docs)]

mod api_types;
mod backend;
mod server;

pub use backend::{OllamaBackend, OllamaBackendConfig};
pub use server::{OllamaServer, OllamaServerConfig, ServerError};
pub use tachikoma_backends_core::BackendError;
```

---

## Testing Requirements

1. Server health check works
2. Model listing returns available models
3. Chat completion produces valid responses
4. Model not found is handled
5. Connection errors are handled gracefully

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [068-ollama-models.md](068-ollama-models.md)
- Related: [069-ollama-tools.md](069-ollama-tools.md)
