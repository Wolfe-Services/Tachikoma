//! Claude API Client - Handles communication with Claude Sonnet
//!
//! Uses the Messages API with streaming and tool use support.
//! Includes exploration detection to prevent context burn.

use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::mpsc;

use crate::primitives::{execute_tool, get_tool_definitions, ToolDefinition};

// ============================================================================
// Exploration Detection
// ============================================================================

/// Metrics for detecting exploration spirals
/// 
/// Tracks tool usage across iterations to identify when the agent
/// is spending too much time exploring and not enough time coding.
#[derive(Debug, Default, Clone)]
pub struct IterationMetrics {
    pub read_file_count: u32,
    pub list_files_count: u32,
    pub code_search_count: u32,
    pub edit_file_count: u32,
    pub bash_count: u32,
    pub beads_count: u32,
}

impl IterationMetrics {
    /// Record a tool call
    pub fn record_tool(&mut self, name: &str) {
        match name {
            "read_file" => self.read_file_count += 1,
            "list_files" => self.list_files_count += 1,
            "code_search" => self.code_search_count += 1,
            "edit_file" => self.edit_file_count += 1,
            "bash" => self.bash_count += 1,
            "beads" => self.beads_count += 1,
            _ => {}
        }
    }

    /// Total exploration calls (read + list + search)
    pub fn total_exploration(&self) -> u32 {
        self.read_file_count + self.list_files_count + self.code_search_count
    }

    /// Total action calls (edit + bash)
    pub fn total_action(&self) -> u32 {
        self.edit_file_count + self.bash_count
    }

    /// Check if we're in an exploration spiral
    /// 
    /// Returns true if:
    /// - 5+ exploration calls (read_file, list_files, code_search)
    /// - 0 action calls (edit_file, bash)
    pub fn is_exploration_heavy(&self) -> bool {
        self.total_exploration() >= 5 && self.total_action() == 0
    }

    /// Generate an intervention message for exploration spiral
    pub fn intervention_message(&self) -> String {
        format!(
            "[INTERVENTION] You've made {} exploration calls but 0 edits.\n\
             The task description contains file paths. Create files NOW with:\n\
             edit_file path=\"<path>\" old_string=\"\" new_string=\"<content>\"\n\n\
             Current stats:\n\
             - read_file: {}\n\
             - list_files: {}\n\
             - code_search: {}\n\
             - edit_file: {} ← NEED MORE OF THIS\n\
             - bash: {}\n\n\
             NO MORE EXPLORATION. START CODING.",
            self.total_exploration(),
            self.read_file_count,
            self.list_files_count,
            self.code_search_count,
            self.edit_file_count,
            self.bash_count
        )
    }
}

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514"; // Sonnet for agentic work

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Content block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

/// Request body for Claude API
#[derive(Debug, Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
    tools: Vec<ApiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ApiTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// Response from Claude API
#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Stream event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: ApiResponse },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: ContentDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaContent,
        usage: Option<Usage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: ApiError },
}

#[derive(Debug, Deserialize)]
pub struct ContentDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub partial_json: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageDeltaContent {
    pub stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Claude client for making API calls
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    project_root: std::path::PathBuf,
}

impl ClaudeClient {
    pub fn new(api_key: String, project_root: impl AsRef<Path>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Run a complete agentic loop for a task
    ///
    /// Returns when:
    /// - The model completes without tool calls (end_turn)
    /// - Max iterations reached
    /// - Token redline exceeded (needs fresh context)
    /// - An error occurs
    ///
    /// Includes exploration detection: if the agent makes 5+ exploration calls
    /// (read_file, list_files, code_search) without any edits, an intervention
    /// message is injected to nudge it toward action.
    pub async fn run_agentic_loop(
        &self,
        system_prompt: &str,
        initial_message: &str,
        max_iterations: usize,
        redline_threshold: u32,
        output_tx: Option<mpsc::Sender<String>>,
    ) -> Result<LoopResult> {
        let mut messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: initial_message.to_string(),
            }],
        }];

        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;
        let mut iterations = 0;
        
        // Track tool usage to detect exploration spirals
        let mut session_metrics = IterationMetrics::default();
        let mut intervention_sent = false;

        loop {
            iterations += 1;

            if iterations > max_iterations {
                tracing::warn!("Max iterations ({}) reached", max_iterations);
                return Ok(LoopResult {
                    iterations,
                    total_input_tokens,
                    total_output_tokens,
                    final_text: String::new(),
                    messages,
                    stop_reason: StopReason::MaxIterations,
                });
            }

            // Log iteration with metrics
            if let Some(tx) = &output_tx {
                let _ = tx
                    .send(format!(
                        "\n--- Iteration {} [explore:{}/action:{}] ---\n",
                        iterations,
                        session_metrics.total_exploration(),
                        session_metrics.total_action()
                    ))
                    .await;
            }
            
            // Check for exploration spiral every 3 iterations
            if iterations % 3 == 0 && session_metrics.is_exploration_heavy() && !intervention_sent {
                intervention_sent = true;
                let intervention = session_metrics.intervention_message();
                
                if let Some(tx) = &output_tx {
                    let _ = tx.send(format!("\n{}\n", intervention)).await;
                }
                
                tracing::warn!(
                    "Exploration spiral detected: {} exploration calls, {} edits",
                    session_metrics.total_exploration(),
                    session_metrics.edit_file_count
                );
                
                // Inject intervention as a user message
                messages.push(Message {
                    role: Role::User,
                    content: vec![ContentBlock::Text { text: intervention }],
                });
            }

            // Make API call
            let response = self
                .call_api(system_prompt, &messages, output_tx.clone())
                .await?;

            // Track token usage
            if let Some(usage) = &response.usage {
                total_input_tokens += usage.input_tokens;
                total_output_tokens += usage.output_tokens;
            }

            // Check for context redline - STOP if exceeded
            let total_tokens = total_input_tokens + total_output_tokens;
            if total_tokens > redline_threshold {
                tracing::warn!(
                    "Context redline exceeded: {} tokens (threshold: {}). Stopping for fresh context.",
                    total_tokens, redline_threshold
                );
                if let Some(tx) = &output_tx {
                    let _ = tx
                        .send(format!(
                            "\n[REDLINE EXCEEDED: {} tokens > {} threshold - stopping for fresh context]\n",
                            total_tokens, redline_threshold
                        ))
                        .await;
                }
                return Ok(LoopResult {
                    iterations,
                    total_input_tokens,
                    total_output_tokens,
                    final_text: String::new(),
                    messages,
                    stop_reason: StopReason::Redline,
                });
            }

            // Add assistant response to messages
            messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            // Check if we need to handle tool calls
            let tool_calls: Vec<_> = response
                .content
                .iter()
                .filter_map(|c| match c {
                    ContentBlock::ToolUse { id, name, input } => {
                        Some((id.clone(), name.clone(), input.clone()))
                    }
                    _ => None,
                })
                .collect();

            if tool_calls.is_empty() {
                // No tool calls - check stop reason
                if response.stop_reason.as_deref() == Some("end_turn") {
                    tracing::info!("Loop completed: end_turn");
                    // Extract final text before returning
                    let final_text = messages
                        .iter()
                        .rev()
                        .find_map(|m| {
                            if matches!(m.role, Role::Assistant) {
                                m.content.iter().find_map(|c| match c {
                                    ContentBlock::Text { text } => Some(text.clone()),
                                    _ => None,
                                })
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    return Ok(LoopResult {
                        iterations,
                        total_input_tokens,
                        total_output_tokens,
                        final_text,
                        messages,
                        stop_reason: StopReason::Completed,
                    });
                }
            } else {
                // Execute tool calls and add results
                let mut tool_results = Vec::new();

                for (id, name, input) in tool_calls {
                    // Record tool call for exploration detection
                    session_metrics.record_tool(&name);
                    
                    // Reset intervention flag if we finally make an edit
                    if name == "edit_file" {
                        intervention_sent = false; // Allow future interventions if we spiral again
                    }
                    
                    if let Some(tx) = &output_tx {
                        let _ = tx
                            .send(format!("\n[Executing tool: {}]\n", name))
                            .await;
                    }

                    let result = execute_tool(&name, &input, &self.project_root).await;

                    // Log result preview
                    if let Some(tx) = &output_tx {
                        let preview = if result.output.len() > 500 {
                            // Safe truncation respecting UTF-8 character boundaries
                            let mut end = 500;
                            while !result.output.is_char_boundary(end) && end > 0 {
                                end -= 1;
                            }
                            format!("{}...[truncated]", &result.output[..end])
                        } else {
                            result.output.clone()
                        };
                        let _ = tx.send(format!("{}\n", preview)).await;
                    }

                    tool_results.push(ContentBlock::ToolResult {
                        tool_use_id: id,
                        content: if result.success {
                            result.output
                        } else {
                            result.error.unwrap_or_else(|| "Unknown error".to_string())
                        },
                        is_error: if result.success { None } else { Some(true) },
                    });
                }

                // Add tool results as user message
                messages.push(Message {
                    role: Role::User,
                    content: tool_results,
                });
            }
        }
    }

    /// Make a single API call with streaming
    async fn call_api(
        &self,
        system_prompt: &str,
        messages: &[Message],
        output_tx: Option<mpsc::Sender<String>>,
    ) -> Result<ApiResponse> {
        let tools: Vec<ApiTool> = get_tool_definitions()
            .into_iter()
            .map(|t| ApiTool {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
            })
            .collect();

        let request = ApiRequest {
            model: CLAUDE_MODEL.to_string(),
            max_tokens: 8192,
            system: system_prompt.to_string(),
            messages: messages.to_vec(),
            tools,
            stream: Some(true),
        };

        let response = self
            .client
            .post(CLAUDE_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Claude API error {}: {}", status, body);
        }

        // Process SSE stream
        self.process_stream(response, output_tx).await
    }

    /// Process SSE stream and reconstruct response
    async fn process_stream(
        &self,
        response: reqwest::Response,
        output_tx: Option<mpsc::Sender<String>>,
    ) -> Result<ApiResponse> {
        let mut content_blocks: Vec<ContentBlock> = Vec::new();
        let mut current_text = String::new();
        let mut current_tool_input = String::new();
        let mut current_block_index: Option<usize> = None;
        let mut stop_reason: Option<String> = None;
        let mut usage: Option<Usage> = None;
        let mut message_id = String::new();

        let stream = response.bytes_stream();
        let mut buffer = String::new();

        tokio::pin!(stream);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read stream chunk")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() || line == "event: message_start" || line.starts_with("event:") {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }

                    match serde_json::from_str::<StreamEvent>(data) {
                        Ok(event) => match event {
                            StreamEvent::MessageStart { message } => {
                                message_id = message.id;
                            }
                            StreamEvent::ContentBlockStart {
                                index,
                                content_block,
                            } => {
                                current_block_index = Some(index);
                                match &content_block {
                                    ContentBlock::Text { .. } => {
                                        current_text.clear();
                                    }
                                    ContentBlock::ToolUse { .. } => {
                                        current_tool_input.clear();
                                    }
                                    _ => {}
                                }
                                content_blocks.push(content_block);
                            }
                            StreamEvent::ContentBlockDelta { index, delta } => {
                                if delta.delta_type == "text_delta" {
                                    current_text.push_str(&delta.text);
                                    // Stream text to output
                                    if let Some(tx) = &output_tx {
                                        let _ = tx.send(delta.text.clone()).await;
                                    }
                                } else if delta.delta_type == "input_json_delta" {
                                    current_tool_input.push_str(&delta.partial_json);
                                }
                            }
                            StreamEvent::ContentBlockStop { index } => {
                                if let Some(block) = content_blocks.get_mut(index) {
                                    match block {
                                        ContentBlock::Text { text } => {
                                            *text = current_text.clone();
                                        }
                                        ContentBlock::ToolUse { input, .. } => {
                                            if let Ok(parsed) =
                                                serde_json::from_str(&current_tool_input)
                                            {
                                                *input = parsed;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            StreamEvent::MessageDelta {
                                delta,
                                usage: delta_usage,
                            } => {
                                stop_reason = delta.stop_reason;
                                if delta_usage.is_some() {
                                    usage = delta_usage;
                                }
                            }
                            StreamEvent::MessageStop => {
                                // Stream complete
                            }
                            StreamEvent::Error { error } => {
                                anyhow::bail!("Stream error: {} - {}", error.error_type, error.message);
                            }
                            StreamEvent::Ping => {}
                        },
                        Err(e) => {
                            tracing::debug!("Failed to parse stream event: {} - {}", e, data);
                        }
                    }
                }
            }
        }

        Ok(ApiResponse {
            id: message_id,
            content: content_blocks,
            stop_reason,
            usage,
        })
    }
}

/// Why the loop stopped
#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    /// Completed successfully (end_turn)
    Completed,
    /// Hit max iterations
    MaxIterations,
    /// Hit token redline - needs fresh context
    Redline,
}

/// Result from running the agentic loop
#[derive(Debug)]
pub struct LoopResult {
    pub iterations: usize,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub final_text: String,
    pub messages: Vec<Message>,
    pub stop_reason: StopReason,
}

impl LoopResult {
    pub fn total_tokens(&self) -> u32 {
        self.total_input_tokens + self.total_output_tokens
    }

    pub fn estimated_cost(&self) -> f64 {
        // Sonnet pricing (as of Jan 2025)
        // Input: $3 / 1M tokens
        // Output: $15 / 1M tokens
        let input_cost = (self.total_input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (self.total_output_tokens as f64 / 1_000_000.0) * 15.0;
        input_cost + output_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_result_cost() {
        let result = LoopResult {
            iterations: 5,
            total_input_tokens: 100_000,
            total_output_tokens: 10_000,
            final_text: String::new(),
            messages: vec![],
            stop_reason: StopReason::Completed,
        };

        // Input: 100k * $3/1M = $0.30
        // Output: 10k * $15/1M = $0.15
        // Total: $0.45
        let cost = result.estimated_cost();
        assert!((cost - 0.45).abs() < 0.01);
    }
}
