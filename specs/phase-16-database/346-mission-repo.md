# Spec 346: Mission Repository

## Overview
Implement the repository pattern for Mission CRUD operations, providing a clean abstraction over database queries.

## Rust Implementation

### Repository Trait
```rust
// src/database/repository/traits.rs

use async_trait::async_trait;
use std::future::Future;

#[async_trait]
pub trait Repository<T, ID> {
    type Error;

    async fn find_by_id(&self, id: &ID) -> Result<Option<T>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;
    async fn create(&self, entity: &T) -> Result<T, Self::Error>;
    async fn update(&self, entity: &T) -> Result<T, Self::Error>;
    async fn delete(&self, id: &ID) -> Result<bool, Self::Error>;
    async fn exists(&self, id: &ID) -> Result<bool, Self::Error>;
    async fn count(&self) -> Result<i64, Self::Error>;
}
```

### Mission Repository Implementation
```rust
// src/database/repository/mission.rs

use super::traits::Repository;
use crate::database::schema::mission::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MissionRepoError {
    #[error("Mission not found: {0}")]
    NotFound(String),

    #[error("Duplicate mission: {0}")]
    Duplicate(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Input for creating a new mission
#[derive(Debug, Clone)]
pub struct CreateMission {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<MissionStatus>,
    pub priority: Option<MissionPriority>,
    pub parent_id: Option<String>,
    pub owner_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Input for updating a mission
#[derive(Debug, Clone, Default)]
pub struct UpdateMission {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<MissionStatus>,
    pub priority: Option<MissionPriority>,
    pub parent_id: Option<Option<String>>,
    pub owner_id: Option<Option<String>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub progress: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

/// Query filters for finding missions
#[derive(Debug, Clone, Default)]
pub struct MissionFilter {
    pub status: Option<Vec<MissionStatus>>,
    pub priority: Option<Vec<MissionPriority>>,
    pub parent_id: Option<Option<String>>,
    pub owner_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

/// Pagination options
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Sort options
#[derive(Debug, Clone)]
pub struct MissionSort {
    pub field: MissionSortField,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MissionSortField {
    #[default]
    CreatedAt,
    UpdatedAt,
    Title,
    Priority,
    DueDate,
    Progress,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}

impl MissionSort {
    fn to_sql(&self) -> String {
        let field = match self.field {
            MissionSortField::CreatedAt => "created_at",
            MissionSortField::UpdatedAt => "updated_at",
            MissionSortField::Title => "title",
            MissionSortField::Priority => "priority",
            MissionSortField::DueDate => "due_date",
            MissionSortField::Progress => "progress",
        };
        let dir = match self.direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };
        format!("{} {}", field, dir)
    }
}

/// Mission repository
pub struct MissionRepository {
    pool: SqlitePool,
}

impl MissionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new mission
    #[instrument(skip(self, input), fields(title = %input.title))]
    pub async fn create(&self, input: CreateMission) -> Result<Mission, MissionRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let status = input.status.unwrap_or_default();
        let priority = input.priority.unwrap_or_default();
        let tags_json = input.tags.map(|t| serde_json::to_string(&t).unwrap());
        let metadata_json = input.metadata.map(|m| m.to_string());

        sqlx::query(r#"
            INSERT INTO missions (
                id, title, description, status, priority,
                parent_id, owner_id, created_at, updated_at,
                due_date, progress, tags, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#)
        .bind(&id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(status)
        .bind(priority)
        .bind(&input.parent_id)
        .bind(&input.owner_id)
        .bind(now)
        .bind(now)
        .bind(input.due_date)
        .bind(&tags_json)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await?;

        // Insert tags into mission_tags table
        if let Some(tags) = &input.tags {
            for tag in tags {
                sqlx::query("INSERT OR IGNORE INTO mission_tags (mission_id, tag) VALUES (?, ?)")
                    .bind(&id)
                    .bind(tag)
                    .execute(&self.pool)
                    .await?;
            }
        }

        debug!("Created mission {}", id);
        self.find_by_id(&id).await?.ok_or(MissionRepoError::NotFound(id))
    }

    /// Find a mission by ID
    #[instrument(skip(self))]
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Mission>, MissionRepoError> {
        let mission = sqlx::query_as::<_, Mission>(
            "SELECT * FROM missions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(mission)
    }

    /// Find missions with filters
    #[instrument(skip(self))]
    pub async fn find_many(
        &self,
        filter: MissionFilter,
        pagination: Pagination,
        sort: Option<MissionSort>,
    ) -> Result<Vec<Mission>, MissionRepoError> {
        let mut sql = String::from("SELECT * FROM missions WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        // Build WHERE clauses
        if let Some(statuses) = &filter.status {
            if !statuses.is_empty() {
                let placeholders: Vec<_> = statuses.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND status IN ({})", placeholders.join(",")));
                for s in statuses {
                    bindings.push(format!("{:?}", s).to_lowercase());
                }
            }
        }

        if let Some(priorities) = &filter.priority {
            if !priorities.is_empty() {
                let placeholders: Vec<_> = priorities.iter().map(|_| "?").collect();
                sql.push_str(&format!(" AND priority IN ({})", placeholders.join(",")));
                for p in priorities {
                    bindings.push(format!("{:?}", p).to_lowercase());
                }
            }
        }

        if let Some(parent) = &filter.parent_id {
            match parent {
                Some(pid) => {
                    sql.push_str(" AND parent_id = ?");
                    bindings.push(pid.clone());
                }
                None => {
                    sql.push_str(" AND parent_id IS NULL");
                }
            }
        }

        if let Some(owner) = &filter.owner_id {
            sql.push_str(" AND owner_id = ?");
            bindings.push(owner.clone());
        }

        if let Some(search) = &filter.search {
            sql.push_str(" AND id IN (SELECT id FROM missions_fts WHERE missions_fts MATCH ?)");
            bindings.push(search.clone());
        }

        // Add sorting
        let sort_clause = sort.unwrap_or_default().to_sql();
        sql.push_str(&format!(" ORDER BY {}", sort_clause));

        // Add pagination
        sql.push_str(" LIMIT ? OFFSET ?");

        // Build and execute query
        let mut query = sqlx::query_as::<_, Mission>(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(pagination.limit).bind(pagination.offset);

        let missions = query.fetch_all(&self.pool).await?;
        Ok(missions)
    }

    /// Update a mission
    #[instrument(skip(self, input))]
    pub async fn update(&self, id: &str, input: UpdateMission) -> Result<Mission, MissionRepoError> {
        // Get current mission for history tracking
        let current = self.find_by_id(id).await?
            .ok_or_else(|| MissionRepoError::NotFound(id.to_string()))?;

        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(title) = &input.title {
            updates.push("title = ?");
            bindings.push(title.clone());
            self.record_history(id, "title", Some(&current.title), Some(title)).await?;
        }

        if let Some(desc) = &input.description {
            updates.push("description = ?");
            bindings.push(desc.clone().unwrap_or_default());
        }

        if let Some(status) = &input.status {
            updates.push("status = ?");
            bindings.push(format!("{:?}", status).to_lowercase());

            // Set started_at when becoming active
            if *status == MissionStatus::Active && current.started_at.is_none() {
                updates.push("started_at = datetime('now')");
            }

            // Set completed_at when completing
            if *status == MissionStatus::Completed && current.completed_at.is_none() {
                updates.push("completed_at = datetime('now')");
            }
        }

        if let Some(priority) = &input.priority {
            updates.push("priority = ?");
            bindings.push(format!("{:?}", priority).to_lowercase());
        }

        if let Some(progress) = input.progress {
            updates.push("progress = ?");
            bindings.push(progress.to_string());
        }

        if let Some(tags) = &input.tags {
            let tags_json = serde_json::to_string(tags).unwrap();
            updates.push("tags = ?");
            bindings.push(tags_json);

            // Update mission_tags table
            sqlx::query("DELETE FROM mission_tags WHERE mission_id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;

            for tag in tags {
                sqlx::query("INSERT INTO mission_tags (mission_id, tag) VALUES (?, ?)")
                    .bind(id)
                    .bind(tag)
                    .execute(&self.pool)
                    .await?;
            }
        }

        if updates.is_empty() {
            return Ok(current);
        }

        let sql = format!(
            "UPDATE missions SET {} WHERE id = ?",
            updates.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(id);

        query.execute(&self.pool).await?;

        debug!("Updated mission {}", id);
        self.find_by_id(id).await?.ok_or(MissionRepoError::NotFound(id.to_string()))
    }

    /// Delete a mission
    #[instrument(skip(self))]
    pub async fn delete(&self, id: &str) -> Result<bool, MissionRepoError> {
        let result = sqlx::query("DELETE FROM missions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get child missions
    pub async fn find_children(&self, parent_id: &str) -> Result<Vec<Mission>, MissionRepoError> {
        let missions = sqlx::query_as::<_, Mission>(
            "SELECT * FROM missions WHERE parent_id = ? ORDER BY created_at"
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(missions)
    }

    /// Get missions by tag
    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<Mission>, MissionRepoError> {
        let missions = sqlx::query_as::<_, Mission>(r#"
            SELECT m.* FROM missions m
            INNER JOIN mission_tags mt ON m.id = mt.mission_id
            WHERE mt.tag = ?
            ORDER BY m.created_at DESC
        "#)
        .bind(tag)
        .fetch_all(&self.pool)
        .await?;

        Ok(missions)
    }

    /// Search missions using full-text search
    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<Mission>, MissionRepoError> {
        let missions = sqlx::query_as::<_, Mission>(r#"
            SELECT m.* FROM missions m
            INNER JOIN missions_fts fts ON m.id = fts.id
            WHERE missions_fts MATCH ?
            ORDER BY rank
            LIMIT ?
        "#)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(missions)
    }

    /// Get mission statistics
    pub async fn stats(&self) -> Result<MissionStats, MissionRepoError> {
        let row = sqlx::query_as::<_, MissionStats>(r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active,
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed,
                SUM(CASE WHEN status = 'draft' THEN 1 ELSE 0 END) as draft,
                SUM(CASE WHEN due_date < datetime('now') AND status NOT IN ('completed', 'archived') THEN 1 ELSE 0 END) as overdue
            FROM missions
        "#)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    /// Record history entry
    async fn record_history(
        &self,
        mission_id: &str,
        field: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
    ) -> Result<(), MissionRepoError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(r#"
            INSERT INTO mission_history (id, mission_id, field_name, old_value, new_value)
            VALUES (?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(mission_id)
        .bind(field)
        .bind(old_value)
        .bind(new_value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MissionStats {
    pub total: i64,
    pub active: i64,
    pub completed: i64,
    pub draft: i64,
    pub overdue: i64,
}

impl Default for MissionSort {
    fn default() -> Self {
        Self {
            field: MissionSortField::CreatedAt,
            direction: SortDirection::Desc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup() -> MissionRepository {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Run migrations
        sqlx::query(include_str!("../migrations/001_create_missions.sql"))
            .execute(&pool)
            .await
            .unwrap();

        MissionRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let repo = setup().await;

        let input = CreateMission {
            title: "Test Mission".to_string(),
            description: Some("A test mission".to_string()),
            status: None,
            priority: Some(MissionPriority::High),
            parent_id: None,
            owner_id: None,
            due_date: None,
            tags: Some(vec!["test".to_string(), "demo".to_string()]),
            metadata: None,
        };

        let created = repo.create(input).await.unwrap();
        assert_eq!(created.title, "Test Mission");
        assert_eq!(created.priority, MissionPriority::High);

        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn test_update() {
        let repo = setup().await;

        let created = repo.create(CreateMission {
            title: "Original Title".to_string(),
            ..Default::default()
        }).await.unwrap();

        let updated = repo.update(&created.id, UpdateMission {
            title: Some("Updated Title".to_string()),
            status: Some(MissionStatus::Active),
            ..Default::default()
        }).await.unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.status, MissionStatus::Active);
        assert!(updated.started_at.is_some());
    }
}
```

## Files to Create
- `src/database/repository/traits.rs` - Repository trait
- `src/database/repository/mission.rs` - Mission repository
- `src/database/repository/mod.rs` - Module exports
