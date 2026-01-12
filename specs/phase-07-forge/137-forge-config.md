# 137 - Forge Configuration

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 137
**Status:** Planned
**Dependencies:** 136-forge-session-types, 015-yaml-config-parsing
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define the configuration system for Forge sessions, including model selection, round parameters, convergence thresholds, and cost limits.

---

## Acceptance Criteria

- [ ] `ForgeConfig` with all session parameters
- [ ] `ModelConfig` for per-model settings
- [ ] YAML configuration file support
- [ ] Environment variable overrides
- [ ] Validation for all config values
- [ ] Default configurations for common use cases

---

## Implementation Details

### 1. Configuration Types (src/config.rs)

```rust
//! Forge configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::{ModelProvider, ParticipantRole};

/// Root Forge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    /// Default session settings.
    #[serde(default)]
    pub defaults: SessionDefaults,

    /// Model configurations.
    #[serde(default)]
    pub models: ModelConfigs,

    /// Round configurations.
    #[serde(default)]
    pub rounds: RoundConfigs,

    /// Convergence settings.
    #[serde(default)]
    pub convergence: ConvergenceConfig,

    /// Cost and resource limits.
    #[serde(default)]
    pub limits: LimitConfig,

    /// Prompt templates.
    #[serde(default)]
    pub templates: TemplateConfig,

    /// Persistence settings.
    #[serde(default)]
    pub persistence: PersistenceConfig,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            defaults: SessionDefaults::default(),
            models: ModelConfigs::default(),
            rounds: RoundConfigs::default(),
            convergence: ConvergenceConfig::default(),
            limits: LimitConfig::default(),
            templates: TemplateConfig::default(),
            persistence: PersistenceConfig::default(),
        }
    }
}

/// Default session settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDefaults {
    /// Default number of participants.
    pub participant_count: usize,
    /// Default attended mode.
    pub attended: bool,
    /// Enable recursive refinement by default.
    pub recursive_refinement: bool,
    /// Default output format.
    pub output_format: OutputFormat,
}

impl Default for SessionDefaults {
    fn default() -> Self {
        Self {
            participant_count: 3,
            attended: false,
            recursive_refinement: true,
            output_format: OutputFormat::Markdown,
        }
    }
}

/// Output format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Markdown,
    Json,
    Yaml,
    Plain,
}

/// Model configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigs {
    /// Available models by name.
    pub available: HashMap<String, ModelConfig>,
    /// Default model for drafting.
    pub default_drafter: String,
    /// Default models for critique.
    pub default_critics: Vec<String>,
    /// Default model for synthesis.
    pub default_synthesizer: String,
}

impl Default for ModelConfigs {
    fn default() -> Self {
        let mut available = HashMap::new();

        available.insert("claude-opus".to_string(), ModelConfig {
            model_id: "claude-3-opus-20240229".to_string(),
            display_name: "Claude Opus".to_string(),
            provider: ModelProvider::Anthropic,
            max_tokens: 4096,
            temperature: 0.7,
            cost_per_1k_input: 0.015,
            cost_per_1k_output: 0.075,
            preferred_roles: vec![ParticipantRole::Drafter, ParticipantRole::Synthesizer],
            enabled: true,
        });

        available.insert("claude-sonnet".to_string(), ModelConfig {
            model_id: "claude-3-5-sonnet-20241022".to_string(),
            display_name: "Claude Sonnet".to_string(),
            provider: ModelProvider::Anthropic,
            max_tokens: 8192,
            temperature: 0.7,
            cost_per_1k_input: 0.003,
            cost_per_1k_output: 0.015,
            preferred_roles: vec![ParticipantRole::Critic, ParticipantRole::CodeReviewer],
            enabled: true,
        });

        available.insert("gpt-4-turbo".to_string(), ModelConfig {
            model_id: "gpt-4-turbo".to_string(),
            display_name: "GPT-4 Turbo".to_string(),
            provider: ModelProvider::OpenAI,
            max_tokens: 4096,
            temperature: 0.7,
            cost_per_1k_input: 0.01,
            cost_per_1k_output: 0.03,
            preferred_roles: vec![ParticipantRole::Critic, ParticipantRole::DevilsAdvocate],
            enabled: true,
        });

        Self {
            available,
            default_drafter: "claude-opus".to_string(),
            default_critics: vec!["claude-sonnet".to_string(), "gpt-4-turbo".to_string()],
            default_synthesizer: "claude-opus".to_string(),
        }
    }
}

/// Configuration for a single model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model identifier for API calls.
    pub model_id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Provider.
    pub provider: ModelProvider,
    /// Max output tokens.
    pub max_tokens: usize,
    /// Temperature setting.
    pub temperature: f32,
    /// Cost per 1000 input tokens.
    pub cost_per_1k_input: f64,
    /// Cost per 1000 output tokens.
    pub cost_per_1k_output: f64,
    /// Preferred roles for this model.
    pub preferred_roles: Vec<ParticipantRole>,
    /// Whether model is enabled.
    pub enabled: bool,
}

impl ModelConfig {
    /// Calculate cost for given token counts.
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.cost_per_1k_input;
        let output_cost = (output_tokens as f64 / 1000.0) * self.cost_per_1k_output;
        input_cost + output_cost
    }
}

/// Round configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundConfigs {
    /// Draft round settings.
    pub draft: DraftRoundConfig,
    /// Critique round settings.
    pub critique: CritiqueRoundConfig,
    /// Synthesis round settings.
    pub synthesis: SynthesisRoundConfig,
    /// Refinement round settings.
    pub refinement: RefinementRoundConfig,
}

impl Default for RoundConfigs {
    fn default() -> Self {
        Self {
            draft: DraftRoundConfig::default(),
            critique: CritiqueRoundConfig::default(),
            synthesis: SynthesisRoundConfig::default(),
            refinement: RefinementRoundConfig::default(),
        }
    }
}

/// Draft round configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRoundConfig {
    /// Timeout for draft generation.
    pub timeout_secs: u64,
    /// Max retries on failure.
    pub max_retries: usize,
}

impl Default for DraftRoundConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 120,
            max_retries: 2,
        }
    }
}

/// Critique round configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritiqueRoundConfig {
    /// Timeout per critique.
    pub timeout_secs: u64,
    /// Run critiques in parallel.
    pub parallel: bool,
    /// Minimum critiques required.
    pub min_critiques: usize,
    /// Require structured output.
    pub structured_output: bool,
}

impl Default for CritiqueRoundConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 90,
            parallel: true,
            min_critiques: 2,
            structured_output: true,
        }
    }
}

/// Synthesis round configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisRoundConfig {
    /// Timeout for synthesis.
    pub timeout_secs: u64,
    /// Require explicit conflict resolution.
    pub require_conflict_resolution: bool,
    /// Track all changes.
    pub track_changes: bool,
}

impl Default for SynthesisRoundConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 180,
            require_conflict_resolution: true,
            track_changes: true,
        }
    }
}

/// Refinement round configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementRoundConfig {
    /// Timeout per refinement.
    pub timeout_secs: u64,
    /// Maximum recursion depth.
    pub max_depth: usize,
    /// Focus areas for refinement.
    pub focus_areas: Vec<String>,
}

impl Default for RefinementRoundConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 120,
            max_depth: 3,
            focus_areas: vec![
                "code_quality".to_string(),
                "completeness".to_string(),
                "clarity".to_string(),
            ],
        }
    }
}

/// Convergence configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceConfig {
    /// Minimum score for convergence (0.0-1.0).
    pub threshold: f64,
    /// Minimum rounds before checking convergence.
    pub min_rounds: usize,
    /// Maximum rounds before forcing convergence.
    pub max_rounds: usize,
    /// Require unanimous agreement.
    pub require_unanimous: bool,
    /// Minimum participants that must agree.
    pub min_consensus: usize,
    /// Metrics to track.
    pub metrics: Vec<ConvergenceMetric>,
}

impl Default for ConvergenceConfig {
    fn default() -> Self {
        Self {
            threshold: 0.85,
            min_rounds: 2,
            max_rounds: 10,
            require_unanimous: false,
            min_consensus: 2,
            metrics: vec![
                ConvergenceMetric::AgreementScore,
                ConvergenceMetric::ChangeVelocity,
                ConvergenceMetric::IssueCount,
            ],
        }
    }
}

/// Metrics for convergence detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvergenceMetric {
    /// Overall agreement score.
    AgreementScore,
    /// Rate of changes between rounds.
    ChangeVelocity,
    /// Number of remaining issues.
    IssueCount,
    /// Semantic similarity between rounds.
    SemanticSimilarity,
    /// Stability of key sections.
    SectionStability,
}

/// Resource limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitConfig {
    /// Maximum cost in USD.
    pub max_cost_usd: f64,
    /// Maximum tokens total.
    pub max_tokens: u64,
    /// Maximum session duration.
    pub max_duration_secs: u64,
    /// Maximum concurrent API calls.
    pub max_concurrent_calls: usize,
    /// Cost warning threshold (percentage).
    pub cost_warning_threshold: f64,
}

impl Default for LimitConfig {
    fn default() -> Self {
        Self {
            max_cost_usd: 10.0,
            max_tokens: 500_000,
            max_duration_secs: 3600,
            max_concurrent_calls: 5,
            cost_warning_threshold: 0.8,
        }
    }
}

/// Template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Directory for custom templates.
    pub template_dir: Option<PathBuf>,
    /// Default template set.
    pub default_set: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            template_dir: None,
            default_set: "standard".to_string(),
        }
    }
}

/// Persistence configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Directory for session storage.
    pub session_dir: PathBuf,
    /// Auto-save interval in seconds.
    pub auto_save_interval_secs: u64,
    /// Keep completed sessions.
    pub keep_completed: bool,
    /// Max sessions to keep.
    pub max_sessions: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            session_dir: PathBuf::from(".tachikoma/forge/sessions"),
            auto_save_interval_secs: 30,
            keep_completed: true,
            max_sessions: 100,
        }
    }
}
```

### 2. Configuration Loading (src/config_loader.rs)

```rust
//! Configuration loading utilities.

use std::path::Path;
use std::env;

use crate::{ForgeConfig, ForgeError, ForgeResult};

/// Load Forge configuration from file and environment.
pub fn load_config(config_path: Option<&Path>) -> ForgeResult<ForgeConfig> {
    // Start with defaults
    let mut config = ForgeConfig::default();

    // Load from file if provided
    if let Some(path) = config_path {
        let file_config = load_from_file(path)?;
        merge_config(&mut config, file_config);
    }

    // Apply environment overrides
    apply_env_overrides(&mut config)?;

    // Validate
    validate_config(&config)?;

    Ok(config)
}

/// Load configuration from a YAML file.
fn load_from_file(path: &Path) -> ForgeResult<ForgeConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ForgeError::Config(format!("Failed to read config: {}", e)))?;

    serde_yaml::from_str(&content)
        .map_err(|e| ForgeError::Config(format!("Invalid config YAML: {}", e)))
}

/// Merge file config into base config.
fn merge_config(base: &mut ForgeConfig, file: ForgeConfig) {
    // Merge models
    for (name, model) in file.models.available {
        base.models.available.insert(name, model);
    }

    // Override non-default values
    if file.defaults.participant_count != 3 {
        base.defaults.participant_count = file.defaults.participant_count;
    }

    if file.limits.max_cost_usd != 10.0 {
        base.limits.max_cost_usd = file.limits.max_cost_usd;
    }

    if file.convergence.threshold != 0.85 {
        base.convergence.threshold = file.convergence.threshold;
    }

    // Take file persistence settings
    base.persistence = file.persistence;
}

/// Apply environment variable overrides.
fn apply_env_overrides(config: &mut ForgeConfig) -> ForgeResult<()> {
    // FORGE_MAX_COST
    if let Ok(val) = env::var("FORGE_MAX_COST") {
        config.limits.max_cost_usd = val.parse()
            .map_err(|_| ForgeError::Config("Invalid FORGE_MAX_COST".to_string()))?;
    }

    // FORGE_MAX_ROUNDS
    if let Ok(val) = env::var("FORGE_MAX_ROUNDS") {
        config.convergence.max_rounds = val.parse()
            .map_err(|_| ForgeError::Config("Invalid FORGE_MAX_ROUNDS".to_string()))?;
    }

    // FORGE_CONVERGENCE_THRESHOLD
    if let Ok(val) = env::var("FORGE_CONVERGENCE_THRESHOLD") {
        config.convergence.threshold = val.parse()
            .map_err(|_| ForgeError::Config("Invalid FORGE_CONVERGENCE_THRESHOLD".to_string()))?;
    }

    // FORGE_ATTENDED
    if let Ok(val) = env::var("FORGE_ATTENDED") {
        config.defaults.attended = val.to_lowercase() == "true" || val == "1";
    }

    // FORGE_SESSION_DIR
    if let Ok(val) = env::var("FORGE_SESSION_DIR") {
        config.persistence.session_dir = val.into();
    }

    Ok(())
}

/// Validate configuration values.
fn validate_config(config: &ForgeConfig) -> ForgeResult<()> {
    // Validate convergence threshold
    if config.convergence.threshold < 0.0 || config.convergence.threshold > 1.0 {
        return Err(ForgeError::Config(
            "convergence.threshold must be between 0.0 and 1.0".to_string()
        ));
    }

    // Validate min_rounds <= max_rounds
    if config.convergence.min_rounds > config.convergence.max_rounds {
        return Err(ForgeError::Config(
            "convergence.min_rounds cannot exceed max_rounds".to_string()
        ));
    }

    // Validate cost limit is positive
    if config.limits.max_cost_usd <= 0.0 {
        return Err(ForgeError::Config(
            "limits.max_cost_usd must be positive".to_string()
        ));
    }

    // Validate at least one model is enabled
    let enabled_count = config.models.available.values()
        .filter(|m| m.enabled)
        .count();

    if enabled_count == 0 {
        return Err(ForgeError::Config(
            "At least one model must be enabled".to_string()
        ));
    }

    // Validate default models exist
    if !config.models.available.contains_key(&config.models.default_drafter) {
        return Err(ForgeError::Config(
            format!("Default drafter '{}' not in available models", config.models.default_drafter)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config_is_valid() {
        let config = ForgeConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_invalid_threshold() {
        let mut config = ForgeConfig::default();
        config.convergence.threshold = 1.5;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_load_from_yaml() {
        let yaml = r#"
defaults:
  participant_count: 4
  attended: true
limits:
  max_cost_usd: 20.0
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(yaml.as_bytes()).unwrap();

        let config = load_from_file(file.path()).unwrap();
        assert_eq!(config.defaults.participant_count, 4);
        assert!(config.defaults.attended);
        assert_eq!(config.limits.max_cost_usd, 20.0);
    }
}
```

### 3. Sample Configuration File (.tachikoma/forge/config.yaml)

```yaml
# Forge Configuration

defaults:
  participant_count: 3
  attended: false
  recursive_refinement: true
  output_format: markdown

models:
  available:
    claude-opus:
      model_id: "claude-3-opus-20240229"
      display_name: "Claude Opus"
      provider: anthropic
      max_tokens: 4096
      temperature: 0.7
      cost_per_1k_input: 0.015
      cost_per_1k_output: 0.075
      preferred_roles: [drafter, synthesizer]
      enabled: true

    claude-sonnet:
      model_id: "claude-3-5-sonnet-20241022"
      display_name: "Claude Sonnet"
      provider: anthropic
      max_tokens: 8192
      temperature: 0.7
      cost_per_1k_input: 0.003
      cost_per_1k_output: 0.015
      preferred_roles: [critic, code_reviewer]
      enabled: true

    gpt-4-turbo:
      model_id: "gpt-4-turbo"
      display_name: "GPT-4 Turbo"
      provider: openai
      max_tokens: 4096
      temperature: 0.7
      cost_per_1k_input: 0.01
      cost_per_1k_output: 0.03
      preferred_roles: [critic, devils_advocate]
      enabled: true

  default_drafter: claude-opus
  default_critics: [claude-sonnet, gpt-4-turbo]
  default_synthesizer: claude-opus

rounds:
  draft:
    timeout_secs: 120
    max_retries: 2
  critique:
    timeout_secs: 90
    parallel: true
    min_critiques: 2
    structured_output: true
  synthesis:
    timeout_secs: 180
    require_conflict_resolution: true
    track_changes: true
  refinement:
    timeout_secs: 120
    max_depth: 3
    focus_areas:
      - code_quality
      - completeness
      - clarity

convergence:
  threshold: 0.85
  min_rounds: 2
  max_rounds: 10
  require_unanimous: false
  min_consensus: 2
  metrics:
    - agreement_score
    - change_velocity
    - issue_count

limits:
  max_cost_usd: 10.0
  max_tokens: 500000
  max_duration_secs: 3600
  max_concurrent_calls: 5
  cost_warning_threshold: 0.8

persistence:
  session_dir: .tachikoma/forge/sessions
  auto_save_interval_secs: 30
  keep_completed: true
  max_sessions: 100
```

---

## Testing Requirements

1. Default configuration validates successfully
2. YAML parsing handles all fields correctly
3. Environment overrides work for all supported vars
4. Validation catches invalid values
5. Config merging preserves non-default values

---

## Related Specs

- Depends on: [136-forge-session-types.md](136-forge-session-types.md)
- Depends on: [015-yaml-config-parsing.md](../phase-01-common/015-yaml-config-parsing.md)
- Next: [138-forge-participants.md](138-forge-participants.md)
- Used by: All Forge operational specs
