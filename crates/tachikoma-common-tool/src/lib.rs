//! Tool types for LLM function calling.
//!
//! This crate provides types for defining tools that can be called by LLMs,
//! handling tool invocations, and managing tool execution results.

#![warn(missing_docs)]

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

// Re-export common types
pub use serde_json::Value as JsonValue;

/// A delta for streaming tool call construction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCallDelta {
    /// Index of the tool call being built.
    pub index: usize,
    /// Tool call ID (if provided in this delta).
    pub id: Option<String>,
    /// Tool name (if provided in this delta).
    pub name: Option<String>,
    /// Arguments delta (partial JSON string).
    pub arguments_delta: String,
}

/// A message type for LLM conversations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    /// Message role.
    pub role: String,
    /// Message content.
    pub content: String,
    /// Tool call ID for tool results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_tool_result() {
        let msg = Message::tool_result("call_123", "File content here");
        assert_eq!(msg.role, "tool");
        assert_eq!(msg.content, "File content here");
        assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
    }
}