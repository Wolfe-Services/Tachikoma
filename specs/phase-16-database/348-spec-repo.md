# Spec 348: Spec Repository

## Overview
Implement the repository pattern for Spec CRUD operations with version management, comment handling, and approval workflows.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Spec Repository
```rust
// src/database/repository/spec.rs

use crate::database::schema::spec::*;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SpecRepoError {
    #[error("Spec not found: {0}")]
    NotFound(String),

    #[error("Invalid status transition from {0:?} to {1:?}")]
    InvalidStatusTransition(SpecStatus, SpecStatus),

    #[error("Spec is not editable in status {0:?}")]
    NotEditable(SpecStatus),

    #[error("Mission not found: {0}")]
    MissionNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Input for creating a new spec
#[derive(Debug, Clone)]
pub struct CreateSpec {
    pub mission_id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub complexity: Option<SpecComplexity>,
    pub author_id: Option<String>,
    pub estimated_hours: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
}

/// Input for updating a spec
#[derive(Debug, Clone, Default)]
pub struct UpdateSpec {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub complexity: Option<SpecComplexity>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
}

/// Query filters for specs
#[derive(Debug, Clone, Default)]
pub struct SpecFilter {
    pub mission_id: Option<String>,
    pub status: Option<Vec<SpecStatus>>,
    pub complexity: Option<Vec<SpecComplexity>>,
    pub author_id: Option<String>,
    pub reviewer_id: Option<String>,
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Pagination options
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { limit: 50, offset: 0 }
    }
}

pub struct SpecRepository {
    pool: SqlitePool,
}

impl SpecRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new spec
    #[instrument(skip(self, input), fields(mission_id = %input.mission_id, title = %input.title))]
    pub async fn create(&self, input: CreateSpec) -> Result<Spec, SpecRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let complexity = input.complexity.unwrap_or_default();
        let tags_json = input.tags.map(|t| serde_json::to_string(&t).unwrap());
        let criteria_json = input.acceptance_criteria
            .map(|c| serde_json::to_string(&c).unwrap());

        sqlx::query(r#"
            INSERT INTO specs (
                id, mission_id, title, description, content,
                status, complexity, version, author_id,
                created_at, updated_at, estimated_hours, tags, acceptance_criteria
            ) VALUES (?, ?, ?, ?, ?, 'draft', ?, 1, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&input.mission_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.content)
        .bind(complexity)
        .bind(&input.author_id)
        .bind(now)
        .bind(now)
        .bind(input.estimated_hours)
        .bind(&tags_json)
        .bind(&criteria_json)
        .execute(&self.pool)
        .await?;

        // Create initial version
        self.create_version(&id, 1, &input.title, input.content.as_deref(), None).await?;

        debug!("Created spec {}", id);
        self.find_by_id(&id).await?.ok_or(SpecRepoError::NotFound(id))
    }

    /// Find a spec by ID
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Spec>, SpecRepoError> {
        let spec = sqlx::query_as::<_, Spec>("SELECT * FROM specs WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(spec)
    }

    /// Find specs with filters
    #[instrument(skip(self))]
    pub async fn find_many(
        &self,
        filter: SpecFilter,
        pagination: Pagination,
    ) -> Result<Vec<Spec>, SpecRepoError> {
        let mut sql = String::from("SELECT * FROM specs WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(mission_id) = &filter.mission_id {
            sql.push_str(" AND mission_id = ?");
            bindings.push(mission_id.clone());
        }

        if let Some(statuses) = &filter.status {
            if !statuses.is_empty() {
                let placeholders: Vec<_> = statuses.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
                for s in statuses {
                    bindings.push(format!("{:?}", s).to_lowercase());
                }
            }
        }

        if let Some(author) = &filter.author_id {
            sql.push_str(" AND author_id = ?");
            bindings.push(author.clone());
        }

        if let Some(reviewer) = &filter.reviewer_id {
            sql.push_str(" AND reviewer_id = ?");
            bindings.push(reviewer.clone());
        }

        if let Some(search) = &filter.search {
            sql.push_str(" AND id IN (SELECT id FROM specs_fts WHERE specs_fts MATCH ?)");
            bindings.push(search.clone());
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, Spec>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(pagination.limit).bind(pagination.offset);

        let specs = query.fetch_all(&self.pool).await?;
        Ok(specs)
    }

    /// Update a spec
    #[instrument(skip(self, input))]
    pub async fn update(&self, id: &str, input: UpdateSpec) -> Result<Spec, SpecRepoError> {
        let current = self.find_by_id(id).await?
            .ok_or_else(|| SpecRepoError::NotFound(id.to_string()))?;

        if !current.is_editable() {
            return Err(SpecRepoError::NotEditable(current.status));
        }

        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();
        let mut version_bump = false;

        if let Some(title) = &input.title {
            updates.push("title = ?");
            bindings.push(title.clone());
            version_bump = true;
        }

        if let Some(description) = &input.description {
            updates.push("description = ?");
            bindings.push(description.clone());
        }

        if let Some(content) = &input.content {
            updates.push("content = ?");
            bindings.push(content.clone());
            version_bump = true;
        }

        if let Some(complexity) = &input.complexity {
            updates.push("complexity = ?");
            bindings.push(format!("{:?}", complexity).to_lowercase());
        }

        if let Some(hours) = input.estimated_hours {
            updates.push("estimated_hours = ?");
            bindings.push(hours.to_string());
        }

        if let Some(actual) = input.actual_hours {
            updates.push("actual_hours = ?");
            bindings.push(actual.to_string());
        }

        if let Some(tags) = &input.tags {
            updates.push("tags = ?");
            bindings.push(serde_json::to_string(tags).unwrap());
        }

        if let Some(criteria) = &input.acceptance_criteria {
            updates.push("acceptance_criteria = ?");
            bindings.push(serde_json::to_string(criteria).unwrap());
        }

        if updates.is_empty() {
            return Ok(current);
        }

        // Bump version if content changed
        if version_bump {
            updates.push("version = version + 1");
        }

        let sql = format!("UPDATE specs SET {} WHERE id = ?", updates.join(", "));

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(id);

        query.execute(&self.pool).await?;

        debug!("Updated spec {}", id);
        self.find_by_id(id).await?.ok_or(SpecRepoError::NotFound(id.to_string()))
    }

    /// Transition spec status
    #[instrument(skip(self))]
    pub async fn transition_status(
        &self,
        id: &str,
        new_status: SpecStatus,
        reviewer_id: Option<&str>,
    ) -> Result<Spec, SpecRepoError> {
        let current = self.find_by_id(id).await?
            .ok_or_else(|| SpecRepoError::NotFound(id.to_string()))?;

        // Validate transition
        if !Self::is_valid_transition(current.status, new_status) {
            return Err(SpecRepoError::InvalidStatusTransition(current.status, new_status));
        }

        let mut sql = "UPDATE specs SET status = ?".to_string();
        let mut bindings: Vec<String> = vec![format!("{:?}", new_status).to_lowercase()];

        // Set approved_at when transitioning to approved
        if new_status == SpecStatus::Approved {
            sql.push_str(", approved_at = datetime('now')");
        }

        // Set reviewer if provided
        if let Some(reviewer) = reviewer_id {
            sql.push_str(", reviewer_id = ?");
            bindings.push(reviewer.to_string());
        }

        sql.push_str(" WHERE id = ?");
        bindings.push(id.to_string());

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }

        query.execute(&self.pool).await?;

        self.find_by_id(id).await?.ok_or(SpecRepoError::NotFound(id.to_string()))
    }

    /// Check if status transition is valid
    fn is_valid_transition(from: SpecStatus, to: SpecStatus) -> bool {
        use SpecStatus::*;
        matches!(
            (from, to),
            (Draft, Review) |
            (Review, Approved) |
            (Review, Rejected) |
            (Review, Draft) |
            (Approved, Implementation) |
            (Implementation, Testing) |
            (Testing, Complete) |
            (Testing, Implementation) |
            (Rejected, Draft)
        )
    }

    /// Delete a spec
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<bool, SpecRepoError> {
        let result = sqlx::query("DELETE FROM specs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get specs by mission
    pub async fn find_by_mission(&self, mission_id: &str) -> Result<Vec<Spec>, SpecRepoError> {
        let specs = sqlx::query_as::<_, Spec>(
            "SELECT * FROM specs WHERE mission_id = ? ORDER BY created_at"
        )
        .bind(mission_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(specs)
    }

    /// Create version record
    async fn create_version(
        &self,
        spec_id: &str,
        version: i32,
        title: &str,
        content: Option<&str>,
        change_summary: Option<&str>,
    ) -> Result<(), SpecRepoError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(r#"
            INSERT INTO spec_versions (id, spec_id, version, title, content, change_summary)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(spec_id)
        .bind(version)
        .bind(title)
        .bind(content)
        .bind(change_summary)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get version history
    pub async fn get_versions(&self, spec_id: &str) -> Result<Vec<SpecVersion>, SpecRepoError> {
        let versions = sqlx::query_as::<_, SpecVersion>(
            "SELECT * FROM spec_versions WHERE spec_id = ? ORDER BY version DESC"
        )
        .bind(spec_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(versions)
    }

    /// Get specific version
    pub async fn get_version(
        &self,
        spec_id: &str,
        version: i32,
    ) -> Result<Option<SpecVersion>, SpecRepoError> {
        let version = sqlx::query_as::<_, SpecVersion>(
            "SELECT * FROM spec_versions WHERE spec_id = ? AND version = ?"
        )
        .bind(spec_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(version)
    }

    /// Add comment to spec
    pub async fn add_comment(
        &self,
        spec_id: &str,
        author_id: Option<&str>,
        content: &str,
        parent_id: Option<&str>,
    ) -> Result<SpecComment, SpecRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO spec_comments (id, spec_id, parent_id, author_id, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(spec_id)
        .bind(parent_id)
        .bind(author_id)
        .bind(content)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let comment = sqlx::query_as::<_, SpecComment>(
            "SELECT * FROM spec_comments WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    /// Get comments for spec
    pub async fn get_comments(&self, spec_id: &str) -> Result<Vec<SpecComment>, SpecRepoError> {
        let comments = sqlx::query_as::<_, SpecComment>(
            "SELECT * FROM spec_comments WHERE spec_id = ? ORDER BY created_at"
        )
        .bind(spec_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    /// Resolve/unresolve comment
    pub async fn resolve_comment(&self, comment_id: &str, resolved: bool) -> Result<(), SpecRepoError> {
        sqlx::query("UPDATE spec_comments SET resolved = ? WHERE id = ?")
            .bind(resolved as i32)
            .bind(comment_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Search specs
    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<Spec>, SpecRepoError> {
        let specs = sqlx::query_as::<_, Spec>(r#"
            SELECT s.* FROM specs s
            INNER JOIN specs_fts fts ON s.id = fts.id
            WHERE specs_fts MATCH ?
            ORDER BY rank
            LIMIT ?
        "#)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(specs)
    }

    /// Update acceptance criterion
    pub async fn verify_criterion(
        &self,
        spec_id: &str,
        criterion_id: &str,
        verified: bool,
        verified_by: Option<&str>,
    ) -> Result<Spec, SpecRepoError> {
        let spec = self.find_by_id(spec_id).await?
            .ok_or_else(|| SpecRepoError::NotFound(spec_id.to_string()))?;

        let mut criteria = spec.get_acceptance_criteria();

        if let Some(criterion) = criteria.iter_mut().find(|c| c.id == criterion_id) {
            criterion.verified = verified;
            criterion.verified_at = if verified { Some(Utc::now()) } else { None };
            criterion.verified_by = verified_by.map(String::from);
        }

        let criteria_json = serde_json::to_string(&criteria).unwrap();

        sqlx::query("UPDATE specs SET acceptance_criteria = ? WHERE id = ?")
            .bind(&criteria_json)
            .bind(spec_id)
            .execute(&self.pool)
            .await?;

        self.find_by_id(spec_id).await?.ok_or(SpecRepoError::NotFound(spec_id.to_string()))
    }

    /// Get spec statistics
    pub async fn stats(&self, mission_id: Option<&str>) -> Result<SpecStats, SpecRepoError> {
        let mut sql = r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'draft' THEN 1 ELSE 0 END) as draft,
                SUM(CASE WHEN status = 'review' THEN 1 ELSE 0 END) as review,
                SUM(CASE WHEN status = 'approved' THEN 1 ELSE 0 END) as approved,
                SUM(CASE WHEN status = 'implementation' THEN 1 ELSE 0 END) as implementation,
                SUM(CASE WHEN status = 'complete' THEN 1 ELSE 0 END) as complete,
                COALESCE(SUM(estimated_hours), 0) as total_estimated_hours,
                COALESCE(SUM(actual_hours), 0) as total_actual_hours
            FROM specs
        "#.to_string();

        let stats = if let Some(mid) = mission_id {
            sql.push_str(" WHERE mission_id = ?");
            sqlx::query_as::<_, SpecStats>(&sql)
                .bind(mid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, SpecStats>(&sql)
                .fetch_one(&self.pool)
                .await?
        };

        Ok(stats)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SpecStats {
    pub total: i64,
    pub draft: i64,
    pub review: i64,
    pub approved: i64,
    pub implementation: i64,
    pub complete: i64,
    pub total_estimated_hours: f64,
    pub total_actual_hours: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/spec.rs` - Spec repository implementation
