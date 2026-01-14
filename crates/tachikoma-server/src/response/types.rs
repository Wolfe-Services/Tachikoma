//! Standard API response types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API response envelope.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// Response data (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error information (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// Response metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Error information in responses.
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    /// Error code (machine-readable).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Field-specific validation errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, Vec<String>>>,
}

/// Response metadata.
#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    /// Request ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Response timestamp.
    pub timestamp: String,
    /// API version used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response.
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ResponseMeta::now()),
        }
    }

    /// Create a successful response with metadata.
    pub fn success_with_meta(data: T, meta: ResponseMeta) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(meta),
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: code.into(),
                message: message.into(),
                details: None,
                fields: None,
            }),
            meta: Some(ResponseMeta::now()),
        }
    }

    /// Create an error response with field errors.
    pub fn validation_error(fields: HashMap<String, Vec<String>>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: "validation_error".into(),
                message: "Validation failed".into(),
                details: None,
                fields: Some(fields),
            }),
            meta: Some(ResponseMeta::now()),
        }
    }
}

impl ResponseMeta {
    /// Create metadata with current timestamp.
    pub fn now() -> Self {
        Self {
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            api_version: None,
        }
    }

    /// Add request ID.
    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    /// Add API version.
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }
}