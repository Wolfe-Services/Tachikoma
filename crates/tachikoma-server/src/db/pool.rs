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