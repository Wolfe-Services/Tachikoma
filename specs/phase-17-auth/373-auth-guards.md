# Spec 373: Route Guards

## Phase
17 - Authentication/Authorization

## Spec ID
373

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 372: Auth Middleware
- Spec 374: Role-Based Access
- Spec 375: Permissions

## Estimated Context
~9%

---

## Objective

Implement route guards for protecting endpoints based on authentication status, roles, and permissions. Guards should be composable, allowing complex authorization rules to be expressed declaratively. The implementation should integrate seamlessly with the Axum web framework.

---

## Acceptance Criteria

- [ ] Create `Guard` trait for authorization checks
- [ ] Implement `RequireAuth` guard for authentication
- [ ] Implement `RequireRole` guard for role-based access
- [ ] Implement `RequirePermission` guard for permission-based access
- [ ] Implement `RequireAny` and `RequireAll` for composite guards
- [ ] Create guard extractors for Axum handlers
- [ ] Support custom guard implementations
- [ ] Provide clear error messages for authorization failures

---

## Implementation Details

### Guard Traits and Implementations

```rust
// src/auth/guards.rs

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::auth::{
    middleware::auth_error_response,
    types::*,
};

/// Trait for authorization guards
#[async_trait]
pub trait Guard: Send + Sync {
    /// Check if the guard allows access
    async fn check(&self, identity: &AuthIdentity) -> GuardResult;

    /// Get a description of the guard requirement
    fn description(&self) -> String;
}

/// Result of a guard check
#[derive(Debug, Clone)]
pub enum GuardResult {
    /// Access allowed
    Allow,
    /// Access denied with reason
    Deny(AuthError),
}

impl GuardResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, GuardResult::Allow)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, GuardResult::Deny(_))
    }
}

/// Guard that requires authentication
#[derive(Debug, Clone)]
pub struct AuthenticatedGuard;

#[async_trait]
impl Guard for AuthenticatedGuard {
    async fn check(&self, _identity: &AuthIdentity) -> GuardResult {
        // If we have an identity, user is authenticated
        GuardResult::Allow
    }

    fn description(&self) -> String {
        "Requires authentication".to_string()
    }
}

/// Guard that requires a specific role
#[derive(Debug, Clone)]
pub struct RoleGuard {
    role: String,
}

impl RoleGuard {
    pub fn new(role: impl Into<String>) -> Self {
        Self { role: role.into() }
    }
}

#[async_trait]
impl Guard for RoleGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        if identity.has_role(&self.role) {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::RoleRequired(self.role.clone()))
        }
    }

    fn description(&self) -> String {
        format!("Requires role: {}", self.role)
    }
}

/// Guard that requires any of the specified roles
#[derive(Debug, Clone)]
pub struct AnyRoleGuard {
    roles: Vec<String>,
}

impl AnyRoleGuard {
    pub fn new(roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl Guard for AnyRoleGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        let role_refs: Vec<&str> = self.roles.iter().map(|s| s.as_str()).collect();
        if identity.has_any_role(&role_refs) {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::InsufficientPermissions)
        }
    }

    fn description(&self) -> String {
        format!("Requires any role: {:?}", self.roles)
    }
}

/// Guard that requires all specified roles
#[derive(Debug, Clone)]
pub struct AllRolesGuard {
    roles: Vec<String>,
}

impl AllRolesGuard {
    pub fn new(roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl Guard for AllRolesGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        let has_all = self.roles.iter().all(|r| identity.has_role(r));
        if has_all {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::InsufficientPermissions)
        }
    }

    fn description(&self) -> String {
        format!("Requires all roles: {:?}", self.roles)
    }
}

/// Guard that requires a specific permission
#[derive(Debug, Clone)]
pub struct PermissionGuard {
    permission: String,
}

impl PermissionGuard {
    pub fn new(permission: impl Into<String>) -> Self {
        Self {
            permission: permission.into(),
        }
    }
}

#[async_trait]
impl Guard for PermissionGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        if identity.has_permission(&self.permission) {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::InsufficientPermissions)
        }
    }

    fn description(&self) -> String {
        format!("Requires permission: {}", self.permission)
    }
}

/// Guard that requires all specified permissions
#[derive(Debug, Clone)]
pub struct AllPermissionsGuard {
    permissions: Vec<String>,
}

impl AllPermissionsGuard {
    pub fn new(permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            permissions: permissions.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl Guard for AllPermissionsGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        let perm_refs: Vec<&str> = self.permissions.iter().map(|s| s.as_str()).collect();
        if identity.has_all_permissions(&perm_refs) {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::InsufficientPermissions)
        }
    }

    fn description(&self) -> String {
        format!("Requires all permissions: {:?}", self.permissions)
    }
}

/// Guard that requires any of the specified permissions
#[derive(Debug, Clone)]
pub struct AnyPermissionGuard {
    permissions: Vec<String>,
}

impl AnyPermissionGuard {
    pub fn new(permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            permissions: permissions.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait]
impl Guard for AnyPermissionGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        let has_any = self.permissions.iter().any(|p| identity.has_permission(p));
        if has_any {
            GuardResult::Allow
        } else {
            GuardResult::Deny(AuthError::InsufficientPermissions)
        }
    }

    fn description(&self) -> String {
        format!("Requires any permission: {:?}", self.permissions)
    }
}

/// Composite guard that requires all guards to pass
#[derive(Clone)]
pub struct AllGuards {
    guards: Vec<Arc<dyn Guard>>,
}

impl AllGuards {
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    pub fn add(mut self, guard: impl Guard + 'static) -> Self {
        self.guards.push(Arc::new(guard));
        self
    }
}

impl Default for AllGuards {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Guard for AllGuards {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        for guard in &self.guards {
            match guard.check(identity).await {
                GuardResult::Allow => continue,
                deny => return deny,
            }
        }
        GuardResult::Allow
    }

    fn description(&self) -> String {
        let descriptions: Vec<_> = self.guards.iter().map(|g| g.description()).collect();
        format!("All of: [{}]", descriptions.join(", "))
    }
}

/// Composite guard that requires any guard to pass
#[derive(Clone)]
pub struct AnyGuard {
    guards: Vec<Arc<dyn Guard>>,
}

impl AnyGuard {
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    pub fn add(mut self, guard: impl Guard + 'static) -> Self {
        self.guards.push(Arc::new(guard));
        self
    }
}

impl Default for AnyGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Guard for AnyGuard {
    async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        for guard in &self.guards {
            if let GuardResult::Allow = guard.check(identity).await {
                return GuardResult::Allow;
            }
        }
        GuardResult::Deny(AuthError::InsufficientPermissions)
    }

    fn description(&self) -> String {
        let descriptions: Vec<_> = self.guards.iter().map(|g| g.description()).collect();
        format!("Any of: [{}]", descriptions.join(", "))
    }
}

/// Extractor that requires a specific role
pub struct RequireRole<const ROLE: &'static str>(pub AuthIdentity);

#[axum::async_trait]
impl<S, const ROLE: &'static str> FromRequestParts<S> for RequireRole<ROLE>
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))?;

        let identity = context
            .identity
            .clone()
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))?;

        if identity.has_role(ROLE) {
            Ok(RequireRole(identity))
        } else {
            Err(auth_error_response(AuthError::RoleRequired(ROLE.to_string())))
        }
    }
}

/// Extractor that requires admin role
pub struct RequireAdmin(pub AuthIdentity);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let context = parts
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))?;

        let identity = context
            .identity
            .clone()
            .ok_or_else(|| auth_error_response(AuthError::NotAuthenticated))?;

        if identity.is_admin() {
            Ok(RequireAdmin(identity))
        } else {
            Err(auth_error_response(AuthError::RoleRequired("admin".to_string())))
        }
    }
}

/// Dynamic guard extractor
pub struct WithGuard<G: Guard + Clone + 'static> {
    pub identity: AuthIdentity,
    _phantom: PhantomData<G>,
}

impl<G: Guard + Clone + Default + 'static> WithGuard<G> {
    async fn check(identity: &AuthIdentity) -> GuardResult {
        let guard = G::default();
        guard.check(identity).await
    }
}

/// Guard builder for fluent API
pub struct GuardBuilder {
    guards: Vec<Arc<dyn Guard>>,
    require_all: bool,
}

impl GuardBuilder {
    pub fn new() -> Self {
        Self {
            guards: Vec::new(),
            require_all: true,
        }
    }

    /// Add a guard that requires authentication
    pub fn authenticated(self) -> Self {
        self.with_guard(AuthenticatedGuard)
    }

    /// Add a guard that requires a specific role
    pub fn role(self, role: impl Into<String>) -> Self {
        self.with_guard(RoleGuard::new(role))
    }

    /// Add a guard that requires any of the specified roles
    pub fn any_role(self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.with_guard(AnyRoleGuard::new(roles))
    }

    /// Add a guard that requires all specified roles
    pub fn all_roles(self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.with_guard(AllRolesGuard::new(roles))
    }

    /// Add a guard that requires a specific permission
    pub fn permission(self, permission: impl Into<String>) -> Self {
        self.with_guard(PermissionGuard::new(permission))
    }

    /// Add a guard that requires all specified permissions
    pub fn all_permissions(self, permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.with_guard(AllPermissionsGuard::new(permissions))
    }

    /// Add a guard that requires any of the specified permissions
    pub fn any_permission(self, permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.with_guard(AnyPermissionGuard::new(permissions))
    }

    /// Add a custom guard
    pub fn with_guard(mut self, guard: impl Guard + 'static) -> Self {
        self.guards.push(Arc::new(guard));
        self
    }

    /// Set to require any guard to pass (default is all)
    pub fn any(mut self) -> Self {
        self.require_all = false;
        self
    }

    /// Build the composite guard
    pub fn build(self) -> Arc<dyn Guard> {
        if self.guards.is_empty() {
            Arc::new(AuthenticatedGuard)
        } else if self.guards.len() == 1 {
            self.guards.into_iter().next().unwrap()
        } else if self.require_all {
            let mut all = AllGuards::new();
            for guard in self.guards {
                all.guards.push(guard);
            }
            Arc::new(all)
        } else {
            let mut any = AnyGuard::new();
            for guard in self.guards {
                any.guards.push(guard);
            }
            Arc::new(any)
        }
    }

    /// Check identity against built guards
    pub async fn check(&self, identity: &AuthIdentity) -> GuardResult {
        let guard = self.clone().build();
        guard.check(identity).await
    }
}

impl Default for GuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GuardBuilder {
    fn clone(&self) -> Self {
        Self {
            guards: self.guards.clone(),
            require_all: self.require_all,
        }
    }
}

/// Convenience functions for creating guards
pub fn require_auth() -> GuardBuilder {
    GuardBuilder::new().authenticated()
}

pub fn require_role(role: impl Into<String>) -> GuardBuilder {
    GuardBuilder::new().role(role)
}

pub fn require_admin() -> GuardBuilder {
    GuardBuilder::new().role("admin")
}

pub fn require_permission(permission: impl Into<String>) -> GuardBuilder {
    GuardBuilder::new().permission(permission)
}

/// Macro for defining guards declaratively
#[macro_export]
macro_rules! guard {
    (auth) => {
        $crate::auth::guards::require_auth()
    };
    (role: $role:expr) => {
        $crate::auth::guards::require_role($role)
    };
    (admin) => {
        $crate::auth::guards::require_admin()
    };
    (permission: $perm:expr) => {
        $crate::auth::guards::require_permission($perm)
    };
    (roles: [$($role:expr),+]) => {
        $crate::auth::guards::GuardBuilder::new().all_roles([$($role),+])
    };
    (any_role: [$($role:expr),+]) => {
        $crate::auth::guards::GuardBuilder::new().any_role([$($role),+])
    };
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_identity(roles: &[&str], permissions: &[&str]) -> AuthIdentity {
        AuthIdentity {
            user_id: UserId::new(),
            username: "testuser".to_string(),
            display_name: None,
            email: None,
            email_verified: false,
            roles: roles.iter().map(|s| s.to_string()).collect(),
            permissions: permissions.iter().map(|s| s.to_string()).collect(),
            auth_method: AuthMethod::Password,
            authenticated_at: chrono::Utc::now(),
            session_id: None,
            claims: serde_json::Value::Null,
        }
    }

    #[tokio::test]
    async fn test_authenticated_guard() {
        let guard = AuthenticatedGuard;
        let identity = create_identity(&[], &[]);

        let result = guard.check(&identity).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_role_guard_allowed() {
        let guard = RoleGuard::new("admin");
        let identity = create_identity(&["admin", "user"], &[]);

        let result = guard.check(&identity).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_role_guard_denied() {
        let guard = RoleGuard::new("admin");
        let identity = create_identity(&["user"], &[]);

        let result = guard.check(&identity).await;
        assert!(result.is_denied());
    }

    #[tokio::test]
    async fn test_any_role_guard() {
        let guard = AnyRoleGuard::new(["admin", "moderator"]);

        let identity1 = create_identity(&["moderator"], &[]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&["user"], &[]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_all_roles_guard() {
        let guard = AllRolesGuard::new(["admin", "verified"]);

        let identity1 = create_identity(&["admin", "verified"], &[]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&["admin"], &[]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_permission_guard() {
        let guard = PermissionGuard::new("users:write");

        let identity1 = create_identity(&[], &["users:read", "users:write"]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&[], &["users:read"]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_all_permissions_guard() {
        let guard = AllPermissionsGuard::new(["users:read", "users:write"]);

        let identity1 = create_identity(&[], &["users:read", "users:write", "users:delete"]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&[], &["users:read"]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_composite_all_guards() {
        let guard = AllGuards::new()
            .add(RoleGuard::new("user"))
            .add(PermissionGuard::new("posts:write"));

        let identity1 = create_identity(&["user"], &["posts:write"]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&["user"], &[]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_composite_any_guard() {
        let guard = AnyGuard::new()
            .add(RoleGuard::new("admin"))
            .add(PermissionGuard::new("override"));

        let identity1 = create_identity(&["admin"], &[]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&[], &["override"]);
        assert!(guard.check(&identity2).await.is_allowed());

        let identity3 = create_identity(&["user"], &[]);
        assert!(guard.check(&identity3).await.is_denied());
    }

    #[tokio::test]
    async fn test_guard_builder() {
        let guard = GuardBuilder::new()
            .authenticated()
            .role("admin")
            .permission("sensitive:access")
            .build();

        let identity1 = create_identity(&["admin"], &["sensitive:access"]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&["admin"], &[]);
        assert!(guard.check(&identity2).await.is_denied());
    }

    #[tokio::test]
    async fn test_guard_builder_any() {
        let guard = GuardBuilder::new()
            .role("admin")
            .role("superuser")
            .any()
            .build();

        let identity1 = create_identity(&["admin"], &[]);
        assert!(guard.check(&identity1).await.is_allowed());

        let identity2 = create_identity(&["superuser"], &[]);
        assert!(guard.check(&identity2).await.is_allowed());

        let identity3 = create_identity(&["user"], &[]);
        assert!(guard.check(&identity3).await.is_denied());
    }

    #[test]
    fn test_guard_description() {
        let guard = RoleGuard::new("admin");
        assert_eq!(guard.description(), "Requires role: admin");

        let guard = AllRolesGuard::new(["a", "b"]);
        assert!(guard.description().contains("all roles"));

        let guard = AllGuards::new()
            .add(RoleGuard::new("admin"))
            .add(PermissionGuard::new("test"));
        assert!(guard.description().contains("All of"));
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthIdentity and AuthError
- **Spec 372**: Auth Middleware - Provides AuthContext for guards
- **Spec 374**: Role-Based Access - Defines role hierarchy
- **Spec 375**: Permissions - Defines permission structure
