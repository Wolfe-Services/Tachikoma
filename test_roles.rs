// Test roles module in isolation

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    pub name: String,
    pub codename: String,
    pub description: String,
    pub responsibilities: Vec<String>,
    pub constraints: Vec<String>,
    pub output_guidelines: Vec<String>,
    pub thinking_style: ThinkingStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinkingStyle {
    Analytical,
    Creative,
    Critical,
    Pragmatic,
    Systematic,
}

impl AgentRole {
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::new();
        
        prompt.push_str(&format!("# Agent Role: {}\n\n", self.name));
        prompt.push_str(&format!("{}\n\n", self.description));
        
        prompt.push_str("## Your Responsibilities\n\n");
        for resp in &self.responsibilities {
            prompt.push_str(&format!("- {}\n", resp));
        }
        prompt.push('\n');
        
        prompt.push_str("## Constraints\n\n");
        for constraint in &self.constraints {
            prompt.push_str(&format!("- {}\n", constraint));
        }
        prompt.push('\n');
        
        prompt.push_str("## Output Guidelines\n\n");
        for guideline in &self.output_guidelines {
            prompt.push_str(&format!("- {}\n", guideline));
        }
        prompt.push('\n');
        
        prompt.push_str(&format!(
            "## Thinking Style: {:?}\n\n\
            Apply this thinking style consistently in your responses.\n",
            self.thinking_style
        ));
        
        prompt
    }
    
    pub fn builder(name: impl Into<String>) -> AgentRoleBuilder {
        AgentRoleBuilder::new(name)
    }
    
    // Presets
    pub fn architect() -> Self {
        Self::builder("Systems Architect")
            .codename("architect")
            .description(
                "You are a senior systems architect with expertise in designing \
                scalable, maintainable software systems. You think in terms of \
                components, interfaces, and data flows."
            )
            .responsibility("Design the high-level structure of solutions")
            .responsibility("Define component boundaries and interfaces")
            .responsibility("Ensure the design is extensible and maintainable")
            .responsibility("Consider performance and scalability implications")
            .constraint("Do not get lost in implementation details")
            .constraint("Always consider the full system context")
            .constraint("Prefer simplicity over cleverness")
            .output_guideline("Start with a brief overview of your approach")
            .output_guideline("Use diagrams (ASCII or markdown) when helpful")
            .output_guideline("Explicitly state trade-offs you're making")
            .thinking_style(ThinkingStyle::Systematic)
            .build()
    }
}

pub struct AgentRoleBuilder {
    name: String,
    codename: String,
    description: String,
    responsibilities: Vec<String>,
    constraints: Vec<String>,
    output_guidelines: Vec<String>,
    thinking_style: ThinkingStyle,
}

impl AgentRoleBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let codename = name.to_lowercase().replace(' ', "_");
        Self {
            name,
            codename,
            description: String::new(),
            responsibilities: Vec::new(),
            constraints: Vec::new(),
            output_guidelines: Vec::new(),
            thinking_style: ThinkingStyle::Analytical,
        }
    }
    
    pub fn codename(mut self, codename: impl Into<String>) -> Self {
        self.codename = codename.into();
        self
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    pub fn responsibility(mut self, resp: impl Into<String>) -> Self {
        self.responsibilities.push(resp.into());
        self
    }
    
    pub fn constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
    
    pub fn output_guideline(mut self, guideline: impl Into<String>) -> Self {
        self.output_guidelines.push(guideline.into());
        self
    }
    
    pub fn thinking_style(mut self, style: ThinkingStyle) -> Self {
        self.thinking_style = style;
        self
    }
    
    pub fn build(self) -> AgentRole {
        AgentRole {
            name: self.name,
            codename: self.codename,
            description: self.description,
            responsibilities: self.responsibilities,
            constraints: self.constraints,
            output_guidelines: self.output_guidelines,
            thinking_style: self.thinking_style,
        }
    }
}

fn main() {
    println!("Testing AgentRole system...");
    
    // Test builder pattern
    let role = AgentRole::builder("Test Role")
        .description("Test description")
        .responsibility("Test responsibility")
        .constraint("Test constraint")
        .output_guideline("Test guideline")
        .thinking_style(ThinkingStyle::Critical)
        .build();
    
    println!("Built role: {}", role.name);
    
    // Test system prompt generation
    let prompt = role.to_system_prompt();
    println!("Generated prompt length: {}", prompt.len());
    assert!(prompt.contains("# Agent Role: Test Role"));
    
    // Test architect preset
    let architect = AgentRole::architect();
    println!("Architect role: {}", architect.name);
    assert_eq!(architect.name, "Systems Architect");
    
    println!("All tests passed!");
}
