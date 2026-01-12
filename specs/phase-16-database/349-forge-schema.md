# Spec 349: Forge Database Schema

## Overview
Define the SQLite schema for storing forge items (implementation artifacts including code, tests, and documentation).

## Rust Implementation

### Schema Models
```rust
// src/database/schema/forge.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Forge item type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum ForgeItemType {
    Code,
    Test,
    Documentation,
    Config,
    Script,
    Asset,
    Other,
}

impl Default for ForgeItemType {
    fn default() -> Self {
        Self::Code
    }
}

/// Forge item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum ForgeItemStatus {
    Pending,
    InProgress,
    Review,
    Approved,
    Merged,
    Rejected,
}

impl Default for ForgeItemStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Forge item (implementation artifact)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ForgeItem {
    pub id: String,
    pub spec_id: String,
    pub item_type: ForgeItemType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: Option<String>,
    pub content: Option<String>,
    pub language: Option<String>,
    pub status: ForgeItemStatus,
    pub author_id: Option<String>,
    pub reviewer_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub lines_of_code: Option<i32>,
    pub test_coverage: Option<f64>,
    pub metadata: Option<String>,
}

/// Forge revision (version history)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ForgeRevision {
    pub id: String,
    pub forge_item_id: String,
    pub revision_number: i32,
    pub content: Option<String>,
    pub change_description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub diff: Option<String>,
}

/// Code review comment
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ForgeReviewComment {
    pub id: String,
    pub forge_item_id: String,
    pub revision_id: Option<String>,
    pub author_id: Option<String>,
    pub line_number: Option<i32>,
    pub content: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Build/test result
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ForgeBuildResult {
    pub id: String,
    pub forge_item_id: String,
    pub revision_id: Option<String>,
    pub build_type: BuildType,
    pub status: BuildStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum BuildType {
    Lint,
    Test,
    Build,
    Deploy,
    Integration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum BuildStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

/// Dependency between forge items
#[derive(Debug, Clone, FromRow)]
pub struct ForgeDependency {
    pub id: String,
    pub forge_item_id: String,
    pub depends_on_id: String,
    pub dependency_type: String,
    pub created_at: DateTime<Utc>,
}
```

### Migration SQL
```rust
// src/database/migrations/003_create_forge.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000003,
        "create_forge",
        r#"
-- Forge items table
CREATE TABLE IF NOT EXISTS forge_items (
    id TEXT PRIMARY KEY NOT NULL,
    spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL DEFAULT 'code'
        CHECK (item_type IN ('code', 'test', 'documentation', 'config', 'script', 'asset', 'other')),
    title TEXT NOT NULL,
    description TEXT,
    file_path TEXT,
    content TEXT,
    language TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'review', 'approved', 'merged', 'rejected')),
    author_id TEXT,
    reviewer_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    reviewed_at TEXT,
    lines_of_code INTEGER,
    test_coverage REAL,
    metadata TEXT
);

-- Indexes for forge_items
CREATE INDEX IF NOT EXISTS idx_forge_items_spec_id ON forge_items(spec_id);
CREATE INDEX IF NOT EXISTS idx_forge_items_type ON forge_items(item_type);
CREATE INDEX IF NOT EXISTS idx_forge_items_status ON forge_items(status);
CREATE INDEX IF NOT EXISTS idx_forge_items_author ON forge_items(author_id);
CREATE INDEX IF NOT EXISTS idx_forge_items_file_path ON forge_items(file_path);

-- Forge revisions (version history)
CREATE TABLE IF NOT EXISTS forge_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    forge_item_id TEXT NOT NULL REFERENCES forge_items(id) ON DELETE CASCADE,
    revision_number INTEGER NOT NULL,
    content TEXT,
    change_description TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    diff TEXT,
    UNIQUE(forge_item_id, revision_number)
);

CREATE INDEX IF NOT EXISTS idx_forge_revisions_item ON forge_revisions(forge_item_id);

-- Code review comments
CREATE TABLE IF NOT EXISTS forge_review_comments (
    id TEXT PRIMARY KEY NOT NULL,
    forge_item_id TEXT NOT NULL REFERENCES forge_items(id) ON DELETE CASCADE,
    revision_id TEXT REFERENCES forge_revisions(id) ON DELETE SET NULL,
    author_id TEXT,
    line_number INTEGER,
    content TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_forge_comments_item ON forge_review_comments(forge_item_id);
CREATE INDEX IF NOT EXISTS idx_forge_comments_revision ON forge_review_comments(revision_id);

-- Build/test results
CREATE TABLE IF NOT EXISTS forge_build_results (
    id TEXT PRIMARY KEY NOT NULL,
    forge_item_id TEXT NOT NULL REFERENCES forge_items(id) ON DELETE CASCADE,
    revision_id TEXT REFERENCES forge_revisions(id) ON DELETE SET NULL,
    build_type TEXT NOT NULL DEFAULT 'build'
        CHECK (build_type IN ('lint', 'test', 'build', 'deploy', 'integration')),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'success', 'failed', 'cancelled')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    duration_ms INTEGER,
    output TEXT,
    error_message TEXT,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_forge_builds_item ON forge_build_results(forge_item_id);
CREATE INDEX IF NOT EXISTS idx_forge_builds_status ON forge_build_results(status);

-- Forge dependencies
CREATE TABLE IF NOT EXISTS forge_dependencies (
    id TEXT PRIMARY KEY NOT NULL,
    forge_item_id TEXT NOT NULL REFERENCES forge_items(id) ON DELETE CASCADE,
    depends_on_id TEXT NOT NULL REFERENCES forge_items(id) ON DELETE CASCADE,
    dependency_type TEXT NOT NULL DEFAULT 'requires',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(forge_item_id, depends_on_id)
);

CREATE INDEX IF NOT EXISTS idx_forge_deps_item ON forge_dependencies(forge_item_id);
CREATE INDEX IF NOT EXISTS idx_forge_deps_depends ON forge_dependencies(depends_on_id);

-- Update timestamp trigger
CREATE TRIGGER IF NOT EXISTS update_forge_items_timestamp
AFTER UPDATE ON forge_items
BEGIN
    UPDATE forge_items SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Auto-create revision on content update
CREATE TRIGGER IF NOT EXISTS create_forge_revision
AFTER UPDATE OF content ON forge_items
WHEN OLD.content IS NOT NEW.content
BEGIN
    INSERT INTO forge_revisions (
        id, forge_item_id, revision_number, content, created_at
    )
    SELECT
        lower(hex(randomblob(16))),
        NEW.id,
        COALESCE((SELECT MAX(revision_number) FROM forge_revisions WHERE forge_item_id = NEW.id), 0) + 1,
        NEW.content,
        datetime('now');
END;

-- Full-text search for forge items
CREATE VIRTUAL TABLE IF NOT EXISTS forge_items_fts USING fts5(
    id,
    title,
    description,
    content,
    file_path,
    content='forge_items',
    content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS forge_items_fts_insert AFTER INSERT ON forge_items BEGIN
    INSERT INTO forge_items_fts(rowid, id, title, description, content, file_path)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.content, NEW.file_path);
END;

CREATE TRIGGER IF NOT EXISTS forge_items_fts_delete AFTER DELETE ON forge_items BEGIN
    INSERT INTO forge_items_fts(forge_items_fts, rowid, id, title, description, content, file_path)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.content, OLD.file_path);
END;

CREATE TRIGGER IF NOT EXISTS forge_items_fts_update AFTER UPDATE ON forge_items BEGIN
    INSERT INTO forge_items_fts(forge_items_fts, rowid, id, title, description, content, file_path)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.content, OLD.file_path);
    INSERT INTO forge_items_fts(rowid, id, title, description, content, file_path)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.content, NEW.file_path);
END;
"#
    ).with_down(r#"
DROP TRIGGER IF EXISTS forge_items_fts_update;
DROP TRIGGER IF EXISTS forge_items_fts_delete;
DROP TRIGGER IF EXISTS forge_items_fts_insert;
DROP TABLE IF EXISTS forge_items_fts;
DROP TRIGGER IF EXISTS create_forge_revision;
DROP TRIGGER IF EXISTS update_forge_items_timestamp;
DROP TABLE IF EXISTS forge_dependencies;
DROP TABLE IF EXISTS forge_build_results;
DROP TABLE IF EXISTS forge_review_comments;
DROP TABLE IF EXISTS forge_revisions;
DROP TABLE IF EXISTS forge_items;
"#)
}
```

### Language Detection Helper
```rust
// src/database/schema/forge_helpers.rs

impl ForgeItem {
    /// Detect language from file extension
    pub fn detect_language_from_path(path: &str) -> Option<String> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())?;

        let lang = match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" | "cc" => "cpp",
            "rb" => "ruby",
            "php" => "php",
            "swift" => "swift",
            "kt" => "kotlin",
            "scala" => "scala",
            "sql" => "sql",
            "sh" | "bash" => "shell",
            "yaml" | "yml" => "yaml",
            "json" => "json",
            "toml" => "toml",
            "md" | "markdown" => "markdown",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            _ => return None,
        };

        Some(lang.to_string())
    }

    /// Count lines of code
    pub fn count_lines(&self) -> Option<i32> {
        self.content.as_ref().map(|c| {
            c.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with('#')
                })
                .count() as i32
        })
    }

    /// Check if item is code
    pub fn is_code(&self) -> bool {
        matches!(self.item_type, ForgeItemType::Code | ForgeItemType::Test | ForgeItemType::Script)
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.file_path.as_ref().and_then(|p| {
            std::path::Path::new(p)
                .extension()
                .and_then(|e| e.to_str())
        })
    }
}
```

## Schema Design Decisions

1. **Version Control**: Automatic revisions on content change
2. **Code Review**: Line-level comments with resolution
3. **Build Integration**: Track lint/test/build results
4. **Language Detection**: Automatic from file extension
5. **Dependencies**: Track implementation dependencies

## Files to Create
- `src/database/schema/forge.rs` - Forge models
- `src/database/schema/forge_helpers.rs` - Helper methods
- `src/database/migrations/003_create_forge.rs` - Migration
