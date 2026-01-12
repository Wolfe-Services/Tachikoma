# 056 - Claude API Client

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 056
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config, 020-http-client-foundation
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the Anthropic Claude API client that implements the `Backend` trait. This provides access to Claude models (Opus, Sonnet, Haiku) via the Messages API with support for streaming, tool calling, and vision.

---

## Acceptance Criteria

- [ ] `ClaudeBackend` implementing `Backend` trait
- [ ] Messages API integration
- [ ] Proper header handling (API key, version, beta features)
- [ ] Request/response type mapping
- [ ] Model enumeration and validation
- [ ] Connection management

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backend-claude/Cargo.toml)

```toml
[package]
name = "tachikoma-backend-claude"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Anthropic Claude backend for Tachikoma"

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
eventsource-stream = "0.2"
pin-project-lite = "0.2"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
wiremock = "0.5"
```

### 2. Claude Models (src/models.rs)

```rust
//! Claude model definitions.

use serde::{Deserialize, Serialize};

/// Available Claude models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaudeModel {
    /// Claude Opus 4 - highest capability
    #[serde(rename = "claude-opus-4-20250514")]
    Opus4,
    /// Claude Sonnet 4 - balanced performance
    #[serde(rename = "claude-sonnet-4-20250514")]
    Sonnet4,
    /// Claude 3.5 Sonnet - previous generation balanced
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    Sonnet35,
    /// Claude 3.5 Haiku - fast and efficient
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Haiku35,
    /// Claude 3 Opus - previous flagship
    #[serde(rename = "claude-3-opus-20240229")]
    Opus3,
}

impl ClaudeModel {
    /// Get the model ID string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Opus4 => "claude-opus-4-20250514",
            Self::Sonnet4 => "claude-sonnet-4-20250514",
            Self::Sonnet35 => "claude-3-5-sonnet-20241022",
            Self::Haiku35 => "claude-3-5-haiku-20241022",
            Self::Opus3 => "claude-3-opus-20240229",
        }
    }

    /// Get the context window size.
    pub fn context_window(&self) -> u32 {
        match self {
            Self::Opus4 | Self::Sonnet4 | Self::Sonnet35 => 200_000,
            Self::Haiku35 => 200_000,
            Self::Opus3 => 200_000,
        }
    }

    /// Get the maximum output tokens.
    pub fn max_output_tokens(&self) -> u32 {
        match self {
            Self::Opus4 | Self::Sonnet4 | Self::Sonnet35 => 8192,
            Self::Haiku35 => 8192,
            Self::Opus3 => 4096,
        }
    }

    /// Check if the model supports vision.
    pub fn supports_vision(&self) -> bool {
        true // All Claude 3+ models support vision
    }

    /// Check if the model supports tool use.
    pub fn supports_tools(&self) -> bool {
        true // All Claude 3+ models support tools
    }

    /// Parse from model ID string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude-opus-4-20250514" => Some(Self::Opus4),
            "claude-sonnet-4-20250514" => Some(Self::Sonnet4),
            "claude-3-5-sonnet-20241022" => Some(Self::Sonnet35),
            "claude-3-5-haiku-20241022" => Some(Self::Haiku35),
            "claude-3-opus-20240229" => Some(Self::Opus3),
            _ => None,
        }
    }

    /// Get all available models.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Opus4,
            Self::Sonnet4,
            Self::Sonnet35,
            Self::Haiku35,
            Self::Opus3,
        ]
    }
}

impl std::fmt::Display for ClaudeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for ClaudeModel {
    fn default() -> Self {
        Self::Sonnet4
    }
}
```

### 3. API Types (src/api_types.rs)

```rust
//! Claude API request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Request to the Messages API.
#[derive(Debug, Clone, Serialize)]
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<ApiMessage>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ApiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ApiToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// A message in the API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: ApiContent,
}

/// Content in the API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

/// A content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: JsonValue,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Image source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// Tool definition for the API.
#[derive(Debug, Clone, Serialize)]
pub struct ApiTool {
    pub name: String,
    pub description: String,
    pub input_schema: JsonValue,
}

/// Tool choice.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ApiToolChoice {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "tool")]
    Tool { name: String },
}

/// Response from the Messages API.
#[derive(Debug, Clone, Deserialize)]
pub struct MessagesResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: ApiUsage,
}

/// Token usage.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Error response.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
```

### 4. Claude Backend Implementation (src/backend.rs)

```rust
//! Claude backend implementation.

use crate::api_types::*;
use crate::models::ClaudeModel;
use async_trait::async_trait;
use reqwest::Client;
use tachikoma_backends_core::{
    Backend, BackendCapabilities, BackendError, BackendInfo,
    CompletionRequest, CompletionResponse, CompletionStream,
    FinishReason, Message, Role, ToolCall, ToolChoice, Usage,
};
use tachikoma_common_config::Secret;
use tracing::{debug, instrument, warn};

/// Claude backend implementation.
#[derive(Debug)]
pub struct ClaudeBackend {
    /// HTTP client.
    client: Client,
    /// API configuration.
    config: ClaudeBackendConfig,
    /// Backend info.
    info: BackendInfo,
}

/// Configuration for the Claude backend.
#[derive(Debug, Clone)]
pub struct ClaudeBackendConfig {
    /// API key.
    pub api_key: Secret<String>,
    /// Base URL.
    pub base_url: String,
    /// Default model.
    pub model: ClaudeModel,
    /// API version.
    pub api_version: String,
    /// Default max tokens.
    pub max_tokens: u32,
}

impl ClaudeBackend {
    /// Create a new Claude backend.
    pub fn new(config: ClaudeBackendConfig) -> Result<Self, BackendError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| BackendError::Configuration(e.to_string()))?;

        let info = BackendInfo {
            name: "claude".to_string(),
            version: config.api_version.clone(),
            default_model: config.model.to_string(),
            available_models: ClaudeModel::all().iter().map(|m| m.to_string()).collect(),
            capabilities: BackendCapabilities {
                streaming: true,
                tool_calling: true,
                vision: true,
                json_mode: false,
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

    /// Get the messages endpoint URL.
    fn messages_url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'))
    }

    /// Build request headers.
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            self.config.api_key.expose().parse().unwrap(),
        );
        headers.insert(
            "anthropic-version",
            self.config.api_version.parse().unwrap(),
        );
        headers.insert("content-type", "application/json".parse().unwrap());
        headers
    }

    /// Convert internal request to API request.
    fn to_api_request(&self, request: &CompletionRequest) -> MessagesRequest {
        let model = request
            .model
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| self.config.model.to_string());

        let max_tokens = request.max_tokens.unwrap_or(self.config.max_tokens);

        // Extract system message
        let (system, messages) = self.convert_messages(&request.messages);

        // Convert tools
        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|t| ApiTool {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    input_schema: t.parameters.to_json_schema(),
                })
                .collect()
        });

        // Convert tool choice
        let tool_choice = request.tool_choice.as_ref().map(|tc| match tc {
            ToolChoice::Auto => ApiToolChoice::Auto,
            ToolChoice::Required => ApiToolChoice::Any,
            ToolChoice::None => ApiToolChoice::Auto, // Claude doesn't have "none"
            ToolChoice::Tool { name } => ApiToolChoice::Tool { name: name.clone() },
        });

        MessagesRequest {
            model,
            messages,
            max_tokens,
            system,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop.clone(),
            tools,
            tool_choice,
            stream: None,
        }
    }

    /// Convert messages, extracting system message.
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<ApiMessage>) {
        let mut system = None;
        let mut api_messages = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system = Some(msg.content.to_text());
                }
                Role::User => {
                    api_messages.push(ApiMessage {
                        role: "user".to_string(),
                        content: self.convert_content(&msg.content),
                    });
                }
                Role::Assistant => {
                    api_messages.push(ApiMessage {
                        role: "assistant".to_string(),
                        content: self.convert_content(&msg.content),
                    });
                }
                Role::Tool => {
                    // Tool results go as user messages with tool_result blocks
                    let tool_use_id = msg.tool_call_id.clone().unwrap_or_default();
                    api_messages.push(ApiMessage {
                        role: "user".to_string(),
                        content: ApiContent::Blocks(vec![ContentBlock::ToolResult {
                            tool_use_id,
                            content: msg.content.to_text(),
                            is_error: None,
                        }]),
                    });
                }
            }
        }

        (system, api_messages)
    }

    /// Convert message content.
    fn convert_content(&self, content: &tachikoma_backends_core::MessageContent) -> ApiContent {
        match content {
            tachikoma_backends_core::MessageContent::Text(s) => ApiContent::Text(s.clone()),
            tachikoma_backends_core::MessageContent::Parts(parts) => {
                let blocks: Vec<ContentBlock> = parts
                    .iter()
                    .filter_map(|p| match p {
                        tachikoma_backends_core::ContentPart::Text { text } => {
                            Some(ContentBlock::Text { text: text.clone() })
                        }
                        tachikoma_backends_core::ContentPart::Image { source } => {
                            match source {
                                tachikoma_backends_core::ImageSource::Base64 { media_type, data } => {
                                    Some(ContentBlock::Image {
                                        source: ImageSource {
                                            source_type: "base64".to_string(),
                                            media_type: media_type.clone(),
                                            data: data.clone(),
                                        },
                                    })
                                }
                                _ => None,
                            }
                        }
                    })
                    .collect();
                ApiContent::Blocks(blocks)
            }
        }
    }

    /// Convert API response to internal response.
    fn from_api_response(&self, response: MessagesResponse) -> CompletionResponse {
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for block in response.content {
            match block {
                ContentBlock::Text { text } => {
                    content.push_str(&text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: serde_json::to_string(&input).unwrap_or_default(),
                    });
                }
                _ => {}
            }
        }

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolUse,
            _ => FinishReason::Stop,
        };

        CompletionResponse {
            content: if content.is_empty() { None } else { Some(content) },
            tool_calls,
            finish_reason,
            usage: Usage::new(response.usage.input_tokens, response.usage.output_tokens),
            model: response.model,
        }
    }
}

#[async_trait]
impl Backend for ClaudeBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    #[instrument(skip(self, request), fields(model = %self.config.model))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        let api_request = self.to_api_request(&request);

        debug!("Sending request to Claude API");

        let response = self
            .client
            .post(&self.messages_url())
            .headers(self.build_headers())
            .json(&api_request)
            .send()
            .await
            .map_err(|e| BackendError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(self.handle_error_response(status.as_u16(), &error_body));
        }

        let api_response: MessagesResponse = response
            .json()
            .await
            .map_err(|e| BackendError::Parsing(e.to_string()))?;

        Ok(self.from_api_response(api_response))
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        // Streaming implementation in 058-claude-streaming.md
        todo!("See 058-claude-streaming.md for streaming implementation")
    }

    async fn health_check(&self) -> Result<bool, BackendError> {
        // Send a minimal request to check connectivity
        let request = CompletionRequest::new(vec![Message::user("Hi")])
            .with_max_tokens(1);

        match self.complete(request).await {
            Ok(_) => Ok(true),
            Err(BackendError::RateLimit { .. }) => Ok(true), // Rate limited but reachable
            Err(e) => Err(e),
        }
    }

    fn count_tokens(&self, text: &str) -> u32 {
        // Claude uses ~4 characters per token on average
        (text.len() / 4) as u32
    }
}

impl ClaudeBackend {
    /// Handle error responses.
    fn handle_error_response(&self, status: u16, body: &str) -> BackendError {
        if let Ok(api_error) = serde_json::from_str::<ApiError>(body) {
            match api_error.error.error_type.as_str() {
                "rate_limit_error" => BackendError::RateLimit {
                    retry_after: None,
                    message: api_error.error.message,
                },
                "authentication_error" => {
                    BackendError::Authentication(api_error.error.message)
                }
                "invalid_request_error" => {
                    BackendError::InvalidRequest(api_error.error.message)
                }
                "overloaded_error" => BackendError::ServiceUnavailable {
                    message: api_error.error.message,
                    retry_after: None,
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
```

### 5. Library Root (src/lib.rs)

```rust
//! Anthropic Claude backend for Tachikoma.
//!
//! This crate provides the Claude backend implementation for accessing
//! Anthropic's Claude models via the Messages API.

#![warn(missing_docs)]

mod api_types;
mod backend;
mod models;

pub use backend::{ClaudeBackend, ClaudeBackendConfig};
pub use models::ClaudeModel;

// Re-export error types from core
pub use tachikoma_backends_core::BackendError;
```

---

## Testing Requirements

1. Request conversion produces valid API format
2. Response parsing handles all content block types
3. Error responses are correctly categorized
4. Model enumeration returns correct values
5. Header construction includes all required fields

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: [052-backend-config.md](052-backend-config.md)
- Next: [057-claude-auth.md](057-claude-auth.md)
- Related: [058-claude-streaming.md](058-claude-streaming.md)
