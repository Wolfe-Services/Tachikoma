# Spec 419: Error Tracking

## Phase
19 - Analytics/Telemetry

## Spec ID
419

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 408: Analytics Collector (event collection)

## Estimated Context
~10%

---

## Objective

Implement comprehensive error tracking and reporting for Tachikoma, enabling systematic capture, categorization, and analysis of errors to improve reliability and user experience.

---

## Acceptance Criteria

- [ ] Capture and categorize errors by type
- [ ] Track error rates and trends
- [ ] Support stack trace capture (privacy-aware)
- [ ] Implement error grouping/deduplication
- [ ] Create error alerting mechanisms
- [ ] Track error recovery success
- [ ] Support error context enrichment
- [ ] Enable error pattern analysis

---

## Implementation Details

### Error Tracking

```rust
// src/analytics/errors.rs

use crate::analytics::collector::EventCollector;
use crate::analytics::privacy::PiiDetector;
use crate::analytics::types::{
    ErrorEventData, ErrorSeverity, EventBuilder, EventData, EventType,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error category for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Network-related errors
    Network,
    /// Authentication/authorization errors
    Auth,
    /// Backend/LLM provider errors
    Backend,
    /// Configuration errors
    Config,
    /// Resource limit errors
    Resource,
    /// Validation errors
    Validation,
    /// Internal/logic errors
    Internal,
    /// Plugin errors
    Plugin,
    /// Unknown/uncategorized
    Unknown,
}

impl ErrorCategory {
    pub fn from_code(code: &str) -> Self {
        let code_lower = code.to_lowercase();

        if code_lower.contains("network") || code_lower.contains("connection") || code_lower.contains("timeout") {
            Self::Network
        } else if code_lower.contains("auth") || code_lower.contains("permission") || code_lower.contains("forbidden") {
            Self::Auth
        } else if code_lower.contains("backend") || code_lower.contains("api") || code_lower.contains("provider") {
            Self::Backend
        } else if code_lower.contains("config") || code_lower.contains("setting") {
            Self::Config
        } else if code_lower.contains("limit") || code_lower.contains("quota") || code_lower.contains("memory") {
            Self::Resource
        } else if code_lower.contains("valid") || code_lower.contains("parse") || code_lower.contains("format") {
            Self::Validation
        } else if code_lower.contains("internal") || code_lower.contains("panic") {
            Self::Internal
        } else if code_lower.contains("plugin") || code_lower.contains("extension") {
            Self::Plugin
        } else {
            Self::Unknown
        }
    }
}

/// Tracked error record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedError {
    /// Unique error instance ID
    pub id: String,
    /// Error fingerprint for grouping
    pub fingerprint: String,
    /// Error code
    pub code: String,
    /// Error message (sanitized)
    pub message: String,
    /// Error category
    pub category: ErrorCategory,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Component where error occurred
    pub component: String,
    /// Stack trace (if available and allowed)
    pub stack_trace: Option<String>,
    /// When the error occurred
    pub timestamp: DateTime<Utc>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Whether error was recovered from
    pub recovered: bool,
    /// Recovery time if recovered
    pub recovery_time_ms: Option<u64>,
    /// Related request/mission ID
    pub correlation_id: Option<String>,
}

impl TrackedError {
    pub fn new(code: &str, message: &str, component: &str) -> Self {
        let fingerprint = calculate_fingerprint(code, component);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            fingerprint,
            code: code.to_string(),
            message: message.to_string(),
            category: ErrorCategory::from_code(code),
            severity: ErrorSeverity::Error,
            component: component.to_string(),
            stack_trace: None,
            timestamp: Utc::now(),
            context: HashMap::new(),
            recovered: false,
            recovery_time_ms: None,
            correlation_id: None,
        }
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_stack_trace(mut self, trace: &str) -> Self {
        self.stack_trace = Some(trace.to_string());
        self
    }

    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.context.insert(key.to_string(), value);
        self
    }

    pub fn with_correlation(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }
}

/// Calculate error fingerprint for grouping
fn calculate_fingerprint(code: &str, component: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update(b":");
    hasher.update(component.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Error group (deduplicated errors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorGroup {
    /// Group fingerprint
    pub fingerprint: String,
    /// Representative error code
    pub code: String,
    /// Representative error message
    pub message: String,
    /// Error category
    pub category: ErrorCategory,
    /// Most severe severity seen
    pub max_severity: ErrorSeverity,
    /// Component
    pub component: String,
    /// First occurrence
    pub first_seen: DateTime<Utc>,
    /// Last occurrence
    pub last_seen: DateTime<Utc>,
    /// Total occurrences
    pub occurrence_count: u64,
    /// Occurrences in last hour
    pub recent_count: u64,
    /// Recovery rate
    pub recovery_rate: f64,
    /// Is currently active (seen recently)
    pub active: bool,
}

/// Error statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Total errors tracked
    pub total_errors: u64,
    /// Errors by category
    pub by_category: HashMap<ErrorCategory, u64>,
    /// Errors by severity
    pub by_severity: HashMap<ErrorSeverity, u64>,
    /// Errors by component
    pub by_component: HashMap<String, u64>,
    /// Error rate per hour
    pub hourly_rate: f64,
    /// Overall recovery rate
    pub recovery_rate: f64,
    /// Unique error types
    pub unique_errors: u64,
}

/// Error alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAlert {
    /// Alert identifier
    pub id: String,
    /// Alert name
    pub name: String,
    /// Error categories to alert on
    pub categories: Vec<ErrorCategory>,
    /// Minimum severity
    pub min_severity: ErrorSeverity,
    /// Threshold (errors per hour)
    pub rate_threshold: Option<f64>,
    /// Count threshold (total errors)
    pub count_threshold: Option<u64>,
    /// Whether alert is enabled
    pub enabled: bool,
}

/// Error tracking system
pub struct ErrorTracker {
    /// Event collector
    collector: Arc<EventCollector>,
    /// PII detector for sanitization
    pii_detector: Arc<PiiDetector>,
    /// Error history
    errors: Arc<RwLock<Vec<TrackedError>>>,
    /// Error groups by fingerprint
    groups: Arc<RwLock<HashMap<String, ErrorGroup>>>,
    /// Error counts by category
    category_counts: Arc<RwLock<HashMap<ErrorCategory, AtomicU64>>>,
    /// Active alerts
    alerts: Arc<RwLock<Vec<ErrorAlert>>>,
    /// Alert callback
    alert_handler: Arc<RwLock<Option<Box<dyn Fn(&ErrorAlert, &TrackedError) + Send + Sync>>>>,
    /// Include stack traces
    include_traces: bool,
}

impl ErrorTracker {
    pub fn new(collector: Arc<EventCollector>) -> Self {
        Self {
            collector,
            pii_detector: Arc::new(PiiDetector::default()),
            errors: Arc::new(RwLock::new(Vec::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            category_counts: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            alert_handler: Arc::new(RwLock::new(None)),
            include_traces: false,
        }
    }

    pub fn with_stack_traces(mut self, enabled: bool) -> Self {
        self.include_traces = enabled;
        self
    }

    /// Track an error
    pub async fn track(&self, mut error: TrackedError) -> Result<(), ErrorTrackingError> {
        // Sanitize error data
        error.message = self.pii_detector.redact(&error.message);
        if let Some(ref mut trace) = error.stack_trace {
            if self.include_traces {
                *trace = self.pii_detector.redact(trace);
            } else {
                error.stack_trace = None;
            }
        }

        // Store error
        {
            let mut errors = self.errors.write().await;
            errors.push(error.clone());

            // Trim old errors (keep last 24 hours)
            let cutoff = Utc::now() - Duration::hours(24);
            errors.retain(|e| e.timestamp >= cutoff);
        }

        // Update group
        self.update_group(&error).await;

        // Update category counts
        {
            let mut counts = self.category_counts.write().await;
            let counter = counts
                .entry(error.category.clone())
                .or_insert_with(|| AtomicU64::new(0));
            counter.fetch_add(1, Ordering::Relaxed);
        }

        // Check alerts
        self.check_alerts(&error).await;

        // Emit event
        let event = EventBuilder::new(EventType::ErrorOccurred)
            .data(EventData::Error(ErrorEventData {
                code: error.code.clone(),
                message: error.message.clone(),
                severity: error.severity,
                stack_trace: error.stack_trace.clone(),
                component: error.component.clone(),
                recovered: error.recovered,
            }))
            .custom_metadata("category", serde_json::json!(format!("{:?}", error.category)))
            .custom_metadata("fingerprint", serde_json::json!(error.fingerprint))
            .build();

        self.collector
            .collect(event)
            .await
            .map_err(|e| ErrorTrackingError::CollectionFailed(e.to_string()))?;

        Ok(())
    }

    /// Track error from std Error
    pub async fn track_std_error(
        &self,
        err: &dyn std::error::Error,
        component: &str,
    ) -> Result<(), ErrorTrackingError> {
        let code = format!("{}", std::any::type_name_of_val(err))
            .split("::")
            .last()
            .unwrap_or("UnknownError")
            .to_string();

        let error = TrackedError::new(&code, &err.to_string(), component);
        self.track(error).await
    }

    /// Mark an error as recovered
    pub async fn mark_recovered(
        &self,
        error_id: &str,
        recovery_time_ms: u64,
    ) -> Result<(), ErrorTrackingError> {
        let mut errors = self.errors.write().await;

        if let Some(error) = errors.iter_mut().find(|e| e.id == error_id) {
            error.recovered = true;
            error.recovery_time_ms = Some(recovery_time_ms);
        }

        Ok(())
    }

    /// Update error group statistics
    async fn update_group(&self, error: &TrackedError) {
        let mut groups = self.groups.write().await;

        let group = groups
            .entry(error.fingerprint.clone())
            .or_insert_with(|| ErrorGroup {
                fingerprint: error.fingerprint.clone(),
                code: error.code.clone(),
                message: error.message.clone(),
                category: error.category.clone(),
                max_severity: error.severity,
                component: error.component.clone(),
                first_seen: error.timestamp,
                last_seen: error.timestamp,
                occurrence_count: 0,
                recent_count: 0,
                recovery_rate: 0.0,
                active: true,
            });

        group.last_seen = error.timestamp;
        group.occurrence_count += 1;
        group.recent_count += 1;

        if error.severity as u8 > group.max_severity as u8 {
            group.max_severity = error.severity;
        }
    }

    /// Check and trigger alerts
    async fn check_alerts(&self, error: &TrackedError) {
        let alerts = self.alerts.read().await;

        for alert in alerts.iter() {
            if !alert.enabled {
                continue;
            }

            // Check category filter
            if !alert.categories.is_empty()
                && !alert.categories.contains(&error.category)
            {
                continue;
            }

            // Check severity
            if (error.severity as u8) < (alert.min_severity as u8) {
                continue;
            }

            // Check thresholds
            let should_alert = if let Some(_rate_threshold) = alert.rate_threshold {
                // Would need rate calculation
                true
            } else if let Some(_count_threshold) = alert.count_threshold {
                // Would need count check
                true
            } else {
                true
            };

            if should_alert {
                if let Some(ref handler) = *self.alert_handler.read().await {
                    handler(alert, error);
                }
            }
        }
    }

    /// Add an alert
    pub async fn add_alert(&self, alert: ErrorAlert) {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
    }

    /// Set alert handler
    pub async fn set_alert_handler<F>(&self, handler: F)
    where
        F: Fn(&ErrorAlert, &TrackedError) + Send + Sync + 'static,
    {
        let mut alert_handler = self.alert_handler.write().await;
        *alert_handler = Some(Box::new(handler));
    }

    /// Get error groups
    pub async fn get_groups(&self) -> Vec<ErrorGroup> {
        let mut groups: Vec<_> = self.groups.read().await.values().cloned().collect();

        // Mark inactive groups
        let cutoff = Utc::now() - Duration::hours(1);
        for group in &mut groups {
            group.active = group.last_seen >= cutoff;
        }

        // Sort by occurrence count descending
        groups.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));

        groups
    }

    /// Get errors for a specific group
    pub async fn get_errors_for_group(&self, fingerprint: &str) -> Vec<TrackedError> {
        self.errors
            .read()
            .await
            .iter()
            .filter(|e| e.fingerprint == fingerprint)
            .cloned()
            .collect()
    }

    /// Get recent errors
    pub async fn get_recent_errors(&self, limit: usize) -> Vec<TrackedError> {
        let errors = self.errors.read().await;
        let mut recent: Vec<_> = errors.iter().cloned().collect();
        recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        recent.truncate(limit);
        recent
    }

    /// Get error statistics
    pub async fn get_stats(&self) -> ErrorStats {
        let errors = self.errors.read().await;
        let groups = self.groups.read().await;

        let mut stats = ErrorStats::default();
        stats.total_errors = errors.len() as u64;
        stats.unique_errors = groups.len() as u64;

        let mut recovered_count = 0u64;

        for error in errors.iter() {
            *stats.by_category.entry(error.category.clone()).or_insert(0) += 1;
            *stats.by_severity.entry(error.severity).or_insert(0) += 1;
            *stats.by_component.entry(error.component.clone()).or_insert(0) += 1;

            if error.recovered {
                recovered_count += 1;
            }
        }

        // Calculate hourly rate
        let hour_ago = Utc::now() - Duration::hours(1);
        let recent_count = errors.iter().filter(|e| e.timestamp >= hour_ago).count();
        stats.hourly_rate = recent_count as f64;

        // Calculate recovery rate
        if !errors.is_empty() {
            stats.recovery_rate = recovered_count as f64 / errors.len() as f64;
        }

        stats
    }

    /// Analyze error patterns
    pub async fn analyze_patterns(&self) -> Vec<ErrorPattern> {
        let errors = self.errors.read().await;
        let mut patterns = Vec::new();

        // Find errors that tend to occur together
        let mut co_occurrences: HashMap<(String, String), u64> = HashMap::new();

        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                let time_diff = (errors[j].timestamp - errors[i].timestamp).num_seconds().abs();

                if time_diff < 60 && errors[i].fingerprint != errors[j].fingerprint {
                    let key = if errors[i].fingerprint < errors[j].fingerprint {
                        (errors[i].fingerprint.clone(), errors[j].fingerprint.clone())
                    } else {
                        (errors[j].fingerprint.clone(), errors[i].fingerprint.clone())
                    };
                    *co_occurrences.entry(key).or_insert(0) += 1;
                }
            }
        }

        // Find significant co-occurrences
        for ((fp1, fp2), count) in co_occurrences {
            if count >= 3 {
                patterns.push(ErrorPattern {
                    pattern_type: PatternType::CoOccurrence,
                    fingerprints: vec![fp1, fp2],
                    occurrence_count: count,
                    description: "These errors tend to occur together".to_string(),
                });
            }
        }

        // Find burst patterns
        let groups = self.groups.read().await;
        for group in groups.values() {
            if group.recent_count >= 5 {
                patterns.push(ErrorPattern {
                    pattern_type: PatternType::Burst,
                    fingerprints: vec![group.fingerprint.clone()],
                    occurrence_count: group.recent_count,
                    description: format!("Error burst detected: {} occurrences in the last hour", group.recent_count),
                });
            }
        }

        patterns
    }
}

/// Error pattern detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub pattern_type: PatternType,
    pub fingerprints: Vec<String>,
    pub occurrence_count: u64,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Errors occurring together
    CoOccurrence,
    /// Burst of errors
    Burst,
    /// Recurring at specific times
    Recurring,
    /// Cascading failures
    Cascade,
}

/// Error tracking errors
#[derive(Debug, thiserror::Error)]
pub enum ErrorTrackingError {
    #[error("Collection failed: {0}")]
    CollectionFailed(String),

    #[error("Error not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::collector::EventCollector;
    use crate::analytics::config::AnalyticsConfigManager;

    async fn create_tracker() -> ErrorTracker {
        let config = AnalyticsConfigManager::new();
        let collector = Arc::new(EventCollector::new(config));
        ErrorTracker::new(collector)
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let tracker = create_tracker().await;

        let error = TrackedError::new("NETWORK_TIMEOUT", "Connection timed out", "backend")
            .with_severity(ErrorSeverity::Error)
            .with_context("url", serde_json::json!("https://api.example.com"));

        tracker.track(error).await.unwrap();

        let stats = tracker.get_stats().await;
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.by_category.get(&ErrorCategory::Network), Some(&1));
    }

    #[tokio::test]
    async fn test_error_grouping() {
        let tracker = create_tracker().await;

        // Track same error multiple times
        for _ in 0..5 {
            let error = TrackedError::new("AUTH_FAILED", "Invalid token", "auth");
            tracker.track(error).await.unwrap();
        }

        let groups = tracker.get_groups().await;
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].occurrence_count, 5);
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let tracker = create_tracker().await;

        let error = TrackedError::new("TEMP_ERROR", "Temporary failure", "service");
        let error_id = error.id.clone();
        tracker.track(error).await.unwrap();

        tracker.mark_recovered(&error_id, 100).await.unwrap();

        let errors = tracker.get_recent_errors(10).await;
        assert!(errors[0].recovered);
        assert_eq!(errors[0].recovery_time_ms, Some(100));
    }

    #[test]
    fn test_error_category_detection() {
        assert_eq!(ErrorCategory::from_code("NETWORK_TIMEOUT"), ErrorCategory::Network);
        assert_eq!(ErrorCategory::from_code("AUTH_FAILED"), ErrorCategory::Auth);
        assert_eq!(ErrorCategory::from_code("VALIDATION_ERROR"), ErrorCategory::Validation);
        assert_eq!(ErrorCategory::from_code("RANDOM_CODE"), ErrorCategory::Unknown);
    }

    #[test]
    fn test_fingerprint_consistency() {
        let fp1 = calculate_fingerprint("ERROR_CODE", "component");
        let fp2 = calculate_fingerprint("ERROR_CODE", "component");
        let fp3 = calculate_fingerprint("ERROR_CODE", "other_component");

        assert_eq!(fp1, fp2);
        assert_ne!(fp1, fp3);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Error tracking and storage
   - Fingerprint calculation
   - Category detection
   - Group statistics

2. **Integration Tests**
   - Full tracking pipeline
   - Alert triggering
   - Pattern detection

3. **Privacy Tests**
   - PII redaction in messages
   - Stack trace handling

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 413: Privacy Controls
- Spec 418: Performance Metrics
