# 394 - Feature Flag Evaluation Engine

## Overview

Core evaluation engine for feature flags that determines flag values based on context, rules, percentage rollouts, and experiments.

## Rust Implementation

```rust
// crates/flags/src/evaluation.rs

use crate::definition::FlagDefinition;
use crate::storage::{FlagStorage, StorageError};
use crate::types::*;
use chrono::Utc;
use regex::Regex;
use semver::Version;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Invalid context: {0}")]
    InvalidContext(String),
    #[error("Rule evaluation error: {0}")]
    RuleError(String),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

/// Evaluation context containing user and environment information
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    /// User identifier
    pub user_id: Option<String>,
    /// Anonymous/device identifier
    pub anonymous_id: Option<String>,
    /// User's groups/segments
    pub groups: Vec<String>,
    /// User properties (plan, role, etc.)
    pub user_properties: Properties,
    /// Request/session properties (ip, country, etc.)
    pub request_properties: Properties,
    /// Current environment
    pub environment: Environment,
    /// Custom properties
    pub custom: Properties,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_anonymous(mut self, id: &str) -> Self {
        self.anonymous_id = Some(id.to_string());
        self
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.groups.push(group.to_string());
        self
    }

    pub fn with_property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.user_properties.insert(key.to_string(), value.into());
        self
    }

    pub fn with_environment(mut self, env: Environment) -> Self {
        self.environment = env;
        self
    }

    /// Get the identifier to use for bucketing
    pub fn bucket_key(&self, bucket_by: &str) -> Option<String> {
        match bucket_by {
            "user_id" => self.user_id.clone().or_else(|| self.anonymous_id.clone()),
            "anonymous_id" => self.anonymous_id.clone(),
            key => self.user_properties.get(key)
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }
    }

    /// Get a property value from the context
    pub fn get_property(&self, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let (namespace, key) = if parts.len() == 1 {
            ("user", parts[0])
        } else {
            (parts[0], parts[1])
        };

        let props = match namespace {
            "user" => &self.user_properties,
            "request" => &self.request_properties,
            "custom" => &self.custom,
            _ => return None,
        };

        props.get(key).cloned()
    }
}

/// Feature flag evaluator
pub struct FlagEvaluator {
    storage: Arc<dyn FlagStorage>,
    cache: Option<Arc<dyn FlagCache>>,
}

/// Cache trait for flag values
#[async_trait::async_trait]
pub trait FlagCache: Send + Sync {
    async fn get(&self, flag_id: &FlagId) -> Option<FlagDefinition>;
    async fn set(&self, flag_id: &FlagId, definition: FlagDefinition);
    async fn invalidate(&self, flag_id: &FlagId);
    async fn invalidate_all(&self);
}

impl FlagEvaluator {
    pub fn new(storage: Arc<dyn FlagStorage>) -> Self {
        Self {
            storage,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<dyn FlagCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Evaluate a single flag
    pub async fn evaluate(
        &self,
        flag_id: &FlagId,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, EvaluationError> {
        let start = Instant::now();

        // Try to get from cache first
        let definition = if let Some(cache) = &self.cache {
            if let Some(def) = cache.get(flag_id).await {
                def
            } else {
                let stored = self.storage.get(flag_id).await?;
                if let Some(stored) = stored {
                    cache.set(flag_id, stored.definition.clone()).await;
                    stored.definition
                } else {
                    return Ok(EvaluationResult::not_found(flag_id.clone()));
                }
            }
        } else {
            match self.storage.get(flag_id).await? {
                Some(stored) => stored.definition,
                None => return Ok(EvaluationResult::not_found(flag_id.clone())),
            }
        };

        let result = self.evaluate_definition(&definition, context).await?;

        Ok(EvaluationResult {
            evaluation_time_us: start.elapsed().as_micros() as u64,
            ..result
        })
    }

    /// Evaluate a flag definition against a context
    async fn evaluate_definition(
        &self,
        definition: &FlagDefinition,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult, EvaluationError> {
        let flag_id = definition.id.clone();

        // Check if flag is disabled
        if definition.status == FlagStatus::Disabled || definition.status == FlagStatus::Archived {
            return Ok(EvaluationResult::disabled(flag_id, definition.default_value.clone()));
        }

        // Check environment
        if !self.is_enabled_for_environment(definition, &context.environment) {
            return Ok(EvaluationResult {
                flag_id,
                value: definition.default_value.clone(),
                found: true,
                reason: EvaluationReason::Disabled,
                matched_rule: None,
                evaluation_time_us: 0,
                evaluated_at: Utc::now(),
                in_experiment: false,
                experiment_variant: None,
            });
        }

        // Check user overrides
        if let Some(user_id) = &context.user_id {
            if let Some(value) = definition.user_overrides.get(user_id) {
                return Ok(EvaluationResult {
                    flag_id,
                    value: value.clone(),
                    found: true,
                    reason: EvaluationReason::Override,
                    matched_rule: Some(format!("user:{}", user_id)),
                    evaluation_time_us: 0,
                    evaluated_at: Utc::now(),
                    in_experiment: false,
                    experiment_variant: None,
                });
            }
        }

        // Check group overrides
        for group in &context.groups {
            if let Some(value) = definition.group_overrides.get(group) {
                return Ok(EvaluationResult {
                    flag_id,
                    value: value.clone(),
                    found: true,
                    reason: EvaluationReason::GroupTargeted,
                    matched_rule: Some(format!("group:{}", group)),
                    evaluation_time_us: 0,
                    evaluated_at: Utc::now(),
                    in_experiment: false,
                    experiment_variant: None,
                });
            }
        }

        // Evaluate rules in priority order
        for rule in &definition.rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_rule(rule, context)? {
                return Ok(EvaluationResult {
                    flag_id,
                    value: rule.value.clone(),
                    found: true,
                    reason: EvaluationReason::RuleMatched,
                    matched_rule: Some(rule.id.clone()),
                    evaluation_time_us: 0,
                    evaluated_at: Utc::now(),
                    in_experiment: false,
                    experiment_variant: None,
                });
            }
        }

        // Handle experiment/A/B test
        if let Some(experiment) = &definition.experiment {
            if let Some(bucket_key) = context.bucket_key(&experiment.bucket_by) {
                let variant = self.select_variant(&bucket_key, &flag_id, &experiment.variants);
                return Ok(EvaluationResult {
                    flag_id,
                    value: variant.value.clone(),
                    found: true,
                    reason: EvaluationReason::Experiment,
                    matched_rule: None,
                    evaluation_time_us: 0,
                    evaluated_at: Utc::now(),
                    in_experiment: true,
                    experiment_variant: Some(variant.key.clone()),
                });
            }
        }

        // Handle percentage rollout
        if let Some(rollout) = &definition.rollout {
            if let Some(bucket_key) = context.bucket_key(&rollout.bucket_by) {
                let in_rollout = self.is_in_rollout(&bucket_key, &flag_id, rollout);
                if in_rollout {
                    // Return enabled value for boolean flags, default otherwise
                    let value = match &definition.default_value {
                        FlagValue::Boolean(_) => FlagValue::Boolean(true),
                        _ => definition.default_value.clone(),
                    };

                    return Ok(EvaluationResult {
                        flag_id,
                        value,
                        found: true,
                        reason: EvaluationReason::PercentageRollout,
                        matched_rule: None,
                        evaluation_time_us: 0,
                        evaluated_at: Utc::now(),
                        in_experiment: false,
                        experiment_variant: None,
                    });
                }
            }
        }

        // Return default value
        Ok(EvaluationResult {
            flag_id,
            value: definition.default_value.clone(),
            found: true,
            reason: EvaluationReason::Default,
            matched_rule: None,
            evaluation_time_us: 0,
            evaluated_at: Utc::now(),
            in_experiment: false,
            experiment_variant: None,
        })
    }

    fn is_enabled_for_environment(&self, definition: &FlagDefinition, env: &Environment) -> bool {
        if definition.environments.is_empty() {
            return true; // Enabled everywhere if no environments specified
        }

        definition.environments.iter()
            .any(|e| &e.environment == env && e.enabled)
    }

    fn evaluate_rule(&self, rule: &Rule, context: &EvaluationContext) -> Result<bool, EvaluationError> {
        // All conditions must match (AND logic)
        for condition in &rule.conditions {
            let matches = self.evaluate_condition(condition, context)?;
            let result = if condition.negate { !matches } else { matches };
            if !result {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn evaluate_condition(&self, condition: &Condition, context: &EvaluationContext) -> Result<bool, EvaluationError> {
        let property_value = context.get_property(&condition.property);

        match &condition.operator {
            Operator::IsSet => Ok(property_value.is_some()),
            Operator::IsNotSet => Ok(property_value.is_none()),
            _ => {
                let prop = match property_value {
                    Some(v) => v,
                    None => return Ok(false),
                };

                self.compare_values(&prop, &condition.operator, &condition.value)
            }
        }
    }

    fn compare_values(
        &self,
        actual: &serde_json::Value,
        operator: &Operator,
        expected: &ConditionValue,
    ) -> Result<bool, EvaluationError> {
        match operator {
            Operator::Equals => Ok(self.values_equal(actual, expected)),
            Operator::NotEquals => Ok(!self.values_equal(actual, expected)),

            Operator::Contains => {
                if let (Some(actual_str), ConditionValue::String(expected_str)) = (actual.as_str(), expected) {
                    Ok(actual_str.contains(expected_str))
                } else {
                    Ok(false)
                }
            }

            Operator::NotContains => {
                if let (Some(actual_str), ConditionValue::String(expected_str)) = (actual.as_str(), expected) {
                    Ok(!actual_str.contains(expected_str))
                } else {
                    Ok(true)
                }
            }

            Operator::StartsWith => {
                if let (Some(actual_str), ConditionValue::String(expected_str)) = (actual.as_str(), expected) {
                    Ok(actual_str.starts_with(expected_str))
                } else {
                    Ok(false)
                }
            }

            Operator::EndsWith => {
                if let (Some(actual_str), ConditionValue::String(expected_str)) = (actual.as_str(), expected) {
                    Ok(actual_str.ends_with(expected_str))
                } else {
                    Ok(false)
                }
            }

            Operator::Matches => {
                if let (Some(actual_str), ConditionValue::String(pattern)) = (actual.as_str(), expected) {
                    let regex = Regex::new(pattern)?;
                    Ok(regex.is_match(actual_str))
                } else {
                    Ok(false)
                }
            }

            Operator::GreaterThan | Operator::GreaterThanOrEqual |
            Operator::LessThan | Operator::LessThanOrEqual => {
                self.compare_numeric(actual, operator, expected)
            }

            Operator::In => {
                if let ConditionValue::List(list) = expected {
                    if let Some(actual_str) = actual.as_str() {
                        Ok(list.contains(&actual_str.to_string()))
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }

            Operator::NotIn => {
                if let ConditionValue::List(list) = expected {
                    if let Some(actual_str) = actual.as_str() {
                        Ok(!list.contains(&actual_str.to_string()))
                    } else {
                        Ok(true)
                    }
                } else {
                    Ok(true)
                }
            }

            Operator::SemverEquals | Operator::SemverGreaterThan | Operator::SemverLessThan => {
                self.compare_semver(actual, operator, expected)
            }

            _ => Ok(false),
        }
    }

    fn values_equal(&self, actual: &serde_json::Value, expected: &ConditionValue) -> bool {
        match expected {
            ConditionValue::String(s) => actual.as_str() == Some(s),
            ConditionValue::Number(n) => actual.as_f64() == Some(*n),
            ConditionValue::Boolean(b) => actual.as_bool() == Some(*b),
            ConditionValue::Null => actual.is_null(),
            ConditionValue::List(_) => false,
        }
    }

    fn compare_numeric(
        &self,
        actual: &serde_json::Value,
        operator: &Operator,
        expected: &ConditionValue,
    ) -> Result<bool, EvaluationError> {
        let actual_num = actual.as_f64();
        let expected_num = match expected {
            ConditionValue::Number(n) => Some(*n),
            _ => None,
        };

        match (actual_num, expected_num) {
            (Some(a), Some(e)) => match operator {
                Operator::GreaterThan => Ok(a > e),
                Operator::GreaterThanOrEqual => Ok(a >= e),
                Operator::LessThan => Ok(a < e),
                Operator::LessThanOrEqual => Ok(a <= e),
                _ => Ok(false),
            },
            _ => Ok(false),
        }
    }

    fn compare_semver(
        &self,
        actual: &serde_json::Value,
        operator: &Operator,
        expected: &ConditionValue,
    ) -> Result<bool, EvaluationError> {
        let actual_ver = actual.as_str()
            .and_then(|s| Version::parse(s).ok());
        let expected_ver = match expected {
            ConditionValue::String(s) => Version::parse(s).ok(),
            _ => None,
        };

        match (actual_ver, expected_ver) {
            (Some(a), Some(e)) => match operator {
                Operator::SemverEquals => Ok(a == e),
                Operator::SemverGreaterThan => Ok(a > e),
                Operator::SemverLessThan => Ok(a < e),
                _ => Ok(false),
            },
            _ => Ok(false),
        }
    }

    /// Deterministic hash for percentage rollout
    fn is_in_rollout(&self, bucket_key: &str, flag_id: &FlagId, rollout: &RolloutConfig) -> bool {
        let hash_input = format!("{}:{}", flag_id.as_str(), bucket_key);
        let hash_value = self.hash_to_percentage(&hash_input, rollout.seed);
        hash_value <= rollout.percentage
    }

    /// Select variant based on consistent hashing
    fn select_variant<'a>(
        &self,
        bucket_key: &str,
        flag_id: &FlagId,
        variants: &'a [ExperimentVariant],
    ) -> &'a ExperimentVariant {
        let hash_input = format!("{}:{}", flag_id.as_str(), bucket_key);
        let hash_value = self.hash_to_percentage(&hash_input, None);

        let mut cumulative = 0.0;
        for variant in variants {
            cumulative += variant.weight;
            if hash_value <= cumulative {
                return variant;
            }
        }

        // Fallback to last variant
        variants.last().unwrap()
    }

    /// Hash a string to a percentage (0-100)
    fn hash_to_percentage(&self, input: &str, seed: Option<u64>) -> f64 {
        let mut hasher = Sha256::new();
        if let Some(s) = seed {
            hasher.update(s.to_le_bytes());
        }
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        // Use first 8 bytes as u64
        let bytes: [u8; 8] = result[..8].try_into().unwrap();
        let hash_int = u64::from_le_bytes(bytes);

        // Convert to percentage
        (hash_int as f64 / u64::MAX as f64) * 100.0
    }

    /// Evaluate multiple flags at once (batch evaluation)
    pub async fn evaluate_all(
        &self,
        flag_ids: &[FlagId],
        context: &EvaluationContext,
    ) -> Result<HashMap<FlagId, EvaluationResult>, EvaluationError> {
        let mut results = HashMap::new();

        // Batch fetch from storage
        let flags = self.storage.get_many(flag_ids).await?;

        for flag_id in flag_ids {
            let result = if let Some(stored) = flags.get(flag_id) {
                self.evaluate_definition(&stored.definition, context).await?
            } else {
                EvaluationResult::not_found(flag_id.clone())
            };
            results.insert(flag_id.clone(), result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    #[tokio::test]
    async fn test_evaluate_boolean_flag() {
        let storage = Arc::new(InMemoryStorage::new());
        let evaluator = FlagEvaluator::new(storage.clone());

        let mut flag = FlagDefinition::new_boolean("test-feature", "Test", true).unwrap();
        flag.status = FlagStatus::Active;
        storage.create(flag).await.unwrap();

        let context = EvaluationContext::new().with_user("user-123");
        let result = evaluator.evaluate(&FlagId::new("test-feature"), &context).await.unwrap();

        assert!(result.found);
        assert_eq!(result.value.as_bool(), Some(true));
    }

    #[tokio::test]
    async fn test_user_override() {
        let storage = Arc::new(InMemoryStorage::new());
        let evaluator = FlagEvaluator::new(storage.clone());

        let mut flag = FlagDefinition::new_boolean("test-feature", "Test", false).unwrap();
        flag.status = FlagStatus::Active;
        flag.user_overrides.insert("special-user".to_string(), FlagValue::Boolean(true));
        storage.create(flag).await.unwrap();

        let context = EvaluationContext::new().with_user("special-user");
        let result = evaluator.evaluate(&FlagId::new("test-feature"), &context).await.unwrap();

        assert_eq!(result.value.as_bool(), Some(true));
        assert_eq!(result.reason, EvaluationReason::Override);
    }

    #[tokio::test]
    async fn test_rule_evaluation() {
        let storage = Arc::new(InMemoryStorage::new());
        let evaluator = FlagEvaluator::new(storage.clone());

        let mut flag = FlagDefinition::new_boolean("beta-feature", "Beta", false).unwrap();
        flag.status = FlagStatus::Active;
        flag.rules.push(Rule {
            id: "beta-users".to_string(),
            name: "Beta Users".to_string(),
            priority: 100,
            conditions: vec![Condition {
                property: "user.plan".to_string(),
                operator: Operator::Equals,
                value: ConditionValue::String("beta".to_string()),
                negate: false,
            }],
            value: FlagValue::Boolean(true),
            enabled: true,
        });
        storage.create(flag).await.unwrap();

        let context = EvaluationContext::new()
            .with_user("user-123")
            .with_property("plan", "beta");

        let result = evaluator.evaluate(&FlagId::new("beta-feature"), &context).await.unwrap();

        assert_eq!(result.value.as_bool(), Some(true));
        assert_eq!(result.reason, EvaluationReason::RuleMatched);
    }

    #[test]
    fn test_hash_consistency() {
        let evaluator = FlagEvaluator::new(Arc::new(InMemoryStorage::new()));

        // Same input should always produce same output
        let hash1 = evaluator.hash_to_percentage("test:user123", None);
        let hash2 = evaluator.hash_to_percentage("test:user123", None);
        assert_eq!(hash1, hash2);

        // Different input should (usually) produce different output
        let hash3 = evaluator.hash_to_percentage("test:user456", None);
        assert_ne!(hash1, hash3);
    }
}
```

## Related Specs

- 391-flags-core-types.md - Core types
- 395-flag-context.md - Evaluation context details
- 396-percentage-rollout.md - Rollout implementation
- 399-ab-testing.md - A/B testing
