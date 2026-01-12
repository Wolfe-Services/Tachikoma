# Spec 390: Authentication Tests

## Overview
Comprehensive test suite for the authentication system covering all components.

## Rust Implementation

### Test Infrastructure
```rust
// src/auth/tests/mod.rs

pub mod fixtures;
pub mod helpers;
pub mod integration;
pub mod unit;

use sqlx::sqlite::SqlitePool;
use std::sync::Arc;

/// Test database setup
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::query(include_str!("../migrations/auth.sql"))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Test user for fixtures
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password: String,
    pub role: crate::auth::types::UserRole,
}

impl Default for TestUser {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email: format!("test-{}@example.com", uuid::Uuid::new_v4()),
            password: "Test123!@#".to_string(),
            role: crate::auth::types::UserRole::User,
        }
    }
}
```

### Unit Tests
```rust
// src/auth/tests/unit.rs

use super::*;
use crate::auth::*;

mod password_tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let validator = PasswordValidator::default();

        // Valid passwords
        assert!(validator.validate("Test123!@#").is_ok());
        assert!(validator.validate("SecureP@ss1").is_ok());

        // Invalid passwords
        assert!(validator.validate("short").is_err());
        assert!(validator.validate("nouppercase123!").is_err());
        assert!(validator.validate("NOLOWERCASE123!").is_err());
        assert!(validator.validate("NoNumbers!!!").is_err());
        assert!(validator.validate("NoSpecialChars123").is_err());
    }

    #[test]
    fn test_password_entropy() {
        let weak = "password";
        let strong = "Tr0ub4dor&3";

        assert!(calculate_entropy(weak) < calculate_entropy(strong));
    }
}

mod jwt_tests {
    use super::*;
    use crate::auth::jwt::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-key-at-least-32-characters".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_jwt_encode_decode() {
        let handler = JwtHandler::new(test_config()).unwrap();

        let claims = Claims::new(
            "user-123",
            Some("test@example.com".to_string()),
            types::UserRole::User,
            vec!["read".to_string()],
            "tachikoma",
            vec!["tachikoma".to_string()],
            chrono::Duration::hours(1),
        );

        let token = handler.encode(&claims).unwrap();
        let decoded = handler.validate(&token).unwrap();

        assert_eq!(decoded.sub, "user-123");
        assert_eq!(decoded.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_jwt_expiration() {
        let handler = JwtHandler::new(test_config()).unwrap();

        let mut claims = Claims::new(
            "user-123",
            None,
            types::UserRole::User,
            vec![],
            "tachikoma",
            vec!["tachikoma".to_string()],
            chrono::Duration::hours(1),
        );

        // Make token expired
        claims.exp = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp();

        let token = handler.encode(&claims).unwrap();
        assert!(handler.validate(&token).is_err());
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("Basic abc123"), None);
        assert_eq!(extract_bearer_token("abc123"), None);
    }
}

mod session_tests {
    use super::*;
    use crate::auth::session::*;

    #[test]
    fn test_session_creation() {
        let session = Session {
            id: "session-123".to_string(),
            user_id: "user-123".to_string(),
            email: Some("test@example.com".to_string()),
            role: types::UserRole::User,
            permissions: vec![],
            auth_method: types::AuthMethod::Password,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            last_active_at: chrono::Utc::now(),
            tenant_id: None,
        };

        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expiration() {
        let session = Session {
            id: "session-123".to_string(),
            user_id: "user-123".to_string(),
            email: None,
            role: types::UserRole::User,
            permissions: vec![],
            auth_method: types::AuthMethod::Password,
            ip_address: None,
            user_agent: None,
            created_at: chrono::Utc::now() - chrono::Duration::hours(48),
            expires_at: chrono::Utc::now() - chrono::Duration::hours(24),
            last_active_at: chrono::Utc::now() - chrono::Duration::hours(24),
            tenant_id: None,
        };

        assert!(session.is_expired());
    }
}

mod token_tests {
    use super::*;
    use crate::auth::tokens::*;

    #[test]
    fn test_opaque_token_generation() {
        let token = OpaqueToken::generate(
            TokenType::Access,
            chrono::Duration::hours(1),
            32
        );

        assert!(!token.value.is_empty());
        assert!(!token.hash.is_empty());
        assert!(token.value != token.hash);
    }

    #[test]
    fn test_token_verification() {
        let token = OpaqueToken::generate(
            TokenType::Access,
            chrono::Duration::hours(1),
            32
        );

        // Verify correct token
        let verified = OpaqueToken::verify(&token.value, &token.hash);
        assert!(verified);

        // Verify wrong token
        let wrong = OpaqueToken::verify("wrong-token", &token.hash);
        assert!(!wrong);
    }
}

mod permission_tests {
    use super::*;
    use crate::auth::types::*;

    #[test]
    fn test_role_hierarchy() {
        assert!(UserRole::Admin.has_role(UserRole::User));
        assert!(UserRole::SuperAdmin.has_role(UserRole::Admin));
        assert!(!UserRole::User.has_role(UserRole::Admin));
    }

    #[test]
    fn test_permissions_for_role() {
        let user_perms = permissions::permissions_for_role(UserRole::User);
        let admin_perms = permissions::permissions_for_role(UserRole::Admin);

        assert!(user_perms.contains(&Permission::Read));
        assert!(!user_perms.contains(&Permission::ManageUsers));
        assert!(admin_perms.contains(&Permission::ManageUsers));
    }
}

mod rate_limit_tests {
    use super::*;
    use crate::auth::rate_limit::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_initial() {
        let limiter = RateLimiter::new(RateLimitConfig::default());
        let key = RateLimitKey::email(RateLimitAction::Login, "test@example.com");

        let result = limiter.check(&key).await;
        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_excess() {
        let config = RateLimitConfig {
            login_attempts: 2,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);
        let key = RateLimitKey::email(RateLimitAction::Login, "test@example.com");

        // Use up attempts
        limiter.record(&key).await;
        limiter.record(&key).await;

        // Third should be blocked
        let result = limiter.check(&key).await;
        assert!(!result.is_allowed());
    }
}

mod magic_link_tests {
    use super::*;
    use crate::auth::magic_link::types::*;

    #[test]
    fn test_magic_link_purpose() {
        assert_eq!(MagicLinkPurpose::Login.as_str(), "login");
        assert_eq!(MagicLinkPurpose::from_str("signup"), MagicLinkPurpose::Signup);
    }

    #[test]
    fn test_magic_link_lifetime() {
        let login_lifetime = MagicLinkPurpose::Login.default_lifetime();
        let signup_lifetime = MagicLinkPurpose::Signup.default_lifetime();

        assert!(login_lifetime < signup_lifetime);
    }
}

mod device_code_tests {
    use super::*;
    use crate::auth::device_code::types::*;

    #[test]
    fn test_user_code_format() {
        let format = UserCodeFormat::AlphanumericWithDash;

        assert_eq!(format.format("ABCD1234"), "ABCD-1234");
        assert_eq!(format.normalize("abcd-1234"), "ABCD1234");
        assert_eq!(format.normalize("ABCD 1234"), "ABCD1234");
    }

    #[test]
    fn test_device_code_status() {
        assert_eq!(DeviceCodeStatus::default(), DeviceCodeStatus::Pending);
    }
}
```

### Integration Tests
```rust
// src/auth/tests/integration.rs

use super::*;
use crate::auth::*;

mod auth_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration() {
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, AuthConfig::default());

        let user = service
            .register("test@example.com", "SecurePass123!", Some("Test User"))
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, Some("Test User".to_string()));
        assert_eq!(user.role, types::UserRole::User);
    }

    #[tokio::test]
    async fn test_user_authentication() {
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, AuthConfig::default());

        // Register
        service
            .register("test@example.com", "SecurePass123!", None)
            .await
            .unwrap();

        // Login
        let user = service
            .authenticate("test@example.com", "SecurePass123!")
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, AuthConfig::default());

        service
            .register("test@example.com", "SecurePass123!", None)
            .await
            .unwrap();

        let result = service
            .authenticate("test@example.com", "WrongPassword!")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_email() {
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, AuthConfig::default());

        service
            .register("test@example.com", "SecurePass123!", None)
            .await
            .unwrap();

        let result = service
            .register("test@example.com", "AnotherPass123!", None)
            .await;

        assert!(matches!(result, Err(AuthError::EmailAlreadyExists)));
    }

    #[tokio::test]
    async fn test_password_change() {
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, AuthConfig::default());

        let user = service
            .register("test@example.com", "OldPass123!", None)
            .await
            .unwrap();

        service
            .change_password(&user.id, "OldPass123!", "NewPass456!")
            .await
            .unwrap();

        // Old password should fail
        assert!(service
            .authenticate("test@example.com", "OldPass123!")
            .await
            .is_err());

        // New password should work
        assert!(service
            .authenticate("test@example.com", "NewPass456!")
            .await
            .is_ok());
    }
}

mod session_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let pool = setup_test_db().await;
        let store = SqliteSessionStore::new(pool.clone());
        let config = SessionConfig::default();

        // Create session
        let session = store
            .create("user-123", config.lifetime, None, None, None)
            .await
            .unwrap();

        assert_eq!(session.user_id, "user-123");

        // Get session
        let retrieved = store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, session.id);

        // Delete session
        store.delete(&session.id).await.unwrap();
        assert!(store.get(&session.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_session_touch() {
        let pool = setup_test_db().await;
        let store = SqliteSessionStore::new(pool);
        let config = SessionConfig::default();

        let session = store
            .create("user-123", config.lifetime, None, None, None)
            .await
            .unwrap();

        let original_active = session.last_active_at;

        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Touch session
        store.touch(&session.id).await.unwrap();

        let updated = store.get(&session.id).await.unwrap().unwrap();
        assert!(updated.last_active_at > original_active);
    }
}

mod refresh_token_tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh_token_rotation() {
        let pool = setup_test_db().await;
        let jwt_handler = jwt::JwtHandler::new(jwt::JwtConfig::default()).unwrap();
        let service = refresh::RefreshTokenService::new(
            pool,
            jwt_handler,
            refresh::RefreshConfig::default(),
        );

        // Create token
        let (token, stored) = service
            .create("user-123", None, None, None)
            .await
            .unwrap();

        assert!(!token.is_empty());
        assert_eq!(stored.user_id, "user-123");

        // Note: Full refresh test would require mock user lookup
    }
}

mod oauth_tests {
    use super::*;

    #[test]
    fn test_github_config() {
        let config = oauth::github::types::GitHubOAuthConfig {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            ..Default::default()
        };

        assert!(config.is_valid());

        let url = config.authorization_url("test-state");
        assert!(url.contains("client_id=test-client"));
        assert!(url.contains("state=test-state"));
    }

    #[test]
    fn test_google_config() {
        let config = oauth::google::types::GoogleOAuthConfig {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            ..Default::default()
        };

        assert!(config.is_valid());
        assert!(config.scopes.contains(&"openid".to_string()));
    }

    #[test]
    fn test_oauth_state() {
        let state = oauth::github::types::OAuthState::new("github", Some("/dashboard".to_string()));

        assert!(!state.is_expired());
        assert_eq!(state.provider, "github");
        assert_eq!(state.redirect_to, Some("/dashboard".to_string()));
    }
}

mod tenant_tests {
    use super::*;
    use crate::auth::tenant::*;

    #[tokio::test]
    async fn test_tenant_creation() {
        let pool = setup_test_db().await;
        let service = TenantService::new(pool, TenantConfig::default());

        let tenant = service
            .create("Test Company", "test-company", "owner-123", None)
            .await
            .unwrap();

        assert_eq!(tenant.name, "Test Company");
        assert_eq!(tenant.slug, "test-company");
        assert_eq!(tenant.status(), types::TenantStatus::Trial);
    }

    #[tokio::test]
    async fn test_tenant_membership() {
        let pool = setup_test_db().await;
        let service = TenantService::new(pool, TenantConfig::default());

        let tenant = service
            .create("Test Company", "test-company", "owner-123", Some("pro"))
            .await
            .unwrap();

        let membership = service
            .validate_access(&tenant.id, "owner-123")
            .await
            .unwrap();

        assert_eq!(membership.role, "owner");
    }
}
```

### Test Helpers
```rust
// src/auth/tests/helpers.rs

use super::*;
use crate::auth::*;

/// Create a test auth service with default configuration
pub async fn create_test_auth_service() -> AuthService {
    let pool = setup_test_db().await;
    AuthService::new(pool, AuthConfig::default())
}

/// Create a test user and return credentials
pub async fn create_test_user(service: &AuthService) -> TestUser {
    let user = TestUser::default();

    service
        .register(&user.email, &user.password, None)
        .await
        .expect("Failed to create test user");

    user
}

/// Create a test JWT handler
pub fn create_test_jwt_handler() -> jwt::JwtHandler {
    jwt::JwtHandler::new(jwt::JwtConfig {
        secret: "test-secret-key-at-least-32-characters-long".to_string(),
        ..Default::default()
    })
    .expect("Failed to create JWT handler")
}

/// Assert that an auth result is a specific error type
#[macro_export]
macro_rules! assert_auth_error {
    ($result:expr, $error:pat) => {
        match $result {
            Err($error) => (),
            Err(e) => panic!("Expected {} but got {:?}", stringify!($error), e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    };
}
```

## Test Coverage Areas
- Password hashing and validation
- JWT token generation and validation
- Session management
- OAuth state and callback handling
- Magic link generation and verification
- Device code flow
- Rate limiting
- Multi-tenant operations
- Permission checks
- Error handling

## Files to Create
- `src/auth/tests/mod.rs` - Test module exports
- `src/auth/tests/unit.rs` - Unit tests
- `src/auth/tests/integration.rs` - Integration tests
- `src/auth/tests/helpers.rs` - Test helpers
- `src/auth/tests/fixtures.rs` - Test fixtures
