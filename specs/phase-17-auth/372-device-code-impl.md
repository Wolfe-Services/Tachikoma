# Spec 372: Device Code Implementation

## Overview
Implement the OAuth 2.0 Device Authorization Grant flow service for authenticating input-constrained devices.

## Rust Implementation

### Device Code Service
```rust
// src/auth/device_code/service.rs

use super::types::*;
use chrono::{DateTime, Duration, Utc};
use sqlx::sqlite::SqlitePool;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

/// Device code service
pub struct DeviceCodeService {
    pool: SqlitePool,
    config: DeviceCodeConfig,
}

impl DeviceCodeService {
    pub fn new(pool: SqlitePool, config: DeviceCodeConfig) -> Self {
        Self { pool, config }
    }

    /// Initialize a device authorization request
    #[instrument(skip(self))]
    pub async fn initialize(
        &self,
        client_id: Option<&str>,
        scope: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<DeviceAuthorizationResponse, DeviceCodeError> {
        let id = Uuid::new_v4().to_string();
        let device_code = self.generate_device_code();
        let device_code_hash = self.hash_code(&device_code);
        let user_code = self.generate_user_code();
        let now = Utc::now();
        let expires_at = now + self.config.code_lifetime;

        sqlx::query(r#"
            INSERT INTO device_codes (
                id, device_code_hash, user_code, client_id, scope, status,
                ip_address, user_agent, created_at, expires_at, poll_count
            ) VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?, 0)
        "#)
        .bind(&id)
        .bind(&device_code_hash)
        .bind(&user_code)
        .bind(client_id)
        .bind(scope)
        .bind(ip_address)
        .bind(user_agent)
        .bind(now)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        let verification_uri_complete = format!(
            "{}?user_code={}",
            self.config.verification_uri,
            user_code.replace("-", "")
        );

        info!("Created device code {} with user code {}", id, user_code);

        Ok(DeviceAuthorizationResponse {
            device_code,
            user_code,
            verification_uri: self.config.verification_uri.clone(),
            verification_uri_complete: Some(verification_uri_complete),
            expires_in: self.config.code_lifetime.num_seconds(),
            interval: self.config.poll_interval.num_seconds(),
        })
    }

    /// Poll for token (called by device)
    #[instrument(skip(self, device_code))]
    pub async fn poll(&self, device_code: &str) -> Result<PollResult, DeviceCodeError> {
        let device_code_hash = self.hash_code(device_code);

        // Find device code
        let stored = sqlx::query_as::<_, DeviceCode>(
            "SELECT * FROM device_codes WHERE device_code_hash = ?"
        )
        .bind(&device_code_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DeviceCodeError::NotFound)?;

        // Check if expired
        if stored.is_expired() {
            // Update status
            sqlx::query("UPDATE device_codes SET status = 'expired' WHERE id = ?")
                .bind(&stored.id)
                .execute(&self.pool)
                .await?;
            return Err(DeviceCodeError::Expired);
        }

        // Check polling rate
        if !stored.can_poll(self.config.poll_interval) {
            return Err(DeviceCodeError::SlowDown);
        }

        // Check max poll attempts
        if stored.poll_count >= self.config.max_poll_attempts {
            return Err(DeviceCodeError::Expired);
        }

        // Update poll tracking
        sqlx::query(
            "UPDATE device_codes SET last_polled_at = datetime('now'), poll_count = poll_count + 1 WHERE id = ?"
        )
        .bind(&stored.id)
        .execute(&self.pool)
        .await?;

        // Check status
        match stored.status() {
            DeviceCodeStatus::Pending => {
                Err(DeviceCodeError::AuthorizationPending)
            }
            DeviceCodeStatus::Authorized => {
                // Return user_id for token generation
                let user_id = stored.user_id
                    .ok_or(DeviceCodeError::NotFound)?;

                Ok(PollResult::Authorized {
                    user_id,
                    scope: stored.scope,
                })
            }
            DeviceCodeStatus::Denied => {
                Err(DeviceCodeError::AccessDenied)
            }
            DeviceCodeStatus::Completed => {
                Err(DeviceCodeError::AlreadyUsed)
            }
            DeviceCodeStatus::Expired => {
                Err(DeviceCodeError::Expired)
            }
        }
    }

    /// Mark device code as completed (after token issued)
    pub async fn mark_completed(&self, device_code: &str) -> Result<(), DeviceCodeError> {
        let device_code_hash = self.hash_code(device_code);

        sqlx::query(
            "UPDATE device_codes SET status = 'completed', completed_at = datetime('now') WHERE device_code_hash = ?"
        )
        .bind(&device_code_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Authorize device code (called after user approves)
    #[instrument(skip(self))]
    pub async fn authorize(
        &self,
        user_code: &str,
        user_id: &str,
    ) -> Result<DeviceCode, DeviceCodeError> {
        let normalized_code = self.config.user_code_format.normalize(user_code);

        // Find by user code
        let stored = sqlx::query_as::<_, DeviceCode>(
            "SELECT * FROM device_codes WHERE REPLACE(user_code, '-', '') = ?"
        )
        .bind(&normalized_code)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DeviceCodeError::InvalidUserCode)?;

        // Validate state
        if stored.is_expired() {
            return Err(DeviceCodeError::Expired);
        }

        if stored.status() != DeviceCodeStatus::Pending {
            return Err(DeviceCodeError::AlreadyAuthorized);
        }

        // Update to authorized
        sqlx::query(
            "UPDATE device_codes SET status = 'authorized', user_id = ?, authorized_at = datetime('now') WHERE id = ?"
        )
        .bind(user_id)
        .bind(&stored.id)
        .execute(&self.pool)
        .await?;

        info!("Device code {} authorized by user {}", stored.id, user_id);

        // Return updated record
        self.get_by_id(&stored.id).await
    }

    /// Deny device code (called when user rejects)
    #[instrument(skip(self))]
    pub async fn deny(&self, user_code: &str) -> Result<(), DeviceCodeError> {
        let normalized_code = self.config.user_code_format.normalize(user_code);

        let result = sqlx::query(
            "UPDATE device_codes SET status = 'denied' WHERE REPLACE(user_code, '-', '') = ? AND status = 'pending'"
        )
        .bind(&normalized_code)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DeviceCodeError::InvalidUserCode);
        }

        info!("Device code denied for user code {}", user_code);
        Ok(())
    }

    /// Get device code by user code (for display in authorization page)
    pub async fn get_by_user_code(&self, user_code: &str) -> Result<Option<DeviceCode>, DeviceCodeError> {
        let normalized_code = self.config.user_code_format.normalize(user_code);

        let code = sqlx::query_as::<_, DeviceCode>(
            "SELECT * FROM device_codes WHERE REPLACE(user_code, '-', '') = ?"
        )
        .bind(&normalized_code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(code)
    }

    /// Get device code by ID
    async fn get_by_id(&self, id: &str) -> Result<DeviceCode, DeviceCodeError> {
        sqlx::query_as::<_, DeviceCode>("SELECT * FROM device_codes WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(DeviceCodeError::NotFound)
    }

    /// Generate random device code
    fn generate_device_code(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
    }

    /// Generate user-friendly code
    fn generate_user_code(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let charset: Vec<char> = self.config.user_code_format.charset().chars().collect();

        let code: String = (0..self.config.user_code_length)
            .map(|_| charset[rng.gen_range(0..charset.len())])
            .collect();

        self.config.user_code_format.format(&code)
    }

    /// Hash code for storage
    fn hash_code(&self, code: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Cleanup expired codes
    pub async fn cleanup(&self) -> Result<usize, DeviceCodeError> {
        let result = sqlx::query(
            "DELETE FROM device_codes WHERE expires_at < datetime('now')"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// Get page data for user authorization
    pub async fn get_authorization_page(&self, user_code: &str) -> Result<Option<DeviceAuthorizationPage>, DeviceCodeError> {
        let code = self.get_by_user_code(user_code).await?;

        Ok(code.map(|c| {
            let expires_in_minutes = c.expires_at
                .signed_duration_since(Utc::now())
                .num_minutes()
                .max(0);

            let scope_descriptions = c.scope
                .as_ref()
                .map(|s| self.describe_scopes(s))
                .unwrap_or_default();

            DeviceAuthorizationPage {
                user_code: c.user_code,
                client_name: c.client_id,  // Could lookup client name
                scope: c.scope,
                scope_descriptions,
                expires_in_minutes,
            }
        }))
    }

    /// Convert scope string to human-readable descriptions
    fn describe_scopes(&self, scope: &str) -> Vec<String> {
        scope.split_whitespace()
            .filter_map(|s| match s {
                "read" => Some("Read your data".to_string()),
                "write" => Some("Modify your data".to_string()),
                "profile" => Some("Access your profile information".to_string()),
                "email" => Some("Access your email address".to_string()),
                "offline_access" => Some("Access data while you're away".to_string()),
                _ => Some(format!("Access to: {}", s)),
            })
            .collect()
    }
}

/// Poll result
#[derive(Debug, Clone)]
pub enum PollResult {
    Authorized {
        user_id: String,
        scope: Option<String>,
    },
}

/// Device code database schema
pub fn device_code_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS device_codes (
    id TEXT PRIMARY KEY NOT NULL,
    device_code_hash TEXT NOT NULL UNIQUE,
    user_code TEXT NOT NULL,
    client_id TEXT,
    scope TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    user_id TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    authorized_at TEXT,
    completed_at TEXT,
    last_polled_at TEXT,
    poll_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_device_codes_hash ON device_codes(device_code_hash);
CREATE INDEX IF NOT EXISTS idx_device_codes_user ON device_codes(user_code);
CREATE INDEX IF NOT EXISTS idx_device_codes_expires ON device_codes(expires_at);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/auth/device_code/service.rs` - Device code service
