# 058 - Claude Streaming (SSE)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 058
**Status:** Planned
**Dependencies:** 056-claude-api-client
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Server-Sent Events (SSE) streaming for the Claude backend to enable real-time response streaming. This provides progressive output display and tool call streaming.

---

## Acceptance Criteria

- [ ] SSE stream parsing from Claude API
- [ ] Delta event handling (text, tool calls)
- [ ] Stream error handling and recovery
- [ ] Proper stream cleanup on cancellation
- [ ] Token usage from stream end event
- [ ] Backpressure handling

---

## Implementation Details

### 1. Stream Event Types (src/streaming/events.rs)

```rust
//! SSE event types from Claude API.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// SSE event from Claude streaming API.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Initial message start.
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStartData },

    /// Content block start.
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlockData,
    },

    /// Content block delta (incremental update).
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },

    /// Content block stop.
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },

    /// Message delta (final metadata).
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaData,
        usage: Option<StreamUsage>,
    },

    /// Message stop (stream end).
    #[serde(rename = "message_stop")]
    MessageStop,

    /// Ping (keepalive).
    #[serde(rename = "ping")]
    Ping,

    /// Error event.
    #[serde(rename = "error")]
    Error { error: StreamError },
}

/// Data in message_start event.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStartData {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub model: String,
    pub usage: StreamUsage,
}

/// Content block data.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockData {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: JsonValue },
}

/// Content delta (incremental update).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
}

/// Message delta data.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaData {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

/// Stream usage statistics.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct StreamUsage {
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
}

/// Stream error.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
```

### 2. Stream Parser (src/streaming/parser.rs)

```rust
//! SSE stream parser.

use super::events::StreamEvent;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{debug, trace, warn};

/// Parse SSE events from a byte stream.
pub struct SseParser<S> {
    inner: S,
    buffer: String,
}

impl<S> SseParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new SSE parser.
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: String::new(),
        }
    }

    /// Parse a single SSE event from the buffer.
    fn parse_event(&mut self) -> Option<Result<StreamEvent, StreamParseError>> {
        // SSE events are separated by double newlines
        let event_end = self.buffer.find("\n\n")?;
        let event_data = self.buffer[..event_end].to_string();
        self.buffer = self.buffer[event_end + 2..].to_string();

        // Parse the event
        let mut event_type = None;
        let mut data = String::new();

        for line in event_data.lines() {
            if let Some(value) = line.strip_prefix("event: ") {
                event_type = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("data: ") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(value);
            }
        }

        if data.is_empty() {
            return None;
        }

        trace!(event_type = ?event_type, data_len = data.len(), "Parsing SSE event");

        // Parse JSON data
        match serde_json::from_str::<StreamEvent>(&data) {
            Ok(event) => Some(Ok(event)),
            Err(e) => {
                warn!(error = %e, data = %data, "Failed to parse SSE event");
                Some(Err(StreamParseError::JsonParse(e.to_string())))
            }
        }
    }
}

impl<S> Stream for SseParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<StreamEvent, StreamParseError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First, try to parse any buffered data
        if let Some(event) = self.parse_event() {
            return Poll::Ready(Some(event));
        }

        // Need more data
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                match String::from_utf8(bytes.to_vec()) {
                    Ok(text) => {
                        self.buffer.push_str(&text);
                        // Try parsing again
                        if let Some(event) = self.parse_event() {
                            Poll::Ready(Some(event))
                        } else {
                            // Need more data, re-poll
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                    Err(e) => Poll::Ready(Some(Err(StreamParseError::Utf8(e.to_string())))),
                }
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(StreamParseError::Network(e.to_string()))))
            }
            Poll::Ready(None) => {
                // Stream ended, flush remaining buffer
                if let Some(event) = self.parse_event() {
                    Poll::Ready(Some(event))
                } else {
                    Poll::Ready(None)
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum StreamParseError {
    #[error("network error: {0}")]
    Network(String),
    #[error("UTF-8 decode error: {0}")]
    Utf8(String),
    #[error("JSON parse error: {0}")]
    JsonParse(String),
}
```

### 3. Claude Stream Implementation (src/streaming/claude_stream.rs)

```rust
//! Claude streaming response handler.

use super::events::{ContentBlockData, ContentDelta, StreamEvent, StreamUsage};
use super::parser::{SseParser, StreamParseError};
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tachikoma_backends_core::{
    CompletionChunk, CompletionStream, FinishReason, ToolCallDelta, Usage,
};
use tracing::{debug, trace};

/// State for tracking tool calls being built.
#[derive(Debug, Default)]
struct ToolCallState {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

/// Convert Claude SSE stream to CompletionStream.
pub struct ClaudeStream<S> {
    inner: SseParser<S>,
    /// Model name from message_start.
    model: Option<String>,
    /// Accumulated usage.
    usage: StreamUsage,
    /// Active tool calls by index.
    tool_calls: Vec<ToolCallState>,
    /// Current stop reason.
    stop_reason: Option<String>,
    /// Whether we've seen message_stop.
    finished: bool,
}

impl<S> ClaudeStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new Claude stream.
    pub fn new(byte_stream: S) -> Self {
        Self {
            inner: SseParser::new(byte_stream),
            model: None,
            usage: StreamUsage::default(),
            tool_calls: Vec::new(),
            stop_reason: None,
            finished: false,
        }
    }

    /// Convert a stream event to a completion chunk.
    fn event_to_chunk(&mut self, event: StreamEvent) -> Option<CompletionChunk> {
        match event {
            StreamEvent::MessageStart { message } => {
                debug!(model = %message.model, "Stream started");
                self.model = Some(message.model);
                self.usage = message.usage;
                None // No content yet
            }

            StreamEvent::ContentBlockStart { index, content_block } => {
                trace!(index, "Content block started");
                match content_block {
                    ContentBlockData::Text { text } => {
                        if text.is_empty() {
                            None
                        } else {
                            Some(CompletionChunk::text(text))
                        }
                    }
                    ContentBlockData::ToolUse { id, name, input: _ } => {
                        // Ensure tool_calls vec is large enough
                        while self.tool_calls.len() <= index {
                            self.tool_calls.push(ToolCallState::default());
                        }
                        self.tool_calls[index].id = Some(id.clone());
                        self.tool_calls[index].name = Some(name.clone());

                        Some(CompletionChunk {
                            delta: String::new(),
                            tool_calls: vec![ToolCallDelta {
                                index,
                                id: Some(id),
                                name: Some(name),
                                arguments_delta: String::new(),
                            }],
                            is_final: false,
                            usage: None,
                            finish_reason: None,
                        })
                    }
                }
            }

            StreamEvent::ContentBlockDelta { index, delta } => {
                match delta {
                    ContentDelta::TextDelta { text } => {
                        Some(CompletionChunk::text(text))
                    }
                    ContentDelta::InputJsonDelta { partial_json } => {
                        // Accumulate tool arguments
                        if index < self.tool_calls.len() {
                            self.tool_calls[index].arguments.push_str(&partial_json);
                        }

                        Some(CompletionChunk {
                            delta: String::new(),
                            tool_calls: vec![ToolCallDelta {
                                index,
                                id: None,
                                name: None,
                                arguments_delta: partial_json,
                            }],
                            is_final: false,
                            usage: None,
                            finish_reason: None,
                        })
                    }
                }
            }

            StreamEvent::ContentBlockStop { index } => {
                trace!(index, "Content block stopped");
                None
            }

            StreamEvent::MessageDelta { delta, usage } => {
                self.stop_reason = delta.stop_reason;
                if let Some(u) = usage {
                    self.usage.output_tokens = u.output_tokens;
                }
                None
            }

            StreamEvent::MessageStop => {
                debug!("Stream finished");
                self.finished = true;

                let finish_reason = match self.stop_reason.as_deref() {
                    Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
                    Some("max_tokens") => FinishReason::Length,
                    Some("tool_use") => FinishReason::ToolUse,
                    _ => FinishReason::Stop,
                };

                Some(CompletionChunk::final_chunk(
                    Usage::new(self.usage.input_tokens, self.usage.output_tokens),
                    finish_reason,
                ))
            }

            StreamEvent::Ping => {
                trace!("Received ping");
                None
            }

            StreamEvent::Error { error } => {
                debug!(error_type = %error.error_type, message = %error.message, "Stream error");
                // Return as error through the stream
                None
            }
        }
    }
}

impl<S> Stream for ClaudeStream<S>
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<CompletionChunk, tachikoma_backends_core::BackendError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(event))) => {
                    if let StreamEvent::Error { error } = &event {
                        return Poll::Ready(Some(Err(
                            tachikoma_backends_core::BackendError::Api {
                                status: 0,
                                message: format!("{}: {}", error.error_type, error.message),
                            },
                        )));
                    }

                    if let Some(chunk) = self.event_to_chunk(event) {
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                    // No chunk for this event, continue polling
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(
                        tachikoma_backends_core::BackendError::Network(e.to_string()),
                    )));
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

/// Create a CompletionStream from a reqwest response.
pub fn create_stream(
    response: reqwest::Response,
) -> CompletionStream {
    let byte_stream = response.bytes_stream();
    Box::pin(ClaudeStream::new(byte_stream))
}
```

### 4. Backend Integration (src/streaming/mod.rs)

```rust
//! Streaming support for Claude backend.

mod claude_stream;
mod events;
mod parser;

pub use claude_stream::create_stream;
pub use events::{StreamEvent, StreamUsage};
pub use parser::{SseParser, StreamParseError};
```

### 5. Backend complete_stream Implementation

Add to `src/backend.rs`:

```rust
// Add to ClaudeBackend impl

#[async_trait]
impl Backend for ClaudeBackend {
    // ... existing methods ...

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionStream, BackendError> {
        let mut api_request = self.to_api_request(&request);
        api_request.stream = Some(true);

        debug!("Sending streaming request to Claude API");

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

        Ok(crate::streaming::create_stream(response))
    }
}
```

---

## Testing Requirements

1. SSE parser correctly handles multi-line events
2. Text deltas accumulate properly
3. Tool call deltas build complete arguments
4. Final chunk contains usage statistics
5. Error events are propagated correctly
6. Stream cancellation cleans up resources

---

## Related Specs

- Depends on: [056-claude-api-client.md](056-claude-api-client.md)
- Next: [059-claude-tools.md](059-claude-tools.md)
- Used by: Brain, real-time UI updates
