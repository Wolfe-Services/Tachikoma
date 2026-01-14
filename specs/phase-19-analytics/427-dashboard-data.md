# 427 - Dashboard Data

## Overview

Dashboard configuration, saved queries, and real-time data endpoints for analytics visualization.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/dashboard.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use crate::query::{AnalyticsQuery, QueryResult, TrendsQuery, FunnelQuery};

/// Dashboard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard ID
    pub id: String,
    /// Dashboard name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Owner user ID
    pub owner_id: String,
    /// Team ID (if shared)
    pub team_id: Option<String>,
    /// Dashboard tiles
    pub tiles: Vec<DashboardTile>,
    /// Layout configuration
    pub layout: DashboardLayout,
    /// Filters that apply to all tiles
    pub global_filters: Vec<DashboardFilter>,
    /// Date range
    pub date_range: DateRangeConfig,
    /// Refresh interval (seconds)
    pub refresh_interval: Option<u32>,
    /// Tags
    pub tags: Vec<String>,
    /// Is pinned
    pub pinned: bool,
    /// Visibility
    pub visibility: Visibility,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Team,
    Organization,
    Public,
}

/// Dashboard tile (widget)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTile {
    /// Tile ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// Tile type
    pub tile_type: TileType,
    /// Query configuration
    pub query: SavedQuery,
    /// Position
    pub position: TilePosition,
    /// Size
    pub size: TileSize,
    /// Display options
    pub display: DisplayOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileType {
    /// Line/area chart
    Trend,
    /// Bar chart
    Bar,
    /// Pie/donut chart
    Pie,
    /// Single number
    Number,
    /// Data table
    Table,
    /// Funnel visualization
    Funnel,
    /// Retention table
    Retention,
    /// Paths/flow diagram
    Paths,
    /// World map
    WorldMap,
    /// Custom HTML
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    /// Chart colors
    pub colors: Option<Vec<String>>,
    /// Show legend
    pub show_legend: bool,
    /// Show labels
    pub show_labels: bool,
    /// Y-axis config
    pub y_axis: Option<AxisConfig>,
    /// Number format
    pub number_format: Option<NumberFormat>,
    /// Goal line
    pub goal: Option<f64>,
    /// Comparison mode
    pub comparison: Option<ComparisonMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberFormat {
    Integer,
    Decimal,
    Percentage,
    Currency,
    Duration,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonMode {
    PreviousPeriod,
    PreviousYear,
    Custom,
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    /// Number of columns
    pub columns: u32,
    /// Row height
    pub row_height: u32,
    /// Gap between tiles
    pub gap: u32,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            columns: 12,
            row_height: 80,
            gap: 16,
        }
    }
}

/// Dashboard filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    /// Filter ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Property key
    pub property: String,
    /// Filter type
    pub filter_type: FilterType,
    /// Default value
    pub default_value: Option<serde_json::Value>,
    /// Available options (for dropdown)
    pub options: Option<Vec<FilterOption>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterType {
    Dropdown,
    MultiSelect,
    DateRange,
    Text,
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: serde_json::Value,
    pub label: String,
}

/// Date range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeConfig {
    /// Preset range
    pub preset: Option<DateRangePreset>,
    /// Custom start
    pub custom_start: Option<DateTime<Utc>>,
    /// Custom end
    pub custom_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateRangePreset {
    Today,
    Yesterday,
    Last7Days,
    Last30Days,
    Last90Days,
    LastMonth,
    LastQuarter,
    LastYear,
    MonthToDate,
    QuarterToDate,
    YearToDate,
    AllTime,
}

impl DateRangePreset {
    pub fn to_date_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        let end = now;

        let start = match self {
            DateRangePreset::Today => now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            DateRangePreset::Yesterday => (now - chrono::Duration::days(1)).date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc(),
            DateRangePreset::Last7Days => now - chrono::Duration::days(7),
            DateRangePreset::Last30Days => now - chrono::Duration::days(30),
            DateRangePreset::Last90Days => now - chrono::Duration::days(90),
            DateRangePreset::LastMonth => now - chrono::Duration::days(30),
            DateRangePreset::LastQuarter => now - chrono::Duration::days(90),
            DateRangePreset::LastYear => now - chrono::Duration::days(365),
            _ => now - chrono::Duration::days(30),
        };

        (start, end)
    }
}

/// Saved query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedQuery {
    /// Query ID
    pub id: String,
    /// Query name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Query definition
    pub query: AnalyticsQuery,
    /// Owner
    pub owner_id: String,
    /// Tags
    pub tags: Vec<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Last run
    pub last_run: Option<DateTime<Utc>>,
    /// Cached result
    pub cached_result: Option<CachedResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub result: QueryResult,
    pub cached_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Dashboard storage trait
#[async_trait]
pub trait DashboardStorage: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<Dashboard>, DashboardError>;
    async fn save(&self, dashboard: &Dashboard) -> Result<(), DashboardError>;
    async fn delete(&self, id: &str) -> Result<(), DashboardError>;
    async fn list(&self, owner_id: &str, limit: u32, offset: u32) -> Result<Vec<Dashboard>, DashboardError>;
    async fn list_shared(&self, team_id: &str, limit: u32, offset: u32) -> Result<Vec<Dashboard>, DashboardError>;

    async fn get_query(&self, id: &str) -> Result<Option<SavedQuery>, DashboardError>;
    async fn save_query(&self, query: &SavedQuery) -> Result<(), DashboardError>;
    async fn delete_query(&self, id: &str) -> Result<(), DashboardError>;
    async fn list_queries(&self, owner_id: &str) -> Result<Vec<SavedQuery>, DashboardError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    #[error("Dashboard not found")]
    NotFound,
    #[error("Access denied")]
    AccessDenied,
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Dashboard service
pub struct DashboardService {
    storage: std::sync::Arc<dyn DashboardStorage>,
    query_executor: std::sync::Arc<dyn QueryExecutor>,
    cache: std::sync::Arc<dyn DashboardCache>,
}

#[async_trait]
pub trait QueryExecutor: Send + Sync {
    async fn execute(&self, query: &AnalyticsQuery) -> Result<QueryResult, DashboardError>;
}

#[async_trait]
pub trait DashboardCache: Send + Sync {
    async fn get(&self, key: &str) -> Option<QueryResult>;
    async fn set(&self, key: &str, result: QueryResult, ttl_seconds: u32);
    async fn invalidate(&self, pattern: &str);
}

impl DashboardService {
    pub fn new(
        storage: std::sync::Arc<dyn DashboardStorage>,
        query_executor: std::sync::Arc<dyn QueryExecutor>,
        cache: std::sync::Arc<dyn DashboardCache>,
    ) -> Self {
        Self { storage, query_executor, cache }
    }

    /// Create a new dashboard
    pub async fn create(&self, mut dashboard: Dashboard) -> Result<Dashboard, DashboardError> {
        dashboard.id = uuid::Uuid::new_v4().to_string();
        dashboard.created_at = Utc::now();
        dashboard.updated_at = Utc::now();

        self.storage.save(&dashboard).await?;
        Ok(dashboard)
    }

    /// Get dashboard with data
    pub async fn get_with_data(
        &self,
        id: &str,
        filters: HashMap<String, serde_json::Value>,
    ) -> Result<DashboardWithData, DashboardError> {
        let dashboard = self.storage.get(id).await?
            .ok_or(DashboardError::NotFound)?;

        let mut tile_results = Vec::new();

        for tile in &dashboard.tiles {
            let cache_key = self.generate_cache_key(&tile.query.id, &filters);

            // Check cache first
            let result = if let Some(cached) = self.cache.get(&cache_key).await {
                cached
            } else {
                // Execute query
                let result = self.query_executor.execute(&tile.query.query).await?;

                // Cache result
                self.cache.set(&cache_key, result.clone(), 300).await;

                result
            };

            tile_results.push(TileResult {
                tile_id: tile.id.clone(),
                result,
            });
        }

        Ok(DashboardWithData {
            dashboard,
            results: tile_results,
            generated_at: Utc::now(),
        })
    }

    /// Refresh a single tile
    pub async fn refresh_tile(
        &self,
        dashboard_id: &str,
        tile_id: &str,
        filters: HashMap<String, serde_json::Value>,
    ) -> Result<TileResult, DashboardError> {
        let dashboard = self.storage.get(dashboard_id).await?
            .ok_or(DashboardError::NotFound)?;

        let tile = dashboard.tiles.iter()
            .find(|t| t.id == tile_id)
            .ok_or(DashboardError::NotFound)?;

        let result = self.query_executor.execute(&tile.query.query).await?;

        let cache_key = self.generate_cache_key(&tile.query.id, &filters);
        self.cache.set(&cache_key, result.clone(), 300).await;

        Ok(TileResult {
            tile_id: tile_id.to_string(),
            result,
        })
    }

    fn generate_cache_key(&self, query_id: &str, filters: &HashMap<String, serde_json::Value>) -> String {
        let filter_hash = {
            let json = serde_json::to_string(filters).unwrap_or_default();
            use sha2::{Sha256, Digest};
            let hash = Sha256::digest(json.as_bytes());
            hex::encode(&hash[..8])
        };

        format!("dashboard:{}:{}", query_id, filter_hash)
    }

    /// Add tile to dashboard
    pub async fn add_tile(
        &self,
        dashboard_id: &str,
        tile: DashboardTile,
    ) -> Result<Dashboard, DashboardError> {
        let mut dashboard = self.storage.get(dashboard_id).await?
            .ok_or(DashboardError::NotFound)?;

        dashboard.tiles.push(tile);
        dashboard.updated_at = Utc::now();

        self.storage.save(&dashboard).await?;
        Ok(dashboard)
    }

    /// Remove tile from dashboard
    pub async fn remove_tile(
        &self,
        dashboard_id: &str,
        tile_id: &str,
    ) -> Result<Dashboard, DashboardError> {
        let mut dashboard = self.storage.get(dashboard_id).await?
            .ok_or(DashboardError::NotFound)?;

        dashboard.tiles.retain(|t| t.id != tile_id);
        dashboard.updated_at = Utc::now();

        self.storage.save(&dashboard).await?;
        Ok(dashboard)
    }

    /// Update tile positions
    pub async fn update_layout(
        &self,
        dashboard_id: &str,
        positions: Vec<TilePositionUpdate>,
    ) -> Result<Dashboard, DashboardError> {
        let mut dashboard = self.storage.get(dashboard_id).await?
            .ok_or(DashboardError::NotFound)?;

        for update in positions {
            if let Some(tile) = dashboard.tiles.iter_mut().find(|t| t.id == update.tile_id) {
                tile.position = update.position;
                tile.size = update.size;
            }
        }

        dashboard.updated_at = Utc::now();
        self.storage.save(&dashboard).await?;

        Ok(dashboard)
    }

    /// Save a query
    pub async fn save_query(&self, mut query: SavedQuery) -> Result<SavedQuery, DashboardError> {
        if query.id.is_empty() {
            query.id = uuid::Uuid::new_v4().to_string();
            query.created_at = Utc::now();
        }
        query.updated_at = Utc::now();

        self.storage.save_query(&query).await?;
        Ok(query)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWithData {
    pub dashboard: Dashboard,
    pub results: Vec<TileResult>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileResult {
    pub tile_id: String,
    pub result: QueryResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilePositionUpdate {
    pub tile_id: String,
    pub position: TilePosition,
    pub size: TileSize,
}

/// Pre-built dashboard templates
pub mod templates {
    use super::*;

    pub fn web_analytics() -> Dashboard {
        Dashboard {
            id: String::new(),
            name: "Web Analytics Overview".to_string(),
            description: Some("Key website metrics".to_string()),
            owner_id: String::new(),
            team_id: None,
            tiles: vec![
                DashboardTile {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: "Total Pageviews".to_string(),
                    description: None,
                    tile_type: TileType::Number,
                    query: SavedQuery {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: "Pageviews".to_string(),
                        description: None,
                        query: AnalyticsQuery::Trends(TrendsQuery {
                            events: vec![crate::query::EventQuery {
                                event: "$pageview".to_string(),
                                name: None,
                                math: crate::query::MathType::Total,
                                math_property: None,
                                filters: vec![],
                            }],
                            date_range: crate::query::DateRange::last_n_days(30),
                            interval: crate::query::QueryInterval::Day,
                            filters: vec![],
                            breakdown: None,
                            compare: true,
                            formula: None,
                        }),
                        owner_id: String::new(),
                        tags: vec![],
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        last_run: None,
                        cached_result: None,
                    },
                    position: TilePosition { x: 0, y: 0 },
                    size: TileSize { width: 3, height: 2 },
                    display: DisplayOptions {
                        colors: None,
                        show_legend: false,
                        show_labels: true,
                        y_axis: None,
                        number_format: Some(NumberFormat::Compact),
                        goal: None,
                        comparison: Some(ComparisonMode::PreviousPeriod),
                    },
                },
            ],
            layout: DashboardLayout::default(),
            global_filters: vec![],
            date_range: DateRangeConfig {
                preset: Some(DateRangePreset::Last30Days),
                custom_start: None,
                custom_end: None,
            },
            refresh_interval: Some(300),
            tags: vec!["web".to_string(), "analytics".to_string()],
            pinned: false,
            visibility: Visibility::Private,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_range_presets() {
        let (start, end) = DateRangePreset::Last7Days.to_date_range();
        let diff = end - start;
        assert!(diff.num_days() >= 6 && diff.num_days() <= 8);
    }

    #[test]
    fn test_dashboard_template() {
        let dashboard = templates::web_analytics();
        assert!(!dashboard.tiles.is_empty());
        assert!(dashboard.date_range.preset.is_some());
    }

    #[test]
    fn test_default_layout() {
        let layout = DashboardLayout::default();
        assert_eq!(layout.columns, 12);
    }
}
```

## REST API

```yaml
openapi: 3.0.0
paths:
  /api/dashboards:
    get:
      summary: List dashboards
    post:
      summary: Create dashboard

  /api/dashboards/{id}:
    get:
      summary: Get dashboard with data
    put:
      summary: Update dashboard
    delete:
      summary: Delete dashboard

  /api/dashboards/{id}/tiles:
    post:
      summary: Add tile
    patch:
      summary: Update tile positions

  /api/dashboards/{id}/tiles/{tileId}:
    delete:
      summary: Remove tile

  /api/dashboards/{id}/tiles/{tileId}/refresh:
    post:
      summary: Refresh single tile data

  /api/queries:
    get:
      summary: List saved queries
    post:
      summary: Create saved query

  /api/queries/{id}/run:
    post:
      summary: Execute query
```

## Related Specs

- 423-analytics-query.md - Query definitions
- 428-realtime-analytics.md - Live data updates
- 416-event-aggregation.md - Pre-computed data
