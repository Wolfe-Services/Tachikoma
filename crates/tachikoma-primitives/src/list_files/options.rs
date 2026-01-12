//! Options for list_files primitive.

/// Sorting options.
#[derive(Debug, Clone, Copy, Default)]
pub enum SortBy {
    /// Sort by filename.
    #[default]
    Name,
    /// Sort by file size.
    Size,
    /// Sort by file extension.
    Extension,
    /// Sort by type (directories first, then files).
    Type,
}

/// Options for listing files.
#[derive(Debug, Clone, Default)]
pub struct ListFilesOptions {
    /// Filter by file extension.
    pub extension: Option<String>,
    /// Filter by glob pattern.
    pub pattern: Option<String>,
    /// Include directories in output.
    pub include_dirs: bool,
    /// Only list directories.
    pub dirs_only: bool,
    /// Include hidden files (starting with .).
    pub include_hidden: bool,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Offset for pagination.
    pub offset: Option<usize>,
    /// Sort order.
    pub sort_by: SortBy,
    /// Reverse sort order.
    pub reverse: bool,
}

impl ListFilesOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by extension.
    pub fn extension(mut self, ext: &str) -> Self {
        self.extension = Some(ext.trim_start_matches('.').to_string());
        self
    }

    /// Filter by glob pattern.
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    /// Include directories in results.
    pub fn include_directories(mut self) -> Self {
        self.include_dirs = true;
        self
    }

    /// Only list directories.
    pub fn directories_only(mut self) -> Self {
        self.dirs_only = true;
        self.include_dirs = true;
        self
    }

    /// Include hidden files.
    pub fn include_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }

    /// Limit number of results.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination offset.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set sort order.
    pub fn sort(mut self, sort_by: SortBy) -> Self {
        self.sort_by = sort_by;
        self
    }

    /// Reverse sort order.
    pub fn reversed(mut self) -> Self {
        self.reverse = true;
        self
    }
}