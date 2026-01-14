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