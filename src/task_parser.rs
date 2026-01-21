//! Task Parser - Integrates with Beads issue tracker
//!
//! Parses issues from `bd ready` or `.beads/issues.jsonl` to find work.
//! Extracts acceptance criteria from issue descriptions.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// A beads issue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadTask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub notes: String,
    pub status: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub issue_type: String,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub blocks: Vec<String>,
}

/// Acceptance criteria item from an issue description
#[derive(Debug, Clone)]
pub struct AcceptanceCriteria {
    pub text: String,
    pub completed: bool,
    pub line_number: usize,
}

/// A parsed task with its acceptance criteria
#[derive(Debug, Clone)]
pub struct ParsedTask {
    pub task: BeadTask,
    pub acceptance_criteria: Vec<AcceptanceCriteria>,
    pub all_complete: bool,
}

/// Get all ready (unblocked) tasks from beads
pub fn get_ready_tasks(project_root: &Path) -> Result<Vec<BeadTask>> {
    let output = Command::new("bd")
        .args(["ready", "--json"])
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd ready'. Is beads installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bd ready failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse JSON array of tasks
    let tasks: Vec<BeadTask> = serde_json::from_str(&stdout)
        .context("Failed to parse bd ready JSON output")?;

    Ok(tasks)
}

/// Get all open tasks from beads
pub fn get_all_open_tasks(project_root: &Path) -> Result<Vec<BeadTask>> {
    let output = Command::new("bd")
        .args(["list", "--status=open", "--json"])
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd list'. Is beads installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bd list failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let tasks: Vec<BeadTask> = serde_json::from_str(&stdout)
        .context("Failed to parse bd list JSON output")?;

    Ok(tasks)
}

/// Get a specific task by ID
pub fn get_task(project_root: &Path, task_id: &str) -> Result<BeadTask> {
    let output = Command::new("bd")
        .args(["show", task_id, "--json"])
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd show'. Is beads installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bd show {} failed: {}", task_id, stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // bd show --json returns an array with the task and its dependencies
    // The first element is always the requested task
    let tasks: Vec<BeadTask> = serde_json::from_str(&stdout)
        .context("Failed to parse bd show JSON output")?;

    tasks.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))
}

/// Parse acceptance criteria checkboxes from issue description
pub fn parse_acceptance_criteria(content: &str) -> Vec<AcceptanceCriteria> {
    let mut criteria = Vec::new();
    
    // Match checkbox lines: - [ ] or - [x] or - [X]
    let checkbox_re = Regex::new(r"^\s*-\s*\[([ xX])\]\s*(.+)$").unwrap();

    for (idx, line) in content.lines().enumerate() {
        if let Some(caps) = checkbox_re.captures(line) {
            let completed = caps[1].to_lowercase() == "x";
            let text = caps[2].trim().to_string();

            criteria.push(AcceptanceCriteria {
                text,
                completed,
                line_number: idx + 1, // 1-indexed
            });
        }
    }

    criteria
}

/// Parse a beads task and extract acceptance criteria
pub fn parse_task(task: &BeadTask) -> ParsedTask {
    // Combine description and notes for criteria parsing
    let full_content = format!("{}\n{}", task.description, task.notes);
    let acceptance_criteria = parse_acceptance_criteria(&full_content);
    
    let all_complete = !acceptance_criteria.is_empty()
        && acceptance_criteria.iter().all(|ac| ac.completed);

    ParsedTask {
        task: task.clone(),
        acceptance_criteria,
        all_complete,
    }
}

/// Find the next unblocked, uncompleted task
pub fn find_next_task(project_root: &Path) -> Result<Option<ParsedTask>> {
    let tasks = get_ready_tasks(project_root)?;

    for task in tasks {
        // Skip closed/completed tasks
        if task.status == "closed" || task.status == "completed" {
            continue;
        }

        let parsed = parse_task(&task);

        // Return first task that has uncompleted criteria or no criteria defined
        // (If no criteria, we still need to work on it)
        if !parsed.all_complete || parsed.acceptance_criteria.is_empty() {
            return Ok(Some(parsed));
        }
    }

    Ok(None)
}

/// Update a task's status via bd
pub fn update_task_status(project_root: &Path, task_id: &str, status: &str) -> Result<()> {
    let output = Command::new("bd")
        .args(["update", task_id, &format!("--status={}", status)])
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd update'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bd update failed: {}", stderr);
    }

    Ok(())
}

/// Close a task via bd
pub fn close_task(project_root: &Path, task_id: &str, reason: Option<&str>) -> Result<()> {
    let mut args = vec!["close".to_string(), task_id.to_string()];
    
    if let Some(r) = reason {
        args.push(format!("--reason={}", r));
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    let output = Command::new("bd")
        .args(&args_refs)
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd close'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bd close failed: {}", stderr);
    }

    Ok(())
}

/// Sync beads state (commits and pushes .beads/ changes)
pub fn sync_beads(project_root: &Path) -> Result<()> {
    let output = Command::new("bd")
        .args(["sync"])
        .current_dir(project_root)
        .output()
        .context("Failed to run 'bd sync'")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail on sync errors - they might just mean no changes
        tracing::warn!("bd sync warning: {}", stderr);
    }

    Ok(())
}

/// Get progress summary for all tasks
pub fn get_progress_summary(project_root: &Path) -> Result<ProgressSummary> {
    let all_tasks = get_all_open_tasks(project_root).unwrap_or_default();
    let ready_tasks = get_ready_tasks(project_root).unwrap_or_default();

    let mut total_tasks = 0;
    let mut completed_tasks = 0;
    let mut total_criteria = 0;
    let mut completed_criteria = 0;

    for task in &all_tasks {
        total_tasks += 1;
        
        let parsed = parse_task(task);
        
        if parsed.all_complete && !parsed.acceptance_criteria.is_empty() {
            completed_tasks += 1;
        }

        for ac in &parsed.acceptance_criteria {
            total_criteria += 1;
            if ac.completed {
                completed_criteria += 1;
            }
        }
    }

    Ok(ProgressSummary {
        total_tasks,
        completed_tasks,
        ready_tasks: ready_tasks.len(),
        total_criteria,
        completed_criteria,
    })
}

#[derive(Debug)]
pub struct ProgressSummary {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub ready_tasks: usize,
    pub total_criteria: usize,
    pub completed_criteria: usize,
}

impl ProgressSummary {
    pub fn task_percentage(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
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
        let content = r#"
## Acceptance Criteria
- [ ] Create the schema file
- [x] Define field mappings
- [ ] Add PostgreSQL adaptations
"#;

        let criteria = parse_acceptance_criteria(content);
        assert_eq!(criteria.len(), 3);
        assert!(!criteria[0].completed);
        assert_eq!(criteria[0].text, "Create the schema file");
        assert!(criteria[1].completed);
        assert_eq!(criteria[1].text, "Define field mappings");
        assert!(!criteria[2].completed);
    }

    #[test]
    fn test_parse_empty_description() {
        let criteria = parse_acceptance_criteria("");
        assert!(criteria.is_empty());
    }
}
