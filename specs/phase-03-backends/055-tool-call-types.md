# 055 - Tool Call Types (Request/Response)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 055
**Status:** Planned
**Dependencies:** 054-tool-definitions
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define the types for tool call requests and responses. This includes the structure of tool invocations from the LLM, parsing of arguments, and formatting of results to send back to the model.

---

## Acceptance Criteria

- [x] `ToolCall` struct for LLM tool invocations
- [x] `ToolResult` struct for tool execution results
- [x] Argument parsing and validation
- [x] Error handling for tool failures
- [x] Serialization to provider-specific formats
- [x] Streaming tool call accumulation

---

## Implementation Details

### 1. Tool Call Types (src/tool/call.rs)

```rust
//! Tool call types for LLM function invocations.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// A tool call requested by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments as a JSON string.
    pub arguments: String,
}

impl ToolCall {
    /// Create a new tool call.
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    /// Parse the arguments as JSON.
    pub fn parse_arguments(&self) -> Result<JsonValue, ToolCallError> {
        serde_json::from_str(&self.arguments)
            .map_err(|e| ToolCallError::InvalidArguments {
                tool: self.name.clone(),
                reason: e.to_string(),
            })
    }

    /// Parse arguments into a specific type.
    pub fn parse_arguments_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, ToolCallError> {
        serde_json::from_str(&self.arguments)
            .map_err(|e| ToolCallError::InvalidArguments {
                tool: self.name.clone(),
                reason: e.to_string(),
            })
    }

    /// Get a string argument.
    pub fn get_string(&self, key: &str) -> Result<String, ToolCallError> {
        let args = self.parse_arguments()?;
        args.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ToolCallError::MissingArgument {
                tool: self.name.clone(),
                argument: key.to_string(),
            })
    }

    /// Get an optional string argument.
    pub fn get_string_opt(&self, key: &str) -> Result<Option<String>, ToolCallError> {
        let args = self.parse_arguments()?;
        Ok(args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()))
    }

    /// Get an integer argument.
    pub fn get_int(&self, key: &str) -> Result<i64, ToolCallError> {
        let args = self.parse_arguments()?;
        args.get(key)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ToolCallError::MissingArgument {
                tool: self.name.clone(),
                argument: key.to_string(),
            })
    }

    /// Get an optional integer argument.
    pub fn get_int_opt(&self, key: &str) -> Result<Option<i64>, ToolCallError> {
        let args = self.parse_arguments()?;
        Ok(args.get(key).and_then(|v| v.as_i64()))
    }

    /// Get a boolean argument.
    pub fn get_bool(&self, key: &str) -> Result<bool, ToolCallError> {
        let args = self.parse_arguments()?;
        args.get(key)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| ToolCallError::MissingArgument {
                tool: self.name.clone(),
                argument: key.to_string(),
            })
    }

    /// Get an optional boolean argument with default.
    pub fn get_bool_or(&self, key: &str, default: bool) -> Result<bool, ToolCallError> {
        let args = self.parse_arguments()?;
        Ok(args.get(key).and_then(|v| v.as_bool()).unwrap_or(default))
    }
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this is a result for.
    pub tool_call_id: String,
    /// Name of the tool.
    pub name: String,
    /// Result content.
    pub content: ToolResultContent,
    /// Whether the execution was successful.
    pub success: bool,
    /// Execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
}

impl ToolResult {
    /// Create a successful text result.
    pub fn success(tool_call_id: impl Into<String>, name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: ToolResultContent::Text(content.into()),
            success: true,
            execution_time_ms: None,
        }
    }

    /// Create a successful JSON result.
    pub fn success_json(tool_call_id: impl Into<String>, name: impl Into<String>, value: JsonValue) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: ToolResultContent::Json(value),
            success: true,
            execution_time_ms: None,
        }
    }

    /// Create an error result.
    pub fn error(tool_call_id: impl Into<String>, name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: ToolResultContent::Error(error.into()),
            success: false,
            execution_time_ms: None,
        }
    }

    /// Set execution time.
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = Some(ms);
        self
    }

    /// Convert to a message for the LLM.
    pub fn to_message(&self) -> crate::Message {
        let content = match &self.content {
            ToolResultContent::Text(s) => s.clone(),
            ToolResultContent::Json(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            ToolResultContent::Error(e) => format!("Error: {}", e),
        };

        crate::Message::tool_result(&self.tool_call_id, content)
    }

    /// Convert to Claude API format.
    pub fn to_claude_format(&self) -> JsonValue {
        let content = match &self.content {
            ToolResultContent::Text(s) => s.clone(),
            ToolResultContent::Json(v) => serde_json::to_string(v).unwrap_or_default(),
            ToolResultContent::Error(e) => format!("Error: {}", e),
        };

        serde_json::json!({
            "type": "tool_result",
            "tool_use_id": self.tool_call_id,
            "content": content,
            "is_error": !self.success
        })
    }

    /// Convert to OpenAI API format.
    pub fn to_openai_format(&self) -> JsonValue {
        let content = match &self.content {
            ToolResultContent::Text(s) => s.clone(),
            ToolResultContent::Json(v) => serde_json::to_string(v).unwrap_or_default(),
            ToolResultContent::Error(e) => format!("Error: {}", e),
        };

        serde_json::json!({
            "role": "tool",
            "tool_call_id": self.tool_call_id,
            "content": content
        })
    }
}

/// Content of a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Plain text result.
    Text(String),
    /// Structured JSON result.
    Json(JsonValue),
    /// Error message.
    Error(String),
}

impl ToolResultContent {
    /// Get as text.
    pub fn as_text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Json(v) => serde_json::to_string_pretty(v).unwrap_or_default(),
            Self::Error(e) => format!("Error: {}", e),
        }
    }
}

/// Tool call error.
#[derive(Debug, thiserror::Error)]
pub enum ToolCallError {
    #[error("invalid arguments for tool '{tool}': {reason}")]
    InvalidArguments { tool: String, reason: String },

    #[error("missing required argument '{argument}' for tool '{tool}'")]
    MissingArgument { tool: String, argument: String },

    #[error("tool '{0}' not found")]
    ToolNotFound(String),

    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("tool execution timed out after {0}ms")]
    Timeout(u64),

    #[error("tool execution denied: {0}")]
    Denied(String),
}
```

### 2. Tool Call Accumulator (src/tool/accumulator.rs)

```rust
//! Accumulator for streaming tool calls.

use super::ToolCall;
use std::collections::HashMap;

/// Accumulates streaming tool call deltas into complete tool calls.
#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    /// Tool calls being built, keyed by index.
    pending: HashMap<usize, PendingToolCall>,
    /// Completed tool calls.
    completed: Vec<ToolCall>,
}

#[derive(Debug, Default)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCallAccumulator {
    /// Create a new accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a tool call delta from streaming.
    pub fn process_delta(&mut self, delta: crate::ToolCallDelta) {
        let pending = self.pending.entry(delta.index).or_default();

        if let Some(id) = delta.id {
            pending.id = id;
        }
        if let Some(name) = delta.name {
            pending.name = name;
        }
        pending.arguments.push_str(&delta.arguments_delta);
    }

    /// Mark a tool call as complete by index.
    pub fn complete(&mut self, index: usize) {
        if let Some(pending) = self.pending.remove(&index) {
            if !pending.id.is_empty() && !pending.name.is_empty() {
                self.completed.push(ToolCall {
                    id: pending.id,
                    name: pending.name,
                    arguments: pending.arguments,
                });
            }
        }
    }

    /// Finalize all pending tool calls.
    pub fn finalize(&mut self) {
        let indices: Vec<usize> = self.pending.keys().copied().collect();
        for index in indices {
            self.complete(index);
        }
    }

    /// Get completed tool calls.
    pub fn completed(&self) -> &[ToolCall] {
        &self.completed
    }

    /// Take completed tool calls.
    pub fn take_completed(&mut self) -> Vec<ToolCall> {
        std::mem::take(&mut self.completed)
    }

    /// Check if there are any pending tool calls.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Get the number of pending tool calls.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolCallDelta;

    #[test]
    fn test_accumulate_tool_call() {
        let mut acc = ToolCallAccumulator::new();

        // First delta with id and name
        acc.process_delta(ToolCallDelta {
            index: 0,
            id: Some("call_123".to_string()),
            name: Some("read_file".to_string()),
            arguments_delta: r#"{"pa"#.to_string(),
        });

        // Second delta with more arguments
        acc.process_delta(ToolCallDelta {
            index: 0,
            id: None,
            name: None,
            arguments_delta: r#"th": "/tmp/test"}"#.to_string(),
        });

        // Finalize
        acc.finalize();

        let completed = acc.completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, "call_123");
        assert_eq!(completed[0].name, "read_file");
        assert_eq!(completed[0].arguments, r#"{"path": "/tmp/test"}"#);
    }
}
```

### 3. Tool Execution Context (src/tool/context.rs)

```rust
//! Tool execution context and helpers.

use super::{ToolCall, ToolCallError, ToolResult};
use std::time::{Duration, Instant};

/// Context for tool execution.
#[derive(Debug)]
pub struct ToolExecutionContext {
    /// Maximum execution time.
    pub timeout: Duration,
    /// Working directory.
    pub working_dir: Option<String>,
    /// Whether to capture stderr.
    pub capture_stderr: bool,
    /// Environment variables.
    pub env: std::collections::HashMap<String, String>,
    /// Approval callback for dangerous operations.
    approval_callback: Option<Box<dyn Fn(&ToolCall) -> bool + Send + Sync>>,
}

impl Default for ToolExecutionContext {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            working_dir: None,
            capture_stderr: true,
            env: std::collections::HashMap::new(),
            approval_callback: None,
        }
    }
}

impl ToolExecutionContext {
    /// Create a new context with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the working directory.
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the approval callback.
    pub fn with_approval<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ToolCall) -> bool + Send + Sync + 'static,
    {
        self.approval_callback = Some(Box::new(callback));
        self
    }

    /// Check if a tool call is approved.
    pub fn is_approved(&self, call: &ToolCall) -> bool {
        self.approval_callback
            .as_ref()
            .map(|cb| cb(call))
            .unwrap_or(true)
    }
}

/// Helper for timed tool execution.
pub struct TimedExecution {
    start: Instant,
    tool_name: String,
    tool_call_id: String,
}

impl TimedExecution {
    /// Start a timed execution.
    pub fn start(call: &ToolCall) -> Self {
        Self {
            start: Instant::now(),
            tool_name: call.name.clone(),
            tool_call_id: call.id.clone(),
        }
    }

    /// Complete with success.
    pub fn success(self, content: impl Into<String>) -> ToolResult {
        ToolResult::success(&self.tool_call_id, &self.tool_name, content)
            .with_execution_time(self.start.elapsed().as_millis() as u64)
    }

    /// Complete with JSON result.
    pub fn success_json(self, value: serde_json::Value) -> ToolResult {
        ToolResult::success_json(&self.tool_call_id, &self.tool_name, value)
            .with_execution_time(self.start.elapsed().as_millis() as u64)
    }

    /// Complete with error.
    pub fn error(self, error: impl Into<String>) -> ToolResult {
        ToolResult::error(&self.tool_call_id, &self.tool_name, error)
            .with_execution_time(self.start.elapsed().as_millis() as u64)
    }

    /// Get elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Batch tool call processor.
pub struct ToolBatch {
    calls: Vec<ToolCall>,
    results: Vec<ToolResult>,
}

impl ToolBatch {
    /// Create a new batch from tool calls.
    pub fn new(calls: Vec<ToolCall>) -> Self {
        Self {
            calls,
            results: Vec::new(),
        }
    }

    /// Get the tool calls.
    pub fn calls(&self) -> &[ToolCall] {
        &self.calls
    }

    /// Add a result.
    pub fn add_result(&mut self, result: ToolResult) {
        self.results.push(result);
    }

    /// Check if all calls have results.
    pub fn is_complete(&self) -> bool {
        self.results.len() >= self.calls.len()
    }

    /// Get results.
    pub fn results(&self) -> &[ToolResult] {
        &self.results
    }

    /// Take results.
    pub fn take_results(self) -> Vec<ToolResult> {
        self.results
    }

    /// Convert results to messages.
    pub fn to_messages(&self) -> Vec<crate::Message> {
        self.results.iter().map(|r| r.to_message()).collect()
    }
}
```

### 4. Module Exports (src/tool/mod.rs)

```rust
//! Tool types for LLM function calling.

mod accumulator;
mod call;
mod context;
mod definition;
mod predefined;
mod registry;

pub use accumulator::ToolCallAccumulator;
pub use call::{ToolCall, ToolCallError, ToolResult, ToolResultContent};
pub use context::{TimedExecution, ToolBatch, ToolExecutionContext};
pub use definition::{
    DangerLevel, ParameterType, ToolDefinition, ToolDefinitionBuilder,
    ToolMetadata, ToolParameter, ToolParameters, ToolValidationError,
};
pub use predefined::*;
pub use registry::ToolRegistry;
```

---

## Testing Requirements

1. Tool call argument parsing works correctly
2. Tool results serialize to correct formats
3. Accumulator handles streaming deltas
4. Timed execution tracks duration
5. Batch processing completes correctly

---

## Related Specs

- Depends on: [054-tool-definitions.md](054-tool-definitions.md)
- Next: [056-claude-api-client.md](056-claude-api-client.md)
- Used by: All backend implementations
