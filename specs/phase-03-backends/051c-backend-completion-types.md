# 051c - Backend Completion Types

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051c
**Status:** Planned
**Dependencies:** 051b-backend-message-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define completion request and response types for LLM API calls, including tool choice and usage tracking.

---

## Acceptance Criteria

- [ ] `CompletionRequest` with builder pattern
- [ ] `CompletionResponse` with content and tool calls
- [ ] `Usage` for token counting
- [ ] `FinishReason` enum

---

## Implementation Details

### 1. Completion Types (src/completion.rs)

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

---

## Testing Requirements

1. CompletionRequest builder pattern works
2. Usage calculation is accurate
3. Serialization matches API formats

---

## Related Specs

- Depends on: [051b-backend-message-types.md](051b-backend-message-types.md)
- Next: [051d-backend-trait.md](051d-backend-trait.md)
