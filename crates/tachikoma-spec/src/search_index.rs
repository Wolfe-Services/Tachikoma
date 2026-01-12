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

/// Search index errors
#[derive(Debug, thiserror::Error)]
pub enum SearchIndexError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Index not found")]
    NotFound,
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