# Spec 421: Report Generation

## Phase
19 - Analytics/Telemetry

## Spec ID
421

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 410: Analytics Aggregation (aggregated data)
- Spec 420: Analytics Trends (trend analysis)

## Estimated Context
~9%

---

## Objective

Implement comprehensive report generation capabilities for analytics data, enabling users to create, customize, and schedule various types of analytics reports.

---

## Acceptance Criteria

- [ ] Generate usage summary reports
- [ ] Create cost and token reports
- [ ] Support performance reports
- [ ] Implement error summary reports
- [ ] Enable custom report templates
- [ ] Support scheduled report generation
- [ ] Create multiple output formats
- [ ] Enable report sharing/export

---

## Implementation Details

### Report Generation

```rust
// src/analytics/reports.rs

use crate::analytics::aggregation::{AggregatedMetric, Aggregator};
use crate::analytics::costs::{CostAggregation, CostTracker};
use crate::analytics::errors::{ErrorStats, ErrorTracker};
use crate::analytics::performance::{LatencyStats, PerformanceCollector};
use crate::analytics::tokens::TokenAggregation;
use crate::analytics::trends::{TrendAnalysis, TrendAnalyzer, TimeSeries};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    /// Daily usage summary
    DailySummary,
    /// Weekly overview
    WeeklyOverview,
    /// Monthly report
    MonthlyReport,
    /// Cost analysis
    CostAnalysis,
    /// Performance analysis
    PerformanceAnalysis,
    /// Error summary
    ErrorSummary,
    /// Custom report
    Custom,
}

/// Report time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl ReportPeriod {
    pub fn last_day() -> Self {
        let end = Utc::now();
        let start = end - Duration::days(1);
        Self { start, end }
    }

    pub fn last_week() -> Self {
        let end = Utc::now();
        let start = end - Duration::weeks(1);
        Self { start, end }
    }

    pub fn last_month() -> Self {
        let end = Utc::now();
        let start = end - Duration::days(30);
        Self { start, end }
    }

    pub fn custom(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

/// Report section content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ReportSection {
    /// Summary statistics
    Summary(SummarySection),
    /// Usage metrics
    Usage(UsageSection),
    /// Cost breakdown
    Costs(CostSection),
    /// Performance metrics
    Performance(PerformanceSection),
    /// Error statistics
    Errors(ErrorSection),
    /// Trend analysis
    Trends(TrendSection),
    /// Table data
    Table(TableSection),
    /// Chart data
    Chart(ChartSection),
    /// Text content
    Text(TextSection),
}

/// Summary section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySection {
    pub title: String,
    pub metrics: Vec<SummaryMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetric {
    pub name: String,
    pub value: String,
    pub change: Option<f64>,
    pub change_period: Option<String>,
}

/// Usage section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSection {
    pub title: String,
    pub total_sessions: u64,
    pub total_missions: u64,
    pub completed_missions: u64,
    pub failed_missions: u64,
    pub total_commands: u64,
    pub usage_by_feature: HashMap<String, u64>,
    pub usage_trend: Option<TrendAnalysis>,
}

/// Cost section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSection {
    pub title: String,
    pub total_cost_usd: f64,
    pub cost_by_provider: HashMap<String, f64>,
    pub cost_by_model: HashMap<String, f64>,
    pub daily_costs: Vec<(String, f64)>,
    pub projected_monthly: f64,
    pub cost_trend: Option<TrendAnalysis>,
}

/// Performance section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSection {
    pub title: String,
    pub latency_stats: LatencyStats,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub availability: f64,
    pub performance_by_backend: HashMap<String, LatencyStats>,
}

/// Error section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSection {
    pub title: String,
    pub total_errors: u64,
    pub unique_errors: u64,
    pub error_rate_per_hour: f64,
    pub recovery_rate: f64,
    pub top_errors: Vec<(String, u64)>,
    pub errors_by_category: HashMap<String, u64>,
}

/// Trend section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSection {
    pub title: String,
    pub analyses: Vec<NamedTrendAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedTrendAnalysis {
    pub name: String,
    pub analysis: TrendAnalysis,
}

/// Table section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSection {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Chart section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSection {
    pub title: String,
    pub chart_type: ChartType,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
}

/// Text section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSection {
    pub title: Option<String>,
    pub content: String,
    pub style: TextStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextStyle {
    Normal,
    Header,
    Highlight,
    Warning,
}

/// Complete report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Report identifier
    pub id: String,
    /// Report title
    pub title: String,
    /// Report type
    pub report_type: ReportType,
    /// Time period covered
    pub period: ReportPeriod,
    /// When report was generated
    pub generated_at: DateTime<Utc>,
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Report metadata
    pub metadata: ReportMetadata,
}

/// Report metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Report template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Report type this template creates
    pub report_type: ReportType,
    /// Section definitions
    pub sections: Vec<SectionDefinition>,
    /// Default period
    pub default_period: TemplatePeriod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionDefinition {
    pub section_type: String,
    pub title: String,
    pub enabled: bool,
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplatePeriod {
    LastDay,
    LastWeek,
    LastMonth,
    Custom,
}

/// Report generator
pub struct ReportGenerator {
    trend_analyzer: TrendAnalyzer,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            trend_analyzer: TrendAnalyzer::new(),
        }
    }

    /// Generate a daily summary report
    pub fn generate_daily_summary(
        &self,
        usage: &UsageData,
        costs: &CostAggregation,
        errors: &ErrorStats,
        performance: &LatencyStats,
    ) -> Report {
        let period = ReportPeriod::last_day();
        let mut sections = Vec::new();

        // Summary section
        sections.push(ReportSection::Summary(SummarySection {
            title: "Daily Summary".to_string(),
            metrics: vec![
                SummaryMetric {
                    name: "Total Sessions".to_string(),
                    value: usage.total_sessions.to_string(),
                    change: usage.session_change,
                    change_period: Some("vs yesterday".to_string()),
                },
                SummaryMetric {
                    name: "Total Cost".to_string(),
                    value: format!("${:.2}", costs.total_cost_usd),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Avg Latency".to_string(),
                    value: format!("{:.0}ms", performance.mean_ms),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Error Rate".to_string(),
                    value: format!("{:.1}%", errors.hourly_rate),
                    change: None,
                    change_period: None,
                },
            ],
        }));

        // Cost breakdown
        sections.push(ReportSection::Costs(CostSection {
            title: "Cost Breakdown".to_string(),
            total_cost_usd: costs.total_cost_usd,
            cost_by_provider: costs.by_provider.clone(),
            cost_by_model: costs.by_model.clone(),
            daily_costs: costs.by_day.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            projected_monthly: costs.total_cost_usd * 30.0,
            cost_trend: None,
        }));

        // Error summary
        if errors.total_errors > 0 {
            sections.push(ReportSection::Errors(ErrorSection {
                title: "Error Summary".to_string(),
                total_errors: errors.total_errors,
                unique_errors: errors.unique_errors,
                error_rate_per_hour: errors.hourly_rate,
                recovery_rate: errors.recovery_rate,
                top_errors: Vec::new(),
                errors_by_category: errors
                    .by_category
                    .iter()
                    .map(|(k, v)| (format!("{:?}", k), *v))
                    .collect(),
            }));
        }

        Report {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("Daily Summary - {}", period.start.format("%Y-%m-%d")),
            report_type: ReportType::DailySummary,
            period,
            generated_at: Utc::now(),
            sections,
            metadata: ReportMetadata::default(),
        }
    }

    /// Generate a weekly overview report
    pub fn generate_weekly_overview(
        &self,
        usage: &UsageData,
        costs: &CostAggregation,
        tokens: &TokenAggregation,
        errors: &ErrorStats,
    ) -> Report {
        let period = ReportPeriod::last_week();
        let mut sections = Vec::new();

        // Usage overview
        sections.push(ReportSection::Usage(UsageSection {
            title: "Weekly Usage".to_string(),
            total_sessions: usage.total_sessions,
            total_missions: usage.total_missions,
            completed_missions: usage.completed_missions,
            failed_missions: usage.failed_missions,
            total_commands: usage.total_commands,
            usage_by_feature: usage.by_feature.clone(),
            usage_trend: None,
        }));

        // Token usage chart
        sections.push(ReportSection::Chart(ChartSection {
            title: "Token Usage by Model".to_string(),
            chart_type: ChartType::Pie,
            labels: tokens.by_model.keys().cloned().collect(),
            datasets: vec![ChartDataset {
                label: "Tokens".to_string(),
                data: tokens.by_model.values().map(|v| *v as f64).collect(),
            }],
        }));

        // Cost trend
        sections.push(ReportSection::Chart(ChartSection {
            title: "Daily Costs".to_string(),
            chart_type: ChartType::Bar,
            labels: costs.by_day.keys().cloned().collect(),
            datasets: vec![ChartDataset {
                label: "Cost (USD)".to_string(),
                data: costs.by_day.values().cloned().collect(),
            }],
        }));

        Report {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!(
                "Weekly Overview - {} to {}",
                period.start.format("%Y-%m-%d"),
                period.end.format("%Y-%m-%d")
            ),
            report_type: ReportType::WeeklyOverview,
            period,
            generated_at: Utc::now(),
            sections,
            metadata: ReportMetadata::default(),
        }
    }

    /// Generate a cost analysis report
    pub fn generate_cost_analysis(
        &self,
        costs: &CostAggregation,
        tokens: &TokenAggregation,
        period: ReportPeriod,
    ) -> Report {
        let mut sections = Vec::new();

        // Cost summary
        let cost_per_token = if tokens.total > 0 {
            costs.total_cost_usd / tokens.total as f64
        } else {
            0.0
        };

        sections.push(ReportSection::Summary(SummarySection {
            title: "Cost Summary".to_string(),
            metrics: vec![
                SummaryMetric {
                    name: "Total Cost".to_string(),
                    value: format!("${:.2}", costs.total_cost_usd),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Cost per 1K Tokens".to_string(),
                    value: format!("${:.4}", cost_per_token * 1000.0),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Total Tokens".to_string(),
                    value: format_number(tokens.total),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Avg Cost/Request".to_string(),
                    value: format!("${:.4}", costs.avg_cost_per_request),
                    change: None,
                    change_period: None,
                },
            ],
        }));

        // Cost by provider table
        let mut rows: Vec<Vec<String>> = costs
            .by_provider
            .iter()
            .map(|(provider, cost)| {
                let pct = if costs.total_cost_usd > 0.0 {
                    (cost / costs.total_cost_usd) * 100.0
                } else {
                    0.0
                };
                vec![
                    provider.clone(),
                    format!("${:.2}", cost),
                    format!("{:.1}%", pct),
                ]
            })
            .collect();
        rows.sort_by(|a, b| {
            b[1].parse::<f64>()
                .unwrap_or(0.0)
                .partial_cmp(&a[1].parse::<f64>().unwrap_or(0.0))
                .unwrap()
        });

        sections.push(ReportSection::Table(TableSection {
            title: "Cost by Provider".to_string(),
            headers: vec![
                "Provider".to_string(),
                "Cost".to_string(),
                "Percentage".to_string(),
            ],
            rows,
        }));

        // Cost by model chart
        sections.push(ReportSection::Chart(ChartSection {
            title: "Cost Distribution by Model".to_string(),
            chart_type: ChartType::Pie,
            labels: costs.by_model.keys().cloned().collect(),
            datasets: vec![ChartDataset {
                label: "Cost (USD)".to_string(),
                data: costs.by_model.values().cloned().collect(),
            }],
        }));

        Report {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Cost Analysis Report".to_string(),
            report_type: ReportType::CostAnalysis,
            period,
            generated_at: Utc::now(),
            sections,
            metadata: ReportMetadata::default(),
        }
    }

    /// Generate from a template
    pub fn generate_from_template(
        &self,
        template: &ReportTemplate,
        data: &ReportData,
        period: ReportPeriod,
    ) -> Report {
        let mut sections = Vec::new();

        for section_def in &template.sections {
            if !section_def.enabled {
                continue;
            }

            let section = match section_def.section_type.as_str() {
                "summary" => self.build_summary_section(&section_def.title, data),
                "usage" => self.build_usage_section(&section_def.title, data),
                "costs" => self.build_cost_section(&section_def.title, data),
                "errors" => self.build_error_section(&section_def.title, data),
                "performance" => self.build_performance_section(&section_def.title, data),
                _ => None,
            };

            if let Some(s) = section {
                sections.push(s);
            }
        }

        Report {
            id: uuid::Uuid::new_v4().to_string(),
            title: template.name.clone(),
            report_type: template.report_type,
            period,
            generated_at: Utc::now(),
            sections,
            metadata: ReportMetadata::default(),
        }
    }

    fn build_summary_section(&self, title: &str, data: &ReportData) -> Option<ReportSection> {
        Some(ReportSection::Summary(SummarySection {
            title: title.to_string(),
            metrics: vec![
                SummaryMetric {
                    name: "Total Sessions".to_string(),
                    value: data.usage.total_sessions.to_string(),
                    change: None,
                    change_period: None,
                },
                SummaryMetric {
                    name: "Total Cost".to_string(),
                    value: format!("${:.2}", data.costs.total_cost_usd),
                    change: None,
                    change_period: None,
                },
            ],
        }))
    }

    fn build_usage_section(&self, title: &str, data: &ReportData) -> Option<ReportSection> {
        Some(ReportSection::Usage(UsageSection {
            title: title.to_string(),
            total_sessions: data.usage.total_sessions,
            total_missions: data.usage.total_missions,
            completed_missions: data.usage.completed_missions,
            failed_missions: data.usage.failed_missions,
            total_commands: data.usage.total_commands,
            usage_by_feature: data.usage.by_feature.clone(),
            usage_trend: None,
        }))
    }

    fn build_cost_section(&self, title: &str, data: &ReportData) -> Option<ReportSection> {
        Some(ReportSection::Costs(CostSection {
            title: title.to_string(),
            total_cost_usd: data.costs.total_cost_usd,
            cost_by_provider: data.costs.by_provider.clone(),
            cost_by_model: data.costs.by_model.clone(),
            daily_costs: data.costs.by_day.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            projected_monthly: data.costs.total_cost_usd * 30.0 / data.period_days as f64,
            cost_trend: None,
        }))
    }

    fn build_error_section(&self, title: &str, data: &ReportData) -> Option<ReportSection> {
        Some(ReportSection::Errors(ErrorSection {
            title: title.to_string(),
            total_errors: data.errors.total_errors,
            unique_errors: data.errors.unique_errors,
            error_rate_per_hour: data.errors.hourly_rate,
            recovery_rate: data.errors.recovery_rate,
            top_errors: Vec::new(),
            errors_by_category: data.errors.by_category
                .iter()
                .map(|(k, v)| (format!("{:?}", k), *v))
                .collect(),
        }))
    }

    fn build_performance_section(&self, title: &str, data: &ReportData) -> Option<ReportSection> {
        Some(ReportSection::Performance(PerformanceSection {
            title: title.to_string(),
            latency_stats: data.performance.clone(),
            throughput_rps: 0.0,
            error_rate: data.errors.hourly_rate / 100.0,
            availability: 1.0 - (data.errors.hourly_rate / 100.0),
            performance_by_backend: HashMap::new(),
        }))
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Usage data for reports
#[derive(Debug, Clone, Default)]
pub struct UsageData {
    pub total_sessions: u64,
    pub total_missions: u64,
    pub completed_missions: u64,
    pub failed_missions: u64,
    pub total_commands: u64,
    pub by_feature: HashMap<String, u64>,
    pub session_change: Option<f64>,
}

/// Combined report data
#[derive(Debug, Clone)]
pub struct ReportData {
    pub usage: UsageData,
    pub costs: CostAggregation,
    pub tokens: TokenAggregation,
    pub errors: ErrorStats,
    pub performance: LatencyStats,
    pub period_days: u32,
}

/// Format large numbers for display
fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Predefined report templates
pub fn default_templates() -> Vec<ReportTemplate> {
    vec![
        ReportTemplate {
            id: "daily-summary".to_string(),
            name: "Daily Summary".to_string(),
            report_type: ReportType::DailySummary,
            sections: vec![
                SectionDefinition {
                    section_type: "summary".to_string(),
                    title: "Overview".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
                SectionDefinition {
                    section_type: "usage".to_string(),
                    title: "Usage".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
                SectionDefinition {
                    section_type: "costs".to_string(),
                    title: "Costs".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
            ],
            default_period: TemplatePeriod::LastDay,
        },
        ReportTemplate {
            id: "weekly-overview".to_string(),
            name: "Weekly Overview".to_string(),
            report_type: ReportType::WeeklyOverview,
            sections: vec![
                SectionDefinition {
                    section_type: "summary".to_string(),
                    title: "Week Summary".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
                SectionDefinition {
                    section_type: "usage".to_string(),
                    title: "Usage Trends".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
                SectionDefinition {
                    section_type: "costs".to_string(),
                    title: "Cost Analysis".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
                SectionDefinition {
                    section_type: "errors".to_string(),
                    title: "Error Summary".to_string(),
                    enabled: true,
                    options: HashMap::new(),
                },
            ],
            default_period: TemplatePeriod::LastWeek,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> ReportData {
        ReportData {
            usage: UsageData {
                total_sessions: 100,
                total_missions: 50,
                completed_missions: 45,
                failed_missions: 5,
                total_commands: 500,
                by_feature: HashMap::new(),
                session_change: Some(10.0),
            },
            costs: CostAggregation {
                total_cost_usd: 25.50,
                request_count: 1000,
                avg_cost_per_request: 0.0255,
                by_provider: [("anthropic".to_string(), 20.0), ("openai".to_string(), 5.50)]
                    .into_iter()
                    .collect(),
                by_model: HashMap::new(),
                by_project: HashMap::new(),
                by_day: HashMap::new(),
                period_start: Utc::now() - Duration::days(7),
                period_end: Utc::now(),
            },
            tokens: TokenAggregation {
                total_input: 500000,
                total_output: 250000,
                total: 750000,
                avg_input: 500.0,
                avg_output: 250.0,
                request_count: 1000,
                by_provider: HashMap::new(),
                by_model: HashMap::new(),
                by_hour: HashMap::new(),
            },
            errors: ErrorStats {
                total_errors: 25,
                unique_errors: 5,
                hourly_rate: 1.5,
                recovery_rate: 0.8,
                by_category: HashMap::new(),
                by_severity: HashMap::new(),
                by_component: HashMap::new(),
            },
            performance: LatencyStats {
                count: 1000,
                mean_ms: 250.0,
                min_ms: 50.0,
                max_ms: 1500.0,
                p50_ms: 200.0,
                p90_ms: 500.0,
                p95_ms: 750.0,
                p99_ms: 1200.0,
                stddev_ms: 150.0,
            },
            period_days: 7,
        }
    }

    #[test]
    fn test_daily_summary_generation() {
        let generator = ReportGenerator::new();
        let data = create_test_data();

        let report = generator.generate_daily_summary(
            &data.usage,
            &data.costs,
            &data.errors,
            &data.performance,
        );

        assert_eq!(report.report_type, ReportType::DailySummary);
        assert!(!report.sections.is_empty());
    }

    #[test]
    fn test_cost_analysis_generation() {
        let generator = ReportGenerator::new();
        let data = create_test_data();

        let report = generator.generate_cost_analysis(
            &data.costs,
            &data.tokens,
            ReportPeriod::last_week(),
        );

        assert_eq!(report.report_type, ReportType::CostAnalysis);

        // Check for cost section
        let has_cost_section = report.sections.iter().any(|s| {
            matches!(s, ReportSection::Costs(_))
        });
        assert!(has_cost_section);
    }

    #[test]
    fn test_template_generation() {
        let generator = ReportGenerator::new();
        let templates = default_templates();
        let data = create_test_data();

        let report = generator.generate_from_template(
            &templates[0],
            &data,
            ReportPeriod::last_day(),
        );

        assert_eq!(report.title, "Daily Summary");
        assert!(!report.sections.is_empty());
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1500000), "1.5M");
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Report generation for each type
   - Section building logic
   - Template processing

2. **Integration Tests**
   - Full report generation pipeline
   - Data aggregation for reports

3. **Output Tests**
   - Report serialization
   - Section completeness

---

## Related Specs

- Spec 410: Analytics Aggregation
- Spec 420: Trend Analysis
- Spec 422: Dashboard API
- Spec 423: Export Formats
