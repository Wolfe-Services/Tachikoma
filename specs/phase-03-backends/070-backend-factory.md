# 070 - Backend Factory

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 070
**Status:** Planned
**Dependencies:** 051-backend-trait, 052-backend-config, 056-claude-api-client, 061-codex-api-client, 064-gemini-api-client, 067-ollama-setup
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement a factory pattern for creating backend instances from configuration, enabling dynamic backend selection and providing a unified interface for backend instantiation.

---

## Acceptance Criteria

- [x] `BackendFactory` for creating backends
- [x] Configuration-based backend creation
- [x] Provider detection and validation
- [x] Default backend selection
- [x] Backend registry for available providers
- [x] Async initialization support

---

## Implementation Details

### 1. Factory Types (src/factory/types.rs)

```rust
//! Factory types for backend creation.

use std::sync::Arc;
use tachikoma_backends_core::{Backend, BackendError};

/// Provider identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendProvider {
    /// Anthropic Claude.
    Claude,
    /// OpenAI/Codex.
    Codex,
    /// Google Gemini.
    Gemini,
    /// Local Ollama.
    Ollama,
}

impl BackendProvider {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Some(Self::Claude),
            "codex" | "openai" | "gpt" => Some(Self::Codex),
            "gemini" | "google" => Some(Self::Gemini),
            "ollama" | "local" => Some(Self::Ollama),
            _ => None,
        }
    }

    /// Get the provider name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
        }
    }

    /// Check if this provider requires API key.
    pub fn requires_api_key(&self) -> bool {
        !matches!(self, Self::Ollama)
    }

    /// Get environment variable names for API key.
    pub fn api_key_env_vars(&self) -> &'static [&'static str] {
        match self {
            Self::Claude => &["ANTHROPIC_API_KEY"],
            Self::Codex => &["OPENAI_API_KEY"],
            Self::Gemini => &["GOOGLE_API_KEY", "GEMINI_API_KEY"],
            Self::Ollama => &[],
        }
    }
}

impl std::fmt::Display for BackendProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result of backend creation.
pub type BackendResult = Result<Arc<dyn Backend>, BackendError>;

/// Factory errors.
#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),

    #[error("missing API key for {provider}: set {env_var}")]
    MissingApiKey {
        provider: BackendProvider,
        env_var: &'static str,
    },

    #[error("backend creation failed: {0}")]
    CreationFailed(String),

    #[error("no available backends")]
    NoAvailable,
}

impl From<FactoryError> for BackendError {
    fn from(e: FactoryError) -> Self {
        match e {
            FactoryError::MissingApiKey { env_var, .. } => {
                BackendError::Authentication(format!("Missing API key: {}", env_var))
            }
            _ => BackendError::Configuration(e.to_string()),
        }
    }
}
```

### 2. Backend Factory (src/factory/mod.rs)

```rust
//! Backend factory for creating backend instances.

mod types;

pub use types::{BackendProvider, BackendResult, FactoryError};

use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{Backend, BackendError, BackendConfig};
use tracing::{debug, info, warn};

/// Factory for creating backend instances.
pub struct BackendFactory {
    /// Registered creators.
    creators: HashMap<BackendProvider, Box<dyn BackendCreator>>,
    /// Default provider.
    default_provider: Option<BackendProvider>,
}

impl Default for BackendFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendFactory {
    /// Create a new factory with default creators.
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
            default_provider: None,
        };

        // Register default creators
        factory.register(BackendProvider::Claude, ClaudeCreator);
        factory.register(BackendProvider::Codex, CodexCreator);
        factory.register(BackendProvider::Gemini, GeminiCreator);
        factory.register(BackendProvider::Ollama, OllamaCreator);

        factory
    }

    /// Register a backend creator.
    pub fn register(&mut self, provider: BackendProvider, creator: impl BackendCreator + 'static) {
        self.creators.insert(provider, Box::new(creator));
    }

    /// Set the default provider.
    pub fn set_default(&mut self, provider: BackendProvider) {
        self.default_provider = Some(provider);
    }

    /// Create a backend from configuration.
    pub async fn create(&self, config: &BackendConfig) -> BackendResult {
        let provider = match config {
            BackendConfig::Claude(_) => BackendProvider::Claude,
            BackendConfig::Codex(_) => BackendProvider::Codex,
            BackendConfig::Gemini(_) => BackendProvider::Gemini,
            BackendConfig::Ollama(_) => BackendProvider::Ollama,
        };

        self.create_provider(provider, Some(config)).await
    }

    /// Create a backend for a specific provider.
    pub async fn create_provider(
        &self,
        provider: BackendProvider,
        config: Option<&BackendConfig>,
    ) -> BackendResult {
        debug!(provider = %provider, "Creating backend");

        let creator = self
            .creators
            .get(&provider)
            .ok_or_else(|| FactoryError::UnknownProvider(provider.to_string()))?;

        let backend = creator.create(config).await?;

        info!(provider = %provider, name = %backend.name(), "Backend created");

        Ok(backend)
    }

    /// Create a backend from environment.
    pub async fn create_from_env(&self, provider: BackendProvider) -> BackendResult {
        self.create_provider(provider, None).await
    }

    /// Create the default backend.
    pub async fn create_default(&self) -> BackendResult {
        let provider = self.default_provider
            .or_else(|| self.detect_available())
            .ok_or(FactoryError::NoAvailable)?;

        self.create_from_env(provider).await
    }

    /// Detect which providers have credentials available.
    pub fn detect_available(&self) -> Option<BackendProvider> {
        // Check providers in order of preference
        let providers = [
            BackendProvider::Claude,
            BackendProvider::Codex,
            BackendProvider::Gemini,
            BackendProvider::Ollama,
        ];

        for provider in providers {
            if self.has_credentials(provider) {
                return Some(provider);
            }
        }

        None
    }

    /// Check if credentials are available for a provider.
    pub fn has_credentials(&self, provider: BackendProvider) -> bool {
        if !provider.requires_api_key() {
            return true;
        }

        provider
            .api_key_env_vars()
            .iter()
            .any(|var| std::env::var(var).is_ok())
    }

    /// Get all available providers.
    pub fn available_providers(&self) -> Vec<BackendProvider> {
        [
            BackendProvider::Claude,
            BackendProvider::Codex,
            BackendProvider::Gemini,
            BackendProvider::Ollama,
        ]
        .into_iter()
        .filter(|p| self.has_credentials(*p))
        .collect()
    }
}

/// Trait for backend creators.
#[async_trait::async_trait]
pub trait BackendCreator: Send + Sync {
    /// Create a backend instance.
    async fn create(&self, config: Option<&BackendConfig>) -> BackendResult;
}

/// Claude backend creator.
struct ClaudeCreator;

#[async_trait::async_trait]
impl BackendCreator for ClaudeCreator {
    async fn create(&self, config: Option<&BackendConfig>) -> BackendResult {
        use tachikoma_backend_claude::{ClaudeBackend, ClaudeBackendConfig};

        let backend_config = match config {
            Some(BackendConfig::Claude(c)) => ClaudeBackendConfig {
                api_key: c.api_key.clone(),
                base_url: c.base_url.clone(),
                model: tachikoma_backend_claude::ClaudeModel::from_str(&c.model)
                    .unwrap_or_default(),
                api_version: c.api_version.clone(),
                max_tokens: c.max_tokens,
            },
            _ => {
                // Load from environment
                let config = tachikoma_backends_core::config::ClaudeConfig::from_env()
                    .map_err(|e| BackendError::Configuration(e.to_string()))?;

                ClaudeBackendConfig {
                    api_key: config.api_key,
                    base_url: config.base_url,
                    model: tachikoma_backend_claude::ClaudeModel::from_str(&config.model)
                        .unwrap_or_default(),
                    api_version: config.api_version,
                    max_tokens: config.max_tokens,
                }
            }
        };

        let backend = ClaudeBackend::new(backend_config)?;
        Ok(Arc::new(backend))
    }
}

/// Codex backend creator.
struct CodexCreator;

#[async_trait::async_trait]
impl BackendCreator for CodexCreator {
    async fn create(&self, config: Option<&BackendConfig>) -> BackendResult {
        use tachikoma_backend_codex::{CodexBackend, CodexBackendConfig};

        let backend_config = match config {
            Some(BackendConfig::Codex(c)) => CodexBackendConfig {
                api_key: c.api_key.clone(),
                organization: c.organization.clone(),
                base_url: c.base_url.clone(),
                model: tachikoma_backend_codex::OpenAIModel::from_str(&c.model)
                    .unwrap_or_default(),
                max_tokens: c.max_tokens,
            },
            _ => {
                let config = tachikoma_backends_core::config::CodexConfig::from_env()
                    .map_err(|e| BackendError::Configuration(e.to_string()))?;

                CodexBackendConfig {
                    api_key: config.api_key,
                    organization: config.organization,
                    base_url: config.base_url,
                    model: tachikoma_backend_codex::OpenAIModel::from_str(&config.model)
                        .unwrap_or_default(),
                    max_tokens: config.max_tokens,
                }
            }
        };

        let backend = CodexBackend::new(backend_config)?;
        Ok(Arc::new(backend))
    }
}

/// Gemini backend creator.
struct GeminiCreator;

#[async_trait::async_trait]
impl BackendCreator for GeminiCreator {
    async fn create(&self, config: Option<&BackendConfig>) -> BackendResult {
        use tachikoma_backend_gemini::{GeminiBackend, GeminiBackendConfig};

        let backend_config = match config {
            Some(BackendConfig::Gemini(c)) => GeminiBackendConfig {
                api_key: c.api_key.clone(),
                base_url: c.base_url.clone(),
                model: tachikoma_backend_gemini::GeminiModel::from_str(&c.model)
                    .unwrap_or_default(),
                max_output_tokens: c.max_output_tokens,
                safety_settings: vec![],
            },
            _ => {
                let config = tachikoma_backends_core::config::GeminiConfig::from_env()
                    .map_err(|e| BackendError::Configuration(e.to_string()))?;

                GeminiBackendConfig {
                    api_key: config.api_key,
                    base_url: config.base_url,
                    model: tachikoma_backend_gemini::GeminiModel::from_str(&config.model)
                        .unwrap_or_default(),
                    max_output_tokens: config.max_output_tokens,
                    safety_settings: vec![],
                }
            }
        };

        let backend = GeminiBackend::new(backend_config)?;
        Ok(Arc::new(backend))
    }
}

/// Ollama backend creator.
struct OllamaCreator;

#[async_trait::async_trait]
impl BackendCreator for OllamaCreator {
    async fn create(&self, config: Option<&BackendConfig>) -> BackendResult {
        use tachikoma_backend_ollama::{
            OllamaBackend, OllamaBackendConfig, OllamaServerConfig,
        };

        let (server_config, backend_config) = match config {
            Some(BackendConfig::Ollama(c)) => {
                let server = OllamaServerConfig {
                    base_url: c.base_url.clone(),
                    ..Default::default()
                };
                let backend = OllamaBackendConfig {
                    model: c.model.clone(),
                    num_ctx: c.num_ctx,
                    keep_alive: c.keep_alive.clone(),
                    enable_tools: false,
                };
                (server, backend)
            }
            _ => {
                let config = tachikoma_backends_core::config::OllamaConfig::from_env();
                let server = OllamaServerConfig {
                    base_url: config.base_url,
                    ..Default::default()
                };
                let backend = OllamaBackendConfig {
                    model: config.model,
                    num_ctx: config.num_ctx,
                    keep_alive: config.keep_alive,
                    enable_tools: false,
                };
                (server, backend)
            }
        };

        let backend = OllamaBackend::new(server_config, backend_config).await?;
        Ok(Arc::new(backend))
    }
}
```

### 3. Multi-Backend Support (src/factory/multi.rs)

```rust
//! Multi-backend management.

use super::{BackendFactory, BackendProvider};
use std::collections::HashMap;
use std::sync::Arc;
use tachikoma_backends_core::{Backend, BackendError};
use tokio::sync::RwLock;

/// Manager for multiple backend instances.
pub struct BackendManager {
    factory: BackendFactory,
    backends: RwLock<HashMap<BackendProvider, Arc<dyn Backend>>>,
    primary: RwLock<Option<BackendProvider>>,
}

impl BackendManager {
    /// Create a new backend manager.
    pub fn new() -> Self {
        Self {
            factory: BackendFactory::new(),
            backends: RwLock::new(HashMap::new()),
            primary: RwLock::new(None),
        }
    }

    /// Initialize a backend.
    pub async fn initialize(&self, provider: BackendProvider) -> Result<(), BackendError> {
        let backend = self.factory.create_from_env(provider).await?;

        let mut backends = self.backends.write().await;
        backends.insert(provider, backend);

        // Set as primary if none set
        let mut primary = self.primary.write().await;
        if primary.is_none() {
            *primary = Some(provider);
        }

        Ok(())
    }

    /// Get a backend by provider.
    pub async fn get(&self, provider: BackendProvider) -> Option<Arc<dyn Backend>> {
        self.backends.read().await.get(&provider).cloned()
    }

    /// Get the primary backend.
    pub async fn primary(&self) -> Option<Arc<dyn Backend>> {
        let primary = self.primary.read().await;
        if let Some(provider) = *primary {
            self.get(provider).await
        } else {
            None
        }
    }

    /// Set the primary backend.
    pub async fn set_primary(&self, provider: BackendProvider) -> Result<(), BackendError> {
        if !self.backends.read().await.contains_key(&provider) {
            self.initialize(provider).await?;
        }
        *self.primary.write().await = Some(provider);
        Ok(())
    }

    /// Get all initialized backends.
    pub async fn all(&self) -> Vec<(BackendProvider, Arc<dyn Backend>)> {
        self.backends
            .read()
            .await
            .iter()
            .map(|(p, b)| (*p, Arc::clone(b)))
            .collect()
    }

    /// Initialize all available backends.
    pub async fn initialize_all(&self) -> Vec<(BackendProvider, Result<(), BackendError>)> {
        let available = self.factory.available_providers();
        let mut results = Vec::new();

        for provider in available {
            let result = self.initialize(provider).await;
            results.push((provider, result));
        }

        results
    }
}

impl Default for BackendManager {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Testing Requirements

1. Factory creates backends from config
2. Environment-based creation works
3. Provider detection finds available backends
4. Multi-backend manager works correctly
5. Missing credentials are handled

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: All backend implementations
- Next: [071-backend-health.md](071-backend-health.md)
