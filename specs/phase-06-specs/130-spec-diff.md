# Spec 130: Spec Diff Generation

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 130
- **Status**: Planned
- **Dependencies**: 129-spec-versioning, 120-spec-parsing
- **Estimated Context**: ~9%

## Objective

Implement diff generation for specifications that produces meaningful, structured differences between spec versions. The diff system understands spec structure (sections, checkboxes, code blocks) to provide semantic diffs rather than just line-by-line text changes.

## Acceptance Criteria

- [ ] Line-level diffs are generated correctly
- [ ] Section-level diffs show structural changes
- [ ] Checkbox state changes are tracked
- [ ] Code block changes are highlighted
- [ ] Metadata changes are identified
- [ ] Diff output supports multiple formats (text, HTML, JSON)
- [ ] Three-way merge support for conflicts
- [ ] Diff statistics are calculated

## Implementation Details

### Diff Generation System

```rust
// src/spec/diff.rs

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::spec::parsing::{ParsedSpec, Checkbox, CodeBlock};

/// A complete diff between two spec versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDiff {
    /// Old spec ID (for reference)
    pub old_version: String,
    /// New spec ID
    pub new_version: String,
    /// Title change
    pub title_change: Option<StringChange>,
    /// Metadata changes
    pub metadata_changes: Vec<MetadataChange>,
    /// Section changes
    pub section_changes: Vec<SectionChange>,
    /// Checkbox changes
    pub checkbox_changes: Vec<CheckboxChange>,
    /// Code block changes
    pub code_block_changes: Vec<CodeBlockChange>,
    /// Statistics
    pub stats: DiffStats,
}

/// Change to a string value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringChange {
    pub old: String,
    pub new: String,
}

/// Change to metadata field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChange {
    pub field: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// Change to a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionChange {
    pub section_name: String,
    pub change_type: ChangeType,
    /// Line-by-line diff for modified sections
    pub line_diff: Option<LineDiff>,
}

/// Change to checkbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckboxChange {
    pub section: String,
    pub text: String,
    pub change_type: CheckboxChangeType,
    pub old_state: Option<bool>,
    pub new_state: Option<bool>,
}

/// Checkbox change types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckboxChangeType {
    Added,
    Removed,
    StateChanged,
    TextModified,
}

/// Change to code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockChange {
    pub section: String,
    pub language: Option<String>,
    pub change_type: ChangeType,
    pub line_diff: Option<LineDiff>,
}

/// Change type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Line-by-line diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineDiff {
    pub hunks: Vec<DiffHunk>,
}

/// A hunk in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

/// Line type in diff
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineType {
    Context,
    Addition,
    Deletion,
}

/// Diff statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub sections_added: usize,
    pub sections_removed: usize,
    pub sections_modified: usize,
    pub checkboxes_added: usize,
    pub checkboxes_removed: usize,
    pub checkboxes_toggled: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub code_blocks_changed: usize,
}

/// Spec diff generator
pub struct SpecDiffGenerator {
    context_lines: usize,
}

impl SpecDiffGenerator {
    pub fn new() -> Self {
        Self { context_lines: 3 }
    }

    pub fn with_context(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Generate diff between two parsed specs
    pub fn diff(&self, old: &ParsedSpec, new: &ParsedSpec) -> SpecDiff {
        let mut stats = DiffStats::default();

        // Compare title
        let title_change = if old.title != new.title {
            Some(StringChange {
                old: old.title.clone(),
                new: new.title.clone(),
            })
        } else {
            None
        };

        // Compare metadata
        let metadata_changes = self.diff_metadata(old, new);

        // Compare sections
        let section_changes = self.diff_sections(old, new, &mut stats);

        // Compare checkboxes
        let checkbox_changes = self.diff_checkboxes(old, new, &mut stats);

        // Compare code blocks
        let code_block_changes = self.diff_code_blocks(old, new, &mut stats);

        SpecDiff {
            old_version: format!("spec:{}", old.metadata.spec_id),
            new_version: format!("spec:{}", new.metadata.spec_id),
            title_change,
            metadata_changes,
            section_changes,
            checkbox_changes,
            code_block_changes,
            stats,
        }
    }

    /// Diff metadata fields
    fn diff_metadata(&self, old: &ParsedSpec, new: &ParsedSpec) -> Vec<MetadataChange> {
        let mut changes = Vec::new();

        // Check standard fields
        if old.metadata.phase != new.metadata.phase {
            changes.push(MetadataChange {
                field: "Phase".to_string(),
                change_type: ChangeType::Modified,
                old_value: Some(old.metadata.phase.to_string()),
                new_value: Some(new.metadata.phase.to_string()),
            });
        }

        if old.metadata.status != new.metadata.status {
            changes.push(MetadataChange {
                field: "Status".to_string(),
                change_type: ChangeType::Modified,
                old_value: Some(old.metadata.status.clone()),
                new_value: Some(new.metadata.status.clone()),
            });
        }

        // Check dependencies
        let old_deps: HashSet<_> = old.metadata.dependencies.iter().collect();
        let new_deps: HashSet<_> = new.metadata.dependencies.iter().collect();

        for dep in old_deps.difference(&new_deps) {
            changes.push(MetadataChange {
                field: "Dependencies".to_string(),
                change_type: ChangeType::Removed,
                old_value: Some((*dep).clone()),
                new_value: None,
            });
        }

        for dep in new_deps.difference(&old_deps) {
            changes.push(MetadataChange {
                field: "Dependencies".to_string(),
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some((*dep).clone()),
            });
        }

        changes
    }

    /// Diff sections
    fn diff_sections(
        &self,
        old: &ParsedSpec,
        new: &ParsedSpec,
        stats: &mut DiffStats,
    ) -> Vec<SectionChange> {
        let mut changes = Vec::new();

        let old_sections: HashSet<_> = old.sections.keys().collect();
        let new_sections: HashSet<_> = new.sections.keys().collect();

        // Removed sections
        for section in old_sections.difference(&new_sections) {
            changes.push(SectionChange {
                section_name: (*section).clone(),
                change_type: ChangeType::Removed,
                line_diff: None,
            });
            stats.sections_removed += 1;
        }

        // Added sections
        for section in new_sections.difference(&old_sections) {
            changes.push(SectionChange {
                section_name: (*section).clone(),
                change_type: ChangeType::Added,
                line_diff: None,
            });
            stats.sections_added += 1;
        }

        // Modified sections
        for section in old_sections.intersection(&new_sections) {
            let old_content = old.sections.get(*section).unwrap();
            let new_content = new.sections.get(*section).unwrap();

            if old_content != new_content {
                let line_diff = self.generate_line_diff(old_content, new_content, stats);
                changes.push(SectionChange {
                    section_name: (*section).clone(),
                    change_type: ChangeType::Modified,
                    line_diff: Some(line_diff),
                });
                stats.sections_modified += 1;
            }
        }

        changes
    }

    /// Generate line-by-line diff
    fn generate_line_diff(&self, old: &str, new: &str, stats: &mut DiffStats) -> LineDiff {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        // Use Myers diff algorithm (simplified version)
        let edit_script = self.compute_edit_script(&old_lines, &new_lines);

        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_idx = 0;
        let mut new_idx = 0;

        for edit in edit_script {
            match edit {
                Edit::Keep => {
                    // Add context or close hunk
                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(DiffLine {
                            line_type: LineType::Context,
                            content: old_lines.get(old_idx).unwrap_or(&"").to_string(),
                            old_line_num: Some(old_idx),
                            new_line_num: Some(new_idx),
                        });
                    }
                    old_idx += 1;
                    new_idx += 1;
                }
                Edit::Insert => {
                    // Start hunk if needed
                    if current_hunk.is_none() {
                        current_hunk = Some(self.start_hunk(old_idx, new_idx, &old_lines));
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(DiffLine {
                            line_type: LineType::Addition,
                            content: new_lines.get(new_idx).unwrap_or(&"").to_string(),
                            old_line_num: None,
                            new_line_num: Some(new_idx),
                        });
                        hunk.new_count += 1;
                    }
                    stats.lines_added += 1;
                    new_idx += 1;
                }
                Edit::Delete => {
                    if current_hunk.is_none() {
                        current_hunk = Some(self.start_hunk(old_idx, new_idx, &old_lines));
                    }

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(DiffLine {
                            line_type: LineType::Deletion,
                            content: old_lines.get(old_idx).unwrap_or(&"").to_string(),
                            old_line_num: Some(old_idx),
                            new_line_num: None,
                        });
                        hunk.old_count += 1;
                    }
                    stats.lines_removed += 1;
                    old_idx += 1;
                }
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        LineDiff { hunks }
    }

    /// Start a new diff hunk with context
    fn start_hunk(&self, old_idx: usize, new_idx: usize, old_lines: &[&str]) -> DiffHunk {
        let context_start = old_idx.saturating_sub(self.context_lines);

        let mut hunk = DiffHunk {
            old_start: context_start,
            old_count: 0,
            new_start: new_idx.saturating_sub(self.context_lines),
            new_count: 0,
            lines: Vec::new(),
        };

        // Add leading context
        for i in context_start..old_idx {
            if let Some(line) = old_lines.get(i) {
                hunk.lines.push(DiffLine {
                    line_type: LineType::Context,
                    content: line.to_string(),
                    old_line_num: Some(i),
                    new_line_num: Some(new_idx.saturating_sub(old_idx - i)),
                });
            }
        }

        hunk
    }

    /// Simplified edit script computation
    fn compute_edit_script<'a>(&self, old: &[&'a str], new: &[&'a str]) -> Vec<Edit> {
        let mut edits = Vec::new();
        let mut old_idx = 0;
        let mut new_idx = 0;

        while old_idx < old.len() || new_idx < new.len() {
            if old_idx >= old.len() {
                edits.push(Edit::Insert);
                new_idx += 1;
            } else if new_idx >= new.len() {
                edits.push(Edit::Delete);
                old_idx += 1;
            } else if old[old_idx] == new[new_idx] {
                edits.push(Edit::Keep);
                old_idx += 1;
                new_idx += 1;
            } else {
                // Simple heuristic: look ahead for matches
                let old_ahead = new[new_idx..].iter().position(|&l| l == old[old_idx]);
                let new_ahead = old[old_idx..].iter().position(|&l| l == new[new_idx]);

                match (old_ahead, new_ahead) {
                    (Some(o), Some(n)) if o <= n => {
                        for _ in 0..o {
                            edits.push(Edit::Insert);
                            new_idx += 1;
                        }
                    }
                    (Some(_), Some(_)) | (None, Some(_)) => {
                        edits.push(Edit::Delete);
                        old_idx += 1;
                    }
                    (Some(o), None) => {
                        for _ in 0..o {
                            edits.push(Edit::Insert);
                            new_idx += 1;
                        }
                    }
                    (None, None) => {
                        edits.push(Edit::Delete);
                        edits.push(Edit::Insert);
                        old_idx += 1;
                        new_idx += 1;
                    }
                }
            }
        }

        edits
    }

    /// Diff checkboxes
    fn diff_checkboxes(
        &self,
        old: &ParsedSpec,
        new: &ParsedSpec,
        stats: &mut DiffStats,
    ) -> Vec<CheckboxChange> {
        let mut changes = Vec::new();

        // Build maps by text for comparison
        let old_map: HashMap<_, _> = old.acceptance_criteria.iter()
            .map(|c| (&c.text, c))
            .collect();
        let new_map: HashMap<_, _> = new.acceptance_criteria.iter()
            .map(|c| (&c.text, c))
            .collect();

        // Find removed
        for (text, old_cb) in &old_map {
            if !new_map.contains_key(text) {
                changes.push(CheckboxChange {
                    section: old_cb.section.clone(),
                    text: (*text).clone(),
                    change_type: CheckboxChangeType::Removed,
                    old_state: Some(old_cb.checked),
                    new_state: None,
                });
                stats.checkboxes_removed += 1;
            }
        }

        // Find added and changed
        for (text, new_cb) in &new_map {
            if let Some(old_cb) = old_map.get(text) {
                if old_cb.checked != new_cb.checked {
                    changes.push(CheckboxChange {
                        section: new_cb.section.clone(),
                        text: (*text).clone(),
                        change_type: CheckboxChangeType::StateChanged,
                        old_state: Some(old_cb.checked),
                        new_state: Some(new_cb.checked),
                    });
                    stats.checkboxes_toggled += 1;
                }
            } else {
                changes.push(CheckboxChange {
                    section: new_cb.section.clone(),
                    text: (*text).clone(),
                    change_type: CheckboxChangeType::Added,
                    old_state: None,
                    new_state: Some(new_cb.checked),
                });
                stats.checkboxes_added += 1;
            }
        }

        changes
    }

    /// Diff code blocks
    fn diff_code_blocks(
        &self,
        old: &ParsedSpec,
        new: &ParsedSpec,
        stats: &mut DiffStats,
    ) -> Vec<CodeBlockChange> {
        let mut changes = Vec::new();

        // Simple comparison by section and language
        let old_blocks: HashMap<_, _> = old.code_blocks.iter()
            .map(|b| ((&b.section, &b.language), b))
            .collect();

        for new_block in &new.code_blocks {
            let key = (&new_block.section, &new_block.language);

            if let Some(old_block) = old_blocks.get(&key) {
                if old_block.content != new_block.content {
                    let line_diff = self.generate_line_diff(
                        &old_block.content,
                        &new_block.content,
                        stats,
                    );
                    changes.push(CodeBlockChange {
                        section: new_block.section.clone(),
                        language: new_block.language.clone(),
                        change_type: ChangeType::Modified,
                        line_diff: Some(line_diff),
                    });
                    stats.code_blocks_changed += 1;
                }
            } else {
                changes.push(CodeBlockChange {
                    section: new_block.section.clone(),
                    language: new_block.language.clone(),
                    change_type: ChangeType::Added,
                    line_diff: None,
                });
                stats.code_blocks_changed += 1;
            }
        }

        changes
    }
}

/// Edit operation
#[derive(Debug, Clone, Copy)]
enum Edit {
    Keep,
    Insert,
    Delete,
}

/// Diff renderer
pub struct DiffRenderer;

impl DiffRenderer {
    /// Render diff as unified diff text
    pub fn to_unified(diff: &SpecDiff) -> String {
        let mut output = String::new();

        output.push_str(&format!("--- {}\n", diff.old_version));
        output.push_str(&format!("+++ {}\n", diff.new_version));

        if let Some(ref title) = diff.title_change {
            output.push_str(&format!("-# {}\n", title.old));
            output.push_str(&format!("+# {}\n", title.new));
        }

        for section in &diff.section_changes {
            output.push_str(&format!("\n## Section: {}\n", section.section_name));

            match section.change_type {
                ChangeType::Added => output.push_str("+ [Section Added]\n"),
                ChangeType::Removed => output.push_str("- [Section Removed]\n"),
                ChangeType::Modified => {
                    if let Some(ref line_diff) = section.line_diff {
                        for hunk in &line_diff.hunks {
                            output.push_str(&format!(
                                "@@ -{},{} +{},{} @@\n",
                                hunk.old_start, hunk.old_count,
                                hunk.new_start, hunk.new_count
                            ));
                            for line in &hunk.lines {
                                let prefix = match line.line_type {
                                    LineType::Context => ' ',
                                    LineType::Addition => '+',
                                    LineType::Deletion => '-',
                                };
                                output.push_str(&format!("{}{}\n", prefix, line.content));
                            }
                        }
                    }
                }
                ChangeType::Unchanged => {}
            }
        }

        output
    }

    /// Render diff as HTML
    pub fn to_html(diff: &SpecDiff) -> String {
        let mut output = String::new();

        output.push_str("<div class=\"spec-diff\">\n");
        output.push_str(&format!(
            "<div class=\"diff-header\">{} → {}</div>\n",
            diff.old_version, diff.new_version
        ));

        // Stats
        output.push_str("<div class=\"diff-stats\">\n");
        output.push_str(&format!(
            "<span class=\"added\">+{}</span> / <span class=\"removed\">-{}</span>\n",
            diff.stats.lines_added, diff.stats.lines_removed
        ));
        output.push_str("</div>\n");

        // Section changes
        for section in &diff.section_changes {
            let class = match section.change_type {
                ChangeType::Added => "added",
                ChangeType::Removed => "removed",
                ChangeType::Modified => "modified",
                ChangeType::Unchanged => "unchanged",
            };

            output.push_str(&format!(
                "<div class=\"section-change {}\">\n",
                class
            ));
            output.push_str(&format!("<h3>{}</h3>\n", section.section_name));

            if let Some(ref line_diff) = section.line_diff {
                output.push_str("<pre class=\"diff-content\">\n");
                for hunk in &line_diff.hunks {
                    for line in &hunk.lines {
                        let (class, prefix) = match line.line_type {
                            LineType::Context => ("context", " "),
                            LineType::Addition => ("addition", "+"),
                            LineType::Deletion => ("deletion", "-"),
                        };
                        output.push_str(&format!(
                            "<span class=\"{}\">{}{}</span>\n",
                            class, prefix,
                            html_escape(&line.content)
                        ));
                    }
                }
                output.push_str("</pre>\n");
            }

            output.push_str("</div>\n");
        }

        output.push_str("</div>\n");
        output
    }
}

/// Simple HTML escape
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl Default for SpecDiffGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_diff() {
        let generator = SpecDiffGenerator::new();

        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nmodified line 2\nline 3\nline 4";

        let mut stats = DiffStats::default();
        let diff = generator.generate_line_diff(old, new, &mut stats);

        assert!(!diff.hunks.is_empty());
        assert!(stats.lines_added > 0);
    }

    #[test]
    fn test_checkbox_diff() {
        let generator = SpecDiffGenerator::new();

        let mut old_spec = ParsedSpec::default();
        old_spec.acceptance_criteria.push(Checkbox {
            text: "Item 1".to_string(),
            checked: false,
            line: 0,
            section: "Test".to_string(),
        });

        let mut new_spec = ParsedSpec::default();
        new_spec.acceptance_criteria.push(Checkbox {
            text: "Item 1".to_string(),
            checked: true,
            line: 0,
            section: "Test".to_string(),
        });

        let diff = generator.diff(&old_spec, &new_spec);

        assert!(!diff.checkbox_changes.is_empty());
        assert_eq!(diff.checkbox_changes[0].change_type, CheckboxChangeType::StateChanged);
    }
}
```

## Testing Requirements

- [ ] Unit tests for line diff generation
- [ ] Tests for section diff comparison
- [ ] Tests for checkbox change detection
- [ ] Tests for code block diff
- [ ] Tests for unified diff output
- [ ] Tests for HTML diff output
- [ ] Tests for edge cases (empty specs, etc.)
- [ ] Performance tests for large diffs

## Related Specs

- **129-spec-versioning.md**: Version management
- **120-spec-parsing.md**: Parses specs for comparison
- **119-readme-autogen.md**: Changelog generation
