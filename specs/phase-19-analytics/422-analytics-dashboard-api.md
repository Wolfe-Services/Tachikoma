# Spec 422: Dashboard API

## Phase
19 - Analytics/Telemetry

## Spec ID
422

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 410: Analytics Aggregation (aggregated data)
- Spec 421: Analytics Reports (report generation)

## Estimated Context
~10%

---

## Objective

Implement a comprehensive API for analytics dashboards, providing real-time and historical data endpoints for visualization, monitoring, and analysis interfaces.

---

## Acceptance Criteria

- [ ] Provide real-time metrics endpoints
- [ ] Support historical data queries
- [ ] Enable dashboard widget data fetching
- [ ] Implement WebSocket streaming for live data
- [ ] Support customizable time ranges
- [ ] Create aggregation endpoints
- [ ] Enable comparison queries
- [ ] Support dashboard state persistence

---

## Implementation Details

### Dashboard API

```rust
// src/analytics/dashboard_api.rs

use crate::analytics::aggregation::{AggregatedMetric, TimeGranularity};
use crate::analytics::backends::BackendStats;
use crate::analytics::costs::CostAggregation;
use crate::analytics::errors::ErrorStats;
use crate::analytics::performance::LatencyStats;
use crate::analytics::reports::{Report, ReportPeriod};
use crate::analytics::tokens::TokenAggregation;
use crate::analytics::trends::TrendAnalysis;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Dashboard widget types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    /// Single metric value
    Metric,
    /// Time series chart
    TimeSeries,
    /// Pie/donut chart
    Distribution,
    /// Bar chart
    BarChart,
    /// Table widget
    Table,
    /// Trend indicator
    Trend,
    /// Health status
    Health,
    /// Activity feed
    ActivityFeed,
}

/// Dashboard widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget identifier
    pub id: String,
    /// Widget type
    pub widget_type: WidgetType,
    /// Widget title
    pub title: String,
    /// Data source configuration
    pub data_source: DataSource,
    /// Widget position and size
    pub layout: WidgetLayout,
    /// Widget-specific options
    pub options: HashMap<String, serde_json::Value>,
}

/// Data source for a widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// Data source type
    pub source_type: DataSourceType,
    /// Metric names to fetch
    pub metrics: Vec<String>,
    /// Time granularity
    pub granularity: Option<TimeGranularity>,
    /// Filters to apply
    pub filters: HashMap<String, String>,
    /// Refresh interval in seconds
    pub refresh_interval: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceType {
    Metrics,
    Tokens,
    Costs,
    Errors,
    Performance,
    Backend,
    Missions,
    Custom,
}

/// Widget layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetLayout {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Dashboard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard identifier
    pub id: String,
    /// Dashboard name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Widgets in this dashboard
    pub widgets: Vec<Widget>,
    /// Default time range
    pub default_time_range: TimeRange,
    /// Auto-refresh interval in seconds
    pub auto_refresh: Option<u64>,
    /// Dashboard metadata
    pub metadata: DashboardMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardMetadata {
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
}

/// Time range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimeRange {
    #[serde(rename = "relative")]
    Relative { duration: String },
    #[serde(rename = "absolute")]
    Absolute {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

impl TimeRange {
    pub fn last_hour() -> Self {
        Self::Relative {
            duration: "1h".to_string(),
        }
    }

    pub fn last_day() -> Self {
        Self::Relative {
            duration: "24h".to_string(),
        }
    }

    pub fn last_week() -> Self {
        Self::Relative {
            duration: "7d".to_string(),
        }
    }

    pub fn to_period(&self) -> ReportPeriod {
        match self {
            Self::Relative { duration } => {
                let end = Utc::now();
                let start = parse_duration(duration)
                    .map(|d| end - d)
                    .unwrap_or(end - Duration::hours(1));
                ReportPeriod { start, end }
            }
            Self::Absolute { start, end } => ReportPeriod {
                start: *start,
                end: *end,
            },
        }
    }
}

fn parse_duration(s: &str) -> Option<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    match unit {
        "m" => Some(Duration::minutes(num)),
        "h" => Some(Duration::hours(num)),
        "d" => Some(Duration::days(num)),
        "w" => Some(Duration::weeks(num)),
        _ => None,
    }
}

/// Widget data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    /// Widget ID this data is for
    pub widget_id: String,
    /// Data payload
    pub data: WidgetDataPayload,
    /// When data was fetched
    pub fetched_at: DateTime<Utc>,
    /// Time range of data
    pub time_range: TimeRange,
}

/// Widget data payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum WidgetDataPayload {
    /// Single metric value
    Metric(MetricData),
    /// Time series data
    TimeSeries(TimeSeriesData),
    /// Distribution data
    Distribution(DistributionData),
    /// Table data
    Table(TableData),
    /// Trend data
    Trend(TrendData),
    /// Health status
    Health(HealthData),
    /// Activity feed
    Activity(ActivityData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub change: Option<f64>,
    pub change_period: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub series: Vec<SeriesData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesData {
    pub name: String,
    pub points: Vec<(DateTime<Utc>, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub segments: Vec<DistributionSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSegment {
    pub label: String,
    pub value: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub label: String,
    pub data_type: String,
    pub sortable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub direction: String,
    pub change_percent: f64,
    pub current_value: f64,
    pub previous_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthData {
    pub status: String,
    pub components: Vec<ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    pub items: Vec<ActivityItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub message: String,
    pub severity: String,
}

/// API query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    /// Time range
    pub time_range: TimeRange,
    /// Granularity
    pub granularity: Option<TimeGranularity>,
    /// Filters
    pub filters: HashMap<String, String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            time_range: TimeRange::last_hour(),
            granularity: None,
            filters: HashMap::new(),
            limit: None,
            offset: None,
        }
    }
}

/// Real-time update message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeUpdate {
    /// Update type
    pub update_type: UpdateType,
    /// Widget ID if applicable
    pub widget_id: Option<String>,
    /// Update payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    MetricUpdate,
    EventReceived,
    AlertTriggered,
    StatusChange,
}

/// Dashboard API service
pub struct DashboardApi {
    /// Dashboard storage
    dashboards: Arc<RwLock<HashMap<String, Dashboard>>>,
    /// Realtime broadcast channel
    realtime_tx: broadcast::Sender<RealtimeUpdate>,
    /// Data providers
    data_provider: Arc<DataProvider>,
}

impl DashboardApi {
    pub fn new(data_provider: Arc<DataProvider>) -> Self {
        let (realtime_tx, _) = broadcast::channel(1000);

        Self {
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            realtime_tx,
            data_provider,
        }
    }

    /// Create a new dashboard
    pub async fn create_dashboard(&self, dashboard: Dashboard) -> Result<String, ApiError> {
        let id = dashboard.id.clone();

        let mut dashboards = self.dashboards.write().await;
        if dashboards.contains_key(&id) {
            return Err(ApiError::AlreadyExists(id));
        }

        dashboards.insert(id.clone(), dashboard);
        Ok(id)
    }

    /// Get a dashboard by ID
    pub async fn get_dashboard(&self, id: &str) -> Result<Dashboard, ApiError> {
        let dashboards = self.dashboards.read().await;
        dashboards
            .get(id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(id.to_string()))
    }

    /// List all dashboards
    pub async fn list_dashboards(&self) -> Vec<Dashboard> {
        self.dashboards.read().await.values().cloned().collect()
    }

    /// Update a dashboard
    pub async fn update_dashboard(&self, dashboard: Dashboard) -> Result<(), ApiError> {
        let mut dashboards = self.dashboards.write().await;
        if !dashboards.contains_key(&dashboard.id) {
            return Err(ApiError::NotFound(dashboard.id.clone()));
        }

        dashboards.insert(dashboard.id.clone(), dashboard);
        Ok(())
    }

    /// Delete a dashboard
    pub async fn delete_dashboard(&self, id: &str) -> Result<(), ApiError> {
        let mut dashboards = self.dashboards.write().await;
        dashboards
            .remove(id)
            .ok_or_else(|| ApiError::NotFound(id.to_string()))?;
        Ok(())
    }

    /// Fetch data for a widget
    pub async fn fetch_widget_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetData, ApiError> {
        let period = params.time_range.to_period();

        let payload = match widget.data_source.source_type {
            DataSourceType::Metrics => {
                self.data_provider.fetch_metrics_data(widget, params).await?
            }
            DataSourceType::Tokens => {
                self.data_provider.fetch_token_data(widget, params).await?
            }
            DataSourceType::Costs => {
                self.data_provider.fetch_cost_data(widget, params).await?
            }
            DataSourceType::Errors => {
                self.data_provider.fetch_error_data(widget, params).await?
            }
            DataSourceType::Performance => {
                self.data_provider.fetch_performance_data(widget, params).await?
            }
            DataSourceType::Backend => {
                self.data_provider.fetch_backend_data(widget, params).await?
            }
            DataSourceType::Missions => {
                self.data_provider.fetch_mission_data(widget, params).await?
            }
            DataSourceType::Custom => {
                return Err(ApiError::InvalidRequest("Custom data sources not supported".to_string()));
            }
        };

        Ok(WidgetData {
            widget_id: widget.id.clone(),
            data: payload,
            fetched_at: Utc::now(),
            time_range: params.time_range.clone(),
        })
    }

    /// Fetch data for all widgets in a dashboard
    pub async fn fetch_dashboard_data(
        &self,
        dashboard_id: &str,
        params: &QueryParams,
    ) -> Result<Vec<WidgetData>, ApiError> {
        let dashboard = self.get_dashboard(dashboard_id).await?;

        let mut data = Vec::new();
        for widget in &dashboard.widgets {
            match self.fetch_widget_data(widget, params).await {
                Ok(widget_data) => data.push(widget_data),
                Err(e) => {
                    tracing::warn!("Failed to fetch data for widget {}: {}", widget.id, e);
                }
            }
        }

        Ok(data)
    }

    /// Subscribe to real-time updates
    pub fn subscribe(&self) -> broadcast::Receiver<RealtimeUpdate> {
        self.realtime_tx.subscribe()
    }

    /// Publish a real-time update
    pub fn publish_update(&self, update: RealtimeUpdate) {
        let _ = self.realtime_tx.send(update);
    }

    /// Get summary metrics
    pub async fn get_summary(&self, params: &QueryParams) -> Result<SummaryResponse, ApiError> {
        let period = params.time_range.to_period();

        Ok(SummaryResponse {
            tokens: self.data_provider.get_token_summary(&period).await?,
            costs: self.data_provider.get_cost_summary(&period).await?,
            errors: self.data_provider.get_error_summary(&period).await?,
            performance: self.data_provider.get_performance_summary(&period).await?,
            time_range: params.time_range.clone(),
            generated_at: Utc::now(),
        })
    }
}

/// Summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub tokens: TokenSummary,
    pub costs: CostSummary,
    pub errors: ErrorSummary,
    pub performance: PerformanceSummary,
    pub time_range: TimeRange,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub total: u64,
    pub input: u64,
    pub output: u64,
    pub change_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_usd: f64,
    pub by_provider: HashMap<String, f64>,
    pub change_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub total: u64,
    pub error_rate: f64,
    pub top_errors: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub requests_per_second: f64,
}

/// Data provider for dashboard API
pub struct DataProvider {
    // Would hold references to various analytics services
}

impl DataProvider {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn fetch_metrics_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        // Mock implementation
        Ok(WidgetDataPayload::Metric(MetricData {
            name: widget.data_source.metrics.first().cloned().unwrap_or_default(),
            value: 100.0,
            unit: "count".to_string(),
            change: Some(5.0),
            change_period: Some("vs last period".to_string()),
        }))
    }

    pub async fn fetch_token_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::TimeSeries(TimeSeriesData {
            series: vec![SeriesData {
                name: "tokens".to_string(),
                points: vec![],
            }],
        }))
    }

    pub async fn fetch_cost_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::Distribution(DistributionData {
            segments: vec![
                DistributionSegment {
                    label: "Anthropic".to_string(),
                    value: 75.0,
                    percentage: 75.0,
                },
                DistributionSegment {
                    label: "OpenAI".to_string(),
                    value: 25.0,
                    percentage: 25.0,
                },
            ],
        }))
    }

    pub async fn fetch_error_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::Table(TableData {
            columns: vec![
                TableColumn {
                    key: "code".to_string(),
                    label: "Error Code".to_string(),
                    data_type: "string".to_string(),
                    sortable: true,
                },
                TableColumn {
                    key: "count".to_string(),
                    label: "Count".to_string(),
                    data_type: "number".to_string(),
                    sortable: true,
                },
            ],
            rows: vec![],
            total_rows: 0,
        }))
    }

    pub async fn fetch_performance_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::Metric(MetricData {
            name: "latency".to_string(),
            value: 250.0,
            unit: "ms".to_string(),
            change: None,
            change_period: None,
        }))
    }

    pub async fn fetch_backend_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::Health(HealthData {
            status: "healthy".to_string(),
            components: vec![
                ComponentHealth {
                    name: "Anthropic".to_string(),
                    status: "healthy".to_string(),
                    message: None,
                },
            ],
        }))
    }

    pub async fn fetch_mission_data(
        &self,
        widget: &Widget,
        params: &QueryParams,
    ) -> Result<WidgetDataPayload, ApiError> {
        Ok(WidgetDataPayload::Activity(ActivityData {
            items: vec![],
        }))
    }

    pub async fn get_token_summary(&self, period: &ReportPeriod) -> Result<TokenSummary, ApiError> {
        Ok(TokenSummary {
            total: 1000000,
            input: 600000,
            output: 400000,
            change_percent: Some(10.0),
        })
    }

    pub async fn get_cost_summary(&self, period: &ReportPeriod) -> Result<CostSummary, ApiError> {
        Ok(CostSummary {
            total_usd: 50.0,
            by_provider: [("anthropic".to_string(), 40.0), ("openai".to_string(), 10.0)]
                .into_iter()
                .collect(),
            change_percent: Some(5.0),
        })
    }

    pub async fn get_error_summary(&self, period: &ReportPeriod) -> Result<ErrorSummary, ApiError> {
        Ok(ErrorSummary {
            total: 25,
            error_rate: 0.01,
            top_errors: vec![],
        })
    }

    pub async fn get_performance_summary(
        &self,
        period: &ReportPeriod,
    ) -> Result<PerformanceSummary, ApiError> {
        Ok(PerformanceSummary {
            avg_latency_ms: 250.0,
            p99_latency_ms: 1200.0,
            requests_per_second: 10.0,
        })
    }
}

impl Default for DataProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// API errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Create default dashboard
pub fn create_default_dashboard() -> Dashboard {
    Dashboard {
        id: "default".to_string(),
        name: "Overview Dashboard".to_string(),
        description: Some("Main analytics overview".to_string()),
        widgets: vec![
            Widget {
                id: "total-tokens".to_string(),
                widget_type: WidgetType::Metric,
                title: "Total Tokens".to_string(),
                data_source: DataSource {
                    source_type: DataSourceType::Tokens,
                    metrics: vec!["total".to_string()],
                    granularity: None,
                    filters: HashMap::new(),
                    refresh_interval: Some(60),
                },
                layout: WidgetLayout {
                    x: 0,
                    y: 0,
                    width: 3,
                    height: 2,
                },
                options: HashMap::new(),
            },
            Widget {
                id: "total-cost".to_string(),
                widget_type: WidgetType::Metric,
                title: "Total Cost".to_string(),
                data_source: DataSource {
                    source_type: DataSourceType::Costs,
                    metrics: vec!["total".to_string()],
                    granularity: None,
                    filters: HashMap::new(),
                    refresh_interval: Some(60),
                },
                layout: WidgetLayout {
                    x: 3,
                    y: 0,
                    width: 3,
                    height: 2,
                },
                options: HashMap::new(),
            },
            Widget {
                id: "cost-distribution".to_string(),
                widget_type: WidgetType::Distribution,
                title: "Cost by Provider".to_string(),
                data_source: DataSource {
                    source_type: DataSourceType::Costs,
                    metrics: vec!["by_provider".to_string()],
                    granularity: None,
                    filters: HashMap::new(),
                    refresh_interval: Some(300),
                },
                layout: WidgetLayout {
                    x: 6,
                    y: 0,
                    width: 6,
                    height: 4,
                },
                options: HashMap::new(),
            },
        ],
        default_time_range: TimeRange::last_day(),
        auto_refresh: Some(60),
        metadata: DashboardMetadata::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_dashboard() {
        let data_provider = Arc::new(DataProvider::new());
        let api = DashboardApi::new(data_provider);

        let dashboard = create_default_dashboard();
        let id = api.create_dashboard(dashboard.clone()).await.unwrap();

        assert_eq!(id, "default");

        let retrieved = api.get_dashboard(&id).await.unwrap();
        assert_eq!(retrieved.name, "Overview Dashboard");
    }

    #[tokio::test]
    async fn test_fetch_widget_data() {
        let data_provider = Arc::new(DataProvider::new());
        let api = DashboardApi::new(data_provider);

        let widget = Widget {
            id: "test".to_string(),
            widget_type: WidgetType::Metric,
            title: "Test".to_string(),
            data_source: DataSource {
                source_type: DataSourceType::Metrics,
                metrics: vec!["count".to_string()],
                granularity: None,
                filters: HashMap::new(),
                refresh_interval: None,
            },
            layout: WidgetLayout {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            options: HashMap::new(),
        };

        let data = api
            .fetch_widget_data(&widget, &QueryParams::default())
            .await
            .unwrap();

        assert_eq!(data.widget_id, "test");
    }

    #[test]
    fn test_time_range_parsing() {
        let range = TimeRange::last_hour();
        let period = range.to_period();
        let duration = period.end - period.start;

        assert!(duration >= Duration::minutes(59));
        assert!(duration <= Duration::minutes(61));
    }

    #[test]
    fn test_duration_parsing() {
        assert_eq!(parse_duration("1h"), Some(Duration::hours(1)));
        assert_eq!(parse_duration("24h"), Some(Duration::hours(24)));
        assert_eq!(parse_duration("7d"), Some(Duration::days(7)));
        assert_eq!(parse_duration("2w"), Some(Duration::weeks(2)));
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Dashboard CRUD operations
   - Widget data fetching
   - Time range parsing
   - Query parameter handling

2. **Integration Tests**
   - Full API workflow
   - Real-time subscriptions
   - Dashboard state persistence

3. **API Tests**
   - Response format validation
   - Error handling

---

## Related Specs

- Spec 410: Analytics Aggregation
- Spec 421: Report Generation
- Spec 423: Export Formats
