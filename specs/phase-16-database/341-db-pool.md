# Spec 341: Database Connection Pool

## Overview
Implement a connection pool for SQLite using sqlx with configurable pool settings, health checks, and connection lifecycle management.

## Rust Implementation

### Dependencies (Cargo.toml)
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1.0", features = ["full"] }
thiserror = "1.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
```

### Pool Configuration
```rust
// src/database/pool.rs

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn, error, instrument};

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("Failed to create connection pool: {0}")]
    Creation(#[from] sqlx::Error),

    #[error("Pool health check failed: {0}")]
    HealthCheck(String),

    #[error("Database file not found: {0}")]
    DatabaseNotFound(String),

    #[error("Invalid pool configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Path to SQLite database file
    pub database_path: String,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Connection acquire timeout
    pub acquire_timeout: Duration,
    /// Idle connection timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
    /// Enable WAL mode for better concurrency
    pub wal_mode: bool,
    /// Synchronous mode setting
    pub synchronous: SynchronousMode,
    /// Busy timeout for locked database
    pub busy_timeout: Duration,
    /// Create database if not exists
    pub create_if_missing: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum SynchronousMode {
    Off,
    Normal,
    #[default]
    Full,
    Extra,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            database_path: "tachikoma.db".to_string(),
            min_connections: 1,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            wal_mode: true,
            synchronous: SynchronousMode::Full,
            busy_timeout: Duration::from_secs(5),
            create_if_missing: true,
        }
    }
}

impl PoolConfig {
    pub fn builder() -> PoolConfigBuilder {
        PoolConfigBuilder::default()
    }

    pub fn in_memory() -> Self {
        Self {
            database_path: ":memory:".to_string(),
            min_connections: 1,
            max_connections: 1,
            wal_mode: false,
            ..Default::default()
        }
    }

    pub fn validate(&self) -> Result<(), PoolError> {
        if self.min_connections > self.max_connections {
            return Err(PoolError::InvalidConfig(
                "min_connections cannot exceed max_connections".to_string()
            ));
        }

        if self.max_connections == 0 {
            return Err(PoolError::InvalidConfig(
                "max_connections must be at least 1".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct PoolConfigBuilder {
    config: PoolConfig,
}

impl PoolConfigBuilder {
    pub fn database_path(mut self, path: impl Into<String>) -> Self {
        self.config.database_path = path.into();
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.config.min_connections = min;
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }

    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.config.acquire_timeout = timeout;
        self
    }

    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.idle_timeout = timeout;
        self
    }

    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.config.max_lifetime = lifetime;
        self
    }

    pub fn wal_mode(mut self, enabled: bool) -> Self {
        self.config.wal_mode = enabled;
        self
    }

    pub fn synchronous(mut self, mode: SynchronousMode) -> Self {
        self.config.synchronous = mode;
        self
    }

    pub fn busy_timeout(mut self, timeout: Duration) -> Self {
        self.config.busy_timeout = timeout;
        self
    }

    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.config.create_if_missing = create;
        self
    }

    pub fn build(self) -> Result<PoolConfig, PoolError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Database connection pool wrapper
pub struct DatabasePool {
    pool: SqlitePool,
    config: PoolConfig,
}

impl DatabasePool {
    #[instrument(skip(config), fields(path = %config.database_path))]
    pub async fn new(config: PoolConfig) -> Result<Self, PoolError> {
        config.validate()?;

        let connect_options = Self::build_connect_options(&config)?;

        let pool = SqlitePoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .max_lifetime(Some(config.max_lifetime))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Enable foreign keys
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(connect_options)
            .await?;

        info!("Database pool created with {} max connections", config.max_connections);

        let db_pool = Self { pool, config };
        db_pool.health_check().await?;

        Ok(db_pool)
    }

    fn build_connect_options(config: &PoolConfig) -> Result<SqliteConnectOptions, PoolError> {
        let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.database_path))
            .map_err(|e| PoolError::InvalidConfig(e.to_string()))?
            .create_if_missing(config.create_if_missing)
            .busy_timeout(config.busy_timeout);

        if config.wal_mode {
            options = options.journal_mode(SqliteJournalMode::Wal);
        }

        options = match config.synchronous {
            SynchronousMode::Off => options.synchronous(SqliteSynchronous::Off),
            SynchronousMode::Normal => options.synchronous(SqliteSynchronous::Normal),
            SynchronousMode::Full => options.synchronous(SqliteSynchronous::Full),
            SynchronousMode::Extra => options.synchronous(SqliteSynchronous::Extra),
        };

        Ok(options)
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Perform health check
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<(), PoolError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| PoolError::HealthCheck(e.to_string()))?;

        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max_connections: self.config.max_connections,
        }
    }

    /// Close the pool gracefully
    #[instrument(skip(self))]
    pub async fn close(&self) {
        info!("Closing database pool");
        self.pool.close().await;
    }

    /// Check if pool is closed
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub max_connections: u32,
}

impl PoolStats {
    pub fn utilization(&self) -> f64 {
        if self.max_connections == 0 {
            return 0.0;
        }
        (self.size as f64 - self.idle as f64) / self.max_connections as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_in_memory_pool() {
        let config = PoolConfig::in_memory();
        let pool = DatabasePool::new(config).await.unwrap();

        assert!(!pool.is_closed());
        pool.health_check().await.unwrap();
        pool.close().await;
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = PoolConfig::in_memory();
        let pool = DatabasePool::new(config).await.unwrap();

        let stats = pool.stats();
        assert!(stats.size >= 1);
        assert!(stats.utilization() <= 1.0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_config_validation() {
        let result = PoolConfig::builder()
            .min_connections(10)
            .max_connections(5)
            .build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pool_builder() {
        let config = PoolConfig::builder()
            .database_path(":memory:")
            .max_connections(5)
            .wal_mode(false)
            .build()
            .unwrap();

        assert_eq!(config.max_connections, 5);
        assert!(!config.wal_mode);
    }
}
```

### Pool Manager
```rust
// src/database/manager.rs

use super::pool::{DatabasePool, PoolConfig, PoolError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Global database pool manager
pub struct PoolManager {
    pools: RwLock<Vec<Arc<DatabasePool>>>,
    primary: RwLock<Option<Arc<DatabasePool>>>,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(Vec::new()),
            primary: RwLock::new(None),
        }
    }

    /// Initialize primary database pool
    pub async fn init_primary(&self, config: PoolConfig) -> Result<Arc<DatabasePool>, PoolError> {
        let pool = Arc::new(DatabasePool::new(config).await?);

        let mut pools = self.pools.write().await;
        pools.push(pool.clone());

        let mut primary = self.primary.write().await;
        *primary = Some(pool.clone());

        info!("Primary database pool initialized");
        Ok(pool)
    }

    /// Get primary pool
    pub async fn primary(&self) -> Option<Arc<DatabasePool>> {
        self.primary.read().await.clone()
    }

    /// Add additional pool
    pub async fn add_pool(&self, config: PoolConfig) -> Result<Arc<DatabasePool>, PoolError> {
        let pool = Arc::new(DatabasePool::new(config).await?);

        let mut pools = self.pools.write().await;
        pools.push(pool.clone());

        Ok(pool)
    }

    /// Health check all pools
    pub async fn health_check_all(&self) -> Vec<Result<(), PoolError>> {
        let pools = self.pools.read().await;
        let mut results = Vec::new();

        for pool in pools.iter() {
            results.push(pool.health_check().await);
        }

        results
    }

    /// Close all pools
    pub async fn close_all(&self) {
        let pools = self.pools.read().await;
        for pool in pools.iter() {
            pool.close().await;
        }
        info!("All database pools closed");
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    pub static ref POOL_MANAGER: PoolManager = PoolManager::new();
}
```

## Connection Lifecycle

1. **Acquisition**: Connection acquired from pool with timeout
2. **Validation**: Health check on acquire if configured
3. **Usage**: Connection used for queries
4. **Return**: Connection returned to pool
5. **Cleanup**: Idle connections closed after timeout

## Testing Requirements

- Unit tests for pool creation
- Integration tests for connection lifecycle
- Load tests for pool under pressure
- Failure tests for connection errors

## Files to Create
- `src/database/pool.rs` - Pool implementation
- `src/database/manager.rs` - Pool manager
- `src/database/mod.rs` - Module exports
