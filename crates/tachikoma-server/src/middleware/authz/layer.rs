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
    S::Error: From<ApiError>,
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