# Spec 345: Mission Database Schema

## Overview
Define the SQLite schema for storing missions (high-level objectives), including tables, indexes, and relationships.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Schema Definition
```rust
// src/database/schema/mission.rs

use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Mission status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum MissionStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Archived,
    Failed,
}

impl Default for MissionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Mission priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum MissionPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for MissionPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Mission database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: MissionStatus,
    pub priority: MissionPriority,
    pub parent_id: Option<String>,
    pub owner_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub progress: i32,
    pub tags: Option<String>, // JSON array stored as text
    pub metadata: Option<String>, // JSON object stored as text
}

/// Mission tag association
#[derive(Debug, Clone, FromRow)]
pub struct MissionTag {
    pub mission_id: String,
    pub tag: String,
    pub created_at: DateTime<Utc>,
}

/// Mission dependency
#[derive(Debug, Clone, FromRow)]
pub struct MissionDependency {
    pub id: String,
    pub mission_id: String,
    pub depends_on_id: String,
    pub dependency_type: DependencyType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum DependencyType {
    BlockedBy,
    RelatedTo,
    ParentOf,
}

/// Mission history entry
#[derive(Debug, Clone, FromRow)]
pub struct MissionHistory {
    pub id: String,
    pub mission_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}
```

### Migration SQL
```rust
// src/database/migrations/001_create_missions.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000001,
        "create_missions",
        r#"
-- Missions table
CREATE TABLE IF NOT EXISTS missions (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'active', 'paused', 'completed', 'archived', 'failed')),
    priority TEXT NOT NULL DEFAULT 'medium'
        CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    parent_id TEXT REFERENCES missions(id) ON DELETE SET NULL,
    owner_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    due_date TEXT,
    progress INTEGER NOT NULL DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    tags TEXT, -- JSON array
    metadata TEXT -- JSON object
);

-- Indexes for missions
CREATE INDEX IF NOT EXISTS idx_missions_status ON missions(status);
CREATE INDEX IF NOT EXISTS idx_missions_priority ON missions(priority);
CREATE INDEX IF NOT EXISTS idx_missions_parent_id ON missions(parent_id);
CREATE INDEX IF NOT EXISTS idx_missions_owner_id ON missions(owner_id);
CREATE INDEX IF NOT EXISTS idx_missions_created_at ON missions(created_at);
CREATE INDEX IF NOT EXISTS idx_missions_due_date ON missions(due_date);

-- Mission tags table (for efficient tag queries)
CREATE TABLE IF NOT EXISTS mission_tags (
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    tag TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (mission_id, tag)
);

CREATE INDEX IF NOT EXISTS idx_mission_tags_tag ON mission_tags(tag);

-- Mission dependencies
CREATE TABLE IF NOT EXISTS mission_dependencies (
    id TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    depends_on_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    dependency_type TEXT NOT NULL DEFAULT 'blocked_by'
        CHECK (dependency_type IN ('blocked_by', 'related_to', 'parent_of')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (mission_id, depends_on_id, dependency_type)
);

CREATE INDEX IF NOT EXISTS idx_mission_deps_mission ON mission_dependencies(mission_id);
CREATE INDEX IF NOT EXISTS idx_mission_deps_depends_on ON mission_dependencies(depends_on_id);

-- Mission history for audit trail
CREATE TABLE IF NOT EXISTS mission_history (
    id TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by TEXT,
    changed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mission_history_mission ON mission_history(mission_id);
CREATE INDEX IF NOT EXISTS idx_mission_history_changed_at ON mission_history(changed_at);

-- Trigger to update updated_at
CREATE TRIGGER IF NOT EXISTS update_missions_timestamp
AFTER UPDATE ON missions
BEGIN
    UPDATE missions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Full-text search for missions (optional, for search functionality)
CREATE VIRTUAL TABLE IF NOT EXISTS missions_fts USING fts5(
    id,
    title,
    description,
    tags,
    content='missions',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS missions_fts_insert AFTER INSERT ON missions BEGIN
    INSERT INTO missions_fts(rowid, id, title, description, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.tags);
END;

CREATE TRIGGER IF NOT EXISTS missions_fts_delete AFTER DELETE ON missions BEGIN
    INSERT INTO missions_fts(missions_fts, rowid, id, title, description, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.tags);
END;

CREATE TRIGGER IF NOT EXISTS missions_fts_update AFTER UPDATE ON missions BEGIN
    INSERT INTO missions_fts(missions_fts, rowid, id, title, description, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.tags);
    INSERT INTO missions_fts(rowid, id, title, description, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.tags);
END;
"#
    ).with_down(r#"
DROP TRIGGER IF EXISTS missions_fts_update;
DROP TRIGGER IF EXISTS missions_fts_delete;
DROP TRIGGER IF EXISTS missions_fts_insert;
DROP TABLE IF EXISTS missions_fts;
DROP TRIGGER IF EXISTS update_missions_timestamp;
DROP TABLE IF EXISTS mission_history;
DROP TABLE IF EXISTS mission_dependencies;
DROP TABLE IF EXISTS mission_tags;
DROP TABLE IF EXISTS missions;
"#)
}
```

### Schema Validation
```rust
// src/database/schema/validation.rs

use sqlx::sqlite::SqlitePool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Table {0} not found")]
    TableNotFound(String),

    #[error("Column {0}.{1} not found")]
    ColumnNotFound(String, String),

    #[error("Index {0} not found")]
    IndexNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub struct SchemaValidator {
    pool: SqlitePool,
}

impl SchemaValidator {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn validate_missions_schema(&self) -> Result<(), SchemaError> {
        // Check tables exist
        self.check_table_exists("missions").await?;
        self.check_table_exists("mission_tags").await?;
        self.check_table_exists("mission_dependencies").await?;
        self.check_table_exists("mission_history").await?;

        // Check required columns
        let mission_columns = vec![
            "id", "title", "description", "status", "priority",
            "parent_id", "owner_id", "created_at", "updated_at",
            "started_at", "completed_at", "due_date", "progress",
            "tags", "metadata"
        ];

        for col in mission_columns {
            self.check_column_exists("missions", col).await?;
        }

        // Check indexes
        self.check_index_exists("idx_missions_status").await?;
        self.check_index_exists("idx_missions_priority").await?;

        Ok(())
    }

    async fn check_table_exists(&self, table: &str) -> Result<(), SchemaError> {
        let exists: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(table)
        .fetch_one(&self.pool)
        .await?;

        if exists.0 == 0 {
            return Err(SchemaError::TableNotFound(table.to_string()));
        }

        Ok(())
    }

    async fn check_column_exists(&self, table: &str, column: &str) -> Result<(), SchemaError> {
        let columns: Vec<(String,)> = sqlx::query_as(
            &format!("PRAGMA table_info({})", table)
        )
        .fetch_all(&self.pool)
        .await?;

        // table_info returns (cid, name, type, notnull, dflt_value, pk)
        // We need to check the 'name' column (index 1)
        let column_exists = columns.iter().any(|c| c.0 == column);

        if !column_exists {
            return Err(SchemaError::ColumnNotFound(table.to_string(), column.to_string()));
        }

        Ok(())
    }

    async fn check_index_exists(&self, index: &str) -> Result<(), SchemaError> {
        let exists: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?"
        )
        .bind(index)
        .fetch_one(&self.pool)
        .await?;

        if exists.0 == 0 {
            return Err(SchemaError::IndexNotFound(index.to_string()));
        }

        Ok(())
    }
}
```

## Schema Design Decisions

1. **Text IDs**: Using UUIDs as text for portability
2. **JSON in Text**: Tags and metadata as JSON text for flexibility
3. **FTS5**: Full-text search for mission discovery
4. **Soft Relationships**: Parent-child via optional parent_id
5. **Audit Trail**: Separate history table for changes

## Files to Create
- `src/database/schema/mission.rs` - Mission models
- `src/database/migrations/001_create_missions.rs` - Migration
- `src/database/schema/validation.rs` - Schema validation
