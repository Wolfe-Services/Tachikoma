//! Options for edit_file primitive.

use crate::edit_file::unique::MatchSelection;

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
    /// Force edit with specific match selection when not unique.
    pub force_selection: Option<MatchSelection>,
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

    /// Force edit with specific match selection.
    pub fn force_match(mut self, selection: MatchSelection) -> Self {
        self.force_selection = Some(selection);
        self
    }

    /// Force edit of first match.
    pub fn force_first(mut self) -> Self {
        self.force_selection = Some(MatchSelection::First);
        self
    }

    /// Force edit of last match.
    pub fn force_last(mut self) -> Self {
        self.force_selection = Some(MatchSelection::Last);
        self
    }

    /// Force edit of match at specific line.
    pub fn force_line(mut self, line: usize) -> Self {
        self.force_selection = Some(MatchSelection::Line(line));
        self
    }

    /// Force edit of match by index.
    pub fn force_index(mut self, index: usize) -> Self {
        self.force_selection = Some(MatchSelection::Index(index));
        self
    }
}