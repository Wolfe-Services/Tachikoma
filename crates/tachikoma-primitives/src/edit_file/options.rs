//! Options for edit_file primitive.

/// Options for editing a file.
#[derive(Debug, Clone, Default)]
pub struct EditFileOptions {
    /// Replace all occurrences (not just first unique match).
    pub replace_all: bool,
    /// Create a backup of the original file.
    pub backup: bool,
    /// Don't actually write changes (preview mode).
    pub dry_run: bool,
    /// Preserve original file permissions.
    pub preserve_permissions: bool,
}

impl EditFileOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace all occurrences.
    pub fn replace_all(mut self) -> Self {
        self.replace_all = true;
        self
    }

    /// Create backup before editing.
    pub fn with_backup(mut self) -> Self {
        self.backup = true;
        self
    }

    /// Preview changes without writing.
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Preserve file permissions.
    pub fn preserve_permissions(mut self) -> Self {
        self.preserve_permissions = true;
        self
    }
}