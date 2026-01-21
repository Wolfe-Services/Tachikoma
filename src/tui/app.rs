//! App state machine for the TUI

use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Maximum number of output lines to keep in history
const MAX_OUTPUT_LINES: usize = 10000;

/// Current view mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Split,
    Dashboard,
    Help,
}

/// Status of a task/spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A task/spec entry
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub status: TaskStatus,
    pub criteria_done: usize,
    pub criteria_total: usize,
}

/// A single line of output with metadata
#[derive(Debug, Clone)]
pub struct OutputLine {
    pub timestamp: DateTime<Utc>,
    pub level: OutputLevel,
    pub text: String,
}

/// Output level/type for coloring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLevel {
    Info,
    Debug,
    Tool,
    ToolResult,
    Error,
    Success,
    Text,
}

/// Loop event for structured updates
#[derive(Debug, Clone)]
pub enum LoopEvent {
    IterationStart(usize),
    ToolCall { name: String, input: String },
    ToolResult { name: String, output: String, success: bool },
    Text(String),
    TokenUpdate { input: u32, output: u32 },
    SpecComplete(u32),
    Redline,
}

/// Main app state
pub struct App {
    // View state
    pub current_view: View,
    pub selected_task: usize,
    pub output_scroll: usize,
    pub task_scroll: usize,
    pub focus_pane: FocusPane,
    
    // Tasks
    pub tasks: Vec<Task>,
    
    // Execution state
    pub is_running: bool,
    pub is_paused: bool,
    pub current_spec_id: Option<u32>,
    
    // Metrics
    pub iterations: usize,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_cost: f64,
    pub reboots: usize,
    pub commits: usize,
    
    // Session timing
    pub session_start: DateTime<Utc>,
    
    // Output buffer
    pub output_lines: VecDeque<OutputLine>,
    
    // Progress
    pub specs_completed: usize,
    pub specs_total: usize,
    pub criteria_completed: usize,
    pub criteria_total: usize,
    
    // Token limits
    pub redline_threshold: u32,
    
    // Should quit
    pub should_quit: bool,
    pub quit_requested: bool,
}

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusPane {
    #[default]
    Tasks,
    Output,
}

impl App {
    /// Create a new app with default state
    pub fn new(redline_threshold: u32) -> Self {
        Self {
            current_view: View::default(),
            selected_task: 0,
            output_scroll: 0,
            task_scroll: 0,
            focus_pane: FocusPane::default(),
            tasks: Vec::new(),
            is_running: false,
            is_paused: false,
            current_spec_id: None,
            iterations: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_cost: 0.0,
            reboots: 0,
            commits: 0,
            session_start: Utc::now(),
            output_lines: VecDeque::with_capacity(MAX_OUTPUT_LINES),
            specs_completed: 0,
            specs_total: 0,
            criteria_completed: 0,
            criteria_total: 0,
            redline_threshold,
            should_quit: false,
            quit_requested: false,
        }
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    /// Get progress percentage (0-100)
    pub fn progress_percentage(&self) -> f64 {
        if self.specs_total == 0 {
            0.0
        } else {
            (self.specs_completed as f64 / self.specs_total as f64) * 100.0
        }
    }

    /// Get token usage percentage (0-100)
    pub fn token_percentage(&self) -> f64 {
        if self.redline_threshold == 0 {
            0.0
        } else {
            (self.total_tokens() as f64 / self.redline_threshold as f64) * 100.0
        }
    }

    /// Check if we're at or above redline
    pub fn is_redline(&self) -> bool {
        self.total_tokens() >= self.redline_threshold
    }

    /// Get session duration as human-readable string
    pub fn session_duration(&self) -> String {
        let duration = Utc::now().signed_duration_since(self.session_start);
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        
        if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Add an output line
    pub fn add_output(&mut self, level: OutputLevel, text: String) {
        // Remove oldest if at capacity
        if self.output_lines.len() >= MAX_OUTPUT_LINES {
            self.output_lines.pop_front();
        }
        
        self.output_lines.push_back(OutputLine {
            timestamp: Utc::now(),
            level,
            text,
        });
        
        // Auto-scroll to bottom if near bottom
        let visible_lines = 20; // Approximate
        if self.output_scroll + visible_lines >= self.output_lines.len().saturating_sub(1) {
            self.output_scroll = self.output_lines.len().saturating_sub(1);
        }
    }

    /// Handle a loop event
    pub fn handle_loop_event(&mut self, event: LoopEvent) {
        match event {
            LoopEvent::IterationStart(n) => {
                self.iterations = n;
                self.add_output(OutputLevel::Info, format!("--- Iteration {} ---", n));
            }
            LoopEvent::ToolCall { name, input } => {
                self.add_output(OutputLevel::Tool, format!("[{}] {}", name, 
                    if input.len() > 100 { format!("{}...", &input[..100]) } else { input }));
            }
            LoopEvent::ToolResult { name, output, success } => {
                let level = if success { OutputLevel::ToolResult } else { OutputLevel::Error };
                let preview = if output.len() > 200 { 
                    format!("{}...", &output[..200]) 
                } else { 
                    output 
                };
                self.add_output(level, format!("[{}] → {}", name, preview));
            }
            LoopEvent::Text(text) => {
                for line in text.lines() {
                    self.add_output(OutputLevel::Text, line.to_string());
                }
            }
            LoopEvent::TokenUpdate { input, output } => {
                self.input_tokens = input;
                self.output_tokens = output;
                self.update_cost();
            }
            LoopEvent::SpecComplete(id) => {
                self.add_output(OutputLevel::Success, format!("✓ Spec {} complete!", id));
                self.specs_completed += 1;
                // Update task status
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    task.status = TaskStatus::Completed;
                }
            }
            LoopEvent::Redline => {
                self.add_output(OutputLevel::Error, "⚠ REDLINE: Token limit reached".to_string());
                self.reboots += 1;
            }
        }
    }

    /// Update cost estimate based on current tokens
    fn update_cost(&mut self) {
        // Claude Sonnet pricing: $3/1M input, $15/1M output
        let input_cost = (self.input_tokens as f64 / 1_000_000.0) * 3.0;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * 15.0;
        self.total_cost = input_cost + output_cost;
    }

    /// Toggle between views
    pub fn toggle_view(&mut self) {
        self.current_view = match self.current_view {
            View::Split => View::Dashboard,
            View::Dashboard => View::Split,
            View::Help => View::Split,
        };
    }

    /// Show help view
    pub fn show_help(&mut self) {
        self.current_view = View::Help;
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        let status = if self.is_paused { "PAUSED" } else { "RESUMED" };
        self.add_output(OutputLevel::Info, format!("⏸ {}", status));
    }

    /// Switch focus between panes
    pub fn toggle_focus(&mut self) {
        self.focus_pane = match self.focus_pane {
            FocusPane::Tasks => FocusPane::Output,
            FocusPane::Output => FocusPane::Tasks,
        };
    }

    /// Scroll up in the active pane
    pub fn scroll_up(&mut self) {
        match self.focus_pane {
            FocusPane::Tasks => {
                self.selected_task = self.selected_task.saturating_sub(1);
            }
            FocusPane::Output => {
                self.output_scroll = self.output_scroll.saturating_sub(1);
            }
        }
    }

    /// Scroll down in the active pane
    pub fn scroll_down(&mut self) {
        match self.focus_pane {
            FocusPane::Tasks => {
                if !self.tasks.is_empty() && self.selected_task < self.tasks.len() - 1 {
                    self.selected_task += 1;
                }
            }
            FocusPane::Output => {
                if self.output_scroll < self.output_lines.len().saturating_sub(1) {
                    self.output_scroll += 1;
                }
            }
        }
    }

    /// Page up in output
    pub fn page_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(20);
    }

    /// Page down in output
    pub fn page_down(&mut self) {
        self.output_scroll = (self.output_scroll + 20).min(self.output_lines.len().saturating_sub(1));
    }

    /// Request quit (will show confirmation if running)
    pub fn request_quit(&mut self) {
        if self.is_running && !self.quit_requested {
            self.quit_requested = true;
            self.add_output(OutputLevel::Info, "Press 'q' again to confirm quit".to_string());
        } else {
            self.should_quit = true;
        }
    }

    /// Set tasks from parsed specs
    pub fn set_tasks(&mut self, tasks: Vec<Task>) {
        self.specs_total = tasks.len();
        self.specs_completed = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        self.criteria_total = tasks.iter().map(|t| t.criteria_total).sum();
        self.criteria_completed = tasks.iter().map(|t| t.criteria_done).sum();
        self.tasks = tasks;
    }

    /// Mark a spec as in progress
    pub fn set_current_spec(&mut self, id: u32) {
        self.current_spec_id = Some(id);
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::InProgress;
        }
        // Select the current spec in the list
        if let Some(idx) = self.tasks.iter().position(|t| t.id == id) {
            self.selected_task = idx;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(150_000)
    }
}
