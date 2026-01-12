//! Prompt template engine for Forge.

use std::collections::HashMap;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{ForgeError, ForgeResult};

/// Output types supported by the forge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    Specification,
    Code,
    Documentation,
    Analysis,
    Design,
}

impl OutputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputType::Specification => "specification",
            OutputType::Code => "code", 
            OutputType::Documentation => "documentation",
            OutputType::Analysis => "analysis",
            OutputType::Design => "design",
        }
    }
}

/// Roles that participants can take in forge sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Drafter,
    Critic,
    DevilsAdvocate,
    Synthesizer,
    Specialist,
}

impl ParticipantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParticipantRole::Drafter => "drafter",
            ParticipantRole::Critic => "critic",
            ParticipantRole::DevilsAdvocate => "devils_advocate",
            ParticipantRole::Synthesizer => "synthesizer",
            ParticipantRole::Specialist => "specialist",
        }
    }
}

/// Template engine for prompt generation.
pub struct TemplateEngine {
    /// Registered templates.
    templates: HashMap<String, Template>,
    /// Template fragments for composition.
    fragments: HashMap<String, String>,
}

/// A prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name.
    pub name: String,
    /// System prompt template.
    pub system: String,
    /// User prompt template.
    pub user: String,
    /// Required variables.
    pub required_vars: Vec<String>,
    /// Optional variables with defaults.
    pub optional_vars: HashMap<String, String>,
    /// Output type this template is for (None = universal).
    pub output_type: Option<OutputType>,
    /// Role this template is for (None = universal).
    pub role: Option<ParticipantRole>,
}

/// Context for template rendering.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Variable values.
    vars: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Set multiple variables.
    pub fn set_many(mut self, vars: HashMap<String, String>) -> Self {
        self.vars.extend(vars);
        self
    }

    /// Get a variable.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }
}

impl TemplateEngine {
    /// Create a new template engine with built-in templates.
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            fragments: HashMap::new(),
        };

        // Register built-in templates
        engine.register_builtin_templates();
        engine.register_builtin_fragments();

        engine
    }

    /// Register built-in templates.
    fn register_builtin_templates(&mut self) {
        // Draft template
        self.templates.insert("draft".to_string(), Template {
            name: "draft".to_string(),
            system: r#"You are an expert technical writer participating in a collaborative brainstorming session.

Your task is to create an initial draft based on the given topic. This draft will be:
1. Critiqued by other AI models
2. Synthesized with feedback
3. Refined iteratively until convergence

{{#role_context}}

Guidelines:
- Be thorough but concise
- Structure content with clear sections
- Include code examples where relevant
- Mark uncertain areas with [UNCERTAIN]
- Use markdown formatting

{{#output_type_instructions}}"#.to_string(),
            user: r#"# {{topic_title}}

{{topic_description}}

{{#if constraints}}
## Constraints
{{#each constraints}}
- {{this}}
{{/each}}
{{/if}}

{{#if references}}
## Reference Materials
{{#each references}}
### {{name}}
{{content}}
{{/each}}
{{/if}}

Please create an initial draft for this {{output_type}}. Structure your output clearly and be comprehensive."#.to_string(),
            required_vars: vec!["topic_title".to_string(), "topic_description".to_string(), "output_type".to_string()],
            optional_vars: [
                ("role_context".to_string(), "".to_string()),
                ("output_type_instructions".to_string(), "".to_string()),
            ].into_iter().collect(),
            output_type: None,
            role: Some(ParticipantRole::Drafter),
        });

        // Critique template
        self.templates.insert("critique".to_string(), Template {
            name: "critique".to_string(),
            system: r#"You are an expert reviewer participating in a multi-model brainstorming session.

Your Role: {{role_name}}
{{role_context}}

You are critiquing a {{output_type}} draft. Your critique should:
1. Be constructive and actionable
2. Identify specific strengths and weaknesses
3. Provide concrete suggestions for improvement
4. Score the draft objectively

{{evaluation_criteria}}

IMPORTANT: Structure your critique using the exact format specified."#.to_string(),
            user: r#"# Critique Request

## Original Topic
**Title:** {{topic_title}}
**Description:** {{topic_description}}

## Draft to Critique

<draft>
{{draft_content}}
</draft>

## Your Task

Analyze this draft and provide a structured critique.

## Required Output Format

```critique
## Strengths
- [List each strength as a bullet point]

## Weaknesses
- [List each weakness as a bullet point]

## Suggestions
### Suggestion 1
- **Section:** [Section name or "General"]
- **Category:** [correctness/clarity/completeness/code_quality/architecture/performance/security/other]
- **Priority:** [1-5, 1 is highest]
- **Description:** [Detailed suggestion]

## Overall Score
**Score:** [0-100]
**Justification:** [Brief explanation]
```"#.to_string(),
            required_vars: vec![
                "role_name".to_string(),
                "output_type".to_string(),
                "topic_title".to_string(),
                "draft_content".to_string(),
            ],
            optional_vars: [
                ("role_context".to_string(), "".to_string()),
                ("evaluation_criteria".to_string(), "".to_string()),
                ("topic_description".to_string(), "".to_string()),
            ].into_iter().collect(),
            output_type: None,
            role: Some(ParticipantRole::Critic),
        });

        // Synthesis template
        self.templates.insert("synthesis".to_string(), Template {
            name: "synthesis".to_string(),
            system: r#"You are the synthesizer in a multi-model brainstorming session.

Your role is to:
1. Analyze critiques from multiple AI reviewers
2. Identify areas of consensus and conflict
3. Merge improvements into an updated draft
4. Resolve conflicts with clear rationale
5. Track all changes made

Key principles:
- Give weight to suggestions that multiple critics agree on
- When critics disagree, consider the strength of their arguments
- Maintain the original structure unless changes are necessary
- Preserve what works well while addressing weaknesses
- Be explicit about trade-offs when resolving conflicts"#.to_string(),
            user: r#"# Synthesis Request

## Original Topic
**Title:** {{topic_title}}

## Current Draft
<draft>
{{current_draft}}
</draft>

## Critique Summary
{{critique_summary}}

## Your Task

Create an improved version addressing the feedback.

## Required Output Format

```synthesis
## Conflict Resolutions
[List any conflicts and how you resolved them]

## Changes Made
[List significant changes with rationale]

## Improved Draft
[Complete improved draft]
```"#.to_string(),
            required_vars: vec![
                "topic_title".to_string(),
                "current_draft".to_string(),
                "critique_summary".to_string(),
            ],
            optional_vars: HashMap::new(),
            output_type: None,
            role: Some(ParticipantRole::Synthesizer),
        });

        // Refinement template
        self.templates.insert("refinement".to_string(), Template {
            name: "refinement".to_string(),
            system: r#"You are refining a draft, focusing specifically on: {{focus_area}}

This is refinement pass {{depth}} of maximum {{max_depth}}.

{{focus_instructions}}

Guidelines:
1. Focus ONLY on the specified area
2. Make targeted, high-impact improvements
3. Preserve the overall structure
4. Be more aggressive at lower depths, more conservative at higher depths"#.to_string(),
            user: r#"# Refinement Request

## Topic
**Title:** {{topic_title}}
**Focus Area:** {{focus_area}}

## Current Draft
<draft>
{{current_draft}}
</draft>

## Issues to Address
{{issues}}

## Your Task

Refine the draft with focus on **{{focus_area}}**.
Output the COMPLETE refined document."#.to_string(),
            required_vars: vec![
                "focus_area".to_string(),
                "topic_title".to_string(),
                "current_draft".to_string(),
            ],
            optional_vars: [
                ("depth".to_string(), "1".to_string()),
                ("max_depth".to_string(), "3".to_string()),
                ("focus_instructions".to_string(), "".to_string()),
                ("issues".to_string(), "No specific issues flagged.".to_string()),
            ].into_iter().collect(),
            output_type: None,
            role: None,
        });

        // Convergence template
        self.templates.insert("convergence".to_string(), Template {
            name: "convergence".to_string(),
            system: r#"You are evaluating whether a draft has reached a satisfactory state.

Evaluate the draft and provide:
1. Whether you agree it's ready (yes/no)
2. A score from 0-100
3. Any remaining concerns"#.to_string(),
            user: r#"## Topic: {{topic_title}}

## Draft to Evaluate
{{draft_content}}

## Instructions

Is this draft ready for finalization?

Respond in this exact format:
AGREES: [yes/no]
SCORE: [0-100]
CONCERNS:
- [concern 1]
- [concern 2]
(or "none" if no concerns)"#.to_string(),
            required_vars: vec!["topic_title".to_string(), "draft_content".to_string()],
            optional_vars: HashMap::new(),
            output_type: None,
            role: None,
        });
    }

    /// Register built-in fragments.
    fn register_builtin_fragments(&mut self) {
        // Output type instructions
        self.fragments.insert("spec_instructions".to_string(), r#"
Output Type: Specification
- Use clear section headers
- Include acceptance criteria
- Provide implementation details with code
- Define testing requirements"#.to_string());

        self.fragments.insert("code_instructions".to_string(), r#"
Output Type: Code
- Write production-ready code
- Include proper error handling
- Add documentation comments
- Follow language idioms"#.to_string());

        self.fragments.insert("docs_instructions".to_string(), r#"
Output Type: Documentation
- Write for the target audience
- Use clear examples
- Organize logically
- Keep it concise but complete"#.to_string());

        // Role contexts
        self.fragments.insert("critic_context".to_string(), r#"
As a critic, you should:
- Be thorough in identifying issues
- Prioritize actionable feedback
- Balance criticism with acknowledgment of strengths"#.to_string());

        self.fragments.insert("devils_advocate_context".to_string(), r#"
As the devil's advocate:
- Challenge assumptions
- Find edge cases
- Question design decisions
- Consider failure modes"#.to_string());
    }

    /// Render a template with context.
    pub fn render(&self, template_name: &str, context: &TemplateContext) -> ForgeResult<(String, String)> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| ForgeError::Config(format!("Template '{}' not found", template_name)))?;

        // Check required variables
        for var in &template.required_vars {
            if !context.vars.contains_key(var) {
                return Err(ForgeError::Config(format!(
                    "Missing required variable '{}' for template '{}'",
                    var, template_name
                )));
            }
        }

        // Render system prompt
        let system = self.render_string(&template.system, context, &template.optional_vars)?;

        // Render user prompt
        let user = self.render_string(&template.user, context, &template.optional_vars)?;

        Ok((system, user))
    }

    /// Render a string with variable substitution.
    fn render_string(
        &self,
        template: &str,
        context: &TemplateContext,
        defaults: &HashMap<String, String>,
    ) -> ForgeResult<String> {
        let mut result = template.to_string();

        // Simple variable substitution: {{var_name}}
        let var_pattern = Regex::new(r"\{\{(\w+)\}\}").unwrap();

        result = var_pattern.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];

            context.get(var_name)
                .or_else(|| defaults.get(var_name))
                .cloned()
                .unwrap_or_else(|| format!("{{{{{}}}}}", var_name))
        }).to_string();

        // Fragment inclusion: {{#fragment_name}}
        let fragment_pattern = Regex::new(r"\{\{#(\w+)\}\}").unwrap();

        result = fragment_pattern.replace_all(&result, |caps: &regex::Captures| {
            let fragment_name = &caps[1];

            // First check context, then fragments
            context.get(fragment_name)
                .or_else(|| self.fragments.get(fragment_name))
                .cloned()
                .unwrap_or_default()
        }).to_string();

        // Conditional blocks: {{#if var}}...{{/if}}
        let if_pattern = Regex::new(r"\{\{#if\s+(\w+)\}\}([\s\S]*?)\{\{/if\}\}").unwrap();

        result = if_pattern.replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            let content = &caps[2];

            if context.get(var_name).map(|v| !v.is_empty()).unwrap_or(false) {
                content.to_string()
            } else {
                String::new()
            }
        }).to_string();

        Ok(result)
    }

    /// Load custom templates from a directory.
    pub async fn load_templates_from_dir(&mut self, dir: &Path) -> ForgeResult<usize> {
        let mut loaded = 0;

        let mut entries = tokio::fs::read_dir(dir).await
            .map_err(|e| ForgeError::Template(format!("Failed to read template dir: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| ForgeError::Template(format!("Failed to iterate template dir: {}", e)))?
        {
            let path = entry.path();

            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                let content = tokio::fs::read_to_string(&path).await
                    .map_err(|e| ForgeError::Template(format!("Failed to read template file: {}", e)))?;

                let template: Template = serde_yaml::from_str(&content)
                    .map_err(|e| ForgeError::Template(format!("Failed to parse template YAML: {}", e)))?;

                self.templates.insert(template.name.clone(), template);
                loaded += 1;
            }
        }

        Ok(loaded)
    }

    /// Get a template by name.
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// List available templates.
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Register a custom template.
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Validate a template - check that it can be rendered with required variables.
    pub fn validate_template(&self, template: &Template) -> ForgeResult<()> {
        // Create a minimal context with all required variables
        let mut context = TemplateContext::new();
        for var in &template.required_vars {
            context = context.set(var, "test_value");
        }

        // Try to render the template
        self.render_string(&template.system, &context, &template.optional_vars)?;
        self.render_string(&template.user, &context, &template.optional_vars)?;

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let engine = TemplateEngine::new();
        let context = TemplateContext::new()
            .set("topic_title", "Test Topic")
            .set("topic_description", "A test description")
            .set("output_type", "specification");

        let (system, user) = engine.render("draft", &context).unwrap();

        assert!(system.contains("technical writer"));
        assert!(user.contains("Test Topic"));
    }

    #[test]
    fn test_missing_required_var() {
        let engine = TemplateEngine::new();
        let context = TemplateContext::new()
            .set("topic_title", "Test");

        let result = engine.render("draft", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_substitution() {
        let engine = TemplateEngine::new();
        let template_str = "Hello {{name}}, your score is {{score}}!";
        let context = TemplateContext::new()
            .set("name", "Alice")
            .set("score", "95");

        let result = engine.render_string(template_str, &context, &HashMap::new()).unwrap();
        assert_eq!(result, "Hello Alice, your score is 95!");
    }

    #[test]
    fn test_conditional_blocks() {
        let engine = TemplateEngine::new();
        let template_str = "{{#if show_section}}This section is visible{{/if}}";

        // Test with variable present
        let context = TemplateContext::new().set("show_section", "yes");
        let result = engine.render_string(template_str, &context, &HashMap::new()).unwrap();
        assert_eq!(result, "This section is visible");

        // Test with variable absent
        let context = TemplateContext::new();
        let result = engine.render_string(template_str, &context, &HashMap::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_fragment_inclusion() {
        let mut engine = TemplateEngine::new();
        engine.fragments.insert("greeting".to_string(), "Hello there!".to_string());

        let template_str = "{{#greeting}} How are you?";
        let context = TemplateContext::new();

        let result = engine.render_string(template_str, &context, &HashMap::new()).unwrap();
        assert_eq!(result, "Hello there! How are you?");
    }

    #[test]
    fn test_builtin_templates_exist() {
        let engine = TemplateEngine::new();
        assert!(engine.get("draft").is_some());
        assert!(engine.get("critique").is_some());
        assert!(engine.get("synthesis").is_some());
        assert!(engine.get("refinement").is_some());
        assert!(engine.get("convergence").is_some());
    }

    #[test]
    fn test_template_validation() {
        let engine = TemplateEngine::new();
        
        let valid_template = Template {
            name: "test".to_string(),
            system: "Hello {{name}}".to_string(),
            user: "Your task is {{task}}".to_string(),
            required_vars: vec!["name".to_string(), "task".to_string()],
            optional_vars: HashMap::new(),
            output_type: None,
            role: None,
        };

        assert!(engine.validate_template(&valid_template).is_ok());
    }

    #[tokio::test]
    async fn test_load_custom_templates() {
        use tempfile::TempDir;
        use std::fs;

        // Create a temporary directory with a custom template
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("custom.yaml");

        let template_yaml = r#"
name: custom_test
system: "You are testing {{feature}}"
user: "Please test: {{test_case}}"
required_vars: ["feature", "test_case"]
optional_vars:
  context: "default context"
output_type: null
role: null
"#;

        fs::write(&template_path, template_yaml).unwrap();

        // Load the template
        let mut engine = TemplateEngine::new();
        let initial_count = engine.list_templates().len();
        
        let loaded = engine.load_templates_from_dir(temp_dir.path()).await.unwrap();
        
        assert_eq!(loaded, 1);
        assert_eq!(engine.list_templates().len(), initial_count + 1);
        assert!(engine.get("custom_test").is_some());

        // Test rendering the custom template
        let context = TemplateContext::new()
            .set("feature", "template loading")
            .set("test_case", "load from YAML");

        let (system, user) = engine.render("custom_test", &context).unwrap();
        assert!(system.contains("testing template loading"));
        assert!(user.contains("load from YAML"));
    }

    #[test]
    fn test_output_type_specific_templates() {
        let mut engine = TemplateEngine::new();
        
        // Create a code-specific template
        let code_template = Template {
            name: "code_specific".to_string(),
            system: "You are a code generator for {{language}}".to_string(),
            user: "Generate {{output_type}} code: {{description}}".to_string(),
            required_vars: vec!["language".to_string(), "description".to_string(), "output_type".to_string()],
            optional_vars: HashMap::new(),
            output_type: Some(OutputType::Code),
            role: None,
        };
        
        engine.register(code_template);
        assert!(engine.get("code_specific").is_some());
        assert_eq!(engine.get("code_specific").unwrap().output_type, Some(OutputType::Code));
    }

    #[test]
    fn test_template_inheritance() {
        let engine = TemplateEngine::new();
        
        // Test that fragments are properly included
        let template_str = "Base content: {{#spec_instructions}} Additional info.";
        let context = TemplateContext::new();
        
        let result = engine.render_string(template_str, &context, &HashMap::new()).unwrap();
        
        // Should include the spec instructions fragment
        assert!(result.contains("Use clear section headers"));
        assert!(result.contains("Additional info."));
    }
}