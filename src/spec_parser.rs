//! Spec Parser - LEGACY: Navigates THE PIN (specs/README.md)
//!
//! NOTE: This module is kept for backward compatibility with spec-based projects.
//! For beads-tracked projects, use task_parser.rs instead.
//!
//! Parses the master lookup table to find specs and their completion status.

use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

/// A spec entry from the README lookup table
#[derive(Debug, Clone)]
pub struct SpecEntry {
    pub id: u32,
    pub name: String,
    pub path: PathBuf,
    pub keywords: Vec<String>,
    pub phase: u32,
    pub phase_name: String,
}

/// Acceptance criteria item from a spec file
#[derive(Debug, Clone)]
pub struct AcceptanceCriteria {
    pub text: String,
    pub completed: bool,
    pub line_number: usize,
}

/// A parsed spec with its acceptance criteria
#[derive(Debug, Clone)]
pub struct ParsedSpec {
    pub entry: SpecEntry,
    pub content: String,
    pub acceptance_criteria: Vec<AcceptanceCriteria>,
    pub all_complete: bool,
}

/// Parse the specs/README.md to extract all spec entries
pub fn parse_readme(specs_dir: &Path) -> Result<Vec<SpecEntry>> {
    let readme_path = specs_dir.join("README.md");
    let content = std::fs::read_to_string(&readme_path)
        .with_context(|| format!("Failed to read {}", readme_path.display()))?;

    let mut entries = Vec::new();

    // Match table rows: | 001 | [Project Structure](phase-00-setup/001-project-structure.md) | init, scaffold |
    let row_re = Regex::new(
        r"\|\s*(\d{3})\s*\|\s*\[([^\]]+)\]\(([^)]+)\)\s*\|\s*([^|]+)\s*\|"
    )?;

    // Track current phase
    let phase_re = Regex::new(r"##\s*Phase\s*(\d+):\s*(.+?)\s*\(")?;
    let mut current_phase = 0u32;
    let mut current_phase_name = String::new();

    for line in content.lines() {
        // Check for phase header
        if let Some(caps) = phase_re.captures(line) {
            current_phase = caps[1].parse().unwrap_or(0);
            current_phase_name = caps[2].trim().to_string();
        }

        // Check for spec row
        if let Some(caps) = row_re.captures(line) {
            let id: u32 = caps[1].parse().unwrap_or(0);
            let name = caps[2].to_string();
            let relative_path = caps[3].to_string();
            let keywords: Vec<String> = caps[4]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let path = specs_dir.join(&relative_path);

            entries.push(SpecEntry {
                id,
                name,
                path,
                keywords,
                phase: current_phase,
                phase_name: current_phase_name.clone(),
            });
        }
    }

    Ok(entries)
}

/// Parse a spec file to extract acceptance criteria
pub fn parse_spec(entry: &SpecEntry) -> Result<ParsedSpec> {
    let content = std::fs::read_to_string(&entry.path)
        .with_context(|| format!("Failed to read spec {}", entry.path.display()))?;

    let mut acceptance_criteria = Vec::new();
    let mut in_acceptance_section = false;

    // Match checkbox lines: - [ ] or - [x]
    let checkbox_re = Regex::new(r"^\s*-\s*\[([ xX])\]\s*(.+)$")?;

    for (idx, line) in content.lines().enumerate() {
        // Check if we're entering the acceptance criteria section
        if line.contains("## Acceptance Criteria") {
            in_acceptance_section = true;
            continue;
        }

        // Check if we're leaving (next section)
        if in_acceptance_section && line.starts_with("## ") && !line.contains("Acceptance") {
            in_acceptance_section = false;
        }

        // Parse checkboxes in acceptance section
        if in_acceptance_section {
            if let Some(caps) = checkbox_re.captures(line) {
                let completed = caps[1].to_lowercase() == "x";
                let text = caps[2].trim().to_string();

                acceptance_criteria.push(AcceptanceCriteria {
                    text,
                    completed,
                    line_number: idx + 1, // 1-indexed
                });
            }
        }
    }

    let all_complete = !acceptance_criteria.is_empty()
        && acceptance_criteria.iter().all(|ac| ac.completed);

    Ok(ParsedSpec {
        entry: entry.clone(),
        content,
        acceptance_criteria,
        all_complete,
    })
}

/// Find the next uncompleted spec in order
pub fn find_next_spec(specs_dir: &Path) -> Result<Option<ParsedSpec>> {
    let entries = parse_readme(specs_dir)?;

    for entry in entries {
        // Check if spec file exists
        if !entry.path.exists() {
            tracing::warn!("Spec file not found: {}", entry.path.display());
            continue;
        }

        let parsed = parse_spec(&entry)?;

        // Return first incomplete spec
        if !parsed.all_complete {
            return Ok(Some(parsed));
        }
    }

    Ok(None)
}

/// Update a checkbox in a spec file
pub fn update_checkbox(spec_path: &Path, line_number: usize, completed: bool) -> Result<()> {
    let content = std::fs::read_to_string(spec_path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if line_number == 0 || line_number > lines.len() {
        anyhow::bail!("Invalid line number: {}", line_number);
    }

    let line = &mut lines[line_number - 1];
    let checkbox_re = Regex::new(r"^(\s*-\s*\[)[ xX](\]\s*.+)$")?;

    if let Some(caps) = checkbox_re.captures(line) {
        let new_mark = if completed { "x" } else { " " };
        *line = format!("{}{}{}", &caps[1], new_mark, &caps[2]);
    }

    std::fs::write(spec_path, lines.join("\n") + "\n")?;
    Ok(())
}

/// Get progress summary for all specs
pub fn get_progress_summary(specs_dir: &Path) -> Result<ProgressSummary> {
    let entries = parse_readme(specs_dir)?;
    let mut total_specs = 0;
    let mut completed_specs = 0;
    let mut total_criteria = 0;
    let mut completed_criteria = 0;

    for entry in &entries {
        if !entry.path.exists() {
            continue;
        }

        if let Ok(parsed) = parse_spec(entry) {
            total_specs += 1;
            if parsed.all_complete {
                completed_specs += 1;
            }

            for ac in &parsed.acceptance_criteria {
                total_criteria += 1;
                if ac.completed {
                    completed_criteria += 1;
                }
            }
        }
    }

    Ok(ProgressSummary {
        total_specs,
        completed_specs,
        total_criteria,
        completed_criteria,
    })
}

#[derive(Debug)]
pub struct ProgressSummary {
    pub total_specs: usize,
    pub completed_specs: usize,
    pub total_criteria: usize,
    pub completed_criteria: usize,
}

impl ProgressSummary {
    pub fn spec_percentage(&self) -> f64 {
        if self.total_specs == 0 {
            0.0
        } else {
            (self.completed_specs as f64 / self.total_specs as f64) * 100.0
        }
    }

    pub fn criteria_percentage(&self) -> f64 {
        if self.total_criteria == 0 {
            0.0
        } else {
            (self.completed_criteria as f64 / self.total_criteria as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_regex() {
        let re = Regex::new(r"^\s*-\s*\[([ xX])\]\s*(.+)$").unwrap();

        let line1 = "- [ ] Root directory contains all required folders";
        let caps1 = re.captures(line1).unwrap();
        assert_eq!(&caps1[1], " ");
        assert_eq!(&caps1[2], "Root directory contains all required folders");

        let line2 = "- [x] Something completed";
        let caps2 = re.captures(line2).unwrap();
        assert_eq!(&caps2[1], "x");
    }
}
