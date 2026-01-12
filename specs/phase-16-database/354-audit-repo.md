# Spec 354: Audit Log Repository

## Overview
Implement the repository pattern for audit logging with efficient querying, retention management, and analytics.

## Rust Implementation

### Audit Repository
```rust
// src/database/repository/audit.rs

use crate::database::schema::audit::*;
use chrono::{DateTime, Duration, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuditRepoError {
    #[error("Audit log not found: {0}")]
    NotFound(String),

    #[error("Invalid query parameters: {0}")]
    InvalidQuery(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Query filters for audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub category: Option<Vec<AuditCategory>>,
    pub action: Option<String>,
    pub action_prefix: Option<String>,
    pub severity: Option<Vec<AuditSeverity>>,
    pub outcome: Option<Vec<AuditOutcome>>,
    pub actor_id: Option<String>,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

/// Pagination with cursor support
#[derive(Debug, Clone)]
pub struct AuditPagination {
    pub limit: i64,
    pub offset: Option<i64>,
    pub cursor: Option<String>,  // Last seen ID for cursor pagination
}

impl Default for AuditPagination {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: None,
            cursor: None,
        }
    }
}

/// Sort options
#[derive(Debug, Clone)]
pub struct AuditSort {
    pub field: AuditSortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AuditSortField {
    #[default]
    Timestamp,
    Severity,
    Category,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}

impl Default for AuditSort {
    fn default() -> Self {
        Self {
            field: AuditSortField::Timestamp,
            direction: SortDirection::Desc,
        }
    }
}

pub struct AuditRepository {
    pool: SqlitePool,
}

impl AuditRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Log an audit event
    #[instrument(skip(self, entry), fields(action = %entry.action))]
    pub async fn log(&self, entry: AuditLog) -> Result<AuditLog, AuditRepoError> {
        sqlx::query(r#"
            INSERT INTO audit_logs (
                id, timestamp, category, action, severity, outcome,
                actor_id, actor_type, target_type, target_id,
                resource_type, resource_id, ip_address, user_agent,
                session_id, request_id, description, old_value,
                new_value, metadata, error_message, duration_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&entry.id)
        .bind(entry.timestamp)
        .bind(entry.category)
        .bind(&entry.action)
        .bind(entry.severity)
        .bind(entry.outcome)
        .bind(&entry.actor_id)
        .bind(&entry.actor_type)
        .bind(&entry.target_type)
        .bind(&entry.target_id)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(&entry.ip_address)
        .bind(&entry.user_agent)
        .bind(&entry.session_id)
        .bind(&entry.request_id)
        .bind(&entry.description)
        .bind(&entry.old_value)
        .bind(&entry.new_value)
        .bind(&entry.metadata)
        .bind(&entry.error_message)
        .bind(entry.duration_ms)
        .execute(&self.pool)
        .await?;

        debug!("Logged audit event: {} - {}", entry.id, entry.action);
        Ok(entry)
    }

    /// Log a security event
    pub async fn log_security_event(
        &self,
        event: &SecurityEvent,
        context: &AuditContext,
    ) -> Result<AuditLog, AuditRepoError> {
        use crate::database::schema::audit_builder::AuditLogBuilder;

        let entry = AuditLogBuilder::new(AuditCategory::Security, event.to_audit_action())
            .with_context(context)
            .severity(event.severity())
            .metadata(serde_json::to_value(event).unwrap_or_default())
            .build();

        self.log(entry).await
    }

    /// Find audit log by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<AuditLog>, AuditRepoError> {
        let log = sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(log)
    }

    /// Query audit logs with filters
    #[instrument(skip(self))]
    pub async fn query(
        &self,
        filter: AuditFilter,
        pagination: AuditPagination,
        sort: Option<AuditSort>,
    ) -> Result<Vec<AuditLog>, AuditRepoError> {
        let mut sql = String::from("SELECT * FROM audit_logs WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        // Build filter clauses
        if let Some(categories) = &filter.category {
            if !categories.is_empty() {
                let placeholders: Vec<_> = categories.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND category IN ({})", placeholders.join(",")));
                for c in categories {
                    bindings.push(format!("{:?}", c).to_lowercase());
                }
            }
        }

        if let Some(action) = &filter.action {
            sql.push_str(" AND action = ?");
            bindings.push(action.clone());
        }

        if let Some(prefix) = &filter.action_prefix {
            sql.push_str(" AND action LIKE ?");
            bindings.push(format!("{}%", prefix));
        }

        if let Some(severities) = &filter.severity {
            if !severities.is_empty() {
                let placeholders: Vec<_> = severities.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND severity IN ({})", placeholders.join(",")));
                for s in severities {
                    bindings.push(format!("{:?}", s).to_lowercase());
                }
            }
        }

        if let Some(outcomes) = &filter.outcome {
            if !outcomes.is_empty() {
                let placeholders: Vec<_> = outcomes.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND outcome IN ({})", placeholders.join(",")));
                for o in outcomes {
                    bindings.push(format!("{:?}", o).to_lowercase());
                }
            }
        }

        if let Some(actor) = &filter.actor_id {
            sql.push_str(" AND actor_id = ?");
            bindings.push(actor.clone());
        }

        if let Some(target_type) = &filter.target_type {
            sql.push_str(" AND target_type = ?");
            bindings.push(target_type.clone());
        }

        if let Some(target_id) = &filter.target_id {
            sql.push_str(" AND target_id = ?");
            bindings.push(target_id.clone());
        }

        if let Some(session) = &filter.session_id {
            sql.push_str(" AND session_id = ?");
            bindings.push(session.clone());
        }

        if let Some(request) = &filter.request_id {
            sql.push_str(" AND request_id = ?");
            bindings.push(request.clone());
        }

        if let Some(ip) = &filter.ip_address {
            sql.push_str(" AND ip_address = ?");
            bindings.push(ip.clone());
        }

        if let Some(from) = filter.from_timestamp {
            sql.push_str(" AND timestamp >= ?");
            bindings.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to_timestamp {
            sql.push_str(" AND timestamp <= ?");
            bindings.push(to.to_rfc3339());
        }

        if let Some(search) = &filter.search {
            sql.push_str(" AND (description LIKE ? OR action LIKE ? OR error_message LIKE ?)");
            let search_pattern = format!("%{}%", search);
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern);
        }

        // Cursor pagination
        if let Some(cursor) = &pagination.cursor {
            sql.push_str(" AND id < ?");
            bindings.push(cursor.clone());
        }

        // Sorting
        let sort = sort.unwrap_or_default();
        let sort_field = match sort.field {
            AuditSortField::Timestamp => "timestamp",
            AuditSortField::Severity => "severity",
            AuditSortField::Category => "category",
        };
        let sort_dir = match sort.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_dir));

        // Pagination
        sql.push_str(" LIMIT ?");

        if let Some(offset) = pagination.offset {
            sql.push_str(" OFFSET ?");
        }

        let mut query = sqlx::query_as::<_, AuditLog>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(pagination.limit);
        if let Some(offset) = pagination.offset {
            query = query.bind(offset);
        }

        let logs = query.fetch_all(&self.pool).await?;
        Ok(logs)
    }

    /// Get logs for a specific actor
    pub async fn get_actor_logs(
        &self,
        actor_id: &str,
        limit: i64,
    ) -> Result<Vec<AuditLog>, AuditRepoError> {
        let logs = sqlx::query_as::<_, AuditLog>(r#"
            SELECT * FROM audit_logs
            WHERE actor_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
        "#)
        .bind(actor_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get logs for a specific session
    pub async fn get_session_logs(&self, session_id: &str) -> Result<Vec<AuditLog>, AuditRepoError> {
        let logs = sqlx::query_as::<_, AuditLog>(r#"
            SELECT * FROM audit_logs
            WHERE session_id = ?
            ORDER BY timestamp
        "#)
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get recent security events
    pub async fn get_security_events(
        &self,
        hours: i64,
        severity_min: Option<AuditSeverity>,
    ) -> Result<Vec<AuditLog>, AuditRepoError> {
        let since = Utc::now() - Duration::hours(hours);

        let mut sql = r#"
            SELECT * FROM audit_logs
            WHERE category = 'security'
            AND timestamp >= ?
        "#.to_string();

        if let Some(min_sev) = severity_min {
            sql.push_str(" AND severity IN ('warning', 'error', 'critical')");
        }

        sql.push_str(" ORDER BY timestamp DESC");

        let logs = sqlx::query_as::<_, AuditLog>(&sql)
            .bind(since)
            .fetch_all(&self.pool)
            .await?;

        Ok(logs)
    }

    /// Count logs matching filter
    pub async fn count(&self, filter: AuditFilter) -> Result<i64, AuditRepoError> {
        let mut sql = String::from("SELECT COUNT(*) FROM audit_logs WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        // Same filter logic as query...
        if let Some(actor) = &filter.actor_id {
            sql.push_str(" AND actor_id = ?");
            bindings.push(actor.clone());
        }

        if let Some(from) = filter.from_timestamp {
            sql.push_str(" AND timestamp >= ?");
            bindings.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to_timestamp {
            sql.push_str(" AND timestamp <= ?");
            bindings.push(to.to_rfc3339());
        }

        let mut query = sqlx::query_as::<_, (i64,)>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }

        let (count,) = query.fetch_one(&self.pool).await?;
        Ok(count)
    }

    /// Delete logs older than retention period
    #[instrument(skip(self))]
    pub async fn cleanup(&self) -> Result<i64, AuditRepoError> {
        // Get retention policies
        let policies: Vec<(Option<String>, Option<String>, i32)> = sqlx::query_as(
            "SELECT category, severity, retention_days FROM audit_retention_policies WHERE enabled = 1"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total_deleted = 0i64;

        for (category, severity, days) in policies {
            let cutoff = Utc::now() - Duration::days(days as i64);

            let mut sql = "DELETE FROM audit_logs WHERE timestamp < ?".to_string();
            let mut bindings = vec![cutoff.to_rfc3339()];

            if let Some(cat) = &category {
                sql.push_str(" AND category = ?");
                bindings.push(cat.clone());
            }

            if let Some(sev) = &severity {
                sql.push_str(" AND severity = ?");
                bindings.push(sev.clone());
            }

            let mut query = sqlx::query(&sql);
            for binding in &bindings {
                query = query.bind(binding);
            }

            let result = query.execute(&self.pool).await?;
            total_deleted += result.rows_affected() as i64;
        }

        debug!("Cleaned up {} audit logs", total_deleted);
        Ok(total_deleted)
    }

    /// Generate summary statistics
    pub async fn generate_summary(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        period_type: &str,
    ) -> Result<Vec<AuditSummary>, AuditRepoError> {
        let rows = sqlx::query_as::<_, AuditSummaryRow>(r#"
            SELECT
                category,
                action,
                SUM(CASE WHEN outcome = 'success' THEN 1 ELSE 0 END) as success_count,
                SUM(CASE WHEN outcome = 'failure' THEN 1 ELSE 0 END) as failure_count,
                COUNT(DISTINCT actor_id) as unique_actors,
                AVG(duration_ms) as avg_duration_ms
            FROM audit_logs
            WHERE timestamp >= ? AND timestamp < ?
            GROUP BY category, action
        "#)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.pool)
        .await?;

        let summaries: Vec<AuditSummary> = rows.into_iter().map(|r| {
            AuditSummary {
                id: Uuid::new_v4().to_string(),
                period_start,
                period_end,
                period_type: period_type.to_string(),
                category: r.category,
                action: r.action,
                success_count: r.success_count,
                failure_count: r.failure_count,
                unique_actors: r.unique_actors,
                avg_duration_ms: r.avg_duration_ms,
            }
        }).collect();

        // Store summaries
        for summary in &summaries {
            sqlx::query(r#"
                INSERT OR REPLACE INTO audit_summaries (
                    id, period_start, period_end, period_type,
                    category, action, success_count, failure_count,
                    unique_actors, avg_duration_ms
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#)
            .bind(&summary.id)
            .bind(summary.period_start)
            .bind(summary.period_end)
            .bind(&summary.period_type)
            .bind(&summary.category)
            .bind(&summary.action)
            .bind(summary.success_count)
            .bind(summary.failure_count)
            .bind(summary.unique_actors)
            .bind(summary.avg_duration_ms)
            .execute(&self.pool)
            .await?;
        }

        Ok(summaries)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct AuditSummaryRow {
    category: String,
    action: String,
    success_count: i64,
    failure_count: i64,
    unique_actors: i64,
    avg_duration_ms: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AuditSummary {
    pub id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_type: String,
    pub category: String,
    pub action: String,
    pub success_count: i64,
    pub failure_count: i64,
    pub unique_actors: i64,
    pub avg_duration_ms: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/audit.rs` - Audit repository implementation
