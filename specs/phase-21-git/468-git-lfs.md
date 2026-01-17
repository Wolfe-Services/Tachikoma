# 468 - Git LFS

**Phase:** 21 - Git Integration
**Spec ID:** 468
**Status:** Planned
**Dependencies:** 452-git-detect
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement Git LFS (Large File Storage) support for handling large binary files in Git repositories.

---

## Acceptance Criteria

- [x] Detect LFS installation
- [x] Track/untrack patterns
- [x] LFS file status
- [x] Fetch/pull LFS objects
- [x] Push LFS objects
- [x] LFS pointer handling

---

## Implementation Details

### 1. LFS Types (src/lfs.rs)

```rust
//! Git LFS support.

use crate::{GitRepository, GitResult, GitError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// LFS pointer file content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsPointer {
    /// LFS version.
    pub version: String,
    /// OID hash algorithm and value.
    pub oid: String,
    /// File size in bytes.
    pub size: u64,
}

impl LfsPointer {
    /// Parse LFS pointer from content.
    pub fn parse(content: &str) -> Option<Self> {
        let mut version = None;
        let mut oid = None;
        let mut size = None;

        for line in content.lines() {
            if let Some(v) = line.strip_prefix("version ") {
                version = Some(v.to_string());
            } else if let Some(o) = line.strip_prefix("oid sha256:") {
                oid = Some(format!("sha256:{}", o));
            } else if let Some(s) = line.strip_prefix("size ") {
                size = s.parse().ok();
            }
        }

        Some(Self {
            version: version?,
            oid: oid?,
            size: size?,
        })
    }

    /// Check if content is an LFS pointer.
    pub fn is_pointer(content: &str) -> bool {
        content.starts_with("version https://git-lfs.github.com/spec/")
    }

    /// Get the SHA256 hash.
    pub fn sha256(&self) -> Option<&str> {
        self.oid.strip_prefix("sha256:")
    }

    /// Create pointer content.
    pub fn to_content(&self) -> String {
        format!(
            "version {}\noid {}\nsize {}\n",
            self.version, self.oid, self.size
        )
    }
}

/// LFS track pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsTrackPattern {
    /// Pattern string.
    pub pattern: String,
    /// Whether pattern is tracked.
    pub tracked: bool,
    /// Filter type (usually "lfs").
    pub filter: String,
}

/// LFS file status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsFileStatus {
    /// File path.
    pub path: PathBuf,
    /// LFS pointer information.
    pub pointer: Option<LfsPointer>,
    /// Whether the file is downloaded.
    pub downloaded: bool,
    /// Local file size (0 if not downloaded).
    pub local_size: u64,
}

/// LFS installation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsStatus {
    /// LFS is installed system-wide.
    pub installed: bool,
    /// LFS version.
    pub version: Option<String>,
    /// LFS is initialized in repository.
    pub initialized: bool,
    /// Number of tracked patterns.
    pub tracked_patterns: usize,
    /// Number of LFS files.
    pub lfs_files: usize,
}

/// Git LFS manager.
pub struct LfsManager {
    repo: GitRepository,
}

impl LfsManager {
    /// Create a new LFS manager.
    pub fn new(repo: GitRepository) -> Self {
        Self { repo }
    }

    /// Check if Git LFS is installed.
    pub fn is_installed() -> bool {
        Command::new("git")
            .args(["lfs", "version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get LFS version.
    pub fn version() -> Option<String> {
        let output = Command::new("git")
            .args(["lfs", "version"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse "git-lfs/3.0.0 (GitHub; darwin amd64; go 1.17.2)"
            stdout.split_whitespace()
                .next()
                .and_then(|v| v.strip_prefix("git-lfs/"))
                .map(String::from)
        } else {
            None
        }
    }

    /// Initialize LFS in the repository.
    pub fn init(&self) -> GitResult<()> {
        let output = Command::new("git")
            .args(["lfs", "install"])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "Failed to initialize LFS: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// Get LFS status for the repository.
    pub fn status(&self) -> GitResult<LfsStatus> {
        let installed = Self::is_installed();
        let version = Self::version();

        // Check if LFS is initialized (has .gitattributes with lfs filter)
        let gitattributes = self.repo.root_path().join(".gitattributes");
        let initialized = gitattributes.exists() && {
            std::fs::read_to_string(&gitattributes)
                .map(|c| c.contains("filter=lfs"))
                .unwrap_or(false)
        };

        let tracked_patterns = self.list_tracked_patterns()
            .map(|p| p.len())
            .unwrap_or(0);

        let lfs_files = self.list_lfs_files()
            .map(|f| f.len())
            .unwrap_or(0);

        Ok(LfsStatus {
            installed,
            version,
            initialized,
            tracked_patterns,
            lfs_files,
        })
    }

    /// Track a pattern with LFS.
    pub fn track(&self, pattern: &str) -> GitResult<()> {
        let output = Command::new("git")
            .args(["lfs", "track", pattern])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "Failed to track pattern '{}': {}",
                    pattern,
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// Untrack a pattern.
    pub fn untrack(&self, pattern: &str) -> GitResult<()> {
        let output = Command::new("git")
            .args(["lfs", "untrack", pattern])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "Failed to untrack pattern '{}': {}",
                    pattern,
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// List tracked patterns.
    pub fn list_tracked_patterns(&self) -> GitResult<Vec<LfsTrackPattern>> {
        let output = Command::new("git")
            .args(["lfs", "track"])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut patterns = Vec::new();

        for line in stdout.lines() {
            // Parse lines like "    *.bin (filter=lfs diff=lfs merge=lfs)"
            let line = line.trim();
            if line.is_empty() || line.starts_with("Listing tracked patterns") {
                continue;
            }

            if let Some((pattern, _attrs)) = line.split_once(" (") {
                patterns.push(LfsTrackPattern {
                    pattern: pattern.to_string(),
                    tracked: true,
                    filter: "lfs".to_string(),
                });
            }
        }

        Ok(patterns)
    }

    /// List LFS files in the repository.
    pub fn list_lfs_files(&self) -> GitResult<Vec<LfsFileStatus>> {
        let output = Command::new("git")
            .args(["lfs", "ls-files", "--long"])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();

        for line in stdout.lines() {
            // Parse lines like "a1b2c3d4... * path/to/file.bin"
            // or "a1b2c3d4... - path/to/file.bin" (not downloaded)
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let oid = parts[0].trim_end_matches("...");
                let downloaded = parts[1] == "*";
                let path = PathBuf::from(parts[2]);

                let local_size = if downloaded {
                    let full_path = self.repo.root_path().join(&path);
                    std::fs::metadata(&full_path)
                        .map(|m| m.len())
                        .unwrap_or(0)
                } else {
                    0
                };

                files.push(LfsFileStatus {
                    path,
                    pointer: Some(LfsPointer {
                        version: "https://git-lfs.github.com/spec/v1".to_string(),
                        oid: format!("sha256:{}", oid),
                        size: local_size,
                    }),
                    downloaded,
                    local_size,
                });
            }
        }

        Ok(files)
    }

    /// Fetch LFS objects.
    pub fn fetch(&self, refs: Option<&[&str]>) -> GitResult<FetchResult> {
        let mut cmd = Command::new("git");
        cmd.arg("lfs").arg("fetch");

        if let Some(refs) = refs {
            for r in refs {
                cmd.arg(r);
            }
        }

        let output = cmd
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "LFS fetch failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        // Parse fetch output
        let stderr = String::from_utf8_lossy(&output.stderr);
        let fetched = self.parse_transfer_count(&stderr, "Downloading");

        Ok(FetchResult {
            objects_fetched: fetched,
            bytes_downloaded: 0, // Would need more parsing
        })
    }

    /// Pull LFS objects (fetch + checkout).
    pub fn pull(&self) -> GitResult<FetchResult> {
        let output = Command::new("git")
            .args(["lfs", "pull"])
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "LFS pull failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let fetched = self.parse_transfer_count(&stderr, "Downloading");

        Ok(FetchResult {
            objects_fetched: fetched,
            bytes_downloaded: 0,
        })
    }

    /// Push LFS objects.
    pub fn push(&self, remote: &str, refs: Option<&[&str]>) -> GitResult<PushResult> {
        let mut cmd = Command::new("git");
        cmd.arg("lfs").arg("push").arg(remote);

        if let Some(refs) = refs {
            for r in refs {
                cmd.arg(r);
            }
        } else {
            cmd.arg("--all");
        }

        let output = cmd
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "LFS push failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        let pushed = self.parse_transfer_count(&stderr, "Uploading");

        Ok(PushResult {
            objects_pushed: pushed,
            bytes_uploaded: 0,
        })
    }

    /// Check if a file is an LFS pointer.
    pub fn is_lfs_pointer(&self, path: impl AsRef<Path>) -> GitResult<bool> {
        let full_path = self.repo.root_path().join(path);

        if !full_path.exists() {
            return Ok(false);
        }

        // Read first 1KB to check for pointer
        let content = std::fs::read_to_string(&full_path)
            .map(|c| c.chars().take(1024).collect::<String>())
            .unwrap_or_default();

        Ok(LfsPointer::is_pointer(&content))
    }

    /// Get LFS pointer for a file.
    pub fn get_pointer(&self, path: impl AsRef<Path>) -> GitResult<Option<LfsPointer>> {
        let full_path = self.repo.root_path().join(path);

        if !full_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&full_path)?;

        if LfsPointer::is_pointer(&content) {
            Ok(LfsPointer::parse(&content))
        } else {
            Ok(None)
        }
    }

    /// Prune old LFS objects.
    pub fn prune(&self, dry_run: bool) -> GitResult<PruneResult> {
        let mut cmd = Command::new("git");
        cmd.arg("lfs").arg("prune");

        if dry_run {
            cmd.arg("--dry-run");
        }

        let output = cmd
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "LFS prune failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        // Parse prune output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let objects_removed = self.parse_prune_count(&stdout);

        Ok(PruneResult {
            objects_removed,
            bytes_freed: 0,
            dry_run,
        })
    }

    /// Migrate files to LFS.
    pub fn migrate(
        &self,
        patterns: &[&str],
        include_history: bool,
    ) -> GitResult<MigrateResult> {
        let mut cmd = Command::new("git");
        cmd.arg("lfs").arg("migrate").arg("import");

        for pattern in patterns {
            cmd.arg("--include").arg(pattern);
        }

        if include_history {
            cmd.arg("--everything");
        } else {
            cmd.arg("--no-rewrite");
        }

        let output = cmd
            .current_dir(self.repo.root_path())
            .output()?;

        if !output.status.success() {
            return Err(GitError::InvalidOperation {
                message: format!(
                    "LFS migrate failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(MigrateResult {
            files_migrated: patterns.len(),
            commits_rewritten: if include_history { Some(0) } else { None },
        })
    }

    fn parse_transfer_count(&self, output: &str, prefix: &str) -> usize {
        for line in output.lines() {
            if line.contains(prefix) {
                // Try to extract count
                if let Some(count) = line.split_whitespace()
                    .find_map(|w| w.parse::<usize>().ok())
                {
                    return count;
                }
            }
        }
        0
    }

    fn parse_prune_count(&self, output: &str) -> usize {
        for line in output.lines() {
            if line.contains("prune") || line.contains("objects") {
                if let Some(count) = line.split_whitespace()
                    .find_map(|w| w.parse::<usize>().ok())
                {
                    return count;
                }
            }
        }
        0
    }
}

/// LFS fetch result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResult {
    /// Number of objects fetched.
    pub objects_fetched: usize,
    /// Bytes downloaded.
    pub bytes_downloaded: u64,
}

/// LFS push result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    /// Number of objects pushed.
    pub objects_pushed: usize,
    /// Bytes uploaded.
    pub bytes_uploaded: u64,
}

/// LFS prune result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneResult {
    /// Number of objects removed.
    pub objects_removed: usize,
    /// Bytes freed.
    pub bytes_freed: u64,
    /// Was this a dry run.
    pub dry_run: bool,
}

/// LFS migrate result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateResult {
    /// Number of files migrated.
    pub files_migrated: usize,
    /// Number of commits rewritten (if rewriting history).
    pub commits_rewritten: Option<usize>,
}

/// Common LFS file patterns.
pub mod patterns {
    /// Image files.
    pub const IMAGES: &[&str] = &[
        "*.png", "*.jpg", "*.jpeg", "*.gif", "*.bmp", "*.tiff", "*.ico", "*.webp",
    ];

    /// Video files.
    pub const VIDEOS: &[&str] = &[
        "*.mp4", "*.avi", "*.mov", "*.wmv", "*.flv", "*.webm", "*.mkv",
    ];

    /// Audio files.
    pub const AUDIO: &[&str] = &[
        "*.mp3", "*.wav", "*.flac", "*.aac", "*.ogg", "*.wma",
    ];

    /// Archive files.
    pub const ARCHIVES: &[&str] = &[
        "*.zip", "*.tar", "*.gz", "*.bz2", "*.7z", "*.rar",
    ];

    /// Binary/compiled files.
    pub const BINARIES: &[&str] = &[
        "*.exe", "*.dll", "*.so", "*.dylib", "*.a", "*.lib",
    ];

    /// Document files.
    pub const DOCUMENTS: &[&str] = &[
        "*.pdf", "*.doc", "*.docx", "*.xls", "*.xlsx", "*.ppt", "*.pptx",
    ];
}
```

---

## Testing Requirements

1. LFS installation detection works
2. Track/untrack patterns work
3. Pointer parsing is correct
4. Fetch/pull downloads objects
5. Push uploads objects
6. Prune removes old objects

---

## Related Specs

- Depends on: [452-git-detect.md](452-git-detect.md)
- Next: [469-git-api.md](469-git-api.md)
