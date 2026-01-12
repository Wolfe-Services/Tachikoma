# 062 - Codex Authentication

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 062
**Status:** Planned
**Dependencies:** 061-codex-api-client, 017-secret-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement secure API key and organization handling for the OpenAI/Codex backend, including environment variable loading, project-based keys, and proper secret hygiene.

---

## Acceptance Criteria

- [ ] Secure API key storage using `Secret<T>`
- [ ] Environment variable loading (`OPENAI_API_KEY`)
- [ ] Organization ID handling (`OPENAI_ORGANIZATION`)
- [ ] Project-based API key support
- [ ] Key validation before use
- [ ] Azure OpenAI authentication support

---

## Implementation Details

### 1. Authentication Types (src/auth/types.rs)

```rust
//! Authentication types for OpenAI API.

use serde::{Deserialize, Serialize};
use std::fmt;
use tachikoma_common_config::Secret;

/// API key for OpenAI authentication.
#[derive(Clone)]
pub struct OpenAIApiKey {
    /// The secret key value.
    inner: Secret<String>,
    /// Key identifier (for logging).
    key_id: String,
    /// Key type.
    key_type: KeyType,
}

/// Type of OpenAI API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    /// Standard API key (sk-...)
    Standard,
    /// Project-based key (sk-proj-...)
    Project,
    /// Service account key
    ServiceAccount,
    /// Azure deployment key
    Azure,
}

impl OpenAIApiKey {
    /// Create a new API key.
    pub fn new(key: impl Into<String>) -> Result<Self, AuthError> {
        let key = key.into();
        Self::validate_format(&key)?;

        let key_type = Self::detect_key_type(&key);
        let key_id = Self::derive_key_id(&key);

        Ok(Self {
            inner: Secret::new(key),
            key_id,
            key_type,
        })
    }

    /// Validate API key format.
    fn validate_format(key: &str) -> Result<(), AuthError> {
        if key.is_empty() {
            return Err(AuthError::EmptyKey);
        }

        // OpenAI keys start with "sk-"
        if !key.starts_with("sk-") {
            return Err(AuthError::InvalidFormat(
                "API key should start with 'sk-'".to_string(),
            ));
        }

        if key.len() < 20 {
            return Err(AuthError::InvalidFormat(
                "API key appears too short".to_string(),
            ));
        }

        Ok(())
    }

    /// Detect the type of key.
    fn detect_key_type(key: &str) -> KeyType {
        if key.starts_with("sk-proj-") {
            KeyType::Project
        } else if key.starts_with("sk-svcacct-") {
            KeyType::ServiceAccount
        } else {
            KeyType::Standard
        }
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

    /// Get the key identifier.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Get the key type.
    pub fn key_type(&self) -> KeyType {
        self.key_type
    }
}

impl fmt::Debug for OpenAIApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAIApiKey")
            .field("key_id", &self.key_id)
            .field("key_type", &self.key_type)
            .finish()
    }
}

impl fmt::Display for OpenAIApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpenAIApiKey({}, {:?})", self.key_id, self.key_type)
    }
}

/// Organization identifier.
#[derive(Debug, Clone)]
pub struct OrganizationId(String);

impl OrganizationId {
    /// Create a new organization ID.
    pub fn new(id: impl Into<String>) -> Result<Self, AuthError> {
        let id = id.into();
        if id.is_empty() {
            return Err(AuthError::InvalidFormat("Organization ID is empty".to_string()));
        }
        // Organization IDs start with "org-"
        if !id.starts_with("org-") {
            return Err(AuthError::InvalidFormat(
                "Organization ID should start with 'org-'".to_string(),
            ));
        }
        Ok(Self(id))
    }

    /// Get the organization ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Azure OpenAI credentials.
#[derive(Clone)]
pub struct AzureCredentials {
    /// Azure API key.
    pub api_key: Secret<String>,
    /// Azure endpoint URL.
    pub endpoint: String,
    /// Deployment name.
    pub deployment: String,
    /// API version.
    pub api_version: String,
}

impl AzureCredentials {
    /// Create new Azure credentials.
    pub fn new(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment: impl Into<String>,
    ) -> Self {
        Self {
            api_key: Secret::new(api_key.into()),
            endpoint: endpoint.into(),
            deployment: deployment.into(),
            api_version: "2024-02-01".to_string(),
        }
    }

    /// Get the completions endpoint URL.
    pub fn completions_url(&self) -> String {
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint.trim_end_matches('/'),
            self.deployment,
            self.api_version
        )
    }
}

impl fmt::Debug for AzureCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AzureCredentials")
            .field("endpoint", &self.endpoint)
            .field("deployment", &self.deployment)
            .field("api_version", &self.api_version)
            .finish()
    }
}

/// Authentication errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("API key is empty")]
    EmptyKey,

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("API key not found in environment")]
    NotInEnvironment,

    #[error("failed to read API key: {0}")]
    ReadError(String),
}
```

### 2. Key Loading (src/auth/loader.rs)

```rust
//! API key loading for OpenAI.

use super::{AzureCredentials, AuthError, OpenAIApiKey, OrganizationId};
use tracing::{debug, info};

/// Load API key from environment.
pub fn load_api_key_from_env() -> Result<OpenAIApiKey, AuthError> {
    debug!("Loading OpenAI API key from environment");

    let key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| AuthError::NotInEnvironment)?;

    OpenAIApiKey::new(key)
}

/// Load organization ID from environment.
pub fn load_organization_from_env() -> Option<OrganizationId> {
    std::env::var("OPENAI_ORGANIZATION")
        .ok()
        .and_then(|id| OrganizationId::new(id).ok())
}

/// Load Azure credentials from environment.
pub fn load_azure_from_env() -> Result<AzureCredentials, AuthError> {
    debug!("Loading Azure OpenAI credentials from environment");

    let api_key = std::env::var("AZURE_OPENAI_API_KEY")
        .map_err(|_| AuthError::NotInEnvironment)?;

    let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
        .map_err(|_| AuthError::NotInEnvironment)?;

    let deployment = std::env::var("AZURE_OPENAI_DEPLOYMENT")
        .unwrap_or_else(|_| "gpt-4".to_string());

    Ok(AzureCredentials::new(api_key, endpoint, deployment))
}

/// Credentials loaded from environment.
#[derive(Debug)]
pub struct LoadedCredentials {
    pub api_key: OpenAIApiKey,
    pub organization: Option<OrganizationId>,
}

/// Load all credentials from environment.
pub fn load_credentials_from_env() -> Result<LoadedCredentials, AuthError> {
    let api_key = load_api_key_from_env()?;
    let organization = load_organization_from_env();

    info!(
        key_id = %api_key.key_id(),
        key_type = ?api_key.key_type(),
        has_org = organization.is_some(),
        "Loaded OpenAI credentials"
    );

    Ok(LoadedCredentials {
        api_key,
        organization,
    })
}
```

### 3. Authentication Provider (src/auth/provider.rs)

```rust
//! High-level authentication provider for OpenAI.

use super::{
    load_api_key_from_env, load_azure_from_env, load_organization_from_env,
    AzureCredentials, AuthError, OpenAIApiKey, OrganizationId,
};
use reqwest::header::HeaderMap;
use tracing::debug;

/// Authentication mode.
#[derive(Debug, Clone)]
pub enum AuthMode {
    /// Standard OpenAI API.
    OpenAI {
        api_key: OpenAIApiKey,
        organization: Option<OrganizationId>,
    },
    /// Azure OpenAI.
    Azure(AzureCredentials),
}

/// Authentication provider for OpenAI/Azure.
#[derive(Debug)]
pub struct OpenAIAuthProvider {
    mode: AuthMode,
}

impl OpenAIAuthProvider {
    /// Create from environment (auto-detect mode).
    pub fn from_env() -> Result<Self, AuthError> {
        // Try Azure first
        if let Ok(azure) = load_azure_from_env() {
            debug!("Using Azure OpenAI authentication");
            return Ok(Self {
                mode: AuthMode::Azure(azure),
            });
        }

        // Fall back to standard OpenAI
        let api_key = load_api_key_from_env()?;
        let organization = load_organization_from_env();

        debug!("Using standard OpenAI authentication");
        Ok(Self {
            mode: AuthMode::OpenAI {
                api_key,
                organization,
            },
        })
    }

    /// Create with OpenAI API key.
    pub fn with_api_key(key: impl Into<String>) -> Result<Self, AuthError> {
        let api_key = OpenAIApiKey::new(key)?;
        Ok(Self {
            mode: AuthMode::OpenAI {
                api_key,
                organization: None,
            },
        })
    }

    /// Create with Azure credentials.
    pub fn with_azure(credentials: AzureCredentials) -> Self {
        Self {
            mode: AuthMode::Azure(credentials),
        }
    }

    /// Set organization (only for OpenAI mode).
    pub fn with_organization(mut self, org: OrganizationId) -> Self {
        if let AuthMode::OpenAI { organization, .. } = &mut self.mode {
            *organization = Some(org);
        }
        self
    }

    /// Get the authentication mode.
    pub fn mode(&self) -> &AuthMode {
        &self.mode
    }

    /// Check if using Azure.
    pub fn is_azure(&self) -> bool {
        matches!(self.mode, AuthMode::Azure(_))
    }

    /// Get the base URL for API requests.
    pub fn base_url(&self) -> String {
        match &self.mode {
            AuthMode::OpenAI { .. } => "https://api.openai.com".to_string(),
            AuthMode::Azure(creds) => creds.endpoint.clone(),
        }
    }

    /// Get the completions endpoint URL.
    pub fn completions_url(&self) -> String {
        match &self.mode {
            AuthMode::OpenAI { .. } => {
                format!("{}/v1/chat/completions", self.base_url())
            }
            AuthMode::Azure(creds) => creds.completions_url(),
        }
    }

    /// Build authorization headers.
    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        match &self.mode {
            AuthMode::OpenAI { api_key, organization } => {
                headers.insert(
                    "Authorization",
                    format!("Bearer {}", api_key.expose()).parse().unwrap(),
                );

                if let Some(org) = organization {
                    headers.insert(
                        "OpenAI-Organization",
                        org.as_str().parse().unwrap(),
                    );
                }
            }
            AuthMode::Azure(creds) => {
                headers.insert(
                    "api-key",
                    creds.api_key.expose().parse().unwrap(),
                );
            }
        }

        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
}
```

### 4. Module Exports (src/auth/mod.rs)

```rust
//! Authentication module for OpenAI API.

mod loader;
mod provider;
mod types;

pub use loader::{
    load_api_key_from_env, load_azure_from_env, load_credentials_from_env,
    load_organization_from_env, LoadedCredentials,
};
pub use provider::{AuthMode, OpenAIAuthProvider};
pub use types::{AuthError, AzureCredentials, KeyType, OpenAIApiKey, OrganizationId};
```

---

## Testing Requirements

1. API key format validation works
2. Key type detection is correct
3. Organization ID validation works
4. Azure credentials load properly
5. Headers are constructed correctly
6. Keys are not exposed in debug output

---

## Related Specs

- Depends on: [061-codex-api-client.md](061-codex-api-client.md)
- Depends on: [017-secret-types.md](../phase-01-common/017-secret-types.md)
- Next: [063-codex-tools.md](063-codex-tools.md)
