//! Ralph Wiggum Loop - Agentic Coding Harness
//!
//! "It's not that hard to build a coding agent. It's 300 lines of code
//! running in a loop with LLM tokens. The model does all the heavy lifting."
//! - Geoffrey Huntley
//!
//! This harness:
//! 1. Uses beads issue tracker to find the next task
//! 2. Starts a fresh context for each task implementation
//! 3. Uses the six primitives (read_file, list_files, bash, edit_file, code_search, beads)
//! 4. Updates issue status when tasks complete
//! 5. Auto-syncs beads after each successful implementation

mod claude_client;
mod decompose;
mod git;
mod primitives;
mod progress;
mod task_parser;
mod tui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::io::stdout;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::prelude::*;

use claude_client::{ClaudeClient, StopReason};
use task_parser::{find_next_task, get_progress_summary, get_ready_tasks, get_task, parse_task, ParsedTask};
use tui::{App, EventHandler};
use tui::app::{Task, TaskStatus, OutputLevel};

/// Result of running a single task
#[derive(Debug, Clone, PartialEq)]
enum TaskResult {
    /// Task completed successfully
    Completed,
    /// Hit redline, needs fresh context to continue
    NeedsReboot { had_changes: bool },
    /// Hit max iterations without completing
    MaxIterations,
}

/// Ralph Wiggum Loop - Agentic coding harness
#[derive(Parser)]
#[command(name = "ralph")]
#[command(about = "Agentic coding harness that implements tasks from beads issue tracker")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Project root directory (defaults to current directory)
    #[arg(short, long, global = true)]
    project: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Ralph loop once (implement one task)
    Run {
        /// Task ID to implement (if not provided, picks next ready task)
        #[arg(short, long)]
        issue: Option<String>,

        /// Maximum iterations per task (default: 50)
        #[arg(short, long, default_value = "50")]
        max_iterations: usize,

        /// Token limit before forcing fresh context (default: 150000)
        #[arg(long, default_value = "150000")]
        redline: u32,

        /// Skip auto-sync after completion
        #[arg(long)]
        no_sync: bool,

        /// Auto-decompose large tasks before running
        #[arg(long)]
        auto_decompose: bool,
    },

    /// Run the Ralph loop continuously until all tasks complete
    Loop {
        /// Maximum iterations per task (default: 50)
        #[arg(short, long, default_value = "50")]
        max_iterations: usize,

        /// Token limit before forcing fresh context (default: 150000)
        #[arg(long, default_value = "150000")]
        redline: u32,

        /// Maximum tasks to process (default: unlimited)
        #[arg(long)]
        max_tasks: Option<usize>,

        /// Stop on consecutive failures (default: 3)
        #[arg(long, default_value = "3")]
        fail_streak: usize,

        /// Skip auto-sync
        #[arg(long)]
        no_sync: bool,

        /// Auto-decompose large tasks before running
        #[arg(long)]
        auto_decompose: bool,
    },

    /// Show current progress
    Status,

    /// List all ready tasks
    List {
        /// Show all open tasks (not just ready/unblocked)
        #[arg(long)]
        all: bool,
    },

    /// Show next task to implement
    Next,

    /// Show details of a specific task
    Show {
        /// Task ID to show
        issue: String,
    },

    /// Analyze and decompose large tasks into smaller subtasks
    Decompose {
        /// Only analyze without creating subtasks (dry run)
        #[arg(long)]
        dry_run: bool,

        /// Decompose a specific task by ID
        #[arg(short, long)]
        issue: Option<String>,
    },

    /// Run with TUI (split-pane terminal interface)
    Tui {
        /// Maximum iterations per task (default: 50)
        #[arg(short, long, default_value = "50")]
        max_iterations: usize,

        /// Token limit before forcing fresh context (default: 150000)
        #[arg(long, default_value = "150000")]
        redline: u32,

        /// Maximum tasks to process (default: unlimited)
        #[arg(long)]
        max_tasks: Option<usize>,

        /// Stop on consecutive failures (default: 3)
        #[arg(long, default_value = "3")]
        fail_streak: usize,

        /// Skip auto-sync
        #[arg(long)]
        no_sync: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("ralph=debug,info")
    } else {
        EnvFilter::new("ralph=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Determine project root
    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let beads_dir = project_root.join(".beads");

    if !beads_dir.exists() {
        anyhow::bail!(
            ".beads/ directory not found at {}. Is this a beads-tracked project?\nRun 'bd init' to initialize beads.",
            project_root.display()
        );
    }

    match cli.command {
        Commands::Run {
            issue,
            max_iterations,
            redline,
            no_sync,
            auto_decompose,
        } => {
            // Auto-decompose if requested
            if auto_decompose {
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY environment variable not set")?;
                decompose::preprocess_tasks(&project_root, &api_key).await?;
            }
            run_single(&project_root, issue.as_deref(), max_iterations, redline, !no_sync).await?;
        }
        Commands::Loop {
            max_iterations,
            redline,
            max_tasks,
            fail_streak,
            no_sync,
            auto_decompose,
        } => {
            // Auto-decompose if requested
            if auto_decompose {
                let api_key = std::env::var("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY environment variable not set")?;
                decompose::preprocess_tasks(&project_root, &api_key).await?;
            }
            run_loop(&project_root, max_iterations, redline, max_tasks, fail_streak, !no_sync).await?;
        }
        Commands::Status => {
            show_status(&project_root)?;
        }
        Commands::List { all } => {
            list_tasks(&project_root, all)?;
        }
        Commands::Next => {
            show_next(&project_root)?;
        }
        Commands::Show { issue } => {
            show_task(&project_root, &issue)?;
        }
        Commands::Decompose { dry_run, issue } => {
            decompose_command(&project_root, dry_run, issue.as_deref()).await?;
        }
        Commands::Tui {
            max_iterations,
            redline,
            max_tasks,
            fail_streak,
            no_sync,
        } => {
            run_tui(&project_root, max_iterations, redline, max_tasks, fail_streak, !no_sync).await?;
        }
    }

    Ok(())
}

/// Run the Ralph loop once for a single task
async fn run_single(
    project_root: &PathBuf,
    task_id: Option<&str>,
    max_iterations: usize,
    redline_threshold: u32,
    auto_sync: bool,
) -> Result<TaskResult> {
    // Find the task to implement
    let parsed = if let Some(id) = task_id {
        // Find specific task
        let task = get_task(project_root, id)?;
        parse_task(&task)
    } else {
        // Find next ready task
        find_next_task(project_root)?.ok_or_else(|| anyhow::anyhow!("No ready tasks found! Run 'bd ready' to check."))?
    };

    // Mark task as in_progress
    task_parser::update_task_status(project_root, &parsed.task.id, "in_progress")?;

    println!("\n========================================");
    println!("  RALPH LOOP - Task: {}", parsed.task.id);
    println!("  {}", parsed.task.title);
    println!("  Priority: P{} | Type: {}", parsed.task.priority, parsed.task.issue_type);
    println!("========================================\n");

    // Show acceptance criteria if any
    if !parsed.acceptance_criteria.is_empty() {
        println!("Acceptance Criteria:");
        for ac in &parsed.acceptance_criteria {
            let mark = if ac.completed { "x" } else { " " };
            println!("  [{}] {}", mark, ac.text);
        }
        println!();
    }

    // Get API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    // Build the system prompt
    let system_prompt = build_system_prompt(project_root);

    // Build the task prompt
    let task_prompt = build_task_prompt(&parsed);

    // Create output channel for streaming
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn a task to print output
    let output_handle = tokio::spawn(async move {
        while let Some(text) = rx.recv().await {
            print!("{}", text);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    });

    // Run the agentic loop
    let client = ClaudeClient::new(api_key, project_root);

    println!("Starting agentic loop (max {} iterations)...\n", max_iterations);

    let result = client
        .run_agentic_loop(&system_prompt, &task_prompt, max_iterations, redline_threshold, Some(tx))
        .await?;

    // Wait for output to finish
    output_handle.await?;

    println!("\n\n========================================");
    println!("  LOOP COMPLETE");
    println!("========================================");
    println!("  Iterations: {}", result.iterations);
    println!("  Input tokens: {}", result.total_input_tokens);
    println!("  Output tokens: {}", result.total_output_tokens);
    println!("  Total tokens: {}", result.total_tokens());
    println!("  Estimated cost: ${:.4}", result.estimated_cost());
    println!("  Stop reason: {:?}", result.stop_reason);
    println!("========================================\n");

    // Determine task result based on stop reason
    let task_result = match result.stop_reason {
        StopReason::Completed => {
            // Auto-sync if enabled
            if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    println!("Committed changes as {}", hash);
                }
            }
            TaskResult::Completed
        }
        StopReason::Redline => {
            println!("⚠️  REDLINE: Token limit exceeded. Will retry with fresh context.");
            // Still sync any progress made
            let had_changes = if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    println!("Committed partial progress as {}", hash);
                    true
                } else {
                    println!("No changes to commit.");
                    false
                }
            } else {
                false
            };
            TaskResult::NeedsReboot { had_changes }
        }
        StopReason::MaxIterations => {
            println!("⚠️  Max iterations reached without completing.");
            if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    println!("Committed partial progress as {}", hash);
                }
            }
            TaskResult::MaxIterations
        }
    };

    Ok(task_result)
}

/// Run the Ralph loop continuously
async fn run_loop(
    project_root: &PathBuf,
    max_iterations: usize,
    redline_threshold: u32,
    max_tasks: Option<usize>,
    fail_streak_limit: usize,
    auto_sync: bool,
) -> Result<()> {
    let mut tasks_completed = 0;
    let mut consecutive_failures = 0;
    let mut reboot_count = 0;
    let mut no_changes_count = 0;
    const MAX_REBOOTS_PER_TASK: usize = 3;

    println!("\n========================================");
    println!("  RALPH LOOP - CONTINUOUS MODE");
    println!("  Max iterations per task: {}", max_iterations);
    println!("  Redline threshold: {} tokens", redline_threshold);
    println!("  Fail streak limit: {}", fail_streak_limit);
    println!("========================================\n");

    loop {
        // Check if we've hit max tasks
        if let Some(max) = max_tasks {
            if tasks_completed >= max {
                println!("Reached max tasks limit ({})", max);
                break;
            }
        }

        // Find next task
        let parsed = match find_next_task(project_root)? {
            Some(t) => t,
            None => {
                println!("\nNo more ready tasks!");
                break;
            }
        };

        println!("\n--- Starting Task: {} ---", parsed.task.id);
        println!("    {}\n", parsed.task.title);
        reboot_count = 0;
        no_changes_count = 0;

        // Run for this task (with reboot support)
        loop {
            match run_single(project_root, Some(&parsed.task.id), max_iterations, redline_threshold, auto_sync).await {
                Ok(TaskResult::Completed) => {
                    tasks_completed += 1;
                    consecutive_failures = 0;
                    println!("\n✅ Task {} completed successfully!", parsed.task.id);
                    break;
                }
                Ok(TaskResult::NeedsReboot { had_changes }) => {
                    reboot_count += 1;
                    if !had_changes {
                        no_changes_count += 1;
                    } else {
                        no_changes_count = 0;
                    }

                    // Check if task was actually completed during this run
                    if let Ok(Some(refreshed)) = find_next_task(project_root) {
                        if refreshed.task.id != parsed.task.id {
                            // Task was completed! Move on.
                            tasks_completed += 1;
                            consecutive_failures = 0;
                            println!("\n✅ Task {} was completed. Moving to next task.", parsed.task.id);
                            break;
                        }
                    } else {
                        // All tasks complete
                        tasks_completed += 1;
                        println!("\n✅ Task {} was completed (last task!).", parsed.task.id);
                        break;
                    }

                    if reboot_count >= MAX_REBOOTS_PER_TASK {
                        println!("\n⚠️  Task {} hit redline {} times. Moving to next task.", parsed.task.id, reboot_count);
                        if no_changes_count >= reboot_count {
                            consecutive_failures += 1;
                        } else {
                            consecutive_failures = 0;
                        }
                        break;
                    }
                    println!("\n🔄 Rebooting with fresh context (attempt {}/{})...\n", reboot_count, MAX_REBOOTS_PER_TASK);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Ok(TaskResult::MaxIterations) => {
                    consecutive_failures += 1;
                    println!("\nTask {} hit max iterations without completing.", parsed.task.id);
                    break;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    tracing::error!("Task {} failed: {}", parsed.task.id, e);
                    println!("\nTask {} FAILED: {}", parsed.task.id, e);
                    break;
                }
            }
        }

        if consecutive_failures >= fail_streak_limit {
            println!(
                "\nStopping: {} consecutive failures (limit: {})",
                consecutive_failures, fail_streak_limit
            );
            break;
        }

        // Brief pause between tasks
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    println!("\n========================================");
    println!("  LOOP SESSION COMPLETE");
    println!("  Tasks completed: {}", tasks_completed);
    println!("========================================\n");

    Ok(())
}

/// Show current progress
fn show_status(project_root: &PathBuf) -> Result<()> {
    let summary = get_progress_summary(project_root)?;

    println!("\n========================================");
    println!("  BEADS PROGRESS");
    println!("========================================");
    println!(
        "  Tasks: {}/{} ({:.1}%)",
        summary.completed_tasks,
        summary.total_tasks,
        summary.task_percentage()
    );
    println!("  Ready (unblocked): {}", summary.ready_tasks);
    if summary.total_criteria > 0 {
        println!(
            "  Criteria: {}/{} ({:.1}%)",
            summary.completed_criteria,
            summary.total_criteria,
            summary.criteria_percentage()
        );
    }
    println!("========================================\n");

    Ok(())
}

/// List tasks
fn list_tasks(project_root: &PathBuf, show_all: bool) -> Result<()> {
    let tasks = if show_all {
        task_parser::get_all_open_tasks(project_root)?
    } else {
        get_ready_tasks(project_root)?
    };

    if tasks.is_empty() {
        println!("\nNo {} tasks found.", if show_all { "open" } else { "ready" });
        return Ok(());
    }

    println!("\n{} Tasks ({}):\n", if show_all { "Open" } else { "Ready" }, tasks.len());

    for task in tasks {
        let parsed = parse_task(&task);
        let criteria_status = if parsed.acceptance_criteria.is_empty() {
            String::new()
        } else {
            let done = parsed.acceptance_criteria.iter().filter(|c| c.completed).count();
            format!(" [{}/{}]", done, parsed.acceptance_criteria.len())
        };
        
        let status_icon = match task.status.as_str() {
            "in_progress" => "●",
            "closed" => "✓",
            _ => "○",
        };

        println!(
            "  {} {} · P{} · {}{}", 
            status_icon,
            task.id, 
            task.priority,
            task.title,
            criteria_status
        );
    }

    println!();
    Ok(())
}

/// Show next task to implement
fn show_next(project_root: &PathBuf) -> Result<()> {
    match find_next_task(project_root)? {
        Some(parsed) => {
            println!("\n========================================");
            println!("  NEXT TASK");
            println!("========================================");
            println!("  ID: {}", parsed.task.id);
            println!("  Title: {}", parsed.task.title);
            println!("  Priority: P{}", parsed.task.priority);
            println!("  Type: {}", parsed.task.issue_type);
            
            if !parsed.task.labels.is_empty() {
                println!("  Labels: {}", parsed.task.labels.join(", "));
            }

            if !parsed.acceptance_criteria.is_empty() {
                println!("\n  Acceptance Criteria:");
                for ac in &parsed.acceptance_criteria {
                    let mark = if ac.completed { "x" } else { " " };
                    println!("    [{}] {}", mark, ac.text);
                }
            }

            if !parsed.task.description.is_empty() {
                println!("\n  Description:");
                for line in parsed.task.description.lines().take(10) {
                    println!("    {}", line);
                }
                if parsed.task.description.lines().count() > 10 {
                    println!("    ...[truncated]");
                }
            }

            println!("========================================\n");
        }
        None => {
            println!("\nNo ready tasks found!");
        }
    }

    Ok(())
}

/// Show details of a specific task
fn show_task(project_root: &PathBuf, task_id: &str) -> Result<()> {
    let task = get_task(project_root, task_id)?;
    let parsed = parse_task(&task);

    println!("\n========================================");
    println!("  TASK: {}", task.id);
    println!("========================================");
    println!("  Title: {}", task.title);
    println!("  Status: {}", task.status);
    println!("  Priority: P{}", task.priority);
    println!("  Type: {}", task.issue_type);
    
    if let Some(owner) = &task.owner {
        println!("  Owner: {}", owner);
    }
    
    if !task.labels.is_empty() {
        println!("  Labels: {}", task.labels.join(", "));
    }

    if !parsed.acceptance_criteria.is_empty() {
        println!("\n  Acceptance Criteria:");
        for ac in &parsed.acceptance_criteria {
            let mark = if ac.completed { "x" } else { " " };
            println!("    [{}] {}", mark, ac.text);
        }
    }

    if !task.description.is_empty() {
        println!("\n  Description:\n{}", task.description);
    }

    if !task.notes.is_empty() {
        println!("\n  Notes:\n{}", task.notes);
    }

    println!("========================================\n");

    Ok(())
}

/// Decompose large tasks into smaller subtasks
async fn decompose_command(
    project_root: &PathBuf,
    dry_run: bool,
    specific_issue: Option<&str>,
) -> Result<()> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    if let Some(issue_id) = specific_issue {
        // Decompose specific task
        let task = get_task(project_root, issue_id)?;
        let parsed = parse_task(&task);
        let analysis = decompose::analyze_task(&parsed, project_root);

        println!("\n========================================");
        println!("  TASK ANALYSIS: {}", task.id);
        println!("========================================");
        println!("  Title: {}", task.title);
        println!("  Description chars: {}", analysis.description_chars);
        println!("  Criteria count: {}", analysis.criteria_count);
        println!("  Has subtasks: {}", analysis.has_subtasks);
        println!("  Needs decomposition: {}", analysis.is_too_large);
        if !analysis.reason.is_empty() {
            println!("  Reason: {}", analysis.reason);
        }
        println!("========================================\n");

        if !analysis.is_too_large {
            println!("Task does not need decomposition.");
            return Ok(());
        }

        if dry_run {
            println!("DRY RUN: Would decompose this task. Run without --dry-run to create subtasks.");
            return Ok(());
        }

        println!("Decomposing task...\n");
        let subtasks = decompose::decompose_task(&parsed, &api_key).await?;

        if subtasks.is_empty() {
            println!("No subtasks suggested.");
            return Ok(());
        }

        println!("\nCreating {} subtasks...\n", subtasks.len());
        let created = decompose::create_subtasks(
            project_root,
            &task.id,
            &task.labels,
            &subtasks,
        )?;

        println!("\n✓ Created {} subtasks.", created.len());
        
        // Sync beads
        task_parser::sync_beads(project_root)?;
    } else {
        // Analyze all ready tasks
        if dry_run {
            println!("\n🔍 Analyzing tasks (dry run)...\n");
            let needs_decomposition = decompose::find_tasks_needing_decomposition(project_root)?;

            if needs_decomposition.is_empty() {
                println!("✓ All tasks are appropriately sized.\n");
                return Ok(());
            }

            println!("Found {} task(s) that need decomposition:\n", needs_decomposition.len());
            for (parsed, analysis) in &needs_decomposition {
                println!("  {} · {}", parsed.task.id, parsed.task.title);
                println!("    Reason: {}", analysis.reason);
                println!("    Description: {} chars, {} criteria\n", 
                    analysis.description_chars, analysis.criteria_count);
            }
            println!("Run without --dry-run to decompose these tasks.");
        } else {
            decompose::preprocess_tasks(project_root, &api_key).await?;
        }
    }

    Ok(())
}

/// Build the system prompt for Claude
/// 
/// Includes:
/// - Codebase map (if available)
/// - Recent progress from previous iterations
/// - Explicit anti-patterns section
/// - 3-iteration rule enforcement
fn build_system_prompt(project_root: &PathBuf) -> String {
    // Load codebase map (if available)
    let codemap = progress::load_codebase_summary(project_root);
    
    // Load recent progress (last 3 entries)
    let recent_progress = progress::load_recent_progress(project_root, 3)
        .unwrap_or_default();

    // Build codemap section
    let codemap_section = if codemap.is_empty() {
        "No codemap available. The task description contains file paths - trust them.".to_string()
    } else {
        format!("```\n{}\n```", codemap)
    };

    // Build progress section
    let progress_section = if recent_progress.is_empty() {
        "No previous progress recorded. You're starting fresh.".to_string()
    } else {
        format!("Study these patterns from previous work:\n\n{}", recent_progress)
    };

    format!(
        r#"You are Ralph, an AI coding assistant. Your mission: implement tasks efficiently with minimal exploration.

## CRITICAL RULES (Violations trigger intervention)

1. **3-iteration rule**: You MUST produce an edit_file call within 3 iterations
2. **Trust the task**: File paths in the task description are accurate - don't verify them
3. **One search max**: Maximum ONE code_search per pattern, then implement
4. **No recursive exploration**: Use targeted file reads, not recursive list_files

## Codebase Overview
{codemap_section}

## Previous Work (Study First)
{progress_section}

## Project Root
{project_root}

## Available Tools
| Tool | Purpose | Limits |
|------|---------|--------|
| read_file | Read file contents | Use start_line/end_line for large files |
| list_files | List directory | Recursive limited to 50 entries |
| code_search | Find patterns | ONE search per concept, then code |
| edit_file | Create/modify files | empty old_string = new file |
| bash | Build/test/git | Build commands only, NOT exploration |
| beads | Task management | close when done |

## Creating New Files

To create a new file:
```
edit_file path="path/to/NewFile.cs" old_string="" new_string="<full file contents>"
```

## Anti-Patterns (DO NOT)

❌ Reading more than 3 files before making an edit
❌ Using list_files with recursive=true for "exploring"
❌ Running bash commands like find, grep -r, cat (use dedicated tools)
❌ Reading the entire project structure before starting
❌ Searching for the same pattern multiple times
❌ Asking "what files exist" - the task description tells you

## Allowed Bash Commands

✅ cargo build, cargo test, cargo check, cargo fmt
✅ npm run, npm test, npm build
✅ dotnet build, dotnet test, dotnet run
✅ git status, git diff, git add, git commit
✅ bd close, bd sync, bd update

## Beads Tool Usage

- Close task: `beads action="close" task_id="<id>" reason="<summary>"`
- Create subtask: `beads action="create" title="..." description="..." blocks="<parent-id>"`

## Completion

When done, close the task immediately:
```
beads action="close" task_id="<id>" reason="Implemented X, created Y, tested with Z"
```
"#,
        codemap_section = codemap_section,
        progress_section = progress_section,
        project_root = project_root.display()
    )
}

/// Build the task prompt for a specific task
/// 
/// Includes explicit numbered execution plan to guide the agent
/// through efficient task completion.
fn build_task_prompt(parsed: &ParsedTask) -> String {
    let incomplete: Vec<_> = parsed
        .acceptance_criteria
        .iter()
        .filter(|ac| !ac.completed)
        .collect();
    
    let completed: Vec<_> = parsed
        .acceptance_criteria
        .iter()
        .filter(|ac| ac.completed)
        .collect();

    let criteria_section = if incomplete.is_empty() {
        if parsed.acceptance_criteria.is_empty() {
            "No specific acceptance criteria defined. Use your judgment.".to_string()
        } else {
            "✅ All acceptance criteria complete! Verify and close the task.".to_string()
        }
    } else {
        let incomplete_str: Vec<String> = incomplete
            .iter()
            .map(|ac| format!("- [ ] {}", ac.text))
            .collect();
        let completed_str: Vec<String> = completed
            .iter()
            .map(|ac| format!("- [x] {}", ac.text))
            .collect();
        
        let mut section = String::from("### Acceptance Criteria\n\n");
        if !completed_str.is_empty() {
            section.push_str(&completed_str.join("\n"));
            section.push('\n');
        }
        section.push_str(&incomplete_str.join("\n"));
        section
    };

    // Extract file paths mentioned in description (simple heuristic)
    let mentioned_files: Vec<&str> = parsed.task.description
        .split_whitespace()
        .filter(|w| {
            w.contains('/') && (
                w.ends_with(".rs") || w.ends_with(".cs") || w.ends_with(".ts") ||
                w.ends_with(".tsx") || w.ends_with(".js") || w.ends_with(".py") ||
                w.ends_with(".go") || w.ends_with(".java") || w.ends_with(".md") ||
                w.ends_with(".yaml") || w.ends_with(".json") || w.ends_with(".toml")
            )
        })
        .take(5) // Limit to 5 files
        .collect();
    
    let files_hint = if mentioned_files.is_empty() {
        String::new()
    } else {
        format!("\n**Files mentioned**: {}", mentioned_files.join(", "))
    };

    format!(
        r#"## TASK: {id} - {title}

### Description
{description}
{files_hint}

{criteria}

---

## YOUR EXECUTION PLAN

**Iteration 1: Orient** (if needed)
- ONE code_search to find similar pattern (e.g., "class.*Service" or "fn.*handler")
- OR read ONE file mentioned above
- Skip this if the task is clear

**Iteration 2-3: Implement**
- Create/edit files with edit_file
- Use empty old_string for new files: `edit_file path="..." old_string="" new_string="..."`
- Implement ALL criteria

**Iteration 4: Verify & Close**
- Build: `bash command="cargo build"` (or npm/dotnet as appropriate)
- Close: `beads action="close" task_id="{id}" reason="<what was done>"`

---

## IMPORTANT

✓ The description tells you EXACTLY which files to create/edit - TRUST IT
✓ Don't explore - START CODING
✓ If you're on iteration 3+ without an edit, you're doing it wrong
"#,
        id = parsed.task.id,
        title = parsed.task.title,
        description = parsed.task.description,
        files_hint = files_hint,
        criteria = criteria_section,
    )
}

/// Run with TUI interface
async fn run_tui(
    project_root: &PathBuf,
    max_iterations: usize,
    redline_threshold: u32,
    max_tasks: Option<usize>,
    fail_streak_limit: usize,
    auto_sync: bool,
) -> Result<()> {
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(redline_threshold);
    app.is_running = true;

    // Load all ready tasks
    let ready_tasks = get_ready_tasks(project_root).unwrap_or_default();
    let tasks: Vec<Task> = ready_tasks.iter().map(|task| {
        let parsed = parse_task(task);
        let criteria_done = parsed.acceptance_criteria.iter().filter(|c| c.completed).count();
        let criteria_total = parsed.acceptance_criteria.len();
        
        Task {
            id: 0, // Use index as numeric ID for TUI
            name: format!("{}: {}", task.id, task.title),
            status: match task.status.as_str() {
                "in_progress" => TaskStatus::InProgress,
                "closed" => TaskStatus::Completed,
                _ => TaskStatus::Pending,
            },
            criteria_done,
            criteria_total,
        }
    }).collect();
    
    app.set_tasks(tasks);

    // Create event handler
    let event_handler = EventHandler::new(50);

    // Create channels for output
    let (output_tx, mut output_rx) = mpsc::channel::<String>(1000);

    // Clone for the spawned task
    let project_root_clone = project_root.clone();

    // Spawn the loop runner in a separate task
    let loop_handle = tokio::spawn(async move {
        run_loop_internal(
            &project_root_clone,
            max_iterations,
            redline_threshold,
            max_tasks,
            fail_streak_limit,
            auto_sync,
            output_tx,
        ).await
    });

    // Main TUI loop
    loop {
        // Draw UI
        terminal.draw(|frame| {
            tui::ui::Ui::render(frame, &app);
        })?;

        // Handle output from the loop
        while let Ok(text) = output_rx.try_recv() {
            let level = if text.starts_with("---") {
                OutputLevel::Info
            } else if text.starts_with("[") && text.contains("Executing tool:") {
                OutputLevel::Tool
            } else if text.starts_with("[REDLINE") {
                OutputLevel::Error
            } else if text.contains("✓") || text.contains("complete") {
                OutputLevel::Success
            } else {
                OutputLevel::Text
            };
            
            for line in text.lines() {
                if !line.is_empty() {
                    app.add_output(level, line.to_string());
                }
            }
        }

        // Handle keyboard events
        if let Some(key) = event_handler.poll()? {
            event_handler.handle_key(&mut app, key);
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Check if loop is done
        if loop_handle.is_finished() {
            app.is_running = false;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Wait for loop to finish if still running
    if !loop_handle.is_finished() {
        loop_handle.abort();
    }

    Ok(())
}

/// Internal loop runner that sends output to channel
async fn run_loop_internal(
    project_root: &PathBuf,
    max_iterations: usize,
    redline_threshold: u32,
    max_tasks: Option<usize>,
    fail_streak_limit: usize,
    auto_sync: bool,
    output_tx: mpsc::Sender<String>,
) -> Result<()> {
    let mut tasks_completed = 0;
    let mut consecutive_failures = 0;
    let mut reboot_count = 0;
    let mut no_changes_count = 0;
    const MAX_REBOOTS_PER_TASK: usize = 3;

    let _ = output_tx.send("Starting Ralph Loop...\n".to_string()).await;

    loop {
        if let Some(max) = max_tasks {
            if tasks_completed >= max {
                let _ = output_tx.send(format!("Reached max tasks limit ({})\n", max)).await;
                break;
            }
        }

        let parsed = match find_next_task(project_root)? {
            Some(t) => t,
            None => {
                let _ = output_tx.send("✓ No more ready tasks!\n".to_string()).await;
                break;
            }
        };

        let _ = output_tx.send(format!("\n→ Starting Task: {}\n  {}\n", parsed.task.id, parsed.task.title)).await;
        reboot_count = 0;
        no_changes_count = 0;

        loop {
            match run_single_internal(project_root, Some(&parsed.task.id), max_iterations, redline_threshold, auto_sync, output_tx.clone()).await {
                Ok(TaskResult::Completed) => {
                    tasks_completed += 1;
                    consecutive_failures = 0;
                    let _ = output_tx.send(format!("\n✓ Task {} completed!\n", parsed.task.id)).await;
                    break;
                }
                Ok(TaskResult::NeedsReboot { had_changes }) => {
                    reboot_count += 1;
                    if !had_changes {
                        no_changes_count += 1;
                    } else {
                        no_changes_count = 0;
                    }

                    if let Ok(Some(refreshed)) = find_next_task(project_root) {
                        if refreshed.task.id != parsed.task.id {
                            tasks_completed += 1;
                            consecutive_failures = 0;
                            let _ = output_tx.send(format!("\n✓ Task {} was completed!\n", parsed.task.id)).await;
                            break;
                        }
                    } else {
                        tasks_completed += 1;
                        let _ = output_tx.send(format!("\n✓ Task {} was completed (last task)!\n", parsed.task.id)).await;
                        break;
                    }

                    if reboot_count >= MAX_REBOOTS_PER_TASK {
                        let _ = output_tx.send(format!("\n⚠ Hit redline {} times, moving on\n", reboot_count)).await;
                        if no_changes_count >= reboot_count {
                            consecutive_failures += 1;
                        } else {
                            consecutive_failures = 0;
                        }
                        break;
                    }
                    let _ = output_tx.send(format!("\n🔄 Rebooting ({}/{})...\n", reboot_count, MAX_REBOOTS_PER_TASK)).await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
                Ok(TaskResult::MaxIterations) => {
                    consecutive_failures += 1;
                    let _ = output_tx.send(format!("\n⚠ Max iterations reached\n")).await;
                    break;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    let _ = output_tx.send(format!("\n✗ Error: {}\n", e)).await;
                    break;
                }
            }
        }

        if consecutive_failures >= fail_streak_limit {
            let _ = output_tx.send(format!("\nStopping: {} consecutive failures\n", consecutive_failures)).await;
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    let _ = output_tx.send(format!("\nLoop complete. {} tasks done.\n", tasks_completed)).await;
    Ok(())
}

/// Run a single task with output to channel
async fn run_single_internal(
    project_root: &PathBuf,
    task_id: Option<&str>,
    max_iterations: usize,
    redline_threshold: u32,
    auto_sync: bool,
    output_tx: mpsc::Sender<String>,
) -> Result<TaskResult> {
    let parsed = if let Some(id) = task_id {
        let task = get_task(project_root, id)?;
        parse_task(&task)
    } else {
        find_next_task(project_root)?.ok_or_else(|| anyhow::anyhow!("No ready tasks!"))?
    };

    // Mark as in_progress
    task_parser::update_task_status(project_root, &parsed.task.id, "in_progress")?;

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    let system_prompt = build_system_prompt(project_root);
    let task_prompt = build_task_prompt(&parsed);

    let client = ClaudeClient::new(api_key, project_root);

    let result = client
        .run_agentic_loop(&system_prompt, &task_prompt, max_iterations, redline_threshold, Some(output_tx.clone()))
        .await?;

    let task_result = match result.stop_reason {
        StopReason::Completed => {
            if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    let _ = output_tx.send(format!("Committed: {}\n", hash)).await;
                }
            }
            TaskResult::Completed
        }
        StopReason::Redline => {
            let had_changes = if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    let _ = output_tx.send(format!("Committed partial: {}\n", hash)).await;
                    true
                } else {
                    false
                }
            } else {
                false
            };
            TaskResult::NeedsReboot { had_changes }
        }
        StopReason::MaxIterations => {
            if auto_sync {
                task_parser::sync_beads(project_root)?;
                if let Some(hash) = git::auto_commit_task(project_root, &parsed.task.id, &parsed.task.title)? {
                    let _ = output_tx.send(format!("Committed partial: {}\n", hash)).await;
                }
            }
            TaskResult::MaxIterations
        }
    };

    Ok(task_result)
}
