# Spec 077: CLI Argument Parsing

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 077
- **Status**: Planned
- **Dependencies**: 076-cli-crate
- **Estimated Context**: ~10%

## Objective

Implement comprehensive argument parsing patterns using clap derive macros, supporting various argument types, validation, environment variables, and value hints for shell completion.

## Acceptance Criteria

- [ ] Positional arguments with validation
- [ ] Optional and required flags
- [ ] Arguments with default values
- [ ] Environment variable fallbacks
- [ ] Value hints for completion
- [ ] Custom value parsers
- [ ] Argument groups and conflicts
- [ ] Repeated arguments (vectors)
- [ ] Subcommand-specific arguments

## Implementation Details

### src/args/mod.rs

```rust
//! Argument parsing utilities and common argument types.

mod parsers;
mod validators;

pub use parsers::*;
pub use validators::*;

use std::path::PathBuf;
use clap::{Args, ValueHint};

/// Common path-related arguments
#[derive(Debug, Clone, Args)]
pub struct PathArgs {
    /// Target directory (defaults to current directory)
    #[arg(
        short = 'd',
        long,
        default_value = ".",
        value_hint = ValueHint::DirPath,
        help = "Target directory"
    )]
    pub directory: PathBuf,

    /// Output file path
    #[arg(
        short = 'o',
        long,
        value_hint = ValueHint::FilePath,
        help = "Output file path"
    )]
    pub output: Option<PathBuf>,
}

/// Common filter arguments
#[derive(Debug, Clone, Args)]
pub struct FilterArgs {
    /// Filter by name pattern (glob)
    #[arg(
        short = 'n',
        long,
        help = "Filter by name pattern"
    )]
    pub name: Option<String>,

    /// Filter by tag
    #[arg(
        short = 't',
        long,
        action = clap::ArgAction::Append,
        help = "Filter by tag (can be repeated)"
    )]
    pub tags: Vec<String>,

    /// Include disabled items
    #[arg(
        long,
        help = "Include disabled items"
    )]
    pub include_disabled: bool,
}

/// Pagination arguments
#[derive(Debug, Clone, Args)]
pub struct PaginationArgs {
    /// Maximum number of items to return
    #[arg(
        short = 'l',
        long,
        default_value = "25",
        value_parser = clap::value_parser!(u32).range(1..=1000),
        help = "Maximum items to return"
    )]
    pub limit: u32,

    /// Number of items to skip
    #[arg(
        short = 's',
        long,
        default_value = "0",
        help = "Number of items to skip"
    )]
    pub offset: u32,
}

/// Network-related arguments
#[derive(Debug, Clone, Args)]
pub struct NetworkArgs {
    /// Request timeout in seconds
    #[arg(
        long,
        default_value = "30",
        value_parser = parse_duration_secs,
        help = "Request timeout in seconds"
    )]
    pub timeout: std::time::Duration,

    /// Number of retries
    #[arg(
        long,
        default_value = "3",
        value_parser = clap::value_parser!(u8).range(0..=10),
        help = "Number of retry attempts"
    )]
    pub retries: u8,

    /// Disable TLS verification (dangerous)
    #[arg(
        long,
        help = "Disable TLS certificate verification"
    )]
    pub insecure: bool,
}

/// Authentication arguments
#[derive(Debug, Clone, Args)]
pub struct AuthArgs {
    /// API key for authentication
    #[arg(
        long,
        env = "TACHIKOMA_API_KEY",
        value_hint = ValueHint::Other,
        help = "API key for authentication"
    )]
    pub api_key: Option<String>,

    /// Authentication token
    #[arg(
        long,
        env = "TACHIKOMA_TOKEN",
        help = "Bearer token for authentication"
    )]
    pub token: Option<String>,
}
```

### src/args/parsers.rs

```rust
//! Custom value parsers for CLI arguments.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Parse a duration from seconds string
pub fn parse_duration_secs(s: &str) -> Result<Duration, String> {
    let secs: u64 = s
        .parse()
        .map_err(|_| format!("Invalid duration: {s}"))?;
    Ok(Duration::from_secs(secs))
}

/// Parse a duration with unit suffix (e.g., "30s", "5m", "1h")
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let (num, unit) = if s.ends_with("ms") {
        (&s[..s.len() - 2], "ms")
    } else if s.ends_with('s') {
        (&s[..s.len() - 1], "s")
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else if s.ends_with('d') {
        (&s[..s.len() - 1], "d")
    } else {
        (s, "s") // Default to seconds
    };

    let value: u64 = num
        .parse()
        .map_err(|_| format!("Invalid number: {num}"))?;

    let millis = match unit {
        "ms" => value,
        "s" => value * 1000,
        "m" => value * 60 * 1000,
        "h" => value * 60 * 60 * 1000,
        "d" => value * 24 * 60 * 60 * 1000,
        _ => return Err(format!("Unknown unit: {unit}")),
    };

    Ok(Duration::from_millis(millis))
}

/// Parse a key=value pair
pub fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key=value pair: {s}"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// Parse a URL with validation
pub fn parse_url(s: &str) -> Result<url::Url, String> {
    url::Url::parse(s).map_err(|e| format!("Invalid URL: {e}"))
}

/// Parse a semantic version
pub fn parse_semver(s: &str) -> Result<semver::Version, String> {
    semver::Version::parse(s).map_err(|e| format!("Invalid version: {e}"))
}

/// Parse a glob pattern
pub fn parse_glob(s: &str) -> Result<glob::Pattern, String> {
    glob::Pattern::new(s).map_err(|e| format!("Invalid glob pattern: {e}"))
}

/// Parse a size with unit suffix (e.g., "100KB", "1MB", "1GB")
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num, multiplier) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1)
    } else {
        (s.as_str(), 1)
    };

    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("Invalid size: {s}"))?;

    Ok(value * multiplier)
}

/// Parse a list of items separated by comma
pub fn parse_comma_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parser for model identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelId {
    pub provider: String,
    pub model: String,
}

impl FromStr for ModelId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((provider, model)) => Ok(ModelId {
                provider: provider.to_string(),
                model: model.to_string(),
            }),
            None => Ok(ModelId {
                provider: "default".to_string(),
                model: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider, self.model)
    }
}
```

### src/args/validators.rs

```rust
//! Argument validators.

use std::path::Path;

/// Validate that a path exists
pub fn validate_path_exists(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(format!("Path does not exist: {path}"))
    }
}

/// Validate that a path is a directory
pub fn validate_is_directory(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_dir() {
        Ok(())
    } else if p.exists() {
        Err(format!("Path is not a directory: {path}"))
    } else {
        Err(format!("Directory does not exist: {path}"))
    }
}

/// Validate that a path is a file
pub fn validate_is_file(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_file() {
        Ok(())
    } else if p.exists() {
        Err(format!("Path is not a file: {path}"))
    } else {
        Err(format!("File does not exist: {path}"))
    }
}

/// Validate a port number
pub fn validate_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| format!("Invalid port: {s}"))?;
    if port == 0 {
        Err("Port cannot be 0".to_string())
    } else {
        Ok(port)
    }
}

/// Validate an identifier (alphanumeric + underscore + hyphen)
pub fn validate_identifier(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if !s.chars().next().unwrap().is_alphabetic() {
        return Err("Identifier must start with a letter".to_string());
    }

    if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        Ok(s.to_string())
    } else {
        Err("Identifier can only contain letters, numbers, underscores, and hyphens".to_string())
    }
}

/// Validate a semantic version string
pub fn validate_semver(s: &str) -> Result<String, String> {
    semver::Version::parse(s)
        .map(|_| s.to_string())
        .map_err(|e| format!("Invalid semantic version: {e}"))
}
```

### Example Command with Various Arguments

```rust
//! Example command demonstrating various argument patterns.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::args::{
    parse_duration, parse_key_value, parse_size, validate_identifier,
    FilterArgs, NetworkArgs, PaginationArgs,
};

/// Example command showcasing argument patterns
#[derive(Debug, Parser)]
pub struct ExampleCommand {
    #[command(subcommand)]
    pub action: ExampleAction,
}

#[derive(Debug, Subcommand)]
pub enum ExampleAction {
    /// Create a new resource
    Create(CreateArgs),
    /// List resources
    List(ListArgs),
    /// Run a task
    Run(RunArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Resource name (required positional)
    #[arg(
        value_parser = validate_identifier,
        help = "Name of the resource to create"
    )]
    pub name: String,

    /// Resource type
    #[arg(
        short = 't',
        long,
        value_enum,
        default_value = "standard",
        help = "Type of resource"
    )]
    pub resource_type: ResourceType,

    /// Key-value properties
    #[arg(
        short = 'p',
        long = "property",
        value_parser = parse_key_value,
        action = clap::ArgAction::Append,
        help = "Property in key=value format (can be repeated)"
    )]
    pub properties: Vec<(String, String)>,

    /// Tags for the resource
    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated tags"
    )]
    pub tags: Vec<String>,

    /// Maximum size
    #[arg(
        long,
        value_parser = parse_size,
        default_value = "100MB",
        help = "Maximum size (e.g., 100KB, 1MB, 1GB)"
    )]
    pub max_size: u64,

    /// Don't prompt for confirmation
    #[arg(
        short = 'y',
        long,
        help = "Skip confirmation prompt"
    )]
    pub yes: bool,

    /// Dry run mode
    #[arg(
        long,
        help = "Show what would be done without making changes"
    )]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ResourceType {
    Standard,
    Premium,
    Custom,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(flatten)]
    pub filter: FilterArgs,

    #[command(flatten)]
    pub pagination: PaginationArgs,

    /// Sort field
    #[arg(
        long,
        default_value = "name",
        help = "Field to sort by"
    )]
    pub sort: String,

    /// Sort in descending order
    #[arg(
        long,
        help = "Sort in descending order"
    )]
    pub descending: bool,
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Script or command to run
    #[arg(
        required = true,
        help = "Script or command to execute"
    )]
    pub script: PathBuf,

    /// Arguments to pass to the script
    #[arg(
        last = true,
        help = "Arguments to pass to the script"
    )]
    pub args: Vec<String>,

    /// Working directory
    #[arg(
        short = 'w',
        long,
        help = "Working directory for execution"
    )]
    pub workdir: Option<PathBuf>,

    /// Timeout duration
    #[arg(
        long,
        value_parser = parse_duration,
        default_value = "5m",
        help = "Execution timeout (e.g., 30s, 5m, 1h)"
    )]
    pub timeout: std::time::Duration,

    /// Environment variables
    #[arg(
        short = 'e',
        long = "env",
        value_parser = parse_key_value,
        action = clap::ArgAction::Append,
        help = "Environment variable in KEY=VALUE format"
    )]
    pub env_vars: Vec<(String, String)>,

    #[command(flatten)]
    pub network: NetworkArgs,
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
    }

    #[test]
    fn test_parse_key_value() {
        let (k, v) = parse_key_value("key=value").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "value");

        let (k, v) = parse_key_value("key=val=ue").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "val=ue");

        assert!(parse_key_value("noequals").is_err());
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("100B").unwrap(), 100);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("valid_name").is_ok());
        assert!(validate_identifier("valid-name").is_ok());
        assert!(validate_identifier("ValidName123").is_ok());
        assert!(validate_identifier("123invalid").is_err());
        assert!(validate_identifier("").is_err());
        assert!(validate_identifier("invalid name").is_err());
    }

    #[test]
    fn test_model_id_parsing() {
        let id: ModelId = "anthropic/claude-3".parse().unwrap();
        assert_eq!(id.provider, "anthropic");
        assert_eq!(id.model, "claude-3");

        let id: ModelId = "gpt-4".parse().unwrap();
        assert_eq!(id.provider, "default");
        assert_eq!(id.model, "gpt-4");
    }
}
```

## Related Specs

- **076-cli-crate.md**: Base CLI structure
- **078-cli-subcommands.md**: Subcommand organization
- **090-cli-help.md**: Help system integration
- **093-cli-completions.md**: Shell completion generation
