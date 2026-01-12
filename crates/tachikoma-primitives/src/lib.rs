//! Tachikoma primitives - core agent operations.
//!
//! This crate provides the five primitives that form the foundation
//! of Tachikoma's agent capabilities:
//!
//! - `read_file` - Read file contents
//! - `list_files` - List directory contents
//! - `bash` - Execute shell commands
//! - `edit_file` - Search and replace in files
//! - `code_search` - Search code with ripgrep

#![warn(missing_docs)]

pub mod context;
pub mod error;
pub mod result;

#[cfg(feature = "read-file")]
pub mod read_file;

#[cfg(feature = "list-files")]
pub mod list_files;

#[cfg(feature = "bash")]
pub mod bash;

#[cfg(feature = "edit-file")]
pub mod edit_file;

// Re-exports
pub use context::{PrimitiveConfig, PrimitiveContext};
pub use error::{PrimitiveError, PrimitiveResult};
pub use result::{ExecutionMetadata, ReadFileResult, ListFilesResult, FileEntry, BashResult, EditFileResult};

#[cfg(feature = "read-file")]
pub use read_file::{read_file, ReadFileOptions};

#[cfg(feature = "list-files")]
pub use list_files::{list_files, ListFilesOptions, SortBy, list_files_recursive, list_files_recursive_with_callback, RecursiveOptions, RecursiveIterator};

#[cfg(feature = "bash")]
pub use bash::{bash, bash_success, bash_sequence, bash_with_timeout, BashOptions, TimeoutCommand, CancellationToken, CancellationWatcher};

#[cfg(feature = "edit-file")]
pub use edit_file::{
    edit_file, edit_file_preview, EditFileOptions, Diff, EditPreview,
    UniquenessResult, MatchLocation, MatchSelection, EditValidationError,
    check_uniqueness, format_matches, select_match, validate_edit_target
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Integration test covering all major features.
    #[cfg(feature = "bash")]
    #[tokio::test]
    async fn test_bash_integration() {
        let ctx = PrimitiveContext::new(PathBuf::from("/tmp"));
        
        // Test basic functionality
        let result = bash(&ctx, "echo 'integration test'", None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "integration test");
        assert!(result.stderr.is_empty());
        assert!(!result.timed_out);
        
        // Test with all options
        let opts = BashOptions::new()
            .env("TEST_VAR", "test_value")
            .working_dir("/tmp")
            .timeout(std::time::Duration::from_secs(5));
            
        let result = bash(&ctx, "echo $TEST_VAR && pwd", Some(opts)).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("test_value"));
        assert!(result.stdout.contains("/tmp") || result.stdout.contains("/private/tmp"));
        
        // Test error handling
        let opts = BashOptions::new().block_command("dangerous_command");
        let result = bash(&ctx, "dangerous_command", Some(opts)).await;
        assert!(result.is_err());
        
        // Test sequence functionality
        let commands = ["echo 'first'", "echo 'second'"];
        let results = bash_sequence(&ctx, &commands, None).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].stdout.trim(), "first");
        assert_eq!(results[1].stdout.trim(), "second");
    }
}