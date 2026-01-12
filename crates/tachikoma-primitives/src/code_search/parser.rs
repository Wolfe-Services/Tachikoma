//! Ripgrep JSON output parser.

use crate::{error::PrimitiveError, result::SearchMatch};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Ripgrep JSON output structure.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepOutput {
    /// Type of output entry.
    #[serde(rename = "type")]
    pub entry_type: String,
    /// Data payload.
    pub data: Option<RipgrepData>,
}

/// Ripgrep data payload.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepData {
    /// File path.
    pub path: Option<RipgrepPath>,
    /// Line information.
    pub lines: Option<RipgrepLines>,
    /// Line number.
    pub line_number: Option<u64>,
    /// Absolute byte offset.
    pub absolute_offset: Option<u64>,
    /// Submatches.
    pub submatches: Option<Vec<RipgrepSubmatch>>,
}

/// Ripgrep path information.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepPath {
    /// Text representation of the path.
    pub text: String,
}

/// Ripgrep lines information.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepLines {
    /// Line text.
    pub text: String,
}

/// Ripgrep submatch information.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepSubmatch {
    /// Match text.
    #[serde(rename = "match")]
    pub match_text: RipgrepMatch,
    /// Start byte offset.
    pub start: u64,
    /// End byte offset.
    pub end: u64,
}

/// Ripgrep match text.
#[derive(Debug, Clone, Deserialize)]
pub struct RipgrepMatch {
    /// Matched text.
    pub text: String,
}

/// Parse ripgrep JSON output into search matches.
pub fn parse_ripgrep_output(
    output: &str,
    max_matches: usize,
) -> Result<(Vec<SearchMatch>, usize, bool), PrimitiveError> {
    let mut matches = Vec::new();
    let mut context_before = Vec::new();
    let mut context_after = Vec::new();
    let mut collecting_after = false;
    let mut current_match: Option<SearchMatch> = None;
    let mut total_count = 0;

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let entry: RipgrepOutput = serde_json::from_str(line).map_err(|e| {
            PrimitiveError::Validation {
                message: format!("Failed to parse ripgrep output: {}", e),
            }
        })?;

        match entry.entry_type.as_str() {
            "match" => {
                // Save any previous match that was being built
                if let Some(mut m) = current_match.take() {
                    m.context_after = context_after.clone();
                    if matches.len() < max_matches {
                        matches.push(m);
                    }
                    context_after.clear();
                }

                if let Some(data) = entry.data {
                    if let (Some(path), Some(lines), Some(line_number)) =
                        (data.path, data.lines, data.line_number)
                    {
                        let column = if let Some(submatches) = data.submatches {
                            if let Some(submatch) = submatches.first() {
                                // Calculate column from byte offset
                                let line_start = lines.text.as_bytes();
                                let match_start = submatch.start as usize;
                                
                                // Find the column position (1-indexed)
                                let mut column = 1;
                                let mut byte_pos = 0;
                                for ch in lines.text.chars() {
                                    if byte_pos >= match_start {
                                        break;
                                    }
                                    byte_pos += ch.len_utf8();
                                    column += 1;
                                }
                                column
                            } else {
                                1
                            }
                        } else {
                            1
                        };

                        current_match = Some(SearchMatch {
                            path: PathBuf::from(path.text),
                            line_number: line_number as usize,
                            column,
                            line_content: lines.text,
                            context_before: context_before.clone(),
                            context_after: Vec::new(),
                        });

                        total_count += 1;
                        context_before.clear();
                        collecting_after = true;
                    }
                }
            }
            "context" => {
                if let Some(data) = entry.data {
                    if let Some(lines) = data.lines {
                        if collecting_after {
                            context_after.push(lines.text);
                        } else {
                            context_before.push(lines.text);
                        }
                    }
                }
            }
            "summary" => {
                // End of output
                if let Some(mut m) = current_match.take() {
                    m.context_after = context_after.clone();
                    if matches.len() < max_matches {
                        matches.push(m);
                    }
                }
                break;
            }
            _ => {
                // Ignore other entry types
            }
        }
    }

    // Handle case where there's no summary at the end
    if let Some(mut m) = current_match {
        m.context_after = context_after;
        if matches.len() < max_matches {
            matches.push(m);
        }
    }

    let truncated = total_count > max_matches;

    Ok((matches, total_count, truncated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match() {
        let json_output = r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}
{"type":"summary","data":{"elapsed_total":{"human":"0.000s","nanos":123456},"stats":{"elapsed":{"human":"0.000s","nanos":123456},"searches":1,"searches_with_match":1,"bytes_searched":100,"bytes_printed":50,"matched_lines":1,"matches":1}}}"#;

        let (matches, total_count, truncated) = parse_ripgrep_output(json_output, 100).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(total_count, 1);
        assert!(!truncated);

        let m = &matches[0];
        assert_eq!(m.path, PathBuf::from("test.rs"));
        assert_eq!(m.line_number, 1);
        assert_eq!(m.column, 4); // "main" starts at column 4 in "fn main() {"
        assert_eq!(m.line_content, "fn main() {");
    }

    #[test]
    fn test_parse_with_context() {
        let json_output = r#"{"type":"context","data":{"path":{"text":"test.rs"},"lines":{"text":"// Comment above"},"line_number":1,"absolute_offset":0}}
{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {"},"line_number":2,"absolute_offset":16,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}
{"type":"context","data":{"path":{"text":"test.rs"},"lines":{"text":"    println!(\"Hello\");"},"line_number":3,"absolute_offset":28}}
{"type":"summary","data":{"elapsed_total":{"human":"0.000s","nanos":123456}}}"#;

        let (matches, total_count, truncated) = parse_ripgrep_output(json_output, 100).unwrap();

        assert_eq!(matches.len(), 1);
        let m = &matches[0];
        assert_eq!(m.context_before.len(), 1);
        assert_eq!(m.context_before[0], "// Comment above");
        assert_eq!(m.context_after.len(), 1);
        assert_eq!(m.context_after[0], "    println!(\"Hello\");");
    }

    #[test]
    fn test_parse_no_matches() {
        let json_output = r#"{"type":"summary","data":{"elapsed_total":{"human":"0.000s","nanos":123456},"stats":{"elapsed":{"human":"0.000s","nanos":123456},"searches":1,"searches_with_match":0,"bytes_searched":100,"bytes_printed":0,"matched_lines":0,"matches":0}}}"#;

        let (matches, total_count, truncated) = parse_ripgrep_output(json_output, 100).unwrap();

        assert_eq!(matches.len(), 0);
        assert_eq!(total_count, 0);
        assert!(!truncated);
    }

    #[test]
    fn test_parse_truncated_results() {
        let json_output = r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}
{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn test() {"},"line_number":5,"absolute_offset":50,"submatches":[{"match":{"text":"test"},"start":3,"end":7}]}}
{"type":"summary","data":{"elapsed_total":{"human":"0.000s","nanos":123456}}}"#;

        let (matches, total_count, truncated) = parse_ripgrep_output(json_output, 1).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(total_count, 2);
        assert!(truncated);
    }
}