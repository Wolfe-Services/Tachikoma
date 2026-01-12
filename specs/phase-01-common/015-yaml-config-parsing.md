# 015 - YAML Config Parsing

**Phase:** 1 - Core Common Crates
**Spec ID:** 015
**Status:** Complete
**Dependencies:** 014-config-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement YAML configuration file loading, parsing, and validation with support for environment variable expansion.

---

## Acceptance Criteria

- [x] Load config from `.tachikoma/config.yaml`
- [x] Merge with defaults for missing fields
- [x] Environment variable expansion (`${VAR}`)
- [x] Config validation
- [x] Error messages with line numbers

---

## Implementation Details

### 1. Config Loader (crates/tachikoma-common-config/src/loader.rs)

```rust
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

    #[error("invalid YAML at line {line}: {message}")]
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

    #[test]
    fn test_load_defaults_when_no_file() {
        let dir = tempdir().unwrap();
        let loader = ConfigLoader::new(dir.path());
        let config = loader.load().unwrap();
        assert_eq!(config.backend.brain, "claude");
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
}
```

### 2. Add Dependencies

```toml
[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_yaml = { workspace = true }
thiserror = { workspace = true }
regex = "1.10"

[dev-dependencies]
tempfile = "3.9"
```

---

## Testing Requirements

1. Missing config file returns defaults
2. Partial config merges with defaults
3. Environment variables are expanded
4. Invalid values produce clear errors

---

## Related Specs

- Depends on: [014-config-core-types.md](014-config-core-types.md)
- Next: [016-environment-variables.md](016-environment-variables.md)
