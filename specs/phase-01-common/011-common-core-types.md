# 011 - Common Core Types

**Phase:** 1 - Core Common Crates
**Spec ID:** 011
**Status:** Planned
**Dependencies:** 002-rust-workspace
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Define the foundational types used across all Tachikoma crates: IDs, timestamps, status enums, and common traits.

---

## Acceptance Criteria

- [x] `tachikoma-common-core` crate created
- [x] ID types with validation (MissionId, SpecId, etc.)
- [x] Timestamp wrapper with serialization
- [x] Common status enums
- [x] Serialize/Deserialize for all types
- [x] Display implementations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-common-core/Cargo.toml)

```toml
[package]
name = "tachikoma-common-core"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core types for Tachikoma"

[dependencies]
serde = { workspace = true, features = ["derive"] }
thiserror.workspace = true
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
proptest.workspace = true
serde_json = { workspace = true }
```

### 2. ID Types (src/id.rs)

```rust
//! Strongly-typed identifiers.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A strongly-typed ID wrapper.
macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
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
}
```

### 3. Timestamp Type (src/timestamp.rs)

```rust
//! Timestamp utilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A UTC timestamp.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Current time.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// From a DateTime.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Get the inner DateTime.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// ISO 8601 string.
    pub fn to_iso8601(&self) -> String {
        self.0.to_rfc3339()
    }

    /// Duration since this timestamp.
    pub fn elapsed(&self) -> chrono::Duration {
        Utc::now() - self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timestamp({})", self)
    }
}
```

### 4. Status Enums (src/status.rs)

```rust
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
    Planned,
    InProgress,
    Complete,
    Blocked,
}

/// Forge session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeStatus {
    Drafting,
    Critiquing,
    Synthesizing,
    Converged,
    Aborted,
}

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

### 5. Library Root (src/lib.rs)

```rust
//! Tachikoma common core types.
//!
//! This crate provides foundational types used across all Tachikoma components.

#![warn(missing_docs)]

pub mod id;
pub mod status;
pub mod timestamp;

pub use id::*;
pub use status::*;
pub use timestamp::*;
```

---

## Testing Requirements

1. All ID types serialize/deserialize correctly
2. ID parsing handles with/without prefix
3. Timestamp comparisons work correctly
4. Status enums have correct JSON representation

---

## Related Specs

- Depends on: [002-rust-workspace.md](../phase-00-setup/002-rust-workspace.md)
- Next: [012-error-types.md](012-error-types.md)
- Used by: All other crates
