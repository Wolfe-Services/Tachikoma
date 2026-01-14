//! Template engine for prompt customization
//!
//! Uses Handlebars templates for system and task prompts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use handlebars::Handlebars;
use serde::Serialize;

use crate::{PluginError, Result};

/// Template engine for rendering prompts
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    template_dir: PathBuf,
    loaded_templates: Vec<String>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);
        handlebars.register_escape_fn(handlebars::no_escape);
        
        Self {
            handlebars,
            template_dir: PathBuf::new(),
            loaded_templates: Vec::new(),
        }
    }
    
    /// Load templates from a directory
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Err(PluginError::NotFound(format!(
                "Template directory not found: {}",
                dir.display()
            )));
        }
        
        self.template_dir = dir.to_path_buf();
        
        // Load all .hbs files in the directory
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "hbs").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                
                let content = std::fs::read_to_string(&path)?;
                
                self.handlebars
                    .register_template_string(name, &content)
                    .map_err(|e| PluginError::TemplateError(e.to_string()))?;
                
                self.loaded_templates.push(name.to_string());
            }
        }
        
        Ok(())
    }
    
    /// Register a template from a string
    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.handlebars
            .register_template_string(name, template)
            .map_err(|e| PluginError::TemplateError(e.to_string()))?;
        self.loaded_templates.push(name.to_string());
        Ok(())
    }
    
    /// Render a template with the given data
    pub fn render<T: Serialize>(&self, name: &str, data: &T) -> Result<String> {
        self.handlebars
            .render(name, data)
            .map_err(|e| PluginError::TemplateError(e.to_string()))
    }
    
    /// Check if a template is registered
    pub fn has_template(&self, name: &str) -> bool {
        self.handlebars.has_template(name)
    }
    
    /// List loaded templates
    pub fn list_templates(&self) -> &[String] {
        &self.loaded_templates
    }
    
    /// Get the template directory
    pub fn template_dir(&self) -> &Path {
        &self.template_dir
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for rendering system prompts
#[derive(Debug, Clone, Serialize)]
pub struct SystemPromptContext {
    /// Project information
    pub project: ProjectInfo,
    
    /// Available tools
    pub tools: Vec<ToolInfo>,
    
    /// Current spec (if any)
    pub spec: Option<SpecInfo>,
    
    /// Custom instructions
    pub custom_instructions: Option<String>,
}

/// Project information for templates
#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,
    
    /// Project root path
    pub root: String,
    
    /// Tech stack
    pub tech_stack: Vec<String>,
}

/// Tool information for templates
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
}

/// Spec information for templates
#[derive(Debug, Clone, Serialize)]
pub struct SpecInfo {
    /// Spec ID
    pub id: u32,
    
    /// Spec name
    pub name: String,
    
    /// Phase name
    pub phase: String,
    
    /// Spec file path
    pub path: String,
    
    /// Incomplete criteria
    pub incomplete_criteria: Vec<String>,
}

/// Context for rendering task prompts
#[derive(Debug, Clone, Serialize)]
pub struct TaskPromptContext {
    /// Spec information
    pub spec: SpecInfo,
    
    /// Relevant patterns to follow
    pub patterns: Vec<PatternInfo>,
}

/// Pattern information for templates
#[derive(Debug, Clone, Serialize)]
pub struct PatternInfo {
    /// File path
    pub file: String,
    
    /// Description of the pattern
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_engine_basic() {
        let mut engine = TemplateEngine::new();
        
        engine.register_template("test", "Hello, {{name}}!").unwrap();
        
        let mut data = HashMap::new();
        data.insert("name", "World");
        
        let result = engine.render("test", &data).unwrap();
        assert_eq!(result, "Hello, World!");
    }
    
    #[test]
    fn test_template_with_context() {
        let mut engine = TemplateEngine::new();
        
        let template = r#"
## Project: {{project.name}}
Root: {{project.root}}

## Tools
{{#each tools}}
- **{{name}}**: {{description}}
{{/each}}
"#;
        
        engine.register_template("system", template).unwrap();
        
        let context = SystemPromptContext {
            project: ProjectInfo {
                name: "Tachikoma".to_string(),
                root: "/home/user/project".to_string(),
                tech_stack: vec!["Rust".to_string(), "TypeScript".to_string()],
            },
            tools: vec![
                ToolInfo {
                    name: "read_file".to_string(),
                    description: "Read file contents".to_string(),
                },
                ToolInfo {
                    name: "edit_file".to_string(),
                    description: "Edit file contents".to_string(),
                },
            ],
            spec: None,
            custom_instructions: None,
        };
        
        let result = engine.render("system", &context).unwrap();
        assert!(result.contains("Tachikoma"));
        assert!(result.contains("read_file"));
    }
}
