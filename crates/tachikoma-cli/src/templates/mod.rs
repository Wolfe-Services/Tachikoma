//! Project template system.

mod builtin;
mod loader;
mod manifest;
mod renderer;

pub use builtin::BuiltinTemplates;
pub use loader::{TemplateLoader, TemplateSource};
pub use manifest::{TemplateManifest, TemplateVariable};
pub use renderer::TemplateRenderer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A project template
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub source: TemplateSource,
    pub manifest: TemplateManifest,
    pub files: Vec<TemplateFile>,
}

/// A file within a template
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Relative path within the template
    pub path: PathBuf,
    /// File content (may contain variables)
    pub content: String,
    /// Whether to process variables
    pub process: bool,
    /// File permissions (Unix)
    pub executable: bool,
}

/// Context for template rendering
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub variables: HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Create context with standard variables
    pub fn with_defaults(project_name: &str) -> Self {
        let mut ctx = Self::new();

        ctx.set("project_name", project_name);
        ctx.set("project_name_snake", to_snake_case(project_name));
        ctx.set("project_name_kebab", to_kebab_case(project_name));
        ctx.set("project_name_pascal", to_pascal_case(project_name));

        ctx.set("year", chrono::Utc::now().format("%Y").to_string());
        ctx.set("date", chrono::Utc::now().format("%Y-%m-%d").to_string());

        if let Some(author) = get_git_user() {
            ctx.set("author", author);
        }

        ctx.set("tachikoma_version", env!("CARGO_PKG_VERSION"));

        ctx
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap());
    }
    result.replace('-', "_")
}

fn to_kebab_case(s: &str) -> String {
    to_snake_case(s).replace('_', "-")
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '_' || c == '-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn get_git_user() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}