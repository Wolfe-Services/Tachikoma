# Spec 347: Spec Database Schema

## Overview
Define the SQLite schema for storing specifications (detailed technical requirements derived from missions).


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Schema Models
```rust
// src/database/schema/spec.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Spec status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum SpecStatus {
    Draft,
    Review,
    Approved,
    Implementation,
    Testing,
    Complete,
    Rejected,
}

impl Default for SpecStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Spec complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum SpecComplexity {
    Trivial,
    Simple,
    Medium,
    Complex,
    Epic,
}

impl Default for SpecComplexity {
    fn default() -> Self {
        Self::Medium
    }
}

/// Spec database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Spec {
    pub id: String,
    pub mission_id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,  // Full spec content (markdown)
    pub status: SpecStatus,
    pub complexity: SpecComplexity,
    pub version: i32,
    pub author_id: Option<String>,
    pub reviewer_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub tags: Option<String>,
    pub acceptance_criteria: Option<String>,  // JSON array
    pub metadata: Option<String>,
}

/// Spec version history
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SpecVersion {
    pub id: String,
    pub spec_id: String,
    pub version: i32,
    pub title: String,
    pub content: Option<String>,
    pub change_summary: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Spec comment
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SpecComment {
    pub id: String,
    pub spec_id: String,
    pub parent_id: Option<String>,
    pub author_id: Option<String>,
    pub content: String,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Spec file attachment
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SpecAttachment {
    pub id: String,
    pub spec_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub uploaded_by: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}

/// Acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<String>,
}
```

### Migration SQL
```rust
// src/database/migrations/002_create_specs.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000002,
        "create_specs",
        r#"
-- Specs table
CREATE TABLE IF NOT EXISTS specs (
    id TEXT PRIMARY KEY NOT NULL,
    mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    content TEXT,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'review', 'approved', 'implementation', 'testing', 'complete', 'rejected')),
    complexity TEXT NOT NULL DEFAULT 'medium'
        CHECK (complexity IN ('trivial', 'simple', 'medium', 'complex', 'epic')),
    version INTEGER NOT NULL DEFAULT 1,
    author_id TEXT,
    reviewer_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    approved_at TEXT,
    estimated_hours REAL,
    actual_hours REAL,
    tags TEXT,
    acceptance_criteria TEXT,
    metadata TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_specs_mission_id ON specs(mission_id);
CREATE INDEX IF NOT EXISTS idx_specs_status ON specs(status);
CREATE INDEX IF NOT EXISTS idx_specs_author_id ON specs(author_id);
CREATE INDEX IF NOT EXISTS idx_specs_created_at ON specs(created_at);

-- Spec versions for history
CREATE TABLE IF NOT EXISTS spec_versions (
    id TEXT PRIMARY KEY NOT NULL,
    spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    change_summary TEXT,
    created_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(spec_id, version)
);

CREATE INDEX IF NOT EXISTS idx_spec_versions_spec ON spec_versions(spec_id);

-- Spec comments for collaboration
CREATE TABLE IF NOT EXISTS spec_comments (
    id TEXT PRIMARY KEY NOT NULL,
    spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    parent_id TEXT REFERENCES spec_comments(id) ON DELETE CASCADE,
    author_id TEXT,
    content TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_spec_comments_spec ON spec_comments(spec_id);
CREATE INDEX IF NOT EXISTS idx_spec_comments_parent ON spec_comments(parent_id);

-- Spec attachments
CREATE TABLE IF NOT EXISTS spec_attachments (
    id TEXT PRIMARY KEY NOT NULL,
    spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    storage_path TEXT NOT NULL,
    uploaded_by TEXT,
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_spec_attachments_spec ON spec_attachments(spec_id);

-- Spec links (relationships between specs)
CREATE TABLE IF NOT EXISTS spec_links (
    id TEXT PRIMARY KEY NOT NULL,
    source_spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    target_spec_id TEXT NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
    link_type TEXT NOT NULL DEFAULT 'related'
        CHECK (link_type IN ('related', 'blocks', 'implements', 'extends', 'supersedes')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_spec_id, target_spec_id, link_type)
);

CREATE INDEX IF NOT EXISTS idx_spec_links_source ON spec_links(source_spec_id);
CREATE INDEX IF NOT EXISTS idx_spec_links_target ON spec_links(target_spec_id);

-- Update timestamp trigger
CREATE TRIGGER IF NOT EXISTS update_specs_timestamp
AFTER UPDATE ON specs
BEGIN
    UPDATE specs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Version creation trigger
CREATE TRIGGER IF NOT EXISTS create_spec_version
AFTER UPDATE OF content, title ON specs
WHEN OLD.content IS NOT NEW.content OR OLD.title IS NOT NEW.title
BEGIN
    INSERT INTO spec_versions (id, spec_id, version, title, content, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        NEW.id,
        NEW.version,
        NEW.title,
        NEW.content,
        datetime('now')
    );
END;

-- Full-text search for specs
CREATE VIRTUAL TABLE IF NOT EXISTS specs_fts USING fts5(
    id,
    title,
    description,
    content,
    tags,
    content='specs',
    content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS specs_fts_insert AFTER INSERT ON specs BEGIN
    INSERT INTO specs_fts(rowid, id, title, description, content, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.content, NEW.tags);
END;

CREATE TRIGGER IF NOT EXISTS specs_fts_delete AFTER DELETE ON specs BEGIN
    INSERT INTO specs_fts(specs_fts, rowid, id, title, description, content, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.content, OLD.tags);
END;

CREATE TRIGGER IF NOT EXISTS specs_fts_update AFTER UPDATE ON specs BEGIN
    INSERT INTO specs_fts(specs_fts, rowid, id, title, description, content, tags)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.title, OLD.description, OLD.content, OLD.tags);
    INSERT INTO specs_fts(rowid, id, title, description, content, tags)
    VALUES (NEW.rowid, NEW.id, NEW.title, NEW.description, NEW.content, NEW.tags);
END;
"#
    ).with_down(r#"
DROP TRIGGER IF EXISTS specs_fts_update;
DROP TRIGGER IF EXISTS specs_fts_delete;
DROP TRIGGER IF EXISTS specs_fts_insert;
DROP TABLE IF EXISTS specs_fts;
DROP TRIGGER IF EXISTS create_spec_version;
DROP TRIGGER IF EXISTS update_specs_timestamp;
DROP TABLE IF EXISTS spec_links;
DROP TABLE IF EXISTS spec_attachments;
DROP TABLE IF EXISTS spec_comments;
DROP TABLE IF EXISTS spec_versions;
DROP TABLE IF EXISTS specs;
"#)
}
```

### Helper Functions
```rust
// src/database/schema/spec_helpers.rs

use super::spec::*;
use serde_json;

impl Spec {
    /// Parse acceptance criteria from JSON
    pub fn get_acceptance_criteria(&self) -> Vec<AcceptanceCriterion> {
        self.acceptance_criteria
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    /// Set acceptance criteria as JSON
    pub fn set_acceptance_criteria(&mut self, criteria: Vec<AcceptanceCriterion>) {
        self.acceptance_criteria = Some(serde_json::to_string(&criteria).unwrap());
    }

    /// Check if all acceptance criteria are verified
    pub fn all_criteria_verified(&self) -> bool {
        let criteria = self.get_acceptance_criteria();
        !criteria.is_empty() && criteria.iter().all(|c| c.verified)
    }

    /// Get completion percentage based on acceptance criteria
    pub fn completion_percentage(&self) -> f64 {
        let criteria = self.get_acceptance_criteria();
        if criteria.is_empty() {
            return 0.0;
        }
        let verified = criteria.iter().filter(|c| c.verified).count();
        (verified as f64 / criteria.len() as f64) * 100.0
    }

    /// Check if spec is editable (draft or rejected)
    pub fn is_editable(&self) -> bool {
        matches!(self.status, SpecStatus::Draft | SpecStatus::Rejected)
    }

    /// Get estimated complexity in hours
    pub fn complexity_hours(&self) -> f64 {
        match self.complexity {
            SpecComplexity::Trivial => 0.5,
            SpecComplexity::Simple => 2.0,
            SpecComplexity::Medium => 8.0,
            SpecComplexity::Complex => 24.0,
            SpecComplexity::Epic => 80.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptance_criteria() {
        let mut spec = Spec {
            id: "test".to_string(),
            mission_id: "mission".to_string(),
            title: "Test Spec".to_string(),
            description: None,
            content: None,
            status: SpecStatus::Draft,
            complexity: SpecComplexity::Medium,
            version: 1,
            author_id: None,
            reviewer_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            approved_at: None,
            estimated_hours: None,
            actual_hours: None,
            tags: None,
            acceptance_criteria: None,
            metadata: None,
        };

        let criteria = vec![
            AcceptanceCriterion {
                id: "1".to_string(),
                description: "Test criterion".to_string(),
                verified: false,
                verified_at: None,
                verified_by: None,
            }
        ];

        spec.set_acceptance_criteria(criteria);
        let retrieved = spec.get_acceptance_criteria();

        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].description, "Test criterion");
    }
}
```

## Schema Design Decisions

1. **Version History**: Automatic versioning via triggers
2. **Acceptance Criteria**: JSON array for flexible validation
3. **Comments**: Threaded via parent_id
4. **Links**: Many-to-many relationships between specs

## Files to Create
- `src/database/schema/spec.rs` - Spec models
- `src/database/schema/spec_helpers.rs` - Helper methods
- `src/database/migrations/002_create_specs.rs` - Migration
