//! Tachikoma specification system.
//!
//! This crate provides the core functionality for managing Tachikoma specifications,
//! including directory structure discovery, parsing, validation, and template-based generation.

pub mod directory;
pub mod readme;
pub mod templates;
pub mod watcher;

pub use directory::*;
pub use readme::*;
pub use templates::*;
pub use watcher::*;