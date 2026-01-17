//! Security audit event capture.

use crate::{
    AuditAction, AuditActor, AuditCapture, AuditCategory, AuditEvent,
    AuditOutcome, AuditTarget,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use tachikoma_audit_types::AuditSeverity;

/// Security event recorder.
pub struct SecurityEventRecorder {
    capture: AuditCapture,
}

impl SecurityEventRecorder {
    /// Create a new security event recorder.
    pub fn new(capture: AuditCapture) -> Self {
        Self { capture }
    }

    /// Record a successful login.
    pub fn login_success(&self, user_id: &str, username: &str, method: AuthMethod, context: LoginContext) {
        let event = AuditEvent::builder(AuditCategory::Authentication, AuditAction::Login)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: Some(username.to_string()),
                session_id: context.session_id.clone(),
            })
            .severity(AuditSeverity::Low)
            .outcome(AuditOutcome::Success)
            .metadata("method", format!("{:?}", method))
            .metadata("user_id", user_id)
            .ip_address(context.ip_address.map(|ip| ip.to_string()).unwrap_or_default())
            .user_agent(context.user_agent.unwrap_or_default())
            .build();
        self.capture.record(event);
    }

    /// Record a failed login attempt.
    pub fn login_failed(&self, username: &str, reason: &str, method: AuthMethod, context: LoginContext) {
        let event = AuditEvent::builder(AuditCategory::Authentication, AuditAction::LoginFailed)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: Some(username.to_string()),
                session_id: None,
            })
            .severity(AuditSeverity::High)
            .outcome(AuditOutcome::Failure {
                reason: reason.to_string(),
            })
            .metadata("method", format!("{:?}", method))
            .metadata("attempted_username", username)
            .ip_address(context.ip_address.map(|ip| ip.to_string()).unwrap_or_default())
            .user_agent(context.user_agent.unwrap_or_default())
            .build();
        self.capture.record(event);
    }

    /// Record a logout.
    pub fn logout(&self, user_id: &str, username: &str, reason: LogoutReason) {
        let event = AuditEvent::builder(AuditCategory::Authentication, AuditAction::Logout)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: Some(username.to_string()),
                session_id: None,
            })
            .severity(AuditSeverity::Info)
            .metadata("reason", format!("{:?}", reason))
            .metadata("user_id", user_id)
            .build();
        self.capture.record(event);
    }

    /// Record an authorization decision.
    pub fn authorization_check(
        &self,
        user_id: &str,
        resource: &str,
        permission: &str,
        granted: bool,
        context: Option<&str>,
    ) {
        let action = if granted {
            AuditAction::AccessGranted
        } else {
            AuditAction::AccessDenied
        };

        let severity = if granted {
            AuditSeverity::Info
        } else {
            AuditSeverity::Medium
        };

        let outcome = if granted {
            AuditOutcome::Success
        } else {
            AuditOutcome::Denied {
                reason: format!("Permission '{}' denied on '{}'", permission, resource),
            }
        };

        let mut builder = AuditEvent::builder(AuditCategory::Authorization, action)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: None,
                session_id: None,
            })
            .severity(severity)
            .outcome(outcome)
            .target(AuditTarget::new("resource", resource))
            .metadata("permission", permission)
            .metadata("user_id", user_id);

        if let Some(ctx) = context {
            builder = builder.metadata("context", ctx);
        }

        self.capture.record(builder.build());
    }

    /// Record a permission change.
    pub fn permission_changed(
        &self,
        changed_by: &str,
        target_user: &str,
        permission: &str,
        granted: bool,
    ) {
        let action = if granted {
            AuditAction::RoleAssigned
        } else {
            AuditAction::RoleRevoked
        };

        let event = AuditEvent::builder(AuditCategory::Authorization, action)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: Some(changed_by.to_string()),
                session_id: None,
            })
            .severity(AuditSeverity::Medium)
            .target(AuditTarget::new("user", target_user))
            .metadata("permission", permission)
            .metadata("granted", granted)
            .build();
        self.capture.record(event);
    }

    /// Record a security incident.
    pub fn security_incident(&self, incident: SecurityIncident) {
        let action = match incident.incident_type {
            IncidentType::SuspiciousActivity => AuditAction::SuspiciousActivity,
            IncidentType::PolicyViolation => AuditAction::SecurityViolation,
            IncidentType::IntrusionAttempt => AuditAction::IntrusionDetected,
            IncidentType::DataBreach => AuditAction::DataBreach,
            IncidentType::BruteForce => AuditAction::SuspiciousActivity,
            IncidentType::PrivilegeEscalation => AuditAction::SecurityViolation,
        };

        let severity = match incident.severity {
            IncidentSeverity::Low => AuditSeverity::Medium,
            IncidentSeverity::Medium => AuditSeverity::High,
            IncidentSeverity::High => AuditSeverity::Critical,
            IncidentSeverity::Critical => AuditSeverity::Critical,
        };

        let mut builder = AuditEvent::builder(AuditCategory::Security, action)
            .actor(incident.actor.unwrap_or(AuditActor::Unknown))
            .severity(severity)
            .metadata("incident_type", format!("{:?}", incident.incident_type))
            .metadata("description", &incident.description);

        if let Some(target) = incident.target {
            builder = builder.target(target);
        }

        if let Some(ip) = incident.source_ip {
            builder = builder.ip_address(ip.to_string());
        }

        for (key, value) in incident.details {
            builder = builder.metadata(key, value);
        }

        self.capture.record(builder.build());
    }

    /// Record a token event.
    pub fn token_event(&self, user_id: &str, token_type: &str, action: TokenAction) {
        let audit_action = match action {
            TokenAction::Created => AuditAction::TokenRefresh,
            TokenAction::Refreshed => AuditAction::TokenRefresh,
            TokenAction::Revoked => AuditAction::TokenRevoked,
            TokenAction::Expired => AuditAction::SessionExpired,
        };

        let severity = match action {
            TokenAction::Revoked => AuditSeverity::Medium,
            _ => AuditSeverity::Low,
        };

        let event = AuditEvent::builder(AuditCategory::Authentication, audit_action)
            .actor(AuditActor::User {
                user_id: tachikoma_common_core::UserId::new(),
                username: None,
                session_id: None,
            })
            .severity(severity)
            .metadata("user_id", user_id)
            .metadata("token_type", token_type)
            .metadata("action", format!("{:?}", action))
            .build();
        self.capture.record(event);
    }
}

/// Authentication method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Password,
    ApiKey,
    OAuth,
    Saml,
    Certificate,
    Mfa,
}

/// Logout reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogoutReason {
    UserInitiated,
    SessionTimeout,
    AdminForced,
    SecurityPolicy,
    TokenRevoked,
}

/// Login context information.
#[derive(Debug, Clone, Default)]
pub struct LoginContext {
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub mfa_used: bool,
}

/// Token action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenAction {
    Created,
    Refreshed,
    Revoked,
    Expired,
}

/// Security incident information.
#[derive(Debug, Clone)]
pub struct SecurityIncident {
    pub incident_type: IncidentType,
    pub severity: IncidentSeverity,
    pub description: String,
    pub actor: Option<AuditActor>,
    pub target: Option<AuditTarget>,
    pub source_ip: Option<IpAddr>,
    pub details: HashMap<String, String>,
}

/// Type of security incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    SuspiciousActivity,
    PolicyViolation,
    IntrusionAttempt,
    DataBreach,
    BruteForce,
    PrivilegeEscalation,
}

/// Incident severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}