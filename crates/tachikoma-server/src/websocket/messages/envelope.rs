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