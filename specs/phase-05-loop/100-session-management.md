# 100 - Session Management

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 100
**Status:** Planned
**Dependencies:** 096-loop-runner-core, 019-async-runtime
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement session lifecycle management for Claude Code sessions - creating, maintaining, monitoring, and terminating sessions with proper state tracking and resource cleanup.

---

## Acceptance Criteria

- [ ] `Session` struct representing a Claude Code session
- [ ] `SessionManager` for lifecycle orchestration
- [ ] Session creation with configuration
- [ ] Session state monitoring
- [ ] Graceful session termination
- [ ] Session persistence for recovery
- [ ] Multiple concurrent session support
- [ ] Session pooling (optional)

---

## Implementation Details

### 1. Session Types (src/session/types.rs)

```rust
//! Session type definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ses_{}", self.0)
    }
}

/// State of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is being created.
    Creating,
    /// Session is ready and idle.
    Ready,
    /// Session is executing a prompt.
    Executing,
    /// Session is paused.
    Paused,
    /// Session has ended normally.
    Ended,
    /// Session terminated due to error.
    Error,
    /// Session was terminated by user.
    Terminated,
}

impl SessionState {
    /// Is the session in a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Ended | Self::Error | Self::Terminated)
    }

    /// Can the session accept prompts?
    pub fn can_execute(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Configuration for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Working directory for the session.
    pub working_dir: PathBuf,

    /// Claude command to execute.
    pub command: String,

    /// Command arguments.
    pub args: Vec<String>,

    /// Environment variables.
    pub env: HashMap<String, String>,

    /// Session timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: std::time::Duration,

    /// Whether to persist session state.
    pub persist: bool,

    /// State persistence path.
    pub state_path: Option<PathBuf>,

    /// Enable verbose logging.
    pub verbose: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            working_dir: PathBuf::from("."),
            command: "claude".to_string(),
            args: vec![],
            env: HashMap::new(),
            timeout: std::time::Duration::from_secs(3600),
            persist: true,
            state_path: None,
            verbose: false,
        }
    }
}

/// Information about a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: SessionId,
    /// Current state.
    pub state: SessionState,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Last activity time.
    pub last_activity: DateTime<Utc>,
    /// Number of prompts executed.
    pub prompt_count: u32,
    /// Total execution time in milliseconds.
    pub total_execution_ms: u64,
    /// Current context usage percentage.
    pub context_usage: u8,
    /// Session configuration.
    pub config: SessionConfig,
}

/// Response from executing a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    /// Exit code from the command.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether changes were made.
    pub made_changes: bool,
    /// Detected test results.
    pub tests_passed: Option<bool>,
    /// Files that were changed.
    pub files_changed: u32,
    /// Context usage after execution.
    pub context_usage: u8,
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

### 2. Session Implementation (src/session/session.rs)

```rust
//! Individual session implementation.

use super::types::{PromptResponse, SessionConfig, SessionId, SessionInfo, SessionState};
use crate::error::{LoopError, LoopResult};

use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, instrument, warn};

/// A Claude Code session.
pub struct Session {
    /// Session ID.
    id: SessionId,
    /// Configuration.
    config: SessionConfig,
    /// Current state.
    state: RwLock<SessionState>,
    /// The child process.
    process: Mutex<Option<Child>>,
    /// Stdin writer.
    stdin: Mutex<Option<tokio::process::ChildStdin>>,
    /// Statistics.
    stats: RwLock<SessionStats>,
    /// Context usage tracker.
    context_tracker: Arc<ContextTracker>,
}

/// Session statistics.
#[derive(Debug, Clone, Default)]
struct SessionStats {
    prompt_count: u32,
    total_execution_ms: u64,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// Create a new session.
    pub async fn new(config: SessionConfig) -> LoopResult<Self> {
        let id = SessionId::new();
        let now = chrono::Utc::now();

        let session = Self {
            id,
            config,
            state: RwLock::new(SessionState::Creating),
            process: Mutex::new(None),
            stdin: Mutex::new(None),
            stats: RwLock::new(SessionStats {
                created_at: now,
                last_activity: now,
                ..Default::default()
            }),
            context_tracker: Arc::new(ContextTracker::new()),
        };

        Ok(session)
    }

    /// Get the session ID.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Get current state.
    pub async fn state(&self) -> SessionState {
        *self.state.read().await
    }

    /// Start the session.
    #[instrument(skip(self), fields(session_id = %self.id))]
    pub async fn start(&self) -> LoopResult<()> {
        info!("Starting session");

        let mut process = Command::new(&self.config.command)
            .args(&self.config.args)
            .current_dir(&self.config.working_dir)
            .envs(&self.config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| LoopError::SessionStartFailed {
                session_id: self.id,
                source: e,
            })?;

        // Take ownership of stdin
        let stdin = process.stdin.take();

        *self.process.lock().await = Some(process);
        *self.stdin.lock().await = stdin;
        *self.state.write().await = SessionState::Ready;

        info!("Session started successfully");
        Ok(())
    }

    /// Execute a prompt in the session.
    #[instrument(skip(self, prompt), fields(session_id = %self.id))]
    pub async fn execute_prompt(&self, prompt: &str) -> LoopResult<PromptResponse> {
        // Verify state
        let current_state = self.state().await;
        if !current_state.can_execute() {
            return Err(LoopError::SessionNotReady {
                session_id: self.id,
                state: current_state,
            });
        }

        *self.state.write().await = SessionState::Executing;

        let start = std::time::Instant::now();

        // Send prompt to stdin
        {
            let mut stdin_guard = self.stdin.lock().await;
            if let Some(stdin) = stdin_guard.as_mut() {
                stdin
                    .write_all(prompt.as_bytes())
                    .await
                    .map_err(|e| LoopError::SessionWriteFailed {
                        session_id: self.id,
                        source: e,
                    })?;
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(|e| LoopError::SessionWriteFailed {
                        session_id: self.id,
                        source: e,
                    })?;
                stdin.flush().await.ok();
            } else {
                return Err(LoopError::SessionNotReady {
                    session_id: self.id,
                    state: SessionState::Error,
                });
            }
        }

        // Wait for response (with timeout)
        let response = tokio::time::timeout(
            self.config.timeout,
            self.wait_for_response(),
        )
        .await
        .map_err(|_| LoopError::SessionTimeout {
            session_id: self.id,
            timeout: self.config.timeout,
        })??;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.prompt_count += 1;
            stats.total_execution_ms += duration_ms;
            stats.last_activity = chrono::Utc::now();
        }

        // Update context tracker
        self.context_tracker.update_from_output(&response.stdout);

        *self.state.write().await = SessionState::Ready;

        Ok(PromptResponse {
            exit_code: response.exit_code,
            stdout: response.stdout,
            stderr: response.stderr,
            duration_ms,
            made_changes: self.detect_changes(&response.stdout),
            tests_passed: self.detect_test_results(&response.stdout),
            files_changed: self.count_file_changes(&response.stdout),
            context_usage: self.context_tracker.usage_percent(),
        })
    }

    /// Wait for response from the session.
    async fn wait_for_response(&self) -> LoopResult<RawResponse> {
        let mut process_guard = self.process.lock().await;
        let process = process_guard.as_mut().ok_or(LoopError::SessionNotReady {
            session_id: self.id,
            state: SessionState::Error,
        })?;

        let stdout = process.stdout.take();
        let stderr = process.stderr.take();

        let mut stdout_content = String::new();
        let mut stderr_content = String::new();

        // Read stdout
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            // Read until we see a completion marker or EOF
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        stdout_content.push_str(&line);
                        // Check for completion marker
                        if self.is_completion_marker(&line) {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }

            // Return stdout to process
            process.stdout = Some(reader.into_inner());
        }

        // Read stderr (non-blocking)
        if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();

            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                stderr_content.push_str(&line);
                line.clear();
            }

            process.stderr = Some(reader.into_inner());
        }

        Ok(RawResponse {
            exit_code: 0, // Process still running
            stdout: stdout_content,
            stderr: stderr_content,
        })
    }

    /// Check if a line indicates completion.
    fn is_completion_marker(&self, line: &str) -> bool {
        // Claude Code typically outputs specific markers when done
        line.contains(">>> ") || line.contains("[DONE]") || line.trim().is_empty()
    }

    /// Detect if changes were made from output.
    fn detect_changes(&self, output: &str) -> bool {
        let change_indicators = [
            "Created file",
            "Modified file",
            "Deleted file",
            "wrote to",
            "updated",
            "changed",
        ];

        change_indicators.iter().any(|ind| output.contains(ind))
    }

    /// Detect test results from output.
    fn detect_test_results(&self, output: &str) -> Option<bool> {
        if output.contains("test result:") || output.contains("Tests:") {
            // Look for failure indicators
            if output.contains("FAILED") || output.contains("failed") {
                Some(false)
            } else if output.contains("passed") || output.contains("ok") {
                Some(true)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Count file changes from output.
    fn count_file_changes(&self, output: &str) -> u32 {
        let patterns = ["Created file", "Modified file", "Deleted file"];
        patterns
            .iter()
            .map(|p| output.matches(p).count() as u32)
            .sum()
    }

    /// Get current context usage.
    pub async fn get_context_usage(&self) -> LoopResult<u8> {
        Ok(self.context_tracker.usage_percent())
    }

    /// Get session info.
    pub async fn info(&self) -> SessionInfo {
        let stats = self.stats.read().await;
        SessionInfo {
            id: self.id,
            state: *self.state.read().await,
            created_at: stats.created_at,
            last_activity: stats.last_activity,
            prompt_count: stats.prompt_count,
            total_execution_ms: stats.total_execution_ms,
            context_usage: self.context_tracker.usage_percent(),
            config: self.config.clone(),
        }
    }

    /// End the session gracefully.
    #[instrument(skip(self), fields(session_id = %self.id))]
    pub async fn end(&self) -> LoopResult<()> {
        info!("Ending session");

        let mut process_guard = self.process.lock().await;
        if let Some(mut process) = process_guard.take() {
            // Send exit command
            if let Some(stdin) = &mut *self.stdin.lock().await {
                let _ = stdin.write_all(b"/exit\n").await;
            }

            // Give it a moment to exit gracefully
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Kill if still running
            if let Err(e) = process.kill().await {
                debug!("Process may have already exited: {}", e);
            }
        }

        *self.state.write().await = SessionState::Ended;
        info!("Session ended");
        Ok(())
    }

    /// Terminate the session immediately.
    pub async fn terminate(&self) -> LoopResult<()> {
        info!("Terminating session {}", self.id);

        let mut process_guard = self.process.lock().await;
        if let Some(mut process) = process_guard.take() {
            process.kill().await.ok();
        }

        *self.state.write().await = SessionState::Terminated;
        Ok(())
    }
}

/// Raw response from session.
struct RawResponse {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

/// Tracks context window usage.
pub struct ContextTracker {
    /// Current estimated usage (0-100).
    usage: std::sync::atomic::AtomicU8,
    /// Token count estimate.
    token_count: std::sync::atomic::AtomicU64,
}

impl ContextTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            usage: std::sync::atomic::AtomicU8::new(0),
            token_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Update usage from output.
    pub fn update_from_output(&self, output: &str) {
        // Look for context usage indicators in output
        // Claude Code may output something like "Context: 45% used"
        if let Some(usage) = self.parse_context_usage(output) {
            self.usage.store(usage, std::sync::atomic::Ordering::Relaxed);
        }

        // Estimate tokens from output length
        let estimated_tokens = (output.len() / 4) as u64; // Rough estimate
        self.token_count.fetch_add(estimated_tokens, std::sync::atomic::Ordering::Relaxed);
    }

    /// Parse context usage from output.
    fn parse_context_usage(&self, output: &str) -> Option<u8> {
        // Look for patterns like "Context: 45%" or "context usage: 45%"
        let patterns = [
            regex::Regex::new(r"[Cc]ontext[:\s]+(\d+)%").ok()?,
        ];

        for pattern in &patterns {
            if let Some(caps) = pattern.captures(output) {
                if let Some(m) = caps.get(1) {
                    if let Ok(usage) = m.as_str().parse::<u8>() {
                        return Some(usage.min(100));
                    }
                }
            }
        }

        None
    }

    /// Get current usage percentage.
    pub fn usage_percent(&self) -> u8 {
        self.usage.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset the tracker.
    pub fn reset(&self) {
        self.usage.store(0, std::sync::atomic::Ordering::Relaxed);
        self.token_count.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}
```

### 3. Session Manager (src/session/manager.rs)

```rust
//! Session manager for lifecycle orchestration.

use super::session::Session;
use super::types::{SessionConfig, SessionId, SessionInfo, SessionState};
use crate::error::{LoopError, LoopResult};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Manages Claude Code sessions.
pub struct SessionManager {
    /// Default configuration for new sessions.
    default_config: SessionConfig,
    /// Active sessions.
    sessions: RwLock<HashMap<SessionId, Arc<Session>>>,
    /// Current session (the one being used by the loop).
    current_session: RwLock<Option<SessionId>>,
    /// Maximum concurrent sessions.
    max_sessions: usize,
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new(config: SessionConfig) -> Self {
        Self {
            default_config: config,
            sessions: RwLock::new(HashMap::new()),
            current_session: RwLock::new(None),
            max_sessions: 5,
        }
    }

    /// Get or create the current session.
    pub async fn get_or_create_session(&self) -> LoopResult<Arc<Session>> {
        // Check for existing current session
        if let Some(session_id) = *self.current_session.read().await {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&session_id) {
                let state = session.state().await;
                if !state.is_terminal() {
                    return Ok(session.clone());
                }
            }
        }

        // Create new session
        self.create_fresh_session().await
    }

    /// Create a fresh session.
    #[instrument(skip(self))]
    pub async fn create_fresh_session(&self) -> LoopResult<Arc<Session>> {
        info!("Creating fresh session");

        // Check capacity
        let session_count = self.sessions.read().await.len();
        if session_count >= self.max_sessions {
            // Clean up old sessions
            self.cleanup_terminated_sessions().await;

            let new_count = self.sessions.read().await.len();
            if new_count >= self.max_sessions {
                return Err(LoopError::TooManySessions {
                    max: self.max_sessions,
                });
            }
        }

        // Create session
        let session = Session::new(self.default_config.clone()).await?;
        let session_id = session.id();
        let session = Arc::new(session);

        // Start it
        session.start().await?;

        // Store it
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        // Set as current
        *self.current_session.write().await = Some(session_id);

        info!("Created session {}", session_id);
        Ok(session)
    }

    /// End the current session.
    pub async fn end_current_session(&self) -> LoopResult<()> {
        let session_id = self.current_session.write().await.take();

        if let Some(id) = session_id {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&id) {
                session.end().await?;
            }
        }

        Ok(())
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: SessionId) -> Option<Arc<Session>> {
        self.sessions.read().await.get(&id).cloned()
    }

    /// Get the current session.
    pub async fn current_session(&self) -> Option<Arc<Session>> {
        let current_id = *self.current_session.read().await;
        if let Some(id) = current_id {
            self.get_session(id).await
        } else {
            None
        }
    }

    /// List all sessions.
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::new();

        for session in sessions.values() {
            infos.push(session.info().await);
        }

        infos
    }

    /// Clean up terminated sessions.
    async fn cleanup_terminated_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let mut to_remove = Vec::new();

        for (id, session) in sessions.iter() {
            if session.state().await.is_terminal() {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            debug!("Removing terminated session {}", id);
            sessions.remove(&id);
        }
    }

    /// Terminate all sessions.
    pub async fn terminate_all(&self) -> LoopResult<()> {
        let sessions = self.sessions.read().await;

        for session in sessions.values() {
            session.terminate().await?;
        }

        Ok(())
    }

    /// Get session count.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get active session count.
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        let mut count = 0;

        for session in sessions.values() {
            if !session.state().await.is_terminal() {
                count += 1;
            }
        }

        count
    }
}
```

### 4. Module Root (src/session/mod.rs)

```rust
//! Session management for Claude Code.

pub mod manager;
pub mod session;
pub mod types;

pub use manager::SessionManager;
pub use session::{ContextTracker, Session};
pub use types::{PromptResponse, SessionConfig, SessionId, SessionInfo, SessionState};
```

---

## Testing Requirements

1. Session creates and starts successfully
2. Prompt execution returns response
3. Session state transitions correctly
4. Context usage is tracked
5. Session ends gracefully
6. Manager creates fresh sessions
7. Manager limits concurrent sessions
8. Terminated sessions are cleaned up

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Depends on: [019-async-runtime.md](../phase-01-common/019-async-runtime.md)
- Next: [101-fresh-context.md](101-fresh-context.md)
- Related: [102-redline-detection.md](102-redline-detection.md)
