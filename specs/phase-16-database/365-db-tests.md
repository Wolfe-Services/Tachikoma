# Spec 365: Database Tests

## Overview
Implement comprehensive testing infrastructure for database operations including unit tests, integration tests, and performance benchmarks.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Test Utilities
```rust
// src/database/testing.rs

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::{TempDir, NamedTempFile};
use tokio::sync::OnceCell;

static TEST_DB_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Test database configuration
pub struct TestDb {
    pool: SqlitePool,
    _temp_dir: Option<TempDir>,
    _temp_file: Option<NamedTempFile>,
}

impl TestDb {
    /// Create an in-memory test database
    pub async fn memory() -> Self {
        let id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:file:test_{}?mode=memory&cache=shared", id))
            .await
            .expect("Failed to create test database");

        Self {
            pool,
            _temp_dir: None,
            _temp_file: None,
        }
    }

    /// Create a file-based test database
    pub async fn file() -> Self {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let path = temp_file.path().to_string_lossy().to_string();

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", path))
            .await
            .expect("Failed to create test database");

        Self {
            pool,
            _temp_dir: None,
            _temp_file: Some(temp_file),
        }
    }

    /// Get pool reference
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run migrations
    pub async fn migrate(&self) -> &Self {
        // Run all migrations
        crate::database::migration::run_all_migrations(self.pool()).await
            .expect("Failed to run migrations");
        self
    }

    /// Seed with test data
    pub async fn seed(&self) -> &Self {
        seed_test_data(self.pool()).await
            .expect("Failed to seed test data");
        self
    }

    /// Reset database (drop all tables)
    pub async fn reset(&self) {
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();

        for (table,) in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
                .execute(&self.pool)
                .await
                .unwrap();
        }
    }

    /// Close database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

/// Seed test data
pub async fn seed_test_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Insert test missions
    sqlx::query(r#"
        INSERT INTO missions (id, title, description, status, priority, created_at, updated_at)
        VALUES
            ('mission-1', 'Test Mission 1', 'Description 1', 'active', 'high', datetime('now'), datetime('now')),
            ('mission-2', 'Test Mission 2', 'Description 2', 'draft', 'medium', datetime('now'), datetime('now')),
            ('mission-3', 'Test Mission 3', 'Description 3', 'completed', 'low', datetime('now'), datetime('now'))
    "#)
    .execute(pool)
    .await?;

    // Insert test specs
    sqlx::query(r#"
        INSERT INTO specs (id, mission_id, title, description, status, complexity, version, created_at, updated_at)
        VALUES
            ('spec-1', 'mission-1', 'Test Spec 1', 'Spec description 1', 'draft', 'medium', 1, datetime('now'), datetime('now')),
            ('spec-2', 'mission-1', 'Test Spec 2', 'Spec description 2', 'approved', 'simple', 1, datetime('now'), datetime('now'))
    "#)
    .execute(pool)
    .await?;

    Ok(())
}

/// Test fixture for repository tests
pub struct RepositoryFixture<R> {
    pub db: TestDb,
    pub repo: R,
}

impl<R> RepositoryFixture<R> {
    pub async fn new<F>(create_repo: F) -> Self
    where
        F: FnOnce(SqlitePool) -> R,
    {
        let db = TestDb::memory().await;
        db.migrate().await;
        let repo = create_repo(db.pool().clone());
        Self { db, repo }
    }

    pub async fn with_seed<F>(create_repo: F) -> Self
    where
        F: FnOnce(SqlitePool) -> R,
    {
        let db = TestDb::memory().await;
        db.migrate().await;
        db.seed().await;
        let repo = create_repo(db.pool().clone());
        Self { db, repo }
    }
}
```

### Repository Tests
```rust
// src/database/repository/mission_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::testing::*;
    use crate::database::repository::mission::*;
    use crate::database::schema::mission::*;

    async fn setup() -> RepositoryFixture<MissionRepository> {
        RepositoryFixture::new(MissionRepository::new).await
    }

    #[tokio::test]
    async fn test_create_mission() {
        let fixture = setup().await;

        let input = CreateMission {
            title: "New Mission".to_string(),
            description: Some("Test description".to_string()),
            status: None,
            priority: Some(MissionPriority::High),
            parent_id: None,
            owner_id: None,
            due_date: None,
            tags: Some(vec!["test".to_string()]),
            metadata: None,
        };

        let mission = fixture.repo.create(input).await.unwrap();

        assert_eq!(mission.title, "New Mission");
        assert_eq!(mission.priority, MissionPriority::High);
        assert_eq!(mission.status, MissionStatus::Draft);
        assert!(mission.id.len() > 0);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        let found = fixture.repo.find_by_id("mission-1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Mission 1");

        let not_found = fixture.repo.find_by_id("nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_update_mission() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        let update = UpdateMission {
            title: Some("Updated Title".to_string()),
            status: Some(MissionStatus::Active),
            ..Default::default()
        };

        let updated = fixture.repo.update("mission-2", update).await.unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.status, MissionStatus::Active);
        assert!(updated.started_at.is_some());
    }

    #[tokio::test]
    async fn test_delete_mission() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        let deleted = fixture.repo.delete("mission-1").await.unwrap();
        assert!(deleted);

        let not_found = fixture.repo.find_by_id("mission-1").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_find_with_filters() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        // Filter by status
        let filter = MissionFilter {
            status: Some(vec![MissionStatus::Active]),
            ..Default::default()
        };

        let missions = fixture.repo.find_many(filter, Pagination::default(), None).await.unwrap();
        assert_eq!(missions.len(), 1);
        assert_eq!(missions[0].status, MissionStatus::Active);
    }

    #[tokio::test]
    async fn test_pagination() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        let pagination = Pagination { limit: 2, offset: 0 };
        let page1 = fixture.repo.find_many(
            MissionFilter::default(),
            pagination,
            None
        ).await.unwrap();
        assert_eq!(page1.len(), 2);

        let pagination = Pagination { limit: 2, offset: 2 };
        let page2 = fixture.repo.find_many(
            MissionFilter::default(),
            pagination,
            None
        ).await.unwrap();
        assert_eq!(page2.len(), 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let fixture = RepositoryFixture::with_seed(MissionRepository::new).await;

        let stats = fixture.repo.stats().await.unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.active, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.draft, 1);
    }
}
```

### Integration Tests
```rust
// tests/database_integration.rs

use tachikoma::database::*;

#[tokio::test]
async fn test_full_workflow() {
    let db = testing::TestDb::file().await;
    db.migrate().await;

    let mission_repo = repository::MissionRepository::new(db.pool().clone());
    let spec_repo = repository::SpecRepository::new(db.pool().clone());

    // Create mission
    let mission = mission_repo.create(repository::CreateMission {
        title: "Integration Test Mission".to_string(),
        description: Some("Testing full workflow".to_string()),
        ..Default::default()
    }).await.unwrap();

    // Create spec for mission
    let spec = spec_repo.create(repository::CreateSpec {
        mission_id: mission.id.clone(),
        title: "Test Spec".to_string(),
        description: Some("Spec for testing".to_string()),
        ..Default::default()
    }).await.unwrap();

    // Verify relationships
    let specs = spec_repo.find_by_mission(&mission.id).await.unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].id, spec.id);

    // Update spec status
    spec_repo.transition_status(&spec.id, schema::SpecStatus::Review, None).await.unwrap();

    // Verify status
    let updated_spec = spec_repo.find_by_id(&spec.id).await.unwrap().unwrap();
    assert_eq!(updated_spec.status, schema::SpecStatus::Review);

    db.close().await;
}

#[tokio::test]
async fn test_transaction_rollback() {
    let db = testing::TestDb::memory().await;
    db.migrate().await;

    let pool = db.pool().clone();

    // Start transaction that will fail
    let result: Result<(), sqlx::Error> = async {
        let mut tx = pool.begin().await?;

        // Insert valid data
        sqlx::query("INSERT INTO missions (id, title, status, priority, created_at, updated_at) VALUES ('tx-test', 'TX Test', 'draft', 'medium', datetime('now'), datetime('now'))")
            .execute(&mut *tx)
            .await?;

        // This should fail (duplicate key)
        sqlx::query("INSERT INTO missions (id, title, status, priority, created_at, updated_at) VALUES ('tx-test', 'Duplicate', 'draft', 'medium', datetime('now'), datetime('now'))")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }.await;

    assert!(result.is_err());

    // Verify rollback - no data should exist
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM missions WHERE id = 'tx-test'")
        .fetch_one(db.pool())
        .await
        .unwrap();

    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn test_concurrent_access() {
    let db = testing::TestDb::file().await;
    db.migrate().await;

    let pool = db.pool().clone();

    // Spawn multiple tasks
    let mut handles = vec![];

    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let repo = repository::MissionRepository::new(pool_clone);
            repo.create(repository::CreateMission {
                title: format!("Concurrent Mission {}", i),
                ..Default::default()
            }).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all created
    let repo = repository::MissionRepository::new(pool);
    let missions = repo.find_many(
        repository::MissionFilter::default(),
        repository::Pagination { limit: 100, offset: 0 },
        None
    ).await.unwrap();

    assert_eq!(missions.len(), 10);
}
```

### Performance Benchmarks
```rust
// benches/database_bench.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tachikoma::database::*;
use tokio::runtime::Runtime;

fn create_mission_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let db = rt.block_on(async {
        let db = testing::TestDb::memory().await;
        db.migrate().await;
        db
    });

    let pool = db.pool().clone();

    c.bench_function("create_mission", |b| {
        b.iter(|| {
            rt.block_on(async {
                let repo = repository::MissionRepository::new(pool.clone());
                repo.create(repository::CreateMission {
                    title: "Benchmark Mission".to_string(),
                    ..Default::default()
                }).await.unwrap()
            })
        })
    });
}

fn query_missions_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let db = rt.block_on(async {
        let db = testing::TestDb::memory().await;
        db.migrate().await;

        // Create many missions
        let repo = repository::MissionRepository::new(db.pool().clone());
        for i in 0..1000 {
            repo.create(repository::CreateMission {
                title: format!("Mission {}", i),
                ..Default::default()
            }).await.unwrap();
        }

        db
    });

    let pool = db.pool().clone();

    let mut group = c.benchmark_group("query_missions");

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let repo = repository::MissionRepository::new(pool.clone());
                    repo.find_many(
                        repository::MissionFilter::default(),
                        repository::Pagination { limit: size, offset: 0 },
                        None
                    ).await.unwrap()
                })
            })
        });
    }

    group.finish();
}

criterion_group!(benches, create_mission_benchmark, query_missions_benchmark);
criterion_main!(benches);
```

## Testing Strategy

1. **Unit Tests**: Test individual repository methods
2. **Integration Tests**: Test cross-repository workflows
3. **Transaction Tests**: Verify rollback behavior
4. **Concurrency Tests**: Test parallel access
5. **Performance Benchmarks**: Measure operation speed

## Files to Create
- `src/database/testing.rs` - Test utilities
- `src/database/repository/mission_tests.rs` - Mission tests
- `src/database/repository/spec_tests.rs` - Spec tests
- `tests/database_integration.rs` - Integration tests
- `benches/database_bench.rs` - Benchmarks
