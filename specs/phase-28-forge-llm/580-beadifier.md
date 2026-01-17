# Spec 580: Beadifier - Consensus to Tasks

**Priority:** P0  
**Status:** planned  
**Depends on:** 579  
**Estimated Effort:** 4 hours  
**Target Files:**
- `crates/tachikoma-forge/src/output/beadifier.rs` (new)
- `crates/tachikoma-forge/src/output/mod.rs` (update)

---

## Overview

The Beadifier takes a Think Tank consensus and breaks it into atomic, actionable tasks. Each task becomes either:
1. A bead issue via `bd create` (preferred for tracking)
2. A small markdown spec file (for Ralph loop)

**Critical Design Rule**: LLMs bias toward monolithic outputs. The Beadifier FORCES decomposition by:
- Calling the LLM with a strict "one task per response" prompt
- Iterating until all tasks are extracted
- Validating each task is truly atomic (single action, <100 words)

Reference: [Beads](https://github.com/steveyegge/beads) - "Distributed, git-backed graph issue tracker for AI agents"

---

## Acceptance Criteria

- [x] Create `crates/tachikoma-forge/src/output/beadifier.rs`
- [x] Define `BeadTask` struct: title, description, priority (P0-P4), dependencies, type (task/bug/feature)
- [x] Define `BeadifyConfig`: max_tasks, target (Beads | SpecFiles), epic_id (for hierarchy)
- [x] Implement `Beadifier::extract_tasks(summary: &ConsensusSummary, llm: &dyn LlmProvider) -> Vec<BeadTask>`
- [x] Use iterative prompting: "Given this decision, what is the FIRST atomic task?"
- [x] Validate each task: title < 80 chars, description < 200 chars, no compound verbs ("and", "then")
- [x] Implement `Beadifier::to_beads(tasks: &[BeadTask]) -> Vec<String>` returning `bd create` commands
- [x] Implement `Beadifier::to_spec_files(tasks: &[BeadTask], dir: &Path) -> Result<()>` writing markdown files
- [x] Export from `output/mod.rs`
- [x] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/output/beadifier.rs

use crate::llm::{LlmProvider, LlmRequest, LlmMessage, MessageRole};
use crate::output::ConsensusSummary;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BeadTask {
    pub title: String,         // Max 80 chars
    pub description: String,   // Max 200 chars
    pub priority: Priority,
    pub task_type: TaskType,
    pub dependencies: Vec<String>,  // Other task titles this blocks on
}

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    P0, P1, P2, P3, P4
}

#[derive(Debug, Clone, Copy)]
pub enum TaskType {
    Task,
    Bug,
    Feature,
    Docs,
}

#[derive(Debug, Clone)]
pub struct BeadifyConfig {
    pub max_tasks: usize,
    pub target: BeadifyTarget,
    pub epic_id: Option<String>,  // e.g., "bd-a3f8" for hierarchical IDs
}

#[derive(Debug, Clone)]
pub enum BeadifyTarget {
    Beads,      // Output `bd create` commands
    SpecFiles,  // Output markdown spec files
}

impl Default for BeadifyConfig {
    fn default() -> Self {
        Self {
            max_tasks: 20,
            target: BeadifyTarget::Beads,
            epic_id: None,
        }
    }
}

pub struct Beadifier {
    config: BeadifyConfig,
}

impl Beadifier {
    pub fn new(config: BeadifyConfig) -> Self {
        Self { config }
    }
    
    /// Extract atomic tasks from consensus using iterative LLM prompting
    pub async fn extract_tasks(
        &self,
        summary: &ConsensusSummary,
        llm: &dyn LlmProvider,
    ) -> Result<Vec<BeadTask>, crate::llm::LlmError> {
        let mut tasks = Vec::new();
        let mut remaining_context = summary.decision.clone();
        
        for i in 0..self.config.max_tasks {
            let prompt = EXTRACT_TASK_PROMPT
                .replace("{decision}", &remaining_context)
                .replace("{task_num}", &(i + 1).to_string());
            
            let request = LlmRequest {
                model: llm.model().to_string(),
                messages: vec![
                    LlmMessage {
                        role: MessageRole::System,
                        content: BEADIFIER_SYSTEM_PROMPT.to_string(),
                    },
                    LlmMessage {
                        role: MessageRole::User,
                        content: prompt,
                    },
                ],
                temperature: Some(0.3), // Low temp for consistency
                max_tokens: Some(300),
            };
            
            let response = llm.complete(request).await?;
            
            // Parse JSON response
            if let Some(task) = self.parse_task_response(&response.content) {
                if task.title == "DONE" {
                    break;
                }
                
                // Validate atomicity
                if self.is_atomic(&task) {
                    tasks.push(task);
                }
            } else {
                break;
            }
        }
        
        Ok(tasks)
    }
    
    fn is_atomic(&self, task: &BeadTask) -> bool {
        let title = task.title.to_lowercase();
        
        // Reject compound tasks
        let compound_markers = [" and ", " then ", " also ", " plus "];
        if compound_markers.iter().any(|m| title.contains(m)) {
            return false;
        }
        
        // Enforce length limits
        if task.title.len() > 80 || task.description.len() > 200 {
            return false;
        }
        
        true
    }
    
    fn parse_task_response(&self, content: &str) -> Option<BeadTask> {
        // Parse JSON like: {"title": "...", "description": "...", "priority": "P1"}
        serde_json::from_str(content).ok()
    }
    
    /// Generate `bd create` commands for each task
    pub fn to_beads(&self, tasks: &[BeadTask]) -> Vec<String> {
        tasks.iter().enumerate().map(|(i, task)| {
            let mut cmd = format!(
                "bd create \"{}\" -p {} --type {}",
                task.title.replace('"', r#"\""#),
                task.priority as u8,
                task.task_type.as_str(),
            );
            
            if let Some(ref epic) = self.config.epic_id {
                cmd.push_str(&format!(" --parent {}", epic));
            }
            
            // Add dependency links as a follow-up
            if !task.dependencies.is_empty() {
                cmd.push_str(&format!(" # deps: {}", task.dependencies.join(", ")));
            }
            
            cmd
        }).collect()
    }
    
    /// Write markdown spec files for Ralph
    pub fn to_spec_files(&self, tasks: &[BeadTask], dir: &Path, start_id: u32) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        
        for (i, task) in tasks.iter().enumerate() {
            let spec_id = start_id + i as u32;
            let filename = format!("{:03}-{}.md", spec_id, slugify(&task.title));
            let path = dir.join(&filename);
            
            // Generate spec markdown content
            let content = generate_spec_content(
                spec_id,
                &task.title,
                task.priority.as_str(),
                task.task_type.as_str(),
                &task.description,
            );
            
            std::fs::write(path, content)?;
        }
        
        Ok(())
    }
}

impl TaskType {
    fn as_str(&self) -> &'static str {
        match self {
            TaskType::Task => "task",
            TaskType::Bug => "bug",
            TaskType::Feature => "feature",
            TaskType::Docs => "docs",
        }
    }
}

impl Priority {
    fn as_str(&self) -> &'static str {
        match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
            Priority::P4 => "P4",
        }
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

const BEADIFIER_SYSTEM_PROMPT: &str = r#"
You are a task decomposer. Your job is to extract ATOMIC tasks from decisions.

Rules:
1. Each task must be ONE action (no "and", "then", "also")
2. Title: max 80 characters, starts with a verb
3. Description: max 200 characters
4. If no more tasks remain, respond with {"title": "DONE"}

Respond ONLY with JSON:
{"title": "...", "description": "...", "priority": "P1", "type": "task"}
"#;

const EXTRACT_TASK_PROMPT: &str = r#"
Decision to implement:
{decision}

Extract task #{task_num}. What is ONE atomic action needed?
If all tasks have been extracted, respond with {"title": "DONE"}.
"#;
```

---

## Usage Example

```rust
// After Think Tank converges
let summary = ConsensusSummary::generate(&session);
let beadifier = Beadifier::new(BeadifyConfig::default());
let tasks = beadifier.extract_tasks(&summary, &provider).await?;

// Option A: Create beads issues
for cmd in beadifier.to_beads(&tasks) {
    println!("{}", cmd);
    // Or: std::process::Command::new("bd").args(cmd.split_whitespace().skip(1)).spawn();
}

// Option B: Create spec files for Ralph
beadifier.to_spec_files(&tasks, Path::new("specs/phase-99-generated"), 900)?;
```
