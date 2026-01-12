//! Options for code_search primitive.

/// Options for code search.
#[derive(Debug, Clone, Default)]
pub struct CodeSearchOptions {
    /// Filter by file type (e.g., "rust", "python").
    pub file_type: Option<String>,
    /// Glob patterns to include.
    pub globs: Vec<String>,
    /// Lines of context before match.
    pub context_before: usize,
    /// Lines of context after match.
    pub context_after: usize,
    /// Case insensitive search.
    pub case_insensitive: bool,
    /// Smart case (case-insensitive if pattern is all lowercase).
    pub smart_case: bool,
    /// Don't respect gitignore.
    pub no_ignore: bool,
    /// Include hidden files.
    pub include_hidden: bool,
    /// Maximum number of matches.
    pub max_matches: Option<usize>,
    /// Search in file names only.
    pub files_only: bool,
}

impl CodeSearchOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by file type.
    pub fn file_type(mut self, ft: &str) -> Self {
        self.file_type = Some(ft.to_string());
        self
    }

    /// Add glob pattern.
    pub fn glob(mut self, pattern: &str) -> Self {
        self.globs.push(pattern.to_string());
        self
    }

    /// Set context lines (before and after).
    pub fn context(mut self, lines: usize) -> Self {
        self.context_before = lines;
        self.context_after = lines;
        self
    }

    /// Set context lines separately.
    pub fn context_lines(mut self, before: usize, after: usize) -> Self {
        self.context_before = before;
        self.context_after = after;
        self
    }

    /// Enable case insensitive search.
    pub fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    /// Enable smart case.
    pub fn smart_case(mut self) -> Self {
        self.smart_case = true;
        self
    }

    /// Don't respect gitignore.
    pub fn no_ignore(mut self) -> Self {
        self.no_ignore = true;
        self
    }

    /// Include hidden files.
    pub fn include_hidden(mut self) -> Self {
        self.include_hidden = true;
        self
    }

    /// Set maximum matches.
    pub fn max_matches(mut self, max: usize) -> Self {
        self.max_matches = Some(max);
        self
    }

    /// Search file names only.
    pub fn files_only(mut self) -> Self {
        self.files_only = true;
        self
    }
}