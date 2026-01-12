# Spec 415: Backend Analytics

## Phase
19 - Analytics/Telemetry

## Spec ID
415

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 408: Analytics Collector (event collection)
- Spec 102: Backend System (backend infrastructure)

## Estimated Context
~9%

---

## Objective

Implement comprehensive analytics for backend/LLM provider usage, tracking performance, reliability, cost efficiency, and usage patterns across different AI providers and models.

---

## Acceptance Criteria

- [ ] Track backend selection and usage
- [ ] Measure request/response latencies
- [ ] Monitor error rates by backend
- [ ] Calculate cost per backend/model
- [ ] Implement availability tracking
- [ ] Create backend comparison metrics
- [ ] Support model performance analysis
- [ ] Enable backend optimization insights

---

## Implementation Details

### Backend Analytics

```rust
// src/analytics/backends.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::types::{
    AnalyticsEvent, BusinessEventData, BusinessMetricType, EventBuilder,
    EventData, EventType, PerformanceEventData,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Backend identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackendId {
    pub provider: String,
    pub model: String,
}

impl BackendId {
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
        }
    }
}

impl std::fmt::Display for BackendId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.provider, self.model)
    }
}

/// Request metrics for a single backend call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Request identifier
    pub request_id: String,
    /// Backend used
    pub backend: BackendId,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time
    pub completed_at: Option<DateTime<Utc>>,
    /// Time to first token (ms)
    pub time_to_first_token_ms: Option<u64>,
    /// Total latency (ms)
    pub total_latency_ms: Option<u64>,
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Cost in USD
    pub cost_usd: f64,
    /// Whether request succeeded
    pub success: bool,
    /// Error code if failed
    pub error_code: Option<String>,
    /// HTTP status code
    pub http_status: Option<u16>,
    /// Retry count
    pub retry_count: u32,
    /// Whether response was streamed
    pub streaming: bool,
}

impl RequestMetrics {
    pub fn new(request_id: &str, backend: BackendId) -> Self {
        Self {
            request_id: request_id.to_string(),
            backend,
            started_at: Utc::now(),
            completed_at: None,
            time_to_first_token_ms: None,
            total_latency_ms: None,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0.0,
            success: false,
            error_code: None,
            http_status: None,
            retry_count: 0,
            streaming: false,
        }
    }

    pub fn complete(&mut self, success: bool) {
        self.completed_at = Some(Utc::now());
        self.success = success;
        if let Some(completed) = self.completed_at {
            self.total_latency_ms = Some(
                (completed - self.started_at).num_milliseconds() as u64
            );
        }
    }
}

/// Aggregated backend statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendStats {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// Median latency (ms)
    pub median_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// P99 latency (ms)
    pub p99_latency_ms: f64,
    /// Average time to first token (ms)
    pub avg_ttft_ms: f64,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Total cost (USD)
    pub total_cost_usd: f64,
    /// Average cost per request
    pub avg_cost_per_request: f64,
    /// Tokens per second (output)
    pub avg_tokens_per_second: f64,
    /// Error breakdown by code
    pub errors_by_code: HashMap<String, u64>,
    /// Requests by hour
    pub requests_by_hour: HashMap<u32, u64>,
}

/// Backend health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl BackendHealth {
    pub fn from_metrics(stats: &BackendStats, window_minutes: u64) -> Self {
        // Unhealthy if success rate < 90%
        if stats.success_rate < 0.9 {
            return Self::Unhealthy;
        }

        // Degraded if success rate < 99% or high latency
        if stats.success_rate < 0.99 || stats.avg_latency_ms > 5000.0 {
            return Self::Degraded;
        }

        // Check if we have recent data
        if stats.total_requests == 0 {
            return Self::Unknown;
        }

        Self::Healthy
    }
}

/// Backend analytics tracker
pub struct BackendTracker {
    /// Event collector
    collector: Arc<EventCollector>,
    /// Active requests being tracked
    active_requests: Arc<RwLock<HashMap<String, RequestMetrics>>>,
    /// Historical request data
    request_history: Arc<RwLock<Vec<RequestMetrics>>>,
    /// Cached stats per backend
    stats_cache: Arc<RwLock<HashMap<BackendId, BackendStats>>>,
    /// Last cache update
    cache_updated: Arc<RwLock<DateTime<Utc>>>,
}

impl BackendTracker {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        Self {
            collector,
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
            stats_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_updated: Arc::new(RwLock::new(Utc::now() - Duration::hours(1))),
        }
    }

    /// Start tracking a request
    pub async fn start_request(
        &self,
        request_id: &str,
        provider: &str,
        model: &str,
    ) -> Result<(), BackendTrackingError> {
        let backend = BackendId::new(provider, model);
        let metrics = RequestMetrics::new(request_id, backend.clone());

        {
            let mut active = self.active_requests.write().await;
            active.insert(request_id.to_string(), metrics);
        }

        // Emit event
        let event = EventBuilder::new(EventType::BackendSelected)
            .usage_data("backend", &backend.to_string(), true)
            .custom_metadata("request_id", serde_json::json!(request_id))
            .custom_metadata("provider", serde_json::json!(provider))
            .custom_metadata("model", serde_json::json!(model))
            .build();

        self.collector
            .collect(event)
            .await
            .map_err(|e| BackendTrackingError::CollectionFailed(e.to_string()))?;

        Ok(())
    }

    /// Record time to first token
    pub async fn record_first_token(
        &self,
        request_id: &str,
    ) -> Result<(), BackendTrackingError> {
        let mut active = self.active_requests.write().await;
        let metrics = active
            .get_mut(request_id)
            .ok_or(BackendTrackingError::RequestNotFound)?;

        let elapsed = (Utc::now() - metrics.started_at).num_milliseconds() as u64;
        metrics.time_to_first_token_ms = Some(elapsed);

        Ok(())
    }

    /// Complete a request successfully
    pub async fn complete_request(
        &self,
        request_id: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
    ) -> Result<(), BackendTrackingError> {
        let metrics = {
            let mut active = self.active_requests.write().await;
            let mut metrics = active
                .remove(request_id)
                .ok_or(BackendTrackingError::RequestNotFound)?;

            metrics.complete(true);
            metrics.input_tokens = input_tokens;
            metrics.output_tokens = output_tokens;
            metrics.cost_usd = cost_usd;
            metrics
        };

        // Emit latency event
        if let Some(latency) = metrics.total_latency_ms {
            let event = EventBuilder::new(EventType::ResponseLatency)
                .data(EventData::Performance(PerformanceEventData {
                    metric: "response_latency".to_string(),
                    value: latency as f64,
                    unit: "ms".to_string(),
                    tags: [
                        ("provider".to_string(), metrics.backend.provider.clone()),
                        ("model".to_string(), metrics.backend.model.clone()),
                    ]
                    .into_iter()
                    .collect(),
                }))
                .build();

            self.collector.collect(event).await.ok();
        }

        // Emit token event
        let event = EventBuilder::new(EventType::TokensConsumed)
            .data(EventData::Business(BusinessEventData {
                metric_type: BusinessMetricType::TotalTokens,
                value: (input_tokens + output_tokens) as f64,
                unit: "tokens".to_string(),
                backend: Some(metrics.backend.provider.clone()),
                model: Some(metrics.backend.model.clone()),
            }))
            .custom_metadata("input_tokens", serde_json::json!(input_tokens))
            .custom_metadata("output_tokens", serde_json::json!(output_tokens))
            .build();

        self.collector.collect(event).await.ok();

        // Add to history
        let mut history = self.request_history.write().await;
        history.push(metrics);

        // Invalidate cache
        *self.cache_updated.write().await = Utc::now() - Duration::hours(1);

        Ok(())
    }

    /// Record a failed request
    pub async fn fail_request(
        &self,
        request_id: &str,
        error_code: &str,
        http_status: Option<u16>,
    ) -> Result<(), BackendTrackingError> {
        let metrics = {
            let mut active = self.active_requests.write().await;
            let mut metrics = active
                .remove(request_id)
                .ok_or(BackendTrackingError::RequestNotFound)?;

            metrics.complete(false);
            metrics.error_code = Some(error_code.to_string());
            metrics.http_status = http_status;
            metrics
        };

        // Emit error event
        let event = EventBuilder::new(EventType::ErrorOccurred)
            .error_data(
                error_code,
                &format!("Backend request failed: {}", error_code),
                crate::analytics::types::ErrorSeverity::Error,
                &format!("backend:{}", metrics.backend),
            )
            .custom_metadata("request_id", serde_json::json!(request_id))
            .custom_metadata("provider", serde_json::json!(metrics.backend.provider.clone()))
            .custom_metadata("model", serde_json::json!(metrics.backend.model.clone()))
            .build();

        self.collector.collect(event).await.ok();

        // Add to history
        let mut history = self.request_history.write().await;
        history.push(metrics);

        Ok(())
    }

    /// Get statistics for a specific backend
    pub async fn get_backend_stats(&self, backend: &BackendId) -> BackendStats {
        self.refresh_cache_if_needed().await;

        let cache = self.stats_cache.read().await;
        cache.get(backend).cloned().unwrap_or_default()
    }

    /// Get statistics for all backends
    pub async fn get_all_stats(&self) -> HashMap<BackendId, BackendStats> {
        self.refresh_cache_if_needed().await;

        self.stats_cache.read().await.clone()
    }

    /// Get health status for a backend
    pub async fn get_health(&self, backend: &BackendId) -> BackendHealth {
        let stats = self.get_backend_stats(backend).await;
        BackendHealth::from_metrics(&stats, 60)
    }

    /// Refresh stats cache if stale
    async fn refresh_cache_if_needed(&self) {
        let cache_age = Utc::now() - *self.cache_updated.read().await;

        if cache_age > Duration::minutes(5) {
            self.calculate_stats().await;
        }
    }

    /// Calculate statistics from history
    async fn calculate_stats(&self) {
        let history = self.request_history.read().await;

        let mut stats_map: HashMap<BackendId, BackendStats> = HashMap::new();
        let mut latencies: HashMap<BackendId, Vec<f64>> = HashMap::new();
        let mut ttfts: HashMap<BackendId, Vec<f64>> = HashMap::new();

        for request in history.iter() {
            let stats = stats_map
                .entry(request.backend.clone())
                .or_insert_with(BackendStats::default);

            stats.total_requests += 1;

            if request.success {
                stats.successful_requests += 1;
            } else {
                stats.failed_requests += 1;
                if let Some(ref code) = request.error_code {
                    *stats.errors_by_code.entry(code.clone()).or_insert(0) += 1;
                }
            }

            stats.total_input_tokens += request.input_tokens;
            stats.total_output_tokens += request.output_tokens;
            stats.total_cost_usd += request.cost_usd;

            // Track latencies
            if let Some(latency) = request.total_latency_ms {
                latencies
                    .entry(request.backend.clone())
                    .or_default()
                    .push(latency as f64);
            }

            // Track TTFT
            if let Some(ttft) = request.time_to_first_token_ms {
                ttfts
                    .entry(request.backend.clone())
                    .or_default()
                    .push(ttft as f64);
            }

            // Requests by hour
            let hour = request.started_at.hour();
            *stats.requests_by_hour.entry(hour).or_insert(0) += 1;
        }

        // Calculate derived metrics
        for (backend, stats) in stats_map.iter_mut() {
            if stats.total_requests > 0 {
                stats.success_rate =
                    stats.successful_requests as f64 / stats.total_requests as f64;
                stats.avg_cost_per_request =
                    stats.total_cost_usd / stats.total_requests as f64;
            }

            // Latency percentiles
            if let Some(lats) = latencies.get_mut(backend) {
                if !lats.is_empty() {
                    lats.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let len = lats.len();

                    stats.avg_latency_ms = lats.iter().sum::<f64>() / len as f64;
                    stats.median_latency_ms = lats[len / 2];
                    stats.p95_latency_ms = lats[(len * 95) / 100];
                    stats.p99_latency_ms = lats[(len * 99) / 100];

                    // Tokens per second
                    let total_output_time: f64 = lats.iter().sum();
                    if total_output_time > 0.0 {
                        stats.avg_tokens_per_second =
                            (stats.total_output_tokens as f64 * 1000.0) / total_output_time;
                    }
                }
            }

            // Average TTFT
            if let Some(ts) = ttfts.get(backend) {
                if !ts.is_empty() {
                    stats.avg_ttft_ms = ts.iter().sum::<f64>() / ts.len() as f64;
                }
            }
        }

        // Update cache
        *self.stats_cache.write().await = stats_map;
        *self.cache_updated.write().await = Utc::now();
    }

    /// Compare multiple backends
    pub async fn compare_backends(
        &self,
        backends: &[BackendId],
    ) -> BackendComparison {
        let all_stats = self.get_all_stats().await;

        let mut comparisons: Vec<BackendComparisonEntry> = backends
            .iter()
            .filter_map(|b| {
                all_stats.get(b).map(|stats| BackendComparisonEntry {
                    backend: b.clone(),
                    stats: stats.clone(),
                    health: BackendHealth::from_metrics(stats, 60),
                    score: calculate_backend_score(stats),
                })
            })
            .collect();

        comparisons.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        BackendComparison {
            entries: comparisons,
            compared_at: Utc::now(),
        }
    }
}

/// Calculate overall score for a backend (0-100)
fn calculate_backend_score(stats: &BackendStats) -> f64 {
    if stats.total_requests == 0 {
        return 0.0;
    }

    let mut score = 0.0;

    // Reliability (40%)
    score += stats.success_rate * 40.0;

    // Speed (30%) - inverse of latency, normalized
    let latency_score = if stats.avg_latency_ms > 0.0 {
        (1.0 - (stats.avg_latency_ms / 10000.0).min(1.0)) * 30.0
    } else {
        30.0
    };
    score += latency_score;

    // Cost efficiency (20%) - inverse of cost per token
    let cost_per_token = if stats.total_output_tokens > 0 {
        stats.total_cost_usd / stats.total_output_tokens as f64
    } else {
        0.0
    };
    let cost_score = (1.0 - (cost_per_token * 10000.0).min(1.0)) * 20.0;
    score += cost_score;

    // Throughput (10%)
    let throughput_score = (stats.avg_tokens_per_second / 100.0).min(1.0) * 10.0;
    score += throughput_score;

    score.min(100.0)
}

/// Backend comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendComparison {
    pub entries: Vec<BackendComparisonEntry>,
    pub compared_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendComparisonEntry {
    pub backend: BackendId,
    pub stats: BackendStats,
    pub health: BackendHealth,
    pub score: f64,
}

/// Backend tracking errors
#[derive(Debug, thiserror::Error)]
pub enum BackendTrackingError {
    #[error("Request not found")]
    RequestNotFound,

    #[error("Collection failed: {0}")]
    CollectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_tracker() -> BackendTracker {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        BackendTracker::new(collector)
    }

    #[tokio::test]
    async fn test_request_tracking() {
        let tracker = create_tracker().await;

        tracker
            .start_request("req-1", "anthropic", "claude-3-opus")
            .await
            .unwrap();

        tracker.record_first_token("req-1").await.unwrap();

        tracker
            .complete_request("req-1", 500, 300, 0.05)
            .await
            .unwrap();

        let backend = BackendId::new("anthropic", "claude-3-opus");
        let stats = tracker.get_backend_stats(&backend).await;

        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.total_input_tokens, 500);
        assert_eq!(stats.total_output_tokens, 300);
    }

    #[tokio::test]
    async fn test_failure_tracking() {
        let tracker = create_tracker().await;

        tracker
            .start_request("req-1", "openai", "gpt-4")
            .await
            .unwrap();

        tracker
            .fail_request("req-1", "RATE_LIMIT", Some(429))
            .await
            .unwrap();

        let backend = BackendId::new("openai", "gpt-4");
        let stats = tracker.get_backend_stats(&backend).await;

        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.errors_by_code.get("RATE_LIMIT"), Some(&1));
    }

    #[tokio::test]
    async fn test_backend_comparison() {
        let tracker = create_tracker().await;

        // Simulate requests to different backends
        for i in 0..10 {
            let req_id = format!("req-anthropic-{}", i);
            tracker
                .start_request(&req_id, "anthropic", "claude-3-opus")
                .await
                .unwrap();
            tracker
                .complete_request(&req_id, 500, 300, 0.05)
                .await
                .unwrap();
        }

        for i in 0..5 {
            let req_id = format!("req-openai-{}", i);
            tracker
                .start_request(&req_id, "openai", "gpt-4")
                .await
                .unwrap();
            if i % 2 == 0 {
                tracker
                    .complete_request(&req_id, 400, 200, 0.04)
                    .await
                    .unwrap();
            } else {
                tracker
                    .fail_request(&req_id, "ERROR", None)
                    .await
                    .unwrap();
            }
        }

        let backends = vec![
            BackendId::new("anthropic", "claude-3-opus"),
            BackendId::new("openai", "gpt-4"),
        ];

        let comparison = tracker.compare_backends(&backends).await;

        assert_eq!(comparison.entries.len(), 2);
        // Anthropic should score higher (100% success rate)
        assert!(comparison.entries[0].backend.provider == "anthropic");
    }

    #[test]
    fn test_backend_health() {
        let mut stats = BackendStats::default();
        stats.total_requests = 100;
        stats.successful_requests = 100;
        stats.success_rate = 1.0;

        assert_eq!(BackendHealth::from_metrics(&stats, 60), BackendHealth::Healthy);

        stats.successful_requests = 95;
        stats.success_rate = 0.95;
        assert_eq!(BackendHealth::from_metrics(&stats, 60), BackendHealth::Degraded);

        stats.successful_requests = 80;
        stats.success_rate = 0.8;
        assert_eq!(BackendHealth::from_metrics(&stats, 60), BackendHealth::Unhealthy);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Request tracking lifecycle
   - Stats calculation accuracy
   - Health status determination
   - Backend scoring

2. **Integration Tests**
   - Multi-backend tracking
   - Concurrent request handling
   - Event emission

3. **Performance Tests**
   - High-volume request tracking
   - Stats calculation speed

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 416: Token Tracking
- Spec 417: Cost Tracking
