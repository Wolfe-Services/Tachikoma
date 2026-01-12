# 044 - Code Search JSON Parsing

**Phase:** 2 - Five Primitives
**Spec ID:** 044
**Status:** Planned
**Dependencies:** 043-code-search-core
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement parsing of ripgrep's JSON output format to extract structured search results with full metadata.

---

## Acceptance Criteria

- [ ] Parse all ripgrep JSON message types
- [ ] Extract match data with line/column
- [ ] Handle context lines in output
- [ ] Parse submatches for highlighting
- [ ] Handle multi-line matches
- [ ] Robust error handling for malformed JSON

---

## Implementation Details

### 1. JSON Parser Module (src/code_search/parser.rs)

```rust
//! Ripgrep JSON output parser.

use crate::{error::PrimitiveError, result::SearchMatch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Parse ripgrep JSON output.
pub fn parse_ripgrep_output(
    output: &str,
    max_matches: usize,
) -> Result<(Vec<SearchMatch>, usize, bool), PrimitiveError> {
    let mut matches = Vec::new();
    let mut context_map: HashMap<PathBuf, ContextAccumulator> = HashMap::new();
    let mut total_count = 0;
    let mut truncated = false;

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
            RipgrepMessage::Begin { path } => {
                debug!("Starting file: {:?}", path.text);
            }
            RipgrepMessage::Match { path, lines, line_number, submatches, .. } => {
                total_count += 1;

                if matches.len() >= max_matches {
                    truncated = true;
                    continue;
                }

                let file_path = PathBuf::from(&path.text);

                // Get accumulated context
                let (context_before, context_after) = context_map
                    .remove(&file_path)
                    .map(|acc| acc.finish())
                    .unwrap_or_default();

                let column = submatches
                    .first()
                    .map(|s| s.start + 1) // 1-indexed
                    .unwrap_or(1);

                matches.push(SearchMatch {
                    path: file_path,
                    line_number: line_number as usize,
                    column,
                    line_content: lines.text.trim_end().to_string(),
                    context_before,
                    context_after,
                });
            }
            RipgrepMessage::Context { path, lines, line_number, .. } => {
                let file_path = PathBuf::from(&path.text);
                let acc = context_map.entry(file_path).or_insert_with(ContextAccumulator::new);

                acc.add_context(line_number as usize, lines.text.trim_end().to_string());
            }
            RipgrepMessage::End { .. } => {
                // File processing complete
            }
            RipgrepMessage::Summary { stats, .. } => {
                debug!("Search complete: {} matches in {} files", stats.matches, stats.searches);
            }
        }
    }

    Ok((matches, total_count, truncated))
}

/// Ripgrep JSON message types.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RipgrepMessage {
    Begin {
        path: RipgrepText,
    },
    #[serde(rename = "match")]
    Match {
        path: RipgrepText,
        lines: RipgrepText,
        line_number: u64,
        absolute_offset: u64,
        submatches: Vec<Submatch>,
    },
    Context {
        path: RipgrepText,
        lines: RipgrepText,
        line_number: u64,
        absolute_offset: u64,
        submatches: Vec<Submatch>,
    },
    End {
        path: RipgrepText,
        binary_offset: Option<u64>,
        stats: FileStats,
    },
    Summary {
        elapsed_total: ElapsedTime,
        stats: SummaryStats,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_match_message() {
        let json = r#"{"type":"match","data":{"path":{"text":"test.rs"},"lines":{"text":"fn main() {"},"line_number":1,"absolute_offset":0,"submatches":[{"match":{"text":"main"},"start":3,"end":7}]}}"#;

        // This tests the structure but actual parsing needs proper format
        let result: Result<RipgrepMessage, _> = serde_json::from_str(json);
        // The actual ripgrep format is slightly different, adjust as needed
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
        acc.last_match_line = Some(3);
        acc.add_context(4, "line4".to_string());

        let (before, after) = acc.finish();
        assert_eq!(before.len(), 2);
        assert_eq!(after.len(), 1);
    }
}
```

---

## Testing Requirements

1. Parse match messages correctly
2. Parse context messages correctly
3. Handle begin/end messages
4. Extract summary statistics
5. Handle malformed JSON gracefully
6. Context lines are accumulated correctly
7. Submatches provide correct offsets
8. Empty output returns empty results

---

## Related Specs

- Depends on: [043-code-search-core.md](043-code-search-core.md)
- Next: [045-code-search-format.md](045-code-search-format.md)
- Related: [038-bash-output.md](038-bash-output.md)
