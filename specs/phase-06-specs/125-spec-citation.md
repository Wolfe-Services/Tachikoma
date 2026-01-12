# Spec 125: Spec Citation System

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 125
- **Status**: Planned
- **Dependencies**: 120-spec-parsing, 121-spec-metadata
- **Estimated Context**: ~8%

## Objective

Implement a citation system for referencing specs in code, documentation, and conversations. Citations provide traceable links between implementation and specification, enabling verification that code fulfills spec requirements and allowing navigation from code back to specs.

## Acceptance Criteria

- [ ] Multiple citation formats are supported
- [ ] Citations are parsed from code comments
- [ ] Citations are validated against spec index
- [ ] Citation coverage reports are generated
- [ ] Broken citations are detected
- [ ] Citations support section-level references
- [ ] IDE integration provides citation hints
- [ ] Citation index is searchable

## Implementation Details

### Citation System

```rust
// src/spec/citation.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A spec citation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Citation format used
    pub format: CitationFormat,
    /// Referenced spec ID
    pub spec_id: u32,
    /// Optional section reference
    pub section: Option<String>,
    /// Optional criterion index
    pub criterion: Option<u32>,
    /// Source file containing citation
    pub source_file: PathBuf,
    /// Line number in source
    pub line: usize,
    /// Column position
    pub column: usize,
    /// Full citation text
    pub text: String,
    /// Whether citation is valid
    pub valid: bool,
    /// Validation message if invalid
    pub validation_message: Option<String>,
}

/// Supported citation formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitationFormat {
    /// spec:116 or SPEC-116
    SpecId,
    /// spec:116#Objective
    SpecSection,
    /// spec:116#AC-3 (acceptance criterion 3)
    SpecCriterion,
    /// spec:116-spec-directory.md
    SpecFilename,
    /// [Spec 116](../specs/...)
    MarkdownLink,
    /// @spec 116
    AtSpec,
    /// Implements: spec:116
    ImplementsTag,
    /// Per spec:116
    PerTag,
}

impl CitationFormat {
    /// Get regex pattern for this format
    pub fn pattern(&self) -> &'static str {
        match self {
            Self::SpecId => r"(?i)spec[:\-](\d{3})",
            Self::SpecSection => r"(?i)spec[:\-](\d{3})#([A-Za-z][A-Za-z0-9\s\-]+)",
            Self::SpecCriterion => r"(?i)spec[:\-](\d{3})#AC[:\-](\d+)",
            Self::SpecFilename => r"(\d{3})-[\w\-]+\.md",
            Self::MarkdownLink => r"\[.*?[Ss]pec\s*(\d+).*?\]\([^)]+\)",
            Self::AtSpec => r"@spec\s+(\d{3})",
            Self::ImplementsTag => r"[Ii]mplements:\s*spec[:\-](\d{3})",
            Self::PerTag => r"[Pp]er\s+spec[:\-](\d{3})",
        }
    }
}

/// Citation parser
pub struct CitationParser {
    patterns: Vec<(CitationFormat, Regex)>,
}

impl CitationParser {
    pub fn new() -> Self {
        let formats = [
            CitationFormat::SpecCriterion, // Most specific first
            CitationFormat::SpecSection,
            CitationFormat::ImplementsTag,
            CitationFormat::PerTag,
            CitationFormat::AtSpec,
            CitationFormat::SpecFilename,
            CitationFormat::MarkdownLink,
            CitationFormat::SpecId, // Most general last
        ];

        let patterns = formats.iter()
            .map(|f| (*f, Regex::new(f.pattern()).unwrap()))
            .collect();

        Self { patterns }
    }

    /// Parse citations from content
    pub fn parse(&self, content: &str, source_file: &Path) -> Vec<Citation> {
        let mut citations = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            citations.extend(self.parse_line(line, line_num, source_file));
        }

        citations
    }

    /// Parse a single line for citations
    fn parse_line(&self, line: &str, line_num: usize, source_file: &Path) -> Vec<Citation> {
        let mut citations = Vec::new();
        let mut matched_positions: Vec<(usize, usize)> = Vec::new();

        for (format, regex) in &self.patterns {
            for mat in regex.find_iter(line) {
                // Skip if this position was already matched by a more specific pattern
                let pos = (mat.start(), mat.end());
                if matched_positions.iter().any(|&(s, e)| pos.0 >= s && pos.1 <= e) {
                    continue;
                }

                if let Some(caps) = regex.captures(mat.as_str()) {
                    if let Some(citation) = self.create_citation(
                        *format,
                        &caps,
                        mat.as_str(),
                        line_num,
                        mat.start(),
                        source_file,
                    ) {
                        citations.push(citation);
                        matched_positions.push(pos);
                    }
                }
            }
        }

        citations
    }

    /// Create citation from regex capture
    fn create_citation(
        &self,
        format: CitationFormat,
        caps: &regex::Captures,
        text: &str,
        line: usize,
        column: usize,
        source_file: &Path,
    ) -> Option<Citation> {
        let spec_id: u32 = caps.get(1)?.as_str().parse().ok()?;

        let section = match format {
            CitationFormat::SpecSection => caps.get(2).map(|m| m.as_str().to_string()),
            _ => None,
        };

        let criterion = match format {
            CitationFormat::SpecCriterion => caps.get(2).and_then(|m| m.as_str().parse().ok()),
            _ => None,
        };

        Some(Citation {
            format,
            spec_id,
            section,
            criterion,
            source_file: source_file.to_path_buf(),
            line,
            column,
            text: text.to_string(),
            valid: true,
            validation_message: None,
        })
    }

    /// Parse citations from a source file
    pub async fn parse_file(&self, path: &Path) -> Result<Vec<Citation>, CitationError> {
        let content = tokio::fs::read_to_string(path).await?;
        Ok(self.parse(&content, path))
    }
}

/// Citation validator
pub struct CitationValidator {
    /// Known spec IDs
    known_specs: HashMap<u32, SpecInfo>,
}

/// Minimal spec info for validation
#[derive(Debug, Clone)]
pub struct SpecInfo {
    pub id: u32,
    pub title: String,
    pub sections: Vec<String>,
    pub acceptance_criteria_count: u32,
}

impl CitationValidator {
    pub fn new() -> Self {
        Self {
            known_specs: HashMap::new(),
        }
    }

    /// Load spec info for validation
    pub fn load_spec(&mut self, info: SpecInfo) {
        self.known_specs.insert(info.id, info);
    }

    /// Validate a citation
    pub fn validate(&self, citation: &mut Citation) {
        // Check if spec exists
        let spec = match self.known_specs.get(&citation.spec_id) {
            Some(s) => s,
            None => {
                citation.valid = false;
                citation.validation_message = Some(format!(
                    "Unknown spec ID: {}",
                    citation.spec_id
                ));
                return;
            }
        };

        // Validate section reference
        if let Some(section) = &citation.section {
            if !spec.sections.iter().any(|s| s.eq_ignore_ascii_case(section)) {
                citation.valid = false;
                citation.validation_message = Some(format!(
                    "Section '{}' not found in spec {}",
                    section, citation.spec_id
                ));
                return;
            }
        }

        // Validate criterion reference
        if let Some(criterion) = citation.criterion {
            if criterion > spec.acceptance_criteria_count || criterion == 0 {
                citation.valid = false;
                citation.validation_message = Some(format!(
                    "Acceptance criterion {} not found in spec {} (has {} criteria)",
                    criterion, citation.spec_id, spec.acceptance_criteria_count
                ));
                return;
            }
        }

        citation.valid = true;
        citation.validation_message = None;
    }

    /// Validate all citations
    pub fn validate_all(&self, citations: &mut [Citation]) {
        for citation in citations {
            self.validate(citation);
        }
    }
}

/// Citation index for searching
pub struct CitationIndex {
    /// Citations by spec ID
    by_spec: HashMap<u32, Vec<Citation>>,
    /// Citations by source file
    by_file: HashMap<PathBuf, Vec<Citation>>,
    /// All citations
    all: Vec<Citation>,
}

impl CitationIndex {
    pub fn new() -> Self {
        Self {
            by_spec: HashMap::new(),
            by_file: HashMap::new(),
            all: Vec::new(),
        }
    }

    /// Add citations to index
    pub fn add(&mut self, citations: Vec<Citation>) {
        for citation in citations {
            self.by_spec
                .entry(citation.spec_id)
                .or_default()
                .push(citation.clone());

            self.by_file
                .entry(citation.source_file.clone())
                .or_default()
                .push(citation.clone());

            self.all.push(citation);
        }
    }

    /// Get citations for a spec
    pub fn for_spec(&self, spec_id: u32) -> &[Citation] {
        self.by_spec.get(&spec_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get citations in a file
    pub fn for_file(&self, path: &Path) -> &[Citation] {
        self.by_file.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all invalid citations
    pub fn invalid(&self) -> Vec<&Citation> {
        self.all.iter().filter(|c| !c.valid).collect()
    }

    /// Generate coverage report
    pub fn coverage_report(&self, total_specs: u32) -> CitationCoverage {
        let cited_specs: std::collections::HashSet<_> =
            self.by_spec.keys().copied().collect();

        let uncited: Vec<_> = (1..=total_specs)
            .filter(|id| !cited_specs.contains(id))
            .collect();

        let citations_per_spec: HashMap<_, _> = self.by_spec.iter()
            .map(|(id, citations)| (*id, citations.len()))
            .collect();

        CitationCoverage {
            total_specs,
            cited_specs: cited_specs.len() as u32,
            uncited_specs: uncited,
            total_citations: self.all.len(),
            valid_citations: self.all.iter().filter(|c| c.valid).count(),
            invalid_citations: self.all.iter().filter(|c| !c.valid).count(),
            citations_per_spec,
        }
    }
}

/// Citation coverage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationCoverage {
    pub total_specs: u32,
    pub cited_specs: u32,
    pub uncited_specs: Vec<u32>,
    pub total_citations: usize,
    pub valid_citations: usize,
    pub invalid_citations: usize,
    pub citations_per_spec: HashMap<u32, usize>,
}

/// Citation generator for code
pub struct CitationGenerator;

impl CitationGenerator {
    /// Generate citation comment for Rust
    pub fn rust_comment(spec_id: u32, section: Option<&str>) -> String {
        match section {
            Some(s) => format!("// Implements: spec:{}#{}", spec_id, s),
            None => format!("// Implements: spec:{}", spec_id),
        }
    }

    /// Generate citation comment for Python
    pub fn python_comment(spec_id: u32, section: Option<&str>) -> String {
        match section {
            Some(s) => format!("# Implements: spec:{}#{}", spec_id, s),
            None => format!("# Implements: spec:{}", spec_id),
        }
    }

    /// Generate citation for documentation
    pub fn doc_citation(spec_id: u32, spec_title: &str, path: &str) -> String {
        format!("[Spec {}: {}]({})", spec_id, spec_title, path)
    }

    /// Generate inline citation
    pub fn inline(spec_id: u32) -> String {
        format!("spec:{:03}", spec_id)
    }
}

/// Citation errors
#[derive(Debug, thiserror::Error)]
pub enum CitationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl Default for CitationParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CitationValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CitationIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_parsing() {
        let parser = CitationParser::new();
        let content = r#"
// Implements: spec:116
fn example() {
    // Per spec:117#Implementation
    todo!()
}
// spec:118#AC-3
"#;

        let citations = parser.parse(content, Path::new("test.rs"));

        assert!(citations.iter().any(|c| c.spec_id == 116));
        assert!(citations.iter().any(|c| c.spec_id == 117 && c.section.is_some()));
        assert!(citations.iter().any(|c| c.spec_id == 118 && c.criterion == Some(3)));
    }

    #[test]
    fn test_citation_validation() {
        let mut validator = CitationValidator::new();
        validator.load_spec(SpecInfo {
            id: 116,
            title: "Test Spec".to_string(),
            sections: vec!["Objective".to_string(), "Implementation".to_string()],
            acceptance_criteria_count: 5,
        });

        let mut valid_citation = Citation {
            format: CitationFormat::SpecSection,
            spec_id: 116,
            section: Some("Objective".to_string()),
            criterion: None,
            source_file: PathBuf::new(),
            line: 0,
            column: 0,
            text: "spec:116#Objective".to_string(),
            valid: true,
            validation_message: None,
        };

        validator.validate(&mut valid_citation);
        assert!(valid_citation.valid);

        let mut invalid_citation = Citation {
            format: CitationFormat::SpecSection,
            spec_id: 116,
            section: Some("NonExistent".to_string()),
            criterion: None,
            source_file: PathBuf::new(),
            line: 0,
            column: 0,
            text: "spec:116#NonExistent".to_string(),
            valid: true,
            validation_message: None,
        };

        validator.validate(&mut invalid_citation);
        assert!(!invalid_citation.valid);
    }

    #[test]
    fn test_citation_generator() {
        let rust = CitationGenerator::rust_comment(116, Some("Implementation"));
        assert!(rust.contains("spec:116#Implementation"));

        let inline = CitationGenerator::inline(5);
        assert_eq!(inline, "spec:005");
    }
}
```

## Testing Requirements

- [ ] Unit tests for citation parsing
- [ ] Tests for each citation format
- [ ] Tests for citation validation
- [ ] Tests for coverage report generation
- [ ] Tests for citation index operations
- [ ] Tests for citation generation
- [ ] Integration tests with real codebases
- [ ] Tests for edge cases and malformed citations

## Related Specs

- **120-spec-parsing.md**: Parses specs for validation data
- **126-pattern-linking.md**: Pattern references in specs
- **131-spec-search-index.md**: Search includes citations
