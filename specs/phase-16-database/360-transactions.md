# Spec 360: Database Transactions

## Overview
Implement robust transaction management with savepoints, automatic rollback, and nested transaction support.

## Rust Implementation

### Transaction Manager
```rust
// src/database/transaction.rs

use sqlx::sqlite::{SqlitePool, SqliteConnection};
use sqlx::{Acquire, Transaction, Sqlite};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Transaction failed: {0}")]
    Failed(String),

    #[error("Transaction already committed")]
    AlreadyCommitted,

    #[error("Transaction already rolled back")]
    AlreadyRolledBack,

    #[error("Savepoint not found: {0}")]
    SavepointNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Transaction options
#[derive(Debug, Clone, Default)]
pub struct TransactionOptions {
    /// Enable immediate mode (BEGIN IMMEDIATE)
    pub immediate: bool,
    /// Enable exclusive mode (BEGIN EXCLUSIVE)
    pub exclusive: bool,
    /// Timeout for acquiring locks
    pub timeout_ms: Option<u32>,
    /// Retry count for busy errors
    pub retry_count: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl TransactionOptions {
    pub fn immediate() -> Self {
        Self {
            immediate: true,
            ..Default::default()
        }
    }

    pub fn exclusive() -> Self {
        Self {
            exclusive: true,
            ..Default::default()
        }
    }

    pub fn with_retry(mut self, count: u32, delay_ms: u64) -> Self {
        self.retry_count = count;
        self.retry_delay_ms = delay_ms;
        self
    }
}

/// Transaction wrapper with automatic rollback on drop
pub struct TransactionGuard<'a> {
    tx: Option<Transaction<'a, Sqlite>>,
    committed: bool,
    rolled_back: bool,
    savepoints: Vec<String>,
}

impl<'a> TransactionGuard<'a> {
    /// Create a new transaction
    pub async fn begin(pool: &'a SqlitePool) -> Result<Self, TransactionError> {
        let tx = pool.begin().await?;
        debug!("Transaction started");
        Ok(Self {
            tx: Some(tx),
            committed: false,
            rolled_back: false,
            savepoints: Vec::new(),
        })
    }

    /// Create a new transaction with options
    pub async fn begin_with_options(
        pool: &'a SqlitePool,
        options: &TransactionOptions,
    ) -> Result<Self, TransactionError> {
        let tx = pool.begin().await?;

        // Set transaction mode
        if options.immediate {
            sqlx::query("BEGIN IMMEDIATE")
                .execute(&mut *tx.as_ref())
                .await
                .ok(); // May fail if already in transaction
        } else if options.exclusive {
            sqlx::query("BEGIN EXCLUSIVE")
                .execute(&mut *tx.as_ref())
                .await
                .ok();
        }

        debug!("Transaction started with options: {:?}", options);
        Ok(Self {
            tx: Some(tx),
            committed: false,
            rolled_back: false,
            savepoints: Vec::new(),
        })
    }

    /// Get a reference to the transaction for queries
    pub fn get(&mut self) -> Result<&mut Transaction<'a, Sqlite>, TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }
        self.tx.as_mut().ok_or(TransactionError::Failed("Transaction not available".to_string()))
    }

    /// Create a savepoint
    #[instrument(skip(self))]
    pub async fn savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        let tx = self.get()?;
        sqlx::query(&format!("SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await?;
        self.savepoints.push(name.to_string());
        debug!("Savepoint created: {}", name);
        Ok(())
    }

    /// Release a savepoint
    #[instrument(skip(self))]
    pub async fn release_savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        if !self.savepoints.contains(&name.to_string()) {
            return Err(TransactionError::SavepointNotFound(name.to_string()));
        }

        let tx = self.get()?;
        sqlx::query(&format!("RELEASE SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await?;

        self.savepoints.retain(|s| s != name);
        debug!("Savepoint released: {}", name);
        Ok(())
    }

    /// Rollback to a savepoint
    #[instrument(skip(self))]
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        if !self.savepoints.contains(&name.to_string()) {
            return Err(TransactionError::SavepointNotFound(name.to_string()));
        }

        let tx = self.get()?;
        sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", name))
            .execute(&mut **tx)
            .await?;

        debug!("Rolled back to savepoint: {}", name);
        Ok(())
    }

    /// Commit the transaction
    #[instrument(skip(self))]
    pub async fn commit(mut self) -> Result<(), TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }

        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            self.committed = true;
            debug!("Transaction committed");
        }

        Ok(())
    }

    /// Rollback the transaction
    #[instrument(skip(self))]
    pub async fn rollback(mut self) -> Result<(), TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted);
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack);
        }

        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
            self.rolled_back = true;
            debug!("Transaction rolled back");
        }

        Ok(())
    }
}

impl<'a> Drop for TransactionGuard<'a> {
    fn drop(&mut self) {
        if !self.committed && !self.rolled_back {
            if self.tx.is_some() {
                warn!("Transaction dropped without commit or explicit rollback - will be rolled back");
            }
        }
    }
}

/// Execute a closure within a transaction
pub async fn with_transaction<F, T, E>(
    pool: &SqlitePool,
    f: F,
) -> Result<T, E>
where
    F: for<'c> FnOnce(&'c mut Transaction<'_, Sqlite>) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await.map_err(E::from)?;
    let result = f(&mut tx).await;

    match result {
        Ok(value) => {
            tx.commit().await.map_err(E::from)?;
            Ok(value)
        }
        Err(e) => {
            // Transaction will be rolled back on drop
            Err(e)
        }
    }
}

/// Execute with retry on busy errors
pub async fn with_retry<F, T, E>(
    pool: &SqlitePool,
    options: &TransactionOptions,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts > options.retry_count {
                    error!("Transaction failed after {} attempts: {:?}", attempts, e);
                    return Err(e);
                }

                warn!("Transaction attempt {} failed, retrying: {:?}", attempts, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(options.retry_delay_ms)).await;
            }
        }
    }
}

/// Unit of Work pattern implementation
pub struct UnitOfWork<'a> {
    tx: TransactionGuard<'a>,
    operations: Vec<String>,  // For logging
}

impl<'a> UnitOfWork<'a> {
    pub async fn new(pool: &'a SqlitePool) -> Result<Self, TransactionError> {
        Ok(Self {
            tx: TransactionGuard::begin(pool).await?,
            operations: Vec::new(),
        })
    }

    pub fn transaction(&mut self) -> Result<&mut Transaction<'a, Sqlite>, TransactionError> {
        self.tx.get()
    }

    pub fn record_operation(&mut self, op: impl Into<String>) {
        self.operations.push(op.into());
    }

    pub async fn savepoint(&mut self, name: &str) -> Result<(), TransactionError> {
        self.tx.savepoint(name).await
    }

    pub async fn rollback_to(&mut self, name: &str) -> Result<(), TransactionError> {
        self.tx.rollback_to_savepoint(name).await
    }

    pub async fn commit(self) -> Result<Vec<String>, TransactionError> {
        self.tx.commit().await?;
        Ok(self.operations)
    }

    pub async fn rollback(self) -> Result<(), TransactionError> {
        self.tx.rollback().await
    }
}

/// Macro for transaction blocks
#[macro_export]
macro_rules! transaction {
    ($pool:expr, |$tx:ident| $body:block) => {{
        let mut $tx = $pool.begin().await?;
        let result = (|| async { $body })().await;
        match result {
            Ok(value) => {
                $tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                // tx will rollback on drop
                Err(e)
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let pool = test_pool().await;

        // Create table
        sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();

        // Use transaction
        let mut guard = TransactionGuard::begin(&pool).await.unwrap();
        {
            let tx = guard.get().unwrap();
            sqlx::query("INSERT INTO test (id) VALUES (1)")
                .execute(&mut **tx)
                .await
                .unwrap();
        }
        guard.commit().await.unwrap();

        // Verify
        let (count,): (i32,) = sqlx::query_as("SELECT COUNT(*) FROM test")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let pool = test_pool().await;

        sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();

        // Transaction that will be rolled back
        {
            let mut guard = TransactionGuard::begin(&pool).await.unwrap();
            {
                let tx = guard.get().unwrap();
                sqlx::query("INSERT INTO test (id) VALUES (1)")
                    .execute(&mut **tx)
                    .await
                    .unwrap();
            }
            guard.rollback().await.unwrap();
        }

        // Verify - should be empty
        let (count,): (i32,) = sqlx::query_as("SELECT COUNT(*) FROM test")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_savepoints() {
        let pool = test_pool().await;

        sqlx::query("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .execute(&pool)
            .await
            .unwrap();

        let mut guard = TransactionGuard::begin(&pool).await.unwrap();

        // Insert first value
        {
            let tx = guard.get().unwrap();
            sqlx::query("INSERT INTO test (id) VALUES (1)")
                .execute(&mut **tx)
                .await
                .unwrap();
        }

        // Create savepoint
        guard.savepoint("sp1").await.unwrap();

        // Insert second value
        {
            let tx = guard.get().unwrap();
            sqlx::query("INSERT INTO test (id) VALUES (2)")
                .execute(&mut **tx)
                .await
                .unwrap();
        }

        // Rollback to savepoint
        guard.rollback_to_savepoint("sp1").await.unwrap();

        guard.commit().await.unwrap();

        // Only first value should exist
        let (count,): (i32,) = sqlx::query_as("SELECT COUNT(*) FROM test")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}
```

## Files to Create
- `src/database/transaction.rs` - Transaction management
