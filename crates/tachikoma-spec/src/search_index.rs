// src/spec/search_index.rs

use std::collections::{HashMap, HashSet, BTreeMap};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::fs;

/// A searchable spec document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedSpec {
    /// Spec ID
    pub id: u32,
    /// Document title
    pub title: String,
    /// Phase number
    pub phase: u32,
    /// Status
    pub status: String,
    /// All text content (for full-text search)
    pub content: String,
    /// Section names
    pub sections: Vec<String>,
    /// Acceptance criteria text
    pub criteria: Vec<String>,
    /// Code block content
    pub code: String,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Tags/keywords extracted
    pub tags: Vec<String>,
    /// File path
    pub path: String,
    /// Last indexed timestamp
    pub indexed_at: u64,
}

/// Search token with position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPosition {
    pub doc_id: u32,
    pub field: IndexField,
    pub position: u32,
    pub term_frequency: u32,
}

/// Searchable fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndexField {
    Title,
    Content,
    Section,
    Code,
    Criteria,
    Tags,
}

impl IndexField {
    pub fn boost(&self) -> f32 {
        match self {
            Self::Title => 3.0,
            Self::Tags => 2.5,
            Self::Section => 2.0,
            Self::Criteria => 1.5,
            Self::Content => 1.0,
            Self::Code => 0.8,
        }
    }
}

/// Inverted index for fast lookup
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvertedIndex {
    /// Term -> List of (doc_id, field, positions)
    index: HashMap<String, Vec<TokenPosition>>,
    /// Document count
    doc_count: u32,
    /// Document lengths for normalization
    doc_lengths: HashMap<u32, u32>,
    /// Average document length
    avg_doc_length: f32,
}

impl InvertedIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a term to the index
    pub fn add_term(
        &mut self,
        term: &str,
        doc_id: u32,
        field: IndexField,
        position: u32,
    ) {
        let normalized = Self::normalize_term(term);
        if normalized.len() < 2 {
            return; // Skip very short terms
        }

        let positions = self.index.entry(normalized).or_default();

        // Update or add position
        if let Some(pos) = positions.iter_mut().find(|p| p.doc_id == doc_id && p.field == field) {
            pos.term_frequency += 1;
        } else {
            positions.push(TokenPosition {
                doc_id,
                field,
                position,
                term_frequency: 1,
            });
        }
    }

    /// Normalize a term for indexing
    fn normalize_term(term: &str) -> String {
        term.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Search for a term
    pub fn search_term(&self, term: &str) -> Vec<&TokenPosition> {
        let normalized = Self::normalize_term(term);
        self.index.get(&normalized)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Search with prefix matching
    pub fn search_prefix(&self, prefix: &str) -> Vec<&TokenPosition> {
        let normalized = Self::normalize_term(prefix);
        let mut results = Vec::new();

        for (term, positions) in &self.index {
            if term.starts_with(&normalized) {
                results.extend(positions.iter());
            }
        }

        results
    }

    /// Update statistics
    pub fn update_stats(&mut self) {
        let total_length: u32 = self.doc_lengths.values().sum();
        self.avg_doc_length = if self.doc_count > 0 {
            total_length as f32 / self.doc_count as f32
        } else {
            0.0
        };
    }
}

/// Spec search index
#[derive(Debug, Serialize, Deserialize)]
pub struct SpecSearchIndex {
    /// Inverted index for full-text search
    inverted: InvertedIndex,
    /// Document store
    documents: HashMap<u32, IndexedSpec>,
    /// Facets for filtering
    facets: SearchFacets,
    /// N-gram index for fuzzy matching
    ngrams: HashMap<String, HashSet<String>>,
    /// Index version
    version: u32,
}

/// Facets for filtering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFacets {
    /// Specs by phase
    by_phase: HashMap<u32, HashSet<u32>>,
    /// Specs by status
    by_status: HashMap<String, HashSet<u32>>,
    /// Specs by tag
    by_tag: HashMap<String, HashSet<u32>>,
}

impl SpecSearchIndex {
    pub fn new() -> Self {
        Self {
            inverted: InvertedIndex::new(),
            documents: HashMap::new(),
            facets: SearchFacets::default(),
            ngrams: HashMap::new(),
            version: 1,
        }
    }

    /// Index a spec document
    pub fn index_spec(&mut self, spec: IndexedSpec) {
        let doc_id = spec.id;

        // Index title
        self.index_text(&spec.title, doc_id, IndexField::Title);

        // Index content
        self.index_text(&spec.content, doc_id, IndexField::Content);

        // Index sections
        for section in &spec.sections {
            self.index_text(section, doc_id, IndexField::Section);
        }

        // Index criteria
        for criterion in &spec.criteria {
            self.index_text(criterion, doc_id, IndexField::Criteria);
        }

        // Index code
        self.index_text(&spec.code, doc_id, IndexField::Code);

        // Index tags
        for tag in &spec.tags {
            self.index_text(tag, doc_id, IndexField::Tags);
        }

        // Update facets
        self.facets.by_phase.entry(spec.phase).or_default().insert(doc_id);
        self.facets.by_status.entry(spec.status.clone()).or_default().insert(doc_id);
        for tag in &spec.tags {
            self.facets.by_tag.entry(tag.clone()).or_default().insert(doc_id);
        }

        // Store document
        self.documents.insert(doc_id, spec);

        // Update inverted index stats
        self.inverted.doc_count = self.documents.len() as u32;
        self.inverted.update_stats();
    }

    /// Index text content
    fn index_text(&mut self, text: &str, doc_id: u32, field: IndexField) {
        let tokens = Self::tokenize(text);

        for (position, token) in tokens.iter().enumerate() {
            self.inverted.add_term(token, doc_id, field, position as u32);

            // Add n-grams for fuzzy matching
            self.add_ngrams(token);
        }

        // Track document length
        *self.inverted.doc_lengths.entry(doc_id).or_default() += tokens.len() as u32;
    }

    /// Tokenize text
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() >= 2)
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Add n-grams for a term
    fn add_ngrams(&mut self, term: &str) {
        let chars: Vec<char> = term.chars().collect();
        if chars.len() < 3 {
            return;
        }

        for n in 2..=3 {
            for window in chars.windows(n) {
                let ngram: String = window.iter().collect();
                self.ngrams.entry(ngram).or_default().insert(term.to_string());
            }
        }
    }

    /// Remove a spec from the index
    pub fn remove_spec(&mut self, spec_id: u32) {
        if let Some(spec) = self.documents.remove(&spec_id) {
            // Clean up facets
            if let Some(phase_set) = self.facets.by_phase.get_mut(&spec.phase) {
                phase_set.remove(&spec_id);
            }
            if let Some(status_set) = self.facets.by_status.get_mut(&spec.status) {
                status_set.remove(&spec_id);
            }
            for tag in &spec.tags {
                if let Some(tag_set) = self.facets.by_tag.get_mut(tag) {
                    tag_set.remove(&spec_id);
                }
            }

            // Note: Full inverted index cleanup is expensive, so we do lazy cleanup
            // or full reindex periodically
        }
    }

    /// Get document by ID
    pub fn get_spec(&self, spec_id: u32) -> Option<&IndexedSpec> {
        self.documents.get(&spec_id)
    }

    /// Get all spec IDs
    pub fn all_ids(&self) -> Vec<u32> {
        self.documents.keys().copied().collect()
    }

    /// Save index to file
    pub async fn save(&self, path: &Path) -> Result<(), SearchIndexError> {
        let data = serde_json::to_vec(self)?;
        fs::write(path, data).await?;
        Ok(())
    }

    /// Load index from file
    pub async fn load(path: &Path) -> Result<Self, SearchIndexError> {
        let data = fs::read(path).await?;
        let index: Self = serde_json::from_slice(&data)?;
        Ok(index)
    }

    /// Get facet values
    pub fn get_facets(&self) -> FacetSummary {
        FacetSummary {
            phases: self.facets.by_phase.keys().copied().collect(),
            statuses: self.facets.by_status.keys().cloned().collect(),
            tags: self.facets.by_tag.keys().cloned().collect(),
        }
    }

    /// Search the index
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResults, SearchIndexError> {
        let start = std::time::Instant::now();

        let mut candidates = self.find_candidates(query)?;

        // Apply facet filters
        candidates = self.apply_facet_filters(candidates, query);

        // Score and rank results
        let mut results = self.score_and_rank(candidates, query);

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        let took = start.elapsed().as_millis() as u64;
        let total = results.len(); // TODO: Calculate total before limit

        Ok(SearchResults {
            results,
            total,
            took,
            facets: self.get_facets(),
        })
    }

    /// Find candidate documents
    fn find_candidates(&self, query: &SearchQuery) -> Result<HashSet<u32>, SearchIndexError> {
        let mut candidates = HashSet::new();

        // Full-text search
        if let Some(text) = &query.text {
            let text_candidates = if query.fuzzy {
                self.search_fuzzy(text, query.fuzzy_threshold)
            } else {
                self.search_exact(text)
            };
            candidates.extend(text_candidates);
        }

        // Field-specific searches
        for (field, field_query) in &query.fields {
            let field_candidates = if query.fuzzy {
                self.search_field_fuzzy(field_query, *field, query.fuzzy_threshold)
            } else {
                self.search_field_exact(field_query, *field)
            };
            
            if candidates.is_empty() {
                candidates.extend(field_candidates);
            } else {
                // Intersect with existing candidates
                candidates.retain(|id| field_candidates.contains(id));
            }
        }

        // If no text queries, return all documents for facet filtering
        if query.text.is_none() && query.fields.is_empty() {
            candidates = self.documents.keys().copied().collect();
        }

        Ok(candidates)
    }

    /// Apply facet filters to candidates
    fn apply_facet_filters(&self, mut candidates: HashSet<u32>, query: &SearchQuery) -> HashSet<u32> {
        // Filter by phases
        if !query.phases.is_empty() {
            let phase_docs: HashSet<u32> = query.phases
                .iter()
                .filter_map(|phase| self.facets.by_phase.get(phase))
                .flat_map(|set| set.iter())
                .copied()
                .collect();
            candidates.retain(|id| phase_docs.contains(id));
        }

        // Filter by statuses
        if !query.statuses.is_empty() {
            let status_docs: HashSet<u32> = query.statuses
                .iter()
                .filter_map(|status| self.facets.by_status.get(status))
                .flat_map(|set| set.iter())
                .copied()
                .collect();
            candidates.retain(|id| status_docs.contains(id));
        }

        // Filter by tags
        if !query.tags.is_empty() {
            let tag_docs: HashSet<u32> = query.tags
                .iter()
                .filter_map(|tag| self.facets.by_tag.get(tag))
                .flat_map(|set| set.iter())
                .copied()
                .collect();
            candidates.retain(|id| tag_docs.contains(id));
        }

        candidates
    }

    /// Search exact term matches
    fn search_exact(&self, text: &str) -> HashSet<u32> {
        let terms = Self::tokenize(text);
        if terms.is_empty() {
            return HashSet::new();
        }

        let mut doc_sets: Vec<HashSet<u32>> = Vec::new();

        for term in terms {
            let positions = self.inverted.search_term(&term);
            let doc_ids: HashSet<u32> = positions.iter()
                .map(|pos| pos.doc_id)
                .collect();
            doc_sets.push(doc_ids);
        }

        // For multi-term queries, require all terms to match
        if doc_sets.len() == 1 {
            doc_sets.into_iter().next().unwrap_or_default()
        } else {
            doc_sets.into_iter()
                .reduce(|acc, set| acc.intersection(&set).copied().collect())
                .unwrap_or_default()
        }
    }

    /// Search with fuzzy matching
    fn search_fuzzy(&self, text: &str, threshold: f32) -> HashSet<u32> {
        let terms = Self::tokenize(text);
        if terms.is_empty() {
            return HashSet::new();
        }

        let mut all_matches = HashSet::new();

        for term in terms {
            // Try exact match first
            let exact_positions = self.inverted.search_term(&term);
            all_matches.extend(exact_positions.iter().map(|pos| pos.doc_id));

            // Then try fuzzy matching
            let fuzzy_terms = self.find_fuzzy_matches(&term, threshold);
            for fuzzy_term in fuzzy_terms {
                let positions = self.inverted.search_term(&fuzzy_term);
                all_matches.extend(positions.iter().map(|pos| pos.doc_id));
            }
        }

        all_matches
    }

    /// Search exact matches in specific field
    fn search_field_exact(&self, text: &str, field: IndexField) -> HashSet<u32> {
        let terms = Self::tokenize(text);
        if terms.is_empty() {
            return HashSet::new();
        }

        let mut results = HashSet::new();
        for term in terms {
            let positions = self.inverted.search_term(&term);
            results.extend(
                positions.iter()
                    .filter(|pos| pos.field == field)
                    .map(|pos| pos.doc_id)
            );
        }

        results
    }

    /// Search fuzzy matches in specific field
    fn search_field_fuzzy(&self, text: &str, field: IndexField, threshold: f32) -> HashSet<u32> {
        let terms = Self::tokenize(text);
        if terms.is_empty() {
            return HashSet::new();
        }

        let mut results = HashSet::new();
        for term in terms {
            // Exact matches
            let exact_positions = self.inverted.search_term(&term);
            results.extend(
                exact_positions.iter()
                    .filter(|pos| pos.field == field)
                    .map(|pos| pos.doc_id)
            );

            // Fuzzy matches
            let fuzzy_terms = self.find_fuzzy_matches(&term, threshold);
            for fuzzy_term in fuzzy_terms {
                let positions = self.inverted.search_term(&fuzzy_term);
                results.extend(
                    positions.iter()
                        .filter(|pos| pos.field == field)
                        .map(|pos| pos.doc_id)
                );
            }
        }

        results
    }

    /// Find fuzzy term matches using n-grams
    fn find_fuzzy_matches(&self, term: &str, threshold: f32) -> Vec<String> {
        if term.len() < 3 {
            return Vec::new();
        }

        let term_ngrams = self.get_ngrams_for_term(term);
        let mut candidates: HashMap<String, f32> = HashMap::new();

        for ngram in &term_ngrams {
            if let Some(terms) = self.ngrams.get(ngram) {
                for candidate_term in terms {
                    if candidate_term == term {
                        continue; // Skip exact matches (handled separately)
                    }
                    
                    let similarity = self.calculate_similarity(term, candidate_term);
                    if similarity >= threshold {
                        candidates.insert(candidate_term.clone(), similarity);
                    }
                }
            }
        }

        // Return candidates sorted by similarity (best first)
        let mut results: Vec<(String, f32)> = candidates.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter().map(|(term, _)| term).collect()
    }

    /// Get n-grams for a term
    fn get_ngrams_for_term(&self, term: &str) -> Vec<String> {
        let chars: Vec<char> = term.chars().collect();
        if chars.len() < 3 {
            return Vec::new();
        }

        let mut ngrams = Vec::new();
        for n in 2..=3 {
            for window in chars.windows(n) {
                ngrams.push(window.iter().collect());
            }
        }
        ngrams
    }

    /// Calculate Jaccard similarity between two terms using n-grams
    fn calculate_similarity(&self, term1: &str, term2: &str) -> f32 {
        let ngrams1: HashSet<String> = self.get_ngrams_for_term(term1).into_iter().collect();
        let ngrams2: HashSet<String> = self.get_ngrams_for_term(term2).into_iter().collect();

        if ngrams1.is_empty() && ngrams2.is_empty() {
            return if term1 == term2 { 1.0 } else { 0.0 };
        }

        let intersection_size = ngrams1.intersection(&ngrams2).count();
        let union_size = ngrams1.union(&ngrams2).count();

        intersection_size as f32 / union_size as f32
    }

    /// Score and rank search results
    fn score_and_rank(&self, candidates: HashSet<u32>, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for doc_id in candidates {
            if let Some(spec) = self.documents.get(&doc_id) {
                let score = self.calculate_relevance_score(spec, query);
                let snippets = self.generate_snippets(spec, query);
                let matched_fields = self.get_matched_fields(spec, query);

                results.push(SearchResult {
                    spec_id: doc_id,
                    score,
                    snippets,
                    matched_fields,
                });
            }
        }

        // Sort by relevance score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        results
    }

    /// Calculate relevance score using TF-IDF with field boosting
    fn calculate_relevance_score(&self, spec: &IndexedSpec, query: &SearchQuery) -> f32 {
        let mut total_score = 0.0;

        // Combine all query terms
        let mut all_terms = Vec::new();
        
        if let Some(text) = &query.text {
            all_terms.extend(Self::tokenize(text));
        }
        
        for field_query in query.fields.values() {
            all_terms.extend(Self::tokenize(field_query));
        }

        for term in &all_terms {
            let term_positions = self.inverted.search_term(term);
            let spec_positions: Vec<_> = term_positions.iter()
                .filter(|pos| pos.doc_id == spec.id)
                .collect();

            for position in spec_positions {
                // TF component
                let tf = position.term_frequency as f32;
                let tf_score = (1.0 + tf.ln()).max(0.0);

                // IDF component
                let df = term_positions.len() as f32;
                let idf = if df > 0.0 {
                    (self.inverted.doc_count as f32 / df).ln()
                } else {
                    0.0
                };

                // Field boost
                let field_boost = position.field.boost();

                // Document length normalization
                let doc_length = *self.inverted.doc_lengths.get(&spec.id).unwrap_or(&1);
                let length_norm = 1.0 / (1.0 + (doc_length as f32 / self.inverted.avg_doc_length).sqrt());

                total_score += tf_score * idf * field_boost * length_norm;
            }
        }

        total_score
    }

    /// Generate search snippets with highlights
    fn generate_snippets(&self, spec: &IndexedSpec, query: &SearchQuery) -> Vec<SearchSnippet> {
        let mut snippets = Vec::new();

        // Collect all query terms
        let mut all_terms = Vec::new();
        if let Some(text) = &query.text {
            all_terms.extend(Self::tokenize(text));
        }
        for field_query in query.fields.values() {
            all_terms.extend(Self::tokenize(field_query));
        }

        // Generate snippets for different fields
        for field in [IndexField::Title, IndexField::Content, IndexField::Criteria] {
            let text = match field {
                IndexField::Title => &spec.title,
                IndexField::Content => &spec.content,
                IndexField::Criteria => &spec.criteria.join(" "),
                _ => continue,
            };

            if let Some(snippet) = self.create_snippet(text, &all_terms, field) {
                snippets.push(snippet);
            }
        }

        snippets
    }

    /// Create a snippet with highlights for a field
    fn create_snippet(&self, text: &str, terms: &[String], field: IndexField) -> Option<SearchSnippet> {
        let text_lower = text.to_lowercase();
        let mut highlights = Vec::new();

        // Find term positions
        for term in terms {
            let term_lower = term.to_lowercase();
            let mut start = 0;
            while let Some(pos) = text_lower[start..].find(&term_lower) {
                let absolute_pos = start + pos;
                highlights.push((absolute_pos, absolute_pos + term.len()));
                start = absolute_pos + term.len();
            }
        }

        if highlights.is_empty() {
            return None;
        }

        // Sort highlights and create snippet
        highlights.sort();
        
        // Truncate text around highlights for readability
        let snippet_text = if text.len() > 200 {
            // Find best position to show highlights
            let first_highlight = highlights[0].0;
            let start = if first_highlight > 100 { first_highlight - 100 } else { 0 };
            let end = (start + 200).min(text.len());
            
            // Adjust highlights for truncated text
            let adjusted_highlights: Vec<_> = highlights.iter()
                .filter_map(|(h_start, h_end)| {
                    if *h_start >= start && *h_end <= end {
                        Some((h_start - start, h_end - start))
                    } else {
                        None
                    }
                })
                .collect();

            if adjusted_highlights.is_empty() {
                return None;
            }

            return Some(SearchSnippet {
                field,
                text: text[start..end].to_string(),
                highlights: adjusted_highlights,
            });
        };

        Some(SearchSnippet {
            field,
            text: text.to_string(),
            highlights,
        })
    }

    /// Get fields that matched the query
    fn get_matched_fields(&self, spec: &IndexedSpec, query: &SearchQuery) -> Vec<IndexField> {
        let mut matched = Vec::new();

        // Check field-specific queries
        for (field, _) in &query.fields {
            matched.push(*field);
        }

        // For full-text queries, check all fields
        if query.text.is_some() {
            for field in [IndexField::Title, IndexField::Content, IndexField::Section, 
                         IndexField::Code, IndexField::Criteria, IndexField::Tags] {
                if !matched.contains(&field) {
                    matched.push(field);
                }
            }
        }

        matched
    }

    /// Update an existing spec in the index (incremental)
    pub fn update_spec(&mut self, spec: IndexedSpec) -> Result<(), SearchIndexError> {
        // Remove old version if it exists
        self.remove_spec(spec.id);
        
        // Add new version
        self.index_spec(spec);
        
        Ok(())
    }

    /// Rebuild index from directory of spec files
    pub async fn rebuild_from_directory(&mut self, spec_dir: &Path) -> Result<usize, SearchIndexError> {
        use crate::parsing::SpecParser;
        let mut count = 0;

        // Clear existing index
        self.inverted = InvertedIndex::new();
        self.documents.clear();
        self.facets = SearchFacets::default();
        self.ngrams.clear();

        let parser = SpecParser::new();

        // Find and parse all spec files
        let mut entries = tokio::fs::read_dir(spec_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    let parsed = parser.parse_safe(&content);
                    let indexed = IndexBuilder::from_parsed(&parsed, &path);
                    self.index_spec(indexed);
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

/// Summary of available facets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetSummary {
    pub phases: Vec<u32>,
    pub statuses: Vec<String>,
    pub tags: Vec<String>,
}

/// Index builder for batch indexing
pub struct IndexBuilder {
    index: SpecSearchIndex,
}

impl IndexBuilder {
    pub fn new() -> Self {
        Self {
            index: SpecSearchIndex::new(),
        }
    }

    /// Add a spec to the index
    pub fn add(&mut self, spec: IndexedSpec) {
        self.index.index_spec(spec);
    }

    /// Build the final index
    pub fn build(self) -> SpecSearchIndex {
        self.index
    }

    /// Create IndexedSpec from ParsedSpec
    pub fn from_parsed(
        parsed: &crate::parsing::ParsedSpec,
        path: &Path,
    ) -> IndexedSpec {
        // Combine all content
        let mut content = parsed.title.clone();
        for section_content in parsed.sections.values() {
            content.push(' ');
            content.push_str(section_content);
        }

        // Extract criteria text
        let criteria: Vec<String> = parsed.acceptance_criteria
            .iter()
            .map(|c| c.text.clone())
            .collect();

        // Combine code blocks
        let code: String = parsed.code_blocks
            .iter()
            .map(|b| b.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        // Extract tags from dependencies and custom metadata
        let mut tags = parsed.metadata.dependencies.clone();
        tags.extend(parsed.metadata.custom.keys().cloned());

        IndexedSpec {
            id: parsed.metadata.spec_id,
            title: parsed.title.clone(),
            phase: parsed.metadata.phase,
            status: parsed.metadata.status.clone(),
            content,
            sections: parsed.sections.keys().cloned().collect(),
            criteria,
            code,
            dependencies: parsed.metadata.dependencies.clone(),
            tags,
            path: path.to_string_lossy().to_string(),
            indexed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Search query for finding specs
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Query text (for full-text or field-specific search)
    pub text: Option<String>,
    /// Field-specific queries
    pub fields: HashMap<IndexField, String>,
    /// Phase filters
    pub phases: Vec<u32>,
    /// Status filters
    pub statuses: Vec<String>,
    /// Tag filters
    pub tags: Vec<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Enable fuzzy matching
    pub fuzzy: bool,
    /// Fuzzy matching distance (0.0 = exact, 1.0 = very loose)
    pub fuzzy_threshold: f32,
}

/// Search result with relevance scoring
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// Document ID
    pub spec_id: u32,
    /// Relevance score (higher = more relevant)
    pub score: f32,
    /// Matching snippets with highlights
    pub snippets: Vec<SearchSnippet>,
    /// Matched fields
    pub matched_fields: Vec<IndexField>,
}

/// Search snippet with context
#[derive(Debug, Clone, PartialEq)]
pub struct SearchSnippet {
    /// Field where match was found
    pub field: IndexField,
    /// Text content with highlights
    pub text: String,
    /// Byte positions of highlights in text
    pub highlights: Vec<(usize, usize)>,
}

/// Search results collection
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// Found results
    pub results: Vec<SearchResult>,
    /// Total number of matching documents (before limit)
    pub total: usize,
    /// Query processing time in milliseconds
    pub took: u64,
    /// Facets for filtering
    pub facets: FacetSummary,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self {
            fuzzy_threshold: 0.7, // Default moderate fuzzy matching
            ..Default::default()
        }
    }

    /// Create a simple text query
    pub fn text(query: &str) -> Self {
        Self {
            text: Some(query.to_string()),
            ..Self::new()
        }
    }

    /// Add field-specific query
    pub fn field(mut self, field: IndexField, query: &str) -> Self {
        self.fields.insert(field, query.to_string());
        self
    }

    /// Filter by phases
    pub fn phases(mut self, phases: Vec<u32>) -> Self {
        self.phases = phases;
        self
    }

    /// Filter by status
    pub fn statuses(mut self, statuses: Vec<String>) -> Self {
        self.statuses = statuses;
        self
    }

    /// Filter by tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Enable/disable fuzzy matching
    pub fn fuzzy(mut self, enabled: bool) -> Self {
        self.fuzzy = enabled;
        self
    }

    /// Set fuzzy matching threshold
    pub fn fuzzy_threshold(mut self, threshold: f32) -> Self {
        self.fuzzy_threshold = threshold.clamp(0.0, 1.0);
        self
    }
}

/// Search index errors
#[derive(Debug, thiserror::Error)]
pub enum SearchIndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Index not found")]
    NotFound,

    #[error("Query error: {0}")]
    Query(String),
}

impl Default for SpecSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for IndexBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenization() {
        let tokens = SpecSearchIndex::tokenize("Hello, World! This is a test_case.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test_case".to_string()));
    }

    #[test]
    fn test_term_normalization() {
        assert_eq!(InvertedIndex::normalize_term("Hello"), "hello");
        assert_eq!(InvertedIndex::normalize_term("Test-Case"), "testcase");
    }

    #[test]
    fn test_indexing_and_search() {
        let mut index = SpecSearchIndex::new();

        let spec = IndexedSpec {
            id: 116,
            title: "Spec Directory Structure".to_string(),
            phase: 6,
            status: "Planned".to_string(),
            content: "This spec defines the directory structure for specs.".to_string(),
            sections: vec!["Objective".to_string()],
            criteria: vec!["Directory structure is defined".to_string()],
            code: String::new(),
            dependencies: vec![],
            tags: vec!["directory".to_string()],
            path: "/specs/116.md".to_string(),
            indexed_at: 0,
        };

        index.index_spec(spec);

        // Search for term
        let results = index.inverted.search_term("directory");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.doc_id == 116));
    }

    #[test]
    fn test_facets() {
        let mut index = SpecSearchIndex::new();

        let spec = IndexedSpec {
            id: 116,
            title: "Test".to_string(),
            phase: 6,
            status: "Planned".to_string(),
            content: String::new(),
            sections: vec![],
            criteria: vec![],
            code: String::new(),
            dependencies: vec![],
            tags: vec!["test".to_string()],
            path: String::new(),
            indexed_at: 0,
        };

        index.index_spec(spec);

        let facets = index.get_facets();
        assert!(facets.phases.contains(&6));
        assert!(facets.statuses.contains(&"Planned".to_string()));
    }
}