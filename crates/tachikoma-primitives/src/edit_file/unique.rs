//! Uniqueness validation for edit_file.

/// Result of uniqueness check.
#[derive(Debug, Clone)]
pub struct UniquenessResult {
    /// Whether the target is unique.
    pub is_unique: bool,
    /// Number of matches found.
    pub match_count: usize,
    /// Details of each match.
    pub matches: Vec<MatchLocation>,
    /// Suggested expanded context if not unique.
    pub suggestion: Option<String>,
}

/// Location of a match in the file.
#[derive(Debug, Clone)]
pub struct MatchLocation {
    /// Line number (1-indexed).
    pub line: usize,
    /// Column (1-indexed).
    pub column: usize,
    /// Byte offset from start of file.
    pub offset: usize,
    /// Context lines before the match.
    pub context_before: Vec<String>,
    /// The matched line(s).
    pub matched_lines: Vec<String>,
    /// Context lines after the match.
    pub context_after: Vec<String>,
}

impl MatchLocation {
    /// Format the match with context.
    pub fn format_with_context(&self) -> String {
        let mut lines = Vec::new();

        let context_start = self.line.saturating_sub(self.context_before.len());

        for (i, line) in self.context_before.iter().enumerate() {
            lines.push(format!("{:>4} | {}", context_start + i, line));
        }

        for (i, line) in self.matched_lines.iter().enumerate() {
            lines.push(format!("{:>4} > {}", self.line + i, line));
        }

        for (i, line) in self.context_after.iter().enumerate() {
            let line_num = self.line + self.matched_lines.len() + i;
            lines.push(format!("{:>4} | {}", line_num, line));
        }

        lines.join("\n")
    }
}

/// Check if a target string is unique in content.
pub fn check_uniqueness(content: &str, target: &str, context_lines: usize) -> UniquenessResult {
    let lines: Vec<&str> = content.lines().collect();
    let mut matches = Vec::new();
    let mut byte_offset = 0;

    // Handle multiline targets differently
    if target.contains('\n') {
        // For multiline targets, search in the full content
        let mut search_start = 0;
        while let Some(pos) = content[search_start..].find(target) {
            let absolute_pos = search_start + pos;
            
            // Find which line this match starts on
            let content_up_to_match = &content[..absolute_pos];
            let line_number = content_up_to_match.matches('\n').count() + 1;
            
            // Calculate column position
            let line_start = content_up_to_match.rfind('\n').map(|p| p + 1).unwrap_or(0);
            let column = absolute_pos - line_start + 1;
            
            // Get the matched lines
            let target_lines: Vec<&str> = target.lines().collect();
            let matched_lines: Vec<String> = target_lines.iter().map(|s| s.to_string()).collect();
            
            // Get context
            let start_line_idx = line_number.saturating_sub(1); // Convert to 0-indexed
            let context_before: Vec<String> = lines
                [start_line_idx.saturating_sub(context_lines)..start_line_idx]
                .iter()
                .map(|s| s.to_string())
                .collect();

            let end_line_idx = start_line_idx + target_lines.len();
            let context_after: Vec<String> = lines
                [end_line_idx.min(lines.len())..(end_line_idx + context_lines).min(lines.len())]
                .iter()
                .map(|s| s.to_string())
                .collect();

            matches.push(MatchLocation {
                line: line_number,
                column,
                offset: absolute_pos,
                context_before,
                matched_lines,
                context_after,
            });

            search_start = absolute_pos + target.len().max(1);
        }
    } else {
        // For single-line targets, search line by line
        for (line_idx, line) in lines.iter().enumerate() {
            let mut search_start = 0;
            while let Some(col) = line[search_start..].find(target) {
                let actual_col = search_start + col;
                
                let matched_lines = vec![line.to_string()];

                // Get context
                let context_before: Vec<String> = lines
                    [line_idx.saturating_sub(context_lines)..line_idx]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                let context_after: Vec<String> = lines
                    [(line_idx + 1)..(line_idx + 1 + context_lines).min(lines.len())]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                matches.push(MatchLocation {
                    line: line_idx + 1, // 1-indexed
                    column: actual_col + 1, // 1-indexed
                    offset: byte_offset + actual_col,
                    context_before,
                    matched_lines,
                    context_after,
                });

                search_start = actual_col + target.len().max(1);
            }

            byte_offset += line.len() + 1; // +1 for newline
        }
    }

    let is_unique = matches.len() == 1;
    let suggestion = if !is_unique && matches.len() > 1 {
        suggest_unique_context(&matches, target)
    } else {
        None
    };

    UniquenessResult {
        is_unique,
        match_count: matches.len(),
        matches,
        suggestion,
    }
}

/// Suggest expanded context to make the match unique.
fn suggest_unique_context(matches: &[MatchLocation], target: &str) -> Option<String> {
    if matches.len() < 2 {
        return None;
    }

    // Try to find distinguishing context
    let first = &matches[0];

    // Check if adding context before makes it unique
    if !first.context_before.is_empty() {
        let expanded = format!(
            "{}\n{}",
            first.context_before.last().unwrap_or(&String::new()),
            target
        );
        return Some(format!(
            "Consider including the line before:\n{}",
            expanded.lines().take(3).collect::<Vec<_>>().join("\n")
        ));
    }

    // Check if adding context after makes it unique
    if !first.context_after.is_empty() {
        let expanded = format!(
            "{}\n{}",
            target,
            first.context_after.first().unwrap_or(&String::new())
        );
        return Some(format!(
            "Consider including the line after:\n{}",
            expanded.lines().take(3).collect::<Vec<_>>().join("\n")
        ));
    }

    Some(format!(
        "Found {} matches. Consider expanding the search string to include more context.",
        matches.len()
    ))
}

/// Format all matches for display.
pub fn format_matches(result: &UniquenessResult) -> String {
    let mut output = Vec::new();

    output.push(format!(
        "Found {} match{}:",
        result.match_count,
        if result.match_count == 1 { "" } else { "es" }
    ));

    for (i, m) in result.matches.iter().enumerate() {
        output.push(format!("\nMatch {} at line {}, column {}:", i + 1, m.line, m.column));
        output.push(m.format_with_context());
    }

    if let Some(ref suggestion) = result.suggestion {
        output.push(format!("\nSuggestion: {}", suggestion));
    }

    output.join("\n")
}

/// Find the best match given explicit selection.
pub fn select_match(result: &UniquenessResult, selection: MatchSelection) -> Option<&MatchLocation> {
    match selection {
        MatchSelection::First => result.matches.first(),
        MatchSelection::Last => result.matches.last(),
        MatchSelection::Index(i) => result.matches.get(i),
        MatchSelection::Line(line) => {
            result.matches.iter().find(|m| m.line == line)
        }
    }
}

/// Selection criteria for non-unique matches.
#[derive(Debug, Clone, Copy)]
pub enum MatchSelection {
    /// Select the first match.
    First,
    /// Select the last match.
    Last,
    /// Select by index (0-indexed).
    Index(usize),
    /// Select by line number.
    Line(usize),
}

/// Validate that a target can be safely edited.
pub fn validate_edit_target(
    content: &str,
    target: &str,
    allow_multiple: bool,
) -> Result<UniquenessResult, EditValidationError> {
    let result = check_uniqueness(content, target, 3);

    if result.match_count == 0 {
        return Err(EditValidationError::TargetNotFound);
    }

    if !result.is_unique && !allow_multiple {
        return Err(EditValidationError::NotUnique {
            count: result.match_count,
            formatted: format_matches(&result),
        });
    }

    Ok(result)
}

/// Errors from edit validation.
#[derive(Debug, Clone)]
pub enum EditValidationError {
    /// Target string not found.
    TargetNotFound,
    /// Target string not unique.
    NotUnique {
        /// Number of matches found.
        count: usize,
        /// Formatted match details.
        formatted: String,
    },
}

impl std::fmt::Display for EditValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetNotFound => write!(f, "Target string not found in file"),
            Self::NotUnique { count, formatted } => {
                write!(f, "Target string not unique ({} matches):\n{}", count, formatted)
            }
        }
    }
}

impl std::error::Error for EditValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_match() {
        let content = "line1\nunique_target\nline3";
        let result = check_uniqueness(content, "unique_target", 2);

        assert!(result.is_unique);
        assert_eq!(result.match_count, 1);
        assert_eq!(result.matches[0].line, 2);
    }

    #[test]
    fn test_multiple_matches() {
        let content = "foo bar\nbaz foo\nfoo qux";
        let result = check_uniqueness(content, "foo", 2);

        assert!(!result.is_unique);
        assert_eq!(result.match_count, 3);
    }

    #[test]
    fn test_no_matches() {
        let content = "line1\nline2\nline3";
        let result = check_uniqueness(content, "notfound", 2);

        assert!(!result.is_unique); // Not unique because no matches found
        assert_eq!(result.match_count, 0);
    }

    #[test]
    fn test_context_capture() {
        let content = "context1\ncontext2\ntarget\ncontext3\ncontext4";
        let result = check_uniqueness(content, "target", 2);

        assert_eq!(result.matches[0].context_before.len(), 2);
        assert_eq!(result.matches[0].context_after.len(), 2);
    }

    #[test]
    fn test_select_match() {
        let content = "foo\nbar\nfoo\nbaz\nfoo";
        let result = check_uniqueness(content, "foo", 1);

        let first = select_match(&result, MatchSelection::First).unwrap();
        assert_eq!(first.line, 1);

        let last = select_match(&result, MatchSelection::Last).unwrap();
        assert_eq!(last.line, 5);

        let by_index = select_match(&result, MatchSelection::Index(1)).unwrap();
        assert_eq!(by_index.line, 3);
    }

    #[test]
    fn test_validate_unique() {
        let content = "unique line here";
        let result = validate_edit_target(content, "unique", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_unique() {
        let content = "foo bar foo";
        let result = validate_edit_target(content, "foo", false);
        assert!(matches!(result, Err(EditValidationError::NotUnique { .. })));
    }

    #[test]
    fn test_multiline_match() {
        let content = "start\nfirst line\nsecond line\nend";
        let target = "first line\nsecond line";
        let result = check_uniqueness(content, target, 1);

        assert!(result.is_unique);
        assert_eq!(result.match_count, 1);
        assert_eq!(result.matches[0].line, 2);
        assert_eq!(result.matches[0].matched_lines.len(), 2);
    }

    #[test]
    fn test_line_column_reporting() {
        let content = "abc def foo ghi\njkl foo mno";
        let result = check_uniqueness(content, "foo", 0);

        assert!(!result.is_unique);
        assert_eq!(result.match_count, 2);
        
        // First match at line 1, column 9
        assert_eq!(result.matches[0].line, 1);
        assert_eq!(result.matches[0].column, 9);
        
        // Second match at line 2, column 5
        assert_eq!(result.matches[1].line, 2);
        assert_eq!(result.matches[1].column, 5);
    }

    #[test]
    fn test_format_matches() {
        let content = "line 1\nfoo here\nline 3\nfoo there";
        let result = check_uniqueness(content, "foo", 1);
        let formatted = format_matches(&result);

        assert!(formatted.contains("Found 2 matches:"));
        assert!(formatted.contains("Match 1 at line 2, column 1:"));
        assert!(formatted.contains("Match 2 at line 4, column 1:"));
        assert!(formatted.contains("Suggestion:"));
    }

    #[test]
    fn test_match_selection_by_line() {
        let content = "foo\nbar\nfoo again\nbaz\nfoo final";
        let result = check_uniqueness(content, "foo", 1);

        let match_at_line_3 = select_match(&result, MatchSelection::Line(3)).unwrap();
        assert_eq!(match_at_line_3.line, 3);
        assert!(match_at_line_3.matched_lines[0].contains("again"));
    }
}