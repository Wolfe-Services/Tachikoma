# Spec 410: Analytics Aggregation

## Phase
19 - Analytics/Telemetry

## Spec ID
410

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 409: Analytics Storage (data persistence)

## Estimated Context
~10%

---

## Objective

Implement analytics aggregation system that computes summary statistics, time-series data, and derived metrics from raw analytics events for efficient querying and reporting.

---

## Acceptance Criteria

- [ ] Implement time-based aggregation (hourly, daily, weekly)
- [ ] Create metric aggregation (sum, avg, min, max, percentiles)
- [ ] Support dimensional aggregation (by category, type, session)
- [ ] Implement incremental aggregation for efficiency
- [ ] Create pre-computed rollups for common queries
- [ ] Support custom aggregation functions
- [ ] Implement aggregation scheduling
- [ ] Create aggregation state persistence

---

## Implementation Details

### Aggregation System

```rust
// src/analytics/aggregation.rs

use crate::analytics::storage::{AnalyticsStorage, StorageError};
use crate::analytics::types::{
    AnalyticsEvent, BusinessEventData, EventCategory, EventData, EventType,
    PerformanceEventData,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Time granularity for aggregations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeGranularity {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl TimeGranularity {
    /// Get the duration for this granularity
    pub fn duration(&self) -> Duration {
        match self {
            Self::Minute => Duration::minutes(1),
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
            Self::Month => Duration::days(30), // Approximate
        }
    }

    /// Truncate a timestamp to this granularity
    pub fn truncate(&self, dt: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Self::Minute => dt
                .with_second(0)
                .and_then(|d| d.with_nanosecond(0))
                .unwrap_or(dt),
            Self::Hour => dt
                .with_minute(0)
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap_or(dt),
            Self::Day => dt
                .with_hour(0)
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap_or(dt),
            Self::Week => {
                let days_since_monday = dt.weekday().num_days_from_monday();
                (dt - Duration::days(days_since_monday as i64))
                    .with_hour(0)
                    .and_then(|d| d.with_minute(0))
                    .and_then(|d| d.with_second(0))
                    .and_then(|d| d.with_nanosecond(0))
                    .unwrap_or(dt)
            }
            Self::Month => dt
                .with_day(1)
                .and_then(|d| d.with_hour(0))
                .and_then(|d| d.with_minute(0))
                .and_then(|d| d.with_second(0))
                .and_then(|d| d.with_nanosecond(0))
                .unwrap_or(dt),
        }
    }
}

/// Aggregation function types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationFunction {
    Count,
    Sum,
    Average,
    Min,
    Max,
    Percentile50,
    Percentile90,
    Percentile99,
}

/// A single aggregated metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric name
    pub name: String,
    /// Aggregation function used
    pub function: AggregationFunction,
    /// Computed value
    pub value: f64,
    /// Number of data points
    pub count: u64,
    /// Time period start
    pub period_start: DateTime<Utc>,
    /// Time period end
    pub period_end: DateTime<Utc>,
    /// Dimensions for this metric
    pub dimensions: HashMap<String, String>,
}

/// Aggregation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationSpec {
    /// Unique identifier for this aggregation
    pub id: String,
    /// Name of the aggregation
    pub name: String,
    /// Event types to aggregate
    pub event_types: Vec<EventType>,
    /// Event categories to aggregate
    pub categories: Vec<EventCategory>,
    /// Time granularity
    pub granularity: TimeGranularity,
    /// Aggregation functions to compute
    pub functions: Vec<AggregationFunction>,
    /// Dimensions to group by
    pub dimensions: Vec<String>,
    /// Field to aggregate (for numeric aggregations)
    pub value_field: Option<String>,
}

impl AggregationSpec {
    /// Create a count aggregation spec
    pub fn count(id: &str, name: &str, granularity: TimeGranularity) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            event_types: vec![],
            categories: vec![],
            granularity,
            functions: vec![AggregationFunction::Count],
            dimensions: vec![],
            value_field: None,
        }
    }

    /// Add event type filter
    pub fn with_event_types(mut self, types: Vec<EventType>) -> Self {
        self.event_types = types;
        self
    }

    /// Add category filter
    pub fn with_categories(mut self, categories: Vec<EventCategory>) -> Self {
        self.categories = categories;
        self
    }

    /// Add dimension grouping
    pub fn with_dimensions(mut self, dimensions: Vec<String>) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Set the value field for numeric aggregations
    pub fn with_value_field(mut self, field: &str) -> Self {
        self.value_field = Some(field.to_string());
        self
    }

    /// Add aggregation functions
    pub fn with_functions(mut self, functions: Vec<AggregationFunction>) -> Self {
        self.functions = functions;
        self
    }
}

/// Running statistics accumulator
#[derive(Debug, Clone, Default)]
pub struct StatsAccumulator {
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    values: Vec<f64>, // For percentile calculations
}

impl StatsAccumulator {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.values.push(value);
    }

    pub fn merge(&mut self, other: &StatsAccumulator) {
        self.count += other.count;
        self.sum += other.sum;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.values.extend(&other.values);
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn sum(&self) -> f64 {
        self.sum
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn min(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.min
        }
    }

    pub fn max(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.max
        }
    }

    pub fn percentile(&mut self, p: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }

        self.values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((p / 100.0) * (self.values.len() - 1) as f64).round() as usize;
        self.values[idx.min(self.values.len() - 1)]
    }

    pub fn compute(&mut self, function: AggregationFunction) -> f64 {
        match function {
            AggregationFunction::Count => self.count as f64,
            AggregationFunction::Sum => self.sum(),
            AggregationFunction::Average => self.average(),
            AggregationFunction::Min => self.min(),
            AggregationFunction::Max => self.max(),
            AggregationFunction::Percentile50 => self.percentile(50.0),
            AggregationFunction::Percentile90 => self.percentile(90.0),
            AggregationFunction::Percentile99 => self.percentile(99.0),
        }
    }
}

/// Aggregation bucket key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BucketKey {
    period_start: DateTime<Utc>,
    dimensions: Vec<(String, String)>,
}

/// Analytics aggregator
pub struct Aggregator {
    /// Storage backend
    storage: Arc<dyn AnalyticsStorage>,

    /// Registered aggregation specs
    specs: Arc<RwLock<HashMap<String, AggregationSpec>>>,

    /// Cached aggregation results
    cache: Arc<RwLock<HashMap<String, Vec<AggregatedMetric>>>>,

    /// Last aggregation time per spec
    last_aggregation: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl Aggregator {
    pub fn new(storage: Arc<dyn AnalyticsStorage>) -> Self {
        Self {
            storage,
            specs: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            last_aggregation: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an aggregation specification
    pub async fn register_spec(&self, spec: AggregationSpec) {
        let mut specs = self.specs.write().await;
        specs.insert(spec.id.clone(), spec);
    }

    /// Unregister an aggregation specification
    pub async fn unregister_spec(&self, id: &str) {
        let mut specs = self.specs.write().await;
        specs.remove(id);
    }

    /// Run aggregation for a specific spec
    pub async fn aggregate(
        &self,
        spec_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AggregatedMetric>, AggregationError> {
        let specs = self.specs.read().await;
        let spec = specs
            .get(spec_id)
            .ok_or_else(|| AggregationError::SpecNotFound(spec_id.to_string()))?
            .clone();
        drop(specs);

        self.run_aggregation(&spec, start, end).await
    }

    /// Run all registered aggregations
    pub async fn aggregate_all(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<String, Vec<AggregatedMetric>>, AggregationError> {
        let specs = self.specs.read().await;
        let spec_list: Vec<_> = specs.values().cloned().collect();
        drop(specs);

        let mut results = HashMap::new();

        for spec in spec_list {
            let metrics = self.run_aggregation(&spec, start, end).await?;
            results.insert(spec.id.clone(), metrics);
        }

        // Update cache
        let mut cache = self.cache.write().await;
        for (id, metrics) in &results {
            cache.insert(id.clone(), metrics.clone());
        }

        Ok(results)
    }

    async fn run_aggregation(
        &self,
        spec: &AggregationSpec,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AggregatedMetric>, AggregationError> {
        // Fetch events from storage
        let events = self
            .storage
            .query_by_time(start, end, None)
            .await
            .map_err(|e| AggregationError::StorageError(e.to_string()))?;

        // Filter events based on spec
        let filtered_events: Vec<_> = events
            .into_iter()
            .filter(|e| {
                (spec.event_types.is_empty() || spec.event_types.contains(&e.event_type))
                    && (spec.categories.is_empty() || spec.categories.contains(&e.category))
            })
            .collect();

        // Group events into buckets
        let mut buckets: HashMap<BucketKey, StatsAccumulator> = HashMap::new();

        for event in filtered_events {
            let period_start = spec.granularity.truncate(event.timestamp);

            // Extract dimensions
            let dimensions = self.extract_dimensions(&event, &spec.dimensions);

            let key = BucketKey {
                period_start,
                dimensions,
            };

            let accumulator = buckets.entry(key).or_insert_with(StatsAccumulator::new);

            // Extract value
            let value = self.extract_value(&event, spec.value_field.as_deref());
            accumulator.add(value);
        }

        // Compute aggregated metrics
        let mut metrics = Vec::new();

        for (key, mut accumulator) in buckets {
            let period_end = key.period_start + spec.granularity.duration();

            for function in &spec.functions {
                let value = accumulator.compute(*function);

                metrics.push(AggregatedMetric {
                    name: spec.name.clone(),
                    function: *function,
                    value,
                    count: accumulator.count(),
                    period_start: key.period_start,
                    period_end,
                    dimensions: key.dimensions.iter().cloned().collect(),
                });
            }
        }

        // Sort by period start
        metrics.sort_by_key(|m| m.period_start);

        // Update last aggregation time
        let mut last_agg = self.last_aggregation.write().await;
        last_agg.insert(spec.id.clone(), Utc::now());

        Ok(metrics)
    }

    fn extract_dimensions(
        &self,
        event: &AnalyticsEvent,
        dimension_names: &[String],
    ) -> Vec<(String, String)> {
        let mut dimensions = Vec::new();

        for name in dimension_names {
            let value = match name.as_str() {
                "category" => Some(format!("{:?}", event.category)),
                "event_type" => Some(format!("{:?}", event.event_type)),
                "session_id" => event.session_id.map(|id| id.to_string()),
                "priority" => Some(format!("{:?}", event.priority)),
                _ => {
                    // Try to extract from metadata
                    event
                        .metadata
                        .custom
                        .get(name)
                        .and_then(|v| v.as_str().map(String::from))
                }
            };

            if let Some(v) = value {
                dimensions.push((name.clone(), v));
            }
        }

        dimensions
    }

    fn extract_value(&self, event: &AnalyticsEvent, field: Option<&str>) -> f64 {
        match (&event.data, field) {
            (EventData::Performance(data), Some("value")) => data.value,
            (EventData::Performance(data), None) => data.value,
            (EventData::Business(data), Some("value")) => data.value,
            (EventData::Business(data), None) => data.value,
            (EventData::Usage(data), Some("duration_ms")) => data.duration_ms.unwrap_or(0) as f64,
            _ => 1.0, // Default to 1 for counting
        }
    }

    /// Get cached aggregation results
    pub async fn get_cached(&self, spec_id: &str) -> Option<Vec<AggregatedMetric>> {
        let cache = self.cache.read().await;
        cache.get(spec_id).cloned()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get time series data for a metric
    pub async fn time_series(
        &self,
        spec_id: &str,
        function: AggregationFunction,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, f64)>, AggregationError> {
        let metrics = self.aggregate(spec_id, start, end).await?;

        let series: Vec<_> = metrics
            .into_iter()
            .filter(|m| m.function == function)
            .map(|m| (m.period_start, m.value))
            .collect();

        Ok(series)
    }
}

/// Aggregation errors
#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    #[error("Aggregation spec not found: {0}")]
    SpecNotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Pre-defined aggregation specs for common use cases
pub fn default_aggregation_specs() -> Vec<AggregationSpec> {
    vec![
        // Event counts by category (hourly)
        AggregationSpec::count("event_counts_hourly", "Event Counts (Hourly)", TimeGranularity::Hour)
            .with_dimensions(vec!["category".to_string()]),
        // Event counts by type (daily)
        AggregationSpec::count("event_counts_daily", "Event Counts (Daily)", TimeGranularity::Day)
            .with_dimensions(vec!["category".to_string(), "event_type".to_string()]),
        // Token usage (daily)
        AggregationSpec::count("tokens_daily", "Token Usage (Daily)", TimeGranularity::Day)
            .with_event_types(vec![EventType::TokensConsumed])
            .with_functions(vec![
                AggregationFunction::Sum,
                AggregationFunction::Average,
                AggregationFunction::Max,
            ])
            .with_value_field("value"),
        // Response latency (hourly)
        AggregationSpec::count("latency_hourly", "Response Latency (Hourly)", TimeGranularity::Hour)
            .with_event_types(vec![EventType::ResponseLatency])
            .with_functions(vec![
                AggregationFunction::Average,
                AggregationFunction::Percentile50,
                AggregationFunction::Percentile90,
                AggregationFunction::Percentile99,
            ])
            .with_value_field("value"),
        // Error rates (daily)
        AggregationSpec::count("errors_daily", "Error Counts (Daily)", TimeGranularity::Day)
            .with_categories(vec![EventCategory::Error])
            .with_dimensions(vec!["event_type".to_string()]),
        // Mission completion (daily)
        AggregationSpec::count("missions_daily", "Mission Stats (Daily)", TimeGranularity::Day)
            .with_event_types(vec![
                EventType::MissionCreated,
                EventType::MissionCompleted,
                EventType::MissionFailed,
            ])
            .with_dimensions(vec!["event_type".to_string()]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::storage::SqliteAnalyticsStorage;
    use crate::analytics::config::StorageConfig;
    use crate::analytics::types::{EventBatch, EventBuilder};

    async fn create_test_aggregator() -> Aggregator {
        let storage = Arc::new(
            SqliteAnalyticsStorage::in_memory(StorageConfig::default()).unwrap()
        );
        Aggregator::new(storage)
    }

    #[tokio::test]
    async fn test_simple_count_aggregation() {
        let aggregator = create_test_aggregator().await;

        // Register a simple count spec
        let spec = AggregationSpec::count("test", "Test Count", TimeGranularity::Hour);
        aggregator.register_spec(spec).await;

        // Store some test events
        let storage = aggregator.storage.clone();
        let events: Vec<_> = (0..10)
            .map(|_| EventBuilder::new(EventType::FeatureUsed).build())
            .collect();
        let batch = EventBatch::new(events, 1);
        storage.store_batch(&batch).await.unwrap();

        // Run aggregation
        let start = Utc::now() - Duration::hours(1);
        let end = Utc::now() + Duration::hours(1);
        let results = aggregator.aggregate("test", start, end).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].count, 10);
    }

    #[tokio::test]
    async fn test_stats_accumulator() {
        let mut acc = StatsAccumulator::new();

        acc.add(1.0);
        acc.add(2.0);
        acc.add(3.0);
        acc.add(4.0);
        acc.add(5.0);

        assert_eq!(acc.count(), 5);
        assert_eq!(acc.sum(), 15.0);
        assert_eq!(acc.average(), 3.0);
        assert_eq!(acc.min(), 1.0);
        assert_eq!(acc.max(), 5.0);
        assert_eq!(acc.percentile(50.0), 3.0);
    }

    #[tokio::test]
    async fn test_time_granularity_truncate() {
        let dt = DateTime::parse_from_rfc3339("2024-03-15T14:30:45Z")
            .unwrap()
            .with_timezone(&Utc);

        let hourly = TimeGranularity::Hour.truncate(dt);
        assert_eq!(hourly.hour(), 14);
        assert_eq!(hourly.minute(), 0);

        let daily = TimeGranularity::Day.truncate(dt);
        assert_eq!(daily.hour(), 0);
        assert_eq!(daily.day(), 15);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Stats accumulator correctness
   - Time truncation accuracy
   - Dimension extraction
   - Value extraction from various event types

2. **Integration Tests**
   - Full aggregation pipeline
   - Multiple concurrent aggregations
   - Large dataset aggregation

3. **Performance Tests**
   - Aggregation speed with large datasets
   - Memory usage during aggregation

---

## Related Specs

- Spec 406: Analytics Types
- Spec 409: Analytics Storage
- Spec 420: Trend Analysis
- Spec 421: Report Generation
