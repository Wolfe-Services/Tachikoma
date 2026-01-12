//! Template system for Forge prompt generation.

pub mod engine;

// Re-export main types
pub use engine::{TemplateEngine, Template, TemplateContext, OutputType, ParticipantRole};