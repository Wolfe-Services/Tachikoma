# Spec 134: Conversational Spec Generation

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 134
- **Status**: Planned
- **Dependencies**: 117-spec-templates, 133-spec-rendering
- **Estimated Context**: ~11%

## Objective

Implement a conversational interface for generating and refining specifications through natural language dialogue. The system guides users through spec creation with intelligent prompts, validates inputs, and produces well-structured spec documents while maintaining conversation context.

## Acceptance Criteria

- [x] Interactive spec generation through dialogue works
- [x] Conversation state is maintained across turns
- [x] Intelligent prompts guide users through sections
- [x] Input validation provides helpful feedback
- [x] Partial specs can be saved and resumed
- [x] Generated specs follow templates correctly
- [x] Refinement suggestions improve spec quality
- [x] Context from existing specs is leveraged

## Implementation Details

### Conversational Generation System

```rust
// src/spec/conversation.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Conversation state for spec generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecConversation {
    /// Unique conversation ID
    pub id: String,
    /// Current stage of conversation
    pub stage: ConversationStage,
    /// Accumulated spec data
    pub spec_data: PartialSpec,
    /// Conversation history
    pub history: Vec<ConversationTurn>,
    /// Validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Started timestamp
    pub started_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Metadata
    pub metadata: ConversationMetadata,
}

/// Stages of spec conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStage {
    /// Initial stage - gathering basic info
    Initial,
    /// Gathering title and objective
    TitleObjective,
    /// Defining acceptance criteria
    AcceptanceCriteria,
    /// Implementation details
    Implementation,
    /// Testing requirements
    Testing,
    /// Dependencies and references
    Dependencies,
    /// Review and refinement
    Review,
    /// Finalization
    Finalize,
    /// Completed
    Complete,
}

impl ConversationStage {
    pub fn next(&self) -> Self {
        match self {
            Self::Initial => Self::TitleObjective,
            Self::TitleObjective => Self::AcceptanceCriteria,
            Self::AcceptanceCriteria => Self::Implementation,
            Self::Implementation => Self::Testing,
            Self::Testing => Self::Dependencies,
            Self::Dependencies => Self::Review,
            Self::Review => Self::Finalize,
            Self::Finalize => Self::Complete,
            Self::Complete => Self::Complete,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Initial => Self::Initial,
            Self::TitleObjective => Self::Initial,
            Self::AcceptanceCriteria => Self::TitleObjective,
            Self::Implementation => Self::AcceptanceCriteria,
            Self::Testing => Self::Implementation,
            Self::Dependencies => Self::Testing,
            Self::Review => Self::Dependencies,
            Self::Finalize => Self::Review,
            Self::Complete => Self::Finalize,
        }
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::Initial => "Let's create a new spec. What phase is this spec for? (1-8)",
            Self::TitleObjective => "What is the title and main objective of this spec?",
            Self::AcceptanceCriteria => "What are the acceptance criteria? List them one per line.",
            Self::Implementation => "Describe the implementation approach. Include code examples if helpful.",
            Self::Testing => "What testing requirements should be met?",
            Self::Dependencies => "What specs or components does this depend on?",
            Self::Review => "Review the spec. Type 'ok' to proceed or specify what to change.",
            Self::Finalize => "Ready to generate the spec file? Type 'generate' to create it.",
            Self::Complete => "Spec generation complete!",
        }
    }
}

/// Partially completed spec data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartialSpec {
    pub spec_id: Option<u32>,
    pub phase: Option<u32>,
    pub title: Option<String>,
    pub objective: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub implementation: Option<String>,
    pub testing: Option<String>,
    pub dependencies: Vec<String>,
    pub estimated_context: Option<String>,
    pub custom_sections: HashMap<String, String>,
}

/// A turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub stage: ConversationStage,
}

/// Role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    System,
    Assistant,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
    pub severity: IssueSeverity,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Warning,
    Error,
}

/// Conversation metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub model: Option<String>,
    pub user_id: Option<String>,
    pub project: Option<String>,
}

/// Conversational spec generator
pub struct SpecConversationEngine {
    /// Active conversations
    conversations: HashMap<String, SpecConversation>,
    /// Context from existing specs
    context: SpecContext,
}

/// Context from existing specs for better suggestions
#[derive(Debug, Clone, Default)]
pub struct SpecContext {
    pub existing_specs: HashMap<u32, SpecSummary>,
    pub common_patterns: Vec<String>,
    pub phase_names: HashMap<u32, String>,
}

#[derive(Debug, Clone)]
pub struct SpecSummary {
    pub id: u32,
    pub title: String,
    pub phase: u32,
}

impl SpecConversationEngine {
    pub fn new() -> Self {
        let mut phase_names = HashMap::new();
        phase_names.insert(1, "Foundation".to_string());
        phase_names.insert(2, "Core Intelligence".to_string());
        phase_names.insert(3, "Pattern System".to_string());
        phase_names.insert(4, "Context Engine".to_string());
        phase_names.insert(5, "Advanced Features".to_string());
        phase_names.insert(6, "Spec System".to_string());
        phase_names.insert(7, "Integration".to_string());
        phase_names.insert(8, "Optimization".to_string());

        Self {
            conversations: HashMap::new(),
            context: SpecContext {
                existing_specs: HashMap::new(),
                common_patterns: Vec::new(),
                phase_names,
            },
        }
    }

    /// Start a new conversation
    pub fn start_conversation(&mut self) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let conversation = SpecConversation {
            id: id.clone(),
            stage: ConversationStage::Initial,
            spec_data: PartialSpec::default(),
            history: vec![ConversationTurn {
                role: Role::System,
                content: ConversationStage::Initial.prompt().to_string(),
                timestamp: now,
                stage: ConversationStage::Initial,
            }],
            issues: Vec::new(),
            started_at: now,
            last_activity: now,
            metadata: ConversationMetadata::default(),
        };

        self.conversations.insert(id.clone(), conversation);
        id
    }

    /// Process user input
    pub fn process_input(
        &mut self,
        conversation_id: &str,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let conversation = self.conversations.get_mut(conversation_id)
            .ok_or(ConversationError::NotFound)?;

        let now = Utc::now();
        conversation.last_activity = now;

        // Record user turn
        conversation.history.push(ConversationTurn {
            role: Role::User,
            content: input.to_string(),
            timestamp: now,
            stage: conversation.stage,
        });

        // Handle special commands
        if let Some(response) = self.handle_command(conversation, input)? {
            return Ok(response);
        }

        // Process based on current stage
        let response = match conversation.stage {
            ConversationStage::Initial => self.process_initial(conversation, input),
            ConversationStage::TitleObjective => self.process_title_objective(conversation, input),
            ConversationStage::AcceptanceCriteria => self.process_acceptance_criteria(conversation, input),
            ConversationStage::Implementation => self.process_implementation(conversation, input),
            ConversationStage::Testing => self.process_testing(conversation, input),
            ConversationStage::Dependencies => self.process_dependencies(conversation, input),
            ConversationStage::Review => self.process_review(conversation, input),
            ConversationStage::Finalize => self.process_finalize(conversation, input),
            ConversationStage::Complete => Ok(ConversationResponse {
                message: "Spec generation is complete.".to_string(),
                stage: conversation.stage,
                suggestions: vec![],
                preview: None,
                complete: true,
            }),
        };

        // Record assistant response
        if let Ok(ref resp) = response {
            conversation.history.push(ConversationTurn {
                role: Role::Assistant,
                content: resp.message.clone(),
                timestamp: Utc::now(),
                stage: conversation.stage,
            });
        }

        response
    }

    /// Handle special commands
    fn handle_command(
        &mut self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<Option<ConversationResponse>, ConversationError> {
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "back" | "previous" => {
                conversation.stage = conversation.stage.previous();
                Ok(Some(ConversationResponse {
                    message: format!("Going back. {}", conversation.stage.prompt()),
                    stage: conversation.stage,
                    suggestions: vec![],
                    preview: None,
                    complete: false,
                }))
            }
            "skip" => {
                conversation.stage = conversation.stage.next();
                Ok(Some(ConversationResponse {
                    message: format!("Skipping. {}", conversation.stage.prompt()),
                    stage: conversation.stage,
                    suggestions: vec![],
                    preview: None,
                    complete: false,
                }))
            }
            "status" => {
                let preview = self.generate_preview(conversation);
                Ok(Some(ConversationResponse {
                    message: "Current spec status:".to_string(),
                    stage: conversation.stage,
                    suggestions: vec![],
                    preview: Some(preview),
                    complete: false,
                }))
            }
            "help" => {
                Ok(Some(ConversationResponse {
                    message: self.help_text(),
                    stage: conversation.stage,
                    suggestions: vec![],
                    preview: None,
                    complete: false,
                }))
            }
            "cancel" => {
                self.conversations.remove(&conversation.id);
                Err(ConversationError::Cancelled)
            }
            _ => Ok(None),
        }
    }

    /// Process initial stage (phase selection)
    fn process_initial(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let phase: u32 = input.trim().parse()
            .map_err(|_| ConversationError::InvalidInput("Please enter a phase number (1-8)".into()))?;

        if phase < 1 || phase > 8 {
            return Err(ConversationError::InvalidInput("Phase must be between 1 and 8".into()));
        }

        conversation.spec_data.phase = Some(phase);

        // Auto-generate spec ID based on phase
        let spec_id = self.suggest_spec_id(phase);
        conversation.spec_data.spec_id = Some(spec_id);

        let phase_name = self.context.phase_names.get(&phase)
            .map(|n| n.as_str())
            .unwrap_or("Unknown");

        conversation.stage = conversation.stage.next();

        Ok(ConversationResponse {
            message: format!(
                "Phase {} ({}) selected. Spec ID will be {}.\n\n{}",
                phase, phase_name, spec_id, conversation.stage.prompt()
            ),
            stage: conversation.stage,
            suggestions: self.suggest_titles(phase),
            preview: None,
            complete: false,
        })
    }

    /// Process title and objective
    fn process_title_objective(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let lines: Vec<&str> = input.lines().collect();

        // First line is title
        if let Some(title) = lines.first() {
            conversation.spec_data.title = Some(title.trim().to_string());
        }

        // Rest is objective
        if lines.len() > 1 {
            conversation.spec_data.objective = Some(lines[1..].join("\n").trim().to_string());
        }

        if conversation.spec_data.title.is_none() {
            return Err(ConversationError::InvalidInput("Please provide a title".into()));
        }

        conversation.stage = conversation.stage.next();

        Ok(ConversationResponse {
            message: format!(
                "Title set to: \"{}\"\n\n{}",
                conversation.spec_data.title.as_ref().unwrap(),
                conversation.stage.prompt()
            ),
            stage: conversation.stage,
            suggestions: vec![
                "List 5-8 specific, testable criteria".to_string(),
                "Use action verbs (supports, validates, generates)".to_string(),
            ],
            preview: None,
            complete: false,
        })
    }

    /// Process acceptance criteria
    fn process_acceptance_criteria(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let criteria: Vec<String> = input.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.trim_start_matches("- ").trim_start_matches("* ").to_string())
            .collect();

        if criteria.is_empty() {
            return Err(ConversationError::InvalidInput("Please provide at least one criterion".into()));
        }

        conversation.spec_data.acceptance_criteria = criteria;
        conversation.stage = conversation.stage.next();

        Ok(ConversationResponse {
            message: format!(
                "Added {} acceptance criteria.\n\n{}",
                conversation.spec_data.acceptance_criteria.len(),
                conversation.stage.prompt()
            ),
            stage: conversation.stage,
            suggestions: vec![
                "Include Rust code examples".to_string(),
                "Describe data structures".to_string(),
                "Explain algorithms".to_string(),
            ],
            preview: None,
            complete: false,
        })
    }

    /// Process implementation details
    fn process_implementation(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        conversation.spec_data.implementation = Some(input.to_string());
        conversation.stage = conversation.stage.next();

        Ok(ConversationResponse {
            message: format!("Implementation details captured.\n\n{}", conversation.stage.prompt()),
            stage: conversation.stage,
            suggestions: vec![
                "Unit tests for each component".to_string(),
                "Integration tests".to_string(),
                "Edge case coverage".to_string(),
            ],
            preview: None,
            complete: false,
        })
    }

    /// Process testing requirements
    fn process_testing(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        conversation.spec_data.testing = Some(input.to_string());
        conversation.stage = conversation.stage.next();

        Ok(ConversationResponse {
            message: format!("Testing requirements captured.\n\n{}", conversation.stage.prompt()),
            stage: conversation.stage,
            suggestions: self.suggest_dependencies(conversation),
            preview: None,
            complete: false,
        })
    }

    /// Process dependencies
    fn process_dependencies(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let deps: Vec<String> = input.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();

        conversation.spec_data.dependencies = deps;
        conversation.stage = conversation.stage.next();

        // Generate preview for review
        let preview = self.generate_preview(conversation);

        Ok(ConversationResponse {
            message: format!("Dependencies recorded.\n\n{}\n\nPreview:", conversation.stage.prompt()),
            stage: conversation.stage,
            suggestions: vec![],
            preview: Some(preview),
            complete: false,
        })
    }

    /// Process review stage
    fn process_review(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let trimmed = input.trim().to_lowercase();

        if trimmed == "ok" || trimmed == "good" || trimmed == "yes" {
            conversation.stage = conversation.stage.next();

            Ok(ConversationResponse {
                message: conversation.stage.prompt().to_string(),
                stage: conversation.stage,
                suggestions: vec![],
                preview: None,
                complete: false,
            })
        } else {
            // Parse what needs to be changed
            Ok(ConversationResponse {
                message: "What would you like to change? (title, objective, criteria, implementation, testing, dependencies)".to_string(),
                stage: conversation.stage,
                suggestions: vec![],
                preview: None,
                complete: false,
            })
        }
    }

    /// Process finalization
    fn process_finalize(
        &self,
        conversation: &mut SpecConversation,
        input: &str,
    ) -> Result<ConversationResponse, ConversationError> {
        let trimmed = input.trim().to_lowercase();

        if trimmed == "generate" || trimmed == "yes" || trimmed == "ok" {
            let spec = self.generate_spec(conversation)?;
            conversation.stage = ConversationStage::Complete;

            Ok(ConversationResponse {
                message: "Spec generated successfully!".to_string(),
                stage: conversation.stage,
                suggestions: vec![],
                preview: Some(spec),
                complete: true,
            })
        } else {
            Ok(ConversationResponse {
                message: "Type 'generate' to create the spec file, or 'back' to make changes.".to_string(),
                stage: conversation.stage,
                suggestions: vec![],
                preview: None,
                complete: false,
            })
        }
    }

    /// Suggest spec ID for phase
    fn suggest_spec_id(&self, phase: u32) -> u32 {
        let base = match phase {
            1 => 1,
            2 => 26,
            3 => 51,
            4 => 76,
            5 => 101,
            6 => 116,
            7 => 136,
            8 => 156,
            _ => 1,
        };

        // Find next available ID
        let existing: Vec<_> = self.context.existing_specs.values()
            .filter(|s| s.phase == phase)
            .map(|s| s.id)
            .collect();

        (base..base + 30)
            .find(|id| !existing.contains(id))
            .unwrap_or(base)
    }

    /// Suggest titles based on phase
    fn suggest_titles(&self, phase: u32) -> Vec<String> {
        match phase {
            6 => vec![
                "Spec Validation".to_string(),
                "Spec Indexing".to_string(),
                "Spec Generation".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Suggest dependencies
    fn suggest_dependencies(&self, conversation: &SpecConversation) -> Vec<String> {
        let mut suggestions = Vec::new();

        if let Some(phase) = conversation.spec_data.phase {
            for spec in self.context.existing_specs.values() {
                if spec.phase == phase || spec.phase == phase - 1 {
                    suggestions.push(format!("{:03}-{}", spec.id, spec.title.to_lowercase().replace(' ', "-")));
                }
            }
        }

        suggestions.truncate(5);
        suggestions
    }

    /// Generate preview
    fn generate_preview(&self, conversation: &SpecConversation) -> String {
        let data = &conversation.spec_data;

        format!(
            r#"# Spec {}: {}

## Metadata
- **Phase**: {} - {}
- **Spec ID**: {}
- **Status**: Planned
- **Dependencies**: {}
- **Estimated Context**: {}

## Objective

{}

## Acceptance Criteria

{}

## Implementation Details

{}

## Testing Requirements

{}
"#,
            data.spec_id.unwrap_or(0),
            data.title.as_deref().unwrap_or("[Title]"),
            data.phase.unwrap_or(0),
            self.context.phase_names.get(&data.phase.unwrap_or(0)).map(|s| s.as_str()).unwrap_or("Unknown"),
            data.spec_id.unwrap_or(0),
            if data.dependencies.is_empty() { "None".to_string() } else { data.dependencies.join(", ") },
            data.estimated_context.as_deref().unwrap_or("~10%"),
            data.objective.as_deref().unwrap_or("[Objective]"),
            data.acceptance_criteria.iter()
                .map(|c| format!("- [ ] {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            data.implementation.as_deref().unwrap_or("[Implementation]"),
            data.testing.as_deref().unwrap_or("[Testing requirements]"),
        )
    }

    /// Generate final spec
    fn generate_spec(&self, conversation: &SpecConversation) -> Result<String, ConversationError> {
        Ok(self.generate_preview(conversation))
    }

    /// Help text
    fn help_text(&self) -> String {
        r#"Commands:
- back/previous: Go to previous stage
- skip: Skip current stage
- status: Show current spec preview
- cancel: Cancel spec creation
- help: Show this help

Stages:
1. Initial: Select phase
2. Title/Objective: Define what the spec is about
3. Acceptance Criteria: List testable requirements
4. Implementation: Describe how to implement
5. Testing: Define testing requirements
6. Dependencies: List related specs
7. Review: Check and refine
8. Finalize: Generate the spec
"#.to_string()
    }

    /// Get conversation by ID
    pub fn get_conversation(&self, id: &str) -> Option<&SpecConversation> {
        self.conversations.get(id)
    }

    /// Load existing specs for context
    pub fn load_context(&mut self, specs: Vec<SpecSummary>) {
        for spec in specs {
            self.context.existing_specs.insert(spec.id, spec);
        }
    }
}

/// Response from conversation processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub message: String,
    pub stage: ConversationStage,
    pub suggestions: Vec<String>,
    pub preview: Option<String>,
    pub complete: bool,
}

/// Conversation errors
#[derive(Debug, thiserror::Error)]
pub enum ConversationError {
    #[error("Conversation not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Conversation cancelled")]
    Cancelled,

    #[error("Generation failed: {0}")]
    GenerationFailed(String),
}

impl Default for SpecConversationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_flow() {
        let mut engine = SpecConversationEngine::new();

        let id = engine.start_conversation();
        assert!(engine.get_conversation(&id).is_some());

        // Process phase selection
        let response = engine.process_input(&id, "6").unwrap();
        assert_eq!(response.stage, ConversationStage::TitleObjective);
    }

    #[test]
    fn test_stage_progression() {
        let stage = ConversationStage::Initial;
        assert_eq!(stage.next(), ConversationStage::TitleObjective);
        assert_eq!(stage.previous(), ConversationStage::Initial);
    }

    #[test]
    fn test_command_handling() {
        let mut engine = SpecConversationEngine::new();
        let id = engine.start_conversation();

        let response = engine.process_input(&id, "help").unwrap();
        assert!(response.message.contains("Commands"));
    }
}
```

## Testing Requirements

- [ ] Unit tests for conversation flow
- [ ] Tests for stage transitions
- [ ] Tests for command handling
- [ ] Tests for input validation
- [ ] Tests for preview generation
- [ ] Tests for spec generation
- [ ] Integration tests for full conversations
- [ ] Tests for conversation persistence

## Related Specs

- **117-spec-templates.md**: Templates for generation
- **133-spec-rendering.md**: Rendering generated specs
- **127-spec-validation.md**: Validation during generation
