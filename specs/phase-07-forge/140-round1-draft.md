# 140 - Round 1: Initial Draft Generation

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 140
**Status:** Planned
**Dependencies:** 139-forge-rounds, 158-forge-templates
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement the initial draft generation round where the designated drafter model creates the first version of the specification or content based on the brainstorm topic.

---

## Acceptance Criteria

- [ ] Draft prompt construction from topic
- [ ] Context assembly with references
- [ ] Draft execution with timeout
- [ ] Output parsing and validation
- [ ] Draft metadata capture (tokens, timing)
- [ ] Support for different output types

---

## Implementation Details

### 1. Draft Round Executor (src/rounds/draft.rs)

```rust
//! Initial draft round implementation.

use std::time::{Duration, Instant};
use tokio::time::timeout;

use crate::{
    BrainstormTopic, DraftRound, ForgeConfig, ForgeError, ForgeResult, ModelRequest,
    OutputType, Participant, ParticipantManager, Reference, ReferenceType,
    TokenCount,
};

/// Executor for draft rounds.
pub struct DraftExecutor<'a> {
    participants: &'a ParticipantManager,
    config: &'a ForgeConfig,
}

impl<'a> DraftExecutor<'a> {
    /// Create a new draft executor.
    pub fn new(participants: &'a ParticipantManager, config: &'a ForgeConfig) -> Self {
        Self { participants, config }
    }

    /// Execute a draft round.
    pub async fn execute(
        &self,
        round_number: usize,
        topic: &BrainstormTopic,
    ) -> ForgeResult<DraftRound> {
        // Select drafter
        let drafter = self.participants.get_drafter().await?;

        // Build the prompt
        let request = self.build_draft_request(topic, &drafter)?;

        // Execute with timeout and retries
        let mut last_error = None;
        let max_retries = self.config.rounds.draft.max_retries;
        let timeout_duration = Duration::from_secs(self.config.rounds.draft.timeout_secs);

        for attempt in 0..=max_retries {
            let start = Instant::now();

            match timeout(
                timeout_duration,
                self.participants.send_request(&drafter, request.clone()),
            ).await {
                Ok(Ok(response)) => {
                    // Validate the draft
                    self.validate_draft(&response.content, topic)?;

                    return Ok(DraftRound {
                        round_number,
                        drafter,
                        content: response.content,
                        prompt: self.get_prompt_summary(topic),
                        timestamp: response.timestamp,
                        tokens: response.tokens,
                        duration_ms: response.duration_ms,
                    });
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // Exponential backoff
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt as u32))).await;
                    }
                }
                Err(_) => {
                    last_error = Some(ForgeError::Timeout(
                        format!("Draft attempt {} timed out after {}s",
                            attempt + 1,
                            timeout_duration.as_secs())
                    ));
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ForgeError::Orchestration(
            "Draft failed with unknown error".to_string()
        )))
    }

    /// Build the draft request.
    fn build_draft_request(
        &self,
        topic: &BrainstormTopic,
        drafter: &Participant,
    ) -> ForgeResult<ModelRequest> {
        let system_prompt = self.build_system_prompt(topic, drafter);
        let user_prompt = self.build_user_prompt(topic);

        let mut request = ModelRequest::new(system_prompt)
            .with_user_message(user_prompt)
            .with_max_tokens(self.config.models.available
                .get(&self.config.models.default_drafter)
                .map(|m| m.max_tokens)
                .unwrap_or(4096))
            .with_temperature(0.7);

        Ok(request)
    }

    /// Build the system prompt for drafting.
    fn build_system_prompt(&self, topic: &BrainstormTopic, drafter: &Participant) -> String {
        let output_type_instruction = match topic.output_type {
            OutputType::Specification => {
                r#"You are drafting a technical specification. Your output should be:
- Well-structured with clear sections
- Technically precise and implementable
- Complete with all necessary details
- Include code examples where appropriate
- Follow markdown formatting"#
            }
            OutputType::Code => {
                r#"You are drafting code. Your output should be:
- Clean, well-documented code
- Include all necessary imports and dependencies
- Have proper error handling
- Include inline comments for complex logic
- Be production-ready quality"#
            }
            OutputType::Documentation => {
                r#"You are drafting documentation. Your output should be:
- Clear and accessible to the target audience
- Well-organized with logical flow
- Include examples where helpful
- Use proper markdown formatting
- Be comprehensive yet concise"#
            }
            OutputType::Design => {
                r#"You are drafting a design document. Your output should be:
- Present clear problem statement
- Explore alternatives considered
- Justify design decisions
- Address trade-offs
- Include diagrams or ASCII art where helpful"#
            }
            OutputType::Freeform => {
                "You are drafting content based on the given topic. Be comprehensive and well-structured."
            }
        };

        let role_modifier = drafter.role.system_prompt_modifier();

        format!(
            r#"You are participating in a multi-model brainstorming session as the initial drafter.

{output_type_instruction}

{role_modifier}

Your draft will be critiqued by other AI models, so:
1. Be thorough but leave room for improvement
2. Clearly state any assumptions you're making
3. Mark areas where you're uncertain with [UNCERTAIN] tags
4. Structure your output so it's easy to critique specific sections

This is a collaborative process - your draft is the starting point for refinement."#
        )
    }

    /// Build the user prompt.
    fn build_user_prompt(&self, topic: &BrainstormTopic) -> String {
        let mut prompt = format!(
            "# Topic: {}\n\n## Description\n{}\n",
            topic.title,
            topic.description
        );

        // Add constraints
        if !topic.constraints.is_empty() {
            prompt.push_str("\n## Constraints\n");
            for (i, constraint) in topic.constraints.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, constraint));
            }
        }

        // Add references
        if !topic.references.is_empty() {
            prompt.push_str("\n## Reference Materials\n");
            for reference in &topic.references {
                match reference.ref_type {
                    ReferenceType::Inline => {
                        prompt.push_str(&format!(
                            "\n### {}\n```\n{}\n```\n",
                            reference.name,
                            reference.content
                        ));
                    }
                    ReferenceType::File | ReferenceType::Spec => {
                        prompt.push_str(&format!(
                            "\n### {} (from {})\n{}\n",
                            reference.name,
                            reference.content,
                            // Content would be loaded
                            "[Content loaded from file]"
                        ));
                    }
                    ReferenceType::Url => {
                        prompt.push_str(&format!(
                            "\n### {} (see: {})\n",
                            reference.name,
                            reference.content
                        ));
                    }
                }
            }
        }

        // Add output instructions
        prompt.push_str(&format!(
            "\n## Instructions\nPlease create an initial draft for this {}. \
             Structure your output clearly and be comprehensive.\n",
            match topic.output_type {
                OutputType::Specification => "specification",
                OutputType::Code => "code implementation",
                OutputType::Documentation => "documentation",
                OutputType::Design => "design document",
                OutputType::Freeform => "content",
            }
        ));

        prompt
    }

    /// Get a summary of the prompt for logging.
    fn get_prompt_summary(&self, topic: &BrainstormTopic) -> String {
        format!(
            "Draft {} for: {}",
            match topic.output_type {
                OutputType::Specification => "spec",
                OutputType::Code => "code",
                OutputType::Documentation => "docs",
                OutputType::Design => "design",
                OutputType::Freeform => "content",
            },
            topic.title
        )
    }

    /// Validate the draft output.
    fn validate_draft(&self, content: &str, topic: &BrainstormTopic) -> ForgeResult<()> {
        // Check minimum length
        if content.len() < 100 {
            return Err(ForgeError::Validation(
                "Draft is too short (minimum 100 characters)".to_string()
            ));
        }

        // Check for output type-specific requirements
        match topic.output_type {
            OutputType::Specification => {
                // Should have headers
                if !content.contains('#') {
                    return Err(ForgeError::Validation(
                        "Specification should have section headers".to_string()
                    ));
                }
            }
            OutputType::Code => {
                // Should have code blocks
                if !content.contains("```") && !content.contains("fn ") && !content.contains("function") {
                    return Err(ForgeError::Validation(
                        "Code draft should contain code".to_string()
                    ));
                }
            }
            _ => {}
        }

        // Check for placeholder text that shouldn't be there
        let bad_patterns = [
            "[INSERT",
            "[TODO: fill",
            "[PLACEHOLDER]",
            "Lorem ipsum",
        ];

        for pattern in bad_patterns {
            if content.contains(pattern) {
                return Err(ForgeError::Validation(
                    format!("Draft contains placeholder text: {}", pattern)
                ));
            }
        }

        Ok(())
    }
}

/// Build a draft prompt (convenience function).
pub fn build_draft_prompt(topic: &BrainstormTopic, config: &ForgeConfig) -> ModelRequest {
    let system = format!(
        r#"You are an expert technical writer participating in a collaborative brainstorming session.

Your task is to create an initial draft based on the given topic. This draft will be:
1. Critiqued by other AI models
2. Synthesized with feedback
3. Refined iteratively until convergence

Guidelines:
- Be thorough but concise
- Structure content with clear sections
- Include code examples where relevant
- Mark uncertain areas with [UNCERTAIN]
- Use markdown formatting

Output type: {:?}"#,
        topic.output_type
    );

    let user = format!(
        "# {}\n\n{}\n\n{}",
        topic.title,
        topic.description,
        if topic.constraints.is_empty() {
            String::new()
        } else {
            format!("Constraints:\n- {}", topic.constraints.join("\n- "))
        }
    );

    ModelRequest::new(system)
        .with_user_message(user)
        .with_temperature(0.7)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_user_prompt() {
        let topic = BrainstormTopic::new(
            "Test Topic",
            "A description of the test topic"
        ).with_constraint("Must be testable");

        let executor = DraftExecutor {
            participants: todo!(),
            config: &ForgeConfig::default(),
        };

        let prompt = executor.build_user_prompt(&topic);

        assert!(prompt.contains("Test Topic"));
        assert!(prompt.contains("A description"));
        assert!(prompt.contains("Must be testable"));
    }

    #[test]
    fn test_validate_draft_too_short() {
        let topic = BrainstormTopic::new("Test", "Test description");
        let executor = DraftExecutor {
            participants: todo!(),
            config: &ForgeConfig::default(),
        };

        let result = executor.validate_draft("Too short", &topic);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_draft_placeholder() {
        let topic = BrainstormTopic::new("Test", "Test description");
        let executor = DraftExecutor {
            participants: todo!(),
            config: &ForgeConfig::default(),
        };

        let content = "This is a longer draft that contains [INSERT YOUR CONTENT HERE] placeholder";
        let result = executor.validate_draft(content, &topic);
        assert!(result.is_err());
    }
}
```

### 2. Reference Loading (src/references.rs)

```rust
//! Reference material loading.

use std::path::Path;

use crate::{ForgeError, ForgeResult, Reference, ReferenceType};

/// Load reference content if needed.
pub async fn load_reference_content(reference: &Reference) -> ForgeResult<String> {
    match reference.ref_type {
        ReferenceType::Inline => Ok(reference.content.clone()),
        ReferenceType::File => load_file_content(&reference.content).await,
        ReferenceType::Spec => load_spec_content(&reference.content).await,
        ReferenceType::Url => {
            // URLs are just linked, not fetched during drafting
            Ok(format!("See: {}", reference.content))
        }
    }
}

/// Load content from a file.
async fn load_file_content(path: &str) -> ForgeResult<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| ForgeError::Io(format!("Failed to load reference file: {}", e)))
}

/// Load content from a spec file.
async fn load_spec_content(spec_id: &str) -> ForgeResult<String> {
    // Find spec in specs directory
    let spec_path = find_spec_file(spec_id)?;
    load_file_content(&spec_path).await
}

/// Find a spec file by ID.
fn find_spec_file(spec_id: &str) -> ForgeResult<String> {
    // Implementation would search specs directory
    // For now, assume specs/phase-XX/XXX-name.md format
    let pattern = format!("specs/**/{}-*.md", spec_id);

    // Would use glob to find matching file
    Err(ForgeError::NotFound(format!("Spec {} not found", spec_id)))
}

/// Truncate content if too long.
pub fn truncate_reference(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        let truncated = &content[..max_chars];
        format!("{}\n\n[Content truncated at {} characters]", truncated, max_chars)
    }
}
```

---

## Testing Requirements

1. Draft generation completes within timeout
2. Retry logic handles transient failures
3. Validation catches invalid drafts
4. Reference content is properly included
5. Different output types generate appropriate prompts
6. Token usage is tracked accurately

---

## Related Specs

- Depends on: [139-forge-rounds.md](139-forge-rounds.md)
- Depends on: [158-forge-templates.md](158-forge-templates.md)
- Next: [141-round2-critique-prompts.md](141-round2-critique-prompts.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
