# Spec 328: Request Validation

## Phase
15 - Server/API Layer

## Spec ID
328

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 315: Error Handling

## Estimated Context
~9%

---

## Objective

Implement comprehensive request validation for the Tachikoma API, providing type-safe input validation, sanitization, and detailed error messages using the validator crate and custom validation rules.

---

## Acceptance Criteria

- [ ] All request bodies are validated against schemas
- [ ] Path and query parameters are validated
- [ ] Custom validators for domain-specific rules
- [ ] Validation errors include field-level details
- [ ] Input sanitization for security
- [ ] Async validation for database checks
- [ ] Validation middleware integration

---

## Implementation Details

### Validation Traits and Macros

```rust
// src/server/validation/mod.rs
pub mod rules;
pub mod sanitize;
pub mod extractor;

use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};

use crate::server::error::{ApiError, FieldError};

/// Validate and return errors as ApiError
pub fn validate_request<T: Validate>(request: &T) -> Result<(), ApiError> {
    request.validate().map_err(|e| {
        ApiError::Validation {
            errors: validation_errors_to_field_errors(e),
        }
    })
}

/// Convert validator errors to our FieldError format
pub fn validation_errors_to_field_errors(errors: ValidationErrors) -> Vec<FieldError> {
    let mut field_errors = Vec::new();

    for (field, errs) in errors.field_errors() {
        for err in errs {
            field_errors.push(FieldError {
                field: field.to_string(),
                message: err.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Validation failed for {}", field)),
                code: Some(err.code.to_string()),
            });
        }
    }

    // Handle nested errors
    for (field, nested) in errors.errors() {
        if let validator::ValidationErrorsKind::Struct(box_errors) = nested {
            for nested_error in validation_errors_to_field_errors(*box_errors.clone()) {
                field_errors.push(FieldError {
                    field: format!("{}.{}", field, nested_error.field),
                    message: nested_error.message,
                    code: nested_error.code,
                });
            }
        }
    }

    field_errors
}
```

### Custom Validators

```rust
// src/server/validation/rules.rs
use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

/// Validate UUID format
pub fn validate_uuid(value: &str) -> Result<(), ValidationError> {
    if uuid::Uuid::parse_str(value).is_ok() {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_uuid");
        err.message = Some("Invalid UUID format".into());
        Err(err)
    }
}

/// Validate spec ID format (e.g., "311", "311a")
static SPEC_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{3}[a-z]?$").unwrap()
});

pub fn validate_spec_id(value: &str) -> Result<(), ValidationError> {
    if SPEC_ID_REGEX.is_match(value) {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_spec_id");
        err.message = Some("Spec ID must be 3 digits optionally followed by a letter".into());
        Err(err)
    }
}

/// Validate file path (no directory traversal)
pub fn validate_file_path(value: &str) -> Result<(), ValidationError> {
    if value.contains("..") || value.starts_with('/') || value.contains('\0') {
        let mut err = ValidationError::new("invalid_path");
        err.message = Some("Invalid file path".into());
        Err(err)
    } else {
        Ok(())
    }
}

/// Validate URL format
pub fn validate_url(value: &str) -> Result<(), ValidationError> {
    if url::Url::parse(value).is_ok() {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_url");
        err.message = Some("Invalid URL format".into());
        Err(err)
    }
}

/// Validate JSON content
pub fn validate_json(value: &str) -> Result<(), ValidationError> {
    if serde_json::from_str::<serde_json::Value>(value).is_ok() {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_json");
        err.message = Some("Invalid JSON format".into());
        Err(err)
    }
}

/// Validate slug format (lowercase, alphanumeric, hyphens)
static SLUG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap()
});

pub fn validate_slug(value: &str) -> Result<(), ValidationError> {
    if SLUG_REGEX.is_match(value) {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_slug");
        err.message = Some("Must be lowercase alphanumeric with hyphens".into());
        Err(err)
    }
}

/// Validate semver version
pub fn validate_semver(value: &str) -> Result<(), ValidationError> {
    if semver::Version::parse(value).is_ok() {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_semver");
        err.message = Some("Invalid semantic version format".into());
        Err(err)
    }
}

/// Validate Git branch name
static BRANCH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9/_-]+$").unwrap()
});

pub fn validate_branch_name(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || !BRANCH_REGEX.is_match(value) {
        let mut err = ValidationError::new("invalid_branch");
        err.message = Some("Invalid Git branch name".into());
        Err(err)
    } else {
        Ok(())
    }
}

/// Validate status transition
pub fn validate_status_transition(
    from: &str,
    to: &str,
    allowed: &[(String, String)],
) -> Result<(), ValidationError> {
    let transition = (from.to_string(), to.to_string());

    if allowed.contains(&transition) || from == to {
        Ok(())
    } else {
        let mut err = ValidationError::new("invalid_transition");
        err.message = Some(format!("Cannot transition from {} to {}", from, to).into());
        Err(err)
    }
}
```

### Input Sanitization

```rust
// src/server/validation/sanitize.rs
use ammonia::Builder;
use once_cell::sync::Lazy;

static HTML_CLEANER: Lazy<Builder<'static>> = Lazy::new(|| {
    Builder::default()
});

/// Sanitize HTML content
pub fn sanitize_html(input: &str) -> String {
    HTML_CLEANER.clean(input).to_string()
}

/// Sanitize string for safe usage (trim, normalize whitespace)
pub fn sanitize_string(input: &str) -> String {
    input
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sanitize file name
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

/// Sanitize path components
pub fn sanitize_path(input: &str) -> String {
    input
        .split('/')
        .filter(|p| !p.is_empty() && *p != "." && *p != "..")
        .collect::<Vec<_>>()
        .join("/")
}

/// Normalize and validate email
pub fn normalize_email(input: &str) -> Option<String> {
    let normalized = input.trim().to_lowercase();

    if validator::validate_email(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

/// Truncate string to max length
pub fn truncate(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else {
        format!("{}...", &input[..max_len.saturating_sub(3)])
    }
}
```

### Validated Request Extractor

```rust
// src/server/validation/extractor.rs
use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::server::error::{ApiError, ErrorResponse};

/// Validated JSON extractor
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Extract JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e: JsonRejection| {
                ApiError::bad_request(format!("Invalid JSON: {}", e))
            })?;

        // Validate
        value.validate().map_err(|e| {
            ApiError::Validation {
                errors: super::validation_errors_to_field_errors(e),
            }
        })?;

        Ok(ValidatedJson(value))
    }
}

/// Validated query parameters extractor
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(value) =
            axum::extract::Query::<T>::from_request(req, state)
                .await
                .map_err(|e| {
                    ApiError::bad_request(format!("Invalid query parameters: {}", e))
                })?;

        value.validate().map_err(|e| {
            ApiError::Validation {
                errors: super::validation_errors_to_field_errors(e),
            }
        })?;

        Ok(ValidatedQuery(value))
    }
}

/// Validated path parameters extractor
pub struct ValidatedPath<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedPath<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(value) =
            axum::extract::Path::<T>::from_request(req, state)
                .await
                .map_err(|e| {
                    ApiError::bad_request(format!("Invalid path parameters: {}", e))
                })?;

        value.validate().map_err(|e| {
            ApiError::Validation {
                errors: super::validation_errors_to_field_errors(e),
            }
        })?;

        Ok(ValidatedPath(value))
    }
}
```

### Async Validators

```rust
// src/server/validation/async_validators.rs
use uuid::Uuid;

use crate::server::state::AppState;
use crate::server::error::ApiError;

/// Validate that a mission exists
pub async fn validate_mission_exists(
    state: &AppState,
    mission_id: Uuid,
) -> Result<(), ApiError> {
    state.storage()
        .missions()
        .exists(mission_id)
        .await
        .map_err(|_| ApiError::not_found_with_id("Mission", mission_id.to_string()))?
        .then_some(())
        .ok_or_else(|| ApiError::not_found_with_id("Mission", mission_id.to_string()))
}

/// Validate that a spec exists
pub async fn validate_spec_exists(
    state: &AppState,
    spec_id: Uuid,
) -> Result<(), ApiError> {
    state.storage()
        .specs()
        .exists(spec_id)
        .await
        .map_err(|_| ApiError::not_found_with_id("Spec", spec_id.to_string()))?
        .then_some(())
        .ok_or_else(|| ApiError::not_found_with_id("Spec", spec_id.to_string()))
}

/// Validate that a backend exists
pub async fn validate_backend_exists(
    state: &AppState,
    backend_id: Uuid,
) -> Result<(), ApiError> {
    state.backend_manager()
        .get(backend_id)
        .ok_or_else(|| ApiError::not_found_with_id("Backend", backend_id.to_string()))?;

    Ok(())
}

/// Validate unique constraint
pub async fn validate_unique_name(
    state: &AppState,
    table: &str,
    name: &str,
    exclude_id: Option<Uuid>,
) -> Result<(), ApiError> {
    let exists = state.storage()
        .raw()
        .name_exists(table, name, exclude_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    if exists {
        Err(ApiError::Conflict {
            message: format!("{} with this name already exists", table),
        })
    } else {
        Ok(())
    }
}

/// Validate dependencies are met
pub async fn validate_dependencies_completed(
    state: &AppState,
    spec_id: Uuid,
) -> Result<(), ApiError> {
    let deps = state.storage()
        .specs()
        .get_dependencies(spec_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let incomplete: Vec<_> = deps
        .iter()
        .filter(|d| d.status != SpecStatus::Completed)
        .collect();

    if incomplete.is_empty() {
        Ok(())
    } else {
        Err(ApiError::UnprocessableEntity {
            message: format!(
                "Dependencies not completed: {}",
                incomplete.iter().map(|d| d.spec_id.as_str()).collect::<Vec<_>>().join(", ")
            ),
        })
    }
}
```

### Example Validated Request Types

```rust
// src/api/types/validated.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::server::validation::rules::*;

/// Validated mission creation request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMissionRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,

    #[validate(length(max = 2000, message = "Description cannot exceed 2000 characters"))]
    pub description: Option<String>,

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    #[validate]
    pub tags: Option<Vec<ValidatedTag>>,

    pub template_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ValidatedTag {
    #[validate(length(min = 1, max = 50, message = "Tag must be 1-50 characters"))]
    #[validate(custom = "validate_slug")]
    pub value: String,
}

/// Validated spec creation request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateSpecRequest {
    pub phase_id: Uuid,

    #[validate(custom = "validate_spec_id")]
    pub spec_id: String,

    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 10000))]
    pub description: Option<String>,

    #[validate(length(max = 50000))]
    pub acceptance_criteria: Option<String>,

    #[validate(range(min = 0.0, max = 100.0, message = "Context must be 0-100%"))]
    pub estimated_context: Option<f32>,

    #[validate(length(max = 50))]
    pub dependencies: Option<Vec<Uuid>>,
}

/// Validated file write request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct WriteFileRequest {
    #[validate(length(min = 1, max = 1000))]
    #[validate(custom = "validate_file_path")]
    pub path: String,

    #[validate(length(max = 10485760, message = "Content cannot exceed 10MB"))]
    pub content: String,

    pub encoding: Option<FileEncoding>,

    #[serde(default)]
    pub create_parents: bool,
}

/// Validated pagination parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    #[serde(default = "default_page")]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "Per page must be 1-100"))]
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_spec_id() {
        assert!(validate_spec_id("311").is_ok());
        assert!(validate_spec_id("311a").is_ok());
        assert!(validate_spec_id("31").is_err());
        assert!(validate_spec_id("3111").is_err());
        assert!(validate_spec_id("abc").is_err());
    }

    #[test]
    fn test_validate_file_path() {
        assert!(validate_file_path("src/main.rs").is_ok());
        assert!(validate_file_path("../secret").is_err());
        assert!(validate_file_path("/etc/passwd").is_err());
    }

    #[test]
    fn test_sanitize_path() {
        assert_eq!(sanitize_path("../foo/../bar"), "foo/bar");
        assert_eq!(sanitize_path("./test/./file"), "test/file");
    }

    #[test]
    fn test_validated_request() {
        let request = CreateMissionRequest {
            name: "Test Mission".to_string(),
            description: None,
            tags: None,
            template_id: None,
        };

        assert!(request.validate().is_ok());

        let invalid = CreateMissionRequest {
            name: "".to_string(),
            ..request
        };

        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_validation_error_conversion() {
        let request = CreateMissionRequest {
            name: "".to_string(),
            description: Some("x".repeat(3000)),
            tags: None,
            template_id: None,
        };

        let errors = request.validate().unwrap_err();
        let field_errors = validation_errors_to_field_errors(errors);

        assert!(!field_errors.is_empty());
        assert!(field_errors.iter().any(|e| e.field == "name"));
    }
}
```

---

## Related Specs

- **Spec 315**: Error Handling
- **Spec 317**: Missions API
- **Spec 318**: Specs API
