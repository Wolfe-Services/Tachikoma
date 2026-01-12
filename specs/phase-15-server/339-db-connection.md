# 339 - Database Connection

**Phase:** 15 - Server
**Spec ID:** 339
**Status:** Planned
**Dependencies:** 332-server-config
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement database connection management with connection pooling, health monitoring, and query instrumentation.

---

## Acceptance Criteria

- [ ] Connection pool configuration
- [ ] Connection health monitoring
- [ ] Query timeout handling
- [ ] Transaction helpers
- [ ] Query instrumentation
- [ ] Migration support
- [ ] Connection retry logic

---

## Implementation Details

### 1. Database Config (crates/tachikoma-server/src/db/config.rs)

```rust
//! Database configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// Database connection URL.
    pub url: String,
    /// Maximum connections in pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Minimum connections in pool.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// Connection acquire timeout.
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout_secs: u64,
    /// Connection idle timeout.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    /// Maximum connection lifetime.
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_secs: u64,
    /// Statement cache size.
    #[serde(default = "default_statement_cache")]
    pub statement_cache_size: usize,
    /// Enable query logging.
    #[serde(default)]
    pub log_queries: bool,
    /// Slow query threshold (ms).
    #[serde(default = "default_slow_query")]
    pub slow_query_threshold_ms: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_acquire_timeout() -> u64 {
    10
}

fn default_idle_timeout() -> u64 {
    600
}

fn default_max_lifetime() -> u64 {
    3600
}

fn default_statement_cache() -> usize {
    100
}

fn default_slow_query() -> u64 {
    1000
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            acquire_timeout_secs: default_acquire_timeout(),
            idle_timeout_secs: default_idle_timeout(),
            max_lifetime_secs: default_max_lifetime(),
            statement_cache_size: default_statement_cache(),
            log_queries: false,
            slow_query_threshold_ms: default_slow_query(),
        }
    }
}

impl DbConfig {
    pub fn acquire_timeout(&self) -> Duration {
        Duration::from_secs(self.acquire_timeout_secs)
    }

    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }

    pub fn slow_query_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_query_threshold_ms)
    }
}
```

### 2. Pool Builder (crates/tachikoma-server/src/db/pool.rs)

```rust
//! Database pool management.

use super::config::DbConfig;
use anyhow::{Context, Result};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::str::FromStr;
use tracing::{info, warn};

/// Create a database connection pool.
pub async fn create_pool(config: &DbConfig) -> Result<PgPool> {
    info!("Creating database connection pool...");

    let connect_options = PgConnectOptions::from_str(&config.url)
        .context("Invalid database URL")?
        .statement_cache_capacity(config.statement_cache_size);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout())
        .idle_timeout(Some(config.idle_timeout()))
        .max_lifetime(Some(config.max_lifetime()))
        .connect_with(connect_options)
        .await
        .context("Failed to create database pool")?;

    // Verify connection
    verify_connection(&pool).await?;

    info!(
        max_connections = config.max_connections,
        min_connections = config.min_connections,
        "Database pool created"
    );

    Ok(pool)
}

/// Verify database connection.
pub async fn verify_connection(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .context("Database connection verification failed")?;

    info!("Database connection verified");
    Ok(())
}

/// Get pool statistics.
pub fn pool_stats(pool: &PgPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        idle: pool.num_idle(),
        active: pool.size() - pool.num_idle() as u32,
    }
}

/// Pool statistics.
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub active: u32,
}
```

### 3. Query Instrumentation (crates/tachikoma-server/src/db/instrumentation.rs)

```rust
//! Query instrumentation for logging and metrics.

use std::time::{Duration, Instant};
use tracing::{debug, info, warn, Span};

/// Query execution timer.
pub struct QueryTimer {
    query: String,
    start: Instant,
    slow_threshold: Duration,
}

impl QueryTimer {
    pub fn new(query: impl Into<String>, slow_threshold: Duration) -> Self {
        Self {
            query: query.into(),
            start: Instant::now(),
            slow_threshold,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn finish(self) -> Duration {
        let elapsed = self.elapsed();
        let elapsed_ms = elapsed.as_millis();

        if elapsed > self.slow_threshold {
            warn!(
                query = %self.query,
                elapsed_ms = elapsed_ms,
                "Slow query detected"
            );
        } else {
            debug!(
                query = %self.query,
                elapsed_ms = elapsed_ms,
                "Query completed"
            );
        }

        elapsed
    }
}

/// Macro for timing queries.
#[macro_export]
macro_rules! timed_query {
    ($pool:expr, $query:expr, $slow_threshold:expr) => {{
        let timer = $crate::db::instrumentation::QueryTimer::new(
            stringify!($query),
            $slow_threshold,
        );
        let result = $query;
        timer.finish();
        result
    }};
}

/// Query logger for SQLx.
pub struct QueryLogger {
    slow_threshold: Duration,
}

impl QueryLogger {
    pub fn new(slow_threshold: Duration) -> Self {
        Self { slow_threshold }
    }
}

// Note: Would implement sqlx::QueryLogger trait here
```

### 4. Transaction Helpers (crates/tachikoma-server/src/db/transaction.rs)

```rust
//! Transaction helper utilities.

use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;

/// Execute a function within a transaction.
pub async fn with_transaction<F, Fut, T>(pool: &PgPool, f: F) -> Result<T>
where
    F: FnOnce(Transaction<'_, Postgres>) -> Fut,
    Fut: Future<Output = Result<(Transaction<'_, Postgres>, T)>>,
{
    let tx = pool.begin().await?;
    let (tx, result) = f(tx).await?;
    tx.commit().await?;
    Ok(result)
}

/// Execute with automatic retry on serialization failures.
pub async fn with_retry<F, Fut, T>(pool: &PgPool, max_retries: u32, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;

                // Check if it's a serialization failure
                let is_serialization = e
                    .to_string()
                    .contains("could not serialize access");

                if is_serialization && attempts < max_retries {
                    tracing::warn!(
                        attempt = attempts,
                        max_retries = max_retries,
                        "Serialization failure, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64))
                        .await;
                    continue;
                }

                return Err(e);
            }
        }
    }
}

/// Transaction isolation levels.
#[derive(Debug, Clone, Copy)]
pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl IsolationLevel {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

/// Begin a transaction with specific isolation level.
pub async fn begin_with_isolation(
    pool: &PgPool,
    level: IsolationLevel,
) -> Result<Transaction<'_, Postgres>> {
    let tx = pool.begin().await?;
    sqlx::query(&format!("SET TRANSACTION ISOLATION LEVEL {}", level.to_sql()))
        .execute(&mut *tx.as_ref())
        .await?;
    Ok(tx)
}
```

### 5. Migration Support (crates/tachikoma-server/src/db/migration.rs)

```rust
//! Database migration utilities.

use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

/// Run database migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run migrations")?;

    info!("Migrations completed");
    Ok(())
}

/// Check pending migrations.
pub async fn check_migrations(pool: &PgPool) -> Result<Vec<String>> {
    let migrator = sqlx::migrate!("./migrations");
    let applied = migrator
        .get_applied_migrations(pool)
        .await
        .context("Failed to check applied migrations")?;

    let pending: Vec<String> = migrator
        .migrations
        .iter()
        .filter(|m| !applied.iter().any(|a| a.version == m.version))
        .map(|m| format!("{}_{}", m.version, m.description))
        .collect();

    Ok(pending)
}

/// Revert last migration (for development).
#[cfg(debug_assertions)]
pub async fn revert_last(pool: &PgPool) -> Result<()> {
    use sqlx::Row;

    // Get last applied migration
    let last: Option<i64> = sqlx::query("SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1")
        .fetch_optional(pool)
        .await?
        .map(|row| row.get("version"));

    if let Some(version) = last {
        info!(version = version, "Reverting migration");
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
            .bind(version)
            .execute(pool)
            .await?;
        info!("Migration reverted (schema changes not undone)");
    }

    Ok(())
}
```

### 6. Connection Health (crates/tachikoma-server/src/db/health.rs)

```rust
//! Database health monitoring.

use super::pool::pool_stats;
use sqlx::PgPool;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Database health status.
#[derive(Debug, Clone)]
pub struct DbHealth {
    pub is_healthy: bool,
    pub latency_ms: u64,
    pub pool_size: u32,
    pub pool_idle: usize,
    pub pool_active: u32,
    pub message: Option<String>,
}

/// Check database health.
pub async fn check_health(pool: &PgPool, timeout: Duration) -> DbHealth {
    let start = Instant::now();

    let result = tokio::time::timeout(
        timeout,
        sqlx::query("SELECT 1").fetch_one(pool),
    )
    .await;

    let latency = start.elapsed();
    let stats = pool_stats(pool);

    match result {
        Ok(Ok(_)) => {
            debug!(latency_ms = latency.as_millis(), "Database health check passed");
            DbHealth {
                is_healthy: true,
                latency_ms: latency.as_millis() as u64,
                pool_size: stats.size,
                pool_idle: stats.idle,
                pool_active: stats.active,
                message: None,
            }
        }
        Ok(Err(e)) => {
            warn!(error = %e, "Database health check failed");
            DbHealth {
                is_healthy: false,
                latency_ms: latency.as_millis() as u64,
                pool_size: stats.size,
                pool_idle: stats.idle,
                pool_active: stats.active,
                message: Some(e.to_string()),
            }
        }
        Err(_) => {
            warn!("Database health check timed out");
            DbHealth {
                is_healthy: false,
                latency_ms: latency.as_millis() as u64,
                pool_size: stats.size,
                pool_idle: stats.idle,
                pool_active: stats.active,
                message: Some("Connection timeout".to_string()),
            }
        }
    }
}
```

---

## Testing Requirements

1. Pool configuration works
2. Connection health checks work
3. Transaction helpers work
4. Retry logic works
5. Migration runs successfully
6. Query logging works
7. Slow query detection works

---

## Related Specs

- Depends on: [332-server-config.md](332-server-config.md)
- Next: [340-server-tests.md](340-server-tests.md)
- Used by: All database operations
