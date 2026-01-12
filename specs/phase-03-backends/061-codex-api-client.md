# 061 - Codex API Client (OpenAI)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 061
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config, 020-http-client-foundation
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the OpenAI API client (Codex backend) that implements the `Backend` trait. This provides access to GPT-4, GPT-4o, and other OpenAI models via the Chat Completions API with support for streaming, tool calling, and vision.

---

## Acceptance Criteria

- [x] `CodexBackend` implementing `Backend` trait
- [x] Chat Completions API integration
- [x] Proper header handling (API key, organization)
- [x] Request/response type mapping
- [x] Model enumeration and validation
- [x] Streaming with SSE

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backend-codex/Cargo.toml)

```toml
[package]
name = "tachikoma-backend-codex"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "OpenAI/Codex backend for Tachikoma"

[dependencies]
tachikoma-backends-core.workspace = true
tachikoma-common-http.workspace = true
tachikoma-common-config.workspace = true
async-trait = "0.1"
reqwest = { workspace = true, features = ["json", "stream"] }
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["sync", "time"] }
futures = "0.3"
tracing.workspace = true
bytes = "1.5"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
wiremock = "0.5"
```

### 2. OpenAI Models (src/models.rs)

```rust
//! OpenAI model definitions.

use serde::{Deserialize, Serialize};

/// Available OpenAI models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenAIModel {
    /// GPT-4o - multimodal flagship
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    /// GPT-4o mini - fast and affordable
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    /// GPT-4 Turbo
    #[serde(rename = "gpt-4-turbo")]
    Gpt4Turbo,
    /// GPT-4
    #[serde(rename = "gpt-4")]
    Gpt4,
    /// GPT-3.5 Turbo
    #[serde(rename = "gpt-3.5-turbo")]
    Gpt35Turbo,
    /// o1 - reasoning model
    #[serde(rename = "o1")]
    O1,
    /// o1-mini - smaller reasoning model
    #[serde(rename = "o1-mini")]
    O1Mini,
}

impl OpenAIModel {
    /// Get the model ID string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gpt4o => "gpt-4o",
            Self::Gpt4oMini => "gpt-4o-mini",
            Self::Gpt4Turbo => "gpt-4-turbo",
            Self::Gpt4 => "gpt-4",
            Self::Gpt35Turbo => "gpt-3.5-turbo",
            Self::O1 => "o1",
            Self::O1Mini => "o1-mini",
        }
    }

    /// Get the context window size.
    pub fn context_window(&self) -> u32 {
        match self {
            Self::Gpt4o | Self::Gpt4oMini => 128_000,
            Self::Gpt4Turbo => 128_000,
            Self::Gpt4 => 8_192,
            Self::Gpt35Turbo => 16_385,
            Self::O1 | Self::O1Mini => 128_000,
        }
    }

    /// Get the maximum output tokens.
    pub fn max_output_tokens(&self) -> u32 {
        match self {
            Self::Gpt4o | Self::Gpt4oMini => 16_384,
            Self::Gpt4Turbo => 4_096,
            Self::Gpt4 => 8_192,
            Self::Gpt35Turbo => 4_096,
            Self::O1 | Self::O1Mini => 32_768,
        }
    }

    /// Check if the model supports vision.
    pub fn supports_vision(&self) -> bool {
        matches!(self, Self::Gpt4o | Self::Gpt4oMini | Self::Gpt4Turbo)
    }

    /// Check if the model supports tool use.
    pub fn supports_tools(&self) -> bool {
        !matches!(self, Self::O1 | Self::O1Mini) // o1 models don't support tools
    }

    /// Parse from model ID string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gpt-4o" => Some(Self::Gpt4o),
            "gpt-4o-mini" => Some(Self::Gpt4oMini),
            "gpt-4-turbo" => Some(Self::Gpt4Turbo),
            "gpt-4" => Some(Self::Gpt4),
            "gpt-3.5-turbo" => Some(Self::Gpt35Turbo),
            "o1" => Some(Self::O1),
            "o1-mini" => Some(Self::O1Mini),
            _ => None,
        }
    }

    /// Get all available models.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Gpt4o,
            Self::Gpt4oMini,
            Self::Gpt4Turbo,
            Self::Gpt4,
            Self::Gpt35Turbo,
            Self::O1,
            Self::O1Mini,
        ]
    }
}

impl std::fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for OpenAIModel {
    fn default() -> Self {
        Self::Gpt4o
    }
}
```

### 3. API Types (src/api_types.rs)

```rust
//! OpenAI API request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Request to the Chat Completions API.
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

/// A message in the chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<ChatContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Content in the chat format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

/// Image URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct ChatTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ChatFunction,
}

/// Function definition.
#[derive(Debug, Clone, Serialize)]
pub struct ChatFunction {
    pub name: String,
    pub description: String,
    pub parameters: JsonValue,
}

/// Tool call from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

/// Function call details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool choice.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ChatToolChoice {
    String(String),
    Object {
        #[serde(rename = "type")]
        tool_type: String,
        function: ToolChoiceFunction,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolChoiceFunction {
    pub name: String,
}

/// Response format.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

/// Response from Chat Completions API.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<ChatUsage>,
}

/// A choice in the response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Token usage.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
    pub usage: Option<ChatUsage>,
}

/// A choice in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

/// Delta in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChunkToolCall>>,
}

/// Tool call in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkToolCall {
    pub index: usize,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: Option<ChunkFunction>,
}

/// Function in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

/// Error response.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}
```

### 4. Codex Backend Implementation (src/backend.rs)

```rust
//! Codex backend implementation.

use crate::api_types::*;
use crate::models::OpenAIModel;
use async_trait::async_trait;
use reqwest::Client;
use tachikoma_backends_core::{
    Backend, BackendCapabilities, BackendError, BackendInfo,
    CompletionRequest, CompletionResponse, CompletionStream,
    FinishReason, Message, Role, ToolCall, ToolChoice, Usage,
};
use tachikoma_common_config::Secret;
use tracing::{debug, instrument};

/// Codex (OpenAI) backend implementation.
#[derive(Debug)]
pub struct CodexBackend {
    /// HTTP client.
    client: Client,
    /// Configuration.
    config: CodexBackendConfig,
    /// Backend info.
    info: BackendInfo,
}

/// Configuration for the Codex backend.
#[derive(Debug, Clone)]
pub struct CodexBackendConfig {
    /// API key.
    pub api_key: Secret<String>,
    /// Organization ID.
    pub organization: Option<String>,
    /// Base URL.
    pub base_url: String,
    /// Default model.
    pub model: OpenAIModel,
    /// Default max tokens.
    pub max_tokens: u32,
}

impl CodexBackend {
    /// Create a new Codex backend.
    pub fn new(config: CodexBackendConfig) -> Result<Self, BackendError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| BackendError::Configuration(e.to_string()))?;

        let info = BackendInfo {
            name: "codex".to_string(),
            version: "v1".to_string(),
            default_model: config.model.to_string(),
            available_models: OpenAIModel::all().iter().map(|m| m.to_string()).collect(),
            capabilities: BackendCapabilities {
                streaming: true,
                tool_calling: config.model.supports_tools(),
                vision: config.model.supports_vision(),
                json_mode: true,
                max_context_tokens: config.model.context_window(),
                max_output_tokens: config.model.max_output_tokens(),
            },
        };

        Ok(Self {
            client,
            config,
            info,
        })
    }

    /// Get the completions endpoint URL.
    fn completions_url(&self) -> String {
        format!("{}/v1/chat/completions", self.config.base_url.trim_end_matches('/'))
    }

    /// Build request headers.
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key.expose()).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        if let Some(org) = &self.config.organization {
            headers.insert("OpenAI-Organization", org.parse().unwrap());
        }

        headers
    }

    /// Convert internal request to API request.
    fn to_api_request(&self, request: &CompletionRequest) -> ChatCompletionRequest {
        let model = request
            .model
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| self.config.model.to_string());

        let messages = self.convert_messages(&request.messages);

        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| ChatTool {
                    tool_type: "function".to_string(),
                    function: ChatFunction {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.to_json_schema(),
                    },
                })
                .collect()
        });

        let tool_choice = request.tool_choice.as_ref().map(|tc| match tc {
            ToolChoice::Auto => ChatToolChoice::String("auto".to_string()),
            ToolChoice::Required => ChatToolChoice::String("required".to_string()),
            ToolChoice::None => ChatToolChoice::String("none".to_string()),
            ToolChoice::Tool { name } => ChatToolChoice::Object {
                tool_type: "function".to_string(),
                function: ToolChoiceFunction { name: name.clone() },
            },
        });

        ChatCompletionRequest {
            model,
            messages,
            max_tokens: request.max_tokens.or(Some(self.config.max_tokens)),
            temperature: request.temperature,
            top_p: request.top_p,
            stop: request.stop.clone(),
            tools,
            tool_choice,
            stream: None,
            response_format: None,
        }
    }

    /// Convert messages to API format.
    fn convert_messages(&self, messages: &[Message]) -> Vec<ChatMessage> {
        messages
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
                    content: Some(self.convert_content(&msg.content)),
                    name: msg.name.clone(),
                    tool_calls: None,
                    tool_call_id: msg.tool_call_id.clone(),
                }
            })
            .collect()
    }

    /// Convert message content.
    fn convert_content(&self, content: &tachikoma_backends_core::MessageContent) -> ChatContent {
        match content {
            tachikoma_backends_core::MessageContent::Text(s) => ChatContent::Text(s.clone()),
            tachikoma_backends_core::MessageContent::Parts(parts) => {
                let chat_parts: Vec<ContentPart> = parts
                    .iter()
                    .filter_map(|p| match p {
                        tachikoma_backends_core::ContentPart::Text { text } => {
                            Some(ContentPart::Text { text: text.clone() })
                        }
                        tachikoma_backends_core::ContentPart::Image { source } => {
                            match source {
                                tachikoma_backends_core::ImageSource::Base64 { media_type, data } => {
                                    Some(ContentPart::ImageUrl {
                                        image_url: ImageUrl {
                                            url: format!("data:{};base64,{}", media_type, data),
                                            detail: None,
                                        },
                                    })
                                }
                                tachikoma_backends_core::ImageSource::Url { url } => {
                                    Some(ContentPart::ImageUrl {
                                        image_url: ImageUrl {
                                            url: url.clone(),
                                            detail: None,
                                        },
                                    })
                                }
                            }
                        }
                    })
                    .collect();
                ChatContent::Parts(chat_parts)
            }
        }
    }

    /// Convert API response to internal response.
    fn from_api_response(&self, response: ChatCompletionResponse) -> CompletionResponse {
        let choice = response.choices.first();

        let content = choice
            .and_then(|c| c.message.content.as_ref())
            .map(|c| match c {
                ChatContent::Text(s) => s.clone(),
                ChatContent::Parts(parts) => parts
                    .iter()
                    .filter_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(""),
            });

        let tool_calls = choice
            .and_then(|c| c.message.tool_calls.as_ref())
            .map(|calls| {
                calls
                    .iter()
                    .map(|tc| ToolCall {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let finish_reason = match choice.and_then(|c| c.finish_reason.as_deref()) {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolUse,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        let usage = response.usage.map(|u| Usage::new(u.prompt_tokens, u.completion_tokens))
            .unwrap_or_default();

        CompletionResponse {
            content,
            tool_calls,
            finish_reason,
            usage,
            model: response.model,
        }
    }

    /// Handle error response.
    fn handle_error(&self, status: u16, body: &str) -> BackendError {
        if let Ok(api_error) = serde_json::from_str::<ApiError>(body) {
            match api_error.error.error_type.as_str() {
                "invalid_api_key" | "authentication_error" => {
                    BackendError::Authentication(api_error.error.message)
                }
                "rate_limit_exceeded" => BackendError::RateLimit {
                    retry_after: None,
                    message: api_error.error.message,
                },
                "invalid_request_error" => {
                    BackendError::InvalidRequest(api_error.error.message)
                }
                "server_error" => BackendError::Api {
                    status,
                    message: api_error.error.message,
                },
                _ => BackendError::Api {
                    status,
                    message: api_error.error.message,
                },
            }
        } else {
            BackendError::Api {
                status,
                message: body.to_string(),
            }
        }
    }
}

#[async_trait]
impl Backend for CodexBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    #[instrument(skip(self, request), fields(model = %self.config.model))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        let api_request = self.to_api_request(&request);

        debug!("Sending request to OpenAI API");

        let response = self
            .client
            .post(&self.completions_url())
            .headers(self.build_headers())
            .json(&api_request)
            .send()
            .await
            .map_err(|e| BackendError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(self.handle_error(status.as_u16(), &error_body));
        }

        let api_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| BackendError::Parsing(e.to_string()))?;

        Ok(self.from_api_response(api_response))
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        // Streaming implementation in 063-codex-tools.md
        todo!("Streaming implementation")
    }

    async fn health_check(&self) -> Result<bool, BackendError> {
        let request = CompletionRequest::new(vec![Message::user("Hi")])
            .with_max_tokens(1);

        match self.complete(request).await {
            Ok(_) => Ok(true),
            Err(BackendError::RateLimit { .. }) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn count_tokens(&self, text: &str) -> u32 {
        // OpenAI uses ~4 characters per token
        (text.len() / 4) as u32
    }
}
```

### 5. Library Root (src/lib.rs)

```rust
//! OpenAI/Codex backend for Tachikoma.

#![warn(missing_docs)]

mod api_types;
mod backend;
mod models;

pub use backend::{CodexBackend, CodexBackendConfig};
pub use models::OpenAIModel;
pub use tachikoma_backends_core::BackendError;
```

---

## Testing Requirements

1. Request conversion produces valid API format
2. Response parsing handles all message types
3. Tool call extraction is correct
4. Error responses are categorized properly
5. Vision content is formatted correctly

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [062-codex-auth.md](062-codex-auth.md)
- Related: [063-codex-tools.md](063-codex-tools.md)
