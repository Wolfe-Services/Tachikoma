# Spec 089: Init Templates

## Metadata
- **Phase**: 4 - CLI Foundation
- **Spec ID**: 089
- **Status**: Planned
- **Dependencies**: 088-cli-init-scaffold
- **Estimated Context**: ~10%

## Objective

Implement a template system for project initialization that supports built-in templates, custom templates from registries, and local template directories.

## Acceptance Criteria

- [x] Built-in templates (basic, tools, workflow, chat, minimal)
- [x] Custom templates from Git repositories
- [x] Local template directories
- [x] Template variable substitution
- [x] Template manifest files
- [x] Template validation
- [x] Template caching
- [x] Template listing and search

## Implementation Details

### src/templates/mod.rs

```rust
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
```

### src/templates/manifest.rs

```rust
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
```

### src/templates/loader.rs

```rust
//! Template loading from various sources.

use std::path::{Path, PathBuf};

use crate::templates::{Template, TemplateFile, TemplateManifest, TemplateError};

/// Source of a template
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Built-in template
    Builtin(String),
    /// Local directory
    Local(PathBuf),
    /// Git repository
    Git { url: String, ref_: Option<String> },
    /// Registry template
    Registry { name: String, version: Option<String> },
}

/// Template loader
pub struct TemplateLoader {
    cache_dir: PathBuf,
}

impl TemplateLoader {
    pub fn new() -> Result<Self, TemplateError> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("tachikoma")
            .join("templates");

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Load a template from any source
    pub async fn load(&self, source: &TemplateSource) -> Result<Template, TemplateError> {
        match source {
            TemplateSource::Builtin(name) => self.load_builtin(name),
            TemplateSource::Local(path) => self.load_local(path).await,
            TemplateSource::Git { url, ref_ } => self.load_git(url, ref_.as_deref()).await,
            TemplateSource::Registry { name, version } => {
                self.load_registry(name, version.as_deref()).await
            }
        }
    }

    /// Load a built-in template
    fn load_builtin(&self, name: &str) -> Result<Template, TemplateError> {
        crate::templates::BuiltinTemplates::get(name)
            .ok_or_else(|| TemplateError::NotFound(name.to_string()))
    }

    /// Load a template from local directory
    async fn load_local(&self, path: &Path) -> Result<Template, TemplateError> {
        // Load manifest
        let manifest_path = path.join("template.toml");
        let manifest = if manifest_path.exists() {
            TemplateManifest::load(&manifest_path)?
        } else {
            // Create default manifest
            TemplateManifest {
                template: crate::templates::manifest::TemplateMetadata {
                    name: path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "custom".to_string()),
                    description: "Custom template".to_string(),
                    version: "0.1.0".to_string(),
                    authors: vec![],
                    tags: vec![],
                    min_tachikoma_version: None,
                },
                variables: vec![],
                files: Default::default(),
                hooks: Default::default(),
            }
        };

        // Load files
        let files = self.load_files(path, &manifest).await?;

        Ok(Template {
            name: manifest.template.name.clone(),
            description: manifest.template.description.clone(),
            source: TemplateSource::Local(path.to_path_buf()),
            manifest,
            files,
        })
    }

    /// Load a template from Git repository
    async fn load_git(&self, url: &str, ref_: Option<&str>) -> Result<Template, TemplateError> {
        // Create cache key from URL
        let cache_key = url
            .replace("://", "_")
            .replace('/', "_")
            .replace('.', "_");

        let cache_path = self.cache_dir.join(&cache_key);

        // Clone or update repository
        if cache_path.exists() {
            // Update existing clone
            let status = tokio::process::Command::new("git")
                .args(["pull"])
                .current_dir(&cache_path)
                .output()
                .await?;

            if !status.status.success() {
                // If pull fails, remove and re-clone
                std::fs::remove_dir_all(&cache_path)?;
            }
        }

        if !cache_path.exists() {
            // Clone repository
            let mut args = vec!["clone", "--depth", "1"];

            if let Some(r) = ref_ {
                args.extend(["--branch", r]);
            }

            args.extend([url, cache_path.to_str().unwrap()]);

            let status = tokio::process::Command::new("git")
                .args(&args)
                .output()
                .await?;

            if !status.status.success() {
                return Err(TemplateError::NotFound(format!(
                    "Failed to clone: {}",
                    String::from_utf8_lossy(&status.stderr)
                )));
            }
        }

        self.load_local(&cache_path).await
    }

    /// Load a template from registry
    async fn load_registry(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Template, TemplateError> {
        // Registry URL would be configurable
        let registry_url = "https://registry.tachikoma.dev/templates";

        let url = match version {
            Some(v) => format!("{}/{}/{}", registry_url, name, v),
            None => format!("{}/{}/latest", registry_url, name),
        };

        // Fetch template info
        let response = reqwest::get(&url)
            .await
            .map_err(|e| TemplateError::NotFound(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TemplateError::NotFound(format!(
                "Template '{}' not found in registry",
                name
            )));
        }

        #[derive(serde::Deserialize)]
        struct RegistryEntry {
            git_url: String,
            git_ref: Option<String>,
        }

        let entry: RegistryEntry = response
            .json()
            .await
            .map_err(|e| TemplateError::NotFound(e.to_string()))?;

        self.load_git(&entry.git_url, entry.git_ref.as_deref()).await
    }

    /// Load files from a template directory
    async fn load_files(
        &self,
        path: &Path,
        manifest: &TemplateManifest,
    ) -> Result<Vec<TemplateFile>, TemplateError> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip hidden files and template.toml
                !name.starts_with('.') && name != "template.toml"
            })
        {
            let entry = entry?;

            if !entry.file_type().is_file() {
                continue;
            }

            let relative_path = entry.path().strip_prefix(path).unwrap();

            // Check include/exclude patterns
            let path_str = relative_path.to_string_lossy();

            let excluded = manifest.files.exclude.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            });

            if excluded {
                continue;
            }

            // Check if should be processed
            let no_process = manifest.files.no_process.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            });

            // Read content
            let content = std::fs::read_to_string(entry.path())?;

            // Check if executable
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                entry.metadata()?.permissions().mode() & 0o111 != 0
            };

            #[cfg(not(unix))]
            let executable = false;

            files.push(TemplateFile {
                path: relative_path.to_path_buf(),
                content,
                process: !no_process,
                executable,
            });
        }

        Ok(files)
    }

    /// List available templates from all sources
    pub async fn list_all(&self) -> Vec<TemplateInfo> {
        let mut templates = Vec::new();

        // Built-in templates
        templates.extend(crate::templates::BuiltinTemplates::list());

        // Cached templates
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(template) = self.load_local(&entry.path()).await {
                    templates.push(TemplateInfo {
                        name: template.name,
                        description: template.description,
                        source: "cached".to_string(),
                    });
                }
            }
        }

        templates
    }
}

impl Default for TemplateLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create template loader")
    }
}

/// Summary information about a template
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub source: String,
}
```

### src/templates/renderer.rs

```rust
//! Template rendering with variable substitution.

use std::path::Path;

use regex::Regex;

use crate::templates::{Template, TemplateContext, TemplateFile, TemplateError};

/// Template renderer
pub struct TemplateRenderer {
    variable_pattern: Regex,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        // Match {{variable_name}} or {{ variable_name }}
        let variable_pattern = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

        Self { variable_pattern }
    }

    /// Render a template to a target directory
    pub async fn render(
        &self,
        template: &Template,
        target: &Path,
        context: &TemplateContext,
    ) -> Result<Vec<RenderedFile>, TemplateError> {
        // Validate all required variables are present
        self.validate_context(template, context)?;

        let mut rendered = Vec::new();

        for file in &template.files {
            // Check conditional inclusion
            if !self.should_include_file(file, &template.manifest, context)? {
                continue;
            }

            // Render path (may contain variables)
            let rendered_path = if file.process {
                self.render_string(&file.path.to_string_lossy(), context)?
            } else {
                file.path.to_string_lossy().to_string()
            };

            let target_path = target.join(&rendered_path);

            // Render content
            let content = if file.process {
                self.render_string(&file.content, context)?
            } else {
                file.content.clone()
            };

            rendered.push(RenderedFile {
                path: target_path,
                content,
                executable: file.executable,
            });
        }

        Ok(rendered)
    }

    /// Render a string with variable substitution
    pub fn render_string(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let mut result = template.to_string();
        let mut missing = Vec::new();

        for cap in self.variable_pattern.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = &cap[1];

            match context.get(var_name) {
                Some(value) => {
                    result = result.replace(full_match, value);
                }
                None => {
                    missing.push(var_name.to_string());
                }
            }
        }

        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(missing.join(", ")));
        }

        Ok(result)
    }

    /// Validate that all required variables are present
    fn validate_context(
        &self,
        template: &Template,
        context: &TemplateContext,
    ) -> Result<(), TemplateError> {
        let missing: Vec<_> = template
            .manifest
            .variables
            .iter()
            .filter(|v| v.required && context.get(&v.name).is_none())
            .map(|v| v.name.clone())
            .collect();

        if !missing.is_empty() {
            return Err(TemplateError::MissingVariable(missing.join(", ")));
        }

        Ok(())
    }

    /// Check if a file should be included based on conditions
    fn should_include_file(
        &self,
        file: &TemplateFile,
        manifest: &crate::templates::TemplateManifest,
        context: &TemplateContext,
    ) -> Result<bool, TemplateError> {
        let path_str = file.path.to_string_lossy();

        for conditional in &manifest.files.conditional {
            if glob::Pattern::new(&conditional.path)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
            {
                // Evaluate condition
                return self.evaluate_condition(&conditional.condition, context);
            }
        }

        Ok(true)
    }

    /// Evaluate a simple condition expression
    fn evaluate_condition(
        &self,
        condition: &str,
        context: &TemplateContext,
    ) -> Result<bool, TemplateError> {
        // Simple condition parser: "var == value" or "var != value" or just "var"
        let condition = condition.trim();

        if condition.contains("==") {
            let parts: Vec<_> = condition.split("==").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let value = context.get(parts[0]).map(|s| s.as_str()).unwrap_or("");
                return Ok(value == parts[1].trim_matches('"'));
            }
        } else if condition.contains("!=") {
            let parts: Vec<_> = condition.split("!=").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let value = context.get(parts[0]).map(|s| s.as_str()).unwrap_or("");
                return Ok(value != parts[1].trim_matches('"'));
            }
        } else {
            // Just a variable name - check if it's truthy
            let value = context.get(condition).map(|s| s.as_str()).unwrap_or("");
            return Ok(!value.is_empty() && value != "false" && value != "0");
        }

        Err(TemplateError::Render(format!(
            "Invalid condition: {condition}"
        )))
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// A rendered file ready to be written
#[derive(Debug)]
pub struct RenderedFile {
    pub path: std::path::PathBuf,
    pub content: String,
    pub executable: bool,
}

impl RenderedFile {
    /// Write the file to disk
    pub fn write(&self) -> Result<(), TemplateError> {
        // Create parent directories
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write content
        std::fs::write(&self.path, &self.content)?;

        // Set executable permission on Unix
        #[cfg(unix)]
        if self.executable {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.path)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&self.path, perms)?;
        }

        Ok(())
    }
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_string() {
        let renderer = TemplateRenderer::new();
        let mut context = TemplateContext::new();
        context.set("name", "MyProject");
        context.set("version", "1.0.0");

        let template = "Project: {{name}} v{{version}}";
        let result = renderer.render_string(template, &context).unwrap();

        assert_eq!(result, "Project: MyProject v1.0.0");
    }

    #[test]
    fn test_render_string_missing_variable() {
        let renderer = TemplateRenderer::new();
        let context = TemplateContext::new();

        let template = "Hello {{name}}";
        let result = renderer.render_string(template, &context);

        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_condition_equals() {
        let renderer = TemplateRenderer::new();
        let mut context = TemplateContext::new();
        context.set("use_tools", "true");

        assert!(renderer.evaluate_condition("use_tools == \"true\"", &context).unwrap());
        assert!(!renderer.evaluate_condition("use_tools == \"false\"", &context).unwrap());
    }

    #[test]
    fn test_context_defaults() {
        let context = TemplateContext::with_defaults("my-project");

        assert_eq!(context.get("project_name").unwrap(), "my-project");
        assert_eq!(context.get("project_name_snake").unwrap(), "my_project");
        assert_eq!(context.get("project_name_kebab").unwrap(), "my-project");
        assert_eq!(context.get("project_name_pascal").unwrap(), "MyProject");
    }
}
```

## Related Specs

- **088-cli-init-scaffold.md**: Uses templates for initialization
- **076-cli-crate.md**: Base CLI structure
- **083-cli-prompts.md**: Variable prompts
