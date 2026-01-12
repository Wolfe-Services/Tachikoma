# Spec 371: Device Code Types

## Overview
Define types for the OAuth 2.0 Device Authorization Grant flow, enabling authentication on input-constrained devices.

## Rust Implementation

### Device Code Types
```rust
// src/auth/device_code/types.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceCodeError {
    #[error("Device code expired")]
    Expired,

    #[error("Device code not found")]
    NotFound,

    #[error("Authorization pending")]
    AuthorizationPending,

    #[error("Access denied by user")]
    AccessDenied,

    #[error("Slow down - polling too fast")]
    SlowDown,

    #[error("Invalid user code")]
    InvalidUserCode,

    #[error("User code already used")]
    AlreadyUsed,

    #[error("Device code already authorized")]
    AlreadyAuthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Device authorization request (response to client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorizationResponse {
    /// Device verification code (for polling)
    pub device_code: String,
    /// User verification code (for display)
    pub user_code: String,
    /// URL for user to visit
    pub verification_uri: String,
    /// URL with code embedded (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    /// Lifetime of codes in seconds
    pub expires_in: i64,
    /// Minimum polling interval in seconds
    pub interval: i64,
}

/// Device code status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCodeStatus {
    /// Waiting for user authorization
    Pending,
    /// User authorized the request
    Authorized,
    /// User denied the request
    Denied,
    /// Code has expired
    Expired,
    /// Authorization was completed (token issued)
    Completed,
}

impl Default for DeviceCodeStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Stored device code
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct DeviceCode {
    pub id: String,
    pub device_code_hash: String,
    pub user_code: String,
    pub client_id: Option<String>,
    pub scope: Option<String>,
    pub status: String,  // DeviceCodeStatus as string
    pub user_id: Option<String>,  // Set when authorized
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_polled_at: Option<DateTime<Utc>>,
    pub poll_count: i32,
}

impl DeviceCode {
    pub fn status(&self) -> DeviceCodeStatus {
        match self.status.as_str() {
            "pending" => DeviceCodeStatus::Pending,
            "authorized" => DeviceCodeStatus::Authorized,
            "denied" => DeviceCodeStatus::Denied,
            "expired" => DeviceCodeStatus::Expired,
            "completed" => DeviceCodeStatus::Completed,
            _ => DeviceCodeStatus::Pending,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_pending(&self) -> bool {
        self.status() == DeviceCodeStatus::Pending && !self.is_expired()
    }

    pub fn is_authorized(&self) -> bool {
        self.status() == DeviceCodeStatus::Authorized
    }

    pub fn can_poll(&self, min_interval: Duration) -> bool {
        match self.last_polled_at {
            Some(last) => Utc::now() >= last + min_interval,
            None => true,
        }
    }
}

/// Device code configuration
#[derive(Debug, Clone)]
pub struct DeviceCodeConfig {
    /// Lifetime of device codes
    pub code_lifetime: Duration,
    /// Minimum polling interval
    pub poll_interval: Duration,
    /// User code length
    pub user_code_length: usize,
    /// User code format (e.g., "XXXX-XXXX")
    pub user_code_format: UserCodeFormat,
    /// Base verification URI
    pub verification_uri: String,
    /// Maximum poll attempts
    pub max_poll_attempts: i32,
}

impl Default for DeviceCodeConfig {
    fn default() -> Self {
        Self {
            code_lifetime: Duration::minutes(15),
            poll_interval: Duration::seconds(5),
            user_code_length: 8,
            user_code_format: UserCodeFormat::AlphanumericWithDash,
            verification_uri: "/device".to_string(),
            max_poll_attempts: 180,  // 15 minutes at 5-second intervals
        }
    }
}

/// User code format
#[derive(Debug, Clone, Copy)]
pub enum UserCodeFormat {
    /// XXXX-XXXX (8 chars with dash)
    AlphanumericWithDash,
    /// XXXXXXXX (8 chars no separator)
    Alphanumeric,
    /// 123-456 (numeric with dash)
    NumericWithDash,
    /// 123456 (numeric no separator)
    Numeric,
}

impl UserCodeFormat {
    pub fn charset(&self) -> &'static str {
        match self {
            Self::AlphanumericWithDash | Self::Alphanumeric => "BCDFGHJKLMNPQRSTVWXYZ23456789",
            Self::NumericWithDash | Self::Numeric => "0123456789",
        }
    }

    pub fn format(&self, code: &str) -> String {
        match self {
            Self::AlphanumericWithDash | Self::NumericWithDash => {
                if code.len() >= 8 {
                    format!("{}-{}", &code[..4], &code[4..8])
                } else if code.len() >= 6 {
                    format!("{}-{}", &code[..3], &code[3..6])
                } else {
                    code.to_string()
                }
            }
            Self::Alphanumeric | Self::Numeric => code.to_string(),
        }
    }

    pub fn normalize(&self, code: &str) -> String {
        code.chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect::<String>()
            .to_uppercase()
    }
}

/// Token request for device authorization
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceTokenRequest {
    pub grant_type: String,  // Should be "urn:ietf:params:oauth:grant-type:device_code"
    pub device_code: String,
    pub client_id: Option<String>,
}

/// Token response
#[derive(Debug, Clone, Serialize)]
pub struct DeviceTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Error response (OAuth 2.0 format)
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

impl DeviceCodeErrorResponse {
    pub fn authorization_pending() -> Self {
        Self {
            error: "authorization_pending".to_string(),
            error_description: Some("The authorization request is still pending".to_string()),
        }
    }

    pub fn slow_down() -> Self {
        Self {
            error: "slow_down".to_string(),
            error_description: Some("Polling too frequently, please slow down".to_string()),
        }
    }

    pub fn access_denied() -> Self {
        Self {
            error: "access_denied".to_string(),
            error_description: Some("The user denied the authorization request".to_string()),
        }
    }

    pub fn expired_token() -> Self {
        Self {
            error: "expired_token".to_string(),
            error_description: Some("The device code has expired".to_string()),
        }
    }

    pub fn invalid_grant() -> Self {
        Self {
            error: "invalid_grant".to_string(),
            error_description: Some("The device code is invalid or has already been used".to_string()),
        }
    }
}

/// Device authorization page data
#[derive(Debug, Clone, Serialize)]
pub struct DeviceAuthorizationPage {
    pub user_code: String,
    pub client_name: Option<String>,
    pub scope: Option<String>,
    pub scope_descriptions: Vec<String>,
    pub expires_in_minutes: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_code_format() {
        let format = UserCodeFormat::AlphanumericWithDash;

        assert_eq!(format.format("ABCD1234"), "ABCD-1234");
        assert_eq!(format.normalize("ABCD-1234"), "ABCD1234");
        assert_eq!(format.normalize("abcd 1234"), "ABCD1234");
    }

    #[test]
    fn test_device_code_status() {
        let code = DeviceCode {
            id: "test".to_string(),
            device_code_hash: "hash".to_string(),
            user_code: "ABCD-1234".to_string(),
            client_id: None,
            scope: None,
            status: "pending".to_string(),
            user_id: None,
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(15),
            authorized_at: None,
            completed_at: None,
            last_polled_at: None,
            poll_count: 0,
        };

        assert!(code.is_pending());
        assert!(!code.is_expired());
        assert!(!code.is_authorized());
    }
}
```

## Files to Create
- `src/auth/device_code/types.rs` - Device code types
- `src/auth/device_code/mod.rs` - Module exports
