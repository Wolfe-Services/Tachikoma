# Spec 374: Role-Based Access Control

## Phase
17 - Authentication/Authorization

## Spec ID
374

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~10%

---

## Objective

Implement a comprehensive role-based access control (RBAC) system. This includes role definitions, role hierarchy, role assignment, and role-to-permission mapping. The system should support predefined system roles and custom user-defined roles.

---

## Acceptance Criteria

- [ ] Define `Role` struct with metadata
- [ ] Implement role hierarchy with inheritance
- [ ] Create `RoleManager` for role operations
- [ ] Support role-to-permission mapping
- [ ] Implement role assignment to users
- [ ] Support system roles (admin, user, guest)
- [ ] Allow custom role creation
- [ ] Provide role validation and conflict detection

---

## Implementation Details

### Role Types and Manager

```rust
// src/auth/roles.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    events::{AuthEvent, AuthEventEmitter},
    types::*,
};

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the role
    pub description: Option<String>,

    /// Permissions granted by this role
    pub permissions: HashSet<String>,

    /// Parent roles (for inheritance)
    pub inherits_from: Vec<String>,

    /// Whether this is a system role (cannot be deleted)
    pub system_role: bool,

    /// Whether the role is active
    pub enabled: bool,

    /// Role priority (higher = more priority in conflicts)
    pub priority: i32,

    /// When the role was created
    pub created_at: DateTime<Utc>,

    /// When the role was last updated
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Role {
    /// Create a new role
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            permissions: HashSet::new(),
            inherits_from: Vec::new(),
            system_role: false,
            enabled: true,
            priority: 0,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Create a system role
    pub fn system(id: impl Into<String>, name: impl Into<String>) -> Self {
        let mut role = Self::new(id, name);
        role.system_role = true;
        role
    }

    /// Add a permission to this role
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.insert(permission.into());
        self
    }

    /// Add multiple permissions
    pub fn with_permissions(mut self, permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.permissions.extend(permissions.into_iter().map(Into::into));
        self
    }

    /// Set parent role for inheritance
    pub fn inherits(mut self, parent_role: impl Into<String>) -> Self {
        self.inherits_from.push(parent_role.into());
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Predefined system roles
pub struct SystemRoles;

impl SystemRoles {
    /// Super admin with all permissions
    pub fn super_admin() -> Role {
        Role::system("super_admin", "Super Administrator")
            .with_description("Full system access with all permissions")
            .with_permission("*") // Wildcard permission
            .with_priority(1000)
    }

    /// Admin role with administrative permissions
    pub fn admin() -> Role {
        Role::system("admin", "Administrator")
            .with_description("Administrative access")
            .with_permissions([
                "users:read",
                "users:write",
                "users:delete",
                "roles:read",
                "roles:write",
                "settings:read",
                "settings:write",
                "audit:read",
            ])
            .with_priority(100)
    }

    /// Moderator role
    pub fn moderator() -> Role {
        Role::system("moderator", "Moderator")
            .with_description("Content moderation access")
            .inherits("user")
            .with_permissions([
                "content:moderate",
                "users:read",
                "reports:read",
                "reports:resolve",
            ])
            .with_priority(50)
    }

    /// Standard user role
    pub fn user() -> Role {
        Role::system("user", "User")
            .with_description("Standard user access")
            .with_permissions([
                "profile:read",
                "profile:write",
                "content:read",
                "content:create",
            ])
            .with_priority(10)
    }

    /// Guest role with minimal permissions
    pub fn guest() -> Role {
        Role::system("guest", "Guest")
            .with_description("Limited guest access")
            .with_permissions(["content:read"])
            .with_priority(0)
    }

    /// Get all system roles
    pub fn all() -> Vec<Role> {
        vec![
            Self::super_admin(),
            Self::admin(),
            Self::moderator(),
            Self::user(),
            Self::guest(),
        ]
    }
}

/// Role manager for role operations
pub struct RoleManager {
    storage: Arc<dyn RoleStorage>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    /// Cache of role hierarchy (role_id -> all effective permissions)
    permission_cache: RwLock<HashMap<String, HashSet<String>>>,
}

impl RoleManager {
    pub fn new(
        storage: Arc<dyn RoleStorage>,
        event_emitter: Arc<dyn AuthEventEmitter>,
    ) -> Self {
        Self {
            storage,
            event_emitter,
            permission_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize with system roles
    pub async fn initialize(&self) -> AuthResult<()> {
        for role in SystemRoles::all() {
            if self.storage.get(&role.id).await?.is_none() {
                self.storage.create(&role).await?;
            }
        }
        self.rebuild_cache().await?;
        Ok(())
    }

    /// Create a new role
    #[instrument(skip(self), fields(role_id = %role.id))]
    pub async fn create_role(&self, role: Role) -> AuthResult<()> {
        // Validate role
        self.validate_role(&role).await?;

        // Check for duplicate
        if self.storage.get(&role.id).await?.is_some() {
            return Err(AuthError::InvalidCredentials); // Role already exists
        }

        // Validate inheritance
        for parent_id in &role.inherits_from {
            if self.storage.get(parent_id).await?.is_none() {
                return Err(AuthError::ConfigError(format!(
                    "Parent role '{}' does not exist",
                    parent_id
                )));
            }
        }

        self.storage.create(&role).await?;
        self.invalidate_cache().await;

        self.event_emitter
            .emit(AuthEvent::RoleCreated {
                role_id: role.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!("Role created");
        Ok(())
    }

    /// Update an existing role
    #[instrument(skip(self), fields(role_id = %role.id))]
    pub async fn update_role(&self, role: Role) -> AuthResult<()> {
        let existing = self
            .storage
            .get(&role.id)
            .await?
            .ok_or(AuthError::UserNotFound)?; // Role not found

        // Cannot modify system roles' core properties
        if existing.system_role {
            if role.id != existing.id {
                return Err(AuthError::InsufficientPermissions);
            }
        }

        self.validate_role(&role).await?;
        self.storage.update(&role).await?;
        self.invalidate_cache().await;

        self.event_emitter
            .emit(AuthEvent::RoleUpdated {
                role_id: role.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        info!("Role updated");
        Ok(())
    }

    /// Delete a role
    #[instrument(skip(self), fields(role_id = %role_id))]
    pub async fn delete_role(&self, role_id: &str) -> AuthResult<()> {
        let role = self
            .storage
            .get(role_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        if role.system_role {
            return Err(AuthError::InsufficientPermissions);
        }

        // Check if any other roles inherit from this one
        let all_roles = self.storage.list().await?;
        for r in all_roles {
            if r.inherits_from.contains(&role_id.to_string()) {
                return Err(AuthError::ConfigError(format!(
                    "Role '{}' is inherited by '{}'",
                    role_id, r.id
                )));
            }
        }

        self.storage.delete(role_id).await?;
        self.invalidate_cache().await;

        self.event_emitter
            .emit(AuthEvent::RoleDeleted {
                role_id: role_id.to_string(),
                timestamp: Utc::now(),
            })
            .await;

        info!("Role deleted");
        Ok(())
    }

    /// Get a role by ID
    pub async fn get_role(&self, role_id: &str) -> AuthResult<Option<Role>> {
        self.storage.get(role_id).await
    }

    /// List all roles
    pub async fn list_roles(&self) -> AuthResult<Vec<Role>> {
        self.storage.list().await
    }

    /// Get all effective permissions for a role (including inherited)
    pub async fn get_effective_permissions(&self, role_id: &str) -> AuthResult<HashSet<String>> {
        // Check cache first
        {
            let cache = self.permission_cache.read().await;
            if let Some(permissions) = cache.get(role_id) {
                return Ok(permissions.clone());
            }
        }

        // Calculate and cache
        let permissions = self.calculate_permissions(role_id, &mut HashSet::new()).await?;

        {
            let mut cache = self.permission_cache.write().await;
            cache.insert(role_id.to_string(), permissions.clone());
        }

        Ok(permissions)
    }

    /// Get all effective permissions for multiple roles
    pub async fn get_permissions_for_roles(&self, role_ids: &[String]) -> AuthResult<HashSet<String>> {
        let mut all_permissions = HashSet::new();

        for role_id in role_ids {
            let permissions = self.get_effective_permissions(role_id).await?;
            all_permissions.extend(permissions);
        }

        Ok(all_permissions)
    }

    /// Calculate permissions recursively
    async fn calculate_permissions(
        &self,
        role_id: &str,
        visited: &mut HashSet<String>,
    ) -> AuthResult<HashSet<String>> {
        // Detect circular inheritance
        if visited.contains(role_id) {
            warn!(role_id = %role_id, "Circular role inheritance detected");
            return Ok(HashSet::new());
        }
        visited.insert(role_id.to_string());

        let role = match self.storage.get(role_id).await? {
            Some(r) => r,
            None => return Ok(HashSet::new()),
        };

        let mut permissions = role.permissions.clone();

        // Add inherited permissions
        for parent_id in &role.inherits_from {
            let parent_perms = self.calculate_permissions(parent_id, visited).await?;
            permissions.extend(parent_perms);
        }

        Ok(permissions)
    }

    /// Assign roles to a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn assign_roles(
        &self,
        user_id: UserId,
        role_ids: Vec<String>,
    ) -> AuthResult<()> {
        // Validate all roles exist
        for role_id in &role_ids {
            if self.storage.get(role_id).await?.is_none() {
                return Err(AuthError::ConfigError(format!(
                    "Role '{}' does not exist",
                    role_id
                )));
            }
        }

        self.storage.assign_to_user(user_id, role_ids.clone()).await?;

        self.event_emitter
            .emit(AuthEvent::RolesAssigned {
                user_id,
                role_ids,
                timestamp: Utc::now(),
            })
            .await;

        info!("Roles assigned to user");
        Ok(())
    }

    /// Remove roles from a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn remove_roles(
        &self,
        user_id: UserId,
        role_ids: Vec<String>,
    ) -> AuthResult<()> {
        self.storage.remove_from_user(user_id, role_ids.clone()).await?;

        self.event_emitter
            .emit(AuthEvent::RolesRemoved {
                user_id,
                role_ids,
                timestamp: Utc::now(),
            })
            .await;

        info!("Roles removed from user");
        Ok(())
    }

    /// Get roles for a user
    pub async fn get_user_roles(&self, user_id: UserId) -> AuthResult<Vec<Role>> {
        self.storage.get_user_roles(user_id).await
    }

    /// Check if user has a specific role
    pub async fn user_has_role(&self, user_id: UserId, role_id: &str) -> AuthResult<bool> {
        let roles = self.storage.get_user_roles(user_id).await?;
        Ok(roles.iter().any(|r| r.id == role_id))
    }

    /// Validate role definition
    async fn validate_role(&self, role: &Role) -> AuthResult<()> {
        if role.id.is_empty() {
            return Err(AuthError::ConfigError("Role ID cannot be empty".to_string()));
        }

        if role.name.is_empty() {
            return Err(AuthError::ConfigError("Role name cannot be empty".to_string()));
        }

        // Check for valid permission format
        for permission in &role.permissions {
            if !is_valid_permission_format(permission) {
                return Err(AuthError::ConfigError(format!(
                    "Invalid permission format: {}",
                    permission
                )));
            }
        }

        // Check for circular inheritance
        if self.has_circular_inheritance(role).await? {
            return Err(AuthError::ConfigError("Circular role inheritance detected".to_string()));
        }

        Ok(())
    }

    /// Check for circular inheritance
    async fn has_circular_inheritance(&self, role: &Role) -> AuthResult<bool> {
        let mut visited = HashSet::new();
        self.check_inheritance_cycle(&role.id, &role.inherits_from, &mut visited).await
    }

    async fn check_inheritance_cycle(
        &self,
        start_id: &str,
        parents: &[String],
        visited: &mut HashSet<String>,
    ) -> AuthResult<bool> {
        for parent_id in parents {
            if parent_id == start_id {
                return Ok(true);
            }

            if visited.contains(parent_id) {
                continue;
            }
            visited.insert(parent_id.clone());

            if let Some(parent) = self.storage.get(parent_id).await? {
                if self.check_inheritance_cycle(start_id, &parent.inherits_from, visited).await? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Invalidate permission cache
    async fn invalidate_cache(&self) {
        let mut cache = self.permission_cache.write().await;
        cache.clear();
    }

    /// Rebuild entire permission cache
    async fn rebuild_cache(&self) -> AuthResult<()> {
        self.invalidate_cache().await;

        let roles = self.storage.list().await?;
        for role in roles {
            let _ = self.get_effective_permissions(&role.id).await;
        }

        Ok(())
    }
}

/// Validate permission format (e.g., "resource:action")
fn is_valid_permission_format(permission: &str) -> bool {
    if permission == "*" {
        return true;
    }

    // Allow formats like "resource:action" or "resource:*"
    let parts: Vec<&str> = permission.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    let resource = parts[0];
    let action = parts[1];

    !resource.is_empty() && !action.is_empty()
}

/// Role storage trait
#[async_trait]
pub trait RoleStorage: Send + Sync {
    /// Create a new role
    async fn create(&self, role: &Role) -> AuthResult<()>;

    /// Get a role by ID
    async fn get(&self, id: &str) -> AuthResult<Option<Role>>;

    /// Update a role
    async fn update(&self, role: &Role) -> AuthResult<()>;

    /// Delete a role
    async fn delete(&self, id: &str) -> AuthResult<()>;

    /// List all roles
    async fn list(&self) -> AuthResult<Vec<Role>>;

    /// Assign roles to a user
    async fn assign_to_user(&self, user_id: UserId, role_ids: Vec<String>) -> AuthResult<()>;

    /// Remove roles from a user
    async fn remove_from_user(&self, user_id: UserId, role_ids: Vec<String>) -> AuthResult<()>;

    /// Get roles for a user
    async fn get_user_roles(&self, user_id: UserId) -> AuthResult<Vec<Role>>;
}

/// In-memory role storage
pub struct InMemoryRoleStorage {
    roles: RwLock<HashMap<String, Role>>,
    user_roles: RwLock<HashMap<UserId, Vec<String>>>,
}

impl InMemoryRoleStorage {
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRoleStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RoleStorage for InMemoryRoleStorage {
    async fn create(&self, role: &Role) -> AuthResult<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.id.clone(), role.clone());
        Ok(())
    }

    async fn get(&self, id: &str) -> AuthResult<Option<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.get(id).cloned())
    }

    async fn update(&self, role: &Role) -> AuthResult<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.id.clone(), role.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> AuthResult<()> {
        let mut roles = self.roles.write().await;
        roles.remove(id);
        Ok(())
    }

    async fn list(&self) -> AuthResult<Vec<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }

    async fn assign_to_user(&self, user_id: UserId, role_ids: Vec<String>) -> AuthResult<()> {
        let mut user_roles = self.user_roles.write().await;
        let entry = user_roles.entry(user_id).or_insert_with(Vec::new);
        for role_id in role_ids {
            if !entry.contains(&role_id) {
                entry.push(role_id);
            }
        }
        Ok(())
    }

    async fn remove_from_user(&self, user_id: UserId, role_ids: Vec<String>) -> AuthResult<()> {
        let mut user_roles = self.user_roles.write().await;
        if let Some(entry) = user_roles.get_mut(&user_id) {
            entry.retain(|r| !role_ids.contains(r));
        }
        Ok(())
    }

    async fn get_user_roles(&self, user_id: UserId) -> AuthResult<Vec<Role>> {
        let user_roles = self.user_roles.read().await;
        let roles = self.roles.read().await;

        let role_ids = user_roles.get(&user_id).cloned().unwrap_or_default();
        Ok(role_ids
            .iter()
            .filter_map(|id| roles.get(id).cloned())
            .collect())
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

    async fn setup_manager() -> RoleManager {
        let storage = Arc::new(InMemoryRoleStorage::new());
        let events = Arc::new(NoOpEventEmitter);
        let manager = RoleManager::new(storage, events);
        manager.initialize().await.unwrap();
        manager
    }

    #[tokio::test]
    async fn test_system_roles_initialization() {
        let manager = setup_manager().await;

        let admin = manager.get_role("admin").await.unwrap();
        assert!(admin.is_some());
        assert!(admin.unwrap().system_role);

        let user = manager.get_role("user").await.unwrap();
        assert!(user.is_some());
    }

    #[tokio::test]
    async fn test_create_custom_role() {
        let manager = setup_manager().await;

        let role = Role::new("custom_role", "Custom Role")
            .with_description("A custom role")
            .with_permission("custom:action");

        manager.create_role(role).await.unwrap();

        let retrieved = manager.get_role("custom_role").await.unwrap();
        assert!(retrieved.is_some());
        assert!(!retrieved.unwrap().system_role);
    }

    #[tokio::test]
    async fn test_role_inheritance() {
        let manager = setup_manager().await;

        // Create a role that inherits from user
        let role = Role::new("premium_user", "Premium User")
            .inherits("user")
            .with_permission("premium:feature");

        manager.create_role(role).await.unwrap();

        let permissions = manager.get_effective_permissions("premium_user").await.unwrap();

        // Should have own permission plus inherited from user
        assert!(permissions.contains("premium:feature"));
        assert!(permissions.contains("profile:read")); // From user role
    }

    #[tokio::test]
    async fn test_cannot_delete_system_role() {
        let manager = setup_manager().await;

        let result = manager.delete_role("admin").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_assign_roles_to_user() {
        let manager = setup_manager().await;
        let user_id = UserId::new();

        manager
            .assign_roles(user_id, vec!["user".to_string(), "moderator".to_string()])
            .await
            .unwrap();

        let roles = manager.get_user_roles(user_id).await.unwrap();
        assert_eq!(roles.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_roles_from_user() {
        let manager = setup_manager().await;
        let user_id = UserId::new();

        manager
            .assign_roles(user_id, vec!["user".to_string(), "moderator".to_string()])
            .await
            .unwrap();

        manager
            .remove_roles(user_id, vec!["moderator".to_string()])
            .await
            .unwrap();

        let roles = manager.get_user_roles(user_id).await.unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].id, "user");
    }

    #[tokio::test]
    async fn test_circular_inheritance_detection() {
        let manager = setup_manager().await;

        // Create role A
        let role_a = Role::new("role_a", "Role A");
        manager.create_role(role_a).await.unwrap();

        // Try to create role B that inherits from A
        let role_b = Role::new("role_b", "Role B").inherits("role_a");
        manager.create_role(role_b).await.unwrap();

        // Try to update A to inherit from B (circular)
        let role_a_updated = Role::new("role_a", "Role A").inherits("role_b");
        let result = manager.update_role(role_a_updated).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_permission_format_validation() {
        assert!(is_valid_permission_format("users:read"));
        assert!(is_valid_permission_format("users:*"));
        assert!(is_valid_permission_format("*"));
        assert!(!is_valid_permission_format("invalid"));
        assert!(!is_valid_permission_format(""));
        assert!(!is_valid_permission_format(":action"));
    }

    #[tokio::test]
    async fn test_get_permissions_for_multiple_roles() {
        let manager = setup_manager().await;

        let permissions = manager
            .get_permissions_for_roles(&["admin".to_string(), "user".to_string()])
            .await
            .unwrap();

        // Should have permissions from both roles
        assert!(permissions.contains("users:read")); // From admin
        assert!(permissions.contains("profile:read")); // From user
    }

    struct NoOpEventEmitter;
    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthIdentity for role attachment
- **Spec 373**: Auth Guards - Uses roles for access control
- **Spec 375**: Permissions - Role-to-permission mapping
- **Spec 381**: Audit Logging - Logs role changes
