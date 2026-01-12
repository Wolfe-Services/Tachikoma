# 327 - Health Endpoints

**Phase:** 15 - Server
**Spec ID:** 327
**Status:** Planned
**Dependencies:** 317-axum-router
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement health check endpoints for liveness, readiness, and startup probes compatible with Kubernetes and load balancers.

---

## Acceptance Criteria

- [ ] Liveness probe endpoint (/health/live)
- [ ] Readiness probe endpoint (/health/ready)
- [ ] Startup probe endpoint (/health/startup)
- [ ] Detailed health endpoint (/health)
- [ ] Component health checks
- [ ] Configurable health thresholds
- [ ] Health check caching

---

## Implementation Details

### 1. Health Types (crates/tachikoma-server/src/health/types.rs)

```rust
//! Health check types.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

/// Overall health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational.
    Healthy,
    /// Some systems degraded but functional.
    Degraded,
    /// System is unhealthy.
    Unhealthy,
}

impl HealthStatus {
    /// Combine two statuses (worst wins).
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unhealthy, _) | (_, Self::Unhealthy) => Self::Unhealthy,
            (Self::Degraded, _) | (_, Self::Degraded) => Self::Degraded,
            _ => Self::Healthy,
        }
    }
}

/// Individual component health.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    pub checked_at: DateTime<Utc>,
}

impl ComponentHealth {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
            checked_at: Utc::now(),
        }
    }

    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            latency_ms: None,
            checked_at: Utc::now(),
        }
    }

    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            latency_ms: None,
            checked_at: Utc::now(),
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Detailed health response.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentHealth>,
}

/// Simple health response (for probes).
#[derive(Debug, Clone, Serialize)]
pub struct SimpleHealthResponse {
    pub status: HealthStatus,
}

/// Startup status.
#[derive(Debug, Clone, Serialize)]
pub struct StartupResponse {
    pub started: bool,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
```

### 2. Health Checks (crates/tachikoma-server/src/health/checks.rs)

```rust
//! Health check implementations.

use super::types::{ComponentHealth, HealthStatus};
use async_trait::async_trait;
use sqlx::PgPool;
use std::time::{Duration, Instant};

/// Trait for health checks.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the component name.
    fn name(&self) -> &str;

    /// Perform the health check.
    async fn check(&self) -> ComponentHealth;
}

/// Database health check.
pub struct DatabaseCheck {
    pool: PgPool,
    timeout: Duration,
}

impl DatabaseCheck {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl HealthCheck for DatabaseCheck {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        let result = tokio::time::timeout(
            self.timeout,
            sqlx::query("SELECT 1").fetch_one(&self.pool),
        )
        .await;

        let latency = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(_)) => {
                let status = if latency > 1000 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                let mut health = ComponentHealth {
                    name: self.name().to_string(),
                    status,
                    message: if status == HealthStatus::Degraded {
                        Some(format!("High latency: {}ms", latency))
                    } else {
                        None
                    },
                    latency_ms: Some(latency),
                    checked_at: chrono::Utc::now(),
                };
                health
            }
            Ok(Err(e)) => ComponentHealth::unhealthy(self.name(), e.to_string())
                .with_latency(latency),
            Err(_) => ComponentHealth::unhealthy(self.name(), "Connection timeout")
                .with_latency(latency),
        }
    }
}

/// Redis health check.
#[cfg(feature = "redis")]
pub struct RedisCheck {
    client: redis::Client,
    timeout: Duration,
}

#[cfg(feature = "redis")]
impl RedisCheck {
    pub fn new(client: redis::Client) -> Self {
        Self {
            client,
            timeout: Duration::from_secs(2),
        }
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl HealthCheck for RedisCheck {
    fn name(&self) -> &str {
        "redis"
    }

    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();

        match self.client.get_async_connection().await {
            Ok(mut conn) => {
                let result: Result<String, _> = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await;

                let latency = start.elapsed().as_millis() as u64;

                match result {
                    Ok(_) => ComponentHealth::healthy(self.name()).with_latency(latency),
                    Err(e) => ComponentHealth::unhealthy(self.name(), e.to_string())
                        .with_latency(latency),
                }
            }
            Err(e) => ComponentHealth::unhealthy(self.name(), e.to_string())
                .with_latency(start.elapsed().as_millis() as u64),
        }
    }
}

/// Memory health check.
pub struct MemoryCheck {
    threshold_mb: u64,
}

impl MemoryCheck {
    pub fn new(threshold_mb: u64) -> Self {
        Self { threshold_mb }
    }
}

#[async_trait]
impl HealthCheck for MemoryCheck {
    fn name(&self) -> &str {
        "memory"
    }

    async fn check(&self) -> ComponentHealth {
        // Simple memory check using process info
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb.parse::<u64>() {
                                let mb = kb / 1024;
                                if mb > self.threshold_mb {
                                    return ComponentHealth::degraded(
                                        self.name(),
                                        format!("High memory usage: {}MB", mb),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        ComponentHealth::healthy(self.name())
    }
}

/// Disk space health check.
pub struct DiskCheck {
    path: String,
    threshold_percent: u8,
}

impl DiskCheck {
    pub fn new(path: impl Into<String>, threshold_percent: u8) -> Self {
        Self {
            path: path.into(),
            threshold_percent,
        }
    }
}

#[async_trait]
impl HealthCheck for DiskCheck {
    fn name(&self) -> &str {
        "disk"
    }

    async fn check(&self) -> ComponentHealth {
        // Platform-specific disk check would go here
        ComponentHealth::healthy(self.name())
    }
}
```

### 3. Health Handlers (crates/tachikoma-server/src/health/handlers.rs)

```rust
//! Health endpoint handlers.

use super::{
    checks::HealthCheck,
    types::{HealthResponse, HealthStatus, SimpleHealthResponse, StartupResponse},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use tokio::sync::RwLock;

/// Health service state.
pub struct HealthState {
    /// Application version.
    version: String,
    /// Startup time.
    startup_time: Instant,
    /// Whether startup is complete.
    started: AtomicBool,
    /// Health checks to run.
    checks: RwLock<Vec<Box<dyn HealthCheck>>>,
    /// Cached health results.
    cache: RwLock<Option<CachedHealth>>,
    /// Cache TTL in seconds.
    cache_ttl_secs: u64,
}

struct CachedHealth {
    response: HealthResponse,
    cached_at: Instant,
}

impl HealthState {
    pub fn new(version: String) -> Self {
        Self {
            version,
            startup_time: Instant::now(),
            started: AtomicBool::new(false),
            checks: RwLock::new(Vec::new()),
            cache: RwLock::new(None),
            cache_ttl_secs: 5,
        }
    }

    /// Mark startup as complete.
    pub fn mark_started(&self) {
        self.started.store(true, Ordering::SeqCst);
    }

    /// Check if started.
    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::SeqCst)
    }

    /// Add a health check.
    pub async fn add_check(&self, check: Box<dyn HealthCheck>) {
        self.checks.write().await.push(check);
    }

    /// Get uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.startup_time.elapsed().as_secs()
    }

    /// Run all health checks.
    pub async fn check_health(&self) -> HealthResponse {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.cached_at.elapsed().as_secs() < self.cache_ttl_secs {
                    return cached.response.clone();
                }
            }
        }

        // Run checks
        let checks = self.checks.read().await;
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in checks.iter() {
            let result = check.check().await;
            overall_status = overall_status.combine(result.status);
            components.insert(check.name().to_string(), result);
        }

        let response = HealthResponse {
            status: overall_status,
            version: self.version.clone(),
            uptime_seconds: self.uptime_seconds(),
            timestamp: chrono::Utc::now(),
            components,
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedHealth {
                response: response.clone(),
                cached_at: Instant::now(),
            });
        }

        response
    }
}

/// Liveness probe handler.
/// Returns 200 if the application is running.
pub async fn liveness() -> impl IntoResponse {
    Json(SimpleHealthResponse {
        status: HealthStatus::Healthy,
    })
}

/// Readiness probe handler.
/// Returns 200 if the application is ready to serve traffic.
pub async fn readiness(State(state): State<Arc<HealthState>>) -> Response {
    let health = state.check_health().await;

    let status_code = match health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still ready, just degraded
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (
        status_code,
        Json(SimpleHealthResponse {
            status: health.status,
        }),
    )
        .into_response()
}

/// Startup probe handler.
/// Returns 200 once startup is complete.
pub async fn startup(State(state): State<Arc<HealthState>>) -> Response {
    if state.is_started() {
        (
            StatusCode::OK,
            Json(StartupResponse {
                started: true,
                status: HealthStatus::Healthy,
                message: None,
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(StartupResponse {
                started: false,
                status: HealthStatus::Unhealthy,
                message: Some("Application is still starting".to_string()),
            }),
        )
            .into_response()
    }
}

/// Detailed health handler.
pub async fn health(State(state): State<Arc<HealthState>>) -> Response {
    let health = state.check_health().await;

    let status_code = match health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health)).into_response()
}
```

### 4. Health Router (crates/tachikoma-server/src/health/router.rs)

```rust
//! Health routes configuration.

use super::handlers::{health, liveness, readiness, startup, HealthState};
use axum::{routing::get, Router};
use std::sync::Arc;

/// Create health routes.
pub fn health_routes(state: Arc<HealthState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/health/startup", get(startup))
        .with_state(state)
}
```

---

## Testing Requirements

1. Liveness always returns 200
2. Readiness reflects actual health
3. Startup probe works correctly
4. Component checks run properly
5. Health caching works
6. Degraded status handled correctly
7. Latency thresholds work

---

## Related Specs

- Depends on: [317-axum-router.md](317-axum-router.md)
- Next: [328-metrics-endpoints.md](328-metrics-endpoints.md)
- Used by: Kubernetes, load balancers
