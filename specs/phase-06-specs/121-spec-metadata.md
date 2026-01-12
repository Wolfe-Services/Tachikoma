# Spec 121: Spec Metadata Extraction

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 121
- **Status**: Planned
- **Dependencies**: 120-spec-parsing
- **Estimated Context**: ~9%

## Objective

Define comprehensive metadata extraction and management for specification documents. This extends the basic parsing to provide rich metadata including computed fields, relationships, history tracking, and integration with the broader project context.

## Acceptance Criteria

- [x] All standard metadata fields are extracted
- [x] Custom metadata fields are supported
- [x] Computed fields (age, staleness, complexity) work
- [x] Relationship metadata (dependencies, dependents) is built
- [x] Metadata is serializable for caching
- [x] Metadata updates trigger appropriate events
- [x] Bulk metadata extraction is efficient
- [x] Metadata schema is versioned

## Implementation Details

### Metadata Extraction System

```rust
// src/spec/metadata.rs

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::spec::parsing::{ParsedSpec, SpecMetadata as BasicMetadata};

/// Complete spec metadata with computed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMetadata {
    // ===== Core Fields =====
    /// Spec ID (unique identifier)
    pub id: u32,
    /// Spec title
    pub title: String,
    /// Phase number
    pub phase: u32,
    /// Phase name
    pub phase_name: String,
    /// Current status
    pub status: SpecStatus,
    /// File path
    pub path: PathBuf,

    // ===== Dependency Fields =====
    /// Direct dependencies (spec IDs or references)
    pub dependencies: Vec<SpecDependency>,
    /// Specs that depend on this one (computed)
    pub dependents: Vec<u32>,
    /// Dependency depth (max chain length)
    pub dependency_depth: u32,

    // ===== Progress Fields =====
    /// Estimated context consumption
    pub estimated_context: ContextEstimate,
    /// Acceptance criteria stats
    pub acceptance_criteria: AcceptanceCriteriaStats,
    /// Implementation status
    pub implementation_status: ImplementationStatus,

    // ===== Temporal Fields =====
    /// Creation timestamp (from git or file)
    pub created_at: Option<DateTime<Utc>>,
    /// Last modification timestamp
    pub modified_at: Option<DateTime<Utc>>,
    /// Age in days
    pub age_days: Option<u32>,
    /// Staleness score (0-100, higher = more stale)
    pub staleness_score: u8,

    // ===== Complexity Fields =====
    /// Estimated complexity (1-10)
    pub complexity: u8,
    /// Word count
    pub word_count: u32,
    /// Code block count
    pub code_block_count: u32,
    /// Section count
    pub section_count: u32,

    // ===== Custom Fields =====
    /// Custom metadata key-value pairs
    pub custom: HashMap<String, MetadataValue>,

    // ===== Meta Fields =====
    /// Schema version
    pub schema_version: u32,
    /// Extraction timestamp
    pub extracted_at: DateTime<Utc>,
}

/// Spec status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecStatus {
    Draft,
    Planned,
    InProgress,
    Review,
    Complete,
    Deprecated,
    Blocked,
}

impl SpecStatus {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => Self::Draft,
            "planned" => Self::Planned,
            "in progress" | "inprogress" | "in-progress" | "wip" => Self::InProgress,
            "review" | "in review" => Self::Review,
            "complete" | "completed" | "done" => Self::Complete,
            "deprecated" => Self::Deprecated,
            "blocked" => Self::Blocked,
            _ => Self::Planned,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Planned | Self::InProgress | Self::Review)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Deprecated)
    }
}

/// Dependency reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecDependency {
    /// Referenced spec ID
    pub spec_id: u32,
    /// Dependency type
    pub dep_type: DependencyType,
    /// Original reference string
    pub reference: String,
    /// Whether dependency is satisfied
    pub satisfied: bool,
}

/// Types of dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Must be implemented before
    Requires,
    /// Related but not blocking
    RelatedTo,
    /// Extends functionality of
    Extends,
    /// Replaces/supersedes
    Replaces,
}

/// Context consumption estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEstimate {
    /// Percentage string (e.g., "~10%")
    pub display: String,
    /// Numeric value (0-100)
    pub percentage: u8,
    /// Confidence level
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Acceptance criteria statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriteriaStats {
    /// Total criteria count
    pub total: u32,
    /// Completed criteria count
    pub completed: u32,
    /// Completion percentage
    pub percentage: u8,
    /// Criteria by section
    pub by_section: HashMap<String, (u32, u32)>,
}

/// Implementation status details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationStatus {
    /// Has implementation section
    pub has_implementation: bool,
    /// Has code blocks
    pub has_code: bool,
    /// Has tests section
    pub has_tests: bool,
    /// Languages used
    pub languages: Vec<String>,
}

/// Custom metadata value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
}

/// Metadata extractor
pub struct MetadataExtractor {
    schema_version: u32,
}

impl MetadataExtractor {
    pub fn new() -> Self {
        Self { schema_version: 1 }
    }

    /// Extract complete metadata from parsed spec
    pub fn extract(&self, parsed: &ParsedSpec, path: PathBuf) -> SpecMetadata {
        let basic = &parsed.metadata;

        // Calculate stats
        let ac_stats = self.calculate_acceptance_stats(parsed);
        let impl_status = self.calculate_impl_status(parsed);
        let complexity = self.estimate_complexity(parsed);
        let context = self.parse_context_estimate(&basic.estimated_context);

        // Extract dependencies
        let dependencies = self.extract_dependencies(basic);

        SpecMetadata {
            id: basic.spec_id,
            title: parsed.title.clone(),
            phase: basic.phase,
            phase_name: basic.phase_name.clone().unwrap_or_default(),
            status: SpecStatus::from_string(&basic.status),
            path,
            dependencies,
            dependents: Vec::new(), // Computed later in bulk
            dependency_depth: 0,     // Computed later
            estimated_context: context,
            acceptance_criteria: ac_stats,
            implementation_status: impl_status,
            created_at: None,  // Requires git integration
            modified_at: None, // Requires file system
            age_days: None,
            staleness_score: 0,
            complexity,
            word_count: self.count_words(parsed),
            code_block_count: parsed.code_blocks.len() as u32,
            section_count: parsed.sections.len() as u32,
            custom: self.convert_custom(&basic.custom),
            schema_version: self.schema_version,
            extracted_at: Utc::now(),
        }
    }

    /// Calculate acceptance criteria stats
    fn calculate_acceptance_stats(&self, parsed: &ParsedSpec) -> AcceptanceCriteriaStats {
        let total = parsed.acceptance_criteria.len() as u32;
        let completed = parsed.acceptance_criteria.iter()
            .filter(|c| c.checked)
            .count() as u32;

        let mut by_section: HashMap<String, (u32, u32)> = HashMap::new();
        for criteria in &parsed.acceptance_criteria {
            let entry = by_section.entry(criteria.section.clone()).or_insert((0, 0));
            entry.0 += 1;
            if criteria.checked {
                entry.1 += 1;
            }
        }

        AcceptanceCriteriaStats {
            total,
            completed,
            percentage: if total > 0 { ((completed as f32 / total as f32) * 100.0) as u8 } else { 0 },
            by_section,
        }
    }

    /// Calculate implementation status
    fn calculate_impl_status(&self, parsed: &ParsedSpec) -> ImplementationStatus {
        let has_implementation = parsed.sections.contains_key("Implementation Details")
            || parsed.sections.contains_key("Implementation");

        let has_tests = parsed.sections.contains_key("Testing Requirements")
            || parsed.sections.contains_key("Tests");

        let languages: Vec<String> = parsed.code_blocks.iter()
            .filter_map(|b| b.language.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        ImplementationStatus {
            has_implementation,
            has_code: !parsed.code_blocks.is_empty(),
            has_tests,
            languages,
        }
    }

    /// Estimate complexity (1-10)
    fn estimate_complexity(&self, parsed: &ParsedSpec) -> u8 {
        let mut score: f32 = 1.0;

        // Word count factor
        let words = self.count_words(parsed);
        score += (words as f32 / 500.0).min(3.0);

        // Code block factor
        score += (parsed.code_blocks.len() as f32 * 0.5).min(2.0);

        // Acceptance criteria factor
        score += (parsed.acceptance_criteria.len() as f32 * 0.2).min(2.0);

        // Dependencies factor
        score += (parsed.metadata.dependencies.len() as f32 * 0.3).min(1.5);

        score.min(10.0).max(1.0) as u8
    }

    /// Count words in spec
    fn count_words(&self, parsed: &ParsedSpec) -> u32 {
        let mut count = 0u32;
        for content in parsed.sections.values() {
            count += content.split_whitespace().count() as u32;
        }
        count
    }

    /// Parse context estimate string
    fn parse_context_estimate(&self, estimate: &Option<String>) -> ContextEstimate {
        let display = estimate.clone().unwrap_or_else(|| "~10%".to_string());

        let percentage = display
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(10);

        let confidence = if display.contains('~') {
            Confidence::Medium
        } else if display.contains('-') {
            Confidence::Low
        } else {
            Confidence::High
        };

        ContextEstimate {
            display,
            percentage,
            confidence,
        }
    }

    /// Extract and parse dependencies
    fn extract_dependencies(&self, basic: &BasicMetadata) -> Vec<SpecDependency> {
        basic.dependencies.iter().map(|dep| {
            let spec_id = self.extract_spec_id(dep);
            SpecDependency {
                spec_id: spec_id.unwrap_or(0),
                dep_type: DependencyType::Requires,
                reference: dep.clone(),
                satisfied: false, // Computed later
            }
        }).collect()
    }

    /// Extract spec ID from reference
    fn extract_spec_id(&self, reference: &str) -> Option<u32> {
        let re = regex::Regex::new(r"(\d{3})").ok()?;
        re.captures(reference)?
            .get(1)?
            .as_str()
            .parse()
            .ok()
    }

    /// Convert custom fields
    fn convert_custom(&self, custom: &HashMap<String, String>) -> HashMap<String, MetadataValue> {
        custom.iter()
            .map(|(k, v)| {
                let value = if let Ok(n) = v.parse::<f64>() {
                    MetadataValue::Number(n)
                } else if v == "true" || v == "false" {
                    MetadataValue::Boolean(v == "true")
                } else {
                    MetadataValue::String(v.clone())
                };
                (k.clone(), value)
            })
            .collect()
    }
}

/// Bulk metadata operations
pub struct MetadataIndex {
    specs: HashMap<u32, SpecMetadata>,
}

impl MetadataIndex {
    pub fn new() -> Self {
        Self { specs: HashMap::new() }
    }

    /// Add spec metadata
    pub fn add(&mut self, metadata: SpecMetadata) {
        self.specs.insert(metadata.id, metadata);
    }

    /// Compute derived fields (dependents, depth, staleness)
    pub fn compute_derived(&mut self) {
        // Build dependents map
        let mut dependents: HashMap<u32, Vec<u32>> = HashMap::new();
        for (id, meta) in &self.specs {
            for dep in &meta.dependencies {
                dependents.entry(dep.spec_id).or_default().push(*id);
            }
        }

        // Apply dependents
        for (id, deps) in dependents {
            if let Some(meta) = self.specs.get_mut(&id) {
                meta.dependents = deps;
            }
        }

        // Compute dependency depth
        let depths = self.compute_depths();
        for (id, depth) in depths {
            if let Some(meta) = self.specs.get_mut(&id) {
                meta.dependency_depth = depth;
            }
        }

        // Compute staleness
        self.compute_staleness();
    }

    /// Compute dependency depths
    fn compute_depths(&self) -> HashMap<u32, u32> {
        let mut depths = HashMap::new();

        for id in self.specs.keys() {
            let depth = self.compute_depth(*id, &mut HashSet::new());
            depths.insert(*id, depth);
        }

        depths
    }

    fn compute_depth(&self, id: u32, visited: &mut HashSet<u32>) -> u32 {
        if visited.contains(&id) {
            return 0; // Circular dependency
        }
        visited.insert(id);

        let meta = match self.specs.get(&id) {
            Some(m) => m,
            None => return 0,
        };

        let max_dep = meta.dependencies.iter()
            .map(|d| self.compute_depth(d.spec_id, visited))
            .max()
            .unwrap_or(0);

        visited.remove(&id);
        max_dep + 1
    }

    /// Compute staleness scores
    fn compute_staleness(&mut self) {
        let now = Utc::now();

        for meta in self.specs.values_mut() {
            if let Some(modified) = meta.modified_at {
                let age = now.signed_duration_since(modified);
                let days = age.num_days() as u32;
                meta.age_days = Some(days);

                // Staleness increases with age, modified by status
                let base = (days as f32 / 30.0 * 10.0).min(100.0) as u8;
                meta.staleness_score = match meta.status {
                    SpecStatus::Complete => base / 4,
                    SpecStatus::InProgress => base / 2,
                    _ => base,
                };
            }
        }
    }

    /// Get spec by ID
    pub fn get(&self, id: u32) -> Option<&SpecMetadata> {
        self.specs.get(&id)
    }

    /// Get all specs
    pub fn all(&self) -> impl Iterator<Item = &SpecMetadata> {
        self.specs.values()
    }

    /// Filter specs by status
    pub fn by_status(&self, status: SpecStatus) -> Vec<&SpecMetadata> {
        self.specs.values()
            .filter(|m| m.status == status)
            .collect()
    }

    /// Get specs by phase
    pub fn by_phase(&self, phase: u32) -> Vec<&SpecMetadata> {
        self.specs.values()
            .filter(|m| m.phase == phase)
            .collect()
    }
}

impl Default for MetadataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetadataIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_parsing() {
        assert_eq!(SpecStatus::from_string("In Progress"), SpecStatus::InProgress);
        assert_eq!(SpecStatus::from_string("complete"), SpecStatus::Complete);
        assert_eq!(SpecStatus::from_string("WIP"), SpecStatus::InProgress);
    }

    #[test]
    fn test_context_estimate_parsing() {
        let extractor = MetadataExtractor::new();

        let estimate = extractor.parse_context_estimate(&Some("~10%".to_string()));
        assert_eq!(estimate.percentage, 10);
        assert!(matches!(estimate.confidence, Confidence::Medium));

        let estimate = extractor.parse_context_estimate(&Some("8-12%".to_string()));
        assert!(matches!(estimate.confidence, Confidence::Low));
    }

    #[test]
    fn test_complexity_estimation() {
        // Would need a full ParsedSpec mock for comprehensive testing
    }
}
```

## Testing Requirements

- [ ] Unit tests for status parsing
- [ ] Tests for context estimate parsing
- [ ] Tests for complexity estimation
- [ ] Tests for dependency extraction
- [ ] Tests for bulk metadata indexing
- [ ] Tests for computed fields
- [ ] Tests for staleness calculation
- [ ] Integration tests with real specs

## Related Specs

- **120-spec-parsing.md**: Basic parsing this extends
- **124-progress-calc.md**: Progress calculation using metadata
- **127-spec-validation.md**: Validation using metadata
- **131-spec-search-index.md**: Search indexing of metadata
