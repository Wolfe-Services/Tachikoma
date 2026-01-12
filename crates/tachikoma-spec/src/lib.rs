//! Tachikoma specification system.
//!
//! This crate provides the core functionality for managing Tachikoma specifications,
//! including directory structure discovery, parsing, and validation.

pub mod directory;
pub mod watcher;

pub use directory::*;
pub use watcher::*;