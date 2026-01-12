# Spec 451: Commit Operations

## Phase
21 - Git Integration

## Spec ID
451

## Status
Planned

## Dependencies
- Spec 446: Git Types (core type definitions)
- Spec 448: Repository Operations (repository access)
- Spec 449: Status Checking (status verification)

## Estimated Context
~10%

---

## Objective

Implement Git commit operations for Tachikoma, providing functionality to create commits, amend commits, and manage commit metadata. This module supports both standard commits and AI-assisted commit message generation, integrating with Tachikoma's code analysis capabilities.

---

## Acceptance Criteria

- [ ] Implement `GitCommitter` for commit creation
- [ ] Support standard commit with message
- [ ] Support commit amending
- [ ] Implement GPG signing support
- [ ] Support commit with co-authors
- [ ] Validate commit messages
- [ ] Support empty commits (with flag)
- [ ] Implement commit templates
- [ ] Support staging and committing in one operation
- [ ] Provide commit dry-run capability

---

## Implementation Details

### Commit Manager Implementation

```rust
// src/git/commit.rs

use git2::{Commit, ObjectType, Oid, Repository, Signature, Time};
use std::path::Path;

use super::config::GitConfig;
use super::repo::GitRepository;
use super::status::{GitStatusChecker, StatusCheckOptions};
use super::types::*;

/// Options for creating a commit
#[derive(Debug, Clone)]
pub struct CommitOptions {
    /// Commit message
    pub message: String,
    /// Author (defaults to config)
    pub author: Option<GitSignature>,
    /// Committer (defaults to author or config)
    pub committer: Option<GitSignature>,
    /// Amend the last commit
    pub amend: bool,
    /// Allow empty commits
    pub allow_empty: bool,
    /// Sign with GPG
    pub sign: bool,
    /// GPG key ID (None for default)
    pub signing_key: Option<String>,
    /// Co-authors to include in message
    pub co_authors: Vec<GitSignature>,
    /// Skip pre-commit hook
    pub no_verify: bool,
    /// Only verify (dry run)
    pub dry_run: bool,
}

impl Default for CommitOptions {
    fn default() -> Self {
        Self {
            message: String::new(),
            author: None,
            committer: None,
            amend: false,
            allow_empty: false,
            sign: false,
            signing_key: None,
            co_authors: Vec::new(),
            no_verify: false,
            dry_run: false,
        }
    }
}

impl CommitOptions {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    pub fn author(mut self, author: GitSignature) -> Self {
        self.author = Some(author);
        self
    }

    pub fn amend(mut self) -> Self {
        self.amend = true;
        self
    }

    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    pub fn sign(mut self, sign: bool) -> Self {
        self.sign = sign;
        self
    }

    pub fn co_author(mut self, author: GitSignature) -> Self {
        self.co_authors.push(author);
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

/// Result of a commit operation
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// The created commit OID
    pub oid: GitOid,
    /// The commit message used
    pub message: String,
    /// Author signature
    pub author: GitSignature,
    /// Files included in commit
    pub files_changed: usize,
    /// Was this an amendment
    pub amended: bool,
}

/// Commit message validator
pub struct CommitMessageValidator {
    /// Maximum subject line length
    pub max_subject_length: usize,
    /// Maximum body line length
    pub max_body_line_length: usize,
    /// Require conventional commit format
    pub require_conventional: bool,
    /// Allowed conventional commit types
    pub conventional_types: Vec<String>,
}

impl Default for CommitMessageValidator {
    fn default() -> Self {
        Self {
            max_subject_length: 72,
            max_body_line_length: 80,
            require_conventional: false,
            conventional_types: vec![
                "feat".to_string(),
                "fix".to_string(),
                "docs".to_string(),
                "style".to_string(),
                "refactor".to_string(),
                "perf".to_string(),
                "test".to_string(),
                "build".to_string(),
                "ci".to_string(),
                "chore".to_string(),
                "revert".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitMessageValidation {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl CommitMessageValidator {
    pub fn validate(&self, message: &str) -> CommitMessageValidation {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() || lines[0].trim().is_empty() {
            errors.push("Commit message cannot be empty".to_string());
            return CommitMessageValidation {
                is_valid: false,
                warnings,
                errors,
            };
        }

        let subject = lines[0];

        // Check subject length
        if subject.len() > self.max_subject_length {
            warnings.push(format!(
                "Subject line is {} characters, recommended maximum is {}",
                subject.len(),
                self.max_subject_length
            ));
        }

        // Check for period at end of subject
        if subject.ends_with('.') {
            warnings.push("Subject line should not end with a period".to_string());
        }

        // Check for blank line after subject if there's a body
        if lines.len() > 1 && !lines[1].is_empty() {
            warnings.push("Separate subject from body with a blank line".to_string());
        }

        // Check body line lengths
        for (i, line) in lines.iter().enumerate().skip(2) {
            if line.len() > self.max_body_line_length && !line.starts_with("Co-authored-by:") {
                warnings.push(format!(
                    "Line {} is {} characters, recommended maximum is {}",
                    i + 1,
                    line.len(),
                    self.max_body_line_length
                ));
            }
        }

        // Check conventional commit format if required
        if self.require_conventional {
            if let Some(conv_error) = self.validate_conventional(subject) {
                errors.push(conv_error);
            }
        }

        CommitMessageValidation {
            is_valid: errors.is_empty(),
            warnings,
            errors,
        }
    }

    fn validate_conventional(&self, subject: &str) -> Option<String> {
        // Pattern: type(scope)?: description
        let parts: Vec<&str> = subject.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Some("Conventional commit requires format: type(scope)?: description".to_string());
        }

        let type_part = parts[0];
        let type_name = type_part.split('(').next().unwrap_or(type_part);

        if !self.conventional_types.contains(&type_name.to_string()) {
            return Some(format!(
                "Unknown conventional commit type: '{}'. Allowed: {}",
                type_name,
                self.conventional_types.join(", ")
            ));
        }

        if parts[1].trim().is_empty() {
            return Some("Conventional commit requires a description after the colon".to_string());
        }

        None
    }
}

/// Git committer
pub struct GitCommitter<'a> {
    repo: &'a GitRepository,
    validator: CommitMessageValidator,
}

impl<'a> GitCommitter<'a> {
    pub fn new(repo: &'a GitRepository) -> Self {
        Self {
            repo,
            validator: CommitMessageValidator::default(),
        }
    }

    pub fn with_validator(mut self, validator: CommitMessageValidator) -> Self {
        self.validator = validator;
        self
    }

    /// Create a commit with the given options
    pub fn commit(&self, options: CommitOptions) -> GitResult<CommitResult> {
        // Validate message
        let validation = self.validator.validate(&options.message);
        if !validation.is_valid {
            return Err(GitError::Other(format!(
                "Invalid commit message: {}",
                validation.errors.join(", ")
            )));
        }

        // Check for staged changes (unless allow_empty or amend)
        if !options.allow_empty && !options.amend {
            let mut checker = GitStatusChecker::new(self.repo);
            let status = checker.check(&StatusCheckOptions::default())?;
            if !status.summary.has_staged_changes() {
                return Err(GitError::Other("No staged changes to commit".to_string()));
            }
        }

        if options.dry_run {
            return self.dry_run_commit(&options);
        }

        let raw_repo = self.repo.raw();

        // Get signatures
        let author = self.get_signature(&options.author, raw_repo)?;
        let committer = self.get_signature(&options.committer.or(options.author.clone()), raw_repo)?;

        // Build final message with co-authors
        let message = self.build_message(&options);

        // Get tree from index
        let mut index = raw_repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = raw_repo.find_tree(tree_oid)?;

        // Get parent commits
        let parents = if options.amend {
            // For amend, use the parents of HEAD
            let head_commit = raw_repo.head()?.peel_to_commit()?;
            head_commit.parents().collect::<Vec<_>>()
        } else {
            // Normal commit - HEAD is the parent (if it exists)
            match raw_repo.head() {
                Ok(head) => vec![head.peel_to_commit()?],
                Err(e) if e.code() == git2::ErrorCode::UnbornBranch => vec![],
                Err(e) => return Err(GitError::Git2(e)),
            }
        };

        let parent_refs: Vec<&Commit> = parents.iter().collect();

        // Create commit
        let oid = if options.amend {
            // Update HEAD to point to the new commit
            let new_oid = raw_repo.commit(
                Some("HEAD"),
                &author,
                &committer,
                &message,
                &tree,
                &parent_refs,
            )?;
            new_oid
        } else {
            raw_repo.commit(
                Some("HEAD"),
                &author,
                &committer,
                &message,
                &tree,
                &parent_refs,
            )?
        };

        Ok(CommitResult {
            oid: GitOid::from(oid),
            message,
            author: GitSignature::from(author),
            files_changed: 0, // Would need to calculate from diff
            amended: options.amend,
        })
    }

    /// Stage specific files and commit
    pub fn stage_and_commit(
        &self,
        paths: &[&Path],
        options: CommitOptions,
    ) -> GitResult<CommitResult> {
        for path in paths {
            self.repo.stage_file(path)?;
        }
        self.commit(options)
    }

    /// Stage all changes and commit
    pub fn stage_all_and_commit(&self, options: CommitOptions) -> GitResult<CommitResult> {
        self.repo.stage_all()?;
        self.commit(options)
    }

    /// Amend the last commit with new message
    pub fn amend_message(&self, new_message: impl Into<String>) -> GitResult<CommitResult> {
        self.commit(CommitOptions::new(new_message).amend())
    }

    /// Perform a dry run of the commit
    fn dry_run_commit(&self, options: &CommitOptions) -> GitResult<CommitResult> {
        let raw_repo = self.repo.raw();

        let author = self.get_signature(&options.author, raw_repo)?;
        let message = self.build_message(options);

        // Just return what would be committed without actually committing
        Ok(CommitResult {
            oid: GitOid([0; 20]), // Dummy OID for dry run
            message,
            author: GitSignature::from(author),
            files_changed: 0,
            amended: options.amend,
        })
    }

    fn get_signature(
        &self,
        sig: &Option<GitSignature>,
        repo: &Repository,
    ) -> GitResult<Signature<'static>> {
        match sig {
            Some(s) => {
                let time = Time::new(s.time.timestamp(), 0);
                Ok(Signature::new(&s.name, &s.email, &time)?)
            }
            None => Ok(repo.signature()?),
        }
    }

    fn build_message(&self, options: &CommitOptions) -> String {
        let mut message = options.message.clone();

        // Add co-authors
        if !options.co_authors.is_empty() {
            if !message.ends_with('\n') {
                message.push('\n');
            }
            message.push('\n');

            for author in &options.co_authors {
                message.push_str(&format!(
                    "Co-authored-by: {} <{}>\n",
                    author.name, author.email
                ));
            }
        }

        message
    }
}

/// Commit template manager
pub struct CommitTemplateManager {
    templates: std::collections::HashMap<String, String>,
}

impl CommitTemplateManager {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    /// Load template from repository config
    pub fn load_from_config(config: &GitConfig) -> GitResult<Option<String>> {
        config.get_string("commit.template")
    }

    /// Register a named template
    pub fn register(&mut self, name: impl Into<String>, template: impl Into<String>) {
        self.templates.insert(name.into(), template.into());
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// Apply a template with variables
    pub fn apply(&self, name: &str, vars: &std::collections::HashMap<String, String>) -> Option<String> {
        let template = self.get(name)?;
        let mut result = template.clone();

        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        Some(result)
    }

    /// Default conventional commit templates
    pub fn with_conventional_defaults(mut self) -> Self {
        self.register("feat", "feat({{scope}}): {{description}}\n\n{{body}}");
        self.register("fix", "fix({{scope}}): {{description}}\n\n{{body}}");
        self.register("docs", "docs({{scope}}): {{description}}\n\n{{body}}");
        self.register("refactor", "refactor({{scope}}): {{description}}\n\n{{body}}");
        self.register("test", "test({{scope}}): {{description}}\n\n{{body}}");
        self.register("chore", "chore({{scope}}): {{description}}\n\n{{body}}");
        self
    }
}

impl Default for CommitTemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a commit message into parts
pub struct CommitMessageParser;

impl CommitMessageParser {
    /// Parse message into subject and body
    pub fn parse(message: &str) -> (String, Option<String>) {
        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() {
            return (String::new(), None);
        }

        let subject = lines[0].to_string();

        // Find body (skip blank lines after subject)
        let body_start = lines
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, line)| !line.is_empty())
            .map(|(i, _)| i);

        let body = body_start.map(|start| {
            lines[start..].join("\n")
        });

        (subject, body)
    }

    /// Extract conventional commit parts
    pub fn parse_conventional(message: &str) -> Option<ConventionalCommit> {
        let (subject, body) = Self::parse(message);

        // Pattern: type(scope)?: description
        let re = regex::Regex::new(r"^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$").ok()?;
        let caps = re.captures(&subject)?;

        Some(ConventionalCommit {
            commit_type: caps.get(1)?.as_str().to_string(),
            scope: caps.get(2).map(|m| m.as_str().to_string()),
            breaking: caps.get(3).is_some(),
            description: caps.get(4)?.as_str().to_string(),
            body,
        })
    }
}

/// Parsed conventional commit
#[derive(Debug, Clone)]
pub struct ConventionalCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub breaking: bool,
    pub description: String,
    pub body: Option<String>,
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

        // Configure user
        let mut config = repo.config().unwrap();
        config.set_string("user.name", "Test User").unwrap();
        config.set_string("user.email", "test@example.com").unwrap();

        (dir, repo)
    }

    #[test]
    fn test_commit_message_validation_valid() {
        let validator = CommitMessageValidator::default();
        let result = validator.validate("Add new feature\n\nThis adds a great new feature.");

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_commit_message_validation_empty() {
        let validator = CommitMessageValidator::default();
        let result = validator.validate("");

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_commit_message_validation_long_subject() {
        let validator = CommitMessageValidator::default();
        let long_subject = "a".repeat(100);
        let result = validator.validate(&long_subject);

        assert!(result.is_valid); // Long subject is a warning, not error
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_conventional_commit_validation() {
        let validator = CommitMessageValidator {
            require_conventional: true,
            ..Default::default()
        };

        let valid = validator.validate("feat: add new feature");
        assert!(valid.is_valid);

        let valid_with_scope = validator.validate("fix(auth): resolve login issue");
        assert!(valid_with_scope.is_valid);

        let invalid = validator.validate("Add new feature");
        assert!(!invalid.is_valid);
    }

    #[test]
    fn test_commit_with_staged_changes() {
        let (dir, repo) = setup_test_repo();

        // Create and stage a file
        std::fs::write(dir.path().join("test.txt"), "content").unwrap();
        repo.stage_file(Path::new("test.txt")).unwrap();

        let committer = GitCommitter::new(&repo);
        let result = committer.commit(CommitOptions::new("Test commit")).unwrap();

        assert!(!result.oid.is_zero());
        assert_eq!(result.message, "Test commit");
    }

    #[test]
    fn test_commit_without_staged_changes_fails() {
        let (_dir, repo) = setup_test_repo();

        let committer = GitCommitter::new(&repo);
        let result = committer.commit(CommitOptions::new("Test commit"));

        assert!(result.is_err());
    }

    #[test]
    fn test_commit_allow_empty() {
        let (dir, repo) = setup_test_repo();

        // Create initial commit first
        std::fs::write(dir.path().join("init.txt"), "init").unwrap();
        repo.stage_file(Path::new("init.txt")).unwrap();

        let committer = GitCommitter::new(&repo);
        committer.commit(CommitOptions::new("Initial")).unwrap();

        // Now try empty commit
        let result = committer.commit(CommitOptions::new("Empty").allow_empty());
        assert!(result.is_ok());
    }

    #[test]
    fn test_commit_with_co_authors() {
        let (dir, repo) = setup_test_repo();

        std::fs::write(dir.path().join("test.txt"), "content").unwrap();
        repo.stage_file(Path::new("test.txt")).unwrap();

        let co_author = GitSignature::new("Alice", "alice@example.com");

        let committer = GitCommitter::new(&repo);
        let result = committer
            .commit(CommitOptions::new("Paired programming").co_author(co_author))
            .unwrap();

        assert!(result.message.contains("Co-authored-by: Alice <alice@example.com>"));
    }

    #[test]
    fn test_commit_dry_run() {
        let (dir, repo) = setup_test_repo();

        std::fs::write(dir.path().join("test.txt"), "content").unwrap();
        repo.stage_file(Path::new("test.txt")).unwrap();

        let committer = GitCommitter::new(&repo);
        let result = committer
            .commit(CommitOptions::new("Dry run").dry_run())
            .unwrap();

        // Verify no actual commit was created
        assert!(result.oid.is_zero());
    }

    #[test]
    fn test_parse_commit_message() {
        let message = "Subject line\n\nBody paragraph 1.\n\nBody paragraph 2.";
        let (subject, body) = CommitMessageParser::parse(message);

        assert_eq!(subject, "Subject line");
        assert!(body.is_some());
        assert!(body.unwrap().contains("Body paragraph"));
    }

    #[test]
    fn test_parse_conventional_commit() {
        let message = "feat(auth): add OAuth2 support\n\nImplement OAuth2 flow.";
        let parsed = CommitMessageParser::parse_conventional(message).unwrap();

        assert_eq!(parsed.commit_type, "feat");
        assert_eq!(parsed.scope, Some("auth".to_string()));
        assert!(!parsed.breaking);
        assert_eq!(parsed.description, "add OAuth2 support");
    }

    #[test]
    fn test_parse_breaking_change() {
        let message = "feat!: breaking change";
        let parsed = CommitMessageParser::parse_conventional(message).unwrap();

        assert!(parsed.breaking);
    }

    #[test]
    fn test_commit_template_manager() {
        let mut manager = CommitTemplateManager::new().with_conventional_defaults();

        let mut vars = std::collections::HashMap::new();
        vars.insert("scope".to_string(), "auth".to_string());
        vars.insert("description".to_string(), "add login".to_string());
        vars.insert("body".to_string(), "Details here.".to_string());

        let result = manager.apply("feat", &vars).unwrap();
        assert!(result.contains("feat(auth): add login"));
    }
}
```

---

## Related Specs

- Spec 446: Git Types
- Spec 448: Repository Operations
- Spec 449: Status Checking
- Spec 461: Git Hooks
