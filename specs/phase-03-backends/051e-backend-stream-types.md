# 051e - Backend Stream Types

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 051e
**Status:** Planned
**Dependencies:** 051d-backend-trait
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define streaming response types for real-time completion output, including chunk collection into full responses.

---

## Acceptance Criteria

- [ ] `CompletionChunk` for stream chunks
- [ ] `ToolCallDelta` for streaming tool calls
- [ ] `CompletionStream` type alias
- [ ] `CollectingStream` for full response assembly

---

## Implementation Details

### 1. Stream Types (src/stream.rs)

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
            content: if self.content.is_empty() { None } else { Some(self.content) },
            tool_calls,
            finish_reason: self.finish_reason.unwrap_or(crate::completion::FinishReason::Stop),
            usage: self.usage.unwrap_or_default(),
            model: String::new(),
        })
    }
}
```

---

## Testing Requirements

1. Stream collection produces correct responses
2. Tool call deltas are assembled correctly
3. Final chunk usage is captured

---

## Related Specs

- Depends on: [051d-backend-trait.md](051d-backend-trait.md)
- Next: [052-backend-config.md](052-backend-config.md)
