//! Chat command for conversational spec creation.

use clap::Parser;
use std::path::PathBuf;
use std::io::{self, Write};

use crate::cli::CommandContext;
use crate::commands::Execute;
use crate::error::CliError;

/// Start conversational spec creation
#[derive(Debug, Parser)]
pub struct ChatCommand {
    /// Initial prompt for spec creation
    #[arg(help = "Initial description of what you want to build")]
    pub prompt: Option<String>,

    /// Output file path for generated spec
    #[arg(short, long, help = "Output path for the generated spec")]
    pub output: Option<PathBuf>,

    /// Auto-generate questions and answers (non-interactive)
    #[arg(long, help = "Generate questions and answers automatically")]
    pub auto: bool,

    /// Create spec without conversation
    #[arg(long, help = "Create spec directly from prompt without interview")]
    pub direct: bool,
}

impl ChatCommand {
    pub async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        if self.auto || self.direct {
            self.execute_auto_mode(ctx).await
        } else {
            self.execute_interactive_mode(ctx).await
        }
    }

    async fn execute_interactive_mode(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        println!("🤖 Tachikoma Spec Interview");
        println!("Let's create a specification together!\n");

        // Get initial description
        let description = if let Some(prompt) = &self.prompt {
            prompt.clone()
        } else {
            self.prompt_user("What would you like to build? Describe it in your own words:")?
        };

        // Start interview flow
        let interview = SpecInterview::new(description);
        let conversation = self.conduct_interview(interview).await?;
        
        // Generate spec from conversation
        let spec_content = self.generate_spec_from_conversation(&conversation)?;
        
        // Handle output
        match &self.output {
            Some(path) => {
                std::fs::write(path, &spec_content)?;
                println!("\n✅ Spec saved to: {}", path.display());
            }
            None => {
                println!("\n📄 Generated Spec:\n");
                println!("{}", spec_content);
                
                // Ask if they want to save
                if self.prompt_yes_no("Would you like to save this spec to a file?")? {
                    let filename = self.prompt_user("Enter filename (e.g., specs/auth/github-oauth.md):")?;
                    let path = PathBuf::from(filename);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, &spec_content)?;
                    println!("✅ Spec saved to: {}", path.display());
                }
            }
        }

        // Offer next steps
        println!("\nNext steps:");
        println!("1. Review and edit the generated spec");
        println!("2. Send to Forge for multi-model review: tachikoma forge review <spec-file>");
        println!("3. Start implementation: tachikoma run <spec-file>");

        Ok(())
    }

    async fn execute_auto_mode(&self, _ctx: &CommandContext) -> Result<(), CliError> {
        let prompt = self.prompt.as_ref().ok_or_else(|| {
            CliError::InvalidInput("Prompt required for auto mode".to_string())
        })?;

        println!("🤖 Auto-generating spec for: {}", prompt);
        
        // For now, just generate a basic spec
        let spec_content = self.generate_basic_spec(prompt)?;
        
        match &self.output {
            Some(path) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, &spec_content)?;
                println!("✅ Spec saved to: {}", path.display());
            }
            None => {
                println!("{}", spec_content);
            }
        }

        Ok(())
    }

    async fn conduct_interview(&self, mut interview: SpecInterview) -> Result<Conversation, CliError> {
        let mut conversation = Conversation::new(interview.initial_description.clone());

        // Phase 1: Goal Gathering
        conversation.add_message("tachikoma", &format!(
            "Got it! You want to: {}\nLet me ask a few clarifying questions...\n",
            interview.initial_description
        ));

        // Phase 2: Clarifying Questions
        let questions = interview.generate_clarifying_questions();
        for question in questions {
            let answer = self.prompt_user(&question)?;
            conversation.add_exchange(&question, &answer);
            interview.record_answer(&question, &answer);
        }

        // Phase 3: Technical Probing  
        let technical_questions = interview.generate_technical_questions();
        for question in technical_questions {
            let answer = self.prompt_user(&question)?;
            conversation.add_exchange(&question, &answer);
            interview.record_answer(&question, &answer);
        }

        Ok(conversation)
    }

    fn generate_spec_from_conversation(&self, conversation: &Conversation) -> Result<String, CliError> {
        // Extract key information from conversation
        let title = self.extract_title_from_conversation(conversation);
        let phase = self.extract_phase_from_conversation(conversation);
        let acceptance_criteria = self.extract_acceptance_criteria_from_conversation(conversation);
        
        // Generate spec using template
        let spec = format!(
r#"# Spec XXX: {}

**Phase:** {} - TBD  
**Status:** Planned  
**Priority:** P2 - Medium  
**Dependencies:** TBD

## Overview

{}

## Problem Statement

{}

## Acceptance Criteria

{}

## Implementation Details

### Core Implementation

```rust
// TODO: Add implementation details based on conversation
```

## Testing Requirements

- [ ] Unit tests for core functionality
- [ ] Integration tests
- [ ] Documentation and examples

## References

Generated from conversational interview on {}.
"#,
            title,
            phase,
            conversation.initial_description,
            self.extract_problem_statement(conversation),
            acceptance_criteria,
            chrono::Utc::now().format("%Y-%m-%d")
        );

        Ok(spec)
    }

    fn generate_basic_spec(&self, prompt: &str) -> Result<String, CliError> {
        let title = prompt.trim();
        let acceptance_criteria = vec![
            "Core functionality is implemented",
            "Error handling is comprehensive", 
            "Documentation is complete",
            "Tests provide adequate coverage",
        ];

        let spec = format!(
r#"# Spec XXX: {}

**Phase:** TBD - TBD  
**Status:** Planned  
**Priority:** P2 - Medium  
**Dependencies:** TBD

## Overview

{}

## Problem Statement

Define and implement the requirements for: {}

## Acceptance Criteria

{}

## Implementation Details

### Core Implementation

```rust
// TODO: Add implementation details
```

## Testing Requirements

- [ ] Unit tests for core functionality
- [ ] Integration tests  
- [ ] Documentation and examples

## References

Auto-generated from prompt on {}.
"#,
            title,
            prompt,
            prompt,
            acceptance_criteria.iter()
                .map(|c| format!("- [ ] {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            chrono::Utc::now().format("%Y-%m-%d")
        );

        Ok(spec)
    }

    fn extract_title_from_conversation(&self, conversation: &Conversation) -> String {
        // Simple extraction - in practice this would be more sophisticated
        conversation.initial_description
            .split_whitespace()
            .take(8)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn extract_phase_from_conversation(&self, _conversation: &Conversation) -> u32 {
        // For now, default to phase 27 (current phase)
        27
    }

    fn extract_acceptance_criteria_from_conversation(&self, conversation: &Conversation) -> String {
        let mut criteria = Vec::new();
        
        // Extract from conversation answers
        for exchange in &conversation.exchanges {
            if exchange.answer.contains("should") || exchange.answer.contains("must") || exchange.answer.contains("need") {
                let criterion = self.extract_criterion_from_text(&exchange.answer);
                if !criterion.is_empty() {
                    criteria.push(format!("- [ ] {}", criterion));
                }
            }
        }

        if criteria.is_empty() {
            criteria = vec![
                "- [ ] Core functionality is implemented".to_string(),
                "- [ ] Error handling is comprehensive".to_string(),
                "- [ ] Tests provide adequate coverage".to_string(),
            ];
        }

        criteria.join("\n")
    }

    fn extract_criterion_from_text(&self, text: &str) -> String {
        // Simple extraction - in practice this would use NLP
        text.lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }

    fn extract_problem_statement(&self, conversation: &Conversation) -> String {
        format!(
            "Currently, there is a need for: {}\n\nThis spec addresses the requirements gathered through conversational interview.",
            conversation.initial_description
        )
    }

    fn prompt_user(&self, question: &str) -> Result<String, CliError> {
        print!("\n🤖 {}\n👤 ", question);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    fn prompt_yes_no(&self, question: &str) -> Result<bool, CliError> {
        print!("\n🤖 {} [y/N] ", question);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }
}

#[async_trait::async_trait]
impl Execute for ChatCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<(), CliError> {
        ChatCommand::execute(self, ctx).await
    }
}

/// Interview state for gathering spec requirements
#[derive(Debug)]
struct SpecInterview {
    initial_description: String,
    answers: Vec<(String, String)>,
}

impl SpecInterview {
    fn new(initial_description: String) -> Self {
        Self {
            initial_description,
            answers: Vec::new(),
        }
    }

    fn generate_clarifying_questions(&self) -> Vec<String> {
        vec![
            "What is the main goal or purpose of this feature?".to_string(),
            "Who are the primary users or stakeholders?".to_string(),
            "Are there any existing similar features or patterns to follow?".to_string(),
            "What should happen when things go wrong (error cases)?".to_string(),
        ]
    }

    fn generate_technical_questions(&self) -> Vec<String> {
        vec![
            "Are there any performance requirements or constraints?".to_string(),
            "What testing approach should we use?".to_string(),
            "Are there any security considerations?".to_string(),
            "How should this integrate with existing code?".to_string(),
        ]
    }

    fn record_answer(&mut self, question: &str, answer: &str) {
        self.answers.push((question.to_string(), answer.to_string()));
    }
}

/// Conversation history
#[derive(Debug)]
struct Conversation {
    initial_description: String,
    exchanges: Vec<Exchange>,
    messages: Vec<Message>,
}

impl Conversation {
    fn new(initial_description: String) -> Self {
        Self {
            initial_description,
            exchanges: Vec::new(),
            messages: Vec::new(),
        }
    }

    fn add_exchange(&mut self, question: &str, answer: &str) {
        self.exchanges.push(Exchange {
            question: question.to_string(),
            answer: answer.to_string(),
        });
    }

    fn add_message(&mut self, speaker: &str, content: &str) {
        self.messages.push(Message {
            speaker: speaker.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
        });
    }
}

#[derive(Debug)]
struct Exchange {
    question: String,
    answer: String,
}

#[derive(Debug)]
struct Message {
    speaker: String,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}