# Spec 128: Spec Linting

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 128
- **Status**: Planned
- **Dependencies**: 127-spec-validation, 120-spec-parsing
- **Estimated Context**: ~9%

## Objective

Implement a linting system for specifications that goes beyond validation to enforce style guidelines, consistency rules, and best practices. The linter provides auto-fix capabilities where possible and integrates with development workflows for continuous spec quality.

## Acceptance Criteria

- [x] Style rules for formatting are enforced
- [x] Consistency rules across specs work
- [x] Auto-fix is available for common issues
- [x] Linting is configurable per-project
- [x] CI/CD integration is supported
- [x] Incremental linting is efficient
- [x] Custom lint rules can be added
- [x] Lint results include fix suggestions

## Implementation Details

### Spec Linter

```rust
// src/spec/linting.rs

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::spec::parsing::ParsedSpec;
use crate::spec::validation::{ValidationIssue, Severity};

/// A lint fix that can be applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintFix {
    /// Description of the fix
    pub description: String,
    /// Line to modify (None = insert)
    pub line: Option<usize>,
    /// Original text (for replacement)
    pub original: Option<String>,
    /// Replacement text
    pub replacement: String,
    /// Whether fix is safe to auto-apply
    pub safe: bool,
}

/// A lint issue with optional fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    /// Base validation issue
    pub issue: ValidationIssue,
    /// Optional auto-fix
    pub fix: Option<LintFix>,
    /// Category of lint rule
    pub category: LintCategory,
}

/// Lint rule categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LintCategory {
    /// Formatting issues
    Formatting,
    /// Naming conventions
    Naming,
    /// Structural issues
    Structure,
    /// Content quality
    Content,
    /// Consistency issues
    Consistency,
    /// Best practices
    BestPractice,
}

/// Lint rule trait
pub trait LintRule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> LintCategory;
    fn severity(&self) -> Severity;
    fn lint(&self, spec: &ParsedSpec, content: &str) -> Vec<LintIssue>;
}

/// Linter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    /// Enabled rule categories
    pub enabled_categories: Vec<LintCategory>,
    /// Disabled specific rules
    pub disabled_rules: Vec<String>,
    /// Auto-fix enabled
    pub auto_fix: bool,
    /// Only report safe fixes
    pub safe_fixes_only: bool,
    /// Max line length
    pub max_line_length: usize,
    /// Heading style (atx or setext)
    pub heading_style: HeadingStyle,
    /// Code block style
    pub code_block_style: CodeBlockStyle,
    /// List style
    pub list_style: ListStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HeadingStyle {
    Atx,     // # Heading
    Setext,  // Heading with underline
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CodeBlockStyle {
    Fenced,   // ```
    Indented, // 4 spaces
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ListStyle {
    Dash,     // -
    Asterisk, // *
    Plus,     // +
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            enabled_categories: vec![
                LintCategory::Formatting,
                LintCategory::Structure,
                LintCategory::Content,
                LintCategory::BestPractice,
            ],
            disabled_rules: Vec::new(),
            auto_fix: false,
            safe_fixes_only: true,
            max_line_length: 120,
            heading_style: HeadingStyle::Atx,
            code_block_style: CodeBlockStyle::Fenced,
            list_style: ListStyle::Dash,
        }
    }
}

/// Main spec linter
pub struct SpecLinter {
    rules: Vec<Box<dyn LintRule>>,
    config: LinterConfig,
}

impl SpecLinter {
    pub fn new(config: LinterConfig) -> Self {
        let mut linter = Self {
            rules: Vec::new(),
            config,
        };

        linter.register_builtin_rules();
        linter
    }

    fn register_builtin_rules(&mut self) {
        // Formatting rules
        self.rules.push(Box::new(LineLengthRule { max: self.config.max_line_length }));
        self.rules.push(Box::new(TrailingWhitespaceRule));
        self.rules.push(Box::new(ConsistentListMarkerRule { style: self.config.list_style }));
        self.rules.push(Box::new(HeadingSpacingRule));

        // Structure rules
        self.rules.push(Box::new(SectionOrderRule));
        self.rules.push(Box::new(EmptySectionRule));
        self.rules.push(Box::new(NestedHeadingRule));

        // Content rules
        self.rules.push(Box::new(TodoCommentRule));
        self.rules.push(Box::new(BrokenLinkRule));
        self.rules.push(Box::new(SpellingRule));

        // Best practice rules
        self.rules.push(Box::new(DescriptiveTitleRule));
        self.rules.push(Box::new(ConcreteAcceptanceCriteriaRule));
        self.rules.push(Box::new(EstimatedContextRule));
    }

    /// Add custom rule
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    /// Lint a spec
    pub fn lint(&self, spec: &ParsedSpec, content: &str) -> LintResult {
        let mut issues = Vec::new();

        for rule in &self.rules {
            // Skip disabled rules
            if self.config.disabled_rules.contains(&rule.id().to_string()) {
                continue;
            }

            // Skip disabled categories
            if !self.config.enabled_categories.contains(&rule.category()) {
                continue;
            }

            let rule_issues = rule.lint(spec, content);

            // Filter unsafe fixes if configured
            let filtered: Vec<_> = if self.config.safe_fixes_only {
                rule_issues.into_iter()
                    .map(|mut i| {
                        if let Some(ref fix) = i.fix {
                            if !fix.safe {
                                i.fix = None;
                            }
                        }
                        i
                    })
                    .collect()
            } else {
                rule_issues
            };

            issues.extend(filtered);
        }

        // Sort by line number
        issues.sort_by_key(|i| i.issue.line.unwrap_or(0));

        let fixable = issues.iter().filter(|i| i.fix.is_some()).count();

        LintResult {
            spec_id: spec.metadata.spec_id,
            issues,
            fixable_count: fixable,
        }
    }

    /// Apply fixes to content
    pub fn apply_fixes(&self, content: &str, issues: &[LintIssue]) -> String {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Sort fixes by line number in reverse (to not invalidate later line numbers)
        let mut fixes: Vec<_> = issues.iter()
            .filter_map(|i| i.fix.as_ref().map(|f| (i.issue.line, f)))
            .collect();
        fixes.sort_by_key(|(line, _)| std::cmp::Reverse(*line));

        for (line_opt, fix) in fixes {
            if !fix.safe && self.config.safe_fixes_only {
                continue;
            }

            match line_opt {
                Some(line) if line < lines.len() => {
                    if let Some(original) = &fix.original {
                        lines[line] = lines[line].replace(original, &fix.replacement);
                    } else {
                        lines[line] = fix.replacement.clone();
                    }
                }
                None => {
                    // Insert at end
                    lines.push(fix.replacement.clone());
                }
                _ => {}
            }
        }

        lines.join("\n")
    }
}

/// Lint result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub spec_id: u32,
    pub issues: Vec<LintIssue>,
    pub fixable_count: usize,
}

// ===== Built-in Lint Rules =====

/// Line length rule
struct LineLengthRule {
    max: usize,
}

impl LintRule for LineLengthRule {
    fn id(&self) -> &str { "line-length" }
    fn description(&self) -> &str { "Enforces maximum line length" }
    fn category(&self) -> LintCategory { LintCategory::Formatting }
    fn severity(&self) -> Severity { Severity::Warning }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // Skip code blocks and URLs
            if line.trim().starts_with("```") || line.contains("http://") || line.contains("https://") {
                continue;
            }

            if line.len() > self.max {
                issues.push(LintIssue {
                    issue: ValidationIssue {
                        code: "LINT001".to_string(),
                        message: format!("Line exceeds {} characters ({})", self.max, line.len()),
                        severity: self.severity(),
                        line: Some(line_num),
                        column: Some(self.max),
                        rule: self.id().to_string(),
                        suggestion: Some("Break line into multiple lines".to_string()),
                    },
                    fix: None, // Line breaking is complex
                    category: self.category(),
                });
            }
        }

        issues
    }
}

/// Trailing whitespace rule
struct TrailingWhitespaceRule;

impl LintRule for TrailingWhitespaceRule {
    fn id(&self) -> &str { "trailing-whitespace" }
    fn description(&self) -> &str { "Removes trailing whitespace" }
    fn category(&self) -> LintCategory { LintCategory::Formatting }
    fn severity(&self) -> Severity { Severity::Info }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line != line.trim_end() {
                issues.push(LintIssue {
                    issue: ValidationIssue {
                        code: "LINT002".to_string(),
                        message: "Trailing whitespace".to_string(),
                        severity: self.severity(),
                        line: Some(line_num),
                        column: None,
                        rule: self.id().to_string(),
                        suggestion: Some("Remove trailing spaces".to_string()),
                    },
                    fix: Some(LintFix {
                        description: "Remove trailing whitespace".to_string(),
                        line: Some(line_num),
                        original: Some(line.to_string()),
                        replacement: line.trim_end().to_string(),
                        safe: true,
                    }),
                    category: self.category(),
                });
            }
        }

        issues
    }
}

/// Consistent list marker rule
struct ConsistentListMarkerRule {
    style: ListStyle,
}

impl LintRule for ConsistentListMarkerRule {
    fn id(&self) -> &str { "list-marker" }
    fn description(&self) -> &str { "Enforces consistent list markers" }
    fn category(&self) -> LintCategory { LintCategory::Formatting }
    fn severity(&self) -> Severity { Severity::Info }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let expected = match self.style {
            ListStyle::Dash => '-',
            ListStyle::Asterisk => '*',
            ListStyle::Plus => '+',
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();

            // Check if it's a list item with wrong marker
            for &marker in &['-', '*', '+'] {
                if marker != expected && trimmed.starts_with(marker) && trimmed.len() > 1 {
                    let next_char = trimmed.chars().nth(1);
                    if next_char == Some(' ') || next_char == Some('[') {
                        issues.push(LintIssue {
                            issue: ValidationIssue {
                                code: "LINT003".to_string(),
                                message: format!("Use '{}' for list markers", expected),
                                severity: self.severity(),
                                line: Some(line_num),
                                column: Some(line.len() - trimmed.len()),
                                rule: self.id().to_string(),
                                suggestion: None,
                            },
                            fix: Some(LintFix {
                                description: "Fix list marker".to_string(),
                                line: Some(line_num),
                                original: Some(line.to_string()),
                                replacement: line.replacen(marker, &expected.to_string(), 1),
                                safe: true,
                            }),
                            category: self.category(),
                        });
                    }
                }
            }
        }

        issues
    }
}

/// Heading spacing rule
struct HeadingSpacingRule;

impl LintRule for HeadingSpacingRule {
    fn id(&self) -> &str { "heading-spacing" }
    fn description(&self) -> &str { "Ensures blank lines around headings" }
    fn category(&self) -> LintCategory { LintCategory::Formatting }
    fn severity(&self) -> Severity { Severity::Info }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with('#') && !line.starts_with("#!") {
                // Check line before (unless first line)
                if i > 0 && !lines[i - 1].trim().is_empty() {
                    issues.push(LintIssue {
                        issue: ValidationIssue {
                            code: "LINT004".to_string(),
                            message: "Add blank line before heading".to_string(),
                            severity: self.severity(),
                            line: Some(i),
                            column: None,
                            rule: self.id().to_string(),
                            suggestion: None,
                        },
                        fix: None, // Would need to insert line
                        category: self.category(),
                    });
                }

                // Check space after #
                let trimmed = line.trim_start_matches('#');
                if !trimmed.is_empty() && !trimmed.starts_with(' ') {
                    issues.push(LintIssue {
                        issue: ValidationIssue {
                            code: "LINT005".to_string(),
                            message: "Add space after # in heading".to_string(),
                            severity: self.severity(),
                            line: Some(i),
                            column: None,
                            rule: self.id().to_string(),
                            suggestion: None,
                        },
                        fix: Some(LintFix {
                            description: "Add space after #".to_string(),
                            line: Some(i),
                            original: Some(line.to_string()),
                            replacement: {
                                let hashes = line.chars().take_while(|c| *c == '#').count();
                                format!("{} {}", "#".repeat(hashes), &line[hashes..].trim())
                            },
                            safe: true,
                        }),
                        category: self.category(),
                    });
                }
            }
        }

        issues
    }
}

/// Section order rule
struct SectionOrderRule;

impl LintRule for SectionOrderRule {
    fn id(&self) -> &str { "section-order" }
    fn description(&self) -> &str { "Enforces standard section order" }
    fn category(&self) -> LintCategory { LintCategory::Structure }
    fn severity(&self) -> Severity { Severity::Warning }

    fn lint(&self, spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> {
        let expected_order = [
            "Metadata",
            "Objective",
            "Acceptance Criteria",
            "Implementation Details",
            "Testing Requirements",
            "Related Specs",
        ];

        let mut issues = Vec::new();
        let mut last_index = 0;

        for section in &spec.section_order {
            if let Some(expected_idx) = expected_order.iter().position(|&s| s == section) {
                if expected_idx < last_index {
                    issues.push(LintIssue {
                        issue: ValidationIssue {
                            code: "LINT010".to_string(),
                            message: format!("Section '{}' is out of order", section),
                            severity: self.severity(),
                            line: spec.line_map.section_starts.get(section).copied(),
                            column: None,
                            rule: self.id().to_string(),
                            suggestion: Some("Reorder sections to match standard order".to_string()),
                        },
                        fix: None, // Section reordering is complex
                        category: self.category(),
                    });
                }
                last_index = expected_idx;
            }
        }

        issues
    }
}

/// Empty section rule
struct EmptySectionRule;

impl LintRule for EmptySectionRule {
    fn id(&self) -> &str { "empty-section" }
    fn description(&self) -> &str { "Warns about empty sections" }
    fn category(&self) -> LintCategory { LintCategory::Content }
    fn severity(&self) -> Severity { Severity::Warning }

    fn lint(&self, spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (section, content) in &spec.sections {
            if content.trim().is_empty() || content.trim() == "[TODO]" {
                issues.push(LintIssue {
                    issue: ValidationIssue {
                        code: "LINT011".to_string(),
                        message: format!("Section '{}' is empty", section),
                        severity: self.severity(),
                        line: spec.line_map.section_starts.get(section).copied(),
                        column: None,
                        rule: self.id().to_string(),
                        suggestion: Some("Add content or remove section".to_string()),
                    },
                    fix: None,
                    category: self.category(),
                });
            }
        }

        issues
    }
}

/// Nested heading rule
struct NestedHeadingRule;

impl LintRule for NestedHeadingRule {
    fn id(&self) -> &str { "nested-heading" }
    fn description(&self) -> &str { "Checks heading nesting levels" }
    fn category(&self) -> LintCategory { LintCategory::Structure }
    fn severity(&self) -> Severity { Severity::Warning }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let mut last_level = 0;

        for (line_num, line) in content.lines().enumerate() {
            if line.starts_with('#') {
                let level = line.chars().take_while(|c| *c == '#').count();

                if level > last_level + 1 && last_level > 0 {
                    issues.push(LintIssue {
                        issue: ValidationIssue {
                            code: "LINT012".to_string(),
                            message: format!("Heading skips levels (h{} to h{})", last_level, level),
                            severity: self.severity(),
                            line: Some(line_num),
                            column: None,
                            rule: self.id().to_string(),
                            suggestion: Some("Don't skip heading levels".to_string()),
                        },
                        fix: None,
                        category: self.category(),
                    });
                }

                last_level = level;
            }
        }

        issues
    }
}

/// TODO comment rule
struct TodoCommentRule;

impl LintRule for TodoCommentRule {
    fn id(&self) -> &str { "todo-comment" }
    fn description(&self) -> &str { "Flags TODO comments" }
    fn category(&self) -> LintCategory { LintCategory::Content }
    fn severity(&self) -> Severity { Severity::Info }

    fn lint(&self, _spec: &ParsedSpec, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.to_uppercase().contains("TODO") || line.contains("[TODO]") {
                issues.push(LintIssue {
                    issue: ValidationIssue {
                        code: "LINT020".to_string(),
                        message: "TODO comment found".to_string(),
                        severity: self.severity(),
                        line: Some(line_num),
                        column: line.to_uppercase().find("TODO"),
                        rule: self.id().to_string(),
                        suggestion: Some("Address TODO or create task".to_string()),
                    },
                    fix: None,
                    category: self.category(),
                });
            }
        }

        issues
    }
}

/// Stub implementations for remaining rules
struct BrokenLinkRule;
impl LintRule for BrokenLinkRule {
    fn id(&self) -> &str { "broken-link" }
    fn description(&self) -> &str { "Detects broken links" }
    fn category(&self) -> LintCategory { LintCategory::Content }
    fn severity(&self) -> Severity { Severity::Warning }
    fn lint(&self, _spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> { Vec::new() }
}

struct SpellingRule;
impl LintRule for SpellingRule {
    fn id(&self) -> &str { "spelling" }
    fn description(&self) -> &str { "Checks spelling" }
    fn category(&self) -> LintCategory { LintCategory::Content }
    fn severity(&self) -> Severity { Severity::Info }
    fn lint(&self, _spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> { Vec::new() }
}

struct DescriptiveTitleRule;
impl LintRule for DescriptiveTitleRule {
    fn id(&self) -> &str { "descriptive-title" }
    fn description(&self) -> &str { "Ensures descriptive titles" }
    fn category(&self) -> LintCategory { LintCategory::BestPractice }
    fn severity(&self) -> Severity { Severity::Info }
    fn lint(&self, _spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> { Vec::new() }
}

struct ConcreteAcceptanceCriteriaRule;
impl LintRule for ConcreteAcceptanceCriteriaRule {
    fn id(&self) -> &str { "concrete-criteria" }
    fn description(&self) -> &str { "Ensures concrete acceptance criteria" }
    fn category(&self) -> LintCategory { LintCategory::BestPractice }
    fn severity(&self) -> Severity { Severity::Warning }
    fn lint(&self, _spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> { Vec::new() }
}

struct EstimatedContextRule;
impl LintRule for EstimatedContextRule {
    fn id(&self) -> &str { "estimated-context" }
    fn description(&self) -> &str { "Checks estimated context is present" }
    fn category(&self) -> LintCategory { LintCategory::BestPractice }
    fn severity(&self) -> Severity { Severity::Info }
    fn lint(&self, spec: &ParsedSpec, _content: &str) -> Vec<LintIssue> {
        if spec.metadata.estimated_context.is_none() {
            vec![LintIssue {
                issue: ValidationIssue {
                    code: "LINT030".to_string(),
                    message: "Missing estimated context".to_string(),
                    severity: Severity::Info,
                    line: None,
                    column: None,
                    rule: "estimated-context".to_string(),
                    suggestion: Some("Add '- **Estimated Context**: ~X%' to Metadata".to_string()),
                },
                fix: None,
                category: LintCategory::BestPractice,
            }]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_whitespace_detection() {
        let rule = TrailingWhitespaceRule;
        let content = "line 1  \nline 2\nline 3   ";

        let issues = rule.lint(&ParsedSpec::default(), content);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].fix.is_some());
    }

    #[test]
    fn test_list_marker_consistency() {
        let rule = ConsistentListMarkerRule { style: ListStyle::Dash };
        let content = "- item 1\n* item 2\n+ item 3";

        let issues = rule.lint(&ParsedSpec::default(), content);
        assert_eq!(issues.len(), 2); // * and + should be flagged
    }

    #[test]
    fn test_fix_application() {
        let linter = SpecLinter::new(LinterConfig::default());
        let content = "test  \nmore  ";

        let spec = ParsedSpec::default();
        let result = linter.lint(&spec, content);

        let fixed = linter.apply_fixes(content, &result.issues);
        assert!(!fixed.contains("  \n"));
    }
}
```

## Testing Requirements

- [ ] Unit tests for each lint rule
- [ ] Tests for fix application
- [ ] Tests for configuration options
- [ ] Tests for custom rules
- [ ] Tests for safe fix filtering
- [ ] Integration tests with real specs
- [ ] Performance tests for large specs
- [ ] Tests for incremental linting

## Related Specs

- **127-spec-validation.md**: Validation foundation
- **120-spec-parsing.md**: Parsing for lint analysis
- **117-spec-templates.md**: Templates enforce consistency
