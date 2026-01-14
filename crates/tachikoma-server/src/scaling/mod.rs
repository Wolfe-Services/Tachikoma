//! Server scaling implementation.
//!
//! This module provides comprehensive scaling capabilities for the Tachikoma server:
//!
//! - **Worker/thread pool configuration** - Configurable worker threads for handling requests
//! - **Connection limits** - Control concurrent connection limits with backpressure
//! - **Request queue management** - Queue overflow protection with timeouts
//! - **Backpressure handling** - Graceful handling when at capacity
//! - **Load balancing support** - Multi-instance coordination and load distribution
//! - **Instance coordination** - Registry for tracking multiple server instances
//! - **Scale metrics** - Comprehensive metrics for scaling decisions

pub mod config;
pub mod coordinator;
pub mod limiter;
pub mod queue;

use config::ScalingConfig;
use coordinator::{InstanceInfo, InstanceRegistry, ScaleMetrics};
use limiter::{ConnectionGuard, ConnectionLimiter};
use queue::{QueueError, RequestQueue};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Complete server scaling system.
pub struct ScalingSystem {
    /// Configuration.
    config: ScalingConfig,
    /// Connection limiter.
    limiter: Arc<ConnectionLimiter>,
    /// Instance registry.
    registry: Arc<InstanceRegistry>,
    /// Current instance info.
    self_info: InstanceInfo,
    /// Metrics sender.
    metrics_tx: watch::Sender<ScalingMetrics>,
    /// Metrics receiver.
    metrics_rx: watch::Receiver<ScalingMetrics>,
}

impl ScalingSystem {
    /// Create a new scaling system.
    pub fn new(config: ScalingConfig, host: String, port: u16) -> Self {
        let instance_id = Uuid::new_v4();
        let limiter = Arc::new(ConnectionLimiter::new(config.max_connections));
        let registry = Arc::new(InstanceRegistry::new(instance_id));

        let self_info = InstanceInfo {
            id: instance_id,
            host,
            port,
            started_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            connections: 0,
            cpu_percent: 0.0,
            memory_percent: 0.0,
        };

        let initial_metrics = ScalingMetrics {
            instance_id,
            connections: 0,
            available_slots: config.max_connections,
            cpu_percent: 0.0,
            memory_percent: 0.0,
            queue_depth: 0,
            total_instances: 1,
            healthy_instances: 1,
        };

        let (metrics_tx, metrics_rx) = watch::channel(initial_metrics);

        Self {
            config,
            limiter,
            registry,
            self_info,
            metrics_tx,
            metrics_rx,
        }
    }

    /// Start the scaling system.
    pub async fn start(&self) {
        info!("Starting scaling system");

        // Register this instance
        self.registry.register(self.self_info.clone()).await;

        // Start heartbeat loop
        self.start_heartbeat_loop();

        // Start cleanup loop
        self.start_cleanup_loop();

        // Start metrics update loop
        self.start_metrics_loop();

        info!("Scaling system started");
    }

    /// Try to acquire a connection slot.
    pub fn try_acquire_connection(&self) -> Option<ConnectionGuard> {
        self.limiter.try_acquire()
    }

    /// Acquire a connection slot (with backpressure).
    pub async fn acquire_connection(&self) -> ConnectionGuard {
        self.limiter.acquire().await
    }

    /// Get current connection stats.
    pub fn connection_stats(&self) -> ConnectionStats {
        ConnectionStats {
            current: self.limiter.current(),
            max: self.limiter.max(),
            available: self.limiter.available(),
        }
    }

    /// Get scaling configuration.
    pub fn config(&self) -> &ScalingConfig {
        &self.config
    }

    /// Get instance registry.
    pub fn registry(&self) -> Arc<InstanceRegistry> {
        self.registry.clone()
    }

    /// Subscribe to scaling metrics.
    pub fn subscribe_metrics(&self) -> watch::Receiver<ScalingMetrics> {
        self.metrics_rx.clone()
    }

    /// Get current scaling metrics.
    pub fn current_metrics(&self) -> ScalingMetrics {
        self.metrics_rx.borrow().clone()
    }

    fn start_heartbeat_loop(&self) {
        let registry = self.registry.clone();
        let mut self_info = self.self_info.clone();
        let limiter = self.limiter.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Update connection count
                self_info.connections = limiter.current() as u64;

                // TODO: Get actual CPU/memory from monitoring system
                // For now, use dummy values
                self_info.cpu_percent = 0.0;
                self_info.memory_percent = 0.0;

                // Register/update this instance
                registry.register(self_info.clone()).await;

                debug!("Sent heartbeat with {} connections", self_info.connections);
            }
        });
    }

    fn start_cleanup_loop(&self) {
        let registry = self.registry.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;
                registry.cleanup().await;
                debug!("Cleaned up stale instances");
            }
        });
    }

    fn start_metrics_loop(&self) {
        let registry = self.registry.clone();
        let limiter = self.limiter.clone();
        let metrics_tx = self.metrics_tx.clone();
        let instance_id = self.self_info.id;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                let scale_metrics = registry.metrics().await;
                let connection_stats = ConnectionStats {
                    current: limiter.current(),
                    max: limiter.max(),
                    available: limiter.available(),
                };

                let metrics = ScalingMetrics {
                    instance_id,
                    connections: connection_stats.current,
                    available_slots: connection_stats.available,
                    cpu_percent: 0.0,      // TODO: Get from monitoring
                    memory_percent: 0.0,   // TODO: Get from monitoring
                    queue_depth: 0,        // TODO: Track queue depth
                    total_instances: scale_metrics.total_instances,
                    healthy_instances: scale_metrics.healthy_instances,
                };

                if let Err(_) = metrics_tx.send(metrics) {
                    warn!("Failed to send scaling metrics");
                    break;
                }
            }
        });
    }
}

/// Connection statistics.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStats {
    pub current: usize,
    pub max: usize,
    pub available: usize,
}

/// Scaling metrics.
#[derive(Debug, Clone, Serialize)]
pub struct ScalingMetrics {
    pub instance_id: Uuid,
    pub connections: usize,
    pub available_slots: usize,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub queue_depth: usize,
    pub total_instances: usize,
    pub healthy_instances: usize,
}

// Re-export main types
pub use config::ScalingConfig;
pub use coordinator::{InstanceInfo, InstanceRegistry, ScaleMetrics};
pub use limiter::{ConnectionGuard, ConnectionLimiter};
pub use queue::{QueueError, RequestQueue};