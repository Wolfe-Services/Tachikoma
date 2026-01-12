# 435 - Audit Query

**Phase:** 20 - Audit System
**Spec ID:** 435
**Status:** Planned
**Dependencies:** 434-audit-persistence
**Estimated Context:** ~14% of Sonnet window

---

## Objective

Implement a flexible query interface for retrieving audit events with filtering, pagination, and aggregation capabilities.

---

## Acceptance Criteria

- [ ] Query builder pattern for audit queries
- [ ] Pagination support (cursor and offset)
- [ ] Filtering by all event fields
- [ ] Aggregation queries (counts, summaries)
- [ ] Full-text search on metadata

---

## Implementation Details

### 1. Query Types (src/query.rs)

```rust
//! Audit query types and builders.

use crate::{AuditCategory, AuditSeverity, AuditAction, AuditEvent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Query result page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPage<T> {
    /// Items in this page.
    pub items: Vec<T>,
    /// Total count (if requested).
    pub total_count: Option<u64>,
    /// Cursor for next page.
    pub next_cursor: Option<String>,
    /// Whether more results exist.
    pub has_more: bool,
}

impl<T> QueryPage<T> {
    /// Create an empty page.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total_count: Some(0),
            next_cursor: None,
            has_more: false,
        }
    }
}

/// Sort order for queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Desc
    }
}

/// Sort field for audit queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Timestamp,
    Category,
    Severity,
    Action,
}

impl Default for SortField {
    fn default() -> Self {
        Self::Timestamp
    }
}

/// Time range for queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl TimeRange {
    /// Last N hours.
    pub fn last_hours(hours: i64) -> Self {
        Self {
            start: Some(Utc::now() - chrono::Duration::hours(hours)),
            end: None,
        }
    }

    /// Last N days.
    pub fn last_days(days: i64) -> Self {
        Self {
            start: Some(Utc::now() - chrono::Duration::days(days)),
            end: None,
        }
    }

    /// Specific range.
    pub fn between(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }
}

/// Audit query parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by categories.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<AuditCategory>,
    /// Filter by actions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<AuditAction>,
    /// Minimum severity.
    pub min_severity: Option<AuditSeverity>,
    /// Filter by actor ID.
    pub actor_id: Option<String>,
    /// Filter by target ID.
    pub target_id: Option<String>,
    /// Filter by target type.
    pub target_type: Option<String>,
    /// Filter by correlation ID.
    pub correlation_id: Option<String>,
    /// Filter by outcome success.
    pub success_only: Option<bool>,
    /// Time range filter.
    pub time_range: Option<TimeRange>,
    /// Full-text search query.
    pub search: Option<String>,
    /// Sort field.
    #[serde(default)]
    pub sort_by: SortField,
    /// Sort order.
    #[serde(default)]
    pub sort_order: SortOrder,
    /// Page size.
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Cursor for pagination.
    pub cursor: Option<String>,
    /// Offset for offset-based pagination.
    pub offset: Option<u32>,
    /// Include total count in response.
    #[serde(default)]
    pub include_count: bool,
}

fn default_page_size() -> u32 {
    50
}

impl AuditQuery {
    /// Create a new query builder.
    pub fn builder() -> AuditQueryBuilder {
        AuditQueryBuilder::default()
    }

    /// Maximum allowed page size.
    pub const MAX_PAGE_SIZE: u32 = 1000;

    /// Validate and normalize the query.
    pub fn validate(&mut self) {
        if self.page_size > Self::MAX_PAGE_SIZE {
            self.page_size = Self::MAX_PAGE_SIZE;
        }
        if self.page_size == 0 {
            self.page_size = default_page_size();
        }
    }
}

/// Builder for audit queries.
#[derive(Debug, Default)]
pub struct AuditQueryBuilder {
    query: AuditQuery,
}

impl AuditQueryBuilder {
    /// Filter by category.
    pub fn category(mut self, category: AuditCategory) -> Self {
        self.query.categories.push(category);
        self
    }

    /// Filter by categories.
    pub fn categories(mut self, categories: impl IntoIterator<Item = AuditCategory>) -> Self {
        self.query.categories.extend(categories);
        self
    }

    /// Filter by action.
    pub fn action(mut self, action: AuditAction) -> Self {
        self.query.actions.push(action);
        self
    }

    /// Filter by minimum severity.
    pub fn min_severity(mut self, severity: AuditSeverity) -> Self {
        self.query.min_severity = Some(severity);
        self
    }

    /// Filter by actor ID.
    pub fn actor_id(mut self, id: impl Into<String>) -> Self {
        self.query.actor_id = Some(id.into());
        self
    }

    /// Filter by target ID.
    pub fn target_id(mut self, id: impl Into<String>) -> Self {
        self.query.target_id = Some(id.into());
        self
    }

    /// Filter by target type.
    pub fn target_type(mut self, t: impl Into<String>) -> Self {
        self.query.target_type = Some(t.into());
        self
    }

    /// Filter by correlation ID.
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.query.correlation_id = Some(id.into());
        self
    }

    /// Filter by success.
    pub fn success_only(mut self) -> Self {
        self.query.success_only = Some(true);
        self
    }

    /// Filter by failure.
    pub fn failures_only(mut self) -> Self {
        self.query.success_only = Some(false);
        self
    }

    /// Filter by time range.
    pub fn time_range(mut self, range: TimeRange) -> Self {
        self.query.time_range = Some(range);
        self
    }

    /// Full-text search.
    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.query.search = Some(query.into());
        self
    }

    /// Set sort field and order.
    pub fn sort(mut self, field: SortField, order: SortOrder) -> Self {
        self.query.sort_by = field;
        self.query.sort_order = order;
        self
    }

    /// Set page size.
    pub fn page_size(mut self, size: u32) -> Self {
        self.query.page_size = size;
        self
    }

    /// Set cursor for pagination.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.query.cursor = Some(cursor.into());
        self
    }

    /// Include total count.
    pub fn with_count(mut self) -> Self {
        self.query.include_count = true;
        self
    }

    /// Build the query.
    pub fn build(mut self) -> AuditQuery {
        self.query.validate();
        self.query
    }
}
```

### 2. Query Executor (src/executor.rs)

```rust
//! Query execution against SQLite.

use crate::{
    query::{AuditQuery, QueryPage, SortField, SortOrder, TimeRange},
    AuditEvent, AuditCategory, AuditSeverity,
};
use rusqlite::{params, Connection, Row};
use parking_lot::Mutex;
use std::sync::Arc;

/// Query executor for audit events.
pub struct QueryExecutor {
    conn: Arc<Mutex<Connection>>,
}

impl QueryExecutor {
    /// Create a new executor.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Execute an audit query.
    pub fn execute(&self, query: &AuditQuery) -> Result<QueryPage<AuditEventSummary>, QueryError> {
        let conn = self.conn.lock();

        let (sql, params) = self.build_query(query);
        let mut stmt = conn.prepare(&sql)?;

        let events: Vec<AuditEventSummary> = stmt
            .query_map(params.as_slice(), |row| self.map_row(row))?
            .filter_map(|r| r.ok())
            .collect();

        let total_count = if query.include_count {
            Some(self.count_total(&conn, query)?)
        } else {
            None
        };

        let has_more = events.len() > query.page_size as usize;
        let items: Vec<_> = events.into_iter().take(query.page_size as usize).collect();

        let next_cursor = if has_more {
            items.last().map(|e| e.id.clone())
        } else {
            None
        };

        Ok(QueryPage {
            items,
            total_count,
            next_cursor,
            has_more,
        })
    }

    fn build_query(&self, query: &AuditQuery) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
        let mut sql = String::from(
            "SELECT id, timestamp, category, action, severity, actor_type, actor_id,
                    target_type, target_id, outcome, correlation_id
             FROM audit_events WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if !query.categories.is_empty() {
            let placeholders: Vec<_> = query.categories.iter()
                .map(|c| c.to_string())
                .collect();
            sql.push_str(&format!(
                " AND category IN ({})",
                placeholders.iter().map(|_| "?").collect::<Vec<_>>().join(",")
            ));
            for cat in &placeholders {
                params.push(Box::new(cat.clone()));
            }
        }

        if let Some(ref time_range) = query.time_range {
            if let Some(start) = time_range.start {
                sql.push_str(" AND timestamp >= ?");
                params.push(Box::new(start.to_rfc3339()));
            }
            if let Some(end) = time_range.end {
                sql.push_str(" AND timestamp <= ?");
                params.push(Box::new(end.to_rfc3339()));
            }
        }

        if let Some(ref actor_id) = query.actor_id {
            sql.push_str(" AND actor_id = ?");
            params.push(Box::new(actor_id.clone()));
        }

        if let Some(ref target_id) = query.target_id {
            sql.push_str(" AND target_id = ?");
            params.push(Box::new(target_id.clone()));
        }

        if let Some(min_sev) = query.min_severity {
            let severities = match min_sev {
                AuditSeverity::Info => vec!["info", "low", "medium", "high", "critical"],
                AuditSeverity::Low => vec!["low", "medium", "high", "critical"],
                AuditSeverity::Medium => vec!["medium", "high", "critical"],
                AuditSeverity::High => vec!["high", "critical"],
                AuditSeverity::Critical => vec!["critical"],
            };
            let placeholders = severities.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(" AND severity IN ({})", placeholders));
            for sev in severities {
                params.push(Box::new(sev.to_string()));
            }
        }

        if let Some(success) = query.success_only {
            if success {
                sql.push_str(" AND outcome = 'success'");
            } else {
                sql.push_str(" AND outcome != 'success'");
            }
        }

        // Sorting
        let sort_col = match query.sort_by {
            SortField::Timestamp => "timestamp",
            SortField::Category => "category",
            SortField::Severity => "severity",
            SortField::Action => "action",
        };
        let sort_dir = match query.sort_order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };
        sql.push_str(&format!(" ORDER BY {} {}", sort_col, sort_dir));

        // Pagination
        sql.push_str(&format!(" LIMIT {}", query.page_size + 1));
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    fn count_total(&self, conn: &Connection, query: &AuditQuery) -> Result<u64, QueryError> {
        // Simplified count query
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    fn map_row(&self, row: &Row) -> Result<AuditEventSummary, rusqlite::Error> {
        Ok(AuditEventSummary {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            category: row.get(2)?,
            action: row.get(3)?,
            severity: row.get(4)?,
            actor_type: row.get(5)?,
            actor_id: row.get(6)?,
            target_type: row.get(7)?,
            target_id: row.get(8)?,
            outcome: row.get(9)?,
            correlation_id: row.get(10)?,
        })
    }
}

/// Summarized audit event for query results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEventSummary {
    pub id: String,
    pub timestamp: String,
    pub category: String,
    pub action: String,
    pub severity: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub outcome: String,
    pub correlation_id: Option<String>,
}

/// Query error.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid query: {0}")]
    InvalidQuery(String),
}
```

---

## Testing Requirements

1. Query builder produces correct SQL
2. Pagination works with cursor and offset
3. All filters apply correctly
4. Sort ordering is respected
5. Count queries are accurate

---

## Related Specs

- Depends on: [434-audit-persistence.md](434-audit-persistence.md)
- Next: [436-audit-retention.md](436-audit-retention.md)
