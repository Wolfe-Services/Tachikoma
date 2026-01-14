//! Audit event categories.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// High-level category for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Display, EnumIter, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AuditCategory {
    /// Authentication events (login, logout, token refresh).
    Authentication,
    /// Authorization events (permission checks, access denied).
    Authorization,
    /// User management events (create, update, delete users).
    UserManagement,
    /// Mission lifecycle events.
    Mission,
    /// Forge session events.
    Forge,
    /// Configuration changes.
    Configuration,
    /// File system operations.
    FileSystem,
    /// API interactions with LLM backends.
    ApiCall,
    /// System events (startup, shutdown, errors).
    System,
    /// Security events (suspicious activity, violations).
    Security,
    /// Data export/import events.
    DataTransfer,
}

impl AuditCategory {
    /// Get all categories.
    pub fn all() -> impl Iterator<Item = Self> {
        use strum::IntoEnumIterator;
        Self::iter()
    }

    /// Check if this category requires elevated retention.
    pub fn requires_extended_retention(&self) -> bool {
        matches!(
            self,
            Self::Authentication
                | Self::Authorization
                | Self::Security
                | Self::UserManagement
        )
    }
}