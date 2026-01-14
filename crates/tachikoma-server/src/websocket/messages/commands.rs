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