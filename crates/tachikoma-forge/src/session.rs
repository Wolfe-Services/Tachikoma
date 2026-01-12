//! Forge session types and management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tachikoma_common_core::ForgeSessionId;
use uuid::Uuid;

use crate::round::ForgeRound;

/// Status of a forge session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForgeSessionStatus {
    /// Session is being created.
    Creating,
    /// Session is active and rounds are being conducted.
    Active,
    /// Session has converged on a solution.
    Converged,
    /// Session was manually stopped.
    Stopped,
    /// Session failed due to an error.
    Failed(String),
}

/// Configuration for a forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSessionConfig {
    /// Maximum number of rounds to run.
    pub max_rounds: usize,
    /// Convergence threshold (0.0 to 1.0).
    pub convergence_threshold: f64,
    /// Timeout for individual rounds in milliseconds.
    pub round_timeout_ms: u64,
}

impl Default for ForgeSessionConfig {
    fn default() -> Self {
        Self {
            max_rounds: 10,
            convergence_threshold: 0.8,
            round_timeout_ms: 300_000, // 5 minutes
        }
    }
}

/// Topic for a forge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeTopic {
    /// Title of the topic.
    pub title: String,
    /// Description of what needs to be created/solved.
    pub description: String,
    /// Constraints or requirements.
    pub constraints: Vec<String>,
}

/// Token usage tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
}

impl TokenUsage {
    pub fn total(&self) -> u32 {
        self.input + self.output
    }
    
    pub fn add(&mut self, other: &TokenUsage) {
        self.input += other.input;
        self.output += other.output;
    }
}

/// A Forge session for collaborative document creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    /// Unique identifier for this session.
    pub id: ForgeSessionId,
    /// Current status of the session.
    pub status: ForgeSessionStatus,
    /// Session configuration.
    pub config: ForgeSessionConfig,
    /// The topic being worked on.
    pub topic: ForgeTopic,
    /// All rounds conducted in this session.
    pub rounds: Vec<ForgeRound>,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,
    /// Total cost in USD for this session.
    pub total_cost_usd: f64,
    /// Total token usage for this session.
    pub total_tokens: TokenUsage,
}

impl ForgeSession {
    /// Create a new forge session.
    pub fn new(config: ForgeSessionConfig, topic: ForgeTopic) -> Self {
        let now = Utc::now();
        Self {
            id: ForgeSessionId(Uuid::new_v4()),
            status: ForgeSessionStatus::Creating,
            config,
            topic,
            rounds: Vec::new(),
            created_at: now,
            updated_at: now,
            total_cost_usd: 0.0,
            total_tokens: TokenUsage::default(),
        }
    }
    
    /// Get the latest draft from the session, if any.
    pub fn latest_draft(&self) -> Option<&str> {
        // Find the most recent draft round
        self.rounds
            .iter()
            .rev()
            .find_map(|round| match round {
                ForgeRound::Draft(draft) => Some(draft.content.as_str()),
                ForgeRound::Synthesis(synthesis) => Some(synthesis.content.as_str()),
                ForgeRound::Refinement(refinement) => Some(refinement.content.as_str()),
                _ => None,
            })
    }
    
    /// Add a round to the session.
    pub fn add_round(&mut self, round: ForgeRound) {
        self.rounds.push(round);
        self.updated_at = Utc::now();
    }
    
    /// Update session status.
    pub fn set_status(&mut self, status: ForgeSessionStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
    
    /// Add to token usage and cost.
    pub fn add_usage(&mut self, tokens: &TokenUsage, cost_usd: f64) {
        self.total_tokens.add(tokens);
        self.total_cost_usd += cost_usd;
        self.updated_at = Utc::now();
    }
}