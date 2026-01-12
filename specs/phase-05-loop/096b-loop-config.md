# 096b - Loop Configuration Types

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 096b
**Status:** Planned
**Dependencies:** 096a-loop-crate-setup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define configuration types for the loop runner including iteration settings, stop conditions, and session configuration.

---

## Acceptance Criteria

- [ ] `LoopConfig` struct with all settings
- [ ] `StopConditionsConfig` struct
- [ ] `SessionConfig` struct
- [ ] Sensible default values

---

## Implementation Details

### 1. Configuration Types (src/config.rs)

```rust
//! Loop runner configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the Ralph Loop Runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Working directory for the loop.
    pub working_dir: PathBuf,
    /// Path to the prompt.md file.
    pub prompt_path: PathBuf,
    /// Maximum iterations (0 = unlimited).
    pub max_iterations: u32,
    /// Delay between iterations.
    #[serde(with = "humantime_serde")]
    pub iteration_delay: Duration,
    /// Context redline threshold (percentage 0-100).
    pub context_redline_percent: u8,
    /// Enable attended mode (requires user confirmation).
    pub attended_mode: bool,
    /// Stop conditions configuration.
    pub stop_conditions: StopConditionsConfig,
    /// Session configuration.
    pub session: SessionConfig,
}

/// Stop conditions configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopConditionsConfig {
    /// Stop after N consecutive test failures.
    pub max_test_failure_streak: u32,
    /// Stop after N iterations with no progress.
    pub max_no_progress_iterations: u32,
    /// Stop on specific error patterns.
    pub stop_on_patterns: Vec<String>,
    /// Custom stop condition scripts.
    pub custom_scripts: Vec<PathBuf>,
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Claude Code command to execute.
    pub claude_command: String,
    /// Session timeout.
    #[serde(with = "humantime_serde")]
    pub session_timeout: Duration,
    /// Enable session persistence.
    pub persist_sessions: bool,
    /// Session state directory.
    pub state_dir: PathBuf,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("."),
            prompt_path: PathBuf::from("prompt.md"),
            max_iterations: 0,
            iteration_delay: Duration::from_secs(5),
            context_redline_percent: 85,
            attended_mode: false,
            stop_conditions: StopConditionsConfig::default(),
            session: SessionConfig::default(),
        }
    }
}

impl Default for StopConditionsConfig {
    fn default() -> Self {
        Self {
            max_test_failure_streak: 3,
            max_no_progress_iterations: 5,
            stop_on_patterns: vec![],
            custom_scripts: vec![],
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            claude_command: "claude".to_string(),
            session_timeout: Duration::from_secs(3600),
            persist_sessions: true,
            state_dir: PathBuf::from(".ralph/sessions"),
        }
    }
}
```

---

## Testing Requirements

1. Default configurations are valid
2. Configuration serializes/deserializes correctly
3. Duration fields use human-readable format

---

## Related Specs

- Depends on: [096a-loop-crate-setup.md](096a-loop-crate-setup.md)
- Next: [096c-loop-state.md](096c-loop-state.md)
