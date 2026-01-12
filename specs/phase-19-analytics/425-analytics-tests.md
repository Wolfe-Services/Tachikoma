# Spec 425: Analytics Tests

## Phase
19 - Analytics/Telemetry

## Spec ID
425

## Status
Planned

## Dependencies
- Spec 406-424: All Analytics Specs

## Estimated Context
~12%

---

## Objective

Define comprehensive testing strategy and implement test suites for the entire analytics system, ensuring reliability, accuracy, and compliance across all analytics components.

---

## Acceptance Criteria

- [ ] Implement unit tests for all analytics modules
- [ ] Create integration tests for analytics pipeline
- [ ] Develop end-to-end tests for complete flows
- [ ] Implement performance/load tests
- [ ] Create privacy compliance tests
- [ ] Develop accuracy verification tests
- [ ] Implement chaos/resilience tests
- [ ] Create test utilities and fixtures

---

## Implementation Details

### Analytics Test Suite

```rust
// tests/analytics/mod.rs

//! Analytics Test Suite
//!
//! Comprehensive tests for all analytics functionality

pub mod unit;
pub mod integration;
pub mod e2e;
pub mod performance;
pub mod privacy;
pub mod fixtures;

// Re-export common test utilities
pub use fixtures::*;
```

### Test Fixtures and Utilities

```rust
// tests/analytics/fixtures.rs

use tachikoma::analytics::config::{AnalyticsConfig, AnalyticsConfigManager, StorageConfig};
use tachikoma::analytics::collector::EventCollector;
use tachikoma::analytics::storage::SqliteAnalyticsStorage;
use tachikoma::analytics::types::{
    AnalyticsEvent, EventBuilder, EventCategory, EventData, EventType,
    BusinessEventData, BusinessMetricType, ErrorEventData, ErrorSeverity,
    PerformanceEventData, UsageEventData,
};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

/// Test context holding all test dependencies
pub struct TestContext {
    pub temp_dir: TempDir,
    pub config: AnalyticsConfigManager,
    pub collector: Arc<EventCollector>,
    pub storage: Arc<SqliteAnalyticsStorage>,
}

impl TestContext {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config = AnalyticsConfigManager::new();

        let storage_config = StorageConfig {
            database_path: Some(temp_dir.path().join("analytics.db")),
            ..Default::default()
        };

        let storage = Arc::new(
            SqliteAnalyticsStorage::new(
                temp_dir.path().join("analytics.db"),
                storage_config,
            )
            .unwrap()
        );

        let collector = Arc::new(EventCollector::new(config.clone()));

        Self {
            temp_dir,
            config,
            collector,
            storage,
        }
    }

    /// Generate test events
    pub fn generate_events(&self, count: usize) -> Vec<AnalyticsEvent> {
        (0..count)
            .map(|i| {
                let event_type = match i % 5 {
                    0 => EventType::SessionStarted,
                    1 => EventType::MissionCreated,
                    2 => EventType::FeatureUsed,
                    3 => EventType::TokensConsumed,
                    _ => EventType::ResponseLatency,
                };

                EventBuilder::new(event_type)
                    .custom_metadata("test_index", serde_json::json!(i))
                    .build()
            })
            .collect()
    }

    /// Generate events with specific timestamps
    pub fn generate_events_over_time(
        &self,
        count: usize,
        start: DateTime<Utc>,
        interval: Duration,
    ) -> Vec<AnalyticsEvent> {
        (0..count)
            .map(|i| {
                let mut event = EventBuilder::new(EventType::FeatureUsed).build();
                // Note: Would need to modify event timestamp which requires mutable field
                event
            })
            .collect()
    }

    /// Generate usage events
    pub fn generate_usage_events(&self, count: usize) -> Vec<AnalyticsEvent> {
        (0..count)
            .map(|i| {
                EventBuilder::new(EventType::FeatureUsed)
                    .data(EventData::Usage(UsageEventData {
                        feature: format!("feature_{}", i % 5),
                        action: "use".to_string(),
                        target: Some("target".to_string()),
                        duration_ms: Some(100 + (i * 10) as u64),
                        success: i % 4 != 0,
                        extra: HashMap::new(),
                    }))
                    .build()
            })
            .collect()
    }

    /// Generate performance events
    pub fn generate_performance_events(&self, count: usize) -> Vec<AnalyticsEvent> {
        (0..count)
            .map(|i| {
                EventBuilder::new(EventType::ResponseLatency)
                    .data(EventData::Performance(PerformanceEventData {
                        metric: "latency".to_string(),
                        value: 100.0 + (i as f64 * 10.0),
                        unit: "ms".to_string(),
                        tags: [("backend".to_string(), "anthropic".to_string())]
                            .into_iter()
                            .collect(),
                    }))
                    .build()
            })
            .collect()
    }

    /// Generate error events
    pub fn generate_error_events(&self, count: usize) -> Vec<AnalyticsEvent> {
        let error_codes = ["NETWORK_ERROR", "AUTH_FAILED", "TIMEOUT", "RATE_LIMIT"];

        (0..count)
            .map(|i| {
                EventBuilder::new(EventType::ErrorOccurred)
                    .data(EventData::Error(ErrorEventData {
                        code: error_codes[i % error_codes.len()].to_string(),
                        message: format!("Test error {}", i),
                        severity: if i % 3 == 0 {
                            ErrorSeverity::Critical
                        } else {
                            ErrorSeverity::Error
                        },
                        stack_trace: None,
                        component: "test".to_string(),
                        recovered: i % 2 == 0,
                    }))
                    .build()
            })
            .collect()
    }

    /// Generate token consumption events
    pub fn generate_token_events(&self, count: usize) -> Vec<AnalyticsEvent> {
        let models = ["claude-3-opus", "claude-3-sonnet", "gpt-4"];

        (0..count)
            .map(|i| {
                EventBuilder::new(EventType::TokensConsumed)
                    .data(EventData::Business(BusinessEventData {
                        metric_type: BusinessMetricType::TotalTokens,
                        value: (1000 + i * 100) as f64,
                        unit: "tokens".to_string(),
                        backend: Some("anthropic".to_string()),
                        model: Some(models[i % models.len()].to_string()),
                    }))
                    .build()
            })
            .collect()
    }
}

/// Assertion helpers for analytics tests
pub mod assertions {
    use super::*;

    pub fn assert_event_category(event: &AnalyticsEvent, expected: EventCategory) {
        assert_eq!(
            event.category, expected,
            "Event category mismatch: expected {:?}, got {:?}",
            expected, event.category
        );
    }

    pub fn assert_events_ordered_by_time(events: &[AnalyticsEvent]) {
        for i in 1..events.len() {
            assert!(
                events[i - 1].timestamp <= events[i].timestamp,
                "Events not ordered by time at index {}",
                i
            );
        }
    }

    pub fn assert_no_pii(text: &str) {
        // Check for common PII patterns
        let pii_patterns = [
            r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
            r"\d{3}-\d{2}-\d{4}",
            r"sk-[a-zA-Z0-9]{20,}",
        ];

        for pattern in pii_patterns {
            let re = regex::Regex::new(pattern).unwrap();
            assert!(
                !re.is_match(text),
                "Found potential PII matching pattern: {}",
                pattern
            );
        }
    }

    pub fn assert_within_tolerance(actual: f64, expected: f64, tolerance: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff <= tolerance,
            "Value {} not within tolerance {} of expected {}",
            actual,
            tolerance,
            expected
        );
    }
}

/// Mock implementations for testing
pub mod mocks {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::sync::RwLock;

    /// Mock event sink that records all received events
    pub struct MockEventSink {
        pub events: Arc<RwLock<Vec<AnalyticsEvent>>>,
        pub flush_count: AtomicU64,
    }

    impl MockEventSink {
        pub fn new() -> Self {
            Self {
                events: Arc::new(RwLock::new(Vec::new())),
                flush_count: AtomicU64::new(0),
            }
        }

        pub async fn event_count(&self) -> usize {
            self.events.read().await.len()
        }

        pub async fn get_events(&self) -> Vec<AnalyticsEvent> {
            self.events.read().await.clone()
        }

        pub fn flush_count(&self) -> u64 {
            self.flush_count.load(Ordering::Relaxed)
        }
    }

    impl Default for MockEventSink {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Mock storage for testing without database
    pub struct MockStorage {
        events: Arc<RwLock<Vec<AnalyticsEvent>>>,
    }

    impl MockStorage {
        pub fn new() -> Self {
            Self {
                events: Arc::new(RwLock::new(Vec::new())),
            }
        }
    }
}
```

### Unit Tests

```rust
// tests/analytics/unit/mod.rs

pub mod types_tests;
pub mod config_tests;
pub mod collector_tests;
pub mod storage_tests;
pub mod aggregation_tests;
pub mod privacy_tests;

// tests/analytics/unit/types_tests.rs

use tachikoma::analytics::types::*;

#[test]
fn test_event_id_uniqueness() {
    let id1 = EventId::new();
    let id2 = EventId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_event_type_category_mapping() {
    let test_cases = vec![
        (EventType::SessionStarted, EventCategory::Usage),
        (EventType::ErrorOccurred, EventCategory::Error),
        (EventType::TokensConsumed, EventCategory::Business),
        (EventType::ResponseLatency, EventCategory::Performance),
        (EventType::AuthAttempted, EventCategory::Security),
        (EventType::ConfigChanged, EventCategory::System),
    ];

    for (event_type, expected_category) in test_cases {
        assert_eq!(
            event_type.category(),
            expected_category,
            "Event type {:?} should map to category {:?}",
            event_type,
            expected_category
        );
    }
}

#[test]
fn test_event_builder() {
    let event = EventBuilder::new(EventType::MissionCreated)
        .priority(EventPriority::High)
        .usage_data("mission", "create", true)
        .build();

    assert_eq!(event.event_type, EventType::MissionCreated);
    assert_eq!(event.priority, EventPriority::High);
    assert_eq!(event.category, EventCategory::Usage);
}

#[test]
fn test_event_serialization_roundtrip() {
    let original = EventBuilder::new(EventType::FeatureUsed)
        .usage_data("test", "action", true)
        .build();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: AnalyticsEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(original.id, deserialized.id);
    assert_eq!(original.event_type, deserialized.event_type);
}

#[test]
fn test_event_batch_creation() {
    let events: Vec<AnalyticsEvent> = (0..10)
        .map(|_| EventBuilder::new(EventType::FeatureUsed).build())
        .collect();

    let batch = EventBatch::new(events.clone(), 1);

    assert_eq!(batch.len(), 10);
    assert_eq!(batch.sequence, 1);
    assert!(!batch.is_empty());
}

#[test]
fn test_event_validation() {
    let event = EventBuilder::new(EventType::SessionStarted).build();
    let result = validate_event(&event);

    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_event_sanitization() {
    let event = EventBuilder::new(EventType::ErrorOccurred)
        .error_data(
            "API_ERROR",
            "Failed with key sk-abc123def456ghi789jkl012mno345pqr678stu901vwx234",
            ErrorSeverity::Error,
            "backend",
        )
        .build();

    let sanitized = sanitize_event(event);

    if let EventData::Error(error_data) = &sanitized.data {
        assert!(!error_data.message.contains("sk-abc123"));
        assert!(error_data.message.contains("[REDACTED"));
    }
}
```

### Integration Tests

```rust
// tests/analytics/integration/mod.rs

pub mod pipeline_tests;
pub mod storage_tests;
pub mod export_tests;

// tests/analytics/integration/pipeline_tests.rs

use crate::fixtures::*;
use tachikoma::analytics::collector::*;
use tachikoma::analytics::types::*;

#[tokio::test]
async fn test_event_collection_pipeline() {
    let ctx = TestContext::new().await;

    let sink = Arc::new(mocks::MockEventSink::new());
    ctx.collector.register_sink(sink.clone()).await;

    // Collect events
    let events = ctx.generate_events(100);
    for event in events {
        ctx.collector.collect(event).await.unwrap();
    }

    // Flush
    ctx.collector.flush().await.unwrap();

    // Verify sink received events
    assert!(sink.event_count().await >= 100);
}

#[tokio::test]
async fn test_event_sampling() {
    // Test that sampling reduces event count appropriately
    let ctx = TestContext::new().await;

    let sink = Arc::new(mocks::MockEventSink::new());
    ctx.collector.register_sink(sink.clone()).await;

    // Generate many performance events (which should be sampled)
    let events = ctx.generate_performance_events(1000);
    for event in events {
        ctx.collector.collect(event).await.unwrap();
    }

    ctx.collector.flush().await.unwrap();

    let stats = ctx.collector.stats();
    // Sampling should have reduced the number of processed events
    assert!(stats.events_sampled_out > 0 || stats.events_processed < 1000);
}

#[tokio::test]
async fn test_session_enrichment() {
    let ctx = TestContext::new().await;

    let sink = Arc::new(mocks::MockEventSink::new());
    ctx.collector.register_sink(sink.clone()).await;

    // Collect event without session ID
    let event = EventBuilder::new(EventType::SessionStarted).build();
    assert!(event.session_id.is_none());

    ctx.collector.collect(event).await.unwrap();
    ctx.collector.flush().await.unwrap();

    // Verify event was enriched with session ID
    let collected_events = sink.get_events().await;
    assert!(!collected_events.is_empty());
    assert!(collected_events[0].session_id.is_some());
    assert_eq!(collected_events[0].session_id, Some(ctx.collector.session_id()));
}
```

### End-to-End Tests

```rust
// tests/analytics/e2e/mod.rs

pub mod full_flow_tests;
pub mod report_tests;

// tests/analytics/e2e/full_flow_tests.rs

use crate::fixtures::*;
use tachikoma::analytics::aggregation::*;
use tachikoma::analytics::export::*;
use tachikoma::analytics::reports::*;
use tachikoma::analytics::storage::*;

#[tokio::test]
async fn test_full_analytics_flow() {
    let ctx = TestContext::new().await;

    // 1. Generate and collect events
    let events = ctx.generate_events(100);
    for event in events {
        ctx.collector.collect(event).await.unwrap();
    }
    ctx.collector.flush().await.unwrap();

    // 2. Store events
    // (Events would flow through storage sink)

    // 3. Query events
    let start = Utc::now() - chrono::Duration::hours(1);
    let end = Utc::now() + chrono::Duration::hours(1);
    let stored_events = ctx.storage.query_by_time(start, end, None).await.unwrap();

    // 4. Aggregate
    let aggregator = Aggregator::new(ctx.storage.clone());
    let spec = AggregationSpec::count("test", "Test Count", TimeGranularity::Hour);
    aggregator.register_spec(spec).await;

    let metrics = aggregator.aggregate("test", start, end).await.unwrap();

    // 5. Export
    let exporter = Exporter::new(ctx.storage.clone());
    let export_path = ctx.temp_dir.path().join("export");

    let result = exporter
        .export_to_file(
            &export_path,
            ExportFilter::new(),
            ExportOptions {
                format: ExportFormat::Json,
                compress: false,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert!(result.path.exists());
    assert!(result.metadata.event_count > 0);
}

#[tokio::test]
async fn test_report_generation_flow() {
    let ctx = TestContext::new().await;

    // Generate diverse events
    let usage_events = ctx.generate_usage_events(50);
    let perf_events = ctx.generate_performance_events(50);
    let error_events = ctx.generate_error_events(10);
    let token_events = ctx.generate_token_events(20);

    for event in usage_events
        .into_iter()
        .chain(perf_events)
        .chain(error_events)
        .chain(token_events)
    {
        ctx.collector.collect(event).await.unwrap();
    }
    ctx.collector.flush().await.unwrap();

    // Generate report
    let generator = ReportGenerator::new();

    let usage = UsageData {
        total_sessions: 100,
        total_missions: 50,
        completed_missions: 45,
        failed_missions: 5,
        total_commands: 500,
        by_feature: HashMap::new(),
        session_change: Some(10.0),
    };

    let costs = CostAggregation::default();
    let errors = ErrorStats::default();
    let performance = LatencyStats::default();

    let report = generator.generate_daily_summary(&usage, &costs, &errors, &performance);

    assert!(!report.sections.is_empty());
    assert_eq!(report.report_type, ReportType::DailySummary);
}
```

### Performance Tests

```rust
// tests/analytics/performance/mod.rs

pub mod load_tests;
pub mod stress_tests;

// tests/analytics/performance/load_tests.rs

use crate::fixtures::*;
use std::time::Instant;

#[tokio::test]
async fn test_high_volume_collection() {
    let ctx = TestContext::new().await;

    let start = Instant::now();
    let event_count = 10_000;

    // Generate and collect many events
    let events = ctx.generate_events(event_count);
    for event in events {
        ctx.collector.collect(event).await.unwrap();
    }

    ctx.collector.flush().await.unwrap();

    let elapsed = start.elapsed();
    let events_per_second = event_count as f64 / elapsed.as_secs_f64();

    println!(
        "Collected {} events in {:?} ({:.0} events/second)",
        event_count, elapsed, events_per_second
    );

    // Should handle at least 1000 events per second
    assert!(events_per_second > 1000.0);
}

#[tokio::test]
async fn test_storage_query_performance() {
    let ctx = TestContext::new().await;

    // Insert many events
    let events = ctx.generate_events(10_000);
    let batch = tachikoma::analytics::types::EventBatch::new(events, 1);
    ctx.storage.store_batch(&batch).await.unwrap();

    // Time queries
    let start = Instant::now();
    let query_count = 100;

    for _ in 0..query_count {
        let from = Utc::now() - chrono::Duration::hours(1);
        let to = Utc::now() + chrono::Duration::hours(1);
        let _ = ctx.storage.query_by_time(from, to, Some(100)).await.unwrap();
    }

    let elapsed = start.elapsed();
    let avg_query_time = elapsed / query_count;

    println!(
        "Average query time: {:?} ({} queries in {:?})",
        avg_query_time, query_count, elapsed
    );

    // Average query should be under 50ms
    assert!(avg_query_time.as_millis() < 50);
}

#[tokio::test]
async fn test_aggregation_performance() {
    let ctx = TestContext::new().await;

    // Insert events with varied timestamps
    let events = ctx.generate_events(5_000);
    let batch = tachikoma::analytics::types::EventBatch::new(events, 1);
    ctx.storage.store_batch(&batch).await.unwrap();

    let aggregator = Aggregator::new(ctx.storage.clone());
    let spec = AggregationSpec::count("perf_test", "Performance Test", TimeGranularity::Hour)
        .with_dimensions(vec!["category".to_string(), "event_type".to_string()]);
    aggregator.register_spec(spec).await;

    let start = Instant::now();

    let from = Utc::now() - chrono::Duration::days(7);
    let to = Utc::now();
    let _ = aggregator.aggregate("perf_test", from, to).await.unwrap();

    let elapsed = start.elapsed();

    println!("Aggregation completed in {:?}", elapsed);

    // Should complete in under 1 second
    assert!(elapsed.as_secs() < 1);
}
```

### Privacy Compliance Tests

```rust
// tests/analytics/privacy/mod.rs

pub mod pii_tests;
pub mod consent_tests;
pub mod retention_tests;

// tests/analytics/privacy/pii_tests.rs

use crate::fixtures::assertions::*;
use tachikoma::analytics::privacy::*;

#[test]
fn test_email_detection() {
    let detector = PiiDetector::default();

    let text = "Contact us at support@example.com for help";
    let detections = detector.detect(text);

    assert!(!detections.is_empty());
    assert!(detections.iter().any(|d| d.pii_type == PiiType::Email));
}

#[test]
fn test_api_key_detection() {
    let detector = PiiDetector::default();

    let text = "Using key sk-abc123def456ghi789jkl012mno345pqr678";
    let detections = detector.detect(text);

    assert!(!detections.is_empty());
    assert!(detections.iter().any(|d| d.pii_type == PiiType::ApiKey));
}

#[test]
fn test_pii_redaction() {
    let detector = PiiDetector::default();

    let text = "Email: user@domain.com, Key: sk-secretkey123456789012345";
    let redacted = detector.redact(text);

    assert!(!redacted.contains("user@domain.com"));
    assert!(!redacted.contains("sk-secretkey"));
    assert!(redacted.contains("[EMAIL_REDACTED]"));
    assert!(redacted.contains("[KEY_REDACTED]"));
}

#[test]
fn test_sensitive_field_detection() {
    let detector = PiiDetector::default();

    assert!(detector.is_sensitive_field("password"));
    assert!(detector.is_sensitive_field("api_key"));
    assert!(detector.is_sensitive_field("user_email"));
    assert!(!detector.is_sensitive_field("event_type"));
    assert!(!detector.is_sensitive_field("timestamp"));
}

#[test]
fn test_event_anonymization() {
    let anonymizer = DataAnonymizer::new(PiiDetector::default(), "test_salt");

    let event = EventBuilder::new(EventType::ErrorOccurred)
        .error_data(
            "ERROR",
            "Failed for user@example.com with key sk-secret123",
            ErrorSeverity::Error,
            "test",
        )
        .build();

    let anonymized = anonymizer.anonymize_event(event.clone());

    // Session ID should be hashed
    if event.session_id.is_some() && anonymized.session_id.is_some() {
        assert_ne!(event.session_id, anonymized.session_id);
    }

    // Error message should be sanitized
    if let EventData::Error(error_data) = &anonymized.data {
        assert_no_pii(&error_data.message);
    }
}

// tests/analytics/privacy/consent_tests.rs

use tachikoma::analytics::consent::*;

#[tokio::test]
async fn test_consent_categories() {
    let manager = ConsentManager::new();

    // Verify all categories start as pending
    for category in ConsentCategory::optional() {
        assert!(!manager.is_consented(category).await);
    }

    // Grant usage consent
    manager
        .grant(ConsentCategory::Usage, ConsentMethod::Api)
        .await
        .unwrap();

    assert!(manager.is_consented(ConsentCategory::Usage).await);
    assert!(!manager.is_consented(ConsentCategory::Performance).await);
}

#[tokio::test]
async fn test_essential_consent_protection() {
    let manager = ConsentManager::new();

    // Should not be able to deny essential consent
    let result = manager
        .deny(ConsentCategory::Essential, ConsentMethod::Api, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_consent_history() {
    let manager = ConsentManager::new();

    manager
        .grant(ConsentCategory::Usage, ConsentMethod::Api)
        .await
        .unwrap();

    manager
        .deny(ConsentCategory::ThirdParty, ConsentMethod::Settings, Some("Privacy"))
        .await
        .unwrap();

    let state = manager.get_state().await;
    assert_eq!(state.history.len(), 2);
}
```

---

## Testing Requirements Summary

### Test Categories

1. **Unit Tests** (~200 tests)
   - Type creation and validation
   - Serialization/deserialization
   - Configuration handling
   - Individual component logic

2. **Integration Tests** (~50 tests)
   - Component interactions
   - Data flow pipelines
   - Storage operations
   - Export functionality

3. **End-to-End Tests** (~20 tests)
   - Complete user workflows
   - Report generation
   - Dashboard data flow

4. **Performance Tests** (~15 tests)
   - Load handling
   - Query performance
   - Memory usage
   - Scalability

5. **Privacy Tests** (~30 tests)
   - PII detection accuracy
   - Consent enforcement
   - Data retention compliance

### Coverage Targets

- Line coverage: > 80%
- Branch coverage: > 70%
- Critical path coverage: 100%

### Test Infrastructure

- Automated CI/CD integration
- Test fixtures and factories
- Mock implementations
- Performance benchmarking

---

## Related Specs

- Spec 406-424: All Analytics Implementation Specs
