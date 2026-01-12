//! Command examples.

use std::collections::HashMap;

use crate::output::color::{ColorMode, Styled, Color};

/// Command examples collection
pub struct CommandExamples {
    examples: HashMap<&'static str, Vec<Example>>,
}

/// A single example
pub struct Example {
    pub description: &'static str,
    pub command: &'static str,
    pub output: Option<&'static str>,
}

impl CommandExamples {
    pub fn new() -> Self {
        let mut examples = HashMap::new();

        // Init command examples
        examples.insert("init", vec![
            Example {
                description: "Create a new project in the current directory",
                command: "tachikoma init my-project",
                output: None,
            },
            Example {
                description: "Create a project with tools template",
                command: "tachikoma init my-project --template tools",
                output: None,
            },
            Example {
                description: "Create project interactively",
                command: "tachikoma init",
                output: None,
            },
        ]);

        // Doctor command examples
        examples.insert("doctor", vec![
            Example {
                description: "Run all health checks",
                command: "tachikoma doctor",
                output: Some("System check: OK\nConfig check: OK"),
            },
            Example {
                description: "Run specific category checks",
                command: "tachikoma doctor --category backends",
                output: None,
            },
            Example {
                description: "Output as JSON",
                command: "tachikoma --format json doctor",
                output: Some("{\"checks\": [...]}"),
            },
        ]);

        // Config command examples
        examples.insert("config", vec![
            Example {
                description: "List all configuration",
                command: "tachikoma config list",
                output: None,
            },
            Example {
                description: "Get a specific value",
                command: "tachikoma config get backend.default",
                output: Some("anthropic"),
            },
            Example {
                description: "Set a configuration value",
                command: "tachikoma config set agent.temperature 0.8",
                output: None,
            },
        ]);

        // Tools command examples
        examples.insert("tools", vec![
            Example {
                description: "List installed tools",
                command: "tachikoma tools list",
                output: None,
            },
            Example {
                description: "Install a tool",
                command: "tachikoma tools install filesystem",
                output: None,
            },
            Example {
                description: "Search for tools",
                command: "tachikoma tools search database",
                output: None,
            },
            Example {
                description: "Test a tool",
                command: "tachikoma tools test filesystem --input '{\"path\": \".\"}'",
                output: None,
            },
        ]);

        // Backends command examples
        examples.insert("backends", vec![
            Example {
                description: "List configured backends",
                command: "tachikoma backends list",
                output: None,
            },
            Example {
                description: "Add Anthropic backend",
                command: "tachikoma backends add anthropic --backend-type anthropic",
                output: None,
            },
            Example {
                description: "Test backend connectivity",
                command: "tachikoma backends test anthropic",
                output: None,
            },
            Example {
                description: "List available models",
                command: "tachikoma backends models --refresh",
                output: None,
            },
        ]);

        Self { examples }
    }

    /// Get examples for a command
    pub fn get(&self, command: &str) -> Option<&Vec<Example>> {
        self.examples.get(command)
    }

    /// Format examples for display
    pub fn format(&self, command: &str, color_mode: ColorMode) -> Option<String> {
        let examples = self.get(command)?;

        let mut output = String::new();
        
        let header = Styled::new("Examples:")
            .with_color_mode(color_mode)
            .fg(Color::Yellow)
            .bold();
        output.push_str(&format!("{header}\n"));

        for example in examples {
            output.push_str(&format!("\n  # {}\n", example.description));

            let command_styled = Styled::new(format!("$ {}", example.command))
                .with_color_mode(color_mode)
                .fg(Color::Cyan);
            output.push_str(&format!("  {command_styled}\n"));

            if let Some(expected_output) = example.output {
                for line in expected_output.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
            }
        }

        Some(output)
    }

    /// Add an example for a command
    pub fn add_example(&mut self, command: &'static str, example: Example) {
        self.examples.entry(command).or_default().push(example);
    }
}

impl Default for CommandExamples {
    fn default() -> Self {
        Self::new()
    }
}