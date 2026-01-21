//! Task Decomposition - Pre-analyzes large tasks and breaks them into subtasks
//!
//! Uses Claude to analyze epic/large tasks and decompose them into smaller,
//! more focused subtasks that fit within context limits.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use crate::task_parser::{BeadTask, ParsedTask, parse_task, get_ready_tasks};

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";

/// Threshold for considering a task "too large" (in characters)
const LARGE_TASK_DESCRIPTION_THRESHOLD: usize = 2000;

/// Maximum acceptable number of items in a single task
const MAX_SCOPE_ITEMS: usize = 5;

/// Analysis result for a task
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub task_id: String,
    pub is_too_large: bool,
    pub reason: String,
    pub description_chars: usize,
    pub criteria_count: usize,
    pub has_subtasks: bool,
    pub suggested_subtasks: Vec<SubtaskSuggestion>,
}

/// A suggested subtask from decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskSuggestion {
    pub title: String,
    pub description: String,
    pub priority: u8,
    pub issue_type: String,
    pub labels: Vec<String>,
}

/// Response from Claude for decomposition
#[derive(Debug, Deserialize)]
struct DecomposeResponse {
    subtasks: Vec<SubtaskSuggestion>,
    reasoning: String,
}

/// Analyze a task to determine if it needs decomposition
pub fn analyze_task(parsed: &ParsedTask, project_root: &Path) -> TaskAnalysis {
    let task = &parsed.task;
    let description_chars = task.description.len() + task.notes.len();
    let criteria_count = parsed.acceptance_criteria.len();
    
    // Check if this task already has subtasks (blocks other tasks)
    let has_subtasks = !task.blocks.is_empty();
    
    // Determine if task is too large
    let mut is_too_large = false;
    let mut reasons = Vec::new();
    
    // Check 1: Description is very long
    if description_chars > LARGE_TASK_DESCRIPTION_THRESHOLD {
        is_too_large = true;
        reasons.push(format!(
            "Description is {} chars (threshold: {})",
            description_chars, LARGE_TASK_DESCRIPTION_THRESHOLD
        ));
    }
    
    // Check 2: No acceptance criteria (vague task)
    if criteria_count == 0 && description_chars > 500 {
        is_too_large = true;
        reasons.push("No acceptance criteria defined for complex task".to_string());
    }
    
    // Check 3: Task mentions multiple files/components to create
    let multi_file_indicators = [
        "## Files to Create",
        "## Components",
        "### Files:",
        "multiple files",
        "several components",
    ];
    
    for indicator in &multi_file_indicators {
        if task.description.contains(indicator) {
            is_too_large = true;
            reasons.push(format!("Task mentions '{}'", indicator));
            break;
        }
    }
    
    // Check 4: Task type is "epic" or "feature" with no subtasks
    if (task.issue_type == "epic" || task.issue_type == "feature") && !has_subtasks {
        if description_chars > 800 {
            is_too_large = true;
            reasons.push(format!(
                "Task type '{}' with no subtasks and large description",
                task.issue_type
            ));
        }
    }
    
    // Don't flag if already has subtasks
    if has_subtasks {
        is_too_large = false;
        reasons.clear();
        reasons.push("Task already has subtasks defined".to_string());
    }
    
    TaskAnalysis {
        task_id: task.id.clone(),
        is_too_large,
        reason: reasons.join("; "),
        description_chars,
        criteria_count,
        has_subtasks,
        suggested_subtasks: Vec::new(),
    }
}

/// Use Claude to decompose a large task into subtasks
pub async fn decompose_task(
    parsed: &ParsedTask,
    api_key: &str,
) -> Result<Vec<SubtaskSuggestion>> {
    let task = &parsed.task;
    
    let system_prompt = r#"You are a software project manager breaking down large tasks into smaller, focused subtasks.

Your goal is to analyze a large epic/feature task and decompose it into 3-7 smaller subtasks that:
1. Can each be completed in a single coding session
2. Have clear, focused scope (1-2 files max)
3. Have logical dependencies (if any)
4. Together fully implement the original task

Output JSON only with this structure:
{
  "subtasks": [
    {
      "title": "Short descriptive title",
      "description": "Clear description with acceptance criteria as checkboxes:\n- [ ] Criterion 1\n- [ ] Criterion 2",
      "priority": 2,
      "issue_type": "task",
      "labels": ["relevant", "labels"]
    }
  ],
  "reasoning": "Brief explanation of decomposition strategy"
}

Rules:
- Each subtask should be completable in 30-50 Claude API iterations
- Include specific file paths when known
- First subtask should be foundational (types, interfaces, etc.)
- Last subtask should be integration/testing
- Priority should match parent (default 2)
- Labels should inherit from parent plus be specific"#;

    let user_prompt = format!(
        r#"Decompose this large task into smaller subtasks:

## Task ID: {}
## Title: {}
## Type: {}
## Priority: P{}
## Labels: {}

## Description:
{}

## Notes:
{}

Output ONLY valid JSON with the subtasks array."#,
        task.id,
        task.title,
        task.issue_type,
        task.priority,
        task.labels.join(", "),
        task.description,
        task.notes
    );

    let client = Client::new();
    
    let request_body = serde_json::json!({
        "model": CLAUDE_MODEL,
        "max_tokens": 4096,
        "messages": [
            {"role": "user", "content": user_prompt}
        ],
        "system": system_prompt
    });

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to call Claude API for decomposition")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Claude API error {}: {}", status, body);
    }

    let api_response: serde_json::Value = response.json().await?;
    
    // Extract text content from response
    let content = api_response["content"][0]["text"]
        .as_str()
        .context("No text in Claude response")?;
    
    // Parse JSON from response (handle markdown code blocks)
    let json_str = if content.contains("```json") {
        content
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(content)
    } else if content.contains("```") {
        content
            .split("```")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(content)
    } else {
        content
    };
    
    let decompose_response: DecomposeResponse = serde_json::from_str(json_str.trim())
        .context("Failed to parse decomposition response as JSON")?;
    
    println!("Decomposition reasoning: {}", decompose_response.reasoning);
    
    Ok(decompose_response.subtasks)
}

/// Create subtasks in beads for a parent task
pub fn create_subtasks(
    project_root: &Path,
    parent_id: &str,
    parent_labels: &[String],
    subtasks: &[SubtaskSuggestion],
) -> Result<Vec<String>> {
    let mut created_ids = Vec::new();
    
    for subtask in subtasks {
        // Merge parent labels with subtask labels
        let mut all_labels: Vec<String> = parent_labels.to_vec();
        for label in &subtask.labels {
            if !all_labels.contains(label) {
                all_labels.push(label.clone());
            }
        }
        
        // Build bd create command
        let mut args = vec![
            "create".to_string(),
            format!("--title={}", subtask.title),
            format!("--type={}", subtask.issue_type),
            format!("--priority={}", subtask.priority),
        ];
        
        if !all_labels.is_empty() {
            args.push(format!("--labels={}", all_labels.join(",")));
        }
        
        if !subtask.description.is_empty() {
            args.push(format!("--description={}", subtask.description));
        }
        
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        
        let output = Command::new("bd")
            .args(&args_refs)
            .current_dir(project_root)
            .output()
            .context("Failed to run 'bd create'")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to create subtask '{}': {}", subtask.title, stderr);
            continue;
        }
        
        // Parse created task ID from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output format: "✓ Created issue: <id>"
        if let Some(id) = stdout.split("issue:").nth(1) {
            let id = id.trim().split_whitespace().next().unwrap_or("").to_string();
            if !id.is_empty() {
                // Add dependency: parent epic depends on subtask (subtask blocks parent)
                // This makes subtasks immediately ready to work on
                let dep_output = Command::new("bd")
                    .args(["dep", "add", parent_id, &id])
                    .current_dir(project_root)
                    .output();
                
                if let Ok(dep_out) = dep_output {
                    if !dep_out.status.success() {
                        tracing::warn!("Failed to add dependency for {}", id);
                    }
                }
                
                created_ids.push(id);
                println!("  Created subtask: {}", subtask.title);
            }
        }
    }
    
    Ok(created_ids)
}

/// Analyze all ready tasks and return those needing decomposition
pub fn find_tasks_needing_decomposition(project_root: &Path) -> Result<Vec<(ParsedTask, TaskAnalysis)>> {
    let ready_tasks = get_ready_tasks(project_root)?;
    let mut needs_decomposition = Vec::new();
    
    for task in ready_tasks {
        let parsed = parse_task(&task);
        let analysis = analyze_task(&parsed, project_root);
        
        if analysis.is_too_large {
            needs_decomposition.push((parsed, analysis));
        }
    }
    
    Ok(needs_decomposition)
}

/// Pre-process: analyze and decompose large tasks before running the loop
pub async fn preprocess_tasks(project_root: &Path, api_key: &str) -> Result<usize> {
    println!("\n🔍 Analyzing tasks for decomposition...\n");
    
    let needs_decomposition = find_tasks_needing_decomposition(project_root)?;
    
    if needs_decomposition.is_empty() {
        println!("✓ All tasks are appropriately sized.\n");
        return Ok(0);
    }
    
    println!("Found {} task(s) that may need decomposition:\n", needs_decomposition.len());
    
    let mut total_created = 0;
    
    for (parsed, analysis) in &needs_decomposition {
        println!("─ {} ({})", parsed.task.id, parsed.task.title);
        println!("  Reason: {}", analysis.reason);
        println!("  Decomposing...");
        
        match decompose_task(parsed, api_key).await {
            Ok(subtasks) => {
                if subtasks.is_empty() {
                    println!("  ⚠ No subtasks suggested");
                    continue;
                }
                
                println!("  Creating {} subtasks...", subtasks.len());
                
                let created = create_subtasks(
                    project_root,
                    &parsed.task.id,
                    &parsed.task.labels,
                    &subtasks,
                )?;
                
                total_created += created.len();
                println!("  ✓ Created {} subtasks\n", created.len());
            }
            Err(e) => {
                println!("  ✗ Decomposition failed: {}\n", e);
            }
        }
    }
    
    // Sync beads after creating subtasks
    if total_created > 0 {
        println!("Syncing beads...");
        crate::task_parser::sync_beads(project_root)?;
    }
    
    println!("\n✓ Pre-processing complete. Created {} subtasks.\n", total_created);
    
    Ok(total_created)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_task_large_description() {
        let task = BeadTask {
            id: "test-1".to_string(),
            title: "Large task".to_string(),
            description: "x".repeat(2500),
            notes: String::new(),
            status: "open".to_string(),
            priority: 2,
            issue_type: "feature".to_string(),
            owner: None,
            labels: vec![],
            depends_on: vec![],
            blocks: vec![],
        };
        
        let parsed = parse_task(&task);
        let analysis = analyze_task(&parsed, Path::new("."));
        
        assert!(analysis.is_too_large);
        assert!(analysis.reason.contains("chars"));
    }
    
    #[test]
    fn test_analyze_task_with_subtasks() {
        let task = BeadTask {
            id: "test-2".to_string(),
            title: "Epic with subtasks".to_string(),
            description: "x".repeat(3000),
            notes: String::new(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "epic".to_string(),
            owner: None,
            labels: vec![],
            depends_on: vec![],
            blocks: vec!["test-3".to_string()],
        };
        
        let parsed = parse_task(&task);
        let analysis = analyze_task(&parsed, Path::new("."));
        
        // Should NOT be marked as too large since it has subtasks
        assert!(!analysis.is_too_large);
        assert!(analysis.has_subtasks);
    }
}
