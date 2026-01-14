# 323 - Authorization Middleware

**Phase:** 15 - Server
**Spec ID:** 323
**Status:** Planned
**Dependencies:** 322-auth-middleware
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement authorization middleware for role-based access control (RBAC) and resource-level permissions checking.

---

## Acceptance Criteria

- [x] Role-based access control
- [x] Permission checking middleware
- [x] Resource ownership validation
- [x] Policy-based authorization
- [x] Action-resource matrices
- [x] Dynamic permission evaluation
- [x] Authorization audit logging

---

## Implementation Details

### 1. Authorization Types (crates/tachikoma-server/src/middleware/authz/types.rs)

```rust
//! Authorization types and policies.

use std::collections::{HashMap, HashSet};

/// Actions that can be performed on resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Admin,
}

/// Resource types in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Mission,
    Spec,
    ForgeSession,
    Config,
    User,
    ApiKey,
    Metrics,
}

/// Permission definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    pub action: Action,
    pub resource: Resource,
}

impl Permission {
    pub fn new(action: Action, resource: Resource) -> Self {
        Self { action, resource }
    }
}

/// Role definitions with associated permissions.
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

impl Role {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            permissions: HashSet::new(),
        }
    }

    pub fn with_permission(mut self, action: Action, resource: Resource) -> Self {
        self.permissions.insert(Permission::new(action, resource));
        self
    }

    pub fn with_full_access(mut self, resource: Resource) -> Self {
        for action in [Action::Create, Action::Read, Action::Update, Action::Delete] {
            self.permissions.insert(Permission::new(action, resource));
        }
        self
    }

    pub fn has_permission(&self, action: Action, resource: Resource) -> bool {
        self.permissions.contains(&Permission::new(action, resource))
    }
}

/// Role registry with predefined roles.
pub struct RoleRegistry {
    roles: HashMap<String, Role>,
}

impl RoleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            roles: HashMap::new(),
        };

        // Define standard roles
        registry.register(Self::admin_role());
        registry.register(Self::user_role());
        registry.register(Self::viewer_role());

        registry
    }

    fn admin_role() -> Role {
        Role::new("admin")
            .with_full_access(Resource::Mission)
            .with_full_access(Resource::Spec)
            .with_full_access(Resource::ForgeSession)
            .with_full_access(Resource::Config)
            .with_full_access(Resource::User)
            .with_full_access(Resource::ApiKey)
            .with_permission(Action::Admin, Resource::Metrics)
    }

    fn user_role() -> Role {
        Role::new("user")
            .with_full_access(Resource::Mission)
            .with_full_access(Resource::Spec)
            .with_full_access(Resource::ForgeSession)
            .with_permission(Action::Read, Resource::Config)
            .with_permission(Action::Read, Resource::Metrics)
    }

    fn viewer_role() -> Role {
        Role::new("viewer")
            .with_permission(Action::Read, Resource::Mission)
            .with_permission(Action::Read, Resource::Spec)
            .with_permission(Action::Read, Resource::Config)
    }

    pub fn register(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn get(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    pub fn check_permission(&self, roles: &[String], action: Action, resource: Resource) -> bool {
        roles.iter().any(|role_name| {
            self.roles
                .get(role_name)
                .map(|role| role.has_permission(action, resource))
                .unwrap_or(false)
        })
    }
}

impl Default for RoleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Authorization Layer (crates/tachikoma-server/src/middleware/authz/layer.rs)

```rust
//! Authorization middleware layer.

use super::types::{Action, Resource, RoleRegistry};
use crate::{
    error::ApiError,
    middleware::auth::types::AuthUser,
};
use axum::{
    body::Body,
    http::Request,
    response::Response,
};
use std::sync::Arc;
use tower::{Layer, Service};
use tracing::warn;

/// Authorization layer configuration.
#[derive(Clone)]
pub struct AuthzLayer {
    registry: Arc<RoleRegistry>,
    required_action: Action,
    required_resource: Resource,
}

impl AuthzLayer {
    pub fn new(registry: Arc<RoleRegistry>, action: Action, resource: Resource) -> Self {
        Self {
            registry,
            required_action: action,
            required_resource: resource,
        }
    }

    /// Create layer requiring read permission.
    pub fn read(registry: Arc<RoleRegistry>, resource: Resource) -> Self {
        Self::new(registry, Action::Read, resource)
    }

    /// Create layer requiring create permission.
    pub fn create(registry: Arc<RoleRegistry>, resource: Resource) -> Self {
        Self::new(registry, Action::Create, resource)
    }

    /// Create layer requiring update permission.
    pub fn update(registry: Arc<RoleRegistry>, resource: Resource) -> Self {
        Self::new(registry, Action::Update, resource)
    }

    /// Create layer requiring delete permission.
    pub fn delete(registry: Arc<RoleRegistry>, resource: Resource) -> Self {
        Self::new(registry, Action::Delete, resource)
    }
}

impl<S> Layer<S> for AuthzLayer {
    type Service = AuthzMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthzMiddleware {
            inner,
            registry: self.registry.clone(),
            required_action: self.required_action,
            required_resource: self.required_resource,
        }
    }
}

#[derive(Clone)]
pub struct AuthzMiddleware<S> {
    inner: S,
    registry: Arc<RoleRegistry>,
    required_action: Action,
    required_resource: Resource,
}

impl<S> Service<Request<Body>> for AuthzMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let registry = self.registry.clone();
        let action = self.required_action;
        let resource = self.required_resource;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Get authenticated user from request extensions
            let auth_user = req
                .extensions()
                .get::<AuthUser>()
                .ok_or_else(|| {
                    warn!("Authorization check without authentication");
                    ApiError::Unauthorized
                })?;

            // Check if user has required permission
            if !registry.check_permission(&auth_user.roles, action, resource) {
                warn!(
                    user_id = %auth_user.id,
                    action = ?action,
                    resource = ?resource,
                    "Authorization denied"
                );
                return Err(ApiError::InsufficientPermissions.into());
            }

            inner.call(req).await
        })
    }
}
```

### 3. Resource Authorization (crates/tachikoma-server/src/middleware/authz/resource.rs)

```rust
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
```

### 4. Authorization Audit (crates/tachikoma-server/src/middleware/authz/audit.rs)

```rust
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
```

---

## Testing Requirements

1. Role permissions checked correctly
2. Unknown roles denied by default
3. Admin bypasses most checks
4. Resource ownership validated
5. Audit events logged correctly
6. Custom policies evaluated
7. Hierarchical roles work

---

## Related Specs

- Depends on: [322-auth-middleware.md](322-auth-middleware.md)
- Next: [324-rate-limit-mw.md](324-rate-limit-mw.md)
- Used by: Protected handlers
