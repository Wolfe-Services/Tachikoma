# 101 - Fresh Context Creation

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 101
**Status:** Planned
**Dependencies:** 100-session-management
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement fresh context creation for the Ralph Loop - the process of starting a new Claude Code session with a clean context window, preserving essential state while discarding accumulated context.

---

## Acceptance Criteria

- [x] Fresh context creates new session
- [x] Essential state is preserved across reboots
- [x] Context handoff message generation
- [x] Configurable state preservation rules
- [x] Clean termination of old session
- [x] Metrics tracking for reboots
- [x] Hooks for pre/post reboot actions
- [x] Warm-up prompt support

---

## Implementation Details

### 1. Context Types (src/context/types.rs)

```rust
//! Context management types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// State that should be preserved across context reboots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreservedState {
    /// Current task/objective being worked on.
    pub current_objective: Option<String>,

    /// Files that have been modified in this loop run.
    pub modified_files: Vec<PathBuf>,

    /// Key decisions made during the session.
    pub decisions: Vec<Decision>,

    /// Known issues or blockers.
    pub known_issues: Vec<Issue>,

    /// Progress markers (completed items).
    pub completed_items: Vec<String>,

    /// Pending items still to do.
    pub pending_items: Vec<String>,

    /// Custom key-value state.
    pub custom: HashMap<String, String>,

    /// Test state.
    pub test_state: Option<TestState>,
}

/// A recorded decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// What was decided.
    pub description: String,
    /// Why it was decided.
    pub rationale: Option<String>,
    /// When it was made.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A known issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue description.
    pub description: String,
    /// Severity level.
    pub severity: IssueSeverity,
    /// Related files.
    pub files: Vec<PathBuf>,
    /// Whether it's been addressed.
    pub addressed: bool,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Test execution state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestState {
    /// Last known passing tests.
    pub passing: Vec<String>,
    /// Currently failing tests.
    pub failing: Vec<String>,
    /// Tests that were fixed.
    pub fixed: Vec<String>,
    /// Tests that regressed.
    pub regressed: Vec<String>,
}

/// Configuration for fresh context creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshContextConfig {
    /// Maximum preserved state size (bytes).
    pub max_state_size: usize,

    /// Files to include in context handoff.
    pub include_files: Vec<PathBuf>,

    /// Whether to include recent git history.
    pub include_git_history: bool,

    /// Number of recent commits to include.
    pub git_history_count: u32,

    /// Whether to include test state.
    pub include_test_state: bool,

    /// Warm-up prompt to send after context creation.
    pub warmup_prompt: Option<String>,

    /// Delay after warm-up before resuming.
    #[serde(with = "humantime_serde")]
    pub warmup_delay: std::time::Duration,
}

impl Default for FreshContextConfig {
    fn default() -> Self {
        Self {
            max_state_size: 50_000, // 50KB
            include_files: vec![],
            include_git_history: true,
            git_history_count: 5,
            include_test_state: true,
            warmup_prompt: None,
            warmup_delay: std::time::Duration::from_secs(2),
        }
    }
}

/// Result of a fresh context creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshContextResult {
    /// The new session ID.
    pub new_session_id: crate::session::SessionId,
    /// The old session ID.
    pub old_session_id: Option<crate::session::SessionId>,
    /// State that was preserved.
    pub preserved_state: PreservedState,
    /// Handoff message that was sent.
    pub handoff_message: String,
    /// Time taken for the transition.
    pub transition_time_ms: u64,
    /// Whether warm-up was performed.
    pub warmed_up: bool,
}

mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&humantime::format_duration(*duration).to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(serde::de::Error::custom)
    }
}
```

### 2. Context Manager (src/context/manager.rs)

```rust
//! Fresh context management.

use super::types::{
    Decision, FreshContextConfig, FreshContextResult, Issue, PreservedState, TestState,
};
use crate::error::{LoopError, LoopResult};
use crate::session::{Session, SessionManager};

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Manages fresh context creation and state preservation.
pub struct ContextManager {
    /// Configuration.
    config: FreshContextConfig,
    /// Current preserved state.
    state: RwLock<PreservedState>,
    /// Session manager reference.
    session_manager: Arc<SessionManager>,
    /// State extractor.
    extractor: StateExtractor,
    /// Handoff generator.
    handoff_generator: HandoffGenerator,
}

impl ContextManager {
    /// Create a new context manager.
    pub fn new(config: FreshContextConfig, session_manager: Arc<SessionManager>) -> Self {
        Self {
            config: config.clone(),
            state: RwLock::new(PreservedState::default()),
            session_manager,
            extractor: StateExtractor::new(),
            handoff_generator: HandoffGenerator::new(config),
        }
    }

    /// Create a fresh context.
    #[instrument(skip(self))]
    pub async fn create_fresh_context(&self) -> LoopResult<FreshContextResult> {
        info!("Creating fresh context");
        let start = std::time::Instant::now();

        // Get old session ID
        let old_session = self.session_manager.current_session().await;
        let old_session_id = old_session.as_ref().map(|s| s.id());

        // Extract state from current session before ending it
        if let Some(session) = &old_session {
            self.extract_state_from_session(session).await?;
        }

        // Get preserved state
        let preserved_state = self.state.read().await.clone();

        // Generate handoff message
        let handoff_message = self.handoff_generator.generate(&preserved_state).await?;

        // End old session
        self.session_manager.end_current_session().await?;

        // Create new session
        let new_session = self.session_manager.create_fresh_session().await?;
        let new_session_id = new_session.id();

        // Send handoff message to new session
        debug!("Sending handoff message to new session");
        new_session.execute_prompt(&handoff_message).await?;

        // Perform warm-up if configured
        let warmed_up = if let Some(warmup) = &self.config.warmup_prompt {
            debug!("Performing warm-up");
            tokio::time::sleep(self.config.warmup_delay).await;
            new_session.execute_prompt(warmup).await?;
            true
        } else {
            false
        };

        let transition_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "Fresh context created in {}ms (old: {:?}, new: {})",
            transition_time_ms, old_session_id, new_session_id
        );

        Ok(FreshContextResult {
            new_session_id,
            old_session_id,
            preserved_state,
            handoff_message,
            transition_time_ms,
            warmed_up,
        })
    }

    /// Extract state from current session.
    async fn extract_state_from_session(&self, session: &Session) -> LoopResult<()> {
        let mut state = self.state.write().await;

        // Extract what we can from the session
        // This would involve parsing session output for state markers

        // Add any git changes
        if self.config.include_git_history {
            if let Ok(files) = self.extractor.get_modified_files().await {
                state.modified_files.extend(files);
            }
        }

        Ok(())
    }

    /// Update the current objective.
    pub async fn set_objective(&self, objective: impl Into<String>) {
        let mut state = self.state.write().await;
        state.current_objective = Some(objective.into());
    }

    /// Record a decision.
    pub async fn record_decision(&self, description: impl Into<String>, rationale: Option<String>) {
        let mut state = self.state.write().await;
        state.decisions.push(Decision {
            description: description.into(),
            rationale,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Record a known issue.
    pub async fn record_issue(&self, issue: Issue) {
        let mut state = self.state.write().await;
        state.known_issues.push(issue);
    }

    /// Mark an item as completed.
    pub async fn mark_completed(&self, item: impl Into<String>) {
        let mut state = self.state.write().await;
        let item = item.into();
        state.pending_items.retain(|i| i != &item);
        state.completed_items.push(item);
    }

    /// Add a pending item.
    pub async fn add_pending(&self, item: impl Into<String>) {
        let mut state = self.state.write().await;
        state.pending_items.push(item.into());
    }

    /// Update test state.
    pub async fn update_test_state(&self, test_state: TestState) {
        let mut state = self.state.write().await;
        state.test_state = Some(test_state);
    }

    /// Get current preserved state.
    pub async fn get_state(&self) -> PreservedState {
        self.state.read().await.clone()
    }

    /// Clear preserved state.
    pub async fn clear_state(&self) {
        let mut state = self.state.write().await;
        *state = PreservedState::default();
    }

    /// Set custom state value.
    pub async fn set_custom(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut state = self.state.write().await;
        state.custom.insert(key.into(), value.into());
    }
}

/// Extracts state from various sources.
pub struct StateExtractor;

impl StateExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Get modified files from git.
    pub async fn get_modified_files(&self) -> LoopResult<Vec<std::path::PathBuf>> {
        let output = tokio::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .await
            .map_err(|e| LoopError::CommandFailed {
                command: "git status".to_string(),
                source: e,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<std::path::PathBuf> = stdout
            .lines()
            .filter_map(|line| {
                if line.len() > 3 {
                    Some(std::path::PathBuf::from(line[3..].trim()))
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }

    /// Get recent git commits.
    pub async fn get_recent_commits(&self, count: u32) -> LoopResult<Vec<String>> {
        let output = tokio::process::Command::new("git")
            .args(["log", "--oneline", "-n", &count.to_string()])
            .output()
            .await
            .map_err(|e| LoopError::CommandFailed {
                command: "git log".to_string(),
                source: e,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(String::from).collect())
    }
}

/// Generates handoff messages for new contexts.
pub struct HandoffGenerator {
    config: FreshContextConfig,
}

impl HandoffGenerator {
    pub fn new(config: FreshContextConfig) -> Self {
        Self { config }
    }

    /// Generate a handoff message from preserved state.
    pub async fn generate(&self, state: &PreservedState) -> LoopResult<String> {
        let mut parts = Vec::new();

        parts.push("# Context Handoff\n".to_string());
        parts.push("You are continuing work from a previous session. Here is the preserved context:\n".to_string());

        // Current objective
        if let Some(objective) = &state.current_objective {
            parts.push(format!("## Current Objective\n{}\n", objective));
        }

        // Completed items
        if !state.completed_items.is_empty() {
            parts.push("## Completed\n".to_string());
            for item in &state.completed_items {
                parts.push(format!("- [x] {}\n", item));
            }
        }

        // Pending items
        if !state.pending_items.is_empty() {
            parts.push("## Pending\n".to_string());
            for item in &state.pending_items {
                parts.push(format!("- [ ] {}\n", item));
            }
        }

        // Modified files
        if !state.modified_files.is_empty() {
            parts.push("## Modified Files\n".to_string());
            for file in &state.modified_files {
                parts.push(format!("- {}\n", file.display()));
            }
        }

        // Recent decisions
        if !state.decisions.is_empty() {
            parts.push("## Key Decisions\n".to_string());
            for decision in state.decisions.iter().rev().take(5) {
                parts.push(format!("- {}", decision.description));
                if let Some(rationale) = &decision.rationale {
                    parts.push(format!(" ({})", rationale));
                }
                parts.push("\n".to_string());
            }
        }

        // Known issues
        let unaddressed: Vec<_> = state.known_issues.iter().filter(|i| !i.addressed).collect();
        if !unaddressed.is_empty() {
            parts.push("## Known Issues\n".to_string());
            for issue in unaddressed {
                parts.push(format!("- [{}] {}\n", format!("{:?}", issue.severity).to_uppercase(), issue.description));
            }
        }

        // Test state
        if self.config.include_test_state {
            if let Some(test_state) = &state.test_state {
                if !test_state.failing.is_empty() {
                    parts.push("## Failing Tests\n".to_string());
                    for test in &test_state.failing {
                        parts.push(format!("- {}\n", test));
                    }
                }
            }
        }

        parts.push("\nPlease continue working on the pending items.\n".to_string());

        let message = parts.join("");

        // Truncate if too large
        if message.len() > self.config.max_state_size {
            warn!(
                "Handoff message too large ({} bytes), truncating to {}",
                message.len(),
                self.config.max_state_size
            );
            Ok(message.chars().take(self.config.max_state_size).collect())
        } else {
            Ok(message)
        }
    }
}
```

### 3. Module Root (src/context/mod.rs)

```rust
//! Fresh context management.

pub mod manager;
pub mod types;

pub use manager::{ContextManager, HandoffGenerator, StateExtractor};
pub use types::{
    Decision, FreshContextConfig, FreshContextResult, Issue, IssueSeverity,
    PreservedState, TestState,
};
```

---

## Testing Requirements

1. Fresh context creates new session
2. State is preserved across transitions
3. Handoff message contains essential info
4. Old session is properly terminated
5. Warm-up prompt executes if configured
6. State size limits are enforced
7. Git integration extracts modified files
8. Test state is preserved when enabled

---

## Related Specs

- Depends on: [100-session-management.md](100-session-management.md)
- Next: [102-redline-detection.md](102-redline-detection.md)
- Related: [103-auto-reboot.md](103-auto-reboot.md)
