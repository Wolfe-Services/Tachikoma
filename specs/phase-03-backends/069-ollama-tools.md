# 069 - Ollama Tool Calling

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 069
**Status:** Planned
**Dependencies:** 067-ollama-setup, 068-ollama-models, 054-tool-definitions
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement tool calling support for the Ollama backend, including tool definition conversion, tool call parsing, streaming support, and handling of models that support native function calling.

---

## Acceptance Criteria

- [ ] Convert `ToolDefinition` to Ollama format
- [ ] Parse tool_calls from responses
- [ ] Format tool results for continuation
- [ ] Streaming response handling
- [ ] Fallback for models without tool support

---

## Implementation Details

### 1. Tool Conversion (src/tools/convert.rs)

```rust
//! Tool definition conversion for Ollama API.

use serde_json::Value as JsonValue;
use tachikoma_backends_core::{ToolCall, ToolDefinition, ToolResult};

/// Convert a ToolDefinition to Ollama tool format.
pub fn to_ollama_tool(tool: &ToolDefinition) -> JsonValue {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters.to_json_schema()
        }
    })
}

/// Convert multiple tools to Ollama format.
pub fn to_ollama_tools(tools: &[ToolDefinition]) -> Vec<JsonValue> {
    tools.iter().map(to_ollama_tool).collect()
}

/// Parse a tool call from Ollama response.
pub fn parse_tool_call(call: &crate::api_types::ToolCall, index: usize) -> ToolCall {
    ToolCall {
        id: format!("call_{}", index),
        name: call.function.name.clone(),
        arguments: serde_json::to_string(&call.function.arguments).unwrap_or_default(),
    }
}

/// Format a tool result as an Ollama message.
pub fn to_ollama_tool_message(result: &ToolResult) -> crate::api_types::ChatMessage {
    let content = match &result.content {
        tachikoma_backends_core::ToolResultContent::Text(s) => s.clone(),
        tachikoma_backends_core::ToolResultContent::Json(v) => {
            serde_json::to_string(v).unwrap_or_default()
        }
        tachikoma_backends_core::ToolResultContent::Error(e) => format!("Error: {}", e),
    };

    crate::api_types::ChatMessage {
        role: "tool".to_string(),
        content,
        images: None,
        tool_calls: None,
    }
}

/// Format multiple tool results as messages.
pub fn to_ollama_tool_messages(results: &[ToolResult]) -> Vec<crate::api_types::ChatMessage> {
    results.iter().map(to_ollama_tool_message).collect()
}
```

### 2. Streaming Support (src/tools/streaming.rs)

```rust
//! Streaming support for Ollama responses.

use crate::api_types::ChatResponse;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tachikoma_backends_core::{
    BackendError, CompletionChunk, CompletionStream, FinishReason,
    ToolCall, ToolCallDelta, Usage,
};
use tracing::{debug, trace};

/// Ollama streaming response handler.
pub struct OllamaStream<S> {
    inner: S,
    buffer: String,
    model: String,
    accumulated_content: String,
    tool_calls: Vec<ToolCall>,
    total_eval_count: u32,
    total_prompt_count: u32,
    finished: bool,
}

impl<S> OllamaStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new Ollama stream.
    pub fn new(byte_stream: S) -> Self {
        Self {
            inner: byte_stream,
            buffer: String::new(),
            model: String::new(),
            accumulated_content: String::new(),
            tool_calls: Vec::new(),
            total_eval_count: 0,
            total_prompt_count: 0,
            finished: false,
        }
    }

    /// Parse a line from the stream.
    fn parse_line(&mut self) -> Option<Result<CompletionChunk, BackendError>> {
        let line_end = self.buffer.find('\n')?;
        let line = self.buffer[..line_end].to_string();
        self.buffer = self.buffer[line_end + 1..].to_string();

        if line.trim().is_empty() {
            return None;
        }

        match serde_json::from_str::<ChatResponse>(&line) {
            Ok(response) => self.process_response(response),
            Err(e) => {
                trace!(error = %e, line = %line, "Failed to parse line");
                None
            }
        }
    }

    /// Process a parsed response.
    fn process_response(&mut self, response: ChatResponse) -> Option<Result<CompletionChunk, BackendError>> {
        if self.model.is_empty() {
            self.model = response.model.clone();
        }

        // Accumulate token counts
        if let Some(count) = response.eval_count {
            self.total_eval_count += count;
        }
        if let Some(count) = response.prompt_eval_count {
            self.total_prompt_count = count;
        }

        let text_delta = response.message.content.clone();
        self.accumulated_content.push_str(&text_delta);

        // Check for tool calls
        let mut tool_call_deltas = Vec::new();
        if let Some(calls) = response.message.tool_calls {
            for (i, call) in calls.iter().enumerate() {
                let index = self.tool_calls.len() + i;
                let tool_call = super::convert::parse_tool_call(call, index);

                tool_call_deltas.push(ToolCallDelta {
                    index,
                    id: Some(tool_call.id.clone()),
                    name: Some(tool_call.name.clone()),
                    arguments_delta: tool_call.arguments.clone(),
                });

                self.tool_calls.push(tool_call);
            }
        }

        let is_final = response.done;
        if is_final {
            self.finished = true;
        }

        let finish_reason = if is_final {
            if !self.tool_calls.is_empty() {
                Some(FinishReason::ToolUse)
            } else {
                Some(FinishReason::Stop)
            }
        } else {
            None
        };

        if text_delta.is_empty() && tool_call_deltas.is_empty() && !is_final {
            return None;
        }

        let usage = if is_final {
            Some(Usage::new(self.total_prompt_count, self.total_eval_count))
        } else {
            None
        };

        Some(Ok(CompletionChunk {
            delta: text_delta,
            tool_calls: tool_call_deltas,
            is_final,
            usage,
            finish_reason,
        }))
    }
}

impl<S> Stream for OllamaStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<CompletionChunk, BackendError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        loop {
            if let Some(result) = self.parse_line() {
                return Poll::Ready(Some(result));
            }

            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(text) => {
                            self.buffer.push_str(&text);
                        }
                        Err(e) => {
                            return Poll::Ready(Some(Err(BackendError::Parsing(e.to_string()))));
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(BackendError::Network(e.to_string()))));
                }
                Poll::Ready(None) => {
                    if !self.finished {
                        self.finished = true;
                        return Poll::Ready(Some(Ok(CompletionChunk::final_chunk(
                            Usage::new(self.total_prompt_count, self.total_eval_count),
                            if self.tool_calls.is_empty() {
                                FinishReason::Stop
                            } else {
                                FinishReason::ToolUse
                            },
                        ))));
                    }
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
pub fn create_stream(response: reqwest::Response) -> CompletionStream {
    let byte_stream = response.bytes_stream();
    Box::pin(OllamaStream::new(byte_stream))
}
```

### 3. Tool Handler (src/tools/handler.rs)

```rust
//! Tool execution handler for Ollama.

use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{ToolCall, ToolDefinition, ToolResult};
use tracing::{debug, info, warn};

/// Tool handler for Ollama backend.
#[derive(Default)]
pub struct OllamaToolHandler {
    executors: HashMap<String, Arc<dyn Fn(ToolCall) -> futures::future::BoxFuture<'static, ToolResult> + Send + Sync>>,
    definitions: HashMap<String, ToolDefinition>,
    timeout: std::time::Duration,
}

impl OllamaToolHandler {
    /// Create a new tool handler.
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
            definitions: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Set execution timeout.
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Register a tool.
    pub fn register<F, Fut>(&mut self, definition: ToolDefinition, executor: F)
    where
        F: Fn(ToolCall) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ToolResult> + Send + 'static,
    {
        let name = definition.name.clone();
        self.definitions.insert(name.clone(), definition);
        self.executors.insert(name, Arc::new(move |call| Box::pin(executor(call))));
    }

    /// Get tool definitions.
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.definitions.values().cloned().collect()
    }

    /// Execute a tool call.
    pub async fn execute(&self, call: ToolCall) -> ToolResult {
        let name = call.name.clone();
        let id = call.id.clone();

        let executor = match self.executors.get(&name) {
            Some(e) => Arc::clone(e),
            None => {
                warn!(tool = %name, "Tool not found");
                return ToolResult::error(id, name, "Tool not found");
            }
        };

        debug!(tool = %name, "Executing tool");

        match tokio::time::timeout(self.timeout, executor(call)).await {
            Ok(result) => result,
            Err(_) => {
                warn!(tool = %name, "Tool timed out");
                ToolResult::error(id, name, format!("Timed out after {:?}", self.timeout))
            }
        }
    }

    /// Execute multiple tool calls.
    pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        use futures::future::join_all;
        join_all(calls.into_iter().map(|c| self.execute(c))).await
    }
}

impl std::fmt::Debug for OllamaToolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaToolHandler")
            .field("tools", &self.definitions.keys().collect::<Vec<_>>())
            .finish()
    }
}
```

### 4. Backend Streaming Integration

Add to `src/backend.rs`:

```rust
// Add streaming implementation to OllamaBackend

async fn complete_stream(
    &self,
    request: CompletionRequest,
) -> Result<CompletionStream, BackendError> {
    let mut chat_request = self.to_chat_request(&request);
    chat_request.stream = Some(true);

    debug!("Sending streaming request to Ollama");

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

    Ok(crate::tools::streaming::create_stream(response))
}
```

### 5. Module Exports (src/tools/mod.rs)

```rust
//! Tool support for Ollama backend.

mod convert;
mod handler;
mod streaming;

pub use convert::{
    parse_tool_call, to_ollama_tool, to_ollama_tool_message,
    to_ollama_tool_messages, to_ollama_tools,
};
pub use handler::OllamaToolHandler;
pub use streaming::{create_stream, OllamaStream};
```

---

## Testing Requirements

1. Tool definition conversion is correct
2. Tool calls parse from responses
3. Streaming accumulates content properly
4. Tool results format correctly
5. Timeout handling works

---

## Related Specs

- Depends on: [067-ollama-setup.md](067-ollama-setup.md)
- Depends on: [068-ollama-models.md](068-ollama-models.md)
- Next: [070-backend-factory.md](070-backend-factory.md)
