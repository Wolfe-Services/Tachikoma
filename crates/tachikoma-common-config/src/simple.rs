//! Simple configuration mode with progressive disclosure.
//!
//! Provides a minimal 5-option configuration that auto-expands to full config.
//! Power users can "upgrade" to full config when they need more control.

use serde::{Deserialize, Serialize};
use crate::{BackendConfig, ForgeConfig, LoopConfig, PolicyConfig, StopCondition, TachikomaConfig};

/// Configuration mode - simple or full.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigMode {
    /// Simple mode with ~5 essential options.
    #[default]
    Simple,
    /// Full mode with all configuration options.
    Full,
}

/// Simple configuration with minimal options.
///
/// This provides a beginner-friendly configuration with just the essentials.
/// It auto-expands to a full `TachikomaConfig` at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimpleConfig {
    /// Configuration mode marker.
    pub config_mode: ConfigMode,
    
    /// The AI agent that does the coding (claude, opencode, ollama).
    pub agent: String,
    
    /// How long to run before stopping (max iterations).
    pub max_iterations: u32,
    
    /// Watch and approve each step? (true = safer, false = autonomous).
    pub attended: bool,
    
    /// Commit changes automatically after each spec?
    pub auto_commit: bool,
    
    /// Push commits automatically?
    pub auto_push: bool,
}

impl Default for SimpleConfig {
    fn default() -> Self {
        Self {
            config_mode: ConfigMode::Simple,
            agent: "claude".to_string(),
            max_iterations: 50,
            attended: true,
            auto_commit: true,
            auto_push: false,
        }
    }
}

impl SimpleConfig {
    /// Expand simple config to full config with smart defaults.
    pub fn expand(&self) -> TachikomaConfig {
        TachikomaConfig {
            backend: BackendConfig {
                brain: self.agent.clone(),
                think_tank: "o3".to_string(),
                api_keys: Default::default(),
                endpoints: Default::default(),
            },
            loop_config: LoopConfig {
                max_iterations: self.max_iterations,
                stop_on: vec![
                    StopCondition::Redline,
                    StopCondition::TestFailStreak(3),
                    StopCondition::NoProgress(5),
                ],
                redline_threshold: 0.75,
                iteration_delay_ms: 100,
            },
            policies: PolicyConfig {
                deploy_requires_tests: true,
                attended_by_default: self.attended,
                auto_commit: self.auto_commit,
                auto_push: self.auto_push,
                require_spec: true,
            },
            forge: ForgeConfig::default(),
        }
    }
    
    /// Create a simple config from a full config.
    ///
    /// This extracts the essential options, discarding advanced settings.
    pub fn from_full(full: &TachikomaConfig) -> Self {
        Self {
            config_mode: ConfigMode::Simple,
            agent: full.backend.brain.clone(),
            max_iterations: full.loop_config.max_iterations,
            attended: full.policies.attended_by_default,
            auto_commit: full.policies.auto_commit,
            auto_push: full.policies.auto_push,
        }
    }
    
    /// Generate a simple config YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let yaml = serde_yaml::to_string(self)?;
        // Add helpful comments
        let commented = format!(
            "# Tachikoma Simple Configuration\n\
             # For more options, run: tachikoma config upgrade\n\
             \n\
             {yaml}"
        );
        Ok(commented)
    }
}

/// Full configuration wrapper that includes mode marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullConfig {
    /// Configuration mode marker.
    pub config_mode: ConfigMode,
    
    /// Backend configuration.
    pub backend: BackendConfig,
    
    /// Loop runner configuration.
    pub loop_config: LoopConfig,
    
    /// Policy configuration.
    pub policies: PolicyConfig,
    
    /// Forge configuration.
    pub forge: ForgeConfig,
}

impl FullConfig {
    /// Create from a TachikomaConfig.
    pub fn from_config(config: TachikomaConfig) -> Self {
        Self {
            config_mode: ConfigMode::Full,
            backend: config.backend,
            loop_config: config.loop_config,
            policies: config.policies,
            forge: config.forge,
        }
    }
    
    /// Convert to TachikomaConfig.
    pub fn into_config(self) -> TachikomaConfig {
        TachikomaConfig {
            backend: self.backend,
            loop_config: self.loop_config,
            policies: self.policies,
            forge: self.forge,
        }
    }
    
    /// Generate a full config YAML string with comments.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let yaml = serde_yaml::to_string(self)?;
        let commented = format!(
            "# Tachikoma Full Configuration\n\
             # Generated from simple config upgrade\n\
             \n\
             {yaml}"
        );
        Ok(commented)
    }
}

/// Upgrade result with warnings about lost simple mode.
#[derive(Debug, Clone)]
pub struct UpgradeResult {
    /// The upgraded full configuration.
    pub config: FullConfig,
    /// Information messages about the upgrade.
    pub messages: Vec<String>,
}

impl SimpleConfig {
    /// Upgrade to full config with informational messages.
    pub fn upgrade(&self) -> UpgradeResult {
        let full = FullConfig::from_config(self.expand());
        
        let messages = vec![
            "Upgraded to full configuration mode.".to_string(),
            "You now have access to all advanced options.".to_string(),
            "See docs/guides/configuration.md for details.".to_string(),
        ];
        
        UpgradeResult {
            config: full,
            messages,
        }
    }
}

/// Configuration that can be either simple or full.
#[derive(Debug, Clone)]
pub enum AnyConfig {
    Simple(SimpleConfig),
    Full(FullConfig),
}

impl AnyConfig {
    /// Parse from YAML, detecting the mode.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        // First, check if config_mode is present to determine format
        #[derive(Deserialize)]
        struct ModeCheck {
            #[serde(default)]
            config_mode: Option<ConfigMode>,
            // Check for full config markers
            backend: Option<serde_yaml::Value>,
        }
        
        let check: ModeCheck = serde_yaml::from_str(yaml)?;
        
        match (check.config_mode, check.backend) {
            (Some(ConfigMode::Full), _) | (None, Some(_)) => {
                // Has backend section or explicit full mode - parse as full
                let full: FullConfig = serde_yaml::from_str(yaml)?;
                Ok(AnyConfig::Full(full))
            }
            _ => {
                // Otherwise parse as simple
                let simple: SimpleConfig = serde_yaml::from_str(yaml)?;
                Ok(AnyConfig::Simple(simple))
            }
        }
    }
    
    /// Get the expanded TachikomaConfig regardless of mode.
    pub fn into_config(self) -> TachikomaConfig {
        match self {
            AnyConfig::Simple(simple) => simple.expand(),
            AnyConfig::Full(full) => full.into_config(),
        }
    }
    
    /// Check if this is simple mode.
    pub fn is_simple(&self) -> bool {
        matches!(self, AnyConfig::Simple(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_config_defaults() {
        let config = SimpleConfig::default();
        
        assert_eq!(config.agent, "claude");
        assert_eq!(config.max_iterations, 50);
        assert!(config.attended);
        assert!(config.auto_commit);
        assert!(!config.auto_push);
    }
    
    #[test]
    fn test_simple_expand() {
        let simple = SimpleConfig {
            config_mode: ConfigMode::Simple,
            agent: "ollama".to_string(),
            max_iterations: 100,
            attended: false,
            auto_commit: false,
            auto_push: false,
        };
        
        let full = simple.expand();
        
        assert_eq!(full.backend.brain, "ollama");
        assert_eq!(full.loop_config.max_iterations, 100);
        assert!(!full.policies.attended_by_default);
        assert!(!full.policies.auto_commit);
    }
    
    #[test]
    fn test_simple_yaml_parse() {
        let yaml = r#"
config_mode: simple
agent: claude
max_iterations: 50
attended: true
auto_commit: true
"#;
        
        let config: SimpleConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent, "claude");
        assert_eq!(config.max_iterations, 50);
    }
    
    #[test]
    fn test_any_config_detection() {
        let simple_yaml = r#"
config_mode: simple
agent: claude
max_iterations: 50
"#;
        
        let any = AnyConfig::from_yaml(simple_yaml).unwrap();
        assert!(any.is_simple());
        
        let full_yaml = r#"
config_mode: full
backend:
  brain: claude
  think_tank: o3
loop_config:
  max_iterations: 100
policies:
  auto_commit: true
forge:
  participants: [claude, gemini]
"#;
        
        let any = AnyConfig::from_yaml(full_yaml).unwrap();
        assert!(!any.is_simple());
    }
    
    #[test]
    fn test_upgrade() {
        let simple = SimpleConfig::default();
        let result = simple.upgrade();
        
        assert_eq!(result.config.config_mode, ConfigMode::Full);
        assert!(!result.messages.is_empty());
    }
}
