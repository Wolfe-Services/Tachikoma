//! Alert management.

use super::{requests::RequestStats, resources::ResourceSnapshot};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Alert severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert definition.
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: serde_json::Value,
}

/// Alert thresholds configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU warning threshold (percentage).
    pub cpu_warning: f64,
    /// CPU critical threshold.
    pub cpu_critical: f64,
    /// Memory warning threshold (percentage).
    pub memory_warning: f64,
    /// Memory critical threshold.
    pub memory_critical: f64,
    /// Error rate warning threshold (percentage).
    pub error_rate_warning: f64,
    /// Error rate critical threshold.
    pub error_rate_critical: f64,
    /// P99 latency warning (ms).
    pub latency_warning_ms: f64,
    /// P99 latency critical (ms).
    pub latency_critical_ms: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            error_rate_warning: 1.0,
            error_rate_critical: 5.0,
            latency_warning_ms: 500.0,
            latency_critical_ms: 2000.0,
        }
    }
}

/// Alert manager for generating and routing alerts.
pub struct AlertManager {
    thresholds: AlertThresholds,
    sender: mpsc::Sender<Alert>,
    receiver: mpsc::Receiver<Alert>,
}

impl AlertManager {
    pub fn new(thresholds: AlertThresholds) -> Self {
        let (sender, receiver) = mpsc::channel(100);
        Self {
            thresholds,
            sender,
            receiver,
        }
    }

    /// Get alert sender for publishing alerts.
    pub fn sender(&self) -> mpsc::Sender<Alert> {
        self.sender.clone()
    }

    /// Check resources and generate alerts.
    pub async fn check_resources(&self, snapshot: &ResourceSnapshot) {
        // CPU alerts
        if snapshot.cpu_percent >= self.thresholds.cpu_critical {
            self.send_alert(Alert {
                id: "cpu_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("CPU usage critical: {:.1}%", snapshot.cpu_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({ "cpu_percent": snapshot.cpu_percent }),
            })
            .await;
        } else if snapshot.cpu_percent >= self.thresholds.cpu_warning {
            self.send_alert(Alert {
                id: "cpu_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("CPU usage high: {:.1}%", snapshot.cpu_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({ "cpu_percent": snapshot.cpu_percent }),
            })
            .await;
        }

        // Memory alerts
        if snapshot.memory_percent >= self.thresholds.memory_critical {
            self.send_alert(Alert {
                id: "memory_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Memory usage critical: {:.1}%", snapshot.memory_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "memory_percent": snapshot.memory_percent,
                    "memory_bytes": snapshot.memory_bytes,
                }),
            })
            .await;
        } else if snapshot.memory_percent >= self.thresholds.memory_warning {
            self.send_alert(Alert {
                id: "memory_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("Memory usage high: {:.1}%", snapshot.memory_percent),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "memory_percent": snapshot.memory_percent,
                    "memory_bytes": snapshot.memory_bytes,
                }),
            })
            .await;
        }
    }

    /// Check request metrics and generate alerts.
    pub async fn check_requests(&self, stats: &RequestStats) {
        // Error rate alerts
        if stats.error_rate >= self.thresholds.error_rate_critical {
            self.send_alert(Alert {
                id: "error_rate_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Error rate critical: {:.2}%", stats.error_rate),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "error_rate": stats.error_rate,
                    "error_count": stats.error_count,
                }),
            })
            .await;
        } else if stats.error_rate >= self.thresholds.error_rate_warning {
            self.send_alert(Alert {
                id: "error_rate_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("Error rate elevated: {:.2}%", stats.error_rate),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "error_rate": stats.error_rate,
                    "error_count": stats.error_count,
                }),
            })
            .await;
        }

        // Latency alerts
        if stats.p99_latency_ms >= self.thresholds.latency_critical_ms {
            self.send_alert(Alert {
                id: "latency_critical".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("P99 latency critical: {:.0}ms", stats.p99_latency_ms),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "p99_latency_ms": stats.p99_latency_ms,
                    "avg_latency_ms": stats.avg_latency_ms,
                }),
            })
            .await;
        } else if stats.p99_latency_ms >= self.thresholds.latency_warning_ms {
            self.send_alert(Alert {
                id: "latency_warning".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("P99 latency elevated: {:.0}ms", stats.p99_latency_ms),
                timestamp: chrono::Utc::now(),
                details: serde_json::json!({
                    "p99_latency_ms": stats.p99_latency_ms,
                    "avg_latency_ms": stats.avg_latency_ms,
                }),
            })
            .await;
        }
    }

    async fn send_alert(&self, alert: Alert) {
        match alert.severity {
            AlertSeverity::Critical => error!(
                alert_id = %alert.id,
                message = %alert.message,
                "CRITICAL ALERT"
            ),
            AlertSeverity::Warning => warn!(
                alert_id = %alert.id,
                message = %alert.message,
                "WARNING ALERT"
            ),
            AlertSeverity::Info => info!(
                alert_id = %alert.id,
                message = %alert.message,
                "INFO ALERT"
            ),
        }

        let _ = self.sender.send(alert).await;
    }
}