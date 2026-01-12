# Spec 324: WebSocket Event Types

## Phase
15 - Server/API Layer

## Spec ID
324

## Status
Planned

## Dependencies
- Spec 323: WebSocket Setup

## Estimated Context
~9%

---

## Objective

Define the WebSocket message protocol and event types for Tachikoma, enabling structured communication between clients and server for real-time updates, streaming responses, and collaborative features.

---

## Acceptance Criteria

- [ ] Complete client-to-server message types defined
- [ ] Complete server-to-client message types defined
- [ ] Message serialization/deserialization implemented
- [ ] Event routing and handling system
- [ ] Subscription/channel management
- [ ] Error event handling
- [ ] Type-safe event builders

---

## Implementation Details

### Protocol Definition

```rust
// src/server/websocket/protocol.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Client-to-server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Ping for keepalive
    Ping,

    /// Subscribe to a channel
    Subscribe {
        channel: String,
        #[serde(default)]
        params: Option<serde_json::Value>,
    },

    /// Unsubscribe from a channel
    Unsubscribe {
        channel: String,
    },

    /// Execute a spec
    ExecuteSpec {
        spec_id: Uuid,
        #[serde(default)]
        options: ExecutionOptions,
    },

    /// Cancel an ongoing execution
    CancelExecution {
        execution_id: Uuid,
    },

    /// Send a message to a spec conversation
    SendMessage {
        spec_id: Uuid,
        content: String,
        #[serde(default)]
        execute_after: bool,
    },

    /// Apply a file change
    ApplyChange {
        change_id: Uuid,
    },

    /// Reject a file change
    RejectChange {
        change_id: Uuid,
    },

    /// Request current state
    GetState {
        resource: StateResource,
        id: Option<Uuid>,
    },

    /// Acknowledge receipt of a message
    Ack {
        message_id: Uuid,
    },

    /// Custom/extension message
    Custom {
        action: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExecutionOptions {
    pub backend_id: Option<Uuid>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateResource {
    Mission,
    Phase,
    Spec,
    Execution,
}

impl ClientMessage {
    pub fn message_type(&self) -> &'static str {
        match self {
            ClientMessage::Ping => "ping",
            ClientMessage::Subscribe { .. } => "subscribe",
            ClientMessage::Unsubscribe { .. } => "unsubscribe",
            ClientMessage::ExecuteSpec { .. } => "execute_spec",
            ClientMessage::CancelExecution { .. } => "cancel_execution",
            ClientMessage::SendMessage { .. } => "send_message",
            ClientMessage::ApplyChange { .. } => "apply_change",
            ClientMessage::RejectChange { .. } => "reject_change",
            ClientMessage::GetState { .. } => "get_state",
            ClientMessage::Ack { .. } => "ack",
            ClientMessage::Custom { .. } => "custom",
        }
    }
}

/// Server-to-client messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Response to ping
    Pong,

    /// Welcome message on connection
    Welcome {
        connection_id: String,
        session_id: Option<String>,
        server_version: String,
    },

    /// Subscription confirmed
    Subscribed {
        channel: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_state: Option<serde_json::Value>,
    },

    /// Unsubscription confirmed
    Unsubscribed {
        channel: String,
    },

    /// Error message
    Error {
        code: String,
        message: String,
    },

    /// Execution started
    ExecutionStarted {
        execution_id: Uuid,
        spec_id: Uuid,
        model: String,
        timestamp: DateTime<Utc>,
    },

    /// Streaming token
    StreamToken {
        execution_id: Uuid,
        token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        finish_reason: Option<String>,
    },

    /// Streaming content delta
    StreamDelta {
        execution_id: Uuid,
        delta: ContentDelta,
    },

    /// Execution completed
    ExecutionCompleted {
        execution_id: Uuid,
        spec_id: Uuid,
        result: ExecutionResult,
        timestamp: DateTime<Utc>,
    },

    /// Execution failed
    ExecutionFailed {
        execution_id: Uuid,
        spec_id: Uuid,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// Execution cancelled
    ExecutionCancelled {
        execution_id: Uuid,
        spec_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// File change detected
    FileChange {
        change_id: Uuid,
        spec_id: Uuid,
        file_path: String,
        change_type: FileChangeType,
        preview: Option<String>,
    },

    /// File change applied
    FileChangeApplied {
        change_id: Uuid,
        file_path: String,
    },

    /// Spec status updated
    SpecStatusChanged {
        spec_id: Uuid,
        old_status: String,
        new_status: String,
        timestamp: DateTime<Utc>,
    },

    /// Mission updated
    MissionUpdated {
        mission_id: Uuid,
        update_type: MissionUpdateType,
        timestamp: DateTime<Utc>,
    },

    /// Message added to conversation
    MessageAdded {
        spec_id: Uuid,
        message: ConversationMessage,
    },

    /// State response
    State {
        resource: StateResource,
        data: serde_json::Value,
    },

    /// Progress update
    Progress {
        execution_id: Uuid,
        progress: f32,
        message: Option<String>,
    },

    /// Server notification
    Notification {
        id: Uuid,
        level: NotificationLevel,
        title: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<NotificationAction>,
    },

    /// Heartbeat
    Heartbeat {
        timestamp: DateTime<Utc>,
        server_time: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    Text { content: String },
    ToolCall { id: String, name: String, arguments: String },
    ToolResult { id: String, content: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResult {
    pub message_id: Uuid,
    pub content: String,
    pub tokens_used: TokenUsage,
    pub file_changes: Vec<FileChangeSummary>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileChangeSummary {
    pub change_id: Uuid,
    pub file_path: String,
    pub change_type: FileChangeType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileChangeType {
    Create,
    Modify,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionUpdateType {
    StatusChanged,
    PhaseAdded,
    PhaseRemoved,
    SpecAdded,
    SpecRemoved,
    MetadataUpdated,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub tokens: Option<u32>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationAction {
    pub label: String,
    pub action: String,
    pub payload: Option<serde_json::Value>,
}
```

### Event Handler

```rust
// src/server/websocket/handler.rs
use std::sync::Arc;
use tokio::sync::mpsc;

use super::protocol::{ClientMessage, ServerMessage};
use super::connection::WebSocketConnection;
use crate::server::state::AppState;

/// Handles WebSocket events
pub struct EventHandler {
    state: AppState,
    event_tx: mpsc::Sender<WebSocketEvent>,
}

pub struct WebSocketEvent {
    pub connection_id: Uuid,
    pub message: ClientMessage,
}

impl EventHandler {
    pub fn new(state: AppState) -> (Self, mpsc::Receiver<WebSocketEvent>) {
        let (event_tx, event_rx) = mpsc::channel(1000);

        (Self { state, event_tx }, event_rx)
    }

    /// Process an incoming message
    pub async fn handle(
        &self,
        connection: &Arc<WebSocketConnection>,
        message: ClientMessage,
    ) -> Option<ServerMessage> {
        match message {
            ClientMessage::Ping => Some(ServerMessage::Pong),

            ClientMessage::Subscribe { channel, params } => {
                self.handle_subscribe(connection, &channel, params).await
            }

            ClientMessage::Unsubscribe { channel } => {
                self.handle_unsubscribe(connection, &channel).await
            }

            ClientMessage::ExecuteSpec { spec_id, options } => {
                self.handle_execute_spec(connection, spec_id, options).await
            }

            ClientMessage::CancelExecution { execution_id } => {
                self.handle_cancel_execution(connection, execution_id).await
            }

            ClientMessage::SendMessage { spec_id, content, execute_after } => {
                self.handle_send_message(connection, spec_id, content, execute_after).await
            }

            ClientMessage::ApplyChange { change_id } => {
                self.handle_apply_change(connection, change_id).await
            }

            ClientMessage::RejectChange { change_id } => {
                self.handle_reject_change(connection, change_id).await
            }

            ClientMessage::GetState { resource, id } => {
                self.handle_get_state(connection, resource, id).await
            }

            ClientMessage::Ack { message_id } => {
                // Acknowledgment received, no response needed
                None
            }

            ClientMessage::Custom { action, payload } => {
                self.handle_custom(connection, &action, payload).await
            }
        }
    }

    async fn handle_subscribe(
        &self,
        connection: &Arc<WebSocketConnection>,
        channel: &str,
        params: Option<serde_json::Value>,
    ) -> Option<ServerMessage> {
        // Validate channel format
        let parts: Vec<&str> = channel.split(':').collect();
        if parts.is_empty() {
            return Some(ServerMessage::Error {
                code: "INVALID_CHANNEL".to_string(),
                message: "Invalid channel format".to_string(),
            });
        }

        // Check permissions based on channel type
        let channel_type = parts[0];
        match channel_type {
            "mission" | "spec" | "execution" => {
                // Validate resource exists
                if parts.len() < 2 {
                    return Some(ServerMessage::Error {
                        code: "INVALID_CHANNEL".to_string(),
                        message: "Resource ID required".to_string(),
                    });
                }
            }
            "system" | "notifications" => {
                // Global channels, no validation needed
            }
            _ => {
                return Some(ServerMessage::Error {
                    code: "UNKNOWN_CHANNEL".to_string(),
                    message: format!("Unknown channel type: {}", channel_type),
                });
            }
        }

        // Subscribe
        connection.subscribe(channel.to_string()).await;

        // Get initial state if applicable
        let initial_state = self.get_channel_initial_state(channel).await;

        Some(ServerMessage::Subscribed {
            channel: channel.to_string(),
            initial_state,
        })
    }

    async fn handle_unsubscribe(
        &self,
        connection: &Arc<WebSocketConnection>,
        channel: &str,
    ) -> Option<ServerMessage> {
        connection.unsubscribe(channel).await;

        Some(ServerMessage::Unsubscribed {
            channel: channel.to_string(),
        })
    }

    async fn handle_execute_spec(
        &self,
        connection: &Arc<WebSocketConnection>,
        spec_id: Uuid,
        options: ExecutionOptions,
    ) -> Option<ServerMessage> {
        let execution_id = Uuid::new_v4();

        // Spawn execution task
        let state = self.state.clone();
        let conn_id = connection.id();

        tokio::spawn(async move {
            if let Err(e) = execute_spec_streaming(state, conn_id, execution_id, spec_id, options).await {
                tracing::error!(
                    execution_id = %execution_id,
                    error = %e,
                    "Execution failed"
                );
            }
        });

        // Return started message immediately
        Some(ServerMessage::ExecutionStarted {
            execution_id,
            spec_id,
            model: options.model.unwrap_or_else(|| "default".to_string()),
            timestamp: Utc::now(),
        })
    }

    async fn handle_cancel_execution(
        &self,
        connection: &Arc<WebSocketConnection>,
        execution_id: Uuid,
    ) -> Option<ServerMessage> {
        // Signal cancellation
        if let Err(e) = self.state.execution_manager().cancel(execution_id).await {
            return Some(ServerMessage::Error {
                code: "CANCEL_FAILED".to_string(),
                message: e.to_string(),
            });
        }

        None // Cancellation event will be sent by the execution task
    }

    async fn handle_send_message(
        &self,
        connection: &Arc<WebSocketConnection>,
        spec_id: Uuid,
        content: String,
        execute_after: bool,
    ) -> Option<ServerMessage> {
        let storage = self.state.storage();

        // Create message
        let message = Message {
            id: Uuid::new_v4(),
            spec_id,
            role: MessageRole::User,
            content: content.clone(),
            tokens: None,
            model: None,
            created_at: Utc::now(),
        };

        match storage.messages().create(message.clone()).await {
            Ok(saved) => {
                // Broadcast to spec channel
                self.state.ws_manager().broadcast_to_channel(
                    &format!("spec:{}", spec_id),
                    serde_json::to_string(&ServerMessage::MessageAdded {
                        spec_id,
                        message: ConversationMessage {
                            id: saved.id,
                            role: "user".to_string(),
                            content: saved.content,
                            tokens: saved.tokens.map(|t| t as u32),
                            timestamp: saved.created_at,
                        },
                    }).unwrap(),
                ).await;

                // Trigger execution if requested
                if execute_after {
                    return self.handle_execute_spec(
                        connection,
                        spec_id,
                        ExecutionOptions::default(),
                    ).await;
                }

                None
            }
            Err(e) => Some(ServerMessage::Error {
                code: "MESSAGE_FAILED".to_string(),
                message: e.to_string(),
            }),
        }
    }

    async fn handle_apply_change(
        &self,
        connection: &Arc<WebSocketConnection>,
        change_id: Uuid,
    ) -> Option<ServerMessage> {
        let storage = self.state.storage();

        match storage.file_changes().apply(change_id).await {
            Ok(change) => {
                Some(ServerMessage::FileChangeApplied {
                    change_id,
                    file_path: change.file_path,
                })
            }
            Err(e) => Some(ServerMessage::Error {
                code: "APPLY_FAILED".to_string(),
                message: e.to_string(),
            }),
        }
    }

    async fn handle_reject_change(
        &self,
        connection: &Arc<WebSocketConnection>,
        change_id: Uuid,
    ) -> Option<ServerMessage> {
        let storage = self.state.storage();

        match storage.file_changes().reject(change_id).await {
            Ok(_) => None, // No response needed
            Err(e) => Some(ServerMessage::Error {
                code: "REJECT_FAILED".to_string(),
                message: e.to_string(),
            }),
        }
    }

    async fn handle_get_state(
        &self,
        connection: &Arc<WebSocketConnection>,
        resource: StateResource,
        id: Option<Uuid>,
    ) -> Option<ServerMessage> {
        let storage = self.state.storage();

        let data = match (resource, id) {
            (StateResource::Mission, Some(id)) => {
                storage.missions().get(id).await
                    .ok()
                    .map(|m| serde_json::to_value(m).unwrap())
            }
            (StateResource::Spec, Some(id)) => {
                storage.specs().get(id).await
                    .ok()
                    .map(|s| serde_json::to_value(s).unwrap())
            }
            _ => None,
        };

        match data {
            Some(data) => Some(ServerMessage::State { resource, data }),
            None => Some(ServerMessage::Error {
                code: "NOT_FOUND".to_string(),
                message: "Resource not found".to_string(),
            }),
        }
    }

    async fn handle_custom(
        &self,
        connection: &Arc<WebSocketConnection>,
        action: &str,
        payload: serde_json::Value,
    ) -> Option<ServerMessage> {
        // Extension point for custom actions
        tracing::debug!(
            connection_id = %connection.id(),
            action = %action,
            "Custom action received"
        );

        None
    }

    async fn get_channel_initial_state(&self, channel: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = channel.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let id = Uuid::parse_str(parts[1]).ok()?;
        let storage = self.state.storage();

        match parts[0] {
            "spec" => {
                let spec = storage.specs().get(id).await.ok()?;
                let messages = storage.messages().list_for_spec(id).await.ok()?;
                Some(serde_json::json!({
                    "spec": spec,
                    "messages": messages,
                }))
            }
            "mission" => {
                let mission = storage.missions().get(id).await.ok()?;
                Some(serde_json::to_value(mission).ok()?)
            }
            _ => None,
        }
    }
}
```

### Channel Types

```rust
// src/server/websocket/channels.rs

/// Channel naming conventions and helpers
pub struct Channel;

impl Channel {
    /// Create a spec channel name
    pub fn spec(spec_id: Uuid) -> String {
        format!("spec:{}", spec_id)
    }

    /// Create a mission channel name
    pub fn mission(mission_id: Uuid) -> String {
        format!("mission:{}", mission_id)
    }

    /// Create an execution channel name
    pub fn execution(execution_id: Uuid) -> String {
        format!("execution:{}", execution_id)
    }

    /// System-wide notifications channel
    pub fn notifications() -> &'static str {
        "notifications"
    }

    /// System events channel
    pub fn system() -> &'static str {
        "system"
    }

    /// Parse channel to extract type and ID
    pub fn parse(channel: &str) -> (String, Option<Uuid>) {
        let parts: Vec<&str> = channel.split(':').collect();
        let channel_type = parts[0].to_string();
        let id = parts.get(1).and_then(|s| Uuid::parse_str(s).ok());
        (channel_type, id)
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
    fn test_client_message_parsing() {
        let json = r#"{"type": "subscribe", "channel": "spec:123e4567-e89b-12d3-a456-426614174000"}"#;
        let message: ClientMessage = serde_json::from_str(json).unwrap();

        matches!(message, ClientMessage::Subscribe { .. });
    }

    #[test]
    fn test_server_message_serialization() {
        let message = ServerMessage::ExecutionStarted {
            execution_id: Uuid::new_v4(),
            spec_id: Uuid::new_v4(),
            model: "gpt-4".to_string(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("execution_started"));
    }

    #[test]
    fn test_channel_parsing() {
        let (channel_type, id) = Channel::parse("spec:123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(channel_type, "spec");
        assert!(id.is_some());

        let (channel_type, id) = Channel::parse("notifications");
        assert_eq!(channel_type, "notifications");
        assert!(id.is_none());
    }
}
```

---

## Related Specs

- **Spec 323**: WebSocket Setup
- **Spec 325**: WebSocket Streaming
- **Spec 318**: Specs API
