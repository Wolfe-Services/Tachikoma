# 396 - Percentage Rollout

## Overview

Implementation of percentage-based feature rollouts with consistent hashing for stable user assignment.

## Rust Implementation

```rust
// crates/flags/src/rollout.rs

use crate::types::{FlagId, RolloutConfig};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Percentage rollout manager
pub struct RolloutManager {
    /// Custom salts per flag (for re-bucketing)
    flag_salts: HashMap<FlagId, String>,
}

impl RolloutManager {
    pub fn new() -> Self {
        Self {
            flag_salts: HashMap::new(),
        }
    }

    /// Set custom salt for a flag (used for re-bucketing users)
    pub fn set_salt(&mut self, flag_id: &FlagId, salt: &str) {
        self.flag_salts.insert(flag_id.clone(), salt.to_string());
    }

    /// Check if a user is in the rollout percentage
    pub fn is_in_rollout(
        &self,
        flag_id: &FlagId,
        bucket_key: &str,
        config: &RolloutConfig,
    ) -> bool {
        let percentage = self.calculate_bucket(flag_id, bucket_key, config.seed);
        percentage <= config.percentage
    }

    /// Get the bucket percentage for a user (0-100)
    pub fn calculate_bucket(
        &self,
        flag_id: &FlagId,
        bucket_key: &str,
        seed: Option<u64>,
    ) -> f64 {
        let salt = self.flag_salts.get(flag_id)
            .map(|s| s.as_str())
            .unwrap_or("");

        let input = format!("{}:{}:{}", flag_id.as_str(), bucket_key, salt);
        hash_to_percentage(&input, seed)
    }

    /// Get assigned bucket for staged rollout
    pub fn get_rollout_stage(
        &self,
        flag_id: &FlagId,
        bucket_key: &str,
        stages: &[f64],
    ) -> Option<usize> {
        let percentage = self.calculate_bucket(flag_id, bucket_key, None);

        let mut cumulative = 0.0;
        for (i, &stage) in stages.iter().enumerate() {
            cumulative += stage;
            if percentage <= cumulative {
                return Some(i);
            }
        }

        None
    }
}

impl Default for RolloutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash a string to a percentage (0-100) using SHA256
pub fn hash_to_percentage(input: &str, seed: Option<u64>) -> f64 {
    let mut hasher = Sha256::new();

    // Add seed if provided
    if let Some(s) = seed {
        hasher.update(s.to_le_bytes());
    }

    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    // Use first 8 bytes as u64
    let bytes: [u8; 8] = result[..8].try_into().unwrap();
    let hash_int = u64::from_le_bytes(bytes);

    // Convert to percentage (0-100)
    (hash_int as f64 / u64::MAX as f64) * 100.0
}

/// Staged rollout configuration
#[derive(Debug, Clone)]
pub struct StagedRollout {
    /// Current percentage being rolled out
    pub current_percentage: f64,
    /// Target percentage
    pub target_percentage: f64,
    /// Increment per stage
    pub increment: f64,
    /// Minutes between stages
    pub stage_interval_minutes: u64,
    /// Auto-rollback on error threshold
    pub error_threshold_percent: Option<f64>,
}

impl StagedRollout {
    pub fn new(target: f64) -> Self {
        Self {
            current_percentage: 0.0,
            target_percentage: target,
            increment: 10.0,
            stage_interval_minutes: 60,
            error_threshold_percent: Some(5.0),
        }
    }

    pub fn with_increment(mut self, increment: f64) -> Self {
        self.increment = increment;
        self
    }

    pub fn with_interval(mut self, minutes: u64) -> Self {
        self.stage_interval_minutes = minutes;
        self
    }

    pub fn advance(&mut self) -> f64 {
        self.current_percentage = (self.current_percentage + self.increment)
            .min(self.target_percentage);
        self.current_percentage
    }

    pub fn rollback(&mut self) {
        self.current_percentage = (self.current_percentage - self.increment).max(0.0);
    }

    pub fn is_complete(&self) -> bool {
        self.current_percentage >= self.target_percentage
    }
}

/// Ring-based rollout for more granular control
#[derive(Debug, Clone)]
pub struct RingRollout {
    /// Rings with their percentages (must sum to 100)
    pub rings: Vec<RolloutRing>,
}

#[derive(Debug, Clone)]
pub struct RolloutRing {
    /// Ring name (e.g., "internal", "beta", "canary", "general")
    pub name: String,
    /// Percentage of users in this ring
    pub percentage: f64,
    /// Whether this ring is enabled
    pub enabled: bool,
}

impl RingRollout {
    pub fn new() -> Self {
        Self { rings: vec![] }
    }

    pub fn add_ring(mut self, name: &str, percentage: f64) -> Self {
        self.rings.push(RolloutRing {
            name: name.to_string(),
            percentage,
            enabled: false,
        });
        self
    }

    pub fn enable_ring(&mut self, name: &str) {
        if let Some(ring) = self.rings.iter_mut().find(|r| r.name == name) {
            ring.enabled = true;
        }
    }

    pub fn disable_ring(&mut self, name: &str) {
        if let Some(ring) = self.rings.iter_mut().find(|r| r.name == name) {
            ring.enabled = false;
        }
    }

    /// Get the effective rollout percentage
    pub fn effective_percentage(&self) -> f64 {
        self.rings.iter()
            .filter(|r| r.enabled)
            .map(|r| r.percentage)
            .sum()
    }

    /// Check if a user is in an enabled ring
    pub fn is_user_in_rollout(&self, user_bucket: f64) -> Option<String> {
        let mut cumulative = 0.0;

        for ring in &self.rings {
            cumulative += ring.percentage;
            if user_bucket <= cumulative {
                return if ring.enabled {
                    Some(ring.name.clone())
                } else {
                    None
                };
            }
        }

        None
    }
}

impl Default for RingRollout {
    fn default() -> Self {
        Self::new()
    }
}

/// Schedule-based rollout
#[derive(Debug, Clone)]
pub struct ScheduledRollout {
    pub stages: Vec<RolloutStage>,
}

#[derive(Debug, Clone)]
pub struct RolloutStage {
    /// Percentage at this stage
    pub percentage: f64,
    /// When this stage activates (Unix timestamp)
    pub activate_at: i64,
    /// Description of the stage
    pub description: Option<String>,
}

impl ScheduledRollout {
    pub fn new() -> Self {
        Self { stages: vec![] }
    }

    pub fn add_stage(mut self, percentage: f64, activate_at: i64) -> Self {
        self.stages.push(RolloutStage {
            percentage,
            activate_at,
            description: None,
        });
        self.stages.sort_by_key(|s| s.activate_at);
        self
    }

    /// Get the current effective percentage based on time
    pub fn current_percentage(&self, now: i64) -> f64 {
        let mut current = 0.0;

        for stage in &self.stages {
            if stage.activate_at <= now {
                current = stage.percentage;
            } else {
                break;
            }
        }

        current
    }
}

impl Default for ScheduledRollout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let p1 = hash_to_percentage("flag:user123", None);
        let p2 = hash_to_percentage("flag:user123", None);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_hash_distribution() {
        // Test that hashing produces roughly uniform distribution
        let mut buckets = [0u32; 10];

        for i in 0..10000 {
            let input = format!("test:user{}", i);
            let percentage = hash_to_percentage(&input, None);
            let bucket = (percentage / 10.0).floor() as usize;
            if bucket < 10 {
                buckets[bucket] += 1;
            }
        }

        // Each bucket should have roughly 1000 users (allow 20% variance)
        for bucket in &buckets {
            assert!(*bucket > 800 && *bucket < 1200, "Bucket has {} users", bucket);
        }
    }

    #[test]
    fn test_rollout_manager() {
        let manager = RolloutManager::new();
        let flag_id = FlagId::new("test-flag");
        let config = RolloutConfig {
            percentage: 50.0,
            bucket_by: "user_id".to_string(),
            seed: None,
        };

        let mut in_rollout = 0;
        for i in 0..1000 {
            if manager.is_in_rollout(&flag_id, &format!("user{}", i), &config) {
                in_rollout += 1;
            }
        }

        // Should be roughly 50% (allow 10% variance)
        assert!(in_rollout > 400 && in_rollout < 600, "In rollout: {}", in_rollout);
    }

    #[test]
    fn test_staged_rollout() {
        let mut rollout = StagedRollout::new(100.0)
            .with_increment(25.0);

        assert_eq!(rollout.current_percentage, 0.0);

        rollout.advance();
        assert_eq!(rollout.current_percentage, 25.0);

        rollout.advance();
        assert_eq!(rollout.current_percentage, 50.0);

        rollout.rollback();
        assert_eq!(rollout.current_percentage, 25.0);
    }

    #[test]
    fn test_ring_rollout() {
        let mut rollout = RingRollout::new()
            .add_ring("internal", 5.0)
            .add_ring("beta", 15.0)
            .add_ring("general", 80.0);

        rollout.enable_ring("internal");
        assert_eq!(rollout.effective_percentage(), 5.0);

        rollout.enable_ring("beta");
        assert_eq!(rollout.effective_percentage(), 20.0);

        // User at 3% bucket should be in internal ring
        assert_eq!(rollout.is_user_in_rollout(3.0), Some("internal".to_string()));

        // User at 10% bucket should be in beta ring
        assert_eq!(rollout.is_user_in_rollout(10.0), Some("beta".to_string()));

        // User at 50% bucket should not be in rollout (general not enabled)
        assert_eq!(rollout.is_user_in_rollout(50.0), None);
    }

    #[test]
    fn test_scheduled_rollout() {
        let rollout = ScheduledRollout::new()
            .add_stage(10.0, 1000)
            .add_stage(50.0, 2000)
            .add_stage(100.0, 3000);

        assert_eq!(rollout.current_percentage(500), 0.0);
        assert_eq!(rollout.current_percentage(1500), 10.0);
        assert_eq!(rollout.current_percentage(2500), 50.0);
        assert_eq!(rollout.current_percentage(3500), 100.0);
    }
}
```

## Rollout Strategies

### Gradual Percentage Increase
```yaml
strategy: percentage
config:
  start: 0
  target: 100
  increment: 10
  interval: 1h
  auto_advance: true
  pause_on_error_rate: 0.05
```

### Ring-Based Deployment
```yaml
strategy: ring
rings:
  - name: internal
    percentage: 2
    criteria:
      - email ends_with "@company.com"
  - name: beta
    percentage: 10
    criteria:
      - user.plan equals "beta"
  - name: canary
    percentage: 5
  - name: general
    percentage: 83
```

### Time-Scheduled Rollout
```yaml
strategy: scheduled
stages:
  - percentage: 10
    at: "2024-01-15T00:00:00Z"
  - percentage: 50
    at: "2024-01-16T00:00:00Z"
  - percentage: 100
    at: "2024-01-17T00:00:00Z"
```

## Related Specs

- 394-flag-evaluation.md - Evaluation engine
- 399-ab-testing.md - A/B testing (variant selection)
- 406-flag-analytics.md - Rollout monitoring
