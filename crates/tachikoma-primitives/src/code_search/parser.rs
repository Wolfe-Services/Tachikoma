//! Ripgrep JSON output parser.

use crate::{error::PrimitiveError, result::SearchMatch};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Ripgrep JSON message types.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RipgrepMessage {
    Begin {
        data: BeginData,
    },
    #[serde(rename = "match")]
    Match {
        data: MatchData,
    },
    Context {
        data: ContextData,
    },
    End {
        data: EndData,
    },
    Summary {
        data: SummaryData,
    },
}

/// Begin data.
#[derive(Debug, Deserialize)]
pub struct BeginData {
    pub path: RipgrepText,
}

/// Match data.
#[derive(Debug, Deserialize)]
pub struct MatchData {
    pub path: RipgrepText,
    pub lines: RipgrepText,
    pub line_number: u64,
    pub absolute_offset: u64,
    pub submatches: Vec<Submatch>,
}

/// Context data.
#[derive(Debug, Deserialize)]
pub struct ContextData {
    pub path: RipgrepText,
    pub lines: RipgrepText,
    pub line_number: u64,
    pub absolute_offset: u64,
    pub submatches: Vec<Submatch>,
}

/// End data.
#[derive(Debug, Deserialize)]
pub struct EndData {
    pub path: RipgrepText,
    pub binary_offset: Option<u64>,
    pub stats: FileStats,
}

/// Summary data.
#[derive(Debug, Deserialize)]
pub struct SummaryData {
    pub elapsed_total: ElapsedTime,
    pub stats: SummaryStats,
}

/// Text content from ripgrep.
#[derive(Debug, Deserialize)]
pub struct RipgrepText {
    pub text: String,
}

/// Submatch information.
#[derive(Debug, Deserialize)]
pub struct Submatch {
    #[serde(rename = "match")]
    pub matched: RipgrepText,
    pub start: usize,
    pub end: usize,
}

/// Per-file statistics.
#[derive(Debug, Deserialize)]
pub struct FileStats {
    pub elapsed: ElapsedTime,
    pub searches: u64,
    pub searches_with_match: u64,
    pub bytes_searched: u64,
    pub bytes_printed: u64,
    pub matched_lines: u64,
    pub matches: u64,
}

/// Summary statistics.
#[derive(Debug, Deserialize)]
pub struct SummaryStats {
    pub elapsed: ElapsedTime,
    pub searches: u64,
    pub searches_with_match: u64,
    pub bytes_searched: u64,
    pub bytes_printed: u64,
    pub matched_lines: u64,
    pub matches: u64,
}

/// Elapsed time.
#[derive(Debug, Deserialize)]
pub struct ElapsedTime {
    pub human: String,
    pub nanos: u64,
    pub secs: u64,
}

/// Accumulator for context lines.
struct ContextAccumulator {
    before: Vec<(usize, String)>,
    after: Vec<(usize, String)>,
    last_match_line: Option<usize>,
}

impl ContextAccumulator {
    fn new() -> Self {
        Self {
            before: Vec::new(),
            after: Vec::new(),
            last_match_line: None,
        }
    }

    fn add_context(&mut self, line_number: usize, content: String) {
        if let Some(last) = self.last_match_line {
            if line_number > last {
                self.after.push((line_number, content));
            } else {
                self.before.push((line_number, content));
            }
        } else {
            self.before.push((line_number, content));
        }
    }

    fn set_match_line(&mut self, line_number: usize) {
        self.last_match_line = Some(line_number);
    }

    fn finish(self) -> (Vec<String>, Vec<String>) {
        let before = self.before.into_iter().map(|(_, s)| s).collect();
        let after = self.after.into_iter().map(|(_, s)| s).collect();
        (before, after)
    }
}

/// Parsed ripgrep output.
#[derive(Debug)]
pub struct RipgrepOutput {
    pub matches: Vec<RipgrepMatch>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub bytes_searched: u64,
}

/// A single match from ripgrep.
#[derive(Debug, Clone)]
pub struct RipgrepMatch {
    /// File path.
    pub path: PathBuf,
    /// Line number (1-indexed).
    pub line_number: usize,
    /// Column number (1-indexed).
    pub column: usize,
    /// The matched line content.
    pub line: String,
    /// Start offset of match within line.
    pub match_start: usize,
    /// End offset of match within line.
    pub match_end: usize,
    /// The matched text.
    pub matched_text: String,
    /// Context lines before.
    pub context_before: Vec<String>,
    /// Context lines after.
    pub context_after: Vec<String>,
}

impl RipgrepMatch {
    /// Format match with highlighting.
    pub fn format_highlighted(&self) -> String {
        let line = &self.line;
        let start = self.match_start;
        let end = self.match_end.min(line.len());

        if start >= line.len() {
            return line.clone();
        }

        format!(
            "{}[{}]{}",
            &line[..start],
            &line[start..end],
            &line[end..]
        )
    }

    /// Format with context.
    pub fn format_with_context(&self) -> String {
        let mut lines = Vec::new();

        let start_line = self.line_number.saturating_sub(self.context_before.len());

        for (i, ctx) in self.context_before.iter().enumerate() {
            lines.push(format!("{:>4} | {}", start_line + i, ctx));
        }

        lines.push(format!("{:>4} > {}", self.line_number, self.format_highlighted()));

        for (i, ctx) in self.context_after.iter().enumerate() {
            lines.push(format!("{:>4} | {}", self.line_number + 1 + i, ctx));
        }

        lines.join("\n")
    }
}

/// Parse ripgrep JSON output into search matches.
pub fn parse_ripgrep_output(
    output: &str,
    max_matches: usize,
) -> Result<(Vec<SearchMatch>, usize, bool), PrimitiveError> {
    let mut matches = Vec::new();
    let mut context_map: HashMap<PathBuf, ContextAccumulator> = HashMap::new();
    let mut total_count = 0;
    let mut truncated = false;
    let mut multi_line_buffer: HashMap<PathBuf, Vec<(usize, String)>> = HashMap::new();

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let message: RipgrepMessage = match serde_json::from_str(line) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to parse ripgrep JSON line: {}", e);
                continue;
            }
        };

        match message {
            RipgrepMessage::Begin { data } => {
                debug!("Starting file: {:?}", data.path.text);
                let file_path = PathBuf::from(&data.path.text);
                context_map.insert(file_path.clone(), ContextAccumulator::new());
                multi_line_buffer.insert(file_path, Vec::new());
            }
            RipgrepMessage::Match { data } => {
                total_count += 1;

                if matches.len() >= max_matches {
                    truncated = true;
                    continue;
                }

                let file_path = PathBuf::from(&data.path.text);

                // Get or create context accumulator
                let context_acc = context_map.entry(file_path.clone()).or_insert_with(ContextAccumulator::new);
                context_acc.set_match_line(data.line_number as usize);

                // Handle multi-line matches
                let buffer = multi_line_buffer.entry(file_path.clone()).or_insert_with(Vec::new);
                buffer.push((data.line_number as usize, data.lines.text.clone()));

                // Process submatches for column and highlighting
                let (column, match_start, match_end, matched_text) = if let Some(submatch) = data.submatches.first() {
                    // Calculate column from byte offset to character offset
                    let line_text = &data.lines.text;
                    let byte_start = submatch.start;
                    let byte_end = submatch.end;

                    let mut column = 1;
                    let mut char_start = 0;
                    let mut char_end = line_text.len();
                    let mut byte_pos = 0;

                    for (char_idx, ch) in line_text.char_indices() {
                        if byte_pos == byte_start {
                            char_start = char_idx;
                            column = line_text[..char_idx].chars().count() + 1;
                        }
                        if byte_pos == byte_end {
                            char_end = char_idx;
                            break;
                        }
                        byte_pos += ch.len_utf8();
                        if byte_pos > byte_end {
                            char_end = char_idx + ch.len_utf8();
                            break;
                        }
                    }

                    let matched_text = submatch.matched.text.clone();
                    (column, char_start, char_end, matched_text)
                } else {
                    (1, 0, data.lines.text.len(), data.lines.text.clone())
                };

                // Get accumulated context
                let (context_before, context_after) = if let Some(acc) = context_map.remove(&file_path) {
                    acc.finish()
                } else {
                    (Vec::new(), Vec::new())
                };

                // Handle multi-line content
                let line_content = if buffer.len() > 1 {
                    buffer.iter().map(|(_, content)| content.clone()).collect::<Vec<_>>().join("\n")
                } else {
                    data.lines.text.trim_end().to_string()
                };

                matches.push(SearchMatch {
                    path: file_path.clone(),
                    line_number: data.line_number as usize,
                    column,
                    line_content,
                    context_before,
                    context_after,
                });

                // Clear multi-line buffer after processing match
                buffer.clear();
                // Recreate context accumulator for this file
                context_map.insert(file_path, ContextAccumulator::new());
            }
            RipgrepMessage::Context { data } => {
                let file_path = PathBuf::from(&data.path.text);
                let acc = context_map.entry(file_path).or_insert_with(ContextAccumulator::new);
                acc.add_context(data.line_number as usize, data.lines.text.trim_end().to_string());
            }
            RipgrepMessage::End { data } => {
                debug!("File processing complete for: {:?}", data.path.text);
            }
            RipgrepMessage::Summary { data } => {
                debug!("Search complete: {} matches in {} files", data.stats.matches, data.stats.searches);
                break;
            }
        }
    }

    Ok((matches, total_count, truncated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_match_message() {
        let json = r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}"#;

        let result: Result<RipgrepMessage, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        
        match result.unwrap() {
            RipgrepMessage::Match { data } => {
                assert_eq!(data.path.text, "test.rs");
                assert_eq!(data.lines.text, "fn main() {");
                assert_eq!(data.line_number, 1);
                assert_eq!(data.submatches.len(), 1);
                assert_eq!(data.submatches[0].matched.text, "main");
                assert_eq!(data.submatches[0].start, 3);
                assert_eq!(data.submatches[0].end, 7);
            }
            _ => panic!("Expected Match message"),
        }
    }

    #[test]
    fn test_parse_actual_ripgrep_format() {
        // Real ripgrep JSON format with a match
        let json_output = r#"{"type":"begin","data":{"path":{"text":"test.rs"}}}
{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {}\n"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}
{"type":"end","data":{"path":{"text":"test.rs"},"binary_offset":null,"stats":{"elapsed":{"secs":0,"nanos":46042,"human":"0.000046s"},"searches":1,"searches_with_match":1,"bytes_searched":13,"bytes_printed":243,"matched_lines":1,"matches":1}}}
{"data":{"elapsed_total":{"human":"0.000508s","nanos":507667,"secs":0},"stats":{"bytes_printed":243,"bytes_searched":13,"elapsed":{"human":"0.000046s","nanos":46042,"secs":0},"matched_lines":1,"matches":1,"searches":1,"searches_with_match":1}},"type":"summary"}"#;

        let (matches, total_count, truncated) = parse_ripgrep_output(json_output, 100).unwrap();

        assert_eq!(total_count, 1);
        assert_eq!(matches.len(), 1);
        assert!(!truncated);
        
        let match_result = &matches[0];
        assert_eq!(match_result.path.to_string_lossy(), "test.rs");
        assert_eq!(match_result.line_number, 1);
        assert_eq!(match_result.column, 4); // 'm' in "main" is at column 4 (1-indexed)
        assert_eq!(match_result.line_content, "fn main() {}");
    }

    #[test]
    fn test_parse_empty_output() {
        let output = "";
        let (matches, total, truncated) = parse_ripgrep_output(output, 100).unwrap();

        assert!(matches.is_empty());
        assert_eq!(total, 0);
        assert!(!truncated);
    }

    #[test]
    fn test_format_highlighted() {
        let m = RipgrepMatch {
            path: PathBuf::from("test.rs"),
            line_number: 1,
            column: 4,
            line: "fn main() {}".to_string(),
            match_start: 3,
            match_end: 7,
            matched_text: "main".to_string(),
            context_before: vec![],
            context_after: vec![],
        };

        assert_eq!(m.format_highlighted(), "fn [main]() {}");
    }

    #[test]
    fn test_context_accumulator() {
        let mut acc = ContextAccumulator::new();
        acc.add_context(1, "line1".to_string());
        acc.add_context(2, "line2".to_string());
        acc.set_match_line(3);
        acc.add_context(4, "line4".to_string());

        let (before, after) = acc.finish();
        assert_eq!(before.len(), 2);
        assert_eq!(after.len(), 1);
        assert_eq!(before[0], "line1");
        assert_eq!(before[1], "line2");
        assert_eq!(after[0], "line4");
    }

    #[test]
    fn test_parse_malformed_json() {
        let malformed = r#"{"type":"match","invalid":json"#;
        let (matches, total, truncated) = parse_ripgrep_output(malformed, 100).unwrap();

        // Should handle malformed JSON gracefully
        assert!(matches.is_empty());
        assert_eq!(total, 0);
        assert!(!truncated);
    }

    #[test]
    fn test_parse_multi_line_match() {
        // Test that we can accumulate multi-line matches
        let mut context_map = HashMap::new();
        let file_path = PathBuf::from("test.rs");
        context_map.insert(file_path.clone(), ContextAccumulator::new());

        let mut buffer = HashMap::new();
        buffer.insert(file_path, vec![(1, "line1".to_string()), (2, "line2".to_string())]);

        // Multi-line content should be joined
        let content = buffer.get(&PathBuf::from("test.rs")).unwrap()
            .iter().map(|(_, content)| content.clone()).collect::<Vec<_>>().join("\n");
        assert_eq!(content, "line1\nline2");
    }

    #[test]
    fn test_submatch_column_calculation() {
        // Test that byte offsets are correctly converted to character columns
        let line_text = "fn main() {}";
        let byte_start = 3; // Points to 'm' in "main"
        
        let mut column = 1;
        let mut byte_pos = 0;
        for ch in line_text.chars() {
            if byte_pos == byte_start {
                break;
            }
            byte_pos += ch.len_utf8();
            column += 1;
        }
        
        assert_eq!(column, 4); // 'm' is at column 4 (1-indexed)
    }

    #[test]
    fn test_truncated_results() {
        // Test that we properly handle truncation
        let json_lines = vec![
            r#"{"type":"begin","data":{"path":{"text":"test.rs"}}}"#,
            r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"match1\n"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"match1"},"start":0,"end":6}]}}"#,
            r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"match2\n"},"line_number":2,"absolute_offset":10,"submatches":[{"match":{"text":"match2"},"start":0,"end":6}]}}"#,
            r#"{"type":"end","data":{"path":{"text":"test.rs"},"binary_offset":null,"stats":{"elapsed":{"secs":0,"nanos":1000000,"human":"0.001s"},"searches":1,"searches_with_match":1,"bytes_searched":100,"bytes_printed":50,"matched_lines":2,"matches":2}}}"#,
            r#"{"data":{"elapsed_total":{"human":"0.001s","nanos":1000000,"secs":0},"stats":{"elapsed":{"human":"0.001s","nanos":1000000,"secs":0},"searches":1,"searches_with_match":1,"bytes_searched":100,"bytes_printed":50,"matched_lines":2,"matches":2}},"type":"summary"}"#,
        ];

        let output = json_lines.join("\n");
        let (matches, total_count, truncated) = parse_ripgrep_output(&output, 1).unwrap();

        assert_eq!(matches.len(), 1); // Only 1 match returned due to limit
        assert_eq!(total_count, 2);   // But 2 total matches were found
        assert!(truncated);           // Results were truncated
    }
}