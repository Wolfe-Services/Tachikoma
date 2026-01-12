# Spec 119: README Auto-Generation

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 119
- **Status**: Planned
- **Dependencies**: 116-spec-directory, 117-spec-templates, 118-readme-lookup
- **Estimated Context**: ~9%

## Objective

Implement automatic generation and updating of README.md files for spec phases. The auto-generation system scans spec files, extracts metadata, calculates progress, and produces formatted README documents that stay synchronized with the actual spec content.

## Acceptance Criteria

- [x] READMEs are auto-generated from spec file analysis
- [x] Spec metadata is extracted and formatted correctly
- [x] Progress indicators are calculated accurately
- [x] READMEs update when specs change
- [x] Manual sections are preserved during regeneration
- [x] Generation is triggered by file watchers
- [x] Batch regeneration is supported
- [x] Generation errors are reported clearly

## Implementation Details

### Auto-Generation Engine

```rust
// src/spec/autogen.rs

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use chrono::{DateTime, Utc};

use crate::spec::directory::{SpecDirectory, SpecFileInfo, PhaseDirectory};
use crate::spec::parsing::{SpecParser, ParsedSpec};
use crate::spec::readme::{SpecStatus, PhaseProgress};

/// Auto-generation configuration
#[derive(Debug, Clone)]
pub struct AutogenConfig {
    /// Preserve manual sections
    pub preserve_manual: bool,
    /// Include progress bars
    pub include_progress_bar: bool,
    /// Include dependency graph
    pub include_dependencies: bool,
    /// Include quick reference
    pub include_quick_ref: bool,
    /// Section markers for preservation
    pub manual_section_marker: String,
}

impl Default for AutogenConfig {
    fn default() -> Self {
        Self {
            preserve_manual: true,
            include_progress_bar: true,
            include_dependencies: true,
            include_quick_ref: true,
            manual_section_marker: "<!-- MANUAL -->".to_string(),
        }
    }
}

/// README auto-generator
pub struct ReadmeAutogen {
    config: AutogenConfig,
    parser: SpecParser,
}

/// Generated README content
#[derive(Debug, Clone)]
pub struct GeneratedReadme {
    pub phase: u32,
    pub content: String,
    pub specs_analyzed: usize,
    pub generated_at: DateTime<Utc>,
}

/// Spec summary for README
#[derive(Debug, Clone)]
pub struct SpecSummary {
    pub id: u32,
    pub title: String,
    pub status: SpecStatus,
    pub completion: u8,
    pub description: String,
    pub dependencies: Vec<String>,
    pub estimated_context: String,
}

impl ReadmeAutogen {
    pub fn new(config: AutogenConfig) -> Self {
        Self {
            config,
            parser: SpecParser::new(),
        }
    }

    /// Generate README for a phase
    pub async fn generate_phase_readme(
        &self,
        phase: &PhaseDirectory,
    ) -> Result<GeneratedReadme, AutogenError> {
        // Parse all specs in the phase
        let mut summaries = Vec::new();

        for spec_info in &phase.specs {
            match self.parse_spec_summary(&spec_info.path).await {
                Ok(summary) => summaries.push(summary),
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", spec_info.path.display(), e);
                }
            }
        }

        // Sort by spec ID
        summaries.sort_by_key(|s| s.id);

        // Calculate progress
        let progress = self.calculate_progress(&summaries);

        // Load existing README for manual section preservation
        let existing_manual = if self.config.preserve_manual {
            self.extract_manual_sections(&phase.path.join("README.md")).await
        } else {
            HashMap::new()
        };

        // Generate content
        let content = self.render_readme(phase, &summaries, &progress, &existing_manual);

        Ok(GeneratedReadme {
            phase: phase.number,
            content,
            specs_analyzed: summaries.len(),
            generated_at: Utc::now(),
        })
    }

    /// Parse a spec file into a summary
    async fn parse_spec_summary(&self, path: &Path) -> Result<SpecSummary, AutogenError> {
        let content = fs::read_to_string(path).await?;
        let parsed = self.parser.parse(&content)?;

        Ok(SpecSummary {
            id: parsed.metadata.spec_id,
            title: parsed.title.clone(),
            status: self.parse_status(&parsed.metadata.status),
            completion: self.calculate_spec_completion(&parsed),
            description: self.extract_description(&parsed),
            dependencies: parsed.metadata.dependencies.clone(),
            estimated_context: parsed.metadata.estimated_context.clone()
                .unwrap_or_else(|| "~10%".to_string()),
        })
    }

    /// Parse status string to enum
    fn parse_status(&self, status: &str) -> SpecStatus {
        SpecStatus::from_str(status)
    }

    /// Calculate completion from acceptance criteria
    fn calculate_spec_completion(&self, spec: &ParsedSpec) -> u8 {
        let total = spec.acceptance_criteria.len();
        if total == 0 {
            return 0;
        }

        let completed = spec.acceptance_criteria.iter()
            .filter(|c| c.checked)
            .count();

        ((completed as f32 / total as f32) * 100.0) as u8
    }

    /// Extract first paragraph as description
    fn extract_description(&self, spec: &ParsedSpec) -> String {
        if let Some(objective) = spec.sections.get("Objective") {
            let first_para = objective.split("\n\n").next().unwrap_or("");
            let truncated = if first_para.len() > 150 {
                format!("{}...", &first_para[..147])
            } else {
                first_para.to_string()
            };
            truncated.replace('\n', " ")
        } else {
            String::new()
        }
    }

    /// Calculate phase progress
    fn calculate_progress(&self, summaries: &[SpecSummary]) -> PhaseProgress {
        let total = summaries.len() as u32;
        let completed = summaries.iter()
            .filter(|s| s.status == SpecStatus::Complete)
            .count() as u32;
        let in_progress = summaries.iter()
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

    /// Extract manual sections from existing README
    async fn extract_manual_sections(&self, path: &Path) -> HashMap<String, String> {
        let mut sections = HashMap::new();

        if let Ok(content) = fs::read_to_string(path).await {
            let marker = &self.config.manual_section_marker;
            let mut in_manual = false;
            let mut section_name = String::new();
            let mut section_content = String::new();

            for line in content.lines() {
                if line.contains(marker) {
                    if in_manual {
                        // End of manual section
                        sections.insert(section_name.clone(), section_content.clone());
                        section_content.clear();
                        in_manual = false;
                    } else {
                        // Start of manual section
                        in_manual = true;
                        section_name = line.replace(marker, "").trim().to_string();
                    }
                } else if in_manual {
                    section_content.push_str(line);
                    section_content.push('\n');
                }
            }
        }

        sections
    }

    /// Render the README content
    fn render_readme(
        &self,
        phase: &PhaseDirectory,
        summaries: &[SpecSummary],
        progress: &PhaseProgress,
        manual_sections: &HashMap<String, String>,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("# Phase {}: {}\n\n", phase.number, phase.name));

        // Auto-generation notice
        output.push_str("<!-- AUTO-GENERATED README - DO NOT EDIT DIRECTLY -->\n");
        output.push_str(&format!("<!-- Generated at: {} -->\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Overview section (manual if exists)
        output.push_str("## Overview\n\n");
        if let Some(manual) = manual_sections.get("Overview") {
            output.push_str(manual);
        } else {
            output.push_str(&self.default_overview(phase));
        }
        output.push('\n');

        // Progress section
        if self.config.include_progress_bar {
            output.push_str("## Progress\n\n");
            output.push_str(&self.render_progress(progress));
            output.push('\n');
        }

        // Specifications table
        output.push_str("## Specifications\n\n");
        output.push_str(&self.render_spec_table(summaries));
        output.push('\n');

        // Dependencies section
        if self.config.include_dependencies {
            output.push_str("## Dependencies\n\n");
            output.push_str(&self.render_dependencies(summaries));
            output.push('\n');
        }

        // Quick reference
        if self.config.include_quick_ref {
            output.push_str("## Quick Reference\n\n");
            if let Some(manual) = manual_sections.get("Quick Reference") {
                output.push_str(manual);
            } else {
                output.push_str(&self.default_quick_reference(phase, summaries));
            }
            output.push('\n');
        }

        // Notes section (always manual)
        output.push_str("## Notes\n\n");
        output.push_str(&format!("{} Notes\n", self.config.manual_section_marker));
        if let Some(manual) = manual_sections.get("Notes") {
            output.push_str(manual);
        } else {
            output.push_str("Add manual notes here.\n");
        }
        output.push_str(&format!("{}\n", self.config.manual_section_marker));

        output
    }

    /// Default overview text
    fn default_overview(&self, phase: &PhaseDirectory) -> String {
        format!(
            "Phase {} contains {} specifications focused on {}.\n",
            phase.number,
            phase.specs.len(),
            phase.name.to_lowercase()
        )
    }

    /// Render progress section
    fn render_progress(&self, progress: &PhaseProgress) -> String {
        let bar_width = 20;
        let filled = (progress.percentage as usize * bar_width) / 100;
        let empty = bar_width - filled;

        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        format!(
            r#"{}  **{}%**

| Metric | Count |
|--------|-------|
| Total Specs | {} |
| Completed | {} |
| In Progress | {} |
| Remaining | {} |
"#,
            bar,
            progress.percentage,
            progress.total,
            progress.completed,
            progress.in_progress,
            progress.total - progress.completed - progress.in_progress
        )
    }

    /// Render specifications table
    fn render_spec_table(&self, summaries: &[SpecSummary]) -> String {
        let mut table = String::new();

        table.push_str("| ID | Title | Status | Progress | Context |\n");
        table.push_str("|----|-------|--------|----------|----------|\n");

        for spec in summaries {
            table.push_str(&format!(
                "| [{:03}](./{:03}-*.md) | {} | {} {} | {}% | {} |\n",
                spec.id,
                spec.id,
                spec.title,
                spec.status.emoji(),
                spec.status.as_str(),
                spec.completion,
                spec.estimated_context,
            ));
        }

        table
    }

    /// Render dependencies section
    fn render_dependencies(&self, summaries: &[SpecSummary]) -> String {
        let mut all_deps: Vec<&String> = summaries.iter()
            .flat_map(|s| &s.dependencies)
            .collect();
        all_deps.sort();
        all_deps.dedup();

        if all_deps.is_empty() {
            return "No external dependencies.\n".to_string();
        }

        let mut output = String::new();
        for dep in all_deps {
            output.push_str(&format!("- {}\n", dep));
        }
        output
    }

    /// Default quick reference
    fn default_quick_reference(&self, phase: &PhaseDirectory, summaries: &[SpecSummary]) -> String {
        let id_range = if let (Some(first), Some(last)) = (
            summaries.first().map(|s| s.id),
            summaries.last().map(|s| s.id),
        ) {
            format!("{:03}-{:03}", first, last)
        } else {
            "N/A".to_string()
        };

        format!(
            r#"- **Phase**: {}
- **Name**: {}
- **Spec Range**: {}
- **Spec Count**: {}
"#,
            phase.number,
            phase.name,
            id_range,
            summaries.len()
        )
    }

    /// Write generated README to file
    pub async fn write_readme(
        &self,
        readme: &GeneratedReadme,
        phase_dir: &Path,
    ) -> Result<PathBuf, AutogenError> {
        let path = phase_dir.join("README.md");
        fs::write(&path, &readme.content).await?;
        Ok(path)
    }

    /// Generate all phase READMEs
    pub async fn generate_all(
        &self,
        spec_dir: &SpecDirectory,
    ) -> Result<Vec<GeneratedReadme>, AutogenError> {
        let mut readmes = Vec::new();

        for phase in &spec_dir.phases {
            let readme = self.generate_phase_readme(phase).await?;
            readmes.push(readme);
        }

        Ok(readmes)
    }
}

/// Auto-generation watcher for continuous updates
pub struct AutogenWatcher {
    autogen: ReadmeAutogen,
    spec_dir: SpecDirectory,
}

impl AutogenWatcher {
    pub fn new(autogen: ReadmeAutogen, spec_dir: SpecDirectory) -> Self {
        Self { autogen, spec_dir }
    }

    /// Handle spec file change
    pub async fn on_spec_changed(&self, spec_path: &Path) -> Result<(), AutogenError> {
        // Find which phase this spec belongs to
        for phase in &self.spec_dir.phases {
            if spec_path.starts_with(&phase.path) {
                let readme = self.autogen.generate_phase_readme(phase).await?;
                self.autogen.write_readme(&readme, &phase.path).await?;
                break;
            }
        }
        Ok(())
    }
}

/// Errors for auto-generation
#[derive(Debug, thiserror::Error)]
pub enum AutogenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] crate::spec::parsing::ParseError),

    #[error("Invalid spec: {0}")]
    InvalidSpec(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_rendering() {
        let autogen = ReadmeAutogen::new(AutogenConfig::default());

        let progress = PhaseProgress {
            total: 20,
            completed: 10,
            in_progress: 5,
            percentage: 50,
        };

        let rendered = autogen.render_progress(&progress);
        assert!(rendered.contains("50%"));
        assert!(rendered.contains("20"));
        assert!(rendered.contains("10"));
    }

    #[test]
    fn test_spec_table_rendering() {
        let autogen = ReadmeAutogen::new(AutogenConfig::default());

        let summaries = vec![
            SpecSummary {
                id: 116,
                title: "Spec Directory".to_string(),
                status: SpecStatus::Complete,
                completion: 100,
                description: "Directory structure".to_string(),
                dependencies: vec![],
                estimated_context: "~10%".to_string(),
            },
        ];

        let table = autogen.render_spec_table(&summaries);
        assert!(table.contains("116"));
        assert!(table.contains("Spec Directory"));
        assert!(table.contains("100%"));
    }
}
```

## Testing Requirements

- [ ] Unit tests for spec summary extraction
- [ ] Tests for progress calculation
- [ ] Tests for table rendering
- [ ] Tests for manual section preservation
- [ ] Tests for dependency extraction
- [ ] Integration tests for full README generation
- [ ] Tests for file watching triggers
- [ ] Tests for batch regeneration

## Related Specs

- **116-spec-directory.md**: Directory structure for spec organization
- **118-readme-lookup.md**: README parsing format this generates
- **120-spec-parsing.md**: Spec parsing for metadata extraction
- **124-progress-calc.md**: Detailed progress calculation
