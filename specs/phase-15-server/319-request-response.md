# 319 - Request Response

**Phase:** 15 - Server
**Spec ID:** 319
**Status:** Planned
**Dependencies:** 317-axum-router, 318-api-versioning
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define standardized request and response structures for the API, including pagination, filtering, sorting, and consistent error response formats.

---

## Acceptance Criteria

- [ ] Standard response envelope
- [ ] Pagination response structure
- [ ] Error response format
- [ ] Request DTOs with validation
- [ ] Response builder utilities
- [ ] Serialization customization
- [ ] HATEOAS links support

---

## Implementation Details

### 1. Response Types (crates/tachikoma-server/src/response/types.rs)

```rust
//! Standard API response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API response envelope.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// Response data (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error information (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// Response metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Error information in responses.
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    /// Error code (machine-readable).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Field-specific validation errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, Vec<String>>>,
}

/// Response metadata.
#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    /// Request ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Response timestamp.
    pub timestamp: String,
    /// API version used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::now()),
        }
    }

    /// Create a successful response with metadata.
    pub fn success_with_meta(data: T, meta: ResponseMeta) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(meta),
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: code.into(),
                message: message.into(),
                details: None,
                fields: None,
            }),
            meta: Some(ResponseMeta::now()),
        }
    }

    /// Create an error response with field errors.
    pub fn validation_error(fields: HashMap<String, Vec<String>>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: "validation_error".into(),
                message: "Validation failed".into(),
                details: None,
                fields: Some(fields),
            }),
            meta: Some(ResponseMeta::now()),
        }
    }
}

impl ResponseMeta {
    /// Create metadata with current timestamp.
    pub fn now() -> Self {
        Self {
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            api_version: None,
        }
    }

    /// Add request ID.
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Add API version.
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }
}
```

### 2. Pagination Types (crates/tachikoma-server/src/response/pagination.rs)

```rust
//! Pagination support for list endpoints.

use serde::{Deserialize, Serialize};

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// List of items.
    pub items: Vec<T>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
    /// Optional HATEOAS links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<PaginationLinks>,
}

/// Pagination metadata.
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed).
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of items.
    pub total_items: u64,
    /// Total number of pages.
    pub total_pages: u32,
    /// Whether there's a next page.
    pub has_next: bool,
    /// Whether there's a previous page.
    pub has_prev: bool,
}

/// HATEOAS pagination links.
#[derive(Debug, Serialize)]
pub struct PaginationLinks {
    /// Link to current page.
    #[serde(rename = "self")]
    pub current: String,
    /// Link to first page.
    pub first: String,
    /// Link to last page.
    pub last: String,
    /// Link to next page (if exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    /// Link to previous page (if exists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response.
    pub fn new(items: Vec<T>, page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            items,
            pagination: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
                has_next,
                has_prev,
            },
            links: None,
        }
    }

    /// Add HATEOAS links.
    pub fn with_links(mut self, base_url: &str) -> Self {
        let pagination = &self.pagination;
        let build_url = |p: u32| format!("{}?page={}&per_page={}", base_url, p, pagination.per_page);

        self.links = Some(PaginationLinks {
            current: build_url(pagination.page),
            first: build_url(1),
            last: build_url(pagination.total_pages.max(1)),
            next: if pagination.has_next {
                Some(build_url(pagination.page + 1))
            } else {
                None
            },
            prev: if pagination.has_prev {
                Some(build_url(pagination.page - 1))
            } else {
                None
            },
        });

        self
    }
}

/// Pagination request parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl PaginationParams {
    /// Get the offset for database queries.
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) * self.per_page) as u64
    }

    /// Get the limit, capped at maximum.
    pub fn limit(&self) -> u32 {
        self.per_page.min(100)
    }
}
```

### 3. Request DTOs (crates/tachikoma-server/src/request/mod.rs)

```rust
//! Request data transfer objects.

use serde::Deserialize;
use validator::Validate;

/// Filter parameters for list endpoints.
#[derive(Debug, Deserialize, Default)]
pub struct FilterParams {
    /// Status filter.
    #[serde(default)]
    pub status: Option<String>,
    /// Search query.
    #[serde(default)]
    pub q: Option<String>,
    /// Date from filter.
    #[serde(default)]
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    /// Date to filter.
    #[serde(default)]
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    /// Tags filter (comma-separated).
    #[serde(default)]
    pub tags: Option<String>,
}

impl FilterParams {
    /// Parse tags into a vector.
    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }
}

/// Sort parameters for list endpoints.
#[derive(Debug, Deserialize)]
pub struct SortParams {
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default)]
    pub sort_order: SortOrder,
}

fn default_sort_by() -> String { "created_at".into() }

#[derive(Debug, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Mission creation request.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMissionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    #[validate(length(min = 1))]
    pub spec_id: String,
    pub config: Option<serde_json::Value>,
}

/// Mission update request.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMissionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}
```

### 4. Response Builder (crates/tachikoma-server/src/response/builder.rs)

```rust
//! Response builder utilities.

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use super::types::{ApiResponse, ResponseMeta};

/// Builder for constructing API responses.
pub struct ResponseBuilder<T> {
    status: StatusCode,
    data: Option<T>,
    meta: ResponseMeta,
    headers: Vec<(header::HeaderName, String)>,
}

impl<T: serde::Serialize> ResponseBuilder<T> {
    /// Create a new response builder.
    pub fn new(data: T) -> Self {
        Self {
            status: StatusCode::OK,
            data: Some(data),
            meta: ResponseMeta::now(),
            headers: Vec::new(),
        }
    }

    /// Set HTTP status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Add request ID to metadata.
    pub fn request_id(mut self, id: impl Into<String>) -> Self {
        self.meta.request_id = Some(id.into());
        self
    }

    /// Add API version to metadata.
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.meta.api_version = Some(version.into());
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: header::HeaderName, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    /// Build the response.
    pub fn build(self) -> Response {
        let response = ApiResponse::success_with_meta(self.data.unwrap(), self.meta);
        let mut res = (self.status, Json(response)).into_response();

        for (name, value) in self.headers {
            if let Ok(v) = value.parse() {
                res.headers_mut().insert(name, v);
            }
        }

        res
    }
}

/// Create a 201 Created response.
pub fn created<T: serde::Serialize>(data: T) -> Response {
    ResponseBuilder::new(data)
        .status(StatusCode::CREATED)
        .build()
}

/// Create a 204 No Content response.
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// Create a 202 Accepted response.
pub fn accepted<T: serde::Serialize>(data: T) -> Response {
    ResponseBuilder::new(data)
        .status(StatusCode::ACCEPTED)
        .build()
}
```

---

## Testing Requirements

1. Response envelope serializes correctly
2. Pagination calculates correctly
3. Error responses include all fields
4. Validation errors list field errors
5. HATEOAS links generate correctly
6. Builder produces valid responses
7. Sort order converts to SQL

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [320-request-validation.md](320-request-validation.md)
- Used by: All handlers
