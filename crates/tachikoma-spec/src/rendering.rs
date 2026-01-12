use std::collections::HashMap;
use std::path::{Path, PathBuf};
use handlebars::{Handlebars, Helper, HelperResult, Context, RenderContext as HbRenderContext, Output, Renderable, Template};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;

/// Output format for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Html,
    Json,
    Text,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Json => "json",
            Self::Text => "txt",
        }
    }
}

/// Template rendering context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Spec metadata
    #[serde(flatten)]
    pub spec: SpecContext,
    /// Project context
    pub project: ProjectContext,
    /// Custom variables
    pub vars: HashMap<String, Value>,
    /// Current date/time
    pub timestamp: String,
}

/// Spec-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecContext {
    pub id: u32,
    pub title: String,
    pub phase: u32,
    pub phase_name: String,
    pub status: String,
    pub dependencies: Vec<DependencyContext>,
    pub estimated_context: String,
    pub sections: HashMap<String, String>,
    pub acceptance_criteria: Vec<CriterionContext>,
    pub code_blocks: Vec<CodeBlockContext>,
}

/// Dependency context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyContext {
    pub id: u32,
    pub name: String,
    pub status: String,
    pub satisfied: bool,
}

/// Acceptance criterion context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionContext {
    pub text: String,
    pub checked: bool,
    pub index: usize,
}

/// Code block context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockContext {
    pub language: String,
    pub content: String,
    pub section: String,
}

/// Project-level context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub version: String,
    pub total_specs: u32,
    pub completed_specs: u32,
    pub phases: Vec<PhaseContext>,
}

/// Phase context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseContext {
    pub number: u32,
    pub name: String,
    pub spec_count: u32,
    pub progress: u8,
}

/// Template renderer
pub struct SpecRenderer {
    handlebars: Handlebars<'static>,
    partials_dir: PathBuf,
    cache_enabled: bool,
}

impl SpecRenderer {
    /// Create a new renderer
    pub fn new(templates_dir: &Path) -> Result<Self, RenderError> {
        let mut handlebars = Handlebars::new();

        // Enable strict mode for better error messages
        handlebars.set_strict_mode(true);

        // Register built-in helpers
        Self::register_helpers(&mut handlebars);

        // Load templates
        Self::load_templates(&mut handlebars, templates_dir)?;

        Ok(Self {
            handlebars,
            partials_dir: templates_dir.join("partials"),
            cache_enabled: true,
        })
    }

    /// Register custom helpers
    fn register_helpers(hb: &mut Handlebars) {
        // Checkbox helper
        hb.register_helper("checkbox", Box::new(checkbox_helper));

        // Progress bar helper
        hb.register_helper("progress_bar", Box::new(progress_bar_helper));

        // Spec reference helper
        hb.register_helper("spec_ref", Box::new(spec_ref_helper));

        // Code block helper
        hb.register_helper("code", Box::new(code_block_helper));

        // Date formatting helper
        hb.register_helper("date", Box::new(date_helper));

        // Status badge helper
        hb.register_helper("status_badge", Box::new(status_badge_helper));

        // Pluralize helper
        hb.register_helper("pluralize", Box::new(pluralize_helper));

        // If equal helper
        hb.register_helper("if_eq", Box::new(if_eq_helper));

        // Join helper
        hb.register_helper("join", Box::new(join_helper));

        // Truncate helper
        hb.register_helper("truncate", Box::new(truncate_helper));

        // Slug helper
        hb.register_helper("slug", Box::new(slug_helper));
    }

    /// Load templates from directory
    fn load_templates(hb: &mut Handlebars, dir: &Path) -> Result<(), RenderError> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "hbs").unwrap_or(false) {
                let name = path.file_stem()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| RenderError::InvalidTemplate("Invalid filename".into()))?;

                let content = std::fs::read_to_string(&path)?;
                hb.register_template_string(name, &content)?;
            }
        }

        // Load partials
        let partials_dir = dir.join("partials");
        if partials_dir.exists() {
            for entry in std::fs::read_dir(&partials_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().map(|e| e == "hbs").unwrap_or(false) {
                    let name = path.file_stem()
                        .and_then(|n| n.to_str())
                        .map(|n| format!("partials/{}", n))
                        .ok_or_else(|| RenderError::InvalidTemplate("Invalid filename".into()))?;

                    let content = std::fs::read_to_string(&path)?;
                    hb.register_partial(&name, &content)?;
                }
            }
        }

        Ok(())
    }

    /// Render a template with context
    pub fn render(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> Result<String, RenderError> {
        let result = self.handlebars.render(template, context)?;
        Ok(result)
    }

    /// Render to a specific format
    pub fn render_format(
        &self,
        template: &str,
        context: &TemplateContext,
        format: OutputFormat,
    ) -> Result<String, RenderError> {
        let content = self.render(template, context)?;

        match format {
            OutputFormat::Markdown => Ok(content),
            OutputFormat::Html => Ok(self.markdown_to_html(&content)),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(context)?;
                Ok(json)
            }
            OutputFormat::Text => Ok(self.strip_markdown(&content)),
        }
    }

    /// Convert markdown to HTML
    fn markdown_to_html(&self, markdown: &str) -> String {
        // Simple markdown conversion (would use pulldown-cmark in real impl)
        let mut html = markdown.to_string();

        // Headers
        for level in (1..=6).rev() {
            let pattern = format!("{} ", "#".repeat(level));
            let lines: Vec<&str> = html.lines().collect();
            let converted: Vec<String> = lines.iter().map(|line| {
                if line.starts_with(&pattern) {
                    format!("<h{level}>{}</h{level}>", &line[level + 1..])
                } else {
                    line.to_string()
                }
            }).collect();
            html = converted.join("\n");
        }

        // Code blocks
        let mut in_code = false;
        let lines: Vec<&str> = html.lines().collect();
        let mut converted = Vec::new();

        for line in lines {
            if line.starts_with("```") {
                if in_code {
                    converted.push("</code></pre>".to_string());
                } else {
                    let lang = line.trim_start_matches('`');
                    converted.push(format!("<pre><code class=\"language-{}\">", lang));
                }
                in_code = !in_code;
            } else if in_code {
                converted.push(html_escape(line));
            } else {
                converted.push(line.to_string());
            }
        }

        // Bold and italic
        html = converted.join("\n");
        html = regex::Regex::new(r"\*\*([^*]+)\*\*").unwrap()
            .replace_all(&html, "<strong>$1</strong>").to_string();
        html = regex::Regex::new(r"\*([^*]+)\*").unwrap()
            .replace_all(&html, "<em>$1</em>").to_string();

        // Wrap in basic HTML
        format!(
            "<!DOCTYPE html>\n<html>\n<head><meta charset=\"utf-8\"></head>\n<body>\n{}\n</body>\n</html>",
            html
        )
    }

    /// Strip markdown formatting
    fn strip_markdown(&self, markdown: &str) -> String {
        let mut text = markdown.to_string();

        // Remove headers
        text = regex::Regex::new(r"^#+\s+").unwrap()
            .replace_all(&text, "").to_string();

        // Remove bold/italic
        text = regex::Regex::new(r"\*+([^*]+)\*+").unwrap()
            .replace_all(&text, "$1").to_string();

        // Remove code fences
        text = regex::Regex::new(r"```\w*\n?").unwrap()
            .replace_all(&text, "").to_string();

        // Remove links
        text = regex::Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap()
            .replace_all(&text, "$1").to_string();

        text
    }

    /// Register a custom template string
    pub fn register_template(
        &mut self,
        name: &str,
        template: &str,
    ) -> Result<(), RenderError> {
        self.handlebars.register_template_string(name, template)?;
        Ok(())
    }

    /// Render to file
    pub async fn render_to_file(
        &self,
        template: &str,
        context: &TemplateContext,
        output_path: &Path,
        format: OutputFormat,
    ) -> Result<PathBuf, RenderError> {
        let content = self.render_format(template, context, format)?;

        // Ensure directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(output_path, content).await?;
        Ok(output_path.to_path_buf())
    }
}

// ===== Helper Functions =====

fn checkbox_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let checked = h.param(0)
        .and_then(|v| v.value().as_bool())
        .unwrap_or(false);

    out.write(if checked { "- [x]" } else { "- [ ]" })?;
    Ok(())
}

fn progress_bar_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let percentage = h.param(0)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(0) as usize;

    let width = h.param(1)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(20) as usize;

    let filled = (percentage * width) / 100;
    let empty = width - filled;

    out.write(&format!("[{}{}] {}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    ))?;
    Ok(())
}

fn spec_ref_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let id = h.param(0)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(0);

    out.write(&format!("spec:{:03}", id))?;
    Ok(())
}

fn code_block_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let lang = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("text");

    let code = h.param(1)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    out.write(&format!("```{}\n{}\n```", lang, code))?;
    Ok(())
}

fn date_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let format = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("%Y-%m-%d");

    let now = chrono::Local::now();
    out.write(&now.format(format).to_string())?;
    Ok(())
}

fn status_badge_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let status = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("Unknown");

    let badge = match status.to_lowercase().as_str() {
        "complete" | "completed" | "done" => "![Complete](https://img.shields.io/badge/status-complete-green)",
        "in progress" | "inprogress" => "![In Progress](https://img.shields.io/badge/status-in%20progress-yellow)",
        "planned" => "![Planned](https://img.shields.io/badge/status-planned-blue)",
        "blocked" => "![Blocked](https://img.shields.io/badge/status-blocked-red)",
        _ => "![Unknown](https://img.shields.io/badge/status-unknown-gray)",
    };

    out.write(badge)?;
    Ok(())
}

fn pluralize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let count = h.param(0)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(0);

    let singular = h.param(1)
        .and_then(|v| v.value().as_str())
        .unwrap_or("item");

    let plural = h.param(2)
        .and_then(|v| v.value().as_str())
        .unwrap_or(&format!("{}s", singular));

    out.write(if count == 1 { singular } else { plural })?;
    Ok(())
}

fn if_eq_helper(
    h: &Helper,
    hb: &Handlebars,
    ctx: &Context,
    rc: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let a = h.param(0).map(|v| v.value());
    let b = h.param(1).map(|v| v.value());

    let template = if a == b {
        h.template()
    } else {
        h.inverse()
    };

    if let Some(t) = template {
        t.render(hb, ctx, rc, out)?;
    }

    Ok(())
}

fn join_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let array = h.param(0)
        .and_then(|v| v.value().as_array());

    let separator = h.param(1)
        .and_then(|v| v.value().as_str())
        .unwrap_or(", ");

    if let Some(arr) = array {
        let strings: Vec<String> = arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        out.write(&strings.join(separator))?;
    }

    Ok(())
}

fn truncate_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let text = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    let length = h.param(1)
        .and_then(|v| v.value().as_u64())
        .unwrap_or(100) as usize;

    if text.len() > length {
        out.write(&format!("{}...", &text[..length]))?;
    } else {
        out.write(text)?;
    }

    Ok(())
}

fn slug_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut HbRenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let text = h.param(0)
        .and_then(|v| v.value().as_str())
        .unwrap_or("");

    let slug: String = text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    out.write(&slug)?;
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render errors
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Template error: {0}")]
    Template(#[from] handlebars::TemplateError),

    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_slug_generation() {
        let slug: String = "Hello World Test"
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        assert_eq!(slug, "hello-world-test");
    }

    #[test]
    fn test_html_escape() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(!escaped.contains('<'));
        assert!(escaped.contains("&lt;"));
    }

    #[test]
    fn test_progress_bar_format() {
        // Test progress bar formatting
        let percentage = 50;
        let width = 10;
        let filled = (percentage * width) / 100;
        let empty = width - filled;

        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
        assert!(bar.contains("█████"));
        assert!(bar.contains("░░░░░"));
    }

    #[tokio::test]
    async fn test_render_context_creation() {
        let project = ProjectContext {
            name: "Tachikoma".to_string(),
            version: "0.1.0".to_string(),
            total_specs: 150,
            completed_specs: 75,
            phases: vec![],
        };

        let spec = SpecContext {
            id: 133,
            title: "Spec Template Rendering".to_string(),
            phase: 6,
            phase_name: "Spec System".to_string(),
            status: "In Progress".to_string(),
            dependencies: vec![],
            estimated_context: "~10%".to_string(),
            sections: HashMap::new(),
            acceptance_criteria: vec![],
            code_blocks: vec![],
        };

        let context = RenderContext {
            spec,
            project,
            vars: HashMap::new(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        assert_eq!(context.spec.id, 133);
        assert_eq!(context.project.name, "Tachikoma");
    }

    #[tokio::test]
    async fn test_template_rendering() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a simple template file
        let template_content = r#"# Spec {{spec.id}}: {{spec.title}}
Status: {{spec.status}}
Project: {{project.name}}

{{#checkbox spec.acceptance_criteria.0.checked}}First criterion{{/checkbox}}
"#;
        
        let template_path = temp_dir.path().join("test.hbs");
        std::fs::write(&template_path, template_content).unwrap();
        
        let renderer = SpecRenderer::new(temp_dir.path()).unwrap();
        
        let context = create_test_context();
        let result = renderer.render("test", &context).unwrap();
        
        assert!(result.contains("Spec 133"));
        assert!(result.contains("Spec Template Rendering"));
        assert!(result.contains("Project: Tachikoma"));
        assert!(result.contains("- [ ]")); // Unchecked checkbox
    }

    #[tokio::test]
    async fn test_multiple_output_formats() {
        let temp_dir = TempDir::new().unwrap();
        
        let template_content = r#"# Test Spec
**Bold text** and *italic text*
```rust
fn test() {}
```"#;
        
        let template_path = temp_dir.path().join("format_test.hbs");
        std::fs::write(&template_path, template_content).unwrap();
        
        let renderer = SpecRenderer::new(temp_dir.path()).unwrap();
        let context = create_test_context();
        
        // Test Markdown (default)
        let md_result = renderer.render_format("format_test", &context, OutputFormat::Markdown).unwrap();
        assert!(md_result.contains("# Test Spec"));
        assert!(md_result.contains("**Bold text**"));
        
        // Test HTML
        let html_result = renderer.render_format("format_test", &context, OutputFormat::Html).unwrap();
        assert!(html_result.contains("<h1>Test Spec</h1>"));
        assert!(html_result.contains("<strong>Bold text</strong>"));
        
        // Test JSON
        let json_result = renderer.render_format("format_test", &context, OutputFormat::Json).unwrap();
        let _: Value = serde_json::from_str(&json_result).unwrap(); // Should parse as valid JSON
        
        // Test Text
        let text_result = renderer.render_format("format_test", &context, OutputFormat::Text).unwrap();
        assert!(text_result.contains("Test Spec"));
        assert!(!text_result.contains("#")); // No markdown headers
        assert!(!text_result.contains("**")); // No bold formatting
    }

    fn create_test_context() -> RenderContext {
        let project = ProjectContext {
            name: "Tachikoma".to_string(),
            version: "0.1.0".to_string(),
            total_specs: 150,
            completed_specs: 75,
            phases: vec![],
        };

        let spec = SpecContext {
            id: 133,
            title: "Spec Template Rendering".to_string(),
            phase: 6,
            phase_name: "Spec System".to_string(),
            status: "In Progress".to_string(),
            dependencies: vec![],
            estimated_context: "~10%".to_string(),
            sections: HashMap::new(),
            acceptance_criteria: vec![
                CriterionContext {
                    text: "First criterion".to_string(),
                    checked: false,
                    index: 0,
                }
            ],
            code_blocks: vec![],
        };

        RenderContext {
            spec,
            project,
            vars: HashMap::new(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}