// src/spec/pattern_link.rs

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use regex::Regex;

/// A pattern reference in a spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternReference {
    /// Pattern ID or name
    pub pattern_id: String,
    /// Pattern category
    pub category: PatternCategory,
    /// Reference format used
    pub format: PatternRefFormat,
    /// Source spec ID
    pub spec_id: u32,
    /// Line number in spec
    pub line: usize,
    /// Reference context (surrounding text)
    pub context: String,
    /// Relationship type
    pub relationship: PatternRelationship,
}

/// Pattern categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Architectural patterns (MVC, CQRS, etc.)
    Architectural,
    /// Design patterns (Singleton, Factory, etc.)
    Design,
    /// Code patterns (idioms, conventions)
    Code,
    /// Anti-patterns (things to avoid)
    AntiPattern,
    /// Project-specific patterns
    Custom,
}

/// Pattern reference formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternRefFormat {
    /// pattern:name
    PatternColon,
    /// [[pattern:name]]
    WikiLink,
    /// @pattern name
    AtPattern,
    /// Uses: pattern-name
    UsesTag,
    /// Implements Pattern: Name
    ImplementsTag,
}

/// Relationship between spec and pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternRelationship {
    /// Spec implements this pattern
    Implements,
    /// Spec uses this pattern
    Uses,
    /// Spec extends this pattern
    Extends,
    /// Spec is related to this pattern
    RelatedTo,
    /// Spec avoids this anti-pattern
    Avoids,
}

/// Pattern definition (from pattern system)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    /// Pattern ID
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Category
    pub category: PatternCategory,
    /// Description
    pub description: String,
    /// Parent pattern (for inheritance)
    pub parent: Option<String>,
    /// Related patterns
    pub related: Vec<String>,
    /// File path
    pub path: PathBuf,
}

/// Pattern reference parser
pub struct PatternRefParser {
    patterns: Vec<(PatternRefFormat, Regex)>,
}

impl PatternRefParser {
    pub fn new() -> Self {
        let patterns = vec![
            (PatternRefFormat::WikiLink, Regex::new(r"\[\[pattern:([^\]]+)\]\]").unwrap()),
            (PatternRefFormat::ImplementsTag, Regex::new(r"(?i)implements\s+pattern:\s*([^\n,]+)").unwrap()),
            (PatternRefFormat::UsesTag, Regex::new(r"(?i)uses:\s*pattern[:\-]([^\n,]+)").unwrap()),
            (PatternRefFormat::AtPattern, Regex::new(r"@pattern\s+([^\n]+)").unwrap()),
            (PatternRefFormat::PatternColon, Regex::new(r"pattern:([a-zA-Z][a-zA-Z0-9_\-]+)").unwrap()),
        ];

        Self { patterns }
    }

    /// Parse pattern references from spec content
    pub fn parse(&self, content: &str, spec_id: u32) -> Vec<PatternReference> {
        let mut references = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for (format, regex) in &self.patterns {
                for caps in regex.captures_iter(line) {
                    if let Some(pattern_match) = caps.get(1) {
                        let pattern_id = pattern_match.as_str().trim().to_string();
                        let relationship = self.infer_relationship(line, *format);
                        let category = self.infer_category(&pattern_id);

                        references.push(PatternReference {
                            pattern_id,
                            category,
                            format: *format,
                            spec_id,
                            line: line_num,
                            context: line.to_string(),
                            relationship,
                        });
                    }
                }
            }
        }

        references
    }

    /// Infer relationship from context
    fn infer_relationship(&self, line: &str, format: PatternRefFormat) -> PatternRelationship {
        let lower = line.to_lowercase();

        match format {
            PatternRefFormat::ImplementsTag => PatternRelationship::Implements,
            PatternRefFormat::UsesTag => PatternRelationship::Uses,
            _ => {
                if lower.contains("implement") {
                    PatternRelationship::Implements
                } else if lower.contains("extend") {
                    PatternRelationship::Extends
                } else if lower.contains("avoid") || lower.contains("anti-pattern") {
                    PatternRelationship::Avoids
                } else if lower.contains("use") || lower.contains("apply") {
                    PatternRelationship::Uses
                } else {
                    PatternRelationship::RelatedTo
                }
            }
        }
    }

    /// Infer category from pattern name
    fn infer_category(&self, pattern_id: &str) -> PatternCategory {
        let lower = pattern_id.to_lowercase();

        // Architectural patterns
        let architectural = ["mvc", "mvvm", "cqrs", "event-sourcing", "microservice",
            "hexagonal", "clean-architecture", "layered", "pipe-filter"];
        if architectural.iter().any(|p| lower.contains(p)) {
            return PatternCategory::Architectural;
        }

        // Design patterns
        let design = ["singleton", "factory", "builder", "observer", "strategy",
            "adapter", "decorator", "facade", "proxy", "command", "visitor"];
        if design.iter().any(|p| lower.contains(p)) {
            return PatternCategory::Design;
        }

        // Anti-patterns
        if lower.contains("anti") || lower.contains("avoid") {
            return PatternCategory::AntiPattern;
        }

        PatternCategory::Custom
    }
}

/// Bidirectional pattern index
pub struct PatternIndex {
    /// Pattern definitions
    patterns: HashMap<String, PatternDefinition>,
    /// Specs implementing each pattern
    pattern_to_specs: HashMap<String, HashSet<u32>>,
    /// Patterns used by each spec
    spec_to_patterns: HashMap<u32, Vec<PatternReference>>,
}

impl PatternIndex {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            pattern_to_specs: HashMap::new(),
            spec_to_patterns: HashMap::new(),
        }
    }

    /// Load pattern definition
    pub fn load_pattern(&mut self, pattern: PatternDefinition) {
        self.patterns.insert(pattern.id.clone(), pattern);
    }

    /// Add pattern reference from a spec
    pub fn add_reference(&mut self, reference: PatternReference) {
        self.pattern_to_specs
            .entry(reference.pattern_id.clone())
            .or_default()
            .insert(reference.spec_id);

        self.spec_to_patterns
            .entry(reference.spec_id)
            .or_default()
            .push(reference);
    }

    /// Get specs implementing a pattern
    pub fn specs_for_pattern(&self, pattern_id: &str) -> Vec<u32> {
        self.pattern_to_specs
            .get(pattern_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get patterns used by a spec
    pub fn patterns_for_spec(&self, spec_id: u32) -> &[PatternReference] {
        self.spec_to_patterns
            .get(&spec_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get pattern definition
    pub fn get_pattern(&self, pattern_id: &str) -> Option<&PatternDefinition> {
        self.patterns.get(pattern_id)
    }

    /// Validate pattern reference
    pub fn validate_reference(&self, reference: &PatternReference) -> ValidationResult {
        if let Some(pattern) = self.patterns.get(&reference.pattern_id) {
            ValidationResult::Valid {
                pattern_name: pattern.name.clone(),
                pattern_path: pattern.path.clone(),
            }
        } else {
            // Check for similar patterns
            let similar = self.find_similar(&reference.pattern_id);
            ValidationResult::Invalid {
                message: format!("Pattern '{}' not found", reference.pattern_id),
                suggestions: similar,
            }
        }
    }

    /// Find similar pattern names (for suggestions)
    fn find_similar(&self, pattern_id: &str) -> Vec<String> {
        let lower = pattern_id.to_lowercase();
        self.patterns.keys()
            .filter(|k| {
                let k_lower = k.to_lowercase();
                k_lower.contains(&lower) || lower.contains(&k_lower) ||
                Self::levenshtein(&k_lower, &lower) <= 3
            })
            .take(3)
            .cloned()
            .collect()
    }

    /// Simple Levenshtein distance
    fn levenshtein(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();

        let mut matrix = vec![vec![0usize; b.len() + 1]; a.len() + 1];

        for i in 0..=a.len() {
            matrix[i][0] = i;
        }
        for j in 0..=b.len() {
            matrix[0][j] = j;
        }

        for i in 1..=a.len() {
            for j in 1..=b.len() {
                let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a.len()][b.len()]
    }

    /// Get pattern usage statistics
    pub fn get_statistics(&self) -> PatternStatistics {
        let mut category_counts: HashMap<PatternCategory, usize> = HashMap::new();
        let mut relationship_counts: HashMap<PatternRelationship, usize> = HashMap::new();

        for references in self.spec_to_patterns.values() {
            for reference in references {
                *category_counts.entry(reference.category).or_default() += 1;
                *relationship_counts.entry(reference.relationship).or_default() += 1;
            }
        }

        let most_used: Vec<_> = self.pattern_to_specs.iter()
            .map(|(id, specs)| (id.clone(), specs.len()))
            .collect::<Vec<_>>()
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .into_iter()
            .take(10)
            .collect();

        PatternStatistics {
            total_patterns: self.patterns.len(),
            total_references: self.spec_to_patterns.values().map(|v| v.len()).sum(),
            specs_with_patterns: self.spec_to_patterns.len(),
            by_category: category_counts,
            by_relationship: relationship_counts,
            most_used_patterns: most_used,
        }
    }
}

/// Validation result for pattern reference
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid {
        pattern_name: String,
        pattern_path: PathBuf,
    },
    Invalid {
        message: String,
        suggestions: Vec<String>,
    },
}

/// Pattern usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    pub total_patterns: usize,
    pub total_references: usize,
    pub specs_with_patterns: usize,
    pub by_category: HashMap<PatternCategory, usize>,
    pub by_relationship: HashMap<PatternRelationship, usize>,
    pub most_used_patterns: Vec<(String, usize)>,
}

/// Generate pattern link documentation
pub struct PatternLinkGenerator;

impl PatternLinkGenerator {
    /// Generate pattern reference for spec
    pub fn generate_reference(pattern_id: &str, relationship: PatternRelationship) -> String {
        match relationship {
            PatternRelationship::Implements => format!("Implements Pattern: {}", pattern_id),
            PatternRelationship::Uses => format!("Uses: pattern:{}", pattern_id),
            PatternRelationship::Extends => format!("Extends Pattern: {}", pattern_id),
            PatternRelationship::Avoids => format!("Avoids Anti-Pattern: {}", pattern_id),
            PatternRelationship::RelatedTo => format!("Related Pattern: [[pattern:{}]]", pattern_id),
        }
    }

    /// Generate patterns section for spec
    pub fn generate_section(references: &[PatternReference]) -> String {
        let mut output = String::new();
        output.push_str("## Related Patterns\n\n");

        for reference in references {
            output.push_str(&format!("- **{}**: {:?} ({})\n",
                reference.pattern_id,
                reference.relationship,
                reference.category.as_str()
            ));
        }

        output
    }
}

impl PatternCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Architectural => "Architectural",
            Self::Design => "Design",
            Self::Code => "Code",
            Self::AntiPattern => "Anti-Pattern",
            Self::Custom => "Custom",
        }
    }
}

impl Default for PatternRefParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PatternIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_reference_parsing() {
        let parser = PatternRefParser::new();
        let content = r#"
This spec implements the pattern:singleton design.
Uses: pattern-factory for object creation.
Implements Pattern: Observer
See also [[pattern:strategy]]
"#;

        let refs = parser.parse(content, 116);

        assert!(refs.iter().any(|r| r.pattern_id == "singleton"));
        assert!(refs.iter().any(|r| r.pattern_id == "factory"));
        assert!(refs.iter().any(|r| r.pattern_id == "Observer"));
        assert!(refs.iter().any(|r| r.pattern_id == "strategy"));
    }

    #[test]
    fn test_category_inference() {
        let parser = PatternRefParser::new();

        assert_eq!(parser.infer_category("mvc"), PatternCategory::Architectural);
        assert_eq!(parser.infer_category("singleton"), PatternCategory::Design);
        assert_eq!(parser.infer_category("anti-pattern-x"), PatternCategory::AntiPattern);
        assert_eq!(parser.infer_category("my-custom-pattern"), PatternCategory::Custom);
    }

    #[test]
    fn test_pattern_index() {
        let mut index = PatternIndex::new();

        index.load_pattern(PatternDefinition {
            id: "singleton".to_string(),
            name: "Singleton".to_string(),
            category: PatternCategory::Design,
            description: "Ensures single instance".to_string(),
            parent: None,
            related: vec![],
            path: PathBuf::new(),
        });

        let reference = PatternReference {
            pattern_id: "singleton".to_string(),
            category: PatternCategory::Design,
            format: PatternRefFormat::PatternColon,
            spec_id: 116,
            line: 10,
            context: String::new(),
            relationship: PatternRelationship::Implements,
        };

        index.add_reference(reference);

        let specs = index.specs_for_pattern("singleton");
        assert!(specs.contains(&116));

        let patterns = index.patterns_for_spec(116);
        assert!(!patterns.is_empty());
    }
}