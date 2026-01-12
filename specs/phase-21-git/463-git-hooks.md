# 463 - Git Hooks

**Phase:** 21 - Git Integration
**Spec ID:** 463
**Status:** Planned
**Dependencies:** 452-git-detect
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement Git hooks management, enabling installation, execution, and management of Git hooks.

---

## Acceptance Criteria

- [ ] List available hooks
- [ ] Install/uninstall hooks
- [ ] Hook execution framework
- [ ] Pre-commit hook support
- [ ] Custom hook scripts

---

## Implementation Details

### 1. Hook Types (src/hooks.rs)

```rust
//! Git hooks management.

use crate::{GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Git hook types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookType {
    /// Before commit message is created.
    PreCommit,
    /// Edit commit message.
    PrepareCommitMsg,
    /// Validate commit message.
    CommitMsg,
    /// After commit is created.
    PostCommit,
    /// Before merge.
    PreMerge,
    /// Before push.
    PrePush,
    /// Before rebase.
    PreRebase,
    /// After checkout.
    PostCheckout,
    /// After merge.
    PostMerge,
    /// Before applying patch.
    ApplypatchMsg,
    /// After applying patch.
    PostApplypatch,
}

impl HookType {
    /// Get the hook filename.
    pub fn filename(&self) -> &'static str {
        match self {
            Self::PreCommit => "pre-commit",
            Self::PrepareCommitMsg => "prepare-commit-msg",
            Self::CommitMsg => "commit-msg",
            Self::PostCommit => "post-commit",
            Self::PreMerge => "pre-merge-commit",
            Self::PrePush => "pre-push",
            Self::PreRebase => "pre-rebase",
            Self::PostCheckout => "post-checkout",
            Self::PostMerge => "post-merge",
            Self::ApplypatchMsg => "applypatch-msg",
            Self::PostApplypatch => "post-applypatch",
        }
    }

    /// Get all hook types.
    pub fn all() -> &'static [HookType] {
        &[
            Self::PreCommit,
            Self::PrepareCommitMsg,
            Self::CommitMsg,
            Self::PostCommit,
            Self::PreMerge,
            Self::PrePush,
            Self::PreRebase,
            Self::PostCheckout,
            Self::PostMerge,
            Self::ApplypatchMsg,
            Self::PostApplypatch,
        ]
    }
}

/// Hook information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInfo {
    /// Hook type.
    pub hook_type: HookType,
    /// Path to hook script.
    pub path: PathBuf,
    /// Is hook installed (executable script exists).
    pub installed: bool,
    /// Is hook executable.
    pub executable: bool,
    /// Hook script content (if readable).
    pub content: Option<String>,
}

/// Hook execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Hook type.
    pub hook_type: HookType,
    /// Exit code.
    pub exit_code: i32,
    /// Stdout.
    pub stdout: String,
    /// Stderr.
    pub stderr: String,
    /// Hook was successful.
    pub success: bool,
}

impl GitRepository {
    /// Get hooks directory path.
    pub fn hooks_path(&self) -> PathBuf {
        self.git_dir().join("hooks")
    }

    /// List all hooks.
    pub fn list_hooks(&self) -> GitResult<Vec<HookInfo>> {
        let hooks_dir = self.hooks_path();
        let mut hooks = Vec::new();

        for hook_type in HookType::all() {
            let path = hooks_dir.join(hook_type.filename());
            let installed = path.exists();
            let executable = installed && is_executable(&path);
            let content = if installed {
                std::fs::read_to_string(&path).ok()
            } else {
                None
            };

            hooks.push(HookInfo {
                hook_type: *hook_type,
                path,
                installed,
                executable,
                content,
            });
        }

        Ok(hooks)
    }

    /// Get a specific hook.
    pub fn get_hook(&self, hook_type: HookType) -> GitResult<HookInfo> {
        let path = self.hooks_path().join(hook_type.filename());
        let installed = path.exists();
        let executable = installed && is_executable(&path);
        let content = if installed {
            std::fs::read_to_string(&path).ok()
        } else {
            None
        };

        Ok(HookInfo {
            hook_type,
            path,
            installed,
            executable,
            content,
        })
    }

    /// Install a hook script.
    pub fn install_hook(&self, hook_type: HookType, script: &str) -> GitResult<()> {
        let hooks_dir = self.hooks_path();
        std::fs::create_dir_all(&hooks_dir)?;

        let path = hooks_dir.join(hook_type.filename());
        std::fs::write(&path, script)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Uninstall a hook.
    pub fn uninstall_hook(&self, hook_type: HookType) -> GitResult<()> {
        let path = self.hooks_path().join(hook_type.filename());
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Execute a hook.
    pub fn run_hook(&self, hook_type: HookType, args: &[&str]) -> GitResult<HookResult> {
        let hook_path = self.hooks_path().join(hook_type.filename());

        if !hook_path.exists() {
            return Ok(HookResult {
                hook_type,
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                success: true, // No hook means success
            });
        }

        if !is_executable(&hook_path) {
            return Err(GitError::InvalidOperation {
                message: format!("Hook {} is not executable", hook_type.filename()),
            });
        }

        let output = Command::new(&hook_path)
            .args(args)
            .current_dir(self.root_path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let exit_code = output.status.code().unwrap_or(-1);

        Ok(HookResult {
            hook_type,
            exit_code,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        })
    }

    /// Check if a hook exists and is executable.
    pub fn has_hook(&self, hook_type: HookType) -> bool {
        let path = self.hooks_path().join(hook_type.filename());
        path.exists() && is_executable(&path)
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.exists()
}

/// Pre-built hook scripts.
pub mod scripts {
    /// Pre-commit hook that runs tests.
    pub const PRE_COMMIT_TESTS: &str = r#"#!/bin/sh
# Run tests before commit
cargo test --quiet
"#;

    /// Pre-commit hook that checks formatting.
    pub const PRE_COMMIT_FORMAT: &str = r#"#!/bin/sh
# Check formatting before commit
cargo fmt --check
"#;

    /// Commit message hook that validates format.
    pub const COMMIT_MSG_VALIDATE: &str = r#"#!/bin/sh
# Validate commit message format
MSG_FILE=$1
MSG=$(cat "$MSG_FILE")

# Check for minimum length
if [ ${#MSG} -lt 10 ]; then
    echo "Error: Commit message too short (minimum 10 characters)"
    exit 1
fi

exit 0
"#;
}
```

---

## Testing Requirements

1. Hook listing is complete
2. Hook installation creates executable
3. Hook execution captures output
4. Missing hooks return success
5. Non-executable hooks error

---

## Related Specs

- Depends on: [452-git-detect.md](452-git-detect.md)
- Next: [464-git-credentials.md](464-git-credentials.md)
