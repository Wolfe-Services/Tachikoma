# Spec 322: Health Checks API

## Phase
15 - Server/API Layer

## Spec ID
322

## Status
Planned

## Dependencies
- Spec 311: Server Setup

## Estimated Context
~8%

---

## Objective

Implement comprehensive health check endpoints for the Tachikoma server, providing liveness, readiness, and detailed component health information for monitoring, load balancing, and debugging purposes.

---

## Acceptance Criteria

- [ ] Liveness endpoint returns server alive status
- [ ] Readiness endpoint checks all critical dependencies
- [ ] Detailed health endpoint shows component status
- [ ] Health checks are fast and non-blocking
- [ ] Metrics are exposed for monitoring systems
- [ ] Health history is tracked for debugging
- [ ] Custom health checks can be registered

---

## Implementation Details

### Health Types

```rust
// src/api/types/health.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Simple liveness response
#[derive(Debug, Clone, Serialize)]
pub struct LivenessResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}

/// Readiness response
#[derive(Debug, Clone, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub status: HealthStatus,
    pub checks: Vec<ReadinessCheck>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadinessCheck {
    pub name: String,
    pub ready: bool,
    pub message: Option<String>,
}

/// Detailed health response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: VersionInfo,
    pub uptime: UptimeInfo,
    pub components: Vec<ComponentHealth>,
    pub system: SystemHealth,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub git_hash: Option<String>,
    pub build_time: String,
    pub rust_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UptimeInfo {
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub uptime_human: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<i64>,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub disk_usage_percent: f32,
    pub open_file_descriptors: Option<u64>,
    pub thread_count: u32,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health endpoints
    pub enabled: bool,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Timeout for individual checks
    pub check_timeout_secs: u64,
    /// Include detailed system info
    pub include_system_info: bool,
    /// Components to check
    pub components: Vec<String>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 30,
            check_timeout_secs: 5,
            include_system_info: true,
            components: vec![
                "database".to_string(),
                "cache".to_string(),
                "backends".to_string(),
            ],
        }
    }
}
```

### Health Check Service

```rust
// src/server/health/service.rs
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::api::types::health::*;
use crate::server::state::AppState;

/// Health check service that manages component health
pub struct HealthService {
    state: AppState,
    config: HealthCheckConfig,
    checkers: Arc<RwLock<HashMap<String, Box<dyn HealthChecker>>>>,
    cache: Arc<RwLock<HealthCache>>,
}

struct HealthCache {
    components: HashMap<String, ComponentHealth>,
    last_full_check: Option<Instant>,
}

/// Trait for implementing custom health checks
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> ComponentHealth;
}

impl HealthService {
    pub fn new(state: AppState, config: HealthCheckConfig) -> Self {
        Self {
            state,
            config,
            checkers: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HealthCache {
                components: HashMap::new(),
                last_full_check: None,
            })),
        }
    }

    /// Register a custom health checker
    pub async fn register_checker(&self, checker: Box<dyn HealthChecker>) {
        let name = checker.name().to_string();
        self.checkers.write().await.insert(name, checker);
    }

    /// Start background health checking
    pub fn start_background_checks(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let interval_secs = self.config.check_interval_secs;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));

            loop {
                interval.tick().await;
                if let Err(e) = self.run_all_checks().await {
                    tracing::error!(error = %e, "Background health check failed");
                }
            }
        })
    }

    /// Run all health checks
    pub async fn run_all_checks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let checkers = self.checkers.read().await;
        let timeout = Duration::from_secs(self.config.check_timeout_secs);

        let mut results = HashMap::new();

        for (name, checker) in checkers.iter() {
            let result = tokio::time::timeout(timeout, checker.check()).await;

            let health = match result {
                Ok(h) => h,
                Err(_) => ComponentHealth {
                    name: name.clone(),
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(timeout.as_millis() as i64),
                    message: Some("Health check timed out".to_string()),
                    details: None,
                    last_check: Utc::now(),
                },
            };

            results.insert(name.clone(), health);
        }

        let mut cache = self.cache.write().await;
        cache.components = results;
        cache.last_full_check = Some(Instant::now());

        Ok(())
    }

    /// Get cached component health
    pub async fn get_component_health(&self, name: &str) -> Option<ComponentHealth> {
        self.cache.read().await.components.get(name).cloned()
    }

    /// Get all cached health status
    pub async fn get_all_health(&self) -> Vec<ComponentHealth> {
        self.cache.read().await.components.values().cloned().collect()
    }

    /// Check if system is ready
    pub async fn is_ready(&self) -> bool {
        let cache = self.cache.read().await;
        cache.components.values().all(|c| c.status != HealthStatus::Unhealthy)
    }

    /// Get overall health status
    pub async fn overall_status(&self) -> HealthStatus {
        let cache = self.cache.read().await;

        if cache.components.is_empty() {
            return HealthStatus::Healthy;
        }

        let has_unhealthy = cache.components.values().any(|c| c.status == HealthStatus::Unhealthy);
        let has_degraded = cache.components.values().any(|c| c.status == HealthStatus::Degraded);

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}
```

### Built-in Health Checkers

```rust
// src/server/health/checkers.rs
use super::service::HealthChecker;
use crate::api::types::health::*;

/// Database health checker
pub struct DatabaseHealthChecker {
    storage: Arc<dyn Storage>,
}

impl DatabaseHealthChecker {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl HealthChecker for DatabaseHealthChecker {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        match self.storage.ping().await {
            Ok(_) => ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as i64),
                message: None,
                details: Some(serde_json::json!({
                    "type": self.storage.storage_type(),
                    "pool_size": self.storage.pool_size().await,
                })),
                last_check: Utc::now(),
            },
            Err(e) => ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: Some(start.elapsed().as_millis() as i64),
                message: Some(e.to_string()),
                details: None,
                last_check: Utc::now(),
            },
        }
    }
}

/// Backend manager health checker
pub struct BackendsHealthChecker {
    backend_manager: Arc<BackendManager>,
}

impl BackendsHealthChecker {
    pub fn new(backend_manager: Arc<BackendManager>) -> Self {
        Self { backend_manager }
    }
}

#[async_trait::async_trait]
impl HealthChecker for BackendsHealthChecker {
    fn name(&self) -> &str {
        "backends"
    }

    async fn check(&self) -> ComponentHealth {
        let backends = self.backend_manager.list_all();
        let total = backends.len();

        if total == 0 {
            return ComponentHealth {
                name: "backends".to_string(),
                status: HealthStatus::Degraded,
                latency_ms: None,
                message: Some("No backends configured".to_string()),
                details: None,
                last_check: Utc::now(),
            };
        }

        let mut healthy = 0;
        let mut backend_status = Vec::new();

        for backend in &backends {
            let health = backend.health_check().await;
            if health.status == HealthStatus::Healthy {
                healthy += 1;
            }
            backend_status.push(serde_json::json!({
                "name": backend.config().name,
                "status": health.status,
            }));
        }

        let status = if healthy == total {
            HealthStatus::Healthy
        } else if healthy > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        ComponentHealth {
            name: "backends".to_string(),
            status,
            latency_ms: None,
            message: Some(format!("{}/{} backends healthy", healthy, total)),
            details: Some(serde_json::json!({
                "backends": backend_status,
            })),
            last_check: Utc::now(),
        }
    }
}

/// Forge registry health checker
pub struct ForgeHealthChecker {
    forge_registry: Arc<RwLock<ForgeRegistry>>,
}

impl ForgeHealthChecker {
    pub fn new(forge_registry: Arc<RwLock<ForgeRegistry>>) -> Self {
        Self { forge_registry }
    }
}

#[async_trait::async_trait]
impl HealthChecker for ForgeHealthChecker {
    fn name(&self) -> &str {
        "forges"
    }

    async fn check(&self) -> ComponentHealth {
        let registry = self.forge_registry.read().await;
        let forges = registry.list_all();
        let total = forges.len();

        if total == 0 {
            return ComponentHealth {
                name: "forges".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: None,
                message: Some("No forges configured".to_string()),
                details: None,
                last_check: Utc::now(),
            };
        }

        let mut connected = 0;
        for forge in &forges {
            if forge.is_connected() {
                connected += 1;
            }
        }

        let status = if connected == total {
            HealthStatus::Healthy
        } else if connected > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        ComponentHealth {
            name: "forges".to_string(),
            status,
            latency_ms: None,
            message: Some(format!("{}/{} forges connected", connected, total)),
            details: Some(serde_json::json!({
                "total": total,
                "connected": connected,
            })),
            last_check: Utc::now(),
        }
    }
}
```

### Health Handlers

```rust
// src/server/handlers/health.rs
use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};

use crate::api::types::health::*;
use crate::server::state::AppState;

/// Liveness probe - is the server running?
#[utoipa::path(
    get,
    path = "/health/live",
    responses(
        (status = 200, description = "Server is alive", body = LivenessResponse),
    ),
    tag = "health"
)]
pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

/// Readiness probe - is the server ready to accept traffic?
#[utoipa::path(
    get,
    path = "/health/ready",
    responses(
        (status = 200, description = "Server is ready"),
        (status = 503, description = "Server is not ready"),
    ),
    tag = "health"
)]
pub async fn readiness(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let health_service = state.health_service();
    let is_ready = health_service.is_ready().await;

    let checks: Vec<ReadinessCheck> = health_service
        .get_all_health()
        .await
        .into_iter()
        .map(|c| ReadinessCheck {
            name: c.name,
            ready: c.status != HealthStatus::Unhealthy,
            message: c.message,
        })
        .collect();

    let response = ReadinessResponse {
        ready: is_ready,
        status: if is_ready { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
        checks,
        timestamp: Utc::now(),
    };

    if is_ready {
        (StatusCode::OK, Json(response))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// Detailed health check
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Detailed health information", body = HealthResponse),
    ),
    tag = "health"
)]
pub async fn health(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    let health_service = state.health_service();
    let build_info = state.build_info();
    let uptime = state.uptime();

    let components = health_service.get_all_health().await;
    let overall_status = health_service.overall_status().await;
    let system = get_system_health();

    Json(HealthResponse {
        status: overall_status,
        version: VersionInfo {
            version: build_info.version.to_string(),
            git_hash: build_info.git_hash.map(|s| s.to_string()),
            build_time: build_info.build_time.to_string(),
            rust_version: build_info.rust_version.to_string(),
        },
        uptime: UptimeInfo {
            started_at: Utc::now() - chrono::Duration::from_std(uptime).unwrap(),
            uptime_seconds: uptime.as_secs(),
            uptime_human: format_duration(uptime),
        },
        components,
        system,
        timestamp: Utc::now(),
    })
}

/// Get specific component health
pub async fn component_health(
    State(state): State<AppState>,
    Path(component): Path<String>,
) -> impl IntoResponse {
    let health_service = state.health_service();

    match health_service.get_component_health(&component).await {
        Some(health) => {
            let status = match health.status {
                HealthStatus::Healthy => StatusCode::OK,
                HealthStatus::Degraded => StatusCode::OK,
                HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
            };
            (status, Json(health)).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Helper functions

fn get_system_health() -> SystemHealth {
    use sysinfo::{System, SystemExt, CpuExt, DiskExt};

    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let memory_used = sys.used_memory() / 1024 / 1024; // MB
    let memory_total = sys.total_memory() / 1024 / 1024; // MB

    let disk = sys.disks().first();
    let (disk_used, disk_total) = disk
        .map(|d| {
            let total = d.total_space() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
            let available = d.available_space() as f64 / 1024.0 / 1024.0 / 1024.0; // GB
            (total - available, total)
        })
        .unwrap_or((0.0, 0.0));

    SystemHealth {
        cpu_usage_percent: cpu_usage,
        memory_used_mb: memory_used,
        memory_total_mb: memory_total,
        memory_usage_percent: (memory_used as f32 / memory_total as f32) * 100.0,
        disk_used_gb: disk_used,
        disk_total_gb: disk_total,
        disk_usage_percent: ((disk_used / disk_total) * 100.0) as f32,
        open_file_descriptors: None, // Platform-specific
        thread_count: sys.processes().len() as u32,
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
```

### Routes

```rust
// src/server/routes/health.rs
use axum::{
    Router,
    routing::get,
};

use crate::server::state::AppState;
use crate::server::handlers::health as handlers;

pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/live", get(handlers::liveness))
        .route("/health/ready", get(handlers::readiness))
        .route("/health/:component", get(handlers::component_health))
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness_always_returns_ok() {
        let response = liveness().await;
        assert_eq!(response.0.status, "ok");
    }

    #[tokio::test]
    async fn test_readiness_reflects_component_health() {
        let state = create_test_state().await;

        // All healthy
        let response = readiness(State(state.clone())).await;
        assert!(response.ready);

        // Simulate unhealthy component
        // ... test implementation
    }

    #[tokio::test]
    async fn test_overall_status_calculation() {
        let service = create_test_health_service().await;

        // All healthy = Healthy
        assert_eq!(service.overall_status().await, HealthStatus::Healthy);

        // One degraded = Degraded
        // ... test implementation

        // One unhealthy = Unhealthy
        // ... test implementation
    }
}
```

---

## Related Specs

- **Spec 311**: Server Setup
- **Spec 333**: Prometheus Metrics
- **Spec 332**: Graceful Shutdown
