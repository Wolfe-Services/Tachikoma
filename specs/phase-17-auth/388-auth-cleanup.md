# Spec 388: Authentication Cleanup

## Overview
Implement cleanup services for expired tokens, sessions, and other authentication artifacts.

## Rust Implementation

### Auth Cleanup Service
```rust
// src/auth/cleanup.rs

use chrono::{DateTime, Duration, Utc};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, instrument};

/// Cleanup service configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// Run cleanup interval
    pub interval: Duration,
    /// Session cleanup: delete sessions older than
    pub session_max_age: Duration,
    /// Refresh token cleanup: delete tokens older than
    pub refresh_token_max_age: Duration,
    /// OAuth state cleanup: delete states older than
    pub oauth_state_max_age: Duration,
    /// Magic link cleanup: delete links older than
    pub magic_link_max_age: Duration,
    /// Device code cleanup: delete codes older than
    pub device_code_max_age: Duration,
    /// Audit log retention (delete logs older than)
    pub audit_log_retention: Duration,
    /// Rate limit cleanup: delete entries older than
    pub rate_limit_max_age: Duration,
    /// Batch size for deletions
    pub batch_size: i32,
    /// Enable vacuum after cleanup
    pub vacuum_after_cleanup: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            interval: Duration::hours(1),
            session_max_age: Duration::days(30),
            refresh_token_max_age: Duration::days(90),
            oauth_state_max_age: Duration::hours(1),
            magic_link_max_age: Duration::days(1),
            device_code_max_age: Duration::hours(1),
            audit_log_retention: Duration::days(90),
            rate_limit_max_age: Duration::days(1),
            batch_size: 1000,
            vacuum_after_cleanup: false,
        }
    }
}

/// Cleanup statistics
#[derive(Debug, Default, Clone)]
pub struct CleanupStats {
    pub sessions_deleted: usize,
    pub refresh_tokens_deleted: usize,
    pub oauth_states_deleted: usize,
    pub magic_links_deleted: usize,
    pub device_codes_deleted: usize,
    pub audit_logs_deleted: usize,
    pub rate_limits_deleted: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

impl CleanupStats {
    pub fn total_deleted(&self) -> usize {
        self.sessions_deleted +
        self.refresh_tokens_deleted +
        self.oauth_states_deleted +
        self.magic_links_deleted +
        self.device_codes_deleted +
        self.audit_logs_deleted +
        self.rate_limits_deleted
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Auth cleanup service
pub struct AuthCleanupService {
    pool: SqlitePool,
    config: CleanupConfig,
    last_run: Arc<Mutex<Option<DateTime<Utc>>>>,
    last_stats: Arc<Mutex<Option<CleanupStats>>>,
}

impl AuthCleanupService {
    pub fn new(pool: SqlitePool, config: CleanupConfig) -> Self {
        Self {
            pool,
            config,
            last_run: Arc::new(Mutex::new(None)),
            last_stats: Arc::new(Mutex::new(None)),
        }
    }

    /// Run all cleanup tasks
    #[instrument(skip(self))]
    pub async fn run_all(&self) -> CleanupStats {
        let start = std::time::Instant::now();
        let mut stats = CleanupStats::default();

        info!("Starting auth cleanup");

        // Clean sessions
        match self.cleanup_sessions().await {
            Ok(count) => stats.sessions_deleted = count,
            Err(e) => stats.errors.push(format!("Sessions: {}", e)),
        }

        // Clean refresh tokens
        match self.cleanup_refresh_tokens().await {
            Ok(count) => stats.refresh_tokens_deleted = count,
            Err(e) => stats.errors.push(format!("Refresh tokens: {}", e)),
        }

        // Clean OAuth states
        match self.cleanup_oauth_states().await {
            Ok(count) => stats.oauth_states_deleted = count,
            Err(e) => stats.errors.push(format!("OAuth states: {}", e)),
        }

        // Clean magic links
        match self.cleanup_magic_links().await {
            Ok(count) => stats.magic_links_deleted = count,
            Err(e) => stats.errors.push(format!("Magic links: {}", e)),
        }

        // Clean device codes
        match self.cleanup_device_codes().await {
            Ok(count) => stats.device_codes_deleted = count,
            Err(e) => stats.errors.push(format!("Device codes: {}", e)),
        }

        // Clean audit logs
        match self.cleanup_audit_logs().await {
            Ok(count) => stats.audit_logs_deleted = count,
            Err(e) => stats.errors.push(format!("Audit logs: {}", e)),
        }

        // Clean rate limits
        match self.cleanup_rate_limits().await {
            Ok(count) => stats.rate_limits_deleted = count,
            Err(e) => stats.errors.push(format!("Rate limits: {}", e)),
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        // Run vacuum if configured and we deleted a lot
        if self.config.vacuum_after_cleanup && stats.total_deleted() > 1000 {
            if let Err(e) = self.vacuum().await {
                stats.errors.push(format!("Vacuum: {}", e));
            }
        }

        // Update last run info
        *self.last_run.lock().await = Some(Utc::now());
        *self.last_stats.lock().await = Some(stats.clone());

        info!(
            deleted = stats.total_deleted(),
            duration_ms = stats.duration_ms,
            errors = stats.errors.len(),
            "Auth cleanup completed"
        );

        stats
    }

    /// Cleanup expired sessions
    async fn cleanup_sessions(&self) -> Result<usize, sqlx::Error> {
        let cutoff = Utc::now() - self.config.session_max_age;

        let result = sqlx::query(
            "DELETE FROM sessions WHERE expires_at < ? OR (last_active_at < ? AND created_at < ?)"
        )
        .bind(Utc::now())
        .bind(cutoff)
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} expired sessions", count);
        }
        Ok(count)
    }

    /// Cleanup expired refresh tokens
    async fn cleanup_refresh_tokens(&self) -> Result<usize, sqlx::Error> {
        // Delete expired tokens
        let result1 = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < datetime('now')")
            .execute(&self.pool)
            .await?;

        // Delete revoked tokens older than retention
        let cutoff = Utc::now() - Duration::days(7);
        let result2 = sqlx::query("DELETE FROM refresh_tokens WHERE revoked_at IS NOT NULL AND revoked_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        let count = (result1.rows_affected() + result2.rows_affected()) as usize;
        if count > 0 {
            debug!("Deleted {} expired/revoked refresh tokens", count);
        }
        Ok(count)
    }

    /// Cleanup expired OAuth states
    async fn cleanup_oauth_states(&self) -> Result<usize, sqlx::Error> {
        let result = sqlx::query("DELETE FROM oauth_states WHERE expires_at < datetime('now')")
            .execute(&self.pool)
            .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} expired OAuth states", count);
        }
        Ok(count)
    }

    /// Cleanup expired magic links
    async fn cleanup_magic_links(&self) -> Result<usize, sqlx::Error> {
        let cutoff = Utc::now() - self.config.magic_link_max_age;

        // Delete expired or used magic links
        let result = sqlx::query(
            "DELETE FROM magic_link_tokens WHERE expires_at < datetime('now') OR (used_at IS NOT NULL AND used_at < ?)"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} expired/used magic links", count);
        }
        Ok(count)
    }

    /// Cleanup expired device codes
    async fn cleanup_device_codes(&self) -> Result<usize, sqlx::Error> {
        let cutoff = Utc::now() - self.config.device_code_max_age;

        // Delete expired or completed device codes
        let result = sqlx::query(
            "DELETE FROM device_codes WHERE expires_at < datetime('now') OR (completed_at IS NOT NULL AND completed_at < ?)"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} expired/completed device codes", count);
        }
        Ok(count)
    }

    /// Cleanup old audit logs
    async fn cleanup_audit_logs(&self) -> Result<usize, sqlx::Error> {
        let cutoff = Utc::now() - self.config.audit_log_retention;

        let result = sqlx::query("DELETE FROM auth_audit_logs WHERE created_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} old audit logs", count);
        }
        Ok(count)
    }

    /// Cleanup old rate limit entries
    async fn cleanup_rate_limits(&self) -> Result<usize, sqlx::Error> {
        let cutoff = Utc::now() - self.config.rate_limit_max_age;

        let result = sqlx::query(
            "DELETE FROM rate_limits WHERE last_attempt < ? AND locked_until IS NULL"
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            debug!("Deleted {} old rate limit entries", count);
        }
        Ok(count)
    }

    /// Run VACUUM on database
    async fn vacuum(&self) -> Result<(), sqlx::Error> {
        debug!("Running VACUUM");
        sqlx::query("VACUUM")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get last run time
    pub async fn last_run(&self) -> Option<DateTime<Utc>> {
        *self.last_run.lock().await
    }

    /// Get last run stats
    pub async fn last_stats(&self) -> Option<CleanupStats> {
        self.last_stats.lock().await.clone()
    }

    /// Check if cleanup is due
    pub async fn is_due(&self) -> bool {
        match *self.last_run.lock().await {
            Some(last) => Utc::now() > last + self.config.interval,
            None => true,
        }
    }
}

/// Background cleanup scheduler
pub struct CleanupScheduler {
    service: Arc<AuthCleanupService>,
    running: Arc<Mutex<bool>>,
}

impl CleanupScheduler {
    pub fn new(service: Arc<AuthCleanupService>) -> Self {
        Self {
            service,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the cleanup scheduler
    pub async fn start(self: Arc<Self>) {
        let mut running = self.running.lock().await;
        if *running {
            warn!("Cleanup scheduler already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Starting cleanup scheduler");

        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler.run_loop().await;
        });
    }

    /// Stop the cleanup scheduler
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
        info!("Cleanup scheduler stopped");
    }

    /// Main cleanup loop
    async fn run_loop(&self) {
        loop {
            // Check if still running
            if !*self.running.lock().await {
                break;
            }

            // Check if cleanup is due
            if self.service.is_due().await {
                let stats = self.service.run_all().await;

                if stats.has_errors() {
                    warn!(errors = ?stats.errors, "Cleanup completed with errors");
                }
            }

            // Sleep until next check (1 minute)
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}

/// Manual cleanup trigger
pub struct CleanupTrigger {
    service: Arc<AuthCleanupService>,
}

impl CleanupTrigger {
    pub fn new(service: Arc<AuthCleanupService>) -> Self {
        Self { service }
    }

    /// Trigger immediate cleanup
    pub async fn trigger(&self) -> CleanupStats {
        self.service.run_all().await
    }

    /// Cleanup specific type only
    pub async fn cleanup_sessions(&self) -> Result<usize, sqlx::Error> {
        self.service.cleanup_sessions().await
    }

    pub async fn cleanup_tokens(&self) -> Result<usize, sqlx::Error> {
        self.service.cleanup_refresh_tokens().await
    }

    pub async fn cleanup_oauth(&self) -> Result<usize, sqlx::Error> {
        self.service.cleanup_oauth_states().await
    }

    /// Force vacuum
    pub async fn vacuum(&self) -> Result<(), sqlx::Error> {
        self.service.vacuum().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_stats() {
        let mut stats = CleanupStats::default();
        stats.sessions_deleted = 10;
        stats.refresh_tokens_deleted = 5;

        assert_eq!(stats.total_deleted(), 15);
        assert!(!stats.has_errors());

        stats.errors.push("Test error".to_string());
        assert!(stats.has_errors());
    }

    #[test]
    fn test_config_defaults() {
        let config = CleanupConfig::default();

        assert_eq!(config.interval, Duration::hours(1));
        assert_eq!(config.session_max_age, Duration::days(30));
        assert_eq!(config.audit_log_retention, Duration::days(90));
    }
}
```

## Files to Create
- `src/auth/cleanup.rs` - Authentication cleanup service
