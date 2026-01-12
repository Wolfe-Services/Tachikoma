# Spec 417: Cost Tracking

## Phase
19 - Analytics/Telemetry

## Spec ID
417

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 416: Token Tracking (token usage)

## Estimated Context
~9%

---

## Objective

Implement comprehensive cost tracking and budget management for LLM usage, enabling accurate financial monitoring, budget enforcement, and cost optimization insights.

---

## Acceptance Criteria

- [ ] Track costs per request/model/provider
- [ ] Support multiple pricing models
- [ ] Implement budget limits and alerts
- [ ] Calculate cost trends and forecasts
- [ ] Support cost allocation to projects
- [ ] Create cost optimization recommendations
- [ ] Enable cost comparison across providers
- [ ] Provide detailed cost breakdowns

---

## Implementation Details

### Cost Tracking

```rust
// src/analytics/costs.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::types::{
    BusinessEventData, BusinessMetricType, EventBuilder, EventData, EventType,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Pricing model for a provider/model combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Price per 1K input tokens (USD)
    pub input_price_per_1k: f64,
    /// Price per 1K output tokens (USD)
    pub output_price_per_1k: f64,
    /// Optional per-request fee
    pub per_request_fee: f64,
    /// Optional minimum charge
    pub minimum_charge: f64,
    /// Effective date of pricing
    pub effective_date: DateTime<Utc>,
    /// Currency (default USD)
    pub currency: String,
}

impl PricingModel {
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_price_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_price_per_1k;
        let total = input_cost + output_cost + self.per_request_fee;

        total.max(self.minimum_charge)
    }
}

/// Default pricing models for common providers
fn default_pricing_models() -> Vec<PricingModel> {
    vec![
        // Anthropic Claude 3 Opus
        PricingModel {
            provider: "anthropic".to_string(),
            model: "claude-3-opus".to_string(),
            input_price_per_1k: 0.015,
            output_price_per_1k: 0.075,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
        // Anthropic Claude 3 Sonnet
        PricingModel {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            input_price_per_1k: 0.003,
            output_price_per_1k: 0.015,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
        // Anthropic Claude 3 Haiku
        PricingModel {
            provider: "anthropic".to_string(),
            model: "claude-3-haiku".to_string(),
            input_price_per_1k: 0.00025,
            output_price_per_1k: 0.00125,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
        // OpenAI GPT-4
        PricingModel {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            input_price_per_1k: 0.03,
            output_price_per_1k: 0.06,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
        // OpenAI GPT-4 Turbo
        PricingModel {
            provider: "openai".to_string(),
            model: "gpt-4-turbo".to_string(),
            input_price_per_1k: 0.01,
            output_price_per_1k: 0.03,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
        // OpenAI GPT-3.5 Turbo
        PricingModel {
            provider: "openai".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            input_price_per_1k: 0.0005,
            output_price_per_1k: 0.0015,
            per_request_fee: 0.0,
            minimum_charge: 0.0,
            effective_date: Utc::now(),
            currency: "USD".to_string(),
        },
    ]
}

/// Cost record for a single request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// Request identifier
    pub request_id: String,
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Input tokens
    pub input_tokens: u64,
    /// Output tokens
    pub output_tokens: u64,
    /// Calculated cost (USD)
    pub cost_usd: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional project/tag for allocation
    pub project: Option<String>,
    /// Mission ID if applicable
    pub mission_id: Option<String>,
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Budget identifier
    pub id: String,
    /// Budget name
    pub name: String,
    /// Budget limit (USD)
    pub limit_usd: f64,
    /// Budget period
    pub period: BudgetPeriod,
    /// Current spend
    pub spent_usd: f64,
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Whether budget is active
    pub active: bool,
    /// Alert thresholds (percentage)
    pub alert_thresholds: Vec<f64>,
    /// Triggered alerts
    pub triggered_alerts: Vec<f64>,
    /// Optional project filter
    pub project: Option<String>,
}

impl Budget {
    pub fn new(id: &str, name: &str, limit_usd: f64, period: BudgetPeriod) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            limit_usd,
            period,
            spent_usd: 0.0,
            period_start: Utc::now(),
            active: true,
            alert_thresholds: vec![0.5, 0.75, 0.9, 1.0],
            triggered_alerts: Vec::new(),
            project: None,
        }
    }

    pub fn remaining(&self) -> f64 {
        (self.limit_usd - self.spent_usd).max(0.0)
    }

    pub fn utilization(&self) -> f64 {
        if self.limit_usd == 0.0 {
            return 0.0;
        }
        self.spent_usd / self.limit_usd
    }

    pub fn is_exceeded(&self) -> bool {
        self.spent_usd >= self.limit_usd
    }

    pub fn needs_reset(&self) -> bool {
        let now = Utc::now();
        match self.period {
            BudgetPeriod::Daily => now - self.period_start >= Duration::days(1),
            BudgetPeriod::Weekly => now - self.period_start >= Duration::weeks(1),
            BudgetPeriod::Monthly => now - self.period_start >= Duration::days(30),
            BudgetPeriod::Quarterly => now - self.period_start >= Duration::days(90),
            BudgetPeriod::Annual => now - self.period_start >= Duration::days(365),
            BudgetPeriod::Unlimited => false,
        }
    }

    pub fn reset(&mut self) {
        self.spent_usd = 0.0;
        self.period_start = Utc::now();
        self.triggered_alerts.clear();
    }

    pub fn check_alerts(&mut self) -> Vec<BudgetAlert> {
        let mut alerts = Vec::new();
        let utilization = self.utilization();

        for threshold in &self.alert_thresholds {
            if utilization >= *threshold && !self.triggered_alerts.contains(threshold) {
                alerts.push(BudgetAlert {
                    budget_id: self.id.clone(),
                    budget_name: self.name.clone(),
                    threshold: *threshold,
                    utilization,
                    spent_usd: self.spent_usd,
                    limit_usd: self.limit_usd,
                    timestamp: Utc::now(),
                });
                self.triggered_alerts.push(*threshold);
            }
        }

        alerts
    }
}

/// Budget period
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetPeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
    Unlimited,
}

/// Budget alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub budget_id: String,
    pub budget_name: String,
    pub threshold: f64,
    pub utilization: f64,
    pub spent_usd: f64,
    pub limit_usd: f64,
    pub timestamp: DateTime<Utc>,
}

/// Cost aggregation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostAggregation {
    /// Total cost (USD)
    pub total_cost_usd: f64,
    /// Request count
    pub request_count: u64,
    /// Average cost per request
    pub avg_cost_per_request: f64,
    /// Cost by provider
    pub by_provider: HashMap<String, f64>,
    /// Cost by model
    pub by_model: HashMap<String, f64>,
    /// Cost by project
    pub by_project: HashMap<String, f64>,
    /// Cost by day
    pub by_day: HashMap<String, f64>,
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Period end
    pub period_end: DateTime<Utc>,
}

/// Cost trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    pub data_points: Vec<(DateTime<Utc>, f64)>,
    pub moving_average: Vec<(DateTime<Utc>, f64)>,
    pub trend_direction: TrendDirection,
    pub trend_percentage: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Cost forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostForecast {
    pub period: BudgetPeriod,
    pub forecasted_cost_usd: f64,
    pub confidence: f64,
    pub based_on_days: u32,
    pub generated_at: DateTime<Utc>,
}

/// Cost tracker
pub struct CostTracker {
    /// Event collector
    collector: Arc<EventCollector>,
    /// Pricing models
    pricing_models: Arc<RwLock<HashMap<String, PricingModel>>>,
    /// Cost history
    history: Arc<RwLock<Vec<CostRecord>>>,
    /// Active budgets
    budgets: Arc<RwLock<HashMap<String, Budget>>>,
    /// Alert callback
    alert_handler: Arc<RwLock<Option<Box<dyn Fn(BudgetAlert) + Send + Sync>>>>,
}

impl CostTracker {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        let mut pricing = HashMap::new();
        for model in default_pricing_models() {
            let key = format!("{}:{}", model.provider, model.model);
            pricing.insert(key, model);
        }

        Self {
            collector,
            pricing_models: Arc::new(RwLock::new(pricing)),
            history: Arc::new(RwLock::new(Vec::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            alert_handler: Arc::new(RwLock::new(None)),
        }
    }

    /// Calculate cost for a request
    pub async fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> f64 {
        let key = format!("{}:{}", provider, model);
        let pricing = self.pricing_models.read().await;

        if let Some(pricing_model) = pricing.get(&key) {
            pricing_model.calculate_cost(input_tokens, output_tokens)
        } else {
            // Default estimation if no pricing model found
            let input_cost = input_tokens as f64 * 0.00001;
            let output_cost = output_tokens as f64 * 0.00003;
            input_cost + output_cost
        }
    }

    /// Record a cost
    pub async fn record(
        &self,
        request_id: &str,
        provider: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        project: Option<&str>,
        mission_id: Option<&str>,
    ) -> Result<CostRecordResult, CostTrackingError> {
        let cost_usd = self
            .calculate_cost(provider, model, input_tokens, output_tokens)
            .await;

        let record = CostRecord {
            request_id: request_id.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens,
            output_tokens,
            cost_usd,
            timestamp: Utc::now(),
            project: project.map(String::from),
            mission_id: mission_id.map(String::from),
        };

        // Add to history
        {
            let mut history = self.history.write().await;
            history.push(record.clone());
        }

        // Update budgets and check alerts
        let alerts = self.update_budgets(cost_usd, project).await;

        // Emit event
        let event = EventBuilder::new(EventType::CostIncurred)
            .data(EventData::Business(BusinessEventData {
                metric_type: BusinessMetricType::CostUsd,
                value: cost_usd,
                unit: "USD".to_string(),
                backend: Some(provider.to_string()),
                model: Some(model.to_string()),
            }))
            .build();

        self.collector.collect(event).await.ok();

        // Handle alerts
        for alert in &alerts {
            if let Some(ref handler) = *self.alert_handler.read().await {
                handler(alert.clone());
            }
        }

        Ok(CostRecordResult {
            record,
            alerts,
            budget_exceeded: alerts.iter().any(|a| a.utilization >= 1.0),
        })
    }

    /// Update budgets with new cost
    async fn update_budgets(
        &self,
        cost: f64,
        project: Option<&str>,
    ) -> Vec<BudgetAlert> {
        let mut budgets = self.budgets.write().await;
        let mut all_alerts = Vec::new();

        for budget in budgets.values_mut() {
            if !budget.active {
                continue;
            }

            // Check project filter
            if let Some(ref budget_project) = budget.project {
                if project != Some(budget_project.as_str()) {
                    continue;
                }
            }

            // Reset if needed
            if budget.needs_reset() {
                budget.reset();
            }

            budget.spent_usd += cost;

            let alerts = budget.check_alerts();
            all_alerts.extend(alerts);
        }

        all_alerts
    }

    /// Add a pricing model
    pub async fn add_pricing_model(&self, model: PricingModel) {
        let key = format!("{}:{}", model.provider, model.model);
        let mut pricing = self.pricing_models.write().await;
        pricing.insert(key, model);
    }

    /// Add a budget
    pub async fn add_budget(&self, budget: Budget) {
        let mut budgets = self.budgets.write().await;
        budgets.insert(budget.id.clone(), budget);
    }

    /// Remove a budget
    pub async fn remove_budget(&self, id: &str) {
        let mut budgets = self.budgets.write().await;
        budgets.remove(id);
    }

    /// Get budget status
    pub async fn get_budget(&self, id: &str) -> Option<Budget> {
        self.budgets.read().await.get(id).cloned()
    }

    /// Get all budgets
    pub async fn get_all_budgets(&self) -> Vec<Budget> {
        self.budgets.read().await.values().cloned().collect()
    }

    /// Set alert handler
    pub async fn set_alert_handler<F>(&self, handler: F)
    where
        F: Fn(BudgetAlert) + Send + Sync + 'static,
    {
        let mut alert_handler = self.alert_handler.write().await;
        *alert_handler = Some(Box::new(handler));
    }

    /// Get cost aggregation for a period
    pub async fn get_aggregation(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> CostAggregation {
        let history = self.history.read().await;

        let filtered: Vec<_> = history
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect();

        let mut aggregation = CostAggregation {
            period_start: start,
            period_end: end,
            ..Default::default()
        };

        for record in filtered {
            aggregation.total_cost_usd += record.cost_usd;
            aggregation.request_count += 1;

            *aggregation
                .by_provider
                .entry(record.provider.clone())
                .or_insert(0.0) += record.cost_usd;

            *aggregation
                .by_model
                .entry(record.model.clone())
                .or_insert(0.0) += record.cost_usd;

            if let Some(ref project) = record.project {
                *aggregation
                    .by_project
                    .entry(project.clone())
                    .or_insert(0.0) += record.cost_usd;
            }

            let day = record.timestamp.format("%Y-%m-%d").to_string();
            *aggregation.by_day.entry(day).or_insert(0.0) += record.cost_usd;
        }

        if aggregation.request_count > 0 {
            aggregation.avg_cost_per_request =
                aggregation.total_cost_usd / aggregation.request_count as f64;
        }

        aggregation
    }

    /// Get cost trend
    pub async fn get_trend(&self, days: u32) -> CostTrend {
        let end = Utc::now();
        let start = end - Duration::days(days as i64);

        let aggregation = self.get_aggregation(start, end).await;

        let mut data_points: Vec<_> = aggregation
            .by_day
            .iter()
            .map(|(day, cost)| {
                let dt = DateTime::parse_from_str(
                    &format!("{} 00:00:00 +0000", day),
                    "%Y-%m-%d %H:%M:%S %z",
                )
                .unwrap()
                .with_timezone(&Utc);
                (dt, *cost)
            })
            .collect();

        data_points.sort_by_key(|(dt, _)| *dt);

        // Calculate 3-day moving average
        let mut moving_average = Vec::new();
        for i in 2..data_points.len() {
            let avg = (data_points[i - 2].1 + data_points[i - 1].1 + data_points[i].1) / 3.0;
            moving_average.push((data_points[i].0, avg));
        }

        // Determine trend
        let (direction, percentage) = if data_points.len() >= 2 {
            let first_half: f64 = data_points[..data_points.len() / 2]
                .iter()
                .map(|(_, c)| c)
                .sum();
            let second_half: f64 = data_points[data_points.len() / 2..]
                .iter()
                .map(|(_, c)| c)
                .sum();

            let change = if first_half > 0.0 {
                (second_half - first_half) / first_half
            } else {
                0.0
            };

            let direction = if change > 0.1 {
                TrendDirection::Increasing
            } else if change < -0.1 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            };

            (direction, change * 100.0)
        } else {
            (TrendDirection::Stable, 0.0)
        };

        CostTrend {
            data_points,
            moving_average,
            trend_direction: direction,
            trend_percentage: percentage,
        }
    }

    /// Forecast costs
    pub async fn forecast(&self, period: BudgetPeriod) -> CostForecast {
        let now = Utc::now();
        let days_back = 30;
        let start = now - Duration::days(days_back);

        let aggregation = self.get_aggregation(start, now).await;
        let daily_average = aggregation.total_cost_usd / days_back as f64;

        let days_ahead = match period {
            BudgetPeriod::Daily => 1,
            BudgetPeriod::Weekly => 7,
            BudgetPeriod::Monthly => 30,
            BudgetPeriod::Quarterly => 90,
            BudgetPeriod::Annual => 365,
            BudgetPeriod::Unlimited => 30,
        };

        let forecasted = daily_average * days_ahead as f64;
        let confidence = if aggregation.request_count > 100 { 0.8 } else { 0.5 };

        CostForecast {
            period,
            forecasted_cost_usd: forecasted,
            confidence,
            based_on_days: days_back as u32,
            generated_at: now,
        }
    }

    /// Compare costs across providers for same tokens
    pub async fn compare_providers(
        &self,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Vec<ProviderComparison> {
        let pricing = self.pricing_models.read().await;

        let mut comparisons: Vec<_> = pricing
            .values()
            .map(|model| {
                let cost = model.calculate_cost(input_tokens, output_tokens);
                ProviderComparison {
                    provider: model.provider.clone(),
                    model: model.model.clone(),
                    cost_usd: cost,
                    input_tokens,
                    output_tokens,
                }
            })
            .collect();

        comparisons.sort_by(|a, b| {
            a.cost_usd
                .partial_cmp(&b.cost_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        comparisons
    }
}

/// Result of recording a cost
#[derive(Debug)]
pub struct CostRecordResult {
    pub record: CostRecord,
    pub alerts: Vec<BudgetAlert>,
    pub budget_exceeded: bool,
}

/// Provider cost comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderComparison {
    pub provider: String,
    pub model: String,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Cost tracking errors
#[derive(Debug, thiserror::Error)]
pub enum CostTrackingError {
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("Unknown pricing model")]
    UnknownPricingModel,

    #[error("Collection failed: {0}")]
    CollectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_tracker() -> CostTracker {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        CostTracker::new(collector)
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let tracker = create_tracker().await;

        let cost = tracker
            .calculate_cost("anthropic", "claude-3-opus", 1000, 500)
            .await;

        // 1000 input * 0.015/1000 + 500 output * 0.075/1000 = 0.015 + 0.0375 = 0.0525
        assert!((cost - 0.0525).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cost_recording() {
        let tracker = create_tracker().await;

        let result = tracker
            .record(
                "req-1",
                "anthropic",
                "claude-3-opus",
                1000,
                500,
                Some("project-1"),
                None,
            )
            .await
            .unwrap();

        assert!(result.record.cost_usd > 0.0);
        assert_eq!(result.record.project, Some("project-1".to_string()));
    }

    #[tokio::test]
    async fn test_budget_alerts() {
        let tracker = create_tracker().await;

        let budget = Budget::new("test", "Test Budget", 0.10, BudgetPeriod::Daily);
        tracker.add_budget(budget).await;

        // Record costs that will trigger alerts
        for i in 0..5 {
            tracker
                .record(
                    &format!("req-{}", i),
                    "anthropic",
                    "claude-3-opus",
                    1000,
                    500,
                    None,
                    None,
                )
                .await
                .unwrap();
        }

        let budget = tracker.get_budget("test").await.unwrap();
        assert!(budget.utilization() > 0.5);
    }

    #[tokio::test]
    async fn test_provider_comparison() {
        let tracker = create_tracker().await;

        let comparisons = tracker.compare_providers(10000, 5000).await;

        assert!(!comparisons.is_empty());

        // Should be sorted by cost
        for i in 1..comparisons.len() {
            assert!(comparisons[i - 1].cost_usd <= comparisons[i].cost_usd);
        }
    }

    #[tokio::test]
    async fn test_aggregation() {
        let tracker = create_tracker().await;

        for i in 0..10 {
            tracker
                .record(
                    &format!("req-{}", i),
                    "anthropic",
                    "claude-3-opus",
                    1000,
                    500,
                    Some("project-1"),
                    None,
                )
                .await
                .unwrap();
        }

        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now() + Duration::hours(1);
        let aggregation = tracker.get_aggregation(start, end).await;

        assert_eq!(aggregation.request_count, 10);
        assert!(aggregation.total_cost_usd > 0.0);
        assert!(aggregation.by_project.contains_key("project-1"));
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Cost calculation accuracy
   - Budget tracking and reset
   - Alert threshold triggering
   - Aggregation calculations

2. **Integration Tests**
   - Multi-provider cost tracking
   - Budget overflow handling
   - Trend analysis accuracy

3. **Financial Tests**
   - Pricing model accuracy
   - Forecast reliability

---

## Related Specs

- Spec 406: Analytics Types
- Spec 416: Token Tracking
- Spec 420: Trend Analysis
