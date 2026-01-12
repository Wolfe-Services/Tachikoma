# 397 - User Targeting

## Overview

User-level targeting for feature flags, allowing flags to be enabled/disabled for specific users based on their attributes.

## Rust Implementation

```rust
// crates/flags/src/targeting/user.rs

use crate::context::EvaluationContext;
use crate::types::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// User targeting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTargeting {
    /// Explicitly included user IDs
    pub include_users: HashSet<String>,
    /// Explicitly excluded user IDs
    pub exclude_users: HashSet<String>,
    /// Include users matching email patterns
    pub include_email_patterns: Vec<String>,
    /// Attribute-based targeting rules
    pub attribute_rules: Vec<UserAttributeRule>,
    /// Allow anonymous users
    pub allow_anonymous: bool,
}

impl Default for UserTargeting {
    fn default() -> Self {
        Self {
            include_users: HashSet::new(),
            exclude_users: HashSet::new(),
            include_email_patterns: vec![],
            attribute_rules: vec![],
            allow_anonymous: true,
        }
    }
}

/// Targeting rule based on user attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAttributeRule {
    /// Attribute path (e.g., "user.plan", "user.created_at")
    pub attribute: String,
    /// Matching operator
    pub operator: TargetingOperator,
    /// Values to match against
    pub values: Vec<serde_json::Value>,
}

/// Operators for targeting rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetingOperator {
    /// Exact match
    Equals,
    /// Not equal
    NotEquals,
    /// String contains
    Contains,
    /// String starts with
    StartsWith,
    /// String ends with
    EndsWith,
    /// Regex match
    Matches,
    /// Value in list
    In,
    /// Value not in list
    NotIn,
    /// Greater than (numeric/date)
    GreaterThan,
    /// Less than (numeric/date)
    LessThan,
    /// Between two values
    Between,
    /// Attribute exists
    Exists,
    /// Attribute does not exist
    NotExists,
}

/// Result of user targeting evaluation
#[derive(Debug, Clone)]
pub struct TargetingResult {
    /// Whether the user matches targeting
    pub matched: bool,
    /// Reason for the match/non-match
    pub reason: TargetingReason,
    /// Which rule matched (if any)
    pub matched_rule: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetingReason {
    ExplicitInclude,
    ExplicitExclude,
    EmailPatternMatch,
    AttributeRuleMatch,
    NoUserContext,
    AnonymousUserBlocked,
    NoRulesMatched,
}

impl UserTargeting {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add user ID to include list
    pub fn include_user(mut self, user_id: &str) -> Self {
        self.include_users.insert(user_id.to_string());
        self
    }

    /// Add multiple user IDs to include list
    pub fn include_users(mut self, user_ids: &[&str]) -> Self {
        for id in user_ids {
            self.include_users.insert(id.to_string());
        }
        self
    }

    /// Add user ID to exclude list
    pub fn exclude_user(mut self, user_id: &str) -> Self {
        self.exclude_users.insert(user_id.to_string());
        self
    }

    /// Include users with matching email domain
    pub fn include_email_domain(mut self, domain: &str) -> Self {
        self.include_email_patterns.push(format!("@{}$", regex::escape(domain)));
        self
    }

    /// Include users with matching email pattern (regex)
    pub fn include_email_pattern(mut self, pattern: &str) -> Self {
        self.include_email_patterns.push(pattern.to_string());
        self
    }

    /// Add attribute-based rule
    pub fn with_attribute_rule(mut self, rule: UserAttributeRule) -> Self {
        self.attribute_rules.push(rule);
        self
    }

    /// Require authenticated users only
    pub fn authenticated_only(mut self) -> Self {
        self.allow_anonymous = false;
        self
    }

    /// Evaluate targeting against a context
    pub fn evaluate(&self, context: &EvaluationContext) -> TargetingResult {
        // Check if we have user context
        let user = match &context.user {
            Some(u) => u,
            None => return TargetingResult {
                matched: false,
                reason: TargetingReason::NoUserContext,
                matched_rule: None,
            },
        };

        // Check for anonymous users
        if user.id.is_none() && !self.allow_anonymous {
            return TargetingResult {
                matched: false,
                reason: TargetingReason::AnonymousUserBlocked,
                matched_rule: None,
            };
        }

        // Get user identifier
        let user_id = user.id.as_ref()
            .or(user.anonymous_id.as_ref());

        // Check explicit exclusions first
        if let Some(id) = user_id {
            if self.exclude_users.contains(id) {
                return TargetingResult {
                    matched: false,
                    reason: TargetingReason::ExplicitExclude,
                    matched_rule: Some(format!("exclude:{}", id)),
                };
            }
        }

        // Check explicit inclusions
        if let Some(id) = user_id {
            if self.include_users.contains(id) {
                return TargetingResult {
                    matched: true,
                    reason: TargetingReason::ExplicitInclude,
                    matched_rule: Some(format!("include:{}", id)),
                };
            }
        }

        // Check email patterns
        if let Some(email) = &user.email {
            for pattern in &self.include_email_patterns {
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(email) {
                        return TargetingResult {
                            matched: true,
                            reason: TargetingReason::EmailPatternMatch,
                            matched_rule: Some(format!("email:{}", pattern)),
                        };
                    }
                }
            }
        }

        // Check attribute rules
        for rule in &self.attribute_rules {
            if self.evaluate_attribute_rule(rule, context) {
                return TargetingResult {
                    matched: true,
                    reason: TargetingReason::AttributeRuleMatch,
                    matched_rule: Some(format!("attr:{}", rule.attribute)),
                };
            }
        }

        TargetingResult {
            matched: false,
            reason: TargetingReason::NoRulesMatched,
            matched_rule: None,
        }
    }

    fn evaluate_attribute_rule(&self, rule: &UserAttributeRule, context: &EvaluationContext) -> bool {
        let value = context.get_property(&rule.attribute);

        match (&rule.operator, value) {
            (TargetingOperator::Exists, Some(_)) => true,
            (TargetingOperator::Exists, None) => false,
            (TargetingOperator::NotExists, None) => true,
            (TargetingOperator::NotExists, Some(_)) => false,

            (_, None) => false,
            (op, Some(val)) => self.compare_value(op, &val, &rule.values),
        }
    }

    fn compare_value(
        &self,
        operator: &TargetingOperator,
        actual: &serde_json::Value,
        expected: &[serde_json::Value],
    ) -> bool {
        if expected.is_empty() {
            return false;
        }

        match operator {
            TargetingOperator::Equals => expected.iter().any(|e| actual == e),

            TargetingOperator::NotEquals => expected.iter().all(|e| actual != e),

            TargetingOperator::Contains => {
                if let Some(actual_str) = actual.as_str() {
                    expected.iter().any(|e| {
                        e.as_str().map(|s| actual_str.contains(s)).unwrap_or(false)
                    })
                } else {
                    false
                }
            }

            TargetingOperator::StartsWith => {
                if let Some(actual_str) = actual.as_str() {
                    expected.iter().any(|e| {
                        e.as_str().map(|s| actual_str.starts_with(s)).unwrap_or(false)
                    })
                } else {
                    false
                }
            }

            TargetingOperator::EndsWith => {
                if let Some(actual_str) = actual.as_str() {
                    expected.iter().any(|e| {
                        e.as_str().map(|s| actual_str.ends_with(s)).unwrap_or(false)
                    })
                } else {
                    false
                }
            }

            TargetingOperator::Matches => {
                if let Some(actual_str) = actual.as_str() {
                    expected.iter().any(|e| {
                        e.as_str()
                            .and_then(|pattern| Regex::new(pattern).ok())
                            .map(|regex| regex.is_match(actual_str))
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            }

            TargetingOperator::In => {
                expected.contains(actual)
            }

            TargetingOperator::NotIn => {
                !expected.contains(actual)
            }

            TargetingOperator::GreaterThan => {
                if let (Some(actual_num), Some(expected_num)) = (
                    actual.as_f64(),
                    expected.first().and_then(|e| e.as_f64())
                ) {
                    actual_num > expected_num
                } else {
                    false
                }
            }

            TargetingOperator::LessThan => {
                if let (Some(actual_num), Some(expected_num)) = (
                    actual.as_f64(),
                    expected.first().and_then(|e| e.as_f64())
                ) {
                    actual_num < expected_num
                } else {
                    false
                }
            }

            TargetingOperator::Between => {
                if expected.len() >= 2 {
                    if let (Some(actual_num), Some(min), Some(max)) = (
                        actual.as_f64(),
                        expected[0].as_f64(),
                        expected[1].as_f64()
                    ) {
                        actual_num >= min && actual_num <= max
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}

/// Builder for user attribute rules
pub struct AttributeRuleBuilder {
    attribute: String,
}

impl AttributeRuleBuilder {
    pub fn new(attribute: &str) -> Self {
        Self {
            attribute: attribute.to_string(),
        }
    }

    pub fn equals(self, value: impl Into<serde_json::Value>) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::Equals,
            values: vec![value.into()],
        }
    }

    pub fn not_equals(self, value: impl Into<serde_json::Value>) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::NotEquals,
            values: vec![value.into()],
        }
    }

    pub fn in_list(self, values: Vec<serde_json::Value>) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::In,
            values,
        }
    }

    pub fn contains(self, substring: &str) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::Contains,
            values: vec![serde_json::json!(substring)],
        }
    }

    pub fn greater_than(self, value: f64) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::GreaterThan,
            values: vec![serde_json::json!(value)],
        }
    }

    pub fn between(self, min: f64, max: f64) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::Between,
            values: vec![serde_json::json!(min), serde_json::json!(max)],
        }
    }

    pub fn exists(self) -> UserAttributeRule {
        UserAttributeRule {
            attribute: self.attribute,
            operator: TargetingOperator::Exists,
            values: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextBuilder;

    #[test]
    fn test_explicit_include() {
        let targeting = UserTargeting::new()
            .include_user("user-123");

        let context = ContextBuilder::new()
            .user_id("user-123")
            .build();

        let result = targeting.evaluate(&context);
        assert!(result.matched);
        assert_eq!(result.reason, TargetingReason::ExplicitInclude);
    }

    #[test]
    fn test_explicit_exclude() {
        let targeting = UserTargeting::new()
            .include_users(&["user-123", "user-456"])
            .exclude_user("user-123");

        let context = ContextBuilder::new()
            .user_id("user-123")
            .build();

        let result = targeting.evaluate(&context);
        assert!(!result.matched);
        assert_eq!(result.reason, TargetingReason::ExplicitExclude);
    }

    #[test]
    fn test_email_pattern() {
        let targeting = UserTargeting::new()
            .include_email_domain("company.com");

        let context = ContextBuilder::new()
            .user_id("user-123")
            .email("employee@company.com")
            .build();

        let result = targeting.evaluate(&context);
        assert!(result.matched);
        assert_eq!(result.reason, TargetingReason::EmailPatternMatch);
    }

    #[test]
    fn test_attribute_rule() {
        let targeting = UserTargeting::new()
            .with_attribute_rule(
                AttributeRuleBuilder::new("user.plan").equals("enterprise")
            );

        let context = ContextBuilder::new()
            .user_id("user-123")
            .plan("enterprise")
            .build();

        let result = targeting.evaluate(&context);
        assert!(result.matched);
        assert_eq!(result.reason, TargetingReason::AttributeRuleMatch);
    }

    #[test]
    fn test_anonymous_blocked() {
        let targeting = UserTargeting::new()
            .authenticated_only();

        let context = ContextBuilder::new()
            .anonymous_id("anon-123")
            .build();

        let result = targeting.evaluate(&context);
        assert!(!result.matched);
        assert_eq!(result.reason, TargetingReason::AnonymousUserBlocked);
    }
}
```

## Related Specs

- 394-flag-evaluation.md - Evaluation engine
- 395-flag-context.md - Context handling
- 398-group-targeting.md - Group-level targeting
