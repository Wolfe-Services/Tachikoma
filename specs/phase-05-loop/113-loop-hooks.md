# 113 - Loop Hooks

**Phase:** 5 - Ralph Loop Runner
**Spec ID:** 113
**Status:** Planned
**Dependencies:** 096-loop-runner-core
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement the hook system for the Ralph Loop - extension points that allow custom code to run at various stages of the loop lifecycle, enabling customization and integration.

---

## Acceptance Criteria

- [ ] Define hook points (pre/post iteration, etc.)
- [ ] Hook registration and unregistration
- [ ] Async hook execution
- [ ] Hook priority ordering
- [ ] Hook timeout handling
- [ ] Error handling in hooks
- [ ] Built-in hooks (git, logging, etc.)
- [ ] External hook scripts support

---

## Implementation Details

### 1. Hook Types (src/hooks/types.rs)

```rust
//! Hook type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Points where hooks can be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookPoint {
    /// Before the loop starts.
    LoopStart,
    /// After the loop ends.
    LoopEnd,
    /// Before each iteration.
    PreIteration,
    /// After each iteration.
    PostIteration,
    /// Before a reboot.
    PreReboot,
    /// After a reboot.
    PostReboot,
    /// On test failure.
    OnTestFailure,
    /// On test success (all pass).
    OnTestSuccess,
    /// On progress detected.
    OnProgress,
    /// On no progress detected.
    OnNoProgress,
    /// On error.
    OnError,
    /// On mode switch.
    OnModeSwitch,
    /// On file change.
    OnFileChange,
    /// Before session start.
    PreSession,
    /// After session end.
    PostSession,
}

impl HookPoint {
    /// Get all hook points.
    pub fn all() -> Vec<Self> {
        vec![
            Self::LoopStart,
            Self::LoopEnd,
            Self::PreIteration,
            Self::PostIteration,
            Self::PreReboot,
            Self::PostReboot,
            Self::OnTestFailure,
            Self::OnTestSuccess,
            Self::OnProgress,
            Self::OnNoProgress,
            Self::OnError,
            Self::OnModeSwitch,
            Self::OnFileChange,
            Self::PreSession,
            Self::PostSession,
        ]
    }
}

/// Configuration for the hook system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Enable hook system.
    pub enabled: bool,
    /// Default timeout for hooks.
    #[serde(with = "humantime_serde")]
    pub default_timeout: Duration,
    /// Stop on hook failure.
    pub fail_on_hook_error: bool,
    /// External hooks directory.
    pub hooks_dir: Option<PathBuf>,
    /// Built-in hooks to enable.
    pub builtin_hooks: Vec<BuiltinHook>,
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_timeout: Duration::from_secs(30),
            fail_on_hook_error: false,
            hooks_dir: Some(PathBuf::from(".ralph/hooks")),
            builtin_hooks: vec![BuiltinHook::Logging],
        }
    }
}

/// Built-in hook types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinHook {
    /// Log events.
    Logging,
    /// Git operations.
    Git,
    /// File backup.
    Backup,
    /// Metrics collection.
    Metrics,
    /// Notification sending.
    Notification,
}

/// Definition of a hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    /// Hook name.
    pub name: String,
    /// Hook point(s).
    pub points: Vec<HookPoint>,
    /// Priority (higher runs first).
    pub priority: i32,
    /// Timeout override.
    pub timeout: Option<Duration>,
    /// Whether hook is enabled.
    pub enabled: bool,
    /// Hook type.
    pub hook_type: HookType,
    /// Conditions for running.
    pub conditions: Vec<HookCondition>,
}

/// Type of hook implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookType {
    /// Built-in Rust hook.
    Builtin { builtin: BuiltinHook },
    /// External script.
    Script { path: PathBuf, args: Vec<String> },
    /// Shell command.
    Command { command: String },
    /// Webhook call.
    Webhook { url: String, method: String },
}

/// Condition for running a hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookCondition {
    /// Run every N iterations.
    EveryN { n: u32 },
    /// Run only in certain modes.
    InMode { modes: Vec<String> },
    /// Run only if pattern matches.
    IfPattern { pattern: String },
    /// Run only if test status matches.
    IfTestStatus { passing: bool },
}

/// Context passed to hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Hook point being executed.
    pub hook_point: HookPoint,
    /// Current iteration.
    pub iteration: u32,
    /// Loop ID.
    pub loop_id: String,
    /// Session ID.
    pub session_id: Option<String>,
    /// Working directory.
    pub working_dir: PathBuf,
    /// Current mode.
    pub mode: String,
    /// Additional data.
    pub data: HashMap<String, serde_json::Value>,
}

impl HookContext {
    /// Set data value.
    pub fn set<V: Serialize>(&mut self, key: &str, value: V) {
        if let Ok(json) = serde_json::to_value(value) {
            self.data.insert(key.to_string(), json);
        }
    }

    /// Get data value.
    pub fn get<V: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<V> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Result from hook execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook name.
    pub hook_name: String,
    /// Whether it succeeded.
    pub success: bool,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Output if any.
    pub output: Option<String>,
    /// Error if failed.
    pub error: Option<String>,
    /// Whether to continue loop.
    pub continue_loop: bool,
    /// Modified context data.
    pub modified_data: HashMap<String, serde_json::Value>,
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

### 2. Hook Manager (src/hooks/manager.rs)

```rust
//! Hook management and execution.

use super::types::{
    BuiltinHook, HookCondition, HookContext, HookDefinition, HookPoint, HookResult,
    HookType, HooksConfig,
};
use crate::error::{LoopError, LoopResult};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages hook registration and execution.
pub struct HookManager {
    /// Configuration.
    config: HooksConfig,
    /// Registered hooks by point.
    hooks: RwLock<HashMap<HookPoint, Vec<RegisteredHook>>>,
    /// Hook implementations.
    implementations: RwLock<HashMap<String, Arc<dyn Hook>>>,
}

/// A registered hook.
struct RegisteredHook {
    definition: HookDefinition,
    implementation: Arc<dyn Hook>,
}

/// Trait for hook implementations.
#[async_trait::async_trait]
pub trait Hook: Send + Sync {
    /// Get hook name.
    fn name(&self) -> &str;

    /// Execute the hook.
    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult>;

    /// Check if hook should run given conditions.
    fn should_run(&self, context: &HookContext, conditions: &[HookCondition]) -> bool {
        for condition in conditions {
            match condition {
                HookCondition::EveryN { n } => {
                    if context.iteration % n != 0 {
                        return false;
                    }
                }
                HookCondition::InMode { modes } => {
                    if !modes.contains(&context.mode) {
                        return false;
                    }
                }
                HookCondition::IfPattern { pattern } => {
                    // Check if pattern matches any data
                    let matches = context.data.values().any(|v| {
                        v.as_str().map(|s| s.contains(pattern)).unwrap_or(false)
                    });
                    if !matches {
                        return false;
                    }
                }
                HookCondition::IfTestStatus { passing } => {
                    let tests_passing = context.get::<bool>("tests_passing").unwrap_or(false);
                    if tests_passing != *passing {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl HookManager {
    /// Create a new hook manager.
    pub fn new(config: HooksConfig) -> Self {
        let manager = Self {
            config: config.clone(),
            hooks: RwLock::new(HashMap::new()),
            implementations: RwLock::new(HashMap::new()),
        };

        manager
    }

    /// Initialize with built-in hooks.
    pub async fn initialize(&self) -> LoopResult<()> {
        for builtin in &self.config.builtin_hooks {
            self.register_builtin(*builtin).await?;
        }

        // Load external hooks from directory
        if let Some(hooks_dir) = &self.config.hooks_dir {
            self.load_external_hooks(hooks_dir).await?;
        }

        Ok(())
    }

    /// Register a builtin hook.
    async fn register_builtin(&self, builtin: BuiltinHook) -> LoopResult<()> {
        let hook: Arc<dyn Hook> = match builtin {
            BuiltinHook::Logging => Arc::new(LoggingHook),
            BuiltinHook::Git => Arc::new(GitHook),
            BuiltinHook::Backup => Arc::new(BackupHook),
            BuiltinHook::Metrics => Arc::new(MetricsHook),
            BuiltinHook::Notification => Arc::new(NotificationHook),
        };

        let definition = HookDefinition {
            name: hook.name().to_string(),
            points: builtin.default_points(),
            priority: builtin.default_priority(),
            timeout: None,
            enabled: true,
            hook_type: HookType::Builtin { builtin },
            conditions: vec![],
        };

        self.register(definition, hook).await
    }

    /// Register a hook.
    pub async fn register(&self, definition: HookDefinition, implementation: Arc<dyn Hook>) -> LoopResult<()> {
        let name = definition.name.clone();
        let points = definition.points.clone();

        // Store implementation
        self.implementations.write().await.insert(name.clone(), implementation.clone());

        // Register at each hook point
        let mut hooks = self.hooks.write().await;
        for point in points {
            let registered = RegisteredHook {
                definition: definition.clone(),
                implementation: implementation.clone(),
            };

            hooks
                .entry(point)
                .or_insert_with(Vec::new)
                .push(registered);

            // Sort by priority
            if let Some(hooks_at_point) = hooks.get_mut(&point) {
                hooks_at_point.sort_by(|a, b| b.definition.priority.cmp(&a.definition.priority));
            }
        }

        info!("Registered hook: {}", name);
        Ok(())
    }

    /// Unregister a hook.
    pub async fn unregister(&self, name: &str) -> LoopResult<()> {
        self.implementations.write().await.remove(name);

        let mut hooks = self.hooks.write().await;
        for hooks_at_point in hooks.values_mut() {
            hooks_at_point.retain(|h| h.definition.name != name);
        }

        info!("Unregistered hook: {}", name);
        Ok(())
    }

    /// Execute hooks at a point.
    pub async fn execute(&self, point: HookPoint, context: &mut HookContext) -> LoopResult<Vec<HookResult>> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let hooks = self.hooks.read().await;
        let hooks_at_point = match hooks.get(&point) {
            Some(h) => h.clone(),
            None => return Ok(vec![]),
        };
        drop(hooks); // Release lock before executing

        let mut results = Vec::new();

        for registered in hooks_at_point {
            if !registered.definition.enabled {
                continue;
            }

            // Check conditions
            if !registered.implementation.should_run(context, &registered.definition.conditions) {
                continue;
            }

            // Execute with timeout
            let timeout = registered.definition.timeout.unwrap_or(self.config.default_timeout);
            let result = self.execute_with_timeout(&registered, context, timeout).await;

            match &result {
                Ok(r) if !r.success && self.config.fail_on_hook_error => {
                    error!("Hook {} failed: {:?}", r.hook_name, r.error);
                    return Err(LoopError::HookFailed {
                        name: r.hook_name.clone(),
                        error: r.error.clone().unwrap_or_default(),
                    });
                }
                Ok(r) => {
                    // Merge modified data back to context
                    for (k, v) in &r.modified_data {
                        context.data.insert(k.clone(), v.clone());
                    }

                    if !r.continue_loop {
                        warn!("Hook {} requested loop stop", r.hook_name);
                    }

                    results.push(r.clone());
                }
                Err(e) => {
                    warn!("Hook execution error: {}", e);
                    if self.config.fail_on_hook_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Execute hook with timeout.
    async fn execute_with_timeout(
        &self,
        registered: &RegisteredHook,
        context: &HookContext,
        timeout: std::time::Duration,
    ) -> LoopResult<HookResult> {
        let start = std::time::Instant::now();

        let result = tokio::time::timeout(timeout, registered.implementation.execute(context)).await;

        match result {
            Ok(Ok(mut hook_result)) => {
                hook_result.duration_ms = start.elapsed().as_millis() as u64;
                Ok(hook_result)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("Hook {} timed out after {:?}", registered.definition.name, timeout);
                Ok(HookResult {
                    hook_name: registered.definition.name.clone(),
                    success: false,
                    duration_ms: timeout.as_millis() as u64,
                    output: None,
                    error: Some(format!("Timed out after {:?}", timeout)),
                    continue_loop: true,
                    modified_data: HashMap::new(),
                })
            }
        }
    }

    /// Load external hooks from directory.
    async fn load_external_hooks(&self, dir: &std::path::Path) -> LoopResult<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                // Load hook definition from YAML
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(definition) = serde_yaml::from_str::<HookDefinition>(&content) {
                        let hook: Arc<dyn Hook> = match &definition.hook_type {
                            HookType::Script { path, args } => {
                                Arc::new(ScriptHook::new(path.clone(), args.clone()))
                            }
                            HookType::Command { command } => {
                                Arc::new(CommandHook::new(command.clone()))
                            }
                            HookType::Webhook { url, method } => {
                                Arc::new(WebhookHook::new(url.clone(), method.clone()))
                            }
                            _ => continue,
                        };

                        self.register(definition, hook).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl BuiltinHook {
    fn default_points(&self) -> Vec<HookPoint> {
        match self {
            Self::Logging => vec![
                HookPoint::LoopStart,
                HookPoint::LoopEnd,
                HookPoint::PreIteration,
                HookPoint::PostIteration,
            ],
            Self::Git => vec![HookPoint::PostIteration, HookPoint::LoopEnd],
            Self::Backup => vec![HookPoint::PreIteration],
            Self::Metrics => vec![HookPoint::PostIteration],
            Self::Notification => vec![
                HookPoint::LoopEnd,
                HookPoint::OnError,
                HookPoint::OnTestFailure,
            ],
        }
    }

    fn default_priority(&self) -> i32 {
        match self {
            Self::Logging => 100,
            Self::Metrics => 90,
            Self::Backup => 80,
            Self::Git => 50,
            Self::Notification => 10,
        }
    }
}

// Built-in hook implementations
struct LoggingHook;
struct GitHook;
struct BackupHook;
struct MetricsHook;
struct NotificationHook;
struct ScriptHook { path: std::path::PathBuf, args: Vec<String> }
struct CommandHook { command: String }
struct WebhookHook { url: String, method: String }

impl ScriptHook {
    fn new(path: std::path::PathBuf, args: Vec<String>) -> Self {
        Self { path, args }
    }
}

impl CommandHook {
    fn new(command: String) -> Self {
        Self { command }
    }
}

impl WebhookHook {
    fn new(url: String, method: String) -> Self {
        Self { url, method }
    }
}

#[async_trait::async_trait]
impl Hook for LoggingHook {
    fn name(&self) -> &str { "logging" }

    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult> {
        info!("Hook point {:?} at iteration {}", context.hook_point, context.iteration);
        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: true,
            duration_ms: 0,
            output: None,
            error: None,
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl Hook for GitHook {
    fn name(&self) -> &str { "git" }

    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult> {
        // Auto-commit changes
        let output = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&context.working_dir)
            .output()
            .await;

        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: output.is_ok(),
            duration_ms: 0,
            output: None,
            error: output.err().map(|e| e.to_string()),
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl Hook for BackupHook {
    fn name(&self) -> &str { "backup" }

    async fn execute(&self, _context: &HookContext) -> LoopResult<HookResult> {
        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: true,
            duration_ms: 0,
            output: None,
            error: None,
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl Hook for MetricsHook {
    fn name(&self) -> &str { "metrics" }

    async fn execute(&self, _context: &HookContext) -> LoopResult<HookResult> {
        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: true,
            duration_ms: 0,
            output: None,
            error: None,
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl Hook for NotificationHook {
    fn name(&self) -> &str { "notification" }

    async fn execute(&self, _context: &HookContext) -> LoopResult<HookResult> {
        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: true,
            duration_ms: 0,
            output: None,
            error: None,
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}

#[async_trait::async_trait]
impl Hook for ScriptHook {
    fn name(&self) -> &str { "script" }

    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult> {
        let output = tokio::process::Command::new(&self.path)
            .args(&self.args)
            .current_dir(&context.working_dir)
            .output()
            .await;

        match output {
            Ok(out) => Ok(HookResult {
                hook_name: self.name().to_string(),
                success: out.status.success(),
                duration_ms: 0,
                output: Some(String::from_utf8_lossy(&out.stdout).to_string()),
                error: if out.status.success() { None } else {
                    Some(String::from_utf8_lossy(&out.stderr).to_string())
                },
                continue_loop: true,
                modified_data: HashMap::new(),
            }),
            Err(e) => Ok(HookResult {
                hook_name: self.name().to_string(),
                success: false,
                duration_ms: 0,
                output: None,
                error: Some(e.to_string()),
                continue_loop: true,
                modified_data: HashMap::new(),
            }),
        }
    }
}

#[async_trait::async_trait]
impl Hook for CommandHook {
    fn name(&self) -> &str { "command" }

    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult> {
        let output = tokio::process::Command::new("sh")
            .args(["-c", &self.command])
            .current_dir(&context.working_dir)
            .output()
            .await;

        match output {
            Ok(out) => Ok(HookResult {
                hook_name: self.name().to_string(),
                success: out.status.success(),
                duration_ms: 0,
                output: Some(String::from_utf8_lossy(&out.stdout).to_string()),
                error: None,
                continue_loop: true,
                modified_data: HashMap::new(),
            }),
            Err(e) => Ok(HookResult {
                hook_name: self.name().to_string(),
                success: false,
                duration_ms: 0,
                output: None,
                error: Some(e.to_string()),
                continue_loop: true,
                modified_data: HashMap::new(),
            }),
        }
    }
}

#[async_trait::async_trait]
impl Hook for WebhookHook {
    fn name(&self) -> &str { "webhook" }

    async fn execute(&self, context: &HookContext) -> LoopResult<HookResult> {
        let client = reqwest::Client::new();
        let payload = serde_json::to_value(context).ok();

        let request = match self.method.to_uppercase().as_str() {
            "POST" => client.post(&self.url).json(&payload),
            "PUT" => client.put(&self.url).json(&payload),
            _ => client.get(&self.url),
        };

        let result = request.send().await;

        Ok(HookResult {
            hook_name: self.name().to_string(),
            success: result.is_ok(),
            duration_ms: 0,
            output: None,
            error: result.err().map(|e| e.to_string()),
            continue_loop: true,
            modified_data: HashMap::new(),
        })
    }
}
```

### 3. Module Root (src/hooks/mod.rs)

```rust
//! Hook system for loop extension points.

pub mod manager;
pub mod types;

pub use manager::{Hook, HookManager};
pub use types::{
    BuiltinHook, HookCondition, HookContext, HookDefinition, HookPoint,
    HookResult, HookType, HooksConfig,
};
```

---

## Testing Requirements

1. Hooks register at correct points
2. Priority ordering is respected
3. Timeout terminates slow hooks
4. Conditions filter execution
5. Hook errors are handled correctly
6. External scripts execute
7. Webhooks are called
8. Context data flows between hooks

---

## Related Specs

- Depends on: [096-loop-runner-core.md](096-loop-runner-core.md)
- Next: [114-loop-notifications.md](114-loop-notifications.md)
- Related: [103-auto-reboot.md](103-auto-reboot.md)
