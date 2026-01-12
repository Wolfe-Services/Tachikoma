# 064 - Gemini API Client (Google)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 064
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config, 020-http-client-foundation
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement the Google Gemini API client that implements the `Backend` trait. This provides access to Gemini Pro, Gemini Ultra, and other Google AI models via the Generative Language API.

---

## Acceptance Criteria

- [ ] `GeminiBackend` implementing `Backend` trait
- [ ] Generative Language API integration
- [ ] Proper authentication handling
- [ ] Request/response type mapping
- [ ] Safety settings configuration
- [ ] Streaming support

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backend-gemini/Cargo.toml)

```toml
[package]
name = "tachikoma-backend-gemini"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Google Gemini backend for Tachikoma"

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

### 2. Gemini Models (src/models.rs)

```rust
//! Gemini model definitions.

use serde::{Deserialize, Serialize};

/// Available Gemini models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeminiModel {
    /// Gemini 1.5 Pro - balanced model
    #[serde(rename = "gemini-1.5-pro")]
    Gemini15Pro,
    /// Gemini 1.5 Pro Latest
    #[serde(rename = "gemini-1.5-pro-latest")]
    Gemini15ProLatest,
    /// Gemini 1.5 Flash - fast model
    #[serde(rename = "gemini-1.5-flash")]
    Gemini15Flash,
    /// Gemini 1.5 Flash Latest
    #[serde(rename = "gemini-1.5-flash-latest")]
    Gemini15FlashLatest,
    /// Gemini 2.0 Flash - next generation
    #[serde(rename = "gemini-2.0-flash")]
    Gemini20Flash,
    /// Gemini Pro (legacy)
    #[serde(rename = "gemini-pro")]
    GeminiPro,
}

impl GeminiModel {
    /// Get the model ID string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemini15Pro => "gemini-1.5-pro",
            Self::Gemini15ProLatest => "gemini-1.5-pro-latest",
            Self::Gemini15Flash => "gemini-1.5-flash",
            Self::Gemini15FlashLatest => "gemini-1.5-flash-latest",
            Self::Gemini20Flash => "gemini-2.0-flash",
            Self::GeminiPro => "gemini-pro",
        }
    }

    /// Get the context window size.
    pub fn context_window(&self) -> u32 {
        match self {
            Self::Gemini15Pro | Self::Gemini15ProLatest => 2_000_000,
            Self::Gemini15Flash | Self::Gemini15FlashLatest => 1_000_000,
            Self::Gemini20Flash => 1_000_000,
            Self::GeminiPro => 32_000,
        }
    }

    /// Get the maximum output tokens.
    pub fn max_output_tokens(&self) -> u32 {
        match self {
            Self::Gemini15Pro | Self::Gemini15ProLatest => 8192,
            Self::Gemini15Flash | Self::Gemini15FlashLatest => 8192,
            Self::Gemini20Flash => 8192,
            Self::GeminiPro => 8192,
        }
    }

    /// Check if the model supports vision.
    pub fn supports_vision(&self) -> bool {
        !matches!(self, Self::GeminiPro)
    }

    /// Check if the model supports tools.
    pub fn supports_tools(&self) -> bool {
        true
    }

    /// Parse from model ID string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "gemini-1.5-pro" => Some(Self::Gemini15Pro),
            "gemini-1.5-pro-latest" => Some(Self::Gemini15ProLatest),
            "gemini-1.5-flash" => Some(Self::Gemini15Flash),
            "gemini-1.5-flash-latest" => Some(Self::Gemini15FlashLatest),
            "gemini-2.0-flash" => Some(Self::Gemini20Flash),
            "gemini-pro" => Some(Self::GeminiPro),
            _ => None,
        }
    }

    /// Get all available models.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Gemini15Pro,
            Self::Gemini15ProLatest,
            Self::Gemini15Flash,
            Self::Gemini15FlashLatest,
            Self::Gemini20Flash,
            Self::GeminiPro,
        ]
    }
}

impl std::fmt::Display for GeminiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for GeminiModel {
    fn default() -> Self {
        Self::Gemini15Pro
    }
}
```

### 3. API Types (src/api_types.rs)

```rust
//! Gemini API request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Request to generateContent endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateContentRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(rename = "safetySettings", skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(rename = "toolConfig", skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
}

/// Content in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// A part of content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCall,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponse,
    },
}

/// Inline data (for images).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String,
}

/// Function call from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: JsonValue,
}

/// Function response to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: JsonValue,
}

/// Generation configuration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "topK", skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Safety setting.
#[derive(Debug, Clone, Serialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

/// Function declaration.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: JsonValue,
}

/// Tool configuration.
#[derive(Debug, Clone, Serialize)]
pub struct ToolConfig {
    #[serde(rename = "functionCallingConfig")]
    pub function_calling_config: FunctionCallingConfig,
}

/// Function calling configuration.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionCallingConfig {
    pub mode: String,
    #[serde(rename = "allowedFunctionNames", skip_serializing_if = "Option::is_none")]
    pub allowed_function_names: Option<Vec<String>>,
}

/// Response from generateContent.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
    #[serde(rename = "promptFeedback")]
    pub prompt_feedback: Option<PromptFeedback>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
}

/// A candidate response.
#[derive(Debug, Clone, Deserialize)]
pub struct Candidate {
    pub content: Option<Content>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Safety rating.
#[derive(Debug, Clone, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Prompt feedback.
#[derive(Debug, Clone, Deserialize)]
pub struct PromptFeedback {
    #[serde(rename = "blockReason")]
    pub block_reason: Option<String>,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Usage metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<u32>,
}

/// Error response.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetail {
    pub code: u32,
    pub message: String,
    pub status: String,
}
```

### 4. Gemini Backend Implementation (src/backend.rs)

```rust
//! Gemini backend implementation.

use crate::api_types::*;
use crate::models::GeminiModel;
use async_trait::async_trait;
use reqwest::Client;
use tachikoma_backends_core::{
    Backend, BackendCapabilities, BackendError, BackendInfo,
    CompletionRequest, CompletionResponse, CompletionStream,
    FinishReason, Message, Role, ToolCall, ToolChoice, Usage,
};
use tachikoma_common_config::Secret;
use tracing::{debug, instrument};

/// Gemini backend implementation.
#[derive(Debug)]
pub struct GeminiBackend {
    client: Client,
    config: GeminiBackendConfig,
    info: BackendInfo,
}

/// Configuration for the Gemini backend.
#[derive(Debug, Clone)]
pub struct GeminiBackendConfig {
    pub api_key: Secret<String>,
    pub base_url: String,
    pub model: GeminiModel,
    pub max_output_tokens: u32,
    pub safety_settings: Vec<SafetySetting>,
}

impl GeminiBackend {
    /// Create a new Gemini backend.
    pub fn new(config: GeminiBackendConfig) -> Result<Self, BackendError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| BackendError::Configuration(e.to_string()))?;

        let info = BackendInfo {
            name: "gemini".to_string(),
            version: "v1beta".to_string(),
            default_model: config.model.to_string(),
            available_models: GeminiModel::all().iter().map(|m| m.to_string()).collect(),
            capabilities: BackendCapabilities {
                streaming: true,
                tool_calling: config.model.supports_tools(),
                vision: config.model.supports_vision(),
                json_mode: true,
                max_context_tokens: config.model.context_window(),
                max_output_tokens: config.model.max_output_tokens(),
            },
        };

        Ok(Self { client, config, info })
    }

    /// Get the generate content endpoint URL.
    fn generate_url(&self, model: &str) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.config.base_url.trim_end_matches('/'),
            model,
            self.config.api_key.expose()
        )
    }

    /// Get the stream generate content endpoint URL.
    fn stream_url(&self, model: &str) -> String {
        format!(
            "{}/v1beta/models/{}:streamGenerateContent?key={}",
            self.config.base_url.trim_end_matches('/'),
            model,
            self.config.api_key.expose()
        )
    }

    /// Convert internal request to API request.
    fn to_api_request(&self, request: &CompletionRequest) -> GenerateContentRequest {
        let (system, contents) = self.convert_messages(&request.messages);

        let generation_config = Some(GenerationConfig {
            temperature: request.temperature,
            top_p: request.top_p,
            top_k: None,
            max_output_tokens: request.max_tokens.or(Some(self.config.max_output_tokens)),
            stop_sequences: request.stop.clone(),
        });

        let tools = request.tools.as_ref().map(|tools| {
            vec![Tool {
                function_declarations: tools
                    .iter()
                    .map(|t| FunctionDeclaration {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.to_json_schema(),
                    })
                    .collect(),
            }]
        });

        let tool_config = request.tool_choice.as_ref().map(|tc| {
            let mode = match tc {
                ToolChoice::Auto => "AUTO",
                ToolChoice::Required => "ANY",
                ToolChoice::None => "NONE",
                ToolChoice::Tool { .. } => "ANY",
            };
            ToolConfig {
                function_calling_config: FunctionCallingConfig {
                    mode: mode.to_string(),
                    allowed_function_names: match tc {
                        ToolChoice::Tool { name } => Some(vec![name.clone()]),
                        _ => None,
                    },
                },
            }
        });

        GenerateContentRequest {
            contents,
            system_instruction: system,
            generation_config,
            safety_settings: if self.config.safety_settings.is_empty() {
                None
            } else {
                Some(self.config.safety_settings.clone())
            },
            tools,
            tool_config,
        }
    }

    /// Convert messages to API format.
    fn convert_messages(&self, messages: &[Message]) -> (Option<Content>, Vec<Content>) {
        let mut system = None;
        let mut contents = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system = Some(Content {
                        role: "user".to_string(),
                        parts: vec![Part::Text {
                            text: msg.content.to_text(),
                        }],
                    });
                }
                Role::User => {
                    contents.push(Content {
                        role: "user".to_string(),
                        parts: self.convert_content(&msg.content),
                    });
                }
                Role::Assistant => {
                    contents.push(Content {
                        role: "model".to_string(),
                        parts: self.convert_content(&msg.content),
                    });
                }
                Role::Tool => {
                    contents.push(Content {
                        role: "function".to_string(),
                        parts: vec![Part::FunctionResponse {
                            function_response: FunctionResponse {
                                name: msg.name.clone().unwrap_or_default(),
                                response: serde_json::json!({
                                    "result": msg.content.to_text()
                                }),
                            },
                        }],
                    });
                }
            }
        }

        (system, contents)
    }

    /// Convert message content to parts.
    fn convert_content(&self, content: &tachikoma_backends_core::MessageContent) -> Vec<Part> {
        match content {
            tachikoma_backends_core::MessageContent::Text(s) => {
                vec![Part::Text { text: s.clone() }]
            }
            tachikoma_backends_core::MessageContent::Parts(parts) => {
                parts
                    .iter()
                    .filter_map(|p| match p {
                        tachikoma_backends_core::ContentPart::Text { text } => {
                            Some(Part::Text { text: text.clone() })
                        }
                        tachikoma_backends_core::ContentPart::Image { source } => {
                            match source {
                                tachikoma_backends_core::ImageSource::Base64 { media_type, data } => {
                                    Some(Part::InlineData {
                                        inline_data: InlineData {
                                            mime_type: media_type.clone(),
                                            data: data.clone(),
                                        },
                                    })
                                }
                                _ => None,
                            }
                        }
                    })
                    .collect()
            }
        }
    }

    /// Convert API response to internal response.
    fn from_api_response(&self, response: GenerateContentResponse, model: &str) -> CompletionResponse {
        let candidate = response.candidates.and_then(|c| c.into_iter().next());

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(candidate) = &candidate {
            if let Some(c) = &candidate.content {
                for part in &c.parts {
                    match part {
                        Part::Text { text } => {
                            content.push_str(text);
                        }
                        Part::FunctionCall { function_call } => {
                            tool_calls.push(ToolCall {
                                id: format!("call_{}", tool_calls.len()),
                                name: function_call.name.clone(),
                                arguments: serde_json::to_string(&function_call.args)
                                    .unwrap_or_default(),
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        let finish_reason = candidate
            .and_then(|c| c.finish_reason)
            .map(|r| match r.as_str() {
                "STOP" => FinishReason::Stop,
                "MAX_TOKENS" => FinishReason::Length,
                "SAFETY" => FinishReason::ContentFilter,
                "RECITATION" => FinishReason::ContentFilter,
                _ => FinishReason::Stop,
            })
            .unwrap_or(FinishReason::Stop);

        let usage = response
            .usage_metadata
            .map(|u| {
                Usage::new(
                    u.prompt_token_count.unwrap_or(0),
                    u.candidates_token_count.unwrap_or(0),
                )
            })
            .unwrap_or_default();

        CompletionResponse {
            content: if content.is_empty() { None } else { Some(content) },
            tool_calls,
            finish_reason,
            usage,
            model: model.to_string(),
        }
    }

    /// Handle error response.
    fn handle_error(&self, status: u16, body: &str) -> BackendError {
        if let Ok(api_error) = serde_json::from_str::<ApiError>(body) {
            match api_error.error.status.as_str() {
                "UNAUTHENTICATED" => BackendError::Authentication(api_error.error.message),
                "PERMISSION_DENIED" => BackendError::Authentication(api_error.error.message),
                "RESOURCE_EXHAUSTED" => BackendError::RateLimit {
                    retry_after: None,
                    message: api_error.error.message,
                },
                "INVALID_ARGUMENT" => BackendError::InvalidRequest(api_error.error.message),
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
impl Backend for GeminiBackend {
    fn info(&self) -> &BackendInfo {
        &self.info
    }

    #[instrument(skip(self, request), fields(model = %self.config.model))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError> {
        let model = request
            .model
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| self.config.model.to_string());

        let api_request = self.to_api_request(&request);

        debug!("Sending request to Gemini API");

        let response = self
            .client
            .post(&self.generate_url(&model))
            .json(&api_request)
            .send()
            .await
            .map_err(|e| BackendError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(self.handle_error(status.as_u16(), &error_body));
        }

        let api_response: GenerateContentResponse = response
            .json()
            .await
            .map_err(|e| BackendError::Parsing(e.to_string()))?;

        Ok(self.from_api_response(api_response, &model))
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        // Streaming implementation
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
        (text.len() / 4) as u32
    }
}
```

### 5. Library Root (src/lib.rs)

```rust
//! Google Gemini backend for Tachikoma.

#![warn(missing_docs)]

mod api_types;
mod backend;
mod models;

pub use backend::{GeminiBackend, GeminiBackendConfig};
pub use models::GeminiModel;
pub use tachikoma_backends_core::BackendError;
```

---

## Testing Requirements

1. Request conversion produces valid API format
2. Response parsing handles all part types
3. Function calls are extracted correctly
4. Safety settings are applied
5. Error responses are categorized

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [065-gemini-auth.md](065-gemini-auth.md)
- Related: [066-gemini-tools.md](066-gemini-tools.md)
