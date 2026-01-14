//! Resource-level authorization.

use crate::{error::ApiError, middleware::auth::types::AuthUser};
use async_trait::async_trait;
use uuid::Uuid;

/// Trait for checking resource ownership.
#[async_trait]
pub trait ResourceOwner: Send + Sync {
    /// Check if user owns or has access to resource.
    async fn check_access(&self, user: &AuthUser, resource_id: &Uuid) -> Result<bool, ApiError>;
}

/// Policy for resource access.
pub enum AccessPolicy {
    /// Owner only.
    OwnerOnly,
    /// Owner or admin.
    OwnerOrAdmin,
    /// Any authenticated user.
    Authenticated,
    /// Public (no auth required).
    Public,
    /// Custom check function.
    Custom(Box<dyn Fn(&AuthUser, &Uuid) -> bool + Send + Sync>),
}

impl AccessPolicy {
    pub fn check(&self, user: &AuthUser, resource_owner_id: &Uuid) -> bool {
        match self {
            Self::OwnerOnly => user.id == *resource_owner_id,
            Self::OwnerOrAdmin => user.id == *resource_owner_id || user.is_admin(),
            Self::Authenticated => true,
            Self::Public => true,
            Self::Custom(f) => f(user, resource_owner_id),
        }
    }
}

/// Macro for creating authorization checks in handlers.
#[macro_export]
macro_rules! authorize_resource {
    ($auth:expr, $owner_id:expr, $policy:expr) => {
        if !$policy.check(&$auth.0, &$owner_id) {
            return Err($crate::error::ApiError::ResourceAccessDenied(
                "You do not have access to this resource".into()
            ));
        }
    };
}

/// Helper function to check resource access.
pub fn check_resource_access(
    user: &AuthUser,
    owner_id: &Uuid,
    policy: &AccessPolicy,
) -> Result<(), ApiError> {
    if policy.check(user, owner_id) {
        Ok(())
    } else {
        Err(ApiError::ResourceAccessDenied(
            "You do not have access to this resource".into()
        ))
    }
}