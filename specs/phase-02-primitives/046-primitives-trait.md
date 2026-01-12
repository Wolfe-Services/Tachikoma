# 046 - Primitives Trait Abstraction

**Phase:** 2 - Five Primitives
**Spec ID:** 046
**Status:** Planned
**Dependencies:** 031-primitives-crate, 032-read-file-impl, 034-list-files-impl, 036-bash-exec-core, 040-edit-file-core, 043-code-search-core
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define a common trait abstraction for all primitives enabling uniform handling, tool registration, and dynamic dispatch.

---

## Acceptance Criteria

- [x] Common Primitive trait definition
- [x] Async execution support
- [x] Tool schema generation for MCP/API
- [x] Dynamic dispatch capability
- [x] Primitive registry for discovery
- [x] Serializable input/output types

---

## Implementation Details

### 1. Primitive Trait (src/traits.rs)

```rust
//! Common trait for all primitives.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{
    context::PrimitiveContext,
    error::PrimitiveResult,
};

/// Common trait for all primitives.
#[async_trait]
pub trait Primitive: Send + Sync {
    /// The input type for this primitive.
    type Input: DeserializeOwned + Send;
    /// The output type for this primitive.
    type Output: Serialize + Send;

    /// Get the primitive name.
    fn name(&self) -> &'static str;

    /// Get the primitive description.
    fn description(&self) -> &'static str;

    /// Execute the primitive.
    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output>;

    /// Get the JSON schema for the input type.
    fn input_schema(&self) -> Value {
        serde_json::json!({})
    }

    /// Get the JSON schema for the output type.
    fn output_schema(&self) -> Value {
        serde_json::json!({})
    }
}

/// A boxed primitive for dynamic dispatch.
#[async_trait]
pub trait DynPrimitive: Send + Sync {
    /// Get the primitive name.
    fn name(&self) -> &'static str;

    /// Get the primitive description.
    fn description(&self) -> &'static str;

    /// Execute with JSON input and output.
    async fn execute_json(
        &self,
        ctx: &PrimitiveContext,
        input: Value,
    ) -> PrimitiveResult<Value>;

    /// Get the input schema.
    fn input_schema(&self) -> Value;

    /// Get the output schema.
    fn output_schema(&self) -> Value;

    /// Get MCP tool definition.
    fn mcp_tool_definition(&self) -> McpToolDefinition;
}

/// Wrapper to make any Primitive into a DynPrimitive.
pub struct PrimitiveWrapper<P> {
    inner: P,
}

impl<P> PrimitiveWrapper<P> {
    pub fn new(primitive: P) -> Self {
        Self { inner: primitive }
    }
}

#[async_trait]
impl<P> DynPrimitive for PrimitiveWrapper<P>
where
    P: Primitive,
    P::Input: 'static,
    P::Output: 'static,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn description(&self) -> &'static str {
        self.inner.description()
    }

    async fn execute_json(
        &self,
        ctx: &PrimitiveContext,
        input: Value,
    ) -> PrimitiveResult<Value> {
        let typed_input: P::Input = serde_json::from_value(input)
            .map_err(|e| crate::error::PrimitiveError::Validation {
                message: format!("Invalid input: {}", e),
            })?;

        let output = self.inner.execute(ctx, typed_input).await?;

        serde_json::to_value(output)
            .map_err(|e| crate::error::PrimitiveError::Validation {
                message: format!("Failed to serialize output: {}", e),
            })
    }

    fn input_schema(&self) -> Value {
        self.inner.input_schema()
    }

    fn output_schema(&self) -> Value {
        self.inner.output_schema()
    }

    fn mcp_tool_definition(&self) -> McpToolDefinition {
        McpToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }
}

/// MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON Schema for input parameters.
    pub input_schema: Value,
}

/// Registry for primitives.
pub struct PrimitiveRegistry {
    primitives: HashMap<String, Box<dyn DynPrimitive>>,
}

impl PrimitiveRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            primitives: HashMap::new(),
        }
    }

    /// Create registry with all default primitives.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register a primitive.
    pub fn register<P>(&mut self, primitive: P)
    where
        P: Primitive + 'static,
        P::Input: 'static,
        P::Output: 'static,
    {
        let name = primitive.name().to_string();
        let wrapper = PrimitiveWrapper::new(primitive);
        self.primitives.insert(name, Box::new(wrapper));
    }

    /// Register default primitives.
    pub fn register_defaults(&mut self) {
        self.register(ReadFilePrimitive);
        self.register(ListFilesPrimitive);
        self.register(BashPrimitive);
        self.register(EditFilePrimitive);
        self.register(CodeSearchPrimitive);
    }

    /// Get a primitive by name.
    pub fn get(&self, name: &str) -> Option<&dyn DynPrimitive> {
        self.primitives.get(name).map(|p| p.as_ref())
    }

    /// Execute a primitive by name.
    pub async fn execute(
        &self,
        name: &str,
        ctx: &PrimitiveContext,
        input: Value,
    ) -> PrimitiveResult<Value> {
        let primitive = self.get(name).ok_or_else(|| {
            crate::error::PrimitiveError::Validation {
                message: format!("Unknown primitive: {}", name),
            }
        })?;

        primitive.execute_json(ctx, input).await
    }

    /// Get all MCP tool definitions.
    pub fn mcp_tools(&self) -> Vec<McpToolDefinition> {
        self.primitives
            .values()
            .map(|p| p.mcp_tool_definition())
            .collect()
    }

    /// List all primitive names.
    pub fn names(&self) -> Vec<&str> {
        self.primitives.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PrimitiveRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// Concrete primitive implementations

/// Read file primitive.
pub struct ReadFilePrimitive;

#[derive(Debug, Deserialize)]
pub struct ReadFileInput {
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

#[async_trait]
impl Primitive for ReadFilePrimitive {
    type Input = ReadFileInput;
    type Output = crate::result::ReadFileResult;

    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read the contents of a file"
    }

    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output> {
        let options = crate::read_file::ReadFileOptions {
            start_line: input.start_line,
            end_line: input.end_line,
            ..Default::default()
        };
        crate::read_file::read_file(ctx, &input.path, Some(options)).await
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Starting line number (1-indexed)"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Ending line number (1-indexed, inclusive)"
                }
            },
            "required": ["path"]
        })
    }
}

/// List files primitive.
pub struct ListFilesPrimitive;

#[derive(Debug, Deserialize)]
pub struct ListFilesInput {
    pub path: String,
    pub extension: Option<String>,
    pub recursive: Option<bool>,
}

#[async_trait]
impl Primitive for ListFilesPrimitive {
    type Input = ListFilesInput;
    type Output = crate::result::ListFilesResult;

    fn name(&self) -> &'static str {
        "list_files"
    }

    fn description(&self) -> &'static str {
        "List files in a directory"
    }

    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output> {
        let mut options = crate::list_files::ListFilesOptions::new();
        if let Some(ext) = input.extension {
            options = options.extension(&ext);
        }
        crate::list_files::list_files(ctx, &input.path, Some(options)).await
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list"
                },
                "extension": {
                    "type": "string",
                    "description": "Filter by file extension"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List files recursively"
                }
            },
            "required": ["path"]
        })
    }
}

/// Bash primitive.
pub struct BashPrimitive;

#[derive(Debug, Deserialize)]
pub struct BashInput {
    pub command: String,
    pub working_dir: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[async_trait]
impl Primitive for BashPrimitive {
    type Input = BashInput;
    type Output = crate::result::BashResult;

    fn name(&self) -> &'static str {
        "bash"
    }

    fn description(&self) -> &'static str {
        "Execute a bash command"
    }

    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output> {
        let mut options = crate::bash::BashOptions::new();
        if let Some(dir) = input.working_dir {
            options = options.working_dir(&dir);
        }
        if let Some(ms) = input.timeout_ms {
            options = options.timeout(std::time::Duration::from_millis(ms));
        }
        crate::bash::bash(ctx, &input.command, Some(options)).await
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory for the command"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds"
                }
            },
            "required": ["command"]
        })
    }
}

/// Edit file primitive.
pub struct EditFilePrimitive;

#[derive(Debug, Deserialize)]
pub struct EditFileInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[async_trait]
impl Primitive for EditFilePrimitive {
    type Input = EditFileInput;
    type Output = crate::result::EditFileResult;

    fn name(&self) -> &'static str {
        "edit_file"
    }

    fn description(&self) -> &'static str {
        "Edit a file by replacing text"
    }

    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output> {
        let mut options = crate::edit_file::EditFileOptions::new();
        if input.replace_all.unwrap_or(false) {
            options = options.replace_all();
        }
        crate::edit_file::edit_file(
            ctx,
            &input.path,
            &input.old_string,
            &input.new_string,
            Some(options),
        ).await
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "String to search for"
                },
                "new_string": {
                    "type": "string",
                    "description": "String to replace with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }
}

/// Code search primitive.
pub struct CodeSearchPrimitive;

#[derive(Debug, Deserialize)]
pub struct CodeSearchInput {
    pub pattern: String,
    pub path: String,
    pub file_type: Option<String>,
    pub context_lines: Option<usize>,
}

#[async_trait]
impl Primitive for CodeSearchPrimitive {
    type Input = CodeSearchInput;
    type Output = crate::result::CodeSearchResult;

    fn name(&self) -> &'static str {
        "code_search"
    }

    fn description(&self) -> &'static str {
        "Search code using regex patterns"
    }

    async fn execute(
        &self,
        ctx: &PrimitiveContext,
        input: Self::Input,
    ) -> PrimitiveResult<Self::Output> {
        let mut options = crate::code_search::CodeSearchOptions::new();
        if let Some(ft) = input.file_type {
            options = options.file_type(&ft);
        }
        if let Some(ctx_lines) = input.context_lines {
            options = options.context(ctx_lines);
        }
        crate::code_search::code_search(ctx, &input.pattern, &input.path, Some(options)).await
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory or file to search"
                },
                "file_type": {
                    "type": "string",
                    "description": "Filter by file type (e.g., 'rust', 'python')"
                },
                "context_lines": {
                    "type": "integer",
                    "description": "Number of context lines"
                }
            },
            "required": ["pattern", "path"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = PrimitiveRegistry::with_defaults();
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("bash").is_some());
        assert!(registry.get("edit_file").is_some());
    }

    #[test]
    fn test_mcp_tools() {
        let registry = PrimitiveRegistry::with_defaults();
        let tools = registry.mcp_tools();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_primitive_names() {
        let registry = PrimitiveRegistry::with_defaults();
        let names = registry.names();
        assert!(names.contains(&"read_file"));
        assert!(names.contains(&"list_files"));
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"edit_file"));
        assert!(names.contains(&"code_search"));
    }
}
```

---

## Testing Requirements

1. All primitives implement the trait correctly
2. Registry discovers all default primitives
3. Dynamic dispatch works for all types
4. JSON serialization/deserialization works
5. MCP tool definitions are valid
6. Input schemas are correct
7. Error handling works through trait

---

## Related Specs

- Depends on: All primitive implementation specs
- Next: [047-primitives-validation.md](047-primitives-validation.md)
- Used by: Agent loop, MCP server
