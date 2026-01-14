//! Resource monitoring.

use serde::Serialize;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, warn};

/// Resource usage snapshot.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSnapshot {
    /// CPU usage percentage (0-100).
    pub cpu_percent: f64,
    /// Memory usage in bytes.
    pub memory_bytes: u64,
    /// Memory usage percentage.
    pub memory_percent: f64,
    /// Open file descriptors.
    pub open_fds: u64,
    /// Thread count.
    pub thread_count: u64,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ResourceSnapshot {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
            open_fds: 0,
            thread_count: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Resource monitor collecting system metrics.
pub struct ResourceMonitor {
    interval: Duration,
    sender: watch::Sender<ResourceSnapshot>,
    receiver: watch::Receiver<ResourceSnapshot>,
}

impl ResourceMonitor {
    pub fn new(interval: Duration) -> Self {
        let (sender, receiver) = watch::channel(ResourceSnapshot::default());
        Self {
            interval,
            sender,
            receiver,
        }
    }

    /// Start the monitoring loop.
    pub fn start(&self) {
        let interval = self.interval;
        let sender = self.sender.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                let snapshot = collect_resources();
                debug!(
                    cpu = snapshot.cpu_percent,
                    memory_mb = snapshot.memory_bytes / 1_000_000,
                    "Resource snapshot"
                );

                let _ = sender.send(snapshot);
            }
        });
    }

    /// Get the latest snapshot.
    pub fn get(&self) -> ResourceSnapshot {
        self.receiver.borrow().clone()
    }

    /// Subscribe to snapshots.
    pub fn subscribe(&self) -> watch::Receiver<ResourceSnapshot> {
        self.receiver.clone()
    }
}

#[cfg(target_os = "linux")]
fn collect_resources() -> ResourceSnapshot {
    use std::fs;

    let mut snapshot = ResourceSnapshot::default();
    snapshot.timestamp = chrono::Utc::now();

    // Read /proc/self/stat for CPU and memory
    if let Ok(stat) = fs::read_to_string("/proc/self/stat") {
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() > 23 {
            // Thread count is at index 19
            if let Ok(threads) = parts[19].parse::<u64>() {
                snapshot.thread_count = threads;
            }
        }
    }

    // Read /proc/self/status for memory
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb.parse::<u64>() {
                        snapshot.memory_bytes = kb * 1024;
                    }
                }
            }
        }
    }

    // Read /proc/self/fd for open file descriptors
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        snapshot.open_fds = entries.count() as u64;
    }

    // Get total memory for percentage calculation
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(total_kb) = kb.parse::<u64>() {
                        let total_bytes = total_kb * 1024;
                        if total_bytes > 0 {
                            snapshot.memory_percent =
                                (snapshot.memory_bytes as f64 / total_bytes as f64) * 100.0;
                        }
                    }
                }
            }
        }
    }

    snapshot
}

#[cfg(not(target_os = "linux"))]
fn collect_resources() -> ResourceSnapshot {
    // Fallback for non-Linux systems
    ResourceSnapshot::default()
}