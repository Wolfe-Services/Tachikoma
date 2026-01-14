//! Middleware for the Tachikoma API server.

pub mod auth;
pub mod authz;
pub mod cors;
pub mod rate_limit;

pub use auth::{Auth, AdminAuth, AuthLayer, AuthMiddleware, AuthUser, MaybeAuth};
pub use authz::{
    Action, AccessPolicy, AuthzLayer, AuthzMiddleware, Permission, Resource, 
    Role, RoleRegistry, check_resource_access, log_authz
};
pub use cors::{AllowedHeaders, AllowedOrigins, CorsBuilder, CorsConfig, CorsLayer, CorsMiddleware};
pub use rate_limit::{
    RateLimitLayer, RateLimitMiddleware, RateLimitStore, InMemoryStore,
    RateLimitConfig, KeyStrategy, RateLimitInfo, RateLimitState
};