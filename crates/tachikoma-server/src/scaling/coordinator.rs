//! Multi-instance coordination.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Instance information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: Uuid,
    pub host: String,
    pub port: u16,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub connections: u64,
    pub cpu_percent: f64,
    pub memory_percent: f64,
}

/// Instance registry for tracking server instances.
pub struct InstanceRegistry {
    instances: RwLock<HashMap<Uuid, RegisteredInstance>>,
    self_id: Uuid,
    heartbeat_interval: Duration,
    instance_timeout: Duration,
}

struct RegisteredInstance {
    info: InstanceInfo,
    last_seen: Instant,
}

impl InstanceRegistry {
    pub fn new(self_id: Uuid) -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
            self_id,
            heartbeat_interval: Duration::from_secs(5),
            instance_timeout: Duration::from_secs(30),
        }
    }

    /// Register or update an instance.
    pub async fn register(&self, info: InstanceInfo) {
        let mut instances = self.instances.write().await;
        instances.insert(
            info.id,
            RegisteredInstance {
                info,
                last_seen: Instant::now(),
            },
        );
    }

    /// Get all healthy instances.
    pub async fn healthy_instances(&self) -> Vec<InstanceInfo> {
        let instances = self.instances.read().await;
        let cutoff = Instant::now() - self.instance_timeout;

        instances
            .values()
            .filter(|i| i.last_seen > cutoff)
            .map(|i| i.info.clone())
            .collect()
    }

    /// Remove stale instances.
    pub async fn cleanup(&self) {
        let mut instances = self.instances.write().await;
        let cutoff = Instant::now() - self.instance_timeout;

        instances.retain(|_, i| i.last_seen > cutoff);
    }

    /// Get instance count.
    pub async fn count(&self) -> usize {
        self.instances.read().await.len()
    }

    /// Get least loaded instance (for load balancing).
    pub async fn least_loaded(&self) -> Option<InstanceInfo> {
        let instances = self.instances.read().await;
        let cutoff = Instant::now() - self.instance_timeout;

        instances
            .values()
            .filter(|i| i.last_seen > cutoff)
            .min_by(|a, b| {
                a.info
                    .connections
                    .partial_cmp(&b.info.connections)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|i| i.info.clone())
    }
}

/// Scale metrics.
#[derive(Debug, Clone, Serialize)]
pub struct ScaleMetrics {
    pub total_instances: usize,
    pub healthy_instances: usize,
    pub total_connections: u64,
    pub average_cpu: f64,
    pub average_memory: f64,
}

impl InstanceRegistry {
    /// Get scaling metrics.
    pub async fn metrics(&self) -> ScaleMetrics {
        let instances = self.instances.read().await;
        let cutoff = Instant::now() - self.instance_timeout;

        let healthy: Vec<_> = instances
            .values()
            .filter(|i| i.last_seen > cutoff)
            .collect();

        let total_connections: u64 = healthy.iter().map(|i| i.info.connections).sum();
        let avg_cpu = if healthy.is_empty() {
            0.0
        } else {
            healthy.iter().map(|i| i.info.cpu_percent).sum::<f64>() / healthy.len() as f64
        };
        let avg_memory = if healthy.is_empty() {
            0.0
        } else {
            healthy.iter().map(|i| i.info.memory_percent).sum::<f64>() / healthy.len() as f64
        };

        ScaleMetrics {
            total_instances: instances.len(),
            healthy_instances: healthy.len(),
            total_connections,
            average_cpu: avg_cpu,
            average_memory: avg_memory,
        }
    }
}