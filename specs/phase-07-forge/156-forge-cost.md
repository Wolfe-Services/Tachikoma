# 156 - Forge Cost Tracking

**Phase:** 7 - Spec Forge Multi-Model Brainstorming
**Spec ID:** 156
**Status:** Planned
**Dependencies:** 138-forge-participants, 137-forge-config
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement comprehensive cost tracking for Forge sessions, including per-model costs, budget enforcement, cost projections, and spending alerts.

---

## Acceptance Criteria

- [ ] Per-model cost calculation
- [ ] Real-time cost accumulation
- [ ] Budget limit enforcement
- [ ] Cost projection/estimation
- [ ] Spending alerts and warnings
- [ ] Cost breakdown reporting

---

## Implementation Details

### 1. Cost Tracker (src/cost/tracker.rs)

```rust
//! Cost tracking for Forge sessions.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::{
    ForgeConfig, ForgeError, ForgeEvent, ForgeResult, ModelConfig, ModelProvider,
    Participant, TokenCount,
};

/// Tracks costs for a Forge session.
pub struct CostTracker {
    config: ForgeConfig,
    /// Total accumulated cost.
    total_cost: Arc<RwLock<f64>>,
    /// Cost by model.
    cost_by_model: Arc<RwLock<HashMap<String, f64>>>,
    /// Cost by round.
    cost_by_round: Arc<RwLock<Vec<f64>>>,
    /// Token usage by model.
    tokens_by_model: Arc<RwLock<HashMap<String, TokenCount>>>,
    /// Event sender for alerts.
    event_tx: broadcast::Sender<ForgeEvent>,
    /// Alert thresholds hit.
    alerts_sent: Arc<RwLock<AlertState>>,
}

/// State of cost alerts.
#[derive(Debug, Clone, Default)]
struct AlertState {
    warned_50: bool,
    warned_75: bool,
    warned_90: bool,
    warned_100: bool,
}

/// Cost breakdown for a session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CostBreakdown {
    /// Total cost.
    pub total_cost_usd: f64,
    /// Budget limit.
    pub budget_limit_usd: f64,
    /// Budget remaining.
    pub budget_remaining_usd: f64,
    /// Percentage used.
    pub budget_used_percent: f64,
    /// Cost by model.
    pub by_model: HashMap<String, ModelCost>,
    /// Cost by round.
    pub by_round: Vec<RoundCost>,
    /// Projected total cost.
    pub projected_total: Option<f64>,
}

/// Cost for a specific model.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelCost {
    /// Model name.
    pub model_name: String,
    /// Provider.
    pub provider: String,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Input tokens used.
    pub input_tokens: u64,
    /// Output tokens used.
    pub output_tokens: u64,
    /// Percentage of total cost.
    pub percent_of_total: f64,
}

/// Cost for a round.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoundCost {
    /// Round number.
    pub round_number: usize,
    /// Round type.
    pub round_type: String,
    /// Cost in USD.
    pub cost_usd: f64,
    /// Running total after this round.
    pub running_total: f64,
}

impl CostTracker {
    /// Create a new cost tracker.
    pub fn new(config: ForgeConfig, event_tx: broadcast::Sender<ForgeEvent>) -> Self {
        Self {
            config,
            total_cost: Arc::new(RwLock::new(0.0)),
            cost_by_model: Arc::new(RwLock::new(HashMap::new())),
            cost_by_round: Arc::new(RwLock::new(Vec::new())),
            tokens_by_model: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            alerts_sent: Arc::new(RwLock::new(AlertState::default())),
        }
    }

    /// Record cost for a model response.
    pub async fn record_cost(
        &self,
        participant: &Participant,
        tokens: &TokenCount,
    ) -> ForgeResult<f64> {
        // Find model config
        let model_config = self.find_model_config(&participant.model_id);

        // Calculate cost
        let cost = if let Some(config) = model_config {
            config.calculate_cost(tokens.input, tokens.output)
        } else {
            // Default pricing if model not found
            self.estimate_cost(participant.provider, tokens)
        };

        // Update totals
        {
            let mut total = self.total_cost.write().await;
            *total += cost;
        }

        // Update by model
        {
            let mut by_model = self.cost_by_model.write().await;
            *by_model.entry(participant.model_id.clone()).or_insert(0.0) += cost;
        }

        // Update token counts
        {
            let mut tokens_map = self.tokens_by_model.write().await;
            let entry = tokens_map.entry(participant.model_id.clone())
                .or_insert_with(TokenCount::default);
            entry.add(tokens);
        }

        // Check alerts
        self.check_and_send_alerts().await;

        // Send cost update event
        let total = *self.total_cost.read().await;
        let remaining = self.config.limits.max_cost_usd - total;

        let _ = self.event_tx.send(ForgeEvent::CostUpdate {
            total_cost: total,
            budget_remaining: remaining.max(0.0),
        });

        Ok(cost)
    }

    /// Record cost for a complete round.
    pub async fn record_round_cost(&self, round_number: usize, round_type: &str) {
        let total = *self.total_cost.read().await;

        let mut by_round = self.cost_by_round.write().await;

        // Calculate round cost as difference from previous
        let previous_total: f64 = by_round.iter().map(|c| c).sum();
        let round_cost = total - previous_total;

        by_round.push(round_cost);
    }

    /// Check if budget is exceeded.
    pub async fn is_budget_exceeded(&self) -> bool {
        let total = *self.total_cost.read().await;
        total >= self.config.limits.max_cost_usd
    }

    /// Check if near budget limit.
    pub async fn is_near_budget_limit(&self) -> bool {
        let total = *self.total_cost.read().await;
        let threshold = self.config.limits.max_cost_usd * self.config.limits.cost_warning_threshold;
        total >= threshold
    }

    /// Get current total cost.
    pub async fn total_cost(&self) -> f64 {
        *self.total_cost.read().await
    }

    /// Get remaining budget.
    pub async fn remaining_budget(&self) -> f64 {
        let total = *self.total_cost.read().await;
        (self.config.limits.max_cost_usd - total).max(0.0)
    }

    /// Get cost breakdown.
    pub async fn get_breakdown(&self, rounds_completed: usize, max_rounds: usize) -> CostBreakdown {
        let total = *self.total_cost.read().await;
        let by_model_raw = self.cost_by_model.read().await;
        let tokens_map = self.tokens_by_model.read().await;
        let rounds = self.cost_by_round.read().await;

        // Build model costs
        let by_model: HashMap<String, ModelCost> = by_model_raw.iter()
            .map(|(model_id, cost)| {
                let tokens = tokens_map.get(model_id).cloned().unwrap_or_default();
                let model_config = self.find_model_config(model_id);

                (model_id.clone(), ModelCost {
                    model_name: model_config
                        .map(|c| c.display_name.clone())
                        .unwrap_or_else(|| model_id.clone()),
                    provider: model_config
                        .map(|c| format!("{:?}", c.provider))
                        .unwrap_or_else(|| "Unknown".to_string()),
                    cost_usd: *cost,
                    input_tokens: tokens.input,
                    output_tokens: tokens.output,
                    percent_of_total: if total > 0.0 { cost / total * 100.0 } else { 0.0 },
                })
            })
            .collect();

        // Build round costs
        let mut running_total = 0.0;
        let by_round: Vec<RoundCost> = rounds.iter()
            .enumerate()
            .map(|(i, cost)| {
                running_total += cost;
                RoundCost {
                    round_number: i,
                    round_type: "unknown".to_string(), // Would be filled from session
                    cost_usd: *cost,
                    running_total,
                }
            })
            .collect();

        // Project total cost
        let projected_total = if rounds_completed > 0 && rounds_completed < max_rounds {
            let avg_cost_per_round = total / rounds_completed as f64;
            let remaining_rounds = max_rounds - rounds_completed;
            Some(total + (avg_cost_per_round * remaining_rounds as f64))
        } else {
            None
        };

        CostBreakdown {
            total_cost_usd: total,
            budget_limit_usd: self.config.limits.max_cost_usd,
            budget_remaining_usd: (self.config.limits.max_cost_usd - total).max(0.0),
            budget_used_percent: (total / self.config.limits.max_cost_usd) * 100.0,
            by_model,
            by_round,
            projected_total,
        }
    }

    /// Estimate cost for given rounds.
    pub async fn estimate_remaining_cost(&self, remaining_rounds: usize) -> f64 {
        let total = *self.total_cost.read().await;
        let rounds = self.cost_by_round.read().await;

        if rounds.is_empty() {
            // Use default estimate
            return remaining_rounds as f64 * 0.50; // $0.50 per round default
        }

        let avg_cost: f64 = rounds.iter().sum::<f64>() / rounds.len() as f64;
        avg_cost * remaining_rounds as f64
    }

    /// Check and send cost alerts.
    async fn check_and_send_alerts(&self) {
        let total = *self.total_cost.read().await;
        let percent_used = (total / self.config.limits.max_cost_usd) * 100.0;
        let mut alerts = self.alerts_sent.write().await;

        if percent_used >= 100.0 && !alerts.warned_100 {
            alerts.warned_100 = true;
            let _ = self.event_tx.send(ForgeEvent::Error {
                message: format!(
                    "BUDGET EXCEEDED: ${:.2} of ${:.2} limit (100%)",
                    total,
                    self.config.limits.max_cost_usd
                ),
                recoverable: false,
            });
        } else if percent_used >= 90.0 && !alerts.warned_90 {
            alerts.warned_90 = true;
            let _ = self.event_tx.send(ForgeEvent::Error {
                message: format!(
                    "Cost warning: ${:.2} of ${:.2} limit (90%)",
                    total,
                    self.config.limits.max_cost_usd
                ),
                recoverable: true,
            });
        } else if percent_used >= 75.0 && !alerts.warned_75 {
            alerts.warned_75 = true;
            let _ = self.event_tx.send(ForgeEvent::Error {
                message: format!(
                    "Cost notice: ${:.2} of ${:.2} limit (75%)",
                    total,
                    self.config.limits.max_cost_usd
                ),
                recoverable: true,
            });
        } else if percent_used >= 50.0 && !alerts.warned_50 {
            alerts.warned_50 = true;
            let _ = self.event_tx.send(ForgeEvent::Error {
                message: format!(
                    "Cost checkpoint: ${:.2} of ${:.2} limit (50%)",
                    total,
                    self.config.limits.max_cost_usd
                ),
                recoverable: true,
            });
        }
    }

    /// Find model configuration.
    fn find_model_config(&self, model_id: &str) -> Option<&ModelConfig> {
        self.config.models.available.values()
            .find(|m| m.model_id == model_id)
    }

    /// Estimate cost using default provider pricing.
    fn estimate_cost(&self, provider: ModelProvider, tokens: &TokenCount) -> f64 {
        let (input_rate, output_rate) = match provider {
            ModelProvider::Anthropic => (0.008, 0.024),
            ModelProvider::OpenAI => (0.01, 0.03),
            ModelProvider::Google => (0.001, 0.002),
            ModelProvider::Local => (0.0, 0.0),
            ModelProvider::Custom => (0.01, 0.03),
        };

        (tokens.input as f64 / 1000.0) * input_rate
            + (tokens.output as f64 / 1000.0) * output_rate
    }
}

/// Budget guard that checks cost before operations.
pub struct BudgetGuard<'a> {
    tracker: &'a CostTracker,
}

impl<'a> BudgetGuard<'a> {
    pub fn new(tracker: &'a CostTracker) -> Self {
        Self { tracker }
    }

    /// Check if operation can proceed within budget.
    pub async fn check_can_proceed(&self, estimated_cost: f64) -> ForgeResult<()> {
        let remaining = self.tracker.remaining_budget().await;

        if remaining < estimated_cost {
            return Err(ForgeError::Cost(format!(
                "Insufficient budget: ${:.2} remaining, ${:.2} estimated needed",
                remaining,
                estimated_cost
            )));
        }

        Ok(())
    }

    /// Check budget status.
    pub async fn status(&self) -> BudgetStatus {
        let total = self.tracker.total_cost().await;
        let limit = self.tracker.config.limits.max_cost_usd;
        let percent = (total / limit) * 100.0;

        if percent >= 100.0 {
            BudgetStatus::Exceeded
        } else if percent >= 90.0 {
            BudgetStatus::Critical
        } else if percent >= 75.0 {
            BudgetStatus::Warning
        } else {
            BudgetStatus::Ok
        }
    }
}

/// Budget status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetStatus {
    Ok,
    Warning,
    Critical,
    Exceeded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cost_tracking() {
        let config = ForgeConfig::default();
        let (event_tx, _) = broadcast::channel(10);
        let tracker = CostTracker::new(config, event_tx);

        let participant = Participant::claude_sonnet();
        let tokens = TokenCount { input: 1000, output: 500 };

        let cost = tracker.record_cost(&participant, &tokens).await.unwrap();

        assert!(cost > 0.0);
        assert_eq!(tracker.total_cost().await, cost);
    }

    #[tokio::test]
    async fn test_budget_exceeded() {
        let mut config = ForgeConfig::default();
        config.limits.max_cost_usd = 0.01;
        let (event_tx, _) = broadcast::channel(10);
        let tracker = CostTracker::new(config, event_tx);

        let participant = Participant::claude_sonnet();
        let tokens = TokenCount { input: 10000, output: 5000 };

        tracker.record_cost(&participant, &tokens).await.unwrap();

        assert!(tracker.is_budget_exceeded().await);
    }
}
```

---

## Testing Requirements

1. Cost calculation is accurate per model
2. Budget alerts fire at correct thresholds
3. Budget exceeded blocks further operations
4. Cost breakdown sums correctly
5. Projections are reasonable
6. Unknown models use default pricing

---

## Related Specs

- Depends on: [138-forge-participants.md](138-forge-participants.md)
- Depends on: [137-forge-config.md](137-forge-config.md)
- Next: [157-forge-quality.md](157-forge-quality.md)
- Used by: [139-forge-rounds.md](139-forge-rounds.md)
