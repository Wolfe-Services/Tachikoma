# 439 - Audit Filtering

**Phase:** 20 - Audit System
**Spec ID:** 439
**Status:** Planned
**Dependencies:** 435-audit-query
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement advanced filtering capabilities for audit queries with support for complex filter expressions and saved filter presets.

---

## Acceptance Criteria

- [x] Composite filter expressions (AND/OR/NOT)
- [x] Filter preset management
- [x] Dynamic filter building
- [x] Filter validation
- [x] Filter serialization/deserialization

---

## Implementation Details

### 1. Filter Types (src/filter.rs)

```rust
//! Advanced audit filtering.

use crate::{AuditCategory, AuditSeverity, AuditAction};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A filter expression for audit queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterExpression {
    /// Match all (no filter).
    All,
    /// Match none.
    None,
    /// Simple field comparison.
    Field(FieldFilter),
    /// Logical AND of multiple expressions.
    And(Vec<FilterExpression>),
    /// Logical OR of multiple expressions.
    Or(Vec<FilterExpression>),
    /// Logical NOT of an expression.
    Not(Box<FilterExpression>),
}

impl FilterExpression {
    /// Create an AND expression.
    pub fn and(filters: Vec<FilterExpression>) -> Self {
        Self::And(filters)
    }

    /// Create an OR expression.
    pub fn or(filters: Vec<FilterExpression>) -> Self {
        Self::Or(filters)
    }

    /// Create a NOT expression.
    pub fn not(filter: FilterExpression) -> Self {
        Self::Not(Box::new(filter))
    }

    /// Check if this expression is empty/matches all.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::All)
    }

    /// Simplify the expression.
    pub fn simplify(self) -> Self {
        match self {
            Self::And(mut filters) => {
                filters.retain(|f| !matches!(f, Self::All));
                if filters.is_empty() {
                    Self::All
                } else if filters.len() == 1 {
                    filters.remove(0).simplify()
                } else {
                    Self::And(filters.into_iter().map(|f| f.simplify()).collect())
                }
            }
            Self::Or(mut filters) => {
                if filters.iter().any(|f| matches!(f, Self::All)) {
                    return Self::All;
                }
                filters.retain(|f| !matches!(f, Self::None));
                if filters.is_empty() {
                    Self::None
                } else if filters.len() == 1 {
                    filters.remove(0).simplify()
                } else {
                    Self::Or(filters.into_iter().map(|f| f.simplify()).collect())
                }
            }
            Self::Not(inner) => {
                let simplified = inner.simplify();
                match simplified {
                    Self::All => Self::None,
                    Self::None => Self::All,
                    Self::Not(inner) => *inner,
                    other => Self::Not(Box::new(other)),
                }
            }
            other => other,
        }
    }
}

/// Field-level filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFilter {
    pub field: FilterField,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl FieldFilter {
    /// Create a new field filter.
    pub fn new(field: FilterField, operator: FilterOperator, value: FilterValue) -> Self {
        Self { field, operator, value }
    }

    /// Convenience: field equals value.
    pub fn eq(field: FilterField, value: FilterValue) -> Self {
        Self::new(field, FilterOperator::Equals, value)
    }

    /// Convenience: field contains value.
    pub fn contains(field: FilterField, value: FilterValue) -> Self {
        Self::new(field, FilterOperator::Contains, value)
    }
}

/// Fields that can be filtered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterField {
    Id,
    Timestamp,
    Category,
    Action,
    Severity,
    ActorType,
    ActorId,
    ActorName,
    TargetType,
    TargetId,
    TargetName,
    Outcome,
    CorrelationId,
    IpAddress,
    UserAgent,
    Metadata,
}

impl FilterField {
    /// Get the SQL column name.
    pub fn column_name(&self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Timestamp => "timestamp",
            Self::Category => "category",
            Self::Action => "action",
            Self::Severity => "severity",
            Self::ActorType => "actor_type",
            Self::ActorId => "actor_id",
            Self::ActorName => "actor_name",
            Self::TargetType => "target_type",
            Self::TargetId => "target_id",
            Self::TargetName => "target_name",
            Self::Outcome => "outcome",
            Self::CorrelationId => "correlation_id",
            Self::IpAddress => "ip_address",
            Self::UserAgent => "user_agent",
            Self::Metadata => "metadata",
        }
    }
}

/// Filter comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

impl FilterOperator {
    /// Get the SQL operator.
    pub fn sql_operator(&self) -> &'static str {
        match self {
            Self::Equals => "=",
            Self::NotEquals => "!=",
            Self::Contains => "LIKE",
            Self::NotContains => "NOT LIKE",
            Self::StartsWith => "LIKE",
            Self::EndsWith => "LIKE",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
            Self::Between => "BETWEEN",
        }
    }
}

/// Filter value types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    List(Vec<FilterValue>),
    Range { min: Box<FilterValue>, max: Box<FilterValue> },
    Null,
}

impl FilterValue {
    /// Create a string value.
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create a list value.
    pub fn list(values: Vec<FilterValue>) -> Self {
        Self::List(values)
    }
}
```

### 2. Filter Builder (src/filter_builder.rs)

```rust
//! Filter expression builder.

use crate::filter::*;
use crate::{AuditCategory, AuditSeverity};
use chrono::{DateTime, Utc};

/// Builder for constructing filter expressions.
#[derive(Debug, Default)]
pub struct FilterBuilder {
    filters: Vec<FilterExpression>,
}

impl FilterBuilder {
    /// Create a new filter builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a filter for category.
    pub fn category(mut self, category: AuditCategory) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::eq(
            FilterField::Category,
            FilterValue::String(category.to_string()),
        )));
        self
    }

    /// Add a filter for multiple categories.
    pub fn categories(mut self, categories: Vec<AuditCategory>) -> Self {
        let values: Vec<FilterValue> = categories
            .into_iter()
            .map(|c| FilterValue::String(c.to_string()))
            .collect();
        self.filters.push(FilterExpression::Field(FieldFilter::new(
            FilterField::Category,
            FilterOperator::In,
            FilterValue::List(values),
        )));
        self
    }

    /// Add a filter for severity.
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::eq(
            FilterField::Severity,
            FilterValue::String(format!("{:?}", severity).to_lowercase()),
        )));
        self
    }

    /// Add a filter for minimum severity.
    pub fn min_severity(mut self, min: AuditSeverity) -> Self {
        let severities = match min {
            AuditSeverity::Info => vec!["info", "low", "medium", "high", "critical"],
            AuditSeverity::Low => vec!["low", "medium", "high", "critical"],
            AuditSeverity::Medium => vec!["medium", "high", "critical"],
            AuditSeverity::High => vec!["high", "critical"],
            AuditSeverity::Critical => vec!["critical"],
        };
        let values: Vec<FilterValue> = severities
            .into_iter()
            .map(|s| FilterValue::String(s.to_string()))
            .collect();
        self.filters.push(FilterExpression::Field(FieldFilter::new(
            FilterField::Severity,
            FilterOperator::In,
            FilterValue::List(values),
        )));
        self
    }

    /// Add a filter for actor ID.
    pub fn actor_id(mut self, id: impl Into<String>) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::eq(
            FilterField::ActorId,
            FilterValue::String(id.into()),
        )));
        self
    }

    /// Add a filter for target ID.
    pub fn target_id(mut self, id: impl Into<String>) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::eq(
            FilterField::TargetId,
            FilterValue::String(id.into()),
        )));
        self
    }

    /// Add a time range filter.
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::new(
            FilterField::Timestamp,
            FilterOperator::Between,
            FilterValue::Range {
                min: Box::new(FilterValue::DateTime(start)),
                max: Box::new(FilterValue::DateTime(end)),
            },
        )));
        self
    }

    /// Add a filter for successful outcomes only.
    pub fn success_only(mut self) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::eq(
            FilterField::Outcome,
            FilterValue::String("success".to_string()),
        )));
        self
    }

    /// Add a filter for failed outcomes only.
    pub fn failures_only(mut self) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::new(
            FilterField::Outcome,
            FilterOperator::NotEquals,
            FilterValue::String("success".to_string()),
        )));
        self
    }

    /// Add a custom field filter.
    pub fn field(mut self, field: FilterField, op: FilterOperator, value: FilterValue) -> Self {
        self.filters.push(FilterExpression::Field(FieldFilter::new(field, op, value)));
        self
    }

    /// Add a raw filter expression.
    pub fn expression(mut self, expr: FilterExpression) -> Self {
        self.filters.push(expr);
        self
    }

    /// Build the filter expression (AND of all filters).
    pub fn build(self) -> FilterExpression {
        if self.filters.is_empty() {
            FilterExpression::All
        } else {
            FilterExpression::And(self.filters).simplify()
        }
    }

    /// Build as OR of all filters.
    pub fn build_or(self) -> FilterExpression {
        if self.filters.is_empty() {
            FilterExpression::None
        } else {
            FilterExpression::Or(self.filters).simplify()
        }
    }
}
```

### 3. Filter Presets (src/preset.rs)

```rust
//! Saved filter presets.

use crate::filter::FilterExpression;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A saved filter preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    /// Unique preset ID.
    pub id: String,
    /// Preset name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// The filter expression.
    pub filter: FilterExpression,
    /// When created.
    pub created_at: DateTime<Utc>,
    /// When last modified.
    pub updated_at: DateTime<Utc>,
    /// Who created it.
    pub created_by: Option<String>,
    /// Is this a system preset.
    pub is_system: bool,
    /// Tags for organization.
    pub tags: Vec<String>,
}

impl FilterPreset {
    /// Create a new preset.
    pub fn new(name: impl Into<String>, filter: FilterExpression) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            filter,
            created_at: now,
            updated_at: now,
            created_by: None,
            is_system: false,
            tags: Vec::new(),
        }
    }

    /// Add a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Mark as system preset.
    pub fn as_system(mut self) -> Self {
        self.is_system = true;
        self
    }
}

/// Preset manager for storing and retrieving filter presets.
pub struct PresetManager {
    presets: HashMap<String, FilterPreset>,
}

impl PresetManager {
    /// Create a new preset manager.
    pub fn new() -> Self {
        let mut manager = Self {
            presets: HashMap::new(),
        };
        manager.load_system_presets();
        manager
    }

    fn load_system_presets(&mut self) {
        use crate::filter_builder::FilterBuilder;
        use crate::{AuditCategory, AuditSeverity};

        // Security events preset
        let security = FilterPreset::new(
            "Security Events",
            FilterBuilder::new()
                .category(AuditCategory::Security)
                .build(),
        )
        .with_description("All security-related audit events")
        .as_system();
        self.presets.insert(security.id.clone(), security);

        // High severity preset
        let high_sev = FilterPreset::new(
            "High Severity",
            FilterBuilder::new()
                .min_severity(AuditSeverity::High)
                .build(),
        )
        .with_description("High and critical severity events")
        .as_system();
        self.presets.insert(high_sev.id.clone(), high_sev);

        // Failed operations preset
        let failures = FilterPreset::new(
            "Failed Operations",
            FilterBuilder::new()
                .failures_only()
                .build(),
        )
        .with_description("All failed or denied operations")
        .as_system();
        self.presets.insert(failures.id.clone(), failures);
    }

    /// Get a preset by ID.
    pub fn get(&self, id: &str) -> Option<&FilterPreset> {
        self.presets.get(id)
    }

    /// List all presets.
    pub fn list(&self) -> Vec<&FilterPreset> {
        self.presets.values().collect()
    }

    /// Save a preset.
    pub fn save(&mut self, preset: FilterPreset) {
        self.presets.insert(preset.id.clone(), preset);
    }

    /// Delete a preset.
    pub fn delete(&mut self, id: &str) -> bool {
        if let Some(preset) = self.presets.get(id) {
            if preset.is_system {
                return false; // Cannot delete system presets
            }
        }
        self.presets.remove(id).is_some()
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Testing Requirements

1. Filter expressions simplify correctly
2. All operators generate correct SQL
3. Builder produces valid expressions
4. Presets serialize/deserialize
5. System presets cannot be deleted

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md)
- Next: [440-audit-timeline.md](440-audit-timeline.md)
