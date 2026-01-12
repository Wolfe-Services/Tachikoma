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