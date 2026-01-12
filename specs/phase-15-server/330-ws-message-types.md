# 330 - WebSocket Message Types

**Phase:** 15 - Server
**Spec ID:** 330
**Status:** Planned
**Dependencies:** 329-websocket-setup
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Define typed WebSocket message structures for client-server communication including events, commands, and responses.

---

## Acceptance Criteria

- [ ] Typed message envelope
- [ ] Event message types
- [ ] Command message types
- [ ] Response/ack messages
- [ ] Error message format
- [ ] Subscription management messages
- [ ] Serialization/deserialization

---

## Implementation Details

### 1. Message Envelope (crates/tachikoma-server/src/websocket/messages/envelope.rs)

```rust
//! WebSocket message envelope.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope<T> {
    /// Message ID for tracking.
    pub id: Uuid,
    /// Message type identifier.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Message payload.
    pub payload: T,
}

impl<T: Serialize> WsEnvelope<T> {
    pub fn new(msg_type: impl Into<String>, payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: msg_type.into(),
            timestamp: Utc::now(),
            payload,
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Incoming message (from client).
#[derive(Debug, Clone, Deserialize)]
pub struct IncomingMessage {
    /// Optional message ID for request-response correlation.
    #[serde(default)]
    pub id: Option<Uuid>,
    /// Message type.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Raw payload.
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl IncomingMessage {
    /// Parse payload into specific type.
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

/// Outgoing message (to client).
#[derive(Debug, Clone, Serialize)]
pub struct OutgoingMessage {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>,
    pub payload: serde_json::Value,
}

impl OutgoingMessage {
    pub fn new(msg_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: msg_type.into(),
            timestamp: Utc::now(),
            reply_to: None,
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn reply(reply_to: Uuid, msg_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: msg_type.into(),
            timestamp: Utc::now(),
            reply_to: Some(reply_to),
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
```

### 2. Event Messages (crates/tachikoma-server/src/websocket/messages/events.rs)

```rust
//! Event message types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mission-related events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum MissionEvent {
    /// Mission created.
    Created {
        mission_id: Uuid,
        name: String,
    },
    /// Mission started.
    Started {
        mission_id: Uuid,
    },
    /// Mission progress update.
    Progress {
        mission_id: Uuid,
        current_spec: String,
        completed_specs: u32,
        total_specs: u32,
        percentage: f32,
    },
    /// Mission completed.
    Completed {
        mission_id: Uuid,
        success: bool,
        duration_seconds: u64,
    },
    /// Mission failed.
    Failed {
        mission_id: Uuid,
        error: String,
    },
    /// Mission status changed.
    StatusChanged {
        mission_id: Uuid,
        old_status: String,
        new_status: String,
    },
}

/// Forge session events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ForgeEvent {
    /// Session started.
    Started {
        session_id: Uuid,
        mission_id: Uuid,
        spec_id: String,
    },
    /// Token usage update.
    TokenUsage {
        session_id: Uuid,
        input_tokens: u64,
        output_tokens: u64,
        total_cost: f64,
    },
    /// Tool execution.
    ToolExecution {
        session_id: Uuid,
        tool_name: String,
        status: String,
    },
    /// Session completed.
    Completed {
        session_id: Uuid,
        success: bool,
    },
    /// Log output.
    Log {
        session_id: Uuid,
        level: String,
        message: String,
    },
}

/// System events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SystemEvent {
    /// Server status update.
    ServerStatus {
        status: String,
        active_missions: u32,
        active_sessions: u32,
    },
    /// Maintenance notification.
    Maintenance {
        message: String,
        starts_at: String,
        duration_minutes: u32,
    },
    /// Version update available.
    VersionUpdate {
        current_version: String,
        new_version: String,
    },
}

/// All event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum Event {
    Mission(MissionEvent),
    Forge(ForgeEvent),
    System(SystemEvent),
}
```

### 3. Command Messages (crates/tachikoma-server/src/websocket/messages/commands.rs)

```rust
//! Command message types (client to server).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SubscriptionCommand {
    /// Subscribe to a topic.
    Subscribe {
        topic: String,
        #[serde(default)]
        filters: Option<SubscriptionFilters>,
    },
    /// Unsubscribe from a topic.
    Unsubscribe { topic: String },
    /// List current subscriptions.
    ListSubscriptions,
}

/// Subscription filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionFilters {
    /// Filter by mission ID.
    pub mission_id: Option<Uuid>,
    /// Filter by session ID.
    pub session_id: Option<Uuid>,
    /// Filter by event types.
    pub event_types: Option<Vec<String>>,
}

/// Mission commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum MissionCommand {
    /// Request mission status.
    GetStatus { mission_id: Uuid },
    /// Start a mission.
    Start { mission_id: Uuid },
    /// Pause a mission.
    Pause { mission_id: Uuid },
    /// Resume a mission.
    Resume { mission_id: Uuid },
    /// Cancel a mission.
    Cancel { mission_id: Uuid },
}

/// Session commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SessionCommand {
    /// Authenticate with token.
    Authenticate { token: String },
    /// Request session info.
    GetInfo,
    /// Ping for keepalive.
    Ping,
}

/// All command types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum Command {
    Subscription(SubscriptionCommand),
    Mission(MissionCommand),
    Session(SessionCommand),
}

impl Command {
    /// Parse from incoming message.
    pub fn from_message(msg_type: &str, payload: serde_json::Value) -> Result<Self, String> {
        match msg_type {
            "subscribe" | "unsubscribe" | "list_subscriptions" => {
                serde_json::from_value(payload)
                    .map(Command::Subscription)
                    .map_err(|e| e.to_string())
            }
            "mission_status" | "mission_start" | "mission_pause" | "mission_resume" | "mission_cancel" => {
                serde_json::from_value(payload)
                    .map(Command::Mission)
                    .map_err(|e| e.to_string())
            }
            "authenticate" | "session_info" | "ping" => {
                serde_json::from_value(payload)
                    .map(Command::Session)
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("Unknown message type: {}", msg_type)),
        }
    }
}
```

### 4. Response Messages (crates/tachikoma-server/src/websocket/messages/responses.rs)

```rust
//! Response message types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Acknowledgment response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl AckResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
        }
    }
}

/// Error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = Some(serde_json::to_value(details).unwrap_or_default());
        self
    }

    /// Authentication required.
    pub fn auth_required() -> Self {
        Self::new("auth_required", "Authentication required")
    }

    /// Invalid message format.
    pub fn invalid_message(details: &str) -> Self {
        Self::new("invalid_message", details)
    }

    /// Not found.
    pub fn not_found(resource: &str) -> Self {
        Self::new("not_found", format!("{} not found", resource))
    }

    /// Permission denied.
    pub fn permission_denied() -> Self {
        Self::new("permission_denied", "Permission denied")
    }
}

/// Authentication response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Subscription list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<String>,
}

/// Session info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoResponse {
    pub session_id: Uuid,
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    pub subscriptions: Vec<String>,
    pub connected_at: String,
}

/// Pong response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongResponse {
    pub timestamp: String,
}

impl PongResponse {
    pub fn now() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
```

### 5. Message Module (crates/tachikoma-server/src/websocket/messages/mod.rs)

```rust
//! WebSocket message types.

pub mod commands;
pub mod envelope;
pub mod events;
pub mod responses;

pub use commands::*;
pub use envelope::*;
pub use events::*;
pub use responses::*;

/// Message type constants.
pub mod types {
    // Events
    pub const EVENT_MISSION: &str = "event.mission";
    pub const EVENT_FORGE: &str = "event.forge";
    pub const EVENT_SYSTEM: &str = "event.system";

    // Commands
    pub const CMD_SUBSCRIBE: &str = "subscribe";
    pub const CMD_UNSUBSCRIBE: &str = "unsubscribe";
    pub const CMD_AUTHENTICATE: &str = "authenticate";
    pub const CMD_PING: &str = "ping";

    // Responses
    pub const RESP_ACK: &str = "ack";
    pub const RESP_ERROR: &str = "error";
    pub const RESP_PONG: &str = "pong";
    pub const RESP_AUTH: &str = "auth_result";
}

/// Topic names for subscriptions.
pub mod topics {
    /// All mission events.
    pub const MISSIONS: &str = "missions";
    /// Specific mission events (format: missions/{id}).
    pub fn mission(id: uuid::Uuid) -> String {
        format!("missions/{}", id)
    }
    /// All forge session events.
    pub const FORGE_SESSIONS: &str = "forge_sessions";
    /// Specific forge session (format: forge_sessions/{id}).
    pub fn forge_session(id: uuid::Uuid) -> String {
        format!("forge_sessions/{}", id)
    }
    /// System events.
    pub const SYSTEM: &str = "system";
}
```

---

## Testing Requirements

1. Message serialization correct
2. Message deserialization works
3. Envelope structure valid
4. Event types serialize properly
5. Command parsing works
6. Error responses formatted correctly
7. Topic helpers work

---

## Related Specs

- Depends on: [329-websocket-setup.md](329-websocket-setup.md)
- Next: [331-ws-connection.md](331-ws-connection.md)
- Used by: WebSocket handlers
