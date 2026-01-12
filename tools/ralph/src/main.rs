//! Ralph Wiggum Loop - Agentic Coding Harness for Tachikoma
//!
//! "It's not that hard to build a coding agent. It's 300 lines of code
//! running in a loop with LLM tokens. The model does all the heavy lifting."
//! - Geoffrey Huntley
//!
//! This harness:
//! 1. Navigates THE PIN (specs/README.md) to find the next spec
//! 2. Starts a fresh context for each spec implementation
//! 3. Uses the five primitives (read_file, list_files, bash, edit_file, code_search)
//! 4. Updates checkboxes when tasks complete
//! 5. Auto-commits after each successful implementation

mod claude_client;
mod git;
mod primitives;
mod spec_parser;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use claude_client::ClaudeClient;
use spec_parser::{find_next_spec, get_progress_summary, parse_readme, ParsedSpec};

/// Ralph Wiggum Loop - Agentic coding harness
#[derive(Parser)]
#[command(name = "ralph")]
#[command(about = "Agentic coding harness that implements specs one-by-one")]
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
    /// Run the Ralph loop once (implement one spec)
    Run {
        /// Spec ID to implement (if not provided, picks next uncompleted)
        #[arg(short, long)]
        spec: Option<u32>,

        /// Maximum iterations per spec (default: 50)
        #[arg(short, long, default_value = "50")]
        max_iterations: usize,

        /// Skip auto-commit after completion
        #[arg(long)]
        no_commit: bool,
    },

    /// Run the Ralph loop continuously until all specs complete
    Loop {
        /// Maximum iterations per spec (default: 50)
        #[arg(short, long, default_value = "50")]
        max_iterations: usize,

        /// Maximum specs to process (default: unlimited)
        #[arg(long)]
        max_specs: Option<usize>,

        /// Stop on consecutive failures (default: 3)
        #[arg(long, default_value = "3")]
        fail_streak: usize,

        /// Skip auto-commit
        #[arg(long)]
        no_commit: bool,
    },

    /// Show current progress
    Status,

    /// List all specs
    List {
        /// Show only incomplete specs
        #[arg(long)]
        incomplete: bool,

        /// Filter by phase
        #[arg(long)]
        phase: Option<u32>,
    },

    /// Show next spec to implement
    Next,

    /// Validate specs structure
    Validate,
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

    let specs_dir = project_root.join("specs");

    if !specs_dir.exists() {
        anyhow::bail!(
            "specs/ directory not found at {}. Are you in the right project?",
            project_root.display()
        );
    }

    match cli.command {
        Commands::Run {
            spec,
            max_iterations,
            no_commit,
        } => {
            run_single(&project_root, spec, max_iterations, !no_commit).await?;
        }
        Commands::Loop {
            max_iterations,
            max_specs,
            fail_streak,
            no_commit,
        } => {
            run_loop(&project_root, max_iterations, max_specs, fail_streak, !no_commit).await?;
        }
        Commands::Status => {
            show_status(&specs_dir)?;
        }
        Commands::List { incomplete, phase } => {
            list_specs(&specs_dir, incomplete, phase)?;
        }
        Commands::Next => {
            show_next(&specs_dir)?;
        }
        Commands::Validate => {
            validate_specs(&specs_dir)?;
        }
    }

    Ok(())
}

/// Run the Ralph loop once for a single spec
async fn run_single(
    project_root: &PathBuf,
    spec_id: Option<u32>,
    max_iterations: usize,
    auto_commit: bool,
) -> Result<()> {
    let specs_dir = project_root.join("specs");

    // Find the spec to implement
    let spec = if let Some(id) = spec_id {
        // Find specific spec
        let entries = parse_readme(&specs_dir)?;
        entries
            .into_iter()
            .find(|e| e.id == id)
            .map(|entry| spec_parser::parse_spec(&entry))
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("Spec {} not found", id))?
    } else {
        // Find next uncompleted
        find_next_spec(&specs_dir)?.ok_or_else(|| anyhow::anyhow!("All specs are complete!"))?
    };

    println!("\n========================================");
    println!("  RALPH LOOP - Spec {:03}: {}", spec.entry.id, spec.entry.name);
    println!("  Phase {}: {}", spec.entry.phase, spec.entry.phase_name);
    println!("========================================\n");

    // Show acceptance criteria
    println!("Acceptance Criteria:");
    for ac in &spec.acceptance_criteria {
        let mark = if ac.completed { "x" } else { " " };
        println!("  [{}] {}", mark, ac.text);
    }
    println!();

    // Get API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY environment variable not set")?;

    // Build the system prompt
    let system_prompt = build_system_prompt(project_root);

    // Build the task prompt
    let task_prompt = build_task_prompt(&spec);

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
        .run_agentic_loop(&system_prompt, &task_prompt, max_iterations, Some(tx))
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
    println!("========================================\n");

    // Auto-commit if enabled and there are changes
    if auto_commit {
        if let Some(hash) = git::auto_commit_spec(project_root, spec.entry.id, &spec.entry.name)? {
            println!("Committed changes as {}", hash);
        }
    }

    Ok(())
}

/// Run the Ralph loop continuously
async fn run_loop(
    project_root: &PathBuf,
    max_iterations: usize,
    max_specs: Option<usize>,
    fail_streak_limit: usize,
    auto_commit: bool,
) -> Result<()> {
    let specs_dir = project_root.join("specs");

    let mut specs_completed = 0;
    let mut consecutive_failures = 0;
    let mut total_cost = 0.0;

    println!("\n========================================");
    println!("  RALPH LOOP - CONTINUOUS MODE");
    println!("  Max iterations per spec: {}", max_iterations);
    println!("  Fail streak limit: {}", fail_streak_limit);
    println!("========================================\n");

    loop {
        // Check if we've hit max specs
        if let Some(max) = max_specs {
            if specs_completed >= max {
                println!("Reached max specs limit ({})", max);
                break;
            }
        }

        // Find next spec
        let spec = match find_next_spec(&specs_dir)? {
            Some(s) => s,
            None => {
                println!("\nAll specs are complete!");
                break;
            }
        };

        println!("\n--- Starting Spec {:03}: {} ---\n", spec.entry.id, spec.entry.name);

        // Run for this spec
        match run_single(project_root, Some(spec.entry.id), max_iterations, auto_commit).await {
            Ok(_) => {
                specs_completed += 1;
                consecutive_failures = 0;
                println!("\nSpec {:03} completed successfully!", spec.entry.id);
            }
            Err(e) => {
                consecutive_failures += 1;
                tracing::error!("Spec {:03} failed: {}", spec.entry.id, e);
                println!("\nSpec {:03} FAILED: {}", spec.entry.id, e);

                if consecutive_failures >= fail_streak_limit {
                    println!(
                        "\nStopping: {} consecutive failures (limit: {})",
                        consecutive_failures, fail_streak_limit
                    );
                    break;
                }
            }
        }

        // Brief pause between specs
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    println!("\n========================================");
    println!("  LOOP SESSION COMPLETE");
    println!("  Specs completed: {}", specs_completed);
    println!("========================================\n");

    Ok(())
}

/// Show current progress
fn show_status(specs_dir: &PathBuf) -> Result<()> {
    let summary = get_progress_summary(specs_dir)?;

    println!("\n========================================");
    println!("  TACHIKOMA PROGRESS");
    println!("========================================");
    println!(
        "  Specs: {}/{} ({:.1}%)",
        summary.completed_specs,
        summary.total_specs,
        summary.spec_percentage()
    );
    println!(
        "  Criteria: {}/{} ({:.1}%)",
        summary.completed_criteria,
        summary.total_criteria,
        summary.criteria_percentage()
    );
    println!("========================================\n");

    Ok(())
}

/// List all specs
fn list_specs(specs_dir: &PathBuf, incomplete_only: bool, phase_filter: Option<u32>) -> Result<()> {
    let entries = parse_readme(specs_dir)?;

    let mut current_phase = None;

    for entry in entries {
        // Filter by phase
        if let Some(p) = phase_filter {
            if entry.phase != p {
                continue;
            }
        }

        // Parse to check completion
        let parsed = spec_parser::parse_spec(&entry);
        let is_complete = parsed.as_ref().map(|p| p.all_complete).unwrap_or(false);

        // Filter incomplete
        if incomplete_only && is_complete {
            continue;
        }

        // Print phase header
        if current_phase != Some(entry.phase) {
            current_phase = Some(entry.phase);
            println!("\n## Phase {}: {}\n", entry.phase, entry.phase_name);
        }

        let status = if is_complete { "[x]" } else { "[ ]" };
        println!("  {} {:03} - {}", status, entry.id, entry.name);
    }

    println!();
    Ok(())
}

/// Show next spec to implement
fn show_next(specs_dir: &PathBuf) -> Result<()> {
    match find_next_spec(specs_dir)? {
        Some(spec) => {
            println!("\n========================================");
            println!("  NEXT SPEC");
            println!("========================================");
            println!("  ID: {:03}", spec.entry.id);
            println!("  Name: {}", spec.entry.name);
            println!("  Phase: {} - {}", spec.entry.phase, spec.entry.phase_name);
            println!("  Path: {}", spec.entry.path.display());
            println!("\n  Acceptance Criteria:");
            for ac in &spec.acceptance_criteria {
                let mark = if ac.completed { "x" } else { " " };
                println!("    [{}] {}", mark, ac.text);
            }
            println!("========================================\n");
        }
        None => {
            println!("\nAll specs are complete!");
        }
    }

    Ok(())
}

/// Validate specs structure
fn validate_specs(specs_dir: &PathBuf) -> Result<()> {
    let entries = parse_readme(specs_dir)?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for entry in &entries {
        // Check file exists
        if !entry.path.exists() {
            errors.push(format!("Spec {:03} file not found: {}", entry.id, entry.path.display()));
            continue;
        }

        // Parse and check structure
        match spec_parser::parse_spec(entry) {
            Ok(parsed) => {
                if parsed.acceptance_criteria.is_empty() {
                    warnings.push(format!(
                        "Spec {:03} has no acceptance criteria checkboxes",
                        entry.id
                    ));
                }
            }
            Err(e) => {
                errors.push(format!("Spec {:03} parse error: {}", entry.id, e));
            }
        }
    }

    println!("\n========================================");
    println!("  SPEC VALIDATION");
    println!("========================================");
    println!("  Total specs: {}", entries.len());
    println!("  Errors: {}", errors.len());
    println!("  Warnings: {}", warnings.len());

    if !errors.is_empty() {
        println!("\nERRORS:");
        for e in &errors {
            println!("  - {}", e);
        }
    }

    if !warnings.is_empty() {
        println!("\nWARNINGS:");
        for w in &warnings {
            println!("  - {}", w);
        }
    }

    println!("========================================\n");

    if !errors.is_empty() {
        anyhow::bail!("{} validation errors found", errors.len());
    }

    Ok(())
}

/// Build the system prompt for Claude
fn build_system_prompt(project_root: &PathBuf) -> String {
    format!(
        r#"You are a Tachikoma - a curious, helpful AI coding assistant implementing the Tachikoma project.

## Core Behaviors

1. **One mission per context** - Focus on the current spec only
2. **Study specs first** - Always read relevant specs before coding
3. **Follow patterns** - Use existing code patterns in the codebase
4. **Test everything** - Write tests, run tests, fix tests
5. **Update plans** - Mark checkboxes when tasks complete

## Project Root
{}

## Available Tools
You have access to five primitives:
- read_file: Read file contents
- list_files: List directory contents
- bash: Execute shell commands (with timeout)
- edit_file: Modify files (old_string must be unique)
- code_search: Search codebase with ripgrep

## Important Rules

1. Read the spec file FIRST before doing anything
2. Check existing code patterns before writing new code
3. Make small, focused changes
4. Run tests after making changes
5. When a task is complete, update the checkbox in the spec file
6. Use edit_file to mark checkboxes: change "- [ ]" to "- [x]"

## On Getting Stuck

1. Reduce scope - focus on one criterion at a time
2. Re-read the spec
3. Search for similar patterns in the codebase
4. If truly stuck, explain what's blocking you

## Completion

When ALL acceptance criteria are met:
1. Verify each criterion is satisfied
2. Update all checkboxes to [x]
3. Summarize what was implemented
"#,
        project_root.display()
    )
}

/// Build the task prompt for a specific spec
fn build_task_prompt(spec: &ParsedSpec) -> String {
    let incomplete: Vec<_> = spec
        .acceptance_criteria
        .iter()
        .filter(|ac| !ac.completed)
        .map(|ac| format!("- [ ] {}", ac.text))
        .collect();

    format!(
        r#"## Mission: Implement Spec {:03} - {}

### Spec File
{}

### Remaining Tasks
{}

### Instructions

1. First, read the spec file to understand the full requirements:
   read_file path="{}"

2. Study any referenced patterns or existing code

3. Implement each unchecked acceptance criterion

4. After implementing each criterion:
   - Verify it works (run tests if applicable)
   - Update the checkbox in the spec file from "- [ ]" to "- [x]"

5. When ALL criteria are complete, summarize what was done

Begin by reading the spec file.
"#,
        spec.entry.id,
        spec.entry.name,
        spec.entry.path.display(),
        incomplete.join("\n"),
        spec.entry.path.display()
    )
}
