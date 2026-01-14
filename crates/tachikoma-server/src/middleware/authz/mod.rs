//! Authorization middleware for role-based access control.

pub mod audit;
pub mod layer;
pub mod resource;
pub mod types;

pub use audit::{log_authz, AuthzAuditEvent};
pub use layer::{AuthzLayer, AuthzMiddleware};
pub use resource::{check_resource_access, AccessPolicy, ResourceOwner};
pub use types::{Action, Permission, Resource, Role, RoleRegistry};