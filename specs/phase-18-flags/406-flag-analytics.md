# 406 - Feature Flag Analytics

## Overview

Analytics and metrics collection for feature flag evaluations, enabling monitoring, debugging, and experiment analysis.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/analytics.rs

use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Flag evaluation event for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationEvent {
    /// Unique event ID
    pub event_id: String,
    /// Flag that was evaluated
    pub flag_id: String,
    /// User identifier
    pub user_id: Option<String>,
    /// Anonymous identifier
    pub anonymous_id: Option<String>,
    /// Evaluation result value
    pub value: serde_json::Value,
    /// Reason for the result
    pub reason: String,
    /// Matched rule ID (if any)
    pub matched_rule: Option<String>,
    /// Whether user is in experiment
    pub in_experiment: bool,
    /// Experiment variant (if applicable)
    pub variant: Option<String>,
    /// Evaluation duration in microseconds
    pub duration_us: u64,
    /// Environment
    pub environment: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional context properties
    pub context: HashMap<String, serde_json::Value>,
}

/// Aggregated flag statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlagAnalytics {
    /// Total evaluations
    pub total_evaluations: u64,
    /// Evaluations by result value
    pub evaluations_by_value: HashMap<String, u64>,
    /// Evaluations by reason
    pub evaluations_by_reason: HashMap<String, u64>,
    /// Unique users
    pub unique_users: u64,
    /// Average evaluation time (microseconds)
    pub avg_duration_us: f64,
    /// P50 evaluation time
    pub p50_duration_us: u64,
    /// P95 evaluation time
    pub p95_duration_us: u64,
    /// P99 evaluation time
    pub p99_duration_us: u64,
    /// Error count
    pub error_count: u64,
    /// Time series data (hourly)
    pub hourly_counts: Vec<HourlyCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyCount {
    pub hour: DateTime<Utc>,
    pub count: u64,
    pub true_count: u64,
    pub false_count: u64,
}

/// Analytics collector interface
#[async_trait::async_trait]
pub trait AnalyticsCollector: Send + Sync {
    /// Record an evaluation event
    async fn record_evaluation(&self, event: EvaluationEvent);

    /// Record a batch of evaluation events
    async fn record_batch(&self, events: Vec<EvaluationEvent>);

    /// Get analytics for a specific flag
    async fn get_flag_analytics(&self, flag_id: &str, period: Duration) -> FlagAnalytics;

    /// Get overall system analytics
    async fn get_system_analytics(&self, period: Duration) -> SystemAnalytics;

    /// Flush pending events
    async fn flush(&self);
}

/// Overall system analytics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemAnalytics {
    pub total_evaluations: u64,
    pub total_flags: u64,
    pub active_flags: u64,
    pub avg_evaluation_time_us: f64,
    pub evaluations_per_second: f64,
    pub top_flags: Vec<TopFlag>,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopFlag {
    pub flag_id: String,
    pub evaluation_count: u64,
    pub unique_users: u64,
}

/// In-memory analytics collector (for development)
pub struct InMemoryAnalyticsCollector {
    events: RwLock<Vec<EvaluationEvent>>,
    max_events: usize,
}

impl InMemoryAnalyticsCollector {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: RwLock::new(Vec::with_capacity(max_events)),
            max_events,
        }
    }
}

#[async_trait::async_trait]
impl AnalyticsCollector for InMemoryAnalyticsCollector {
    async fn record_evaluation(&self, event: EvaluationEvent) {
        let mut events = self.events.write().await;

        if events.len() >= self.max_events {
            events.remove(0);
        }

        events.push(event);
    }

    async fn record_batch(&self, batch: Vec<EvaluationEvent>) {
        let mut events = self.events.write().await;

        for event in batch {
            if events.len() >= self.max_events {
                events.remove(0);
            }
            events.push(event);
        }
    }

    async fn get_flag_analytics(&self, flag_id: &str, period: Duration) -> FlagAnalytics {
        let events = self.events.read().await;
        let cutoff = Utc::now() - period;

        let flag_events: Vec<_> = events.iter()
            .filter(|e| e.flag_id == flag_id && e.timestamp > cutoff)
            .collect();

        if flag_events.is_empty() {
            return FlagAnalytics::default();
        }

        let total_evaluations = flag_events.len() as u64;

        // Count by value
        let mut evaluations_by_value = HashMap::new();
        for event in &flag_events {
            let value_str = event.value.to_string();
            *evaluations_by_value.entry(value_str).or_insert(0u64) += 1;
        }

        // Count by reason
        let mut evaluations_by_reason = HashMap::new();
        for event in &flag_events {
            *evaluations_by_reason.entry(event.reason.clone()).or_insert(0u64) += 1;
        }

        // Unique users
        let mut unique_user_ids = std::collections::HashSet::new();
        for event in &flag_events {
            if let Some(ref user_id) = event.user_id {
                unique_user_ids.insert(user_id.clone());
            }
        }

        // Duration percentiles
        let mut durations: Vec<u64> = flag_events.iter().map(|e| e.duration_us).collect();
        durations.sort();

        let avg_duration_us = if !durations.is_empty() {
            durations.iter().sum::<u64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        let p50 = durations.get(durations.len() / 2).copied().unwrap_or(0);
        let p95 = durations.get((durations.len() as f64 * 0.95) as usize).copied().unwrap_or(0);
        let p99 = durations.get((durations.len() as f64 * 0.99) as usize).copied().unwrap_or(0);

        FlagAnalytics {
            total_evaluations,
            evaluations_by_value,
            evaluations_by_reason,
            unique_users: unique_user_ids.len() as u64,
            avg_duration_us,
            p50_duration_us: p50,
            p95_duration_us: p95,
            p99_duration_us: p99,
            error_count: evaluations_by_reason.get("error").copied().unwrap_or(0),
            hourly_counts: vec![],
        }
    }

    async fn get_system_analytics(&self, period: Duration) -> SystemAnalytics {
        let events = self.events.read().await;
        let cutoff = Utc::now() - period;

        let recent_events: Vec<_> = events.iter()
            .filter(|e| e.timestamp > cutoff)
            .collect();

        let total_evaluations = recent_events.len() as u64;

        // Count per flag
        let mut flag_counts: HashMap<String, u64> = HashMap::new();
        let mut flag_users: HashMap<String, std::collections::HashSet<String>> = HashMap::new();

        for event in &recent_events {
            *flag_counts.entry(event.flag_id.clone()).or_insert(0) += 1;

            if let Some(ref user_id) = event.user_id {
                flag_users.entry(event.flag_id.clone())
                    .or_insert_with(std::collections::HashSet::new)
                    .insert(user_id.clone());
            }
        }

        let top_flags: Vec<_> = flag_counts.iter()
            .map(|(flag_id, count)| TopFlag {
                flag_id: flag_id.clone(),
                evaluation_count: *count,
                unique_users: flag_users.get(flag_id).map(|s| s.len()).unwrap_or(0) as u64,
            })
            .collect();

        let avg_duration = if !recent_events.is_empty() {
            recent_events.iter().map(|e| e.duration_us).sum::<u64>() as f64 / recent_events.len() as f64
        } else {
            0.0
        };

        let evaluations_per_second = if period.num_seconds() > 0 {
            total_evaluations as f64 / period.num_seconds() as f64
        } else {
            0.0
        };

        let error_count = recent_events.iter().filter(|e| e.reason == "error").count() as u64;
        let error_rate = if total_evaluations > 0 {
            error_count as f64 / total_evaluations as f64
        } else {
            0.0
        };

        SystemAnalytics {
            total_evaluations,
            total_flags: flag_counts.len() as u64,
            active_flags: flag_counts.len() as u64,
            avg_evaluation_time_us: avg_duration,
            evaluations_per_second,
            top_flags,
            error_rate,
        }
    }

    async fn flush(&self) {
        // No-op for in-memory
    }
}

/// Buffered analytics collector for production
pub struct BufferedAnalyticsCollector {
    buffer: RwLock<Vec<EvaluationEvent>>,
    buffer_size: usize,
    flush_interval: std::time::Duration,
    backend: Arc<dyn AnalyticsBackend>,
}

#[async_trait::async_trait]
pub trait AnalyticsBackend: Send + Sync {
    async fn write(&self, events: Vec<EvaluationEvent>) -> Result<(), Box<dyn std::error::Error>>;
    async fn query(&self, query: AnalyticsQuery) -> Result<Vec<EvaluationEvent>, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct AnalyticsQuery {
    pub flag_id: Option<String>,
    pub user_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub limit: usize,
}

impl BufferedAnalyticsCollector {
    pub fn new(
        backend: Arc<dyn AnalyticsBackend>,
        buffer_size: usize,
        flush_interval: std::time::Duration,
    ) -> Arc<Self> {
        let collector = Arc::new(Self {
            buffer: RwLock::new(Vec::with_capacity(buffer_size)),
            buffer_size,
            flush_interval,
            backend,
        });

        // Start flush task
        let collector_clone = collector.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collector_clone.flush_interval);

            loop {
                interval.tick().await;
                collector_clone.flush().await;
            }
        });

        collector
    }

    async fn flush_if_needed(&self) {
        let should_flush = {
            let buffer = self.buffer.read().await;
            buffer.len() >= self.buffer_size
        };

        if should_flush {
            self.flush().await;
        }
    }
}

#[async_trait::async_trait]
impl AnalyticsCollector for BufferedAnalyticsCollector {
    async fn record_evaluation(&self, event: EvaluationEvent) {
        {
            let mut buffer = self.buffer.write().await;
            buffer.push(event);
        }

        self.flush_if_needed().await;
    }

    async fn record_batch(&self, events: Vec<EvaluationEvent>) {
        {
            let mut buffer = self.buffer.write().await;
            buffer.extend(events);
        }

        self.flush_if_needed().await;
    }

    async fn get_flag_analytics(&self, flag_id: &str, period: Duration) -> FlagAnalytics {
        let query = AnalyticsQuery {
            flag_id: Some(flag_id.to_string()),
            user_id: None,
            start_time: Utc::now() - period,
            end_time: Utc::now(),
            limit: 100000,
        };

        match self.backend.query(query).await {
            Ok(events) => {
                // Calculate analytics from events
                let total_evaluations = events.len() as u64;

                let mut evaluations_by_value = HashMap::new();
                for event in &events {
                    let value_str = event.value.to_string();
                    *evaluations_by_value.entry(value_str).or_insert(0u64) += 1;
                }

                FlagAnalytics {
                    total_evaluations,
                    evaluations_by_value,
                    ..Default::default()
                }
            }
            Err(_) => FlagAnalytics::default(),
        }
    }

    async fn get_system_analytics(&self, _period: Duration) -> SystemAnalytics {
        SystemAnalytics::default()
    }

    async fn flush(&self) {
        let events = {
            let mut buffer = self.buffer.write().await;
            std::mem::take(&mut *buffer)
        };

        if !events.is_empty() {
            if let Err(e) = self.backend.write(events).await {
                eprintln!("Failed to flush analytics: {}", e);
            }
        }
    }
}

/// Metrics for Prometheus export
#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    evaluations_total: prometheus::IntCounterVec,
    evaluation_duration: prometheus::HistogramVec,
    active_experiments: prometheus::IntGauge,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let evaluations_total = prometheus::register_int_counter_vec!(
            "flag_evaluations_total",
            "Total number of flag evaluations",
            &["flag_id", "environment", "value", "reason"]
        ).unwrap();

        let evaluation_duration = prometheus::register_histogram_vec!(
            "flag_evaluation_duration_seconds",
            "Flag evaluation duration in seconds",
            &["flag_id"],
            vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
        ).unwrap();

        let active_experiments = prometheus::register_int_gauge!(
            "flag_active_experiments",
            "Number of active experiments"
        ).unwrap();

        Self {
            evaluations_total,
            evaluation_duration,
            active_experiments,
        }
    }

    pub fn record_evaluation(&self, event: &EvaluationEvent) {
        self.evaluations_total
            .with_label_values(&[
                &event.flag_id,
                &event.environment,
                &event.value.to_string(),
                &event.reason,
            ])
            .inc();

        self.evaluation_duration
            .with_label_values(&[&event.flag_id])
            .observe(event.duration_us as f64 / 1_000_000.0);
    }
}

impl Default for PrometheusMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inmemory_analytics() {
        let collector = InMemoryAnalyticsCollector::new(1000);

        let event = EvaluationEvent {
            event_id: "evt-1".to_string(),
            flag_id: "test-flag".to_string(),
            user_id: Some("user-1".to_string()),
            anonymous_id: None,
            value: serde_json::json!(true),
            reason: "default".to_string(),
            matched_rule: None,
            in_experiment: false,
            variant: None,
            duration_us: 100,
            environment: "production".to_string(),
            timestamp: Utc::now(),
            context: HashMap::new(),
        };

        collector.record_evaluation(event).await;

        let analytics = collector.get_flag_analytics("test-flag", Duration::hours(1)).await;
        assert_eq!(analytics.total_evaluations, 1);
        assert_eq!(analytics.unique_users, 1);
    }
}
```

## Analytics Dashboard Queries

```sql
-- Evaluations per flag per hour
SELECT
    flag_id,
    date_trunc('hour', timestamp) as hour,
    COUNT(*) as evaluations,
    COUNT(DISTINCT user_id) as unique_users,
    AVG(duration_us) as avg_duration
FROM flag_evaluations
WHERE timestamp > NOW() - INTERVAL '24 hours'
GROUP BY flag_id, date_trunc('hour', timestamp)
ORDER BY hour DESC;

-- Experiment variant distribution
SELECT
    flag_id,
    variant,
    COUNT(*) as count,
    COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (PARTITION BY flag_id) as percentage
FROM flag_evaluations
WHERE in_experiment = true
    AND timestamp > NOW() - INTERVAL '7 days'
GROUP BY flag_id, variant;
```

## Related Specs

- 394-flag-evaluation.md - Evaluation events
- 399-ab-testing.md - Experiment tracking
- 427-dashboard-data.md - Dashboard integration
