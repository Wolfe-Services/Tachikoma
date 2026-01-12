# Spec 118: README.md Lookup Format

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 118
- **Status**: Planned
- **Dependencies**: 116-spec-directory, 117-spec-templates
- **Estimated Context**: ~8%

## Objective

Define the standard format and lookup mechanism for README.md files within the spec system. READMEs serve as navigation hubs for phases, providing summaries, spec listings, progress indicators, and quick reference information for both humans and AI assistants.

## Acceptance Criteria

- [ ] README format is standardized across all phases
- [ ] Spec listings include ID, title, status, and progress
- [ ] Progress indicators show completion percentage
- [ ] Quick navigation links work correctly
- [ ] README lookup is fast and cacheable
- [ ] READMEs support embedded metadata
- [ ] Cross-references to other phases work
- [ ] README validation detects format issues

## Implementation Details

### README Structure and Lookup

```rust
// src/spec/readme.rs

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::fs;

/// Standard README sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadmeSection {
    Header,
    Overview,
    SpecList,
    Progress,
    Dependencies,
    QuickReference,
    Notes,
}

impl ReadmeSection {
    pub fn heading(&self) -> &'static str {
        match self {
            Self::Header => "",
            Self::Overview => "## Overview",
            Self::SpecList => "## Specifications",
            Self::Progress => "## Progress",
            Self::Dependencies => "## Dependencies",
            Self::QuickReference => "## Quick Reference",
            Self::Notes => "## Notes",
        }
    }
}

/// Parsed README content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeContent {
    /// Phase number
    pub phase: u32,
    /// Phase title
    pub title: String,
    /// Phase description/overview
    pub overview: String,
    /// List of specs in this phase
    pub specs: Vec<ReadmeSpecEntry>,
    /// Overall progress
    pub progress: PhaseProgress,
    /// Dependencies on other phases
    pub dependencies: Vec<u32>,
    /// Quick reference items
    pub quick_reference: Vec<QuickRefItem>,
    /// Additional notes
    pub notes: Option<String>,
    /// Raw markdown content
    pub raw: String,
    /// File path
    pub path: PathBuf,
}

/// Spec entry in README
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeSpecEntry {
    /// Spec ID
    pub id: u32,
    /// Spec title
    pub title: String,
    /// Spec status
    pub status: SpecStatus,
    /// Completion percentage
    pub completion: u8,
    /// Brief description
    pub description: Option<String>,
    /// Link to spec file
    pub link: String,
}

/// Spec status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecStatus {
    Planned,
    InProgress,
    Review,
    Complete,
    Blocked,
}

impl SpecStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "in progress" | "inprogress" | "in-progress" => Self::InProgress,
            "review" | "in review" => Self::Review,
            "complete" | "completed" | "done" => Self::Complete,
            "blocked" => Self::Blocked,
            _ => Self::Planned,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Planned => "📋",
            Self::InProgress => "🔄",
            Self::Review => "👀",
            Self::Complete => "✅",
            Self::Blocked => "🚫",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "In Progress",
            Self::Review => "Review",
            Self::Complete => "Complete",
            Self::Blocked => "Blocked",
        }
    }
}

/// Phase progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseProgress {
    /// Total specs in phase
    pub total: u32,
    /// Completed specs
    pub completed: u32,
    /// In progress specs
    pub in_progress: u32,
    /// Overall percentage
    pub percentage: u8,
}

/// Quick reference item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRefItem {
    pub label: String,
    pub value: String,
    pub link: Option<String>,
}

/// README lookup and management
pub struct ReadmeLookup {
    /// Cache of parsed READMEs
    cache: HashMap<u32, ReadmeContent>,
    /// Specs root directory
    specs_root: PathBuf,
}

impl ReadmeLookup {
    /// Create a new README lookup
    pub fn new(specs_root: PathBuf) -> Self {
        Self {
            cache: HashMap::new(),
            specs_root,
        }
    }

    /// Get README for a phase
    pub async fn get(&mut self, phase: u32) -> Result<&ReadmeContent, ReadmeError> {
        if !self.cache.contains_key(&phase) {
            let content = self.load_phase_readme(phase).await?;
            self.cache.insert(phase, content);
        }

        Ok(self.cache.get(&phase).unwrap())
    }

    /// Load and parse phase README
    async fn load_phase_readme(&self, phase: u32) -> Result<ReadmeContent, ReadmeError> {
        let phase_dir = self.specs_root.join(format!("phase-{:02}-specs", phase));
        let readme_path = phase_dir.join("README.md");

        if !readme_path.exists() {
            return Err(ReadmeError::NotFound(readme_path));
        }

        let content = fs::read_to_string(&readme_path).await?;
        self.parse_readme(&content, phase, readme_path)
    }

    /// Parse README content
    fn parse_readme(
        &self,
        content: &str,
        phase: u32,
        path: PathBuf,
    ) -> Result<ReadmeContent, ReadmeError> {
        let mut title = String::new();
        let mut overview = String::new();
        let mut specs = Vec::new();
        let mut dependencies = Vec::new();
        let mut quick_reference = Vec::new();
        let mut notes = None;

        let mut current_section = ReadmeSection::Header;
        let mut section_content = String::new();

        for line in content.lines() {
            // Check for section headers
            if line.starts_with("# ") && current_section == ReadmeSection::Header {
                title = line[2..].trim().to_string();
                continue;
            }

            if let Some(section) = self.detect_section(line) {
                // Process previous section
                self.process_section(
                    current_section,
                    &section_content,
                    &mut overview,
                    &mut specs,
                    &mut dependencies,
                    &mut quick_reference,
                    &mut notes,
                );

                current_section = section;
                section_content.clear();
                continue;
            }

            section_content.push_str(line);
            section_content.push('\n');
        }

        // Process final section
        self.process_section(
            current_section,
            &section_content,
            &mut overview,
            &mut specs,
            &mut dependencies,
            &mut quick_reference,
            &mut notes,
        );

        // Calculate progress
        let progress = self.calculate_progress(&specs);

        Ok(ReadmeContent {
            phase,
            title,
            overview: overview.trim().to_string(),
            specs,
            progress,
            dependencies,
            quick_reference,
            notes,
            raw: content.to_string(),
            path,
        })
    }

    /// Detect section from heading
    fn detect_section(&self, line: &str) -> Option<ReadmeSection> {
        if !line.starts_with("## ") {
            return None;
        }

        let heading = line[3..].trim().to_lowercase();

        match heading.as_str() {
            "overview" | "description" => Some(ReadmeSection::Overview),
            "specifications" | "specs" | "spec list" => Some(ReadmeSection::SpecList),
            "progress" | "status" => Some(ReadmeSection::Progress),
            "dependencies" => Some(ReadmeSection::Dependencies),
            "quick reference" | "reference" => Some(ReadmeSection::QuickReference),
            "notes" | "additional notes" => Some(ReadmeSection::Notes),
            _ => None,
        }
    }

    /// Process section content
    fn process_section(
        &self,
        section: ReadmeSection,
        content: &str,
        overview: &mut String,
        specs: &mut Vec<ReadmeSpecEntry>,
        dependencies: &mut Vec<u32>,
        quick_reference: &mut Vec<QuickRefItem>,
        notes: &mut Option<String>,
    ) {
        match section {
            ReadmeSection::Overview => {
                *overview = content.to_string();
            }
            ReadmeSection::SpecList => {
                specs.extend(self.parse_spec_list(content));
            }
            ReadmeSection::Dependencies => {
                dependencies.extend(self.parse_dependencies(content));
            }
            ReadmeSection::QuickReference => {
                quick_reference.extend(self.parse_quick_reference(content));
            }
            ReadmeSection::Notes => {
                *notes = Some(content.trim().to_string());
            }
            _ => {}
        }
    }

    /// Parse spec list from table or list format
    fn parse_spec_list(&self, content: &str) -> Vec<ReadmeSpecEntry> {
        let mut specs = Vec::new();

        for line in content.lines() {
            // Table row format: | 116 | Spec Directory | Planned | 0% |
            if line.starts_with('|') && !line.contains("---") {
                if let Some(spec) = self.parse_table_row(line) {
                    specs.push(spec);
                }
            }
            // List format: - [116-spec-directory.md](./116-spec-directory.md) - Planned
            else if line.starts_with("- [") || line.starts_with("* [") {
                if let Some(spec) = self.parse_list_entry(line) {
                    specs.push(spec);
                }
            }
        }

        specs
    }

    /// Parse table row format
    fn parse_table_row(&self, line: &str) -> Option<ReadmeSpecEntry> {
        let parts: Vec<&str> = line.split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() >= 3 {
            let id: u32 = parts[0].parse().ok()?;
            let title = parts[1].to_string();
            let status = SpecStatus::from_str(parts.get(2).unwrap_or(&"Planned"));
            let completion: u8 = parts.get(3)
                .and_then(|s| s.trim_end_matches('%').parse().ok())
                .unwrap_or(0);

            Some(ReadmeSpecEntry {
                id,
                title,
                status,
                completion,
                description: parts.get(4).map(|s| s.to_string()),
                link: format!("./{:03}-*.md", id),
            })
        } else {
            None
        }
    }

    /// Parse list entry format
    fn parse_list_entry(&self, line: &str) -> Option<ReadmeSpecEntry> {
        let re = regex::Regex::new(
            r"^\s*[-*]\s*\[(\d+)-([^\]]+)\.md\]\(([^)]+)\)\s*[-:]*\s*(\w+)?"
        ).ok()?;

        if let Some(caps) = re.captures(line) {
            let id: u32 = caps.get(1)?.as_str().parse().ok()?;
            let slug = caps.get(2)?.as_str();
            let link = caps.get(3)?.as_str().to_string();
            let status = caps.get(4)
                .map(|m| SpecStatus::from_str(m.as_str()))
                .unwrap_or(SpecStatus::Planned);

            Some(ReadmeSpecEntry {
                id,
                title: slug.replace('-', " "),
                status,
                completion: if status == SpecStatus::Complete { 100 } else { 0 },
                description: None,
                link,
            })
        } else {
            None
        }
    }

    /// Parse dependencies section
    fn parse_dependencies(&self, content: &str) -> Vec<u32> {
        let mut deps = Vec::new();
        let re = regex::Regex::new(r"[Pp]hase\s*(\d+)").unwrap();

        for cap in re.captures_iter(content) {
            if let Ok(phase) = cap[1].parse() {
                deps.push(phase);
            }
        }

        deps
    }

    /// Parse quick reference section
    fn parse_quick_reference(&self, content: &str) -> Vec<QuickRefItem> {
        let mut items = Vec::new();

        for line in content.lines() {
            if line.starts_with("- **") || line.starts_with("* **") {
                if let Some((label, value)) = line.split_once("**:") {
                    items.push(QuickRefItem {
                        label: label.trim_start_matches("- **")
                            .trim_start_matches("* **")
                            .to_string(),
                        value: value.trim().to_string(),
                        link: None,
                    });
                }
            }
        }

        items
    }

    /// Calculate progress from specs
    fn calculate_progress(&self, specs: &[ReadmeSpecEntry]) -> PhaseProgress {
        let total = specs.len() as u32;
        let completed = specs.iter()
            .filter(|s| s.status == SpecStatus::Complete)
            .count() as u32;
        let in_progress = specs.iter()
            .filter(|s| s.status == SpecStatus::InProgress)
            .count() as u32;

        let percentage = if total > 0 {
            ((completed as f32 / total as f32) * 100.0) as u8
        } else {
            0
        };

        PhaseProgress {
            total,
            completed,
            in_progress,
            percentage,
        }
    }

    /// Invalidate cache for a phase
    pub fn invalidate(&mut self, phase: u32) {
        self.cache.remove(&phase);
    }

    /// Clear all cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Find spec across all cached READMEs
    pub fn find_spec(&self, spec_id: u32) -> Option<&ReadmeSpecEntry> {
        for readme in self.cache.values() {
            if let Some(spec) = readme.specs.iter().find(|s| s.id == spec_id) {
                return Some(spec);
            }
        }
        None
    }
}

/// Standard README template
pub fn readme_template(phase: u32, phase_name: &str) -> String {
    format!(r#"# Phase {phase}: {phase_name}

## Overview

[Phase description]

## Specifications

| ID | Title | Status | Progress | Description |
|----|-------|--------|----------|-------------|
| 000 | Example Spec | Planned | 0% | Example description |

## Progress

- Total Specs: 0
- Completed: 0
- In Progress: 0
- Progress: 0%

## Dependencies

- Phase X: [dependency description]

## Quick Reference

- **Key Concept**: Description
- **Main Components**: Component list

## Notes

Additional notes for this phase.
"#)
}

/// Errors for README operations
#[derive(Debug, thiserror::Error)]
pub enum ReadmeError {
    #[error("README not found: {0}")]
    NotFound(PathBuf),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_status_parsing() {
        assert_eq!(SpecStatus::from_str("In Progress"), SpecStatus::InProgress);
        assert_eq!(SpecStatus::from_str("complete"), SpecStatus::Complete);
        assert_eq!(SpecStatus::from_str("unknown"), SpecStatus::Planned);
    }

    #[test]
    fn test_progress_calculation() {
        let lookup = ReadmeLookup::new(PathBuf::new());

        let specs = vec![
            ReadmeSpecEntry {
                id: 1,
                title: "Spec 1".to_string(),
                status: SpecStatus::Complete,
                completion: 100,
                description: None,
                link: String::new(),
            },
            ReadmeSpecEntry {
                id: 2,
                title: "Spec 2".to_string(),
                status: SpecStatus::InProgress,
                completion: 50,
                description: None,
                link: String::new(),
            },
        ];

        let progress = lookup.calculate_progress(&specs);
        assert_eq!(progress.total, 2);
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.percentage, 50);
    }
}
```

## Testing Requirements

- [ ] Unit tests for README parsing
- [ ] Tests for table row parsing
- [ ] Tests for list entry parsing
- [ ] Tests for progress calculation
- [ ] Tests for section detection
- [ ] Integration tests for README lookup
- [ ] Tests for cache invalidation
- [ ] Tests for cross-phase spec finding

## Related Specs

- **116-spec-directory.md**: Directory structure containing READMEs
- **119-readme-autogen.md**: Auto-generation of READMEs
- **124-progress-calc.md**: Detailed progress calculation
