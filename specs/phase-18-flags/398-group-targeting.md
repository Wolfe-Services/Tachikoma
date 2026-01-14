# 398 - Group Targeting

## Overview

Group-level targeting for feature flags, enabling flags for organizations, teams, or custom segments.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/targeting/group.rs

use crate::context::EvaluationContext;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Group targeting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTargeting {
    /// Groups that are explicitly included
    pub include_groups: HashSet<String>,
    /// Groups that are explicitly excluded
    pub exclude_groups: HashSet<String>,
    /// Group attribute rules
    pub group_rules: Vec<GroupRule>,
    /// Require user to be in at least one group
    pub require_group_membership: bool,
    /// Match mode for multiple groups
    pub match_mode: GroupMatchMode,
}

/// How to match when user is in multiple groups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMatchMode {
    /// Match if ANY group matches (OR logic)
    Any,
    /// Match only if ALL groups match (AND logic)
    All,
    /// Match based on group priority
    Priority,
}

impl Default for GroupTargeting {
    fn default() -> Self {
        Self {
            include_groups: HashSet::new(),
            exclude_groups: HashSet::new(),
            group_rules: vec![],
            require_group_membership: false,
            match_mode: GroupMatchMode::Any,
        }
    }
}

/// Rule for targeting based on group attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRule {
    /// Group type (e.g., "organization", "team", "segment")
    pub group_type: String,
    /// Attribute to check
    pub attribute: String,
    /// Operator for comparison
    pub operator: GroupOperator,
    /// Values to compare against
    pub values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    Contains,
    GreaterThan,
    LessThan,
}

/// Result of group targeting evaluation
#[derive(Debug, Clone)]
pub struct GroupTargetingResult {
    pub matched: bool,
    pub reason: GroupTargetingReason,
    pub matched_groups: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupTargetingReason {
    ExplicitInclude,
    ExplicitExclude,
    GroupRuleMatch,
    NoGroupMembership,
    NotInAnyTargetedGroup,
}

impl GroupTargeting {
    pub fn new() -> Self {
        Self::default()
    }

    /// Include a specific group
    pub fn include_group(mut self, group_id: &str) -> Self {
        self.include_groups.insert(group_id.to_string());
        self
    }

    /// Include multiple groups
    pub fn include_groups(mut self, group_ids: &[&str]) -> Self {
        for id in group_ids {
            self.include_groups.insert(id.to_string());
        }
        self
    }

    /// Exclude a specific group
    pub fn exclude_group(mut self, group_id: &str) -> Self {
        self.exclude_groups.insert(group_id.to_string());
        self
    }

    /// Add a group rule
    pub fn with_rule(mut self, rule: GroupRule) -> Self {
        self.group_rules.push(rule);
        self
    }

    /// Require user to be in at least one group
    pub fn require_membership(mut self) -> Self {
        self.require_group_membership = true;
        self
    }

    /// Set match mode
    pub fn with_match_mode(mut self, mode: GroupMatchMode) -> Self {
        self.match_mode = mode;
        self
    }

    /// Evaluate targeting against context
    pub fn evaluate(&self, context: &EvaluationContext) -> GroupTargetingResult {
        let user_groups = context.groups();

        // Check if group membership is required but user has none
        if self.require_group_membership && user_groups.is_empty() {
            return GroupTargetingResult {
                matched: false,
                reason: GroupTargetingReason::NoGroupMembership,
                matched_groups: vec![],
            };
        }

        let mut matched_groups = Vec::new();
        let mut excluded = false;

        for group in &user_groups {
            // Check exclusions first
            if self.exclude_groups.contains(group) {
                excluded = true;
                continue;
            }

            // Check explicit inclusions
            if self.include_groups.contains(group) {
                matched_groups.push(group.clone());
            }
        }

        // If any group was explicitly excluded and match mode is All
        if excluded && self.match_mode == GroupMatchMode::All {
            return GroupTargetingResult {
                matched: false,
                reason: GroupTargetingReason::ExplicitExclude,
                matched_groups: vec![],
            };
        }

        // If we have explicit matches
        if !matched_groups.is_empty() {
            return GroupTargetingResult {
                matched: true,
                reason: GroupTargetingReason::ExplicitInclude,
                matched_groups,
            };
        }

        // If user was in an excluded group
        if excluded {
            return GroupTargetingResult {
                matched: false,
                reason: GroupTargetingReason::ExplicitExclude,
                matched_groups: vec![],
            };
        }

        GroupTargetingResult {
            matched: false,
            reason: GroupTargetingReason::NotInAnyTargetedGroup,
            matched_groups: vec![],
        }
    }
}

/// Group properties store for attribute-based targeting
#[derive(Debug, Clone, Default)]
pub struct GroupStore {
    /// Group attributes by group type and ID
    groups: HashMap<String, HashMap<String, GroupAttributes>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAttributes {
    pub id: String,
    pub group_type: String,
    pub name: String,
    pub properties: Properties,
}

impl GroupStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a group with attributes
    pub fn register_group(&mut self, group: GroupAttributes) {
        let type_groups = self.groups
            .entry(group.group_type.clone())
            .or_insert_with(HashMap::new);
        type_groups.insert(group.id.clone(), group);
    }

    /// Get group attributes
    pub fn get_group(&self, group_type: &str, group_id: &str) -> Option<&GroupAttributes> {
        self.groups.get(group_type)?.get(group_id)
    }

    /// Check if a group matches a rule
    pub fn evaluate_rule(&self, rule: &GroupRule, context: &EvaluationContext) -> bool {
        let user_groups = context.groups();

        for group_id in &user_groups {
            if let Some(group) = self.get_group(&rule.group_type, group_id) {
                if self.check_rule_against_group(rule, group) {
                    return true;
                }
            }
        }

        false
    }

    fn check_rule_against_group(&self, rule: &GroupRule, group: &GroupAttributes) -> bool {
        let value = group.properties.get(&rule.attribute);

        match (&rule.operator, value) {
            (GroupOperator::Equals, Some(v)) => rule.values.contains(v),
            (GroupOperator::NotEquals, Some(v)) => !rule.values.contains(v),
            (GroupOperator::In, Some(v)) => rule.values.contains(v),
            (GroupOperator::NotIn, Some(v)) => !rule.values.contains(v),
            (GroupOperator::Contains, Some(v)) => {
                if let Some(s) = v.as_str() {
                    rule.values.iter().any(|rv| {
                        rv.as_str().map(|rs| s.contains(rs)).unwrap_or(false)
                    })
                } else {
                    false
                }
            }
            (GroupOperator::GreaterThan, Some(v)) => {
                if let (Some(n), Some(rn)) = (v.as_f64(), rule.values.first().and_then(|rv| rv.as_f64())) {
                    n > rn
                } else {
                    false
                }
            }
            (GroupOperator::LessThan, Some(v)) => {
                if let (Some(n), Some(rn)) = (v.as_f64(), rule.values.first().and_then(|rv| rv.as_f64())) {
                    n < rn
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Segment definition for dynamic group membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// Segment identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Rules for segment membership
    pub rules: Vec<SegmentRule>,
    /// How to combine rules
    pub rule_combination: RuleCombination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentRule {
    /// Property path to check
    pub property: String,
    /// Operator
    pub operator: String,
    /// Value to compare
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleCombination {
    And,
    Or,
}

impl Segment {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            rules: vec![],
            rule_combination: RuleCombination::And,
        }
    }

    /// Check if a context matches this segment
    pub fn matches(&self, context: &EvaluationContext) -> bool {
        if self.rules.is_empty() {
            return false;
        }

        match self.rule_combination {
            RuleCombination::And => self.rules.iter().all(|r| self.evaluate_rule(r, context)),
            RuleCombination::Or => self.rules.iter().any(|r| self.evaluate_rule(r, context)),
        }
    }

    fn evaluate_rule(&self, rule: &SegmentRule, context: &EvaluationContext) -> bool {
        let value = context.get_property(&rule.property);

        match (value, rule.operator.as_str()) {
            (Some(v), "equals") => v == rule.value,
            (Some(v), "not_equals") => v != rule.value,
            (Some(v), "contains") => {
                v.as_str().map(|s| s.contains(rule.value.as_str().unwrap_or(""))).unwrap_or(false)
            }
            (Some(v), "greater_than") => {
                v.as_f64().zip(rule.value.as_f64()).map(|(a, b)| a > b).unwrap_or(false)
            }
            (Some(v), "less_than") => {
                v.as_f64().zip(rule.value.as_f64()).map(|(a, b)| a < b).unwrap_or(false)
            }
            (Some(_), "exists") => true,
            (None, "not_exists") => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextBuilder;

    #[test]
    fn test_group_include() {
        let targeting = GroupTargeting::new()
            .include_group("org-123")
            .include_group("org-456");

        let context = ContextBuilder::new()
            .user_id("user-1")
            .group("org-123")
            .build();

        let result = targeting.evaluate(&context);
        assert!(result.matched);
        assert_eq!(result.reason, GroupTargetingReason::ExplicitInclude);
    }

    #[test]
    fn test_group_exclude() {
        let targeting = GroupTargeting::new()
            .include_group("org-123")
            .exclude_group("org-456");

        let context = ContextBuilder::new()
            .user_id("user-1")
            .group("org-456")
            .build();

        let result = targeting.evaluate(&context);
        assert!(!result.matched);
        assert_eq!(result.reason, GroupTargetingReason::ExplicitExclude);
    }

    #[test]
    fn test_require_membership() {
        let targeting = GroupTargeting::new()
            .include_group("org-123")
            .require_membership();

        let context = ContextBuilder::new()
            .user_id("user-1")
            .build();

        let result = targeting.evaluate(&context);
        assert!(!result.matched);
        assert_eq!(result.reason, GroupTargetingReason::NoGroupMembership);
    }

    #[test]
    fn test_segment_matching() {
        let segment = Segment {
            id: "high-value".to_string(),
            name: "High Value Users".to_string(),
            description: None,
            rules: vec![
                SegmentRule {
                    property: "user.plan".to_string(),
                    operator: "equals".to_string(),
                    value: serde_json::json!("enterprise"),
                },
            ],
            rule_combination: RuleCombination::And,
        };

        let context = ContextBuilder::new()
            .user_id("user-1")
            .plan("enterprise")
            .build();

        assert!(segment.matches(&context));
    }
}
```

## Group Configuration Examples

```yaml
# Target by organization plan
groups:
  type: organization
  include:
    - org-123
    - org-456
  exclude:
    - org-789
  rules:
    - attribute: plan
      operator: in
      values: [enterprise, business]

# Target beta program members
groups:
  include:
    - beta-program
  match_mode: any

# Segment definition
segments:
  - id: power-users
    name: Power Users
    rules:
      - property: user.events_count
        operator: greater_than
        value: 100
      - property: user.created_at
        operator: less_than
        value: "2024-01-01"
    combination: and
```

## Related Specs

- 397-user-targeting.md - User-level targeting
- 394-flag-evaluation.md - Evaluation engine
- 395-flag-context.md - Context handling
