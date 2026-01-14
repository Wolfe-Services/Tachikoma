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