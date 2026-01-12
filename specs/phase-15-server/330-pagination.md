# Spec 330: Pagination

## Phase
15 - Server/API Layer

## Spec ID
330

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 329: Response Types

## Estimated Context
~8%

---

## Objective

Implement robust pagination support for the Tachikoma API, providing both offset-based and cursor-based pagination options with proper metadata and navigation links.

---

## Acceptance Criteria

- [ ] Offset-based pagination for simple lists
- [ ] Cursor-based pagination for large datasets
- [ ] Consistent pagination parameters across endpoints
- [ ] Pagination metadata in responses
- [ ] Navigation links (first, prev, next, last)
- [ ] Configurable default and maximum page sizes
- [ ] Sorting and filtering integration

---

## Implementation Details

### Pagination Types

```rust
// src/api/pagination/types.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pagination parameters for offset-based pagination
#[derive(Debug, Clone, Deserialize)]
pub struct OffsetPagination {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

impl OffsetPagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self { page, per_page }
    }

    /// Calculate offset for SQL query
    pub fn offset(&self) -> u64 {
        ((self.page.saturating_sub(1)) as u64) * (self.per_page as u64)
    }

    /// Get limit for SQL query
    pub fn limit(&self) -> u64 {
        self.per_page as u64
    }

    /// Validate and clamp values
    pub fn validate(self, max_per_page: u32) -> Self {
        Self {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, max_per_page),
        }
    }
}

impl Default for OffsetPagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

/// Pagination parameters for cursor-based pagination
#[derive(Debug, Clone, Deserialize)]
pub struct CursorPagination {
    /// Cursor for the next page
    pub after: Option<String>,

    /// Cursor for the previous page
    pub before: Option<String>,

    /// Number of items to fetch
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 { 20 }

impl CursorPagination {
    pub fn new(limit: u32) -> Self {
        Self {
            after: None,
            before: None,
            limit,
        }
    }

    pub fn after(mut self, cursor: impl Into<String>) -> Self {
        self.after = Some(cursor.into());
        self
    }

    pub fn before(mut self, cursor: impl Into<String>) -> Self {
        self.before = Some(cursor.into());
        self
    }

    /// Decode cursor to get the ID
    pub fn decode_after(&self) -> Option<Uuid> {
        self.after.as_ref().and_then(|c| decode_cursor(c))
    }

    pub fn decode_before(&self) -> Option<Uuid> {
        self.before.as_ref().and_then(|c| decode_cursor(c))
    }
}

impl Default for CursorPagination {
    fn default() -> Self {
        Self {
            after: None,
            before: None,
            limit: 20,
        }
    }
}

/// Unified pagination enum
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Pagination {
    Offset(OffsetPagination),
    Cursor(CursorPagination),
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination::Offset(OffsetPagination::default())
    }
}
```

### Pagination Response

```rust
// src/api/pagination/response.rs
use serde::Serialize;

/// Offset pagination metadata
#[derive(Debug, Clone, Serialize)]
pub struct OffsetPaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl OffsetPaginationMeta {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Cursor pagination metadata
#[derive(Debug, Clone, Serialize)]
pub struct CursorPaginationMeta {
    pub has_next: bool,
    pub has_prev: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

impl CursorPaginationMeta {
    pub fn new(
        items: &[impl Cursorable],
        has_next: bool,
        has_prev: bool,
    ) -> Self {
        Self {
            has_next,
            has_prev,
            start_cursor: items.first().map(|i| encode_cursor(i.cursor_id())),
            end_cursor: items.last().map(|i| encode_cursor(i.cursor_id())),
        }
    }
}

/// Pagination links for HATEOAS
#[derive(Debug, Clone, Serialize)]
pub struct PaginationLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
}

impl PaginationLinks {
    pub fn for_offset(
        base_url: &str,
        page: u32,
        per_page: u32,
        total_pages: u32,
    ) -> Self {
        Self {
            first: Some(format!("{}?page=1&per_page={}", base_url, per_page)),
            prev: if page > 1 {
                Some(format!("{}?page={}&per_page={}", base_url, page - 1, per_page))
            } else {
                None
            },
            next: if page < total_pages {
                Some(format!("{}?page={}&per_page={}", base_url, page + 1, per_page))
            } else {
                None
            },
            last: Some(format!("{}?page={}&per_page={}", base_url, total_pages, per_page)),
        }
    }

    pub fn for_cursor(
        base_url: &str,
        start_cursor: Option<&str>,
        end_cursor: Option<&str>,
        has_prev: bool,
        has_next: bool,
        limit: u32,
    ) -> Self {
        Self {
            first: Some(format!("{}?limit={}", base_url, limit)),
            prev: if has_prev {
                start_cursor.map(|c| format!("{}?before={}&limit={}", base_url, c, limit))
            } else {
                None
            },
            next: if has_next {
                end_cursor.map(|c| format!("{}?after={}&limit={}", base_url, c, limit))
            } else {
                None
            },
            last: None, // Not available for cursor pagination
        }
    }
}

/// Trait for items that can be cursor-paginated
pub trait Cursorable {
    fn cursor_id(&self) -> Uuid;
}
```

### Cursor Encoding

```rust
// src/api/pagination/cursor.rs
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use uuid::Uuid;

/// Encode a UUID as a cursor string
pub fn encode_cursor(id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(id.as_bytes())
}

/// Decode a cursor string to a UUID
pub fn decode_cursor(cursor: &str) -> Option<Uuid> {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).ok()?;
    Uuid::from_slice(&bytes).ok()
}

/// Encode a composite cursor (id + timestamp)
pub fn encode_composite_cursor(id: Uuid, timestamp: i64) -> String {
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(id.as_bytes());
    data.extend_from_slice(&timestamp.to_be_bytes());
    URL_SAFE_NO_PAD.encode(&data)
}

/// Decode a composite cursor
pub fn decode_composite_cursor(cursor: &str) -> Option<(Uuid, i64)> {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).ok()?;
    if bytes.len() != 24 {
        return None;
    }

    let id = Uuid::from_slice(&bytes[..16]).ok()?;
    let timestamp = i64::from_be_bytes(bytes[16..24].try_into().ok()?);

    Some((id, timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_roundtrip() {
        let id = Uuid::new_v4();
        let cursor = encode_cursor(id);
        let decoded = decode_cursor(&cursor);

        assert_eq!(decoded, Some(id));
    }

    #[test]
    fn test_composite_cursor_roundtrip() {
        let id = Uuid::new_v4();
        let timestamp = 1234567890i64;

        let cursor = encode_composite_cursor(id, timestamp);
        let decoded = decode_composite_cursor(&cursor);

        assert_eq!(decoded, Some((id, timestamp)));
    }
}
```

### Paginated Query Builder

```rust
// src/api/pagination/query.rs
use sqlx::{QueryBuilder, Postgres};

/// Query builder for paginated queries
pub struct PaginatedQuery<'a> {
    base_query: &'a str,
    order_by: Option<String>,
    order_dir: OrderDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl<'a> PaginatedQuery<'a> {
    pub fn new(base_query: &'a str) -> Self {
        Self {
            base_query,
            order_by: None,
            order_dir: OrderDirection::Desc,
        }
    }

    pub fn order_by(mut self, column: impl Into<String>, dir: OrderDirection) -> Self {
        self.order_by = Some(column.into());
        self.order_dir = dir;
        self
    }

    /// Build query for offset pagination
    pub fn with_offset(&self, pagination: &OffsetPagination) -> String {
        let mut query = self.base_query.to_string();

        if let Some(ref order) = self.order_by {
            let dir = match self.order_dir {
                OrderDirection::Asc => "ASC",
                OrderDirection::Desc => "DESC",
            };
            query.push_str(&format!(" ORDER BY {} {}", order, dir));
        }

        query.push_str(&format!(
            " LIMIT {} OFFSET {}",
            pagination.limit(),
            pagination.offset()
        ));

        query
    }

    /// Build query for cursor pagination
    pub fn with_cursor(&self, pagination: &CursorPagination, id_column: &str) -> String {
        let mut query = self.base_query.to_string();

        // Add cursor condition
        if let Some(after_id) = pagination.decode_after() {
            query.push_str(&format!(
                " AND {} < '{}'",
                id_column, after_id
            ));
        } else if let Some(before_id) = pagination.decode_before() {
            query.push_str(&format!(
                " AND {} > '{}'",
                id_column, before_id
            ));
        }

        // Add ordering
        if let Some(ref order) = self.order_by {
            let dir = match self.order_dir {
                OrderDirection::Asc => "ASC",
                OrderDirection::Desc => "DESC",
            };
            query.push_str(&format!(" ORDER BY {} {}", order, dir));
        }

        // Fetch one extra to detect has_next
        query.push_str(&format!(" LIMIT {}", pagination.limit + 1));

        query
    }

    /// Build count query
    pub fn count_query(&self) -> String {
        format!(
            "SELECT COUNT(*) as count FROM ({}) as subquery",
            self.base_query
        )
    }
}
```

### Pagination Extractor

```rust
// src/api/pagination/extractor.rs
use axum::{
    async_trait,
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;

use super::types::{OffsetPagination, CursorPagination};
use crate::server::config::ApiConfig;

/// Validated offset pagination extractor
pub struct ValidatedOffset(pub OffsetPagination);

#[derive(Debug, Deserialize)]
struct OffsetParams {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[async_trait]
impl<S> FromRequestParts<S> for ValidatedOffset
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<OffsetParams>::from_request_parts(parts, state)
            .await
            .unwrap_or(Query(OffsetParams { page: None, per_page: None }));

        let pagination = OffsetPagination {
            page: params.page.unwrap_or(1).max(1),
            per_page: params.per_page.unwrap_or(20).clamp(1, 100),
        };

        Ok(ValidatedOffset(pagination))
    }
}

/// Validated cursor pagination extractor
pub struct ValidatedCursor(pub CursorPagination);

#[derive(Debug, Deserialize)]
struct CursorParams {
    after: Option<String>,
    before: Option<String>,
    limit: Option<u32>,
}

#[async_trait]
impl<S> FromRequestParts<S> for ValidatedCursor
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<CursorParams>::from_request_parts(parts, state)
            .await
            .unwrap_or(Query(CursorParams {
                after: None,
                before: None,
                limit: None,
            }));

        let pagination = CursorPagination {
            after: params.after,
            before: params.before,
            limit: params.limit.unwrap_or(20).clamp(1, 100),
        };

        Ok(ValidatedCursor(pagination))
    }
}
```

### Usage Example

```rust
// Example handler using pagination
use crate::api::pagination::{
    ValidatedOffset,
    OffsetPaginationMeta,
    PaginationLinks,
};

pub async fn list_missions(
    State(state): State<AppState>,
    ValidatedOffset(pagination): ValidatedOffset,
) -> ApiResult<Json<MissionListResponse>> {
    let storage = state.storage();

    // Get total count
    let total = storage.missions().count().await?;

    // Get paginated results
    let missions = storage.missions()
        .list_paginated(pagination.offset(), pagination.limit())
        .await?;

    let meta = OffsetPaginationMeta::new(
        pagination.page,
        pagination.per_page,
        total,
    );

    let links = PaginationLinks::for_offset(
        "/api/v1/missions",
        pagination.page,
        pagination.per_page,
        meta.total_pages,
    );

    Ok(Json(MissionListResponse {
        data: missions.into_iter().map(|m| m.into()).collect(),
        pagination: meta,
        links,
    }))
}

// Cursor-based example
pub async fn list_messages(
    State(state): State<AppState>,
    Path(spec_id): Path<Uuid>,
    ValidatedCursor(pagination): ValidatedCursor,
) -> ApiResult<Json<MessageListResponse>> {
    let storage = state.storage();

    // Get paginated results (fetch limit + 1 to detect has_next)
    let mut messages = storage.messages()
        .list_for_spec_cursor(
            spec_id,
            pagination.decode_after(),
            pagination.limit + 1,
        )
        .await?;

    let has_next = messages.len() > pagination.limit as usize;
    if has_next {
        messages.pop(); // Remove the extra item
    }

    let meta = CursorPaginationMeta::new(
        &messages,
        has_next,
        pagination.after.is_some(),
    );

    Ok(Json(MessageListResponse {
        data: messages.into_iter().map(|m| m.into()).collect(),
        pagination: meta,
    }))
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
    fn test_offset_calculation() {
        let pagination = OffsetPagination::new(3, 10);
        assert_eq!(pagination.offset(), 20);
        assert_eq!(pagination.limit(), 10);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = OffsetPaginationMeta::new(2, 10, 25);

        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(meta.has_prev);
    }

    #[test]
    fn test_first_page_no_prev() {
        let meta = OffsetPaginationMeta::new(1, 10, 25);
        assert!(!meta.has_prev);
    }

    #[test]
    fn test_last_page_no_next() {
        let meta = OffsetPaginationMeta::new(3, 10, 25);
        assert!(!meta.has_next);
    }

    #[test]
    fn test_pagination_links() {
        let links = PaginationLinks::for_offset("/api/items", 2, 10, 5);

        assert!(links.first.unwrap().contains("page=1"));
        assert!(links.prev.unwrap().contains("page=1"));
        assert!(links.next.unwrap().contains("page=3"));
        assert!(links.last.unwrap().contains("page=5"));
    }

    #[test]
    fn test_validation_clamps_values() {
        let pagination = OffsetPagination::new(0, 500).validate(100);

        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 100);
    }
}
```

---

## Related Specs

- **Spec 329**: Response Types
- **Spec 317**: Missions API
- **Spec 318**: Specs API
