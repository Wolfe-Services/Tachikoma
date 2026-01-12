# 054 - Tool Definitions (Schema Types)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 054
**Status:** Planned
**Dependencies:** 051-backend-trait
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define the types and structures for tool/function definitions that can be provided to LLM backends. These definitions describe what tools are available, their parameters, and how they should be invoked.

---

## Acceptance Criteria

- [ ] `ToolDefinition` struct for describing tools
- [ ] JSON Schema support for parameters
- [ ] `ToolParameter` types for common parameter patterns
- [ ] Builder pattern for tool construction
- [ ] Serialization to provider-specific formats
- [ ] Validation of tool definitions

---

## Implementation Details

### 1. Tool Definition Types (src/tool/definition.rs)

```rust
//! Tool definition types for LLM function calling.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Definition of a tool that can be called by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique name of the tool.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Input parameters schema.
    pub parameters: ToolParameters,
    /// Whether this tool is currently enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Tool metadata.
    #[serde(default)]
    pub metadata: ToolMetadata,
}

fn default_true() -> bool {
    true
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: ToolParameters::default(),
            enabled: true,
            metadata: ToolMetadata::default(),
        }
    }

    /// Create a builder for constructing a tool definition.
    pub fn builder(name: impl Into<String>) -> ToolDefinitionBuilder {
        ToolDefinitionBuilder::new(name)
    }

    /// Add a required parameter.
    pub fn with_required_param(mut self, param: ToolParameter) -> Self {
        let name = param.name.clone();
        self.parameters.properties.insert(name.clone(), param);
        if !self.parameters.required.contains(&name) {
            self.parameters.required.push(name);
        }
        self
    }

    /// Add an optional parameter.
    pub fn with_optional_param(mut self, param: ToolParameter) -> Self {
        self.parameters.properties.insert(param.name.clone(), param);
        self
    }

    /// Validate the tool definition.
    pub fn validate(&self) -> Result<(), ToolValidationError> {
        if self.name.is_empty() {
            return Err(ToolValidationError::EmptyName);
        }

        if self.name.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            return Err(ToolValidationError::InvalidName(self.name.clone()));
        }

        if self.description.is_empty() {
            return Err(ToolValidationError::EmptyDescription);
        }

        // Check required parameters exist in properties
        for req in &self.parameters.required {
            if !self.parameters.properties.contains_key(req) {
                return Err(ToolValidationError::MissingRequiredParam(req.clone()));
            }
        }

        Ok(())
    }

    /// Convert to Claude API format.
    pub fn to_claude_format(&self) -> JsonValue {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "input_schema": self.parameters.to_json_schema()
        })
    }

    /// Convert to OpenAI API format.
    pub fn to_openai_format(&self) -> JsonValue {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters.to_json_schema()
            }
        })
    }

    /// Convert to Gemini API format.
    pub fn to_gemini_format(&self) -> JsonValue {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters.to_json_schema()
        })
    }
}

/// Builder for tool definitions.
#[derive(Debug)]
pub struct ToolDefinitionBuilder {
    name: String,
    description: Option<String>,
    parameters: ToolParameters,
    enabled: bool,
    metadata: ToolMetadata,
}

impl ToolDefinitionBuilder {
    /// Create a new builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: ToolParameters::default(),
            enabled: true,
            metadata: ToolMetadata::default(),
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a required string parameter.
    pub fn required_string(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        let param = ToolParameter::string(name, description);
        let param_name = param.name.clone();
        self.parameters.properties.insert(param_name.clone(), param);
        self.parameters.required.push(param_name);
        self
    }

    /// Add a required integer parameter.
    pub fn required_int(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        let param = ToolParameter::integer(name, description);
        let param_name = param.name.clone();
        self.parameters.properties.insert(param_name.clone(), param);
        self.parameters.required.push(param_name);
        self
    }

    /// Add a required boolean parameter.
    pub fn required_bool(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        let param = ToolParameter::boolean(name, description);
        let param_name = param.name.clone();
        self.parameters.properties.insert(param_name.clone(), param);
        self.parameters.required.push(param_name);
        self
    }

    /// Add an optional string parameter.
    pub fn optional_string(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        let param = ToolParameter::string(name, description);
        self.parameters.properties.insert(param.name.clone(), param);
        self
    }

    /// Add an optional integer parameter.
    pub fn optional_int(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        let param = ToolParameter::integer(name, description);
        self.parameters.properties.insert(param.name.clone(), param);
        self
    }

    /// Add a custom parameter.
    pub fn param(mut self, param: ToolParameter, required: bool) -> Self {
        let name = param.name.clone();
        self.parameters.properties.insert(name.clone(), param);
        if required {
            self.parameters.required.push(name);
        }
        self
    }

    /// Set enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.extra.insert(key.into(), value.into());
        self
    }

    /// Set danger level.
    pub fn danger_level(mut self, level: DangerLevel) -> Self {
        self.metadata.danger_level = level;
        self
    }

    /// Build the tool definition.
    pub fn build(self) -> Result<ToolDefinition, ToolValidationError> {
        let description = self.description.ok_or(ToolValidationError::EmptyDescription)?;

        let tool = ToolDefinition {
            name: self.name,
            description,
            parameters: self.parameters,
            enabled: self.enabled,
            metadata: self.metadata,
        };

        tool.validate()?;
        Ok(tool)
    }
}

/// Parameters schema for a tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolParameters {
    /// Parameter definitions.
    #[serde(default)]
    pub properties: HashMap<String, ToolParameter>,
    /// Required parameter names.
    #[serde(default)]
    pub required: Vec<String>,
}

impl ToolParameters {
    /// Convert to JSON Schema format.
    pub fn to_json_schema(&self) -> JsonValue {
        let properties: HashMap<String, JsonValue> = self
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json_schema()))
            .collect();

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": self.required
        })
    }
}

/// A single tool parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name.
    pub name: String,
    /// Parameter description.
    pub description: String,
    /// Parameter type.
    #[serde(rename = "type")]
    pub param_type: ParameterType,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,
    /// Enum values (for string enums).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// Minimum value (for numbers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum value (for numbers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Array item type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ToolParameter>>,
}

impl ToolParameter {
    /// Create a string parameter.
    pub fn string(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::String,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create an integer parameter.
    pub fn integer(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::Integer,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create a number parameter.
    pub fn number(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::Number,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create a boolean parameter.
    pub fn boolean(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::Boolean,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Create an array parameter.
    pub fn array(name: impl Into<String>, description: impl Into<String>, items: ToolParameter) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::Array,
            default: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            items: Some(Box::new(items)),
        }
    }

    /// Create a string enum parameter.
    pub fn string_enum(
        name: impl Into<String>,
        description: impl Into<String>,
        values: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type: ParameterType::String,
            default: None,
            enum_values: Some(values),
            minimum: None,
            maximum: None,
            items: None,
        }
    }

    /// Add a default value.
    pub fn with_default(mut self, default: JsonValue) -> Self {
        self.default = Some(default);
        self
    }

    /// Add range constraints.
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.minimum = Some(min);
        self.maximum = Some(max);
        self
    }

    /// Convert to JSON Schema format.
    pub fn to_json_schema(&self) -> JsonValue {
        let mut schema = serde_json::json!({
            "type": self.param_type.as_str(),
            "description": self.description
        });

        if let Some(default) = &self.default {
            schema["default"] = default.clone();
        }

        if let Some(enum_values) = &self.enum_values {
            schema["enum"] = serde_json::json!(enum_values);
        }

        if let Some(min) = self.minimum {
            schema["minimum"] = serde_json::json!(min);
        }

        if let Some(max) = self.maximum {
            schema["maximum"] = serde_json::json!(max);
        }

        if let Some(items) = &self.items {
            schema["items"] = items.to_json_schema();
        }

        schema
    }
}

/// Parameter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

impl ParameterType {
    /// Get the JSON Schema type string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Object => "object",
        }
    }
}

/// Tool metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Danger level for approval flow.
    #[serde(default)]
    pub danger_level: DangerLevel,
    /// Category for grouping.
    #[serde(default)]
    pub category: Option<String>,
    /// Additional metadata.
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// Tool danger level for approval flow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DangerLevel {
    /// Safe operations, no approval needed.
    #[default]
    Safe,
    /// Low risk, auto-approve unless paranoid mode.
    Low,
    /// Medium risk, prompt for approval.
    Medium,
    /// High risk, always require explicit approval.
    High,
    /// Critical operations, require confirmation twice.
    Critical,
}

/// Tool validation error.
#[derive(Debug, thiserror::Error)]
pub enum ToolValidationError {
    #[error("tool name cannot be empty")]
    EmptyName,
    #[error("invalid tool name: {0}")]
    InvalidName(String),
    #[error("tool description cannot be empty")]
    EmptyDescription,
    #[error("required parameter not found in properties: {0}")]
    MissingRequiredParam(String),
}
```

### 2. Predefined Tools (src/tool/predefined.rs)

```rust
//! Predefined tool definitions for common operations.

use super::{DangerLevel, ToolDefinition, ToolParameter};

/// Create the file read tool definition.
pub fn file_read_tool() -> ToolDefinition {
    ToolDefinition::builder("read_file")
        .description("Read the contents of a file at the specified path")
        .required_string("path", "The absolute path to the file to read")
        .optional_int("max_lines", "Maximum number of lines to read")
        .optional_int("start_line", "Line number to start reading from (1-indexed)")
        .danger_level(DangerLevel::Safe)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the file write tool definition.
pub fn file_write_tool() -> ToolDefinition {
    ToolDefinition::builder("write_file")
        .description("Write content to a file, creating it if it doesn't exist")
        .required_string("path", "The absolute path to the file to write")
        .required_string("content", "The content to write to the file")
        .optional_bool("create_dirs", "Create parent directories if they don't exist")
        .danger_level(DangerLevel::Medium)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the shell command tool definition.
pub fn shell_command_tool() -> ToolDefinition {
    ToolDefinition::builder("run_command")
        .description("Execute a shell command and return its output")
        .required_string("command", "The command to execute")
        .optional_string("working_dir", "Working directory for the command")
        .optional_int("timeout_secs", "Timeout in seconds (default: 30)")
        .danger_level(DangerLevel::High)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the web search tool definition.
pub fn web_search_tool() -> ToolDefinition {
    ToolDefinition::builder("web_search")
        .description("Search the web for information")
        .required_string("query", "The search query")
        .optional_int("max_results", "Maximum number of results to return")
        .danger_level(DangerLevel::Safe)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the code edit tool definition.
pub fn code_edit_tool() -> ToolDefinition {
    ToolDefinition::builder("edit_code")
        .description("Edit code in a file by specifying the old and new content")
        .required_string("path", "The path to the file to edit")
        .required_string("old_content", "The exact content to replace")
        .required_string("new_content", "The new content to insert")
        .danger_level(DangerLevel::Medium)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the directory list tool definition.
pub fn list_directory_tool() -> ToolDefinition {
    ToolDefinition::builder("list_directory")
        .description("List the contents of a directory")
        .required_string("path", "The path to the directory")
        .optional_bool("recursive", "List contents recursively")
        .optional_bool("include_hidden", "Include hidden files")
        .danger_level(DangerLevel::Safe)
        .build()
        .expect("predefined tool should be valid")
}

/// Create the grep search tool definition.
pub fn grep_tool() -> ToolDefinition {
    ToolDefinition::builder("grep_search")
        .description("Search for a pattern in files")
        .required_string("pattern", "The regex pattern to search for")
        .required_string("path", "The path to search in")
        .optional_bool("case_insensitive", "Perform case-insensitive search")
        .optional_string("file_pattern", "Glob pattern to filter files (e.g., '*.rs')")
        .danger_level(DangerLevel::Safe)
        .build()
        .expect("predefined tool should be valid")
}

/// Get all predefined tools.
pub fn all_predefined_tools() -> Vec<ToolDefinition> {
    vec![
        file_read_tool(),
        file_write_tool(),
        shell_command_tool(),
        web_search_tool(),
        code_edit_tool(),
        list_directory_tool(),
        grep_tool(),
    ]
}
```

### 3. Tool Registry (src/tool/registry.rs)

```rust
//! Tool registry for managing available tools.

use super::{ToolDefinition, ToolValidationError};
use std::collections::HashMap;

/// Registry of available tools.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with predefined tools.
    pub fn with_predefined() -> Self {
        let mut registry = Self::new();
        for tool in super::predefined::all_predefined_tools() {
            registry.register(tool).expect("predefined tools should be valid");
        }
        registry
    }

    /// Register a tool.
    pub fn register(&mut self, tool: ToolDefinition) -> Result<(), ToolValidationError> {
        tool.validate()?;
        self.tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// Unregister a tool.
    pub fn unregister(&mut self, name: &str) -> Option<ToolDefinition> {
        self.tools.remove(name)
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Get all enabled tools.
    pub fn enabled_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().filter(|t| t.enabled).collect()
    }

    /// Get all tools.
    pub fn all_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Enable a tool.
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(tool) = self.tools.get_mut(name) {
            tool.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a tool.
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(tool) = self.tools.get_mut(name) {
            tool.enabled = false;
            true
        } else {
            false
        }
    }

    /// Check if a tool exists.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
```

---

## Testing Requirements

1. Tool builder creates valid definitions
2. JSON Schema output is correct
3. Provider-specific formats are accurate
4. Validation catches invalid tools
5. Registry operations work correctly

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Next: [055-tool-call-types.md](055-tool-call-types.md)
- Used by: [059-claude-tools.md](059-claude-tools.md), [063-codex-tools.md](063-codex-tools.md)
