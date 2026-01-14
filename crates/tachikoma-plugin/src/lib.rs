//! Plugin System for Tachikoma
//!
//! This crate provides a plugin architecture for:
//! - **Agents**: LLM backends that execute tool calls (Claude, OpenCode, Ollama)
//! - **Trackers**: Task/spec state managers (Specs, Beads, JSON)
//! - **Templates**: Handlebars-based prompt customization
//!
//! ## Plugin Discovery
//!
//! Plugins are discovered from `.tachikoma/plugins/` with the following structure:
//! ```text
//! .tachikoma/plugins/
//! ├── agents/
//! │   ├── claude/plugin.yaml
//! │   └── opencode/plugin.yaml
//! ├── trackers/
//! │   ├── specs/plugin.yaml
//! │   └── beads/plugin.yaml
//! └── templates/
//!     ├── default/system-prompt.hbs
//!     └── minimal/system-prompt.hbs
//! ```

pub mod agent;
pub mod tracker;
pub mod template;
pub mod manifest;
pub mod loader;

pub use agent::AgentPlugin;
pub use tracker::TrackerPlugin;
pub use template::TemplateEngine;
pub use manifest::{PluginManifest, PluginType, PluginRequirement};
pub use loader::PluginLoader;

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;

/// Errors that can occur in the plugin system
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    /// Invalid plugin manifest
    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),
    
    /// Plugin requirement not met
    #[error("Plugin requirement not met: {0}")]
    RequirementNotMet(String),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Plugin execution failed
    #[error("Plugin execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Template error
    #[error("Template error: {0}")]
    TemplateError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// YAML parsing error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
