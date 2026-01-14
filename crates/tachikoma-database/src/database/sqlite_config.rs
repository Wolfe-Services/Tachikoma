use sqlx::Executor;
use thiserror::Error;
use tracing::{info, debug};

#[derive(Debug, Error)]
pub enum SqliteConfigError {
    #[error("Failed to execute PRAGMA: {0}")]
    PragmaError(#[from] sqlx::Error),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone)]
pub struct SqliteConfig {
    /// Journal mode (WAL recommended)
    pub journal_mode: JournalMode,
    /// Synchronous mode
    pub synchronous: Synchronous,
    /// Cache size in pages (negative for KB)
    pub cache_size: i64,
    /// Page size in bytes
    pub page_size: u32,
    /// Memory-mapped I/O size
    pub mmap_size: u64,
    /// Temp store location
    pub temp_store: TempStore,
    /// Auto vacuum mode
    pub auto_vacuum: AutoVacuum,
    /// Foreign keys enforcement
    pub foreign_keys: bool,
    /// Busy timeout in milliseconds
    pub busy_timeout: u32,
    /// WAL autocheckpoint threshold
    pub wal_autocheckpoint: u32,
    /// Secure delete mode
    pub secure_delete: bool,
    /// Recursive triggers
    pub recursive_triggers: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum JournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    #[default]
    Wal,
    Off,
}

impl JournalMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delete => "DELETE",
            Self::Truncate => "TRUNCATE",
            Self::Persist => "PERSIST",
            Self::Memory => "MEMORY",
            Self::Wal => "WAL",
            Self::Off => "OFF",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Synchronous {
    Off,
    Normal,
    #[default]
    Full,
    Extra,
}

impl Synchronous {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Normal => "NORMAL",
            Self::Full => "FULL",
            Self::Extra => "EXTRA",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TempStore {
    #[default]
    Default,
    File,
    Memory,
}

impl TempStore {
    pub fn as_i32(&self) -> i32 {
        match self {
            Self::Default => 0,
            Self::File => 1,
            Self::Memory => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AutoVacuum {
    #[default]
    None,
    Full,
    Incremental,
}

impl AutoVacuum {
    pub fn as_i32(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::Full => 1,
            Self::Incremental => 2,
        }
    }
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            journal_mode: JournalMode::Wal,
            synchronous: Synchronous::Normal,
            cache_size: -64000, // 64MB
            page_size: 4096,
            mmap_size: 256 * 1024 * 1024, // 256MB
            temp_store: TempStore::Memory,
            auto_vacuum: AutoVacuum::Incremental,
            foreign_keys: true,
            busy_timeout: 5000,
            wal_autocheckpoint: 1000,
            secure_delete: false,
            recursive_triggers: true,
        }
    }
}

impl SqliteConfig {
    /// Configuration optimized for read-heavy workloads
    pub fn read_optimized() -> Self {
        Self {
            cache_size: -128000, // 128MB cache
            mmap_size: 512 * 1024 * 1024, // 512MB mmap
            synchronous: Synchronous::Normal,
            ..Default::default()
        }
    }

    /// Configuration optimized for write-heavy workloads
    pub fn write_optimized() -> Self {
        Self {
            synchronous: Synchronous::Normal,
            wal_autocheckpoint: 10000,
            ..Default::default()
        }
    }

    /// Configuration for maximum durability
    pub fn durable() -> Self {
        Self {
            synchronous: Synchronous::Full,
            secure_delete: true,
            ..Default::default()
        }
    }

    /// Configuration for testing (fast, less durable)
    pub fn testing() -> Self {
        Self {
            journal_mode: JournalMode::Memory,
            synchronous: Synchronous::Off,
            temp_store: TempStore::Memory,
            ..Default::default()
        }
    }

    /// Apply configuration to connection
    pub async fn apply<'e, E>(&self, executor: E) -> Result<(), SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite> + Copy,
    {
        // Note: Some PRAGMAs must be set before others or on initial connection

        let pragmas = vec![
            format!("PRAGMA journal_mode = {}", self.journal_mode.as_str()),
            format!("PRAGMA synchronous = {}", self.synchronous.as_str()),
            format!("PRAGMA cache_size = {}", self.cache_size),
            format!("PRAGMA mmap_size = {}", self.mmap_size),
            format!("PRAGMA temp_store = {}", self.temp_store.as_i32()),
            format!("PRAGMA foreign_keys = {}", if self.foreign_keys { "ON" } else { "OFF" }),
            format!("PRAGMA busy_timeout = {}", self.busy_timeout),
            format!("PRAGMA wal_autocheckpoint = {}", self.wal_autocheckpoint),
            format!("PRAGMA secure_delete = {}", if self.secure_delete { "ON" } else { "OFF" }),
            format!("PRAGMA recursive_triggers = {}", if self.recursive_triggers { "ON" } else { "OFF" }),
        ];

        for pragma in pragmas {
            debug!("Executing: {}", pragma);
            sqlx::query(&pragma)
                .execute(executor)
                .await?;
        }

        info!("SQLite configuration applied successfully");
        Ok(())
    }

    /// Apply only safe runtime PRAGMAs (those that don't require exclusive access)
    pub async fn apply_runtime<'e, E>(&self, executor: E) -> Result<(), SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite> + Copy,
    {
        let runtime_pragmas = vec![
            format!("PRAGMA cache_size = {}", self.cache_size),
            format!("PRAGMA busy_timeout = {}", self.busy_timeout),
            format!("PRAGMA wal_autocheckpoint = {}", self.wal_autocheckpoint),
        ];

        for pragma in runtime_pragmas {
            sqlx::query(&pragma)
                .execute(executor)
                .await?;
        }

        Ok(())
    }
}

/// Query current SQLite configuration
pub struct SqliteConfigQuery;

impl SqliteConfigQuery {
    pub async fn journal_mode<'e, E>(executor: E) -> Result<String, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let row: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(executor)
            .await?;
        Ok(row.0)
    }

    pub async fn cache_size<'e, E>(executor: E) -> Result<i64, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let row: (i64,) = sqlx::query_as("PRAGMA cache_size")
            .fetch_one(executor)
            .await?;
        Ok(row.0)
    }

    pub async fn page_count<'e, E>(executor: E) -> Result<i64, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let row: (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(executor)
            .await?;
        Ok(row.0)
    }

    pub async fn freelist_count<'e, E>(executor: E) -> Result<i64, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let row: (i64,) = sqlx::query_as("PRAGMA freelist_count")
            .fetch_one(executor)
            .await?;
        Ok(row.0)
    }

    pub async fn integrity_check<'e, E>(executor: E) -> Result<Vec<String>, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let rows: Vec<(String,)> = sqlx::query_as("PRAGMA integrity_check")
            .fetch_all(executor)
            .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn database_size<'e, E>(executor: E) -> Result<DatabaseSize, SqliteConfigError>
    where
        E: Executor<'e, Database = sqlx::Sqlite> + Copy,
    {
        let page_size: (i64,) = sqlx::query_as("PRAGMA page_size")
            .fetch_one(executor)
            .await?;
        let page_count: (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(executor)
            .await?;
        let freelist_count: (i64,) = sqlx::query_as("PRAGMA freelist_count")
            .fetch_one(executor)
            .await?;

        let total_bytes = page_size.0 * page_count.0;
        let free_bytes = page_size.0 * freelist_count.0;

        Ok(DatabaseSize {
            page_size: page_size.0,
            page_count: page_count.0,
            freelist_count: freelist_count.0,
            total_bytes,
            used_bytes: total_bytes - free_bytes,
            free_bytes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseSize {
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub free_bytes: i64,
}

impl DatabaseSize {
    pub fn fragmentation_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.free_bytes as f64 / self.total_bytes as f64
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_default_config() {
        let config = SqliteConfig::default();
        assert!(config.foreign_keys);
        assert!(matches!(config.journal_mode, JournalMode::Wal));
    }

    #[tokio::test]
    async fn test_apply_config() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let config = SqliteConfig::testing();
        config.apply(&pool).await.unwrap();

        let journal = SqliteConfigQuery::journal_mode(&pool).await.unwrap();
        assert_eq!(journal.to_lowercase(), "memory");
    }

    #[tokio::test]
    async fn test_database_size() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let size = SqliteConfigQuery::database_size(&pool).await.unwrap();
        assert!(size.page_size > 0);
    }
}