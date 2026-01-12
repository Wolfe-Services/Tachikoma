//! Common trait for all primitives.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
        #[cfg(feature = "read-file")]
        self.register(ReadFilePrimitive);
        
        #[cfg(feature = "list-files")]
        self.register(ListFilesPrimitive);
        
        #[cfg(feature = "bash")]
        self.register(BashPrimitive);
        
        #[cfg(feature = "edit-file")]
        self.register(EditFilePrimitive);
        
        #[cfg(feature = "code-search")]
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