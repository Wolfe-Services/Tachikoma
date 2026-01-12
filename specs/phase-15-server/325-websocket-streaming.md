# Spec 325: WebSocket LLM Streaming

## Phase
15 - Server/API Layer

## Spec ID
325

## Status
Planned

## Dependencies
- Spec 323: WebSocket Setup
- Spec 324: WebSocket Events
- Spec 401: Backend Abstraction

## Estimated Context
~11%

---

## Objective

Implement real-time LLM response streaming over WebSocket connections, enabling token-by-token delivery of AI responses with proper buffering, backpressure handling, and cancellation support.

---

## Acceptance Criteria

- [ ] Stream tokens in real-time as they arrive from LLM
- [ ] Handle multiple concurrent streaming sessions
- [ ] Support stream cancellation
- [ ] Implement backpressure handling
- [ ] Buffer partial tokens correctly
- [ ] Track token usage during streaming
- [ ] Handle streaming errors gracefully
- [ ] Support tool call streaming

---

## Implementation Details

### Streaming Service

```rust
// src/server/websocket/streaming.rs
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock, oneshot};
use tokio_stream::StreamExt;
use uuid::Uuid;
use chrono::Utc;

use crate::backend::{Backend, StreamEvent, StreamOptions};
use crate::server::state::AppState;
use crate::server::websocket::protocol::{ServerMessage, ContentDelta, ExecutionResult, TokenUsage};
use crate::server::websocket::manager::ConnectionManager;

/// Manages streaming LLM executions
pub struct StreamingService {
    state: AppState,
    active_streams: Arc<RwLock<HashMap<Uuid, StreamHandle>>>,
}

struct StreamHandle {
    execution_id: Uuid,
    spec_id: Uuid,
    connection_id: Uuid,
    cancel_tx: oneshot::Sender<()>,
    started_at: std::time::Instant,
}

impl StreamingService {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a streaming execution
    pub async fn start_stream(
        &self,
        connection_id: Uuid,
        spec_id: Uuid,
        options: StreamOptions,
    ) -> Result<Uuid, StreamError> {
        let execution_id = Uuid::new_v4();

        // Get the spec and build context
        let storage = self.state.storage();
        let spec = storage.specs().get(spec_id).await?;

        // Get backend
        let backend = if let Some(backend_id) = options.backend_id {
            self.state.backend_manager().get(backend_id)
        } else {
            self.state.backend_manager().get_default()
        }.ok_or(StreamError::NoBackendAvailable)?;

        // Build messages from conversation
        let messages = storage.messages().list_for_spec(spec_id).await?;
        let prompt_messages = build_prompt_messages(&spec, &messages);

        // Create cancellation channel
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Store stream handle
        {
            let mut streams = self.active_streams.write().await;
            streams.insert(execution_id, StreamHandle {
                execution_id,
                spec_id,
                connection_id,
                cancel_tx,
                started_at: std::time::Instant::now(),
            });
        }

        // Spawn streaming task
        let service = self.clone();
        tokio::spawn(async move {
            let result = service
                .run_stream(execution_id, connection_id, spec_id, backend, prompt_messages, options, cancel_rx)
                .await;

            // Remove from active streams
            service.active_streams.write().await.remove(&execution_id);

            if let Err(e) = result {
                tracing::error!(
                    execution_id = %execution_id,
                    error = %e,
                    "Stream failed"
                );
            }
        });

        Ok(execution_id)
    }

    /// Run the streaming execution
    async fn run_stream(
        &self,
        execution_id: Uuid,
        connection_id: Uuid,
        spec_id: Uuid,
        backend: Arc<dyn Backend>,
        messages: Vec<PromptMessage>,
        options: StreamOptions,
        cancel_rx: oneshot::Receiver<()>,
    ) -> Result<(), StreamError> {
        let ws_manager = self.state.ws_manager();
        let storage = self.state.storage();

        let mut accumulated_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut prompt_tokens = 0u32;
        let mut completion_tokens = 0u32;
        let started_at = std::time::Instant::now();

        // Create the stream
        let mut stream = backend.stream(&messages, &options).await?;

        // Buffer for handling partial tokens
        let mut token_buffer = TokenBuffer::new();

        // Process stream events
        tokio::select! {
            _ = cancel_rx => {
                // Cancellation requested
                send_to_connection(&ws_manager, connection_id, ServerMessage::ExecutionCancelled {
                    execution_id,
                    spec_id,
                    timestamp: Utc::now(),
                }).await;
                return Ok(());
            }
            result = async {
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(StreamEvent::Token(token)) => {
                            completion_tokens += 1;
                            accumulated_content.push_str(&token);

                            // Buffer and emit tokens
                            if let Some(buffered) = token_buffer.add(&token) {
                                send_to_connection(&ws_manager, connection_id, ServerMessage::StreamToken {
                                    execution_id,
                                    token: buffered,
                                    finish_reason: None,
                                }).await;
                            }
                        }
                        Ok(StreamEvent::ContentDelta(delta)) => {
                            match &delta {
                                ContentDelta::Text { content } => {
                                    accumulated_content.push_str(content);
                                }
                                ContentDelta::ToolCall { id, name, arguments } => {
                                    // Accumulate tool call
                                    if let Some(tc) = tool_calls.iter_mut().find(|t| t.id == *id) {
                                        tc.arguments.push_str(arguments);
                                    } else {
                                        tool_calls.push(ToolCall {
                                            id: id.clone(),
                                            name: name.clone(),
                                            arguments: arguments.clone(),
                                        });
                                    }
                                }
                                _ => {}
                            }

                            send_to_connection(&ws_manager, connection_id, ServerMessage::StreamDelta {
                                execution_id,
                                delta,
                            }).await;
                        }
                        Ok(StreamEvent::Usage { prompt, completion }) => {
                            prompt_tokens = prompt;
                            completion_tokens = completion;
                        }
                        Ok(StreamEvent::Done { finish_reason }) => {
                            // Flush any remaining buffered tokens
                            if let Some(remaining) = token_buffer.flush() {
                                send_to_connection(&ws_manager, connection_id, ServerMessage::StreamToken {
                                    execution_id,
                                    token: remaining,
                                    finish_reason: Some(finish_reason.clone()),
                                }).await;
                            }
                            break;
                        }
                        Err(e) => {
                            send_to_connection(&ws_manager, connection_id, ServerMessage::ExecutionFailed {
                                execution_id,
                                spec_id,
                                error: e.to_string(),
                                timestamp: Utc::now(),
                            }).await;
                            return Err(StreamError::Backend(e.to_string()));
                        }
                    }
                }
                Ok::<_, StreamError>(())
            } => {
                result?;
            }
        }

        // Save assistant message
        let assistant_message = Message {
            id: Uuid::new_v4(),
            spec_id,
            role: MessageRole::Assistant,
            content: accumulated_content.clone(),
            tokens: Some(completion_tokens as i32),
            model: Some(options.model.unwrap_or_else(|| backend.default_model().to_string())),
            created_at: Utc::now(),
        };
        let saved_message = storage.messages().create(assistant_message).await?;

        // Process tool calls if any
        let file_changes = self.process_tool_calls(spec_id, &saved_message, &tool_calls).await?;

        // Send completion event
        let duration = started_at.elapsed();
        send_to_connection(&ws_manager, connection_id, ServerMessage::ExecutionCompleted {
            execution_id,
            spec_id,
            result: ExecutionResult {
                message_id: saved_message.id,
                content: accumulated_content,
                tokens_used: TokenUsage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                },
                file_changes,
                duration_ms: duration.as_millis() as u64,
            },
            timestamp: Utc::now(),
        }).await;

        // Record execution
        storage.specs().record_execution(spec_id, ExecutionRecord {
            id: execution_id,
            spec_id,
            message_id: saved_message.id,
            prompt_tokens,
            completion_tokens,
            duration_ms: duration.as_millis() as u64,
            model: options.model.unwrap_or_else(|| backend.default_model().to_string()),
            status: ExecutionStatus::Completed,
            created_at: Utc::now(),
        }).await?;

        Ok(())
    }

    /// Process tool calls from the response
    async fn process_tool_calls(
        &self,
        spec_id: Uuid,
        message: &Message,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<FileChangeSummary>, StreamError> {
        let storage = self.state.storage();
        let mut file_changes = Vec::new();

        for tool_call in tool_calls {
            match tool_call.name.as_str() {
                "write_file" | "create_file" => {
                    let args: WriteFileArgs = serde_json::from_str(&tool_call.arguments)?;

                    let change = FileChange {
                        id: Uuid::new_v4(),
                        message_id: message.id,
                        file_path: args.path.clone(),
                        change_type: FileChangeType::Create,
                        original_content: None,
                        new_content: args.content,
                        status: FileChangeStatus::Pending,
                        created_at: Utc::now(),
                    };

                    let saved = storage.file_changes().create(change).await?;
                    file_changes.push(FileChangeSummary {
                        change_id: saved.id,
                        file_path: saved.file_path,
                        change_type: FileChangeType::Create,
                    });

                    // Broadcast file change event
                    self.state.ws_manager().broadcast_to_channel(
                        &format!("spec:{}", spec_id),
                        serde_json::to_string(&ServerMessage::FileChange {
                            change_id: saved.id,
                            spec_id,
                            file_path: args.path,
                            change_type: FileChangeType::Create,
                            preview: Some(truncate_preview(&saved.new_content, 500)),
                        }).unwrap(),
                    ).await;
                }
                "edit_file" | "modify_file" => {
                    let args: EditFileArgs = serde_json::from_str(&tool_call.arguments)?;

                    let change = FileChange {
                        id: Uuid::new_v4(),
                        message_id: message.id,
                        file_path: args.path.clone(),
                        change_type: FileChangeType::Modify,
                        original_content: args.original,
                        new_content: args.new_content,
                        status: FileChangeStatus::Pending,
                        created_at: Utc::now(),
                    };

                    let saved = storage.file_changes().create(change).await?;
                    file_changes.push(FileChangeSummary {
                        change_id: saved.id,
                        file_path: saved.file_path,
                        change_type: FileChangeType::Modify,
                    });
                }
                "delete_file" => {
                    let args: DeleteFileArgs = serde_json::from_str(&tool_call.arguments)?;

                    let change = FileChange {
                        id: Uuid::new_v4(),
                        message_id: message.id,
                        file_path: args.path.clone(),
                        change_type: FileChangeType::Delete,
                        original_content: None,
                        new_content: String::new(),
                        status: FileChangeStatus::Pending,
                        created_at: Utc::now(),
                    };

                    let saved = storage.file_changes().create(change).await?;
                    file_changes.push(FileChangeSummary {
                        change_id: saved.id,
                        file_path: saved.file_path,
                        change_type: FileChangeType::Delete,
                    });
                }
                _ => {
                    tracing::debug!(
                        tool = %tool_call.name,
                        "Unknown tool call, ignoring"
                    );
                }
            }
        }

        Ok(file_changes)
    }

    /// Cancel a streaming execution
    pub async fn cancel(&self, execution_id: Uuid) -> Result<(), StreamError> {
        let mut streams = self.active_streams.write().await;

        if let Some(handle) = streams.remove(&execution_id) {
            let _ = handle.cancel_tx.send(());
            Ok(())
        } else {
            Err(StreamError::ExecutionNotFound)
        }
    }

    /// Check if an execution is active
    pub async fn is_active(&self, execution_id: Uuid) -> bool {
        self.active_streams.read().await.contains_key(&execution_id)
    }

    /// Get active stream count
    pub async fn active_count(&self) -> usize {
        self.active_streams.read().await.len()
    }
}

impl Clone for StreamingService {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            active_streams: self.active_streams.clone(),
        }
    }
}

/// Token buffer for handling partial tokens
struct TokenBuffer {
    buffer: String,
    min_emit_size: usize,
}

impl TokenBuffer {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            min_emit_size: 1,
        }
    }

    fn add(&mut self, token: &str) -> Option<String> {
        self.buffer.push_str(token);

        if self.buffer.len() >= self.min_emit_size {
            Some(std::mem::take(&mut self.buffer))
        } else {
            None
        }
    }

    fn flush(&mut self) -> Option<String> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buffer))
        }
    }
}

/// Tool call arguments
#[derive(Debug, Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct EditFileArgs {
    path: String,
    original: Option<String>,
    new_content: String,
}

#[derive(Debug, Deserialize)]
struct DeleteFileArgs {
    path: String,
}

#[derive(Debug, Clone)]
struct ToolCall {
    id: String,
    name: String,
    arguments: String,
}

/// Streaming errors
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("No backend available")]
    NoBackendAvailable,

    #[error("Execution not found")]
    ExecutionNotFound,

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// Helper functions

async fn send_to_connection(
    ws_manager: &ConnectionManager,
    connection_id: Uuid,
    message: ServerMessage,
) {
    let json = serde_json::to_string(&message).unwrap();
    if let Err(e) = ws_manager.send_to(connection_id, json).await {
        tracing::warn!(
            connection_id = %connection_id,
            error = %e,
            "Failed to send message"
        );
    }
}

fn build_prompt_messages(spec: &Spec, messages: &[Message]) -> Vec<PromptMessage> {
    let mut prompt_messages = Vec::new();

    // System message with spec context
    let system_content = format!(
        "You are helping implement the following specification:\n\n\
        # {}\n\n\
        ## Objective\n{}\n\n\
        ## Acceptance Criteria\n{}\n\n\
        ## Implementation Details\n{}\n\n\
        Follow the spec carefully and provide implementation code.",
        spec.title,
        spec.description.as_deref().unwrap_or(""),
        spec.acceptance_criteria.as_deref().unwrap_or(""),
        spec.implementation_details.as_deref().unwrap_or(""),
    );

    prompt_messages.push(PromptMessage {
        role: "system".to_string(),
        content: system_content,
    });

    // Add conversation history
    for message in messages {
        prompt_messages.push(PromptMessage {
            role: match message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            }.to_string(),
            content: message.content.clone(),
        });
    }

    prompt_messages
}

fn truncate_preview(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len])
    }
}
```

### Stream Options

```rust
// src/server/websocket/streaming.rs (additional types)

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StreamOptions {
    pub backend_id: Option<Uuid>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl StreamOptions {
    pub fn with_defaults(self, config: &ExecutionSettings) -> Self {
        Self {
            backend_id: self.backend_id.or(config.default_backend_id),
            model: self.model,
            temperature: self.temperature.or(Some(config.default_temperature)),
            max_tokens: self.max_tokens.or(Some(config.default_max_tokens)),
            ..self
        }
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_buffer() {
        let mut buffer = TokenBuffer::new();

        // Single character accumulation
        assert!(buffer.add("H").is_some());
        assert!(buffer.add("e").is_some());

        // Flush remaining
        buffer.add("llo");
        let flushed = buffer.flush();
        assert!(flushed.is_some());
    }

    #[tokio::test]
    async fn test_stream_cancellation() {
        let state = create_test_state().await;
        let service = StreamingService::new(state);

        // Start a mock stream
        let execution_id = Uuid::new_v4();

        // Cancel should succeed for non-existent stream (already completed)
        let result = service.cancel(execution_id).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_build_prompt_messages() {
        let spec = Spec {
            title: "Test Spec".to_string(),
            description: Some("A test specification".to_string()),
            ..Default::default()
        };

        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "Help me implement this".to_string(),
                ..Default::default()
            },
        ];

        let prompts = build_prompt_messages(&spec, &messages);

        assert_eq!(prompts.len(), 2); // System + user
        assert_eq!(prompts[0].role, "system");
        assert_eq!(prompts[1].role, "user");
    }
}
```

---

## Related Specs

- **Spec 323**: WebSocket Setup
- **Spec 324**: WebSocket Events
- **Spec 326**: SSE Streaming (alternative)
- **Spec 401**: Backend Abstraction
