# 412 - Analytics Event Schema

## Overview

JSON Schema definitions and validation for analytics events, ensuring data quality and consistency.

## Rust Implementation

```rust
// crates/analytics/src/schema.rs

use crate::event_types::{AnalyticsEvent, EventCategory};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid field type for {field}: expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
    },
    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },
    #[error("Unknown event type: {0}")]
    UnknownEventType(String),
    #[error("Schema validation failed: {0}")]
    ValidationFailed(String),
}

/// Event schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchema {
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: String,
    /// Event name pattern (regex)
    pub event_pattern: Option<String>,
    /// Required properties
    pub required_properties: Vec<PropertySchema>,
    /// Optional properties
    pub optional_properties: Vec<PropertySchema>,
    /// Allow additional properties
    #[serde(default = "default_true")]
    pub additional_properties: bool,
    /// Property constraints
    pub constraints: Vec<SchemaConstraint>,
}

fn default_true() -> bool {
    true
}

/// Property schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    /// Property name
    pub name: String,
    /// Property description
    pub description: Option<String>,
    /// Property type
    pub property_type: PropertyType,
    /// Allowed values (for enums)
    pub allowed_values: Option<Vec<Value>>,
    /// Minimum value (for numbers)
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    pub max: Option<f64>,
    /// String pattern (regex)
    pub pattern: Option<String>,
    /// Minimum length (for strings/arrays)
    pub min_length: Option<usize>,
    /// Maximum length (for strings/arrays)
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
    Any,
}

/// Schema constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SchemaConstraint {
    /// One of the properties must be present
    OneOf { properties: Vec<String> },
    /// All of the properties must be present together
    AllOf { properties: Vec<String> },
    /// If property A is present, B must also be present
    Dependency { property: String, requires: Vec<String> },
    /// Custom validation expression
    Custom { expression: String },
}

/// Schema validator
pub struct SchemaValidator {
    schemas: HashMap<String, EventSchema>,
    default_schema: EventSchema,
}

impl SchemaValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            schemas: HashMap::new(),
            default_schema: Self::create_default_schema(),
        };

        // Register built-in schemas
        validator.register_schema(Self::create_pageview_schema());
        validator.register_schema(Self::create_identify_schema());
        validator.register_schema(Self::create_revenue_schema());

        validator
    }

    pub fn register_schema(&mut self, schema: EventSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Validate an event against its schema
    pub fn validate(&self, event: &AnalyticsEvent) -> Result<(), Vec<SchemaError>> {
        let mut errors = Vec::new();

        // Get schema for event type
        let schema = self.schemas.get(&event.event)
            .unwrap_or(&self.default_schema);

        // Validate required fields on base event
        if event.event.is_empty() {
            errors.push(SchemaError::MissingField("event".to_string()));
        }

        if event.distinct_id.is_empty() {
            errors.push(SchemaError::MissingField("distinct_id".to_string()));
        }

        // Validate required properties
        for prop_schema in &schema.required_properties {
            match event.properties.get(&prop_schema.name) {
                None => {
                    errors.push(SchemaError::MissingField(prop_schema.name.clone()));
                }
                Some(value) => {
                    if let Err(e) = self.validate_property(value, prop_schema) {
                        errors.push(e);
                    }
                }
            }
        }

        // Validate optional properties that are present
        for prop_schema in &schema.optional_properties {
            if let Some(value) = event.properties.get(&prop_schema.name) {
                if let Err(e) = self.validate_property(value, prop_schema) {
                    errors.push(e);
                }
            }
        }

        // Validate constraints
        for constraint in &schema.constraints {
            if let Err(e) = self.validate_constraint(constraint, &event.properties) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_property(&self, value: &Value, schema: &PropertySchema) -> Result<(), SchemaError> {
        // Check type
        let value_type = self.get_value_type(value);
        if schema.property_type != PropertyType::Any && value_type != schema.property_type {
            return Err(SchemaError::InvalidType {
                field: schema.name.clone(),
                expected: format!("{:?}", schema.property_type),
                actual: format!("{:?}", value_type),
            });
        }

        // Check allowed values
        if let Some(ref allowed) = schema.allowed_values {
            if !allowed.contains(value) {
                return Err(SchemaError::InvalidValue {
                    field: schema.name.clone(),
                    message: format!("Value must be one of {:?}", allowed),
                });
            }
        }

        // Check numeric bounds
        if let Some(num) = value.as_f64() {
            if let Some(min) = schema.min {
                if num < min {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("Value must be >= {}", min),
                    });
                }
            }
            if let Some(max) = schema.max {
                if num > max {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("Value must be <= {}", max),
                    });
                }
            }
        }

        // Check string pattern
        if let (Some(s), Some(ref pattern)) = (value.as_str(), &schema.pattern) {
            let regex = regex::Regex::new(pattern).map_err(|_| SchemaError::ValidationFailed(
                format!("Invalid regex pattern: {}", pattern)
            ))?;
            if !regex.is_match(s) {
                return Err(SchemaError::InvalidValue {
                    field: schema.name.clone(),
                    message: format!("Value must match pattern: {}", pattern),
                });
            }
        }

        // Check string length
        if let Some(s) = value.as_str() {
            if let Some(min_len) = schema.min_length {
                if s.len() < min_len {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("String must be at least {} characters", min_len),
                    });
                }
            }
            if let Some(max_len) = schema.max_length {
                if s.len() > max_len {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("String must be at most {} characters", max_len),
                    });
                }
            }
        }

        // Check array length
        if let Some(arr) = value.as_array() {
            if let Some(min_len) = schema.min_length {
                if arr.len() < min_len {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("Array must have at least {} items", min_len),
                    });
                }
            }
            if let Some(max_len) = schema.max_length {
                if arr.len() > max_len {
                    return Err(SchemaError::InvalidValue {
                        field: schema.name.clone(),
                        message: format!("Array must have at most {} items", max_len),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_constraint(
        &self,
        constraint: &SchemaConstraint,
        properties: &HashMap<String, Value>,
    ) -> Result<(), SchemaError> {
        match constraint {
            SchemaConstraint::OneOf { properties: props } => {
                let present_count = props.iter()
                    .filter(|p| properties.contains_key(*p))
                    .count();

                if present_count == 0 {
                    return Err(SchemaError::ValidationFailed(
                        format!("At least one of {:?} must be present", props)
                    ));
                }
            }

            SchemaConstraint::AllOf { properties: props } => {
                let all_present = props.iter().all(|p| properties.contains_key(p));
                if !all_present {
                    return Err(SchemaError::ValidationFailed(
                        format!("All of {:?} must be present", props)
                    ));
                }
            }

            SchemaConstraint::Dependency { property, requires } => {
                if properties.contains_key(property) {
                    for req in requires {
                        if !properties.contains_key(req) {
                            return Err(SchemaError::ValidationFailed(
                                format!("Property '{}' requires '{}'", property, req)
                            ));
                        }
                    }
                }
            }

            SchemaConstraint::Custom { .. } => {
                // Custom expressions would need an expression evaluator
            }
        }

        Ok(())
    }

    fn get_value_type(&self, value: &Value) -> PropertyType {
        match value {
            Value::Null => PropertyType::Null,
            Value::Bool(_) => PropertyType::Boolean,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    PropertyType::Integer
                } else {
                    PropertyType::Number
                }
            }
            Value::String(_) => PropertyType::String,
            Value::Array(_) => PropertyType::Array,
            Value::Object(_) => PropertyType::Object,
        }
    }

    fn create_default_schema() -> EventSchema {
        EventSchema {
            name: "_default".to_string(),
            version: "1.0".to_string(),
            event_pattern: None,
            required_properties: vec![],
            optional_properties: vec![],
            additional_properties: true,
            constraints: vec![],
        }
    }

    fn create_pageview_schema() -> EventSchema {
        EventSchema {
            name: "$pageview".to_string(),
            version: "1.0".to_string(),
            event_pattern: None,
            required_properties: vec![
                PropertySchema {
                    name: "$current_url".to_string(),
                    description: Some("Full URL of the page".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: Some(r"^https?://".to_string()),
                    min_length: Some(1),
                    max_length: Some(2048),
                },
            ],
            optional_properties: vec![
                PropertySchema {
                    name: "$pathname".to_string(),
                    description: Some("Path portion of URL".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: Some(r"^/".to_string()),
                    min_length: None,
                    max_length: Some(512),
                },
                PropertySchema {
                    name: "$title".to_string(),
                    description: Some("Page title".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: None,
                    min_length: None,
                    max_length: Some(512),
                },
            ],
            additional_properties: true,
            constraints: vec![],
        }
    }

    fn create_identify_schema() -> EventSchema {
        EventSchema {
            name: "$identify".to_string(),
            version: "1.0".to_string(),
            event_pattern: None,
            required_properties: vec![
                PropertySchema {
                    name: "$user_id".to_string(),
                    description: Some("User identifier".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: None,
                    min_length: Some(1),
                    max_length: Some(256),
                },
            ],
            optional_properties: vec![],
            additional_properties: true,
            constraints: vec![],
        }
    }

    fn create_revenue_schema() -> EventSchema {
        EventSchema {
            name: "$revenue".to_string(),
            version: "1.0".to_string(),
            event_pattern: None,
            required_properties: vec![
                PropertySchema {
                    name: "$amount".to_string(),
                    description: Some("Revenue amount".to_string()),
                    property_type: PropertyType::Number,
                    allowed_values: None,
                    min: Some(0.0),
                    max: None,
                    pattern: None,
                    min_length: None,
                    max_length: None,
                },
                PropertySchema {
                    name: "$currency".to_string(),
                    description: Some("Currency code".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: Some(r"^[A-Z]{3}$".to_string()),
                    min_length: Some(3),
                    max_length: Some(3),
                },
            ],
            optional_properties: vec![
                PropertySchema {
                    name: "$product_id".to_string(),
                    description: Some("Product identifier".to_string()),
                    property_type: PropertyType::String,
                    allowed_values: None,
                    min: None,
                    max: None,
                    pattern: None,
                    min_length: None,
                    max_length: Some(256),
                },
            ],
            additional_properties: true,
            constraints: vec![],
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Event sanitizer for cleaning and normalizing events
pub struct EventSanitizer {
    /// Maximum property value length
    max_string_length: usize,
    /// Maximum number of properties
    max_properties: usize,
    /// Properties to strip (PII, etc.)
    strip_properties: Vec<String>,
}

impl EventSanitizer {
    pub fn new() -> Self {
        Self {
            max_string_length: 8192,
            max_properties: 1000,
            strip_properties: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "credit_card".to_string(),
                "ssn".to_string(),
            ],
        }
    }

    /// Sanitize an event
    pub fn sanitize(&self, event: &mut AnalyticsEvent) {
        // Remove sensitive properties
        for prop in &self.strip_properties {
            event.properties.remove(prop);
        }

        // Truncate long strings
        for value in event.properties.values_mut() {
            self.truncate_value(value);
        }

        // Limit number of properties
        if event.properties.len() > self.max_properties {
            let keys_to_remove: Vec<_> = event.properties.keys()
                .skip(self.max_properties)
                .cloned()
                .collect();

            for key in keys_to_remove {
                event.properties.remove(&key);
            }
        }
    }

    fn truncate_value(&self, value: &mut Value) {
        match value {
            Value::String(s) => {
                if s.len() > self.max_string_length {
                    s.truncate(self.max_string_length);
                    s.push_str("...[truncated]");
                }
            }
            Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.truncate_value(item);
                }
            }
            Value::Object(map) => {
                for (_, v) in map.iter_mut() {
                    self.truncate_value(v);
                }
            }
            _ => {}
        }
    }
}

impl Default for EventSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pageview_validation() {
        let validator = SchemaValidator::new();

        let mut event = AnalyticsEvent::new("$pageview", "user-123", EventCategory::Pageview);
        event.properties.insert("$current_url".to_string(), serde_json::json!("https://example.com"));

        let result = validator.validate(&event);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_required_property() {
        let validator = SchemaValidator::new();

        let event = AnalyticsEvent::new("$pageview", "user-123", EventCategory::Pageview);
        // Missing $current_url

        let result = validator.validate(&event);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitizer_strips_sensitive() {
        let sanitizer = EventSanitizer::new();

        let mut event = AnalyticsEvent::new("test", "user-123", EventCategory::Custom);
        event.properties.insert("password".to_string(), serde_json::json!("secret123"));
        event.properties.insert("name".to_string(), serde_json::json!("John"));

        sanitizer.sanitize(&mut event);

        assert!(!event.properties.contains_key("password"));
        assert!(event.properties.contains_key("name"));
    }
}
```

## JSON Schema Example

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AnalyticsEvent",
  "type": "object",
  "required": ["event", "distinct_id", "timestamp"],
  "properties": {
    "event": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256
    },
    "distinct_id": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "properties": {
      "type": "object",
      "additionalProperties": true
    }
  }
}
```

## Related Specs

- 411-event-types.md - Event type definitions
- 413-event-capture.md - Event capture API
- 425-privacy-compliance.md - PII handling
