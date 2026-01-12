// src/spec/search_api.rs

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use crate::search_index::{SpecSearchIndex, IndexedSpec, IndexField, TokenPosition};

/// A search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Raw query string
    pub query: String,
    /// Parsed query parts
    pub parts: Vec<QueryPart>,
    /// Filters
    pub filters: SearchFilters,
    /// Pagination
    pub pagination: Pagination,
    /// Sort order
    pub sort: SortOrder,
    /// Options
    pub options: SearchOptions,
}

/// A part of a parsed query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryPart {
    /// Simple term
    Term(String),
    /// Phrase (exact match)
    Phrase(String),
    /// Field-specific search
    Field { field: String, value: String },
    /// Boolean AND
    And(Box<QueryPart>, Box<QueryPart>),
    /// Boolean OR
    Or(Box<QueryPart>, Box<QueryPart>),
    /// Boolean NOT
    Not(Box<QueryPart>),
    /// Fuzzy match
    Fuzzy { term: String, distance: u8 },
    /// Wildcard
    Wildcard(String),
}

/// Search filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by phase
    pub phases: Option<Vec<u32>>,
    /// Filter by status
    pub statuses: Option<Vec<String>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by date range
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub from: Option<u64>,
    pub to: Option<u64>,
}

/// Pagination settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortOrder {
    Relevance,
    SpecId,
    Phase,
    Status,
    DateIndexed,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Relevance
    }
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Enable fuzzy matching
    pub fuzzy: bool,
    /// Fuzzy distance threshold
    pub fuzzy_distance: u8,
    /// Include snippets in results
    pub snippets: bool,
    /// Snippet length
    pub snippet_length: usize,
    /// Highlight matches
    pub highlight: bool,
    /// Highlight tags
    pub highlight_pre: String,
    pub highlight_post: String,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            fuzzy: true,
            fuzzy_distance: 2,
            snippets: true,
            snippet_length: 150,
            highlight: true,
            highlight_pre: "**".to_string(),
            highlight_post: "**".to_string(),
        }
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Spec ID
    pub spec_id: u32,
    /// Spec title
    pub title: String,
    /// Phase
    pub phase: u32,
    /// Status
    pub status: String,
    /// Relevance score
    pub score: f32,
    /// Snippet with highlights
    pub snippet: Option<String>,
    /// Matched fields
    pub matched_fields: Vec<String>,
    /// File path
    pub path: String,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Query that was executed
    pub query: String,
    /// Total matching documents
    pub total: usize,
    /// Results for current page
    pub results: Vec<SearchResult>,
    /// Facet counts
    pub facets: FacetCounts,
    /// Search took (ms)
    pub took_ms: u64,
}

/// Facet counts for filtering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FacetCounts {
    pub by_phase: HashMap<u32, usize>,
    pub by_status: HashMap<String, usize>,
    pub by_tag: HashMap<String, usize>,
}

/// Query parser
pub struct QueryParser;

impl QueryParser {
    /// Parse a query string into structured query
    pub fn parse(query: &str) -> SearchQuery {
        let mut parts = Vec::new();
        let mut filters = SearchFilters::default();
        let mut remaining = query.trim();

        while !remaining.is_empty() {
            // Skip whitespace
            remaining = remaining.trim_start();
            if remaining.is_empty() {
                break;
            }

            // Check for quoted phrase
            if remaining.starts_with('"') {
                if let Some(end) = remaining[1..].find('"') {
                    let phrase = &remaining[1..=end];
                    parts.push(QueryPart::Phrase(phrase.to_string()));
                    remaining = &remaining[end + 2..];
                    continue;
                }
            }

            // Check for field:value
            if let Some(colon_pos) = remaining.find(':') {
                let potential_field = &remaining[..colon_pos];
                if !potential_field.contains(' ') {
                    let field = potential_field.to_lowercase();
                    remaining = &remaining[colon_pos + 1..];

                    // Get value (until space or end)
                    let value_end = remaining.find(' ').unwrap_or(remaining.len());
                    let value = &remaining[..value_end];

                    // Check if it's a filter
                    match field.as_str() {
                        "phase" => {
                            if let Ok(p) = value.parse() {
                                filters.phases.get_or_insert_with(Vec::new).push(p);
                            }
                        }
                        "status" => {
                            filters.statuses.get_or_insert_with(Vec::new).push(value.to_string());
                        }
                        "tag" => {
                            filters.tags.get_or_insert_with(Vec::new).push(value.to_string());
                        }
                        _ => {
                            parts.push(QueryPart::Field {
                                field,
                                value: value.to_string(),
                            });
                        }
                    }

                    remaining = &remaining[value_end..];
                    continue;
                }
            }

            // Check for operators
            if remaining.starts_with("AND ") || remaining.starts_with("&& ") {
                remaining = &remaining[4..];
                continue; // AND is implicit
            }

            if remaining.starts_with("OR ") || remaining.starts_with("|| ") {
                // Handle OR by wrapping previous and next in Or
                remaining = &remaining[3..];
                continue;
            }

            if remaining.starts_with("NOT ") || remaining.starts_with("- ") {
                let skip = if remaining.starts_with("NOT ") { 4 } else { 2 };
                remaining = &remaining[skip..];

                // Get next term
                let term_end = remaining.find(' ').unwrap_or(remaining.len());
                let term = &remaining[..term_end];
                parts.push(QueryPart::Not(Box::new(QueryPart::Term(term.to_string()))));
                remaining = &remaining[term_end..];
                continue;
            }

            // Check for fuzzy (~)
            let next_space = remaining.find(' ').unwrap_or(remaining.len());
            let token = &remaining[..next_space];

            if let Some(tilde_pos) = token.find('~') {
                let term = &token[..tilde_pos];
                let distance: u8 = token[tilde_pos + 1..].parse().unwrap_or(2);
                parts.push(QueryPart::Fuzzy {
                    term: term.to_string(),
                    distance,
                });
            } else if token.contains('*') || token.contains('?') {
                parts.push(QueryPart::Wildcard(token.to_string()));
            } else {
                parts.push(QueryPart::Term(token.to_string()));
            }

            remaining = &remaining[next_space..];
        }

        SearchQuery {
            query: query.to_string(),
            parts,
            filters,
            pagination: Pagination::default(),
            sort: SortOrder::default(),
            options: SearchOptions::default(),
        }
    }
}

/// Search engine
pub struct SearchEngine<'a> {
    index: &'a SpecSearchIndex,
}

impl<'a> SearchEngine<'a> {
    pub fn new(index: &'a SpecSearchIndex) -> Self {
        Self { index }
    }

    /// Execute a search query
    pub fn search(&self, query: &SearchQuery) -> SearchResponse {
        let start = std::time::Instant::now();

        // Get matching document IDs
        let mut scores: HashMap<u32, f32> = HashMap::new();
        let mut matched_fields: HashMap<u32, HashSet<String>> = HashMap::new();

        for part in &query.parts {
            self.evaluate_part(part, &mut scores, &mut matched_fields, &query.options);
        }

        // Apply filters
        let filtered_ids = self.apply_filters(&scores.keys().copied().collect(), &query.filters);

        // Calculate facets
        let facets = self.calculate_facets(&filtered_ids);

        // Sort results
        let mut sorted: Vec<_> = filtered_ids.iter()
            .filter_map(|id| scores.get(id).map(|s| (*id, *s)))
            .collect();

        match query.sort {
            SortOrder::Relevance => sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()),
            SortOrder::SpecId => sorted.sort_by_key(|a| a.0),
            SortOrder::Phase => {
                sorted.sort_by_key(|a| self.index.get_spec(a.0).map(|s| s.phase).unwrap_or(0));
            }
            _ => sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()),
        }

        let total = sorted.len();

        // Paginate
        let page: Vec<_> = sorted.into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.limit)
            .collect();

        // Build results
        let results: Vec<_> = page.iter()
            .filter_map(|(id, score)| {
                let spec = self.index.get_spec(*id)?;
                let snippet = if query.options.snippets {
                    self.generate_snippet(spec, &query.parts, &query.options)
                } else {
                    None
                };

                Some(SearchResult {
                    spec_id: *id,
                    title: spec.title.clone(),
                    phase: spec.phase,
                    status: spec.status.clone(),
                    score: *score,
                    snippet,
                    matched_fields: matched_fields.get(id)
                        .map(|f| f.iter().cloned().collect())
                        .unwrap_or_default(),
                    path: spec.path.clone(),
                })
            })
            .collect();

        SearchResponse {
            query: query.query.clone(),
            total,
            results,
            facets,
            took_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Evaluate a query part
    fn evaluate_part(
        &self,
        part: &QueryPart,
        scores: &mut HashMap<u32, f32>,
        matched_fields: &mut HashMap<u32, HashSet<String>>,
        options: &SearchOptions,
    ) {
        match part {
            QueryPart::Term(term) => {
                self.score_term(term, scores, matched_fields, options);
            }
            QueryPart::Phrase(phrase) => {
                // Score each word in phrase
                for word in phrase.split_whitespace() {
                    self.score_term(word, scores, matched_fields, options);
                }
            }
            QueryPart::Field { field, value } => {
                self.score_field_term(field, value, scores, matched_fields);
            }
            QueryPart::Fuzzy { term, distance } => {
                self.score_fuzzy(term, *distance, scores, matched_fields);
            }
            QueryPart::Wildcard(pattern) => {
                self.score_wildcard(pattern, scores, matched_fields);
            }
            QueryPart::Not(inner) => {
                let mut neg_scores = HashMap::new();
                let mut neg_fields = HashMap::new();
                self.evaluate_part(inner, &mut neg_scores, &mut neg_fields, options);

                // Remove matching docs
                for id in neg_scores.keys() {
                    scores.remove(id);
                    matched_fields.remove(id);
                }
            }
            QueryPart::And(left, right) => {
                self.evaluate_part(left, scores, matched_fields, options);
                self.evaluate_part(right, scores, matched_fields, options);
            }
            QueryPart::Or(left, right) => {
                self.evaluate_part(left, scores, matched_fields, options);
                self.evaluate_part(right, scores, matched_fields, options);
            }
        }
    }

    /// Score a term
    fn score_term(
        &self,
        term: &str,
        scores: &mut HashMap<u32, f32>,
        matched_fields: &mut HashMap<u32, HashSet<String>>,
        options: &SearchOptions,
    ) {
        let positions = self.index.inverted.search_term(term);

        for pos in positions {
            let boost = pos.field.boost();
            let score = boost * (pos.term_frequency as f32).log2().max(1.0);

            *scores.entry(pos.doc_id).or_insert(0.0) += score;
            matched_fields.entry(pos.doc_id)
                .or_default()
                .insert(format!("{:?}", pos.field));
        }

        // Also try prefix match if fuzzy
        if options.fuzzy {
            let prefix_positions = self.index.inverted.search_prefix(term);
            for pos in prefix_positions {
                let boost = pos.field.boost() * 0.5; // Lower boost for prefix
                let score = boost * (pos.term_frequency as f32).log2().max(1.0);

                *scores.entry(pos.doc_id).or_insert(0.0) += score;
            }
        }
    }

    /// Score field-specific term
    fn score_field_term(
        &self,
        field: &str,
        value: &str,
        scores: &mut HashMap<u32, f32>,
        matched_fields: &mut HashMap<u32, HashSet<String>>,
    ) {
        let target_field = match field.to_lowercase().as_str() {
            "title" => Some(IndexField::Title),
            "content" => Some(IndexField::Content),
            "section" => Some(IndexField::Section),
            "code" => Some(IndexField::Code),
            "criteria" => Some(IndexField::Criteria),
            "tag" | "tags" => Some(IndexField::Tags),
            _ => None,
        };

        let positions = self.index.inverted.search_term(value);

        for pos in positions {
            if let Some(target) = target_field {
                if pos.field != target {
                    continue;
                }
            }

            let boost = pos.field.boost() * 2.0; // Higher boost for field match
            let score = boost * (pos.term_frequency as f32).log2().max(1.0);

            *scores.entry(pos.doc_id).or_insert(0.0) += score;
            matched_fields.entry(pos.doc_id)
                .or_default()
                .insert(field.to_string());
        }
    }

    /// Score fuzzy term
    fn score_fuzzy(
        &self,
        term: &str,
        distance: u8,
        scores: &mut HashMap<u32, f32>,
        matched_fields: &mut HashMap<u32, HashSet<String>>,
    ) {
        // Get similar terms from n-gram index
        // (simplified - full implementation would use edit distance)
        let positions = self.index.inverted.search_prefix(&term[..term.len().min(3)]);

        for pos in positions {
            let boost = pos.field.boost() * 0.7; // Lower boost for fuzzy
            let score = boost * (pos.term_frequency as f32).log2().max(1.0);

            *scores.entry(pos.doc_id).or_insert(0.0) += score;
            matched_fields.entry(pos.doc_id)
                .or_default()
                .insert(format!("{:?}", pos.field));
        }
    }

    /// Score wildcard pattern
    fn score_wildcard(
        &self,
        pattern: &str,
        scores: &mut HashMap<u32, f32>,
        matched_fields: &mut HashMap<u32, HashSet<String>>,
    ) {
        // Convert to prefix search
        let prefix = pattern.split('*').next().unwrap_or("");
        if prefix.len() >= 2 {
            let positions = self.index.inverted.search_prefix(prefix);

            for pos in positions {
                let boost = pos.field.boost() * 0.8;
                let score = boost * (pos.term_frequency as f32).log2().max(1.0);

                *scores.entry(pos.doc_id).or_insert(0.0) += score;
                matched_fields.entry(pos.doc_id)
                    .or_default()
                    .insert(format!("{:?}", pos.field));
            }
        }
    }

    /// Apply filters to result set
    fn apply_filters(&self, ids: &HashSet<u32>, filters: &SearchFilters) -> HashSet<u32> {
        let mut result = ids.clone();

        if let Some(phases) = &filters.phases {
            result.retain(|id| {
                self.index.get_spec(*id)
                    .map(|s| phases.contains(&s.phase))
                    .unwrap_or(false)
            });
        }

        if let Some(statuses) = &filters.statuses {
            result.retain(|id| {
                self.index.get_spec(*id)
                    .map(|s| statuses.iter().any(|st| st.eq_ignore_ascii_case(&s.status)))
                    .unwrap_or(false)
            });
        }

        if let Some(tags) = &filters.tags {
            result.retain(|id| {
                self.index.get_spec(*id)
                    .map(|s| tags.iter().any(|t| s.tags.contains(t)))
                    .unwrap_or(false)
            });
        }

        result
    }

    /// Calculate facet counts
    fn calculate_facets(&self, ids: &HashSet<u32>) -> FacetCounts {
        let mut facets = FacetCounts::default();

        for id in ids {
            if let Some(spec) = self.index.get_spec(*id) {
                *facets.by_phase.entry(spec.phase).or_insert(0) += 1;
                *facets.by_status.entry(spec.status.clone()).or_insert(0) += 1;
                for tag in &spec.tags {
                    *facets.by_tag.entry(tag.clone()).or_insert(0) += 1;
                }
            }
        }

        facets
    }

    /// Generate snippet with highlighting
    fn generate_snippet(
        &self,
        spec: &IndexedSpec,
        parts: &[QueryPart],
        options: &SearchOptions,
    ) -> Option<String> {
        let terms: Vec<String> = parts.iter()
            .filter_map(|p| match p {
                QueryPart::Term(t) => Some(t.to_lowercase()),
                QueryPart::Phrase(p) => Some(p.to_lowercase()),
                _ => None,
            })
            .collect();

        // Find best snippet location
        let content = &spec.content;
        let words: Vec<&str> = content.split_whitespace().collect();

        let mut best_start = 0;
        let mut best_score = 0;

        for (i, window) in words.windows(20).enumerate() {
            let window_text = window.join(" ").to_lowercase();
            let score = terms.iter()
                .filter(|t| window_text.contains(t.as_str()))
                .count();

            if score > best_score {
                best_score = score;
                best_start = i;
            }
        }

        // Extract snippet
        let snippet_words: Vec<_> = words.iter()
            .skip(best_start)
            .take(25)
            .copied()
            .collect();

        let mut snippet = snippet_words.join(" ");

        // Apply highlighting
        if options.highlight {
            for term in &terms {
                let pattern = regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(term))).ok()?;
                snippet = pattern.replace_all(
                    &snippet,
                    &format!("{}{}{}", options.highlight_pre, term, options.highlight_post)
                ).to_string();
            }
        }

        // Truncate and add ellipsis
        if snippet.len() > options.snippet_length {
            snippet = format!("{}...", &snippet[..options.snippet_length]);
        }

        if best_start > 0 {
            snippet = format!("...{}", snippet);
        }

        Some(snippet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parsing() {
        let query = QueryParser::parse("spec directory phase:6 status:Planned");

        assert!(!query.parts.is_empty());
        assert!(query.filters.phases.as_ref().map(|p| p.contains(&6)).unwrap_or(false));
    }

    #[test]
    fn test_phrase_parsing() {
        let query = QueryParser::parse("\"exact phrase\" other");

        assert!(query.parts.iter().any(|p| matches!(p, QueryPart::Phrase(s) if s == "exact phrase")));
    }

    #[test]
    fn test_fuzzy_parsing() {
        let query = QueryParser::parse("directory~2");

        assert!(query.parts.iter().any(|p| matches!(p, QueryPart::Fuzzy { term, distance } if term == "directory" && *distance == 2)));
    }

    #[test]
    fn test_field_parsing() {
        let query = QueryParser::parse("title:search content:api");

        assert!(query.parts.iter().any(|p| matches!(
            p, QueryPart::Field { field, value } 
            if field == "title" && value == "search"
        )));
    }

    #[test]
    fn test_boolean_parsing() {
        let query = QueryParser::parse("search AND api OR index NOT old");

        assert!(!query.parts.is_empty());
        assert!(query.parts.iter().any(|p| matches!(p, QueryPart::Not(_))));
    }

    #[test]
    fn test_wildcard_parsing() {
        let query = QueryParser::parse("search* api?");

        assert!(query.parts.iter().any(|p| matches!(p, QueryPart::Wildcard(s) if s == "search*")));
        assert!(query.parts.iter().any(|p| matches!(p, QueryPart::Wildcard(s) if s == "api?")));
    }
}