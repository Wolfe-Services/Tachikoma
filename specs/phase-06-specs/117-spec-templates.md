# Spec 117: Spec Templates

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 117
- **Status**: Planned
- **Dependencies**: 116-spec-directory
- **Estimated Context**: ~10%

## Objective

Define the template system for creating consistent, well-structured specification documents. Templates provide standardized formats for different spec types (feature, component, integration, refactor, test) while allowing customization for project-specific needs.

## Acceptance Criteria

- [ ] Multiple template types are supported (feature, component, integration, refactor, test)
- [ ] Templates include all required sections with placeholders
- [ ] Variable substitution works for dynamic content
- [ ] Custom templates can be added by projects
- [ ] Template inheritance allows extending base templates
- [ ] Validation ensures generated specs meet requirements
- [ ] Template versioning supports evolution
- [ ] CLI commands generate specs from templates

## Implementation Details

### Template System Core

```rust
// src/spec/templates.rs

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use handlebars::Handlebars;
use tokio::fs;

/// Template types for different spec categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    /// New feature specification
    Feature,
    /// Component/module specification
    Component,
    /// Integration specification
    Integration,
    /// Refactoring specification
    Refactor,
    /// Test specification
    Test,
    /// Custom template
    Custom,
}

impl TemplateType {
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Feature => "feature.md.hbs",
            Self::Component => "component.md.hbs",
            Self::Integration => "integration.md.hbs",
            Self::Refactor => "refactor.md.hbs",
            Self::Test => "test.md.hbs",
            Self::Custom => "custom.md.hbs",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Feature => "New feature or capability",
            Self::Component => "System component or module",
            Self::Integration => "Integration with external system",
            Self::Refactor => "Code refactoring or improvement",
            Self::Test => "Test suite or testing strategy",
            Self::Custom => "Custom specification",
        }
    }
}

/// Template variable for substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    pub required: bool,
}

/// Spec template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecTemplate {
    /// Template type
    pub template_type: TemplateType,
    /// Template version
    pub version: String,
    /// Template content (Handlebars format)
    pub content: String,
    /// Variables used in template
    pub variables: Vec<TemplateVariable>,
    /// Parent template for inheritance
    pub extends: Option<String>,
    /// Custom sections
    pub sections: Vec<TemplateSection>,
}

/// A section within a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    pub name: String,
    pub heading: String,
    pub required: bool,
    pub content: String,
}

/// Context for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    /// Spec ID
    pub spec_id: u32,
    /// Spec title
    pub title: String,
    /// Phase number
    pub phase: u32,
    /// Phase name
    pub phase_name: String,
    /// Spec slug (for filename)
    pub slug: String,
    /// Dependencies (spec IDs)
    pub dependencies: Vec<String>,
    /// Estimated context percentage
    pub estimated_context: String,
    /// Custom variables
    #[serde(flatten)]
    pub custom: HashMap<String, String>,
}

/// Template engine for spec generation
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
    templates: HashMap<TemplateType, SpecTemplate>,
    templates_dir: PathBuf,
}

impl TemplateEngine {
    /// Create a new template engine
    pub async fn new(templates_dir: &Path) -> Result<Self, TemplateError> {
        let mut engine = Self {
            handlebars: Handlebars::new(),
            templates: HashMap::new(),
            templates_dir: templates_dir.to_path_buf(),
        };

        // Register built-in helpers
        engine.register_helpers();

        // Load templates from directory
        engine.load_templates().await?;

        Ok(engine)
    }

    /// Register Handlebars helpers
    fn register_helpers(&mut self) {
        // Helper for checkbox formatting
        self.handlebars.register_helper(
            "checkbox",
            Box::new(|h: &handlebars::Helper,
                      _: &Handlebars,
                      _: &handlebars::Context,
                      _: &mut handlebars::RenderContext,
                      out: &mut dyn handlebars::Output| {
                let checked = h.param(0)
                    .and_then(|v| v.value().as_bool())
                    .unwrap_or(false);
                out.write(if checked { "- [x]" } else { "- [ ]" })?;
                Ok(())
            }),
        );

        // Helper for spec references
        self.handlebars.register_helper(
            "spec_ref",
            Box::new(|h: &handlebars::Helper,
                      _: &Handlebars,
                      _: &handlebars::Context,
                      _: &mut handlebars::RenderContext,
                      out: &mut dyn handlebars::Output| {
                if let Some(id) = h.param(0).and_then(|v| v.value().as_u64()) {
                    out.write(&format!("spec:{:03}", id))?;
                }
                Ok(())
            }),
        );

        // Helper for date formatting
        self.handlebars.register_helper(
            "today",
            Box::new(|_: &handlebars::Helper,
                      _: &Handlebars,
                      _: &handlebars::Context,
                      _: &mut handlebars::RenderContext,
                      out: &mut dyn handlebars::Output| {
                let today = chrono::Local::now().format("%Y-%m-%d");
                out.write(&today.to_string())?;
                Ok(())
            }),
        );
    }

    /// Load templates from the templates directory
    async fn load_templates(&mut self) -> Result<(), TemplateError> {
        // Load built-in templates first
        for template_type in [
            TemplateType::Feature,
            TemplateType::Component,
            TemplateType::Integration,
            TemplateType::Refactor,
            TemplateType::Test,
        ] {
            let content = self.get_builtin_template(template_type);
            let template = SpecTemplate {
                template_type,
                version: "1.0.0".to_string(),
                content: content.clone(),
                variables: self.extract_variables(&content),
                extends: None,
                sections: Vec::new(),
            };

            self.handlebars.register_template_string(
                template_type.filename(),
                &content,
            )?;

            self.templates.insert(template_type, template);
        }

        // Load custom templates from directory
        if self.templates_dir.exists() {
            let mut entries = fs::read_dir(&self.templates_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().map(|e| e == "hbs").unwrap_or(false) {
                    self.load_custom_template(&path).await?;
                }
            }
        }

        Ok(())
    }

    /// Load a custom template from file
    async fn load_custom_template(&mut self, path: &Path) -> Result<(), TemplateError> {
        let content = fs::read_to_string(path).await?;
        let name = path.file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| TemplateError::InvalidPath(path.to_path_buf()))?;

        self.handlebars.register_template_string(name, &content)?;
        Ok(())
    }

    /// Extract variables from template content
    fn extract_variables(&self, content: &str) -> Vec<TemplateVariable> {
        let mut variables = Vec::new();
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();

        for cap in re.captures_iter(content) {
            let name = cap[1].to_string();
            if !variables.iter().any(|v: &TemplateVariable| v.name == name) {
                variables.push(TemplateVariable {
                    name: name.clone(),
                    description: format!("Variable: {}", name),
                    default: None,
                    required: true,
                });
            }
        }

        variables
    }

    /// Render a spec from template
    pub fn render(
        &self,
        template_type: TemplateType,
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        let result = self.handlebars.render(
            template_type.filename(),
            context,
        )?;

        Ok(result)
    }

    /// Get built-in template content
    fn get_builtin_template(&self, template_type: TemplateType) -> String {
        match template_type {
            TemplateType::Feature => FEATURE_TEMPLATE.to_string(),
            TemplateType::Component => COMPONENT_TEMPLATE.to_string(),
            TemplateType::Integration => INTEGRATION_TEMPLATE.to_string(),
            TemplateType::Refactor => REFACTOR_TEMPLATE.to_string(),
            TemplateType::Test => TEST_TEMPLATE.to_string(),
            TemplateType::Custom => FEATURE_TEMPLATE.to_string(),
        }
    }

    /// Create a spec file from template
    pub async fn create_spec(
        &self,
        template_type: TemplateType,
        context: &TemplateContext,
        output_path: &Path,
    ) -> Result<PathBuf, TemplateError> {
        let content = self.render(template_type, context)?;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(output_path, content).await?;

        Ok(output_path.to_path_buf())
    }
}

// Built-in template constants
const FEATURE_TEMPLATE: &str = r#"# Spec {{spec_id}}: {{title}}

## Metadata
- **Phase**: {{phase}} - {{phase_name}}
- **Spec ID**: {{spec_id}}
- **Status**: Planned
- **Dependencies**: {{#each dependencies}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- **Estimated Context**: {{estimated_context}}

## Objective

[Describe the objective of this feature]

## Acceptance Criteria

{{checkbox false}} Criterion 1
{{checkbox false}} Criterion 2
{{checkbox false}} Criterion 3

## Implementation Details

### Core Implementation

```rust
// TODO: Add implementation details
```

## Testing Requirements

{{checkbox false}} Unit tests
{{checkbox false}} Integration tests
{{checkbox false}} Documentation

## Related Specs

{{#each dependencies}}
- **{{this}}**: [Description]
{{/each}}
"#;

const COMPONENT_TEMPLATE: &str = r#"# Spec {{spec_id}}: {{title}}

## Metadata
- **Phase**: {{phase}} - {{phase_name}}
- **Spec ID**: {{spec_id}}
- **Status**: Planned
- **Dependencies**: {{#each dependencies}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- **Estimated Context**: {{estimated_context}}

## Objective

[Describe the component and its purpose]

## Component Architecture

### Public API

```rust
// Component public interface
```

### Internal Structure

```rust
// Internal implementation details
```

## Acceptance Criteria

{{checkbox false}} Component is properly encapsulated
{{checkbox false}} Public API is documented
{{checkbox false}} Error handling is comprehensive

## Testing Requirements

{{checkbox false}} Unit tests for all public methods
{{checkbox false}} Integration tests with dependent components
{{checkbox false}} Performance benchmarks

## Related Specs

{{#each dependencies}}
- **{{this}}**: [Description]
{{/each}}
"#;

const INTEGRATION_TEMPLATE: &str = r#"# Spec {{spec_id}}: {{title}}

## Metadata
- **Phase**: {{phase}} - {{phase_name}}
- **Spec ID**: {{spec_id}}
- **Status**: Planned
- **Dependencies**: {{#each dependencies}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- **Estimated Context**: {{estimated_context}}

## Objective

[Describe the integration and external system]

## Integration Points

### External API

```rust
// External API interaction
```

### Data Flow

[Describe data flow between systems]

## Acceptance Criteria

{{checkbox false}} Integration handles errors gracefully
{{checkbox false}} Retry logic is implemented
{{checkbox false}} Rate limiting is respected

## Testing Requirements

{{checkbox false}} Mock tests for external API
{{checkbox false}} Integration tests with sandbox
{{checkbox false}} Error scenario coverage

## Related Specs

{{#each dependencies}}
- **{{this}}**: [Description]
{{/each}}
"#;

const REFACTOR_TEMPLATE: &str = r#"# Spec {{spec_id}}: {{title}}

## Metadata
- **Phase**: {{phase}} - {{phase_name}}
- **Spec ID**: {{spec_id}}
- **Status**: Planned
- **Dependencies**: {{#each dependencies}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- **Estimated Context**: {{estimated_context}}

## Objective

[Describe the refactoring goals]

## Current State

[Describe current implementation issues]

## Target State

[Describe desired implementation]

## Migration Plan

1. Step 1
2. Step 2
3. Step 3

## Acceptance Criteria

{{checkbox false}} No breaking changes to public API
{{checkbox false}} All existing tests pass
{{checkbox false}} Performance is maintained or improved

## Testing Requirements

{{checkbox false}} Existing test suite passes
{{checkbox false}} New tests for refactored code
{{checkbox false}} Regression testing

## Related Specs

{{#each dependencies}}
- **{{this}}**: [Description]
{{/each}}
"#;

const TEST_TEMPLATE: &str = r#"# Spec {{spec_id}}: {{title}}

## Metadata
- **Phase**: {{phase}} - {{phase_name}}
- **Spec ID**: {{spec_id}}
- **Status**: Planned
- **Dependencies**: {{#each dependencies}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}
- **Estimated Context**: {{estimated_context}}

## Objective

[Describe the testing goals]

## Test Categories

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Test implementation
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_integration() {
    // Integration test
}
```

## Acceptance Criteria

{{checkbox false}} Coverage meets target (>80%)
{{checkbox false}} All edge cases covered
{{checkbox false}} Performance tests included

## Related Specs

{{#each dependencies}}
- **{{this}}**: [Description]
{{/each}}
"#;

/// Errors for template operations
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Invalid template path: {0}")]
    InvalidPath(PathBuf),

    #[error("Template render error: {0}")]
    Render(#[from] handlebars::RenderError),

    #[error("Template parse error: {0}")]
    Parse(#[from] handlebars::TemplateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_context() {
        let context = TemplateContext {
            spec_id: 116,
            title: "Test Spec".to_string(),
            phase: 6,
            phase_name: "Spec System".to_string(),
            slug: "test-spec".to_string(),
            dependencies: vec!["001".to_string(), "002".to_string()],
            estimated_context: "~10%".to_string(),
            custom: HashMap::new(),
        };

        assert_eq!(context.spec_id, 116);
        assert_eq!(context.phase, 6);
    }

    #[tokio::test]
    async fn test_template_engine() {
        let temp = tempfile::TempDir::new().unwrap();
        let engine = TemplateEngine::new(temp.path()).await.unwrap();

        let context = TemplateContext {
            spec_id: 100,
            title: "Test Feature".to_string(),
            phase: 5,
            phase_name: "Advanced Features".to_string(),
            slug: "test-feature".to_string(),
            dependencies: vec![],
            estimated_context: "~8%".to_string(),
            custom: HashMap::new(),
        };

        let result = engine.render(TemplateType::Feature, &context).unwrap();
        assert!(result.contains("Spec 100"));
        assert!(result.contains("Test Feature"));
    }
}
```

## Testing Requirements

- [ ] Unit tests for each template type
- [ ] Tests for template rendering with all variables
- [ ] Tests for custom template loading
- [ ] Tests for template inheritance
- [ ] Tests for helper functions
- [ ] Integration tests for spec file creation
- [ ] Tests for variable extraction
- [ ] Tests for template validation

## Related Specs

- **116-spec-directory.md**: Directory structure for templates
- **118-readme-lookup.md**: README generation uses templates
- **133-spec-rendering.md**: Full rendering pipeline
