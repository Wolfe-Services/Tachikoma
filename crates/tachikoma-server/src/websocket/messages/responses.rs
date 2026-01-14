//! Response message types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Acknowledgment response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl AckResponse {
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
        }
    }

    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
        }
    }
}

/// Error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Serialize) -> Self {
        self.details = Some(serde_json::to_value(details).unwrap_or_default());
        self
    }

    /// Authentication required.
    pub fn auth_required() -> Self {
        Self::new("auth_required", "Authentication required")
    }

    /// Invalid message format.
    pub fn invalid_message(details: &str) -> Self {
        Self::new("invalid_message", details)
    }

    /// Not found.
    pub fn not_found(resource: &str) -> Self {
        Self::new("not_found", format!("{} not found", resource))
    }

    /// Permission denied.
    pub fn permission_denied() -> Self {
        Self::new("permission_denied", "Permission denied")
    }
}

/// Authentication response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Subscription list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<String>,
}

/// Session info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfoResponse {
    pub session_id: Uuid,
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    pub subscriptions: Vec<String>,
    pub connected_at: String,
}

/// Pong response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongResponse {
    pub timestamp: String,
}

impl PongResponse {
    pub fn now() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}