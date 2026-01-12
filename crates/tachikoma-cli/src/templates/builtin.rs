//! Built-in project templates.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::templates::{
    Template, TemplateContext, TemplateFile, TemplateManifest, TemplateSource, 
    manifest::{TemplateMetadata, TemplateVariable, VariableType, FileConfig, HooksConfig}
};

/// Built-in templates registry
pub struct BuiltinTemplates;

impl BuiltinTemplates {
    /// Get a built-in template by name
    pub fn get(name: &str) -> Option<Template> {
        match name {
            "basic" => Some(Self::basic_template()),
            "tools" => Some(Self::tools_template()),
            "workflow" => Some(Self::workflow_template()),
            "chat" => Some(Self::chat_template()),
            "minimal" => Some(Self::minimal_template()),
            _ => None,
        }
    }

    /// List all available built-in templates
    pub fn list() -> Vec<crate::templates::loader::TemplateInfo> {
        vec![
            crate::templates::loader::TemplateInfo {
                name: "basic".to_string(),
                description: "Basic Tachikoma project with common files".to_string(),
                source: "builtin".to_string(),
            },
            crate::templates::loader::TemplateInfo {
                name: "tools".to_string(),
                description: "Tachikoma project with tool development setup".to_string(),
                source: "builtin".to_string(),
            },
            crate::templates::loader::TemplateInfo {
                name: "workflow".to_string(),
                description: "Tachikoma project with workflow automation".to_string(),
                source: "builtin".to_string(),
            },
            crate::templates::loader::TemplateInfo {
                name: "chat".to_string(),
                description: "Tachikoma project for chat applications".to_string(),
                source: "builtin".to_string(),
            },
            crate::templates::loader::TemplateInfo {
                name: "minimal".to_string(),
                description: "Minimal Tachikoma project setup".to_string(),
                source: "builtin".to_string(),
            },
        ]
    }

    /// Basic template with common project structure
    fn basic_template() -> Template {
        let manifest = TemplateManifest {
            template: TemplateMetadata {
                name: "basic".to_string(),
                description: "Basic Tachikoma project with common files".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Tachikoma Contributors".to_string()],
                tags: vec!["basic".to_string()],
                min_tachikoma_version: Some("0.1.0".to_string()),
            },
            variables: vec![
                TemplateVariable {
                    name: "description".to_string(),
                    description: "Project description".to_string(),
                    default: Some("A Tachikoma AI agent project".to_string()),
                    required: false,
                    var_type: VariableType::String,
                    prompt: Some("Enter a description for your project".to_string()),
                    choices: vec![],
                    pattern: None,
                },
                TemplateVariable {
                    name: "author".to_string(),
                    description: "Project author".to_string(),
                    default: None,
                    required: false,
                    var_type: VariableType::String,
                    prompt: Some("Enter the author name".to_string()),
                    choices: vec![],
                    pattern: None,
                },
            ],
            files: FileConfig::default(),
            hooks: HooksConfig::default(),
        };

        let files = vec![
            TemplateFile {
                path: PathBuf::from("README.md"),
                content: include_str!("builtin/basic/README.md").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("tachikoma.toml"),
                content: include_str!("builtin/basic/tachikoma.toml").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from(".gitignore"),
                content: include_str!("builtin/basic/.gitignore").to_string(),
                process: false,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("src/main.py"),
                content: include_str!("builtin/basic/src/main.py").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("requirements.txt"),
                content: include_str!("builtin/basic/requirements.txt").to_string(),
                process: false,
                executable: false,
            },
        ];

        Template {
            name: "basic".to_string(),
            description: "Basic Tachikoma project with common files".to_string(),
            source: TemplateSource::Builtin("basic".to_string()),
            manifest,
            files,
        }
    }

    /// Tools template with tool development setup
    fn tools_template() -> Template {
        let manifest = TemplateManifest {
            template: TemplateMetadata {
                name: "tools".to_string(),
                description: "Tachikoma project with tool development setup".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Tachikoma Contributors".to_string()],
                tags: vec!["tools", "development".to_string()],
                min_tachikoma_version: Some("0.1.0".to_string()),
            },
            variables: vec![
                TemplateVariable {
                    name: "description".to_string(),
                    description: "Project description".to_string(),
                    default: Some("A Tachikoma project for tool development".to_string()),
                    required: false,
                    var_type: VariableType::String,
                    prompt: Some("Enter a description for your project".to_string()),
                    choices: vec![],
                    pattern: None,
                },
                TemplateVariable {
                    name: "include_examples".to_string(),
                    description: "Include example tools".to_string(),
                    default: Some("true".to_string()),
                    required: false,
                    var_type: VariableType::Boolean,
                    prompt: Some("Include example tools? (y/n)".to_string()),
                    choices: vec![],
                    pattern: None,
                },
            ],
            files: FileConfig {
                conditional: vec![
                    crate::templates::manifest::ConditionalFile {
                        path: "examples/**/*".to_string(),
                        condition: "include_examples == \"true\"".to_string(),
                    },
                ],
                ..Default::default()
            },
            hooks: HooksConfig::default(),
        };

        let files = vec![
            TemplateFile {
                path: PathBuf::from("README.md"),
                content: include_str!("builtin/tools/README.md").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("tachikoma.toml"),
                content: include_str!("builtin/tools/tachikoma.toml").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("tools/calculator.py"),
                content: include_str!("builtin/tools/tools/calculator.py").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("tools/__init__.py"),
                content: "".to_string(),
                process: false,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("examples/tool_usage.py"),
                content: include_str!("builtin/tools/examples/tool_usage.py").to_string(),
                process: true,
                executable: false,
            },
        ];

        Template {
            name: "tools".to_string(),
            description: "Tachikoma project with tool development setup".to_string(),
            source: TemplateSource::Builtin("tools".to_string()),
            manifest,
            files,
        }
    }

    /// Workflow template for automation
    fn workflow_template() -> Template {
        let manifest = TemplateManifest {
            template: TemplateMetadata {
                name: "workflow".to_string(),
                description: "Tachikoma project with workflow automation".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Tachikoma Contributors".to_string()],
                tags: vec!["workflow", "automation".to_string()],
                min_tachikoma_version: Some("0.1.0".to_string()),
            },
            variables: vec![
                TemplateVariable {
                    name: "workflow_type".to_string(),
                    description: "Type of workflow".to_string(),
                    default: Some("basic".to_string()),
                    required: true,
                    var_type: VariableType::Select,
                    prompt: Some("Select workflow type".to_string()),
                    choices: vec!["basic".to_string(), "data".to_string(), "api".to_string()],
                    pattern: None,
                },
            ],
            files: FileConfig::default(),
            hooks: HooksConfig::default(),
        };

        let files = vec![
            TemplateFile {
                path: PathBuf::from("README.md"),
                content: include_str!("builtin/workflow/README.md").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("workflows/main.yaml"),
                content: include_str!("builtin/workflow/workflows/main.yaml").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("src/workflow.py"),
                content: include_str!("builtin/workflow/src/workflow.py").to_string(),
                process: true,
                executable: false,
            },
        ];

        Template {
            name: "workflow".to_string(),
            description: "Tachikoma project with workflow automation".to_string(),
            source: TemplateSource::Builtin("workflow".to_string()),
            manifest,
            files,
        }
    }

    /// Chat template for conversational applications
    fn chat_template() -> Template {
        let manifest = TemplateManifest {
            template: TemplateMetadata {
                name: "chat".to_string(),
                description: "Tachikoma project for chat applications".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Tachikoma Contributors".to_string()],
                tags: vec!["chat", "conversation".to_string()],
                min_tachikoma_version: Some("0.1.0".to_string()),
            },
            variables: vec![
                TemplateVariable {
                    name: "agent_name".to_string(),
                    description: "Name of the chat agent".to_string(),
                    default: Some("Assistant".to_string()),
                    required: false,
                    var_type: VariableType::String,
                    prompt: Some("Enter the agent name".to_string()),
                    choices: vec![],
                    pattern: None,
                },
            ],
            files: FileConfig::default(),
            hooks: HooksConfig::default(),
        };

        let files = vec![
            TemplateFile {
                path: PathBuf::from("README.md"),
                content: include_str!("builtin/chat/README.md").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("src/chat.py"),
                content: include_str!("builtin/chat/src/chat.py").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("config/agent.yaml"),
                content: include_str!("builtin/chat/config/agent.yaml").to_string(),
                process: true,
                executable: false,
            },
        ];

        Template {
            name: "chat".to_string(),
            description: "Tachikoma project for chat applications".to_string(),
            source: TemplateSource::Builtin("chat".to_string()),
            manifest,
            files,
        }
    }

    /// Minimal template with just the essentials
    fn minimal_template() -> Template {
        let manifest = TemplateManifest {
            template: TemplateMetadata {
                name: "minimal".to_string(),
                description: "Minimal Tachikoma project setup".to_string(),
                version: "1.0.0".to_string(),
                authors: vec!["Tachikoma Contributors".to_string()],
                tags: vec!["minimal".to_string()],
                min_tachikoma_version: Some("0.1.0".to_string()),
            },
            variables: vec![],
            files: FileConfig::default(),
            hooks: HooksConfig::default(),
        };

        let files = vec![
            TemplateFile {
                path: PathBuf::from("tachikoma.toml"),
                content: include_str!("builtin/minimal/tachikoma.toml").to_string(),
                process: true,
                executable: false,
            },
            TemplateFile {
                path: PathBuf::from("README.md"),
                content: include_str!("builtin/minimal/README.md").to_string(),
                process: true,
                executable: false,
            },
        ];

        Template {
            name: "minimal".to_string(),
            description: "Minimal Tachikoma project setup".to_string(),
            source: TemplateSource::Builtin("minimal".to_string()),
            manifest,
            files,
        }
    }
}