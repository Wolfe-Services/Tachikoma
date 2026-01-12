# Spec 375: Permission System

## Phase
17 - Authentication/Authorization

## Spec ID
375

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 374: Role-Based Access

## Estimated Context
~9%

---

## Objective

Implement a fine-grained permission system that supports resource-based access control. Permissions follow the pattern "resource:action" and support wildcards. The system should integrate with roles while also allowing direct permission grants to users.

---

## Acceptance Criteria

- [ ] Define `Permission` struct with resource and action
- [ ] Implement permission parsing and validation
- [ ] Support wildcard permissions (e.g., "users:*", "*")
- [ ] Create `PermissionManager` for permission operations
- [ ] Support direct permission grants to users
- [ ] Implement permission checking with inheritance
- [ ] Support resource-scoped permissions (e.g., "users:123:read")
- [ ] Provide permission hierarchy and grouping

---

## Implementation Details

### Permission Types and Manager

```rust
// src/auth/permissions.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::auth::{
    events::{AuthEvent, AuthEventEmitter},
    roles::RoleManager,
    types::*,
};

/// Permission definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource being accessed (e.g., "users", "posts")
    pub resource: String,

    /// Action being performed (e.g., "read", "write", "delete")
    pub action: String,

    /// Optional scope (e.g., specific resource ID)
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: None,
        }
    }

    /// Create a scoped permission
    pub fn scoped(
        resource: impl Into<String>,
        action: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: Some(scope.into()),
        }
    }

    /// Create a wildcard permission for all actions on a resource
    pub fn all_actions(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: "*".to_string(),
            scope: None,
        }
    }

    /// Create a super admin permission (all resources, all actions)
    pub fn all() -> Self {
        Self {
            resource: "*".to_string(),
            action: "*".to_string(),
            scope: None,
        }
    }

    /// Parse from string format "resource:action" or "resource:action:scope"
    pub fn parse(s: &str) -> Result<Self, PermissionParseError> {
        let parts: Vec<&str> = s.split(':').collect();

        match parts.len() {
            1 if parts[0] == "*" => Ok(Self::all()),
            2 => Ok(Self::new(parts[0], parts[1])),
            3 => Ok(Self::scoped(parts[0], parts[1], parts[2])),
            _ => Err(PermissionParseError::InvalidFormat(s.to_string())),
        }
    }

    /// Convert to string format
    pub fn to_string(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{}:{}:{}", self.resource, self.action, scope),
            None => format!("{}:{}", self.resource, self.action),
        }
    }

    /// Check if this permission matches another (considering wildcards)
    pub fn matches(&self, other: &Permission) -> bool {
        // Wildcard matches everything
        if self.resource == "*" && self.action == "*" {
            return true;
        }

        // Check resource match
        let resource_matches = self.resource == "*" || self.resource == other.resource;

        // Check action match
        let action_matches = self.action == "*" || self.action == other.action;

        // Check scope match (None scope matches all scopes)
        let scope_matches = match (&self.scope, &other.scope) {
            (None, _) => true, // No scope requirement
            (Some(s1), Some(s2)) => s1 == "*" || s1 == s2,
            (Some(_), None) => false,
        };

        resource_matches && action_matches && scope_matches
    }

    /// Check if this permission implies another
    pub fn implies(&self, other: &Permission) -> bool {
        self.matches(other)
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for Permission {
    fn from(s: &str) -> Self {
        Self::parse(s).unwrap_or_else(|_| Self::new(s, "*"))
    }
}

/// Permission parse error
#[derive(Debug, Clone, thiserror::Error)]
pub enum PermissionParseError {
    #[error("Invalid permission format: {0}")]
    InvalidFormat(String),
}

/// Permission grant to a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    /// The permission being granted
    pub permission: Permission,

    /// User the permission is granted to
    pub user_id: UserId,

    /// When the grant was created
    pub granted_at: DateTime<Utc>,

    /// Who granted the permission (user ID)
    pub granted_by: Option<UserId>,

    /// Optional expiration
    pub expires_at: Option<DateTime<Utc>>,

    /// Reason for the grant
    pub reason: Option<String>,
}

impl PermissionGrant {
    pub fn new(permission: Permission, user_id: UserId) -> Self {
        Self {
            permission,
            user_id,
            granted_at: Utc::now(),
            granted_by: None,
            expires_at: None,
            reason: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| Utc::now() > exp)
            .unwrap_or(false)
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Permission group for organizing permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    /// Group identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Permissions in this group
    pub permissions: Vec<Permission>,
}

impl PermissionGroup {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            permissions: Vec::new(),
        }
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }

    pub fn with_permissions(mut self, permissions: impl IntoIterator<Item = Permission>) -> Self {
        self.permissions.extend(permissions);
        self
    }
}

/// Common permission groups
pub struct CommonPermissionGroups;

impl CommonPermissionGroups {
    /// User management permissions
    pub fn user_management() -> PermissionGroup {
        PermissionGroup::new("user_management", "User Management")
            .with_permissions([
                Permission::new("users", "read"),
                Permission::new("users", "create"),
                Permission::new("users", "update"),
                Permission::new("users", "delete"),
                Permission::new("users", "list"),
            ])
    }

    /// Content management permissions
    pub fn content_management() -> PermissionGroup {
        PermissionGroup::new("content_management", "Content Management")
            .with_permissions([
                Permission::new("content", "read"),
                Permission::new("content", "create"),
                Permission::new("content", "update"),
                Permission::new("content", "delete"),
                Permission::new("content", "publish"),
                Permission::new("content", "moderate"),
            ])
    }

    /// System administration permissions
    pub fn system_admin() -> PermissionGroup {
        PermissionGroup::new("system_admin", "System Administration")
            .with_permissions([
                Permission::new("settings", "read"),
                Permission::new("settings", "write"),
                Permission::new("audit", "read"),
                Permission::new("roles", "read"),
                Permission::new("roles", "write"),
                Permission::new("permissions", "read"),
                Permission::new("permissions", "write"),
            ])
    }
}

/// Permission manager
pub struct PermissionManager {
    storage: Arc<dyn PermissionStorage>,
    role_manager: Arc<RoleManager>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    /// Cache of user permissions
    user_cache: RwLock<HashMap<UserId, HashSet<String>>>,
}

impl PermissionManager {
    pub fn new(
        storage: Arc<dyn PermissionStorage>,
        role_manager: Arc<RoleManager>,
        event_emitter: Arc<dyn AuthEventEmitter>,
    ) -> Self {
        Self {
            storage,
            role_manager,
            event_emitter,
            user_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a user has a specific permission
    #[instrument(skip(self), fields(user_id = %user_id, permission = %permission))]
    pub async fn check_permission(
        &self,
        user_id: UserId,
        permission: &Permission,
    ) -> AuthResult<bool> {
        let user_permissions = self.get_user_permissions(user_id).await?;

        // Check if any of the user's permissions match
        for perm_str in &user_permissions {
            if let Ok(user_perm) = Permission::parse(perm_str) {
                if user_perm.implies(permission) {
                    debug!(granted_by = %perm_str, "Permission granted");
                    return Ok(true);
                }
            }
        }

        debug!("Permission denied");
        Ok(false)
    }

    /// Check if a user has a permission by string
    pub async fn check_permission_str(
        &self,
        user_id: UserId,
        permission: &str,
    ) -> AuthResult<bool> {
        let perm = Permission::parse(permission)
            .map_err(|e| AuthError::ConfigError(e.to_string()))?;
        self.check_permission(user_id, &perm).await
    }

    /// Get all effective permissions for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user_permissions(&self, user_id: UserId) -> AuthResult<HashSet<String>> {
        // Check cache
        {
            let cache = self.user_cache.read().await;
            if let Some(permissions) = cache.get(&user_id) {
                return Ok(permissions.clone());
            }
        }

        let mut permissions = HashSet::new();

        // Get permissions from roles
        let roles = self.role_manager.get_user_roles(user_id).await?;
        let role_ids: Vec<String> = roles.iter().map(|r| r.id.clone()).collect();
        let role_permissions = self.role_manager.get_permissions_for_roles(&role_ids).await?;
        permissions.extend(role_permissions);

        // Get direct permission grants
        let grants = self.storage.get_user_grants(user_id).await?;
        for grant in grants {
            if grant.is_valid() {
                permissions.insert(grant.permission.to_string());
            }
        }

        // Cache the result
        {
            let mut cache = self.user_cache.write().await;
            cache.insert(user_id, permissions.clone());
        }

        Ok(permissions)
    }

    /// Grant a permission directly to a user
    #[instrument(skip(self), fields(user_id = %user_id, permission = %permission))]
    pub async fn grant_permission(
        &self,
        user_id: UserId,
        permission: Permission,
        granted_by: Option<UserId>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
    ) -> AuthResult<()> {
        let mut grant = PermissionGrant::new(permission.clone(), user_id);
        grant.granted_by = granted_by;
        grant.expires_at = expires_at;
        grant.reason = reason;

        self.storage.create_grant(&grant).await?;
        self.invalidate_user_cache(user_id).await;

        self.event_emitter
            .emit(AuthEvent::PermissionGranted {
                user_id,
                permission: permission.to_string(),
                granted_by,
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Revoke a permission from a user
    #[instrument(skip(self), fields(user_id = %user_id, permission = %permission))]
    pub async fn revoke_permission(
        &self,
        user_id: UserId,
        permission: &Permission,
    ) -> AuthResult<()> {
        self.storage.revoke_grant(user_id, permission).await?;
        self.invalidate_user_cache(user_id).await;

        self.event_emitter
            .emit(AuthEvent::PermissionRevoked {
                user_id,
                permission: permission.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(())
    }

    /// Get all direct permission grants for a user
    pub async fn get_user_grants(&self, user_id: UserId) -> AuthResult<Vec<PermissionGrant>> {
        self.storage.get_user_grants(user_id).await
    }

    /// Invalidate cache for a user
    async fn invalidate_user_cache(&self, user_id: UserId) {
        let mut cache = self.user_cache.write().await;
        cache.remove(&user_id);
    }

    /// Invalidate all caches
    pub async fn invalidate_all_caches(&self) {
        let mut cache = self.user_cache.write().await;
        cache.clear();
    }

    /// Clean up expired grants
    pub async fn cleanup_expired_grants(&self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

/// Permission checker for use in handlers
pub struct PermissionChecker {
    manager: Arc<PermissionManager>,
}

impl PermissionChecker {
    pub fn new(manager: Arc<PermissionManager>) -> Self {
        Self { manager }
    }

    /// Check permission and return error if denied
    pub async fn require(
        &self,
        user_id: UserId,
        permission: &str,
    ) -> AuthResult<()> {
        if self.manager.check_permission_str(user_id, permission).await? {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }

    /// Check multiple permissions (all required)
    pub async fn require_all(
        &self,
        user_id: UserId,
        permissions: &[&str],
    ) -> AuthResult<()> {
        for perm in permissions {
            self.require(user_id, perm).await?;
        }
        Ok(())
    }

    /// Check multiple permissions (any required)
    pub async fn require_any(
        &self,
        user_id: UserId,
        permissions: &[&str],
    ) -> AuthResult<()> {
        for perm in permissions {
            if self.manager.check_permission_str(user_id, perm).await? {
                return Ok(());
            }
        }
        Err(AuthError::InsufficientPermissions)
    }
}

/// Permission storage trait
#[async_trait]
pub trait PermissionStorage: Send + Sync {
    /// Create a permission grant
    async fn create_grant(&self, grant: &PermissionGrant) -> AuthResult<()>;

    /// Get all grants for a user
    async fn get_user_grants(&self, user_id: UserId) -> AuthResult<Vec<PermissionGrant>>;

    /// Revoke a specific grant
    async fn revoke_grant(&self, user_id: UserId, permission: &Permission) -> AuthResult<()>;

    /// Clean up expired grants
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory permission storage
pub struct InMemoryPermissionStorage {
    grants: RwLock<Vec<PermissionGrant>>,
}

impl InMemoryPermissionStorage {
    pub fn new() -> Self {
        Self {
            grants: RwLock::new(Vec::new()),
        }
    }
}

impl Default for InMemoryPermissionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionStorage for InMemoryPermissionStorage {
    async fn create_grant(&self, grant: &PermissionGrant) -> AuthResult<()> {
        let mut grants = self.grants.write().await;
        grants.push(grant.clone());
        Ok(())
    }

    async fn get_user_grants(&self, user_id: UserId) -> AuthResult<Vec<PermissionGrant>> {
        let grants = self.grants.read().await;
        Ok(grants
            .iter()
            .filter(|g| g.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn revoke_grant(&self, user_id: UserId, permission: &Permission) -> AuthResult<()> {
        let mut grants = self.grants.write().await;
        grants.retain(|g| !(g.user_id == user_id && g.permission == *permission));
        Ok(())
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut grants = self.grants.write().await;
        let before = grants.len();
        grants.retain(|g| g.is_valid());
        Ok(before - grants.len())
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parse() {
        let perm = Permission::parse("users:read").unwrap();
        assert_eq!(perm.resource, "users");
        assert_eq!(perm.action, "read");
        assert!(perm.scope.is_none());

        let scoped = Permission::parse("users:read:123").unwrap();
        assert_eq!(scoped.scope, Some("123".to_string()));

        let wildcard = Permission::parse("*").unwrap();
        assert_eq!(wildcard.resource, "*");
        assert_eq!(wildcard.action, "*");
    }

    #[test]
    fn test_permission_matches() {
        let all = Permission::all();
        let specific = Permission::new("users", "read");

        assert!(all.matches(&specific));
        assert!(!specific.matches(&all));

        let resource_wildcard = Permission::new("users", "*");
        assert!(resource_wildcard.matches(&specific));
        assert!(resource_wildcard.matches(&Permission::new("users", "write")));
        assert!(!resource_wildcard.matches(&Permission::new("posts", "read")));
    }

    #[test]
    fn test_scoped_permission_matches() {
        let unscoped = Permission::new("users", "read");
        let scoped = Permission::scoped("users", "read", "123");

        // Unscoped permission matches scoped
        assert!(unscoped.matches(&scoped));

        // Scoped permission only matches same scope
        assert!(scoped.matches(&scoped));
        assert!(!scoped.matches(&Permission::scoped("users", "read", "456")));
    }

    #[test]
    fn test_permission_to_string() {
        let perm = Permission::new("users", "read");
        assert_eq!(perm.to_string(), "users:read");

        let scoped = Permission::scoped("users", "read", "123");
        assert_eq!(scoped.to_string(), "users:read:123");
    }

    #[tokio::test]
    async fn test_permission_grant_expiration() {
        let grant = PermissionGrant {
            permission: Permission::new("test", "action"),
            user_id: UserId::new(),
            granted_at: Utc::now(),
            granted_by: None,
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
            reason: None,
        };

        assert!(grant.is_expired());
        assert!(!grant.is_valid());
    }

    #[tokio::test]
    async fn test_permission_manager_check() {
        let perm_storage = Arc::new(InMemoryPermissionStorage::new());
        let role_storage = Arc::new(InMemoryRoleStorage::new());
        let events = Arc::new(NoOpEventEmitter);

        let role_manager = Arc::new(RoleManager::new(role_storage, events.clone()));
        role_manager.initialize().await.unwrap();

        let manager = PermissionManager::new(perm_storage, role_manager.clone(), events);

        let user_id = UserId::new();

        // Assign user role
        role_manager
            .assign_roles(user_id, vec!["user".to_string()])
            .await
            .unwrap();

        // Check permission from role
        let has_profile_read = manager
            .check_permission(user_id, &Permission::new("profile", "read"))
            .await
            .unwrap();
        assert!(has_profile_read);

        // Check permission not in role
        let has_admin = manager
            .check_permission(user_id, &Permission::new("users", "delete"))
            .await
            .unwrap();
        assert!(!has_admin);
    }

    #[tokio::test]
    async fn test_direct_permission_grant() {
        let perm_storage = Arc::new(InMemoryPermissionStorage::new());
        let role_storage = Arc::new(InMemoryRoleStorage::new());
        let events = Arc::new(NoOpEventEmitter);

        let role_manager = Arc::new(RoleManager::new(role_storage, events.clone()));
        let manager = PermissionManager::new(perm_storage, role_manager, events);

        let user_id = UserId::new();
        let permission = Permission::new("special", "action");

        // Grant permission directly
        manager
            .grant_permission(user_id, permission.clone(), None, None, None)
            .await
            .unwrap();

        // Check permission
        let has_perm = manager.check_permission(user_id, &permission).await.unwrap();
        assert!(has_perm);

        // Revoke permission
        manager.revoke_permission(user_id, &permission).await.unwrap();

        // Check again
        let has_perm = manager.check_permission(user_id, &permission).await.unwrap();
        assert!(!has_perm);
    }

    #[tokio::test]
    async fn test_permission_checker() {
        let perm_storage = Arc::new(InMemoryPermissionStorage::new());
        let role_storage = Arc::new(InMemoryRoleStorage::new());
        let events = Arc::new(NoOpEventEmitter);

        let role_manager = Arc::new(RoleManager::new(role_storage, events.clone()));
        role_manager.initialize().await.unwrap();

        let manager = Arc::new(PermissionManager::new(perm_storage, role_manager.clone(), events));
        let checker = PermissionChecker::new(manager);

        let user_id = UserId::new();
        role_manager
            .assign_roles(user_id, vec!["user".to_string()])
            .await
            .unwrap();

        // Should succeed
        assert!(checker.require(user_id, "profile:read").await.is_ok());

        // Should fail
        assert!(checker.require(user_id, "admin:action").await.is_err());
    }

    use crate::auth::roles::InMemoryRoleStorage;

    struct NoOpEventEmitter;
    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses UserId and AuthError
- **Spec 373**: Auth Guards - Uses permissions for access control
- **Spec 374**: Role-Based Access - Roles grant permissions
- **Spec 381**: Audit Logging - Logs permission changes
