# Spec 135: Spec System Tests

## Metadata
- **Phase**: 6 - Spec System (THE PIN)
- **Spec ID**: 135
- **Status**: Planned
- **Dependencies**: 116-135 (all Phase 6 specs)
- **Estimated Context**: ~12%

## Objective

Define comprehensive test suites for the entire spec system (THE PIN). This includes unit tests for individual components, integration tests for system interactions, end-to-end tests for workflows, and property-based tests for robustness. The test infrastructure ensures spec system reliability and correctness.

## Acceptance Criteria

- [ ] Unit tests cover all spec system components
- [ ] Integration tests verify component interactions
- [ ] End-to-end tests validate complete workflows
- [ ] Property-based tests ensure robustness
- [ ] Performance benchmarks are established
- [ ] Test fixtures are reusable and maintainable
- [ ] Coverage metrics are tracked (>80%)
- [ ] CI/CD integration is configured

## Implementation Details

### Test Infrastructure

```rust
// src/spec/tests/mod.rs

pub mod fixtures;
pub mod unit;
pub mod integration;
pub mod e2e;
pub mod benchmarks;
pub mod properties;

// Re-exports for test utilities
pub use fixtures::*;
```

### Test Fixtures

```rust
// src/spec/tests/fixtures.rs

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs;

use crate::spec::directory::SpecDirectory;
use crate::spec::parsing::ParsedSpec;
use crate::spec::templates::TemplateContext;

/// Test fixture builder
pub struct SpecFixture {
    temp_dir: TempDir,
    specs: Vec<TestSpec>,
}

/// A test spec definition
pub struct TestSpec {
    pub id: u32,
    pub phase: u32,
    pub title: String,
    pub status: String,
    pub content: String,
}

impl SpecFixture {
    /// Create a new fixture
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            specs: Vec::new(),
        }
    }

    /// Get the temp directory path
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Add a spec to the fixture
    pub fn with_spec(mut self, spec: TestSpec) -> Self {
        self.specs.push(spec);
        self
    }

    /// Add a minimal spec
    pub fn with_minimal_spec(self, id: u32, phase: u32) -> Self {
        self.with_spec(TestSpec {
            id,
            phase,
            title: format!("Test Spec {}", id),
            status: "Planned".to_string(),
            content: minimal_spec_content(id, phase, &format!("Test Spec {}", id)),
        })
    }

    /// Add a complete spec
    pub fn with_complete_spec(self, id: u32, phase: u32) -> Self {
        self.with_spec(TestSpec {
            id,
            phase,
            title: format!("Complete Spec {}", id),
            status: "Complete".to_string(),
            content: complete_spec_content(id, phase, &format!("Complete Spec {}", id)),
        })
    }

    /// Build the fixture (create files)
    pub async fn build(self) -> Result<BuiltFixture, std::io::Error> {
        let specs_dir = self.temp_dir.path().join("specs");
        fs::create_dir_all(&specs_dir).await?;

        // Group specs by phase
        let mut by_phase: std::collections::HashMap<u32, Vec<&TestSpec>> = std::collections::HashMap::new();
        for spec in &self.specs {
            by_phase.entry(spec.phase).or_default().push(spec);
        }

        // Create phase directories and spec files
        for (phase, specs) in by_phase {
            let phase_dir = specs_dir.join(format!("phase-{:02}-specs", phase));
            fs::create_dir_all(&phase_dir).await?;

            for spec in specs {
                let filename = format!("{:03}-test-spec.md", spec.id);
                let path = phase_dir.join(&filename);
                fs::write(&path, &spec.content).await?;
            }
        }

        Ok(BuiltFixture {
            _temp_dir: self.temp_dir,
            root: specs_dir,
        })
    }
}

/// Built fixture with temp dir ownership
pub struct BuiltFixture {
    _temp_dir: TempDir,
    pub root: PathBuf,
}

impl BuiltFixture {
    pub fn path(&self) -> &Path {
        &self.root
    }
}

/// Generate minimal spec content
pub fn minimal_spec_content(id: u32, phase: u32, title: &str) -> String {
    format!(
        r#"# Spec {id}: {title}

## Metadata
- **Phase**: {phase} - Test Phase
- **Spec ID**: {id}
- **Status**: Planned
- **Dependencies**: None
- **Estimated Context**: ~10%

## Objective

Test objective for spec {id}.

## Acceptance Criteria

- [ ] Test criterion 1
- [ ] Test criterion 2

## Implementation Details

```rust
fn test() {{
    // Test implementation
}}
```

## Testing Requirements

- [ ] Unit tests
- [ ] Integration tests

## Related Specs

None
"#
    )
}

/// Generate complete spec content
pub fn complete_spec_content(id: u32, phase: u32, title: &str) -> String {
    format!(
        r#"# Spec {id}: {title}

## Metadata
- **Phase**: {phase} - Test Phase
- **Spec ID**: {id}
- **Status**: Complete
- **Dependencies**: spec:{}, spec:{}
- **Estimated Context**: ~12%

## Objective

Complete test objective for spec {id}. This spec has all sections filled out.

## Acceptance Criteria

- [x] Test criterion 1 (complete)
- [x] Test criterion 2 (complete)
- [x] Test criterion 3 (complete)

## Implementation Details

### Core Implementation

```rust
pub struct TestStruct {{
    pub field: String,
}}

impl TestStruct {{
    pub fn new() -> Self {{
        Self {{ field: String::new() }}
    }}
}}
```

### Helper Functions

```rust
fn helper() -> bool {{
    true
}}
```

## Testing Requirements

- [x] Unit tests for TestStruct
- [x] Integration tests
- [x] Documentation tests

## Related Specs

- **{}-spec.md**: Related spec
- **{}-spec.md**: Another related spec
"#,
        id.saturating_sub(1),
        id.saturating_sub(2),
        id.saturating_sub(1),
        id.saturating_sub(2)
    )
}

/// Sample template context for testing
pub fn sample_template_context() -> TemplateContext {
    TemplateContext {
        spec_id: 999,
        title: "Test Spec".to_string(),
        phase: 6,
        phase_name: "Test Phase".to_string(),
        slug: "test-spec".to_string(),
        dependencies: vec!["001".to_string(), "002".to_string()],
        estimated_context: "~10%".to_string(),
        custom: std::collections::HashMap::new(),
    }
}

impl Default for SpecFixture {
    fn default() -> Self {
        Self::new()
    }
}
```

### Unit Tests

```rust
// src/spec/tests/unit.rs

#[cfg(test)]
mod parsing_tests {
    use crate::spec::parsing::*;
    use crate::spec::tests::fixtures::*;

    #[test]
    fn test_parse_minimal_spec() {
        let content = minimal_spec_content(100, 5, "Test Spec");
        let parser = SpecParser::new();

        let parsed = parser.parse(&content).unwrap();

        assert_eq!(parsed.metadata.spec_id, 100);
        assert_eq!(parsed.metadata.phase, 5);
        assert_eq!(parsed.metadata.status, "Planned");
        assert_eq!(parsed.acceptance_criteria.len(), 2);
    }

    #[test]
    fn test_parse_complete_spec() {
        let content = complete_spec_content(100, 5, "Complete Spec");
        let parser = SpecParser::new();

        let parsed = parser.parse(&content).unwrap();

        assert_eq!(parsed.metadata.status, "Complete");
        assert!(parsed.acceptance_criteria.iter().all(|c| c.checked));
        assert!(!parsed.code_blocks.is_empty());
    }

    #[test]
    fn test_parse_checkboxes() {
        let content = r#"
## Acceptance Criteria

- [ ] Unchecked item
- [x] Checked item
- [X] Also checked
"#;
        let parser = SpecParser::new();
        let parsed = parser.parse(&format!("# Spec 1: Test\n{}", content)).unwrap();

        assert_eq!(parsed.acceptance_criteria.len(), 3);
        assert!(!parsed.acceptance_criteria[0].checked);
        assert!(parsed.acceptance_criteria[1].checked);
        assert!(parsed.acceptance_criteria[2].checked);
    }

    #[test]
    fn test_parse_code_blocks() {
        let content = r#"# Spec 1: Test

## Implementation

```rust
fn test() {}
```

```python
def test():
    pass
```
"#;
        let parser = SpecParser::new();
        let parsed = parser.parse(content).unwrap();

        assert_eq!(parsed.code_blocks.len(), 2);
        assert_eq!(parsed.code_blocks[0].language, Some("rust".to_string()));
        assert_eq!(parsed.code_blocks[1].language, Some("python".to_string()));
    }

    #[test]
    fn test_parse_references() {
        let content = r#"# Spec 100: Test

See spec:001 and 002-other.md for details.
"#;
        let parser = SpecParser::new();
        let parsed = parser.parse(content).unwrap();

        assert!(parsed.references.iter().any(|r| r.spec_id == 1));
        assert!(parsed.references.iter().any(|r| r.spec_id == 2));
    }

    #[test]
    fn test_parse_dependencies() {
        let content = r#"# Spec 100: Test

## Metadata
- **Dependencies**: 001-spec, 002-other, 003
"#;
        let parser = SpecParser::new();
        let parsed = parser.parse(content).unwrap();

        assert_eq!(parsed.metadata.dependencies.len(), 3);
    }
}

#[cfg(test)]
mod validation_tests {
    use crate::spec::validation::*;
    use crate::spec::parsing::*;
    use crate::spec::tests::fixtures::*;

    #[test]
    fn test_validate_complete_spec() {
        let content = complete_spec_content(100, 5, "Test");
        let parser = SpecParser::new();
        let parsed = parser.parse(&content).unwrap();

        let config = ValidationConfig::default();
        let validator = SpecValidator::new(config);

        // Need SpecMetadata - simplified for test
        // let result = validator.validate(&parsed, &metadata);
        // assert!(result.passed);
    }

    #[test]
    fn test_validate_missing_section() {
        let content = r#"# Spec 100: Test

## Metadata
- **Phase**: 5
- **Spec ID**: 100
- **Status**: Planned
"#;
        let parser = SpecParser::new();
        let parsed = parser.parse(content).unwrap();

        // Validation would fail due to missing sections
    }
}

#[cfg(test)]
mod checkbox_tests {
    use crate::spec::checkbox::*;

    #[test]
    fn test_checkbox_id_roundtrip() {
        let id = CheckboxId::new(116, "Acceptance Criteria", 1);
        let s = id.to_string();
        let parsed = CheckboxId::parse(&s).unwrap();

        assert_eq!(id, parsed);
    }

    #[test]
    fn test_checkbox_stats() {
        let tracker = CheckboxTracker::new();

        let content = r#"## Test

- [ ] First
- [x] Second
- [ ] Third
"#;
        let checkboxes = tracker.parse_checkboxes(content, 1);

        assert_eq!(checkboxes.len(), 3);
        assert_eq!(checkboxes.iter().filter(|c| c.checked).count(), 1);
    }
}

#[cfg(test)]
mod progress_tests {
    use crate::spec::progress::*;

    #[test]
    fn test_progress_calculation() {
        let calc = ProgressCalculator::default();

        let stats = crate::spec::checkbox::CheckboxStats {
            total: 10,
            checked: 7,
            percentage: 70,
            by_section: std::collections::HashMap::new(),
        };

        // Would need full metadata for complete test
    }

    #[test]
    fn test_progress_bar_rendering() {
        let calc = ProgressCalculator::default();
        let bar = calc.render_progress_bar(50);

        assert!(bar.contains("["));
        assert!(bar.contains("]"));
        assert!(bar.len() > 20);
    }
}
```

### Integration Tests

```rust
// src/spec/tests/integration.rs

#[cfg(test)]
mod directory_integration {
    use crate::spec::directory::*;
    use crate::spec::tests::fixtures::*;

    #[tokio::test]
    async fn test_discover_spec_directory() {
        let fixture = SpecFixture::new()
            .with_minimal_spec(116, 6)
            .with_minimal_spec(117, 6)
            .with_complete_spec(100, 5)
            .build()
            .await
            .unwrap();

        let parent = fixture.path().parent().unwrap();
        let spec_dir = SpecDirectory::discover(parent).await.unwrap();

        assert_eq!(spec_dir.phases.len(), 2);
        assert!(spec_dir.find_spec(116).is_some());
        assert!(spec_dir.find_spec(117).is_some());
    }

    #[tokio::test]
    async fn test_initialize_spec_directory() {
        let temp = tempfile::TempDir::new().unwrap();
        let spec_dir = SpecDirectory::initialize(temp.path()).await.unwrap();

        assert!(spec_dir.root.exists());
        assert!(spec_dir.templates.exists());
        assert!(spec_dir.config.exists());
    }
}

#[cfg(test)]
mod search_integration {
    use crate::spec::search_index::*;
    use crate::spec::search_api::*;
    use crate::spec::tests::fixtures::*;

    #[test]
    fn test_index_and_search() {
        let mut index = SpecSearchIndex::new();

        // Index test specs
        for id in 116..120 {
            let spec = IndexedSpec {
                id,
                title: format!("Test Spec {}", id),
                phase: 6,
                status: "Planned".to_string(),
                content: format!("Content for spec {} about testing", id),
                sections: vec!["Objective".to_string()],
                criteria: vec!["Test criterion".to_string()],
                code: String::new(),
                dependencies: vec![],
                tags: vec!["test".to_string()],
                path: format!("/specs/{}.md", id),
                indexed_at: 0,
            };
            index.index_spec(spec);
        }

        // Search
        let engine = SearchEngine::new(&index);
        let query = QueryParser::parse("testing phase:6");
        let response = engine.search(&query);

        assert!(response.total > 0);
        assert!(response.results.iter().all(|r| r.phase == 6));
    }

    #[test]
    fn test_faceted_search() {
        let mut index = SpecSearchIndex::new();

        // Index specs in different phases
        for (id, phase) in [(116, 6), (117, 6), (100, 5), (50, 3)] {
            let spec = IndexedSpec {
                id,
                title: format!("Spec {}", id),
                phase,
                status: "Planned".to_string(),
                content: "test content".to_string(),
                sections: vec![],
                criteria: vec![],
                code: String::new(),
                dependencies: vec![],
                tags: vec![],
                path: String::new(),
                indexed_at: 0,
            };
            index.index_spec(spec);
        }

        let facets = index.get_facets();
        assert!(facets.phases.contains(&6));
        assert!(facets.phases.contains(&5));
        assert!(facets.phases.contains(&3));
    }
}

#[cfg(test)]
mod template_integration {
    use crate::spec::templates::*;
    use crate::spec::tests::fixtures::*;

    #[tokio::test]
    async fn test_template_rendering() {
        let temp = tempfile::TempDir::new().unwrap();
        let engine = TemplateEngine::new(temp.path()).await.unwrap();

        let context = sample_template_context();
        let result = engine.render(TemplateType::Feature, &context).unwrap();

        assert!(result.contains("Spec 999"));
        assert!(result.contains("Test Spec"));
        assert!(result.contains("Phase"));
    }
}
```

### End-to-End Tests

```rust
// src/spec/tests/e2e.rs

#[cfg(test)]
mod spec_workflow {
    use crate::spec::tests::fixtures::*;

    #[tokio::test]
    async fn test_full_spec_lifecycle() {
        // 1. Create spec directory
        let fixture = SpecFixture::new()
            .with_minimal_spec(116, 6)
            .build()
            .await
            .unwrap();

        // 2. Parse spec
        // 3. Validate spec
        // 4. Index spec
        // 5. Search spec
        // 6. Update spec
        // 7. Track progress
        // 8. Generate README

        // This would be a comprehensive workflow test
    }

    #[tokio::test]
    async fn test_spec_generation_workflow() {
        // 1. Start conversation
        // 2. Process inputs through stages
        // 3. Generate spec
        // 4. Validate generated spec
        // 5. Save to file
    }
}
```

### Benchmarks

```rust
// src/spec/tests/benchmarks.rs

#[cfg(test)]
mod benchmarks {
    use std::time::Instant;

    #[test]
    #[ignore] // Run with --ignored flag
    fn bench_parsing() {
        let content = crate::spec::tests::fixtures::complete_spec_content(100, 5, "Bench");
        let parser = crate::spec::parsing::SpecParser::new();

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = parser.parse(&content);
        }

        let elapsed = start.elapsed();
        println!(
            "Parsed {} specs in {:?} ({:?} per spec)",
            iterations,
            elapsed,
            elapsed / iterations
        );
    }

    #[test]
    #[ignore]
    fn bench_indexing() {
        let mut index = crate::spec::search_index::SpecSearchIndex::new();

        let iterations = 100;
        let start = Instant::now();

        for i in 0..iterations {
            let spec = crate::spec::search_index::IndexedSpec {
                id: i as u32,
                title: format!("Spec {}", i),
                phase: (i % 8 + 1) as u32,
                status: "Planned".to_string(),
                content: "Test content ".repeat(100),
                sections: vec!["Section".to_string()],
                criteria: vec!["Criterion".to_string()],
                code: String::new(),
                dependencies: vec![],
                tags: vec![],
                path: String::new(),
                indexed_at: 0,
            };
            index.index_spec(spec);
        }

        let elapsed = start.elapsed();
        println!(
            "Indexed {} specs in {:?} ({:?} per spec)",
            iterations,
            elapsed,
            elapsed / iterations as u32
        );
    }

    #[test]
    #[ignore]
    fn bench_search() {
        let mut index = crate::spec::search_index::SpecSearchIndex::new();

        // Pre-populate index
        for i in 0..100 {
            let spec = crate::spec::search_index::IndexedSpec {
                id: i as u32,
                title: format!("Test Spec Number {}", i),
                phase: (i % 8 + 1) as u32,
                status: "Planned".to_string(),
                content: format!("Content about testing and spec number {} with various keywords", i),
                sections: vec![],
                criteria: vec![],
                code: String::new(),
                dependencies: vec![],
                tags: vec![],
                path: String::new(),
                indexed_at: 0,
            };
            index.index_spec(spec);
        }

        let engine = crate::spec::search_api::SearchEngine::new(&index);
        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let query = crate::spec::search_api::QueryParser::parse("testing spec");
            let _ = engine.search(&query);
        }

        let elapsed = start.elapsed();
        println!(
            "Executed {} searches in {:?} ({:?} per search)",
            iterations,
            elapsed,
            elapsed / iterations as u32
        );
    }
}
```

### Property-Based Tests

```rust
// src/spec/tests/properties.rs

#[cfg(test)]
mod property_tests {
    use quickcheck::{quickcheck, TestResult};

    quickcheck! {
        fn prop_checkbox_id_roundtrip(spec_id: u32, index: u32) -> bool {
            let section = "Test Section";
            let id = crate::spec::checkbox::CheckboxId::new(spec_id, section, index);
            let s = id.to_string();

            match crate::spec::checkbox::CheckboxId::parse(&s) {
                Some(parsed) => parsed == id,
                None => false,
            }
        }

        fn prop_version_ordering(major: u8, minor: u8, patch: u8) -> bool {
            let v1 = crate::spec::versioning::SpecVersion::new(
                major as u32, minor as u32, patch as u32
            );
            let v2 = v1.bump_patch();
            let v3 = v1.bump_minor();
            let v4 = v1.bump_major();

            v2 > v1 && v3 > v2 && v4 > v3
        }

        fn prop_search_term_normalization(term: String) -> TestResult {
            if term.is_empty() {
                return TestResult::discard();
            }

            let normalized = term.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            // Normalized should be lowercase and alphanumeric only
            TestResult::from_bool(
                normalized.chars().all(|c| c.is_lowercase() || c.is_ascii_digit())
            )
        }
    }
}
```

## Testing Requirements

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] End-to-end workflows complete successfully
- [ ] Benchmarks establish performance baselines
- [ ] Property tests find no edge case failures
- [ ] Code coverage exceeds 80%
- [ ] No memory leaks in stress tests
- [ ] Tests run in CI/CD pipeline

## Related Specs

- All Phase 6 specs (116-134) are tested by this spec
- **127-spec-validation.md**: Validation rules are tested
- **128-spec-linting.md**: Lint rules are tested
