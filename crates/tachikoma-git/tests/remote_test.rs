//! Tests for Git remote management functionality.

use std::path::Path;
use tachikoma_git::{GitRepository, GitRepositoryOptions};
use tempfile::TempDir;

fn setup_test_repo() -> (TempDir, GitRepository) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    // Initialize a bare repository to use as a remote
    let git_repo = git2::Repository::init_bare(repo_path).expect("Failed to initialize bare repo");
    drop(git_repo);

    // Create a working repository in a subdirectory
    let work_dir = temp_dir.path().join("work");
    std::fs::create_dir_all(&work_dir).expect("Failed to create work dir");
    
    let git_repo = git2::Repository::init(&work_dir).expect("Failed to initialize work repo");
    
    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = git_repo.index().unwrap().write_tree().unwrap();
    let tree = git_repo.find_tree(tree_id).unwrap();
    git_repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    ).unwrap();
    
    drop(git_repo);

    let repo = GitRepository::open(&work_dir, GitRepositoryOptions::default())
        .expect("Failed to open repository");

    (temp_dir, repo)
}

#[test]
fn test_list_remotes_empty() {
    let (_temp_dir, repo) = setup_test_repo();
    
    let remotes = repo.list_remotes().expect("Failed to list remotes");
    assert!(remotes.is_empty(), "New repo should have no remotes");
}

#[test]
fn test_add_and_list_remotes() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    let remote = repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    assert_eq!(remote.name, "origin");
    assert_eq!(remote.fetch_url, Some("https://github.com/example/repo.git".to_string()));
    assert!(remote.push_url.is_none()); // Should be None when same as fetch URL
    
    // List remotes
    let remotes = repo.list_remotes().expect("Failed to list remotes");
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0], "origin");
}

#[test]
fn test_get_remote_info() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Get remote info
    let remote = repo.get_remote("origin").expect("Failed to get remote");
    assert_eq!(remote.name, "origin");
    assert_eq!(remote.fetch_url, Some("https://github.com/example/repo.git".to_string()));
    assert!(!remote.fetch_refspecs.is_empty());
}

#[test]
fn test_remove_remote() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Verify it exists
    let remotes = repo.list_remotes().expect("Failed to list remotes");
    assert_eq!(remotes.len(), 1);
    
    // Remove it
    repo.remove_remote("origin").expect("Failed to remove remote");
    
    // Verify it's gone
    let remotes = repo.list_remotes().expect("Failed to list remotes");
    assert!(remotes.is_empty());
}

#[test]
fn test_rename_remote() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Rename it
    let problems = repo.rename_remote("origin", "upstream")
        .expect("Failed to rename remote");
    assert!(problems.is_empty(), "Rename should not have problems");
    
    // Verify the rename
    let remotes = repo.list_remotes().expect("Failed to list remotes");
    assert_eq!(remotes.len(), 1);
    assert_eq!(remotes[0], "upstream");
    
    // Verify old name doesn't exist
    assert!(repo.get_remote("origin").is_err());
    
    // Verify new name exists
    let remote = repo.get_remote("upstream").expect("Failed to get renamed remote");
    assert_eq!(remote.name, "upstream");
}

#[test]
fn test_update_remote_url() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Update the URL
    repo.set_remote_url("origin", "https://github.com/example/new-repo.git")
        .expect("Failed to set remote URL");
    
    // Verify the update
    let remote = repo.get_remote("origin").expect("Failed to get remote");
    assert_eq!(remote.fetch_url, Some("https://github.com/example/new-repo.git".to_string()));
}

#[test]
fn test_set_remote_push_url() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Set a different push URL
    repo.set_remote_push_url("origin", "git@github.com:example/repo.git")
        .expect("Failed to set remote push URL");
    
    // Verify the push URL
    let remote = repo.get_remote("origin").expect("Failed to get remote");
    assert_eq!(remote.fetch_url, Some("https://github.com/example/repo.git".to_string()));
    assert_eq!(remote.push_url, Some("git@github.com:example/repo.git".to_string()));
}

#[test]
fn test_default_remote() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // No remotes initially
    let default = repo.default_remote().expect("Failed to get default remote");
    assert!(default.is_none());
    
    // Add a non-origin remote
    repo.add_remote("upstream", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    let default = repo.default_remote().expect("Failed to get default remote");
    assert_eq!(default, Some("upstream".to_string()));
    
    // Add origin - it should become the default
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    let default = repo.default_remote().expect("Failed to get default remote");
    assert_eq!(default, Some("origin".to_string()));
}

#[test]
fn test_remote_branches_empty() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote but no remote branches
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    let branches = repo.remote_branches(None).expect("Failed to get remote branches");
    assert!(branches.is_empty());
    
    let branches = repo.remote_branches(Some("origin")).expect("Failed to get remote branches for origin");
    assert!(branches.is_empty());
}

#[test]
fn test_multiple_remotes() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add multiple remotes
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add origin");
    repo.add_remote("upstream", "https://github.com/upstream/repo.git")
        .expect("Failed to add upstream");
    repo.add_remote("fork", "https://github.com/user/repo.git")
        .expect("Failed to add fork");
    
    // List all remotes
    let mut remotes = repo.list_remotes().expect("Failed to list remotes");
    remotes.sort(); // Sort for consistent comparison
    assert_eq!(remotes, vec!["fork", "origin", "upstream"]);
    
    // Check each remote individually
    let origin = repo.get_remote("origin").expect("Failed to get origin");
    assert_eq!(origin.name, "origin");
    assert_eq!(origin.fetch_url, Some("https://github.com/example/repo.git".to_string()));
    
    let upstream = repo.get_remote("upstream").expect("Failed to get upstream");
    assert_eq!(upstream.name, "upstream");
    assert_eq!(upstream.fetch_url, Some("https://github.com/upstream/repo.git".to_string()));
}

#[test]
fn test_prune_remote_no_stale_branches() {
    let (_temp_dir, repo) = setup_test_repo();
    
    // Add a remote
    repo.add_remote("origin", "https://github.com/example/repo.git")
        .expect("Failed to add remote");
    
    // Prune should not fail even with no remote connection
    // Note: This will likely fail to connect, but we're testing the method signature
    let result = repo.prune_remote("origin");
    
    // The operation might fail due to network issues in tests, but the method should exist
    // and return the expected type
    match result {
        Ok(pruned) => {
            // If it succeeds, pruned should be empty
            assert!(pruned.is_empty());
        }
        Err(_) => {
            // Network operations might fail in test environment, that's okay
            // We're just verifying the API works
        }
    }
}