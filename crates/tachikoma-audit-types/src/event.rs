//! Core audit event type.

use crate::{
    AuditAction, AuditActor, AuditCategory, AuditEventId, AuditSeverity,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub id: AuditEventId,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Event category.
    pub category: AuditCategory,
    /// Specific action.
    pub action: AuditAction,
    /// Event severity.
    pub severity: AuditSeverity,
    /// Who initiated the event.
    pub actor: AuditActor,
    /// Optional target resource identifier.
    pub target: Option<AuditTarget>,
    /// Event outcome.
    pub outcome: AuditOutcome,
    /// Additional context data.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Correlation ID for related events.
    pub correlation_id: Option<String>,
    /// IP address if applicable.
    pub ip_address: Option<String>,
    /// User agent if applicable.
    pub user_agent: Option<String>,
}

/// Target of an audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTarget {
    /// Type of the target resource.
    pub resource_type: String,
    /// Resource identifier.
    pub resource_id: String,
    /// Optional resource name.
    pub resource_name: Option<String>,
}

impl AuditTarget {
    /// Create a new target.
    pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            resource_name: None,
        }
    }

    /// Add a resource name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.resource_name = Some(name.into());
        self
    }
}

/// Outcome of an audited action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Action succeeded.
    Success,
    /// Action failed.
    Failure { reason: String },
    /// Action was denied.
    Denied { reason: String },
    /// Action is pending.
    Pending,
    /// Unknown outcome.
    Unknown,
}

impl AuditOutcome {
    /// Check if the outcome is successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if the outcome is a failure.
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure { .. } | Self::Denied { .. })
    }
}

impl Default for AuditOutcome {
    fn default() -> Self {
        Self::Success
    }
}

impl AuditEvent {
    /// Create a new event builder.
    pub fn builder(category: AuditCategory, action: AuditAction) -> AuditEventBuilder {
        AuditEventBuilder::new(category, action)
    }
}

/// Builder for constructing audit events.
#[derive(Debug)]
pub struct AuditEventBuilder {
    category: AuditCategory,
    action: AuditAction,
    severity: Option<AuditSeverity>,
    actor: Option<AuditActor>,
    target: Option<AuditTarget>,
    outcome: AuditOutcome,
    metadata: HashMap<String, serde_json::Value>,
    correlation_id: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl AuditEventBuilder {
    /// Create a new builder.
    pub fn new(category: AuditCategory, action: AuditAction) -> Self {
        Self {
            category,
            action,
            severity: None,
            actor: None,
            target: None,
            outcome: AuditOutcome::Success,
            metadata: HashMap::new(),
            correlation_id: None,
            ip_address: None,
            user_agent: None,
        }
    }

    /// Set the severity (defaults to action's default severity).
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set the actor.
    pub fn actor(mut self, actor: AuditActor) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the target.
    pub fn target(mut self, target: AuditTarget) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the outcome.
    pub fn outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(json) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json);
        }
        self
    }

    /// Set correlation ID.
    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set IP address.
    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent.
    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Build the event.
    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: AuditEventId::new(),
            timestamp: Utc::now(),
            category: self.category,
            severity: self.severity.unwrap_or_else(|| self.action.default_severity()),
            action: self.action,
            actor: self.actor.unwrap_or(AuditActor::Unknown),
            target: self.target,
            outcome: self.outcome,
            metadata: self.metadata,
            correlation_id: self.correlation_id,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
        }
    }
}