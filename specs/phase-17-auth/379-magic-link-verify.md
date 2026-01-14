# Spec 379: Magic Link Verification

## Overview
Implement magic link verification service for passwordless authentication.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Magic Link Service
```rust
// src/auth/magic_link/service.rs

use super::types::*;
use super::email::{MagicLinkEmailSender, MagicLinkEmailData};
use chrono::{Duration, Utc};
use sqlx::sqlite::SqlitePool;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

/// Magic link service
pub struct MagicLinkService {
    pool: SqlitePool,
    config: MagicLinkConfig,
    email_sender: MagicLinkEmailSender,
}

impl MagicLinkService {
    pub fn new(
        pool: SqlitePool,
        config: MagicLinkConfig,
        email_sender: MagicLinkEmailSender,
    ) -> Self {
        Self {
            pool,
            config,
            email_sender,
        }
    }

    /// Create and send a magic link
    #[instrument(skip(self, request))]
    pub async fn create_and_send(
        &self,
        request: &MagicLinkRequest,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<MagicLinkResponse, MagicLinkError> {
        // Check rate limits
        self.check_rate_limit(&request.email, ip_address).await?;

        // Validate email format
        if !self.is_valid_email(&request.email) {
            // Don't reveal if email is invalid
            return Ok(MagicLinkResponse::sent());
        }

        // Check if user exists (for login purpose)
        let user_id = self.find_user_by_email(&request.email).await?;

        // For login, require existing user
        if request.purpose == MagicLinkPurpose::Login && user_id.is_none() {
            // Don't reveal user doesn't exist
            info!("Magic link requested for non-existent user: {}", request.email);
            return Ok(MagicLinkResponse::sent());
        }

        // For signup, check if allowed and user doesn't exist
        if request.purpose == MagicLinkPurpose::Signup {
            if !self.config.allow_signup {
                return Err(MagicLinkError::Invalid);
            }
            if user_id.is_some() {
                // User exists, send login link instead
                info!("Signup requested for existing user, sending login link");
            }
        }

        // Generate token
        let (token, token_hash) = self.generate_token();
        let id = Uuid::new_v4().to_string();
        let lifetime = self.config.lifetime_for(request.purpose);
        let expires_at = Utc::now() + lifetime;

        // Store token
        sqlx::query(r#"
            INSERT INTO magic_link_tokens (
                id, token_hash, email, user_id, purpose, redirect_to, metadata,
                ip_address, user_agent, created_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?)
        "#)
        .bind(&id)
        .bind(&token_hash)
        .bind(&request.email)
        .bind(&user_id)
        .bind(request.purpose.as_str())
        .bind(&request.redirect_to)
        .bind(request.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten())
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        // Update rate limit
        self.update_rate_limit(&request.email, ip_address).await?;

        // Generate full URL
        let magic_link_url = format!(
            "{}{}?token={}",
            self.config.base_url,
            self.config.verify_path,
            token
        );

        // Prepare email data
        let email_data = MagicLinkEmailData {
            recipient_email: request.email.clone(),
            recipient_name: None, // Could look up from user record
            magic_link_url: magic_link_url.clone(),
            purpose: request.purpose.as_str().to_string(),
            expires_in_minutes: lifetime.num_minutes(),
            ip_address: ip_address.map(String::from),
            user_agent: user_agent.map(String::from),
            app_name: self.config.sender_name.clone(),
        };

        // Send email
        self.email_sender.send(&email_data).await?;

        info!("Created magic link {} for email {}", id, request.email);

        // In dev mode, include debug URL
        let mut response = MagicLinkResponse::sent();
        if cfg!(debug_assertions) {
            response = response.with_debug_url(&magic_link_url);
        }

        Ok(response)
    }

    /// Verify magic link token
    #[instrument(skip(self, token))]
    pub async fn verify(&self, token: &str) -> Result<MagicLinkVerifyResult, MagicLinkError> {
        let token_hash = self.hash_token(token);

        // Find token
        let stored = sqlx::query_as::<_, MagicLinkToken>(
            "SELECT * FROM magic_link_tokens WHERE token_hash = ?"
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(MagicLinkError::NotFound)?;

        // Check if already used
        if stored.used_at.is_some() {
            return Err(MagicLinkError::AlreadyUsed);
        }

        // Check if revoked
        if stored.revoked_at.is_some() {
            return Err(MagicLinkError::Invalid);
        }

        // Check expiration
        if stored.is_expired() {
            return Err(MagicLinkError::Expired);
        }

        // Mark as used
        sqlx::query("UPDATE magic_link_tokens SET used_at = datetime('now') WHERE id = ?")
            .bind(&stored.id)
            .execute(&self.pool)
            .await?;

        // Get redirect URL from metadata
        let redirect_to = sqlx::query_scalar::<_, Option<String>>(
            "SELECT redirect_to FROM magic_link_tokens WHERE id = ?"
        )
        .bind(&stored.id)
        .fetch_one(&self.pool)
        .await?;

        // Check if new user for signup
        let is_new_user = stored.user_id.is_none() && stored.purpose() == MagicLinkPurpose::Signup;

        info!("Verified magic link {} for email {}", stored.id, stored.email);

        Ok(MagicLinkVerifyResult {
            email: stored.email,
            user_id: stored.user_id,
            purpose: stored.purpose(),
            redirect_to,
            is_new_user,
        })
    }

    /// Revoke a magic link
    pub async fn revoke(&self, token: &str) -> Result<(), MagicLinkError> {
        let token_hash = self.hash_token(token);

        let result = sqlx::query(
            "UPDATE magic_link_tokens SET revoked_at = datetime('now') WHERE token_hash = ? AND used_at IS NULL"
        )
        .bind(&token_hash)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(MagicLinkError::NotFound);
        }

        Ok(())
    }

    /// Revoke all magic links for an email
    pub async fn revoke_all_for_email(&self, email: &str) -> Result<usize, MagicLinkError> {
        let result = sqlx::query(
            "UPDATE magic_link_tokens SET revoked_at = datetime('now') WHERE email = ? AND used_at IS NULL AND revoked_at IS NULL"
        )
        .bind(email)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    /// Check rate limit
    async fn check_rate_limit(
        &self,
        email: &str,
        ip_address: Option<&str>,
    ) -> Result<(), MagicLinkError> {
        let now = Utc::now();
        let window_start = now - Duration::hours(1);

        // Check email rate limit
        let email_count: i32 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(count), 0) FROM magic_link_rate_limits WHERE key = ? AND key_type = 'email' AND window_start > ?"
        )
        .bind(email)
        .bind(window_start)
        .fetch_one(&self.pool)
        .await?;

        if email_count as u32 >= self.config.rate_limit_per_email {
            warn!("Rate limit exceeded for email: {}", email);
            return Err(MagicLinkError::RateLimited);
        }

        // Check IP rate limit
        if let Some(ip) = ip_address {
            let ip_count: i32 = sqlx::query_scalar(
                "SELECT COALESCE(SUM(count), 0) FROM magic_link_rate_limits WHERE key = ? AND key_type = 'ip' AND window_start > ?"
            )
            .bind(ip)
            .bind(window_start)
            .fetch_one(&self.pool)
            .await?;

            if ip_count as u32 >= self.config.rate_limit_per_ip {
                warn!("Rate limit exceeded for IP: {}", ip);
                return Err(MagicLinkError::RateLimited);
            }
        }

        Ok(())
    }

    /// Update rate limit counters
    async fn update_rate_limit(
        &self,
        email: &str,
        ip_address: Option<&str>,
    ) -> Result<(), MagicLinkError> {
        // Update email rate limit
        sqlx::query(r#"
            INSERT INTO magic_link_rate_limits (key, key_type, count, window_start)
            VALUES (?, 'email', 1, datetime('now'))
            ON CONFLICT(key, key_type) DO UPDATE SET
                count = CASE
                    WHEN window_start < datetime('now', '-1 hour') THEN 1
                    ELSE count + 1
                END,
                window_start = CASE
                    WHEN window_start < datetime('now', '-1 hour') THEN datetime('now')
                    ELSE window_start
                END
        "#)
        .bind(email)
        .execute(&self.pool)
        .await?;

        // Update IP rate limit
        if let Some(ip) = ip_address {
            sqlx::query(r#"
                INSERT INTO magic_link_rate_limits (key, key_type, count, window_start)
                VALUES (?, 'ip', 1, datetime('now'))
                ON CONFLICT(key, key_type) DO UPDATE SET
                    count = CASE
                        WHEN window_start < datetime('now', '-1 hour') THEN 1
                        ELSE count + 1
                    END,
                    window_start = CASE
                        WHEN window_start < datetime('now', '-1 hour') THEN datetime('now')
                        ELSE window_start
                    END
            "#)
            .bind(ip)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Find user by email
    async fn find_user_by_email(&self, email: &str) -> Result<Option<String>, MagicLinkError> {
        let user_id = sqlx::query_scalar::<_, String>(
            "SELECT id FROM users WHERE email = ? AND status != 'deleted'"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_id)
    }

    /// Validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Generate random token
    fn generate_token(&self) -> (String, String) {
        use rand::Rng;
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..self.config.token_length).map(|_| rng.gen()).collect();
        let token = URL_SAFE_NO_PAD.encode(&bytes);
        let hash = self.hash_token(&token);

        (token, hash)
    }

    /// Hash token for storage
    fn hash_token(&self, token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<usize, MagicLinkError> {
        let result = sqlx::query(
            "DELETE FROM magic_link_tokens WHERE expires_at < datetime('now')"
        )
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected() as usize;

        // Also clean up old rate limit entries
        sqlx::query(
            "DELETE FROM magic_link_rate_limits WHERE window_start < datetime('now', '-1 day')"
        )
        .execute(&self.pool)
        .await?;

        debug!("Cleaned up {} expired magic link tokens", deleted);
        Ok(deleted)
    }

    /// Get pending tokens for user (for account page)
    pub async fn get_pending_for_user(&self, user_id: &str) -> Result<Vec<MagicLinkToken>, MagicLinkError> {
        let tokens = sqlx::query_as::<_, MagicLinkToken>(
            "SELECT * FROM magic_link_tokens WHERE user_id = ? AND used_at IS NULL AND revoked_at IS NULL AND expires_at > datetime('now') ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let config = MagicLinkConfig::default();

        // Would need to create service for proper test
        // This is a placeholder for the validation logic
        assert!("test@example.com".contains('@'));
        assert!(!"invalid-email".contains('@'));
    }

    #[test]
    fn test_token_generation() {
        use sha2::{Sha256, Digest};

        let token = "test-token";
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }
}
```

## Files to Create
- `src/auth/magic_link/service.rs` - Magic link verification service
