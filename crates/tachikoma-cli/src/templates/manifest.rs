//! Template manifest definition.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Template manifest (template.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    /// Template metadata
    pub template: TemplateMetadata,

    /// Variables that can be customized
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,

    /// Files to include/exclude
    #[serde(default)]
    pub files: FileConfig,

    /// Post-creation hooks
    #[serde(default)]
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub min_tachikoma_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (used in templates as {{name}})
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Whether this variable is required
    #[serde(default)]
    pub required: bool,

    /// Variable type for validation
    #[serde(default)]
    pub var_type: VariableType,

    /// Prompt text for interactive mode
    #[serde(default)]
    pub prompt: Option<String>,

    /// Choices for select type
    #[serde(default)]
    pub choices: Vec<String>,

    /// Validation regex pattern
    #[serde(default)]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    #[default]
    String,
    Boolean,
    Number,
    Select,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileConfig {
    /// Files to include (glob patterns)
    #[serde(default)]
    pub include: Vec<String>,

    /// Files to exclude (glob patterns)
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Files that should not have variable substitution
    #[serde(default)]
    pub no_process: Vec<String>,

    /// Conditional files based on variables
    #[serde(default)]
    pub conditional: Vec<ConditionalFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFile {
    /// File path pattern
    pub path: String,
    /// Condition expression (e.g., "use_tools == true")
    pub condition: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Commands to run before file generation
    #[serde(default)]
    pub pre_generate: Vec<String>,

    /// Commands to run after file generation
    #[serde(default)]
    pub post_generate: Vec<String>,
}

impl TemplateManifest {
    /// Load manifest from file
    pub fn load(path: &std::path::Path) -> Result<Self, TemplateError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), TemplateError> {
        // Check for duplicate variable names
        let mut seen = std::collections::HashSet::new();
        for var in &self.variables {
            if !seen.insert(&var.name) {
                return Err(TemplateError::InvalidManifest(format!(
                    "Duplicate variable: {}",
                    var.name
                )));
            }

            // Validate select type has choices
            if var.var_type == VariableType::Select && var.choices.is_empty() {
                return Err(TemplateError::InvalidManifest(format!(
                    "Select variable '{}' must have choices",
                    var.name
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Missing variable: {0}")]
    MissingVariable(String),

    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Render error: {0}")]
    Render(String),
}