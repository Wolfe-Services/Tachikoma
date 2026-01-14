//! Authorization audit logging.

use super::types::{Action, Resource};
use crate::middleware::auth::types::AuthUser;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

/// Authorization audit event.
#[derive(Debug, Serialize)]
pub struct AuthzAuditEvent {
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub user_email: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub granted: bool,
    pub reason: Option<String>,
}

impl AuthzAuditEvent {
    pub fn new(
        user: &AuthUser,
        action: Action,
        resource: Resource,
        resource_id: Option<Uuid>,
        granted: bool,
        reason: Option<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            user_id: user.id,
            user_email: user.email.clone(),
            action: format!("{:?}", action),
            resource: format!("{:?}", resource),
            resource_id: resource_id.map(|id| id.to_string()),
            granted,
            reason,
        }
    }

    pub fn log(&self) {
        if self.granted {
            info!(
                event = "authz_granted",
                user_id = %self.user_id,
                action = %self.action,
                resource = %self.resource,
                resource_id = ?self.resource_id,
                "Authorization granted"
            );
        } else {
            info!(
                event = "authz_denied",
                user_id = %self.user_id,
                action = %self.action,
                resource = %self.resource,
                resource_id = ?self.resource_id,
                reason = ?self.reason,
                "Authorization denied"
            );
        }
    }
}

/// Log authorization decision.
pub fn log_authz(
    user: &AuthUser,
    action: Action,
    resource: Resource,
    resource_id: Option<Uuid>,
    granted: bool,
    reason: Option<&str>,
) {
    let event = AuthzAuditEvent::new(
        user,
        action,
        resource,
        resource_id,
        granted,
        reason.map(String::from),
    );
    event.log();
}