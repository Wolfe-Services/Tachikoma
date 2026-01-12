# 047 - Primitives Input Validation

**Phase:** 2 - Five Primitives
**Spec ID:** 047
**Status:** Planned
**Dependencies:** 046-primitives-trait
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement comprehensive input validation for all primitives to ensure safe and correct operation with helpful error messages.

---

## Acceptance Criteria

- [x] Path validation (traversal attacks, allowed paths)
- [x] Command validation (blocked patterns, injection)
- [x] Pattern validation (regex syntax)
- [x] Size and limit validation
- [x] Type validation with coercion
- [x] Custom validators per primitive

---

## Implementation Details

### 1. Validation Module (src/validation/mod.rs)

```rust
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
```

### 2. Path Validation (src/validation/path.rs)

```rust
//! Path validation utilities.

use super::{ValidationError, ValidationErrors};
use std::path::{Component, Path, PathBuf};

/// Path validator.
pub struct PathValidator {
    /// Allowed base paths.
    allowed_paths: Vec<PathBuf>,
    /// Denied paths.
    denied_paths: Vec<PathBuf>,
    /// Allow absolute paths.
    allow_absolute: bool,
    /// Allow path traversal (../).
    allow_traversal: bool,
    /// Maximum path length.
    max_length: usize,
}

impl Default for PathValidator {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            denied_paths: vec![
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/root"),
            ],
            allow_absolute: true,
            allow_traversal: false,
            max_length: 4096,
        }
    }
}

impl PathValidator {
    /// Create a new path validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an allowed path.
    pub fn allow(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Add a denied path.
    pub fn deny(mut self, path: impl Into<PathBuf>) -> Self {
        self.denied_paths.push(path.into());
        self
    }

    /// Disallow absolute paths.
    pub fn no_absolute(mut self) -> Self {
        self.allow_absolute = false;
        self
    }

    /// Allow path traversal.
    pub fn allow_traversal(mut self) -> Self {
        self.allow_traversal = true;
        self
    }

    /// Validate a path string.
    pub fn validate(&self, path: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let path = Path::new(path);

        // Check length
        if path.as_os_str().len() > self.max_length {
            errors.add(ValidationError::new(
                "path",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
        }

        // Check absolute
        if !self.allow_absolute && path.is_absolute() {
            errors.add(ValidationError::new(
                "path",
                "absolute paths are not allowed",
                "no_absolute",
            ).with_suggestion("Use a relative path instead"));
        }

        // Check traversal
        if !self.allow_traversal {
            for component in path.components() {
                if matches!(component, Component::ParentDir) {
                    errors.add(ValidationError::new(
                        "path",
                        "path traversal (../) is not allowed",
                        "no_traversal",
                    ).with_suggestion("Use an absolute path or stay within the working directory"));
                    break;
                }
            }
        }

        // Check denied paths
        let canonical = self.normalize_path(path);
        for denied in &self.denied_paths {
            if canonical.starts_with(denied) {
                errors.add(ValidationError::new(
                    "path",
                    &format!("access to {:?} is denied", denied),
                    "denied_path",
                ));
            }
        }

        // Check allowed paths (if any specified)
        if !self.allowed_paths.is_empty() {
            let is_allowed = self.allowed_paths.iter().any(|allowed| {
                canonical.starts_with(allowed)
            });
            if !is_allowed {
                errors.add(ValidationError::new(
                    "path",
                    "path is not in allowed directories",
                    "allowed_path",
                ).with_suggestion(&format!(
                    "Allowed paths: {:?}",
                    self.allowed_paths
                )));
            }
        }

        errors
    }

    /// Normalize a path for comparison.
    fn normalize_path(&self, path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    normalized.pop();
                }
                Component::CurDir => {}
                _ => {
                    normalized.push(component);
                }
            }
        }
        normalized
    }

    /// Validate and resolve a path relative to a base.
    pub fn validate_and_resolve(
        &self,
        path: &str,
        base: &Path,
    ) -> Result<PathBuf, ValidationErrors> {
        let errors = self.validate(path);
        if !errors.is_empty() {
            return Err(errors);
        }

        let path = Path::new(path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        };

        // Re-validate resolved path
        let errors = self.validate(resolved.to_string_lossy().as_ref());
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(resolved)
    }
}

/// Check for path traversal attempts.
pub fn has_path_traversal(path: &str) -> bool {
    let path = Path::new(path);
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Sanitize a filename (remove directory components).
pub fn sanitize_filename(name: &str) -> String {
    Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let validator = PathValidator::new();
        let errors = validator.validate("src/main.rs");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_traversal_detection() {
        let validator = PathValidator::new();
        let errors = validator.validate("../../../etc/passwd");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_denied_path() {
        let validator = PathValidator::new();
        let errors = validator.validate("/etc/shadow");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_allowed_paths() {
        let validator = PathValidator::new()
            .allow("/project");

        let errors = validator.validate("/project/src/main.rs");
        assert!(errors.is_empty());

        let errors = validator.validate("/other/file.txt");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file.txt"), "file.txt");
        assert_eq!(sanitize_filename("/path/to/file.txt"), "file.txt");
        assert_eq!(sanitize_filename("../file.txt"), "file.txt");
    }
}
```

### 3. Pattern Validation (src/validation/pattern.rs)

```rust
//! Regex pattern validation.

use super::{ValidationError, ValidationErrors};

/// Pattern validator for regex.
pub struct PatternValidator {
    /// Maximum pattern length.
    max_length: usize,
    /// Disallowed patterns (e.g., catastrophic backtracking).
    disallowed: Vec<String>,
}

impl Default for PatternValidator {
    fn default() -> Self {
        Self {
            max_length: 1000,
            disallowed: vec![
                // Patterns that can cause catastrophic backtracking
                r"(a+)+".to_string(),
                r"(a*)*".to_string(),
                r"(a|a)+".to_string(),
            ],
        }
    }
}

impl PatternValidator {
    /// Create a new pattern validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a regex pattern.
    pub fn validate(&self, pattern: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        // Check length
        if pattern.len() > self.max_length {
            errors.add(ValidationError::new(
                "pattern",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
            return errors;
        }

        // Check if it compiles
        match regex::Regex::new(pattern) {
            Ok(_) => {}
            Err(e) => {
                errors.add(ValidationError::new(
                    "pattern",
                    &format!("invalid regex: {}", e),
                    "valid_regex",
                ));
                return errors;
            }
        }

        // Check for potentially dangerous patterns
        for disallowed in &self.disallowed {
            if pattern.contains(disallowed) {
                errors.add(ValidationError::new(
                    "pattern",
                    "pattern may cause performance issues",
                    "safe_pattern",
                ).with_suggestion("Avoid nested repetition operators"));
            }
        }

        errors
    }

    /// Validate and compile a pattern.
    pub fn validate_and_compile(&self, pattern: &str) -> Result<regex::Regex, ValidationErrors> {
        let errors = self.validate(pattern);
        if !errors.is_empty() {
            return Err(errors);
        }

        regex::Regex::new(pattern).map_err(|e| {
            let mut errors = ValidationErrors::new();
            errors.add(ValidationError::new(
                "pattern",
                &format!("failed to compile: {}", e),
                "compile",
            ));
            errors
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pattern() {
        let validator = PatternValidator::new();
        let errors = validator.validate(r"fn\s+\w+");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_invalid_pattern() {
        let validator = PatternValidator::new();
        let errors = validator.validate(r"[invalid");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_pattern_too_long() {
        let validator = PatternValidator::new();
        let long_pattern = "a".repeat(2000);
        let errors = validator.validate(&long_pattern);
        assert!(!errors.is_empty());
    }
}
```

---

## Testing Requirements

1. Path traversal attacks are blocked
2. Denied paths are enforced
3. Allowed paths whitelist works
4. Invalid regex patterns are rejected
5. Pattern length limits work
6. Command injection is blocked
7. Validation errors are informative
8. Validated wrapper type works

---

## Related Specs

- Depends on: [046-primitives-trait.md](046-primitives-trait.md)
- Next: [048-primitives-audit.md](048-primitives-audit.md)
- Related: [033-read-file-errors.md](033-read-file-errors.md)
