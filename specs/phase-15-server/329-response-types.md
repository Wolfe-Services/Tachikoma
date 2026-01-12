# Spec 329: Response Types

## Phase
15 - Server/API Layer

## Spec ID
329

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 315: Error Handling

## Estimated Context
~8%

---

## Objective

Define consistent response types and formatting for the Tachikoma API, ensuring uniform response structure, proper content negotiation, and serialization customization.

---

## Acceptance Criteria

- [ ] Consistent response envelope for all endpoints
- [ ] Proper HTTP status codes for all responses
- [ ] JSON serialization with customizable options
- [ ] Support for different content types
- [ ] Response metadata (timing, version)
- [ ] Empty response handling
- [ ] Binary response support

---

## Implementation Details

### Response Envelope

```rust
// src/api/response/envelope.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// Response data
    pub data: T,

    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize)]
pub struct ResponseMeta {
    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// API version
    pub version: &'static str,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.meta = Some(ResponseMeta {
            request_id: Some(request_id.into()),
            timestamp: Utc::now(),
            duration_ms: None,
            version: env!("CARGO_PKG_VERSION"),
        });
        self
    }
}

/// List response with pagination
#[derive(Debug, Clone, Serialize)]
pub struct ListResponse<T: Serialize> {
    /// List of items
    pub data: Vec<T>,

    /// Pagination information
    pub pagination: PaginationInfo,

    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T: Serialize> ListResponse<T> {
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Self {
            data,
            pagination: PaginationInfo {
                page,
                per_page,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
            meta: None,
        }
    }
}

/// Created response with location
#[derive(Debug, Clone, Serialize)]
pub struct CreatedResponse<T: Serialize> {
    pub data: T,
    #[serde(skip)]
    pub location: String,
}

/// Empty success response
#[derive(Debug, Clone, Serialize)]
pub struct EmptyResponse {
    pub success: bool,
}

impl Default for EmptyResponse {
    fn default() -> Self {
        Self { success: true }
    }
}
```

### Response Builders

```rust
// src/api/response/builders.rs
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use super::envelope::{ApiResponse, ListResponse, CreatedResponse, ResponseMeta};

/// Response builder for standard responses
pub struct ResponseBuilder<T: Serialize> {
    data: T,
    status: StatusCode,
    headers: HeaderMap,
    meta: Option<ResponseMeta>,
}

impl<T: Serialize> ResponseBuilder<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            meta: None,
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, key: header::HeaderName, value: HeaderValue) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn cache_control(self, value: &str) -> Self {
        self.header(
            header::CACHE_CONTROL,
            HeaderValue::from_str(value).unwrap(),
        )
    }

    pub fn no_cache(self) -> Self {
        self.cache_control("no-store, no-cache, must-revalidate")
    }

    pub fn build(self) -> Response {
        let response = ApiResponse::new(self.data).with_meta(
            self.meta.unwrap_or_else(|| ResponseMeta {
                request_id: None,
                timestamp: chrono::Utc::now(),
                duration_ms: None,
                version: env!("CARGO_PKG_VERSION"),
            }),
        );

        let mut res = (self.status, Json(response)).into_response();
        res.headers_mut().extend(self.headers);
        res
    }
}

/// Response builder for created resources
pub struct CreatedBuilder<T: Serialize> {
    data: T,
    location: String,
}

impl<T: Serialize> CreatedBuilder<T> {
    pub fn new(data: T, location: impl Into<String>) -> Self {
        Self {
            data,
            location: location.into(),
        }
    }

    pub fn build(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::LOCATION,
            HeaderValue::from_str(&self.location).unwrap(),
        );

        let response = ApiResponse::new(self.data);
        let mut res = (StatusCode::CREATED, Json(response)).into_response();
        res.headers_mut().extend(headers);
        res
    }
}

/// Response builder for lists
pub struct ListBuilder<T: Serialize> {
    data: Vec<T>,
    page: u32,
    per_page: u32,
    total: u64,
}

impl<T: Serialize> ListBuilder<T> {
    pub fn new(data: Vec<T>) -> Self {
        let len = data.len();
        Self {
            data,
            page: 1,
            per_page: len as u32,
            total: len as u64,
        }
    }

    pub fn pagination(mut self, page: u32, per_page: u32, total: u64) -> Self {
        self.page = page;
        self.per_page = per_page;
        self.total = total;
        self
    }

    pub fn build(self) -> Response {
        let response = ListResponse::new(self.data, self.page, self.per_page, self.total);
        (StatusCode::OK, Json(response)).into_response()
    }
}
```

### Custom Serializers

```rust
// src/api/response/serializers.rs
use serde::{Serialize, Serializer};
use chrono::{DateTime, Utc};

/// Serialize DateTime as ISO 8601 string
pub fn serialize_datetime<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.to_rfc3339())
}

/// Serialize Option<DateTime> as ISO 8601 string
pub fn serialize_option_datetime<S>(
    date: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(d) => serializer.serialize_some(&d.to_rfc3339()),
        None => serializer.serialize_none(),
    }
}

/// Serialize bytes as base64
pub fn serialize_bytes_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(bytes))
}

/// Serialize UUID as string
pub fn serialize_uuid<S>(uuid: &uuid::Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

/// Serialize Duration as milliseconds
pub fn serialize_duration_ms<S>(
    duration: &std::time::Duration,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_millis() as u64)
}

/// Skip serializing if empty string
pub fn skip_empty_string(s: &str) -> bool {
    s.is_empty()
}

/// Skip serializing if empty vec
pub fn skip_empty_vec<T>(v: &Vec<T>) -> bool {
    v.is_empty()
}
```

### Response Types

```rust
// src/api/response/types.rs
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    body::Body,
};

/// No content response (204)
pub struct NoContent;

impl IntoResponse for NoContent {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

/// Accepted response (202)
pub struct Accepted<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Accepted<T> {
    fn into_response(self) -> Response {
        (StatusCode::ACCEPTED, Json(ApiResponse::new(self.0))).into_response()
    }
}

/// Binary response with content type
pub struct Binary {
    pub data: Vec<u8>,
    pub content_type: String,
    pub filename: Option<String>,
}

impl Binary {
    pub fn new(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            data,
            content_type: content_type.into(),
            filename: None,
        }
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }
}

impl IntoResponse for Binary {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&self.content_type).unwrap(),
        );

        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&self.data.len().to_string()).unwrap(),
        );

        if let Some(filename) = self.filename {
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
            );
        }

        let mut response = Response::new(Body::from(self.data));
        *response.headers_mut() = headers;
        response
    }
}

/// JSON download response
pub struct JsonDownload<T: Serialize> {
    pub data: T,
    pub filename: String,
}

impl<T: Serialize> IntoResponse for JsonDownload<T> {
    fn into_response(self) -> Response {
        let json = serde_json::to_string_pretty(&self.data).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", self.filename)).unwrap(),
        );

        let mut response = Response::new(Body::from(json));
        *response.headers_mut() = headers;
        response
    }
}

/// Redirect response
pub struct Redirect {
    pub location: String,
    pub permanent: bool,
}

impl Redirect {
    pub fn temporary(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            permanent: false,
        }
    }

    pub fn permanent(location: impl Into<String>) -> Self {
        Self {
            location: location.into(),
            permanent: true,
        }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        let status = if self.permanent {
            StatusCode::PERMANENT_REDIRECT
        } else {
            StatusCode::TEMPORARY_REDIRECT
        };

        let mut response = status.into_response();
        response.headers_mut().insert(
            header::LOCATION,
            HeaderValue::from_str(&self.location).unwrap(),
        );
        response
    }
}
```

### Content Negotiation

```rust
// src/api/response/negotiate.rs
use axum::{
    extract::Request,
    http::header,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Content types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Yaml,
    Xml,
    Html,
}

impl ContentType {
    pub fn from_accept_header(accept: &str) -> Self {
        if accept.contains("application/yaml") || accept.contains("text/yaml") {
            ContentType::Yaml
        } else if accept.contains("application/xml") || accept.contains("text/xml") {
            ContentType::Xml
        } else if accept.contains("text/html") {
            ContentType::Html
        } else {
            ContentType::Json
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            ContentType::Json => "application/json",
            ContentType::Yaml => "application/yaml",
            ContentType::Xml => "application/xml",
            ContentType::Html => "text/html",
        }
    }
}

/// Negotiated response that serializes based on Accept header
pub struct Negotiated<T: Serialize> {
    pub data: T,
    pub content_type: ContentType,
}

impl<T: Serialize> Negotiated<T> {
    pub fn new(data: T, accept: &str) -> Self {
        Self {
            data,
            content_type: ContentType::from_accept_header(accept),
        }
    }
}

impl<T: Serialize> IntoResponse for Negotiated<T> {
    fn into_response(self) -> Response {
        let body = match self.content_type {
            ContentType::Json => serde_json::to_string(&self.data).unwrap(),
            ContentType::Yaml => serde_yaml::to_string(&self.data).unwrap_or_default(),
            ContentType::Xml => {
                // Simple XML serialization (would use quick-xml in production)
                serde_json::to_string(&self.data).unwrap()
            }
            ContentType::Html => {
                // HTML representation
                format!("<pre>{}</pre>", serde_json::to_string_pretty(&self.data).unwrap())
            }
        };

        let mut response = Response::new(body.into());
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static(self.content_type.mime_type()),
        );
        response
    }
}
```

### Handler Helpers

```rust
// src/api/response/helpers.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use super::envelope::ApiResponse;

/// Return OK with data
pub fn ok<T: Serialize>(data: T) -> Response {
    (StatusCode::OK, Json(ApiResponse::new(data))).into_response()
}

/// Return Created with data and location
pub fn created<T: Serialize>(data: T, location: &str) -> Response {
    super::builders::CreatedBuilder::new(data, location).build()
}

/// Return No Content
pub fn no_content() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

/// Return Accepted
pub fn accepted<T: Serialize>(data: T) -> Response {
    (StatusCode::ACCEPTED, Json(ApiResponse::new(data))).into_response()
}

/// Return list with pagination
pub fn list<T: Serialize>(data: Vec<T>, page: u32, per_page: u32, total: u64) -> Response {
    super::builders::ListBuilder::new(data)
        .pagination(page, per_page, total)
        .build()
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::new("test data");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("data"));
        assert!(json.contains("test data"));
    }

    #[test]
    fn test_list_response_pagination() {
        let response = ListResponse::new(
            vec!["a", "b", "c"],
            2,  // page
            10, // per_page
            25, // total
        );

        assert_eq!(response.pagination.total_pages, 3);
        assert!(response.pagination.has_next);
        assert!(response.pagination.has_prev);
    }

    #[test]
    fn test_content_negotiation() {
        assert_eq!(
            ContentType::from_accept_header("application/json"),
            ContentType::Json
        );
        assert_eq!(
            ContentType::from_accept_header("application/yaml"),
            ContentType::Yaml
        );
        assert_eq!(
            ContentType::from_accept_header("text/html"),
            ContentType::Html
        );
    }

    #[test]
    fn test_binary_response() {
        let binary = Binary::new(vec![1, 2, 3], "application/octet-stream")
            .with_filename("test.bin");

        assert_eq!(binary.filename, Some("test.bin".to_string()));
    }
}
```

---

## Related Specs

- **Spec 315**: Error Handling
- **Spec 330**: Pagination
- **Spec 317**: Missions API
