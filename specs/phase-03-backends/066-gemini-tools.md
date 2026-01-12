# 066 - Gemini Tool Calling

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 066
**Status:** Planned
**Dependencies:** 064-gemini-api-client, 054-tool-definitions, 055-tool-call-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement function calling support for the Google Gemini backend, including function declaration conversion, function call parsing, and response formatting according to Gemini's API specifications.

---

## Acceptance Criteria

- [x] Convert `ToolDefinition` to Gemini function format
- [x] Parse functionCall parts from responses
- [x] Format function responses for continuation
- [x] Support function calling modes
- [x] Handle multiple function calls

---

## Implementation Details

### 1. Tool Conversion (src/tools/convert.rs)

```rust
//! Tool definition conversion for Gemini API.

use serde_json::Value as JsonValue;
use tachikoma_backends_core::{ToolCall, ToolChoice, ToolDefinition, ToolResult};

/// Convert a ToolDefinition to Gemini function declaration.
pub fn to_gemini_function(tool: &ToolDefinition) -> JsonValue {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters.to_json_schema()
    })
}

/// Convert multiple tools to Gemini format (wrapped in Tool object).
pub fn to_gemini_tools(tools: &[ToolDefinition]) -> JsonValue {
    let declarations: Vec<JsonValue> = tools.iter().map(to_gemini_function).collect();

    serde_json::json!([{
        "functionDeclarations": declarations
    }])
}

/// Convert ToolChoice to Gemini function calling config.
pub fn to_gemini_tool_config(choice: &ToolChoice) -> JsonValue {
    match choice {
        ToolChoice::Auto => serde_json::json!({
            "functionCallingConfig": {
                "mode": "AUTO"
            }
        }),
        ToolChoice::Required => serde_json::json!({
            "functionCallingConfig": {
                "mode": "ANY"
            }
        }),
        ToolChoice::None => serde_json::json!({
            "functionCallingConfig": {
                "mode": "NONE"
            }
        }),
        ToolChoice::Tool { name } => serde_json::json!({
            "functionCallingConfig": {
                "mode": "ANY",
                "allowedFunctionNames": [name]
            }
        }),
    }
}

/// Parse a function call from Gemini response part.
pub fn parse_function_call(part: &JsonValue) -> Option<ToolCall> {
    let function_call = part.get("functionCall")?;
    let name = function_call.get("name")?.as_str()?;
    let args = function_call.get("args")?;

    Some(ToolCall {
        id: format!("call_{}", uuid::Uuid::new_v4()),
        name: name.to_string(),
        arguments: serde_json::to_string(args).unwrap_or_default(),
    })
}

/// Format a tool result as a Gemini function response part.
pub fn to_gemini_function_response(result: &ToolResult) -> JsonValue {
    let response_content = match &result.content {
        tachikoma_backends_core::ToolResultContent::Text(s) => {
            serde_json::json!({ "result": s })
        }
        tachikoma_backends_core::ToolResultContent::Json(v) => {
            serde_json::json!({ "result": v })
        }
        tachikoma_backends_core::ToolResultContent::Error(e) => {
            serde_json::json!({ "error": e })
        }
    };

    serde_json::json!({
        "functionResponse": {
            "name": result.name,
            "response": response_content
        }
    })
}

/// Format multiple tool results as a Gemini content message.
pub fn to_gemini_function_response_content(results: &[ToolResult]) -> JsonValue {
    let parts: Vec<JsonValue> = results.iter().map(to_gemini_function_response).collect();

    serde_json::json!({
        "role": "function",
        "parts": parts
    })
}

/// Format a model message with function calls.
pub fn to_model_content_with_functions(
    text: Option<&str>,
    function_calls: &[ToolCall],
) -> JsonValue {
    let mut parts: Vec<JsonValue> = Vec::new();

    if let Some(t) = text {
        parts.push(serde_json::json!({ "text": t }));
    }

    for call in function_calls {
        let args: JsonValue = serde_json::from_str(&call.arguments)
            .unwrap_or(serde_json::json!({}));

        parts.push(serde_json::json!({
            "functionCall": {
                "name": call.name,
                "args": args
            }
        }));
    }

    serde_json::json!({
        "role": "model",
        "parts": parts
    })
}
```

### 2. Tool Handler (src/tools/handler.rs)

```rust
//! Tool execution handler for Gemini backend.

use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{ToolCall, ToolDefinition, ToolResult};
use tracing::{debug, info, warn};

/// Callback type for tool execution.
pub type ToolExecutor = Arc<
    dyn Fn(ToolCall) -> futures::future::BoxFuture<'static, ToolResult> + Send + Sync,
>;

/// Handler for executing function calls from Gemini.
#[derive(Default)]
pub struct GeminiToolHandler {
    executors: HashMap<String, ToolExecutor>,
    definitions: HashMap<String, ToolDefinition>,
    timeout: std::time::Duration,
}

impl GeminiToolHandler {
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

    /// Register a function with its executor.
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

    /// Get function declarations for the API.
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.definitions.values().cloned().collect()
    }

    /// Check if a function is registered.
    pub fn has_function(&self, name: &str) -> bool {
        self.executors.contains_key(name)
    }

    /// Execute a function call.
    pub async fn execute(&self, call: ToolCall) -> ToolResult {
        let name = call.name.clone();
        let id = call.id.clone();

        let executor = match self.executors.get(&name) {
            Some(e) => Arc::clone(e),
            None => {
                warn!(function = %name, "Function not found");
                return ToolResult::error(id, name, "Function not found");
            }
        };

        debug!(function = %name, call_id = %id, "Executing function");

        match tokio::time::timeout(self.timeout, executor(call)).await {
            Ok(result) => {
                info!(function = %name, success = result.success, "Function completed");
                result
            }
            Err(_) => {
                warn!(function = %name, "Function timed out");
                ToolResult::error(id, name, format!("Timed out after {:?}", self.timeout))
            }
        }
    }

    /// Execute multiple function calls.
    pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        use futures::future::join_all;

        let futures: Vec<_> = calls.into_iter().map(|c| self.execute(c)).collect();
        join_all(futures).await
    }
}

impl std::fmt::Debug for GeminiToolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeminiToolHandler")
            .field("functions", &self.definitions.keys().collect::<Vec<_>>())
            .field("timeout", &self.timeout)
            .finish()
    }
}
```

### 3. Streaming Support (src/tools/streaming.rs)

```rust
//! Streaming support for Gemini function calls.

use crate::api_types::GenerateContentResponse;
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

/// Gemini streaming response handler.
pub struct GeminiStream<S> {
    inner: S,
    buffer: String,
    model: String,
    tool_call_index: usize,
    finished: bool,
}

impl<S> GeminiStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new Gemini stream.
    pub fn new(byte_stream: S) -> Self {
        Self {
            inner: byte_stream,
            buffer: String::new(),
            model: String::new(),
            tool_call_index: 0,
            finished: false,
        }
    }

    /// Parse a complete response chunk.
    fn parse_chunk(&mut self) -> Option<Result<CompletionChunk, BackendError>> {
        // Gemini streams newline-delimited JSON
        let line_end = self.buffer.find('\n')?;
        let line = self.buffer[..line_end].to_string();
        self.buffer = self.buffer[line_end + 1..].to_string();

        if line.trim().is_empty() {
            return None;
        }

        match serde_json::from_str::<GenerateContentResponse>(&line) {
            Ok(response) => self.process_response(response),
            Err(e) => {
                trace!(error = %e, line = %line, "Failed to parse chunk");
                None
            }
        }
    }

    /// Process a parsed response.
    fn process_response(
        &mut self,
        response: GenerateContentResponse,
    ) -> Option<Result<CompletionChunk, BackendError>> {
        let candidate = response.candidates?.into_iter().next()?;
        let content = candidate.content?;

        let mut text_delta = String::new();
        let mut tool_call_deltas = Vec::new();

        for part in content.parts {
            match part {
                crate::api_types::Part::Text { text } => {
                    text_delta.push_str(&text);
                }
                crate::api_types::Part::FunctionCall { function_call } => {
                    let args = serde_json::to_string(&function_call.args).unwrap_or_default();

                    tool_call_deltas.push(ToolCallDelta {
                        index: self.tool_call_index,
                        id: Some(format!("call_{}", self.tool_call_index)),
                        name: Some(function_call.name),
                        arguments_delta: args,
                    });

                    self.tool_call_index += 1;
                }
                _ => {}
            }
        }

        let finish_reason = candidate.finish_reason.as_deref().map(|r| match r {
            "STOP" => FinishReason::Stop,
            "MAX_TOKENS" => FinishReason::Length,
            "SAFETY" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        });

        let is_final = finish_reason.is_some();
        if is_final {
            self.finished = true;
        }

        let usage = response.usage_metadata.map(|u| {
            Usage::new(
                u.prompt_token_count.unwrap_or(0),
                u.candidates_token_count.unwrap_or(0),
            )
        });

        if text_delta.is_empty() && tool_call_deltas.is_empty() && !is_final {
            return None;
        }

        Some(Ok(CompletionChunk {
            delta: text_delta,
            tool_calls: tool_call_deltas,
            is_final,
            usage: if is_final { usage } else { None },
            finish_reason,
        }))
    }
}

impl<S> Stream for GeminiStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<CompletionChunk, BackendError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }

        loop {
            if let Some(result) = self.parse_chunk() {
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
                            Usage::default(),
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
    Box::pin(GeminiStream::new(byte_stream))
}
```

### 4. Module Exports (src/tools/mod.rs)

```rust
//! Tool support for Gemini backend.

mod convert;
mod handler;
mod streaming;

pub use convert::{
    parse_function_call, to_gemini_function, to_gemini_function_response,
    to_gemini_function_response_content, to_gemini_tool_config, to_gemini_tools,
    to_model_content_with_functions,
};
pub use handler::GeminiToolHandler;
pub use streaming::{create_stream, GeminiStream};
```

---

## Testing Requirements

1. Function declaration conversion is correct
2. Function calls parse from response parts
3. Function responses format correctly
4. Streaming accumulates function calls
5. Tool config modes are correct

---

## Related Specs

- Depends on: [064-gemini-api-client.md](064-gemini-api-client.md)
- Depends on: [054-tool-definitions.md](054-tool-definitions.md)
- Next: [067-ollama-setup.md](067-ollama-setup.md)
