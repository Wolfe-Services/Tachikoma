//! Tachikoma Forge - Multi-model brainstorming and collaboration system.

pub mod error;
pub mod output;
pub mod session;
pub mod round;

// Re-export common types
pub use error::{ForgeError, ForgeResult};
pub use session::{ForgeSession, ForgeSessionStatus, ForgeSessionConfig, ForgeTopic, TokenUsage};
pub use round::{ForgeRound, DraftRound, CritiqueRound, SynthesisRound, RefinementRound, ConvergenceRound};

// Re-export from tachikoma-common-core
pub use tachikoma_common_core::ForgeSessionId;

// Placeholder types that will be defined in dependent specs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionLog {
    pub session_id: ForgeSessionId,
    pub decisions: Vec<Decision>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Decision {
    pub id: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DecisionLog {
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            session_id,
            decisions: Vec::new(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        for decision in &self.decisions {
            output.push_str(&format!("- **{}**: {} ({})\n", decision.id, decision.description, decision.timestamp.format("%Y-%m-%d %H:%M UTC")));
        }
        output
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DissentLog {
    pub session_id: ForgeSessionId,
    pub dissents: Vec<Dissent>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dissent {
    pub id: String,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DissentLog {
    pub fn new(session_id: ForgeSessionId) -> Self {
        Self {
            session_id,
            dissents: Vec::new(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        for dissent in &self.dissents {
            output.push_str(&format!("- **{}**: {} ({})\n", dissent.id, dissent.description, dissent.timestamp.format("%Y-%m-%d %H:%M UTC")));
        }
        output
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Participant {
    pub id: String,
    pub display_name: String,
    pub model_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Critique {
    pub critic: Participant,
    pub score: u8,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConflictResolution {
    pub issue: String,
    pub resolution: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConvergenceVote {
    pub participant: Participant,
    pub agrees: bool,
    pub reasoning: String,
}