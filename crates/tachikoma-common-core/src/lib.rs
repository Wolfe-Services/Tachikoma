//! Tachikoma common core types.
//!
//! This crate provides foundational types used across all Tachikoma components.

#![warn(missing_docs)]

pub mod error;
pub mod id;
pub mod status;
pub mod timestamp;
pub mod types;

pub use error::Error;
pub use id::*;
pub use status::*;
pub use timestamp::*;
pub use types::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_all_types_work_together() {
        // Test ID types
        let mission_id = MissionId::new();
        let spec_id = SpecId::new();
        
        // Test timestamp
        let timestamp = Timestamp::now();
        
        // Test status enums
        let mission_state = MissionState::Running;
        let spec_status = SpecStatus::InProgress;
        
        // Test serialization
        let mission_json = serde_json::to_string(&mission_id).unwrap();
        let timestamp_json = serde_json::to_string(&timestamp).unwrap();
        let state_json = serde_json::to_string(&mission_state).unwrap();
        
        // Test deserialization
        let _: MissionId = serde_json::from_str(&mission_json).unwrap();
        let _: Timestamp = serde_json::from_str(&timestamp_json).unwrap();
        let _: MissionState = serde_json::from_str(&state_json).unwrap();
        
        // Test display
        assert!(!mission_id.to_string().is_empty());
        assert!(!timestamp.to_string().is_empty());
        assert!(!format!("{:?}", spec_status).is_empty());
    }
}
