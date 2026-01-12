//! Configuration file loading and parsing.

use crate::types::TachikomaConfig;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Config loading errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("failed to read config: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[error("invalid YAML at line {}: {message}", line.map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string()))]
    ParseError { line: Option<usize>, message: String },

    #[error("validation error: {message}")]
    ValidationError { message: String },

    #[error("environment variable not found: {var}")]
    EnvVarNotFound { var: String },
}

/// Configuration loader.
pub struct ConfigLoader {
    base_path: PathBuf,
}

impl ConfigLoader {
    /// Create a loader for the given project directory.
    pub fn new(project_dir: impl AsRef<Path>) -> Self {
        Self {
            base_path: project_dir.as_ref().to_path_buf(),
        }
    }

    /// Load configuration from `.tachikoma/config.yaml`.
    pub fn load(&self) -> Result<TachikomaConfig, ConfigError> {
        let config_path = self.base_path.join(".tachikoma/config.yaml");

        if !config_path.exists() {
            // Return defaults if no config file
            return Ok(TachikomaConfig::default());
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let expanded = self.expand_env_vars(&contents)?;

        let config: TachikomaConfig = serde_yaml::from_str(&expanded)
            .map_err(|e| ConfigError::ParseError {
                line: e.location().map(|l| l.line()),
                message: e.to_string(),
            })?;

        self.validate(&config)?;
        Ok(config)
    }

    /// Expand environment variables in the form `${VAR}` or `${VAR:-default}`.
    fn expand_env_vars(&self, content: &str) -> Result<String, ConfigError> {
        let mut result = content.to_string();
        let re = regex::Regex::new(r"\$\{([^}:]+)(?::-([^}]*))?\}").unwrap();

        for cap in re.captures_iter(content) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = &cap[1];
            let default = cap.get(2).map(|m| m.as_str());

            let value = match std::env::var(var_name) {
                Ok(v) => v,
                Err(_) => match default {
                    Some(d) => d.to_string(),
                    None => {
                        return Err(ConfigError::EnvVarNotFound {
                            var: var_name.to_string(),
                        })
                    }
                },
            };

            result = result.replace(full_match, &value);
        }

        Ok(result)
    }

    /// Validate configuration values.
    fn validate(&self, config: &TachikomaConfig) -> Result<(), ConfigError> {
        // Validate redline threshold
        if config.loop_config.redline_threshold <= 0.0
            || config.loop_config.redline_threshold > 1.0
        {
            return Err(ConfigError::ValidationError {
                message: "redline_threshold must be between 0.0 and 1.0".to_string(),
            });
        }

        // Validate max iterations
        if config.loop_config.max_iterations == 0 {
            return Err(ConfigError::ValidationError {
                message: "max_iterations must be greater than 0".to_string(),
            });
        }

        // Validate forge rounds
        if config.forge.max_rounds == 0 {
            return Err(ConfigError::ValidationError {
                message: "forge.max_rounds must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Save configuration to file.
    pub fn save(&self, config: &TachikomaConfig) -> Result<(), ConfigError> {
        let config_dir = self.base_path.join(".tachikoma");
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.yaml");
        let yaml = serde_yaml::to_string(config)
            .map_err(|e| ConfigError::ParseError {
                line: None,
                message: e.to_string(),
            })?;

        std::fs::write(config_path, yaml)?;
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_load_defaults_when_no_file() {
        let dir = tempdir().unwrap();
        let loader = ConfigLoader::new(dir.path());
        let config = loader.load().unwrap();
        assert_eq!(config.backend.brain, "claude");
    }

    #[test]
    fn test_load_config_from_yaml_file() {
        let dir = tempdir().unwrap();
        let tachikoma_dir = dir.path().join(".tachikoma");
        fs::create_dir_all(&tachikoma_dir).unwrap();
        
        let config_content = r#"
backend:
  brain: gpt-4
  think_tank: gpt-4-turbo
loop_config:
  max_iterations: 50
  redline_threshold: 0.8
policies:
  auto_commit: true
forge:
  max_rounds: 3
"#;
        
        fs::write(tachikoma_dir.join("config.yaml"), config_content).unwrap();
        
        let loader = ConfigLoader::new(dir.path());
        let config = loader.load().unwrap();
        
        assert_eq!(config.backend.brain, "gpt-4");
        assert_eq!(config.backend.think_tank, "gpt-4-turbo");
        assert_eq!(config.loop_config.max_iterations, 50);
        assert_eq!(config.loop_config.redline_threshold, 0.8);
        assert!(config.policies.auto_commit);
        assert_eq!(config.forge.max_rounds, 3);
        
        // Check that unspecified values use defaults
        assert_eq!(config.loop_config.iteration_delay_ms, 1000);
        assert!(!config.policies.auto_push);
    }

    #[test]
    fn test_env_var_expansion() {
        std::env::set_var("TEST_VAR", "test_value");
        let loader = ConfigLoader::new(".");
        let result = loader.expand_env_vars("key: ${TEST_VAR}").unwrap();
        assert_eq!(result, "key: test_value");
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_env_var_default() {
        let loader = ConfigLoader::new(".");
        let result = loader.expand_env_vars("key: ${NONEXISTENT:-default}").unwrap();
        assert_eq!(result, "key: default");
    }

    #[test]
    fn test_env_var_missing_error() {
        let loader = ConfigLoader::new(".");
        let result = loader.expand_env_vars("key: ${MISSING_VAR}");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::EnvVarNotFound { var } => assert_eq!(var, "MISSING_VAR"),
            _ => panic!("Expected EnvVarNotFound error"),
        }
    }

    #[test]
    fn test_env_var_expansion_in_config() {
        std::env::set_var("BRAIN_MODEL", "custom-brain");
        std::env::set_var("MAX_ITER", "75");
        
        let dir = tempdir().unwrap();
        let tachikoma_dir = dir.path().join(".tachikoma");
        fs::create_dir_all(&tachikoma_dir).unwrap();
        
        let config_content = r#"
backend:
  brain: ${BRAIN_MODEL}
loop_config:
  max_iterations: ${MAX_ITER}
  redline_threshold: ${THRESHOLD:-0.9}
"#;
        
        fs::write(tachikoma_dir.join("config.yaml"), config_content).unwrap();
        
        let loader = ConfigLoader::new(dir.path());
        let config = loader.load().unwrap();
        
        assert_eq!(config.backend.brain, "custom-brain");
        assert_eq!(config.loop_config.max_iterations, 75);
        assert_eq!(config.loop_config.redline_threshold, 0.9);
        
        std::env::remove_var("BRAIN_MODEL");
        std::env::remove_var("MAX_ITER");
    }

    #[test]
    fn test_validation_errors() {
        let dir = tempdir().unwrap();
        let loader = ConfigLoader::new(dir.path());
        
        // Test invalid redline threshold
        let mut config = TachikomaConfig::default();
        config.loop_config.redline_threshold = 1.5;
        let result = loader.validate(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError { message } => {
                assert!(message.contains("redline_threshold"));
            }
            _ => panic!("Expected ValidationError"),
        }
        
        // Test zero max iterations
        let mut config = TachikomaConfig::default();
        config.loop_config.max_iterations = 0;
        let result = loader.validate(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError { message } => {
                assert!(message.contains("max_iterations"));
            }
            _ => panic!("Expected ValidationError"),
        }
        
        // Test zero forge rounds
        let mut config = TachikomaConfig::default();
        config.forge.max_rounds = 0;
        let result = loader.validate(&config);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ValidationError { message } => {
                assert!(message.contains("forge.max_rounds"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_parse_error_with_line_number() {
        let dir = tempdir().unwrap();
        let tachikoma_dir = dir.path().join(".tachikoma");
        fs::create_dir_all(&tachikoma_dir).unwrap();
        
        let bad_yaml = r#"
backend:
  brain: claude
  invalid_yaml: [unclosed
"#;
        
        fs::write(tachikoma_dir.join("config.yaml"), bad_yaml).unwrap();
        
        let loader = ConfigLoader::new(dir.path());
        let result = loader.load();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseError { line, message: _ } => {
                assert!(line.is_some());
            }
            _ => panic!("Expected ParseError with line number"),
        }
    }

    #[test]
    fn test_save_config() {
        let dir = tempdir().unwrap();
        let loader = ConfigLoader::new(dir.path());
        
        let mut config = TachikomaConfig::default();
        config.backend.brain = "custom-model".to_string();
        config.loop_config.max_iterations = 42;
        
        loader.save(&config).unwrap();
        
        // Verify the file exists
        let config_path = dir.path().join(".tachikoma/config.yaml");
        assert!(config_path.exists());
        
        // Verify we can load it back
        let loaded_config = loader.load().unwrap();
        assert_eq!(loaded_config.backend.brain, "custom-model");
        assert_eq!(loaded_config.loop_config.max_iterations, 42);
    }

    #[test]
    fn test_multiple_env_vars_in_single_value() {
        std::env::set_var("PREFIX", "test");
        std::env::set_var("SUFFIX", "model");
        
        let loader = ConfigLoader::new(".");
        let result = loader.expand_env_vars("brain: ${PREFIX}-${SUFFIX}").unwrap();
        assert_eq!(result, "brain: test-model");
        
        std::env::remove_var("PREFIX");
        std::env::remove_var("SUFFIX");
    }
}