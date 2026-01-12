# 399 - A/B Testing

## Overview

A/B testing and multivariate experiment support for feature flags with consistent variant assignment and statistical analysis.

## Rust Implementation

```rust
// crates/flags/src/experiment.rs

use crate::rollout::hash_to_percentage;
use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A/B Test experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Experiment identifier
    pub id: String,
    /// Flag this experiment is associated with
    pub flag_id: FlagId,
    /// Human-readable name
    pub name: String,
    /// Experiment description
    pub description: Option<String>,
    /// Experiment status
    pub status: ExperimentStatus,
    /// Variants in this experiment
    pub variants: Vec<Variant>,
    /// Property to use for bucketing
    pub bucket_by: String,
    /// Traffic allocation (0-100)
    pub traffic_allocation: f64,
    /// Whether to track exposure events
    pub track_exposure: bool,
    /// Goal metrics
    pub goals: Vec<Goal>,
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    /// End time
    pub ended_at: Option<DateTime<Utc>>,
    /// Winner (if concluded)
    pub winner: Option<String>,
    /// Experiment configuration
    pub config: ExperimentConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentStatus {
    /// Experiment is being set up
    Draft,
    /// Experiment is running
    Running,
    /// Experiment is paused
    Paused,
    /// Experiment has concluded
    Concluded,
    /// Experiment was abandoned
    Abandoned,
}

/// A variant in an A/B test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    /// Variant key (e.g., "control", "treatment_a")
    pub key: String,
    /// Display name
    pub name: String,
    /// Variant description
    pub description: Option<String>,
    /// Weight for traffic distribution (relative)
    pub weight: f64,
    /// Value returned when this variant is selected
    pub value: FlagValue,
    /// Whether this is the control variant
    pub is_control: bool,
}

/// Goal metric for the experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Goal identifier
    pub id: String,
    /// Goal name
    pub name: String,
    /// Event name to track
    pub event_name: String,
    /// Goal type
    pub goal_type: GoalType,
    /// Whether this is the primary goal
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    /// Conversion rate (binary)
    Conversion,
    /// Numeric value (average)
    Value,
    /// Count per user
    Count,
    /// Retention metric
    Retention,
}

/// Experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Minimum sample size per variant
    pub min_sample_size: u64,
    /// Statistical significance threshold (default 0.95)
    pub significance_level: f64,
    /// Minimum detectable effect (default 0.05)
    pub min_detectable_effect: f64,
    /// Auto-stop when significance reached
    pub auto_stop: bool,
    /// Sticky bucketing (consistent assignment)
    pub sticky_bucketing: bool,
    /// Salt for hash function
    pub salt: Option<String>,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            min_sample_size: 1000,
            significance_level: 0.95,
            min_detectable_effect: 0.05,
            auto_stop: false,
            sticky_bucketing: true,
            salt: None,
        }
    }
}

impl Experiment {
    pub fn new(flag_id: FlagId, name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            flag_id,
            name: name.to_string(),
            description: None,
            status: ExperimentStatus::Draft,
            variants: vec![],
            bucket_by: "user_id".to_string(),
            traffic_allocation: 100.0,
            track_exposure: true,
            goals: vec![],
            started_at: None,
            ended_at: None,
            winner: None,
            config: ExperimentConfig::default(),
        }
    }

    /// Add a variant to the experiment
    pub fn add_variant(&mut self, variant: Variant) {
        self.variants.push(variant);
    }

    /// Add control and treatment variants (simple A/B)
    pub fn with_ab_variants(mut self, control_value: FlagValue, treatment_value: FlagValue) -> Self {
        self.variants = vec![
            Variant {
                key: "control".to_string(),
                name: "Control".to_string(),
                description: None,
                weight: 50.0,
                value: control_value,
                is_control: true,
            },
            Variant {
                key: "treatment".to_string(),
                name: "Treatment".to_string(),
                description: None,
                weight: 50.0,
                value: treatment_value,
                is_control: false,
            },
        ];
        self
    }

    /// Add a goal metric
    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Start the experiment
    pub fn start(&mut self) {
        self.status = ExperimentStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Pause the experiment
    pub fn pause(&mut self) {
        self.status = ExperimentStatus::Paused;
    }

    /// Conclude the experiment with a winner
    pub fn conclude(&mut self, winner_key: Option<&str>) {
        self.status = ExperimentStatus::Concluded;
        self.ended_at = Some(Utc::now());
        self.winner = winner_key.map(|s| s.to_string());
    }

    /// Check if experiment is active
    pub fn is_active(&self) -> bool {
        self.status == ExperimentStatus::Running
    }

    /// Validate experiment configuration
    pub fn validate(&self) -> Result<(), ExperimentError> {
        if self.variants.is_empty() {
            return Err(ExperimentError::NoVariants);
        }

        if self.variants.len() < 2 {
            return Err(ExperimentError::InsufficientVariants);
        }

        let total_weight: f64 = self.variants.iter().map(|v| v.weight).sum();
        if (total_weight - 100.0).abs() > 0.01 {
            return Err(ExperimentError::InvalidWeights(total_weight));
        }

        let control_count = self.variants.iter().filter(|v| v.is_control).count();
        if control_count != 1 {
            return Err(ExperimentError::InvalidControlCount(control_count));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExperimentError {
    #[error("Experiment has no variants")]
    NoVariants,
    #[error("Experiment must have at least 2 variants")]
    InsufficientVariants,
    #[error("Variant weights must sum to 100, got {0}")]
    InvalidWeights(f64),
    #[error("Experiment must have exactly 1 control variant, got {0}")]
    InvalidControlCount(usize),
}

/// Experiment assignment engine
pub struct ExperimentAssigner {
    /// Override assignments for testing
    overrides: HashMap<String, HashMap<String, String>>, // user_id -> experiment_id -> variant_key
    /// Sticky bucket store
    sticky_buckets: HashMap<String, HashMap<String, String>>,
}

impl ExperimentAssigner {
    pub fn new() -> Self {
        Self {
            overrides: HashMap::new(),
            sticky_buckets: HashMap::new(),
        }
    }

    /// Assign a user to a variant
    pub fn assign(&self, experiment: &Experiment, bucket_key: &str) -> AssignmentResult {
        // Check if experiment is active
        if !experiment.is_active() {
            return AssignmentResult {
                variant: None,
                reason: AssignmentReason::ExperimentNotActive,
                in_experiment: false,
            };
        }

        // Check for override
        if let Some(user_overrides) = self.overrides.get(bucket_key) {
            if let Some(variant_key) = user_overrides.get(&experiment.id) {
                if let Some(variant) = experiment.variants.iter().find(|v| &v.key == variant_key) {
                    return AssignmentResult {
                        variant: Some(variant.clone()),
                        reason: AssignmentReason::Override,
                        in_experiment: true,
                    };
                }
            }
        }

        // Check sticky bucket
        if experiment.config.sticky_bucketing {
            if let Some(user_buckets) = self.sticky_buckets.get(bucket_key) {
                if let Some(variant_key) = user_buckets.get(&experiment.id) {
                    if let Some(variant) = experiment.variants.iter().find(|v| &v.key == variant_key) {
                        return AssignmentResult {
                            variant: Some(variant.clone()),
                            reason: AssignmentReason::StickyBucket,
                            in_experiment: true,
                        };
                    }
                }
            }
        }

        // Check traffic allocation
        let traffic_hash = self.calculate_traffic_hash(bucket_key, &experiment.id);
        if traffic_hash > experiment.traffic_allocation {
            return AssignmentResult {
                variant: None,
                reason: AssignmentReason::NotInTraffic,
                in_experiment: false,
            };
        }

        // Assign to variant based on weights
        let variant = self.select_variant(bucket_key, experiment);

        AssignmentResult {
            variant: Some(variant.clone()),
            reason: AssignmentReason::Assigned,
            in_experiment: true,
        }
    }

    fn calculate_traffic_hash(&self, bucket_key: &str, experiment_id: &str) -> f64 {
        let input = format!("traffic:{}:{}", experiment_id, bucket_key);
        hash_to_percentage(&input, None)
    }

    fn select_variant<'a>(&self, bucket_key: &str, experiment: &'a Experiment) -> &'a Variant {
        let salt = experiment.config.salt.as_deref().unwrap_or("");
        let input = format!("{}:{}:{}", experiment.id, bucket_key, salt);
        let hash_value = hash_to_percentage(&input, None);

        let mut cumulative = 0.0;
        for variant in &experiment.variants {
            cumulative += variant.weight;
            if hash_value <= cumulative {
                return variant;
            }
        }

        // Fallback to last variant
        experiment.variants.last().unwrap()
    }

    /// Set an override for a user
    pub fn set_override(&mut self, bucket_key: &str, experiment_id: &str, variant_key: &str) {
        self.overrides
            .entry(bucket_key.to_string())
            .or_insert_with(HashMap::new)
            .insert(experiment_id.to_string(), variant_key.to_string());
    }

    /// Store sticky bucket assignment
    pub fn store_sticky_bucket(&mut self, bucket_key: &str, experiment_id: &str, variant_key: &str) {
        self.sticky_buckets
            .entry(bucket_key.to_string())
            .or_insert_with(HashMap::new)
            .insert(experiment_id.to_string(), variant_key.to_string());
    }
}

impl Default for ExperimentAssigner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of variant assignment
#[derive(Debug, Clone)]
pub struct AssignmentResult {
    pub variant: Option<Variant>,
    pub reason: AssignmentReason,
    pub in_experiment: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentReason {
    Assigned,
    Override,
    StickyBucket,
    NotInTraffic,
    ExperimentNotActive,
}

/// Experiment results for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub experiment_id: String,
    pub variant_results: HashMap<String, VariantResults>,
    pub analysis_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResults {
    pub variant_key: String,
    pub sample_size: u64,
    pub conversions: u64,
    pub conversion_rate: f64,
    pub confidence_interval: (f64, f64),
    pub lift_vs_control: Option<f64>,
    pub statistical_significance: Option<f64>,
}

impl ExperimentResults {
    /// Check if results are statistically significant
    pub fn is_significant(&self, threshold: f64) -> bool {
        self.variant_results.values()
            .filter(|v| !v.variant_key.contains("control"))
            .any(|v| v.statistical_significance.map(|s| s >= threshold).unwrap_or(false))
    }

    /// Get the winning variant (if significant)
    pub fn get_winner(&self, threshold: f64) -> Option<&str> {
        let mut best: Option<(&str, f64)> = None;

        for (key, results) in &self.variant_results {
            if key.contains("control") {
                continue;
            }

            if let Some(sig) = results.statistical_significance {
                if sig >= threshold {
                    if let Some(lift) = results.lift_vs_control {
                        if lift > 0.0 {
                            if best.map(|(_, l)| lift > l).unwrap_or(true) {
                                best = Some((key.as_str(), lift));
                            }
                        }
                    }
                }
            }
        }

        best.map(|(k, _)| k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experiment_validation() {
        let mut experiment = Experiment::new(FlagId::new("test"), "Test Experiment");

        // No variants should fail
        assert!(experiment.validate().is_err());

        // One variant should fail
        experiment.add_variant(Variant {
            key: "control".to_string(),
            name: "Control".to_string(),
            description: None,
            weight: 100.0,
            value: FlagValue::Boolean(false),
            is_control: true,
        });
        assert!(experiment.validate().is_err());

        // Two variants with correct weights should pass
        experiment.add_variant(Variant {
            key: "treatment".to_string(),
            name: "Treatment".to_string(),
            description: None,
            weight: 0.0,
            value: FlagValue::Boolean(true),
            is_control: false,
        });

        // Wrong weights
        assert!(experiment.validate().is_err());

        // Fix weights
        experiment.variants[0].weight = 50.0;
        experiment.variants[1].weight = 50.0;
        assert!(experiment.validate().is_ok());
    }

    #[test]
    fn test_variant_assignment_consistency() {
        let mut experiment = Experiment::new(FlagId::new("test"), "Test");
        experiment = experiment.with_ab_variants(
            FlagValue::Boolean(false),
            FlagValue::Boolean(true),
        );
        experiment.start();

        let assigner = ExperimentAssigner::new();

        // Same user should always get same variant
        let result1 = assigner.assign(&experiment, "user-123");
        let result2 = assigner.assign(&experiment, "user-123");

        assert_eq!(
            result1.variant.as_ref().map(|v| &v.key),
            result2.variant.as_ref().map(|v| &v.key)
        );
    }

    #[test]
    fn test_variant_distribution() {
        let mut experiment = Experiment::new(FlagId::new("distribution-test"), "Test");
        experiment = experiment.with_ab_variants(
            FlagValue::Boolean(false),
            FlagValue::Boolean(true),
        );
        experiment.start();

        let assigner = ExperimentAssigner::new();

        let mut control = 0;
        let mut treatment = 0;

        for i in 0..10000 {
            let result = assigner.assign(&experiment, &format!("user-{}", i));
            if let Some(variant) = result.variant {
                if variant.key == "control" {
                    control += 1;
                } else {
                    treatment += 1;
                }
            }
        }

        // Should be roughly 50/50 (allow 10% variance)
        assert!(control > 4000 && control < 6000, "Control: {}", control);
        assert!(treatment > 4000 && treatment < 6000, "Treatment: {}", treatment);
    }

    #[test]
    fn test_override() {
        let mut experiment = Experiment::new(FlagId::new("test"), "Test");
        experiment = experiment.with_ab_variants(
            FlagValue::Boolean(false),
            FlagValue::Boolean(true),
        );
        experiment.start();

        let mut assigner = ExperimentAssigner::new();
        assigner.set_override("user-123", &experiment.id, "treatment");

        let result = assigner.assign(&experiment, "user-123");
        assert_eq!(result.variant.as_ref().map(|v| v.key.as_str()), Some("treatment"));
        assert_eq!(result.reason, AssignmentReason::Override);
    }
}
```

## Related Specs

- 394-flag-evaluation.md - Evaluation engine
- 396-percentage-rollout.md - Traffic allocation
- 406-flag-analytics.md - Experiment tracking
