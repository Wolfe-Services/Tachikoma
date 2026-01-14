# 392 - Feature Flag Definition

## Overview

Complete structure and management for feature flag definitions, including configuration, rules, and lifecycle management.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/definition.rs

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlagDefinitionError {
    #[error("Invalid flag key: {0}")]
    InvalidKey(String),
    #[error("Invalid percentage: must be between 0 and 100")]
    InvalidPercentage,
    #[error("Duplicate rule ID: {0}")]
    DuplicateRuleId(String),
    #[error("Invalid variant weights: must sum to 100")]
    InvalidVariantWeights,
    #[error("No variants defined for experiment")]
    NoVariants,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Complete feature flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDefinition {
    /// Unique flag identifier
    pub id: FlagId,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Current status
    pub status: FlagStatus,
    /// Flag value type
    pub value_type: FlagValueType,
    /// Default value when no rules match
    pub default_value: FlagValue,
    /// Environments where the flag is active
    pub environments: Vec<EnvironmentConfig>,
    /// Targeting rules (evaluated in priority order)
    pub rules: Vec<Rule>,
    /// Percentage rollout configuration
    pub rollout: Option<RolloutConfig>,
    /// A/B test configuration
    pub experiment: Option<ExperimentConfig>,
    /// User-level overrides
    pub user_overrides: HashMap<String, FlagValue>,
    /// Group-level overrides
    pub group_overrides: HashMap<String, FlagValue>,
    /// Flag metadata
    pub metadata: FlagMetadata,
    /// Schema version for migrations
    pub schema_version: u32,
}

/// Expected value type for a flag
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagValueType {
    Boolean,
    String,
    Number,
    Integer,
    Json,
    Variant,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name
    pub environment: Environment,
    /// Whether the flag is enabled in this environment
    pub enabled: bool,
    /// Environment-specific default value (overrides global default)
    pub default_value: Option<FlagValue>,
    /// Environment-specific rollout percentage
    pub rollout_percentage: Option<f64>,
}

impl FlagDefinition {
    /// Create a new boolean feature flag
    pub fn new_boolean(key: &str, name: &str, default: bool) -> Result<Self, FlagDefinitionError> {
        Self::validate_key(key)?;

        Ok(Self {
            id: FlagId::new(key),
            name: name.to_string(),
            description: String::new(),
            status: FlagStatus::Disabled,
            value_type: FlagValueType::Boolean,
            default_value: FlagValue::Boolean(default),
            environments: vec![],
            rules: vec![],
            rollout: None,
            experiment: None,
            user_overrides: HashMap::new(),
            group_overrides: HashMap::new(),
            metadata: FlagMetadata::new("system"),
            schema_version: 1,
        })
    }

    /// Create a new string/variant flag
    pub fn new_string(key: &str, name: &str, default: &str) -> Result<Self, FlagDefinitionError> {
        Self::validate_key(key)?;

        Ok(Self {
            id: FlagId::new(key),
            name: name.to_string(),
            description: String::new(),
            status: FlagStatus::Disabled,
            value_type: FlagValueType::String,
            default_value: FlagValue::String(default.to_string()),
            environments: vec![],
            rules: vec![],
            rollout: None,
            experiment: None,
            user_overrides: HashMap::new(),
            group_overrides: HashMap::new(),
            metadata: FlagMetadata::new("system"),
            schema_version: 1,
        })
    }

    /// Create a new multivariate experiment flag
    pub fn new_experiment(
        key: &str,
        name: &str,
        variants: Vec<ExperimentVariant>,
    ) -> Result<Self, FlagDefinitionError> {
        Self::validate_key(key)?;

        if variants.is_empty() {
            return Err(FlagDefinitionError::NoVariants);
        }

        let total_weight: f64 = variants.iter().map(|v| v.weight).sum();
        if (total_weight - 100.0).abs() > 0.01 {
            return Err(FlagDefinitionError::InvalidVariantWeights);
        }

        let default_value = variants.first()
            .map(|v| v.value.clone())
            .unwrap_or(FlagValue::Boolean(false));

        Ok(Self {
            id: FlagId::new(key),
            name: name.to_string(),
            description: String::new(),
            status: FlagStatus::Disabled,
            value_type: FlagValueType::Variant,
            default_value,
            environments: vec![],
            rules: vec![],
            rollout: None,
            experiment: Some(ExperimentConfig {
                name: name.to_string(),
                variants,
                bucket_by: "user_id".to_string(),
                track_exposure: true,
                goals: vec![],
            }),
            user_overrides: HashMap::new(),
            group_overrides: HashMap::new(),
            metadata: FlagMetadata::new("system"),
            schema_version: 1,
        })
    }

    fn validate_key(key: &str) -> Result<(), FlagDefinitionError> {
        if key.is_empty() {
            return Err(FlagDefinitionError::InvalidKey("Key cannot be empty".into()));
        }
        if key.len() > 256 {
            return Err(FlagDefinitionError::InvalidKey("Key too long (max 256 chars)".into()));
        }
        if !key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            return Err(FlagDefinitionError::InvalidKey(
                "Key can only contain alphanumeric characters, hyphens, underscores, and dots".into()
            ));
        }
        Ok(())
    }

    /// Set the description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Enable the flag
    pub fn enable(mut self) -> Self {
        self.status = FlagStatus::Active;
        self
    }

    /// Add a targeting rule
    pub fn add_rule(&mut self, rule: Rule) -> Result<(), FlagDefinitionError> {
        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(FlagDefinitionError::DuplicateRuleId(rule.id));
        }
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    /// Configure percentage rollout
    pub fn with_rollout(&mut self, percentage: f64) -> Result<(), FlagDefinitionError> {
        if !(0.0..=100.0).contains(&percentage) {
            return Err(FlagDefinitionError::InvalidPercentage);
        }
        self.rollout = Some(RolloutConfig {
            percentage,
            bucket_by: "user_id".to_string(),
            seed: None,
        });
        Ok(())
    }

    /// Add user override
    pub fn override_for_user(&mut self, user_id: &str, value: FlagValue) {
        self.user_overrides.insert(user_id.to_string(), value);
    }

    /// Add group override
    pub fn override_for_group(&mut self, group_id: &str, value: FlagValue) {
        self.group_overrides.insert(group_id.to_string(), value);
    }

    /// Enable for specific environment
    pub fn enable_for_environment(&mut self, env: Environment) {
        if let Some(config) = self.environments.iter_mut().find(|e| e.environment == env) {
            config.enabled = true;
        } else {
            self.environments.push(EnvironmentConfig {
                environment: env,
                enabled: true,
                default_value: None,
                rollout_percentage: None,
            });
        }
    }

    /// Validate the entire flag definition
    pub fn validate(&self) -> Result<(), FlagDefinitionError> {
        Self::validate_key(self.id.as_str())?;

        // Check value type consistency
        if !self.value_matches_type(&self.default_value) {
            return Err(FlagDefinitionError::ValidationError(
                "Default value doesn't match declared type".into()
            ));
        }

        // Check rule values
        for rule in &self.rules {
            if !self.value_matches_type(&rule.value) {
                return Err(FlagDefinitionError::ValidationError(
                    format!("Rule '{}' value doesn't match declared type", rule.id)
                ));
            }
        }

        // Check experiment variants
        if let Some(experiment) = &self.experiment {
            let total_weight: f64 = experiment.variants.iter().map(|v| v.weight).sum();
            if (total_weight - 100.0).abs() > 0.01 {
                return Err(FlagDefinitionError::InvalidVariantWeights);
            }
        }

        Ok(())
    }

    fn value_matches_type(&self, value: &FlagValue) -> bool {
        match (&self.value_type, value) {
            (FlagValueType::Boolean, FlagValue::Boolean(_)) => true,
            (FlagValueType::String, FlagValue::String(_)) => true,
            (FlagValueType::Number, FlagValue::Number(_)) => true,
            (FlagValueType::Integer, FlagValue::Integer(_)) => true,
            (FlagValueType::Json, FlagValue::Json(_)) => true,
            (FlagValueType::Variant, FlagValue::Variant(_)) => true,
            _ => false,
        }
    }

    /// Mark flag as deprecated
    pub fn deprecate(&mut self, sunset_date: DateTime<Utc>) {
        self.status = FlagStatus::Deprecated;
        self.metadata.sunset_date = Some(sunset_date);
    }

    /// Archive the flag
    pub fn archive(&mut self) {
        self.status = FlagStatus::Archived;
    }
}

impl FlagMetadata {
    pub fn new(created_by: &str) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            created_by: created_by.to_string(),
            updated_at: now,
            updated_by: created_by.to_string(),
            tags: vec![],
            owner: None,
            documentation_url: None,
            sunset_date: None,
        }
    }

    pub fn update(&mut self, updated_by: &str) {
        self.updated_at = Utc::now();
        self.updated_by = updated_by.to_string();
    }
}

/// Builder for creating flag definitions fluently
pub struct FlagDefinitionBuilder {
    definition: FlagDefinition,
}

impl FlagDefinitionBuilder {
    pub fn boolean(key: &str) -> Result<Self, FlagDefinitionError> {
        Ok(Self {
            definition: FlagDefinition::new_boolean(key, key, false)?,
        })
    }

    pub fn string(key: &str) -> Result<Self, FlagDefinitionError> {
        Ok(Self {
            definition: FlagDefinition::new_string(key, key, "")?,
        })
    }

    pub fn name(mut self, name: &str) -> Self {
        self.definition.name = name.to_string();
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.definition.description = desc.to_string();
        self
    }

    pub fn default_value(mut self, value: FlagValue) -> Self {
        self.definition.default_value = value;
        self
    }

    pub fn enabled(mut self) -> Self {
        self.definition.status = FlagStatus::Active;
        self
    }

    pub fn rollout(mut self, percentage: f64) -> Result<Self, FlagDefinitionError> {
        self.definition.with_rollout(percentage)?;
        Ok(self)
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.definition.metadata.tags.push(tag.to_string());
        self
    }

    pub fn owner(mut self, owner: &str) -> Self {
        self.definition.metadata.owner = Some(owner.to_string());
        self
    }

    pub fn build(self) -> Result<FlagDefinition, FlagDefinitionError> {
        self.definition.validate()?;
        Ok(self.definition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_boolean_flag() {
        let flag = FlagDefinition::new_boolean("test-feature", "Test Feature", false)
            .unwrap()
            .with_description("A test feature flag")
            .enable();

        assert_eq!(flag.id.as_str(), "test-feature");
        assert_eq!(flag.status, FlagStatus::Active);
        assert_eq!(flag.default_value.as_bool(), Some(false));
    }

    #[test]
    fn test_builder_pattern() {
        let flag = FlagDefinitionBuilder::boolean("new-feature")
            .unwrap()
            .name("New Feature")
            .description("A new feature")
            .default_value(FlagValue::Boolean(true))
            .enabled()
            .tag("frontend")
            .owner("team-a")
            .build()
            .unwrap();

        assert_eq!(flag.name, "New Feature");
        assert!(flag.metadata.tags.contains(&"frontend".to_string()));
    }

    #[test]
    fn test_invalid_key() {
        let result = FlagDefinition::new_boolean("invalid key!", "Test", false);
        assert!(matches!(result, Err(FlagDefinitionError::InvalidKey(_))));
    }

    #[test]
    fn test_add_rule() {
        let mut flag = FlagDefinition::new_boolean("test", "Test", false).unwrap();

        let rule = Rule {
            id: "rule-1".to_string(),
            name: "Beta Users".to_string(),
            priority: 100,
            conditions: vec![],
            value: FlagValue::Boolean(true),
            enabled: true,
        };

        flag.add_rule(rule.clone()).unwrap();
        assert_eq!(flag.rules.len(), 1);

        // Duplicate should fail
        let result = flag.add_rule(rule);
        assert!(matches!(result, Err(FlagDefinitionError::DuplicateRuleId(_))));
    }

    #[test]
    fn test_experiment_flag() {
        let variants = vec![
            ExperimentVariant {
                key: "control".to_string(),
                name: "Control".to_string(),
                weight: 50.0,
                value: FlagValue::Variant("control".to_string()),
            },
            ExperimentVariant {
                key: "treatment".to_string(),
                name: "Treatment".to_string(),
                weight: 50.0,
                value: FlagValue::Variant("treatment".to_string()),
            },
        ];

        let flag = FlagDefinition::new_experiment("ab-test", "A/B Test", variants).unwrap();
        assert!(flag.experiment.is_some());
        assert_eq!(flag.experiment.as_ref().unwrap().variants.len(), 2);
    }
}
```

## Related Specs

- 391-flags-core-types.md - Core type definitions
- 393-flag-storage.md - Persistence layer
- 394-flag-evaluation.md - Evaluation engine
