# Spec 462: Gitignore Handling

## Phase
21 - Git Integration

## Spec ID
462

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)

## Estimated Context
~8%

---

## Objective

Implement gitignore pattern management for Tachikoma, providing functionality to read, parse, and modify .gitignore files. This module enables intelligent ignore pattern handling, common pattern suggestions, and validation of ignore rules.

---

## Acceptance Criteria

- [ ] Implement `GitIgnoreManager` for ignore operations
- [ ] Parse .gitignore patterns
- [ ] Support global gitignore files
- [ ] Check if paths match ignore patterns
- [ ] Add patterns to .gitignore
- [ ] Remove patterns from .gitignore
- [ ] Suggest common ignore patterns
- [ ] Support .gitignore templates
- [ ] Validate ignore patterns
- [ ] Support nested .gitignore files

---

## Implementation Details

### Gitignore Manager Implementation

```rust
// src/git/ignore.rs

use glob::Pattern;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use super::repo::GitRepository;
use super::types::*;

/// Gitignore pattern entry
#[derive(Debug, Clone)]
pub struct IgnorePattern {
    /// The pattern string
    pub pattern: String,
    /// Is this a negation pattern (starts with !)
    pub negated: bool,
    /// Is directory only (ends with /)
    pub directory_only: bool,
    /// Source file and line number
    pub source: Option<(PathBuf, usize)>,
    /// Optional comment above the pattern
    pub comment: Option<String>,
}

impl IgnorePattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let negated = pattern.starts_with('!');
        let directory_only = pattern.ends_with('/');

        Self {
            pattern,
            negated,
            directory_only,
            source: None,
            comment: None,
        }
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Check if this pattern matches a path
    pub fn matches(&self, path: &Path) -> bool {
        let pattern_str = self.pattern
            .trim_start_matches('!')
            .trim_end_matches('/');

        // Handle directory-only patterns
        if self.directory_only && !path.is_dir() {
            return false;
        }

        // Try glob matching
        if let Ok(glob) = Pattern::new(pattern_str) {
            if glob.matches_path(path) {
                return true;
            }
        }

        // Try simple contains match for patterns like *.log
        if pattern_str.starts_with('*') {
            let suffix = &pattern_str[1..];
            if path.to_string_lossy().ends_with(suffix) {
                return true;
            }
        }

        // Match by filename
        if let Some(name) = path.file_name() {
            if name.to_string_lossy() == pattern_str {
                return true;
            }
        }

        false
    }
}

/// Parsed gitignore file
#[derive(Debug, Clone)]
pub struct GitIgnoreFile {
    /// Path to the gitignore file
    pub path: PathBuf,
    /// Patterns in this file
    pub patterns: Vec<IgnorePattern>,
    /// Header comments
    pub header: Vec<String>,
}

impl GitIgnoreFile {
    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> GitResult<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)?;
        Self::parse(&path, &content)
    }

    /// Parse content
    pub fn parse(path: &Path, content: &str) -> GitResult<Self> {
        let mut patterns = Vec::new();
        let mut header = Vec::new();
        let mut current_comment: Option<String> = None;
        let mut in_header = true;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Empty line
            if line.is_empty() {
                current_comment = None;
                continue;
            }

            // Comment
            if line.starts_with('#') {
                let comment = line.trim_start_matches('#').trim();
                if in_header {
                    header.push(comment.to_string());
                } else {
                    current_comment = Some(comment.to_string());
                }
                continue;
            }

            in_header = false;

            // Pattern
            let mut pattern = IgnorePattern::new(line);
            pattern.source = Some((path.to_path_buf(), line_num + 1));
            if let Some(comment) = current_comment.take() {
                pattern.comment = Some(comment);
            }
            patterns.push(pattern);
        }

        Ok(Self {
            path: path.to_path_buf(),
            patterns,
            header,
        })
    }

    /// Check if path is ignored by this file
    pub fn is_ignored(&self, path: &Path) -> bool {
        let mut ignored = false;

        for pattern in &self.patterns {
            if pattern.matches(path) {
                ignored = !pattern.negated;
            }
        }

        ignored
    }

    /// Add a pattern
    pub fn add_pattern(&mut self, pattern: IgnorePattern) {
        // Check for duplicates
        if !self.patterns.iter().any(|p| p.pattern == pattern.pattern) {
            self.patterns.push(pattern);
        }
    }

    /// Remove a pattern
    pub fn remove_pattern(&mut self, pattern: &str) -> bool {
        let initial_len = self.patterns.len();
        self.patterns.retain(|p| p.pattern != pattern);
        self.patterns.len() < initial_len
    }

    /// Save to file
    pub fn save(&self) -> GitResult<()> {
        let mut content = String::new();

        // Write header
        for comment in &self.header {
            content.push_str(&format!("# {}\n", comment));
        }

        if !self.header.is_empty() {
            content.push('\n');
        }

        // Write patterns
        let mut last_had_comment = false;
        for pattern in &self.patterns {
            if let Some(ref comment) = pattern.comment {
                if !last_had_comment {
                    content.push('\n');
                }
                content.push_str(&format!("# {}\n", comment));
                last_had_comment = true;
            } else {
                last_had_comment = false;
            }
            content.push_str(&format!("{}\n", pattern.pattern));
        }

        fs::write(&self.path, content)?;
        Ok(())
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        let mut content = String::new();

        for comment in &self.header {
            content.push_str(&format!("# {}\n", comment));
        }

        if !self.header.is_empty() {
            content.push('\n');
        }

        for pattern in &self.patterns {
            if let Some(ref comment) = pattern.comment {
                content.push_str(&format!("# {}\n", comment));
            }
            content.push_str(&format!("{}\n", pattern.pattern));
        }

        content
    }
}

/// Gitignore manager
pub struct GitIgnoreManager<'a> {
    repo: &'a GitRepository,
}

impl<'a> GitIgnoreManager<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self { repo }
    }

    /// Get the root .gitignore file
    pub fn root_gitignore(&self) -> PathBuf {
        self.repo.path().join(".gitignore")
    }

    /// Load root .gitignore
    pub fn load_root(&self) -> GitResult<GitIgnoreFile> {
        let path = self.root_gitignore();
        if path.exists() {
            GitIgnoreFile::load(&path)
        } else {
            Ok(GitIgnoreFile {
                path,
                patterns: Vec::new(),
                header: Vec::new(),
            })
        }
    }

    /// Check if path is ignored
    pub fn is_ignored(&self, path: &Path) -> GitResult<bool> {
        // Use git2's built-in ignore checking
        Ok(self.repo.is_ignored(path)?)
    }

    /// Get all .gitignore files in repository
    pub fn find_all(&self) -> GitResult<Vec<PathBuf>> {
        let mut ignores = Vec::new();
        self.find_gitignores_recursive(self.repo.path(), &mut ignores)?;
        Ok(ignores)
    }

    fn find_gitignores_recursive(&self, dir: &Path, ignores: &mut Vec<PathBuf>) -> GitResult<()> {
        let gitignore = dir.join(".gitignore");
        if gitignore.exists() {
            ignores.push(gitignore);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip .git directory
            if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                self.find_gitignores_recursive(&path, ignores)?;
            }
        }

        Ok(())
    }

    /// Add pattern to root .gitignore
    pub fn add_pattern(&self, pattern: impl Into<String>) -> GitResult<()> {
        let mut ignore = self.load_root()?;
        ignore.add_pattern(IgnorePattern::new(pattern));
        ignore.save()
    }

    /// Add patterns with comment section
    pub fn add_patterns(&self, section: &str, patterns: &[&str]) -> GitResult<()> {
        let mut ignore = self.load_root()?;

        for (i, pattern) in patterns.iter().enumerate() {
            let mut p = IgnorePattern::new(*pattern);
            if i == 0 {
                p.comment = Some(section.to_string());
            }
            ignore.add_pattern(p);
        }

        ignore.save()
    }

    /// Remove pattern from root .gitignore
    pub fn remove_pattern(&self, pattern: &str) -> GitResult<bool> {
        let mut ignore = self.load_root()?;
        let removed = ignore.remove_pattern(pattern);
        if removed {
            ignore.save()?;
        }
        Ok(removed)
    }

    /// Get global gitignore path
    pub fn global_gitignore_path(&self) -> Option<PathBuf> {
        if let Ok(config) = self.repo.config() {
            if let Ok(Some(path)) = config.get_string("core.excludesfile") {
                return Some(PathBuf::from(shellexpand::tilde(&path).to_string()));
            }
        }

        // Default locations
        dirs::home_dir().map(|h| h.join(".gitignore_global"))
    }
}

/// Common gitignore templates
pub struct GitIgnoreTemplates;

impl GitIgnoreTemplates {
    /// Get template for a language/framework
    pub fn get(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "rust" => Some(RUST_GITIGNORE),
            "node" | "nodejs" | "javascript" | "typescript" => Some(NODE_GITIGNORE),
            "python" => Some(PYTHON_GITIGNORE),
            "go" | "golang" => Some(GO_GITIGNORE),
            "macos" | "osx" => Some(MACOS_GITIGNORE),
            "windows" => Some(WINDOWS_GITIGNORE),
            "linux" => Some(LINUX_GITIGNORE),
            "jetbrains" | "idea" => Some(JETBRAINS_GITIGNORE),
            "vscode" => Some(VSCODE_GITIGNORE),
            _ => None,
        }
    }

    /// List available templates
    pub fn list() -> Vec<&'static str> {
        vec![
            "rust", "node", "python", "go",
            "macos", "windows", "linux",
            "jetbrains", "vscode"
        ]
    }
}

const RUST_GITIGNORE: &str = r#"# Rust
/target/
Cargo.lock
**/*.rs.bk
*.pdb
"#;

const NODE_GITIGNORE: &str = r#"# Node
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.npm
.yarn-integrity
dist/
build/
.env
.env.local
"#;

const PYTHON_GITIGNORE: &str = r#"# Python
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
.env
.venv
venv/
ENV/
"#;

const GO_GITIGNORE: &str = r#"# Go
*.exe
*.exe~
*.dll
*.so
*.dylib
*.test
*.out
vendor/
go.work
"#;

const MACOS_GITIGNORE: &str = r#"# macOS
.DS_Store
.AppleDouble
.LSOverride
._*
.Spotlight-V100
.Trashes
"#;

const WINDOWS_GITIGNORE: &str = r#"# Windows
Thumbs.db
ehthumbs.db
Desktop.ini
$RECYCLE.BIN/
*.lnk
"#;

const LINUX_GITIGNORE: &str = r#"# Linux
*~
.fuse_hidden*
.directory
.Trash-*
.nfs*
"#;

const JETBRAINS_GITIGNORE: &str = r#"# JetBrains
.idea/
*.iws
*.iml
*.ipr
out/
"#;

const VSCODE_GITIGNORE: &str = r#"# VSCode
.vscode/*
!.vscode/settings.json
!.vscode/tasks.json
!.vscode/launch.json
!.vscode/extensions.json
*.code-workspace
"#;
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
    fn test_ignore_pattern_new() {
        let pattern = IgnorePattern::new("*.log");
        assert_eq!(pattern.pattern, "*.log");
        assert!(!pattern.negated);
        assert!(!pattern.directory_only);
    }

    #[test]
    fn test_ignore_pattern_negated() {
        let pattern = IgnorePattern::new("!important.log");
        assert!(pattern.negated);
    }

    #[test]
    fn test_ignore_pattern_directory() {
        let pattern = IgnorePattern::new("build/");
        assert!(pattern.directory_only);
    }

    #[test]
    fn test_pattern_matches_extension() {
        let pattern = IgnorePattern::new("*.log");
        assert!(pattern.matches(Path::new("test.log")));
        assert!(pattern.matches(Path::new("debug.log")));
        assert!(!pattern.matches(Path::new("test.txt")));
    }

    #[test]
    fn test_pattern_matches_filename() {
        let pattern = IgnorePattern::new(".DS_Store");
        assert!(pattern.matches(Path::new(".DS_Store")));
        assert!(!pattern.matches(Path::new("DS_Store")));
    }

    #[test]
    fn test_gitignore_file_parse() {
        let content = r#"# Test header
# Another comment

# Build output
target/
*.o

# Keep this
!important.txt
"#;

        let file = GitIgnoreFile::parse(Path::new(".gitignore"), content).unwrap();

        assert_eq!(file.header.len(), 2);
        assert_eq!(file.patterns.len(), 3);
        assert!(file.patterns[0].directory_only);
        assert!(file.patterns[2].negated);
    }

    #[test]
    fn test_gitignore_add_pattern() {
        let (dir, repo) = setup_test_repo();
        let manager = GitIgnoreManager::new(&repo);

        manager.add_pattern("*.log").unwrap();

        let loaded = manager.load_root().unwrap();
        assert_eq!(loaded.patterns.len(), 1);
        assert_eq!(loaded.patterns[0].pattern, "*.log");
    }

    #[test]
    fn test_gitignore_remove_pattern() {
        let (dir, repo) = setup_test_repo();
        let manager = GitIgnoreManager::new(&repo);

        manager.add_pattern("*.log").unwrap();
        assert!(manager.remove_pattern("*.log").unwrap());

        let loaded = manager.load_root().unwrap();
        assert!(loaded.patterns.is_empty());
    }

    #[test]
    fn test_gitignore_no_duplicates() {
        let (dir, repo) = setup_test_repo();
        let manager = GitIgnoreManager::new(&repo);

        manager.add_pattern("*.log").unwrap();
        manager.add_pattern("*.log").unwrap();

        let loaded = manager.load_root().unwrap();
        assert_eq!(loaded.patterns.len(), 1);
    }

    #[test]
    fn test_gitignore_is_ignored() {
        let mut file = GitIgnoreFile {
            path: PathBuf::from(".gitignore"),
            patterns: vec![
                IgnorePattern::new("*.log"),
                IgnorePattern::new("!important.log"),
            ],
            header: Vec::new(),
        };

        assert!(file.is_ignored(Path::new("test.log")));
        assert!(!file.is_ignored(Path::new("important.log"))); // Negated
        assert!(!file.is_ignored(Path::new("test.txt")));
    }

    #[test]
    fn test_template_rust() {
        let template = GitIgnoreTemplates::get("rust").unwrap();
        assert!(template.contains("/target/"));
        assert!(template.contains("Cargo.lock"));
    }

    #[test]
    fn test_template_node() {
        let template = GitIgnoreTemplates::get("node").unwrap();
        assert!(template.contains("node_modules/"));
    }

    #[test]
    fn test_template_list() {
        let templates = GitIgnoreTemplates::list();
        assert!(templates.contains(&"rust"));
        assert!(templates.contains(&"python"));
    }

    #[test]
    fn test_template_not_found() {
        assert!(GitIgnoreTemplates::get("unknown").is_none());
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 449: Status Checking
