//! Tachikoma specification system.
//!
//! This crate provides the core functionality for managing Tachikoma specifications,
//! including directory structure discovery, parsing, validation, and template-based generation.

pub mod checkbox;
pub mod citation;
pub mod directory;
pub mod impl_plan;
pub mod linting;
pub mod metadata;
pub mod parsing;
pub mod pattern_link;
pub mod progress;
pub mod readme;
pub mod rendering;
pub mod search_api;
pub mod search_index;
pub mod templates;
pub mod validation;
pub mod versioning;
pub mod watcher;

pub use checkbox::*;
pub use citation::*;
pub use directory::*;
pub use impl_plan::*;
pub use linting::*;
pub use metadata::*;
pub use parsing::*;
pub use pattern_link::*;
pub use progress::*;
pub use readme::*;
pub use search_api::*;
pub use search_index::*;
pub use templates::*;
pub use validation::*;
pub use versioning::*;
pub use watcher::*;