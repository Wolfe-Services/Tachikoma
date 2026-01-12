//! Input validation for primitives.

mod path;
mod command;
mod pattern;

pub use path::PathValidator;
pub use command::CommandValidator;
pub use pattern::PatternValidator;

use crate::error::{PrimitiveError, PrimitiveResult};
use std::collections::HashMap;

/// Validation error details.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation.
    pub field: String,
    /// Error message.
    pub message: String,
    /// Validation rule that failed.
    pub rule: String,
    /// Suggested fix.
    pub suggestion: Option<String>,
}

impl ValidationError {
    pub fn new(field: &str, message: &str, rule: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            rule: rule.to_string(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} (rule: {})", self.field, self.message, self.rule)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " - {}", suggestion)?;
        }
        Ok(())
    }
}

/// Collection of validation errors.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> PrimitiveResult<T> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(PrimitiveError::Validation {
                message: self.to_string(),
            })
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<_> = self.errors.iter().map(|e| e.to_string()).collect();
        write!(f, "Validation failed: {}", messages.join("; "))
    }
}

/// Trait for input validators.
pub trait Validator<T> {
    /// Validate the input.
    fn validate(&self, input: &T) -> ValidationErrors;
}

/// Builder for validation rules.
pub struct ValidationBuilder<T> {
    validators: Vec<Box<dyn Fn(&T) -> Option<ValidationError> + Send + Sync>>,
}

impl<T> ValidationBuilder<T> {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validation rule.
    pub fn rule<F>(mut self, validator: F) -> Self
    where
        F: Fn(&T) -> Option<ValidationError> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Validate input against all rules.
    pub fn validate(&self, input: &T) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for validator in &self.validators {
            if let Some(error) = validator(input) {
                errors.add(error);
            }
        }
        errors
    }
}

impl<T> Default for ValidationBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Common validation functions.
pub mod rules {
    use super::*;

    /// Validate that a string is not empty.
    pub fn not_empty(field: &str) -> impl Fn(&str) -> Option<ValidationError> {
        let field = field.to_string();
        move |s: &str| {
            if s.trim().is_empty() {
                Some(ValidationError::new(
                    &field,
                    "cannot be empty",
                    "not_empty",
                ))
            } else {
                None
            }
        }
    }

    /// Validate string length.
    pub fn max_length(field: &str, max: usize) -> impl Fn(&str) -> Option<ValidationError> {
        let field = field.to_string();
        move |s: &str| {
            if s.len() > max {
                Some(ValidationError::new(
                    &field,
                    &format!("exceeds maximum length of {}", max),
                    "max_length",
                ))
            } else {
                None
            }
        }
    }

    /// Validate numeric range.
    pub fn range<N: PartialOrd + std::fmt::Display + Copy>(
        field: &str,
        min: N,
        max: N,
    ) -> impl Fn(&N) -> Option<ValidationError> {
        let field = field.to_string();
        move |n: &N| {
            if *n < min || *n > max {
                Some(ValidationError::new(
                    &field,
                    &format!("must be between {} and {}", min, max),
                    "range",
                ))
            } else {
                None
            }
        }
    }

    /// Validate against a regex pattern.
    pub fn matches_pattern(
        field: &str,
        pattern: &str,
        description: &str,
    ) -> impl Fn(&str) -> Option<ValidationError> {
        let field = field.to_string();
        let description = description.to_string();
        let regex = regex::Regex::new(pattern).unwrap();
        move |s: &str| {
            if !regex.is_match(s) {
                Some(
                    ValidationError::new(&field, &description, "pattern")
                        .with_suggestion(&format!("Must match pattern: {}", pattern)),
                )
            } else {
                None
            }
        }
    }
}

/// Validated wrapper type.
#[derive(Debug, Clone)]
pub struct Validated<T> {
    value: T,
}

impl<T> Validated<T> {
    /// Create a validated value (assumes already validated).
    pub fn new_unchecked(value: T) -> Self {
        Self { value }
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the inner value.
    pub fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_empty() {
        let validator = rules::not_empty("field");
        assert!(validator("").is_some());
        assert!(validator("   ").is_some());
        assert!(validator("value").is_none());
    }

    #[test]
    fn test_max_length() {
        let validator = rules::max_length("field", 5);
        assert!(validator("123456").is_some());
        assert!(validator("12345").is_none());
        assert!(validator("1234").is_none());
    }

    #[test]
    fn test_validation_builder() {
        let builder = ValidationBuilder::<String>::new()
            .rule(|s| {
                if s.is_empty() {
                    Some(ValidationError::new("value", "empty", "not_empty"))
                } else {
                    None
                }
            });

        let errors = builder.validate(&String::new());
        assert!(!errors.is_empty());

        let errors = builder.validate(&"hello".to_string());
        assert!(errors.is_empty());
    }
}