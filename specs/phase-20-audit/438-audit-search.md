# 438 - Audit Search

**Phase:** 20 - Audit System
**Spec ID:** 438
**Status:** Planned
**Dependencies:** 435-audit-query
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement full-text search capabilities for audit events, enabling complex searches across event metadata and content.

---

## Acceptance Criteria

- [x] SQLite FTS5 integration
- [x] Search query parsing
- [x] Highlighted search results
- [x] Search suggestions/autocomplete
- [x] Search result ranking

---

## Implementation Details

### 1. Search Schema (src/search_schema.rs)

```rust
//! Full-text search schema for audit events.

/// FTS5 table schema for audit search.
pub const AUDIT_FTS_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS audit_fts USING fts5(
    event_id,
    category,
    action,
    actor_info,
    target_info,
    outcome_info,
    metadata_text,
    content='audit_events',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS audit_fts_insert AFTER INSERT ON audit_events BEGIN
    INSERT INTO audit_fts(rowid, event_id, category, action, actor_info, target_info, outcome_info, metadata_text)
    VALUES (
        NEW.rowid,
        NEW.id,
        NEW.category,
        NEW.action,
        COALESCE(NEW.actor_type || ' ' || COALESCE(NEW.actor_id, '') || ' ' || COALESCE(NEW.actor_name, ''), ''),
        COALESCE(NEW.target_type || ' ' || COALESCE(NEW.target_id, '') || ' ' || COALESCE(NEW.target_name, ''), ''),
        COALESCE(NEW.outcome || ' ' || COALESCE(NEW.outcome_reason, ''), ''),
        COALESCE(NEW.metadata, '')
    );
END;

CREATE TRIGGER IF NOT EXISTS audit_fts_delete AFTER DELETE ON audit_events BEGIN
    INSERT INTO audit_fts(audit_fts, rowid, event_id, category, action, actor_info, target_info, outcome_info, metadata_text)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.category, OLD.action, '', '', '', '');
END;

CREATE TRIGGER IF NOT EXISTS audit_fts_update AFTER UPDATE ON audit_events BEGIN
    INSERT INTO audit_fts(audit_fts, rowid, event_id, category, action, actor_info, target_info, outcome_info, metadata_text)
    VALUES ('delete', OLD.rowid, OLD.id, OLD.category, OLD.action, '', '', '', '');
    INSERT INTO audit_fts(rowid, event_id, category, action, actor_info, target_info, outcome_info, metadata_text)
    VALUES (
        NEW.rowid,
        NEW.id,
        NEW.category,
        NEW.action,
        COALESCE(NEW.actor_type || ' ' || COALESCE(NEW.actor_id, '') || ' ' || COALESCE(NEW.actor_name, ''), ''),
        COALESCE(NEW.target_type || ' ' || COALESCE(NEW.target_id, '') || ' ' || COALESCE(NEW.target_name, ''), ''),
        COALESCE(NEW.outcome || ' ' || COALESCE(NEW.outcome_reason, ''), ''),
        COALESCE(NEW.metadata, '')
    );
END;
"#;

/// Rebuild the FTS index.
pub const REBUILD_FTS: &str = "INSERT INTO audit_fts(audit_fts) VALUES ('rebuild');";

/// Optimize the FTS index.
pub const OPTIMIZE_FTS: &str = "INSERT INTO audit_fts(audit_fts) VALUES ('optimize');";
```

### 2. Search Query Parser (src/search_parser.rs)

```rust
//! Search query parsing for audit search.

use serde::{Deserialize, Serialize};

/// Parsed search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    /// Raw query string.
    pub raw: String,
    /// FTS5-compatible query.
    pub fts_query: String,
    /// Extracted field filters.
    pub filters: Vec<FieldFilter>,
    /// Terms for highlighting.
    pub highlight_terms: Vec<String>,
}

/// Field-specific filter extracted from query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFilter {
    pub field: String,
    pub value: String,
    pub operator: FilterOperator,
}

/// Filter operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    Contains,
    StartsWith,
    NotEquals,
}

/// Search query parser.
pub struct SearchQueryParser;

impl SearchQueryParser {
    /// Parse a search query string.
    pub fn parse(query: &str) -> ParsedQuery {
        let mut fts_parts = Vec::new();
        let mut filters = Vec::new();
        let mut highlight_terms = Vec::new();

        let tokens = Self::tokenize(query);

        for token in tokens {
            if let Some((field, value)) = Self::parse_field_filter(&token) {
                filters.push(FieldFilter {
                    field: field.to_string(),
                    value: value.to_string(),
                    operator: FilterOperator::Equals,
                });
            } else if token.starts_with('-') {
                // Negation
                let term = &token[1..];
                fts_parts.push(format!("NOT {}", Self::escape_fts(term)));
            } else if token.starts_with('"') && token.ends_with('"') {
                // Phrase search
                let phrase = &token[1..token.len()-1];
                fts_parts.push(format!("\"{}\"", Self::escape_fts(phrase)));
                highlight_terms.push(phrase.to_string());
            } else if token.contains('*') {
                // Wildcard/prefix search
                fts_parts.push(format!("{}*", Self::escape_fts(&token.replace('*', ""))));
                highlight_terms.push(token.replace('*', ""));
            } else {
                fts_parts.push(Self::escape_fts(&token));
                highlight_terms.push(token);
            }
        }

        let fts_query = if fts_parts.is_empty() {
            "*".to_string()
        } else {
            fts_parts.join(" AND ")
        };

        ParsedQuery {
            raw: query.to_string(),
            fts_query,
            filters,
            highlight_terms,
        }
    }

    fn tokenize(query: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in query.chars() {
            match c {
                '"' => {
                    if in_quotes {
                        current.push(c);
                        tokens.push(current.clone());
                        current.clear();
                        in_quotes = false;
                    } else {
                        if !current.is_empty() {
                            tokens.push(current.clone());
                            current.clear();
                        }
                        current.push(c);
                        in_quotes = true;
                    }
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    fn parse_field_filter(token: &str) -> Option<(&str, &str)> {
        let parts: Vec<&str> = token.splitn(2, ':').collect();
        if parts.len() == 2 {
            let field = parts[0];
            let value = parts[1];
            // Valid field names
            let valid_fields = ["category", "action", "severity", "actor", "target", "outcome"];
            if valid_fields.contains(&field) {
                return Some((field, value));
            }
        }
        None
    }

    fn escape_fts(s: &str) -> String {
        // Escape special FTS5 characters
        s.replace('"', "\"\"")
         .replace('\'', "''")
    }
}
```

### 3. Search Executor (src/search.rs)

```rust
//! Audit event search execution.

use crate::search_parser::{ParsedQuery, SearchQueryParser};
use parking_lot::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Search configuration.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum results to return.
    pub max_results: u32,
    /// Enable snippet generation.
    pub enable_snippets: bool,
    /// Snippet length in tokens.
    pub snippet_length: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 100,
            enable_snippets: true,
            snippet_length: 64,
        }
    }
}

/// Search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub event_id: String,
    pub timestamp: String,
    pub category: String,
    pub action: String,
    pub severity: String,
    pub snippet: Option<String>,
    pub rank: f64,
}

/// Search results page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub total_matches: u64,
    pub results: Vec<SearchResult>,
    pub suggestions: Vec<String>,
}

/// Audit search executor.
pub struct AuditSearch {
    conn: Arc<Mutex<Connection>>,
    config: SearchConfig,
}

impl AuditSearch {
    /// Create a new search executor.
    pub fn new(conn: Arc<Mutex<Connection>>, config: SearchConfig) -> Self {
        Self { conn, config }
    }

    /// Execute a search query.
    pub fn search(&self, query: &str) -> Result<SearchResults, SearchError> {
        let parsed = SearchQueryParser::parse(query);
        let conn = self.conn.lock();

        let sql = format!(
            r#"
            SELECT
                event_id,
                e.timestamp,
                e.category,
                e.action,
                e.severity,
                snippet(audit_fts, -1, '<mark>', '</mark>', '...', {}) as snippet,
                bm25(audit_fts) as rank
            FROM audit_fts
            JOIN audit_events e ON audit_fts.event_id = e.id
            WHERE audit_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
            self.config.snippet_length
        );

        let mut stmt = conn.prepare(&sql)?;
        let results: Vec<SearchResult> = stmt
            .query_map(
                rusqlite::params![parsed.fts_query, self.config.max_results],
                |row| {
                    Ok(SearchResult {
                        event_id: row.get(0)?,
                        timestamp: row.get(1)?,
                        category: row.get(2)?,
                        action: row.get(3)?,
                        severity: row.get(4)?,
                        snippet: row.get(5).ok(),
                        rank: row.get(6)?,
                    })
                },
            )?
            .filter_map(|r| r.ok())
            .collect();

        let total_matches = self.count_matches(&conn, &parsed.fts_query)?;
        let suggestions = self.get_suggestions(&conn, &parsed)?;

        Ok(SearchResults {
            query: query.to_string(),
            total_matches,
            results,
            suggestions,
        })
    }

    fn count_matches(&self, conn: &Connection, fts_query: &str) -> Result<u64, SearchError> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_fts WHERE audit_fts MATCH ?",
            [fts_query],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    fn get_suggestions(&self, conn: &Connection, parsed: &ParsedQuery) -> Result<Vec<String>, SearchError> {
        // Get common terms that match prefix
        let mut suggestions = Vec::new();

        for term in &parsed.highlight_terms {
            if term.len() >= 2 {
                let like_pattern = format!("{}%", term);
                let mut stmt = conn.prepare(
                    "SELECT DISTINCT category FROM audit_events WHERE category LIKE ? LIMIT 5"
                )?;
                let cats: Vec<String> = stmt
                    .query_map([&like_pattern], |row| row.get(0))?
                    .filter_map(|r| r.ok())
                    .collect();
                suggestions.extend(cats);
            }
        }

        suggestions.truncate(10);
        Ok(suggestions)
    }

    /// Rebuild the search index.
    pub fn rebuild_index(&self) -> Result<(), SearchError> {
        let conn = self.conn.lock();
        conn.execute_batch(super::search_schema::REBUILD_FTS)?;
        Ok(())
    }

    /// Optimize the search index.
    pub fn optimize_index(&self) -> Result<(), SearchError> {
        let conn = self.conn.lock();
        conn.execute_batch(super::search_schema::OPTIMIZE_FTS)?;
        Ok(())
    }
}

/// Search error.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("invalid query: {0}")]
    InvalidQuery(String),
}
```

### 4. Search Highlighting (src/highlight.rs)

```rust
//! Search result highlighting.

use regex::Regex;

/// Highlight search terms in text.
pub struct Highlighter {
    terms: Vec<String>,
    open_tag: String,
    close_tag: String,
}

impl Highlighter {
    /// Create a new highlighter.
    pub fn new(terms: Vec<String>) -> Self {
        Self {
            terms,
            open_tag: "<mark>".to_string(),
            close_tag: "</mark>".to_string(),
        }
    }

    /// Set custom highlight tags.
    pub fn with_tags(mut self, open: impl Into<String>, close: impl Into<String>) -> Self {
        self.open_tag = open.into();
        self.close_tag = close.into();
        self
    }

    /// Highlight terms in text.
    pub fn highlight(&self, text: &str) -> String {
        let mut result = text.to_string();

        for term in &self.terms {
            if term.is_empty() {
                continue;
            }

            // Case-insensitive replacement
            let pattern = format!(r"(?i)({})", regex::escape(term));
            if let Ok(re) = Regex::new(&pattern) {
                result = re.replace_all(&result, |caps: &regex::Captures| {
                    format!("{}{}{}", self.open_tag, &caps[1], self.close_tag)
                }).to_string();
            }
        }

        result
    }

    /// Generate a snippet around highlighted terms.
    pub fn snippet(&self, text: &str, max_length: usize) -> String {
        let highlighted = self.highlight(text);

        if highlighted.len() <= max_length {
            return highlighted;
        }

        // Find first highlight and center around it
        if let Some(pos) = highlighted.find(&self.open_tag) {
            let start = pos.saturating_sub(max_length / 3);
            let end = (pos + max_length * 2 / 3).min(highlighted.len());

            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&highlighted[start..end]);
            if end < highlighted.len() {
                snippet.push_str("...");
            }
            snippet
        } else {
            // No highlight found, truncate
            let truncated = &highlighted[..max_length.min(highlighted.len())];
            if highlighted.len() > max_length {
                format!("{}...", truncated)
            } else {
                truncated.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_single_term() {
        let h = Highlighter::new(vec!["error".to_string()]);
        let result = h.highlight("An error occurred");
        assert_eq!(result, "An <mark>error</mark> occurred");
    }

    #[test]
    fn test_highlight_case_insensitive() {
        let h = Highlighter::new(vec!["error".to_string()]);
        let result = h.highlight("An ERROR occurred");
        assert_eq!(result, "An <mark>ERROR</mark> occurred");
    }
}
```

---

## Testing Requirements

1. FTS5 schema creates valid tables
2. Query parsing handles all syntax
3. Search returns relevant results
4. Highlighting is accurate
5. Suggestions are helpful

---

## Related Specs

- Depends on: [435-audit-query.md](435-audit-query.md)
- Next: [439-audit-filtering.md](439-audit-filtering.md)
