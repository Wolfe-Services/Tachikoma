//! Agent plugin trait and types
//!
//! Agents are LLM backends that execute tool calls in an agentic loop.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{PluginManifest, Result};

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model name/identifier
    pub model: String,
    
    /// Maximum tokens for output
    pub max_tokens: u32,
    
    /// API endpoint override
    pub endpoint: Option<String>,
    
    /// Additional model-specific settings
    #[serde(default)]
    pub settings: Value,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            endpoint: None,
            settings: Value::Null,
        }
    }
}

/// Configuration for the agentic loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Maximum iterations
    pub max_iterations: usize,
    
    /// Token limit before fresh context
    pub redline_threshold: u32,
    
    /// Delay between iterations (ms)
    pub iteration_delay_ms: u64,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            redline_threshold: 150_000,
            iteration_delay_ms: 100,
        }
    }
}

/// Tool definition for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// JSON Schema for input parameters
    pub input_schema: Value,
}

/// Result from running the agentic loop
#[derive(Debug, Clone)]
pub struct LoopResult {
    /// Number of iterations executed
    pub iterations: usize,
    
    /// Total input tokens used
    pub input_tokens: u32,
    
    /// Total output tokens used
    pub output_tokens: u32,
    
    /// Final text output (if any)
    pub final_text: String,
    
    /// How the loop stopped
    pub stop_reason: LoopStopReason,
}

impl LoopResult {
    /// Get total tokens used
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
    
    /// Estimate cost (Claude Sonnet pricing)
    pub fn estimated_cost(&self) -> f64 {
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * 15.0;
        input_cost + output_cost
    }
}

/// Why the loop stopped
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopStopReason {
    /// Task completed successfully
    Completed,
    /// Hit max iterations
    MaxIterations,
    /// Hit token redline
    Redline,
    /// User requested pause
    Paused,
    /// Error occurred
    Error,
}

/// Event emitted during loop execution
#[derive(Debug, Clone)]
pub enum LoopEvent {
    /// New iteration started
    IterationStart(usize),
    
    /// Tool call initiated
    ToolCall { name: String, input: Value },
    
    /// Tool call completed
    ToolResult { name: String, output: String, success: bool },
    
    /// Text output from model
    Text(String),
    
    /// Token usage update
    TokenUpdate { input: u32, output: u32 },
    
    /// Spec/task completed
    SpecComplete(u32),
    
    /// Approaching or hit redline
    Redline,
}

/// Trait for agent plugins
///
/// Implement this trait to create a new agent backend.
#[async_trait]
pub trait AgentPlugin: Send + Sync {
    /// Get plugin metadata
    fn manifest(&self) -> &PluginManifest;
    
    /// Initialize the agent with configuration
    async fn init(&mut self, config: &AgentConfig) -> Result<()>;
    
    /// Check if the agent is ready to run
    fn is_ready(&self) -> bool;
    
    /// Run the agentic loop for a task
    ///
    /// # Arguments
    /// * `system_prompt` - The system prompt to use
    /// * `task` - The initial task/user message
    /// * `tools` - Available tools for the agent
    /// * `config` - Loop configuration
    ///
    /// # Returns
    /// The result of running the loop
    async fn run_loop(
        &self,
        system_prompt: &str,
        task: &str,
        tools: &[ToolDefinition],
        config: &LoopConfig,
    ) -> Result<LoopResult>;
    
    /// Get events from the current execution
    ///
    /// This can be used to stream events during loop execution.
    /// Returns None if no events are available.
    async fn poll_event(&self) -> Option<LoopEvent>;
    
    /// Request the loop to pause
    fn request_pause(&self);
    
    /// Resume a paused loop
    fn resume(&self);
    
    /// Stop the loop immediately
    fn stop(&self);
}
