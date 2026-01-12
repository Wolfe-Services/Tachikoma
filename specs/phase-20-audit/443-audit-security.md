# 443 - Audit Security

**Phase:** 20 - Audit System
**Spec ID:** 443
**Status:** Planned
**Dependencies:** 431-audit-event-types, 433-audit-capture
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement security-focused audit event capture for authentication, authorization, and security incidents.

---

## Acceptance Criteria

- [ ] Authentication event capture
- [ ] Authorization decision logging
- [ ] Security incident recording
- [ ] Failed access attempt tracking
- [ ] Privilege escalation detection

---

## Implementation Details

### 1. Security Event Recorder (src/security_events.rs)

```rust
//! Security audit event capture.

use crate::{
    AuditAction, AuditActor, AuditCapture, AuditCategory, AuditEvent,
    AuditOutcome, AuditSeverity, AuditTarget,
};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

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
    pub details: std::collections::HashMap<String, String>,
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
```

### 2. Failed Login Tracker (src/login_tracker.rs)

```rust
//! Failed login attempt tracking.

use crate::security_events::SecurityEventRecorder;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Failed login tracker for detecting brute force attacks.
pub struct FailedLoginTracker {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    config: TrackerConfig,
    recorder: Arc<SecurityEventRecorder>,
}

/// Tracker configuration.
#[derive(Debug, Clone)]
pub struct TrackerConfig {
    /// Window for counting attempts.
    pub window: Duration,
    /// Maximum attempts before lockout.
    pub max_attempts: u32,
    /// Lockout duration.
    pub lockout_duration: Duration,
    /// Alert threshold.
    pub alert_threshold: u32,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_secs(300), // 5 minutes
            max_attempts: 5,
            lockout_duration: Duration::from_secs(900), // 15 minutes
            alert_threshold: 10,
        }
    }
}

impl FailedLoginTracker {
    /// Create a new tracker.
    pub fn new(recorder: Arc<SecurityEventRecorder>, config: TrackerConfig) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            config,
            recorder,
        }
    }

    /// Record a failed login attempt.
    pub fn record_failure(&self, identifier: &str, ip: Option<IpAddr>) -> LoginStatus {
        let mut attempts = self.attempts.lock();
        let now = Instant::now();

        // Clean old attempts
        let entry = attempts.entry(identifier.to_string()).or_insert_with(Vec::new);
        entry.retain(|t| now.duration_since(*t) < self.config.window);

        // Add new attempt
        entry.push(now);

        let count = entry.len() as u32;

        // Check thresholds
        if count >= self.config.alert_threshold {
            self.recorder.security_incident(crate::security_events::SecurityIncident {
                incident_type: crate::security_events::IncidentType::BruteForce,
                severity: crate::security_events::IncidentSeverity::High,
                description: format!("{} failed login attempts for '{}'", count, identifier),
                actor: None,
                target: None,
                source_ip: ip,
                details: {
                    let mut d = std::collections::HashMap::new();
                    d.insert("identifier".to_string(), identifier.to_string());
                    d.insert("attempt_count".to_string(), count.to_string());
                    d
                },
            });
        }

        if count >= self.config.max_attempts {
            LoginStatus::LockedOut {
                until: now + self.config.lockout_duration,
                attempts: count,
            }
        } else {
            LoginStatus::Allowed {
                remaining_attempts: self.config.max_attempts - count,
            }
        }
    }

    /// Check if an identifier is locked out.
    pub fn is_locked_out(&self, identifier: &str) -> bool {
        let attempts = self.attempts.lock();
        let now = Instant::now();

        if let Some(entry) = attempts.get(identifier) {
            let recent = entry.iter()
                .filter(|t| now.duration_since(**t) < self.config.window)
                .count();
            recent as u32 >= self.config.max_attempts
        } else {
            false
        }
    }

    /// Clear attempts for an identifier (e.g., after successful login).
    pub fn clear(&self, identifier: &str) {
        self.attempts.lock().remove(identifier);
    }
}

/// Login attempt status.
#[derive(Debug, Clone)]
pub enum LoginStatus {
    /// Login is allowed.
    Allowed { remaining_attempts: u32 },
    /// Account is locked out.
    LockedOut { until: Instant, attempts: u32 },
}
```

---

## Testing Requirements

1. Login events capture all context
2. Failed login tracking works correctly
3. Lockout triggers at threshold
4. Security incidents have correct severity
5. Authorization decisions are logged

---

## Related Specs

- Depends on: [431-audit-event-types.md](431-audit-event-types.md), [433-audit-capture.md](433-audit-capture.md)
- Next: [444-audit-compliance.md](444-audit-compliance.md)
