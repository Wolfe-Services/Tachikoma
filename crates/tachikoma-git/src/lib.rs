//! Git integration for Tachikoma.
//!
//! This crate provides safe Rust wrappers around the git2 library.

#![warn(missing_docs)]

pub mod blame;
pub mod branch;
pub mod commit;
pub mod conflict;
pub mod credentials;
pub mod detect;
pub mod detached;
pub mod diff;
mod diff_impl;
pub mod error;
pub mod history;
pub mod hooks;
pub mod lfs;
pub mod merge;
pub mod oid;
pub mod push;
pub mod reference;
pub mod remote;
pub mod repository;
pub mod ssh;
pub mod staging;
pub mod status;
mod status_impl;

pub use blame::{BlameEntry, BlameOptions, BlameResult, LineBlame};
pub use commit::{CommitOptions, GitCommit};
pub use conflict::{ConflictRegion, ConflictType, FileConflict, ResolutionStrategy};
pub use credentials::{GitCredential, CredentialStore, CredentialCallback};
pub use detect::{DetectOptions, RepoInfo, detect_repo, find_repo_root, find_repos, is_inside_repo, open_repo, open_repo_with_flags};
pub use diff::{DiffFile, DiffHunk, DiffLine, DiffOptions, DiffStats, DiffStatus, GitDiff, LineOrigin};
pub use error::{GitError, GitResult};
pub use history::{HistoryEntry, HistoryOptions, HistoryPage};
pub use hooks::{HookType, HookInfo, HookResult};
pub use lfs::{
    LfsManager, LfsPointer, LfsTrackPattern, LfsFileStatus, LfsStatus,
    FetchResult, PushResult, PruneResult, MigrateResult, patterns
};
pub use merge::{ConflictBlob, ConflictFile, MergeOptions, MergeResult, MergeResultType};
pub use oid::{GitOid, GitOidError};
pub use push::{PushOpts, PushProgress, PushResult, CredentialProvider, DefaultCredentialProvider};
pub use reference::{GitBranch, GitRef, GitSignature, GitTag, RefType};
pub use remote::{GitRemote, RemoteBranch};
pub use repository::{GitRepository, GitRepositoryOptions};
pub use ssh::{
    SshUrl, SshKeyType, SshKeyPair, SshConfigEntry, KnownHost,
    generate_ssh_key, get_key_fingerprint, list_ssh_keys, add_key_to_agent, list_agent_keys,
    parse_ssh_config, get_ssh_config, parse_known_hosts, is_known_host, add_known_host, get_host_key
};
pub use status::{FileStatus, RepoStatus, StatusEntry, StatusOptions, StatusSummary};

// Re-export git2 for advanced usage
pub use git2;