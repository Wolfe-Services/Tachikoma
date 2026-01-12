# 450 - Audit Tests

**Phase:** 20 - Audit System
**Spec ID:** 450
**Status:** Planned
**Dependencies:** All previous audit specs (431-449)
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Comprehensive test suite for the audit system, covering unit tests, integration tests, and property-based testing.

---

## Acceptance Criteria

- [ ] Unit tests for all modules
- [ ] Integration tests for persistence
- [ ] Property-based tests for serialization
- [ ] Performance benchmarks
- [ ] Security tests

---

## Implementation Details

### 1. Test Utilities (tests/common/mod.rs)

```rust
//! Common test utilities for audit system.

use tachikoma_audit::*;
use rusqlite::Connection;
use std::sync::Arc;
use parking_lot::Mutex;
use tempfile::TempDir;

/// Create an in-memory test database.
pub fn test_db() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().unwrap();
    crate::schema::run_migrations(&conn).unwrap();
    Arc::new(Mutex::new(conn))
}

/// Create a test database with temp directory.
pub fn test_db_with_dir() -> (Arc<Mutex<Connection>>, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_audit.db");
    let conn = Connection::open(&db_path).unwrap();
    crate::schema::run_migrations(&conn).unwrap();
    (Arc::new(Mutex::new(conn)), dir)
}

/// Create a test audit event.
pub fn test_event(category: AuditCategory, action: AuditAction) -> AuditEvent {
    AuditEvent::builder(category, action)
        .actor(AuditActor::system("test"))
        .build()
}

/// Create multiple test events.
pub fn test_events(count: usize) -> Vec<AuditEvent> {
    (0..count)
        .map(|i| {
            AuditEvent::builder(
                AuditCategory::System,
                AuditAction::Custom(format!("test_action_{}", i)),
            )
            .actor(AuditActor::system("test"))
            .metadata("index", i)
            .build()
        })
        .collect()
}

/// Assert events are equal (ignoring timestamps).
pub fn assert_events_equal(a: &AuditEvent, b: &AuditEvent) {
    assert_eq!(a.category, b.category);
    assert_eq!(a.action, b.action);
    assert_eq!(a.severity, b.severity);
}
```

### 2. Event Type Tests (tests/event_types_test.rs)

```rust
//! Tests for audit event types.

use tachikoma_audit::*;
use proptest::prelude::*;

#[test]
fn test_event_builder() {
    let event = AuditEvent::builder(AuditCategory::Security, AuditAction::Login)
        .actor(AuditActor::user(UserId::new()))
        .severity(AuditSeverity::Low)
        .outcome(AuditOutcome::Success)
        .build();

    assert_eq!(event.category, AuditCategory::Security);
    assert!(matches!(event.action, AuditAction::Login));
    assert_eq!(event.severity, AuditSeverity::Low);
    assert!(event.outcome.is_success());
}

#[test]
fn test_default_severity() {
    let critical_event = AuditEvent::builder(
        AuditCategory::Security,
        AuditAction::DataBreach,
    ).build();

    assert_eq!(critical_event.severity, AuditSeverity::Critical);

    let info_event = AuditEvent::builder(
        AuditCategory::System,
        AuditAction::SystemStartup,
    ).build();

    assert_eq!(info_event.severity, AuditSeverity::Info);
}

#[test]
fn test_actor_identifier() {
    let user_actor = AuditActor::User {
        user_id: UserId::new(),
        username: Some("testuser".to_string()),
        session_id: None,
    };
    assert_eq!(user_actor.identifier(), "testuser");

    let system_actor = AuditActor::system("scheduler");
    assert_eq!(system_actor.identifier(), "system:scheduler");
}

#[test]
fn test_severity_ordering() {
    assert!(AuditSeverity::Critical > AuditSeverity::High);
    assert!(AuditSeverity::High > AuditSeverity::Medium);
    assert!(AuditSeverity::Medium > AuditSeverity::Low);
    assert!(AuditSeverity::Low > AuditSeverity::Info);
}

proptest! {
    #[test]
    fn test_event_serialization_roundtrip(
        category in 0..11usize,
        severity in 0..5usize,
    ) {
        let categories = [
            AuditCategory::Authentication,
            AuditCategory::Authorization,
            AuditCategory::UserManagement,
            AuditCategory::Mission,
            AuditCategory::Forge,
            AuditCategory::Configuration,
            AuditCategory::FileSystem,
            AuditCategory::ApiCall,
            AuditCategory::System,
            AuditCategory::Security,
            AuditCategory::DataTransfer,
        ];
        let severities = [
            AuditSeverity::Info,
            AuditSeverity::Low,
            AuditSeverity::Medium,
            AuditSeverity::High,
            AuditSeverity::Critical,
        ];

        let event = AuditEvent::builder(categories[category], AuditAction::Custom("test".into()))
            .severity(severities[severity])
            .build();

        let json = serde_json::to_string(&event).unwrap();
        let parsed: AuditEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.category, parsed.category);
        assert_eq!(event.severity, parsed.severity);
    }
}
```

### 3. Persistence Tests (tests/persistence_test.rs)

```rust
//! Tests for audit persistence.

mod common;

use common::*;
use tachikoma_audit::*;
use chrono::Utc;

#[tokio::test]
async fn test_sqlite_persistence_single_event() {
    let db = test_db();
    let persistence = SqlitePersistence::from_connection(db);

    let event = test_event(AuditCategory::System, AuditAction::SystemStartup);
    persistence.persist(&event).await.unwrap();

    let count: i64 = persistence.conn.lock().query_row(
        "SELECT COUNT(*) FROM audit_events",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_sqlite_persistence_batch() {
    let db = test_db();
    let persistence = SqlitePersistence::from_connection(db);

    let events = test_events(100);
    let batch = EventBatch {
        events: events.into_iter().map(|e| CapturedEvent {
            event: e,
            captured_at: std::time::Instant::now(),
        }).collect(),
        collected_at: std::time::Instant::now(),
    };

    let count = persistence.persist_batch(&batch).await.unwrap();
    assert_eq!(count, 100);
}

#[tokio::test]
async fn test_append_log_integrity() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = AppendLogConfig {
        log_dir: dir.path().to_path_buf(),
        ..Default::default()
    };

    let persistence = AppendLogPersistence::new(config).unwrap();

    // Write events
    for i in 0..10 {
        let event = test_event(AuditCategory::System, AuditAction::Custom(format!("test_{}", i)));
        persistence.persist(&event).await.unwrap();
    }

    // Verify integrity
    let report = persistence.verify_integrity().await.unwrap();
    assert!(report.is_valid);
    assert_eq!(report.total_events, 10);
    assert!(report.corrupted_events.is_empty());
}

#[tokio::test]
async fn test_append_log_tamper_detection() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = AppendLogConfig {
        log_dir: dir.path().to_path_buf(),
        ..Default::default()
    };

    let persistence = AppendLogPersistence::new(config.clone()).unwrap();

    // Write events
    for i in 0..5 {
        let event = test_event(AuditCategory::System, AuditAction::Custom(format!("test_{}", i)));
        persistence.persist(&event).await.unwrap();
    }
    persistence.flush().await.unwrap();

    // Tamper with log file
    let log_files: Vec<_> = std::fs::read_dir(&config.log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "log"))
        .collect();

    if let Some(log_file) = log_files.first() {
        let content = std::fs::read_to_string(log_file.path()).unwrap();
        let tampered = content.replace("test_2", "tampered");
        std::fs::write(log_file.path(), tampered).unwrap();
    }

    // Verify should detect tampering
    let report = persistence.verify_integrity().await.unwrap();
    assert!(!report.is_valid);
}
```

### 4. Query Tests (tests/query_test.rs)

```rust
//! Tests for audit queries.

mod common;

use common::*;
use tachikoma_audit::*;
use chrono::{Duration, Utc};

#[test]
fn test_query_builder() {
    let query = AuditQuery::builder()
        .category(AuditCategory::Security)
        .min_severity(AuditSeverity::High)
        .success_only()
        .page_size(50)
        .build();

    assert!(query.categories.contains(&AuditCategory::Security));
    assert_eq!(query.min_severity, Some(AuditSeverity::High));
    assert_eq!(query.success_only, Some(true));
    assert_eq!(query.page_size, 50);
}

#[test]
fn test_query_time_range() {
    let now = Utc::now();
    let yesterday = now - Duration::days(1);

    let query = AuditQuery::builder()
        .time_range(TimeRange::between(yesterday, now))
        .build();

    let time_range = query.time_range.unwrap();
    assert_eq!(time_range.start, Some(yesterday));
    assert_eq!(time_range.end, Some(now));
}

#[tokio::test]
async fn test_query_execution() {
    let db = test_db();

    // Insert test data
    {
        let conn = db.lock();
        for i in 0..20 {
            let category = if i % 2 == 0 { "security" } else { "system" };
            conn.execute(
                "INSERT INTO audit_events (id, timestamp, category, action, severity, actor_type, outcome, checksum)
                 VALUES (?, datetime('now'), ?, 'test', 'info', 'system', 'success', 'test')",
                rusqlite::params![format!("evt_{}", i), category],
            ).unwrap();
        }
    }

    let executor = QueryExecutor::new(db);

    let query = AuditQuery::builder()
        .category(AuditCategory::Security)
        .page_size(10)
        .with_count()
        .build();

    let result = executor.execute(&query).unwrap();

    assert!(result.total_count.is_some());
    assert!(result.items.len() <= 10);
}
```

### 5. Integration Tests (tests/integration_test.rs)

```rust
//! Integration tests for the full audit system.

mod common;

use common::*;
use tachikoma_audit::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_full_audit_pipeline() {
    let db = test_db();

    // Setup capture
    let (capture, receiver) = AuditCapture::new(CaptureConfig::default());

    // Setup batch processing
    let (batch_tx, mut batch_rx) = tokio::sync::mpsc::channel(100);
    let batch_config = BatchConfig {
        max_batch_size: 10,
        max_batch_age: Duration::from_millis(100),
    };

    tokio::spawn(async move {
        batch_processing_loop(receiver, batch_tx, batch_config).await;
    });

    // Setup persistence
    let persistence = Arc::new(SqlitePersistence::from_connection(db.clone()));

    // Record events
    for i in 0..25 {
        let event = test_event(
            AuditCategory::System,
            AuditAction::Custom(format!("test_{}", i)),
        );
        capture.record(event);
    }

    // Wait for batching
    sleep(Duration::from_millis(200)).await;

    // Process batches
    let mut total_persisted = 0;
    while let Ok(batch) = batch_rx.try_recv() {
        let count = persistence.persist_batch(&batch).await.unwrap();
        total_persisted += count;
    }

    // Verify
    let count: i64 = db.lock().query_row(
        "SELECT COUNT(*) FROM audit_events",
        [],
        |row| row.get(0),
    ).unwrap();

    assert!(count >= 20); // Some events should be persisted
}

#[tokio::test]
async fn test_alert_engine_integration() {
    let (engine, mut alert_rx) = AlertEngine::new(AlertEngineConfig::default());

    // Add a rule
    engine.add_rule(AlertRule {
        id: "test_rule".to_string(),
        name: "Test Alert".to_string(),
        description: None,
        enabled: true,
        conditions: AlertConditions {
            categories: vec![AuditCategory::Security],
            actions: vec![],
            min_severity: Some(AuditSeverity::High),
            failures_only: false,
            actor_pattern: None,
            target_pattern: None,
            field_matches: std::collections::HashMap::new(),
            threshold: None,
        },
        severity: AlertSeverity::Critical,
        channels: vec![],
        throttle: None,
        tags: vec![],
    });

    // Process a matching event
    let event = AuditEvent::builder(AuditCategory::Security, AuditAction::SecurityViolation)
        .severity(AuditSeverity::Critical)
        .build();

    engine.process_event(&event).await;

    // Should receive an alert
    let alert = tokio::time::timeout(Duration::from_millis(100), alert_rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(alert.rule_id, "test_rule");
}
```

### 6. Benchmark Tests (benches/audit_bench.rs)

```rust
//! Performance benchmarks for audit system.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tachikoma_audit::*;

fn event_creation_benchmark(c: &mut Criterion) {
    c.bench_function("create_event_simple", |b| {
        b.iter(|| {
            AuditEvent::builder(AuditCategory::System, AuditAction::SystemStartup)
                .actor(AuditActor::system("bench"))
                .build()
        });
    });

    c.bench_function("create_event_with_metadata", |b| {
        b.iter(|| {
            AuditEvent::builder(AuditCategory::System, AuditAction::SystemStartup)
                .actor(AuditActor::system("bench"))
                .metadata("key1", "value1")
                .metadata("key2", 42)
                .metadata("key3", true)
                .build()
        });
    });
}

fn serialization_benchmark(c: &mut Criterion) {
    let event = AuditEvent::builder(AuditCategory::Security, AuditAction::Login)
        .actor(AuditActor::user(UserId::new()))
        .target(AuditTarget::new("user", "user_123"))
        .metadata("ip", "192.168.1.1")
        .build();

    c.bench_function("serialize_event", |b| {
        b.iter(|| serde_json::to_string(&event).unwrap());
    });

    let json = serde_json::to_string(&event).unwrap();
    c.bench_function("deserialize_event", |b| {
        b.iter(|| serde_json::from_str::<AuditEvent>(&json).unwrap());
    });
}

fn hash_chain_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_chain");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("append", size), size, |b, &size| {
            let mut chain = HashChain::new(b"genesis");
            b.iter(|| {
                for i in 0..size {
                    chain.append(format!("event_{}", i).as_bytes());
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("verify", size), size, |b, &size| {
            let mut chain = HashChain::new(b"genesis");
            for i in 0..size {
                chain.append(format!("event_{}", i).as_bytes());
            }
            b.iter(|| chain.verify_full().unwrap());
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    event_creation_benchmark,
    serialization_benchmark,
    hash_chain_benchmark,
);
criterion_main!(benches);
```

---

## Testing Requirements

1. All public APIs have unit tests
2. Integration tests cover full workflows
3. Property tests verify serialization
4. Benchmarks track performance
5. Security tests verify access control

---

## Related Specs

- Depends on: All audit specs (431-449)
- Completes: Phase 20 - Audit System
