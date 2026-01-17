use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRole {
    pub name: String,
    pub codename: String,  // Short identifier like "architect" or "critic"
    pub description: String,
    pub responsibilities: Vec<String>,
    pub constraints: Vec<String>,
    pub output_guidelines: Vec<String>,
    pub thinking_style: ThinkingStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinkingStyle {
    Analytical,     // Step-by-step logical reasoning
    Creative,       // Exploratory, out-of-box thinking
    Critical,       // Skeptical, finds problems
    Pragmatic,      // Focus on what works
    Systematic,     // Comprehensive, thorough
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let role = AgentRole::builder("Test Role")
            .codename("tester")
            .description("A test role for validation")
            .responsibility("Write tests")
            .responsibility("Validate functionality")
            .constraint("Be thorough")
            .output_guideline("Use clear language")
            .thinking_style(ThinkingStyle::Analytical)
            .build();

        assert_eq!(role.name, "Test Role");
        assert_eq!(role.codename, "tester");
        assert_eq!(role.description, "A test role for validation");
        assert_eq!(role.responsibilities.len(), 2);
        assert_eq!(role.constraints.len(), 1);
        assert_eq!(role.output_guidelines.len(), 1);
        assert!(matches!(role.thinking_style, ThinkingStyle::Analytical));
    }

    #[test]
    fn test_to_system_prompt() {
        let role = AgentRole::builder("Test Agent")
            .description("Test description")
            .responsibility("Do test things")
            .constraint("Don't break things")
            .output_guideline("Be clear")
            .thinking_style(ThinkingStyle::Critical)
            .build();

        let prompt = role.to_system_prompt();
        
        assert!(prompt.contains("# Agent Role: Test Agent"));
        assert!(prompt.contains("Test description"));
        assert!(prompt.contains("## Your Responsibilities"));
        assert!(prompt.contains("- Do test things"));
        assert!(prompt.contains("## Constraints"));
        assert!(prompt.contains("- Don't break things"));
        assert!(prompt.contains("## Output Guidelines"));
        assert!(prompt.contains("- Be clear"));
        assert!(prompt.contains("## Thinking Style: Critical"));
    }

    #[test]
    fn test_auto_codename_generation() {
        let role = AgentRole::builder("Systems Architect").build();
        assert_eq!(role.codename, "systems_architect");
    }
}