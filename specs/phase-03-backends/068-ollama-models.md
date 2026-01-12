# 068 - Ollama Models

**Phase:** 3 - Backend Abstraction Layer
**Spec ID:** 068
**Status:** Planned
**Dependencies:** 067-ollama-setup
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement model management for the Ollama backend, including model discovery, capability detection, loading/unloading, and recommended model configurations for different use cases.

---

## Acceptance Criteria

- [x] Model discovery and enumeration
- [x] Model capability detection (context size, tools)
- [x] Model pull/download support
- [x] Model loading state tracking
- [x] Recommended models for roles
- [x] Model-specific configurations

---

## Implementation Details

### 1. Model Registry (src/models/registry.rs)

```rust
//! Model registry for Ollama.

use std::collections::HashMap;
use tracing::{debug, info};

/// Known model capabilities.
#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    /// Context window size.
    pub context_size: u32,
    /// Supports tool/function calling.
    pub tool_calling: bool,
    /// Supports vision/images.
    pub vision: bool,
    /// Supports JSON mode.
    pub json_mode: bool,
    /// Recommended for coding tasks.
    pub coding_optimized: bool,
    /// Embedding model.
    pub embedding: bool,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            context_size: 4096,
            tool_calling: false,
            vision: false,
            json_mode: true,
            coding_optimized: false,
            embedding: false,
        }
    }
}

/// Known model configurations.
#[derive(Debug, Clone)]
pub struct KnownModel {
    /// Model name pattern.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Model family.
    pub family: ModelFamily,
    /// Known capabilities.
    pub capabilities: ModelCapabilities,
    /// Recommended temperature.
    pub default_temperature: f32,
}

/// Model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    Llama,
    Mistral,
    CodeLlama,
    Gemma,
    Phi,
    Qwen,
    DeepSeek,
    Other,
}

/// Registry of known models.
pub struct ModelRegistry {
    models: HashMap<String, KnownModel>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    /// Create a new registry with known models.
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        registry.register_known_models();
        registry
    }

    /// Register known model configurations.
    fn register_known_models(&mut self) {
        // Llama 3.1 models
        self.register(KnownModel {
            name: "llama3.1".to_string(),
            display_name: "Llama 3.1".to_string(),
            family: ModelFamily::Llama,
            capabilities: ModelCapabilities {
                context_size: 128_000,
                tool_calling: true,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Llama 3.2 vision models
        self.register(KnownModel {
            name: "llama3.2-vision".to_string(),
            display_name: "Llama 3.2 Vision".to_string(),
            family: ModelFamily::Llama,
            capabilities: ModelCapabilities {
                context_size: 128_000,
                tool_calling: true,
                vision: true,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // CodeLlama
        self.register(KnownModel {
            name: "codellama".to_string(),
            display_name: "Code Llama".to_string(),
            family: ModelFamily::CodeLlama,
            capabilities: ModelCapabilities {
                context_size: 16_384,
                tool_calling: false,
                vision: false,
                json_mode: true,
                coding_optimized: true,
                embedding: false,
            },
            default_temperature: 0.2,
        });

        // Mistral
        self.register(KnownModel {
            name: "mistral".to_string(),
            display_name: "Mistral".to_string(),
            family: ModelFamily::Mistral,
            capabilities: ModelCapabilities {
                context_size: 32_768,
                tool_calling: true,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Mixtral
        self.register(KnownModel {
            name: "mixtral".to_string(),
            display_name: "Mixtral".to_string(),
            family: ModelFamily::Mistral,
            capabilities: ModelCapabilities {
                context_size: 32_768,
                tool_calling: true,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Gemma 2
        self.register(KnownModel {
            name: "gemma2".to_string(),
            display_name: "Gemma 2".to_string(),
            family: ModelFamily::Gemma,
            capabilities: ModelCapabilities {
                context_size: 8192,
                tool_calling: false,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Phi-3
        self.register(KnownModel {
            name: "phi3".to_string(),
            display_name: "Phi-3".to_string(),
            family: ModelFamily::Phi,
            capabilities: ModelCapabilities {
                context_size: 128_000,
                tool_calling: false,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Qwen 2.5
        self.register(KnownModel {
            name: "qwen2.5".to_string(),
            display_name: "Qwen 2.5".to_string(),
            family: ModelFamily::Qwen,
            capabilities: ModelCapabilities {
                context_size: 32_768,
                tool_calling: true,
                vision: false,
                json_mode: true,
                coding_optimized: false,
                embedding: false,
            },
            default_temperature: 0.7,
        });

        // Qwen 2.5 Coder
        self.register(KnownModel {
            name: "qwen2.5-coder".to_string(),
            display_name: "Qwen 2.5 Coder".to_string(),
            family: ModelFamily::Qwen,
            capabilities: ModelCapabilities {
                context_size: 32_768,
                tool_calling: true,
                vision: false,
                json_mode: true,
                coding_optimized: true,
                embedding: false,
            },
            default_temperature: 0.2,
        });

        // DeepSeek Coder
        self.register(KnownModel {
            name: "deepseek-coder".to_string(),
            display_name: "DeepSeek Coder".to_string(),
            family: ModelFamily::DeepSeek,
            capabilities: ModelCapabilities {
                context_size: 16_384,
                tool_calling: false,
                vision: false,
                json_mode: true,
                coding_optimized: true,
                embedding: false,
            },
            default_temperature: 0.2,
        });
    }

    /// Register a model.
    pub fn register(&mut self, model: KnownModel) {
        self.models.insert(model.name.clone(), model);
    }

    /// Get model info by name (matches prefix).
    pub fn get(&self, name: &str) -> Option<&KnownModel> {
        // Try exact match first
        if let Some(model) = self.models.get(name) {
            return Some(model);
        }

        // Try prefix match (e.g., "llama3.1:8b" matches "llama3.1")
        let base_name = name.split(':').next().unwrap_or(name);
        self.models.get(base_name)
    }

    /// Get capabilities for a model.
    pub fn capabilities(&self, name: &str) -> ModelCapabilities {
        self.get(name)
            .map(|m| m.capabilities.clone())
            .unwrap_or_default()
    }

    /// Get all known models.
    pub fn all(&self) -> Vec<&KnownModel> {
        self.models.values().collect()
    }

    /// Get models suitable for a role.
    pub fn for_role(&self, role: ModelRole) -> Vec<&KnownModel> {
        self.models
            .values()
            .filter(|m| role.matches_model(m))
            .collect()
    }
}

/// Role-based model selection.
#[derive(Debug, Clone, Copy)]
pub enum ModelRole {
    /// General purpose reasoning.
    General,
    /// Code generation and editing.
    Coding,
    /// Fast responses.
    Fast,
    /// Tool calling.
    ToolUse,
    /// Vision/multimodal.
    Vision,
}

impl ModelRole {
    /// Check if a model matches this role.
    pub fn matches_model(&self, model: &KnownModel) -> bool {
        match self {
            Self::General => true,
            Self::Coding => model.capabilities.coding_optimized,
            Self::Fast => model.capabilities.context_size <= 8192,
            Self::ToolUse => model.capabilities.tool_calling,
            Self::Vision => model.capabilities.vision,
        }
    }
}
```

### 2. Model Manager (src/models/manager.rs)

```rust
//! Model manager for Ollama.

use super::registry::{ModelCapabilities, ModelRegistry};
use crate::server::OllamaServer;
use crate::api_types::ModelInfo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// State of a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelState {
    /// Not downloaded.
    NotPresent,
    /// Downloaded but not loaded.
    Available,
    /// Currently loading.
    Loading,
    /// Loaded in memory.
    Loaded,
    /// Error state.
    Error,
}

/// Information about a model's current state.
#[derive(Debug, Clone)]
pub struct ModelStatus {
    pub name: String,
    pub state: ModelState,
    pub size_bytes: u64,
    pub capabilities: ModelCapabilities,
    pub last_used: Option<std::time::Instant>,
}

/// Manager for Ollama models.
pub struct ModelManager {
    server: Arc<OllamaServer>,
    registry: ModelRegistry,
    status: RwLock<HashMap<String, ModelStatus>>,
}

impl ModelManager {
    /// Create a new model manager.
    pub fn new(server: Arc<OllamaServer>) -> Self {
        Self {
            server,
            registry: ModelRegistry::new(),
            status: RwLock::new(HashMap::new()),
        }
    }

    /// Refresh the list of available models.
    pub async fn refresh(&self) -> Result<(), ManagerError> {
        debug!("Refreshing model list");

        let models = self.server.list_models().await
            .map_err(|e| ManagerError::ServerError(e.to_string()))?;

        let mut status = self.status.write().await;
        status.clear();

        for model in models {
            let capabilities = self.registry.capabilities(&model.name);
            status.insert(model.name.clone(), ModelStatus {
                name: model.name,
                state: ModelState::Available,
                size_bytes: model.size,
                capabilities,
                last_used: None,
            });
        }

        info!(count = status.len(), "Refreshed model list");
        Ok(())
    }

    /// Get status of a specific model.
    pub async fn get_status(&self, name: &str) -> Option<ModelStatus> {
        self.status.read().await.get(name).cloned()
    }

    /// Get all model statuses.
    pub async fn all_statuses(&self) -> Vec<ModelStatus> {
        self.status.read().await.values().cloned().collect()
    }

    /// Check if a model is available.
    pub async fn is_available(&self, name: &str) -> bool {
        self.status
            .read()
            .await
            .get(name)
            .map(|s| s.state != ModelState::NotPresent)
            .unwrap_or(false)
    }

    /// Pull a model if not present.
    pub async fn ensure_available(&self, name: &str) -> Result<(), ManagerError> {
        if self.is_available(name).await {
            return Ok(());
        }

        info!(model = %name, "Pulling model");

        // Update state to loading
        {
            let mut status = self.status.write().await;
            status.insert(name.to_string(), ModelStatus {
                name: name.to_string(),
                state: ModelState::Loading,
                size_bytes: 0,
                capabilities: self.registry.capabilities(name),
                last_used: None,
            });
        }

        // Pull the model
        self.server.pull_model(name).await
            .map_err(|e| ManagerError::PullFailed(e.to_string()))?;

        // Update state
        {
            let mut status = self.status.write().await;
            if let Some(s) = status.get_mut(name) {
                s.state = ModelState::Available;
            }
        }

        Ok(())
    }

    /// Mark a model as used (updates last_used timestamp).
    pub async fn mark_used(&self, name: &str) {
        let mut status = self.status.write().await;
        if let Some(s) = status.get_mut(name) {
            s.last_used = Some(std::time::Instant::now());
            s.state = ModelState::Loaded;
        }
    }

    /// Get the registry.
    pub fn registry(&self) -> &ModelRegistry {
        &self.registry
    }

    /// Get capabilities for a model.
    pub fn capabilities(&self, name: &str) -> ModelCapabilities {
        self.registry.capabilities(name)
    }

    /// Recommend a model for a given role.
    pub async fn recommend(&self, role: super::registry::ModelRole) -> Option<String> {
        let status = self.status.read().await;

        // Get available models that match the role
        let candidates: Vec<&ModelStatus> = status
            .values()
            .filter(|s| s.state != ModelState::NotPresent)
            .filter(|s| role.matches_model(
                self.registry.get(&s.name).unwrap_or(&super::registry::KnownModel {
                    name: s.name.clone(),
                    display_name: s.name.clone(),
                    family: super::registry::ModelFamily::Other,
                    capabilities: s.capabilities.clone(),
                    default_temperature: 0.7,
                })
            ))
            .collect();

        // Prefer recently used models
        candidates
            .iter()
            .max_by_key(|s| s.last_used)
            .map(|s| s.name.clone())
    }
}

impl std::fmt::Debug for ModelManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelManager").finish()
    }
}

/// Manager errors.
#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("server error: {0}")]
    ServerError(String),

    #[error("failed to pull model: {0}")]
    PullFailed(String),

    #[error("model not found: {0}")]
    NotFound(String),
}
```

### 3. Module Exports (src/models/mod.rs)

```rust
//! Model management for Ollama.

mod manager;
mod registry;

pub use manager::{ManagerError, ModelManager, ModelState, ModelStatus};
pub use registry::{KnownModel, ModelCapabilities, ModelFamily, ModelRegistry, ModelRole};
```

---

## Testing Requirements

1. Registry returns correct capabilities
2. Model refresh updates status
3. Model pull works correctly
4. Role-based recommendation works
5. State transitions are correct

---

## Related Specs

- Depends on: [067-ollama-setup.md](067-ollama-setup.md)
- Next: [069-ollama-tools.md](069-ollama-tools.md)
