# 423 - Analytics Query

## Overview

Query interface for analytics data with support for trends, funnels, retention, and custom queries with property filtering.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/query.rs

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Query types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnalyticsQuery {
    Trends(TrendsQuery),
    Funnel(FunnelQuery),
    Retention(RetentionQuery),
    Paths(PathsQuery),
    Stickiness(StickinessQuery),
    Lifecycle(LifecycleQuery),
    Events(EventsQuery),
}

/// Trends query - event counts over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsQuery {
    /// Events to query
    pub events: Vec<EventQuery>,
    /// Date range
    pub date_range: DateRange,
    /// Time granularity
    pub interval: QueryInterval,
    /// Property filters
    pub filters: Vec<PropertyFilter>,
    /// Breakdown by property
    pub breakdown: Option<BreakdownSpec>,
    /// Compare to previous period
    pub compare: bool,
    /// Formula for combining events
    pub formula: Option<String>,
}

/// Single event query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQuery {
    /// Event name (or $pageview, $autocapture, etc.)
    pub event: String,
    /// Display name
    pub name: Option<String>,
    /// Aggregation type
    pub math: MathType,
    /// Property to aggregate (for property-based math)
    pub math_property: Option<String>,
    /// Event-specific filters
    pub filters: Vec<PropertyFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MathType {
    /// Count of events
    Total,
    /// Count of unique users
    UniqueUsers,
    /// Daily/weekly/monthly active users
    Dau,
    Wau,
    Mau,
    /// Property aggregations
    Sum,
    Avg,
    Min,
    Max,
    Median,
    P90,
    P95,
    P99,
}

/// Date range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub start: DateTime<Utc>,
    /// End date (inclusive)
    pub end: DateTime<Utc>,
}

impl DateRange {
    pub fn last_n_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::days(days);
        Self { start, end }
    }

    pub fn last_n_weeks(weeks: i64) -> Self {
        Self::last_n_days(weeks * 7)
    }

    pub fn last_n_months(months: i64) -> Self {
        Self::last_n_days(months * 30)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryInterval {
    Hour,
    Day,
    Week,
    Month,
}

/// Property filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFilter {
    /// Property key
    pub key: String,
    /// Property type
    pub property_type: PropertyType,
    /// Operator
    pub operator: FilterOperator,
    /// Value(s) to match
    pub value: FilterValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    Event,
    Person,
    Session,
    Group,
    Cohort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Exact,
    IsNot,
    Contains,
    NotContains,
    Regex,
    IsSet,
    IsNotSet,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Between,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
    Range { min: f64, max: f64 },
}

/// Breakdown specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakdownSpec {
    /// Property to break down by
    pub property: String,
    /// Property type
    pub property_type: PropertyType,
    /// Limit number of breakdown values
    pub limit: Option<u32>,
}

/// Funnel query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelQuery {
    /// Funnel steps
    pub steps: Vec<FunnelStep>,
    /// Date range
    pub date_range: DateRange,
    /// Conversion window in seconds
    pub conversion_window_seconds: u64,
    /// Funnel ordering
    pub funnel_order: FunnelOrder,
    /// Breakdown
    pub breakdown: Option<BreakdownSpec>,
    /// Filters
    pub filters: Vec<PropertyFilter>,
    /// Exclusion steps
    pub exclusions: Vec<FunnelStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStep {
    /// Event name
    pub event: String,
    /// Step name
    pub name: Option<String>,
    /// Step-specific filters
    pub filters: Vec<PropertyFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunnelOrder {
    /// Steps must occur in exact order
    Strict,
    /// Steps can have other events in between
    Unordered,
}

/// Retention query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionQuery {
    /// Cohort entry event
    pub target_event: EventQuery,
    /// Return event
    pub returning_event: EventQuery,
    /// Date range
    pub date_range: DateRange,
    /// Retention period type
    pub period: RetentionPeriod,
    /// Number of periods
    pub total_periods: u32,
    /// Filters
    pub filters: Vec<PropertyFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPeriod {
    Day,
    Week,
    Month,
}

/// Paths query - user journey analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsQuery {
    /// Path type
    pub path_type: PathType,
    /// Start point (event or URL)
    pub start_point: Option<String>,
    /// End point (event or URL)
    pub end_point: Option<String>,
    /// Date range
    pub date_range: DateRange,
    /// Max path length
    pub max_depth: u32,
    /// Min path count
    pub min_count: u32,
    /// Filters
    pub filters: Vec<PropertyFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    Pageview,
    Event,
    CustomEvent,
}

/// Stickiness query - how often users perform action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickinessQuery {
    /// Event to analyze
    pub event: EventQuery,
    /// Date range
    pub date_range: DateRange,
    /// Interval for counting (day/week/month)
    pub interval: QueryInterval,
    /// Filters
    pub filters: Vec<PropertyFilter>,
}

/// Lifecycle query - new, returning, dormant users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleQuery {
    /// Event that defines activity
    pub event: EventQuery,
    /// Date range
    pub date_range: DateRange,
    /// Interval
    pub interval: QueryInterval,
    /// Filters
    pub filters: Vec<PropertyFilter>,
}

/// Raw events query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsQuery {
    /// Events to include (empty = all)
    pub events: Vec<String>,
    /// Date range
    pub date_range: DateRange,
    /// Filters
    pub filters: Vec<PropertyFilter>,
    /// Properties to select
    pub select: Vec<String>,
    /// Order by
    pub order_by: Option<OrderBy>,
    /// Limit
    pub limit: u32,
    /// Offset
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub property: String,
    pub direction: OrderDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// Query results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryResult {
    Trends(TrendsResult),
    Funnel(FunnelResult),
    Retention(RetentionResult),
    Paths(PathsResult),
    Stickiness(StickinessResult),
    Lifecycle(LifecycleResult),
    Events(EventsResult),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsResult {
    /// Series data
    pub series: Vec<TrendsSeries>,
    /// Labels (dates/times)
    pub labels: Vec<String>,
    /// Compare data (if requested)
    pub compare: Option<Vec<TrendsSeries>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsSeries {
    /// Event name
    pub event: String,
    /// Display label
    pub label: String,
    /// Data points
    pub data: Vec<f64>,
    /// Breakdown key (if breakdown applied)
    pub breakdown_value: Option<String>,
    /// Count or aggregate value
    pub count: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelResult {
    /// Steps with conversion data
    pub steps: Vec<FunnelStepResult>,
    /// Overall conversion rate
    pub conversion_rate: f64,
    /// Average conversion time
    pub avg_conversion_time_seconds: Option<f64>,
    /// Breakdown results
    pub breakdown: Option<Vec<FunnelBreakdown>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelStepResult {
    pub name: String,
    pub count: u64,
    pub conversion_rate_from_previous: f64,
    pub conversion_rate_from_first: f64,
    pub avg_time_from_previous_seconds: Option<f64>,
    pub drop_off_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunnelBreakdown {
    pub breakdown_value: String,
    pub steps: Vec<FunnelStepResult>,
    pub conversion_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionResult {
    /// Cohort data
    pub cohorts: Vec<CohortData>,
    /// Overall retention by period
    pub overall_retention: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortData {
    /// Cohort date
    pub date: String,
    /// Cohort size
    pub size: u64,
    /// Retention per period
    pub retention: Vec<RetentionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPoint {
    pub period: u32,
    pub count: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsResult {
    /// Path nodes
    pub nodes: Vec<PathNode>,
    /// Path links
    pub links: Vec<PathLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathNode {
    pub id: String,
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathLink {
    pub source: String,
    pub target: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickinessResult {
    /// Distribution of user activity
    pub buckets: Vec<StickinessBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickinessBucket {
    /// Days active in period
    pub days_active: u32,
    /// Number of users
    pub users: u64,
    /// Percentage
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResult {
    /// Data per interval
    pub data: Vec<LifecycleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleData {
    pub date: String,
    pub new: u64,
    pub returning: u64,
    pub resurrecting: u64,
    pub dormant: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResult {
    /// Events
    pub events: Vec<EventRow>,
    /// Total count (before limit)
    pub total: u64,
    /// Has more
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRow {
    pub event: String,
    pub distinct_id: String,
    pub timestamp: DateTime<Utc>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Query executor trait
#[async_trait]
pub trait QueryExecutor: Send + Sync {
    async fn execute(&self, query: AnalyticsQuery) -> Result<QueryResult, QueryError>;
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Invalid query: {0}")]
    Invalid(String),
    #[error("Query timeout")]
    Timeout,
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Rate limited")]
    RateLimited,
}

/// Query builder for fluent API
pub struct QueryBuilder {
    events: Vec<EventQuery>,
    date_range: Option<DateRange>,
    interval: QueryInterval,
    filters: Vec<PropertyFilter>,
    breakdown: Option<BreakdownSpec>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            date_range: None,
            interval: QueryInterval::Day,
            filters: Vec::new(),
            breakdown: None,
        }
    }

    pub fn event(mut self, event: &str) -> Self {
        self.events.push(EventQuery {
            event: event.to_string(),
            name: None,
            math: MathType::Total,
            math_property: None,
            filters: Vec::new(),
        });
        self
    }

    pub fn event_unique_users(mut self, event: &str) -> Self {
        self.events.push(EventQuery {
            event: event.to_string(),
            name: None,
            math: MathType::UniqueUsers,
            math_property: None,
            filters: Vec::new(),
        });
        self
    }

    pub fn last_days(mut self, days: i64) -> Self {
        self.date_range = Some(DateRange::last_n_days(days));
        self
    }

    pub fn interval(mut self, interval: QueryInterval) -> Self {
        self.interval = interval;
        self
    }

    pub fn filter(mut self, key: &str, operator: FilterOperator, value: FilterValue) -> Self {
        self.filters.push(PropertyFilter {
            key: key.to_string(),
            property_type: PropertyType::Event,
            operator,
            value,
        });
        self
    }

    pub fn breakdown(mut self, property: &str) -> Self {
        self.breakdown = Some(BreakdownSpec {
            property: property.to_string(),
            property_type: PropertyType::Event,
            limit: Some(10),
        });
        self
    }

    pub fn build_trends(self) -> TrendsQuery {
        TrendsQuery {
            events: self.events,
            date_range: self.date_range.unwrap_or_else(|| DateRange::last_n_days(7)),
            interval: self.interval,
            filters: self.filters,
            breakdown: self.breakdown,
            compare: false,
            formula: None,
        }
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .event("$pageview")
            .event_unique_users("signup")
            .last_days(30)
            .interval(QueryInterval::Week)
            .filter("$browser", FilterOperator::Exact, FilterValue::String("Chrome".to_string()))
            .breakdown("$os")
            .build_trends();

        assert_eq!(query.events.len(), 2);
        assert_eq!(query.events[0].math, MathType::Total);
        assert_eq!(query.events[1].math, MathType::UniqueUsers);
        assert_eq!(query.interval, QueryInterval::Week);
    }

    #[test]
    fn test_date_range() {
        let range = DateRange::last_n_days(7);
        let duration = range.end - range.start;
        assert_eq!(duration.num_days(), 7);
    }
}
```

## REST API

```rust
// Query API handlers
use axum::{extract::State, Json};

pub async fn query_trends(
    State(executor): State<Arc<dyn QueryExecutor>>,
    Json(query): Json<TrendsQuery>,
) -> Result<Json<TrendsResult>, ApiError> {
    let result = executor.execute(AnalyticsQuery::Trends(query)).await?;

    match result {
        QueryResult::Trends(trends) => Ok(Json(trends)),
        _ => Err(ApiError::Internal("Unexpected result type".to_string())),
    }
}

pub async fn query_funnel(
    State(executor): State<Arc<dyn QueryExecutor>>,
    Json(query): Json<FunnelQuery>,
) -> Result<Json<FunnelResult>, ApiError> {
    let result = executor.execute(AnalyticsQuery::Funnel(query)).await?;

    match result {
        QueryResult::Funnel(funnel) => Ok(Json(funnel)),
        _ => Err(ApiError::Internal("Unexpected result type".to_string())),
    }
}
```

## TypeScript Client

```typescript
// Analytics query client
class AnalyticsQuery {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async trends(query: TrendsQuery): Promise<TrendsResult> {
    return this.post('/api/analytics/trends', query);
  }

  async funnel(query: FunnelQuery): Promise<FunnelResult> {
    return this.post('/api/analytics/funnel', query);
  }

  async retention(query: RetentionQuery): Promise<RetentionResult> {
    return this.post('/api/analytics/retention', query);
  }

  // Builder pattern
  trend(event: string): TrendsQueryBuilder {
    return new TrendsQueryBuilder(this).event(event);
  }
}

class TrendsQueryBuilder {
  private client: AnalyticsQuery;
  private query: Partial<TrendsQuery> = {
    events: [],
    filters: [],
  };

  constructor(client: AnalyticsQuery) {
    this.client = client;
  }

  event(name: string, math: MathType = 'total'): this {
    this.query.events!.push({ event: name, math });
    return this;
  }

  lastDays(days: number): this {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);
    this.query.date_range = { start: start.toISOString(), end: end.toISOString() };
    return this;
  }

  interval(interval: QueryInterval): this {
    this.query.interval = interval;
    return this;
  }

  filter(key: string, operator: FilterOperator, value: any): this {
    this.query.filters!.push({ key, operator, value, property_type: 'event' });
    return this;
  }

  breakdown(property: string): this {
    this.query.breakdown = { property, property_type: 'event' };
    return this;
  }

  async execute(): Promise<TrendsResult> {
    return this.client.trends(this.query as TrendsQuery);
  }
}

// Usage
const result = await analytics
  .trend('$pageview')
  .event('signup', 'unique_users')
  .lastDays(30)
  .interval('day')
  .filter('$browser', 'exact', 'Chrome')
  .breakdown('$os')
  .execute();
```

## Related Specs

- 416-event-aggregation.md - Pre-aggregated data
- 415-event-persistence.md - Raw event storage
- 427-dashboard-data.md - Saved queries
