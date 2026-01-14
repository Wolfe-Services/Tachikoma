//! Plugin manifest types
//!
//! Defines the structure for plugin.yaml manifest files.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Type of plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    /// LLM agent backend
    Agent,
    /// Task/spec tracker
    Tracker,
    /// Prompt templates
    Template,
}

/// Plugin manifest (plugin.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (unique identifier)
    pub name: String,
    
    /// Plugin version (semver)
    pub version: String,
    
    /// Plugin type
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    
    /// Human-readable description
    pub description: String,
    
    /// Plugin author
    #[serde(default)]
    pub author: Option<String>,
    
    /// Plugin homepage/repository
    #[serde(default)]
    pub homepage: Option<String>,
    
    /// Required dependencies and environment
    #[serde(default)]
    pub requires: Vec<PluginRequirement>,
    
    /// Configuration schema
    #[serde(default)]
    pub config: HashMap<String, ConfigField>,
    
    /// Entry point for external plugins
    #[serde(default)]
    pub adapter: Option<String>,
    
    /// Whether this is a built-in plugin
    #[serde(default)]
    pub builtin: bool,
}

impl PluginManifest {
    /// Check if all requirements are met
    pub fn check_requirements(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for req in &self.requires {
            if !req.check() {
                errors.push(format!("{}: {}", req.requirement_type(), req.name));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// A requirement for the plugin to function
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PluginRequirement {
    /// Requires a binary in PATH
    Binary {
        name: String,
        #[serde(default)]
        version: Option<String>,
    },
    /// Requires an environment variable
    Env {
        name: String,
    },
    /// Requires another plugin
    Plugin {
        name: String,
        #[serde(default)]
        version: Option<String>,
    },
}

impl PluginRequirement {
    /// Check if the requirement is met
    pub fn check(&self) -> bool {
        match self {
            PluginRequirement::Binary { name, version: _ } => {
                // Check if binary exists in PATH
                which::which(name).is_ok()
            }
            PluginRequirement::Env { name } => {
                std::env::var(name).is_ok()
            }
            PluginRequirement::Plugin { name: _, version: _ } => {
                // TODO: Check if plugin is available
                true
            }
        }
    }
    
    /// Get the requirement type as a string
    pub fn requirement_type(&self) -> &str {
        match self {
            PluginRequirement::Binary { .. } => "binary",
            PluginRequirement::Env { .. } => "env",
            PluginRequirement::Plugin { .. } => "plugin",
        }
    }
    
    /// Get the requirement name
    pub fn name(&self) -> &str {
        match self {
            PluginRequirement::Binary { name, .. } => name,
            PluginRequirement::Env { name } => name,
            PluginRequirement::Plugin { name, .. } => name,
        }
    }
}

/// Configuration field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    /// Field type
    #[serde(rename = "type")]
    pub field_type: ConfigFieldType,
    
    /// Default value
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    
    /// Field description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
}

/// Types for configuration fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_parse() {
        let yaml = r#"
name: claude
version: "1.0.0"
type: agent
description: Claude AI agent integration
requires:
  - type: env
    name: ANTHROPIC_API_KEY
config:
  model:
    type: string
    default: "claude-sonnet-4-20250514"
  max_tokens:
    type: integer
    default: 8192
"#;
        
        let manifest: PluginManifest = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(manifest.name, "claude");
        assert_eq!(manifest.plugin_type, PluginType::Agent);
        assert_eq!(manifest.requires.len(), 1);
        assert!(matches!(manifest.requires[0], PluginRequirement::Env { .. }));
    }
    
    #[test]
    fn test_env_requirement_check() {
        std::env::set_var("TEST_PLUGIN_VAR", "value");
        
        let req = PluginRequirement::Env {
            name: "TEST_PLUGIN_VAR".to_string(),
        };
        
        assert!(req.check());
        
        let req2 = PluginRequirement::Env {
            name: "NONEXISTENT_VAR_12345".to_string(),
        };
        
        assert!(!req2.check());
        
        std::env::remove_var("TEST_PLUGIN_VAR");
    }
}
