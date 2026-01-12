# Spec 385: Authentication Integration Tests

## Phase
17 - Authentication/Authorization

## Spec ID
385

## Status
Planned

## Dependencies
- All Phase 17 Specs (366-384)

## Estimated Context
~12%

---

## Objective

Provide comprehensive integration tests for the authentication and authorization system. These tests validate that all components work together correctly, covering complete authentication flows, security scenarios, and edge cases.

---

## Acceptance Criteria

- [ ] Test complete login flow (username/password)
- [ ] Test session-based authentication
- [ ] Test JWT token authentication
- [ ] Test token refresh flow
- [ ] Test MFA enrollment and verification
- [ ] Test OAuth2 authentication flow
- [ ] Test API key authentication
- [ ] Test role-based access control
- [ ] Test permission checking
- [ ] Test rate limiting integration
- [ ] Test account lockout integration
- [ ] Test password reset flow
- [ ] Test audit logging integration

---

## Implementation Details

### Integration Test Suite

```rust
// tests/auth_integration.rs

use std::collections::HashSet;
use std::sync::Arc;

// Import all auth modules
use tachikoma::auth::{
    api_keys::{ApiKeyManager, CreateApiKeyRequest, InMemoryApiKeyStorage},
    audit::{AuditLogger, InMemoryAuditStorage},
    config::*,
    events::{AuthEvent, AuthEventBus, AuthEventEmitter},
    guards::{GuardBuilder, RoleGuard},
    local::{LocalAuthProvider, InMemoryUserRepository},
    lockout::{LockoutManager, InMemoryLockoutStorage},
    mfa::{MfaManager, InMemoryMfaStorage},
    middleware::{AuthState, Auth, RequireAuth},
    oauth::{OAuth2Manager, InMemoryOAuth2StateStorage, InMemoryLinkedIdentityStorage},
    password::{PasswordHasher, PasswordValidator},
    permissions::{PermissionManager, InMemoryPermissionStorage},
    rate_limit::{FixedWindowRateLimiter, InMemoryRateLimitStorage, RateLimitKey, RateLimitAction},
    recovery::{RecoveryManager, InMemoryResetTokenStorage, MockEmailSender, PasswordResetRequest},
    refresh::{RefreshTokenManager, InMemoryRefreshTokenStorage},
    roles::{RoleManager, InMemoryRoleStorage, SystemRoles},
    session::{SessionManager, InMemorySessionStorage},
    tokens::TokenManager,
    types::*,
};

/// Test fixture for setting up auth system
struct AuthTestFixture {
    event_bus: Arc<AuthEventBus>,
    user_repository: Arc<InMemoryUserRepository>,
    session_manager: Arc<SessionManager>,
    token_manager: Arc<TokenManager>,
    refresh_manager: Arc<RefreshTokenManager>,
    local_auth: Arc<LocalAuthProvider>,
    mfa_manager: Arc<MfaManager>,
    api_key_manager: Arc<ApiKeyManager>,
    role_manager: Arc<RoleManager>,
    permission_manager: Arc<PermissionManager>,
    rate_limiter: Arc<FixedWindowRateLimiter>,
    lockout_manager: Arc<LockoutManager>,
    recovery_manager: Arc<RecoveryManager>,
    audit_logger: Arc<AuditLogger>,
    config: AuthConfig,
}

impl AuthTestFixture {
    async fn new() -> Self {
        let config = AuthConfig::default();

        // Event bus
        let event_bus = Arc::new(AuthEventBus::new(1000));

        // User repository
        let user_repository = Arc::new(InMemoryUserRepository::new());

        // Password hasher
        let password_hasher = Arc::new(PasswordHasher::new(config.password.clone()));

        // Session manager
        let session_storage = Arc::new(InMemorySessionStorage::new());
        let session_manager = Arc::new(SessionManager::new(
            session_storage,
            config.session.clone(),
            event_bus.clone(),
        ));

        // Token manager
        let mut token_config = config.tokens.clone();
        token_config.secret_key = "test-secret-key-at-least-32-bytes-long".to_string();
        let token_manager = Arc::new(TokenManager::new_hmac(token_config.clone()).unwrap());

        // Refresh token manager
        let refresh_storage = Arc::new(InMemoryRefreshTokenStorage::new());
        let refresh_manager = Arc::new(RefreshTokenManager::new(
            refresh_storage,
            token_manager.clone(),
            user_repository.clone(),
            event_bus.clone(),
            token_config,
        ));

        // Role manager
        let role_storage = Arc::new(InMemoryRoleStorage::new());
        let role_manager = Arc::new(RoleManager::new(role_storage, event_bus.clone()));
        role_manager.initialize().await.unwrap();

        // Permission manager
        let permission_storage = Arc::new(InMemoryPermissionStorage::new());
        let permission_manager = Arc::new(PermissionManager::new(
            permission_storage,
            role_manager.clone(),
            event_bus.clone(),
        ));

        // Lockout manager
        let lockout_storage = Arc::new(InMemoryLockoutStorage::new());
        let lockout_manager = Arc::new(LockoutManager::new(
            lockout_storage,
            event_bus.clone(),
            config.lockout.clone(),
        ));

        // Local auth provider
        let local_auth = Arc::new(LocalAuthProvider::new(
            user_repository.clone(),
            password_hasher.clone(),
            lockout_manager.clone(),
            event_bus.clone(),
            config.password.clone(),
        ));

        // MFA manager
        let mfa_storage = Arc::new(InMemoryMfaStorage::new());
        let mfa_manager = Arc::new(MfaManager::new(
            mfa_storage,
            event_bus.clone(),
            config.mfa.clone(),
        ));

        // API key manager
        let api_key_storage = Arc::new(InMemoryApiKeyStorage::new());
        let api_key_manager = Arc::new(ApiKeyManager::new(
            api_key_storage,
            user_repository.clone(),
            event_bus.clone(),
            config.api_keys.clone(),
        ));

        // Rate limiter
        let rate_limit_storage = Arc::new(InMemoryRateLimitStorage::new());
        let rate_limiter = Arc::new(FixedWindowRateLimiter::new(
            rate_limit_storage,
            &config.rate_limit,
        ));

        // Recovery manager
        let reset_token_storage = Arc::new(InMemoryResetTokenStorage::new());
        let email_sender = Arc::new(MockEmailSender::new());
        let recovery_manager = Arc::new(RecoveryManager::new(
            reset_token_storage,
            user_repository.clone(),
            password_hasher,
            email_sender,
            event_bus.clone(),
            Default::default(),
        ));

        // Audit logger
        let audit_storage = Arc::new(InMemoryAuditStorage::new());
        let audit_logger = Arc::new(AuditLogger::new(audit_storage, config.audit.clone()));

        Self {
            event_bus,
            user_repository,
            session_manager,
            token_manager,
            refresh_manager,
            local_auth,
            mfa_manager,
            api_key_manager,
            role_manager,
            permission_manager,
            rate_limiter,
            lockout_manager,
            recovery_manager,
            audit_logger,
            config,
        }
    }

    /// Create a test user
    async fn create_user(&self, username: &str, password: &str) -> User {
        self.local_auth
            .register(username, Some(&format!("{}@test.com", username)), password)
            .await
            .unwrap()
    }

    /// Login a user and get identity
    async fn login(&self, username: &str, password: &str) -> AuthResult<AuthIdentity> {
        self.local_auth
            .login(username, password, &AuthMetadata::default())
            .await
    }
}

// ============================================================================
// Complete Login Flow Tests
// ============================================================================

mod login_flow_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_password_login_flow() {
        let fixture = AuthTestFixture::new().await;

        // Register user
        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;
        assert!(!user.id.to_string().is_empty());

        // Login
        let identity = fixture.login("testuser", "SecureP@ssw0rd!").await.unwrap();
        assert_eq!(identity.username, "testuser");
        assert_eq!(identity.auth_method, AuthMethod::Password);

        // Create session
        let session = fixture
            .session_manager
            .create_session(identity.user_id, &AuthMetadata::default())
            .await
            .unwrap();

        // Create tokens
        let token_pair = fixture.token_manager.create_token_pair(&identity).unwrap();
        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());

        // Validate token
        let validated = fixture
            .token_manager
            .validate_access_token(&token_pair.access_token)
            .await
            .unwrap();
        assert_eq!(validated.user_id, identity.user_id);
    }

    #[tokio::test]
    async fn test_login_with_wrong_password() {
        let fixture = AuthTestFixture::new().await;

        fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        let result = fixture.login("testuser", "wrongpassword").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let fixture = AuthTestFixture::new().await;

        let result = fixture.login("nonexistent", "password").await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }
}

// ============================================================================
// Token Refresh Tests
// ============================================================================

mod token_refresh_tests {
    use super::*;

    #[tokio::test]
    async fn test_token_refresh_flow() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Create initial refresh token
        let (raw_token, _) = fixture
            .refresh_manager
            .create_refresh_token(user.id, None, &AuthMetadata::default())
            .await
            .unwrap();

        // Refresh tokens
        let new_pair = fixture
            .refresh_manager
            .refresh(&raw_token, &AuthMetadata::default())
            .await
            .unwrap();

        assert!(!new_pair.access_token.is_empty());
        assert!(!new_pair.refresh_token.is_empty());
        assert_ne!(raw_token, new_pair.refresh_token);
    }

    #[tokio::test]
    async fn test_token_reuse_detection() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        let (raw_token, _) = fixture
            .refresh_manager
            .create_refresh_token(user.id, None, &AuthMetadata::default())
            .await
            .unwrap();

        // First refresh should succeed
        fixture
            .refresh_manager
            .refresh(&raw_token, &AuthMetadata::default())
            .await
            .unwrap();

        // Second use of same token should fail (reuse detection)
        let result = fixture
            .refresh_manager
            .refresh(&raw_token, &AuthMetadata::default())
            .await;

        assert!(result.is_err());
    }
}

// ============================================================================
// MFA Tests
// ============================================================================

mod mfa_tests {
    use super::*;

    #[tokio::test]
    async fn test_mfa_enrollment_flow() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Start enrollment
        let setup = fixture
            .mfa_manager
            .start_enrollment(user.id, "testuser@test.com")
            .await
            .unwrap();

        assert!(!setup.secret.is_empty());
        assert!(!setup.backup_codes.is_empty());
        assert!(setup.uri.contains("otpauth://totp/"));

        // Generate valid TOTP code
        let code = fixture
            .mfa_manager
            .totp_provider
            .generate_code(&setup.secret)
            .unwrap();

        // Complete enrollment
        fixture
            .mfa_manager
            .complete_enrollment(user.id, &code)
            .await
            .unwrap();

        // Verify MFA is required
        assert!(fixture.mfa_manager.is_required(user.id).await);
    }

    #[tokio::test]
    async fn test_mfa_verification() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Setup MFA
        let setup = fixture
            .mfa_manager
            .start_enrollment(user.id, "testuser@test.com")
            .await
            .unwrap();
        let code = fixture
            .mfa_manager
            .totp_provider
            .generate_code(&setup.secret)
            .unwrap();
        fixture
            .mfa_manager
            .complete_enrollment(user.id, &code)
            .await
            .unwrap();

        // Create challenge
        let challenge = fixture.mfa_manager.create_challenge(user.id).await.unwrap();

        // Verify with TOTP
        let new_code = fixture
            .mfa_manager
            .totp_provider
            .generate_code(&setup.secret)
            .unwrap();
        let verified_user = fixture
            .mfa_manager
            .verify(&challenge.id, &new_code, crate::auth::mfa::MfaCodeType::Totp)
            .await
            .unwrap();

        assert_eq!(verified_user, user.id);
    }

    #[tokio::test]
    async fn test_backup_code_usage() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Setup MFA
        let setup = fixture
            .mfa_manager
            .start_enrollment(user.id, "testuser@test.com")
            .await
            .unwrap();
        let code = fixture
            .mfa_manager
            .totp_provider
            .generate_code(&setup.secret)
            .unwrap();
        fixture
            .mfa_manager
            .complete_enrollment(user.id, &code)
            .await
            .unwrap();

        // Create challenge
        let challenge = fixture.mfa_manager.create_challenge(user.id).await.unwrap();

        // Use backup code
        let backup_code = &setup.backup_codes[0];
        let verified_user = fixture
            .mfa_manager
            .verify(&challenge.id, backup_code, crate::auth::mfa::MfaCodeType::Backup)
            .await
            .unwrap();

        assert_eq!(verified_user, user.id);

        // Same backup code should not work again
        let challenge2 = fixture.mfa_manager.create_challenge(user.id).await.unwrap();
        let result = fixture
            .mfa_manager
            .verify(&challenge2.id, backup_code, crate::auth::mfa::MfaCodeType::Backup)
            .await;

        assert!(result.is_err());
    }
}

// ============================================================================
// API Key Tests
// ============================================================================

mod api_key_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_authentication() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Create API key
        let created = fixture
            .api_key_manager
            .create_key(
                user.id,
                CreateApiKeyRequest {
                    name: "Test Key".to_string(),
                    scopes: Some(vec!["read".to_string()]),
                    expires_in_days: Some(30),
                    allowed_ips: None,
                    metadata: None,
                },
            )
            .await
            .unwrap();

        // Validate key
        let identity = fixture
            .api_key_manager
            .validate_key(&created.raw_key)
            .await
            .unwrap();

        assert_eq!(identity.user_id, user.id);
        assert_eq!(identity.auth_method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_api_key_revocation() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        let created = fixture
            .api_key_manager
            .create_key(
                user.id,
                CreateApiKeyRequest {
                    name: "Test Key".to_string(),
                    scopes: None,
                    expires_in_days: None,
                    allowed_ips: None,
                    metadata: None,
                },
            )
            .await
            .unwrap();

        // Revoke key
        fixture
            .api_key_manager
            .revoke_key(user.id, &created.key.id)
            .await
            .unwrap();

        // Should fail to validate
        let result = fixture.api_key_manager.validate_key(&created.raw_key).await;
        assert!(result.is_err());
    }
}

// ============================================================================
// Role & Permission Tests
// ============================================================================

mod rbac_tests {
    use super::*;

    #[tokio::test]
    async fn test_role_assignment() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Assign roles
        fixture
            .role_manager
            .assign_roles(user.id, vec!["user".to_string(), "moderator".to_string()])
            .await
            .unwrap();

        // Check roles
        let roles = fixture.role_manager.get_user_roles(user.id).await.unwrap();
        assert_eq!(roles.len(), 2);

        let role_ids: Vec<_> = roles.iter().map(|r| r.id.as_str()).collect();
        assert!(role_ids.contains(&"user"));
        assert!(role_ids.contains(&"moderator"));
    }

    #[tokio::test]
    async fn test_permission_inheritance() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Assign moderator role (inherits from user)
        fixture
            .role_manager
            .assign_roles(user.id, vec!["moderator".to_string()])
            .await
            .unwrap();

        // Check permissions
        let permissions = fixture
            .permission_manager
            .get_user_permissions(user.id)
            .await
            .unwrap();

        // Should have moderator permissions
        assert!(permissions.contains("content:moderate"));

        // Should also have inherited user permissions
        assert!(permissions.contains("profile:read"));
    }

    #[tokio::test]
    async fn test_direct_permission_grant() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Grant direct permission
        fixture
            .permission_manager
            .grant_permission(
                user.id,
                crate::auth::permissions::Permission::new("special", "access"),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Check permission
        let has_perm = fixture
            .permission_manager
            .check_permission_str(user.id, "special:access")
            .await
            .unwrap();

        assert!(has_perm);
    }

    #[tokio::test]
    async fn test_guard_evaluation() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("admin", "SecureP@ssw0rd!").await;
        fixture
            .role_manager
            .assign_roles(user.id, vec!["admin".to_string()])
            .await
            .unwrap();

        let identity = fixture.login("admin", "SecureP@ssw0rd!").await.unwrap();

        // Create guard
        let guard = GuardBuilder::new()
            .authenticated()
            .role("admin")
            .build();

        // Check guard
        let result = guard.check(&identity).await;
        assert!(result.is_allowed());
    }
}

// ============================================================================
// Rate Limiting & Lockout Tests
// ============================================================================

mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting() {
        let fixture = AuthTestFixture::new().await;

        let key = RateLimitKey::IpAction("192.168.1.1".to_string(), RateLimitAction::Login);

        // Use up rate limit
        for _ in 0..5 {
            let result = fixture.rate_limiter.record(&key).await;
            if !result.allowed {
                break;
            }
        }

        // Next request should be denied
        let result = fixture.rate_limiter.record(&key).await;
        assert!(!result.allowed);
        assert!(result.retry_after.is_some());
    }

    #[tokio::test]
    async fn test_account_lockout() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Fail login multiple times
        for _ in 0..5 {
            let _ = fixture.login("testuser", "wrongpassword").await;
        }

        // Account should be locked
        let result = fixture.lockout_manager.check_locked(&user).await;
        assert!(matches!(result, Err(AuthError::AccountLocked)));
    }

    #[tokio::test]
    async fn test_lockout_reset_on_success() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Fail a few times (not enough to lock)
        for _ in 0..3 {
            let _ = fixture.login("testuser", "wrongpassword").await;
        }

        // Successful login should reset counter
        fixture.login("testuser", "SecureP@ssw0rd!").await.unwrap();

        // Should be able to fail again without immediate lockout
        for _ in 0..3 {
            let _ = fixture.login("testuser", "wrongpassword").await;
        }

        // Still not locked
        let status = fixture.lockout_manager.get_status(user.id).await.unwrap();
        assert!(status.is_none() || !status.unwrap().is_locked());
    }
}

// ============================================================================
// Password Recovery Tests
// ============================================================================

mod recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_password_reset_flow() {
        let fixture = AuthTestFixture::new().await;

        fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Request reset
        fixture
            .recovery_manager
            .request_reset(
                &PasswordResetRequest {
                    email: "testuser@test.com".to_string(),
                },
                &AuthMetadata::default(),
            )
            .await
            .unwrap();

        // In real test, we'd get the token from the mock email sender
        // and complete the reset
    }
}

// ============================================================================
// Audit Logging Tests
// ============================================================================

mod audit_tests {
    use super::*;
    use crate::auth::audit::{AuditEntry, AuditEventType, AuditQuery};

    #[tokio::test]
    async fn test_audit_log_creation() {
        let fixture = AuthTestFixture::new().await;

        let entry = AuditEntry::new(AuditEventType::LoginSuccess, "User logged in")
            .with_user(UserId::new(), Some("testuser".to_string()))
            .with_success(true);

        fixture.audit_logger.log(entry).await.unwrap();

        let results = fixture
            .audit_logger
            .query(AuditQuery::default())
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, AuditEventType::LoginSuccess);
    }
}

// ============================================================================
// Event System Tests
// ============================================================================

mod event_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_emission_on_login() {
        let fixture = AuthTestFixture::new().await;

        // Subscribe to events
        let mut receiver = fixture.event_bus.subscribe();

        fixture.create_user("testuser", "SecureP@ssw0rd!").await;
        fixture.login("testuser", "SecureP@ssw0rd!").await.unwrap();

        // Should receive login success event
        let event = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            receiver.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(event.event_type(), "login_success");
    }
}

// ============================================================================
// Session Management Tests
// ============================================================================

mod session_tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Create session
        let session = fixture
            .session_manager
            .create_session(user.id, &AuthMetadata::default())
            .await
            .unwrap();

        // Validate session
        let validated = fixture
            .session_manager
            .validate_session(session.id)
            .await
            .unwrap();
        assert_eq!(validated.user_id, user.id);

        // Destroy session
        fixture
            .session_manager
            .destroy_session(session.id)
            .await
            .unwrap();

        // Should be invalid now
        let result = fixture.session_manager.get_session(session.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_session_regeneration() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        let session = fixture
            .session_manager
            .create_session(user.id, &AuthMetadata::default())
            .await
            .unwrap();

        let old_id = session.id;

        // Regenerate
        let new_session = fixture
            .session_manager
            .regenerate_session(session.id)
            .await
            .unwrap();

        // IDs should be different
        assert_ne!(old_id, new_session.id);

        // Old session should be invalid
        let result = fixture.session_manager.get_session(old_id).await.unwrap();
        assert!(result.is_none());

        // New session should be valid
        let result = fixture.session_manager.get_session(new_session.id).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_session_limit() {
        let fixture = AuthTestFixture::new().await;

        let user = fixture.create_user("testuser", "SecureP@ssw0rd!").await;

        // Create sessions up to limit
        let mut sessions = Vec::new();
        for _ in 0..6 {
            let session = fixture
                .session_manager
                .create_session(user.id, &AuthMetadata::default())
                .await
                .unwrap();
            sessions.push(session);
        }

        // Get active sessions
        let active = fixture
            .session_manager
            .get_user_sessions(user.id)
            .await
            .unwrap();

        // Should be limited (default is 5)
        assert!(active.len() <= 5);
    }
}
```

---

## Testing Requirements

### Running the Tests

```bash
# Run all auth integration tests
cargo test --test auth_integration

# Run specific test module
cargo test --test auth_integration login_flow_tests

# Run with verbose output
cargo test --test auth_integration -- --nocapture

# Run with specific features
cargo test --test auth_integration --features "redis"
```

### Test Coverage Requirements

- All public APIs should have test coverage
- Security-critical paths should have multiple test cases
- Edge cases and error conditions should be tested
- Integration between components should be verified

---

## Related Specs

- **All Phase 17 Specs**: This spec tests all components
- Tests serve as documentation for expected behavior
- Tests validate that specifications are correctly implemented
