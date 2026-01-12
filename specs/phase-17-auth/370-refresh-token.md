# Spec 370: Refresh Token Implementation

## Overview
Implement secure refresh token handling for obtaining new access tokens without re-authentication.

## Rust Implementation

### Refresh Token Service
```rust
// src/auth/refresh.rs

use chrono::{DateTime, Duration, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

use super::tokens::{OpaqueToken, TokenType};
use super::jwt::{JwtHandler, Claims};
use super::types::{User, UserRole, AuthError};

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("Refresh token expired")]
    Expired,

    #[error("Refresh token revoked")]
    Revoked,

    #[error("Refresh token not found")]
    NotFound,

    #[error("Token family compromised")]
    FamilyCompromised,

    #[error("User not found")]
    UserNotFound,

    #[error("User disabled")]
    UserDisabled,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JWT error: {0}")]
    Jwt(String),
}

/// Stored refresh token
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: String,
    pub token_hash: String,
    pub user_id: String,
    pub family_id: String,  // For rotation tracking
    pub generation: i32,     // Increases with each rotation
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() &&
        self.rotated_at.is_none() &&
        Utc::now() < self.expires_at
    }
}

/// Refresh token configuration
#[derive(Debug, Clone)]
pub struct RefreshConfig {
    /// Token lifetime
    pub lifetime: Duration,
    /// Enable token rotation
    pub rotation_enabled: bool,
    /// Grace period after rotation (allow old token briefly)
    pub rotation_grace_period: Duration,
    /// Maximum tokens per user
    pub max_tokens_per_user: usize,
    /// Reuse detection (detect if rotated token is reused)
    pub reuse_detection: bool,
}

impl Default for RefreshConfig {
    fn default() -> Self {
        Self {
            lifetime: Duration::days(30),
            rotation_enabled: true,
            rotation_grace_period: Duration::minutes(2),
            reuse_detection: true,
            max_tokens_per_user: 10,
        }
    }
}

/// Refresh token result
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,  // New token if rotation enabled
    pub expires_in: i64,
}

/// Refresh token service
pub struct RefreshTokenService {
    pool: SqlitePool,
    jwt_handler: JwtHandler,
    config: RefreshConfig,
}

impl RefreshTokenService {
    pub fn new(pool: SqlitePool, jwt_handler: JwtHandler, config: RefreshConfig) -> Self {
        Self {
            pool,
            jwt_handler,
            config,
        }
    }

    /// Create a new refresh token
    #[instrument(skip(self))]
    pub async fn create(
        &self,
        user_id: &str,
        device_id: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(String, RefreshToken), RefreshError> {
        // Enforce max tokens
        self.enforce_max_tokens(user_id).await?;

        let id = Uuid::new_v4().to_string();
        let family_id = Uuid::new_v4().to_string();
        let token = OpaqueToken::generate(
            TokenType::Refresh,
            self.config.lifetime,
            32
        );
        let now = Utc::now();

        sqlx::query(r#"
            INSERT INTO refresh_tokens (
                id, token_hash, user_id, family_id, generation,
                device_id, ip_address, user_agent, created_at, expires_at
            ) VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&token.hash)
        .bind(user_id)
        .bind(&family_id)
        .bind(device_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(now)
        .bind(token.expires_at)
        .execute(&self.pool)
        .await?;

        let stored = self.get_by_id(&id).await?;

        info!("Created refresh token {} for user {}", id, user_id);
        Ok((token.value, stored))
    }

    /// Refresh access token
    #[instrument(skip(self, token))]
    pub async fn refresh(
        &self,
        token: &str,
        get_user: impl Fn(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<User, AuthError>> + Send + '_>>,
    ) -> Result<RefreshResult, RefreshError> {
        let token_hash = self.hash_token(token);

        // Find token
        let stored = sqlx::query_as::<_, RefreshToken>(
            "SELECT * FROM refresh_tokens WHERE token_hash = ?"
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RefreshError::NotFound)?;

        // Check if revoked
        if stored.revoked_at.is_some() {
            warn!("Attempt to use revoked refresh token: {}", stored.id);

            // If reuse detection enabled and token was rotated, revoke entire family
            if self.config.reuse_detection && stored.rotated_at.is_some() {
                self.revoke_family(&stored.family_id, "reuse_detected").await?;
                return Err(RefreshError::FamilyCompromised);
            }

            return Err(RefreshError::Revoked);
        }

        // Check if already rotated
        if stored.rotated_at.is_some() {
            // Check grace period
            let rotated_at = stored.rotated_at.unwrap();
            if Utc::now() > rotated_at + self.config.rotation_grace_period {
                warn!("Refresh token reuse detected outside grace period: {}", stored.id);

                if self.config.reuse_detection {
                    self.revoke_family(&stored.family_id, "reuse_detected").await?;
                    return Err(RefreshError::FamilyCompromised);
                }

                return Err(RefreshError::Revoked);
            }
            // Within grace period, allow but don't create new token
        }

        // Check expiration
        if Utc::now() > stored.expires_at {
            return Err(RefreshError::Expired);
        }

        // Get user
        let user = get_user(&stored.user_id).await
            .map_err(|_| RefreshError::UserNotFound)?;

        user.can_login().map_err(|_| RefreshError::UserDisabled)?;

        // Generate new access token
        let permissions = super::permissions::permissions_for_role(user.role)
            .into_iter()
            .collect();

        let access_token = self.jwt_handler
            .create_access_token(&user.id, Some(user.email.clone()), user.role, permissions)
            .map_err(|e| RefreshError::Jwt(e.to_string()))?;

        let expires_in = self.jwt_handler.config().lifetime.num_seconds();

        // Rotate refresh token if enabled and not already rotated
        let new_refresh_token = if self.config.rotation_enabled && stored.rotated_at.is_none() {
            let (new_token, _) = self.rotate(&stored).await?;
            Some(new_token)
        } else {
            None
        };

        debug!("Refreshed access token for user {}", stored.user_id);

        Ok(RefreshResult {
            access_token,
            refresh_token: new_refresh_token,
            expires_in,
        })
    }

    /// Rotate refresh token
    async fn rotate(&self, old_token: &RefreshToken) -> Result<(String, RefreshToken), RefreshError> {
        let id = Uuid::new_v4().to_string();
        let new_token = OpaqueToken::generate(
            TokenType::Refresh,
            self.config.lifetime,
            32
        );
        let now = Utc::now();

        // Create new token
        sqlx::query(r#"
            INSERT INTO refresh_tokens (
                id, token_hash, user_id, family_id, generation,
                device_id, ip_address, user_agent, created_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&new_token.hash)
        .bind(&old_token.user_id)
        .bind(&old_token.family_id)
        .bind(old_token.generation + 1)
        .bind(&old_token.device_id)
        .bind(&old_token.ip_address)
        .bind(&old_token.user_agent)
        .bind(now)
        .bind(new_token.expires_at)
        .execute(&self.pool)
        .await?;

        // Mark old token as rotated
        sqlx::query("UPDATE refresh_tokens SET rotated_at = ? WHERE id = ?")
            .bind(now)
            .bind(&old_token.id)
            .execute(&self.pool)
            .await?;

        let stored = self.get_by_id(&id).await?;

        debug!("Rotated refresh token {} -> {}", old_token.id, id);
        Ok((new_token.value, stored))
    }

    /// Revoke a refresh token
    #[instrument(skip(self))]
    pub async fn revoke(&self, token: &str, reason: Option<&str>) -> Result<(), RefreshError> {
        let token_hash = self.hash_token(token);
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ?, revoked_reason = ? WHERE token_hash = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(&token_hash)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RefreshError::NotFound);
        }

        info!("Revoked refresh token");
        Ok(())
    }

    /// Revoke all tokens in a family
    async fn revoke_family(&self, family_id: &str, reason: &str) -> Result<usize, RefreshError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ?, revoked_reason = ? WHERE family_id = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(family_id)
        .execute(&self.pool)
        .await?;

        let count = result.rows_affected() as usize;
        warn!("Revoked {} tokens in family {} due to: {}", count, family_id, reason);
        Ok(count)
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_for_user(&self, user_id: &str, reason: Option<&str>) -> Result<usize, RefreshError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = ?, revoked_reason = ? WHERE user_id = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(reason)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// List active tokens for user
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<RefreshToken>, RefreshError> {
        let tokens = sqlx::query_as::<_, RefreshToken>(r#"
            SELECT * FROM refresh_tokens
            WHERE user_id = ? AND revoked_at IS NULL AND rotated_at IS NULL AND expires_at > datetime('now')
            ORDER BY created_at DESC
        "#)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tokens)
    }

    /// Enforce maximum tokens per user
    async fn enforce_max_tokens(&self, user_id: &str) -> Result<(), RefreshError> {
        let tokens = self.list_for_user(user_id).await?;

        if tokens.len() >= self.config.max_tokens_per_user {
            let to_revoke = tokens.len() - self.config.max_tokens_per_user + 1;
            for token in tokens.iter().rev().take(to_revoke) {
                sqlx::query("UPDATE refresh_tokens SET revoked_at = datetime('now'), revoked_reason = 'max_tokens_exceeded' WHERE id = ?")
                    .bind(&token.id)
                    .execute(&self.pool)
                    .await?;
            }
        }

        Ok(())
    }

    /// Get token by ID
    async fn get_by_id(&self, id: &str) -> Result<RefreshToken, RefreshError> {
        sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(RefreshError::NotFound)
    }

    /// Hash token
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<usize, RefreshError> {
        let result = sqlx::query(
            "DELETE FROM refresh_tokens WHERE expires_at < datetime('now') OR revoked_at IS NOT NULL"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }
}

/// Refresh token database schema
pub fn refresh_token_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    family_id TEXT NOT NULL,
    generation INTEGER NOT NULL DEFAULT 1,
    device_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    rotated_at TEXT,
    revoked_at TEXT,
    revoked_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family ON refresh_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires ON refresh_tokens(expires_at);
"#
}
```

## Files to Create
- `src/auth/refresh.rs` - Refresh token service
