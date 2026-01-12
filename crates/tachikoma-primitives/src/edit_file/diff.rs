//! Diff generation for edit preview.

use std::fmt;

/// A unified diff representation.
#[derive(Debug, Clone)]
pub struct Diff {
    /// Diff hunks.
    pub hunks: Vec<DiffHunk>,
}

/// A single diff hunk.
#[derive(Debug, Clone)]
pub struct DiffHunk {
    /// Starting line in old file.
    pub old_start: usize,
    /// Number of lines in old file.
    pub old_count: usize,
    /// Starting line in new file.
    pub new_start: usize,
    /// Number of lines in new file.
    pub new_count: usize,
    /// Lines in the hunk.
    pub lines: Vec<DiffLine>,
}

/// A single diff line.
#[derive(Debug, Clone)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}

impl fmt::Display for DiffLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffLine::Context(s) => write!(f, " {}", s),
            DiffLine::Added(s) => write!(f, "+{}", s),
            DiffLine::Removed(s) => write!(f, "-{}", s),
        }
    }
}

impl fmt::Display for DiffHunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_count, self.new_start, self.new_count
        )?;
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for hunk in &self.hunks {
            write!(f, "{}", hunk)?;
        }
        Ok(())
    }
}

/// Create a diff between two strings.
pub fn create_diff(old: &str, new: &str) -> Diff {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut hunks = Vec::new();
    let mut old_idx = 0;
    let mut new_idx = 0;

    while old_idx < old_lines.len() || new_idx < new_lines.len() {
        // Find next difference
        while old_idx < old_lines.len()
            && new_idx < new_lines.len()
            && old_lines[old_idx] == new_lines[new_idx]
        {
            old_idx += 1;
            new_idx += 1;
        }

        if old_idx >= old_lines.len() && new_idx >= new_lines.len() {
            break;
        }

        // Create hunk
        let hunk_old_start = old_idx.saturating_sub(2);
        let hunk_new_start = new_idx.saturating_sub(2);

        let mut lines = Vec::new();

        // Context before
        for i in hunk_old_start..old_idx {
            if i < old_lines.len() {
                lines.push(DiffLine::Context(old_lines[i].to_string()));
            }
        }

        // Find extent of changes
        let mut old_end = old_idx;
        let mut new_end = new_idx;

        while old_end < old_lines.len() || new_end < new_lines.len() {
            if old_end < old_lines.len()
                && new_end < new_lines.len()
                && old_lines[old_end] == new_lines[new_end]
            {
                // Check if we have enough context to end hunk
                let mut context_count = 0;
                let mut check_old = old_end;
                let mut check_new = new_end;
                while check_old < old_lines.len()
                    && check_new < new_lines.len()
                    && old_lines[check_old] == new_lines[check_new]
                {
                    context_count += 1;
                    check_old += 1;
                    check_new += 1;
                    if context_count >= 4 {
                        break;
                    }
                }
                if context_count >= 4 || (check_old >= old_lines.len() && check_new >= new_lines.len()) {
                    break;
                }
            }
            old_end += 1;
            new_end += 1;
        }

        // Add removed lines
        for i in old_idx..old_end.min(old_lines.len()) {
            lines.push(DiffLine::Removed(old_lines[i].to_string()));
        }

        // Add added lines
        for i in new_idx..new_end.min(new_lines.len()) {
            lines.push(DiffLine::Added(new_lines[i].to_string()));
        }

        // Context after
        let context_end = old_end + 2;
        for i in old_end..context_end.min(old_lines.len()) {
            lines.push(DiffLine::Context(old_lines[i].to_string()));
        }

        if !lines.is_empty() {
            hunks.push(DiffHunk {
                old_start: hunk_old_start + 1,
                old_count: old_end - hunk_old_start,
                new_start: hunk_new_start + 1,
                new_count: new_end - hunk_new_start,
                lines,
            });
        }

        old_idx = old_end + 2;
        new_idx = new_end + 2;
    }

    Diff { hunks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_diff() {
        let old = "line1\nline2\nline3";
        let new = "line1\nmodified\nline3";

        let diff = create_diff(old, new);
        let formatted = diff.to_string();

        assert!(formatted.contains("-line2"));
        assert!(formatted.contains("+modified"));
    }

    #[test]
    fn test_no_changes() {
        let content = "line1\nline2\nline3";
        let diff = create_diff(content, content);

        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn test_multiline_change() {
        let old = "fn old() {\n    println!(\"old\");\n}";
        let new = "fn new() {\n    println!(\"new\");\n}";

        let diff = create_diff(old, new);
        let formatted = diff.to_string();

        assert!(formatted.contains("-fn old()"));
        assert!(formatted.contains("+fn new()"));
        assert!(formatted.contains("-    println!(\"old\");"));
        assert!(formatted.contains("+    println!(\"new\");"));
    }

    #[test]
    fn test_context_lines() {
        let old = "line1\nline2\nline3\nline4\nline5";
        let new = "line1\nline2\nmodified\nline4\nline5";

        let diff = create_diff(old, new);
        let formatted = diff.to_string();

        // Should include context lines around the change
        assert!(formatted.contains(" line1") || formatted.contains(" line2"));
        assert!(formatted.contains(" line4") || formatted.contains(" line5"));
        assert!(formatted.contains("-line3"));
        assert!(formatted.contains("+modified"));
    }
}