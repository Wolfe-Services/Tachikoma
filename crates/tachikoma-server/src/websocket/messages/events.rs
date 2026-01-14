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