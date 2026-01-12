# Spec 331: Shared State Management

## Phase
15 - Server/API Layer

## Spec ID
331

## Status
Planned

## Dependencies
- Spec 311: Server Setup

## Estimated Context
~9%

---

## Objective

Implement comprehensive shared state management for the Tachikoma server, providing thread-safe access to application state, connection pools, caches, and runtime configuration across all handlers.

---

## Acceptance Criteria

- [ ] Thread-safe shared state across handlers
- [ ] Connection pool management
- [ ] In-memory caching layer
- [ ] Runtime configuration updates
- [ ] State initialization and cleanup
- [ ] State snapshot for debugging
- [ ] Graceful state migration during updates

---

## Implementation Details

### Application State

```rust
// src/server/state/app.rs
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

use crate::storage::Storage;
use crate::forge::ForgeRegistry;
use crate::backend::BackendManager;
use crate::server::config::ServerConfig;
use crate::server::websocket::ConnectionManager;
use crate::server::health::HealthService;
use crate::server::ratelimit::RateLimiter;

/// Main application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    /// Server configuration
    config: ServerConfig,

    /// Storage layer
    storage: Arc<dyn Storage>,

    /// Forge registry
    forge_registry: Arc<RwLock<ForgeRegistry>>,

    /// Backend manager for LLM providers
    backend_manager: Arc<BackendManager>,

    /// WebSocket connection manager
    ws_manager: Arc<ConnectionManager>,

    /// Health check service
    health_service: Arc<HealthService>,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,

    /// In-memory cache
    cache: Arc<Cache>,

    /// Runtime configuration (mutable)
    runtime_config: Arc<RwLock<RuntimeConfig>>,

    /// Server start time
    started_at: Instant,

    /// Build information
    build_info: BuildInfo,
}

impl AppState {
    pub fn new(builder: AppStateBuilder) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config: builder.config,
                storage: builder.storage,
                forge_registry: Arc::new(RwLock::new(builder.forge_registry)),
                backend_manager: Arc::new(builder.backend_manager),
                ws_manager: Arc::new(ConnectionManager::new().0),
                health_service: Arc::new(builder.health_service),
                rate_limiter: Arc::new(builder.rate_limiter),
                cache: Arc::new(Cache::new(builder.cache_config)),
                runtime_config: Arc::new(RwLock::new(RuntimeConfig::default())),
                started_at: Instant::now(),
                build_info: BuildInfo::current(),
            }),
        }
    }

    // Accessors

    pub fn config(&self) -> &ServerConfig {
        &self.inner.config
    }

    pub fn storage(&self) -> &Arc<dyn Storage> {
        &self.inner.storage
    }

    pub fn forge_registry(&self) -> &Arc<RwLock<ForgeRegistry>> {
        &self.inner.forge_registry
    }

    pub fn backend_manager(&self) -> &Arc<BackendManager> {
        &self.inner.backend_manager
    }

    pub fn ws_manager(&self) -> &Arc<ConnectionManager> {
        &self.inner.ws_manager
    }

    pub fn health_service(&self) -> &Arc<HealthService> {
        &self.inner.health_service
    }

    pub fn rate_limiter(&self) -> &Arc<RateLimiter> {
        &self.inner.rate_limiter
    }

    pub fn cache(&self) -> &Arc<Cache> {
        &self.inner.cache
    }

    pub fn runtime_config(&self) -> &Arc<RwLock<RuntimeConfig>> {
        &self.inner.runtime_config
    }

    pub fn uptime(&self) -> std::time::Duration {
        self.inner.started_at.elapsed()
    }

    pub fn build_info(&self) -> &BuildInfo {
        &self.inner.build_info
    }

    /// Get a snapshot of current state for debugging
    pub async fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            uptime_secs: self.uptime().as_secs(),
            ws_connections: self.ws_manager().connection_count().await,
            active_backends: self.backend_manager().active_count(),
            forge_count: self.forge_registry().read().await.count(),
            cache_stats: self.cache().stats(),
            runtime_config: self.runtime_config().read().await.clone(),
        }
    }
}

/// Builder for AppState
pub struct AppStateBuilder {
    config: ServerConfig,
    storage: Arc<dyn Storage>,
    forge_registry: ForgeRegistry,
    backend_manager: BackendManager,
    health_service: HealthService,
    rate_limiter: RateLimiter,
    cache_config: CacheConfig,
}

impl AppStateBuilder {
    pub fn new(config: ServerConfig, storage: Arc<dyn Storage>) -> Self {
        Self {
            config: config.clone(),
            storage,
            forge_registry: ForgeRegistry::new(),
            backend_manager: BackendManager::new(),
            health_service: HealthService::new(/* ... */),
            rate_limiter: RateLimiter::new(config.api.rate_limit_config()),
            cache_config: CacheConfig::default(),
        }
    }

    pub fn forge_registry(mut self, registry: ForgeRegistry) -> Self {
        self.forge_registry = registry;
        self
    }

    pub fn backend_manager(mut self, manager: BackendManager) -> Self {
        self.backend_manager = manager;
        self
    }

    pub fn health_service(mut self, service: HealthService) -> Self {
        self.health_service = service;
        self
    }

    pub fn rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = limiter;
        self
    }

    pub fn cache_config(mut self, config: CacheConfig) -> Self {
        self.cache_config = config;
        self
    }

    pub fn build(self) -> AppState {
        AppState::new(self)
    }
}

/// State snapshot for debugging
#[derive(Debug, Clone, Serialize)]
pub struct StateSnapshot {
    pub uptime_secs: u64,
    pub ws_connections: usize,
    pub active_backends: usize,
    pub forge_count: usize,
    pub cache_stats: CacheStats,
    pub runtime_config: RuntimeConfig,
}
```

### Runtime Configuration

```rust
// src/server/state/runtime.rs
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

/// Runtime-configurable settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Log level
    pub log_level: LogLevel,

    /// Rate limit multiplier (for emergency throttling)
    pub rate_limit_multiplier: f32,

    /// Maintenance mode
    pub maintenance_mode: bool,

    /// Feature flags
    pub features: RuntimeFeatures,

    /// Custom settings
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFeatures {
    pub websockets_enabled: bool,
    pub streaming_enabled: bool,
    pub new_backends_enabled: bool,
    pub experimental_features: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            rate_limit_multiplier: 1.0,
            maintenance_mode: false,
            features: RuntimeFeatures::default(),
            custom: std::collections::HashMap::new(),
        }
    }
}

impl Default for RuntimeFeatures {
    fn default() -> Self {
        Self {
            websockets_enabled: true,
            streaming_enabled: true,
            new_backends_enabled: true,
            experimental_features: false,
        }
    }
}

/// Handle for watching runtime config changes
pub struct RuntimeConfigWatcher {
    receiver: watch::Receiver<RuntimeConfig>,
}

impl RuntimeConfigWatcher {
    pub fn new(initial: RuntimeConfig) -> (RuntimeConfigUpdater, Self) {
        let (sender, receiver) = watch::channel(initial);
        (
            RuntimeConfigUpdater { sender },
            Self { receiver },
        )
    }

    pub fn current(&self) -> RuntimeConfig {
        self.receiver.borrow().clone()
    }

    pub async fn wait_for_change(&mut self) -> RuntimeConfig {
        self.receiver.changed().await.ok();
        self.receiver.borrow().clone()
    }
}

/// Handle for updating runtime config
pub struct RuntimeConfigUpdater {
    sender: watch::Sender<RuntimeConfig>,
}

impl RuntimeConfigUpdater {
    pub fn update(&self, config: RuntimeConfig) {
        let _ = self.sender.send(config);
    }

    pub fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut RuntimeConfig),
    {
        self.sender.send_modify(f);
    }
}
```

### In-Memory Cache

```rust
// src/server/state/cache.rs
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,

    /// Default TTL
    pub default_ttl: Duration,

    /// Enable statistics
    pub stats_enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            stats_enabled: true,
        }
    }
}

/// Thread-safe in-memory cache
pub struct Cache {
    config: CacheConfig,
    entries: RwLock<HashMap<String, CacheEntry>>,
    stats: RwLock<CacheStats>,
}

struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
    created_at: Instant,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub evictions: u64,
    pub current_entries: usize,
}

impl Cache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
        }
    }

    /// Get a value from cache
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        let entries = self.entries.read().await;

        if let Some(entry) = entries.get(key) {
            if entry.expires_at > Instant::now() {
                if self.config.stats_enabled {
                    self.stats.write().await.hits += 1;
                }
                return serde_json::from_value(entry.value.clone()).ok();
            }
        }

        if self.config.stats_enabled {
            self.stats.write().await.misses += 1;
        }

        None
    }

    /// Set a value in cache with default TTL
    pub async fn set<T: Serialize>(&self, key: impl Into<String>, value: &T) {
        self.set_with_ttl(key, value, self.config.default_ttl).await
    }

    /// Set a value in cache with custom TTL
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: impl Into<String>,
        value: &T,
        ttl: Duration,
    ) {
        let key = key.into();
        let value = serde_json::to_value(value).unwrap();
        let now = Instant::now();

        let mut entries = self.entries.write().await;

        // Evict if at capacity
        if entries.len() >= self.config.max_entries {
            self.evict_oldest(&mut entries).await;
        }

        entries.insert(key, CacheEntry {
            value,
            expires_at: now + ttl,
            created_at: now,
        });

        if self.config.stats_enabled {
            let mut stats = self.stats.write().await;
            stats.inserts += 1;
            stats.current_entries = entries.len();
        }
    }

    /// Remove a value from cache
    pub async fn remove(&self, key: &str) -> bool {
        self.entries.write().await.remove(key).is_some()
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.entries.write().await.clear();
        if self.config.stats_enabled {
            self.stats.write().await.current_entries = 0;
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        // Return default stats synchronously; use stats() for async
        CacheStats::default()
    }

    pub async fn stats_async(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Remove expired entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut entries = self.entries.write().await;
        let before = entries.len();

        entries.retain(|_, entry| entry.expires_at > now);

        let evicted = before - entries.len();
        if self.config.stats_enabled && evicted > 0 {
            let mut stats = self.stats.write().await;
            stats.evictions += evicted as u64;
            stats.current_entries = entries.len();
        }
    }

    async fn evict_oldest(&self, entries: &mut HashMap<String, CacheEntry>) {
        // Find the oldest entry
        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, v)| v.created_at)
            .map(|(k, _)| k.clone())
        {
            entries.remove(&oldest_key);
            if self.config.stats_enabled {
                self.stats.write().await.evictions += 1;
            }
        }
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                self.cleanup().await;
            }
        })
    }
}

/// Cache key builder for consistent key generation
pub struct CacheKey;

impl CacheKey {
    pub fn mission(id: uuid::Uuid) -> String {
        format!("mission:{}", id)
    }

    pub fn spec(id: uuid::Uuid) -> String {
        format!("spec:{}", id)
    }

    pub fn user(id: uuid::Uuid) -> String {
        format!("user:{}", id)
    }

    pub fn settings(scope: &str) -> String {
        format!("settings:{}", scope)
    }

    pub fn list(resource: &str, query_hash: u64) -> String {
        format!("list:{}:{}", resource, query_hash)
    }
}
```

### Build Information

```rust
// src/server/state/build.rs

/// Build information compiled into the binary
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: &'static str,
    pub git_hash: Option<&'static str>,
    pub git_branch: Option<&'static str>,
    pub build_time: &'static str,
    pub rust_version: &'static str,
    pub target: &'static str,
    pub profile: &'static str,
}

impl BuildInfo {
    pub fn current() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_hash: option_env!("GIT_HASH"),
            git_branch: option_env!("GIT_BRANCH"),
            build_time: env!("BUILD_TIME"),
            rust_version: env!("RUSTC_VERSION"),
            target: env!("TARGET"),
            profile: if cfg!(debug_assertions) { "debug" } else { "release" },
        }
    }
}

// Build script (build.rs) to set environment variables
/*
fn main() {
    // Git hash
    if let Ok(output) = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
        }
    }

    // Git branch
    if let Ok(output) = std::process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
    {
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_BRANCH={}", branch.trim());
        }
    }

    // Build time
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now().to_rfc3339());

    // Rust version
    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version::version().unwrap());

    // Target
    println!("cargo:rustc-env=TARGET={}", std::env::var("TARGET").unwrap());
}
*/
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = Cache::new(CacheConfig::default());

        cache.set("key", &"value").await;
        let result: Option<String> = cache.get("key").await;

        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            default_ttl: Duration::from_millis(50),
            ..Default::default()
        };
        let cache = Cache::new(config);

        cache.set("key", &"value").await;

        // Should exist immediately
        let result: Option<String> = cache.get("key").await;
        assert!(result.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be expired
        let result: Option<String> = cache.get("key").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = Cache::new(config);

        cache.set("key1", &"value1").await;
        cache.set("key2", &"value2").await;
        cache.set("key3", &"value3").await;

        // Should have evicted one entry
        let stats = cache.stats_async().await;
        assert_eq!(stats.evictions, 1);
    }

    #[tokio::test]
    async fn test_app_state_builder() {
        let storage = Arc::new(MockStorage::new());
        let config = ServerConfig::default();

        let state = AppStateBuilder::new(config, storage)
            .build();

        assert!(state.uptime().as_secs() == 0);
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 312**: Server Configuration
- **Spec 332**: Graceful Shutdown
