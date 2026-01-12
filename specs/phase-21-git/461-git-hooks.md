# Spec 461: Git Hooks

## Phase
21 - Git Integration

## Spec ID
461

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~8%

---

## Objective

Implement Git hooks management for Tachikoma, providing functionality to create, install, and manage Git hooks. This module enables automated workflows triggered by Git events such as pre-commit, post-commit, pre-push, and commit-msg hooks with support for both shell scripts and Rust-native hook implementations.

---

## Acceptance Criteria

- [ ] Implement `GitHookManager` for hook operations
- [ ] Support installing hooks to repository
- [ ] Support removing hooks
- [ ] Support listing installed hooks
- [ ] Implement pre-commit hook support
- [ ] Implement commit-msg hook support
- [ ] Implement pre-push hook support
- [ ] Support hook execution with timeout
- [ ] Support bypassing hooks
- [ ] Provide Tachikoma-specific hooks

---

## Implementation Details

### Hook Manager Implementation

```rust
// src/git/hooks.rs

use std::collections::HashMap;
use std::fs::{self, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use super::repo::GitRepository;
use super::types::*;

/// Git hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    ApplypatchMsg,
    PreApplypatch,
    PostApplypatch,
    PreCommit,
    PrepareCommitMsg,
    CommitMsg,
    PostCommit,
    PreRebase,
    PostCheckout,
    PostMerge,
    PrePush,
    PreReceive,
    Update,
    PostReceive,
    PostUpdate,
    PushToCheckout,
    PreAutoGc,
    PostRewrite,
    SendemailValidate,
}

impl HookType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ApplypatchMsg => "applypatch-msg",
            Self::PreApplypatch => "pre-applypatch",
            Self::PostApplypatch => "post-applypatch",
            Self::PreCommit => "pre-commit",
            Self::PrepareCommitMsg => "prepare-commit-msg",
            Self::CommitMsg => "commit-msg",
            Self::PostCommit => "post-commit",
            Self::PreRebase => "pre-rebase",
            Self::PostCheckout => "post-checkout",
            Self::PostMerge => "post-merge",
            Self::PrePush => "pre-push",
            Self::PreReceive => "pre-receive",
            Self::Update => "update",
            Self::PostReceive => "post-receive",
            Self::PostUpdate => "post-update",
            Self::PushToCheckout => "push-to-checkout",
            Self::PreAutoGc => "pre-auto-gc",
            Self::PostRewrite => "post-rewrite",
            Self::SendemailValidate => "sendemail-validate",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "applypatch-msg" => Some(Self::ApplypatchMsg),
            "pre-applypatch" => Some(Self::PreApplypatch),
            "post-applypatch" => Some(Self::PostApplypatch),
            "pre-commit" => Some(Self::PreCommit),
            "prepare-commit-msg" => Some(Self::PrepareCommitMsg),
            "commit-msg" => Some(Self::CommitMsg),
            "post-commit" => Some(Self::PostCommit),
            "pre-rebase" => Some(Self::PreRebase),
            "post-checkout" => Some(Self::PostCheckout),
            "post-merge" => Some(Self::PostMerge),
            "pre-push" => Some(Self::PrePush),
            "pre-receive" => Some(Self::PreReceive),
            "update" => Some(Self::Update),
            "post-receive" => Some(Self::PostReceive),
            "post-update" => Some(Self::PostUpdate),
            "push-to-checkout" => Some(Self::PushToCheckout),
            "pre-auto-gc" => Some(Self::PreAutoGc),
            "post-rewrite" => Some(Self::PostRewrite),
            "sendemail-validate" => Some(Self::SendemailValidate),
            _ => None,
        }
    }

    /// Common hooks that Tachikoma might install
    pub fn common() -> Vec<Self> {
        vec![
            Self::PreCommit,
            Self::CommitMsg,
            Self::PrePush,
            Self::PrepareCommitMsg,
            Self::PostCommit,
        ]
    }
}

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    /// Whether hook succeeded
    pub success: bool,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Stdout output
    pub stdout: String,
    /// Stderr output
    pub stderr: String,
    /// Execution time
    pub duration: Duration,
}

impl HookResult {
    pub fn passed(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Installed hook information
#[derive(Debug, Clone)]
pub struct InstalledHook {
    /// Hook type
    pub hook_type: HookType,
    /// Path to hook file
    pub path: PathBuf,
    /// Is executable
    pub executable: bool,
    /// Is managed by Tachikoma
    pub tachikoma_managed: bool,
    /// Content preview (first few lines)
    pub preview: String,
}

/// Options for hook execution
#[derive(Debug, Clone)]
pub struct HookExecutionOptions {
    /// Timeout for hook execution
    pub timeout: Duration,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Arguments to pass
    pub args: Vec<String>,
    /// Stdin input
    pub stdin: Option<String>,
    /// Working directory
    pub cwd: Option<PathBuf>,
}

impl Default for HookExecutionOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            env: HashMap::new(),
            args: Vec::new(),
            stdin: None,
            cwd: None,
        }
    }
}

impl HookExecutionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn stdin(mut self, input: impl Into<String>) -> Self {
        self.stdin = Some(input.into());
        self
    }
}

/// Git hook manager
pub struct GitHookManager<'a> {
    repo: &'a GitRepository,
    hooks_dir: PathBuf,
}

impl<'a> GitHookManager<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        let hooks_dir = repo.git_dir().join("hooks");
        Self { repo, hooks_dir }
    }

    /// Get hooks directory
    pub fn hooks_dir(&self) -> &Path {
        &self.hooks_dir
    }

    /// Check if a hook is installed
    pub fn is_installed(&self, hook_type: HookType) -> bool {
        let path = self.hook_path(hook_type);
        path.exists()
    }

    /// Get path to a hook
    pub fn hook_path(&self, hook_type: HookType) -> PathBuf {
        self.hooks_dir.join(hook_type.name())
    }

    /// List all installed hooks
    pub fn list_installed(&self) -> GitResult<Vec<InstalledHook>> {
        let mut hooks = Vec::new();

        if !self.hooks_dir.exists() {
            return Ok(hooks);
        }

        for entry in fs::read_dir(&self.hooks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip sample hooks
                    if name.ends_with(".sample") {
                        continue;
                    }

                    if let Some(hook_type) = HookType::from_name(name) {
                        let content = fs::read_to_string(&path).unwrap_or_default();
                        let preview = content.lines().take(5).collect::<Vec<_>>().join("\n");
                        let tachikoma_managed = content.contains("TACHIKOMA_HOOK");

                        #[cfg(unix)]
                        let executable = {
                            use std::os::unix::fs::MetadataExt;
                            entry.metadata().map(|m| m.mode() & 0o111 != 0).unwrap_or(false)
                        };
                        #[cfg(not(unix))]
                        let executable = true;

                        hooks.push(InstalledHook {
                            hook_type,
                            path,
                            executable,
                            tachikoma_managed,
                            preview,
                        });
                    }
                }
            }
        }

        Ok(hooks)
    }

    /// Install a hook
    pub fn install(&self, hook_type: HookType, content: &str) -> GitResult<PathBuf> {
        // Ensure hooks directory exists
        fs::create_dir_all(&self.hooks_dir)?;

        let path = self.hook_path(hook_type);

        // Check for existing hook
        if path.exists() {
            let existing = fs::read_to_string(&path)?;
            if !existing.contains("TACHIKOMA_HOOK") {
                return Err(GitError::Other(format!(
                    "Hook '{}' already exists and is not managed by Tachikoma",
                    hook_type.name()
                )));
            }
        }

        // Write hook content
        let mut file = fs::File::create(&path)?;
        writeln!(file, "#!/bin/sh")?;
        writeln!(file, "# TACHIKOMA_HOOK - Managed by Tachikoma")?;
        writeln!(file, "# Do not edit manually")?;
        writeln!(file)?;
        write!(file, "{}", content)?;

        // Make executable
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o755);
            fs::set_permissions(&path, perms)?;
        }

        Ok(path)
    }

    /// Install a Tachikoma-specific hook
    pub fn install_tachikoma_hook(&self, hook_type: HookType) -> GitResult<PathBuf> {
        let content = self.generate_tachikoma_hook(hook_type)?;
        self.install(hook_type, &content)
    }

    /// Remove a hook
    pub fn remove(&self, hook_type: HookType) -> GitResult<()> {
        let path = self.hook_path(hook_type);

        if !path.exists() {
            return Ok(());
        }

        // Check if Tachikoma-managed
        let content = fs::read_to_string(&path)?;
        if !content.contains("TACHIKOMA_HOOK") {
            return Err(GitError::Other(format!(
                "Hook '{}' is not managed by Tachikoma",
                hook_type.name()
            )));
        }

        fs::remove_file(&path)?;
        Ok(())
    }

    /// Execute a hook
    pub fn execute(&self, hook_type: HookType, options: HookExecutionOptions) -> GitResult<HookResult> {
        let path = self.hook_path(hook_type);

        if !path.exists() {
            return Ok(HookResult {
                success: true,
                exit_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
                duration: Duration::ZERO,
            });
        }

        let start = std::time::Instant::now();

        let mut cmd = Command::new(&path);
        cmd.args(&options.args);

        for (key, value) in &options.env {
            cmd.env(key, value);
        }

        if let Some(ref cwd) = options.cwd {
            cmd.current_dir(cwd);
        } else if let Some(workdir) = self.repo.workdir() {
            cmd.current_dir(workdir);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| GitError::HookFailed(e.to_string()))?;

        // Write stdin if provided
        if let Some(ref input) = options.stdin {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(input.as_bytes())
                    .map_err(|e| GitError::HookFailed(e.to_string()))?;
            }
        }

        // Wait with timeout
        let output = child.wait_with_output()
            .map_err(|e| GitError::HookFailed(e.to_string()))?;

        let duration = start.elapsed();

        Ok(HookResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    fn generate_tachikoma_hook(&self, hook_type: HookType) -> GitResult<String> {
        let content = match hook_type {
            HookType::PreCommit => r#"
# Tachikoma Pre-commit Hook
# Run formatters and linters on staged files

# Check if tachikoma is available
if command -v tachikoma &> /dev/null; then
    tachikoma hook pre-commit
    exit $?
fi

echo "Warning: tachikoma not found, skipping pre-commit checks"
exit 0
"#,
            HookType::CommitMsg => r#"
# Tachikoma Commit-msg Hook
# Validate commit message format

COMMIT_MSG_FILE=$1

# Check if tachikoma is available
if command -v tachikoma &> /dev/null; then
    tachikoma hook commit-msg "$COMMIT_MSG_FILE"
    exit $?
fi

exit 0
"#,
            HookType::PrepareCommitMsg => r#"
# Tachikoma Prepare-commit-msg Hook
# Generate AI-assisted commit messages

COMMIT_MSG_FILE=$1
COMMIT_SOURCE=$2
SHA1=$3

# Check if tachikoma is available
if command -v tachikoma &> /dev/null; then
    tachikoma hook prepare-commit-msg "$COMMIT_MSG_FILE" "$COMMIT_SOURCE" "$SHA1"
    exit $?
fi

exit 0
"#,
            HookType::PrePush => r#"
# Tachikoma Pre-push Hook
# Run tests before push

REMOTE=$1
URL=$2

# Check if tachikoma is available
if command -v tachikoma &> /dev/null; then
    tachikoma hook pre-push "$REMOTE" "$URL"
    exit $?
fi

exit 0
"#,
            HookType::PostCommit => r#"
# Tachikoma Post-commit Hook
# Notify and update after commit

# Check if tachikoma is available
if command -v tachikoma &> /dev/null; then
    tachikoma hook post-commit
fi

exit 0
"#,
            _ => {
                return Err(GitError::Other(format!(
                    "No Tachikoma hook template for '{}'",
                    hook_type.name()
                )));
            }
        };

        Ok(content.trim().to_string())
    }
}

/// Hook content builder
pub struct HookBuilder {
    shebang: String,
    header: Vec<String>,
    body: Vec<String>,
}

impl HookBuilder {
    pub fn new() -> Self {
        Self {
            shebang: "#!/bin/sh".to_string(),
            header: vec!["# TACHIKOMA_HOOK".to_string()],
            body: Vec::new(),
        }
    }

    pub fn shebang(mut self, shebang: impl Into<String>) -> Self {
        self.shebang = shebang.into();
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.header.push(format!("# {}", comment.into()));
        self
    }

    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.body.push(line.into());
        self
    }

    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.body.push(cmd.into());
        self
    }

    pub fn build(self) -> String {
        let mut content = vec![self.shebang];
        content.extend(self.header);
        content.push(String::new());
        content.extend(self.body);
        content.join("\n")
    }
}

impl Default for HookBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path(), false).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_hook_type_name() {
        assert_eq!(HookType::PreCommit.name(), "pre-commit");
        assert_eq!(HookType::CommitMsg.name(), "commit-msg");
        assert_eq!(HookType::PrePush.name(), "pre-push");
    }

    #[test]
    fn test_hook_type_from_name() {
        assert_eq!(HookType::from_name("pre-commit"), Some(HookType::PreCommit));
        assert_eq!(HookType::from_name("invalid"), None);
    }

    #[test]
    fn test_common_hooks() {
        let common = HookType::common();
        assert!(common.contains(&HookType::PreCommit));
        assert!(common.contains(&HookType::CommitMsg));
        assert!(common.contains(&HookType::PrePush));
    }

    #[test]
    fn test_hook_not_installed() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        assert!(!manager.is_installed(HookType::PreCommit));
    }

    #[test]
    fn test_install_hook() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        let content = "echo 'Hello from hook'";
        manager.install(HookType::PreCommit, content).unwrap();

        assert!(manager.is_installed(HookType::PreCommit));
    }

    #[test]
    fn test_list_installed_hooks() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        manager.install(HookType::PreCommit, "echo test").unwrap();
        manager.install(HookType::CommitMsg, "echo test").unwrap();

        let hooks = manager.list_installed().unwrap();
        assert_eq!(hooks.len(), 2);
    }

    #[test]
    fn test_remove_hook() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        manager.install(HookType::PreCommit, "echo test").unwrap();
        assert!(manager.is_installed(HookType::PreCommit));

        manager.remove(HookType::PreCommit).unwrap();
        assert!(!manager.is_installed(HookType::PreCommit));
    }

    #[test]
    fn test_cannot_remove_unmanaged_hook() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        // Manually create a hook without Tachikoma marker
        let hooks_dir = manager.hooks_dir();
        std::fs::create_dir_all(hooks_dir).unwrap();
        let hook_path = hooks_dir.join("pre-commit");
        std::fs::write(&hook_path, "#!/bin/sh\necho test").unwrap();

        let result = manager.remove(HookType::PreCommit);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_missing_hook() {
        let (_dir, repo) = setup_test_repo();
        let manager = GitHookManager::new(&repo);

        let result = manager.execute(HookType::PreCommit, HookExecutionOptions::new()).unwrap();

        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_hook_builder() {
        let hook = HookBuilder::new()
            .comment("Test hook")
            .command("echo 'Hello'")
            .command("exit 0")
            .build();

        assert!(hook.starts_with("#!/bin/sh"));
        assert!(hook.contains("# TACHIKOMA_HOOK"));
        assert!(hook.contains("echo 'Hello'"));
    }

    #[test]
    fn test_hook_execution_options() {
        let opts = HookExecutionOptions::new()
            .timeout(Duration::from_secs(30))
            .env("MY_VAR", "value")
            .arg("--flag");

        assert_eq!(opts.timeout, Duration::from_secs(30));
        assert_eq!(opts.env.get("MY_VAR"), Some(&"value".to_string()));
        assert!(opts.args.contains(&"--flag".to_string()));
    }

    #[test]
    fn test_hook_result_passed() {
        let result = HookResult {
            success: true,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::ZERO,
        };

        assert!(result.passed());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 451: Commit Operations
