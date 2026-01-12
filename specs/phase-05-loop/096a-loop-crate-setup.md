# 096a - Loop Runner Crate Setup

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 096a
**Status:** Planned
**Dependencies:** 011-common-core-types, 019-async-runtime
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Create the `tachikoma-loop-runner` crate structure with Cargo.toml, library root, and LoopId type.

---

## Acceptance Criteria

- [ ] `tachikoma-loop-runner` crate created
- [ ] Cargo.toml with dependencies
- [ ] LoopId type defined
- [ ] Library root with module declarations

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-loop-runner/Cargo.toml)

```toml
[package]
name = "tachikoma-loop-runner"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Ralph Wiggum Loop Runner for continuous Claude Code execution"

[dependencies]
tachikoma-common-core.workspace = true

tokio = { workspace = true, features = ["full", "sync", "time"] }
async-trait = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
humantime = "2.1"
humantime-serde = "1.1"

[dev-dependencies]
tokio-test = "0.4"
proptest.workspace = true
mockall = "0.11"
```

### 2. Library Root (src/lib.rs)

```rust
//! Ralph Wiggum Loop Runner.
//!
//! This crate provides the core loop execution logic for continuous
//! Claude Code development sessions with automatic context management.

#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod events;
pub mod runner;
pub mod session;
pub mod state;
pub mod stop;

pub use config::{LoopConfig, SessionConfig, StopConditionsConfig};
pub use error::{LoopError, LoopResult};
pub use events::LoopEvent;
pub use runner::{IterationOutcome, LoopCommand, LoopRunner};
pub use state::{LoopContext, LoopState, LoopStats, LoopStatsSnapshot};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a loop execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoopId(Uuid);

impl LoopId {
    /// Create a new random loop ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for LoopId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LoopId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop_{}", self.0)
    }
}
```

---

## Testing Requirements

1. Crate compiles successfully
2. LoopId generates unique values
3. LoopId serializes correctly

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [096b-loop-config.md](096b-loop-config.md)
