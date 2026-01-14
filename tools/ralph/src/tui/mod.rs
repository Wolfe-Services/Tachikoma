//! TUI module for Ralph Loop Runner
//!
//! Provides a split-pane terminal UI with:
//! - Left panel: Task/spec list with status indicators
//! - Right panel: Live streaming agent output  
//! - Bottom bar: Progress, tokens, cost, and keyboard shortcuts

pub mod app;
pub mod events;
pub mod ui;
pub mod widgets;

pub use app::App;
pub use events::EventHandler;
