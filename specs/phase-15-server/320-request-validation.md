# 320 - Request Validation

**Phase:** 15 - Server
**Spec ID:** 320
**Status:** Planned
**Dependencies:** 319-request-response
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive request validation using the validator crate with custom validators, providing detailed error messages for invalid input.

---

## Acceptance Criteria

- [x] Validator integration with Axum
- [x] Custom validation rules
- [x] Field-level error messages
- [x] Nested object validation
- [x] Array validation
- [x] Custom validators for domain types
- [x] i18n support for error messages

---

## Implementation Details

### 1. Validation Extractor (crates/tachikoma-server/src/validation/extractor.rs)

```rust
//! Validated JSON extractor.

use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

use crate::response::types::ApiResponse;

/// Extractor that validates JSON body using validator crate.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // First, parse JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ValidationError::ParseError(e.to_string()))?;

        // Then validate
        value.validate().map_err(ValidationError::ValidationErrors)?;

        Ok(ValidatedJson(value))
    }
}

/// Validation error type.
pub enum ValidationError {
    ParseError(String),
    ValidationErrors(ValidationErrors),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        match self {
            ValidationError::ParseError(msg) => {
                let response = ApiResponse::<()>::error("parse_error", msg);
                (StatusCode::BAD_REQUEST, Json(response)).into_response()
            }
            ValidationError::ValidationErrors(errors) => {
                let fields = format_validation_errors(&errors);
                let response = ApiResponse::<()>::validation_error(fields);
                (StatusCode::UNPROCESSABLE_ENTITY, Json(response)).into_response()
            }
        }
    }
}

fn format_validation_errors(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid value for {}", field))
            })
            .collect();
        result.insert(field.to_string(), messages);
    }

    result
}
```

### 2. Custom Validators (crates/tachikoma-server/src/validation/validators.rs)

```rust
//! Custom validation functions.

use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

/// Validate a spec ID format.
pub fn validate_spec_id(spec_id: &str) -> Result<(), ValidationError> {
    static SPEC_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^spc_[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$").unwrap()
    });

    if SPEC_ID_REGEX.is_match(spec_id) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_spec_id");
        error.message = Some("Invalid spec ID format".into());
        Err(error)
    }
}

/// Validate a mission ID format.
pub fn validate_mission_id(mission_id: &str) -> Result<(), ValidationError> {
    static MISSION_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^msn_[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$").unwrap()
    });

    if MISSION_ID_REGEX.is_match(mission_id) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_mission_id");
        error.message = Some("Invalid mission ID format".into());
        Err(error)
    }
}

/// Validate a URL.
pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    match url::Url::parse(url) {
        Ok(parsed) => {
            if parsed.scheme() == "http" || parsed.scheme() == "https" {
                Ok(())
            } else {
                let mut error = ValidationError::new("invalid_url_scheme");
                error.message = Some("URL must use http or https scheme".into());
                Err(error)
            }
        }
        Err(_) => {
            let mut error = ValidationError::new("invalid_url");
            error.message = Some("Invalid URL format".into());
            Err(error)
        }
    }
}

/// Validate JSON schema.
pub fn validate_json_config(config: &serde_json::Value) -> Result<(), ValidationError> {
    if config.is_object() {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_config");
        error.message = Some("Configuration must be a JSON object".into());
        Err(error)
    }
}

/// Validate positive integer.
pub fn validate_positive(value: i64) -> Result<(), ValidationError> {
    if value > 0 {
        Ok(())
    } else {
        let mut error = ValidationError::new("must_be_positive");
        error.message = Some("Value must be positive".into());
        Err(error)
    }
}

/// Validate within range.
pub fn validate_range(value: i64, min: i64, max: i64) -> Result<(), ValidationError> {
    if value >= min && value <= max {
        Ok(())
    } else {
        let mut error = ValidationError::new("out_of_range");
        error.message = Some(format!("Value must be between {} and {}", min, max).into());
        Err(error)
    }
}

/// Validate no special characters (alphanumeric + underscore + hyphen only).
pub fn validate_identifier(value: &str) -> Result<(), ValidationError> {
    static IDENTIFIER_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap()
    });

    if IDENTIFIER_REGEX.is_match(value) {
        Ok(())
    } else {
        let mut error = ValidationError::new("invalid_identifier");
        error.message = Some("Must start with a letter and contain only alphanumeric characters, underscores, or hyphens".into());
        Err(error)
    }
}
```

### 3. Validated Request Types (crates/tachikoma-server/src/validation/requests.rs)

```rust
//! Validated request types.

use serde::Deserialize;
use validator::Validate;

use super::validators::*;

/// Create mission request with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMissionRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: String,

    #[validate(length(max = 2000, message = "Description cannot exceed 2000 characters"))]
    #[serde(default)]
    pub description: Option<String>,

    #[validate(custom(function = "validate_spec_id"))]
    pub spec_id: String,

    #[validate(custom(function = "validate_json_config"))]
    #[serde(default)]
    pub config: Option<serde_json::Value>,

    #[validate(length(max = 10, message = "Maximum 10 tags allowed"))]
    #[validate]
    #[serde(default)]
    pub tags: Vec<TagInput>,
}

/// Tag input with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct TagInput {
    #[validate(length(min = 1, max = 50, message = "Tag must be between 1 and 50 characters"))]
    #[validate(custom(function = "validate_identifier"))]
    pub name: String,

    #[validate(length(max = 100, message = "Tag value cannot exceed 100 characters"))]
    #[serde(default)]
    pub value: Option<String>,
}

/// Update mission request with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMissionRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: Option<String>,

    #[validate(length(max = 2000, message = "Description cannot exceed 2000 characters"))]
    pub description: Option<String>,

    #[validate(custom(function = "validate_json_config"))]
    pub config: Option<serde_json::Value>,
}

/// Create spec request with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateSpecRequest {
    #[validate(length(min = 1, max = 200, message = "Title must be between 1 and 200 characters"))]
    pub title: String,

    #[validate(length(min = 1, message = "Content is required"))]
    pub content: String,

    #[validate(range(min = 1, max = 100, message = "Phase must be between 1 and 100"))]
    pub phase: i32,

    #[validate(length(max = 20, message = "Maximum 20 dependencies allowed"))]
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Login request with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Registration request with validation.
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,

    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,

    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub password_confirm: String,
}

/// Validate password strength.
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if has_uppercase && has_lowercase && has_digit && has_special {
        Ok(())
    } else {
        let mut error = validator::ValidationError::new("weak_password");
        error.message = Some("Password must contain uppercase, lowercase, number, and special character".into());
        Err(error)
    }
}
```

### 4. Query Validation (crates/tachikoma-server/src/validation/query.rs)

```rust
//! Query parameter validation.

use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::extractor::ValidationError;

/// Validated query parameters extractor.
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ValidationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|e| ValidationError::ParseError(e.to_string()))?;

        value.validate().map_err(ValidationError::ValidationErrors)?;

        Ok(ValidatedQuery(value))
    }
}

/// Validated list query parameters.
#[derive(Debug, serde::Deserialize, Validate)]
pub struct ListQueryParams {
    #[validate(range(min = 1, max = 1000, message = "Page must be between 1 and 1000"))]
    #[serde(default = "default_page")]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "Per page must be between 1 and 100"))]
    #[serde(default = "default_per_page")]
    pub per_page: u32,

    #[validate(length(max = 50, message = "Sort field name too long"))]
    #[serde(default = "default_sort_by")]
    pub sort_by: String,

    #[serde(default)]
    pub sort_order: SortOrder,

    #[validate(length(max = 100, message = "Search query too long"))]
    pub q: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }
fn default_sort_by() -> String { "created_at".into() }

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}
```

---

## Testing Requirements

1. Valid requests pass validation
2. Invalid field returns field error
3. Multiple errors collected
4. Custom validators work
5. Nested validation works
6. Query params validated
7. Error messages are helpful

---

## Related Specs

- Depends on: [319-request-response.md](319-request-response.md)
- Next: [321-error-response.md](321-error-response.md)
- Used by: All handlers
