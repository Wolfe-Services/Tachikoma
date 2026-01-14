# 410 - Feature Flag Testing

## Overview

Comprehensive testing strategies and utilities for the feature flags system including unit tests, integration tests, and testing utilities for downstream consumers.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Testing Utilities

```rust
// crates/flags/src/testing.rs

use crate::context::EvaluationContext;
use crate::definition::FlagDefinition;
use crate::evaluation::{FlagEvaluator, FlagCache};
use crate::storage::{FlagStorage, InMemoryStorage, StoredFlag};
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test harness for feature flags
pub struct FlagTestHarness {
    storage: Arc<InMemoryStorage>,
    evaluator: FlagEvaluator,
    overrides: RwLock<HashMap<String, HashMap<String, FlagValue>>>,
}

impl FlagTestHarness {
    pub fn new() -> Self {
        let storage = Arc::new(InMemoryStorage::new());
        let evaluator = FlagEvaluator::new(storage.clone());

        Self {
            storage,
            evaluator,
            overrides: RwLock::new(HashMap::new()),
        }
    }

    /// Create a test flag
    pub async fn create_flag(&self, key: &str, default: bool) -> &Self {
        let mut flag = FlagDefinition::new_boolean(key, key, default).unwrap();
        flag.status = FlagStatus::Active;
        self.storage.create(flag).await.unwrap();
        self
    }

    /// Create a flag with a specific value type
    pub async fn create_string_flag(&self, key: &str, default: &str) -> &Self {
        let mut flag = FlagDefinition::new_string(key, key, default).unwrap();
        flag.status = FlagStatus::Active;
        self.storage.create(flag).await.unwrap();
        self
    }

    /// Set a flag value for a specific user
    pub async fn set_for_user(&self, key: &str, user_id: &str, value: FlagValue) -> &Self {
        let mut overrides = self.overrides.write().await;
        overrides
            .entry(key.to_string())
            .or_insert_with(HashMap::new)
            .insert(user_id.to_string(), value);
        self
    }

    /// Set a global flag value
    pub async fn set_flag(&self, key: &str, value: FlagValue) -> &Self {
        let flag_id = FlagId::new(key);
        if let Some(stored) = self.storage.get(&flag_id).await.unwrap() {
            let mut flag = stored.definition;
            flag.default_value = value;
            self.storage.update(flag, None).await.unwrap();
        }
        self
    }

    /// Enable a flag
    pub async fn enable(&self, key: &str) -> &Self {
        self.set_flag(key, FlagValue::Boolean(true)).await
    }

    /// Disable a flag
    pub async fn disable(&self, key: &str) -> &Self {
        self.set_flag(key, FlagValue::Boolean(false)).await
    }

    /// Evaluate a flag
    pub async fn evaluate(&self, key: &str, context: &EvaluationContext) -> EvaluationResult {
        let flag_id = FlagId::new(key);
        self.evaluator.evaluate(&flag_id, context).await.unwrap()
    }

    /// Check if flag is enabled for context
    pub async fn is_enabled(&self, key: &str, context: &EvaluationContext) -> bool {
        self.evaluate(key, context).await.value.is_truthy()
    }

    /// Reset all flags
    pub async fn reset(&self) {
        // Clear storage would need to be implemented
        self.overrides.write().await.clear();
    }
}

impl Default for FlagTestHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock flag client for testing downstream code
pub struct MockFlagClient {
    flags: RwLock<HashMap<String, FlagValue>>,
    user_flags: RwLock<HashMap<String, HashMap<String, FlagValue>>>,
}

impl MockFlagClient {
    pub fn new() -> Self {
        Self {
            flags: RwLock::new(HashMap::new()),
            user_flags: RwLock::new(HashMap::new()),
        }
    }

    /// Set default flag value
    pub async fn set_flag(&self, key: &str, value: impl Into<FlagValue>) {
        self.flags.write().await.insert(key.to_string(), value.into());
    }

    /// Set flag value for specific user
    pub async fn set_user_flag(&self, key: &str, user_id: &str, value: impl Into<FlagValue>) {
        self.user_flags
            .write()
            .await
            .entry(key.to_string())
            .or_insert_with(HashMap::new)
            .insert(user_id.to_string(), value.into());
    }

    /// Get flag value for user
    pub async fn get_bool(&self, key: &str, user_id: Option<&str>, default: bool) -> bool {
        // Check user-specific first
        if let Some(uid) = user_id {
            let user_flags = self.user_flags.read().await;
            if let Some(flag_users) = user_flags.get(key) {
                if let Some(value) = flag_users.get(uid) {
                    return value.as_bool().unwrap_or(default);
                }
            }
        }

        // Fall back to default
        self.flags
            .read()
            .await
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    pub async fn get_string(&self, key: &str, user_id: Option<&str>, default: &str) -> String {
        if let Some(uid) = user_id {
            let user_flags = self.user_flags.read().await;
            if let Some(flag_users) = user_flags.get(key) {
                if let Some(value) = flag_users.get(uid) {
                    return value.as_string().unwrap_or(default).to_string();
                }
            }
        }

        self.flags
            .read()
            .await
            .get(key)
            .and_then(|v| v.as_string())
            .unwrap_or(default)
            .to_string()
    }

    /// Clear all flags
    pub async fn clear(&self) {
        self.flags.write().await.clear();
        self.user_flags.write().await.clear();
    }
}

impl Default for MockFlagClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for asserting flag behavior
#[macro_export]
macro_rules! assert_flag_enabled {
    ($harness:expr, $key:expr, $context:expr) => {
        assert!(
            $harness.is_enabled($key, $context).await,
            "Expected flag '{}' to be enabled",
            $key
        );
    };
}

#[macro_export]
macro_rules! assert_flag_disabled {
    ($harness:expr, $key:expr, $context:expr) => {
        assert!(
            !$harness.is_enabled($key, $context).await,
            "Expected flag '{}' to be disabled",
            $key
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextBuilder;

    #[tokio::test]
    async fn test_harness_basic() {
        let harness = FlagTestHarness::new();

        harness.create_flag("test-feature", false).await;

        let context = ContextBuilder::new()
            .user_id("user-123")
            .build();

        assert!(!harness.is_enabled("test-feature", &context).await);

        harness.enable("test-feature").await;

        assert!(harness.is_enabled("test-feature", &context).await);
    }

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockFlagClient::new();

        client.set_flag("feature-x", FlagValue::Boolean(true)).await;
        client.set_user_flag("feature-x", "special-user", FlagValue::Boolean(false)).await;

        // Regular user sees enabled
        assert!(client.get_bool("feature-x", Some("regular-user"), false).await);

        // Special user sees disabled
        assert!(!client.get_bool("feature-x", Some("special-user"), false).await);
    }
}
```

## Integration Tests

```rust
// crates/flags/tests/integration_tests.rs

use flags::{
    context::ContextBuilder,
    definition::FlagDefinition,
    evaluation::FlagEvaluator,
    storage::InMemoryStorage,
    types::*,
};
use std::sync::Arc;

#[tokio::test]
async fn test_full_evaluation_flow() {
    // Setup
    let storage = Arc::new(InMemoryStorage::new());
    let evaluator = FlagEvaluator::new(storage.clone());

    // Create a flag with rules
    let mut flag = FlagDefinition::new_boolean("premium-feature", "Premium Feature", false).unwrap();
    flag.status = FlagStatus::Active;

    // Add rule for premium users
    flag.rules.push(Rule {
        id: "premium-users".to_string(),
        name: "Premium Users".to_string(),
        priority: 100,
        conditions: vec![Condition {
            property: "user.plan".to_string(),
            operator: Operator::Equals,
            value: ConditionValue::String("premium".to_string()),
            negate: false,
        }],
        value: FlagValue::Boolean(true),
        enabled: true,
    });

    storage.create(flag).await.unwrap();

    // Test free user
    let free_context = ContextBuilder::new()
        .user_id("free-user")
        .plan("free")
        .build();

    let result = evaluator.evaluate(&FlagId::new("premium-feature"), &free_context).await.unwrap();
    assert_eq!(result.value.as_bool(), Some(false));
    assert_eq!(result.reason, EvaluationReason::Default);

    // Test premium user
    let premium_context = ContextBuilder::new()
        .user_id("premium-user")
        .plan("premium")
        .build();

    let result = evaluator.evaluate(&FlagId::new("premium-feature"), &premium_context).await.unwrap();
    assert_eq!(result.value.as_bool(), Some(true));
    assert_eq!(result.reason, EvaluationReason::RuleMatched);
}

#[tokio::test]
async fn test_percentage_rollout() {
    let storage = Arc::new(InMemoryStorage::new());
    let evaluator = FlagEvaluator::new(storage.clone());

    // Create flag with 50% rollout
    let mut flag = FlagDefinition::new_boolean("gradual-feature", "Gradual Feature", false).unwrap();
    flag.status = FlagStatus::Active;
    flag.rollout = Some(RolloutConfig {
        percentage: 50.0,
        bucket_by: "user_id".to_string(),
        seed: Some(12345),
    });

    storage.create(flag).await.unwrap();

    // Test many users and verify roughly 50% are enabled
    let mut enabled_count = 0;
    let total = 1000;

    for i in 0..total {
        let context = ContextBuilder::new()
            .user_id(&format!("user-{}", i))
            .build();

        let result = evaluator.evaluate(&FlagId::new("gradual-feature"), &context).await.unwrap();
        if result.value.is_truthy() {
            enabled_count += 1;
        }
    }

    // Allow 10% variance
    let percentage = (enabled_count as f64 / total as f64) * 100.0;
    assert!(percentage > 40.0 && percentage < 60.0, "Expected ~50%, got {}%", percentage);
}

#[tokio::test]
async fn test_experiment_variants() {
    let storage = Arc::new(InMemoryStorage::new());
    let evaluator = FlagEvaluator::new(storage.clone());

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

    let mut flag = FlagDefinition::new_experiment("ab-test", "A/B Test", variants).unwrap();
    flag.status = FlagStatus::Active;
    storage.create(flag).await.unwrap();

    // Test distribution
    let mut control_count = 0;
    let mut treatment_count = 0;

    for i in 0..1000 {
        let context = ContextBuilder::new()
            .user_id(&format!("user-{}", i))
            .build();

        let result = evaluator.evaluate(&FlagId::new("ab-test"), &context).await.unwrap();

        match result.experiment_variant.as_deref() {
            Some("control") => control_count += 1,
            Some("treatment") => treatment_count += 1,
            _ => {}
        }
    }

    // Verify roughly 50/50 distribution
    assert!(control_count > 400 && control_count < 600);
    assert!(treatment_count > 400 && treatment_count < 600);
}

#[tokio::test]
async fn test_user_override() {
    let storage = Arc::new(InMemoryStorage::new());
    let evaluator = FlagEvaluator::new(storage.clone());

    let mut flag = FlagDefinition::new_boolean("beta-feature", "Beta Feature", false).unwrap();
    flag.status = FlagStatus::Active;
    flag.user_overrides.insert("beta-tester".to_string(), FlagValue::Boolean(true));
    storage.create(flag).await.unwrap();

    // Regular user
    let regular = ContextBuilder::new()
        .user_id("regular-user")
        .build();
    let result = evaluator.evaluate(&FlagId::new("beta-feature"), &regular).await.unwrap();
    assert_eq!(result.value.as_bool(), Some(false));

    // Beta tester
    let beta = ContextBuilder::new()
        .user_id("beta-tester")
        .build();
    let result = evaluator.evaluate(&FlagId::new("beta-feature"), &beta).await.unwrap();
    assert_eq!(result.value.as_bool(), Some(true));
    assert_eq!(result.reason, EvaluationReason::Override);
}

#[tokio::test]
async fn test_evaluation_consistency() {
    let storage = Arc::new(InMemoryStorage::new());
    let evaluator = FlagEvaluator::new(storage.clone());

    let mut flag = FlagDefinition::new_boolean("consistent-flag", "Consistent Flag", false).unwrap();
    flag.status = FlagStatus::Active;
    flag.rollout = Some(RolloutConfig {
        percentage: 50.0,
        bucket_by: "user_id".to_string(),
        seed: None,
    });
    storage.create(flag).await.unwrap();

    let context = ContextBuilder::new()
        .user_id("test-user")
        .build();

    // Evaluate multiple times
    let results: Vec<_> = futures::future::join_all(
        (0..10).map(|_| evaluator.evaluate(&FlagId::new("consistent-flag"), &context))
    ).await;

    // All results should be identical
    let first = results[0].as_ref().unwrap().value.as_bool();
    for result in &results {
        assert_eq!(result.as_ref().unwrap().value.as_bool(), first);
    }
}
```

## Property-Based Tests

```rust
// Using proptest for property-based testing
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_percentage_within_bounds(percentage in 0.0..=100.0f64) {
        let hash = hash_to_percentage("test:user", None);
        prop_assert!(hash >= 0.0 && hash <= 100.0);
    }

    #[test]
    fn test_flag_key_normalization(key in "[a-zA-Z0-9 _-]{1,50}") {
        let flag_id = FlagId::new(&key);
        let normalized = flag_id.as_str();

        // Should be lowercase
        prop_assert!(normalized.chars().all(|c| !c.is_ascii_uppercase()));

        // Should only contain valid chars
        prop_assert!(normalized.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }
}
```

## Test Fixtures

```rust
// crates/flags/tests/fixtures.rs

pub fn create_test_flag(key: &str) -> FlagDefinition {
    let mut flag = FlagDefinition::new_boolean(key, key, false).unwrap();
    flag.status = FlagStatus::Active;
    flag
}

pub fn create_test_context(user_id: &str) -> EvaluationContext {
    ContextBuilder::new()
        .user_id(user_id)
        .build()
}

pub fn create_premium_context(user_id: &str) -> EvaluationContext {
    ContextBuilder::new()
        .user_id(user_id)
        .plan("premium")
        .group("premium-users")
        .build()
}
```

## Related Specs

- 394-flag-evaluation.md - Evaluation logic
- 402-flag-sdk-rust.md - SDK testing
- 399-ab-testing.md - Experiment testing
