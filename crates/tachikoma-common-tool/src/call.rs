//! Tool call types for LLM function invocations.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_new() {
        let call = ToolCall::new("call_123", "read_file", r#"{"path": "/tmp/test"}"#);
        assert_eq!(call.id, "call_123");
        assert_eq!(call.name, "read_file");
        assert_eq!(call.arguments, r#"{"path": "/tmp/test"}"#);
    }

    #[test]
    fn test_tool_call_parse_arguments() {
        let call = ToolCall::new("call_123", "read_file", r#"{"path": "/tmp/test", "max_lines": 100}"#);
        let args = call.parse_arguments().unwrap();
        assert_eq!(args["path"], "/tmp/test");
        assert_eq!(args["max_lines"], 100);
    }

    #[test]
    fn test_tool_call_get_string() {
        let call = ToolCall::new("call_123", "read_file", r#"{"path": "/tmp/test"}"#);
        assert_eq!(call.get_string("path").unwrap(), "/tmp/test");
        assert!(call.get_string("missing").is_err());
    }

    #[test]
    fn test_tool_call_get_optional() {
        let call = ToolCall::new("call_123", "read_file", r#"{"path": "/tmp/test"}"#);
        assert_eq!(call.get_string_opt("path").unwrap(), Some("/tmp/test".to_string()));
        assert_eq!(call.get_string_opt("missing").unwrap(), None);
    }

    #[test]
    fn test_tool_call_get_int() {
        let call = ToolCall::new("call_123", "read_file", r#"{"max_lines": 100}"#);
        assert_eq!(call.get_int("max_lines").unwrap(), 100);
        assert!(call.get_int("missing").is_err());
    }

    #[test]
    fn test_tool_call_get_bool() {
        let call = ToolCall::new("call_123", "read_file", r#"{"recursive": true}"#);
        assert_eq!(call.get_bool("recursive").unwrap(), true);
        assert_eq!(call.get_bool_or("recursive", false).unwrap(), true);
        assert_eq!(call.get_bool_or("missing", false).unwrap(), false);
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("call_123", "read_file", "file content");
        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.name, "read_file");
        assert!(result.success);
        match result.content {
            ToolResultContent::Text(s) => assert_eq!(s, "file content"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("call_123", "read_file", "File not found");
        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.name, "read_file");
        assert!(!result.success);
        match result.content {
            ToolResultContent::Error(e) => assert_eq!(e, "File not found"),
            _ => panic!("Expected error content"),
        }
    }

    #[test]
    fn test_tool_result_json() {
        let json_value = serde_json::json!({"lines": 42, "size": 1024});
        let result = ToolResult::success_json("call_123", "read_file", json_value.clone());
        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.name, "read_file");
        assert!(result.success);
        match result.content {
            ToolResultContent::Json(v) => assert_eq!(v, json_value),
            _ => panic!("Expected JSON content"),
        }
    }

    #[test]
    fn test_tool_result_with_timing() {
        let result = ToolResult::success("call_123", "read_file", "content")
            .with_execution_time(150);
        assert_eq!(result.execution_time_ms, Some(150));
    }

    #[test]
    fn test_tool_result_claude_format() {
        let result = ToolResult::success("call_123", "read_file", "file content");
        let claude_format = result.to_claude_format();
        
        assert_eq!(claude_format["type"], "tool_result");
        assert_eq!(claude_format["tool_use_id"], "call_123");
        assert_eq!(claude_format["content"], "file content");
        assert_eq!(claude_format["is_error"], false);
    }

    #[test]
    fn test_tool_result_openai_format() {
        let result = ToolResult::success("call_123", "read_file", "file content");
        let openai_format = result.to_openai_format();
        
        assert_eq!(openai_format["role"], "tool");
        assert_eq!(openai_format["tool_call_id"], "call_123");
        assert_eq!(openai_format["content"], "file content");
    }

    #[test]
    fn test_tool_result_content_as_text() {
        let text_content = ToolResultContent::Text("hello".to_string());
        assert_eq!(text_content.as_text(), "hello");

        let json_content = ToolResultContent::Json(serde_json::json!({"key": "value"}));
        assert!(json_content.as_text().contains("key"));

        let error_content = ToolResultContent::Error("failed".to_string());
        assert_eq!(error_content.as_text(), "Error: failed");
    }
}