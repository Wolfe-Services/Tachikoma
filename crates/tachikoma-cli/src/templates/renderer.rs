//! Template rendering with variable substitution.

use std::path::Path;

use regex::Regex;

use crate::templates::{Template, TemplateContext, TemplateFile, manifest::TemplateError};

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