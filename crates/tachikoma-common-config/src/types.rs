//! Configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TachikomaConfig {
    /// Backend configuration.
    pub backend: BackendConfig,
    /// Loop runner configuration.
    pub loop_config: LoopConfig,
    /// Policy configuration.
    pub policies: PolicyConfig,
    /// Forge configuration.
    pub forge: ForgeConfig,
}

impl Default for TachikomaConfig {
    fn default() -> Self {
        Self {
            backend: BackendConfig::default(),
            loop_config: LoopConfig::default(),
            policies: PolicyConfig::default(),
            forge: ForgeConfig::default(),
        }
    }
}

/// Backend model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackendConfig {
    /// Fast agentic model (Brain).
    pub brain: String,
    /// Deep reasoning model (Think Tank/Oracle).
    pub think_tank: String,
    /// API keys (loaded from env if not set).
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    /// Custom backend endpoints.
    #[serde(default)]
    pub endpoints: HashMap<String, String>,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            brain: "claude".to_string(),
            think_tank: "o3".to_string(),
            api_keys: HashMap::new(),
            endpoints: HashMap::new(),
        }
    }
}

/// Loop runner configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoopConfig {
    /// Maximum loop iterations.
    pub max_iterations: u32,
    /// Stop conditions.
    pub stop_on: Vec<StopCondition>,
    /// Context usage warning threshold (0.0-1.0).
    pub redline_threshold: f32,
    /// Delay between iterations (ms).
    pub iteration_delay_ms: u64,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            stop_on: vec![
                StopCondition::Redline,
                StopCondition::TestFailStreak(3),
                StopCondition::NoProgress(5),
            ],
            redline_threshold: 0.75,
            iteration_delay_ms: 1000,
        }
    }
}

/// Stop conditions for the loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopCondition {
    /// Context window redlined.
    Redline,
    /// Consecutive test failures.
    TestFailStreak(u32),
    /// No progress on checkboxes.
    NoProgress(u32),
    /// Error rate exceeded.
    ErrorRate(u32),
    /// Manual stop requested.
    ManualStop,
    /// All tasks complete.
    AllComplete,
}

/// Policy configuration for guardrails.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PolicyConfig {
    /// Require tests to pass before deploy.
    pub deploy_requires_tests: bool,
    /// Require attended mode by default.
    pub attended_by_default: bool,
    /// Auto-commit on success.
    pub auto_commit: bool,
    /// Auto-push on commit.
    pub auto_push: bool,
    /// Require spec for each mission.
    pub require_spec: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            deploy_requires_tests: true,
            attended_by_default: true,
            auto_commit: false,
            auto_push: false,
            require_spec: true,
        }
    }
}

/// Spec Forge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ForgeConfig {
    /// Participant models for brainstorming.
    pub participants: Vec<String>,
    /// Oracle model for synthesis.
    pub oracle: String,
    /// Maximum rounds.
    pub max_rounds: u32,
    /// Convergence threshold (0.0-1.0).
    pub convergence_threshold: f32,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            participants: vec![
                "claude".to_string(),
                "gemini".to_string(),
            ],
            oracle: "o3".to_string(),
            max_rounds: 5,
            convergence_threshold: 0.9,
        }
    }
}