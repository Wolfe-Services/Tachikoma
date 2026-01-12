//! Strongly-typed identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A strongly-typed ID wrapper.
macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
        #[doc = concat!("A unique identifier with prefix '", $prefix, "_'.")]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Create a new random ID.
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Parse from string (with or without prefix).
            pub fn parse(s: &str) -> Result<Self, IdParseError> {
                let s = s.strip_prefix(concat!($prefix, "_")).unwrap_or(s);
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| IdParseError::InvalidFormat)
            }

            /// Get the inner UUID.
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}_{}", $prefix, self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }

        impl std::str::FromStr for $name {
            type Err = IdParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }
    };
}

/// Error parsing an ID.
#[derive(Debug, Clone, thiserror::Error)]
pub enum IdParseError {
    /// The ID format is invalid.
    #[error("invalid ID format")]
    InvalidFormat,
}

// Define all ID types
define_id!(MissionId, "msn");
define_id!(SpecId, "spc");
define_id!(ForgeSessionId, "frg");
define_id!(UserId, "usr");
define_id!(ConfigId, "cfg");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_id_roundtrip() {
        let id = MissionId::new();
        let s = id.to_string();
        let parsed = MissionId::parse(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_prefix() {
        let id = MissionId::new();
        assert!(id.to_string().starts_with("msn_"));
    }

    #[test]
    fn test_id_serialization() {
        let id = MissionId::new();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: MissionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_id_parse_without_prefix() {
        let id = MissionId::new();
        let uuid_str = id.as_uuid().to_string();
        let parsed = MissionId::parse(&uuid_str).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_all_id_types() {
        let mission_id = MissionId::new();
        let spec_id = SpecId::new();
        let forge_id = ForgeSessionId::new();
        let user_id = UserId::new();
        let config_id = ConfigId::new();

        assert!(mission_id.to_string().starts_with("msn_"));
        assert!(spec_id.to_string().starts_with("spc_"));
        assert!(forge_id.to_string().starts_with("frg_"));
        assert!(user_id.to_string().starts_with("usr_"));
        assert!(config_id.to_string().starts_with("cfg_"));
    }
}