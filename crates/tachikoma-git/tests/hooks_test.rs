//! Tests for Git hooks support.

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tachikoma_git::{GitRepository, HookType};

#[test]
fn test_list_hooks() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize a git repo
    let repo = GitRepository::init(repo_path, false).unwrap();
    
    // List hooks - should be empty initially
    let hooks = repo.list_hooks().unwrap();
    assert_eq!(hooks.len(), 11); // All hook types
    
    // All should be uninstalled
    for hook in &hooks {
        assert!(!hook.installed);
        assert!(!hook.executable);
        assert!(hook.content.is_none());
    }
}

#[test]
fn test_install_uninstall_hook() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize a git repo
    let repo = GitRepository::init(repo_path, false).unwrap();
    
    let script = "#!/bin/sh\necho 'pre-commit hook'";
    
    // Install hook
    repo.install_hook(HookType::PreCommit, script).unwrap();
    
    // Check hook is installed
    let hook = repo.get_hook(HookType::PreCommit).unwrap();
    assert!(hook.installed);
    assert!(hook.executable);
    assert!(hook.content.is_some());
    assert_eq!(hook.content.unwrap(), script);
    
    // Uninstall hook
    repo.uninstall_hook(HookType::PreCommit).unwrap();
    
    // Check hook is uninstalled
    let hook = repo.get_hook(HookType::PreCommit).unwrap();
    assert!(!hook.installed);
    assert!(!hook.executable);
    assert!(hook.content.is_none());
}

#[test]
fn test_hook_execution() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize a git repo
    let repo = GitRepository::init(repo_path, false).unwrap();
    
    // Install a simple echo hook
    let script = "#!/bin/sh\necho 'Hello from hook'";
    repo.install_hook(HookType::PreCommit, script).unwrap();
    
    // Run the hook
    let result = repo.run_hook(HookType::PreCommit, &[]).unwrap();
    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "Hello from hook");
}

#[test]
fn test_missing_hook_execution() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // Initialize a git repo
    let repo = GitRepository::init(repo_path, false).unwrap();
    
    // Run non-existent hook - should succeed
    let result = repo.run_hook(HookType::PreCommit, &[]).unwrap();
    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
}

#[test]
fn test_hook_types() {
    // Test all hook types have correct filenames
    assert_eq!(HookType::PreCommit.filename(), "pre-commit");
    assert_eq!(HookType::PrepareCommitMsg.filename(), "prepare-commit-msg");
    assert_eq!(HookType::CommitMsg.filename(), "commit-msg");
    assert_eq!(HookType::PostCommit.filename(), "post-commit");
    assert_eq!(HookType::PreMerge.filename(), "pre-merge-commit");
    assert_eq!(HookType::PrePush.filename(), "pre-push");
    assert_eq!(HookType::PreRebase.filename(), "pre-rebase");
    assert_eq!(HookType::PostCheckout.filename(), "post-checkout");
    assert_eq!(HookType::PostMerge.filename(), "post-merge");
    assert_eq!(HookType::ApplypatchMsg.filename(), "applypatch-msg");
    assert_eq!(HookType::PostApplypatch.filename(), "post-applypatch");
    
    // Test all() returns all types
    assert_eq!(HookType::all().len(), 11);
}

#[test]
fn test_hook_scripts() {
    use tachikoma_git::hooks::scripts;
    
    // Test pre-built scripts exist and contain expected content
    assert!(scripts::PRE_COMMIT_TESTS.contains("cargo test"));
    assert!(scripts::PRE_COMMIT_FORMAT.contains("cargo fmt"));
    assert!(scripts::COMMIT_MSG_VALIDATE.contains("minimum 10 characters"));
}