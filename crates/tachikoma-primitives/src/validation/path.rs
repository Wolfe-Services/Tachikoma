//! Path validation utilities.

use super::{ValidationError, ValidationErrors};
use std::path::{Component, Path, PathBuf};

/// Path validator.
pub struct PathValidator {
    /// Allowed base paths.
    allowed_paths: Vec<PathBuf>,
    /// Denied paths.
    denied_paths: Vec<PathBuf>,
    /// Allow absolute paths.
    allow_absolute: bool,
    /// Allow path traversal (../).
    allow_traversal: bool,
    /// Maximum path length.
    max_length: usize,
}

impl Default for PathValidator {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            denied_paths: vec![
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/root"),
            ],
            allow_absolute: true,
            allow_traversal: false,
            max_length: 4096,
        }
    }
}

impl PathValidator {
    /// Create a new path validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an allowed path.
    pub fn allow(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Add a denied path.
    pub fn deny(mut self, path: impl Into<PathBuf>) -> Self {
        self.denied_paths.push(path.into());
        self
    }

    /// Disallow absolute paths.
    pub fn no_absolute(mut self) -> Self {
        self.allow_absolute = false;
        self
    }

    /// Allow path traversal.
    pub fn allow_traversal(mut self) -> Self {
        self.allow_traversal = true;
        self
    }

    /// Validate a path string.
    pub fn validate(&self, path: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let path = Path::new(path);

        // Check length
        if path.as_os_str().len() > self.max_length {
            errors.add(ValidationError::new(
                "path",
                &format!("exceeds maximum length of {}", self.max_length),
                "max_length",
            ));
        }

        // Check absolute
        if !self.allow_absolute && path.is_absolute() {
            errors.add(ValidationError::new(
                "path",
                "absolute paths are not allowed",
                "no_absolute",
            ).with_suggestion("Use a relative path instead"));
        }

        // Check traversal
        if !self.allow_traversal {
            for component in path.components() {
                if matches!(component, Component::ParentDir) {
                    errors.add(ValidationError::new(
                        "path",
                        "path traversal (../) is not allowed",
                        "no_traversal",
                    ).with_suggestion("Use an absolute path or stay within the working directory"));
                    break;
                }
            }
        }

        // Check denied paths
        let canonical = self.normalize_path(path);
        for denied in &self.denied_paths {
            if canonical.starts_with(denied) {
                errors.add(ValidationError::new(
                    "path",
                    &format!("access to {:?} is denied", denied),
                    "denied_path",
                ));
            }
        }

        // Check allowed paths (if any specified)
        if !self.allowed_paths.is_empty() {
            let is_allowed = self.allowed_paths.iter().any(|allowed| {
                canonical.starts_with(allowed)
            });
            if !is_allowed {
                errors.add(ValidationError::new(
                    "path",
                    "path is not in allowed directories",
                    "allowed_path",
                ).with_suggestion(&format!(
                    "Allowed paths: {:?}",
                    self.allowed_paths
                )));
            }
        }

        errors
    }

    /// Normalize a path for comparison.
    fn normalize_path(&self, path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    normalized.pop();
                }
                Component::CurDir => {}
                _ => {
                    normalized.push(component);
                }
            }
        }
        normalized
    }

    /// Validate and resolve a path relative to a base.
    pub fn validate_and_resolve(
        &self,
        path: &str,
        base: &Path,
    ) -> Result<PathBuf, ValidationErrors> {
        let errors = self.validate(path);
        if !errors.is_empty() {
            return Err(errors);
        }

        let path = Path::new(path);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        };

        // Re-validate resolved path
        let errors = self.validate(resolved.to_string_lossy().as_ref());
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(resolved)
    }
}

/// Check for path traversal attempts.
pub fn has_path_traversal(path: &str) -> bool {
    let path = Path::new(path);
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Sanitize a filename (remove directory components).
pub fn sanitize_filename(name: &str) -> String {
    Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_validation() {
        let validator = PathValidator::new();
        let errors = validator.validate("src/main.rs");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_traversal_detection() {
        let validator = PathValidator::new();
        let errors = validator.validate("../../../etc/passwd");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_denied_path() {
        let validator = PathValidator::new();
        let errors = validator.validate("/etc/shadow");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_allowed_paths() {
        let validator = PathValidator::new()
            .allow("/project");

        let errors = validator.validate("/project/src/main.rs");
        assert!(errors.is_empty());

        let errors = validator.validate("/other/file.txt");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("file.txt"), "file.txt");
        assert_eq!(sanitize_filename("/path/to/file.txt"), "file.txt");
        assert_eq!(sanitize_filename("../file.txt"), "file.txt");
    }
}