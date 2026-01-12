# 136 - Forge Session Types

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 136
**Status:** Planned
**Dependencies:** 011-common-core-types, 012-error-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define the core types for Forge brainstorming sessions where multiple LLMs collaborate, critique, and synthesize specifications through structured debate rounds.

---

## Acceptance Criteria

- [ ] `tachikoma-forge-types` crate created
- [ ] `ForgeSession` struct with full lifecycle tracking
- [ ] `ForgeRound` enum with all round types
- [ ] `ModelResponse` for capturing LLM outputs
- [ ] `BrainstormTopic` for session focus
- [ ] Serialization for all types
- [ ] Builder patterns for complex types

---

## Implementation Details

### 1. Crate Setup (crates/tachikoma-forge-types/Cargo.toml)

```toml
[package]
name = "tachikoma-forge-types"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Core types for Tachikoma Spec Forge"

[dependencies]
tachikoma-common-core.workspace = true
serde = { workspace = true, features = ["derive"] }
thiserror.workspace = true
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
serde_json.workspace = true
```

### 2. Session Types (src/session.rs)

```rust
//! Forge session types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tachikoma_common_core::{ForgeSessionId, Timestamp};
use uuid::Uuid;

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

    /// Get the latest draft if any.
    pub fn latest_draft(&self) -> Option<&str> {
        self.rounds.iter().rev().find_map(|r| match r {
            ForgeRound::Draft(d) => Some(d.content.as_str()),
            ForgeRound::Synthesis(s) => Some(s.merged_content.as_str()),
            _ => None,
        })
    }
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeSessionStatus {
    /// Session created but not started.
    Initialized,
    /// Session actively running.
    InProgress,
    /// Session paused (attended mode).
    Paused,
    /// Models have converged on a solution.
    Converged,
    /// Session aborted by user or error.
    Aborted,
    /// Session timed out.
    TimedOut,
    /// Session completed with final output.
    Complete,
}

/// What the session is brainstorming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainstormTopic {
    /// Brief title.
    pub title: String,
    /// Detailed description of what to brainstorm.
    pub description: String,
    /// Optional context/constraints.
    pub constraints: Vec<String>,
    /// Optional reference materials.
    pub references: Vec<Reference>,
    /// Target output type.
    pub output_type: OutputType,
}

impl BrainstormTopic {
    /// Create a new topic.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            constraints: Vec::new(),
            references: Vec::new(),
            output_type: OutputType::Specification,
        }
    }

    /// Add a constraint.
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Add a reference.
    pub fn with_reference(mut self, reference: Reference) -> Self {
        self.references.push(reference);
        self
    }
}

/// Reference material for the topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Reference name.
    pub name: String,
    /// Reference content or URL.
    pub content: String,
    /// Reference type.
    pub ref_type: ReferenceType,
}

/// Type of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    /// Inline content.
    Inline,
    /// File path.
    File,
    /// URL.
    Url,
    /// Existing spec.
    Spec,
}

/// Target output type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    /// Generate a specification.
    Specification,
    /// Generate code.
    Code,
    /// Generate documentation.
    Documentation,
    /// Generate a design document.
    Design,
    /// Free-form output.
    Freeform,
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSessionConfig {
    /// Maximum number of rounds.
    pub max_rounds: usize,
    /// Maximum time in seconds.
    pub max_time_secs: u64,
    /// Maximum cost in USD.
    pub max_cost_usd: f64,
    /// Convergence threshold (0.0 - 1.0).
    pub convergence_threshold: f64,
    /// Whether to run in attended mode.
    pub attended: bool,
    /// Minimum models that must agree.
    pub min_consensus: usize,
    /// Enable recursive refinement.
    pub recursive_refinement: bool,
    /// Maximum recursive depth.
    pub max_recursive_depth: usize,
}

impl Default for ForgeSessionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 10,
            max_time_secs: 3600, // 1 hour
            max_cost_usd: 10.0,
            convergence_threshold: 0.85,
            attended: false,
            min_consensus: 2,
            recursive_refinement: true,
            max_recursive_depth: 3,
        }
    }
}

/// Token usage tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCount {
    /// Input tokens.
    pub input: u64,
    /// Output tokens.
    pub output: u64,
}

impl TokenCount {
    /// Total tokens.
    pub fn total(&self) -> u64 {
        self.input + self.output
    }

    /// Add another token count.
    pub fn add(&mut self, other: &TokenCount) {
        self.input += other.input;
        self.output += other.output;
    }
}
```

### 3. Round Types (src/round.rs)

```rust
//! Forge round types.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::Timestamp;

use crate::{Participant, TokenCount};

/// A round in a Forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ForgeRound {
    /// Initial draft generation.
    Draft(DraftRound),
    /// Critique of previous output.
    Critique(CritiqueRound),
    /// Synthesis of critiques.
    Synthesis(SynthesisRound),
    /// Recursive refinement.
    Refinement(RefinementRound),
    /// Final convergence check.
    Convergence(ConvergenceRound),
}

impl ForgeRound {
    /// Get the round number.
    pub fn round_number(&self) -> usize {
        match self {
            ForgeRound::Draft(r) => r.round_number,
            ForgeRound::Critique(r) => r.round_number,
            ForgeRound::Synthesis(r) => r.round_number,
            ForgeRound::Refinement(r) => r.round_number,
            ForgeRound::Convergence(r) => r.round_number,
        }
    }

    /// Get the timestamp.
    pub fn timestamp(&self) -> Timestamp {
        match self {
            ForgeRound::Draft(r) => r.timestamp,
            ForgeRound::Critique(r) => r.timestamp,
            ForgeRound::Synthesis(r) => r.timestamp,
            ForgeRound::Refinement(r) => r.timestamp,
            ForgeRound::Convergence(r) => r.timestamp,
        }
    }

    /// Get token usage for this round.
    pub fn tokens(&self) -> TokenCount {
        match self {
            ForgeRound::Draft(r) => r.tokens.clone(),
            ForgeRound::Critique(r) => {
                let mut total = TokenCount::default();
                for c in &r.critiques {
                    total.add(&c.tokens);
                }
                total
            }
            ForgeRound::Synthesis(r) => r.tokens.clone(),
            ForgeRound::Refinement(r) => r.tokens.clone(),
            ForgeRound::Convergence(r) => r.tokens.clone(),
        }
    }
}

/// Initial draft round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRound {
    /// Round number.
    pub round_number: usize,
    /// Drafting model.
    pub drafter: Participant,
    /// Draft content.
    pub content: String,
    /// Prompt used.
    pub prompt: String,
    /// Timestamp.
    pub timestamp: Timestamp,
    /// Token usage.
    pub tokens: TokenCount,
    /// Generation time in ms.
    pub duration_ms: u64,
}

/// Critique round collecting feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueRound {
    /// Round number.
    pub round_number: usize,
    /// Individual critiques.
    pub critiques: Vec<Critique>,
    /// Timestamp.
    pub timestamp: Timestamp,
}

/// A single critique from one model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Critique {
    /// Critiquing model.
    pub critic: Participant,
    /// Strengths identified.
    pub strengths: Vec<String>,
    /// Weaknesses identified.
    pub weaknesses: Vec<String>,
    /// Specific suggestions.
    pub suggestions: Vec<Suggestion>,
    /// Overall score (0-100).
    pub score: u8,
    /// Raw response content.
    pub raw_content: String,
    /// Token usage.
    pub tokens: TokenCount,
    /// Generation time in ms.
    pub duration_ms: u64,
}

/// A specific suggestion for improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// What section this applies to.
    pub section: Option<String>,
    /// The suggestion text.
    pub text: String,
    /// Priority (1 = highest).
    pub priority: u8,
    /// Category of suggestion.
    pub category: SuggestionCategory,
}

/// Categories of suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    /// Factual correctness.
    Correctness,
    /// Clarity and readability.
    Clarity,
    /// Completeness.
    Completeness,
    /// Code quality.
    CodeQuality,
    /// Architecture/design.
    Architecture,
    /// Performance.
    Performance,
    /// Security.
    Security,
    /// Other.
    Other,
}

/// Synthesis round merging critiques.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisRound {
    /// Round number.
    pub round_number: usize,
    /// Synthesizing model.
    pub synthesizer: Participant,
    /// Merged content.
    pub merged_content: String,
    /// Conflicts that were resolved.
    pub resolved_conflicts: Vec<ConflictResolution>,
    /// Changes made from previous draft.
    pub changes: Vec<Change>,
    /// Timestamp.
    pub timestamp: Timestamp,
    /// Token usage.
    pub tokens: TokenCount,
    /// Generation time in ms.
    pub duration_ms: u64,
}

/// A resolved conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    /// What the conflict was about.
    pub issue: String,
    /// Conflicting positions.
    pub positions: Vec<ConflictPosition>,
    /// How it was resolved.
    pub resolution: String,
    /// Rationale for the resolution.
    pub rationale: String,
}

/// A position in a conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPosition {
    /// Who holds this position.
    pub participant: Participant,
    /// The position.
    pub position: String,
}

/// A change made in synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Section changed.
    pub section: String,
    /// Type of change.
    pub change_type: ChangeType,
    /// Description.
    pub description: String,
    /// Based on which suggestions.
    pub based_on_suggestions: Vec<usize>,
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

/// Refinement round for recursive improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementRound {
    /// Round number.
    pub round_number: usize,
    /// Refining model.
    pub refiner: Participant,
    /// Focus area for refinement.
    pub focus_area: String,
    /// Refined content.
    pub refined_content: String,
    /// Depth of recursion.
    pub depth: usize,
    /// Timestamp.
    pub timestamp: Timestamp,
    /// Token usage.
    pub tokens: TokenCount,
    /// Generation time in ms.
    pub duration_ms: u64,
}

/// Convergence check round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceRound {
    /// Round number.
    pub round_number: usize,
    /// Convergence score (0.0 - 1.0).
    pub score: f64,
    /// Has converged.
    pub converged: bool,
    /// Votes from each participant.
    pub votes: Vec<ConvergenceVote>,
    /// Remaining issues.
    pub remaining_issues: Vec<String>,
    /// Timestamp.
    pub timestamp: Timestamp,
    /// Token usage.
    pub tokens: TokenCount,
}

/// A convergence vote from a participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceVote {
    /// Voting participant.
    pub participant: Participant,
    /// Agrees with current output.
    pub agrees: bool,
    /// Score (0-100).
    pub score: u8,
    /// Concerns if any.
    pub concerns: Vec<String>,
}
```

### 4. Participant Type (src/participant.rs)

```rust
//! Participant (model) types.

use serde::{Deserialize, Serialize};

/// A participant in a Forge session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Participant {
    /// Model identifier (e.g., "claude-3-opus").
    pub model_id: String,
    /// Display name.
    pub display_name: String,
    /// Provider.
    pub provider: ModelProvider,
    /// Role in the session.
    pub role: ParticipantRole,
}

impl Participant {
    /// Create a new participant.
    pub fn new(
        model_id: impl Into<String>,
        display_name: impl Into<String>,
        provider: ModelProvider,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            display_name: display_name.into(),
            provider,
            role: ParticipantRole::Generalist,
        }
    }

    /// Set the role.
    pub fn with_role(mut self, role: ParticipantRole) -> Self {
        self.role = role;
        self
    }

    /// Create Claude Opus participant.
    pub fn claude_opus() -> Self {
        Self::new(
            "claude-3-opus-20240229",
            "Claude Opus",
            ModelProvider::Anthropic,
        )
    }

    /// Create Claude Sonnet participant.
    pub fn claude_sonnet() -> Self {
        Self::new(
            "claude-3-5-sonnet-20241022",
            "Claude Sonnet",
            ModelProvider::Anthropic,
        )
    }

    /// Create GPT-4 participant.
    pub fn gpt4() -> Self {
        Self::new("gpt-4-turbo", "GPT-4 Turbo", ModelProvider::OpenAI)
    }

    /// Create Gemini participant.
    pub fn gemini_pro() -> Self {
        Self::new("gemini-pro", "Gemini Pro", ModelProvider::Google)
    }
}

/// Model provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Google,
    Local,
    Custom,
}

/// Role in the Forge session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    /// General-purpose participant.
    Generalist,
    /// Primary drafter.
    Drafter,
    /// Primary critic.
    Critic,
    /// Synthesizer/mediator.
    Synthesizer,
    /// Domain expert.
    DomainExpert,
    /// Code reviewer.
    CodeReviewer,
    /// Devil's advocate.
    DevilsAdvocate,
}

impl ParticipantRole {
    /// Get the system prompt modifier for this role.
    pub fn system_prompt_modifier(&self) -> &'static str {
        match self {
            Self::Generalist => "",
            Self::Drafter => "You are the primary drafter. Focus on creating comprehensive initial content.",
            Self::Critic => "You are a critic. Be thorough in identifying weaknesses and suggesting improvements.",
            Self::Synthesizer => "You are the synthesizer. Your job is to merge different perspectives into a coherent whole.",
            Self::DomainExpert => "You are a domain expert. Focus on technical accuracy and best practices.",
            Self::CodeReviewer => "You are a code reviewer. Focus on code quality, security, and maintainability.",
            Self::DevilsAdvocate => "You are the devil's advocate. Challenge assumptions and find edge cases.",
        }
    }
}
```

### 5. Model Response (src/response.rs)

```rust
//! Model response types.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::Timestamp;

use crate::{Participant, TokenCount};

/// A response from a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// Which model responded.
    pub participant: Participant,
    /// The response content.
    pub content: String,
    /// Token usage.
    pub tokens: TokenCount,
    /// Response time in ms.
    pub duration_ms: u64,
    /// When the response was received.
    pub timestamp: Timestamp,
    /// Stop reason.
    pub stop_reason: StopReason,
    /// Raw API response (for debugging).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_response: Option<String>,
}

impl ModelResponse {
    /// Create a new response.
    pub fn new(participant: Participant, content: String) -> Self {
        Self {
            participant,
            content,
            tokens: TokenCount::default(),
            duration_ms: 0,
            timestamp: Timestamp::now(),
            stop_reason: StopReason::EndTurn,
            raw_response: None,
        }
    }

    /// Set token usage.
    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.tokens = TokenCount { input, output };
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}

/// Why the model stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Normal completion.
    EndTurn,
    /// Hit max tokens.
    MaxTokens,
    /// Hit stop sequence.
    StopSequence,
    /// Tool use requested.
    ToolUse,
    /// Error occurred.
    Error,
}
```

### 6. Library Root (src/lib.rs)

```rust
//! Tachikoma Forge types.
//!
//! Core types for the Spec Forge multi-model brainstorming system.

#![warn(missing_docs)]

pub mod participant;
pub mod response;
pub mod round;
pub mod session;

pub use participant::*;
pub use response::*;
pub use round::*;
pub use session::*;
```

---

## Testing Requirements

1. Session creation with all required fields
2. Round type serialization/deserialization
3. Token counting accumulates correctly
4. Session status transitions are valid
5. Participant creation with various providers

---

## Related Specs

- Depends on: [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
- Next: [137-forge-config.md](137-forge-config.md)
- Used by: All Forge specs (138-160)
