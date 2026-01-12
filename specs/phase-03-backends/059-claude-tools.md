# 059 - Claude Tool Calling

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 059
**Status:** Planned
**Dependencies:** 056-claude-api-client, 054-tool-definitions, 055-tool-call-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement tool calling support for the Claude backend, including tool definition conversion, tool use block handling, and tool result formatting according to Claude's API specifications.

---

## Acceptance Criteria

- [x] Convert `ToolDefinition` to Claude API format
- [x] Parse tool_use content blocks from responses
- [x] Format tool results for continuation
- [x] Support tool_choice parameter
- [x] Handle streaming tool calls
- [x] Parallel tool call support

---

## Implementation Details

### 1. Tool Conversion (src/tools/convert.rs)

```rust
//! Tool definition conversion for Claude API.

use serde_json::Value as JsonValue;
use tachikoma_backends_core::{ToolCall, ToolChoice, ToolDefinition, ToolResult};

/// Convert a ToolDefinition to Claude API format.
pub fn to_claude_tool(tool: &ToolDefinition) -> JsonValue {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters.to_json_schema()
    })
}

/// Convert multiple tools to Claude API format.
pub fn to_claude_tools(tools: &[ToolDefinition]) -> Vec<JsonValue> {
    tools.iter().map(to_claude_tool).collect()
}

/// Convert ToolChoice to Claude API format.
pub fn to_claude_tool_choice(choice: &ToolChoice) -> JsonValue {
    match choice {
        ToolChoice::Auto => serde_json::json!({"type": "auto"}),
        ToolChoice::Required => serde_json::json!({"type": "any"}),
        ToolChoice::None => serde_json::json!({"type": "auto"}), // Claude doesn't have "none"
        ToolChoice::Tool { name } => serde_json::json!({
            "type": "tool",
            "name": name
        }),
    }
}

/// Parse a tool_use block from Claude response.
pub fn parse_tool_use(block: &JsonValue) -> Option<ToolCall> {
    let id = block.get("id")?.as_str()?;
    let name = block.get("name")?.as_str()?;
    let input = block.get("input")?;

    Some(ToolCall {
        id: id.to_string(),
        name: name.to_string(),
        arguments: serde_json::to_string(input).unwrap_or_default(),
    })
}

/// Format a tool result for Claude API.
pub fn to_claude_tool_result(result: &ToolResult) -> JsonValue {
    let content = match &result.content {
        tachikoma_backends_core::ToolResultContent::Text(s) => s.clone(),
        tachikoma_backends_core::ToolResultContent::Json(v) => {
            serde_json::to_string(v).unwrap_or_default()
        }
        tachikoma_backends_core::ToolResultContent::Error(e) => format!("Error: {}", e),
    };

    serde_json::json!({
        "type": "tool_result",
        "tool_use_id": result.tool_call_id,
        "content": content,
        "is_error": !result.success
    })
}

/// Format multiple tool results for Claude API.
pub fn to_claude_tool_results(results: &[ToolResult]) -> Vec<JsonValue> {
    results.iter().map(to_claude_tool_result).collect()
}
```

### 2. Tool Execution Handler (src/tools/handler.rs)

```rust
//! Tool execution handler for Claude backend.

use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{
    Message, Role, ToolCall, ToolDefinition, ToolResult, ToolResultContent,
};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Callback type for tool execution.
pub type ToolExecutor = Arc<
    dyn Fn(ToolCall) -> futures::future::BoxFuture<'static, ToolResult> + Send + Sync,
>;

/// Handler for executing tool calls from Claude.
#[derive(Default)]
pub struct ToolHandler {
    /// Registered tool executors.
    executors: HashMap<String, ToolExecutor>,
    /// Tool definitions.
    definitions: HashMap<String, ToolDefinition>,
    /// Maximum parallel tool executions.
    max_parallel: usize,
    /// Execution timeout.
    timeout: std::time::Duration,
}

impl ToolHandler {
    /// Create a new tool handler.
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
            definitions: HashMap::new(),
            max_parallel: 4,
            timeout: std::time::Duration::from_secs(30),
        }
    }

    /// Set maximum parallel executions.
    pub fn with_max_parallel(mut self, max: usize) -> Self {
        self.max_parallel = max;
        self
    }

    /// Set execution timeout.
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

    /// Register a simple string-returning tool.
    pub fn register_simple<F>(&mut self, definition: ToolDefinition, executor: F)
    where
        F: Fn(&ToolCall) -> Result<String, String> + Send + Sync + 'static,
    {
        let name = definition.name.clone();
        self.definitions.insert(name.clone(), definition);
        self.executors.insert(
            name.clone(),
            Arc::new(move |call: ToolCall| {
                let result = executor(&call);
                let name = call.name.clone();
                let id = call.id.clone();
                Box::pin(async move {
                    match result {
                        Ok(content) => ToolResult::success(id, name, content),
                        Err(error) => ToolResult::error(id, name, error),
                    }
                })
            }),
        );
    }

    /// Get all tool definitions.
    pub fn definitions(&self) -> Vec<&ToolDefinition> {
        self.definitions.values().collect()
    }

    /// Get tool definitions as owned vec.
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.definitions.values().cloned().collect()
    }

    /// Check if a tool is registered.
    pub fn has_tool(&self, name: &str) -> bool {
        self.executors.contains_key(name)
    }

    /// Execute a single tool call.
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

        // Execute with timeout
        match tokio::time::timeout(self.timeout, executor(call)).await {
            Ok(result) => {
                info!(
                    tool = %name,
                    call_id = %id,
                    success = result.success,
                    "Tool execution completed"
                );
                result
            }
            Err(_) => {
                warn!(tool = %name, call_id = %id, "Tool execution timed out");
                ToolResult::error(id, name, format!("Execution timed out after {:?}", self.timeout))
            }
        }
    }

    /// Execute multiple tool calls in parallel.
    pub async fn execute_batch(&self, calls: Vec<ToolCall>) -> Vec<ToolResult> {
        use futures::stream::{self, StreamExt};

        let results: Vec<ToolResult> = stream::iter(calls)
            .map(|call| self.execute(call))
            .buffer_unordered(self.max_parallel)
            .collect()
            .await;

        results
    }

    /// Convert tool results to messages for continuation.
    pub fn results_to_messages(&self, results: &[ToolResult]) -> Vec<Message> {
        results.iter().map(|r| r.to_message()).collect()
    }
}

impl std::fmt::Debug for ToolHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolHandler")
            .field("tools", &self.definitions.keys().collect::<Vec<_>>())
            .field("max_parallel", &self.max_parallel)
            .field("timeout", &self.timeout)
            .finish()
    }
}
```

### 3. Tool Loop Runner (src/tools/runner.rs)

```rust
//! Tool loop runner for automatic tool execution.

use super::handler::ToolHandler;
use tachikoma_backends_core::{
    Backend, BackendError, CompletionRequest, CompletionResponse, FinishReason, Message,
};
use tracing::{debug, info, warn};

/// Configuration for the tool loop.
#[derive(Debug, Clone)]
pub struct ToolLoopConfig {
    /// Maximum tool loop iterations.
    pub max_iterations: usize,
    /// Whether to continue on tool errors.
    pub continue_on_error: bool,
}

impl Default for ToolLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            continue_on_error: true,
        }
    }
}

/// Run the tool loop until completion or max iterations.
pub async fn run_tool_loop(
    backend: &dyn Backend,
    handler: &ToolHandler,
    initial_request: CompletionRequest,
    config: ToolLoopConfig,
) -> Result<ToolLoopResult, BackendError> {
    let mut messages = initial_request.messages.clone();
    let mut request = initial_request;
    let mut iterations = 0;
    let mut all_tool_calls = Vec::new();

    loop {
        iterations += 1;
        debug!(iteration = iterations, "Tool loop iteration");

        if iterations > config.max_iterations {
            warn!("Tool loop reached max iterations");
            return Err(BackendError::ToolLoop {
                message: format!("Max iterations ({}) reached", config.max_iterations),
            });
        }

        // Make the request
        request.messages = messages.clone();
        let response = backend.complete(request.clone()).await?;

        // Check if we have tool calls
        if response.tool_calls.is_empty() {
            info!(iterations, "Tool loop completed");
            return Ok(ToolLoopResult {
                response,
                iterations,
                tool_calls: all_tool_calls,
            });
        }

        // Check finish reason
        if response.finish_reason != FinishReason::ToolUse {
            info!(
                finish_reason = ?response.finish_reason,
                "Tool loop ended without tool_use finish reason"
            );
            return Ok(ToolLoopResult {
                response,
                iterations,
                tool_calls: all_tool_calls,
            });
        }

        // Execute tool calls
        debug!(count = response.tool_calls.len(), "Executing tool calls");
        all_tool_calls.extend(response.tool_calls.clone());

        let results = handler.execute_batch(response.tool_calls.clone()).await;

        // Check for errors
        let has_errors = results.iter().any(|r| !r.success);
        if has_errors && !config.continue_on_error {
            let error_msg = results
                .iter()
                .filter(|r| !r.success)
                .map(|r| format!("{}: {}", r.name, r.content.as_text()))
                .collect::<Vec<_>>()
                .join("; ");

            return Err(BackendError::ToolExecution { message: error_msg });
        }

        // Add assistant message with tool calls
        if let Some(content) = &response.content {
            messages.push(Message::assistant(content));
        }

        // Add tool results
        for result in &results {
            messages.push(result.to_message());
        }
    }
}

/// Result of a tool loop execution.
#[derive(Debug)]
pub struct ToolLoopResult {
    /// Final response.
    pub response: CompletionResponse,
    /// Number of iterations.
    pub iterations: usize,
    /// All tool calls made.
    pub tool_calls: Vec<tachikoma_backends_core::ToolCall>,
}

impl ToolLoopResult {
    /// Get the final content.
    pub fn content(&self) -> Option<&str> {
        self.response.content.as_deref()
    }

    /// Get total tool calls.
    pub fn total_tool_calls(&self) -> usize {
        self.tool_calls.len()
    }
}
```

### 4. Streaming Tool Handler (src/tools/streaming.rs)

```rust
//! Streaming tool call handler.

use tachikoma_backends_core::{CompletionChunk, CompletionStream, ToolCall, ToolCallDelta};
use futures::{Stream, StreamExt};
use std::collections::HashMap;
use tracing::debug;

/// Accumulator for streaming tool calls.
#[derive(Debug, Default)]
pub struct StreamingToolAccumulator {
    /// Tool calls being built.
    pending: HashMap<usize, PendingToolCall>,
}

#[derive(Debug, Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl StreamingToolAccumulator {
    /// Create a new accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a chunk and extract any completed tool calls.
    pub fn process_chunk(&mut self, chunk: &CompletionChunk) -> Vec<ToolCall> {
        let mut completed = Vec::new();

        for delta in &chunk.tool_calls {
            let pending = self.pending.entry(delta.index).or_default();

            if let Some(id) = &delta.id {
                pending.id = id.clone();
            }
            if let Some(name) = &delta.name {
                pending.name = name.clone();
            }
            pending.arguments.push_str(&delta.arguments_delta);
        }

        // If this is the final chunk, extract all tool calls
        if chunk.is_final {
            for (_, pending) in self.pending.drain() {
                if !pending.id.is_empty() {
                    completed.push(ToolCall {
                        id: pending.id,
                        name: pending.name,
                        arguments: pending.arguments,
                    });
                }
            }
        }

        completed
    }

    /// Check if there are pending tool calls.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Finalize and return all pending tool calls.
    pub fn finalize(&mut self) -> Vec<ToolCall> {
        self.pending
            .drain()
            .filter_map(|(_, pending)| {
                if !pending.id.is_empty() {
                    Some(ToolCall {
                        id: pending.id,
                        name: pending.name,
                        arguments: pending.arguments,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Wrapper stream that accumulates tool calls while passing through chunks.
pub struct ToolAccumulatingStream {
    inner: CompletionStream,
    accumulator: StreamingToolAccumulator,
    collected_text: String,
    tool_calls: Vec<ToolCall>,
}

impl ToolAccumulatingStream {
    /// Create a new accumulating stream.
    pub fn new(stream: CompletionStream) -> Self {
        Self {
            inner: stream,
            accumulator: StreamingToolAccumulator::new(),
            collected_text: String::new(),
            tool_calls: Vec::new(),
        }
    }

    /// Collect the stream and return text + tool calls.
    pub async fn collect(mut self) -> Result<(String, Vec<ToolCall>), tachikoma_backends_core::BackendError> {
        while let Some(chunk) = self.inner.next().await {
            let chunk = chunk?;
            self.collected_text.push_str(&chunk.delta);

            let completed = self.accumulator.process_chunk(&chunk);
            self.tool_calls.extend(completed);

            if chunk.is_final {
                break;
            }
        }

        // Finalize any remaining tool calls
        self.tool_calls.extend(self.accumulator.finalize());

        Ok((self.collected_text, self.tool_calls))
    }
}
```

### 5. Module Exports (src/tools/mod.rs)

```rust
//! Tool support for Claude backend.

mod convert;
mod handler;
mod runner;
mod streaming;

pub use convert::{
    parse_tool_use, to_claude_tool, to_claude_tool_choice, to_claude_tool_result,
    to_claude_tool_results, to_claude_tools,
};
pub use handler::{ToolExecutor, ToolHandler};
pub use runner::{run_tool_loop, ToolLoopConfig, ToolLoopResult};
pub use streaming::{StreamingToolAccumulator, ToolAccumulatingStream};
```

---

## Testing Requirements

1. Tool definition conversion is correct
2. Tool_use blocks parse correctly
3. Tool results format properly
4. Parallel execution respects limits
5. Streaming accumulator builds complete calls
6. Tool loop terminates correctly

---

## Related Specs

- Depends on: [056-claude-api-client.md](056-claude-api-client.md)
- Depends on: [054-tool-definitions.md](054-tool-definitions.md)
- Next: [060-claude-errors.md](060-claude-errors.md)
