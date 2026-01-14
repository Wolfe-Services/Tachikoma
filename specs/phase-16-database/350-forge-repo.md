# Spec 350: Forge Repository

## Overview
Implement the repository pattern for Forge item CRUD operations with revision management, code review, and build result tracking.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Forge Repository
```rust
// src/database/repository/forge.rs

use crate::database::schema::forge::*;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ForgeRepoError {
    #[error("Forge item not found: {0}")]
    NotFound(String),

    #[error("Spec not found: {0}")]
    SpecNotFound(String),

    #[error("Invalid status transition from {0:?} to {1:?}")]
    InvalidStatusTransition(ForgeItemStatus, ForgeItemStatus),

    #[error("Build in progress")]
    BuildInProgress,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Input for creating a forge item
#[derive(Debug, Clone)]
pub struct CreateForgeItem {
    pub spec_id: String,
    pub item_type: ForgeItemType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub language: Option<String>,
    pub author_id: Option<String>,
}

/// Input for updating a forge item
#[derive(Debug, Clone, Default)]
pub struct UpdateForgeItem {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub language: Option<String>,
    pub test_coverage: Option<f64>,
}

/// Query filters for forge items
#[derive(Debug, Clone, Default)]
pub struct ForgeFilter {
    pub spec_id: Option<String>,
    pub item_type: Option<Vec<ForgeItemType>>,
    pub status: Option<Vec<ForgeItemStatus>>,
    pub author_id: Option<String>,
    pub language: Option<String>,
    pub search: Option<String>,
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

pub struct ForgeRepository {
    pool: SqlitePool,
}

impl ForgeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new forge item
    #[instrument(skip(self, input), fields(spec_id = %input.spec_id))]
    pub async fn create(&self, input: CreateForgeItem) -> Result<ForgeItem, ForgeRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Auto-detect language from file path if not provided
        let language = input.language.or_else(|| {
            input.file_path.as_ref()
                .and_then(|p| ForgeItem::detect_language_from_path(p))
        });

        // Count lines if content provided
        let lines_of_code = input.content.as_ref().map(|c| {
            c.lines()
                .filter(|l| !l.trim().is_empty())
                .count() as i32
        });

        sqlx::query(r#"
            INSERT INTO forge_items (
                id, spec_id, item_type, title, description,
                file_path, content, language, status, author_id,
                created_at, updated_at, lines_of_code
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&input.spec_id)
        .bind(input.item_type)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.file_path)
        .bind(&input.content)
        .bind(&language)
        .bind(&input.author_id)
        .bind(now)
        .bind(now)
        .bind(lines_of_code)
        .execute(&self.pool)
        .await?;

        // Create initial revision if content exists
        if input.content.is_some() {
            self.create_revision(&id, 1, input.content.as_deref(), Some("Initial version")).await?;
        }

        debug!("Created forge item {}", id);
        self.find_by_id(&id).await?.ok_or(ForgeRepoError::NotFound(id))
    }

    /// Find a forge item by ID
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: &str) -> Result<Option<ForgeItem>, ForgeRepoError> {
        let item = sqlx::query_as::<_, ForgeItem>("SELECT * FROM forge_items WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(item)
    }

    /// Find forge items with filters
    #[instrument(skip(self))]
    pub async fn find_many(
        &self,
        filter: ForgeFilter,
        pagination: Pagination,
    ) -> Result<Vec<ForgeItem>, ForgeRepoError> {
        let mut sql = String::from("SELECT * FROM forge_items WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(spec_id) = &filter.spec_id {
            sql.push_str(" AND spec_id = ?");
            bindings.push(spec_id.clone());
        }

        if let Some(types) = &filter.item_type {
            if !types.is_empty() {
                let placeholders: Vec<_> = types.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND item_type IN ({})", placeholders.join(",")));
                for t in types {
                    bindings.push(format!("{:?}", t).to_lowercase());
                }
            }
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

        if let Some(lang) = &filter.language {
            sql.push_str(" AND language = ?");
            bindings.push(lang.clone());
        }

        if let Some(search) = &filter.search {
            sql.push_str(" AND id IN (SELECT id FROM forge_items_fts WHERE forge_items_fts MATCH ?)");
            bindings.push(search.clone());
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, ForgeItem>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(pagination.limit).bind(pagination.offset);

        let items = query.fetch_all(&self.pool).await?;
        Ok(items)
    }

    /// Update a forge item
    #[instrument(skip(self, input))]
    pub async fn update(&self, id: &str, input: UpdateForgeItem) -> Result<ForgeItem, ForgeRepoError> {
        let current = self.find_by_id(id).await?
            .ok_or_else(|| ForgeRepoError::NotFound(id.to_string()))?;

        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(title) = &input.title {
            updates.push("title = ?");
            bindings.push(title.clone());
        }

        if let Some(description) = &input.description {
            updates.push("description = ?");
            bindings.push(description.clone());
        }

        if let Some(content) = &input.content {
            updates.push("content = ?");
            bindings.push(content.clone());

            // Update lines of code
            let loc = content.lines()
                .filter(|l| !l.trim().is_empty())
                .count();
            updates.push("lines_of_code = ?");
            bindings.push(loc.to_string());
        }

        if let Some(path) = &input.file_path {
            updates.push("file_path = ?");
            bindings.push(path.clone());

            // Auto-detect language if changed
            if let Some(lang) = ForgeItem::detect_language_from_path(path) {
                updates.push("language = ?");
                bindings.push(lang);
            }
        }

        if let Some(lang) = &input.language {
            updates.push("language = ?");
            bindings.push(lang.clone());
        }

        if let Some(coverage) = input.test_coverage {
            updates.push("test_coverage = ?");
            bindings.push(coverage.to_string());
        }

        if updates.is_empty() {
            return Ok(current);
        }

        let sql = format!("UPDATE forge_items SET {} WHERE id = ?", updates.join(", "));

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(id);

        query.execute(&self.pool).await?;

        debug!("Updated forge item {}", id);
        self.find_by_id(id).await?.ok_or(ForgeRepoError::NotFound(id.to_string()))
    }

    /// Transition status
    #[instrument(skip(self))]
    pub async fn transition_status(
        &self,
        id: &str,
        new_status: ForgeItemStatus,
        reviewer_id: Option<&str>,
    ) -> Result<ForgeItem, ForgeRepoError> {
        let current = self.find_by_id(id).await?
            .ok_or_else(|| ForgeRepoError::NotFound(id.to_string()))?;

        // Validate transition
        if !Self::is_valid_transition(current.status, new_status) {
            return Err(ForgeRepoError::InvalidStatusTransition(current.status, new_status));
        }

        let mut sql = "UPDATE forge_items SET status = ?".to_string();
        let mut bindings: Vec<String> = vec![format!("{:?}", new_status).to_lowercase()];

        if new_status == ForgeItemStatus::Approved || new_status == ForgeItemStatus::Rejected {
            sql.push_str(", reviewed_at = datetime('now')");
        }

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

        self.find_by_id(id).await?.ok_or(ForgeRepoError::NotFound(id.to_string()))
    }

    fn is_valid_transition(from: ForgeItemStatus, to: ForgeItemStatus) -> bool {
        use ForgeItemStatus::*;
        matches!(
            (from, to),
            (Pending, InProgress) |
            (InProgress, Review) |
            (InProgress, Pending) |
            (Review, Approved) |
            (Review, Rejected) |
            (Review, InProgress) |
            (Approved, Merged) |
            (Rejected, InProgress)
        )
    }

    /// Delete a forge item
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<bool, ForgeRepoError> {
        let result = sqlx::query("DELETE FROM forge_items WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get items by spec
    pub async fn find_by_spec(&self, spec_id: &str) -> Result<Vec<ForgeItem>, ForgeRepoError> {
        let items = sqlx::query_as::<_, ForgeItem>(
            "SELECT * FROM forge_items WHERE spec_id = ? ORDER BY created_at"
        )
        .bind(spec_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    /// Create revision
    async fn create_revision(
        &self,
        forge_item_id: &str,
        revision_number: i32,
        content: Option<&str>,
        change_description: Option<&str>,
    ) -> Result<ForgeRevision, ForgeRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO forge_revisions (id, forge_item_id, revision_number, content, change_description, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(forge_item_id)
        .bind(revision_number)
        .bind(content)
        .bind(change_description)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let revision = sqlx::query_as::<_, ForgeRevision>(
            "SELECT * FROM forge_revisions WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(revision)
    }

    /// Get revisions
    pub async fn get_revisions(&self, forge_item_id: &str) -> Result<Vec<ForgeRevision>, ForgeRepoError> {
        let revisions = sqlx::query_as::<_, ForgeRevision>(
            "SELECT * FROM forge_revisions WHERE forge_item_id = ? ORDER BY revision_number DESC"
        )
        .bind(forge_item_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(revisions)
    }

    /// Add review comment
    pub async fn add_review_comment(
        &self,
        forge_item_id: &str,
        author_id: Option<&str>,
        content: &str,
        line_number: Option<i32>,
        revision_id: Option<&str>,
    ) -> Result<ForgeReviewComment, ForgeRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO forge_review_comments (
                id, forge_item_id, revision_id, author_id, line_number, content, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(forge_item_id)
        .bind(revision_id)
        .bind(author_id)
        .bind(line_number)
        .bind(content)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let comment = sqlx::query_as::<_, ForgeReviewComment>(
            "SELECT * FROM forge_review_comments WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    /// Get review comments
    pub async fn get_review_comments(&self, forge_item_id: &str) -> Result<Vec<ForgeReviewComment>, ForgeRepoError> {
        let comments = sqlx::query_as::<_, ForgeReviewComment>(
            "SELECT * FROM forge_review_comments WHERE forge_item_id = ? ORDER BY created_at"
        )
        .bind(forge_item_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    /// Resolve comment
    pub async fn resolve_comment(&self, comment_id: &str, resolved: bool) -> Result<(), ForgeRepoError> {
        sqlx::query("UPDATE forge_review_comments SET resolved = ? WHERE id = ?")
            .bind(resolved as i32)
            .bind(comment_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Create build result
    pub async fn create_build_result(
        &self,
        forge_item_id: &str,
        build_type: BuildType,
        revision_id: Option<&str>,
    ) -> Result<ForgeBuildResult, ForgeRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO forge_build_results (
                id, forge_item_id, revision_id, build_type, status, started_at
            ) VALUES (?, ?, ?, ?, 'pending', ?)
        "#)
        .bind(&id)
        .bind(forge_item_id)
        .bind(revision_id)
        .bind(build_type)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let result = sqlx::query_as::<_, ForgeBuildResult>(
            "SELECT * FROM forge_build_results WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Update build result
    pub async fn update_build_result(
        &self,
        build_id: &str,
        status: BuildStatus,
        output: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<ForgeBuildResult, ForgeRepoError> {
        let now = Utc::now();

        sqlx::query(r#"
            UPDATE forge_build_results
            SET status = ?, completed_at = ?, output = ?, error_message = ?,
                duration_ms = CAST((julianday(?) - julianday(started_at)) * 86400000 AS INTEGER)
            WHERE id = ?
        "#)
        .bind(status)
        .bind(now)
        .bind(output)
        .bind(error_message)
        .bind(now)
        .bind(build_id)
        .execute(&self.pool)
        .await?;

        let result = sqlx::query_as::<_, ForgeBuildResult>(
            "SELECT * FROM forge_build_results WHERE id = ?"
        )
        .bind(build_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Get build results
    pub async fn get_build_results(&self, forge_item_id: &str) -> Result<Vec<ForgeBuildResult>, ForgeRepoError> {
        let results = sqlx::query_as::<_, ForgeBuildResult>(
            "SELECT * FROM forge_build_results WHERE forge_item_id = ? ORDER BY started_at DESC"
        )
        .bind(forge_item_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }

    /// Search forge items
    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<ForgeItem>, ForgeRepoError> {
        let items = sqlx::query_as::<_, ForgeItem>(r#"
            SELECT f.* FROM forge_items f
            INNER JOIN forge_items_fts fts ON f.id = fts.id
            WHERE forge_items_fts MATCH ?
            ORDER BY rank
            LIMIT ?
        "#)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    /// Get statistics
    pub async fn stats(&self, spec_id: Option<&str>) -> Result<ForgeStats, ForgeRepoError> {
        let mut sql = r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN item_type = 'code' THEN 1 ELSE 0 END) as code_count,
                SUM(CASE WHEN item_type = 'test' THEN 1 ELSE 0 END) as test_count,
                SUM(CASE WHEN status = 'merged' THEN 1 ELSE 0 END) as merged_count,
                COALESCE(SUM(lines_of_code), 0) as total_lines,
                COALESCE(AVG(test_coverage), 0) as avg_coverage
            FROM forge_items
        "#.to_string();

        let stats = if let Some(sid) = spec_id {
            sql.push_str(" WHERE spec_id = ?");
            sqlx::query_as::<_, ForgeStats>(&sql)
                .bind(sid)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, ForgeStats>(&sql)
                .fetch_one(&self.pool)
                .await?
        };

        Ok(stats)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ForgeStats {
    pub total: i64,
    pub code_count: i64,
    pub test_count: i64,
    pub merged_count: i64,
    pub total_lines: i64,
    pub avg_coverage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/forge.rs` - Forge repository implementation
