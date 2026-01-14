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