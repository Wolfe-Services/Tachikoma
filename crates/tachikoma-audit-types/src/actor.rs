//! Audit event actors.

use serde::{Deserialize, Serialize};
use tachikoma_common_core::UserId;

/// The entity that initiated an audit event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditActor {
    /// A human user.
    User {
        user_id: UserId,
        username: Option<String>,
        session_id: Option<String>,
    },
    /// The system itself (automated processes).
    System {
        component: String,
        process_id: Option<u32>,
    },
    /// An API client.
    ApiClient {
        client_id: String,
        client_name: Option<String>,
    },
    /// An LLM backend.
    Backend {
        backend_name: String,
        model: Option<String>,
    },
    /// Unknown actor (for legacy events).
    Unknown,
}

impl AuditActor {
    /// Create a user actor.
    pub fn user(user_id: UserId) -> Self {
        Self::User {
            user_id,
            username: None,
            session_id: None,
        }
    }

    /// Create a system actor.
    pub fn system(component: impl Into<String>) -> Self {
        Self::System {
            component: component.into(),
            process_id: std::process::id().into(),
        }
    }

    /// Create an API client actor.
    pub fn api_client(client_id: impl Into<String>) -> Self {
        Self::ApiClient {
            client_id: client_id.into(),
            client_name: None,
        }
    }

    /// Get a display identifier for this actor.
    pub fn identifier(&self) -> String {
        match self {
            Self::User { user_id, username, .. } => {
                username.clone().unwrap_or_else(|| user_id.to_string())
            }
            Self::System { component, .. } => format!("system:{}", component),
            Self::ApiClient { client_id, client_name, .. } => {
                client_name.clone().unwrap_or_else(|| client_id.clone())
            }
            Self::Backend { backend_name, .. } => format!("backend:{}", backend_name),
            Self::Unknown => "unknown".to_string(),
        }
    }
}