# 408 - Feature Flag Deprecation

## Overview

Lifecycle management for feature flags including deprecation, sunset scheduling, and cleanup processes.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/deprecation.rs

use crate::definition::FlagDefinition;
use crate::storage::{FlagStorage, QueryOptions};
use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Deprecation status for a flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    /// When the flag was marked for deprecation
    pub deprecated_at: DateTime<Utc>,
    /// Who marked it for deprecation
    pub deprecated_by: String,
    /// Reason for deprecation
    pub reason: String,
    /// Planned sunset date
    pub sunset_date: DateTime<Utc>,
    /// Replacement flag (if any)
    pub replacement_flag: Option<String>,
    /// Migration instructions
    pub migration_guide: Option<String>,
    /// Code locations still using this flag
    pub usage_locations: Vec<UsageLocation>,
    /// Last evaluation timestamp
    pub last_evaluated: Option<DateTime<Utc>>,
    /// Days since last evaluation
    pub days_inactive: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLocation {
    /// Repository
    pub repository: String,
    /// File path
    pub file_path: String,
    /// Line number
    pub line_number: u32,
    /// Code snippet
    pub snippet: Option<String>,
    /// Last detected
    pub detected_at: DateTime<Utc>,
}

/// Flag deprecation manager
pub struct DeprecationManager {
    storage: Arc<dyn FlagStorage>,
    notification_service: Arc<dyn DeprecationNotifier>,
    config: DeprecationConfig,
}

/// Configuration for deprecation policies
#[derive(Debug, Clone)]
pub struct DeprecationConfig {
    /// Days before sunset to start warning
    pub warning_period_days: i64,
    /// Days of inactivity to suggest deprecation
    pub inactivity_threshold_days: i64,
    /// Automatically archive after sunset
    pub auto_archive: bool,
    /// Days after sunset to auto-archive
    pub auto_archive_delay_days: i64,
    /// Require removal confirmation
    pub require_confirmation: bool,
}

impl Default for DeprecationConfig {
    fn default() -> Self {
        Self {
            warning_period_days: 14,
            inactivity_threshold_days: 90,
            auto_archive: true,
            auto_archive_delay_days: 7,
            require_confirmation: true,
        }
    }
}

/// Notification interface for deprecation events
#[async_trait::async_trait]
pub trait DeprecationNotifier: Send + Sync {
    /// Notify about upcoming sunset
    async fn notify_upcoming_sunset(&self, flags: Vec<SunsetNotification>);

    /// Notify about inactive flags
    async fn notify_inactive_flags(&self, flags: Vec<InactiveNotification>);

    /// Notify about successful cleanup
    async fn notify_cleanup_complete(&self, results: CleanupResults);
}

#[derive(Debug, Clone)]
pub struct SunsetNotification {
    pub flag_id: String,
    pub flag_name: String,
    pub sunset_date: DateTime<Utc>,
    pub days_remaining: i64,
    pub owner: Option<String>,
    pub usage_count: usize,
}

#[derive(Debug, Clone)]
pub struct InactiveNotification {
    pub flag_id: String,
    pub flag_name: String,
    pub days_inactive: i64,
    pub last_evaluated: Option<DateTime<Utc>>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CleanupResults {
    pub archived_flags: Vec<String>,
    pub deleted_flags: Vec<String>,
    pub errors: Vec<String>,
}

impl DeprecationManager {
    pub fn new(
        storage: Arc<dyn FlagStorage>,
        notification_service: Arc<dyn DeprecationNotifier>,
        config: DeprecationConfig,
    ) -> Self {
        Self {
            storage,
            notification_service,
            config,
        }
    }

    /// Mark a flag for deprecation
    pub async fn deprecate_flag(
        &self,
        flag_id: &FlagId,
        reason: &str,
        sunset_date: DateTime<Utc>,
        deprecated_by: &str,
        replacement: Option<&str>,
    ) -> Result<FlagDefinition, DeprecationError> {
        let stored = self.storage.get(flag_id).await
            .map_err(|e| DeprecationError::Storage(e.to_string()))?
            .ok_or_else(|| DeprecationError::FlagNotFound(flag_id.as_str().to_string()))?;

        let mut flag = stored.definition;

        // Update flag status and metadata
        flag.status = FlagStatus::Deprecated;
        flag.metadata.sunset_date = Some(sunset_date);
        flag.metadata.updated_at = Utc::now();
        flag.metadata.updated_by = deprecated_by.to_string();

        // Add deprecation tag
        if !flag.metadata.tags.contains(&"deprecated".to_string()) {
            flag.metadata.tags.push("deprecated".to_string());
        }

        self.storage.update(flag.clone(), None).await
            .map_err(|e| DeprecationError::Storage(e.to_string()))?;

        Ok(flag)
    }

    /// Get all deprecated flags
    pub async fn get_deprecated_flags(&self) -> Result<Vec<FlagDefinition>, DeprecationError> {
        let flags = self.storage.list(QueryOptions {
            status: Some(FlagStatus::Deprecated),
            include_archived: false,
            limit: 1000,
            ..Default::default()
        }).await
            .map_err(|e| DeprecationError::Storage(e.to_string()))?;

        Ok(flags.into_iter().map(|f| f.definition).collect())
    }

    /// Get flags approaching sunset
    pub async fn get_approaching_sunset(&self) -> Result<Vec<SunsetNotification>, DeprecationError> {
        let deprecated = self.get_deprecated_flags().await?;
        let now = Utc::now();
        let warning_threshold = now + Duration::days(self.config.warning_period_days);

        let notifications: Vec<_> = deprecated.into_iter()
            .filter_map(|flag| {
                flag.metadata.sunset_date.and_then(|sunset| {
                    if sunset <= warning_threshold && sunset > now {
                        Some(SunsetNotification {
                            flag_id: flag.id.as_str().to_string(),
                            flag_name: flag.name.clone(),
                            sunset_date: sunset,
                            days_remaining: (sunset - now).num_days(),
                            owner: flag.metadata.owner.clone(),
                            usage_count: 0, // Would need usage tracking
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(notifications)
    }

    /// Get inactive flags that could be deprecated
    pub async fn get_inactive_flags(
        &self,
        usage_tracker: &dyn FlagUsageTracker,
    ) -> Result<Vec<InactiveNotification>, DeprecationError> {
        let flags = self.storage.list(QueryOptions {
            status: Some(FlagStatus::Active),
            include_archived: false,
            limit: 1000,
            ..Default::default()
        }).await
            .map_err(|e| DeprecationError::Storage(e.to_string()))?;

        let now = Utc::now();
        let threshold = Duration::days(self.config.inactivity_threshold_days);
        let mut notifications = Vec::new();

        for stored in flags {
            let flag = stored.definition;
            let last_eval = usage_tracker.get_last_evaluation(&flag.id).await;

            if let Some(last) = last_eval {
                let inactive_duration = now - last;
                if inactive_duration > threshold {
                    notifications.push(InactiveNotification {
                        flag_id: flag.id.as_str().to_string(),
                        flag_name: flag.name,
                        days_inactive: inactive_duration.num_days(),
                        last_evaluated: Some(last),
                        owner: flag.metadata.owner,
                    });
                }
            }
        }

        Ok(notifications)
    }

    /// Run sunset cleanup
    pub async fn run_sunset_cleanup(&self) -> Result<CleanupResults, DeprecationError> {
        let now = Utc::now();
        let archive_threshold = now - Duration::days(self.config.auto_archive_delay_days);

        let deprecated = self.get_deprecated_flags().await?;
        let mut results = CleanupResults::default();

        for flag in deprecated {
            if let Some(sunset) = flag.metadata.sunset_date {
                if sunset < archive_threshold && self.config.auto_archive {
                    // Archive the flag
                    let mut archived_flag = flag.clone();
                    archived_flag.status = FlagStatus::Archived;
                    archived_flag.metadata.updated_at = now;
                    archived_flag.metadata.updated_by = "system".to_string();

                    match self.storage.update(archived_flag, None).await {
                        Ok(_) => results.archived_flags.push(flag.id.as_str().to_string()),
                        Err(e) => results.errors.push(format!("{}: {}", flag.id.as_str(), e)),
                    }
                }
            }
        }

        // Notify about cleanup
        self.notification_service.notify_cleanup_complete(results.clone()).await;

        Ok(results)
    }

    /// Check and send deprecation notifications
    pub async fn send_notifications(&self) -> Result<(), DeprecationError> {
        let approaching = self.get_approaching_sunset().await?;
        if !approaching.is_empty() {
            self.notification_service.notify_upcoming_sunset(approaching).await;
        }

        Ok(())
    }

    /// Generate deprecation report
    pub async fn generate_report(&self) -> Result<DeprecationReport, DeprecationError> {
        let deprecated = self.get_deprecated_flags().await?;
        let approaching = self.get_approaching_sunset().await?;

        let now = Utc::now();

        let mut by_owner: HashMap<String, Vec<String>> = HashMap::new();
        for flag in &deprecated {
            let owner = flag.metadata.owner.clone().unwrap_or_else(|| "unowned".to_string());
            by_owner.entry(owner).or_default().push(flag.id.as_str().to_string());
        }

        let past_sunset: Vec<_> = deprecated.iter()
            .filter(|f| f.metadata.sunset_date.map(|d| d < now).unwrap_or(false))
            .map(|f| f.id.as_str().to_string())
            .collect();

        Ok(DeprecationReport {
            generated_at: now,
            total_deprecated: deprecated.len(),
            approaching_sunset: approaching.len(),
            past_sunset: past_sunset.len(),
            flags_past_sunset: past_sunset,
            flags_by_owner: by_owner,
            recommendations: self.generate_recommendations(&deprecated),
        })
    }

    fn generate_recommendations(&self, deprecated: &[FlagDefinition]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let past_sunset: Vec<_> = deprecated.iter()
            .filter(|f| f.metadata.sunset_date.map(|d| d < Utc::now()).unwrap_or(false))
            .collect();

        if !past_sunset.is_empty() {
            recommendations.push(format!(
                "{} flags are past their sunset date and should be removed",
                past_sunset.len()
            ));
        }

        let no_owner: Vec<_> = deprecated.iter()
            .filter(|f| f.metadata.owner.is_none())
            .collect();

        if !no_owner.is_empty() {
            recommendations.push(format!(
                "{} deprecated flags have no owner assigned",
                no_owner.len()
            ));
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationReport {
    pub generated_at: DateTime<Utc>,
    pub total_deprecated: usize,
    pub approaching_sunset: usize,
    pub past_sunset: usize,
    pub flags_past_sunset: Vec<String>,
    pub flags_by_owner: HashMap<String, Vec<String>>,
    pub recommendations: Vec<String>,
}

/// Interface for tracking flag usage
#[async_trait::async_trait]
pub trait FlagUsageTracker: Send + Sync {
    async fn get_last_evaluation(&self, flag_id: &FlagId) -> Option<DateTime<Utc>>;
    async fn get_evaluation_count(&self, flag_id: &FlagId, period: Duration) -> u64;
}

#[derive(Debug, thiserror::Error)]
pub enum DeprecationError {
    #[error("Flag not found: {0}")]
    FlagNotFound(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Invalid sunset date: must be in the future")]
    InvalidSunsetDate,
}

/// Scheduled job for deprecation management
pub async fn run_deprecation_job(manager: Arc<DeprecationManager>) {
    // Run daily
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));

    loop {
        interval.tick().await;

        // Send notifications about approaching sunsets
        if let Err(e) = manager.send_notifications().await {
            tracing::error!("Failed to send deprecation notifications: {}", e);
        }

        // Run cleanup for past-sunset flags
        match manager.run_sunset_cleanup().await {
            Ok(results) => {
                tracing::info!(
                    "Deprecation cleanup complete: archived={}, errors={}",
                    results.archived_flags.len(),
                    results.errors.len()
                );
            }
            Err(e) => {
                tracing::error!("Deprecation cleanup failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    struct MockNotifier;

    #[async_trait::async_trait]
    impl DeprecationNotifier for MockNotifier {
        async fn notify_upcoming_sunset(&self, _flags: Vec<SunsetNotification>) {}
        async fn notify_inactive_flags(&self, _flags: Vec<InactiveNotification>) {}
        async fn notify_cleanup_complete(&self, _results: CleanupResults) {}
    }

    #[tokio::test]
    async fn test_deprecate_flag() {
        let storage = Arc::new(InMemoryStorage::new());
        let notifier = Arc::new(MockNotifier);
        let manager = DeprecationManager::new(storage.clone(), notifier, DeprecationConfig::default());

        // Create a flag first
        let flag = FlagDefinition::new_boolean("test-flag", "Test Flag", false).unwrap();
        storage.create(flag).await.unwrap();

        // Deprecate it
        let sunset = Utc::now() + Duration::days(30);
        let deprecated = manager.deprecate_flag(
            &FlagId::new("test-flag"),
            "No longer needed",
            sunset,
            "admin",
            None,
        ).await.unwrap();

        assert_eq!(deprecated.status, FlagStatus::Deprecated);
        assert!(deprecated.metadata.tags.contains(&"deprecated".to_string()));
    }
}
```

## Deprecation Workflow

1. **Mark as Deprecated** - Set sunset date and replacement
2. **Notification Period** - Warn users of upcoming removal
3. **Code Scanning** - Find remaining usages
4. **Migration Support** - Provide replacement guidance
5. **Archive** - Move to archived state
6. **Cleanup** - Eventually remove from system

## Related Specs

- 392-flag-definition.md - Flag lifecycle states
- 407-flag-audit.md - Change tracking
- 406-flag-analytics.md - Usage tracking
