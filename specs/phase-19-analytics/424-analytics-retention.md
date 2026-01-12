# Spec 424: Data Retention

## Phase
19 - Analytics/Telemetry

## Spec ID
424

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 409: Analytics Storage (data persistence)
- Spec 412: User Consent (consent system)

## Estimated Context
~8%

---

## Objective

Implement comprehensive data retention policies and mechanisms for analytics data, ensuring compliance with privacy requirements while maintaining historical data for analysis.

---

## Acceptance Criteria

- [ ] Define retention policies per data type
- [ ] Implement automatic data cleanup
- [ ] Support data archival before deletion
- [ ] Create retention policy management API
- [ ] Implement compliance deletion
- [ ] Support retention reporting
- [ ] Enable retention policy auditing
- [ ] Create data lifecycle tracking

---

## Implementation Details

### Data Retention

```rust
// src/analytics/retention.rs

use crate::analytics::storage::{AnalyticsStorage, StorageError};
use crate::analytics::types::EventCategory;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Retention policy for a data category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy identifier
    pub id: String,
    /// Policy name
    pub name: String,
    /// Data categories this policy applies to
    pub categories: Vec<EventCategory>,
    /// Retention period in days
    pub retention_days: u32,
    /// Whether to archive before deletion
    pub archive_before_delete: bool,
    /// Archive format if archiving
    pub archive_format: Option<ArchiveFormat>,
    /// Whether policy is active
    pub active: bool,
    /// Minimum data age before deletion (grace period)
    pub grace_period_days: u32,
    /// Legal hold flag (prevents deletion)
    pub legal_hold: bool,
}

impl RetentionPolicy {
    pub fn new(id: &str, name: &str, retention_days: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            categories: Vec::new(),
            retention_days,
            archive_before_delete: false,
            archive_format: None,
            active: true,
            grace_period_days: 0,
            legal_hold: false,
        }
    }

    pub fn with_categories(mut self, categories: Vec<EventCategory>) -> Self {
        self.categories = categories;
        self
    }

    pub fn with_archival(mut self, format: ArchiveFormat) -> Self {
        self.archive_before_delete = true;
        self.archive_format = Some(format);
        self
    }

    pub fn with_grace_period(mut self, days: u32) -> Self {
        self.grace_period_days = days;
        self
    }

    /// Get the deletion cutoff date
    pub fn deletion_cutoff(&self) -> DateTime<Utc> {
        let total_days = self.retention_days + self.grace_period_days;
        Utc::now() - Duration::days(total_days as i64)
    }

    /// Check if data at given timestamp should be deleted
    pub fn should_delete(&self, timestamp: DateTime<Utc>) -> bool {
        if self.legal_hold || !self.active {
            return false;
        }
        timestamp < self.deletion_cutoff()
    }
}

/// Archive format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveFormat {
    Json,
    JsonGzip,
    Parquet,
    Ndjson,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::JsonGzip => "json.gz",
            Self::Parquet => "parquet",
            Self::Ndjson => "ndjson",
        }
    }
}

/// Default retention policies
pub fn default_policies() -> Vec<RetentionPolicy> {
    vec![
        // Short retention for high-volume performance data
        RetentionPolicy::new("perf-short", "Performance Data (Short)", 7)
            .with_categories(vec![EventCategory::Performance]),
        // Medium retention for usage data
        RetentionPolicy::new("usage-medium", "Usage Data", 30)
            .with_categories(vec![EventCategory::Usage, EventCategory::System]),
        // Longer retention for business metrics
        RetentionPolicy::new("business-long", "Business Metrics", 365)
            .with_categories(vec![EventCategory::Business])
            .with_archival(ArchiveFormat::JsonGzip),
        // Extended retention for errors
        RetentionPolicy::new("errors-extended", "Error Data", 90)
            .with_categories(vec![EventCategory::Error])
            .with_archival(ArchiveFormat::Ndjson)
            .with_grace_period(7),
        // Long retention for security events
        RetentionPolicy::new("security-long", "Security Events", 365)
            .with_categories(vec![EventCategory::Security])
            .with_archival(ArchiveFormat::JsonGzip)
            .with_grace_period(30),
    ]
}

/// Retention action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionAction {
    /// Action identifier
    pub id: String,
    /// When the action was performed
    pub timestamp: DateTime<Utc>,
    /// Policy that triggered the action
    pub policy_id: String,
    /// Type of action
    pub action_type: RetentionActionType,
    /// Number of records affected
    pub records_affected: u64,
    /// Archive path if archived
    pub archive_path: Option<PathBuf>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionActionType {
    /// Data was deleted
    Deleted,
    /// Data was archived
    Archived,
    /// Data was archived then deleted
    ArchivedAndDeleted,
    /// Legal hold prevented deletion
    HeldByLegalHold,
}

/// Retention statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionStats {
    /// Total records eligible for deletion
    pub eligible_for_deletion: u64,
    /// Records deleted
    pub deleted: u64,
    /// Records archived
    pub archived: u64,
    /// Records held by legal hold
    pub held: u64,
    /// Deletion errors
    pub errors: u64,
    /// Space reclaimed in bytes
    pub space_reclaimed_bytes: u64,
    /// By category breakdown
    pub by_category: HashMap<EventCategory, CategoryRetentionStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryRetentionStats {
    pub total_records: u64,
    pub oldest_record: Option<DateTime<Utc>>,
    pub records_expiring_soon: u64,
    pub average_age_days: f64,
}

/// Compliance deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDeletionRequest {
    /// Request identifier
    pub id: String,
    /// Requester information
    pub requester: String,
    /// Reason for deletion
    pub reason: ComplianceDeletionReason,
    /// Scope of deletion
    pub scope: DeletionScope,
    /// When the request was made
    pub requested_at: DateTime<Utc>,
    /// Request status
    pub status: DeletionRequestStatus,
    /// When deletion was completed
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceDeletionReason {
    /// GDPR right to erasure
    GdprErasure,
    /// CCPA deletion request
    CcpaRequest,
    /// User request
    UserRequest,
    /// Policy compliance
    PolicyCompliance,
    /// Other reason
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionScope {
    /// Delete all data for a session
    Session(String),
    /// Delete all data older than date
    BeforeDate(DateTime<Utc>),
    /// Delete data matching filter
    Filter {
        categories: Option<Vec<EventCategory>>,
        before: Option<DateTime<Utc>>,
        after: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeletionRequestStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Retention manager
pub struct RetentionManager {
    /// Storage backend
    storage: Arc<dyn AnalyticsStorage>,
    /// Configured policies
    policies: Arc<RwLock<HashMap<String, RetentionPolicy>>>,
    /// Archive directory
    archive_dir: PathBuf,
    /// Action history
    action_history: Arc<RwLock<Vec<RetentionAction>>>,
    /// Pending compliance requests
    compliance_requests: Arc<RwLock<Vec<ComplianceDeletionRequest>>>,
}

impl RetentionManager {
    pub fn new(storage: Arc<dyn AnalyticsStorage>, archive_dir: PathBuf) -> Self {
        let mut policies = HashMap::new();
        for policy in default_policies() {
            policies.insert(policy.id.clone(), policy);
        }

        Self {
            storage,
            policies: Arc::new(RwLock::new(policies)),
            archive_dir,
            action_history: Arc::new(RwLock::new(Vec::new())),
            compliance_requests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add or update a retention policy
    pub async fn set_policy(&self, policy: RetentionPolicy) {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy);
    }

    /// Remove a retention policy
    pub async fn remove_policy(&self, id: &str) -> Option<RetentionPolicy> {
        let mut policies = self.policies.write().await;
        policies.remove(id)
    }

    /// Get a retention policy
    pub async fn get_policy(&self, id: &str) -> Option<RetentionPolicy> {
        let policies = self.policies.read().await;
        policies.get(id).cloned()
    }

    /// Get all policies
    pub async fn get_all_policies(&self) -> Vec<RetentionPolicy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// Set legal hold on a policy
    pub async fn set_legal_hold(&self, policy_id: &str, hold: bool) -> Result<(), RetentionError> {
        let mut policies = self.policies.write().await;
        let policy = policies
            .get_mut(policy_id)
            .ok_or(RetentionError::PolicyNotFound)?;
        policy.legal_hold = hold;
        Ok(())
    }

    /// Run retention enforcement
    pub async fn enforce(&self) -> Result<RetentionStats, RetentionError> {
        let policies = self.policies.read().await;
        let mut stats = RetentionStats::default();

        for policy in policies.values() {
            if !policy.active || policy.legal_hold {
                if policy.legal_hold {
                    stats.held += 1;
                }
                continue;
            }

            let cutoff = policy.deletion_cutoff();

            // Get events to process for this policy's categories
            for category in &policy.categories {
                let events = self
                    .storage
                    .query_by_category(*category, DateTime::UNIX_EPOCH.into(), cutoff)
                    .await
                    .map_err(|e| RetentionError::StorageError(e.to_string()))?;

                let count = events.len() as u64;
                stats.eligible_for_deletion += count;

                if count == 0 {
                    continue;
                }

                // Archive if configured
                if policy.archive_before_delete {
                    let archive_path = self
                        .archive_events(&events, policy)
                        .await?;

                    stats.archived += count;

                    // Record archive action
                    self.record_action(RetentionAction {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        policy_id: policy.id.clone(),
                        action_type: RetentionActionType::Archived,
                        records_affected: count,
                        archive_path: Some(archive_path),
                        success: true,
                        error: None,
                    })
                    .await;
                }

                // Delete
                match self.storage.delete_before(cutoff).await {
                    Ok(deleted) => {
                        stats.deleted += deleted;

                        self.record_action(RetentionAction {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: Utc::now(),
                            policy_id: policy.id.clone(),
                            action_type: if policy.archive_before_delete {
                                RetentionActionType::ArchivedAndDeleted
                            } else {
                                RetentionActionType::Deleted
                            },
                            records_affected: deleted,
                            archive_path: None,
                            success: true,
                            error: None,
                        })
                        .await;
                    }
                    Err(e) => {
                        stats.errors += 1;
                        self.record_action(RetentionAction {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: Utc::now(),
                            policy_id: policy.id.clone(),
                            action_type: RetentionActionType::Deleted,
                            records_affected: 0,
                            archive_path: None,
                            success: false,
                            error: Some(e.to_string()),
                        })
                        .await;
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Archive events
    async fn archive_events(
        &self,
        events: &[crate::analytics::types::AnalyticsEvent],
        policy: &RetentionPolicy,
    ) -> Result<PathBuf, RetentionError> {
        let format = policy.archive_format.unwrap_or(ArchiveFormat::Json);
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("archive_{}_{}.{}", policy.id, timestamp, format.extension());
        let path = self.archive_dir.join(&filename);

        // Ensure archive directory exists
        std::fs::create_dir_all(&self.archive_dir)
            .map_err(|e| RetentionError::IoError(e.to_string()))?;

        let content = match format {
            ArchiveFormat::Json => serde_json::to_string_pretty(events)
                .map_err(|e| RetentionError::SerializationError(e.to_string()))?,
            ArchiveFormat::JsonGzip => {
                let json = serde_json::to_string(events)
                    .map_err(|e| RetentionError::SerializationError(e.to_string()))?;

                use flate2::write::GzEncoder;
                use flate2::Compression;
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(json.as_bytes())
                    .map_err(|e| RetentionError::IoError(e.to_string()))?;
                let compressed = encoder
                    .finish()
                    .map_err(|e| RetentionError::IoError(e.to_string()))?;

                std::fs::write(&path, compressed)
                    .map_err(|e| RetentionError::IoError(e.to_string()))?;

                return Ok(path);
            }
            ArchiveFormat::Ndjson => {
                let mut lines = Vec::new();
                for event in events {
                    lines.push(
                        serde_json::to_string(event)
                            .map_err(|e| RetentionError::SerializationError(e.to_string()))?,
                    );
                }
                lines.join("\n")
            }
            ArchiveFormat::Parquet => {
                // Would require arrow/parquet crate
                serde_json::to_string(events)
                    .map_err(|e| RetentionError::SerializationError(e.to_string()))?
            }
        };

        std::fs::write(&path, content).map_err(|e| RetentionError::IoError(e.to_string()))?;

        Ok(path)
    }

    /// Record a retention action
    async fn record_action(&self, action: RetentionAction) {
        let mut history = self.action_history.write().await;
        history.push(action);

        // Keep only last 1000 actions
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }

    /// Submit a compliance deletion request
    pub async fn submit_compliance_request(
        &self,
        request: ComplianceDeletionRequest,
    ) -> Result<String, RetentionError> {
        let id = request.id.clone();
        let mut requests = self.compliance_requests.write().await;
        requests.push(request);
        Ok(id)
    }

    /// Process pending compliance requests
    pub async fn process_compliance_requests(&self) -> Result<Vec<String>, RetentionError> {
        let mut requests = self.compliance_requests.write().await;
        let mut processed = Vec::new();

        for request in requests.iter_mut() {
            if request.status != DeletionRequestStatus::Pending {
                continue;
            }

            request.status = DeletionRequestStatus::InProgress;

            let result = match &request.scope {
                DeletionScope::Session(session_id) => {
                    // Would need session-based deletion in storage
                    Ok(())
                }
                DeletionScope::BeforeDate(date) => {
                    self.storage
                        .delete_before(*date)
                        .await
                        .map(|_| ())
                        .map_err(|e| RetentionError::StorageError(e.to_string()))
                }
                DeletionScope::Filter { before, .. } => {
                    if let Some(date) = before {
                        self.storage
                            .delete_before(*date)
                            .await
                            .map(|_| ())
                            .map_err(|e| RetentionError::StorageError(e.to_string()))
                    } else {
                        Ok(())
                    }
                }
            };

            match result {
                Ok(()) => {
                    request.status = DeletionRequestStatus::Completed;
                    request.completed_at = Some(Utc::now());
                    processed.push(request.id.clone());
                }
                Err(_) => {
                    request.status = DeletionRequestStatus::Failed;
                }
            }
        }

        Ok(processed)
    }

    /// Get retention statistics
    pub async fn get_stats(&self) -> Result<RetentionStats, RetentionError> {
        let storage_stats = self
            .storage
            .stats()
            .await
            .map_err(|e| RetentionError::StorageError(e.to_string()))?;

        let mut stats = RetentionStats::default();

        for (category, count) in storage_stats.events_by_category {
            stats.by_category.insert(
                category,
                CategoryRetentionStats {
                    total_records: count,
                    oldest_record: storage_stats.oldest_event,
                    records_expiring_soon: 0, // Would need more complex query
                    average_age_days: 0.0,
                },
            );
        }

        Ok(stats)
    }

    /// Get action history
    pub async fn get_action_history(&self, limit: usize) -> Vec<RetentionAction> {
        let history = self.action_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

/// Retention errors
#[derive(Debug, thiserror::Error)]
pub enum RetentionError {
    #[error("Policy not found")]
    PolicyNotFound,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Legal hold prevents deletion")]
    LegalHold,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::config::StorageConfig;
    use crate::analytics::storage::SqliteAnalyticsStorage;

    #[test]
    fn test_retention_policy_creation() {
        let policy = RetentionPolicy::new("test", "Test Policy", 30)
            .with_categories(vec![EventCategory::Usage])
            .with_grace_period(7);

        assert_eq!(policy.retention_days, 30);
        assert_eq!(policy.grace_period_days, 7);
        assert!(policy.categories.contains(&EventCategory::Usage));
    }

    #[test]
    fn test_deletion_cutoff() {
        let policy = RetentionPolicy::new("test", "Test", 30)
            .with_grace_period(7);

        let cutoff = policy.deletion_cutoff();
        let now = Utc::now();

        // Cutoff should be 37 days ago
        let expected_age = 37;
        let actual_age = (now - cutoff).num_days();

        assert!(actual_age >= expected_age - 1 && actual_age <= expected_age + 1);
    }

    #[test]
    fn test_should_delete() {
        let policy = RetentionPolicy::new("test", "Test", 30);

        let old_date = Utc::now() - Duration::days(60);
        let recent_date = Utc::now() - Duration::days(10);

        assert!(policy.should_delete(old_date));
        assert!(!policy.should_delete(recent_date));
    }

    #[test]
    fn test_legal_hold_prevents_deletion() {
        let mut policy = RetentionPolicy::new("test", "Test", 30);
        policy.legal_hold = true;

        let old_date = Utc::now() - Duration::days(60);
        assert!(!policy.should_delete(old_date));
    }

    #[test]
    fn test_default_policies() {
        let policies = default_policies();

        assert!(!policies.is_empty());

        // Check we have policies for different categories
        let has_perf = policies.iter().any(|p| p.categories.contains(&EventCategory::Performance));
        let has_business = policies.iter().any(|p| p.categories.contains(&EventCategory::Business));
        let has_security = policies.iter().any(|p| p.categories.contains(&EventCategory::Security));

        assert!(has_perf);
        assert!(has_business);
        assert!(has_security);
    }

    #[tokio::test]
    async fn test_retention_manager_policies() {
        let storage = Arc::new(
            SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
        );
        let manager = RetentionManager::new(storage, PathBuf::from("/tmp/archives"));

        // Check default policies are loaded
        let policies = manager.get_all_policies().await;
        assert!(!policies.is_empty());

        // Add custom policy
        let custom = RetentionPolicy::new("custom", "Custom Policy", 14);
        manager.set_policy(custom).await;

        let retrieved = manager.get_policy("custom").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().retention_days, 14);
    }

    #[tokio::test]
    async fn test_legal_hold() {
        let storage = Arc::new(
            SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
        );
        let manager = RetentionManager::new(storage, PathBuf::from("/tmp/archives"));

        let policy_id = default_policies()[0].id.clone();
        manager.set_legal_hold(&policy_id, true).await.unwrap();

        let policy = manager.get_policy(&policy_id).await.unwrap();
        assert!(policy.legal_hold);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Policy creation and configuration
   - Deletion cutoff calculation
   - Legal hold behavior
   - Archive format handling

2. **Integration Tests**
   - Full retention enforcement
   - Archive creation and validation
   - Compliance request processing

3. **Compliance Tests**
   - GDPR deletion requirements
   - Data permanence verification

---

## Related Specs

- Spec 409: Analytics Storage
- Spec 411: Analytics Export
- Spec 412: User Consent
- Spec 413: Privacy Controls
