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

    #[test]
    fn test_multiple_tool_calls() {
        let mut acc = ToolCallAccumulator::new();

        // First tool call
        acc.process_delta(ToolCallDelta {
            index: 0,
            id: Some("call_1".to_string()),
            name: Some("read_file".to_string()),
            arguments_delta: r#"{"path": "/tmp/test1"}"#.to_string(),
        });

        // Second tool call
        acc.process_delta(ToolCallDelta {
            index: 1,
            id: Some("call_2".to_string()),
            name: Some("write_file".to_string()),
            arguments_delta: r#"{"path": "/tmp/test2", "#.to_string(),
        });

        acc.process_delta(ToolCallDelta {
            index: 1,
            id: None,
            name: None,
            arguments_delta: r#""content": "hello"}"#.to_string(),
        });

        acc.finalize();

        let completed = acc.completed();
        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0].id, "call_1");
        assert_eq!(completed[1].id, "call_2");
        assert_eq!(completed[1].arguments, r#"{"path": "/tmp/test2", "content": "hello"}"#);
    }

    #[test]
    fn test_complete_individual() {
        let mut acc = ToolCallAccumulator::new();

        acc.process_delta(ToolCallDelta {
            index: 0,
            id: Some("call_1".to_string()),
            name: Some("tool1".to_string()),
            arguments_delta: r#"{"arg": 1}"#.to_string(),
        });

        acc.process_delta(ToolCallDelta {
            index: 1,
            id: Some("call_2".to_string()),
            name: Some("tool2".to_string()),
            arguments_delta: r#"{"arg": 2}"#.to_string(),
        });

        // Complete first tool call
        acc.complete(0);
        
        assert_eq!(acc.completed().len(), 1);
        assert_eq!(acc.pending_count(), 1);
        assert!(acc.has_pending());

        // Complete second tool call
        acc.complete(1);
        
        assert_eq!(acc.completed().len(), 2);
        assert_eq!(acc.pending_count(), 0);
        assert!(!acc.has_pending());
    }

    #[test]
    fn test_take_completed() {
        let mut acc = ToolCallAccumulator::new();

        acc.process_delta(ToolCallDelta {
            index: 0,
            id: Some("call_1".to_string()),
            name: Some("tool1".to_string()),
            arguments_delta: r#"{"arg": 1}"#.to_string(),
        });

        acc.finalize();

        let completed = acc.take_completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(acc.completed().len(), 0);
    }

    #[test]
    fn test_incomplete_tool_call() {
        let mut acc = ToolCallAccumulator::new();

        // Only provide partial information
        acc.process_delta(ToolCallDelta {
            index: 0,
            id: Some("call_1".to_string()),
            name: None, // Missing name
            arguments_delta: r#"{"arg": 1}"#.to_string(),
        });

        acc.finalize();

        // Should not create a completed tool call without required fields
        assert_eq!(acc.completed().len(), 0);
    }
}