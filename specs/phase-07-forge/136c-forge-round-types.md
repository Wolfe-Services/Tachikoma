# 136c - Forge Round Types

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 136c
**Status:** Planned
**Dependencies:** 136b-forge-session-types
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Define the round types for Forge sessions including draft, critique, synthesis, and convergence rounds.

---

## Acceptance Criteria

- [ ] `ForgeRound` enum with all round types
- [ ] `DraftRound`, `CritiqueRound`, `SynthesisRound` structs
- [ ] `Critique` and `Suggestion` types
- [ ] `ConvergenceRound` for final checks

---

## Implementation Details

### 1. Round Types (src/round.rs)

```rust
//! Forge round types.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::Timestamp;

use crate::{Participant, TokenCount};

/// A round in a Forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ForgeRound {
    Draft(DraftRound),
    Critique(CritiqueRound),
    Synthesis(SynthesisRound),
    Convergence(ConvergenceRound),
}

impl ForgeRound {
    pub fn round_number(&self) -> usize {
        match self {
            ForgeRound::Draft(r) => r.round_number,
            ForgeRound::Critique(r) => r.round_number,
            ForgeRound::Synthesis(r) => r.round_number,
            ForgeRound::Convergence(r) => r.round_number,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        match self {
            ForgeRound::Draft(r) => r.timestamp,
            ForgeRound::Critique(r) => r.timestamp,
            ForgeRound::Synthesis(r) => r.timestamp,
            ForgeRound::Convergence(r) => r.timestamp,
        }
    }
}

/// Initial draft round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRound {
    pub round_number: usize,
    pub drafter: Participant,
    pub content: String,
    pub prompt: String,
    pub timestamp: Timestamp,
    pub tokens: TokenCount,
    pub duration_ms: u64,
}

/// Critique round collecting feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueRound {
    pub round_number: usize,
    pub critiques: Vec<Critique>,
    pub timestamp: Timestamp,
}

/// A single critique from one model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    pub critic: Participant,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub suggestions: Vec<Suggestion>,
    pub score: u8,
    pub raw_content: String,
    pub tokens: TokenCount,
    pub duration_ms: u64,
}

/// A specific suggestion for improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub section: Option<String>,
    pub text: String,
    pub priority: u8,
    pub category: SuggestionCategory,
}

/// Categories of suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    Correctness,
    Clarity,
    Completeness,
    CodeQuality,
    Architecture,
    Performance,
    Security,
    Other,
}

/// Synthesis round merging critiques.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisRound {
    pub round_number: usize,
    pub synthesizer: Participant,
    pub merged_content: String,
    pub changes: Vec<Change>,
    pub timestamp: Timestamp,
    pub tokens: TokenCount,
    pub duration_ms: u64,
}

/// A change made in synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub section: String,
    pub change_type: ChangeType,
    pub description: String,
}

/// Type of change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
    Restructure,
}

/// Convergence check round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceRound {
    pub round_number: usize,
    pub score: f64,
    pub converged: bool,
    pub votes: Vec<ConvergenceVote>,
    pub remaining_issues: Vec<String>,
    pub timestamp: Timestamp,
    pub tokens: TokenCount,
}

/// A convergence vote from a participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceVote {
    pub participant: Participant,
    pub agrees: bool,
    pub score: u8,
    pub concerns: Vec<String>,
}
```

---

## Testing Requirements

1. Round type serialization/deserialization
2. Token counting for rounds works
3. All round types are complete

---

## Related Specs

- Depends on: [136b-forge-session-types.md](136b-forge-session-types.md)
- Next: [136d-forge-participant-types.md](136d-forge-participant-types.md)
