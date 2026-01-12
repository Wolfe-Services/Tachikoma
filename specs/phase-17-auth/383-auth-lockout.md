# Spec 383: Account Lockout

## Phase
17 - Authentication/Authorization

## Spec ID
383

## Status
Planned

## Dependencies
- Spec 366: Auth Types and Traits
- Spec 367: Auth Configuration

## Estimated Context
~8%

---

## Objective

Implement account lockout functionality to protect against brute force attacks. When a user exceeds the maximum number of failed login attempts, their account should be temporarily locked. The system should support progressive lockout (increasing duration with repeated lockouts) and automatic unlock after the lockout period.

---

## Acceptance Criteria

- [ ] Track failed login attempts per user
- [ ] Lock account after max failed attempts
- [ ] Implement lockout duration configuration
- [ ] Support progressive lockout (increasing duration)
- [ ] Automatic unlock after lockout period
- [ ] Manual unlock by administrator
- [ ] Reset failed attempts on successful login
- [ ] Emit events for lockout/unlock

---

## Implementation Details

### Account Lockout System

```rust
// src/auth/lockout.rs

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

use crate::auth::{
    config::LockoutConfig,
    events::{AuthEvent, AuthEventEmitter},
    provider::User,
    types::*,
};

/// Lockout status for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutStatus {
    /// User ID
    pub user_id: UserId,

    /// Number of failed attempts
    pub failed_attempts: u32,

    /// When the last failed attempt occurred
    pub last_failed_at: Option<DateTime<Utc>>,

    /// Whether currently locked
    pub locked: bool,

    /// When the lockout started
    pub locked_at: Option<DateTime<Utc>>,

    /// When the lockout ends
    pub locked_until: Option<DateTime<Utc>>,

    /// Number of times account has been locked
    pub lockout_count: u32,

    /// IP addresses of failed attempts
    pub failed_ips: Vec<String>,
}

impl LockoutStatus {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            failed_attempts: 0,
            last_failed_at: None,
            locked: false,
            locked_at: None,
            locked_until: None,
            lockout_count: 0,
            failed_ips: Vec::new(),
        }
    }

    /// Check if currently locked
    pub fn is_locked(&self) -> bool {
        if !self.locked {
            return false;
        }

        // Check if lockout has expired
        match self.locked_until {
            Some(until) => Utc::now() < until,
            None => true, // Permanent lock
        }
    }

    /// Get remaining lockout time in seconds
    pub fn remaining_lockout_secs(&self) -> Option<i64> {
        if !self.is_locked() {
            return None;
        }

        self.locked_until.map(|until| {
            (until - Utc::now()).num_seconds().max(0)
        })
    }

    /// Record a failed attempt
    pub fn record_failure(&mut self, ip: Option<String>) {
        self.failed_attempts += 1;
        self.last_failed_at = Some(Utc::now());

        if let Some(ip) = ip {
            if !self.failed_ips.contains(&ip) && self.failed_ips.len() < 10 {
                self.failed_ips.push(ip);
            }
        }
    }

    /// Lock the account
    pub fn lock(&mut self, duration: Duration) {
        self.locked = true;
        self.locked_at = Some(Utc::now());
        self.locked_until = Some(Utc::now() + duration);
        self.lockout_count += 1;
    }

    /// Permanently lock the account
    pub fn lock_permanently(&mut self) {
        self.locked = true;
        self.locked_at = Some(Utc::now());
        self.locked_until = None;
        self.lockout_count += 1;
    }

    /// Unlock the account
    pub fn unlock(&mut self) {
        self.locked = false;
        self.locked_at = None;
        self.locked_until = None;
    }

    /// Reset failed attempts
    pub fn reset(&mut self) {
        self.failed_attempts = 0;
        self.last_failed_at = None;
        self.failed_ips.clear();
    }
}

/// Lockout manager
pub struct LockoutManager {
    storage: Arc<dyn LockoutStorage>,
    event_emitter: Arc<dyn AuthEventEmitter>,
    config: LockoutConfig,
}

impl LockoutManager {
    pub fn new(
        storage: Arc<dyn LockoutStorage>,
        event_emitter: Arc<dyn AuthEventEmitter>,
        config: LockoutConfig,
    ) -> Self {
        Self {
            storage,
            event_emitter,
            config,
        }
    }

    /// Create with default in-memory storage
    pub fn with_defaults(config: LockoutConfig) -> Self {
        Self {
            storage: Arc::new(InMemoryLockoutStorage::new()),
            event_emitter: Arc::new(NoOpEventEmitter),
            config,
        }
    }

    /// Check if a user is locked
    #[instrument(skip(self), fields(user_id = %user.id))]
    pub async fn check_locked(&self, user: &User) -> AuthResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check user's locked flag first
        if user.locked {
            if let Some(until) = user.locked_until {
                if Utc::now() < until {
                    return Err(AuthError::AccountLocked);
                }
            } else {
                return Err(AuthError::AccountLocked);
            }
        }

        // Check lockout status in storage
        if let Some(status) = self.storage.get(user.id).await? {
            if status.is_locked() {
                return Err(AuthError::AccountLocked);
            }
        }

        Ok(())
    }

    /// Record a failed login attempt
    #[instrument(skip(self), fields(user_id = %user.id))]
    pub async fn record_failed_attempt(&self, user: &User) -> AuthResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut status = self
            .storage
            .get(user.id)
            .await?
            .unwrap_or_else(|| LockoutStatus::new(user.id));

        status.record_failure(None);

        // Check if should lock
        if status.failed_attempts >= self.config.max_failed_attempts {
            let duration = self.calculate_lockout_duration(&status);
            status.lock(duration);

            warn!(
                user_id = %user.id,
                failed_attempts = %status.failed_attempts,
                lockout_duration_secs = %duration.num_seconds(),
                "Account locked due to failed attempts"
            );

            self.event_emitter
                .emit(AuthEvent::AccountLocked {
                    user_id: user.id,
                    reason: format!(
                        "Too many failed login attempts ({})",
                        status.failed_attempts
                    ),
                    locked_until: status.locked_until,
                    timestamp: Utc::now(),
                })
                .await;
        }

        self.storage.save(&status).await?;
        Ok(())
    }

    /// Record a failed attempt with IP
    pub async fn record_failed_attempt_with_ip(
        &self,
        user: &User,
        ip: Option<String>,
    ) -> AuthResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut status = self
            .storage
            .get(user.id)
            .await?
            .unwrap_or_else(|| LockoutStatus::new(user.id));

        status.record_failure(ip);

        if status.failed_attempts >= self.config.max_failed_attempts {
            let duration = self.calculate_lockout_duration(&status);
            status.lock(duration);

            self.event_emitter
                .emit(AuthEvent::AccountLocked {
                    user_id: user.id,
                    reason: format!(
                        "Too many failed login attempts ({})",
                        status.failed_attempts
                    ),
                    locked_until: status.locked_until,
                    timestamp: Utc::now(),
                })
                .await;
        }

        self.storage.save(&status).await?;
        Ok(())
    }

    /// Calculate lockout duration (with progressive increase)
    fn calculate_lockout_duration(&self, status: &LockoutStatus) -> Duration {
        if !self.config.progressive_lockout {
            return Duration::seconds(self.config.lockout_duration_secs as i64);
        }

        // Progressive: double duration for each lockout, up to max
        let base_duration = self.config.lockout_duration_secs as i64;
        let multiplier = 2_i64.pow(status.lockout_count);
        let duration_secs = (base_duration * multiplier)
            .min(self.config.max_lockout_duration_secs as i64);

        Duration::seconds(duration_secs)
    }

    /// Reset failed attempts on successful login
    #[instrument(skip(self), fields(user_id = %user.id))]
    pub async fn reset_failed_attempts(&self, user: &User) -> AuthResult<()> {
        if !self.config.enabled || !self.config.reset_on_success {
            return Ok(());
        }

        if let Some(mut status) = self.storage.get(user.id).await? {
            status.reset();
            self.storage.save(&status).await?;
        }

        Ok(())
    }

    /// Manually unlock an account
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn unlock(&self, user_id: UserId) -> AuthResult<()> {
        if let Some(mut status) = self.storage.get(user_id).await? {
            status.unlock();
            status.reset();
            self.storage.save(&status).await?;

            self.event_emitter
                .emit(AuthEvent::AccountUnlocked {
                    user_id,
                    timestamp: Utc::now(),
                })
                .await;

            info!("Account unlocked");
        }

        Ok(())
    }

    /// Get lockout status for a user
    pub async fn get_status(&self, user_id: UserId) -> AuthResult<Option<LockoutStatus>> {
        self.storage.get(user_id).await
    }

    /// Get all currently locked accounts
    pub async fn get_locked_accounts(&self) -> AuthResult<Vec<LockoutStatus>> {
        self.storage.get_all_locked().await
    }

    /// Clean up expired lockouts
    pub async fn cleanup_expired(&self) -> AuthResult<usize> {
        self.storage.cleanup_expired().await
    }
}

/// No-op event emitter for use without events
struct NoOpEventEmitter;

#[async_trait]
impl AuthEventEmitter for NoOpEventEmitter {
    async fn emit(&self, _event: AuthEvent) {}
}

/// Lockout storage trait
#[async_trait]
pub trait LockoutStorage: Send + Sync {
    /// Get lockout status for a user
    async fn get(&self, user_id: UserId) -> AuthResult<Option<LockoutStatus>>;

    /// Save lockout status
    async fn save(&self, status: &LockoutStatus) -> AuthResult<()>;

    /// Delete lockout status
    async fn delete(&self, user_id: UserId) -> AuthResult<()>;

    /// Get all currently locked accounts
    async fn get_all_locked(&self) -> AuthResult<Vec<LockoutStatus>>;

    /// Clean up expired lockouts
    async fn cleanup_expired(&self) -> AuthResult<usize>;
}

/// In-memory lockout storage
pub struct InMemoryLockoutStorage {
    statuses: RwLock<HashMap<UserId, LockoutStatus>>,
}

impl InMemoryLockoutStorage {
    pub fn new() -> Self {
        Self {
            statuses: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryLockoutStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LockoutStorage for InMemoryLockoutStorage {
    async fn get(&self, user_id: UserId) -> AuthResult<Option<LockoutStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.get(&user_id).cloned())
    }

    async fn save(&self, status: &LockoutStatus) -> AuthResult<()> {
        let mut statuses = self.statuses.write().await;
        statuses.insert(status.user_id, status.clone());
        Ok(())
    }

    async fn delete(&self, user_id: UserId) -> AuthResult<()> {
        let mut statuses = self.statuses.write().await;
        statuses.remove(&user_id);
        Ok(())
    }

    async fn get_all_locked(&self) -> AuthResult<Vec<LockoutStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.values().filter(|s| s.is_locked()).cloned().collect())
    }

    async fn cleanup_expired(&self) -> AuthResult<usize> {
        let mut statuses = self.statuses.write().await;
        let mut count = 0;

        for status in statuses.values_mut() {
            if status.locked && !status.is_locked() {
                // Lockout expired, unlock
                status.unlock();
                count += 1;
            }
        }

        Ok(count)
    }
}

/// Lockout middleware check result
#[derive(Debug, Clone, Serialize)]
pub struct LockoutCheckResult {
    pub locked: bool,
    pub remaining_secs: Option<i64>,
    pub failed_attempts: u32,
    pub max_attempts: u32,
    pub message: Option<String>,
}

impl LockoutCheckResult {
    pub fn unlocked(failed_attempts: u32, max_attempts: u32) -> Self {
        Self {
            locked: false,
            remaining_secs: None,
            failed_attempts,
            max_attempts,
            message: None,
        }
    }

    pub fn locked(remaining_secs: i64, failed_attempts: u32, max_attempts: u32) -> Self {
        Self {
            locked: true,
            remaining_secs: Some(remaining_secs),
            failed_attempts,
            max_attempts,
            message: Some(format!(
                "Account locked. Try again in {} seconds.",
                remaining_secs
            )),
        }
    }
}

/// Lockout check service
pub struct LockoutChecker {
    manager: Arc<LockoutManager>,
    config: LockoutConfig,
}

impl LockoutChecker {
    pub fn new(manager: Arc<LockoutManager>, config: LockoutConfig) -> Self {
        Self { manager, config }
    }

    /// Check lockout status and return user-friendly result
    pub async fn check(&self, user_id: UserId) -> AuthResult<LockoutCheckResult> {
        let status = self.manager.get_status(user_id).await?;

        match status {
            Some(status) if status.is_locked() => {
                Ok(LockoutCheckResult::locked(
                    status.remaining_lockout_secs().unwrap_or(0),
                    status.failed_attempts,
                    self.config.max_failed_attempts,
                ))
            }
            Some(status) => {
                Ok(LockoutCheckResult::unlocked(
                    status.failed_attempts,
                    self.config.max_failed_attempts,
                ))
            }
            None => {
                Ok(LockoutCheckResult::unlocked(0, self.config.max_failed_attempts))
            }
        }
    }
}

/// Background task for cleaning up expired lockouts
pub struct LockoutCleanupTask {
    manager: Arc<LockoutManager>,
    interval_secs: u64,
}

impl LockoutCleanupTask {
    pub fn new(manager: Arc<LockoutManager>, interval_secs: u64) -> Self {
        Self {
            manager,
            interval_secs,
        }
    }

    /// Run the cleanup task (call from a background task)
    pub async fn run(&self) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(self.interval_secs)).await;

            match self.manager.cleanup_expired().await {
                Ok(count) if count > 0 => {
                    info!(count = %count, "Cleaned up expired lockouts");
                }
                Err(e) => {
                    warn!(error = %e, "Failed to cleanup expired lockouts");
                }
                _ => {}
            }
        }
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

    fn create_test_user() -> User {
        User::new("testuser")
    }

    fn create_manager(max_attempts: u32) -> LockoutManager {
        let config = LockoutConfig {
            enabled: true,
            max_failed_attempts: max_attempts,
            lockout_duration_secs: 300,
            reset_on_success: true,
            progressive_lockout: false,
            max_lockout_duration_secs: 86400,
        };
        LockoutManager::with_defaults(config)
    }

    #[tokio::test]
    async fn test_lockout_after_max_attempts() {
        let manager = create_manager(3);
        let user = create_test_user();

        // Record failures
        for _ in 0..3 {
            manager.record_failed_attempt(&user).await.unwrap();
        }

        // Should be locked
        let result = manager.check_locked(&user).await;
        assert!(matches!(result, Err(AuthError::AccountLocked)));
    }

    #[tokio::test]
    async fn test_not_locked_before_max_attempts() {
        let manager = create_manager(3);
        let user = create_test_user();

        // Record 2 failures (below max)
        for _ in 0..2 {
            manager.record_failed_attempt(&user).await.unwrap();
        }

        // Should not be locked
        let result = manager.check_locked(&user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_on_success() {
        let manager = create_manager(3);
        let user = create_test_user();

        // Record 2 failures
        manager.record_failed_attempt(&user).await.unwrap();
        manager.record_failed_attempt(&user).await.unwrap();

        // Reset on success
        manager.reset_failed_attempts(&user).await.unwrap();

        // Should be able to fail again without immediate lockout
        manager.record_failed_attempt(&user).await.unwrap();
        manager.record_failed_attempt(&user).await.unwrap();

        // Still not locked (only 2 attempts after reset)
        let result = manager.check_locked(&user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_manual_unlock() {
        let manager = create_manager(3);
        let user = create_test_user();

        // Lock the account
        for _ in 0..3 {
            manager.record_failed_attempt(&user).await.unwrap();
        }

        // Verify locked
        assert!(manager.check_locked(&user).await.is_err());

        // Unlock
        manager.unlock(user.id).await.unwrap();

        // Should be unlocked now
        assert!(manager.check_locked(&user).await.is_ok());
    }

    #[tokio::test]
    async fn test_progressive_lockout() {
        let config = LockoutConfig {
            enabled: true,
            max_failed_attempts: 3,
            lockout_duration_secs: 60,
            reset_on_success: true,
            progressive_lockout: true,
            max_lockout_duration_secs: 3600,
        };
        let storage = Arc::new(InMemoryLockoutStorage::new());
        let events = Arc::new(NoOpEventEmitter);
        let manager = LockoutManager::new(storage, events, config);

        let user = create_test_user();

        // First lockout
        for _ in 0..3 {
            manager.record_failed_attempt(&user).await.unwrap();
        }

        let status1 = manager.get_status(user.id).await.unwrap().unwrap();
        let duration1 = status1.locked_until.unwrap() - status1.locked_at.unwrap();

        // Unlock and trigger second lockout
        manager.unlock(user.id).await.unwrap();
        for _ in 0..3 {
            manager.record_failed_attempt(&user).await.unwrap();
        }

        let status2 = manager.get_status(user.id).await.unwrap().unwrap();
        let duration2 = status2.locked_until.unwrap() - status2.locked_at.unwrap();

        // Second lockout should be longer
        assert!(duration2 > duration1);
    }

    #[test]
    fn test_lockout_status_is_locked() {
        let mut status = LockoutStatus::new(UserId::new());

        // Not locked initially
        assert!(!status.is_locked());

        // Lock for 1 hour
        status.lock(Duration::hours(1));
        assert!(status.is_locked());

        // Simulate time passing
        status.locked_until = Some(Utc::now() - Duration::minutes(1));
        assert!(!status.is_locked());
    }

    #[test]
    fn test_lockout_status_record_failure() {
        let mut status = LockoutStatus::new(UserId::new());

        status.record_failure(Some("192.168.1.1".to_string()));
        status.record_failure(Some("192.168.1.2".to_string()));

        assert_eq!(status.failed_attempts, 2);
        assert_eq!(status.failed_ips.len(), 2);
    }

    #[test]
    fn test_lockout_check_result() {
        let unlocked = LockoutCheckResult::unlocked(2, 5);
        assert!(!unlocked.locked);
        assert_eq!(unlocked.failed_attempts, 2);

        let locked = LockoutCheckResult::locked(300, 5, 5);
        assert!(locked.locked);
        assert_eq!(locked.remaining_secs, Some(300));
        assert!(locked.message.is_some());
    }

    #[tokio::test]
    async fn test_get_locked_accounts() {
        let manager = create_manager(2);
        let user1 = User::new("user1");
        let user2 = User::new("user2");
        let user3 = User::new("user3");

        // Lock user1 and user2
        for _ in 0..2 {
            manager.record_failed_attempt(&user1).await.unwrap();
            manager.record_failed_attempt(&user2).await.unwrap();
        }

        // user3 has failed attempts but not locked
        manager.record_failed_attempt(&user3).await.unwrap();

        let locked = manager.get_locked_accounts().await.unwrap();
        assert_eq!(locked.len(), 2);
    }

    #[async_trait]
    impl AuthEventEmitter for NoOpEventEmitter {
        async fn emit(&self, _: AuthEvent) {}
    }
}
```

---

## Related Specs

- **Spec 366**: Auth Types - Uses AuthError::AccountLocked
- **Spec 367**: Auth Configuration - Uses LockoutConfig
- **Spec 368**: Local Auth - Integrates with login flow
- **Spec 381**: Audit Logging - Logs lockout events
- **Spec 382**: Rate Limiting - Works with rate limiting
- **Spec 384**: Auth Events - Emits lockout events
