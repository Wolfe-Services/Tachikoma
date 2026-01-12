# Spec 420: Trend Analysis

## Phase
19 - Analytics/Telemetry

## Spec ID
420

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 410: Analytics Aggregation (aggregated data)
- Spec 409: Analytics Storage (data persistence)

## Estimated Context
~9%

---

## Objective

Implement trend analysis capabilities for analytics data, enabling identification of patterns, anomalies, and changes over time to support data-driven decision making and proactive issue detection.

---

## Acceptance Criteria

- [ ] Calculate moving averages and trends
- [ ] Implement change detection algorithms
- [ ] Support seasonality analysis
- [ ] Create forecasting capabilities
- [ ] Detect anomalies in time series
- [ ] Generate trend reports
- [ ] Support comparative trend analysis
- [ ] Enable custom trend definitions

---

## Implementation Details

### Trend Analysis

```rust
// src/analytics/trends.rs

use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Time series with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub name: String,
    pub points: Vec<DataPoint>,
    pub unit: String,
}

impl TimeSeries {
    pub fn new(name: &str, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            points: Vec::new(),
            unit: unit.to_string(),
        }
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.points.push(DataPoint { timestamp, value });
        self.points.sort_by_key(|p| p.timestamp);
    }

    pub fn values(&self) -> Vec<f64> {
        self.points.iter().map(|p| p.value).collect()
    }

    pub fn timestamps(&self) -> Vec<DateTime<Utc>> {
        self.points.iter().map(|p| p.timestamp).collect()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

/// Trend strength
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendStrength {
    Strong,
    Moderate,
    Weak,
    None,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength
    pub strength: TrendStrength,
    /// Percentage change
    pub change_percent: f64,
    /// Slope of the trend line
    pub slope: f64,
    /// R-squared value (goodness of fit)
    pub r_squared: f64,
    /// Moving average values
    pub moving_average: Vec<DataPoint>,
    /// Forecast if available
    pub forecast: Option<Vec<DataPoint>>,
    /// Detected seasonality
    pub seasonality: Option<SeasonalityAnalysis>,
    /// Detected anomalies
    pub anomalies: Vec<Anomaly>,
}

/// Seasonality analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityAnalysis {
    /// Period detected (in number of data points)
    pub period: usize,
    /// Strength of seasonality (0-1)
    pub strength: f64,
    /// Type of seasonality
    pub seasonality_type: SeasonalityType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeasonalityType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    None,
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub expected_value: f64,
    pub deviation: f64,
    pub anomaly_type: AnomalyType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    Spike,
    Drop,
    LevelShift,
    TrendChange,
}

/// Trend analyzer
pub struct TrendAnalyzer {
    /// Window size for moving average
    window_size: usize,
    /// Anomaly detection threshold (standard deviations)
    anomaly_threshold: f64,
    /// Minimum data points for analysis
    min_data_points: usize,
}

impl TrendAnalyzer {
    pub fn new() -> Self {
        Self {
            window_size: 7,
            anomaly_threshold: 2.5,
            min_data_points: 10,
        }
    }

    pub fn with_window(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    pub fn with_anomaly_threshold(mut self, threshold: f64) -> Self {
        self.anomaly_threshold = threshold;
        self
    }

    /// Analyze a time series for trends
    pub fn analyze(&self, series: &TimeSeries) -> TrendAnalysis {
        if series.len() < self.min_data_points {
            return TrendAnalysis {
                direction: TrendDirection::Unknown,
                strength: TrendStrength::None,
                change_percent: 0.0,
                slope: 0.0,
                r_squared: 0.0,
                moving_average: Vec::new(),
                forecast: None,
                seasonality: None,
                anomalies: Vec::new(),
            };
        }

        let values = series.values();
        let timestamps = series.timestamps();

        // Calculate linear regression
        let (slope, intercept, r_squared) = self.linear_regression(&values);

        // Calculate moving average
        let moving_avg = self.moving_average(&series.points, self.window_size);

        // Determine trend direction and strength
        let direction = if slope > 0.01 {
            TrendDirection::Increasing
        } else if slope < -0.01 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        let strength = if r_squared > 0.8 {
            TrendStrength::Strong
        } else if r_squared > 0.5 {
            TrendStrength::Moderate
        } else if r_squared > 0.2 {
            TrendStrength::Weak
        } else {
            TrendStrength::None
        };

        // Calculate percentage change
        let first_avg = values.iter().take(self.window_size).sum::<f64>() / self.window_size as f64;
        let last_avg = values.iter().rev().take(self.window_size).sum::<f64>() / self.window_size as f64;
        let change_percent = if first_avg != 0.0 {
            ((last_avg - first_avg) / first_avg) * 100.0
        } else {
            0.0
        };

        // Detect anomalies
        let anomalies = self.detect_anomalies(&series.points, &moving_avg);

        // Detect seasonality
        let seasonality = self.detect_seasonality(&values, &timestamps);

        // Generate forecast
        let forecast = self.forecast(&series.points, slope, intercept, 7);

        TrendAnalysis {
            direction,
            strength,
            change_percent,
            slope,
            r_squared,
            moving_average: moving_avg,
            forecast: Some(forecast),
            seasonality,
            anomalies,
        }
    }

    /// Calculate linear regression (slope, intercept, r-squared)
    fn linear_regression(&self, values: &[f64]) -> (f64, f64, f64) {
        let n = values.len() as f64;
        if n < 2.0 {
            return (0.0, 0.0, 0.0);
        }

        let x: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = x.iter().zip(values.iter()).map(|(xi, yi)| xi * yi).sum();
        let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator == 0.0 {
            return (0.0, values[0], 0.0);
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = values.iter().map(|yi| (yi - mean_y).powi(2)).sum();
        let ss_res: f64 = x
            .iter()
            .zip(values.iter())
            .map(|(xi, yi)| (yi - (slope * xi + intercept)).powi(2))
            .sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, intercept, r_squared.max(0.0))
    }

    /// Calculate moving average
    fn moving_average(&self, points: &[DataPoint], window: usize) -> Vec<DataPoint> {
        if points.len() < window {
            return points.to_vec();
        }

        let mut result = Vec::new();

        for i in (window - 1)..points.len() {
            let sum: f64 = points[(i - window + 1)..=i]
                .iter()
                .map(|p| p.value)
                .sum();
            let avg = sum / window as f64;

            result.push(DataPoint {
                timestamp: points[i].timestamp,
                value: avg,
            });
        }

        result
    }

    /// Detect anomalies in the data
    fn detect_anomalies(
        &self,
        points: &[DataPoint],
        moving_avg: &[DataPoint],
    ) -> Vec<Anomaly> {
        if moving_avg.is_empty() {
            return Vec::new();
        }

        let mut anomalies = Vec::new();

        // Calculate standard deviation of residuals
        let residuals: Vec<f64> = points
            .iter()
            .skip(self.window_size - 1)
            .zip(moving_avg.iter())
            .map(|(p, ma)| p.value - ma.value)
            .collect();

        if residuals.is_empty() {
            return anomalies;
        }

        let mean_residual: f64 = residuals.iter().sum::<f64>() / residuals.len() as f64;
        let variance: f64 = residuals
            .iter()
            .map(|r| (r - mean_residual).powi(2))
            .sum::<f64>()
            / residuals.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return anomalies;
        }

        // Detect points outside threshold
        for (i, ((point, ma), residual)) in points
            .iter()
            .skip(self.window_size - 1)
            .zip(moving_avg.iter())
            .zip(residuals.iter())
            .enumerate()
        {
            let z_score = (residual - mean_residual) / std_dev;

            if z_score.abs() > self.anomaly_threshold {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                };

                anomalies.push(Anomaly {
                    timestamp: point.timestamp,
                    value: point.value,
                    expected_value: ma.value,
                    deviation: z_score,
                    anomaly_type,
                    confidence: 1.0 - 1.0 / (1.0 + z_score.abs()),
                });
            }
        }

        anomalies
    }

    /// Detect seasonality patterns
    fn detect_seasonality(
        &self,
        values: &[f64],
        timestamps: &[DateTime<Utc>],
    ) -> Option<SeasonalityAnalysis> {
        if values.len() < 24 {
            return None;
        }

        // Try different periods
        let periods = [
            (24, SeasonalityType::Daily),
            (168, SeasonalityType::Weekly),
            (720, SeasonalityType::Monthly),
        ];

        let mut best_seasonality: Option<SeasonalityAnalysis> = None;
        let mut best_strength = 0.0;

        for (period, seasonality_type) in &periods {
            if values.len() < *period * 2 {
                continue;
            }

            let strength = self.calculate_seasonality_strength(values, *period);

            if strength > 0.3 && strength > best_strength {
                best_strength = strength;
                best_seasonality = Some(SeasonalityAnalysis {
                    period: *period,
                    strength,
                    seasonality_type: *seasonality_type,
                });
            }
        }

        best_seasonality
    }

    /// Calculate seasonality strength using autocorrelation
    fn calculate_seasonality_strength(&self, values: &[f64], period: usize) -> f64 {
        if values.len() < period * 2 {
            return 0.0;
        }

        let n = values.len();
        let mean: f64 = values.iter().sum::<f64>() / n as f64;

        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;

        if variance == 0.0 {
            return 0.0;
        }

        let mut autocorr = 0.0;
        for i in 0..(n - period) {
            autocorr += (values[i] - mean) * (values[i + period] - mean);
        }
        autocorr /= (n - period) as f64 * variance;

        autocorr.max(0.0).min(1.0)
    }

    /// Generate forecast
    fn forecast(
        &self,
        points: &[DataPoint],
        slope: f64,
        intercept: f64,
        periods_ahead: usize,
    ) -> Vec<DataPoint> {
        if points.is_empty() {
            return Vec::new();
        }

        let last_idx = points.len() as f64;
        let last_timestamp = points.last().unwrap().timestamp;

        // Estimate interval between points
        let interval = if points.len() > 1 {
            let total_duration = points.last().unwrap().timestamp - points.first().unwrap().timestamp;
            Duration::seconds(total_duration.num_seconds() / (points.len() - 1) as i64)
        } else {
            Duration::hours(1)
        };

        (1..=periods_ahead)
            .map(|i| {
                let x = last_idx + i as f64;
                let value = slope * x + intercept;
                DataPoint {
                    timestamp: last_timestamp + interval * i as i32,
                    value: value.max(0.0),
                }
            })
            .collect()
    }

    /// Compare two time series
    pub fn compare(&self, series_a: &TimeSeries, series_b: &TimeSeries) -> TrendComparison {
        let analysis_a = self.analyze(series_a);
        let analysis_b = self.analyze(series_b);

        // Calculate correlation
        let correlation = self.calculate_correlation(series_a, series_b);

        // Determine if trends are similar
        let trends_aligned = analysis_a.direction == analysis_b.direction;

        TrendComparison {
            series_a_name: series_a.name.clone(),
            series_b_name: series_b.name.clone(),
            correlation,
            trends_aligned,
            analysis_a,
            analysis_b,
        }
    }

    /// Calculate correlation between two series
    fn calculate_correlation(&self, series_a: &TimeSeries, series_b: &TimeSeries) -> f64 {
        let values_a = series_a.values();
        let values_b = series_b.values();

        let n = values_a.len().min(values_b.len());
        if n < 2 {
            return 0.0;
        }

        let mean_a: f64 = values_a[..n].iter().sum::<f64>() / n as f64;
        let mean_b: f64 = values_b[..n].iter().sum::<f64>() / n as f64;

        let mut cov = 0.0;
        let mut var_a = 0.0;
        let mut var_b = 0.0;

        for i in 0..n {
            let diff_a = values_a[i] - mean_a;
            let diff_b = values_b[i] - mean_b;
            cov += diff_a * diff_b;
            var_a += diff_a * diff_a;
            var_b += diff_b * diff_b;
        }

        if var_a == 0.0 || var_b == 0.0 {
            return 0.0;
        }

        cov / (var_a.sqrt() * var_b.sqrt())
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison of two time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendComparison {
    pub series_a_name: String,
    pub series_b_name: String,
    pub correlation: f64,
    pub trends_aligned: bool,
    pub analysis_a: TrendAnalysis,
    pub analysis_b: TrendAnalysis,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_linear_series(slope: f64, intercept: f64, n: usize) -> TimeSeries {
        let mut series = TimeSeries::new("test", "units");
        let base_time = Utc::now() - Duration::hours(n as i64);

        for i in 0..n {
            let timestamp = base_time + Duration::hours(i as i64);
            let value = slope * i as f64 + intercept;
            series.add(timestamp, value);
        }

        series
    }

    #[test]
    fn test_linear_regression() {
        let analyzer = TrendAnalyzer::new();
        let values: Vec<f64> = (0..20).map(|i| 2.0 * i as f64 + 5.0).collect();

        let (slope, intercept, r_squared) = analyzer.linear_regression(&values);

        assert!((slope - 2.0).abs() < 0.01);
        assert!((intercept - 5.0).abs() < 0.01);
        assert!(r_squared > 0.99);
    }

    #[test]
    fn test_increasing_trend() {
        let series = create_linear_series(1.0, 10.0, 30);
        let analyzer = TrendAnalyzer::new();
        let analysis = analyzer.analyze(&series);

        assert_eq!(analysis.direction, TrendDirection::Increasing);
        assert!(analysis.change_percent > 0.0);
    }

    #[test]
    fn test_decreasing_trend() {
        let series = create_linear_series(-0.5, 50.0, 30);
        let analyzer = TrendAnalyzer::new();
        let analysis = analyzer.analyze(&series);

        assert_eq!(analysis.direction, TrendDirection::Decreasing);
        assert!(analysis.change_percent < 0.0);
    }

    #[test]
    fn test_moving_average() {
        let analyzer = TrendAnalyzer::new().with_window(3);
        let points: Vec<DataPoint> = (0..10)
            .map(|i| DataPoint {
                timestamp: Utc::now() + Duration::hours(i),
                value: i as f64,
            })
            .collect();

        let ma = analyzer.moving_average(&points, 3);

        assert_eq!(ma.len(), 8);
        assert!((ma[0].value - 1.0).abs() < 0.01); // Average of 0, 1, 2
    }

    #[test]
    fn test_anomaly_detection() {
        let mut series = create_linear_series(1.0, 10.0, 30);

        // Add an anomaly
        series.points[15].value = 100.0; // Big spike

        let analyzer = TrendAnalyzer::new().with_anomaly_threshold(2.0);
        let analysis = analyzer.analyze(&series);

        assert!(!analysis.anomalies.is_empty());
        assert!(analysis.anomalies.iter().any(|a| a.anomaly_type == AnomalyType::Spike));
    }

    #[test]
    fn test_forecast() {
        let series = create_linear_series(2.0, 10.0, 20);
        let analyzer = TrendAnalyzer::new();
        let analysis = analyzer.analyze(&series);

        let forecast = analysis.forecast.unwrap();
        assert_eq!(forecast.len(), 7);

        // Forecast should continue the trend
        assert!(forecast[0].value > series.points.last().unwrap().value);
    }

    #[test]
    fn test_correlation() {
        let series_a = create_linear_series(1.0, 0.0, 20);
        let series_b = create_linear_series(2.0, 5.0, 20); // Same direction, different scale

        let analyzer = TrendAnalyzer::new();
        let correlation = analyzer.calculate_correlation(&series_a, &series_b);

        assert!(correlation > 0.99); // Nearly perfect correlation
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Linear regression accuracy
   - Moving average calculation
   - Trend detection correctness
   - Anomaly detection sensitivity

2. **Integration Tests**
   - Full analysis pipeline
   - Multi-series comparison
   - Forecast accuracy

3. **Statistical Tests**
   - R-squared validity
   - Correlation accuracy

---

## Related Specs

- Spec 406: Analytics Types
- Spec 410: Analytics Aggregation
- Spec 421: Report Generation
