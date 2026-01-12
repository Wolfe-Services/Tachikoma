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

// Re-exports
pub use context::{PrimitiveConfig, PrimitiveContext};
pub use error::{PrimitiveError, PrimitiveResult};
pub use result::{ExecutionMetadata, ReadFileResult};

#[cfg(feature = "read-file")]
pub use read_file::{read_file, ReadFileOptions};