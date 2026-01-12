# Spec 088: Init Scaffolding

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 088
- **Status**: Planned
- **Dependencies**: 076-cli-crate, 083-cli-prompts
- **Estimated Context**: ~10%

## Objective

Implement the `tachikoma init` command for scaffolding new Tachikoma projects with proper directory structure, configuration files, and starter code.

## Acceptance Criteria

- [x] Create project directory structure
- [x] Generate tachikoma.toml configuration
- [x] Create Cargo.toml for Rust projects
- [x] Support multiple project templates
- [x] Interactive wizard mode
- [x] Git repository initialization
- [x] .gitignore generation
- [x] Validate project name
- [x] Prevent overwriting existing projects

## Implementation Details

### src/commands/init.rs

```rust
//! Project initialization and scaffolding.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use clap::Args;

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;
use crate::output::Output;
use crate::output::color::{Styled, Color};
use crate::prompts::{confirm, input, select, is_interactive};

/// Initialize a new Tachikoma project
#[derive(Debug, Args)]
pub struct InitCommand {
    /// Project name (creates directory if path not specified)
    pub name: Option<String>,

    /// Directory to initialize (default: current directory or name)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Project template to use
    #[arg(short, long, value_enum, default_value = "basic")]
    pub template: ProjectTemplate,

    /// Skip interactive prompts
    #[arg(long)]
    pub no_prompt: bool,

    /// Initialize git repository
    #[arg(long, default_value = "true")]
    pub git: bool,

    /// Force initialization even if directory exists
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum ProjectTemplate {
    /// Basic agent with minimal configuration
    #[default]
    Basic,
    /// Agent with MCP tools integration
    Tools,
    /// Multi-agent workflow project
    Workflow,
    /// Chat-style conversational agent
    Chat,
    /// Minimal template for advanced users
    Minimal,
}

impl std::fmt::Display for ProjectTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic => write!(f, "basic"),
            Self::Tools => write!(f, "tools"),
            Self::Workflow => write!(f, "workflow"),
            Self::Chat => write!(f, "chat"),
            Self::Minimal => write!(f, "minimal"),
        }
    }
}

/// Project configuration collected from user
#[derive(Debug)]
struct ProjectConfig {
    name: String,
    path: PathBuf,
    template: ProjectTemplate,
    description: String,
    author: Option<String>,
    license: String,
    default_backend: Option<String>,
    init_git: bool,
}

#[async_trait]
impl Execute for InitCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        let output = Output::new(ctx);

        // Determine project configuration
        let config = if self.no_prompt {
            self.build_config_non_interactive()?
        } else {
            self.build_config_interactive().await?
        };

        // Validate
        validate_project_name(&config.name)?;

        // Check directory
        if config.path.exists() && !self.force {
            let has_content = std::fs::read_dir(&config.path)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);

            if has_content {
                return Err(CliError::InvalidArgument(format!(
                    "Directory '{}' is not empty. Use --force to initialize anyway.",
                    config.path.display()
                )));
            }
        }

        // Create project
        println!("\nCreating project '{}'...\n", config.name);

        // Create directory structure
        create_directory_structure(&config).await?;

        // Generate files
        generate_project_files(&config).await?;

        // Initialize git if requested
        if config.init_git {
            init_git_repository(&config.path).await?;
        }

        // Print success message
        println!();
        output.success(&format!("Created project '{}' at {}", config.name, config.path.display()));

        println!("\nNext steps:");
        println!("  cd {}", config.path.display());
        println!("  tachikoma doctor          # Check system requirements");
        println!("  tachikoma backends add    # Configure an AI backend");
        println!("  cargo build               # Build the project");

        Ok(())
    }
}

impl InitCommand {
    fn build_config_non_interactive(&self) -> Result<ProjectConfig, CliError> {
        let name = self.name.clone().unwrap_or_else(|| "my-agent".to_string());
        let path = self.path.clone().unwrap_or_else(|| PathBuf::from(&name));

        Ok(ProjectConfig {
            name,
            path,
            template: self.template,
            description: "A Tachikoma AI agent".to_string(),
            author: get_git_user(),
            license: "MIT".to_string(),
            default_backend: None,
            init_git: self.git,
        })
    }

    async fn build_config_interactive(&self) -> Result<ProjectConfig, CliError> {
        println!("{}", Styled::new("\nTachikoma Project Initialization").bold());
        println!("{}", Styled::new("================================\n").fg(Color::BrightBlack));

        // Project name
        let default_name = self.name.clone().unwrap_or_else(|| "my-agent".to_string());
        let name = input(&format!("Project name ({})", default_name))?;
        let name = if name.is_empty() { default_name } else { name };

        // Validate name
        validate_project_name(&name)?;

        // Path
        let default_path = self.path.clone().unwrap_or_else(|| PathBuf::from(&name));
        let path_str = input(&format!("Project path ({})", default_path.display()))?;
        let path = if path_str.is_empty() {
            default_path
        } else {
            PathBuf::from(path_str)
        };

        // Template
        let templates = vec![
            "Basic - Simple agent with minimal setup",
            "Tools - Agent with MCP tools integration",
            "Workflow - Multi-agent workflow project",
            "Chat - Conversational chat agent",
            "Minimal - Bare minimum for advanced users",
        ];
        let template_idx = select("Project template", templates)?;
        let template = match template_idx {
            0 => ProjectTemplate::Basic,
            1 => ProjectTemplate::Tools,
            2 => ProjectTemplate::Workflow,
            3 => ProjectTemplate::Chat,
            _ => ProjectTemplate::Minimal,
        };

        // Description
        let description = input("Project description (A Tachikoma AI agent)")?;
        let description = if description.is_empty() {
            "A Tachikoma AI agent".to_string()
        } else {
            description
        };

        // Author
        let default_author = get_git_user();
        let author_prompt = match &default_author {
            Some(a) => format!("Author ({})", a),
            None => "Author".to_string(),
        };
        let author_input = input(&author_prompt)?;
        let author = if author_input.is_empty() {
            default_author
        } else {
            Some(author_input)
        };

        // Backend
        let backend_choices = vec![
            "Anthropic (Claude)",
            "OpenAI (GPT)",
            "Ollama (Local)",
            "None - Configure later",
        ];
        let backend_idx = select("Default AI backend", backend_choices)?;
        let default_backend = match backend_idx {
            0 => Some("anthropic".to_string()),
            1 => Some("openai".to_string()),
            2 => Some("ollama".to_string()),
            _ => None,
        };

        // Git
        let init_git = if self.git {
            confirm("Initialize git repository?")?
        } else {
            false
        };

        Ok(ProjectConfig {
            name,
            path,
            template,
            description,
            author,
            license: "MIT".to_string(),
            default_backend,
            init_git,
        })
    }
}

fn validate_project_name(name: &str) -> Result<(), CliError> {
    if name.is_empty() {
        return Err(CliError::InvalidArgument("Project name cannot be empty".to_string()));
    }

    if !name.chars().next().unwrap().is_alphabetic() {
        return Err(CliError::InvalidArgument(
            "Project name must start with a letter".to_string(),
        ));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(CliError::InvalidArgument(
            "Project name can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }

    // Check for reserved names
    let reserved = ["test", "main", "lib", "src", "target", "build"];
    if reserved.contains(&name.to_lowercase().as_str()) {
        return Err(CliError::InvalidArgument(format!(
            "'{name}' is a reserved name"
        )));
    }

    Ok(())
}

fn get_git_user() -> Option<String> {
    let name = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let email = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    match (name, email) {
        (Some(n), Some(e)) if !n.is_empty() && !e.is_empty() => Some(format!("{n} <{e}>")),
        (Some(n), _) if !n.is_empty() => Some(n),
        _ => None,
    }
}

async fn create_directory_structure(config: &ProjectConfig) -> Result<(), CliError> {
    let dirs = match config.template {
        ProjectTemplate::Basic => vec![
            "",
            "src",
            "config",
            "prompts",
        ],
        ProjectTemplate::Tools => vec![
            "",
            "src",
            "config",
            "prompts",
            "tools",
        ],
        ProjectTemplate::Workflow => vec![
            "",
            "src",
            "src/agents",
            "src/workflows",
            "config",
            "prompts",
        ],
        ProjectTemplate::Chat => vec![
            "",
            "src",
            "config",
            "prompts",
            "history",
        ],
        ProjectTemplate::Minimal => vec![
            "",
            "src",
        ],
    };

    for dir in dirs {
        let path = config.path.join(dir);
        std::fs::create_dir_all(&path)?;
        println!("  {} {}/", Styled::new("create").fg(Color::Green), path.display());
    }

    Ok(())
}

async fn generate_project_files(config: &ProjectConfig) -> Result<(), CliError> {
    // Generate tachikoma.toml
    let tachikoma_toml = generate_tachikoma_toml(config);
    write_file(&config.path.join("tachikoma.toml"), &tachikoma_toml)?;

    // Generate Cargo.toml
    let cargo_toml = generate_cargo_toml(config);
    write_file(&config.path.join("Cargo.toml"), &cargo_toml)?;

    // Generate main.rs
    let main_rs = generate_main_rs(config);
    write_file(&config.path.join("src/main.rs"), &main_rs)?;

    // Generate .gitignore
    let gitignore = generate_gitignore();
    write_file(&config.path.join(".gitignore"), &gitignore)?;

    // Template-specific files
    match config.template {
        ProjectTemplate::Tools => {
            let tools_config = generate_tools_config();
            write_file(&config.path.join("config/tools.toml"), &tools_config)?;
        }
        ProjectTemplate::Workflow => {
            let workflow_rs = generate_workflow_rs();
            write_file(&config.path.join("src/workflows/mod.rs"), &workflow_rs)?;

            let agents_rs = generate_agents_rs();
            write_file(&config.path.join("src/agents/mod.rs"), &agents_rs)?;
        }
        ProjectTemplate::Chat => {
            let chat_prompt = generate_chat_prompt();
            write_file(&config.path.join("prompts/system.md"), &chat_prompt)?;
        }
        _ => {}
    }

    // Generate README
    let readme = generate_readme(config);
    write_file(&config.path.join("README.md"), &readme)?;

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<(), CliError> {
    std::fs::write(path, content)?;
    println!("  {} {}", Styled::new("create").fg(Color::Green), path.display());
    Ok(())
}

fn generate_tachikoma_toml(config: &ProjectConfig) -> String {
    let backend_section = config.default_backend.as_ref().map(|b| {
        format!(r#"
[backends.default]
type = "{b}"
# api_key = "" # Set via environment variable
"#)
    }).unwrap_or_default();

    format!(r#"# Tachikoma Project Configuration

[project]
name = "{name}"
version = "0.1.0"
description = "{description}"

[agent]
# Default model to use
model = "claude-sonnet-4-20250514"

# Maximum tokens for responses
max_tokens = 4096

# Temperature for responses (0.0-1.0)
temperature = 0.7
{backend_section}
[logging]
level = "info"
"#,
        name = config.name,
        description = config.description,
    )
}

fn generate_cargo_toml(config: &ProjectConfig) -> String {
    let author_line = config.author.as_ref()
        .map(|a| format!("authors = [\"{a}\"]"))
        .unwrap_or_default();

    format!(r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
{author_line}
description = "{description}"
license = "{license}"

[dependencies]
tachikoma = "0.1"
tokio = {{ version = "1.40", features = ["full"] }}
tracing = "0.1"
tracing-subscriber = "0.3"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
"#,
        name = config.name,
        description = config.description,
        license = config.license,
    )
}

fn generate_main_rs(config: &ProjectConfig) -> String {
    match config.template {
        ProjectTemplate::Basic => r#"//! Basic Tachikoma agent

use anyhow::Result;
use tachikoma::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::init();

    // Load configuration
    let config = Config::load()?;

    // Create agent
    let agent = Agent::builder()
        .config(config)
        .build()
        .await?;

    // Run the agent
    let response = agent
        .prompt("Hello! What can you help me with?")
        .await?;

    println!("{}", response);

    Ok(())
}
"#.to_string(),

        ProjectTemplate::Tools => r#"//! Tachikoma agent with MCP tools

use anyhow::Result;
use tachikoma::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let config = Config::load()?;

    // Create agent with tools
    let agent = Agent::builder()
        .config(config)
        .with_tools_from_config()
        .await?
        .build()
        .await?;

    // The agent can now use configured tools
    let response = agent
        .prompt("List the files in the current directory")
        .await?;

    println!("{}", response);

    Ok(())
}
"#.to_string(),

        ProjectTemplate::Workflow => r#"//! Multi-agent workflow

mod agents;
mod workflows;

use anyhow::Result;
use tachikoma::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let config = Config::load()?;

    // Create workflow
    let workflow = workflows::create_workflow(&config).await?;

    // Execute workflow
    let result = workflow.run().await?;

    println!("Workflow completed: {:?}", result);

    Ok(())
}
"#.to_string(),

        ProjectTemplate::Chat => r#"//! Chat-style conversational agent

use anyhow::Result;
use tachikoma::prelude::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let config = Config::load()?;

    // Create chat session
    let mut session = ChatSession::builder()
        .config(config)
        .system_prompt_file("prompts/system.md")
        .build()
        .await?;

    println!("Chat started. Type 'quit' to exit.\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input == "quit" || input == "exit" {
            break;
        }

        let response = session.send(input).await?;
        println!("\n{}\n", response);
    }

    Ok(())
}
"#.to_string(),

        ProjectTemplate::Minimal => r#"//! Minimal Tachikoma agent

use tachikoma::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let agent = Agent::new().await?;
    let response = agent.prompt("Hello!").await?;
    println!("{}", response);
    Ok(())
}
"#.to_string(),
    }
}

fn generate_gitignore() -> String {
    r#"# Build artifacts
/target/
Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo

# Environment
.env
.env.local
*.env

# Logs
*.log
logs/

# OS
.DS_Store
Thumbs.db

# Tachikoma
.tachikoma/
*.history
"#.to_string()
}

fn generate_tools_config() -> String {
    r#"# MCP Tools Configuration

[[tools]]
name = "filesystem"
enabled = true

[[tools]]
name = "shell"
enabled = false  # Enable with caution
"#.to_string()
}

fn generate_workflow_rs() -> String {
    r#"//! Workflow definitions

use anyhow::Result;
use tachikoma::prelude::*;

pub async fn create_workflow(config: &Config) -> Result<Workflow> {
    Workflow::builder()
        .name("default")
        .add_step("analyze", analyze_step)
        .add_step("process", process_step)
        .add_step("summarize", summarize_step)
        .build()
}

async fn analyze_step(ctx: &mut Context) -> Result<()> {
    // Analysis logic
    Ok(())
}

async fn process_step(ctx: &mut Context) -> Result<()> {
    // Processing logic
    Ok(())
}

async fn summarize_step(ctx: &mut Context) -> Result<()> {
    // Summarization logic
    Ok(())
}
"#.to_string()
}

fn generate_agents_rs() -> String {
    r#"//! Agent definitions

use anyhow::Result;
use tachikoma::prelude::*;

pub async fn create_analyzer(config: &Config) -> Result<Agent> {
    Agent::builder()
        .name("analyzer")
        .config(config.clone())
        .system_prompt("You are an analytical assistant.")
        .build()
        .await
}

pub async fn create_processor(config: &Config) -> Result<Agent> {
    Agent::builder()
        .name("processor")
        .config(config.clone())
        .system_prompt("You are a data processing assistant.")
        .build()
        .await
}
"#.to_string()
}

fn generate_chat_prompt() -> String {
    r#"# System Prompt

You are a helpful AI assistant. Be concise, accurate, and friendly.

## Guidelines

- Answer questions directly and accurately
- Ask for clarification when needed
- Be honest about limitations
- Maintain a helpful and professional tone
"#.to_string()
}

fn generate_readme(config: &ProjectConfig) -> String {
    format!(r#"# {name}

{description}

## Getting Started

```bash
# Install dependencies
cargo build

# Configure backend
tachikoma backends add

# Run the agent
cargo run
```

## Configuration

Edit `tachikoma.toml` to customize the agent configuration.

## License

{license}
"#,
        name = config.name,
        description = config.description,
        license = config.license,
    )
}

async fn init_git_repository(path: &Path) -> Result<(), CliError> {
    let status = tokio::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .await?;

    if status.status.success() {
        println!("  {} git repository", Styled::new("init").fg(Color::Green));
    }

    Ok(())
}
```

## Testing Requirements

### Integration Tests

```rust
// tests/init_cmd.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_init_creates_project() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("tachikoma")
        .unwrap()
        .current_dir(&dir)
        .args(["init", "test-project", "--no-prompt"])
        .assert()
        .success();

    assert!(dir.path().join("test-project").exists());
    assert!(dir.path().join("test-project/tachikoma.toml").exists());
    assert!(dir.path().join("test-project/Cargo.toml").exists());
    assert!(dir.path().join("test-project/src/main.rs").exists());
}

#[test]
fn test_init_invalid_name() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("tachikoma")
        .unwrap()
        .current_dir(&dir)
        .args(["init", "123invalid", "--no-prompt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must start with a letter"));
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **083-cli-prompts.md**: Interactive prompts
- **089-cli-init-templates.md**: Template system
