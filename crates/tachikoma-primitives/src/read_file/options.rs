//! Options for read_file primitive.

/// Options for reading a file.
#[derive(Debug, Clone, Default)]
pub struct ReadFileOptions {
    /// Starting line number (1-indexed).
    pub start_line: Option<usize>,
    /// Ending line number (1-indexed, inclusive).
    pub end_line: Option<usize>,
    /// Maximum size to read in bytes.
    pub max_size: Option<usize>,
    /// Include line numbers in output.
    pub show_line_numbers: bool,
}

impl ReadFileOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set line range to read.
    pub fn lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set start line only.
    pub fn from_line(mut self, start: usize) -> Self {
        self.start_line = Some(start);
        self
    }

    /// Set maximum size.
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Include line numbers in output.
    pub fn with_line_numbers(mut self) -> Self {
        self.show_line_numbers = true;
        self
    }
}