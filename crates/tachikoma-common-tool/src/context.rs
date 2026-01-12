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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_defaults() {
        let ctx = ToolExecutionContext::new();
        assert_eq!(ctx.timeout, Duration::from_secs(30));
        assert!(ctx.working_dir.is_none());
        assert!(ctx.capture_stderr);
        assert!(ctx.env.is_empty());
    }

    #[test]
    fn test_context_builder() {
        let ctx = ToolExecutionContext::new()
            .with_timeout(Duration::from_secs(60))
            .with_working_dir("/tmp")
            .with_env("KEY", "value");

        assert_eq!(ctx.timeout, Duration::from_secs(60));
        assert_eq!(ctx.working_dir, Some("/tmp".to_string()));
        assert_eq!(ctx.env.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn test_context_approval() {
        let mut ctx = ToolExecutionContext::new()
            .with_approval(|call| call.name != "dangerous_tool");

        let safe_call = ToolCall::new("call_1", "safe_tool", "{}");
        let dangerous_call = ToolCall::new("call_2", "dangerous_tool", "{}");

        assert!(ctx.is_approved(&safe_call));
        assert!(!ctx.is_approved(&dangerous_call));
    }

    #[test]
    fn test_timed_execution() {
        let call = ToolCall::new("call_123", "test_tool", "{}");
        let timer = TimedExecution::start(&call);

        // Simulate some work
        std::thread::sleep(Duration::from_millis(10));

        let result = timer.success("test output");

        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.name, "test_tool");
        assert!(result.success);
        assert!(result.execution_time_ms.is_some());
        assert!(result.execution_time_ms.unwrap() >= 10);
    }

    #[test]
    fn test_tool_batch() {
        let call1 = ToolCall::new("call_1", "tool1", "{}");
        let call2 = ToolCall::new("call_2", "tool2", "{}");
        let mut batch = ToolBatch::new(vec![call1, call2]);

        assert_eq!(batch.calls().len(), 2);
        assert!(!batch.is_complete());

        batch.add_result(ToolResult::success("call_1", "tool1", "result1"));
        assert!(!batch.is_complete());

        batch.add_result(ToolResult::success("call_2", "tool2", "result2"));
        assert!(batch.is_complete());

        let results = batch.take_results();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_batch_to_messages() {
        let calls = vec![ToolCall::new("call_1", "tool1", "{}")];
        let mut batch = ToolBatch::new(calls);
        
        batch.add_result(ToolResult::success("call_1", "tool1", "result1"));
        
        let messages = batch.to_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "tool");
        assert_eq!(messages[0].content, "result1");
        assert_eq!(messages[0].tool_call_id, Some("call_1".to_string()));
    }
}