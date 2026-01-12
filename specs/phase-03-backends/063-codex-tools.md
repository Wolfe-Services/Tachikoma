# 063 - Codex Tool Calling

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 063
**Status:** Planned
**Dependencies:** 061-codex-api-client, 054-tool-definitions, 055-tool-call-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement tool/function calling support for the OpenAI/Codex backend, including tool definition conversion, tool call parsing, result formatting, and streaming support for progressive tool argument delivery.

---

## Acceptance Criteria

- [x] Convert `ToolDefinition` to OpenAI function format
- [x] Parse tool_calls from responses
- [x] Format tool results as messages
- [x] Support tool_choice parameter
- [x] Handle parallel tool calls
- [x] Streaming tool call accumulation

---

## Implementation Details

### 1. Tool Conversion (src/tools/convert.rs)

```rust
//! Tool definition conversion for OpenAI API.

use serde_json::Value as JsonValue;
use tachikoma_backends_core::{ToolCall, ToolChoice, ToolDefinition, ToolResult};

/// Convert a ToolDefinition to OpenAI function format.
pub fn to_openai_tool(tool: &ToolDefinition) -> JsonValue {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters.to_json_schema()
        }
    })
}

/// Convert multiple tools to OpenAI format.
pub fn to_openai_tools(tools: &[ToolDefinition]) -> Vec<JsonValue> {
    tools.iter().map(to_openai_tool).collect()
}

/// Convert ToolChoice to OpenAI format.
pub fn to_openai_tool_choice(choice: &ToolChoice) -> JsonValue {
    match choice {
        ToolChoice::Auto => serde_json::json!("auto"),
        ToolChoice::Required => serde_json::json!("required"),
        ToolChoice::None => serde_json::json!("none"),
        ToolChoice::Tool { name } => serde_json::json!({
            "type": "function",
            "function": {
                "name": name
            }
        }),
    }
}

/// Parse a tool_call from OpenAI response.
pub fn parse_tool_call(call: &JsonValue) -> Option<ToolCall> {
    let id = call.get("id")?.as_str()?;
    let function = call.get("function")?;
    let name = function.get("name")?.as_str()?;
    let arguments = function.get("arguments")?.as_str()?;

    Some(ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: arguments.to_string(),
    })
}

/// Format a tool result as an OpenAI message.
pub fn to_openai_tool_message(result: &ToolResult) -> JsonValue {
    let content = match &result.content {
        tachikoma_backends_core::ToolResultContent::Text(s) => s.clone(),
        tachikoma_backends_core::ToolResultContent::Json(v) => {
            serde_json::to_string(v).unwrap_or_default()
        }
        tachikoma_backends_core::ToolResultContent::Error(e) => format!("Error: {}", e),
    };

    serde_json::json!({
        "role": "tool",
        "tool_call_id": result.tool_call_id,
        "content": content
    })
}

/// Format an assistant message with tool calls.
pub fn to_assistant_message_with_tools(
    content: Option<&str>,
    tool_calls: &[ToolCall],
) -> JsonValue {
    let calls: Vec<JsonValue> = tool_calls
        .iter()
        .map(|tc| {
            serde_json::json!({
                "id": tc.id,
                "type": "function",
                "function": {
                    "name": tc.name,
                    "arguments": tc.arguments
                }
            })
        })
        .collect();

    serde_json::json!({
        "role": "assistant",
        "content": content,
        "tool_calls": calls
    })
}
```

### 2. Streaming Support (src/tools/streaming.rs)

```rust
//! Streaming support for OpenAI tool calls.

use crate::api_types::{ChatCompletionChunk, ChunkChoice, ChunkToolCall};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use tachikoma_backends_core::{
    BackendError, CompletionChunk, CompletionStream, FinishReason,
    ToolCall, ToolCallDelta, Usage,
};
use tracing::{debug, trace};

/// State for accumulating streaming tool calls.
#[derive(Debug, Default)]
struct ToolCallBuilder {
    id: String,
    name: String,
    arguments: String,
}

/// OpenAI streaming response handler.
pub struct OpenAIStream<S> {
    inner: S,
    buffer: String,
    model: String,
    tool_calls: HashMap<usize, ToolCallBuilder>,
    usage: Option<Usage>,
    finished: bool,
}

impl<S> OpenAIStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new OpenAI stream.
    pub fn new(byte_stream: S) -> Self {
        Self {
            inner: byte_stream,
            buffer: String::new(),
            model: String::new(),
            tool_calls: HashMap::new(),
            usage: None,
            finished: false,
        }
    }

    /// Parse a line from the SSE stream.
    fn parse_line(&mut self, line: &str) -> Option<Result<CompletionChunk, BackendError>> {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        // Handle data lines
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                self.finished = true;
                return Some(Ok(CompletionChunk::final_chunk(
                    self.usage.unwrap_or_default(),
                    FinishReason::Stop,
                )));
            }

            match serde_json::from_str::<ChatCompletionChunk>(data) {
                Ok(chunk) => {
                    if self.model.is_empty() {
                        self.model = chunk.model.clone();
                    }
                    if let Some(usage) = chunk.usage {
                        self.usage = Some(Usage::new(usage.prompt_tokens, usage.completion_tokens));
                    }
                    return self.process_chunk(chunk);
                }
                Err(e) => {
                    trace!(error = %e, data = %data, "Failed to parse chunk");
                    return None;
                }
            }
        }

        None
    }

    /// Process a parsed chunk.
    fn process_chunk(&mut self, chunk: ChatCompletionChunk) -> Option<Result<CompletionChunk, BackendError>> {
        let choice = chunk.choices.first()?;

        let mut text_delta = String::new();
        let mut tool_call_deltas = Vec::new();

        // Extract text content
        if let Some(content) = &choice.delta.content {
            text_delta = content.clone();
        }

        // Extract tool calls
        if let Some(calls) = &choice.delta.tool_calls {
            for call in calls {
                let delta = self.process_tool_call_delta(call);
                tool_call_deltas.push(delta);
            }
        }

        // Check finish reason
        let (is_final, finish_reason) = match choice.finish_reason.as_deref() {
            Some("stop") => (true, Some(FinishReason::Stop)),
            Some("length") => (true, Some(FinishReason::Length)),
            Some("tool_calls") => (true, Some(FinishReason::ToolUse)),
            Some("content_filter") => (true, Some(FinishReason::ContentFilter)),
            _ => (false, None),
        };

        if is_final {
            self.finished = true;
        }

        // Only emit chunk if there's content
        if text_delta.is_empty() && tool_call_deltas.is_empty() && !is_final {
            return None;
        }

        Some(Ok(CompletionChunk {
            delta: text_delta,
            tool_calls: tool_call_deltas,
            is_final,
            usage: if is_final { self.usage } else { None },
            finish_reason,
        }))
    }

    /// Process a tool call delta.
    fn process_tool_call_delta(&mut self, call: &ChunkToolCall) -> ToolCallDelta {
        let builder = self.tool_calls.entry(call.index).or_default();

        let id = call.id.clone();
        let name = call.function.as_ref().and_then(|f| f.name.clone());
        let args_delta = call
            .function
            .as_ref()
            .and_then(|f| f.arguments.clone())
            .unwrap_or_default();

        // Update builder
        if let Some(id) = &id {
            builder.id = id.clone();
        }
        if let Some(name) = &name {
            builder.name = name.clone();
        }
        builder.arguments.push_str(&args_delta);

        ToolCallDelta {
            index: call.index,
            id,
            name,
            arguments_delta: args_delta,
        }
    }

    /// Get completed tool calls.
    pub fn get_tool_calls(&self) -> Vec<ToolCall> {
        self.tool_calls
            .values()
            .filter(|b| !b.id.is_empty())
            .map(|b| ToolCall {
                id: b.id.clone(),
                name: b.name.clone(),
                arguments: b.arguments.clone(),
            })
            .collect()
    }
}

impl<S> Stream for OpenAIStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<CompletionChunk, BackendError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        loop {
            // Try to parse buffered data
            if let Some(line_end) = self.buffer.find('\n') {
                let line = self.buffer[..line_end].to_string();
                self.buffer = self.buffer[line_end + 1..].to_string();

                if let Some(result) = self.parse_line(&line) {
                    return Poll::Ready(Some(result));
                }
                continue;
            }

            // Need more data
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
                    // Stream ended, emit final chunk if needed
                    if !self.finished {
                        self.finished = true;
                        return Poll::Ready(Some(Ok(CompletionChunk::final_chunk(
                            self.usage.unwrap_or_default(),
                            FinishReason::Stop,
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
    Box::pin(OpenAIStream::new(byte_stream))
}
```

### 3. Tool Handler (src/tools/handler.rs)

```rust
//! Tool execution handler for Codex backend.

use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{ToolCall, ToolDefinition, ToolResult};
use tracing::{debug, info, warn};

/// Callback type for tool execution.
pub type ToolExecutor = Arc<
    dyn Fn(ToolCall) -> futures::future::BoxFuture<'static, ToolResult> + Send + Sync,
>;

/// Handler for executing tool calls.
#[derive(Default)]
pub struct CodexToolHandler {
    executors: HashMap<String, ToolExecutor>,
    definitions: HashMap<String, ToolDefinition>,
    timeout: std::time::Duration,
}

impl CodexToolHandler {
    /// Create a new tool handler.
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
            definitions: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Set the execution timeout.
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Register a tool with its executor.
    pub fn register<F, Fut>(&mut self, definition: ToolDefinition, executor: F)
    where
        F: Fn(ToolCall) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ToolResult> + Send + 'static,
    {
        let name = definition.name.clone();
        self.definitions.insert(name.clone(), definition);
        self.executors.insert(
            name,
            Arc::new(move |call| Box::pin(executor(call))),
        );
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

        debug!(tool = %name, call_id = %id, "Executing tool");

        match tokio::time::timeout(self.timeout, executor(call)).await {
            Ok(result) => {
                info!(tool = %name, success = result.success, "Tool completed");
                result
            }
            Err(_) => {
                warn!(tool = %name, "Tool timed out");
                ToolResult::error(id, name, format!("Timed out after {:?}", self.timeout))
            }
        }
    }

    /// Execute multiple tool calls.
    pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        use futures::future::join_all;

        let futures: Vec<_> = calls.into_iter().map(|c| self.execute(c)).collect();
        join_all(futures).await
    }
}

impl std::fmt::Debug for CodexToolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexToolHandler")
            .field("tools", &self.definitions.keys().collect::<Vec<_>>())
            .field("timeout", &self.timeout)
            .finish()
    }
}
```

### 4. Backend Integration

Add streaming support to `src/backend.rs`:

```rust
// Add to CodexBackend impl

async fn complete_stream(
    &self,
    request: CompletionRequest,
) -> Result<CompletionStream, BackendError> {
    let mut api_request = self.to_api_request(&request);
    api_request.stream = Some(true);

    debug!("Sending streaming request to OpenAI API");

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

    Ok(crate::tools::streaming::create_stream(response))
}
```

### 5. Module Exports (src/tools/mod.rs)

```rust
//! Tool support for Codex backend.

mod convert;
mod handler;
mod streaming;

pub use convert::{
    parse_tool_call, to_assistant_message_with_tools, to_openai_tool,
    to_openai_tool_choice, to_openai_tool_message, to_openai_tools,
};
pub use handler::CodexToolHandler;
pub use streaming::{create_stream, OpenAIStream};
```

---

## Testing Requirements

1. Tool definition conversion is correct
2. Tool calls parse from responses
3. Tool results format as messages
4. Streaming accumulates tool arguments
5. Parallel execution works correctly

---

## Related Specs

- Depends on: [061-codex-api-client.md](061-codex-api-client.md)
- Depends on: [054-tool-definitions.md](054-tool-definitions.md)
- Next: [064-gemini-api-client.md](064-gemini-api-client.md)
