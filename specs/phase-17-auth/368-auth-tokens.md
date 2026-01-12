# Spec 368: Authentication Tokens

## Overview
Implement token generation and validation including access tokens, refresh tokens, and various token types used in authentication flows.

## Rust Implementation

### Token Types
```rust
// src/auth/tokens.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use rand::Rng;

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token expired")]
    Expired,

    #[error("Token invalid")]
    Invalid,

    #[error("Token revoked")]
    Revoked,

    #[error("Token type mismatch")]
    TypeMismatch,

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Decoding error: {0}")]
    Decoding(String),
}

/// Token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
    MfaChallenge,
    PasswordReset,
    EmailVerification,
    MagicLink,
    DeviceCode,
    ApiKey,
}

/// Token configuration
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub access_token_lifetime: Duration,
    pub refresh_token_lifetime: Duration,
    pub mfa_token_lifetime: Duration,
    pub password_reset_lifetime: Duration,
    pub email_verification_lifetime: Duration,
    pub magic_link_lifetime: Duration,
    pub device_code_lifetime: Duration,
    pub token_length: usize,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            access_token_lifetime: Duration::hours(1),
            refresh_token_lifetime: Duration::days(30),
            mfa_token_lifetime: Duration::minutes(5),
            password_reset_lifetime: Duration::hours(1),
            email_verification_lifetime: Duration::days(7),
            magic_link_lifetime: Duration::minutes(15),
            device_code_lifetime: Duration::minutes(15),
            token_length: 32,
        }
    }
}

/// Opaque token (random bytes)
#[derive(Debug, Clone)]
pub struct OpaqueToken {
    pub value: String,
    pub hash: String,
    pub token_type: TokenType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl OpaqueToken {
    pub fn generate(token_type: TokenType, lifetime: Duration, length: usize) -> Self {
        let value = Self::generate_random_string(length);
        let hash = Self::hash_token(&value);
        let now = Utc::now();

        Self {
            value,
            hash,
            token_type,
            expires_at: now + lifetime,
            created_at: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn verify(&self, token: &str) -> Result<(), TokenError> {
        if self.is_expired() {
            return Err(TokenError::Expired);
        }

        let hash = Self::hash_token(token);
        if hash != self.hash {
            return Err(TokenError::Invalid);
        }

        Ok(())
    }

    fn generate_random_string(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..length).map(|_| rng.gen()).collect();
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes)
    }

    fn hash_token(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Token generator
pub struct TokenGenerator {
    config: TokenConfig,
}

impl TokenGenerator {
    pub fn new(config: TokenConfig) -> Self {
        Self { config }
    }

    pub fn generate_access_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::Access,
            self.config.access_token_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_refresh_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::Refresh,
            self.config.refresh_token_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_mfa_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::MfaChallenge,
            self.config.mfa_token_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_password_reset_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::PasswordReset,
            self.config.password_reset_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_email_verification_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::EmailVerification,
            self.config.email_verification_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_magic_link_token(&self) -> OpaqueToken {
        OpaqueToken::generate(
            TokenType::MagicLink,
            self.config.magic_link_lifetime,
            self.config.token_length,
        )
    }

    pub fn generate_device_code(&self) -> DeviceCodePair {
        DeviceCodePair::generate(self.config.device_code_lifetime)
    }

    /// Generate a short numeric code (for user input)
    pub fn generate_user_code(length: usize) -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "BCDFGHJKLMNPQRSTVWXYZ23456789"  // Avoid confusing chars
            .chars()
            .collect();

        (0..length)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }
}

/// Device code pair (for device authorization flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodePair {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: i64,
    pub interval: i64,
}

impl DeviceCodePair {
    pub fn generate(lifetime: Duration) -> Self {
        let device_code = OpaqueToken::generate_random_string(32);
        let user_code = TokenGenerator::generate_user_code(8);

        Self {
            device_code,
            user_code: format!("{}-{}", &user_code[..4], &user_code[4..]),
            verification_uri: "/device".to_string(),
            verification_uri_complete: None,
            expires_in: lifetime.num_seconds(),
            interval: 5,
        }
    }

    pub fn set_verification_uri(&mut self, base_uri: &str) {
        self.verification_uri = format!("{}/device", base_uri);
        self.verification_uri_complete = Some(format!(
            "{}/device?user_code={}",
            base_uri, self.user_code
        ));
    }
}

/// Stored token record (in database)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredToken {
    pub id: String,
    pub token_hash: String,
    pub token_type: String,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl StoredToken {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() &&
        self.revoked_at.is_none() &&
        Utc::now() < self.expires_at
    }
}

/// Token store
pub struct TokenStore {
    pool: sqlx::sqlite::SqlitePool,
}

impl TokenStore {
    pub fn new(pool: sqlx::sqlite::SqlitePool) -> Self {
        Self { pool }
    }

    /// Store a token
    pub async fn store(
        &self,
        token: &OpaqueToken,
        user_id: Option<&str>,
        email: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), TokenError> {
        let id = Uuid::new_v4().to_string();
        let token_type = format!("{:?}", token.token_type).to_lowercase();
        let metadata_str = metadata.map(|m| m.to_string());

        sqlx::query(r#"
            INSERT INTO tokens (id, token_hash, token_type, user_id, email, metadata, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&token.hash)
        .bind(&token_type)
        .bind(user_id)
        .bind(email)
        .bind(&metadata_str)
        .bind(token.created_at)
        .bind(token.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| TokenError::Encoding(e.to_string()))?;

        Ok(())
    }

    /// Validate and retrieve token
    pub async fn validate(
        &self,
        token: &str,
        expected_type: TokenType,
    ) -> Result<StoredToken, TokenError> {
        let hash = OpaqueToken::hash_token(token);
        let type_str = format!("{:?}", expected_type).to_lowercase();

        let stored = sqlx::query_as::<_, StoredToken>(
            "SELECT * FROM tokens WHERE token_hash = ? AND token_type = ?"
        )
        .bind(&hash)
        .bind(&type_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TokenError::Decoding(e.to_string()))?
        .ok_or(TokenError::Invalid)?;

        if stored.revoked_at.is_some() {
            return Err(TokenError::Revoked);
        }

        if Utc::now() > stored.expires_at {
            return Err(TokenError::Expired);
        }

        if stored.used_at.is_some() {
            return Err(TokenError::Invalid);  // Single-use token already used
        }

        Ok(stored)
    }

    /// Mark token as used (for single-use tokens)
    pub async fn mark_used(&self, token_hash: &str) -> Result<(), TokenError> {
        sqlx::query("UPDATE tokens SET used_at = datetime('now') WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| TokenError::Encoding(e.to_string()))?;

        Ok(())
    }

    /// Revoke token
    pub async fn revoke(&self, token_hash: &str) -> Result<(), TokenError> {
        sqlx::query("UPDATE tokens SET revoked_at = datetime('now') WHERE token_hash = ?")
            .bind(token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| TokenError::Encoding(e.to_string()))?;

        Ok(())
    }

    /// Revoke all tokens for user
    pub async fn revoke_all_for_user(
        &self,
        user_id: &str,
        token_type: Option<TokenType>,
    ) -> Result<usize, TokenError> {
        let result = if let Some(t) = token_type {
            let type_str = format!("{:?}", t).to_lowercase();
            sqlx::query(
                "UPDATE tokens SET revoked_at = datetime('now') WHERE user_id = ? AND token_type = ? AND revoked_at IS NULL"
            )
            .bind(user_id)
            .bind(&type_str)
            .execute(&self.pool)
            .await
        } else {
            sqlx::query(
                "UPDATE tokens SET revoked_at = datetime('now') WHERE user_id = ? AND revoked_at IS NULL"
            )
            .bind(user_id)
            .execute(&self.pool)
            .await
        };

        result
            .map(|r| r.rows_affected() as usize)
            .map_err(|e| TokenError::Encoding(e.to_string()))
    }

    /// Cleanup expired tokens
    pub async fn cleanup(&self) -> Result<usize, TokenError> {
        let result = sqlx::query(
            "DELETE FROM tokens WHERE expires_at < datetime('now')"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TokenError::Encoding(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }
}

/// Token database schema
pub fn token_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS tokens (
    id TEXT PRIMARY KEY NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_type TEXT NOT NULL,
    user_id TEXT,
    email TEXT,
    metadata TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT NOT NULL,
    used_at TEXT,
    revoked_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_tokens_hash ON tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_tokens_user ON tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_tokens_type ON tokens(token_type);
CREATE INDEX IF NOT EXISTS idx_tokens_expires ON tokens(expires_at);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opaque_token_generation() {
        let token = OpaqueToken::generate(
            TokenType::Access,
            Duration::hours(1),
            32
        );

        assert!(!token.value.is_empty());
        assert!(!token.hash.is_empty());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_token_verification() {
        let token = OpaqueToken::generate(
            TokenType::Access,
            Duration::hours(1),
            32
        );

        assert!(token.verify(&token.value).is_ok());
        assert!(token.verify("wrong_token").is_err());
    }

    #[test]
    fn test_user_code_generation() {
        let code = TokenGenerator::generate_user_code(8);
        assert_eq!(code.len(), 8);
        // Should not contain confusing characters
        assert!(!code.contains('0'));
        assert!(!code.contains('O'));
        assert!(!code.contains('I'));
        assert!(!code.contains('1'));
    }
}
```

## Files to Create
- `src/auth/tokens.rs` - Token management
