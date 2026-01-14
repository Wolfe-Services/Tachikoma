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