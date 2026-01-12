//! Forge integration test infrastructure.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use tachikoma_forge::{
    ForgeError, ForgeResult, ForgeSession, ForgeSessionConfig, ForgeTopic, 
    ForgeSessionStatus, Participant, Critique, TokenUsage, ConflictResolution,
    SuggestionCategory, Suggestion
};

/// Mock provider for testing.
pub struct MockProvider {
    /// Responses to return for each request pattern.
    responses: Arc<RwLock<Vec<MockResponse>>>,
    /// Request history.
    requests: Arc<RwLock<Vec<RecordedRequest>>>,
    /// Failure mode.
    failure_mode: Arc<RwLock<Option<FailureMode>>>,
}

/// A mock response configuration.
pub struct MockResponse {
    /// Pattern to match in request.
    pub pattern: String,
    /// Response content.
    pub content: String,
    /// Simulated tokens.
    pub tokens: TokenUsage,
    /// Simulated delay in ms.
    pub delay_ms: u64,
}

/// Recorded request for verification.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub system: String,
    pub user_message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Failure modes for testing.
pub enum FailureMode {
    /// Return error for all requests.
    AlwaysFail(String),
    /// Fail for specific request number.
    FailOnRequest(usize),
    /// Simulate rate limiting.
    RateLimited,
    /// Simulate timeout.
    Timeout,
}

impl MockProvider {
    /// Create a new mock provider.
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(Vec::new())),
            requests: Arc::new(RwLock::new(Vec::new())),
            failure_mode: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a mock response.
    pub async fn add_response(&self, response: MockResponse) {
        self.responses.write().await.push(response);
    }

    /// Set failure mode.
    pub async fn set_failure_mode(&self, mode: Option<FailureMode>) {
        *self.failure_mode.write().await = mode;
    }

    /// Get recorded requests.
    pub async fn get_requests(&self) -> Vec<RecordedRequest> {
        self.requests.read().await.clone()
    }

    /// Clear recorded requests.
    pub async fn clear_requests(&self) {
        self.requests.write().await.clear();
    }

    /// Simulate a model request and return a response.
    pub async fn simulate_request(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> ForgeResult<String> {
        // Record request
        {
            let mut requests = self.requests.write().await;
            requests.push(RecordedRequest {
                system: system_prompt.to_string(),
                user_message: user_message.to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        // Check failure mode
        {
            let failure_mode = self.failure_mode.read().await;
            if let Some(ref mode) = *failure_mode {
                match mode {
                    FailureMode::AlwaysFail(msg) => {
                        return Err(ForgeError::Session(msg.clone()));
                    }
                    FailureMode::FailOnRequest(n) => {
                        let count = self.requests.read().await.len();
                        if count == *n {
                            return Err(ForgeError::Session(
                                format!("Simulated failure on request {}", n)
                            ));
                        }
                    }
                    FailureMode::RateLimited => {
                        return Err(ForgeError::Session("Rate limited".to_string()));
                    }
                    FailureMode::Timeout => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        return Err(ForgeError::Session("Simulated timeout".to_string()));
                    }
                }
            }
        }

        // Find matching response
        let responses = self.responses.read().await;
        let request_text = format!("{} {}", system_prompt, user_message);

        for mock in responses.iter().rev() {
            if request_text.contains(&mock.pattern) {
                if mock.delay_ms > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(mock.delay_ms)).await;
                }

                return Ok(mock.content.clone());
            }
        }

        // Default response
        Ok("Default mock response".to_string())
    }
}

/// Create test configuration.
pub fn test_config() -> ForgeSessionConfig {
    ForgeSessionConfig {
        max_rounds: 5,
        convergence_threshold: 0.8,
        round_timeout_ms: 60_000,
    }
}

/// Create test topic.
pub fn test_topic() -> ForgeTopic {
    ForgeTopic {
        title: "Test Specification".to_string(),
        description: "Create a test specification for the forge system".to_string(),
        constraints: vec![
            "Must be comprehensive".to_string(),
            "Must include examples".to_string(),
        ],
    }
}

/// Create mock responses for a standard session.
pub fn standard_mock_responses() -> Vec<MockResponse> {
    vec![
        // Draft response
        MockResponse {
            pattern: "initial draft".to_string(),
            content: r#"# Test Specification

## Objective
Test the forge system.

## Implementation
```rust
fn test() {
    println!("Hello");
}
```

## Testing
- Unit tests
- Integration tests"#.to_string(),
            tokens: TokenUsage { input: 500, output: 200 },
            delay_ms: 50,
        },
        // Critique response
        MockResponse {
            pattern: "critique".to_string(),
            content: r#"## Strengths
- Good structure
- Clear objective

## Weaknesses
- Missing error handling
- Needs more detail

## Suggestions
### Suggestion 1
- **Section:** Implementation
- **Category:** code_quality
- **Priority:** 2
- **Description:** Add error handling

## Overall Score
**Score:** 75
**Justification:** Good start but needs refinement"#.to_string(),
            tokens: TokenUsage { input: 800, output: 150 },
            delay_ms: 50,
        },
        // Synthesis response
        MockResponse {
            pattern: "synthesis".to_string(),
            content: r#"## Conflict Resolutions
None identified.

## Changes Made
### Change 1
- **Section:** Implementation
- **Type:** modification
- **Description:** Added error handling

## Improved Draft

# Test Specification

## Objective
Test the forge system.

## Implementation
```rust
fn test() -> Result<(), Error> {
    println!("Hello");
    Ok(())
}
```

## Testing
- Unit tests
- Integration tests
- Error case tests"#.to_string(),
            tokens: TokenUsage { input: 1000, output: 300 },
            delay_ms: 50,
        },
        // Convergence response
        MockResponse {
            pattern: "finalization".to_string(),
            content: r#"AGREES: yes
SCORE: 85
CONCERNS:
- none"#.to_string(),
            tokens: TokenUsage { input: 200, output: 50 },
            delay_ms: 20,
        },
    ]
}

/// Helper to create a test critique.
pub fn create_test_critique(score: u8, suggestions: Vec<Suggestion>) -> Critique {
    Critique {
        critic: Participant::claude_sonnet(),
        score,
        strengths: vec!["Good structure".to_string()],
        weaknesses: vec!["Needs improvement".to_string()],
        suggestions,
        raw_content: "Mock critique content".to_string(),
        tokens: TokenUsage { input: 100, output: 50 },
        duration_ms: 100,
    }
}

/// Helper to create a test suggestion.
pub fn create_test_suggestion(
    section: Option<String>,
    text: String,
    priority: u8,
    category: SuggestionCategory,
) -> Suggestion {
    Suggestion {
        section,
        text,
        priority,
        category,
    }
}

/// Helper to create a test conflict resolution.
pub fn create_test_conflict_resolution(issue: String, resolution: String) -> ConflictResolution {
    ConflictResolution { issue, resolution }
}