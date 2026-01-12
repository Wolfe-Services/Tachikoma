//! Common status types.

use serde::{Deserialize, Serialize};

/// Mission execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionState {
    /// Not started.
    Idle,
    /// Currently executing.
    Running,
    /// Paused by user.
    Paused,
    /// Completed successfully.
    Complete,
    /// Failed with error.
    Error,
    /// Context redlined, needs reboot.
    Redlined,
}

impl MissionState {
    /// Is the mission in a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Error)
    }

    /// Is the mission active (running or paused)?
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

/// Spec status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecStatus {
    /// Planned but not started.
    Planned,
    /// Currently being worked on.
    InProgress,
    /// Successfully completed.
    Complete,
    /// Blocked and cannot proceed.
    Blocked,
}

/// Forge session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeStatus {
    /// Initial drafting phase.
    Drafting,
    /// Critiquing the draft.
    Critiquing,
    /// Synthesizing feedback.
    Synthesizing,
    /// Reached convergence.
    Converged,
    /// Session was aborted.
    Aborted,
}

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace level logging.
    Trace,
    /// Debug level logging.
    Debug,
    /// Info level logging.
    Info,
    /// Warning level logging.
    Warn,
    /// Error level logging.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_state_terminal() {
        assert!(MissionState::Complete.is_terminal());
        assert!(MissionState::Error.is_terminal());
        assert!(!MissionState::Running.is_terminal());
        assert!(!MissionState::Idle.is_terminal());
    }

    #[test]
    fn test_mission_state_active() {
        assert!(MissionState::Running.is_active());
        assert!(MissionState::Paused.is_active());
        assert!(!MissionState::Idle.is_active());
        assert!(!MissionState::Complete.is_active());
    }

    #[test]
    fn test_status_serialization() {
        let state = MissionState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");
        
        let status = SpecStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let level = LogLevel::Info;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"info\"");
    }

    #[test]
    fn test_status_deserialization() {
        let state: MissionState = serde_json::from_str("\"running\"").unwrap();
        assert_eq!(state, MissionState::Running);

        let status: SpecStatus = serde_json::from_str("\"in_progress\"").unwrap();
        assert_eq!(status, SpecStatus::InProgress);

        let level: LogLevel = serde_json::from_str("\"info\"").unwrap();
        assert_eq!(level, LogLevel::Info);
    }
}