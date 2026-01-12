# Spec 367: Authentication Sessions

## Overview
Implement session management for authenticated users including creation, validation, renewal, and revocation.

## Rust Implementation

### Session Types
```rust
// src/auth/session.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use uuid::Uuid;
use tracing::{debug, info, warn, instrument};

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,

    #[error("Session expired")]
    Expired,

    #[error("Session revoked")]
    Revoked,

    #[error("Invalid session")]
    Invalid,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
}

impl Session {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && Utc::now() < self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn time_until_expiry(&self) -> Duration {
        self.expires_at.signed_duration_since(Utc::now())
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session lifetime
    pub lifetime: Duration,
    /// Idle timeout (session expires if no activity)
    pub idle_timeout: Duration,
    /// Maximum concurrent sessions per user
    pub max_sessions_per_user: usize,
    /// Enable sliding expiration (extend on activity)
    pub sliding_expiration: bool,
    /// Refresh threshold (extend when less than this time remaining)
    pub refresh_threshold: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            lifetime: Duration::days(30),
            idle_timeout: Duration::hours(24),
            max_sessions_per_user: 10,
            sliding_expiration: true,
            refresh_threshold: Duration::hours(1),
        }
    }
}

/// Session manager
pub struct SessionManager {
    pool: SqlitePool,
    config: SessionConfig,
}

impl SessionManager {
    pub fn new(pool: SqlitePool, config: SessionConfig) -> Self {
        Self { pool, config }
    }

    /// Create a new session
    #[instrument(skip(self, token))]
    pub async fn create(
        &self,
        user_id: &str,
        token: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_name: Option<&str>,
    ) -> Result<Session, SessionError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + self.config.lifetime;
        let token_hash = Self::hash_token(token);

        // Check max sessions and remove oldest if exceeded
        self.enforce_max_sessions(user_id).await?;

        sqlx::query(r#"
            INSERT INTO sessions (
                id, user_id, token_hash, ip_address, user_agent, device_name,
                created_at, expires_at, last_activity_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(ip_address)
        .bind(user_agent)
        .bind(device_name)
        .bind(now)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        info!("Created session {} for user {}", id, user_id);

        self.get(&id).await
    }

    /// Get session by ID
    pub async fn get(&self, session_id: &str) -> Result<Session, SessionError> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(SessionError::NotFound)?;

        Ok(session)
    }

    /// Validate session token
    #[instrument(skip(self, token))]
    pub async fn validate(&self, token: &str) -> Result<Session, SessionError> {
        let token_hash = Self::hash_token(token);

        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE token_hash = ?"
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(SessionError::NotFound)?;

        if session.is_revoked() {
            return Err(SessionError::Revoked);
        }

        if session.is_expired() {
            return Err(SessionError::Expired);
        }

        // Check idle timeout
        let idle_duration = Utc::now().signed_duration_since(session.last_activity_at);
        if idle_duration > self.config.idle_timeout {
            return Err(SessionError::Expired);
        }

        // Update last activity and optionally extend expiration
        self.touch(&session.id).await?;

        Ok(session)
    }

    /// Update session last activity
    async fn touch(&self, session_id: &str) -> Result<(), SessionError> {
        let now = Utc::now();

        if self.config.sliding_expiration {
            // Get current session
            let session = self.get(session_id).await?;

            // Extend if within refresh threshold
            if session.time_until_expiry() < self.config.refresh_threshold {
                let new_expires = now + self.config.lifetime;

                sqlx::query(
                    "UPDATE sessions SET last_activity_at = ?, expires_at = ? WHERE id = ?"
                )
                .bind(now)
                .bind(new_expires)
                .bind(session_id)
                .execute(&self.pool)
                .await?;

                debug!("Extended session {} expiration", session_id);
            } else {
                sqlx::query("UPDATE sessions SET last_activity_at = ? WHERE id = ?")
                    .bind(now)
                    .bind(session_id)
                    .execute(&self.pool)
                    .await?;
            }
        } else {
            sqlx::query("UPDATE sessions SET last_activity_at = ? WHERE id = ?")
                .bind(now)
                .bind(session_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Revoke a session
    #[instrument(skip(self))]
    pub async fn revoke(&self, session_id: &str, reason: Option<&str>) -> Result<(), SessionError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE sessions SET revoked_at = ?, revoked_reason = ? WHERE id = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SessionError::NotFound);
        }

        info!("Revoked session {}: {:?}", session_id, reason);
        Ok(())
    }

    /// Revoke all sessions for a user
    #[instrument(skip(self))]
    pub async fn revoke_all_for_user(&self, user_id: &str, reason: Option<&str>) -> Result<usize, SessionError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE sessions SET revoked_at = ?, revoked_reason = ? WHERE user_id = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        info!("Revoked {} sessions for user {}", count, user_id);
        Ok(count)
    }

    /// Revoke all sessions except current
    pub async fn revoke_other_sessions(
        &self,
        user_id: &str,
        current_session_id: &str,
        reason: Option<&str>,
    ) -> Result<usize, SessionError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE sessions SET revoked_at = ?, revoked_reason = ? WHERE user_id = ? AND id != ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(user_id)
        .bind(current_session_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// List active sessions for a user
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let sessions = sqlx::query_as::<_, Session>(r#"
            SELECT * FROM sessions
            WHERE user_id = ? AND revoked_at IS NULL AND expires_at > datetime('now')
            ORDER BY last_activity_at DESC
        "#)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(sessions)
    }

    /// Enforce maximum sessions per user
    async fn enforce_max_sessions(&self, user_id: &str) -> Result<(), SessionError> {
        let sessions = self.list_for_user(user_id).await?;

        if sessions.len() >= self.config.max_sessions_per_user {
            // Revoke oldest sessions
            let to_revoke = sessions.len() - self.config.max_sessions_per_user + 1;
            let oldest: Vec<_> = sessions
                .into_iter()
                .rev()
                .take(to_revoke)
                .collect();

            for session in oldest {
                self.revoke(&session.id, Some("max_sessions_exceeded")).await?;
            }
        }

        Ok(())
    }

    /// Cleanup expired sessions
    #[instrument(skip(self))]
    pub async fn cleanup_expired(&self) -> Result<usize, SessionError> {
        let result = sqlx::query(
            "DELETE FROM sessions WHERE expires_at < datetime('now') OR revoked_at IS NOT NULL"
        )
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        if count > 0 {
            info!("Cleaned up {} expired sessions", count);
        }
        Ok(count)
    }

    /// Hash session token
    fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate session token
    pub fn generate_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
    }
}

/// Session database schema migration
pub fn session_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    ip_address TEXT,
    user_agent TEXT,
    device_name TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    last_activity_at TEXT NOT NULL DEFAULT (datetime('now')),
    revoked_at TEXT,
    revoked_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/auth/session.rs` - Session management
