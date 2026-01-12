# Spec 416: Token Tracking

## Phase
19 - Analytics/Telemetry

## Spec ID
416

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 415: Backend Analytics (backend tracking)

## Estimated Context
~9%

---

## Objective

Implement detailed token consumption tracking across all LLM interactions, enabling accurate usage monitoring, quota management, and cost prediction capabilities.

---

## Acceptance Criteria

- [ ] Track input/output tokens per request
- [ ] Support multiple tokenization schemes
- [ ] Calculate token usage by backend/model
- [ ] Implement usage quotas and limits
- [ ] Create token usage forecasting
- [ ] Support context window tracking
- [ ] Enable token efficiency analysis
- [ ] Provide real-time usage alerts

---

## Implementation Details

### Token Tracking

```rust
// src/analytics/tokens.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::types::{
    BusinessEventData, BusinessMetricType, EventBuilder, EventData, EventType,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Token type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    /// User input tokens
    Input,
    /// Model output tokens
    Output,
    /// System prompt tokens
    System,
    /// Cached tokens (if supported)
    Cached,
}

/// Token usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Request identifier
    pub request_id: String,
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Input token count
    pub input_tokens: u64,
    /// Output token count
    pub output_tokens: u64,
    /// System prompt tokens (subset of input)
    pub system_tokens: u64,
    /// Cached tokens (if applicable)
    pub cached_tokens: u64,
    /// Context window size used
    pub context_used: u64,
    /// Context window limit
    pub context_limit: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Associated cost
    pub cost_usd: f64,
}

impl TokenUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn context_utilization(&self) -> f64 {
        if self.context_limit == 0 {
            return 0.0;
        }
        self.context_used as f64 / self.context_limit as f64
    }
}

/// Token quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenQuota {
    /// Quota identifier
    pub id: String,
    /// Description
    pub description: String,
    /// Maximum tokens allowed
    pub limit: u64,
    /// Period for quota reset
    pub period: QuotaPeriod,
    /// Current usage
    pub used: u64,
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Whether quota is active
    pub active: bool,
    /// Action when quota exceeded
    pub action: QuotaAction,
}

impl TokenQuota {
    pub fn new(id: &str, limit: u64, period: QuotaPeriod) -> Self {
        Self {
            id: id.to_string(),
            description: String::new(),
            limit,
            period,
            used: 0,
            period_start: Utc::now(),
            active: true,
            action: QuotaAction::Warn,
        }
    }

    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    pub fn utilization(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        self.used as f64 / self.limit as f64
    }

    pub fn is_exceeded(&self) -> bool {
        self.used >= self.limit
    }

    pub fn needs_reset(&self) -> bool {
        let now = Utc::now();
        match self.period {
            QuotaPeriod::Hourly => now - self.period_start >= Duration::hours(1),
            QuotaPeriod::Daily => now - self.period_start >= Duration::days(1),
            QuotaPeriod::Weekly => now - self.period_start >= Duration::weeks(1),
            QuotaPeriod::Monthly => now - self.period_start >= Duration::days(30),
            QuotaPeriod::Unlimited => false,
        }
    }

    pub fn reset(&mut self) {
        self.used = 0;
        self.period_start = Utc::now();
    }
}

/// Quota period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaPeriod {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Unlimited,
}

/// Action when quota is exceeded
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaAction {
    /// Just warn, allow request
    Warn,
    /// Block request
    Block,
    /// Throttle requests
    Throttle,
}

/// Token usage aggregation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenAggregation {
    /// Total input tokens
    pub total_input: u64,
    /// Total output tokens
    pub total_output: u64,
    /// Total tokens
    pub total: u64,
    /// Average input per request
    pub avg_input: f64,
    /// Average output per request
    pub avg_output: f64,
    /// Request count
    pub request_count: u64,
    /// By provider
    pub by_provider: HashMap<String, u64>,
    /// By model
    pub by_model: HashMap<String, u64>,
    /// By hour
    pub by_hour: HashMap<u32, u64>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAlert {
    /// Alert identifier
    pub id: String,
    /// Alert name
    pub name: String,
    /// Threshold type
    pub threshold_type: AlertThresholdType,
    /// Threshold value
    pub threshold: f64,
    /// Whether alert is enabled
    pub enabled: bool,
    /// Callback action
    pub action: AlertAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertThresholdType {
    /// Absolute token count
    AbsoluteTokens,
    /// Percentage of quota
    QuotaPercentage,
    /// Tokens per hour rate
    HourlyRate,
    /// Cost in USD
    CostUsd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertAction {
    Log,
    Notify,
    Callback(String),
}

/// Token tracker
pub struct TokenTracker {
    /// Event collector
    collector: Arc<EventCollector>,
    /// Running totals
    total_input: AtomicU64,
    total_output: AtomicU64,
    /// Usage history
    history: Arc<RwLock<Vec<TokenUsage>>>,
    /// Active quotas
    quotas: Arc<RwLock<HashMap<String, TokenQuota>>>,
    /// Alerts
    alerts: Arc<RwLock<Vec<TokenAlert>>>,
    /// Model context limits
    context_limits: Arc<RwLock<HashMap<String, u64>>>,
}

impl TokenTracker {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        let tracker = Self {
            collector,
            total_input: AtomicU64::new(0),
            total_output: AtomicU64::new(0),
            history: Arc::new(RwLock::new(Vec::new())),
            quotas: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            context_limits: Arc::new(RwLock::new(default_context_limits())),
        };

        tracker
    }

    /// Record token usage
    pub async fn record(
        &self,
        request_id: &str,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost_usd: f64,
    ) -> Result<TokenUsageResult, TokenTrackingError> {
        // Update totals
        self.total_input.fetch_add(input_tokens, Ordering::Relaxed);
        self.total_output.fetch_add(output_tokens, Ordering::Relaxed);

        // Get context limit for model
        let context_limit = self
            .context_limits
            .read()
            .await
            .get(model)
            .copied()
            .unwrap_or(0);

        let usage = TokenUsage {
            request_id: request_id.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens,
            output_tokens,
            system_tokens: 0,
            cached_tokens: 0,
            context_used: input_tokens + output_tokens,
            context_limit,
            timestamp: Utc::now(),
            cost_usd,
        };

        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(usage.clone());
        }

        // Update quotas
        let quota_results = self.update_quotas(input_tokens + output_tokens).await;

        // Check alerts
        self.check_alerts(&usage).await;

        // Emit event
        let event = EventBuilder::new(EventType::TokensConsumed)
            .data(EventData::Business(BusinessEventData {
                metric_type: BusinessMetricType::TotalTokens,
                value: (input_tokens + output_tokens) as f64,
                unit: "tokens".to_string(),
                backend: Some(provider.to_string()),
                model: Some(model.to_string()),
            }))
            .custom_metadata("input_tokens", serde_json::json!(input_tokens))
            .custom_metadata("output_tokens", serde_json::json!(output_tokens))
            .custom_metadata("cost_usd", serde_json::json!(cost_usd))
            .build();

        self.collector.collect(event).await.ok();

        Ok(TokenUsageResult {
            usage,
            quota_warnings: quota_results
                .iter()
                .filter(|(_, exceeded)| *exceeded)
                .map(|(id, _)| id.clone())
                .collect(),
        })
    }

    /// Update all quotas with usage
    async fn update_quotas(&self, tokens: u64) -> Vec<(String, bool)> {
        let mut quotas = self.quotas.write().await;
        let mut results = Vec::new();

        for (id, quota) in quotas.iter_mut() {
            if !quota.active {
                continue;
            }

            // Reset if needed
            if quota.needs_reset() {
                quota.reset();
            }

            quota.used += tokens;
            results.push((id.clone(), quota.is_exceeded()));
        }

        results
    }

    /// Check and trigger alerts
    async fn check_alerts(&self, usage: &TokenUsage) {
        let alerts = self.alerts.read().await;
        let total_tokens = self.get_total().await;

        for alert in alerts.iter() {
            if !alert.enabled {
                continue;
            }

            let should_trigger = match alert.threshold_type {
                AlertThresholdType::AbsoluteTokens => {
                    (total_tokens.total_input + total_tokens.total_output) as f64
                        >= alert.threshold
                }
                AlertThresholdType::CostUsd => usage.cost_usd >= alert.threshold,
                _ => false,
            };

            if should_trigger {
                match &alert.action {
                    AlertAction::Log => {
                        tracing::warn!("Token alert triggered: {}", alert.name);
                    }
                    AlertAction::Notify => {
                        // Would integrate with notification system
                        tracing::info!("Token alert notification: {}", alert.name);
                    }
                    AlertAction::Callback(_callback) => {
                        // Would call external callback
                    }
                }
            }
        }
    }

    /// Add a quota
    pub async fn add_quota(&self, quota: TokenQuota) {
        let mut quotas = self.quotas.write().await;
        quotas.insert(quota.id.clone(), quota);
    }

    /// Remove a quota
    pub async fn remove_quota(&self, id: &str) {
        let mut quotas = self.quotas.write().await;
        quotas.remove(id);
    }

    /// Get quota status
    pub async fn get_quota(&self, id: &str) -> Option<TokenQuota> {
        self.quotas.read().await.get(id).cloned()
    }

    /// Add an alert
    pub async fn add_alert(&self, alert: TokenAlert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
    }

    /// Get total usage
    pub async fn get_total(&self) -> TokenAggregation {
        let history = self.history.read().await;

        let mut aggregation = TokenAggregation::default();

        for usage in history.iter() {
            aggregation.total_input += usage.input_tokens;
            aggregation.total_output += usage.output_tokens;
            aggregation.request_count += 1;

            *aggregation
                .by_provider
                .entry(usage.provider.clone())
                .or_insert(0) += usage.total_tokens();

            *aggregation
                .by_model
                .entry(usage.model.clone())
                .or_insert(0) += usage.total_tokens();

            let hour = usage.timestamp.hour();
            *aggregation.by_hour.entry(hour).or_insert(0) += usage.total_tokens();
        }

        aggregation.total = aggregation.total_input + aggregation.total_output;

        if aggregation.request_count > 0 {
            aggregation.avg_input =
                aggregation.total_input as f64 / aggregation.request_count as f64;
            aggregation.avg_output =
                aggregation.total_output as f64 / aggregation.request_count as f64;
        }

        aggregation
    }

    /// Get usage for a time period
    pub async fn get_usage_for_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> TokenAggregation {
        let history = self.history.read().await;

        let filtered: Vec<_> = history
            .iter()
            .filter(|u| u.timestamp >= start && u.timestamp <= end)
            .cloned()
            .collect();

        let mut aggregation = TokenAggregation::default();

        for usage in filtered {
            aggregation.total_input += usage.input_tokens;
            aggregation.total_output += usage.output_tokens;
            aggregation.request_count += 1;
        }

        aggregation.total = aggregation.total_input + aggregation.total_output;

        if aggregation.request_count > 0 {
            aggregation.avg_input =
                aggregation.total_input as f64 / aggregation.request_count as f64;
            aggregation.avg_output =
                aggregation.total_output as f64 / aggregation.request_count as f64;
        }

        aggregation
    }

    /// Forecast usage based on historical data
    pub async fn forecast_usage(&self, hours_ahead: u32) -> TokenForecast {
        let now = Utc::now();
        let history = self.history.read().await;

        // Calculate average hourly rate from last 24 hours
        let day_ago = now - Duration::hours(24);
        let recent: Vec<_> = history
            .iter()
            .filter(|u| u.timestamp >= day_ago)
            .collect();

        let total_recent: u64 = recent.iter().map(|u| u.total_tokens()).sum();
        let hourly_rate = total_recent as f64 / 24.0;

        let forecasted_tokens = (hourly_rate * hours_ahead as f64) as u64;

        // Estimate cost
        let avg_cost_per_token = if total_recent > 0 {
            recent.iter().map(|u| u.cost_usd).sum::<f64>() / total_recent as f64
        } else {
            0.0
        };

        let forecasted_cost = forecasted_tokens as f64 * avg_cost_per_token;

        TokenForecast {
            hours_ahead,
            forecasted_tokens,
            forecasted_cost_usd: forecasted_cost,
            hourly_rate,
            confidence: if recent.len() >= 10 { 0.8 } else { 0.5 },
            generated_at: now,
        }
    }

    /// Set context limit for a model
    pub async fn set_context_limit(&self, model: &str, limit: u64) {
        let mut limits = self.context_limits.write().await;
        limits.insert(model.to_string(), limit);
    }

    /// Check if request would exceed context
    pub async fn would_exceed_context(
        &self,
        model: &str,
        estimated_tokens: u64,
    ) -> bool {
        let limits = self.context_limits.read().await;
        if let Some(limit) = limits.get(model) {
            estimated_tokens > *limit
        } else {
            false
        }
    }
}

/// Default context limits for common models
fn default_context_limits() -> HashMap<String, u64> {
    let mut limits = HashMap::new();

    // Anthropic
    limits.insert("claude-3-opus".to_string(), 200_000);
    limits.insert("claude-3-sonnet".to_string(), 200_000);
    limits.insert("claude-3-haiku".to_string(), 200_000);

    // OpenAI
    limits.insert("gpt-4".to_string(), 8_192);
    limits.insert("gpt-4-32k".to_string(), 32_768);
    limits.insert("gpt-4-turbo".to_string(), 128_000);
    limits.insert("gpt-3.5-turbo".to_string(), 16_385);

    limits
}

/// Result of recording token usage
#[derive(Debug)]
pub struct TokenUsageResult {
    pub usage: TokenUsage,
    pub quota_warnings: Vec<String>,
}

/// Token usage forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenForecast {
    pub hours_ahead: u32,
    pub forecasted_tokens: u64,
    pub forecasted_cost_usd: f64,
    pub hourly_rate: f64,
    pub confidence: f64,
    pub generated_at: DateTime<Utc>,
}

/// Token tracking errors
#[derive(Debug, thiserror::Error)]
pub enum TokenTrackingError {
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Context limit exceeded")]
    ContextLimitExceeded,

    #[error("Collection failed: {0}")]
    CollectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_tracker() -> TokenTracker {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        TokenTracker::new(collector)
    }

    #[tokio::test]
    async fn test_token_recording() {
        let tracker = create_tracker().await;

        let result = tracker
            .record("req-1", "anthropic", "claude-3-opus", 500, 300, 0.05)
            .await
            .unwrap();

        assert_eq!(result.usage.input_tokens, 500);
        assert_eq!(result.usage.output_tokens, 300);
        assert_eq!(result.usage.total_tokens(), 800);
    }

    #[tokio::test]
    async fn test_quota_tracking() {
        let tracker = create_tracker().await;

        let quota = TokenQuota::new("daily", 1000, QuotaPeriod::Daily);
        tracker.add_quota(quota).await;

        // First request - should be fine
        let result = tracker
            .record("req-1", "anthropic", "claude-3-opus", 500, 300, 0.05)
            .await
            .unwrap();

        assert!(result.quota_warnings.is_empty());

        // Second request - should exceed
        let result = tracker
            .record("req-2", "anthropic", "claude-3-opus", 500, 300, 0.05)
            .await
            .unwrap();

        assert!(result.quota_warnings.contains(&"daily".to_string()));
    }

    #[tokio::test]
    async fn test_aggregation() {
        let tracker = create_tracker().await;

        for i in 0..5 {
            tracker
                .record(
                    &format!("req-{}", i),
                    "anthropic",
                    "claude-3-opus",
                    100,
                    50,
                    0.01,
                )
                .await
                .unwrap();
        }

        let aggregation = tracker.get_total().await;

        assert_eq!(aggregation.total_input, 500);
        assert_eq!(aggregation.total_output, 250);
        assert_eq!(aggregation.request_count, 5);
        assert_eq!(aggregation.avg_input, 100.0);
    }

    #[tokio::test]
    async fn test_context_limits() {
        let tracker = create_tracker().await;

        assert!(tracker
            .would_exceed_context("gpt-4", 10_000)
            .await);

        assert!(!tracker
            .would_exceed_context("claude-3-opus", 100_000)
            .await);
    }

    #[tokio::test]
    async fn test_forecasting() {
        let tracker = create_tracker().await;

        // Record some usage
        for i in 0..24 {
            tracker
                .record(
                    &format!("req-{}", i),
                    "anthropic",
                    "claude-3-opus",
                    1000,
                    500,
                    0.05,
                )
                .await
                .unwrap();
        }

        let forecast = tracker.forecast_usage(24).await;

        assert!(forecast.forecasted_tokens > 0);
        assert!(forecast.hourly_rate > 0.0);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Token recording accuracy
   - Quota tracking and reset
   - Aggregation calculations
   - Context limit checks

2. **Integration Tests**
   - Multi-model tracking
   - Alert triggering
   - Forecasting accuracy

3. **Edge Case Tests**
   - Quota edge conditions
   - Context boundary conditions

---

## Related Specs

- Spec 406: Analytics Types
- Spec 415: Backend Analytics
- Spec 417: Cost Tracking
