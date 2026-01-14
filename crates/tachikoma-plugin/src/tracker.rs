//! Tracker plugin trait and types
//!
//! Trackers manage task/spec state and provide work items to the agentic loop.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{PluginManifest, Result};

/// A task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Full description/content of the task
    pub description: String,
    
    /// Path to the task file (if file-based)
    pub path: Option<String>,
    
    /// Task priority (0 = highest)
    pub priority: u8,
    
    /// Tasks this depends on
    pub dependencies: Vec<String>,
    
    /// Acceptance criteria (if any)
    pub criteria: Vec<TaskCriterion>,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A single acceptance criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCriterion {
    /// Criterion text
    pub text: String,
    
    /// Whether it's completed
    pub completed: bool,
}

/// Progress summary for the tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Total number of tasks
    pub total_tasks: usize,
    
    /// Completed tasks
    pub completed_tasks: usize,
    
    /// In-progress tasks
    pub in_progress_tasks: usize,
    
    /// Total criteria across all tasks
    pub total_criteria: usize,
    
    /// Completed criteria
    pub completed_criteria: usize,
}

impl Progress {
    /// Get task completion percentage
    pub fn task_percentage(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }
    
    /// Get criteria completion percentage
    pub fn criteria_percentage(&self) -> f64 {
        if self.total_criteria == 0 {
            0.0
        } else {
            (self.completed_criteria as f64 / self.total_criteria as f64) * 100.0
        }
    }
}

/// Trait for tracker plugins
///
/// Implement this trait to create a new task/spec tracker.
#[async_trait]
pub trait TrackerPlugin: Send + Sync {
    /// Get plugin metadata
    fn manifest(&self) -> &PluginManifest;
    
    /// Initialize the tracker
    async fn init(&mut self, root: &std::path::Path) -> Result<()>;
    
    /// Get the next task to execute
    ///
    /// Returns None if all tasks are complete or blocked.
    async fn next_task(&self) -> Result<Option<Task>>;
    
    /// Get a specific task by ID
    async fn get_task(&self, id: &str) -> Result<Option<Task>>;
    
    /// List all tasks
    async fn list_tasks(&self) -> Result<Vec<Task>>;
    
    /// Mark a task as in-progress
    async fn start_task(&self, id: &str) -> Result<()>;
    
    /// Mark a task as complete
    async fn complete_task(&self, id: &str) -> Result<()>;
    
    /// Mark a specific criterion as complete
    async fn complete_criterion(&self, task_id: &str, criterion_idx: usize) -> Result<()>;
    
    /// Get progress summary
    async fn progress(&self) -> Result<Progress>;
    
    /// Sync with backing store (e.g., git, file system)
    async fn sync(&self) -> Result<()>;
    
    /// Reload tasks from source
    async fn reload(&mut self) -> Result<()>;
}
