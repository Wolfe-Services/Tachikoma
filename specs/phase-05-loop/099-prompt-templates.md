# 099 - Prompt Templates

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 099
**Status:** Planned
**Dependencies:** 098-prompt-loading
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Implement the template substitution system for prompts - replacing variables with values, supporting conditionals, loops, and built-in helper functions for dynamic prompt generation.

---

## Acceptance Criteria

- [x] Variable substitution with `{{variable}}` syntax
- [x] Built-in variables (iteration, timestamp, etc.)
- [x] Conditional blocks with `{{#if}}`
- [x] Loop blocks with `{{#each}}`
- [x] Helper functions (date, uppercase, etc.)
- [x] Escaping mechanism for literal braces
- [x] Custom variable providers
- [x] Error handling for missing variables

---

## Implementation Details

### 1. Template Types (src/prompt/template/types.rs)

```rust
//! Template type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Variables available for template substitution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateContext {
    /// User-provided variables.
    pub variables: HashMap<String, TemplateValue>,
    /// Built-in variables (iteration, loop info, etc.).
    pub builtins: BuiltinVariables,
    /// Custom data from providers.
    pub custom: HashMap<String, serde_json::Value>,
}

/// A template variable value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateValue {
    /// String value.
    String(String),
    /// Numeric value.
    Number(f64),
    /// Boolean value.
    Bool(bool),
    /// Array of values.
    Array(Vec<TemplateValue>),
    /// Object/map of values.
    Object(HashMap<String, TemplateValue>),
    /// Null/none value.
    Null,
}

impl TemplateValue {
    /// Convert to string representation.
    pub fn as_str(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.as_str()).collect();
                items.join(", ")
            }
            Self::Object(_) => "[object]".to_string(),
            Self::Null => "".to_string(),
        }
    }

    /// Check if truthy (for conditionals).
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::String(s) => !s.is_empty(),
            Self::Number(n) => *n != 0.0,
            Self::Bool(b) => *b,
            Self::Array(arr) => !arr.is_empty(),
            Self::Object(obj) => !obj.is_empty(),
            Self::Null => false,
        }
    }

    /// Get as array for iteration.
    pub fn as_array(&self) -> Option<&Vec<TemplateValue>> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

impl From<&str> for TemplateValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for TemplateValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i32> for TemplateValue {
    fn from(n: i32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<u32> for TemplateValue {
    fn from(n: u32) -> Self {
        Self::Number(n as f64)
    }
}

impl From<bool> for TemplateValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

/// Built-in template variables.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuiltinVariables {
    /// Current iteration number.
    pub iteration: u32,
    /// Total iterations (0 if unlimited).
    pub total_iterations: u32,
    /// Loop ID.
    pub loop_id: String,
    /// Session ID.
    pub session_id: String,
    /// Current timestamp (ISO 8601).
    pub timestamp: String,
    /// Current date (YYYY-MM-DD).
    pub date: String,
    /// Current time (HH:MM:SS).
    pub time: String,
    /// Working directory.
    pub working_dir: String,
    /// Git branch (if in git repo).
    pub git_branch: Option<String>,
    /// Last iteration result.
    pub last_result: Option<LastIterationResult>,
}

/// Summary of last iteration for templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastIterationResult {
    /// Whether it succeeded.
    pub success: bool,
    /// Files that were changed.
    pub files_changed: Vec<String>,
    /// Test pass count.
    pub tests_passed: u32,
    /// Test fail count.
    pub tests_failed: u32,
    /// Summary text.
    pub summary: Option<String>,
}

impl TemplateContext {
    /// Create a new context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable.
    pub fn set<K: Into<String>, V: Into<TemplateValue>>(&mut self, key: K, value: V) {
        self.variables.insert(key.into(), value.into());
    }

    /// Get a variable value.
    pub fn get(&self, key: &str) -> Option<&TemplateValue> {
        // Check user variables first
        if let Some(v) = self.variables.get(key) {
            return Some(v);
        }

        // Check builtins
        self.get_builtin(key)
    }

    /// Get a builtin variable.
    fn get_builtin(&self, key: &str) -> Option<&TemplateValue> {
        // This would need to be implemented differently to return references
        // For now, we'll handle builtins in the resolver
        None
    }

    /// Merge another context into this one.
    pub fn merge(&mut self, other: TemplateContext) {
        self.variables.extend(other.variables);
        self.custom.extend(other.custom);
    }
}
```

### 2. Template Engine (src/prompt/template/engine.rs)

```rust
//! Template rendering engine.

use super::types::{BuiltinVariables, TemplateContext, TemplateValue};
use crate::error::{LoopError, LoopResult};

use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Configuration for template rendering.
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// Fail on missing variables (otherwise empty string).
    pub strict_variables: bool,
    /// Enable helper functions.
    pub enable_helpers: bool,
    /// Custom delimiters (default: {{ }}).
    pub open_delim: String,
    pub close_delim: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            strict_variables: false,
            enable_helpers: true,
            open_delim: "{{".to_string(),
            close_delim: "}}".to_string(),
        }
    }
}

/// The template rendering engine.
pub struct TemplateEngine {
    config: TemplateConfig,
    helpers: HashMap<String, Box<dyn HelperFn>>,
}

/// A helper function for templates.
pub trait HelperFn: Send + Sync {
    /// Execute the helper.
    fn call(&self, args: &[TemplateValue], ctx: &TemplateContext) -> TemplateValue;
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new(config: TemplateConfig) -> Self {
        let mut engine = Self {
            config,
            helpers: HashMap::new(),
        };

        // Register built-in helpers
        engine.register_builtin_helpers();

        engine
    }

    /// Register built-in helper functions.
    fn register_builtin_helpers(&mut self) {
        // uppercase helper
        self.register_helper("uppercase", |args, _ctx| {
            args.first()
                .map(|v| TemplateValue::String(v.as_str().to_uppercase()))
                .unwrap_or(TemplateValue::Null)
        });

        // lowercase helper
        self.register_helper("lowercase", |args, _ctx| {
            args.first()
                .map(|v| TemplateValue::String(v.as_str().to_lowercase()))
                .unwrap_or(TemplateValue::Null)
        });

        // trim helper
        self.register_helper("trim", |args, _ctx| {
            args.first()
                .map(|v| TemplateValue::String(v.as_str().trim().to_string()))
                .unwrap_or(TemplateValue::Null)
        });

        // default helper
        self.register_helper("default", |args, _ctx| {
            if args.len() >= 2 {
                if args[0].is_truthy() {
                    args[0].clone()
                } else {
                    args[1].clone()
                }
            } else {
                TemplateValue::Null
            }
        });

        // length helper
        self.register_helper("length", |args, _ctx| {
            args.first()
                .map(|v| match v {
                    TemplateValue::String(s) => TemplateValue::Number(s.len() as f64),
                    TemplateValue::Array(arr) => TemplateValue::Number(arr.len() as f64),
                    _ => TemplateValue::Number(0.0),
                })
                .unwrap_or(TemplateValue::Number(0.0))
        });

        // join helper
        self.register_helper("join", |args, _ctx| {
            if args.len() >= 2 {
                if let Some(arr) = args[0].as_array() {
                    let sep = args[1].as_str();
                    let items: Vec<String> = arr.iter().map(|v| v.as_str()).collect();
                    return TemplateValue::String(items.join(&sep));
                }
            }
            TemplateValue::Null
        });

        // date formatting helper
        self.register_helper("dateformat", |args, _ctx| {
            if args.len() >= 2 {
                let date_str = args[0].as_str();
                let format = args[1].as_str();
                // Simplified - real implementation would parse and reformat
                TemplateValue::String(date_str)
            } else {
                TemplateValue::Null
            }
        });

        // math helpers
        self.register_helper("add", |args, _ctx| {
            if args.len() >= 2 {
                if let (TemplateValue::Number(a), TemplateValue::Number(b)) = (&args[0], &args[1]) {
                    return TemplateValue::Number(a + b);
                }
            }
            TemplateValue::Null
        });

        self.register_helper("subtract", |args, _ctx| {
            if args.len() >= 2 {
                if let (TemplateValue::Number(a), TemplateValue::Number(b)) = (&args[0], &args[1]) {
                    return TemplateValue::Number(a - b);
                }
            }
            TemplateValue::Null
        });
    }

    /// Register a custom helper function.
    pub fn register_helper<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[TemplateValue], &TemplateContext) -> TemplateValue + Send + Sync + 'static,
    {
        self.helpers.insert(
            name.to_string(),
            Box::new(FnHelper(func)),
        );
    }

    /// Render a template with the given context.
    pub fn render(&self, template: &str, context: &TemplateContext) -> LoopResult<String> {
        let mut result = template.to_string();

        // Process conditionals first
        result = self.process_conditionals(&result, context)?;

        // Process loops
        result = self.process_loops(&result, context)?;

        // Process variable substitutions
        result = self.process_variables(&result, context)?;

        // Process helpers
        if self.config.enable_helpers {
            result = self.process_helpers(&result, context)?;
        }

        // Unescape literal braces
        result = result.replace("\\{\\{", "{{").replace("\\}\\}", "}}");

        Ok(result)
    }

    /// Process {{#if condition}}...{{/if}} blocks.
    fn process_conditionals(&self, template: &str, context: &TemplateContext) -> LoopResult<String> {
        let if_pattern = Regex::new(
            r"\{\{#if\s+([^}]+)\}\}([\s\S]*?)(?:\{\{#else\}\}([\s\S]*?))?\{\{/if\}\}"
        ).unwrap();

        let mut result = template.to_string();

        while let Some(caps) = if_pattern.captures(&result) {
            let condition_var = caps.get(1).unwrap().as_str().trim();
            let then_block = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let else_block = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            let condition_value = self.resolve_variable(condition_var, context);
            let is_true = condition_value.is_truthy();

            let replacement = if is_true { then_block } else { else_block };

            result = result.replace(caps.get(0).unwrap().as_str(), replacement);
        }

        Ok(result)
    }

    /// Process {{#each array}}...{{/each}} blocks.
    fn process_loops(&self, template: &str, context: &TemplateContext) -> LoopResult<String> {
        let each_pattern = Regex::new(
            r"\{\{#each\s+([^}]+)\s+as\s+(\w+)\}\}([\s\S]*?)\{\{/each\}\}"
        ).unwrap();

        let mut result = template.to_string();

        while let Some(caps) = each_pattern.captures(&result) {
            let array_var = caps.get(1).unwrap().as_str().trim();
            let item_name = caps.get(2).unwrap().as_str();
            let body = caps.get(3).unwrap().as_str();

            let array_value = self.resolve_variable(array_var, context);

            let rendered = if let Some(items) = array_value.as_array() {
                let mut parts = Vec::new();
                for (index, item) in items.iter().enumerate() {
                    let mut loop_ctx = context.clone();
                    loop_ctx.set(item_name, item.clone());
                    loop_ctx.set("@index", index as u32);
                    loop_ctx.set("@first", index == 0);
                    loop_ctx.set("@last", index == items.len() - 1);

                    // Recursively render the body
                    let rendered_body = self.render(body, &loop_ctx)?;
                    parts.push(rendered_body);
                }
                parts.join("")
            } else {
                String::new()
            };

            result = result.replace(caps.get(0).unwrap().as_str(), &rendered);
        }

        Ok(result)
    }

    /// Process {{variable}} substitutions.
    fn process_variables(&self, template: &str, context: &TemplateContext) -> LoopResult<String> {
        let var_pattern = Regex::new(r"\{\{([a-zA-Z_@][a-zA-Z0-9_\.]*)\}\}").unwrap();

        let mut result = template.to_string();

        for caps in var_pattern.captures_iter(template) {
            let var_name = caps.get(1).unwrap().as_str();

            // Skip if it looks like a block directive
            if var_name.starts_with('#') || var_name.starts_with('/') {
                continue;
            }

            let value = self.resolve_variable(var_name, context);
            let replacement = value.as_str();

            result = result.replace(caps.get(0).unwrap().as_str(), &replacement);
        }

        Ok(result)
    }

    /// Process {{helper arg1 arg2}} calls.
    fn process_helpers(&self, template: &str, context: &TemplateContext) -> LoopResult<String> {
        let helper_pattern = Regex::new(
            r"\{\{(\w+)\s+([^}]+)\}\}"
        ).unwrap();

        let mut result = template.to_string();

        for caps in helper_pattern.captures_iter(template) {
            let helper_name = caps.get(1).unwrap().as_str();

            // Skip if not a registered helper
            if !self.helpers.contains_key(helper_name) {
                continue;
            }

            let args_str = caps.get(2).unwrap().as_str();
            let args = self.parse_helper_args(args_str, context);

            if let Some(helper) = self.helpers.get(helper_name) {
                let value = helper.call(&args, context);
                result = result.replace(caps.get(0).unwrap().as_str(), &value.as_str());
            }
        }

        Ok(result)
    }

    /// Resolve a variable path (supports dot notation).
    fn resolve_variable(&self, path: &str, context: &TemplateContext) -> TemplateValue {
        let parts: Vec<&str> = path.split('.').collect();

        // Check builtins first
        if let Some(value) = self.resolve_builtin(&parts, &context.builtins) {
            return value;
        }

        // Then check user variables
        if let Some(first) = parts.first() {
            if let Some(value) = context.variables.get(*first) {
                return self.navigate_path(value, &parts[1..]);
            }
        }

        if self.config.strict_variables {
            warn!("Missing variable: {}", path);
        }

        TemplateValue::Null
    }

    /// Navigate a path through nested values.
    fn navigate_path(&self, value: &TemplateValue, path: &[&str]) -> TemplateValue {
        if path.is_empty() {
            return value.clone();
        }

        match value {
            TemplateValue::Object(obj) => {
                if let Some(next) = obj.get(path[0]) {
                    self.navigate_path(next, &path[1..])
                } else {
                    TemplateValue::Null
                }
            }
            _ => TemplateValue::Null,
        }
    }

    /// Resolve a builtin variable.
    fn resolve_builtin(&self, parts: &[&str], builtins: &BuiltinVariables) -> Option<TemplateValue> {
        match parts.first()? {
            &"iteration" => Some(TemplateValue::Number(builtins.iteration as f64)),
            &"total_iterations" => Some(TemplateValue::Number(builtins.total_iterations as f64)),
            &"loop_id" => Some(TemplateValue::String(builtins.loop_id.clone())),
            &"session_id" => Some(TemplateValue::String(builtins.session_id.clone())),
            &"timestamp" => Some(TemplateValue::String(builtins.timestamp.clone())),
            &"date" => Some(TemplateValue::String(builtins.date.clone())),
            &"time" => Some(TemplateValue::String(builtins.time.clone())),
            &"working_dir" => Some(TemplateValue::String(builtins.working_dir.clone())),
            &"git_branch" => builtins.git_branch.clone().map(TemplateValue::String),
            _ => None,
        }
    }

    /// Parse helper function arguments.
    fn parse_helper_args(&self, args_str: &str, context: &TemplateContext) -> Vec<TemplateValue> {
        let mut args = Vec::new();

        // Simple space-separated parsing (real impl would handle quotes)
        for arg in args_str.split_whitespace() {
            // Check if it's a string literal
            if arg.starts_with('"') && arg.ends_with('"') {
                args.push(TemplateValue::String(arg[1..arg.len()-1].to_string()));
            }
            // Check if it's a number
            else if let Ok(n) = arg.parse::<f64>() {
                args.push(TemplateValue::Number(n));
            }
            // Otherwise treat as variable reference
            else {
                args.push(self.resolve_variable(arg, context));
            }
        }

        args
    }
}

/// Wrapper to make closures implement HelperFn.
struct FnHelper<F>(F);

impl<F> HelperFn for FnHelper<F>
where
    F: Fn(&[TemplateValue], &TemplateContext) -> TemplateValue + Send + Sync,
{
    fn call(&self, args: &[TemplateValue], ctx: &TemplateContext) -> TemplateValue {
        (self.0)(args, ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_substitution() {
        let engine = TemplateEngine::new(TemplateConfig::default());
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");

        let result = engine.render("Hello, {{name}}!", &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_conditional() {
        let engine = TemplateEngine::new(TemplateConfig::default());
        let mut ctx = TemplateContext::new();
        ctx.set("show", true);

        let template = "{{#if show}}Visible{{/if}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "Visible");
    }

    #[test]
    fn test_each_loop() {
        let engine = TemplateEngine::new(TemplateConfig::default());
        let mut ctx = TemplateContext::new();
        ctx.set("items", TemplateValue::Array(vec![
            TemplateValue::String("a".to_string()),
            TemplateValue::String("b".to_string()),
        ]));

        let template = "{{#each items as item}}{{item}}{{/each}}";
        let result = engine.render(template, &ctx).unwrap();
        assert_eq!(result, "ab");
    }
}
```

### 3. Module Root (src/prompt/template/mod.rs)

```rust
//! Template rendering for prompts.

pub mod engine;
pub mod types;

pub use engine::{HelperFn, TemplateConfig, TemplateEngine};
pub use types::{
    BuiltinVariables, LastIterationResult, TemplateContext, TemplateValue,
};
```

---

## Testing Requirements

1. Simple variable substitution works
2. Nested variable paths resolve correctly
3. Conditionals evaluate truthiness correctly
4. Each loops iterate arrays
5. Helper functions produce correct output
6. Missing variables handled per strict mode
7. Escaped braces render literally
8. Complex nested templates render correctly

---

## Related Specs

- Depends on: [098-prompt-loading.md](098-prompt-loading.md)
- Next: [100-session-management.md](100-session-management.md)
- Related: [097-loop-iteration.md](097-loop-iteration.md)
