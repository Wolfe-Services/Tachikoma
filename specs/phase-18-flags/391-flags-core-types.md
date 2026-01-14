# 391 - Feature Flags Core Types

## Overview

Core type definitions for the feature flags system, providing type-safe flag values, evaluation contexts, and configuration structures.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a feature flag
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlagId(pub String);

impl FlagId {
    pub fn new(key: &str) -> Self {
        Self(key.to_lowercase().replace(" ", "-"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FlagId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Supported value types for feature flags
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FlagValue {
    /// Simple on/off boolean flag
    Boolean(bool),
    /// String variant for A/B testing
    String(String),
    /// Numeric value for gradual rollouts
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Complex JSON configuration
    Json(serde_json::Value),
    /// Multiple string variants for multivariate tests
    Variant(String),
}

impl FlagValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FlagValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            FlagValue::String(s) | FlagValue::Variant(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            FlagValue::Number(n) => Some(*n),
            FlagValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            FlagValue::Json(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            FlagValue::Boolean(b) => *b,
            FlagValue::String(s) => !s.is_empty(),
            FlagValue::Number(n) => *n != 0.0,
            FlagValue::Integer(i) => *i != 0,
            FlagValue::Json(v) => !v.is_null(),
            FlagValue::Variant(s) => !s.is_empty(),
        }
    }
}

impl Default for FlagValue {
    fn default() -> Self {
        FlagValue::Boolean(false)
    }
}

/// Current state of a feature flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagStatus {
    /// Flag is active and being evaluated
    Active,
    /// Flag is disabled, returns default value
    Disabled,
    /// Flag is in testing mode (only for specified users)
    Testing,
    /// Flag is scheduled for deprecation
    Deprecated,
    /// Flag is archived and not evaluated
    Archived,
}

impl Default for FlagStatus {
    fn default() -> Self {
        FlagStatus::Disabled
    }
}

/// Environment where the flag applies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Staging,
    Production,
    Custom(String),
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

/// Evaluation result containing flag value and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// The flag identifier
    pub flag_id: FlagId,
    /// The resolved value
    pub value: FlagValue,
    /// Whether the flag was found
    pub found: bool,
    /// Reason for the evaluation result
    pub reason: EvaluationReason,
    /// Which rule matched (if any)
    pub matched_rule: Option<String>,
    /// Time taken to evaluate (microseconds)
    pub evaluation_time_us: u64,
    /// Timestamp of evaluation
    pub evaluated_at: DateTime<Utc>,
    /// Whether the user is in an experiment
    pub in_experiment: bool,
    /// Experiment variant (if applicable)
    pub experiment_variant: Option<String>,
}

impl EvaluationResult {
    pub fn not_found(flag_id: FlagId) -> Self {
        Self {
            flag_id,
            value: FlagValue::default(),
            found: false,
            reason: EvaluationReason::NotFound,
            matched_rule: None,
            evaluation_time_us: 0,
            evaluated_at: Utc::now(),
            in_experiment: false,
            experiment_variant: None,
        }
    }

    pub fn disabled(flag_id: FlagId, default: FlagValue) -> Self {
        Self {
            flag_id,
            value: default,
            found: true,
            reason: EvaluationReason::Disabled,
            matched_rule: None,
            evaluation_time_us: 0,
            evaluated_at: Utc::now(),
            in_experiment: false,
            experiment_variant: None,
        }
    }
}

/// Reason why a particular evaluation result was returned
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationReason {
    /// Flag not found in storage
    NotFound,
    /// Flag is disabled
    Disabled,
    /// Default value returned (no rules matched)
    Default,
    /// User was targeted directly
    UserTargeted,
    /// User's group was targeted
    GroupTargeted,
    /// User fell within percentage rollout
    PercentageRollout,
    /// Rule condition matched
    RuleMatched,
    /// Override was applied
    Override,
    /// Flag is in experiment mode
    Experiment,
    /// Fallback due to error
    Error,
    /// Cached value returned
    Cached,
}

/// Operators for rule conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches,          // Regex match
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,               // Value in list
    NotIn,
    SemverEquals,
    SemverGreaterThan,
    SemverLessThan,
    IsSet,
    IsNotSet,
}

/// A condition for rule evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Property to check from evaluation context
    pub property: String,
    /// Comparison operator
    pub operator: Operator,
    /// Value(s) to compare against
    pub value: ConditionValue,
    /// Whether to negate the condition
    #[serde(default)]
    pub negate: bool,
}

/// Values that can be used in conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
    Null,
}

/// A rule that determines flag value for matching contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Priority (higher = checked first)
    pub priority: i32,
    /// Conditions that must all match (AND logic)
    pub conditions: Vec<Condition>,
    /// Value to return when rule matches
    pub value: FlagValue,
    /// Whether the rule is active
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Statistics about flag usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlagStatistics {
    /// Total evaluations
    pub total_evaluations: u64,
    /// Evaluations returning true/enabled
    pub true_evaluations: u64,
    /// Evaluations returning false/disabled
    pub false_evaluations: u64,
    /// Unique users who have seen this flag
    pub unique_users: u64,
    /// Average evaluation time in microseconds
    pub avg_evaluation_time_us: f64,
    /// Last evaluation timestamp
    pub last_evaluated: Option<DateTime<Utc>>,
    /// Error count during evaluations
    pub error_count: u64,
}

/// Metadata for auditing and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagMetadata {
    /// When the flag was created
    pub created_at: DateTime<Utc>,
    /// Who created the flag
    pub created_by: String,
    /// Last modification time
    pub updated_at: DateTime<Utc>,
    /// Who last modified the flag
    pub updated_by: String,
    /// Arbitrary tags for organization
    pub tags: Vec<String>,
    /// Project or team owner
    pub owner: Option<String>,
    /// Link to documentation or ticket
    pub documentation_url: Option<String>,
    /// Expected removal date
    pub sunset_date: Option<DateTime<Utc>>,
}

/// Type alias for custom properties in contexts
pub type Properties = HashMap<String, serde_json::Value>;

/// Configuration for percentage-based rollouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutConfig {
    /// Percentage of users to enable (0-100)
    pub percentage: f64,
    /// Property to use for consistent bucketing
    #[serde(default = "default_bucket_key")]
    pub bucket_by: String,
    /// Seed for hash function (for reproducibility)
    pub seed: Option<u64>,
}

fn default_bucket_key() -> String {
    "user_id".to_string()
}

/// Configuration for A/B testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Experiment name
    pub name: String,
    /// Variants and their weights
    pub variants: Vec<ExperimentVariant>,
    /// Property to use for assignment
    pub bucket_by: String,
    /// Whether to track exposure automatically
    #[serde(default = "default_true")]
    pub track_exposure: bool,
    /// Goal metrics for the experiment
    pub goals: Vec<String>,
}

/// A variant in an A/B test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariant {
    /// Variant key
    pub key: String,
    /// Variant name for display
    pub name: String,
    /// Weight for distribution (relative to other variants)
    pub weight: f64,
    /// Value to return for this variant
    pub value: FlagValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_id_normalization() {
        let flag = FlagId::new("My Feature Flag");
        assert_eq!(flag.as_str(), "my-feature-flag");
    }

    #[test]
    fn test_flag_value_is_truthy() {
        assert!(FlagValue::Boolean(true).is_truthy());
        assert!(!FlagValue::Boolean(false).is_truthy());
        assert!(FlagValue::String("test".into()).is_truthy());
        assert!(!FlagValue::String("".into()).is_truthy());
        assert!(FlagValue::Number(1.0).is_truthy());
        assert!(!FlagValue::Number(0.0).is_truthy());
    }

    #[test]
    fn test_flag_value_conversions() {
        let bool_val = FlagValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_string(), None);

        let str_val = FlagValue::String("variant-a".into());
        assert_eq!(str_val.as_string(), Some("variant-a"));
        assert_eq!(str_val.as_bool(), None);
    }

    #[test]
    fn test_evaluation_result_not_found() {
        let result = EvaluationResult::not_found(FlagId::new("test"));
        assert!(!result.found);
        assert_eq!(result.reason, EvaluationReason::NotFound);
    }
}
```

## TypeScript Types

```typescript
// packages/flags/src/types.ts

export interface FlagId {
  readonly value: string;
}

export type FlagValue =
  | { type: 'boolean'; value: boolean }
  | { type: 'string'; value: string }
  | { type: 'number'; value: number }
  | { type: 'integer'; value: number }
  | { type: 'json'; value: unknown }
  | { type: 'variant'; value: string };

export type FlagStatus =
  | 'active'
  | 'disabled'
  | 'testing'
  | 'deprecated'
  | 'archived';

export type Environment =
  | 'development'
  | 'staging'
  | 'production'
  | { custom: string };

export interface EvaluationResult {
  flagId: FlagId;
  value: FlagValue;
  found: boolean;
  reason: EvaluationReason;
  matchedRule?: string;
  evaluationTimeUs: number;
  evaluatedAt: Date;
  inExperiment: boolean;
  experimentVariant?: string;
}

export type EvaluationReason =
  | 'not_found'
  | 'disabled'
  | 'default'
  | 'user_targeted'
  | 'group_targeted'
  | 'percentage_rollout'
  | 'rule_matched'
  | 'override'
  | 'experiment'
  | 'error'
  | 'cached';

export type Operator =
  | 'equals'
  | 'not_equals'
  | 'contains'
  | 'not_contains'
  | 'starts_with'
  | 'ends_with'
  | 'matches'
  | 'greater_than'
  | 'greater_than_or_equal'
  | 'less_than'
  | 'less_than_or_equal'
  | 'in'
  | 'not_in'
  | 'semver_equals'
  | 'semver_greater_than'
  | 'semver_less_than'
  | 'is_set'
  | 'is_not_set';

export interface Condition {
  property: string;
  operator: Operator;
  value: string | number | boolean | string[] | null;
  negate?: boolean;
}

export interface Rule {
  id: string;
  name: string;
  priority: number;
  conditions: Condition[];
  value: FlagValue;
  enabled: boolean;
}

export interface FlagStatistics {
  totalEvaluations: number;
  trueEvaluations: number;
  falseEvaluations: number;
  uniqueUsers: number;
  avgEvaluationTimeUs: number;
  lastEvaluated?: Date;
  errorCount: number;
}

export interface FlagMetadata {
  createdAt: Date;
  createdBy: string;
  updatedAt: Date;
  updatedBy: string;
  tags: string[];
  owner?: string;
  documentationUrl?: string;
  sunsetDate?: Date;
}

export type Properties = Record<string, unknown>;

export interface RolloutConfig {
  percentage: number;
  bucketBy: string;
  seed?: number;
}

export interface ExperimentConfig {
  name: string;
  variants: ExperimentVariant[];
  bucketBy: string;
  trackExposure: boolean;
  goals: string[];
}

export interface ExperimentVariant {
  key: string;
  name: string;
  weight: number;
  value: FlagValue;
}
```

## Related Specs

- 392-flag-definition.md - Flag definition structure
- 394-flag-evaluation.md - Evaluation engine
- 395-flag-context.md - Evaluation context
