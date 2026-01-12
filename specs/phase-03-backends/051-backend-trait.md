# 051 - Backend Trait Definition

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types, 019-async-runtime
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define the core `Backend` trait that abstracts over different LLM providers (Claude, Codex, Gemini, Ollama). This trait provides a unified interface for message completion, streaming responses, and tool calling across all supported backends.

---

## Acceptance Criteria

- [ ] `tachikoma-backends-core` crate created
- [ ] `Backend` trait with async completion methods
- [ ] Support for streaming and non-streaming responses
- [ ] Tool calling abstraction
- [ ] Model capability querying
- [ ] Proper error handling with `BackendError`
- [ ] Send + Sync bounds for concurrent use

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-backends-core/Cargo.toml)

```toml
[package]
name = "tachikoma-backends-core"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core backend traits and types for Tachikoma LLM integration"

[dependencies]
tachikoma-common-core.workspace = true
async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tokio = { workspace = true, features = ["sync"] }
futures = "0.3"
pin-project-lite = "0.2"

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

### 2. Message Types (src/message.rs)

```rust
//! Message types for LLM communication.

use serde::{Deserialize, Serialize};

/// Role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt/instructions.
    System,
    /// User input.
    User,
    /// Assistant response.
    Assistant,
    /// Tool/function result.
    Tool,
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the sender.
    pub role: Role,
    /// Message content.
    pub content: MessageContent,
    /// Optional name (for tool results).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Tool call ID (for tool results).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Content of a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content.
    Text(String),
    /// Multi-part content (text + images).
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    /// Get text content if available.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Parts(parts) => parts.iter().find_map(|p| {
                if let ContentPart::Text { text } = p {
                    Some(text.as_str())
                } else {
                    None
                }
            }),
        }
    }

    /// Convert to string, concatenating all text parts.
    pub fn to_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|p| {
                    if let ContentPart::Text { text } = p {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// A part of multi-part content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content.
    #[serde(rename = "text")]
    Text { text: String },
    /// Image content.
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data or URL.
        source: ImageSource,
    },
}

/// Source of an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    /// Base64-encoded image.
    #[serde(rename = "base64")]
    Base64 {
        media_type: String,
        data: String,
    },
    /// URL to image.
    #[serde(rename = "url")]
    Url { url: String },
}
```

### 3. Completion Types (src/completion.rs)

```rust
//! Completion request and response types.

use crate::message::Message;
use crate::tool::{ToolCall, ToolDefinition};
use serde::{Deserialize, Serialize};

/// A completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Temperature (0.0 - 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Top-p sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Available tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    /// Tool choice strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

impl CompletionRequest {
    /// Create a new completion request.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            model: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stop: None,
            tools: None,
            tool_choice: None,
        }
    }

    /// Set the model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set available tools.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }
}

/// Tool choice strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model decides whether to use tools.
    Auto,
    /// Model must use a tool.
    Required,
    /// Model cannot use tools.
    None,
    /// Model must use a specific tool.
    Tool { name: String },
}

/// A completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated content.
    pub content: Option<String>,
    /// Tool calls requested by the model.
    pub tool_calls: Vec<ToolCall>,
    /// Finish reason.
    pub finish_reason: FinishReason,
    /// Token usage statistics.
    pub usage: Usage,
    /// Model that generated the response.
    pub model: String,
}

/// Reason the generation stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop or stop sequence.
    Stop,
    /// Max tokens reached.
    Length,
    /// Tool use requested.
    ToolUse,
    /// Content filtered.
    ContentFilter,
    /// Error occurred.
    Error,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Input tokens.
    pub prompt_tokens: u32,
    /// Output tokens.
    pub completion_tokens: u32,
    /// Total tokens.
    pub total_tokens: u32,
}

impl Usage {
    /// Create new usage stats.
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}
```

### 4. Backend Trait (src/backend.rs)

```rust
//! Core backend trait definition.

use crate::completion::{CompletionRequest, CompletionResponse};
use crate::error::BackendError;
use crate::stream::CompletionStream;
use async_trait::async_trait;
use std::fmt::Debug;

/// Capabilities supported by a backend.
#[derive(Debug, Clone, Default)]
pub struct BackendCapabilities {
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports tool/function calling.
    pub tool_calling: bool,
    /// Supports vision/images.
    pub vision: bool,
    /// Supports JSON mode.
    pub json_mode: bool,
    /// Maximum context window size.
    pub max_context_tokens: u32,
    /// Maximum output tokens.
    pub max_output_tokens: u32,
}

/// Information about a backend.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Backend name (e.g., "claude", "codex").
    pub name: String,
    /// Backend version.
    pub version: String,
    /// Default model.
    pub default_model: String,
    /// Available models.
    pub available_models: Vec<String>,
    /// Backend capabilities.
    pub capabilities: BackendCapabilities,
}

/// Core trait for LLM backends.
///
/// This trait must be implemented by all LLM providers to enable
/// unified access to different AI models.
#[async_trait]
pub trait Backend: Send + Sync + Debug {
    /// Get backend information.
    fn info(&self) -> &BackendInfo;

    /// Get the backend name.
    fn name(&self) -> &str {
        &self.info().name
    }

    /// Get backend capabilities.
    fn capabilities(&self) -> &BackendCapabilities {
        &self.info().capabilities
    }

    /// Check if the backend supports a specific model.
    fn supports_model(&self, model: &str) -> bool {
        self.info().available_models.iter().any(|m| m == model)
    }

    /// Create a completion (non-streaming).
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, BackendError>;

    /// Create a streaming completion.
    ///
    /// Returns a stream of completion chunks. The final chunk will have
    /// `is_final: true` and contain usage statistics.
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError>;

    /// Check if the backend is healthy and accessible.
    async fn health_check(&self) -> Result<bool, BackendError>;

    /// Count tokens in text (approximate if not supported natively).
    fn count_tokens(&self, text: &str) -> u32 {
        // Default implementation: rough estimate
        (text.len() / 4) as u32
    }
}

/// Extension trait for backends with additional features.
#[async_trait]
pub trait BackendExt: Backend {
    /// Complete with automatic retries on transient errors.
    async fn complete_with_retry(
        &self,
        request: CompletionRequest,
        max_retries: u32,
    ) -> Result<CompletionResponse, BackendError> {
        let mut last_error = None;
        for attempt in 0..=max_retries {
            match self.complete(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) if e.is_retryable() && attempt < max_retries => {
                    let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_error.unwrap())
    }
}

// Blanket implementation
impl<T: Backend> BackendExt for T {}
```

### 5. Stream Types (src/stream.rs)

```rust
//! Streaming response types.

use crate::completion::Usage;
use crate::error::BackendError;
use crate::tool::ToolCall;
use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A chunk from a streaming completion.
#[derive(Debug, Clone)]
pub struct CompletionChunk {
    /// Text delta (may be empty).
    pub delta: String,
    /// Tool call deltas.
    pub tool_calls: Vec<ToolCallDelta>,
    /// Whether this is the final chunk.
    pub is_final: bool,
    /// Usage stats (only in final chunk).
    pub usage: Option<Usage>,
    /// Finish reason (only in final chunk).
    pub finish_reason: Option<crate::completion::FinishReason>,
}

impl CompletionChunk {
    /// Create a text chunk.
    pub fn text(delta: impl Into<String>) -> Self {
        Self {
            delta: delta.into(),
            tool_calls: vec![],
            is_final: false,
            usage: None,
            finish_reason: None,
        }
    }

    /// Create a final chunk.
    pub fn final_chunk(usage: Usage, finish_reason: crate::completion::FinishReason) -> Self {
        Self {
            delta: String::new(),
            tool_calls: vec![],
            is_final: true,
            usage: Some(usage),
            finish_reason: Some(finish_reason),
        }
    }
}

/// Delta update for a tool call.
#[derive(Debug, Clone)]
pub struct ToolCallDelta {
    /// Tool call index.
    pub index: usize,
    /// Tool call ID (only in first delta).
    pub id: Option<String>,
    /// Tool name (only in first delta).
    pub name: Option<String>,
    /// Arguments delta.
    pub arguments_delta: String,
}

/// Type alias for the completion stream.
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<CompletionChunk, BackendError>> + Send>>;

pin_project! {
    /// A stream that collects chunks into a full response.
    pub struct CollectingStream<S> {
        #[pin]
        inner: S,
        content: String,
        tool_calls: Vec<ToolCallBuilder>,
        usage: Option<Usage>,
        finish_reason: Option<crate::completion::FinishReason>,
    }
}

#[derive(Debug, Default)]
struct ToolCallBuilder {
    id: String,
    name: String,
    arguments: String,
}

impl<S> CollectingStream<S>
where
    S: Stream<Item = Result<CompletionChunk, BackendError>>,
{
    /// Create a new collecting stream.
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            content: String::new(),
            tool_calls: vec![],
            usage: None,
            finish_reason: None,
        }
    }

    /// Collect the full response.
    pub async fn collect(mut self) -> Result<crate::completion::CompletionResponse, BackendError>
    where
        S: Unpin,
    {
        use futures::StreamExt;

        while let Some(chunk) = self.inner.next().await {
            let chunk = chunk?;
            self.content.push_str(&chunk.delta);

            for delta in chunk.tool_calls {
                if delta.index >= self.tool_calls.len() {
                    self.tool_calls.resize_with(delta.index + 1, Default::default);
                }
                let builder = &mut self.tool_calls[delta.index];
                if let Some(id) = delta.id {
                    builder.id = id;
                }
                if let Some(name) = delta.name {
                    builder.name = name;
                }
                builder.arguments.push_str(&delta.arguments_delta);
            }

            if chunk.is_final {
                self.usage = chunk.usage;
                self.finish_reason = chunk.finish_reason;
            }
        }

        let tool_calls: Vec<ToolCall> = self
            .tool_calls
            .into_iter()
            .filter(|b| !b.id.is_empty())
            .map(|b| ToolCall {
                id: b.id,
                name: b.name,
                arguments: b.arguments,
            })
            .collect();

        Ok(crate::completion::CompletionResponse {
            content: if self.content.is_empty() {
                None
            } else {
                Some(self.content)
            },
            tool_calls,
            finish_reason: self.finish_reason.unwrap_or(crate::completion::FinishReason::Stop),
            usage: self.usage.unwrap_or_default(),
            model: String::new(), // Set by caller
        })
    }
}
```

### 6. Library Root (src/lib.rs)

```rust
//! Tachikoma Backend Core
//!
//! This crate provides the core traits and types for LLM backend integration.
//! All backend implementations (Claude, Codex, Gemini, Ollama) implement the
//! `Backend` trait defined here.

#![warn(missing_docs)]

pub mod backend;
pub mod completion;
pub mod error;
pub mod message;
pub mod stream;
pub mod tool;

pub use backend::{Backend, BackendCapabilities, BackendExt, BackendInfo};
pub use completion::{CompletionRequest, CompletionResponse, FinishReason, ToolChoice, Usage};
pub use error::BackendError;
pub use message::{ContentPart, ImageSource, Message, MessageContent, Role};
pub use stream::{CollectingStream, CompletionChunk, CompletionStream, ToolCallDelta};
pub use tool::{ToolCall, ToolDefinition, ToolParameter, ToolResult};
```

---

## Testing Requirements

1. Message creation helpers work correctly
2. CompletionRequest builder pattern functions
3. Usage calculation is accurate
4. Stream collection produces correct responses
5. Backend trait is object-safe

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Depends on: [012-error-types.md](../phase-01-common/012-error-types.md)
- Next: [052-backend-config.md](052-backend-config.md)
- Used by: [056-claude-api-client.md](056-claude-api-client.md), [061-codex-api-client.md](061-codex-api-client.md)
