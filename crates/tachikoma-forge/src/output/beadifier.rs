//! Beadifier - Converts consensus decisions into atomic, actionable tasks.

use crate::llm::{LlmProvider, LlmRequest, LlmMessage, MessageRole, LlmError};
use crate::output::ConsensusSummary;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeadTask {
    pub title: String,         // Max 80 chars
    pub description: String,   // Max 200 chars
    pub priority: Priority,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub dependencies: Vec<String>,  // Other task titles this blocks on
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    P0, P1, P2, P3, P4
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskType {
    #[serde(rename = "task")]
    Task,
    #[serde(rename = "bug")]
    Bug,
    #[serde(rename = "feature")]
    Feature,
    #[serde(rename = "docs")]
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
    ) -> Result<Vec<BeadTask>, LlmError> {
        let mut tasks = Vec::new();
        let remaining_context = &summary.decision;
        
        for i in 0..self.config.max_tasks {
            let prompt = EXTRACT_TASK_PROMPT
                .replace("{decision}", remaining_context)
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
                system_prompt: None,
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
        // Try to parse JSON response
        serde_json::from_str(content).ok()
    }
    
    /// Generate `bd create` commands for each task
    pub fn to_beads(&self, tasks: &[BeadTask]) -> Vec<String> {
        tasks.iter().map(|task| {
            let mut cmd = format!(
                "bd create \"{}\" -p {} --type {}",
                task.title.replace('"', r#"\""#),
                task.priority.as_u8(),
                task.task_type.as_str(),
            );
            
            if let Some(ref epic) = self.config.epic_id {
                cmd.push_str(&format!(" --parent {}", epic));
            }
            
            // Add dependency links as a comment
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
            
            let content = format!(
                "# Spec {}: {}\n\n\
                **Priority:** {}\n\
                **Status:** planned\n\
                **Type:** {}\n\n\
                ---\n\n\
                ## Overview\n\n\
                {}\n\n\
                ---\n\n\
                ## Acceptance Criteria\n\n\
                - [ ] {}\n\
                - [ ] Verify implementation works\n\
                - [ ] Run `cargo check` passes\n",
                spec_id,
                task.title,
                task.priority.as_str(),
                task.task_type.as_str(),
                task.description,
                task.title,
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
    
    fn as_u8(&self) -> u8 {
        match self {
            Priority::P0 => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
            Priority::P4 => 4,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_is_atomic_rejects_compound_tasks() {
        let beadifier = Beadifier::new(BeadifyConfig::default());
        
        let compound_task = BeadTask {
            title: "Create module and add tests".to_string(),
            description: "Create a new module and then add tests".to_string(),
            priority: Priority::P1,
            task_type: TaskType::Task,
            dependencies: vec![],
        };
        
        assert!(!beadifier.is_atomic(&compound_task));
    }
    
    #[test]
    fn test_is_atomic_accepts_simple_tasks() {
        let beadifier = Beadifier::new(BeadifyConfig::default());
        
        let simple_task = BeadTask {
            title: "Create module".to_string(),
            description: "Create a new module in the codebase".to_string(),
            priority: Priority::P1,
            task_type: TaskType::Task,
            dependencies: vec![],
        };
        
        assert!(beadifier.is_atomic(&simple_task));
    }
    
    #[test]
    fn test_is_atomic_rejects_long_titles() {
        let beadifier = Beadifier::new(BeadifyConfig::default());
        
        let long_task = BeadTask {
            title: "A".repeat(85), // Too long
            description: "Short description".to_string(),
            priority: Priority::P1,
            task_type: TaskType::Task,
            dependencies: vec![],
        };
        
        assert!(!beadifier.is_atomic(&long_task));
    }
    
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Create Module"), "create-module");
        assert_eq!(slugify("Fix Bug #123"), "fix-bug-123");
        assert_eq!(slugify("Add_tests-for-API"), "add-tests-for-api");
    }
    
    #[test]
    fn test_to_beads_generates_commands() {
        let beadifier = Beadifier::new(BeadifyConfig::default());
        
        let tasks = vec![
            BeadTask {
                title: "Create module".to_string(),
                description: "Create a new module".to_string(),
                priority: Priority::P1,
                task_type: TaskType::Task,
                dependencies: vec![],
            }
        ];
        
        let commands = beadifier.to_beads(&tasks);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("bd create"));
        assert!(commands[0].contains("Create module"));
        assert!(commands[0].contains("-p 1"));
        assert!(commands[0].contains("--type task"));
    }
    
    #[test]
    fn test_to_spec_files_creates_directory() {
        let beadifier = Beadifier::new(BeadifyConfig::default());
        
        let tasks = vec![
            BeadTask {
                title: "Create test module".to_string(),
                description: "Create a test module for verification".to_string(),
                priority: Priority::P0,
                task_type: TaskType::Feature,
                dependencies: vec![],
            }
        ];
        
        let temp_dir = env::temp_dir().join("beadifier_test");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up if exists
        
        let result = beadifier.to_spec_files(&tasks, &temp_dir, 900);
        assert!(result.is_ok());
        
        // Verify directory was created
        assert!(temp_dir.exists());
        
        // Verify file was created
        let expected_file = temp_dir.join("900-create-test-module.md");
        assert!(expected_file.exists());
        
        // Verify file content
        let content = std::fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("# Spec 900: Create test module"));
        assert!(content.contains("**Priority:** P0"));
        assert!(content.contains("**Type:** feature"));
        assert!(content.contains("Create a test module for verification"));
        
        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_beadify_config_with_epic_id() {
        let config = BeadifyConfig {
            max_tasks: 10,
            target: BeadifyTarget::Beads,
            epic_id: Some("bd-a3f8".to_string()),
        };
        
        let beadifier = Beadifier::new(config);
        
        let tasks = vec![
            BeadTask {
                title: "Implement feature".to_string(),
                description: "Implement a new feature".to_string(),
                priority: Priority::P2,
                task_type: TaskType::Feature,
                dependencies: vec![],
            }
        ];
        
        let commands = beadifier.to_beads(&tasks);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("--parent bd-a3f8"));
    }
}