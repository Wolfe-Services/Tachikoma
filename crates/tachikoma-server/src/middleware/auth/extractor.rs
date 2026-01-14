//! Authentication extractors for handlers.

use super::types::AuthUser;
use crate::error::ApiError;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};

/// Extractor for authenticated user (required).
pub struct Auth(pub AuthUser);

#[async_trait]
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(Auth)
            .ok_or(ApiError::Unauthorized)
    }
}

/// Extractor for optional authenticated user.
pub struct MaybeAuth(pub Option<AuthUser>);

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuth
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(MaybeAuth(parts.extensions.get::<AuthUser>().cloned()))
    }
}

/// Extractor that requires admin role.
pub struct AdminAuth(pub AuthUser);

#[async_trait]
impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(ApiError::Unauthorized)?;

        if user.is_admin() {
            Ok(AdminAuth(user))
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

/// Extractor that requires specific role.
pub struct RequireRole<const ROLE: &'static str>(pub AuthUser);

#[async_trait]
impl<S, const ROLE: &'static str> FromRequestParts<S> for RequireRole<ROLE>
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(ApiError::Unauthorized)?;

        if user.roles.contains(&ROLE.to_string()) {
            Ok(RequireRole(user))
        } else {
            Err(ApiError::InsufficientPermissions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, request::Parts};
    use uuid::Uuid;
    use crate::middleware::auth::types::Claims;

    #[tokio::test]
    async fn test_auth_extractor_success() {
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "test@example.com",
            vec!["user".into()],
            3600,
        );
        let auth_user = AuthUser::from_claims(claims).unwrap();

        let req = Request::new(());
        let (mut parts, _) = req.into_parts();
        parts.extensions.insert(auth_user.clone());

        let Auth(extracted_user) = Auth::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(extracted_user.id, auth_user.id);
    }

    #[tokio::test]
    async fn test_auth_extractor_missing() {
        let req = Request::new(());
        let (mut parts, _) = req.into_parts();

        let result = Auth::from_request_parts(&mut parts, &()).await;
        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn test_maybe_auth_extractor_some() {
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "test@example.com",
            vec!["user".into()],
            3600,
        );
        let auth_user = AuthUser::from_claims(claims).unwrap();

        let req = Request::new(());
        let (mut parts, _) = req.into_parts();
        parts.extensions.insert(auth_user.clone());

        let MaybeAuth(extracted_user) = MaybeAuth::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(extracted_user.is_some());
        assert_eq!(extracted_user.unwrap().id, auth_user.id);
    }

    #[tokio::test]
    async fn test_maybe_auth_extractor_none() {
        let req = Request::new(());
        let (mut parts, _) = req.into_parts();

        let MaybeAuth(extracted_user) = MaybeAuth::from_request_parts(&mut parts, &()).await.unwrap();
        assert!(extracted_user.is_none());
    }

    #[tokio::test]
    async fn test_admin_auth_extractor_success() {
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "admin@example.com",
            vec!["admin".into()],
            3600,
        );
        let auth_user = AuthUser::from_claims(claims).unwrap();

        let req = Request::new(());
        let (mut parts, _) = req.into_parts();
        parts.extensions.insert(auth_user.clone());

        let AdminAuth(extracted_user) = AdminAuth::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(extracted_user.id, auth_user.id);
    }

    #[tokio::test]
    async fn test_admin_auth_extractor_forbidden() {
        let claims = Claims::new_access(
            Uuid::new_v4(),
            "user@example.com",
            vec!["user".into()],
            3600,
        );
        let auth_user = AuthUser::from_claims(claims).unwrap();

        let req = Request::new(());
        let (mut parts, _) = req.into_parts();
        parts.extensions.insert(auth_user);

        let result = AdminAuth::from_request_parts(&mut parts, &()).await;
        assert!(matches!(result, Err(ApiError::Forbidden)));
    }
}