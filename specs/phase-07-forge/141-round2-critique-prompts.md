# 141 - Round 2: Critique Prompt Construction

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 141
**Status:** Planned
**Dependencies:** 140-round1-draft, 158-forge-templates
**Estimated Context:** ~9% of Sonnet window

---

## Objective

Design and implement the critique prompt construction system that generates targeted, role-specific prompts for critic models to analyze the initial draft.

---

## Acceptance Criteria

- [x] Role-specific critique prompts
- [x] Structured output format for critiques
- [x] Category-specific evaluation criteria
- [x] Scoring rubric definition
- [x] Context window management for long drafts
- [x] Support for focused critiques on specific sections

---

## Implementation Details

### 1. Critique Prompt Builder (src/prompts/critique.rs)

```rust
//! Critique prompt construction.

use crate::{
    BrainstormTopic, ForgeConfig, ModelRequest, OutputType, Participant,
    ParticipantRole, SuggestionCategory,
};

/// Build a critique prompt for a participant.
pub fn build_critique_prompt(
    draft_content: &str,
    topic: &BrainstormTopic,
    critic: &Participant,
    config: &ForgeConfig,
) -> ModelRequest {
    let system = build_critique_system_prompt(critic, topic);
    let user = build_critique_user_prompt(draft_content, topic, critic, config);

    ModelRequest::new(system)
        .with_user_message(user)
        .with_temperature(0.6) // Slightly lower for more consistent critiques
}

/// Build the system prompt for critique.
fn build_critique_system_prompt(critic: &Participant, topic: &BrainstormTopic) -> String {
    let role_context = get_role_critique_context(critic.role);
    let output_type_criteria = get_output_type_criteria(topic.output_type);

    format!(
        r#"You are an expert reviewer participating in a multi-model brainstorming session.

Your Role: {role_name}
{role_context}

You are critiquing a {output_type} draft. Your critique should:
1. Be constructive and actionable
2. Identify specific strengths and weaknesses
3. Provide concrete suggestions for improvement
4. Score the draft objectively

{output_type_criteria}

IMPORTANT: Structure your critique using the exact format specified in the user prompt.
This format allows automated parsing of your feedback."#,
        role_name = critic.role.to_string(),
        output_type = output_type_name(topic.output_type),
    )
}

/// Get role-specific critique context.
fn get_role_critique_context(role: ParticipantRole) -> &'static str {
    match role {
        ParticipantRole::Critic => {
            r#"As a general critic, evaluate:
- Overall quality and completeness
- Logical consistency
- Clarity of communication
- Adherence to requirements"#
        }
        ParticipantRole::CodeReviewer => {
            r#"As a code reviewer, focus on:
- Code correctness and safety
- Error handling completeness
- API design quality
- Performance considerations
- Test coverage suggestions"#
        }
        ParticipantRole::DevilsAdvocate => {
            r#"As the devil's advocate, you MUST:
- Challenge fundamental assumptions
- Find edge cases and failure modes
- Question design decisions
- Identify potential security issues
- Consider what could go wrong"#
        }
        ParticipantRole::DomainExpert => {
            r#"As a domain expert, evaluate:
- Technical accuracy
- Industry best practices
- Appropriate abstractions
- Scalability considerations"#
        }
        ParticipantRole::Synthesizer => {
            r#"As a synthesizer reviewing for potential integration:
- Identify areas that may conflict
- Note sections needing clarification
- Flag ambiguous requirements"#
        }
        _ => {
            r#"Evaluate the draft holistically:
- Content quality
- Structure and organization
- Completeness"#
        }
    }
}

/// Get output type-specific evaluation criteria.
fn get_output_type_criteria(output_type: OutputType) -> &'static str {
    match output_type {
        OutputType::Specification => {
            r#"Specification Evaluation Criteria:
- Completeness: Are all necessary sections present?
- Clarity: Is the spec unambiguous?
- Implementability: Can this be directly implemented?
- Testability: Are acceptance criteria verifiable?
- Code Quality: Are code examples correct and idiomatic?"#
        }
        OutputType::Code => {
            r#"Code Evaluation Criteria:
- Correctness: Does it do what it's supposed to?
- Safety: Are there potential bugs or security issues?
- Readability: Is the code clear and well-documented?
- Efficiency: Are there obvious performance problems?
- Idiomaticity: Does it follow language conventions?"#
        }
        OutputType::Documentation => {
            r#"Documentation Evaluation Criteria:
- Accuracy: Is the information correct?
- Completeness: Are all topics covered?
- Accessibility: Is it appropriate for the target audience?
- Structure: Is it well-organized?
- Examples: Are examples helpful and correct?"#
        }
        OutputType::Design => {
            r#"Design Document Evaluation Criteria:
- Problem Definition: Is the problem clearly stated?
- Solution Fit: Does the solution address the problem?
- Trade-offs: Are alternatives and trade-offs discussed?
- Feasibility: Is the design implementable?
- Maintainability: Will this design age well?"#
        }
        OutputType::Freeform => {
            r#"General Evaluation Criteria:
- Quality: Is the content well-written?
- Relevance: Does it address the topic?
- Completeness: Is anything missing?
- Structure: Is it well-organized?"#
        }
    }
}

/// Build the user prompt for critique.
fn build_critique_user_prompt(
    draft_content: &str,
    topic: &BrainstormTopic,
    critic: &Participant,
    config: &ForgeConfig,
) -> String {
    let categories = get_relevant_categories(critic.role, topic.output_type);
    let category_list = categories
        .iter()
        .map(|c| format!("- {:?}", c))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# Critique Request

## Original Topic
**Title:** {title}
**Description:** {description}

## Draft to Critique

<draft>
{draft}
</draft>

## Your Task

Analyze this draft and provide a structured critique. Focus on categories relevant to your role:
{categories}

## Required Output Format

Your response MUST follow this exact structure:

```critique
## Strengths
- [List each strength as a bullet point]
- [Be specific about what works well]

## Weaknesses
- [List each weakness as a bullet point]
- [Be specific about problems]

## Suggestions
### Suggestion 1
- **Section:** [Which section this applies to, or "General"]
- **Category:** [One of: correctness, clarity, completeness, code_quality, architecture, performance, security, other]
- **Priority:** [1-5, where 1 is highest priority]
- **Description:** [Detailed suggestion for improvement]

### Suggestion 2
[Continue pattern for each suggestion]

## Overall Score
**Score:** [0-100]
**Justification:** [Brief explanation of the score]
```

## Guidelines

1. Be specific - reference exact sections or lines when possible
2. Be constructive - every weakness should have a corresponding suggestion
3. Be fair - acknowledge what works well, not just problems
4. Be actionable - suggestions should be implementable
5. Score objectively based on the criteria, not personal preference

Provide your critique now:"#,
        title = topic.title,
        description = topic.description,
        draft = maybe_truncate_draft(draft_content, config),
        categories = category_list,
    )
}

/// Get categories relevant to a role and output type.
fn get_relevant_categories(role: ParticipantRole, output_type: OutputType) -> Vec<SuggestionCategory> {
    let mut categories = vec![
        SuggestionCategory::Correctness,
        SuggestionCategory::Clarity,
        SuggestionCategory::Completeness,
    ];

    // Add role-specific categories
    match role {
        ParticipantRole::CodeReviewer => {
            categories.push(SuggestionCategory::CodeQuality);
            categories.push(SuggestionCategory::Security);
            categories.push(SuggestionCategory::Performance);
        }
        ParticipantRole::DevilsAdvocate => {
            categories.push(SuggestionCategory::Security);
            categories.push(SuggestionCategory::Architecture);
        }
        ParticipantRole::DomainExpert => {
            categories.push(SuggestionCategory::Architecture);
            categories.push(SuggestionCategory::Performance);
        }
        _ => {}
    }

    // Add output type-specific categories
    match output_type {
        OutputType::Code | OutputType::Specification => {
            if !categories.contains(&SuggestionCategory::CodeQuality) {
                categories.push(SuggestionCategory::CodeQuality);
            }
        }
        OutputType::Design => {
            if !categories.contains(&SuggestionCategory::Architecture) {
                categories.push(SuggestionCategory::Architecture);
            }
        }
        _ => {}
    }

    categories
}

/// Truncate draft if too long for context.
fn maybe_truncate_draft(content: &str, config: &ForgeConfig) -> String {
    const MAX_DRAFT_CHARS: usize = 50_000;

    if content.len() <= MAX_DRAFT_CHARS {
        content.to_string()
    } else {
        // Truncate intelligently at section boundaries
        let truncation_point = find_section_boundary(content, MAX_DRAFT_CHARS);
        let truncated = &content[..truncation_point];

        format!(
            "{}\n\n[Draft truncated - {} characters of {} total shown]\n\
             [Focus your critique on the sections shown above]",
            truncated,
            truncation_point,
            content.len()
        )
    }
}

/// Find a good truncation point near the target.
fn find_section_boundary(content: &str, target: usize) -> usize {
    // Look for section headers near the target
    let search_range = target.saturating_sub(1000)..target;

    for i in search_range.rev() {
        if i < content.len() && content[i..].starts_with("\n#") {
            return i;
        }
    }

    // Fall back to paragraph boundary
    for i in (target.saturating_sub(200)..target).rev() {
        if i < content.len() && content[i..].starts_with("\n\n") {
            return i;
        }
    }

    // Last resort: exact target
    target.min(content.len())
}

/// Get output type name.
fn output_type_name(output_type: OutputType) -> &'static str {
    match output_type {
        OutputType::Specification => "specification",
        OutputType::Code => "code",
        OutputType::Documentation => "documentation",
        OutputType::Design => "design document",
        OutputType::Freeform => "content",
    }
}

impl std::fmt::Display for ParticipantRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Generalist => write!(f, "Generalist"),
            Self::Drafter => write!(f, "Drafter"),
            Self::Critic => write!(f, "Critic"),
            Self::Synthesizer => write!(f, "Synthesizer"),
            Self::DomainExpert => write!(f, "Domain Expert"),
            Self::CodeReviewer => write!(f, "Code Reviewer"),
            Self::DevilsAdvocate => write!(f, "Devil's Advocate"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_critique_prompt() {
        let topic = BrainstormTopic::new("Test Feature", "A feature description");
        let critic = Participant::claude_sonnet().with_role(ParticipantRole::Critic);
        let config = ForgeConfig::default();

        let request = build_critique_prompt(
            "This is the draft content",
            &topic,
            &critic,
            &config,
        );

        assert!(!request.system.is_empty());
        assert!(!request.messages.is_empty());
    }

    #[test]
    fn test_truncate_long_draft() {
        let config = ForgeConfig::default();
        let long_content = "x".repeat(100_000);

        let result = maybe_truncate_draft(&long_content, &config);

        assert!(result.len() < long_content.len());
        assert!(result.contains("[Draft truncated"));
    }

    #[test]
    fn test_relevant_categories() {
        let categories = get_relevant_categories(
            ParticipantRole::CodeReviewer,
            OutputType::Code,
        );

        assert!(categories.contains(&SuggestionCategory::CodeQuality));
        assert!(categories.contains(&SuggestionCategory::Security));
    }
}
```

### 2. Focused Critique Prompts (src/prompts/focused_critique.rs)

```rust
//! Focused critique prompts for specific sections.

use crate::{BrainstormTopic, ForgeConfig, ModelRequest, Participant};

/// Build a focused critique prompt for a specific section.
pub fn build_focused_critique_prompt(
    draft_content: &str,
    section_name: &str,
    section_content: &str,
    topic: &BrainstormTopic,
    critic: &Participant,
    config: &ForgeConfig,
) -> ModelRequest {
    let system = format!(
        r#"You are reviewing a specific section of a larger draft.

Section to review: {section_name}

Your task is to provide deep, focused feedback on this section only.
Consider how it fits into the overall document context.

Role: {role}
{role_context}"#,
        role = critic.role,
        role_context = critic.role.system_prompt_modifier(),
    );

    let user = format!(
        r#"# Focused Section Review

## Document Context
**Topic:** {title}
**Section Under Review:** {section_name}

## Full Document (for context)
<full_document>
{full_doc}
</full_document>

## Section to Critique
<section>
{section}
</section>

## Your Task

Provide detailed critique of the "{section_name}" section. Your response should include:

1. **Section-Specific Strengths** (2-3 points)
2. **Section-Specific Weaknesses** (2-3 points)
3. **Detailed Suggestions** with exact proposed changes
4. **Integration Notes** - how this section relates to others
5. **Section Score** (0-100)

Focus on depth over breadth. Be specific and actionable."#,
        title = topic.title,
        section_name = section_name,
        full_doc = truncate_for_context(draft_content, 20_000),
        section = section_content,
    );

    ModelRequest::new(system)
        .with_user_message(user)
        .with_temperature(0.5)
}

/// Extract sections from a markdown document.
pub fn extract_sections(content: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_header = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.starts_with('#') {
            // Save previous section
            if !current_header.is_empty() {
                sections.push((current_header.clone(), current_content.trim().to_string()));
            }
            current_header = line.trim_start_matches('#').trim().to_string();
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if !current_header.is_empty() {
        sections.push((current_header, current_content.trim().to_string()));
    }

    sections
}

fn truncate_for_context(content: &str, max_chars: usize) -> &str {
    if content.len() <= max_chars {
        content
    } else {
        &content[..max_chars]
    }
}
```

---

## Testing Requirements

1. Prompts include all required structural elements
2. Role-specific context is correctly applied
3. Long drafts are truncated appropriately
4. Focused critique extracts correct sections
5. Output format instructions are parseable
6. Category selection matches role and output type

---

## Related Specs

- Depends on: [140-round1-draft.md](140-round1-draft.md)
- Depends on: [158-forge-templates.md](158-forge-templates.md)
- Next: [142-round2-critique-collect.md](142-round2-critique-collect.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
