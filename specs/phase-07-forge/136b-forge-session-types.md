# 136b - Forge Session Types

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 136b
**Status:** Planned
**Dependencies:** 136a-forge-crate-setup
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define the core ForgeSession struct and related types for tracking multi-model brainstorming sessions.

---

## Acceptance Criteria

- [ ] `ForgeSession` struct with full lifecycle tracking
- [ ] `ForgeSessionStatus` enum
- [ ] `BrainstormTopic` struct
- [ ] `ForgeSessionConfig` with defaults
- [ ] `TokenCount` for usage tracking

---

## Implementation Details

### 1. Session Types (src/session.rs)

```rust
//! Forge session types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tachikoma_common_core::{ForgeSessionId, Timestamp};

use crate::round::ForgeRound;
use crate::participant::Participant;

/// A Forge brainstorming session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    /// Unique session identifier.
    pub id: ForgeSessionId,
    /// Human-readable session name.
    pub name: String,
    /// What the session is brainstorming about.
    pub topic: BrainstormTopic,
    /// Current session status.
    pub status: ForgeSessionStatus,
    /// Configuration for this session.
    pub config: ForgeSessionConfig,
    /// Participating models.
    pub participants: Vec<Participant>,
    /// Completed rounds.
    pub rounds: Vec<ForgeRound>,
    /// Current round index (0-based).
    pub current_round: usize,
    /// Session creation time.
    pub created_at: Timestamp,
    /// Last activity time.
    pub updated_at: Timestamp,
    /// Total tokens consumed.
    pub total_tokens: TokenCount,
    /// Total cost in USD.
    pub total_cost_usd: f64,
}

impl ForgeSession {
    /// Create a new session with the given topic.
    pub fn new(name: impl Into<String>, topic: BrainstormTopic) -> Self {
        let now = Timestamp::now();
        Self {
            id: ForgeSessionId::new(),
            name: name.into(),
            topic,
            status: ForgeSessionStatus::Initialized,
            config: ForgeSessionConfig::default(),
            participants: Vec::new(),
            rounds: Vec::new(),
            current_round: 0,
            created_at: now,
            updated_at: now,
            total_tokens: TokenCount::default(),
            total_cost_usd: 0.0,
        }
    }

    /// Check if session can continue.
    pub fn can_continue(&self) -> bool {
        matches!(
            self.status,
            ForgeSessionStatus::Initialized
                | ForgeSessionStatus::InProgress
                | ForgeSessionStatus::Paused
        )
    }

    /// Check if session has converged.
    pub fn is_converged(&self) -> bool {
        matches!(self.status, ForgeSessionStatus::Converged)
    }
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeSessionStatus {
    Initialized,
    InProgress,
    Paused,
    Converged,
    Aborted,
    TimedOut,
    Complete,
}

/// What the session is brainstorming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainstormTopic {
    pub title: String,
    pub description: String,
    pub constraints: Vec<String>,
    pub output_type: OutputType,
}

impl BrainstormTopic {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            constraints: Vec::new(),
            output_type: OutputType::Specification,
        }
    }

    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
}

/// Target output type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Specification,
    Code,
    Documentation,
    Design,
    Freeform,
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSessionConfig {
    pub max_rounds: usize,
    pub max_time_secs: u64,
    pub max_cost_usd: f64,
    pub convergence_threshold: f64,
    pub attended: bool,
    pub min_consensus: usize,
}

impl Default for ForgeSessionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 10,
            max_time_secs: 3600,
            max_cost_usd: 10.0,
            convergence_threshold: 0.85,
            attended: false,
            min_consensus: 2,
        }
    }
}

/// Token usage tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
}

impl TokenCount {
    pub fn total(&self) -> u64 {
        self.input + self.output
    }

    pub fn add(&mut self, other: &TokenCount) {
        self.input += other.input;
        self.output += other.output;
    }
}
```

---

## Testing Requirements

1. Session creation with all required fields
2. Session status transitions are valid
3. Token counting accumulates correctly

---

## Related Specs

- Depends on: [136a-forge-crate-setup.md](136a-forge-crate-setup.md)
- Next: [136c-forge-round-types.md](136c-forge-round-types.md)
