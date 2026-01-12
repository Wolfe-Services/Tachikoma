//! Tachikoma specification system.
//!
//! This crate provides the core functionality for managing Tachikoma specifications,
//! including directory structure discovery, parsing, validation, and template-based generation.

pub mod checkbox;
pub mod directory;
pub mod impl_plan;
pub mod metadata;
pub mod parsing;
pub mod progress;
pub mod readme;
pub mod templates;
pub mod watcher;

pub use checkbox::*;
pub use directory::*;
pub use impl_plan::*;
pub use metadata::*;
pub use parsing::*;
pub use progress::*;
pub use readme::*;
pub use templates::*;
pub use watcher::*;