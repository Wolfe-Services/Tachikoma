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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architect_preset() {
        let architect = AgentRole::architect();
        assert_eq!(architect.name, "Systems Architect");
        assert_eq!(architect.codename, "architect");
        assert!(architect.description.contains("systems architect"));
        assert!(architect.responsibilities.iter().any(|r| r.contains("Design the high-level structure")));
        assert!(matches!(architect.thinking_style, ThinkingStyle::Systematic));
        
        let prompt = architect.to_system_prompt();
        assert!(prompt.contains("# Agent Role: Systems Architect"));
        assert!(prompt.contains("## Thinking Style: Systematic"));
    }

    #[test]
    fn test_critic_preset() {
        let critic = AgentRole::critic();
        assert_eq!(critic.name, "Critical Reviewer");
        assert_eq!(critic.codename, "critic");
        assert!(critic.description.contains("skeptical reviewer"));
        assert!(critic.responsibilities.iter().any(|r| r.contains("Identify logical flaws")));
        assert!(matches!(critic.thinking_style, ThinkingStyle::Critical));
    }

    #[test]
    fn test_advocate_preset() {
        let advocate = AgentRole::advocate();
        assert_eq!(advocate.name, "Solution Advocate");
        assert_eq!(advocate.codename, "advocate");
        assert!(advocate.description.contains("practical, achievable"));
        assert!(matches!(advocate.thinking_style, ThinkingStyle::Pragmatic));
    }

    #[test]
    fn test_synthesizer_preset() {
        let synthesizer = AgentRole::synthesizer();
        assert_eq!(synthesizer.name, "Synthesizer");
        assert_eq!(synthesizer.codename, "synthesizer");
        assert!(synthesizer.description.contains("finding common ground"));
        assert!(matches!(synthesizer.thinking_style, ThinkingStyle::Analytical));
    }

    #[test]
    fn test_security_auditor_preset() {
        let security = AgentRole::security_auditor();
        assert_eq!(security.name, "Security Auditor");
        assert_eq!(security.codename, "security");
        assert!(security.description.contains("security specialist"));
        assert!(matches!(security.thinking_style, ThinkingStyle::Critical));
    }

    #[test]
    fn test_ux_expert_preset() {
        let ux = AgentRole::ux_expert();
        assert_eq!(ux.name, "UX Expert");
        assert_eq!(ux.codename, "ux");
        assert!(ux.description.contains("user experience"));
        assert!(matches!(ux.thinking_style, ThinkingStyle::Creative));
    }

    #[test]
    fn test_all_presets_have_required_fields() {
        let roles = vec![
            AgentRole::architect(),
            AgentRole::critic(),
            AgentRole::advocate(),
            AgentRole::synthesizer(),
            AgentRole::security_auditor(),
            AgentRole::ux_expert(),
        ];

        for role in roles {
            assert!(!role.name.is_empty());
            assert!(!role.codename.is_empty());
            assert!(!role.description.is_empty());
            assert!(!role.responsibilities.is_empty());
            assert!(!role.constraints.is_empty());
            assert!(!role.output_guidelines.is_empty());
            
            // Verify all have at least 3 responsibilities
            assert!(role.responsibilities.len() >= 3);
            
            // Verify prompt generation works
            let prompt = role.to_system_prompt();
            assert!(prompt.contains(&format!("# Agent Role: {}", role.name)));
        }
    }
}