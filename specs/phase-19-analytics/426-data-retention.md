# 426 - Data Retention

## Overview

Configurable data retention policies with automatic data expiration, archival, and deletion for compliance and storage management.

## Rust Implementation

```rust
// crates/analytics/src/retention.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Retention period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPeriod {
    /// Number of days to retain
    pub days: u32,
    /// Human-readable label
    pub label: String,
}

impl RetentionPeriod {
    pub fn days(days: u32) -> Self {
        Self {
            days,
            label: format!("{} days", days),
        }
    }

    pub fn months(months: u32) -> Self {
        let days = months * 30;
        Self {
            days,
            label: format!("{} months", months),
        }
    }

    pub fn years(years: u32) -> Self {
        let days = years * 365;
        Self {
            days,
            label: format!("{} years", years),
        }
    }

    pub fn forever() -> Self {
        Self {
            days: u32::MAX,
            label: "Forever".to_string(),
        }
    }

    pub fn cutoff_date(&self) -> DateTime<Utc> {
        if self.days == u32::MAX {
            DateTime::<Utc>::MIN_UTC
        } else {
            Utc::now() - Duration::days(self.days as i64)
        }
    }
}

/// Data category for retention
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataCategory {
    /// Raw event data
    Events,
    /// Session data
    Sessions,
    /// User profiles
    UserProfiles,
    /// Aggregated data
    Aggregations,
    /// Error data
    Errors,
    /// Performance data
    Performance,
    /// Audit logs
    AuditLogs,
    /// Export files
    Exports,
}

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Retention rules by category
    pub rules: HashMap<DataCategory, RetentionRule>,
    /// Default retention for uncategorized data
    pub default_retention: RetentionPeriod,
    /// Whether to archive before deletion
    pub archive_before_delete: bool,
    /// Archive destination
    pub archive_destination: Option<ArchiveDestination>,
    /// Enabled
    pub enabled: bool,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionRule {
    /// Data category
    pub category: DataCategory,
    /// Retention period
    pub retention: RetentionPeriod,
    /// Action when expired
    pub action: RetentionAction,
    /// Filters (e.g., only certain event types)
    pub filters: Vec<RetentionFilter>,
    /// Granularity for partial retention
    pub granularity: Option<RetentionGranularity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionAction {
    /// Delete permanently
    Delete,
    /// Archive to cold storage
    Archive,
    /// Aggregate and delete raw
    AggregateAndDelete,
    /// Anonymize PII
    Anonymize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionFilter {
    pub property: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionGranularity {
    /// Keep hourly aggregates after daily expires
    HourlyToDaily,
    /// Keep daily aggregates after raw expires
    RawToDaily,
    /// Keep monthly aggregates after daily expires
    DailyToMonthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArchiveDestination {
    S3 {
        bucket: String,
        prefix: String,
        region: String,
        storage_class: S3StorageClass,
    },
    Gcs {
        bucket: String,
        prefix: String,
        storage_class: GcsStorageClass,
    },
    Azure {
        container: String,
        prefix: String,
        tier: AzureAccessTier,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum S3StorageClass {
    Standard,
    StandardIa,
    OneZoneIa,
    Glacier,
    GlacierDeepArchive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GcsStorageClass {
    Standard,
    Nearline,
    Coldline,
    Archive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AzureAccessTier {
    Hot,
    Cool,
    Archive,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        let mut rules = HashMap::new();

        rules.insert(DataCategory::Events, RetentionRule {
            category: DataCategory::Events,
            retention: RetentionPeriod::months(12),
            action: RetentionAction::Delete,
            filters: vec![],
            granularity: Some(RetentionGranularity::RawToDaily),
        });

        rules.insert(DataCategory::Sessions, RetentionRule {
            category: DataCategory::Sessions,
            retention: RetentionPeriod::months(6),
            action: RetentionAction::Delete,
            filters: vec![],
            granularity: None,
        });

        rules.insert(DataCategory::UserProfiles, RetentionRule {
            category: DataCategory::UserProfiles,
            retention: RetentionPeriod::years(2),
            action: RetentionAction::Anonymize,
            filters: vec![],
            granularity: None,
        });

        rules.insert(DataCategory::Aggregations, RetentionRule {
            category: DataCategory::Aggregations,
            retention: RetentionPeriod::years(5),
            action: RetentionAction::Delete,
            filters: vec![],
            granularity: None,
        });

        rules.insert(DataCategory::Errors, RetentionRule {
            category: DataCategory::Errors,
            retention: RetentionPeriod::months(3),
            action: RetentionAction::Delete,
            filters: vec![],
            granularity: None,
        });

        rules.insert(DataCategory::AuditLogs, RetentionRule {
            category: DataCategory::AuditLogs,
            retention: RetentionPeriod::years(7),
            action: RetentionAction::Archive,
            filters: vec![],
            granularity: None,
        });

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Policy".to_string(),
            description: Some("Standard data retention policy".to_string()),
            rules,
            default_retention: RetentionPeriod::months(12),
            archive_before_delete: false,
            archive_destination: None,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Retention executor
pub struct RetentionExecutor {
    policy: RetentionPolicy,
    storage: std::sync::Arc<dyn RetentionStorage>,
    archiver: Option<std::sync::Arc<dyn DataArchiver>>,
}

#[async_trait]
pub trait RetentionStorage: Send + Sync {
    /// Get data to be processed for a category
    async fn get_expired_data(
        &self,
        category: DataCategory,
        cutoff: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<DataRecord>, RetentionError>;

    /// Delete records
    async fn delete_records(&self, category: DataCategory, ids: &[String]) -> Result<u64, RetentionError>;

    /// Anonymize records
    async fn anonymize_records(&self, category: DataCategory, ids: &[String]) -> Result<u64, RetentionError>;

    /// Get aggregation data
    async fn get_aggregation_source(
        &self,
        category: DataCategory,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AggregationRecord>, RetentionError>;

    /// Store aggregation
    async fn store_aggregation(&self, records: Vec<AggregationRecord>) -> Result<(), RetentionError>;
}

#[async_trait]
pub trait DataArchiver: Send + Sync {
    async fn archive(&self, data: &[DataRecord], destination: &ArchiveDestination) -> Result<String, RetentionError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: String,
    pub category: DataCategory,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationRecord {
    pub category: DataCategory,
    pub timestamp: DateTime<Utc>,
    pub granularity: String,
    pub metrics: HashMap<String, f64>,
    pub dimensions: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RetentionError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Archive error: {0}")]
    Archive(String),
    #[error("Invalid policy: {0}")]
    Invalid(String),
}

impl RetentionExecutor {
    pub fn new(
        policy: RetentionPolicy,
        storage: std::sync::Arc<dyn RetentionStorage>,
        archiver: Option<std::sync::Arc<dyn DataArchiver>>,
    ) -> Self {
        Self { policy, storage, archiver }
    }

    /// Run retention for all categories
    pub async fn run(&self) -> Result<RetentionReport, RetentionError> {
        let mut report = RetentionReport::new();

        for (category, rule) in &self.policy.rules {
            match self.process_category(*category, rule).await {
                Ok(category_report) => {
                    report.categories.insert(*category, category_report);
                }
                Err(e) => {
                    tracing::error!("Retention failed for {:?}: {}", category, e);
                    report.errors.push(format!("{:?}: {}", category, e));
                }
            }
        }

        report.completed_at = Some(Utc::now());
        Ok(report)
    }

    /// Process a single category
    async fn process_category(
        &self,
        category: DataCategory,
        rule: &RetentionRule,
    ) -> Result<CategoryReport, RetentionError> {
        let cutoff = rule.retention.cutoff_date();
        let mut report = CategoryReport::new(category);

        let batch_size = 1000u32;
        let mut total_processed = 0u64;

        loop {
            let records = self.storage.get_expired_data(category, cutoff, batch_size).await?;

            if records.is_empty() {
                break;
            }

            let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();

            // Archive if configured
            if self.policy.archive_before_delete {
                if let (Some(archiver), Some(dest)) = (&self.archiver, &self.policy.archive_destination) {
                    let archive_path = archiver.archive(&records, dest).await?;
                    report.archive_paths.push(archive_path);
                }
            }

            // Apply action
            let count = match rule.action {
                RetentionAction::Delete => {
                    self.storage.delete_records(category, &ids).await?
                }
                RetentionAction::Anonymize => {
                    self.storage.anonymize_records(category, &ids).await?
                }
                RetentionAction::Archive => {
                    if let (Some(archiver), Some(dest)) = (&self.archiver, &self.policy.archive_destination) {
                        archiver.archive(&records, dest).await?;
                    }
                    self.storage.delete_records(category, &ids).await?
                }
                RetentionAction::AggregateAndDelete => {
                    self.aggregate_and_delete(category, rule, &records).await?
                }
            };

            total_processed += count;

            if records.len() < batch_size as usize {
                break;
            }
        }

        report.records_processed = total_processed;
        report.action = rule.action;

        Ok(report)
    }

    /// Aggregate data before deletion
    async fn aggregate_and_delete(
        &self,
        category: DataCategory,
        rule: &RetentionRule,
        records: &[DataRecord],
    ) -> Result<u64, RetentionError> {
        // Group records by day
        let mut daily_aggregates: HashMap<String, AggregationRecord> = HashMap::new();

        for record in records {
            let day_key = record.timestamp.format("%Y-%m-%d").to_string();

            let agg = daily_aggregates.entry(day_key.clone()).or_insert_with(|| {
                AggregationRecord {
                    category,
                    timestamp: record.timestamp.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
                    granularity: "daily".to_string(),
                    metrics: HashMap::new(),
                    dimensions: HashMap::new(),
                }
            });

            // Increment count
            *agg.metrics.entry("count".to_string()).or_insert(0.0) += 1.0;
        }

        // Store aggregations
        let aggregations: Vec<_> = daily_aggregates.into_values().collect();
        self.storage.store_aggregation(aggregations).await?;

        // Delete original records
        let ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
        self.storage.delete_records(category, &ids).await
    }
}

/// Retention execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionReport {
    /// Execution ID
    pub id: String,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Per-category reports
    pub categories: HashMap<DataCategory, CategoryReport>,
    /// Errors
    pub errors: Vec<String>,
}

impl RetentionReport {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            completed_at: None,
            categories: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn total_processed(&self) -> u64 {
        self.categories.values().map(|c| c.records_processed).sum()
    }
}

impl Default for RetentionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    pub category: DataCategory,
    pub records_processed: u64,
    pub action: RetentionAction,
    pub archive_paths: Vec<String>,
}

impl CategoryReport {
    pub fn new(category: DataCategory) -> Self {
        Self {
            category,
            records_processed: 0,
            action: RetentionAction::Delete,
            archive_paths: Vec::new(),
        }
    }
}

/// Retention scheduler
pub struct RetentionScheduler {
    executor: std::sync::Arc<RetentionExecutor>,
    schedule: String, // Cron expression
}

impl RetentionScheduler {
    pub fn new(executor: std::sync::Arc<RetentionExecutor>, schedule: &str) -> Self {
        Self {
            executor,
            schedule: schedule.to_string(),
        }
    }

    /// Start the scheduler
    pub async fn start(self: std::sync::Arc<Self>) {
        let scheduler = self.clone();

        tokio::spawn(async move {
            // Run daily at 2 AM by default
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));

            loop {
                interval.tick().await;

                tracing::info!("Starting retention job");

                match scheduler.executor.run().await {
                    Ok(report) => {
                        tracing::info!(
                            "Retention completed: {} records processed",
                            report.total_processed()
                        );
                    }
                    Err(e) => {
                        tracing::error!("Retention failed: {}", e);
                    }
                }
            }
        });
    }
}

/// Storage estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEstimate {
    /// Total storage used (bytes)
    pub total_bytes: u64,
    /// Per-category breakdown
    pub by_category: HashMap<DataCategory, CategoryStorage>,
    /// Projected savings with retention
    pub projected_savings: u64,
    /// Data eligible for deletion
    pub eligible_for_deletion: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStorage {
    pub category: DataCategory,
    pub bytes: u64,
    pub record_count: u64,
    pub oldest_record: DateTime<Utc>,
    pub newest_record: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_period() {
        let period = RetentionPeriod::days(30);
        assert_eq!(period.days, 30);

        let cutoff = period.cutoff_date();
        let days_ago = (Utc::now() - cutoff).num_days();
        assert!(days_ago >= 29 && days_ago <= 31);
    }

    #[test]
    fn test_default_policy() {
        let policy = RetentionPolicy::default();

        assert!(policy.enabled);
        assert!(policy.rules.contains_key(&DataCategory::Events));
        assert!(policy.rules.contains_key(&DataCategory::Sessions));

        let events_rule = policy.rules.get(&DataCategory::Events).unwrap();
        assert_eq!(events_rule.retention.days, 360); // 12 months
    }

    #[test]
    fn test_retention_periods() {
        assert_eq!(RetentionPeriod::months(12).days, 360);
        assert_eq!(RetentionPeriod::years(2).days, 730);
        assert_eq!(RetentionPeriod::forever().days, u32::MAX);
    }

    #[test]
    fn test_retention_report() {
        let mut report = RetentionReport::new();

        let mut cat_report = CategoryReport::new(DataCategory::Events);
        cat_report.records_processed = 100;
        report.categories.insert(DataCategory::Events, cat_report);

        let mut cat_report2 = CategoryReport::new(DataCategory::Sessions);
        cat_report2.records_processed = 50;
        report.categories.insert(DataCategory::Sessions, cat_report2);

        assert_eq!(report.total_processed(), 150);
    }
}
```

## Database Queries

```sql
-- ClickHouse TTL example
ALTER TABLE events
    MODIFY TTL timestamp + INTERVAL 365 DAY;

-- Partitioned deletion
ALTER TABLE events
    DROP PARTITION '202301';

-- PostgreSQL partition management
CREATE OR REPLACE FUNCTION drop_old_partitions(
    retention_days INTEGER DEFAULT 365
) RETURNS void AS $$
DECLARE
    partition_name TEXT;
    cutoff_date DATE;
BEGIN
    cutoff_date := CURRENT_DATE - retention_days;

    FOR partition_name IN
        SELECT tablename
        FROM pg_tables
        WHERE tablename LIKE 'events_p%'
        AND tablename < 'events_p' || to_char(cutoff_date, 'YYYY_MM')
    LOOP
        EXECUTE 'DROP TABLE IF EXISTS ' || partition_name;
        RAISE NOTICE 'Dropped partition: %', partition_name;
    END LOOP;
END;
$$ LANGUAGE plpgsql;
```

## Related Specs

- 415-event-persistence.md - Event storage
- 424-analytics-export.md - Export before deletion
- 425-privacy-compliance.md - GDPR retention requirements
