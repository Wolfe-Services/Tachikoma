# Spec 414: Mission Analytics

## Phase
19 - Analytics/Telemetry

## Spec ID
414

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 408: Analytics Collector (event collection)
- Spec 201: Mission System (mission infrastructure)

## Estimated Context
~10%

---

## Objective

Implement comprehensive analytics for mission execution, tracking success rates, completion times, resource usage, and patterns to provide insights into mission effectiveness and identify optimization opportunities.

---

## Acceptance Criteria

- [ ] Track mission lifecycle events
- [ ] Measure mission completion times
- [ ] Calculate success/failure rates
- [ ] Analyze mission complexity metrics
- [ ] Track tool usage within missions
- [ ] Implement mission pattern detection
- [ ] Create mission efficiency scores
- [ ] Support comparative mission analysis

---

## Implementation Details

### Mission Analytics

```rust
// src/analytics/missions.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::storage::AnalyticsStorage;
use crate::analytics::types::{
    AnalyticsEvent, BusinessEventData, BusinessMetricType, EventBuilder,
    EventData, EventType, UsageEventData,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mission status for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Created,
    Planning,
    Executing,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Mission analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionAnalytics {
    /// Mission identifier
    pub mission_id: Uuid,
    /// Session this mission belongs to
    pub session_id: Option<Uuid>,
    /// Mission status
    pub status: MissionStatus,
    /// When mission was created
    pub created_at: DateTime<Utc>,
    /// When mission started execution
    pub started_at: Option<DateTime<Utc>>,
    /// When mission completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Total execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Number of steps/tasks
    pub step_count: u32,
    /// Steps completed
    pub steps_completed: u32,
    /// Tools invoked during mission
    pub tools_used: Vec<ToolUsage>,
    /// Total tokens consumed
    pub total_tokens: u64,
    /// Total cost incurred
    pub total_cost: f64,
    /// Backend used
    pub backend: String,
    /// Model used
    pub model: String,
    /// Mission tags/categories
    pub tags: Vec<String>,
    /// Error information if failed
    pub error: Option<MissionError>,
    /// Retry count
    pub retry_count: u32,
}

impl MissionAnalytics {
    pub fn new(mission_id: Uuid, backend: &str, model: &str) -> Self {
        Self {
            mission_id,
            session_id: None,
            status: MissionStatus::Created,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            execution_time_ms: None,
            step_count: 0,
            steps_completed: 0,
            tools_used: Vec::new(),
            total_tokens: 0,
            total_cost: 0.0,
            backend: backend.to_string(),
            model: model.to_string(),
            tags: Vec::new(),
            error: None,
            retry_count: 0,
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }

    pub fn success_rate(&self) -> Option<f64> {
        if self.step_count == 0 {
            return None;
        }
        Some(self.steps_completed as f64 / self.step_count as f64)
    }
}

/// Tool usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    pub tool_name: String,
    pub invocation_count: u32,
    pub total_time_ms: u64,
    pub success_count: u32,
    pub failure_count: u32,
}

/// Mission error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionError {
    pub code: String,
    pub message: String,
    pub step_index: Option<u32>,
    pub recoverable: bool,
}

/// Aggregated mission metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionMetrics {
    /// Total missions tracked
    pub total_missions: u64,
    /// Successful completions
    pub completed_missions: u64,
    /// Failed missions
    pub failed_missions: u64,
    /// Cancelled missions
    pub cancelled_missions: u64,
    /// Overall success rate
    pub success_rate: f64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Median execution time
    pub median_execution_time_ms: f64,
    /// 90th percentile execution time
    pub p90_execution_time_ms: f64,
    /// Average steps per mission
    pub avg_steps: f64,
    /// Total tokens consumed
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// Most used tools
    pub top_tools: Vec<(String, u64)>,
    /// Most common failure reasons
    pub top_errors: Vec<(String, u64)>,
    /// Missions by status
    pub by_status: HashMap<MissionStatus, u64>,
    /// Missions by backend
    pub by_backend: HashMap<String, u64>,
}

impl Default for MissionMetrics {
    fn default() -> Self {
        Self {
            total_missions: 0,
            completed_missions: 0,
            failed_missions: 0,
            cancelled_missions: 0,
            success_rate: 0.0,
            avg_execution_time_ms: 0.0,
            median_execution_time_ms: 0.0,
            p90_execution_time_ms: 0.0,
            avg_steps: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
            top_tools: Vec::new(),
            top_errors: Vec::new(),
            by_status: HashMap::new(),
            by_backend: HashMap::new(),
        }
    }
}

/// Mission analytics tracker
pub struct MissionTracker {
    /// Event collector
    collector: Arc<EventCollector>,
    /// Active missions
    active_missions: Arc<RwLock<HashMap<Uuid, MissionAnalytics>>>,
    /// Historical mission data cache
    history_cache: Arc<RwLock<Vec<MissionAnalytics>>>,
}

impl MissionTracker {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        Self {
            collector,
            active_missions: Arc::new(RwLock::new(HashMap::new())),
            history_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start tracking a new mission
    pub async fn start_mission(
        &self,
        mission_id: Uuid,
        backend: &str,
        model: &str,
        tags: Vec<String>,
    ) -> Result<(), MissionTrackingError> {
        let mut analytics = MissionAnalytics::new(mission_id, backend, model);
        analytics.session_id = Some(self.collector.session_id());
        analytics.tags = tags;

        // Store in active missions
        {
            let mut active = self.active_missions.write().await;
            active.insert(mission_id, analytics);
        }

        // Emit event
        let event = EventBuilder::new(EventType::MissionCreated)
            .usage_data("mission", "create", true)
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .custom_metadata("backend", serde_json::json!(backend))
            .custom_metadata("model", serde_json::json!(model))
            .build();

        self.collector
            .collect(event)
            .await
            .map_err(|e| MissionTrackingError::CollectionFailed(e.to_string()))?;

        Ok(())
    }

    /// Mark mission as started
    pub async fn mission_started(&self, mission_id: Uuid) -> Result<(), MissionTrackingError> {
        let mut active = self.active_missions.write().await;
        let mission = active
            .get_mut(&mission_id)
            .ok_or(MissionTrackingError::MissionNotFound)?;

        mission.status = MissionStatus::Executing;
        mission.started_at = Some(Utc::now());

        Ok(())
    }

    /// Record step completion
    pub async fn step_completed(
        &self,
        mission_id: Uuid,
        step_index: u32,
        success: bool,
    ) -> Result<(), MissionTrackingError> {
        let mut active = self.active_missions.write().await;
        let mission = active
            .get_mut(&mission_id)
            .ok_or(MissionTrackingError::MissionNotFound)?;

        mission.step_count = mission.step_count.max(step_index + 1);
        if success {
            mission.steps_completed += 1;
        }

        Ok(())
    }

    /// Record tool invocation
    pub async fn tool_invoked(
        &self,
        mission_id: Uuid,
        tool_name: &str,
        duration_ms: u64,
        success: bool,
    ) -> Result<(), MissionTrackingError> {
        let mut active = self.active_missions.write().await;
        let mission = active
            .get_mut(&mission_id)
            .ok_or(MissionTrackingError::MissionNotFound)?;

        // Find or create tool usage entry
        let tool_usage = mission
            .tools_used
            .iter_mut()
            .find(|t| t.tool_name == tool_name);

        if let Some(usage) = tool_usage {
            usage.invocation_count += 1;
            usage.total_time_ms += duration_ms;
            if success {
                usage.success_count += 1;
            } else {
                usage.failure_count += 1;
            }
        } else {
            mission.tools_used.push(ToolUsage {
                tool_name: tool_name.to_string(),
                invocation_count: 1,
                total_time_ms: duration_ms,
                success_count: if success { 1 } else { 0 },
                failure_count: if success { 0 } else { 1 },
            });
        }

        // Emit tool event
        let event = EventBuilder::new(EventType::ToolInvoked)
            .usage_data("tool", tool_name, success)
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .custom_metadata("duration_ms", serde_json::json!(duration_ms))
            .build();

        drop(active);
        self.collector.collect(event).await.ok();

        Ok(())
    }

    /// Record token consumption
    pub async fn tokens_consumed(
        &self,
        mission_id: Uuid,
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
    ) -> Result<(), MissionTrackingError> {
        let mut active = self.active_missions.write().await;
        let mission = active
            .get_mut(&mission_id)
            .ok_or(MissionTrackingError::MissionNotFound)?;

        mission.total_tokens += input_tokens + output_tokens;
        mission.total_cost += cost;

        // Emit token event
        let event = EventBuilder::new(EventType::TokensConsumed)
            .data(EventData::Business(BusinessEventData {
                metric_type: BusinessMetricType::TotalTokens,
                value: (input_tokens + output_tokens) as f64,
                unit: "tokens".to_string(),
                backend: Some(mission.backend.clone()),
                model: Some(mission.model.clone()),
            }))
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .build();

        drop(active);
        self.collector.collect(event).await.ok();

        Ok(())
    }

    /// Mark mission as completed
    pub async fn mission_completed(&self, mission_id: Uuid) -> Result<(), MissionTrackingError> {
        let analytics = {
            let mut active = self.active_missions.write().await;
            let mut mission = active
                .remove(&mission_id)
                .ok_or(MissionTrackingError::MissionNotFound)?;

            mission.status = MissionStatus::Completed;
            mission.completed_at = Some(Utc::now());

            if let (Some(start), Some(end)) = (mission.started_at, mission.completed_at) {
                mission.execution_time_ms = Some((end - start).num_milliseconds() as u64);
            }

            mission
        };

        // Emit completion event
        let event = EventBuilder::new(EventType::MissionCompleted)
            .usage_data("mission", "complete", true)
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .custom_metadata("execution_time_ms", serde_json::json!(analytics.execution_time_ms))
            .custom_metadata("total_tokens", serde_json::json!(analytics.total_tokens))
            .custom_metadata("total_cost", serde_json::json!(analytics.total_cost))
            .build();

        self.collector.collect(event).await.ok();

        // Add to history cache
        let mut history = self.history_cache.write().await;
        history.push(analytics);

        Ok(())
    }

    /// Mark mission as failed
    pub async fn mission_failed(
        &self,
        mission_id: Uuid,
        error: MissionError,
    ) -> Result<(), MissionTrackingError> {
        let analytics = {
            let mut active = self.active_missions.write().await;
            let mut mission = active
                .remove(&mission_id)
                .ok_or(MissionTrackingError::MissionNotFound)?;

            mission.status = MissionStatus::Failed;
            mission.completed_at = Some(Utc::now());
            mission.error = Some(error.clone());

            if let (Some(start), Some(end)) = (mission.started_at, mission.completed_at) {
                mission.execution_time_ms = Some((end - start).num_milliseconds() as u64);
            }

            mission
        };

        // Emit failure event
        let event = EventBuilder::new(EventType::MissionFailed)
            .usage_data("mission", "fail", false)
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .custom_metadata("error_code", serde_json::json!(error.code))
            .custom_metadata("error_message", serde_json::json!(error.message))
            .build();

        self.collector.collect(event).await.ok();

        // Add to history cache
        let mut history = self.history_cache.write().await;
        history.push(analytics);

        Ok(())
    }

    /// Mark mission as cancelled
    pub async fn mission_cancelled(&self, mission_id: Uuid) -> Result<(), MissionTrackingError> {
        let analytics = {
            let mut active = self.active_missions.write().await;
            let mut mission = active
                .remove(&mission_id)
                .ok_or(MissionTrackingError::MissionNotFound)?;

            mission.status = MissionStatus::Cancelled;
            mission.completed_at = Some(Utc::now());

            mission
        };

        // Emit cancellation event
        let event = EventBuilder::new(EventType::MissionCancelled)
            .usage_data("mission", "cancel", true)
            .custom_metadata("mission_id", serde_json::json!(mission_id.to_string()))
            .build();

        self.collector.collect(event).await.ok();

        // Add to history cache
        let mut history = self.history_cache.write().await;
        history.push(analytics);

        Ok(())
    }

    /// Get active mission count
    pub async fn active_count(&self) -> usize {
        self.active_missions.read().await.len()
    }

    /// Get mission analytics by ID
    pub async fn get_mission(&self, mission_id: Uuid) -> Option<MissionAnalytics> {
        // Check active first
        if let Some(mission) = self.active_missions.read().await.get(&mission_id) {
            return Some(mission.clone());
        }

        // Check history cache
        let history = self.history_cache.read().await;
        history.iter().find(|m| m.mission_id == mission_id).cloned()
    }

    /// Calculate aggregated metrics
    pub async fn calculate_metrics(&self) -> MissionMetrics {
        let history = self.history_cache.read().await;

        if history.is_empty() {
            return MissionMetrics::default();
        }

        let mut metrics = MissionMetrics::default();
        metrics.total_missions = history.len() as u64;

        let mut execution_times: Vec<f64> = Vec::new();
        let mut tool_counts: HashMap<String, u64> = HashMap::new();
        let mut error_counts: HashMap<String, u64> = HashMap::new();

        for mission in history.iter() {
            // Count by status
            *metrics.by_status.entry(mission.status).or_insert(0) += 1;

            // Count by backend
            *metrics.by_backend.entry(mission.backend.clone()).or_insert(0) += 1;

            match mission.status {
                MissionStatus::Completed => metrics.completed_missions += 1,
                MissionStatus::Failed => metrics.failed_missions += 1,
                MissionStatus::Cancelled => metrics.cancelled_missions += 1,
                _ => {}
            }

            // Execution time
            if let Some(time) = mission.execution_time_ms {
                execution_times.push(time as f64);
            }

            // Tool usage
            for tool in &mission.tools_used {
                *tool_counts.entry(tool.tool_name.clone()).or_insert(0) +=
                    tool.invocation_count as u64;
            }

            // Error tracking
            if let Some(ref error) = mission.error {
                *error_counts.entry(error.code.clone()).or_insert(0) += 1;
            }

            // Totals
            metrics.total_tokens += mission.total_tokens;
            metrics.total_cost += mission.total_cost;
            metrics.avg_steps += mission.step_count as f64;
        }

        // Calculate success rate
        if metrics.total_missions > 0 {
            metrics.success_rate =
                metrics.completed_missions as f64 / metrics.total_missions as f64;
            metrics.avg_steps /= metrics.total_missions as f64;
        }

        // Calculate execution time statistics
        if !execution_times.is_empty() {
            execution_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let len = execution_times.len();

            metrics.avg_execution_time_ms =
                execution_times.iter().sum::<f64>() / len as f64;
            metrics.median_execution_time_ms = execution_times[len / 2];
            metrics.p90_execution_time_ms = execution_times[(len * 9) / 10];
        }

        // Top tools
        let mut tools: Vec<_> = tool_counts.into_iter().collect();
        tools.sort_by(|a, b| b.1.cmp(&a.1));
        metrics.top_tools = tools.into_iter().take(10).collect();

        // Top errors
        let mut errors: Vec<_> = error_counts.into_iter().collect();
        errors.sort_by(|a, b| b.1.cmp(&a.1));
        metrics.top_errors = errors.into_iter().take(10).collect();

        metrics
    }
}

/// Mission tracking errors
#[derive(Debug, thiserror::Error)]
pub enum MissionTrackingError {
    #[error("Mission not found")]
    MissionNotFound,

    #[error("Collection failed: {0}")]
    CollectionFailed(String),

    #[error("Invalid state transition")]
    InvalidStateTransition,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_tracker() -> MissionTracker {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        MissionTracker::new(collector)
    }

    #[tokio::test]
    async fn test_mission_lifecycle() {
        let tracker = create_tracker().await;
        let mission_id = Uuid::new_v4();

        // Start mission
        tracker
            .start_mission(mission_id, "anthropic", "claude-3-opus", vec![])
            .await
            .unwrap();

        assert_eq!(tracker.active_count().await, 1);

        // Start execution
        tracker.mission_started(mission_id).await.unwrap();

        // Record some activity
        tracker.step_completed(mission_id, 0, true).await.unwrap();
        tracker
            .tool_invoked(mission_id, "search", 100, true)
            .await
            .unwrap();
        tracker
            .tokens_consumed(mission_id, 500, 300, 0.05)
            .await
            .unwrap();

        // Complete mission
        tracker.mission_completed(mission_id).await.unwrap();

        assert_eq!(tracker.active_count().await, 0);

        // Check analytics
        let analytics = tracker.get_mission(mission_id).await.unwrap();
        assert_eq!(analytics.status, MissionStatus::Completed);
        assert_eq!(analytics.total_tokens, 800);
        assert_eq!(analytics.tools_used.len(), 1);
    }

    #[tokio::test]
    async fn test_mission_failure() {
        let tracker = create_tracker().await;
        let mission_id = Uuid::new_v4();

        tracker
            .start_mission(mission_id, "openai", "gpt-4", vec![])
            .await
            .unwrap();

        tracker.mission_started(mission_id).await.unwrap();

        tracker
            .mission_failed(
                mission_id,
                MissionError {
                    code: "TIMEOUT".to_string(),
                    message: "Request timed out".to_string(),
                    step_index: Some(2),
                    recoverable: true,
                },
            )
            .await
            .unwrap();

        let analytics = tracker.get_mission(mission_id).await.unwrap();
        assert_eq!(analytics.status, MissionStatus::Failed);
        assert!(analytics.error.is_some());
    }

    #[tokio::test]
    async fn test_metrics_calculation() {
        let tracker = create_tracker().await;

        // Complete several missions
        for _ in 0..5 {
            let mission_id = Uuid::new_v4();
            tracker
                .start_mission(mission_id, "anthropic", "claude-3-opus", vec![])
                .await
                .unwrap();
            tracker.mission_started(mission_id).await.unwrap();
            tracker.tokens_consumed(mission_id, 100, 50, 0.01).await.unwrap();
            tracker.mission_completed(mission_id).await.unwrap();
        }

        // Fail one mission
        let mission_id = Uuid::new_v4();
        tracker
            .start_mission(mission_id, "anthropic", "claude-3-opus", vec![])
            .await
            .unwrap();
        tracker.mission_started(mission_id).await.unwrap();
        tracker
            .mission_failed(
                mission_id,
                MissionError {
                    code: "ERROR".to_string(),
                    message: "Test error".to_string(),
                    step_index: None,
                    recoverable: false,
                },
            )
            .await
            .unwrap();

        let metrics = tracker.calculate_metrics().await;

        assert_eq!(metrics.total_missions, 6);
        assert_eq!(metrics.completed_missions, 5);
        assert_eq!(metrics.failed_missions, 1);
        assert!((metrics.success_rate - 0.833).abs() < 0.01);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Mission lifecycle tracking
   - Step completion recording
   - Tool usage aggregation
   - Token consumption tracking

2. **Integration Tests**
   - Full mission flow with events
   - Metrics calculation accuracy
   - Concurrent mission handling

3. **Performance Tests**
   - High-volume mission tracking
   - Metrics calculation speed

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 415: Backend Analytics
- Spec 416: Token Tracking
