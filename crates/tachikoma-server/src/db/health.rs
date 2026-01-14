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