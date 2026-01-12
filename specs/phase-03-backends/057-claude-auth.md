# 057 - Claude Authentication (API Key Handling)

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 057
**Status:** Planned
**Dependencies:** 056-claude-api-client, 017-secret-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement secure API key handling for the Claude backend, including environment variable loading, secure storage, key rotation support, and proper secret hygiene to prevent accidental exposure.

---

## Acceptance Criteria

- [ ] Secure API key storage using `Secret<T>`
- [ ] Environment variable loading (`ANTHROPIC_API_KEY`)
- [ ] File-based key loading (with secure permissions check)
- [ ] Key validation before use
- [ ] Support for multiple API keys (rotation)
- [ ] Audit logging for key usage
- [ ] Prevention of key exposure in logs/debug output

---

## Implementation Details

### 1. Authentication Types (src/auth/types.rs)

```rust
//! Authentication types for Claude API.

use serde::{Deserialize, Serialize};
use std::fmt;
use tachikoma_common_config::Secret;

/// API key for Claude authentication.
#[derive(Clone)]
pub struct ApiKey {
    /// The secret key value.
    inner: Secret<String>,
    /// Key identifier (for logging, first 8 chars).
    key_id: String,
    /// When the key was loaded.
    loaded_at: std::time::Instant,
}

impl ApiKey {
    /// Create a new API key.
    pub fn new(key: impl Into<String>) -> Result<Self, AuthError> {
        let key = key.into();

        // Validate key format
        Self::validate_format(&key)?;

        let key_id = Self::derive_key_id(&key);

        Ok(Self {
            inner: Secret::new(key),
            key_id,
            loaded_at: std::time::Instant::now(),
        })
    }

    /// Validate API key format.
    fn validate_format(key: &str) -> Result<(), AuthError> {
        if key.is_empty() {
            return Err(AuthError::EmptyKey);
        }

        // Anthropic keys start with "sk-ant-"
        if !key.starts_with("sk-ant-") {
            return Err(AuthError::InvalidFormat(
                "API key should start with 'sk-ant-'".to_string(),
            ));
        }

        // Basic length check
        if key.len() < 40 {
            return Err(AuthError::InvalidFormat(
                "API key appears too short".to_string(),
            ));
        }

        Ok(())
    }

    /// Derive a safe identifier for logging.
    fn derive_key_id(key: &str) -> String {
        if key.len() >= 16 {
            format!("{}...{}", &key[..8], &key[key.len() - 4..])
        } else {
            "***".to_string()
        }
    }

    /// Get the key for use in API calls.
    pub fn expose(&self) -> &str {
        self.inner.expose()
    }

    /// Get the key identifier (safe for logging).
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Get how long this key has been loaded.
    pub fn age(&self) -> std::time::Duration {
        self.loaded_at.elapsed()
    }

    /// Check if the key should be rotated (older than max age).
    pub fn should_rotate(&self, max_age: std::time::Duration) -> bool {
        self.age() > max_age
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKey")
            .field("key_id", &self.key_id)
            .field("age_secs", &self.loaded_at.elapsed().as_secs())
            .finish()
    }
}

// Prevent accidental display of the key
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey({})", self.key_id)
    }
}

/// Authentication errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("API key is empty")]
    EmptyKey,

    #[error("invalid API key format: {0}")]
    InvalidFormat(String),

    #[error("API key not found in environment")]
    NotInEnvironment,

    #[error("failed to read API key from file: {0}")]
    FileReadError(String),

    #[error("API key file has insecure permissions")]
    InsecurePermissions,

    #[error("API key validation failed: {0}")]
    ValidationFailed(String),

    #[error("no valid API key available")]
    NoValidKey,
}
```

### 2. Key Loading (src/auth/loader.rs)

```rust
//! API key loading from various sources.

use super::{ApiKey, AuthError};
use std::path::Path;
use tracing::{debug, info, warn};

/// Sources for loading API keys.
#[derive(Debug, Clone)]
pub enum KeySource {
    /// Load from environment variable.
    Environment(String),
    /// Load from a file.
    File(std::path::PathBuf),
    /// Direct value (for testing).
    Direct(String),
}

impl Default for KeySource {
    fn default() -> Self {
        Self::Environment("ANTHROPIC_API_KEY".to_string())
    }
}

/// Load an API key from the environment.
pub fn load_from_env(var_name: &str) -> Result<ApiKey, AuthError> {
    debug!(var_name, "Loading API key from environment");

    let key = std::env::var(var_name).map_err(|_| AuthError::NotInEnvironment)?;

    ApiKey::new(key)
}

/// Load an API key from a file.
pub fn load_from_file(path: &Path) -> Result<ApiKey, AuthError> {
    debug!(path = %path.display(), "Loading API key from file");

    // Check file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)
            .map_err(|e| AuthError::FileReadError(e.to_string()))?;
        let mode = metadata.permissions().mode();

        // Warn if file is readable by group or others
        if mode & 0o077 != 0 {
            warn!(
                path = %path.display(),
                mode = format!("{:o}", mode),
                "API key file has permissive permissions"
            );
            // Optionally fail on insecure permissions
            // return Err(AuthError::InsecurePermissions);
        }
    }

    let key = std::fs::read_to_string(path)
        .map_err(|e| AuthError::FileReadError(e.to_string()))?
        .trim()
        .to_string();

    ApiKey::new(key)
}

/// Load an API key from a source.
pub fn load_from_source(source: &KeySource) -> Result<ApiKey, AuthError> {
    match source {
        KeySource::Environment(var) => load_from_env(var),
        KeySource::File(path) => load_from_file(path),
        KeySource::Direct(key) => ApiKey::new(key.clone()),
    }
}

/// Try loading from multiple sources in order.
pub fn load_from_sources(sources: &[KeySource]) -> Result<ApiKey, AuthError> {
    for source in sources {
        match load_from_source(source) {
            Ok(key) => {
                info!(key_id = %key.key_id(), "Successfully loaded API key");
                return Ok(key);
            }
            Err(e) => {
                debug!(source = ?source, error = %e, "Failed to load key from source");
                continue;
            }
        }
    }

    Err(AuthError::NoValidKey)
}

/// Default key sources in priority order.
pub fn default_sources() -> Vec<KeySource> {
    let mut sources = vec![KeySource::Environment("ANTHROPIC_API_KEY".to_string())];

    // Check common file locations
    if let Some(home) = dirs::home_dir() {
        sources.push(KeySource::File(home.join(".anthropic/api_key")));
        sources.push(KeySource::File(home.join(".config/anthropic/api_key")));
    }

    sources
}
```

### 3. Key Rotation (src/auth/rotation.rs)

```rust
//! API key rotation support.

use super::{ApiKey, AuthError, KeySource, load_from_source};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manages API key rotation.
#[derive(Debug)]
pub struct KeyRotator {
    /// Current active key.
    current: Arc<RwLock<ApiKey>>,
    /// Backup keys.
    backups: Arc<RwLock<Vec<ApiKey>>>,
    /// Key sources for reloading.
    sources: Vec<KeySource>,
    /// Maximum key age before rotation.
    max_age: Duration,
}

impl KeyRotator {
    /// Create a new key rotator.
    pub fn new(initial_key: ApiKey, sources: Vec<KeySource>) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial_key)),
            backups: Arc::new(RwLock::new(Vec::new())),
            sources,
            max_age: Duration::from_secs(24 * 60 * 60), // 24 hours default
        }
    }

    /// Set the maximum key age.
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Get the current API key.
    pub async fn current_key(&self) -> ApiKey {
        self.current.read().await.clone()
    }

    /// Add a backup key.
    pub async fn add_backup(&self, key: ApiKey) {
        self.backups.write().await.push(key);
    }

    /// Check if rotation is needed.
    pub async fn needs_rotation(&self) -> bool {
        let key = self.current.read().await;
        key.should_rotate(self.max_age)
    }

    /// Attempt to rotate to a fresh key.
    pub async fn rotate(&self) -> Result<(), AuthError> {
        debug!("Attempting key rotation");

        // Try to load a fresh key from sources
        for source in &self.sources {
            match load_from_source(source) {
                Ok(new_key) => {
                    let old_key = {
                        let mut current = self.current.write().await;
                        std::mem::replace(&mut *current, new_key.clone())
                    };

                    // Move old key to backups
                    self.backups.write().await.push(old_key);

                    info!(
                        new_key_id = %new_key.key_id(),
                        "Successfully rotated API key"
                    );
                    return Ok(());
                }
                Err(e) => {
                    debug!(source = ?source, error = %e, "Source failed during rotation");
                }
            }
        }

        warn!("Key rotation failed, keeping current key");
        Err(AuthError::NoValidKey)
    }

    /// Fallback to a backup key (on auth failure).
    pub async fn fallback(&self) -> Result<(), AuthError> {
        let mut backups = self.backups.write().await;

        if let Some(backup) = backups.pop() {
            let failed_key = {
                let mut current = self.current.write().await;
                std::mem::replace(&mut *current, backup)
            };

            info!(
                failed_key_id = %failed_key.key_id(),
                "Falling back to backup key"
            );
            Ok(())
        } else {
            Err(AuthError::NoValidKey)
        }
    }

    /// Start background rotation task.
    pub fn start_rotation_task(self: Arc<Self>, check_interval: Duration) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(check_interval).await;

                if self.needs_rotation().await {
                    if let Err(e) = self.rotate().await {
                        warn!(error = %e, "Background key rotation failed");
                    }
                }
            }
        });
    }
}
```

### 4. Authentication Provider (src/auth/provider.rs)

```rust
//! High-level authentication provider.

use super::{ApiKey, AuthError, KeyRotator, KeySource, load_from_sources, default_sources};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// Authentication provider for Claude API.
#[derive(Debug)]
pub struct AuthProvider {
    /// Key rotator (if rotation is enabled).
    rotator: Option<Arc<KeyRotator>>,
    /// Static key (if rotation is disabled).
    static_key: Option<ApiKey>,
}

impl AuthProvider {
    /// Create a new auth provider with automatic key loading.
    pub fn new() -> Result<Self, AuthError> {
        let sources = default_sources();
        Self::with_sources(sources)
    }

    /// Create with specific sources.
    pub fn with_sources(sources: Vec<KeySource>) -> Result<Self, AuthError> {
        let key = load_from_sources(&sources)?;
        Ok(Self {
            rotator: None,
            static_key: Some(key),
        })
    }

    /// Create with a direct API key.
    pub fn with_key(key: impl Into<String>) -> Result<Self, AuthError> {
        let api_key = ApiKey::new(key)?;
        Ok(Self {
            rotator: None,
            static_key: Some(api_key),
        })
    }

    /// Enable key rotation.
    pub fn with_rotation(mut self, sources: Vec<KeySource>, max_age: Duration) -> Self {
        if let Some(key) = self.static_key.take() {
            let rotator = KeyRotator::new(key, sources).with_max_age(max_age);
            self.rotator = Some(Arc::new(rotator));
        }
        self
    }

    /// Start background rotation.
    pub fn start_background_rotation(&self, check_interval: Duration) {
        if let Some(rotator) = &self.rotator {
            Arc::clone(rotator).start_rotation_task(check_interval);
            info!("Started background key rotation");
        }
    }

    /// Get the current API key.
    pub async fn get_key(&self) -> Result<ApiKey, AuthError> {
        if let Some(rotator) = &self.rotator {
            Ok(rotator.current_key().await)
        } else if let Some(key) = &self.static_key {
            Ok(key.clone())
        } else {
            Err(AuthError::NoValidKey)
        }
    }

    /// Report an authentication failure (triggers fallback).
    pub async fn report_auth_failure(&self) -> Result<(), AuthError> {
        if let Some(rotator) = &self.rotator {
            rotator.fallback().await
        } else {
            Err(AuthError::NoValidKey)
        }
    }
}

impl Default for AuthProvider {
    fn default() -> Self {
        Self::new().expect("Failed to initialize auth provider")
    }
}
```

### 5. Module Exports (src/auth/mod.rs)

```rust
//! Authentication module for Claude API.

mod loader;
mod provider;
mod rotation;
mod types;

pub use loader::{default_sources, load_from_env, load_from_file, load_from_source, load_from_sources, KeySource};
pub use provider::AuthProvider;
pub use rotation::KeyRotator;
pub use types::{ApiKey, AuthError};
```

---

## Testing Requirements

1. API key format validation works correctly
2. Environment variable loading succeeds
3. File loading checks permissions (Unix)
4. Key rotation switches keys properly
5. Fallback to backup keys works
6. Keys are not exposed in debug output

---

## Related Specs

- Depends on: [056-claude-api-client.md](056-claude-api-client.md)
- Depends on: [017-secret-types.md](../phase-01-common/017-secret-types.md)
- Next: [058-claude-streaming.md](058-claude-streaming.md)
