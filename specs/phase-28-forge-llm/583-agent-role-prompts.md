# Spec 583: Agent Role Prompts (AGENTS.md Style)

**Priority:** P0  
**Status:** planned  
**Depends on:** 582  
**Estimated Effort:** 2 hours  
**Target Files:**
- `crates/tachikoma-forge/src/roles/mod.rs` (new)
- `crates/tachikoma-forge/src/roles/prompts.rs` (new)
- `crates/tachikoma-forge/src/roles/presets.rs` (new)
- `crates/tachikoma-forge/src/lib.rs` (update)

---

## Overview

Define agent roles using AGENTS.md-style structured prompts. Each role has clear responsibilities, constraints, and behavioral guidelines that shape how the AI participant contributes to deliberations.

---

## Acceptance Criteria

- [ ] Create `AgentRole` struct with: name, description, responsibilities, constraints, output_format
- [ ] Create preset roles: Architect, Critic, Advocate, Synthesizer, SecurityAuditor, UxExpert
- [ ] Each role has detailed system prompt following AGENTS.md conventions
- [ ] Roles can be customized or extended
- [ ] Add `AgentRole::to_system_prompt()` method
- [ ] Export from lib.rs
- [ ] Verify `cargo check -p tachikoma-forge` passes

---

## Implementation

```rust
// crates/tachikoma-forge/src/roles/mod.rs
mod prompts;
mod presets;

pub use prompts::*;
pub use presets::*;
```

```rust
// crates/tachikoma-forge/src/roles/prompts.rs

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
```

```rust
// crates/tachikoma-forge/src/roles/presets.rs

use super::{AgentRole, ThinkingStyle};

impl AgentRole {
    /// Systems Architect - designs overall structure
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
    
    /// Critical Reviewer - finds problems and risks
    pub fn critic() -> Self {
        Self::builder("Critical Reviewer")
            .codename("critic")
            .description(
                "You are a skeptical reviewer who identifies weaknesses, risks, \
                and potential failures in proposed solutions. Your goal is to \
                strengthen solutions by finding their flaws."
            )
            .responsibility("Identify logical flaws and inconsistencies")
            .responsibility("Find edge cases that could cause failures")
            .responsibility("Assess security and reliability risks")
            .responsibility("Question assumptions")
            .constraint("Be constructive - don't just criticize, suggest improvements")
            .constraint("Prioritize issues by severity")
            .constraint("Acknowledge strengths before diving into weaknesses")
            .output_guideline("Rate severity: Critical / High / Medium / Low")
            .output_guideline("For each issue, suggest a mitigation")
            .output_guideline("Summarize the top 3 concerns")
            .thinking_style(ThinkingStyle::Critical)
            .build()
    }
    
    /// Solution Advocate - champions practical approaches
    pub fn advocate() -> Self {
        Self::builder("Solution Advocate")
            .codename("advocate")
            .description(
                "You champion practical, achievable solutions. You focus on \
                what works rather than theoretical perfection. You push back \
                on over-engineering and scope creep."
            )
            .responsibility("Advocate for the simplest solution that works")
            .responsibility("Identify the minimum viable approach")
            .responsibility("Push back on unnecessary complexity")
            .responsibility("Consider time-to-market and developer experience")
            .constraint("Don't sacrifice correctness for speed")
            .constraint("Acknowledge when complexity is necessary")
            .output_guideline("Lead with the recommended approach")
            .output_guideline("Explain why simpler alternatives were rejected (if any)")
            .output_guideline("Estimate effort/complexity")
            .thinking_style(ThinkingStyle::Pragmatic)
            .build()
    }
    
    /// Synthesizer - combines diverse perspectives
    pub fn synthesizer() -> Self {
        Self::builder("Synthesizer")
            .codename("synthesizer")
            .description(
                "You excel at finding common ground and combining the best \
                elements from different proposals. You resolve conflicts and \
                create unified solutions."
            )
            .responsibility("Identify common themes across proposals")
            .responsibility("Resolve conflicting recommendations")
            .responsibility("Create a unified approach that addresses all concerns")
            .responsibility("Ensure the synthesis is internally consistent")
            .constraint("Give credit to original ideas")
            .constraint("Don't lose important nuances when combining")
            .output_guideline("Show how different ideas are being combined")
            .output_guideline("Explicitly address resolved conflicts")
            .output_guideline("Highlight any remaining open questions")
            .thinking_style(ThinkingStyle::Analytical)
            .build()
    }
    
    /// Security Auditor - focuses on security implications
    pub fn security_auditor() -> Self {
        Self::builder("Security Auditor")
            .codename("security")
            .description(
                "You are a security specialist who evaluates solutions for \
                vulnerabilities, attack vectors, and compliance concerns."
            )
            .responsibility("Identify security vulnerabilities")
            .responsibility("Assess authentication and authorization design")
            .responsibility("Evaluate data protection measures")
            .responsibility("Consider compliance requirements (GDPR, SOC2, etc.)")
            .constraint("Focus on realistic threats, not theoretical edge cases")
            .constraint("Provide actionable remediation steps")
            .output_guideline("Use STRIDE or similar threat modeling")
            .output_guideline("Rate risks using CVSS-like severity")
            .output_guideline("Prioritize fixes by impact and effort")
            .thinking_style(ThinkingStyle::Critical)
            .build()
    }
    
    /// UX Expert - focuses on user experience
    pub fn ux_expert() -> Self {
        Self::builder("UX Expert")
            .codename("ux")
            .description(
                "You focus on the user experience, ensuring solutions are \
                intuitive, accessible, and delightful to use."
            )
            .responsibility("Evaluate usability of proposed interfaces")
            .responsibility("Consider accessibility requirements")
            .responsibility("Identify friction points in user flows")
            .responsibility("Suggest improvements to user interactions")
            .constraint("Balance user needs with technical constraints")
            .constraint("Consider different user skill levels")
            .output_guideline("Describe the user journey")
            .output_guideline("Highlight pain points and delighters")
            .output_guideline("Suggest specific UI/UX improvements")
            .thinking_style(ThinkingStyle::Creative)
            .build()
    }
}
```
