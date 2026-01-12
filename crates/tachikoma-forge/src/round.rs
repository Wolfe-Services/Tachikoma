//! Forge round types and implementations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Participant, Critique, ConflictResolution, ConvergenceVote};
use crate::session::TokenUsage;

/// Different types of rounds in a forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForgeRound {
    Draft(DraftRound),
    Critique(CritiqueRound),
    Synthesis(SynthesisRound),
    Refinement(RefinementRound),
    Convergence(ConvergenceRound),
}

/// A round where content is drafted or re-drafted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRound {
    pub drafter: Participant,
    pub content: String,
    pub reasoning: String,
    pub tokens: TokenUsage,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// A round where content is critiqued by multiple participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueRound {
    pub critiques: Vec<Critique>,
    pub timestamp: DateTime<Utc>,
}

/// A round where conflicting critiques are synthesized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisRound {
    pub synthesizer: Participant,
    pub content: String,
    pub reasoning: String,
    pub resolved_conflicts: Vec<ConflictResolution>,
    pub changes: Vec<String>,
    pub tokens: TokenUsage,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// A round focused on refining specific aspects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementRound {
    pub refiner: Participant,
    pub content: String,
    pub focus_area: String,
    pub depth: String,
    pub tokens: TokenUsage,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// A round that checks for convergence among participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceRound {
    pub score: f64,
    pub converged: bool,
    pub votes: Vec<ConvergenceVote>,
    pub remaining_issues: Vec<String>,
    pub timestamp: DateTime<Utc>,
}