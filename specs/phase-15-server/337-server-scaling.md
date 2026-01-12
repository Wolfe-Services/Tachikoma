# 337 - Server Scaling

**Phase:** 15 - Server
**Spec ID:** 337
**Status:** Planned
**Dependencies:** 336-server-monitoring
**Estimated Context:** ~5% of Sonnet window

---

## Objective

Implement server scaling capabilities with connection limits, worker configuration, and horizontal scaling support.

---

## Acceptance Criteria

- [ ] Worker/thread pool configuration
- [ ] Connection limits
- [ ] Request queue management
- [ ] Backpressure handling
- [ ] Load balancing support
- [ ] Instance coordination
- [ ] Scale metrics

---

## Implementation Details

### 1. Scaling Config (crates/tachikoma-server/src/scaling/config.rs)

```rust
//! Scaling configuration.

use serde::{Deserialize, Serialize};

/// Server scaling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Number of worker threads.
    #[serde(default = "default_workers")]
    pub workers: usize,
    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
    /// Maximum pending connections (backlog).
    #[serde(default = "default_backlog")]
    pub backlog: u32,
    /// Request queue size.
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
    /// Enable connection keep-alive.
    #[serde(default = "default_true")]
    pub keep_alive: bool,
    /// Keep-alive timeout (seconds).
    #[serde(default = "default_keepalive_timeout")]
    pub keepalive_timeout_secs: u64,
    /// Request timeout (seconds).
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    /// Enable HTTP/2.
    #[serde(default = "default_true")]
    pub http2: bool,
}

fn default_workers() -> usize {
    num_cpus::get()
}

fn default_max_connections() -> usize {
    10000
}

fn default_backlog() -> u32 {
    1024
}

fn default_queue_size() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

fn default_keepalive_timeout() -> u64 {
    75
}

fn default_request_timeout() -> u64 {
    30
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            workers: default_workers(),
            max_connections: default_max_connections(),
            backlog: default_backlog(),
            queue_size: default_queue_size(),
            keep_alive: true,
            keepalive_timeout_secs: default_keepalive_timeout(),
            request_timeout_secs: default_request_timeout(),
            http2: true,
        }
    }
}

impl ScalingConfig {
    /// Configuration for development.
    pub fn development() -> Self {
        Self {
            workers: 2,
            max_connections: 1000,
            backlog: 128,
            queue_size: 100,
            ..Default::default()
        }
    }

    /// Configuration for production.
    pub fn production() -> Self {
        Self {
            workers: num_cpus::get() * 2,
            max_connections: 50000,
            backlog: 2048,
            queue_size: 10000,
            ..Default::default()
        }
    }
}
```

### 2. Connection Limiter (crates/tachikoma-server/src/scaling/limiter.rs)

```rust
//! Connection limiting.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, warn};

/// Connection limiter for controlling concurrent connections.
pub struct ConnectionLimiter {
    /// Semaphore for limiting connections.
    semaphore: Arc<Semaphore>,
    /// Current connection count.
    current: AtomicUsize,
    /// Maximum connections.
    max: usize,
}

impl ConnectionLimiter {
    pub fn new(max_connections: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_connections)),
            current: AtomicUsize::new(0),
            max: max_connections,
        }
    }

    /// Try to acquire a connection slot.
    pub fn try_acquire(&self) -> Option<ConnectionGuard> {
        match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                let count = self.current.fetch_add(1, Ordering::SeqCst) + 1;
                debug!(connections = count, "Connection acquired");
                Some(ConnectionGuard {
                    _permit: permit,
                    current: &self.current,
                })
            }
            Err(_) => {
                warn!(
                    max = self.max,
                    "Connection limit reached, rejecting connection"
                );
                None
            }
        }
    }

    /// Acquire a connection slot (blocking).
    pub async fn acquire(&self) -> ConnectionGuard {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        let count = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        debug!(connections = count, "Connection acquired");
        ConnectionGuard {
            _permit: permit,
            current: &self.current,
        }
    }

    /// Get current connection count.
    pub fn current(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }

    /// Get available slots.
    pub fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get maximum connections.
    pub fn max(&self) -> usize {
        self.max
    }
}

/// Guard that releases connection slot when dropped.
pub struct ConnectionGuard<'a> {
    _permit: tokio::sync::OwnedSemaphorePermit,
    current: &'a AtomicUsize,
}

impl Drop for ConnectionGuard<'_> {
    fn drop(&mut self) {
        let count = self.current.fetch_sub(1, Ordering::SeqCst) - 1;
        debug!(connections = count, "Connection released");
    }
}
```

### 3. Request Queue (crates/tachikoma-server/src/scaling/queue.rs)

```rust
//! Request queue management.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Request queue for managing pending requests.
pub struct RequestQueue<T> {
    sender: mpsc::Sender<QueuedRequest<T>>,
    receiver: mpsc::Receiver<QueuedRequest<T>>,
    pending: AtomicUsize,
    max_size: usize,
    timeout: Duration,
}

struct QueuedRequest<T> {
    request: T,
    queued_at: Instant,
}

impl<T: Send + 'static> RequestQueue<T> {
    pub fn new(max_size: usize, timeout: Duration) -> Self {
        let (sender, receiver) = mpsc::channel(max_size);
        Self {
            sender,
            receiver,
            pending: AtomicUsize::new(0),
            max_size,
            timeout,
        }
    }

    /// Enqueue a request.
    pub async fn enqueue(&self, request: T) -> Result<(), QueueError> {
        let pending = self.pending.load(Ordering::SeqCst);
        if pending >= self.max_size {
            warn!(pending = pending, max = self.max_size, "Request queue full");
            return Err(QueueError::Full);
        }

        let queued = QueuedRequest {
            request,
            queued_at: Instant::now(),
        };

        match self.sender.send(queued).await {
            Ok(()) => {
                self.pending.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            Err(_) => Err(QueueError::Closed),
        }
    }

    /// Dequeue a request.
    pub async fn dequeue(&mut self) -> Option<T> {
        loop {
            match self.receiver.recv().await {
                Some(queued) => {
                    self.pending.fetch_sub(1, Ordering::SeqCst);

                    // Check if request has timed out
                    if queued.queued_at.elapsed() > self.timeout {
                        debug!("Dropping timed out request from queue");
                        continue;
                    }

                    return Some(queued.request);
                }
                None => return None,
            }
        }
    }

    /// Get pending count.
    pub fn pending(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    /// Check if queue is full.
    pub fn is_full(&self) -> bool {
        self.pending.load(Ordering::SeqCst) >= self.max_size
    }
}

/// Queue errors.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Queue is full")]
    Full,
    #[error("Queue is closed")]
    Closed,
}
```

### 4. Instance Coordinator (crates/tachikoma-server/src/scaling/coordinator.rs)

```rust
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
```

---

## Testing Requirements

1. Worker count configurable
2. Connection limits enforced
3. Queue backpressure works
4. Request timeouts honored
5. Instance registration works
6. Stale instance cleanup
7. Load balancing metrics accurate

---

## Related Specs

- Depends on: [336-server-monitoring.md](336-server-monitoring.md)
- Next: [338-server-caching.md](338-server-caching.md)
- Used by: Deployment, operations
