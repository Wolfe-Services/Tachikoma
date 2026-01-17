//! Custom proptest strategies for Tachikoma domain types.

use proptest::prelude::*;

/// Strategy for generating valid file paths
pub fn valid_file_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z][a-z0-9_]{0,15}", 1..5)
        .prop_map(|parts| parts.join("/"))
}

/// Strategy for generating valid identifiers
pub fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,31}".prop_map(|s| s.to_string())
}

/// Strategy for generating valid YAML content
pub fn valid_yaml_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (valid_identifier(), any::<i64>()),
        1..10,
    )
    .prop_map(|pairs| {
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Strategy for generating valid JSON objects
pub fn valid_json_object() -> impl Strategy<Value = serde_json::Value> {
    prop::collection::btree_map(
        valid_identifier(),
        prop_oneof![
            any::<bool>().prop_map(serde_json::Value::Bool),
            any::<i64>().prop_map(|n| serde_json::Value::Number(n.into())),
            "[a-z ]{0,50}".prop_map(serde_json::Value::String),
        ],
        0..10,
    )
    .prop_map(|map| serde_json::Value::Object(map.into_iter().collect()))
}

/// Strategy for generating backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub name: String,
    pub api_key: String,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

pub fn backend_config_strategy() -> impl Strategy<Value = BackendConfig> {
    (
        valid_identifier(),
        "[a-zA-Z0-9]{32,64}",
        0u32..10,
        100u64..30_000,
    )
        .prop_map(|(name, api_key, max_retries, timeout_ms)| BackendConfig {
            name,
            api_key,
            max_retries,
            timeout_ms,
        })
}

/// Strategy for generating tool definitions
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
}

pub fn tool_definition_strategy() -> impl Strategy<Value = ToolDefinition> {
    (
        valid_identifier(),
        "[A-Za-z ]{10,100}",
        prop::collection::vec(valid_identifier(), 0..5),
    )
        .prop_map(|(name, description, parameters)| ToolDefinition {
            name,
            description,
            parameters,
        })
}

/// Arbitrary implementation for standard types
pub mod arbitrary {
    use super::*;

    impl Arbitrary for BackendConfig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            backend_config_strategy().boxed()
        }
    }

    impl Arbitrary for ToolDefinition {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            tool_definition_strategy().boxed()
        }
    }
}