# 052 - Backend Configuration Types

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 052
**Status:** Planned
**Dependencies:** 051-backend-trait, 014-config-core-types, 017-secret-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Define configuration types for all LLM backends, including API endpoints, authentication settings, rate limits, and model preferences. Support both file-based and environment-based configuration.

---

## Acceptance Criteria

- [x] Backend-agnostic `BackendConfig` enum
- [x] Provider-specific config structs
- [x] Secret/API key handling via `Secret<T>`
- [x] Endpoint URL configuration
- [x] Default values for all providers
- [x] YAML serialization support
- [x] Environment variable overrides

---

## Implementation Details

### 1. Base Config Types (src/config/mod.rs)

```rust
//! Backend configuration types.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tachikoma_common_config::Secret;

/// Configuration for an LLM backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum BackendConfig {
    /// Anthropic Claude configuration.
    Claude(ClaudeConfig),
    /// OpenAI Codex configuration.
    Codex(CodexConfig),
    /// Google Gemini configuration.
    Gemini(GeminiConfig),
    /// Local Ollama configuration.
    Ollama(OllamaConfig),
}

impl BackendConfig {
    /// Get the provider name.
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Claude(_) => "claude",
            Self::Codex(_) => "codex",
            Self::Gemini(_) => "gemini",
            Self::Ollama(_) => "ollama",
        }
    }

    /// Get the default model for this provider.
    pub fn default_model(&self) -> &str {
        match self {
            Self::Claude(c) => &c.model,
            Self::Codex(c) => &c.model,
            Self::Gemini(c) => &c.model,
            Self::Ollama(c) => &c.model,
        }
    }
}

/// Common configuration shared across backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonBackendConfig {
    /// Request timeout.
    #[serde(with = "humantime_serde", default = "default_timeout")]
    pub timeout: Duration,
    /// Maximum retries on transient errors.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Enable request/response logging.
    #[serde(default)]
    pub debug_logging: bool,
}

impl Default for CommonBackendConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug_logging: false,
        }
    }
}

fn default_timeout() -> Duration {
    Duration::from_secs(120)
}

fn default_max_retries() -> u32 {
    3
}
```

### 2. Claude Configuration (src/config/claude.rs)

```rust
//! Claude (Anthropic) configuration.

use super::CommonBackendConfig;
use serde::{Deserialize, Serialize};
use tachikoma_common_config::Secret;

/// Claude API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// API key (can be loaded from env).
    #[serde(default)]
    pub api_key: Secret<String>,
    /// API base URL.
    #[serde(default = "default_claude_base_url")]
    pub base_url: String,
    /// Model to use.
    #[serde(default = "default_claude_model")]
    pub model: String,
    /// API version header.
    #[serde(default = "default_claude_api_version")]
    pub api_version: String,
    /// Maximum tokens in response.
    #[serde(default = "default_claude_max_tokens")]
    pub max_tokens: u32,
    /// Common backend settings.
    #[serde(flatten)]
    pub common: CommonBackendConfig,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: Secret::default(),
            base_url: default_claude_base_url(),
            model: default_claude_model(),
            api_version: default_claude_api_version(),
            max_tokens: default_claude_max_tokens(),
            common: CommonBackendConfig::default(),
        }
    }
}

impl ClaudeConfig {
    /// Create config from environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map(Secret::new)
            .map_err(|_| ConfigError::MissingEnvVar("ANTHROPIC_API_KEY"))?;

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| default_claude_base_url());

        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| default_claude_model());

        Ok(Self {
            api_key,
            base_url,
            model,
            ..Default::default()
        })
    }

    /// Messages API endpoint.
    pub fn messages_endpoint(&self) -> String {
        format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
    }
}

fn default_claude_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_claude_api_version() -> String {
    "2023-06-01".to_string()
}

fn default_claude_max_tokens() -> u32 {
    8192
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}
```

### 3. Codex Configuration (src/config/codex.rs)

```rust
//! Codex (OpenAI) configuration.

use super::CommonBackendConfig;
use serde::{Deserialize, Serialize};
use tachikoma_common_config::Secret;

/// OpenAI Codex API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// API key.
    #[serde(default)]
    pub api_key: Secret<String>,
    /// Organization ID (optional).
    #[serde(default)]
    pub organization: Option<String>,
    /// API base URL.
    #[serde(default = "default_codex_base_url")]
    pub base_url: String,
    /// Model to use.
    #[serde(default = "default_codex_model")]
    pub model: String,
    /// Maximum tokens in response.
    #[serde(default = "default_codex_max_tokens")]
    pub max_tokens: u32,
    /// Common backend settings.
    #[serde(flatten)]
    pub common: CommonBackendConfig,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            api_key: Secret::default(),
            organization: None,
            base_url: default_codex_base_url(),
            model: default_codex_model(),
            max_tokens: default_codex_max_tokens(),
            common: CommonBackendConfig::default(),
        }
    }
}

impl CodexConfig {
    /// Create config from environment.
    pub fn from_env() -> Result<Self, super::claude::ConfigError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map(Secret::new)
            .map_err(|_| super::claude::ConfigError::MissingEnvVar("OPENAI_API_KEY"))?;

        let organization = std::env::var("OPENAI_ORGANIZATION").ok();

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| default_codex_base_url());

        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| default_codex_model());

        Ok(Self {
            api_key,
            organization,
            base_url,
            model,
            ..Default::default()
        })
    }

    /// Chat completions endpoint.
    pub fn completions_endpoint(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

fn default_codex_base_url() -> String {
    "https://api.openai.com".to_string()
}

fn default_codex_model() -> String {
    "gpt-4o".to_string()
}

fn default_codex_max_tokens() -> u32 {
    4096
}
```

### 4. Gemini Configuration (src/config/gemini.rs)

```rust
//! Gemini (Google) configuration.

use super::CommonBackendConfig;
use serde::{Deserialize, Serialize};
use tachikoma_common_config::Secret;

/// Google Gemini API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// API key.
    #[serde(default)]
    pub api_key: Secret<String>,
    /// API base URL.
    #[serde(default = "default_gemini_base_url")]
    pub base_url: String,
    /// Model to use.
    #[serde(default = "default_gemini_model")]
    pub model: String,
    /// Maximum tokens in response.
    #[serde(default = "default_gemini_max_tokens")]
    pub max_output_tokens: u32,
    /// Safety settings.
    #[serde(default)]
    pub safety_settings: Vec<SafetySetting>,
    /// Common backend settings.
    #[serde(flatten)]
    pub common: CommonBackendConfig,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: Secret::default(),
            base_url: default_gemini_base_url(),
            model: default_gemini_model(),
            max_output_tokens: default_gemini_max_tokens(),
            safety_settings: vec![],
            common: CommonBackendConfig::default(),
        }
    }
}

impl GeminiConfig {
    /// Create config from environment.
    pub fn from_env() -> Result<Self, super::claude::ConfigError> {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .or_else(|_| std::env::var("GEMINI_API_KEY"))
            .map(Secret::new)
            .map_err(|_| super::claude::ConfigError::MissingEnvVar("GOOGLE_API_KEY"))?;

        let base_url = std::env::var("GEMINI_BASE_URL")
            .unwrap_or_else(|_| default_gemini_base_url());

        let model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| default_gemini_model());

        Ok(Self {
            api_key,
            base_url,
            model,
            ..Default::default()
        })
    }

    /// Generate content endpoint.
    pub fn generate_endpoint(&self) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            self.model
        )
    }

    /// Stream generate content endpoint.
    pub fn stream_endpoint(&self) -> String {
        format!(
            "{}/v1beta/models/{}:streamGenerateContent",
            self.base_url.trim_end_matches('/'),
            self.model
        )
    }
}

/// Gemini safety setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    /// Safety category.
    pub category: SafetyCategory,
    /// Block threshold.
    pub threshold: BlockThreshold,
}

/// Safety category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SafetyCategory {
    HarmCategoryHateSpeech,
    HarmCategoryDangerousContent,
    HarmCategorySexuallyExplicit,
    HarmCategoryHarassment,
}

/// Block threshold.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockThreshold {
    BlockNone,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
}

fn default_gemini_base_url() -> String {
    "https://generativelanguage.googleapis.com".to_string()
}

fn default_gemini_model() -> String {
    "gemini-1.5-pro".to_string()
}

fn default_gemini_max_tokens() -> u32 {
    8192
}
```

### 5. Ollama Configuration (src/config/ollama.rs)

```rust
//! Ollama (local) configuration.

use super::CommonBackendConfig;
use serde::{Deserialize, Serialize};

/// Ollama local API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// API base URL.
    #[serde(default = "default_ollama_base_url")]
    pub base_url: String,
    /// Model to use.
    #[serde(default = "default_ollama_model")]
    pub model: String,
    /// Keep model loaded in memory.
    #[serde(default = "default_keep_alive")]
    pub keep_alive: String,
    /// Number of GPU layers to use.
    #[serde(default)]
    pub num_gpu: Option<u32>,
    /// Context window size.
    #[serde(default = "default_ollama_context")]
    pub num_ctx: u32,
    /// Common backend settings.
    #[serde(flatten)]
    pub common: CommonBackendConfig,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: default_ollama_base_url(),
            model: default_ollama_model(),
            keep_alive: default_keep_alive(),
            num_gpu: None,
            num_ctx: default_ollama_context(),
            common: CommonBackendConfig::default(),
        }
    }
}

impl OllamaConfig {
    /// Create config from environment.
    pub fn from_env() -> Self {
        let base_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| default_ollama_base_url());

        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| default_ollama_model());

        Self {
            base_url,
            model,
            ..Default::default()
        }
    }

    /// Chat endpoint.
    pub fn chat_endpoint(&self) -> String {
        format!("{}/api/chat", self.base_url.trim_end_matches('/'))
    }

    /// Generate endpoint.
    pub fn generate_endpoint(&self) -> String {
        format!("{}/api/generate", self.base_url.trim_end_matches('/'))
    }

    /// Tags (list models) endpoint.
    pub fn tags_endpoint(&self) -> String {
        format!("{}/api/tags", self.base_url.trim_end_matches('/'))
    }

    /// Show model info endpoint.
    pub fn show_endpoint(&self) -> String {
        format!("{}/api/show", self.base_url.trim_end_matches('/'))
    }
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "llama3.1:8b".to_string()
}

fn default_keep_alive() -> String {
    "5m".to_string()
}

fn default_ollama_context() -> u32 {
    4096
}
```

### 6. Config Loading (src/config/loader.rs)

```rust
//! Configuration loading utilities.

use super::{BackendConfig, ClaudeConfig, CodexConfig, GeminiConfig, OllamaConfig};
use std::path::Path;

/// Load backend configuration from a YAML file.
pub fn load_from_file(path: &Path) -> Result<BackendConfig, ConfigLoadError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigLoadError::Io(e))?;

    serde_yaml::from_str(&content)
        .map_err(|e| ConfigLoadError::Parse(e.to_string()))
}

/// Load backend configuration from environment variables.
pub fn load_from_env(provider: &str) -> Result<BackendConfig, ConfigLoadError> {
    match provider.to_lowercase().as_str() {
        "claude" | "anthropic" => {
            ClaudeConfig::from_env()
                .map(BackendConfig::Claude)
                .map_err(|e| ConfigLoadError::EnvVar(e.to_string()))
        }
        "codex" | "openai" | "gpt" => {
            CodexConfig::from_env()
                .map(BackendConfig::Codex)
                .map_err(|e| ConfigLoadError::EnvVar(e.to_string()))
        }
        "gemini" | "google" => {
            GeminiConfig::from_env()
                .map(BackendConfig::Gemini)
                .map_err(|e| ConfigLoadError::EnvVar(e.to_string()))
        }
        "ollama" | "local" => {
            Ok(BackendConfig::Ollama(OllamaConfig::from_env()))
        }
        _ => Err(ConfigLoadError::UnknownProvider(provider.to_string())),
    }
}

/// Configuration loading errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("IO error: {0}")]
    Io(#[source] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("environment variable error: {0}")]
    EnvVar(String),
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
}
```

---

## Testing Requirements

1. Default configs have sensible values
2. Environment variable loading works
3. YAML serialization round-trips correctly
4. Endpoint URLs are constructed properly
5. Secret types don't leak in debug output

---

## Related Specs

- Depends on: [051-backend-trait.md](051-backend-trait.md)
- Depends on: [014-config-core-types.md](../phase-01-common/014-config-core-types.md)
- Next: [053-model-roles.md](053-model-roles.md)
- Used by: All backend implementations
