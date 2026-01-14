# Spec 389: Multi-Tenant Authentication

## Overview
Implement multi-tenant authentication support for SaaS deployments with tenant isolation.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### Multi-Tenant Types
```rust
// src/auth/tenant/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    PendingSetup,
    Trial,
    Expired,
    Deleted,
}

impl TenantStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::PendingSetup => "pending_setup",
            Self::Trial => "trial",
            Self::Expired => "expired",
            Self::Deleted => "deleted",
        }
    }

    pub fn can_login(&self) -> bool {
        matches!(self, Self::Active | Self::Trial | Self::PendingSetup)
    }
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub slug: String,  // URL-friendly identifier
    pub domain: Option<String>,  // Custom domain
    pub status: String,
    pub plan: Option<String>,
    pub owner_id: Option<String>,
    pub settings: Option<String>,  // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspended_reason: Option<String>,
}

impl Tenant {
    pub fn status(&self) -> TenantStatus {
        match self.status.as_str() {
            "active" => TenantStatus::Active,
            "suspended" => TenantStatus::Suspended,
            "pending_setup" => TenantStatus::PendingSetup,
            "trial" => TenantStatus::Trial,
            "expired" => TenantStatus::Expired,
            "deleted" => TenantStatus::Deleted,
            _ => TenantStatus::Active,
        }
    }

    pub fn can_login(&self) -> bool {
        self.status().can_login()
    }

    pub fn is_trial_expired(&self) -> bool {
        self.trial_ends_at
            .map(|ends| Utc::now() > ends)
            .unwrap_or(false)
    }

    pub fn settings(&self) -> TenantSettings {
        self.settings
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}

/// Tenant settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantSettings {
    /// Allowed authentication methods
    pub auth_methods: Vec<String>,
    /// Require email domain for signup
    pub allowed_email_domains: Option<Vec<String>>,
    /// Enable SSO
    pub sso_enabled: bool,
    /// SSO provider
    pub sso_provider: Option<String>,
    /// SSO configuration (provider-specific)
    pub sso_config: Option<serde_json::Value>,
    /// Session timeout (minutes)
    pub session_timeout: Option<i32>,
    /// Require MFA
    pub require_mfa: bool,
    /// Custom branding
    pub branding: Option<TenantBranding>,
    /// Feature flags
    pub features: HashMap<String, bool>,
}

/// Tenant branding settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub app_name: Option<String>,
    pub support_email: Option<String>,
}

/// Tenant membership (user-tenant relationship)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantMembership {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,  // TenantRole
    pub permissions: Option<String>,  // JSON array
    pub invited_by: Option<String>,
    pub invited_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,
    pub status: String,  // pending, active, suspended
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tenant role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantRole {
    Owner,
    Admin,
    Member,
    Viewer,
    Custom,
}

impl TenantRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "member" => Self::Member,
            "viewer" => Self::Viewer,
            _ => Self::Custom,
        }
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_manage_settings(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }
}

/// Tenant invitation
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantInvitation {
    pub id: String,
    pub tenant_id: String,
    pub email: String,
    pub role: String,
    pub token_hash: String,
    pub invited_by: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TenantInvitation {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted_at.is_some()
    }
}

/// Tenant errors
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Tenant not found")]
    NotFound,

    #[error("Tenant suspended: {0}")]
    Suspended(String),

    #[error("Tenant disabled")]
    Disabled,

    #[error("Tenant trial expired")]
    TrialExpired,

    #[error("User not a member of tenant")]
    NotAMember,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Invitation expired")]
    InvitationExpired,

    #[error("Invitation already used")]
    InvitationAlreadyUsed,

    #[error("Invalid invitation")]
    InvalidInvitation,

    #[error("User already member")]
    AlreadyMember,

    #[error("Cannot remove last owner")]
    CannotRemoveLastOwner,

    #[error("Domain already taken")]
    DomainTaken,

    #[error("Slug already taken")]
    SlugTaken,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Multi-Tenant Service
```rust
// src/auth/tenant/service.rs

use super::types::*;
use chrono::{Duration, Utc};
use sqlx::sqlite::SqlitePool;
use tracing::{debug, info, warn, instrument};
use uuid::Uuid;

/// Multi-tenant service
pub struct TenantService {
    pool: SqlitePool,
    config: TenantConfig,
}

/// Tenant configuration
#[derive(Debug, Clone)]
pub struct TenantConfig {
    /// Default trial duration
    pub trial_duration: Duration,
    /// Invitation expiry
    pub invitation_expiry: Duration,
    /// Max members per tenant (free plan)
    pub max_members_free: usize,
    /// Allow custom domains
    pub allow_custom_domains: bool,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            trial_duration: Duration::days(14),
            invitation_expiry: Duration::days(7),
            max_members_free: 5,
            allow_custom_domains: true,
        }
    }
}

impl TenantService {
    pub fn new(pool: SqlitePool, config: TenantConfig) -> Self {
        Self { pool, config }
    }

    /// Create a new tenant
    #[instrument(skip(self))]
    pub async fn create(
        &self,
        name: &str,
        slug: &str,
        owner_id: &str,
        plan: Option<&str>,
    ) -> Result<Tenant, TenantError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let trial_ends = plan.is_none().then(|| now + self.config.trial_duration);
        let status = if plan.is_some() {
            TenantStatus::Active
        } else {
            TenantStatus::Trial
        };

        // Check slug availability
        let existing = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM tenants WHERE slug = ?"
        )
        .bind(slug)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(TenantError::SlugTaken);
        }

        // Create tenant
        sqlx::query(r#"
            INSERT INTO tenants (id, name, slug, status, plan, owner_id, created_at, updated_at, trial_ends_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(name)
        .bind(slug)
        .bind(status.as_str())
        .bind(plan)
        .bind(owner_id)
        .bind(now)
        .bind(now)
        .bind(trial_ends)
        .execute(&self.pool)
        .await?;

        // Add owner as member
        let membership_id = Uuid::new_v4().to_string();
        sqlx::query(r#"
            INSERT INTO tenant_memberships (id, tenant_id, user_id, role, status, joined_at, created_at, updated_at)
            VALUES (?, ?, ?, 'owner', 'active', ?, ?, ?)
        "#)
        .bind(&membership_id)
        .bind(&id)
        .bind(owner_id)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        info!("Created tenant {} with owner {}", id, owner_id);
        self.get_by_id(&id).await
    }

    /// Get tenant by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Tenant, TenantError> {
        sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(TenantError::NotFound)
    }

    /// Get tenant by slug
    pub async fn get_by_slug(&self, slug: &str) -> Result<Tenant, TenantError> {
        sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE slug = ?")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(TenantError::NotFound)
    }

    /// Get tenant by domain
    pub async fn get_by_domain(&self, domain: &str) -> Result<Option<Tenant>, TenantError> {
        let tenant = sqlx::query_as::<_, Tenant>("SELECT * FROM tenants WHERE domain = ?")
            .bind(domain)
            .fetch_optional(&self.pool)
            .await?;

        Ok(tenant)
    }

    /// Validate tenant access
    pub async fn validate_access(&self, tenant_id: &str, user_id: &str) -> Result<TenantMembership, TenantError> {
        // Get tenant
        let tenant = self.get_by_id(tenant_id).await?;

        // Check tenant status
        if !tenant.can_login() {
            if tenant.is_trial_expired() {
                return Err(TenantError::TrialExpired);
            }
            match tenant.status() {
                TenantStatus::Suspended => {
                    return Err(TenantError::Suspended(
                        tenant.suspended_reason.unwrap_or_default()
                    ));
                }
                _ => return Err(TenantError::Disabled),
            }
        }

        // Check membership
        let membership = sqlx::query_as::<_, TenantMembership>(
            "SELECT * FROM tenant_memberships WHERE tenant_id = ? AND user_id = ? AND status = 'active'"
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(TenantError::NotAMember)?;

        Ok(membership)
    }

    /// Get user's tenants
    pub async fn get_user_tenants(&self, user_id: &str) -> Result<Vec<(Tenant, TenantMembership)>, TenantError> {
        let rows = sqlx::query_as::<_, (Tenant, TenantMembership)>(r#"
            SELECT t.*, m.*
            FROM tenants t
            INNER JOIN tenant_memberships m ON t.id = m.tenant_id
            WHERE m.user_id = ? AND m.status = 'active' AND t.status != 'deleted'
            ORDER BY t.name
        "#)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get tenant members
    pub async fn get_members(&self, tenant_id: &str) -> Result<Vec<TenantMembership>, TenantError> {
        let members = sqlx::query_as::<_, TenantMembership>(
            "SELECT * FROM tenant_memberships WHERE tenant_id = ? ORDER BY role, joined_at"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// Invite user to tenant
    #[instrument(skip(self))]
    pub async fn invite(
        &self,
        tenant_id: &str,
        email: &str,
        role: TenantRole,
        invited_by: &str,
    ) -> Result<(TenantInvitation, String), TenantError> {
        // Verify inviter has permission
        let inviter_membership = self.validate_access(tenant_id, invited_by).await?;
        if !TenantRole::from_str(&inviter_membership.role).can_manage_members() {
            return Err(TenantError::InsufficientPermissions);
        }

        // Check if already member
        let existing = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM tenant_memberships m INNER JOIN users u ON m.user_id = u.id WHERE m.tenant_id = ? AND u.email = ?"
        )
        .bind(tenant_id)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(TenantError::AlreadyMember);
        }

        let id = Uuid::new_v4().to_string();
        let token = self.generate_token();
        let token_hash = self.hash_token(&token);
        let expires_at = Utc::now() + self.config.invitation_expiry;

        sqlx::query(r#"
            INSERT INTO tenant_invitations (id, tenant_id, email, role, token_hash, invited_by, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
        "#)
        .bind(&id)
        .bind(tenant_id)
        .bind(email)
        .bind(role.as_str())
        .bind(&token_hash)
        .bind(invited_by)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        let invitation = sqlx::query_as::<_, TenantInvitation>(
            "SELECT * FROM tenant_invitations WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        info!("Invited {} to tenant {} as {:?}", email, tenant_id, role);
        Ok((invitation, token))
    }

    /// Accept invitation
    pub async fn accept_invitation(&self, token: &str, user_id: &str) -> Result<TenantMembership, TenantError> {
        let token_hash = self.hash_token(token);

        // Find invitation
        let invitation = sqlx::query_as::<_, TenantInvitation>(
            "SELECT * FROM tenant_invitations WHERE token_hash = ?"
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(TenantError::InvalidInvitation)?;

        if invitation.is_expired() {
            return Err(TenantError::InvitationExpired);
        }

        if invitation.is_accepted() {
            return Err(TenantError::InvitationAlreadyUsed);
        }

        let now = Utc::now();
        let membership_id = Uuid::new_v4().to_string();

        // Create membership
        sqlx::query(r#"
            INSERT INTO tenant_memberships (id, tenant_id, user_id, role, status, invited_by, invited_at, joined_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'active', ?, ?, ?, ?, ?)
        "#)
        .bind(&membership_id)
        .bind(&invitation.tenant_id)
        .bind(user_id)
        .bind(&invitation.role)
        .bind(&invitation.invited_by)
        .bind(invitation.created_at)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Mark invitation as accepted
        sqlx::query("UPDATE tenant_invitations SET accepted_at = ? WHERE id = ?")
            .bind(now)
            .bind(&invitation.id)
            .execute(&self.pool)
            .await?;

        let membership = sqlx::query_as::<_, TenantMembership>(
            "SELECT * FROM tenant_memberships WHERE id = ?"
        )
        .bind(&membership_id)
        .fetch_one(&self.pool)
        .await?;

        info!("User {} accepted invitation to tenant {}", user_id, invitation.tenant_id);
        Ok(membership)
    }

    /// Remove member from tenant
    pub async fn remove_member(
        &self,
        tenant_id: &str,
        user_id: &str,
        removed_by: &str,
    ) -> Result<(), TenantError> {
        // Verify remover has permission
        let remover_membership = self.validate_access(tenant_id, removed_by).await?;
        if !TenantRole::from_str(&remover_membership.role).can_manage_members() {
            return Err(TenantError::InsufficientPermissions);
        }

        // Check if trying to remove last owner
        let target = sqlx::query_as::<_, TenantMembership>(
            "SELECT * FROM tenant_memberships WHERE tenant_id = ? AND user_id = ?"
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(TenantError::NotAMember)?;

        if target.role == "owner" {
            let owner_count: i32 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM tenant_memberships WHERE tenant_id = ? AND role = 'owner'"
            )
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await?;

            if owner_count <= 1 {
                return Err(TenantError::CannotRemoveLastOwner);
            }
        }

        sqlx::query("DELETE FROM tenant_memberships WHERE tenant_id = ? AND user_id = ?")
            .bind(tenant_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        info!("Removed user {} from tenant {} by {}", user_id, tenant_id, removed_by);
        Ok(())
    }

    /// Update member role
    pub async fn update_member_role(
        &self,
        tenant_id: &str,
        user_id: &str,
        new_role: TenantRole,
        updated_by: &str,
    ) -> Result<TenantMembership, TenantError> {
        // Verify updater has permission
        let updater_membership = self.validate_access(tenant_id, updated_by).await?;
        let updater_role = TenantRole::from_str(&updater_membership.role);

        if !updater_role.can_manage_members() {
            return Err(TenantError::InsufficientPermissions);
        }

        // Only owners can create new owners
        if new_role == TenantRole::Owner && updater_role != TenantRole::Owner {
            return Err(TenantError::InsufficientPermissions);
        }

        sqlx::query(
            "UPDATE tenant_memberships SET role = ?, updated_at = datetime('now') WHERE tenant_id = ? AND user_id = ?"
        )
        .bind(new_role.as_str())
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let membership = sqlx::query_as::<_, TenantMembership>(
            "SELECT * FROM tenant_memberships WHERE tenant_id = ? AND user_id = ?"
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(membership)
    }

    fn generate_token(&self) -> String {
        use rand::Rng;
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn hash_token(&self, token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Tenant database schema
pub fn tenant_migration_sql() -> &'static str {
    r#"
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    domain TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'active',
    plan TEXT,
    owner_id TEXT,
    settings TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    trial_ends_at TEXT,
    suspended_at TEXT,
    suspended_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_tenants_slug ON tenants(slug);
CREATE INDEX IF NOT EXISTS idx_tenants_domain ON tenants(domain);
CREATE INDEX IF NOT EXISTS idx_tenants_owner ON tenants(owner_id);

CREATE TABLE IF NOT EXISTS tenant_memberships (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member',
    permissions TEXT,
    invited_by TEXT,
    invited_at TEXT,
    joined_at TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_memberships_tenant ON tenant_memberships(tenant_id);
CREATE INDEX IF NOT EXISTS idx_memberships_user ON tenant_memberships(user_id);

CREATE TABLE IF NOT EXISTS tenant_invitations (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    invited_by TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    accepted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_invitations_tenant ON tenant_invitations(tenant_id);
CREATE INDEX IF NOT EXISTS idx_invitations_token ON tenant_invitations(token_hash);
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_role() {
        assert!(TenantRole::Owner.can_manage_members());
        assert!(TenantRole::Admin.can_manage_members());
        assert!(!TenantRole::Member.can_manage_members());
    }

    #[test]
    fn test_tenant_status() {
        assert!(TenantStatus::Active.can_login());
        assert!(TenantStatus::Trial.can_login());
        assert!(!TenantStatus::Suspended.can_login());
    }
}
```

## Files to Create
- `src/auth/tenant/types.rs` - Multi-tenant types
- `src/auth/tenant/service.rs` - Multi-tenant service
- `src/auth/tenant/mod.rs` - Module exports
