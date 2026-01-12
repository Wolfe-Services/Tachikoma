# Spec 127: Spec Validation Rules

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 127
- **Status**: Planned
- **Dependencies**: 120-spec-parsing, 121-spec-metadata
- **Estimated Context**: ~10%

## Objective

Define comprehensive validation rules for specification documents. Validation ensures specs meet quality standards, contain required sections, have valid references, and follow project conventions. This enables automated spec review and quality gates.

## Acceptance Criteria

- [ ] Required section validation works
- [ ] Metadata field validation works
- [ ] Reference validation (specs, patterns) works
- [ ] Code block validation works
- [ ] Custom validation rules are supported
- [ ] Validation severity levels are respected
- [ ] Batch validation is efficient
- [ ] Validation reports are actionable

## Implementation Details

### Validation System

```rust
// src/spec/validation.rs

use std::collections::{HashMap, HashSet};
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::spec::parsing::ParsedSpec;
use crate::spec::metadata::SpecMetadata;

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - suggestion for improvement
    Info,
    /// Warning - should be fixed but not blocking
    Warning,
    /// Error - must be fixed
    Error,
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue code (e.g., "SPEC001")
    pub code: String,
    /// Issue message
    pub message: String,
    /// Severity level
    pub severity: Severity,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Column (if applicable)
    pub column: Option<usize>,
    /// Rule that triggered this issue
    pub rule: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Validation result for a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Spec ID
    pub spec_id: u32,
    /// Spec path
    pub path: String,
    /// List of issues
    pub issues: Vec<ValidationIssue>,
    /// Whether validation passed (no errors)
    pub passed: bool,
    /// Count by severity
    pub counts: ValidationCounts,
}

/// Issue counts by severity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationCounts {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// Validation rule trait
pub trait ValidationRule: Send + Sync {
    /// Rule identifier
    fn id(&self) -> &str;

    /// Rule description
    fn description(&self) -> &str;

    /// Default severity
    fn severity(&self) -> Severity;

    /// Validate a parsed spec
    fn validate(&self, spec: &ParsedSpec, metadata: &SpecMetadata) -> Vec<ValidationIssue>;
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enabled rules
    pub enabled_rules: HashSet<String>,
    /// Disabled rules
    pub disabled_rules: HashSet<String>,
    /// Severity overrides
    pub severity_overrides: HashMap<String, Severity>,
    /// Required sections
    pub required_sections: Vec<String>,
    /// Required metadata fields
    pub required_metadata: Vec<String>,
    /// Minimum acceptance criteria count
    pub min_acceptance_criteria: usize,
    /// Require implementation section
    pub require_implementation: bool,
    /// Require test section
    pub require_tests: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled_rules: HashSet::new(),
            disabled_rules: HashSet::new(),
            severity_overrides: HashMap::new(),
            required_sections: vec![
                "Metadata".to_string(),
                "Objective".to_string(),
                "Acceptance Criteria".to_string(),
            ],
            required_metadata: vec![
                "Phase".to_string(),
                "Spec ID".to_string(),
                "Status".to_string(),
            ],
            min_acceptance_criteria: 3,
            require_implementation: true,
            require_tests: true,
        }
    }
}

/// Main spec validator
pub struct SpecValidator {
    rules: Vec<Box<dyn ValidationRule>>,
    config: ValidationConfig,
}

impl SpecValidator {
    pub fn new(config: ValidationConfig) -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            config,
        };

        // Register built-in rules
        validator.register_builtin_rules();

        validator
    }

    /// Register built-in validation rules
    fn register_builtin_rules(&mut self) {
        self.rules.push(Box::new(RequiredSectionsRule::new(
            self.config.required_sections.clone()
        )));
        self.rules.push(Box::new(RequiredMetadataRule::new(
            self.config.required_metadata.clone()
        )));
        self.rules.push(Box::new(AcceptanceCriteriaRule::new(
            self.config.min_acceptance_criteria
        )));
        self.rules.push(Box::new(ImplementationSectionRule));
        self.rules.push(Box::new(TestSectionRule));
        self.rules.push(Box::new(SpecReferenceRule));
        self.rules.push(Box::new(CodeBlockRule));
        self.rules.push(Box::new(TitleFormatRule));
        self.rules.push(Box::new(StatusValueRule));
    }

    /// Add a custom rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate a spec
    pub fn validate(&self, spec: &ParsedSpec, metadata: &SpecMetadata) -> ValidationResult {
        let mut issues = Vec::new();

        for rule in &self.rules {
            // Skip disabled rules
            if self.config.disabled_rules.contains(rule.id()) {
                continue;
            }

            // Check if rule is enabled (if whitelist is used)
            if !self.config.enabled_rules.is_empty()
                && !self.config.enabled_rules.contains(rule.id())
            {
                continue;
            }

            let mut rule_issues = rule.validate(spec, metadata);

            // Apply severity overrides
            for issue in &mut rule_issues {
                if let Some(override_severity) = self.config.severity_overrides.get(rule.id()) {
                    issue.severity = *override_severity;
                }
            }

            issues.extend(rule_issues);
        }

        // Calculate counts
        let counts = ValidationCounts {
            errors: issues.iter().filter(|i| i.severity == Severity::Error).count(),
            warnings: issues.iter().filter(|i| i.severity == Severity::Warning).count(),
            infos: issues.iter().filter(|i| i.severity == Severity::Info).count(),
        };

        ValidationResult {
            spec_id: metadata.id,
            path: metadata.path.to_string_lossy().to_string(),
            passed: counts.errors == 0,
            issues,
            counts,
        }
    }

    /// Validate multiple specs
    pub fn validate_batch(
        &self,
        specs: &[(ParsedSpec, SpecMetadata)],
    ) -> Vec<ValidationResult> {
        specs.iter()
            .map(|(spec, meta)| self.validate(spec, meta))
            .collect()
    }
}

// ===== Built-in Rules =====

/// Required sections rule
struct RequiredSectionsRule {
    required: Vec<String>,
}

impl RequiredSectionsRule {
    fn new(required: Vec<String>) -> Self {
        Self { required }
    }
}

impl ValidationRule for RequiredSectionsRule {
    fn id(&self) -> &str { "required-sections" }
    fn description(&self) -> &str { "Checks for required sections" }
    fn severity(&self) -> Severity { Severity::Error }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for section in &self.required {
            if !spec.sections.contains_key(section) {
                issues.push(ValidationIssue {
                    code: "SPEC001".to_string(),
                    message: format!("Missing required section: {}", section),
                    severity: self.severity(),
                    line: None,
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: Some(format!("Add a '## {}' section", section)),
                });
            }
        }

        issues
    }
}

/// Required metadata rule
struct RequiredMetadataRule {
    required: Vec<String>,
}

impl RequiredMetadataRule {
    fn new(required: Vec<String>) -> Self {
        Self { required }
    }
}

impl ValidationRule for RequiredMetadataRule {
    fn id(&self) -> &str { "required-metadata" }
    fn description(&self) -> &str { "Checks for required metadata fields" }
    fn severity(&self) -> Severity { Severity::Error }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for field in &self.required {
            let field_lower = field.to_lowercase();
            let has_field = match field_lower.as_str() {
                "phase" => spec.metadata.phase > 0,
                "spec id" | "spec-id" => spec.metadata.spec_id > 0,
                "status" => !spec.metadata.status.is_empty(),
                "dependencies" => true, // Optional
                _ => spec.metadata.custom.contains_key(field),
            };

            if !has_field {
                issues.push(ValidationIssue {
                    code: "SPEC002".to_string(),
                    message: format!("Missing required metadata: {}", field),
                    severity: self.severity(),
                    line: spec.line_map.metadata_range.as_ref().map(|r| r.start),
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: Some(format!("Add '- **{}**: value' to Metadata section", field)),
                });
            }
        }

        issues
    }
}

/// Acceptance criteria rule
struct AcceptanceCriteriaRule {
    min_count: usize,
}

impl AcceptanceCriteriaRule {
    fn new(min_count: usize) -> Self {
        Self { min_count }
    }
}

impl ValidationRule for AcceptanceCriteriaRule {
    fn id(&self) -> &str { "acceptance-criteria" }
    fn description(&self) -> &str { "Validates acceptance criteria" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let ac_count = spec.acceptance_criteria.len();

        if ac_count < self.min_count {
            issues.push(ValidationIssue {
                code: "SPEC003".to_string(),
                message: format!(
                    "Insufficient acceptance criteria: {} (minimum {})",
                    ac_count, self.min_count
                ),
                severity: self.severity(),
                line: spec.line_map.section_starts.get("Acceptance Criteria").copied(),
                column: None,
                rule: self.id().to_string(),
                suggestion: Some("Add more acceptance criteria checkboxes".to_string()),
            });
        }

        // Check for empty criteria
        for criterion in &spec.acceptance_criteria {
            if criterion.text.trim().is_empty() {
                issues.push(ValidationIssue {
                    code: "SPEC004".to_string(),
                    message: "Empty acceptance criterion".to_string(),
                    severity: Severity::Error,
                    line: Some(criterion.line),
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: Some("Add description to the checkbox".to_string()),
                });
            }
        }

        issues
    }
}

/// Implementation section rule
struct ImplementationSectionRule;

impl ValidationRule for ImplementationSectionRule {
    fn id(&self) -> &str { "implementation-section" }
    fn description(&self) -> &str { "Checks for implementation details" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        let has_impl = spec.sections.contains_key("Implementation Details")
            || spec.sections.contains_key("Implementation");

        if !has_impl {
            issues.push(ValidationIssue {
                code: "SPEC005".to_string(),
                message: "Missing Implementation Details section".to_string(),
                severity: self.severity(),
                line: None,
                column: None,
                rule: self.id().to_string(),
                suggestion: Some("Add '## Implementation Details' section with code".to_string()),
            });
        }

        issues
    }
}

/// Test section rule
struct TestSectionRule;

impl ValidationRule for TestSectionRule {
    fn id(&self) -> &str { "test-section" }
    fn description(&self) -> &str { "Checks for testing requirements" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        let has_tests = spec.sections.contains_key("Testing Requirements")
            || spec.sections.contains_key("Tests")
            || spec.sections.contains_key("Test Plan");

        if !has_tests {
            issues.push(ValidationIssue {
                code: "SPEC006".to_string(),
                message: "Missing Testing Requirements section".to_string(),
                severity: self.severity(),
                line: None,
                column: None,
                rule: self.id().to_string(),
                suggestion: Some("Add '## Testing Requirements' section".to_string()),
            });
        }

        issues
    }
}

/// Spec reference validation rule
struct SpecReferenceRule;

impl ValidationRule for SpecReferenceRule {
    fn id(&self) -> &str { "spec-references" }
    fn description(&self) -> &str { "Validates spec references" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check for circular references (self-reference)
        for reference in &spec.references {
            if reference.spec_id == spec.metadata.spec_id {
                issues.push(ValidationIssue {
                    code: "SPEC007".to_string(),
                    message: "Self-referencing spec".to_string(),
                    severity: Severity::Info,
                    line: Some(reference.line),
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: None,
                });
            }
        }

        issues
    }
}

/// Code block rule
struct CodeBlockRule;

impl ValidationRule for CodeBlockRule {
    fn id(&self) -> &str { "code-blocks" }
    fn description(&self) -> &str { "Validates code blocks" }
    fn severity(&self) -> Severity { Severity::Info }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for block in &spec.code_blocks {
            if block.language.is_none() {
                issues.push(ValidationIssue {
                    code: "SPEC008".to_string(),
                    message: "Code block missing language identifier".to_string(),
                    severity: self.severity(),
                    line: Some(block.lines.start),
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: Some("Add language after opening ```".to_string()),
                });
            }

            if block.content.trim().is_empty() {
                issues.push(ValidationIssue {
                    code: "SPEC009".to_string(),
                    message: "Empty code block".to_string(),
                    severity: Severity::Warning,
                    line: Some(block.lines.start),
                    column: None,
                    rule: self.id().to_string(),
                    suggestion: Some("Add code or remove block".to_string()),
                });
            }
        }

        issues
    }
}

/// Title format rule
struct TitleFormatRule;

impl ValidationRule for TitleFormatRule {
    fn id(&self) -> &str { "title-format" }
    fn description(&self) -> &str { "Validates title format" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Expected format: "Spec XXX: Title"
        let title_re = regex::Regex::new(r"^Spec \d+: .+").unwrap();

        if !title_re.is_match(&spec.title) {
            issues.push(ValidationIssue {
                code: "SPEC010".to_string(),
                message: format!("Title doesn't match expected format: '{}'", spec.title),
                severity: self.severity(),
                line: Some(0),
                column: None,
                rule: self.id().to_string(),
                suggestion: Some("Use format: 'Spec XXX: Title'".to_string()),
            });
        }

        issues
    }
}

/// Status value rule
struct StatusValueRule;

impl ValidationRule for StatusValueRule {
    fn id(&self) -> &str { "status-value" }
    fn description(&self) -> &str { "Validates status value" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn validate(&self, spec: &ParsedSpec, _metadata: &SpecMetadata) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        let valid_statuses = ["Planned", "In Progress", "Review", "Complete", "Blocked", "Draft", "Deprecated"];
        let status = &spec.metadata.status;

        if !valid_statuses.iter().any(|s| s.eq_ignore_ascii_case(status)) {
            issues.push(ValidationIssue {
                code: "SPEC011".to_string(),
                message: format!("Invalid status value: '{}'", status),
                severity: self.severity(),
                line: spec.line_map.metadata_range.as_ref().map(|r| r.start),
                column: None,
                rule: self.id().to_string(),
                suggestion: Some(format!("Use one of: {}", valid_statuses.join(", "))),
            });
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_required_sections_rule() {
        let rule = RequiredSectionsRule::new(vec!["Objective".to_string()]);

        let mut spec = ParsedSpec::default();
        spec.sections.insert("Implementation".to_string(), String::new());

        let issues = rule.validate(&spec, &SpecMetadata::default());
        assert!(!issues.is_empty());
        assert!(issues[0].message.contains("Objective"));
    }
}

// Default implementations for testing
impl Default for ParsedSpec {
    fn default() -> Self {
        Self {
            title: String::new(),
            metadata: crate::spec::parsing::SpecMetadata::default(),
            sections: HashMap::new(),
            section_order: Vec::new(),
            acceptance_criteria: Vec::new(),
            code_blocks: Vec::new(),
            references: Vec::new(),
            warnings: Vec::new(),
            line_map: crate::spec::parsing::LineMap::default(),
        }
    }
}
```

## Testing Requirements

- [ ] Unit tests for each validation rule
- [ ] Tests for severity levels
- [ ] Tests for configuration overrides
- [ ] Tests for batch validation
- [ ] Tests for custom rules
- [ ] Integration tests with real specs
- [ ] Tests for validation reporting
- [ ] Performance tests for large spec sets

## Related Specs

- **120-spec-parsing.md**: Parses specs for validation
- **128-spec-linting.md**: Linting extends validation
- **121-spec-metadata.md**: Metadata validation
