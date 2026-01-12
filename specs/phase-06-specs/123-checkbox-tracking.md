# Spec 123: Checkbox State Tracking

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 123
- **Status**: Planned
- **Dependencies**: 120-spec-parsing, 121-spec-metadata
- **Estimated Context**: ~8%

## Objective

Implement comprehensive checkbox state tracking for spec documents. Checkboxes represent actionable items (acceptance criteria, tasks, tests) whose completion state must be accurately tracked, persisted, and synchronized with the source documents.

## Acceptance Criteria

- [ ] Checkbox state is accurately parsed
- [ ] State changes are persisted to files
- [ ] Batch updates are atomic
- [ ] Change history is tracked
- [ ] State synchronization handles conflicts
- [ ] Checkbox IDs are stable across edits
- [ ] Real-time state subscriptions work
- [ ] Undo/redo is supported

## Implementation Details

### Checkbox Tracking System

```rust
// src/spec/checkbox.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A tracked checkbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedCheckbox {
    /// Unique ID (spec_id:section:index)
    pub id: CheckboxId,
    /// Checkbox text
    pub text: String,
    /// Current state
    pub checked: bool,
    /// Line number in source
    pub line: usize,
    /// Section name
    pub section: String,
    /// Parent checkbox (for nested items)
    pub parent: Option<CheckboxId>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Modification source
    pub modified_by: ModificationSource,
}

/// Unique checkbox identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckboxId {
    pub spec_id: u32,
    pub section: String,
    pub index: u32,
}

impl CheckboxId {
    pub fn new(spec_id: u32, section: &str, index: u32) -> Self {
        Self {
            spec_id,
            section: section.to_string(),
            index,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.spec_id, self.section, self.index)
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        if parts.len() == 3 {
            Some(Self {
                spec_id: parts[0].parse().ok()?,
                section: parts[1].to_string(),
                index: parts[2].parse().ok()?,
            })
        } else {
            None
        }
    }
}

/// Source of modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModificationSource {
    User,
    Ai { model: String },
    Automated { process: String },
    Sync,
}

/// Checkbox state change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxChange {
    pub id: CheckboxId,
    pub old_state: bool,
    pub new_state: bool,
    pub timestamp: DateTime<Utc>,
    pub source: ModificationSource,
}

/// Checkbox state tracker
pub struct CheckboxTracker {
    /// Checkboxes by ID
    checkboxes: HashMap<CheckboxId, TrackedCheckbox>,
    /// File paths for specs
    spec_paths: HashMap<u32, PathBuf>,
    /// Change history
    history: Vec<CheckboxChange>,
    /// Change broadcaster
    change_tx: broadcast::Sender<CheckboxChange>,
    /// Undo stack
    undo_stack: Vec<CheckboxChange>,
    /// Redo stack
    redo_stack: Vec<CheckboxChange>,
}

impl CheckboxTracker {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            checkboxes: HashMap::new(),
            spec_paths: HashMap::new(),
            history: Vec::new(),
            change_tx: tx,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Load checkboxes from a spec file
    pub async fn load_spec(&mut self, spec_id: u32, path: &Path) -> Result<(), CheckboxError> {
        let content = fs::read_to_string(path).await?;
        let checkboxes = self.parse_checkboxes(&content, spec_id);

        for checkbox in checkboxes {
            self.checkboxes.insert(checkbox.id.clone(), checkbox);
        }

        self.spec_paths.insert(spec_id, path.to_path_buf());
        Ok(())
    }

    /// Parse checkboxes from content
    fn parse_checkboxes(&self, content: &str, spec_id: u32) -> Vec<TrackedCheckbox> {
        let mut checkboxes = Vec::new();
        let mut current_section = String::new();
        let mut section_indices: HashMap<String, u32> = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            // Track section headers
            if line.starts_with("## ") {
                current_section = line[3..].trim().to_string();
            }

            // Parse checkboxes
            if let Some(checkbox) = self.parse_checkbox_line(line, line_num, spec_id, &current_section, &mut section_indices) {
                checkboxes.push(checkbox);
            }
        }

        checkboxes
    }

    /// Parse a single checkbox line
    fn parse_checkbox_line(
        &self,
        line: &str,
        line_num: usize,
        spec_id: u32,
        section: &str,
        indices: &mut HashMap<String, u32>,
    ) -> Option<TrackedCheckbox> {
        let trimmed = line.trim();

        // Match checkbox patterns
        let (checked, text) = if trimmed.starts_with("- [ ]") {
            (false, trimmed[5..].trim())
        } else if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            (true, trimmed[5..].trim())
        } else if trimmed.starts_with("* [ ]") {
            (false, trimmed[5..].trim())
        } else if trimmed.starts_with("* [x]") || trimmed.starts_with("* [X]") {
            (true, trimmed[5..].trim())
        } else {
            return None;
        };

        let index = indices.entry(section.to_string()).or_insert(0);
        *index += 1;

        Some(TrackedCheckbox {
            id: CheckboxId::new(spec_id, section, *index),
            text: text.to_string(),
            checked,
            line: line_num,
            section: section.to_string(),
            parent: None,
            modified_at: Utc::now(),
            modified_by: ModificationSource::Sync,
        })
    }

    /// Set checkbox state
    pub async fn set_checked(
        &mut self,
        id: &CheckboxId,
        checked: bool,
        source: ModificationSource,
    ) -> Result<(), CheckboxError> {
        let checkbox = self.checkboxes.get_mut(id)
            .ok_or_else(|| CheckboxError::NotFound(id.to_string()))?;

        if checkbox.checked == checked {
            return Ok(()); // No change
        }

        let change = CheckboxChange {
            id: id.clone(),
            old_state: checkbox.checked,
            new_state: checked,
            timestamp: Utc::now(),
            source: source.clone(),
        };

        // Update state
        checkbox.checked = checked;
        checkbox.modified_at = change.timestamp;
        checkbox.modified_by = source;

        // Record change
        self.history.push(change.clone());
        self.undo_stack.push(change.clone());
        self.redo_stack.clear();

        // Broadcast change
        let _ = self.change_tx.send(change);

        // Persist to file
        self.persist_spec(id.spec_id).await?;

        Ok(())
    }

    /// Toggle checkbox state
    pub async fn toggle(
        &mut self,
        id: &CheckboxId,
        source: ModificationSource,
    ) -> Result<bool, CheckboxError> {
        let current = self.checkboxes.get(id)
            .ok_or_else(|| CheckboxError::NotFound(id.to_string()))?
            .checked;

        self.set_checked(id, !current, source).await?;
        Ok(!current)
    }

    /// Batch update multiple checkboxes
    pub async fn batch_update(
        &mut self,
        updates: Vec<(CheckboxId, bool)>,
        source: ModificationSource,
    ) -> Result<(), CheckboxError> {
        let mut affected_specs = std::collections::HashSet::new();

        for (id, checked) in updates {
            if let Some(checkbox) = self.checkboxes.get_mut(&id) {
                if checkbox.checked != checked {
                    let change = CheckboxChange {
                        id: id.clone(),
                        old_state: checkbox.checked,
                        new_state: checked,
                        timestamp: Utc::now(),
                        source: source.clone(),
                    };

                    checkbox.checked = checked;
                    checkbox.modified_at = change.timestamp;
                    checkbox.modified_by = source.clone();

                    self.history.push(change.clone());
                    self.undo_stack.push(change);

                    affected_specs.insert(id.spec_id);
                }
            }
        }

        self.redo_stack.clear();

        // Persist all affected specs
        for spec_id in affected_specs {
            self.persist_spec(spec_id).await?;
        }

        Ok(())
    }

    /// Undo last change
    pub async fn undo(&mut self) -> Result<Option<CheckboxChange>, CheckboxError> {
        let change = match self.undo_stack.pop() {
            Some(c) => c,
            None => return Ok(None),
        };

        // Revert the change
        if let Some(checkbox) = self.checkboxes.get_mut(&change.id) {
            checkbox.checked = change.old_state;
            checkbox.modified_at = Utc::now();
            checkbox.modified_by = ModificationSource::User;
        }

        self.redo_stack.push(change.clone());
        self.persist_spec(change.id.spec_id).await?;

        Ok(Some(change))
    }

    /// Redo last undone change
    pub async fn redo(&mut self) -> Result<Option<CheckboxChange>, CheckboxError> {
        let change = match self.redo_stack.pop() {
            Some(c) => c,
            None => return Ok(None),
        };

        // Reapply the change
        if let Some(checkbox) = self.checkboxes.get_mut(&change.id) {
            checkbox.checked = change.new_state;
            checkbox.modified_at = Utc::now();
            checkbox.modified_by = ModificationSource::User;
        }

        self.undo_stack.push(change.clone());
        self.persist_spec(change.id.spec_id).await?;

        Ok(Some(change))
    }

    /// Persist spec changes to file
    async fn persist_spec(&self, spec_id: u32) -> Result<(), CheckboxError> {
        let path = self.spec_paths.get(&spec_id)
            .ok_or_else(|| CheckboxError::SpecNotLoaded(spec_id))?;

        let content = fs::read_to_string(path).await?;
        let updated = self.update_content(&content, spec_id);

        fs::write(path, updated).await?;
        Ok(())
    }

    /// Update content with current checkbox states
    fn update_content(&self, content: &str, spec_id: u32) -> String {
        let mut result = String::new();
        let mut current_section = String::new();
        let mut section_indices: HashMap<String, u32> = HashMap::new();

        for line in content.lines() {
            // Track sections
            if line.starts_with("## ") {
                current_section = line[3..].trim().to_string();
            }

            // Check if this is a checkbox line
            let trimmed = line.trim();
            if trimmed.starts_with("- [") || trimmed.starts_with("* [") {
                let index = section_indices.entry(current_section.clone()).or_insert(0);
                *index += 1;

                let id = CheckboxId::new(spec_id, &current_section, *index);

                if let Some(checkbox) = self.checkboxes.get(&id) {
                    // Replace checkbox state
                    let check_mark = if checkbox.checked { "x" } else { " " };
                    let prefix = if trimmed.starts_with("- [") { "- [" } else { "* [" };

                    // Find indentation
                    let indent = line.len() - line.trim_start().len();
                    let indent_str = &line[..indent];

                    // Extract text after checkbox
                    let text_start = trimmed.find(']').map(|i| i + 1).unwrap_or(5);
                    let text = trimmed[text_start..].trim();

                    result.push_str(&format!("{}{}{}] {}\n",
                        indent_str, prefix, check_mark, text
                    ));
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        // Remove trailing newline if original didn't have one
        if !content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Get checkbox by ID
    pub fn get(&self, id: &CheckboxId) -> Option<&TrackedCheckbox> {
        self.checkboxes.get(id)
    }

    /// Get all checkboxes for a spec
    pub fn get_spec_checkboxes(&self, spec_id: u32) -> Vec<&TrackedCheckbox> {
        self.checkboxes.values()
            .filter(|c| c.id.spec_id == spec_id)
            .collect()
    }

    /// Get all checkboxes for a section
    pub fn get_section_checkboxes(&self, spec_id: u32, section: &str) -> Vec<&TrackedCheckbox> {
        self.checkboxes.values()
            .filter(|c| c.id.spec_id == spec_id && c.section == section)
            .collect()
    }

    /// Subscribe to checkbox changes
    pub fn subscribe(&self) -> broadcast::Receiver<CheckboxChange> {
        self.change_tx.subscribe()
    }

    /// Get change history
    pub fn get_history(&self, limit: usize) -> &[CheckboxChange] {
        let start = self.history.len().saturating_sub(limit);
        &self.history[start..]
    }

    /// Calculate completion stats
    pub fn get_stats(&self, spec_id: u32) -> CheckboxStats {
        let checkboxes: Vec<_> = self.get_spec_checkboxes(spec_id);

        let total = checkboxes.len();
        let checked = checkboxes.iter().filter(|c| c.checked).count();

        let mut by_section: HashMap<String, (usize, usize)> = HashMap::new();
        for cb in checkboxes {
            let entry = by_section.entry(cb.section.clone()).or_insert((0, 0));
            entry.0 += 1;
            if cb.checked {
                entry.1 += 1;
            }
        }

        CheckboxStats {
            total,
            checked,
            percentage: if total > 0 { (checked * 100 / total) as u8 } else { 0 },
            by_section,
        }
    }
}

/// Checkbox statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxStats {
    pub total: usize,
    pub checked: usize,
    pub percentage: u8,
    pub by_section: HashMap<String, (usize, usize)>,
}

/// Checkbox tracking errors
#[derive(Debug, thiserror::Error)]
pub enum CheckboxError {
    #[error("Checkbox not found: {0}")]
    NotFound(String),

    #[error("Spec not loaded: {0}")]
    SpecNotLoaded(u32),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Conflict: {0}")]
    Conflict(String),
}

impl Default for CheckboxTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_id_parsing() {
        let id = CheckboxId::new(116, "Acceptance Criteria", 1);
        let s = id.to_string();

        let parsed = CheckboxId::parse(&s).unwrap();
        assert_eq!(parsed, id);
    }

    #[test]
    fn test_checkbox_parsing() {
        let tracker = CheckboxTracker::new();
        let content = r#"## Acceptance Criteria

- [ ] First item
- [x] Second item
- [ ] Third item
"#;

        let checkboxes = tracker.parse_checkboxes(content, 116);
        assert_eq!(checkboxes.len(), 3);
        assert!(!checkboxes[0].checked);
        assert!(checkboxes[1].checked);
        assert!(!checkboxes[2].checked);
    }

    #[test]
    fn test_content_update() {
        let mut tracker = CheckboxTracker::new();
        let content = "## Test\n\n- [ ] Unchecked\n";

        let checkboxes = tracker.parse_checkboxes(content, 1);
        for cb in checkboxes {
            tracker.checkboxes.insert(cb.id.clone(), cb);
        }

        // Update first checkbox
        let id = CheckboxId::new(1, "Test", 1);
        if let Some(cb) = tracker.checkboxes.get_mut(&id) {
            cb.checked = true;
        }

        let updated = tracker.update_content(content, 1);
        assert!(updated.contains("- [x]"));
    }
}
```

## Testing Requirements

- [ ] Unit tests for checkbox parsing
- [ ] Tests for state persistence
- [ ] Tests for batch updates
- [ ] Tests for undo/redo
- [ ] Tests for change subscriptions
- [ ] Tests for conflict handling
- [ ] Integration tests with file system
- [ ] Tests for concurrent modifications

## Related Specs

- **120-spec-parsing.md**: Parsing checkboxes from specs
- **124-progress-calc.md**: Progress from checkbox states
- **122-impl-plan-format.md**: Task checkboxes in plans
