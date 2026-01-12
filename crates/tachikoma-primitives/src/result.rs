//! Result types for primitive operations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Metadata about primitive execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Time taken to execute.
    pub duration: Duration,
    /// Operation ID.
    pub operation_id: String,
    /// Primitive name.
    pub primitive: String,
}

/// Result of a read_file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileResult {
    /// File content.
    pub content: String,
    /// File path.
    pub path: PathBuf,
    /// File size in bytes.
    pub size: usize,
    /// Whether content was truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of a list_files operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesResult {
    /// List of file entries.
    pub entries: Vec<FileEntry>,
    /// Base directory.
    pub base_path: PathBuf,
    /// Total files found.
    pub total_count: usize,
    /// Whether results were truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// A file entry in list results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// File path.
    pub path: PathBuf,
    /// Whether it's a directory.
    pub is_dir: bool,
    /// File size (None for directories).
    pub size: Option<u64>,
    /// File extension.
    pub extension: Option<String>,
    /// Modified time (UNIX timestamp).
    pub modified: Option<u64>,
}

/// Result of a bash operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashResult {
    /// Exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Whether the command timed out.
    pub timed_out: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of an edit_file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFileResult {
    /// Whether the edit was successful.
    pub success: bool,
    /// Number of replacements made.
    pub replacements: usize,
    /// File path.
    pub path: PathBuf,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// Result of a code_search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSearchResult {
    /// Search matches.
    pub matches: Vec<SearchMatch>,
    /// Pattern used.
    pub pattern: String,
    /// Total matches found.
    pub total_count: usize,
    /// Whether results were truncated.
    pub truncated: bool,
    /// Execution metadata.
    pub metadata: ExecutionMetadata,
}

/// A single search match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    /// File path.
    pub path: PathBuf,
    /// Line number (1-indexed).
    pub line_number: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// Matched line content.
    pub line_content: String,
    /// Context lines before.
    pub context_before: Vec<String>,
    /// Context lines after.
    pub context_after: Vec<String>,
}