# 159 - Forge Result Validation

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 159
**Status:** Planned
**Dependencies:** 154-forge-output, 157-forge-quality
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement validation for Forge session outputs, ensuring generated content meets structural requirements, passes quality checks, and contains no obvious errors.

---

## Acceptance Criteria

- [x] Structural validation for output types
- [x] Code block syntax checking
- [x] Link validation
- [x] Completeness checking
- [x] Quality threshold enforcement
- [x] Validation report generation

---

## Implementation Details

### 1. Validator (src/validation/validator.rs)

```rust
//! Result validation for Forge sessions.

use std::collections::HashMap;

use crate::{ForgeResult, ForgeSession, OutputType, QualityReport};

/// Validates Forge session outputs.
pub struct ResultValidator {
    /// Validation rules.
    rules: Vec<Box<dyn ValidationRule>>,
    /// Minimum quality threshold.
    quality_threshold: f64,
}

/// A validation rule.
pub trait ValidationRule: Send + Sync {
    /// Rule name.
    fn name(&self) -> &str;

    /// Rule description.
    fn description(&self) -> &str;

    /// Validate content.
    fn validate(&self, content: &str, context: &ValidationContext) -> ValidationResult;

    /// Is this rule required to pass.
    fn is_required(&self) -> bool {
        true
    }
}

/// Context for validation.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Output type.
    pub output_type: OutputType,
    /// Topic title.
    pub topic_title: String,
    /// Session constraints.
    pub constraints: Vec<String>,
}

/// Result of a validation rule.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Rule passed.
    pub passed: bool,
    /// Issues found.
    pub issues: Vec<ValidationIssue>,
    /// Warnings (non-blocking).
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn pass() -> Self {
        Self {
            passed: true,
            issues: vec![],
            warnings: vec![],
        }
    }

    pub fn fail(issues: Vec<ValidationIssue>) -> Self {
        Self {
            passed: false,
            issues,
            warnings: vec![],
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

/// A validation issue.
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// Issue severity.
    pub severity: IssueSeverity,
    /// Issue message.
    pub message: String,
    /// Location in content (line number if applicable).
    pub location: Option<usize>,
    /// Suggestion for fix.
    pub suggestion: Option<String>,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Complete validation report.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Overall passed.
    pub passed: bool,
    /// Results by rule.
    pub results: HashMap<String, ValidationResult>,
    /// Total issues.
    pub total_issues: usize,
    /// Total warnings.
    pub total_warnings: usize,
    /// Summary.
    pub summary: String,
}

impl ResultValidator {
    /// Create a new validator with default rules.
    pub fn new(quality_threshold: f64) -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            quality_threshold,
        };

        // Add default rules
        validator.add_rule(Box::new(NotEmptyRule));
        validator.add_rule(Box::new(MinimumLengthRule { min_chars: 500 }));
        validator.add_rule(Box::new(HasStructureRule));
        validator.add_rule(Box::new(NoPlaceholdersRule));
        validator.add_rule(Box::new(CodeBlockSyntaxRule));
        validator.add_rule(Box::new(MarkdownLinksRule));
        validator.add_rule(Box::new(NoTruncationRule));

        validator
    }

    /// Add a validation rule.
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate session output.
    pub fn validate(
        &self,
        session: &ForgeSession,
        quality_report: Option<&QualityReport>,
    ) -> ValidationReport {
        let content = session.latest_draft().unwrap_or_default();

        let context = ValidationContext {
            output_type: session.topic.output_type,
            topic_title: session.topic.title.clone(),
            constraints: session.topic.constraints.clone(),
        };

        let mut results = HashMap::new();
        let mut all_passed = true;
        let mut total_issues = 0;
        let mut total_warnings = 0;

        // Run all rules
        for rule in &self.rules {
            let result = rule.validate(content, &context);

            total_issues += result.issues.len();
            total_warnings += result.warnings.len();

            if !result.passed && rule.is_required() {
                all_passed = false;
            }

            results.insert(rule.name().to_string(), result);
        }

        // Check quality threshold if report provided
        if let Some(report) = quality_report {
            if report.current_score < self.quality_threshold {
                all_passed = false;
                results.insert("quality_threshold".to_string(), ValidationResult::fail(vec![
                    ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!(
                            "Quality score {:.0} below threshold {:.0}",
                            report.current_score,
                            self.quality_threshold
                        ),
                        location: None,
                        suggestion: Some("Continue refinement to improve quality".to_string()),
                    }
                ]));
            }
        }

        let summary = if all_passed {
            "All validation checks passed".to_string()
        } else {
            format!("{} issues found, {} warnings", total_issues, total_warnings)
        };

        ValidationReport {
            passed: all_passed,
            results,
            total_issues,
            total_warnings,
            summary,
        }
    }
}

// --- Built-in Rules ---

/// Checks that content is not empty.
struct NotEmptyRule;

impl ValidationRule for NotEmptyRule {
    fn name(&self) -> &str { "not_empty" }
    fn description(&self) -> &str { "Content must not be empty" }

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        if content.trim().is_empty() {
            ValidationResult::fail(vec![ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Content is empty".to_string(),
                location: None,
                suggestion: Some("Ensure the draft was generated".to_string()),
            }])
        } else {
            ValidationResult::pass()
        }
    }
}

/// Checks minimum content length.
struct MinimumLengthRule {
    min_chars: usize,
}

impl ValidationRule for MinimumLengthRule {
    fn name(&self) -> &str { "minimum_length" }
    fn description(&self) -> &str { "Content must meet minimum length" }

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        if content.len() < self.min_chars {
            ValidationResult::fail(vec![ValidationIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Content too short: {} characters (minimum: {})",
                    content.len(),
                    self.min_chars
                ),
                location: None,
                suggestion: Some("Expand the content with more detail".to_string()),
            }])
        } else {
            ValidationResult::pass()
        }
    }
}

/// Checks that content has structure (headers).
struct HasStructureRule;

impl ValidationRule for HasStructureRule {
    fn name(&self) -> &str { "has_structure" }
    fn description(&self) -> &str { "Content must have markdown structure" }

    fn validate(&self, content: &str, context: &ValidationContext) -> ValidationResult {
        let header_count = content.lines().filter(|l| l.starts_with('#')).count();

        let min_headers = match context.output_type {
            OutputType::Specification => 3,
            OutputType::Documentation => 2,
            OutputType::Design => 3,
            OutputType::Code => 1,
            OutputType::Freeform => 1,
        };

        if header_count < min_headers {
            ValidationResult::fail(vec![ValidationIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Insufficient structure: {} headers (minimum: {})",
                    header_count,
                    min_headers
                ),
                location: None,
                suggestion: Some("Add section headers to organize content".to_string()),
            }])
        } else {
            ValidationResult::pass()
        }
    }
}

/// Checks for placeholder text.
struct NoPlaceholdersRule;

impl ValidationRule for NoPlaceholdersRule {
    fn name(&self) -> &str { "no_placeholders" }
    fn description(&self) -> &str { "Content must not contain placeholders" }

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        let placeholders = [
            "[INSERT", "[TODO:", "[PLACEHOLDER]", "[TBD]",
            "Lorem ipsum", "[YOUR ", "[FILL IN",
        ];

        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for placeholder in &placeholders {
                if line.contains(placeholder) {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        message: format!("Found placeholder: {}", placeholder),
                        location: Some(line_num + 1),
                        suggestion: Some("Replace placeholder with actual content".to_string()),
                    });
                }
            }
        }

        if issues.is_empty() {
            ValidationResult::pass()
        } else {
            ValidationResult::fail(issues)
        }
    }
}

/// Checks code block syntax.
struct CodeBlockSyntaxRule;

impl ValidationRule for CodeBlockSyntaxRule {
    fn name(&self) -> &str { "code_block_syntax" }
    fn description(&self) -> &str { "Code blocks must be properly formatted" }

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut in_code_block = false;
        let mut code_block_start = 0;

        for (line_num, line) in content.lines().enumerate() {
            if line.starts_with("```") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    code_block_start = line_num + 1;

                    // Check for language specifier
                    let lang = line.trim_start_matches('`').trim();
                    if lang.is_empty() {
                        warnings.push(format!(
                            "Line {}: Code block without language specifier",
                            line_num + 1
                        ));
                    }
                }
            }
        }

        // Check for unclosed code block
        if in_code_block {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: "Unclosed code block".to_string(),
                location: Some(code_block_start),
                suggestion: Some("Add closing ``` to code block".to_string()),
            });
        }

        if issues.is_empty() {
            ValidationResult::pass().with_warnings(warnings)
        } else {
            ValidationResult::fail(issues).with_warnings(warnings)
        }
    }
}

/// Checks markdown links.
struct MarkdownLinksRule;

impl ValidationRule for MarkdownLinksRule {
    fn name(&self) -> &str { "markdown_links" }
    fn description(&self) -> &str { "Markdown links must be properly formatted" }
    fn is_required(&self) -> bool { false } // Warning only

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        let link_pattern = regex::Regex::new(r"\[([^\]]+)\]\(([^)]*)\)").unwrap();
        let mut warnings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for cap in link_pattern.captures_iter(line) {
                let text = &cap[1];
                let url = &cap[2];

                if text.is_empty() {
                    warnings.push(format!("Line {}: Link with empty text", line_num + 1));
                }

                if url.is_empty() {
                    warnings.push(format!("Line {}: Link with empty URL: [{}]", line_num + 1, text));
                }
            }
        }

        ValidationResult::pass().with_warnings(warnings)
    }
}

/// Checks for truncation markers.
struct NoTruncationRule;

impl ValidationRule for NoTruncationRule {
    fn name(&self) -> &str { "no_truncation" }
    fn description(&self) -> &str { "Content must not be truncated" }

    fn validate(&self, content: &str, _context: &ValidationContext) -> ValidationResult {
        let truncation_markers = [
            "[truncated]",
            "[content truncated",
            "...more",
            "[continues]",
            "etc.",
        ];

        let last_lines: Vec<_> = content.lines().rev().take(5).collect();

        for marker in &truncation_markers {
            if last_lines.iter().any(|l| l.to_lowercase().contains(marker)) {
                return ValidationResult::fail(vec![ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!("Content appears to be truncated (found: {})", marker),
                    location: None,
                    suggestion: Some("Regenerate with higher token limit".to_string()),
                }]);
            }
        }

        ValidationResult::pass()
    }
}

impl ValidationReport {
    /// Format as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("## Validation Report\n\n");

        md.push_str(&format!(
            "**Status:** {}\n\n",
            if self.passed { "PASSED" } else { "FAILED" }
        ));

        md.push_str(&format!("**Summary:** {}\n\n", self.summary));

        if !self.passed {
            md.push_str("### Issues\n\n");

            for (rule_name, result) in &self.results {
                if !result.passed {
                    md.push_str(&format!("#### {}\n\n", rule_name));
                    for issue in &result.issues {
                        let loc = issue.location.map(|l| format!(" (line {})", l)).unwrap_or_default();
                        md.push_str(&format!("- **{:?}:** {}{}\n", issue.severity, issue.message, loc));
                        if let Some(ref suggestion) = issue.suggestion {
                            md.push_str(&format!("  - Suggestion: {}\n", suggestion));
                        }
                    }
                    md.push('\n');
                }
            }
        }

        if self.total_warnings > 0 {
            md.push_str("### Warnings\n\n");
            for (_, result) in &self.results {
                for warning in &result.warnings {
                    md.push_str(&format!("- {}\n", warning));
                }
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_empty_rule() {
        let rule = NotEmptyRule;
        let context = ValidationContext {
            output_type: OutputType::Specification,
            topic_title: "Test".to_string(),
            constraints: vec![],
        };

        assert!(!rule.validate("", &context).passed);
        assert!(rule.validate("Some content", &context).passed);
    }

    #[test]
    fn test_placeholder_rule() {
        let rule = NoPlaceholdersRule;
        let context = ValidationContext {
            output_type: OutputType::Specification,
            topic_title: "Test".to_string(),
            constraints: vec![],
        };

        assert!(!rule.validate("[INSERT YOUR NAME HERE]", &context).passed);
        assert!(rule.validate("Actual content here", &context).passed);
    }
}
```

---

## Testing Requirements

1. Empty content fails validation
2. Minimum length enforced
3. Placeholders detected correctly
4. Code block syntax checked
5. Truncation markers caught
6. Validation report formats correctly

---

## Related Specs

- Depends on: [154-forge-output.md](154-forge-output.md)
- Depends on: [157-forge-quality.md](157-forge-quality.md)
- Next: [160-forge-tests.md](160-forge-tests.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md), [153-forge-cli.md](153-forge-cli.md)
