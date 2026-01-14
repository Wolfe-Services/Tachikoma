//! Audit event actions.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Specific actions that can be audited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AuditAction {
    // Authentication
    Login,
    Logout,
    LoginFailed,
    TokenRefresh,
    TokenRevoked,
    SessionExpired,

    // Authorization
    AccessGranted,
    AccessDenied,
    PermissionChanged,
    RoleAssigned,
    RoleRevoked,

    // User Management
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserEnabled,
    UserDisabled,
    PasswordChanged,
    PasswordReset,

    // Mission
    MissionCreated,
    MissionStarted,
    MissionPaused,
    MissionResumed,
    MissionCompleted,
    MissionFailed,
    MissionAborted,
    MissionRebooted,

    // Forge
    ForgeSessionCreated,
    ForgeSessionCompleted,
    ForgeDraftGenerated,
    ForgeCritiqueReceived,
    ForgeSynthesized,

    // Configuration
    ConfigCreated,
    ConfigUpdated,
    ConfigDeleted,
    ConfigExported,
    ConfigImported,

    // File System
    FileCreated,
    FileRead,
    FileUpdated,
    FileDeleted,
    FileMoved,
    FilePermissionChanged,

    // API Calls
    ApiRequestSent,
    ApiResponseReceived,
    ApiRateLimited,
    ApiError,

    // System
    SystemStartup,
    SystemShutdown,
    SystemError,
    BackupCreated,
    BackupRestored,

    // Security
    SuspiciousActivity,
    SecurityViolation,
    IntrusionDetected,
    DataBreach,

    // Data Transfer
    DataExported,
    DataImported,
    DataDeleted,
    DataArchived,

    // Custom action
    Custom(String),
}

impl AuditAction {
    /// Get the default severity for this action.
    pub fn default_severity(&self) -> super::AuditSeverity {
        use super::AuditSeverity;
        match self {
            // Critical
            Self::DataBreach | Self::IntrusionDetected | Self::SecurityViolation => {
                AuditSeverity::Critical
            }

            // High
            Self::LoginFailed
            | Self::AccessDenied
            | Self::SuspiciousActivity
            | Self::UserDeleted
            | Self::MissionFailed
            | Self::SystemError => AuditSeverity::High,

            // Medium
            Self::PasswordChanged
            | Self::PasswordReset
            | Self::PermissionChanged
            | Self::RoleAssigned
            | Self::RoleRevoked
            | Self::ConfigUpdated
            | Self::ConfigDeleted
            | Self::UserUpdated => AuditSeverity::Medium,

            // Low
            Self::Login
            | Self::Logout
            | Self::TokenRefresh
            | Self::UserCreated
            | Self::MissionCreated
            | Self::ConfigCreated => AuditSeverity::Low,

            // Info (default)
            _ => AuditSeverity::Info,
        }
    }
}