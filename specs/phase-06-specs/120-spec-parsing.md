# Spec 120: Spec Markdown Parsing

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 120
- **Status**: Planned
- **Dependencies**: 116-spec-directory, 117-spec-templates
- **Estimated Context**: ~12%

## Objective

Implement a robust markdown parser for specification documents that extracts structured data from the standard spec format. The parser handles metadata blocks, sections, code blocks, checkboxes, and cross-references while maintaining the original document structure for editing and regeneration.

## Acceptance Criteria

- [ ] Metadata block is parsed correctly (Phase, Spec ID, Status, Dependencies)
- [ ] All standard sections are recognized and extracted
- [ ] Code blocks are parsed with language identification
- [ ] Checkboxes/acceptance criteria are tracked
- [ ] Cross-references (spec:XXX) are identified
- [ ] Parser handles malformed content gracefully
- [ ] Line numbers are preserved for error reporting
- [ ] Parser supports incremental updates

## Implementation Details

### Spec Parser Core

```rust
// src/spec/parsing.rs

use std::collections::HashMap;
use std::ops::Range;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Parsed specification document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    /// Document title (from first H1)
    pub title: String,
    /// Extracted metadata
    pub metadata: SpecMetadata,
    /// Sections by heading
    pub sections: HashMap<String, String>,
    /// Ordered list of sections
    pub section_order: Vec<String>,
    /// Acceptance criteria checkboxes
    pub acceptance_criteria: Vec<Checkbox>,
    /// Code blocks
    pub code_blocks: Vec<CodeBlock>,
    /// Cross-references to other specs
    pub references: Vec<SpecReference>,
    /// Parse warnings (non-fatal issues)
    pub warnings: Vec<ParseWarning>,
    /// Source line mappings
    pub line_map: LineMap,
}

/// Spec metadata block
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Phase number
    pub phase: u32,
    /// Phase name
    pub phase_name: Option<String>,
    /// Spec ID
    pub spec_id: u32,
    /// Status (Planned, In Progress, Complete, etc.)
    pub status: String,
    /// Dependencies (spec IDs or file references)
    pub dependencies: Vec<String>,
    /// Estimated context percentage
    pub estimated_context: Option<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, String>,
}

/// Checkbox/task item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkbox {
    /// Checkbox text content
    pub text: String,
    /// Whether checked
    pub checked: bool,
    /// Line number in source
    pub line: usize,
    /// Section containing this checkbox
    pub section: String,
}

/// Code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Language identifier (rust, python, etc.)
    pub language: Option<String>,
    /// Code content
    pub content: String,
    /// Line range in source
    pub lines: Range<usize>,
    /// Section containing this block
    pub section: String,
}

/// Reference to another spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecReference {
    /// Referenced spec ID
    pub spec_id: u32,
    /// Reference format used (spec:XXX, XXX-filename.md, etc.)
    pub format: ReferenceFormat,
    /// Line number
    pub line: usize,
    /// Full matched text
    pub text: String,
}

/// Reference format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceFormat {
    /// spec:116
    SpecColon,
    /// 116-spec-directory.md
    Filename,
    /// [Spec 116](path)
    MarkdownLink,
    /// #116
    HashTag,
}

/// Parse warning (non-fatal issue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    pub message: String,
    pub line: usize,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Line number mapping for source positions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineMap {
    /// Section start lines
    pub section_starts: HashMap<String, usize>,
    /// Metadata block range
    pub metadata_range: Option<Range<usize>>,
    /// Total line count
    pub total_lines: usize,
}

/// Spec document parser
pub struct SpecParser {
    /// Compiled regex patterns
    patterns: ParserPatterns,
}

struct ParserPatterns {
    metadata_field: Regex,
    checkbox: Regex,
    code_fence: Regex,
    heading: Regex,
    spec_ref_colon: Regex,
    spec_ref_filename: Regex,
    spec_ref_link: Regex,
    spec_ref_hash: Regex,
}

impl SpecParser {
    pub fn new() -> Self {
        Self {
            patterns: ParserPatterns {
                metadata_field: Regex::new(
                    r"^\s*[-*]\s*\*\*([^*]+)\*\*:\s*(.+)$"
                ).unwrap(),
                checkbox: Regex::new(
                    r"^\s*[-*]\s*\[([ xX])\]\s*(.+)$"
                ).unwrap(),
                code_fence: Regex::new(
                    r"^```(\w*)$"
                ).unwrap(),
                heading: Regex::new(
                    r"^(#{1,6})\s+(.+)$"
                ).unwrap(),
                spec_ref_colon: Regex::new(
                    r"spec:(\d{3})"
                ).unwrap(),
                spec_ref_filename: Regex::new(
                    r"(\d{3})-[\w-]+\.md"
                ).unwrap(),
                spec_ref_link: Regex::new(
                    r"\[.*?[Ss]pec\s*(\d+).*?\]\([^)]+\)"
                ).unwrap(),
                spec_ref_hash: Regex::new(
                    r"#(\d{3})\b"
                ).unwrap(),
            },
        }
    }

    /// Parse a spec document
    pub fn parse(&self, content: &str) -> Result<ParsedSpec, ParseError> {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let mut parsed = ParsedSpec {
            title: String::new(),
            metadata: SpecMetadata::default(),
            sections: HashMap::new(),
            section_order: Vec::new(),
            acceptance_criteria: Vec::new(),
            code_blocks: Vec::new(),
            references: Vec::new(),
            warnings: Vec::new(),
            line_map: LineMap {
                section_starts: HashMap::new(),
                metadata_range: None,
                total_lines,
            },
        };

        let mut state = ParseState::default();

        for (line_num, line) in lines.iter().enumerate() {
            self.parse_line(&mut parsed, &mut state, line, line_num)?;
        }

        // Finalize last section
        if !state.current_section.is_empty() {
            self.finalize_section(&mut parsed, &state);
        }

        // Finalize code block if still open
        if state.in_code_block {
            parsed.warnings.push(ParseWarning {
                message: "Unclosed code block at end of document".to_string(),
                line: state.code_block_start,
                severity: WarningSeverity::Warning,
            });
        }

        // Extract references from all content
        self.extract_references(&mut parsed, content);

        // Validate required fields
        self.validate(&mut parsed)?;

        Ok(parsed)
    }

    /// Parse a single line
    fn parse_line(
        &self,
        parsed: &mut ParsedSpec,
        state: &mut ParseState,
        line: &str,
        line_num: usize,
    ) -> Result<(), ParseError> {
        // Handle code blocks first (they can contain anything)
        if state.in_code_block {
            if self.patterns.code_fence.is_match(line) && line.trim().starts_with("```") {
                // End of code block
                parsed.code_blocks.push(CodeBlock {
                    language: state.code_block_lang.clone(),
                    content: state.code_block_content.clone(),
                    lines: state.code_block_start..line_num,
                    section: state.current_section.clone(),
                });
                state.in_code_block = false;
                state.code_block_content.clear();
            } else {
                state.code_block_content.push_str(line);
                state.code_block_content.push('\n');
            }
            return Ok(());
        }

        // Check for code block start
        if let Some(caps) = self.patterns.code_fence.captures(line) {
            state.in_code_block = true;
            state.code_block_start = line_num;
            state.code_block_lang = caps.get(1)
                .map(|m| m.as_str().to_string())
                .filter(|s| !s.is_empty());
            return Ok(());
        }

        // Check for headings
        if let Some(caps) = self.patterns.heading.captures(line) {
            let level = caps.get(1).unwrap().as_str().len();
            let text = caps.get(2).unwrap().as_str().to_string();

            if level == 1 && parsed.title.is_empty() {
                // Document title
                parsed.title = text.clone();
                // Try to extract spec ID from title
                if let Some(id) = self.extract_spec_id_from_title(&text) {
                    parsed.metadata.spec_id = id;
                }
            } else if level == 2 {
                // New section
                if !state.current_section.is_empty() {
                    self.finalize_section(parsed, state);
                }
                state.current_section = text.clone();
                state.section_content.clear();
                parsed.section_order.push(text.clone());
                parsed.line_map.section_starts.insert(text, line_num);
            }
            return Ok(());
        }

        // Check for metadata fields
        if state.current_section == "Metadata" ||
           (state.current_section.is_empty() && line_num < 20) {
            if let Some(caps) = self.patterns.metadata_field.captures(line) {
                let field = caps.get(1).unwrap().as_str();
                let value = caps.get(2).unwrap().as_str();
                self.parse_metadata_field(parsed, field, value);

                // Track metadata range
                if parsed.line_map.metadata_range.is_none() {
                    parsed.line_map.metadata_range = Some(line_num..line_num + 1);
                } else if let Some(ref mut range) = parsed.line_map.metadata_range {
                    range.end = line_num + 1;
                }
                return Ok(());
            }
        }

        // Check for checkboxes
        if let Some(caps) = self.patterns.checkbox.captures(line) {
            let checked = caps.get(1).unwrap().as_str() != " ";
            let text = caps.get(2).unwrap().as_str().to_string();

            parsed.acceptance_criteria.push(Checkbox {
                text,
                checked,
                line: line_num,
                section: state.current_section.clone(),
            });
        }

        // Add to current section content
        state.section_content.push_str(line);
        state.section_content.push('\n');

        Ok(())
    }

    /// Finalize current section
    fn finalize_section(&self, parsed: &mut ParsedSpec, state: &ParseState) {
        if !state.current_section.is_empty() {
            parsed.sections.insert(
                state.current_section.clone(),
                state.section_content.trim().to_string(),
            );
        }
    }

    /// Parse a metadata field
    fn parse_metadata_field(&self, parsed: &mut ParsedSpec, field: &str, value: &str) {
        match field.to_lowercase().as_str() {
            "phase" => {
                // Parse "6 - Spec System" format
                let parts: Vec<&str> = value.splitn(2, '-').collect();
                if let Ok(num) = parts[0].trim().parse() {
                    parsed.metadata.phase = num;
                }
                if parts.len() > 1 {
                    parsed.metadata.phase_name = Some(parts[1].trim().to_string());
                }
            }
            "spec id" | "spec-id" | "specid" | "id" => {
                if let Ok(id) = value.trim().parse() {
                    parsed.metadata.spec_id = id;
                }
            }
            "status" => {
                parsed.metadata.status = value.trim().to_string();
            }
            "dependencies" => {
                parsed.metadata.dependencies = value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "estimated context" => {
                parsed.metadata.estimated_context = Some(value.trim().to_string());
            }
            _ => {
                parsed.metadata.custom.insert(
                    field.to_string(),
                    value.to_string(),
                );
            }
        }
    }

    /// Extract spec ID from title
    fn extract_spec_id_from_title(&self, title: &str) -> Option<u32> {
        // "Spec 116: Spec Directory" -> 116
        let re = Regex::new(r"[Ss]pec\s*(\d+)").ok()?;
        re.captures(title)?
            .get(1)?
            .as_str()
            .parse()
            .ok()
    }

    /// Extract all spec references from content
    fn extract_references(&self, parsed: &mut ParsedSpec, content: &str) {
        for (line_num, line) in content.lines().enumerate() {
            // spec:XXX format
            for caps in self.patterns.spec_ref_colon.captures_iter(line) {
                if let Ok(id) = caps.get(1).unwrap().as_str().parse() {
                    parsed.references.push(SpecReference {
                        spec_id: id,
                        format: ReferenceFormat::SpecColon,
                        line: line_num,
                        text: caps.get(0).unwrap().as_str().to_string(),
                    });
                }
            }

            // XXX-filename.md format
            for caps in self.patterns.spec_ref_filename.captures_iter(line) {
                if let Ok(id) = caps.get(1).unwrap().as_str().parse() {
                    parsed.references.push(SpecReference {
                        spec_id: id,
                        format: ReferenceFormat::Filename,
                        line: line_num,
                        text: caps.get(0).unwrap().as_str().to_string(),
                    });
                }
            }
        }
    }

    /// Validate parsed spec
    fn validate(&self, parsed: &mut ParsedSpec) -> Result<(), ParseError> {
        if parsed.title.is_empty() {
            return Err(ParseError::MissingTitle);
        }

        if parsed.metadata.spec_id == 0 {
            parsed.warnings.push(ParseWarning {
                message: "Spec ID not found or is 0".to_string(),
                line: 0,
                severity: WarningSeverity::Warning,
            });
        }

        if parsed.metadata.status.is_empty() {
            parsed.warnings.push(ParseWarning {
                message: "Status not specified".to_string(),
                line: 0,
                severity: WarningSeverity::Info,
            });
            parsed.metadata.status = "Planned".to_string();
        }

        Ok(())
    }
}

/// Parser state
#[derive(Default)]
struct ParseState {
    current_section: String,
    section_content: String,
    in_code_block: bool,
    code_block_start: usize,
    code_block_lang: Option<String>,
    code_block_content: String,
}

/// Parse errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Missing document title")]
    MissingTitle,

    #[error("Invalid metadata at line {0}: {1}")]
    InvalidMetadata(usize, String),

    #[error("Malformed content at line {0}: {1}")]
    MalformedContent(usize, String),
}

impl Default for SpecParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SPEC: &str = r#"# Spec 116: Test Spec

## Metadata
- **Phase**: 6 - Spec System
- **Spec ID**: 116
- **Status**: Planned
- **Dependencies**: 001-project-structure, 044-workspace-discovery
- **Estimated Context**: ~10%

## Objective

This is the objective section.

## Acceptance Criteria

- [ ] First criterion
- [x] Second criterion (complete)
- [ ] Third criterion

## Implementation Details

```rust
fn example() {
    println!("Hello");
}
```

## Related Specs

- **117-spec-templates.md**: Template system
"#;

    #[test]
    fn test_parse_spec() {
        let parser = SpecParser::new();
        let parsed = parser.parse(SAMPLE_SPEC).unwrap();

        assert_eq!(parsed.title, "Spec 116: Test Spec");
        assert_eq!(parsed.metadata.spec_id, 116);
        assert_eq!(parsed.metadata.phase, 6);
        assert_eq!(parsed.metadata.status, "Planned");
    }

    #[test]
    fn test_parse_checkboxes() {
        let parser = SpecParser::new();
        let parsed = parser.parse(SAMPLE_SPEC).unwrap();

        assert_eq!(parsed.acceptance_criteria.len(), 3);
        assert!(!parsed.acceptance_criteria[0].checked);
        assert!(parsed.acceptance_criteria[1].checked);
    }

    #[test]
    fn test_parse_code_blocks() {
        let parser = SpecParser::new();
        let parsed = parser.parse(SAMPLE_SPEC).unwrap();

        assert_eq!(parsed.code_blocks.len(), 1);
        assert_eq!(parsed.code_blocks[0].language, Some("rust".to_string()));
    }

    #[test]
    fn test_parse_references() {
        let parser = SpecParser::new();
        let parsed = parser.parse(SAMPLE_SPEC).unwrap();

        assert!(parsed.references.iter().any(|r| r.spec_id == 117));
    }

    #[test]
    fn test_parse_dependencies() {
        let parser = SpecParser::new();
        let parsed = parser.parse(SAMPLE_SPEC).unwrap();

        assert_eq!(parsed.metadata.dependencies.len(), 2);
        assert!(parsed.metadata.dependencies.contains(&"001-project-structure".to_string()));
    }
}
```

## Testing Requirements

- [ ] Unit tests for metadata parsing
- [ ] Tests for checkbox extraction
- [ ] Tests for code block parsing
- [ ] Tests for reference detection
- [ ] Tests for malformed content handling
- [ ] Tests for line number accuracy
- [ ] Integration tests with real spec files
- [ ] Performance tests for large specs

## Related Specs

- **116-spec-directory.md**: Directory structure for specs
- **121-spec-metadata.md**: Detailed metadata extraction
- **127-spec-validation.md**: Validation using parsed data
