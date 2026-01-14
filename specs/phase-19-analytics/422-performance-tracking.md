# 422 - Performance Tracking

## Overview

Web Vitals and custom performance metrics tracking with real user monitoring (RUM) and server-side timing.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/performance.rs

use crate::event_types::{AnalyticsEvent, EventCategory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core Web Vitals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebVitals {
    /// Largest Contentful Paint (ms)
    pub lcp: Option<f64>,
    /// First Input Delay (ms)
    pub fid: Option<f64>,
    /// Cumulative Layout Shift
    pub cls: Option<f64>,
    /// Interaction to Next Paint (ms)
    pub inp: Option<f64>,
    /// Time to First Byte (ms)
    pub ttfb: Option<f64>,
    /// First Contentful Paint (ms)
    pub fcp: Option<f64>,
}

impl WebVitals {
    /// Rate the LCP value
    pub fn lcp_rating(&self) -> Option<VitalRating> {
        self.lcp.map(|lcp| {
            if lcp <= 2500.0 {
                VitalRating::Good
            } else if lcp <= 4000.0 {
                VitalRating::NeedsImprovement
            } else {
                VitalRating::Poor
            }
        })
    }

    /// Rate the FID value
    pub fn fid_rating(&self) -> Option<VitalRating> {
        self.fid.map(|fid| {
            if fid <= 100.0 {
                VitalRating::Good
            } else if fid <= 300.0 {
                VitalRating::NeedsImprovement
            } else {
                VitalRating::Poor
            }
        })
    }

    /// Rate the CLS value
    pub fn cls_rating(&self) -> Option<VitalRating> {
        self.cls.map(|cls| {
            if cls <= 0.1 {
                VitalRating::Good
            } else if cls <= 0.25 {
                VitalRating::NeedsImprovement
            } else {
                VitalRating::Poor
            }
        })
    }

    /// Rate the INP value
    pub fn inp_rating(&self) -> Option<VitalRating> {
        self.inp.map(|inp| {
            if inp <= 200.0 {
                VitalRating::Good
            } else if inp <= 500.0 {
                VitalRating::NeedsImprovement
            } else {
                VitalRating::Poor
            }
        })
    }

    /// Get overall score (0-100)
    pub fn overall_score(&self) -> f64 {
        let mut score = 0.0;
        let mut count = 0;

        if let Some(rating) = self.lcp_rating() {
            score += rating.score();
            count += 1;
        }
        if let Some(rating) = self.fid_rating() {
            score += rating.score();
            count += 1;
        }
        if let Some(rating) = self.cls_rating() {
            score += rating.score();
            count += 1;
        }
        if let Some(rating) = self.inp_rating() {
            score += rating.score();
            count += 1;
        }

        if count > 0 {
            score / count as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VitalRating {
    Good,
    NeedsImprovement,
    Poor,
}

impl VitalRating {
    pub fn score(&self) -> f64 {
        match self {
            VitalRating::Good => 100.0,
            VitalRating::NeedsImprovement => 50.0,
            VitalRating::Poor => 0.0,
        }
    }
}

/// Performance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEvent {
    /// Event type
    pub event_type: PerformanceEventType,
    /// Page URL
    pub url: String,
    /// Page path
    pub path: String,
    /// Web Vitals
    pub vitals: Option<WebVitals>,
    /// Navigation timing
    pub navigation: Option<NavigationTiming>,
    /// Resource timings
    pub resources: Vec<ResourceTiming>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Device info
    pub device: Option<DeviceInfo>,
    /// Connection info
    pub connection: Option<ConnectionInfo>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformanceEventType {
    PageLoad,
    SpaNavigation,
    WebVital,
    ResourceLoad,
    LongTask,
    CustomMetric,
}

/// Navigation timing (from Performance API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTiming {
    /// DNS lookup time (ms)
    pub dns_lookup: f64,
    /// TCP connection time (ms)
    pub tcp_connection: f64,
    /// TLS negotiation time (ms)
    pub tls_negotiation: Option<f64>,
    /// Time to first byte (ms)
    pub ttfb: f64,
    /// Content download time (ms)
    pub content_download: f64,
    /// DOM parsing time (ms)
    pub dom_parsing: f64,
    /// DOM content loaded (ms from navigation start)
    pub dom_content_loaded: f64,
    /// Load event (ms from navigation start)
    pub load_event: f64,
    /// Total page load time (ms)
    pub total_load_time: f64,
    /// Redirect time (ms)
    pub redirect_time: f64,
    /// Number of redirects
    pub redirect_count: u32,
    /// Transfer size (bytes)
    pub transfer_size: u64,
    /// Encoded body size (bytes)
    pub encoded_body_size: u64,
    /// Decoded body size (bytes)
    pub decoded_body_size: u64,
}

/// Resource timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTiming {
    /// Resource URL
    pub url: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Start time (ms from navigation start)
    pub start_time: f64,
    /// Duration (ms)
    pub duration: f64,
    /// Transfer size (bytes)
    pub transfer_size: u64,
    /// Cached
    pub from_cache: bool,
    /// Initiator type
    pub initiator_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Script,
    Stylesheet,
    Image,
    Font,
    Fetch,
    XmlHttpRequest,
    Other,
}

/// Device information for performance context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device memory (GB)
    pub memory_gb: Option<f64>,
    /// Hardware concurrency (CPU cores)
    pub cpu_cores: Option<u32>,
    /// Screen width
    pub screen_width: Option<u32>,
    /// Screen height
    pub screen_height: Option<u32>,
    /// Device pixel ratio
    pub pixel_ratio: Option<f64>,
    /// Is mobile device
    pub is_mobile: bool,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Effective connection type (4g, 3g, 2g, slow-2g)
    pub effective_type: Option<String>,
    /// Downlink bandwidth (Mbps)
    pub downlink: Option<f64>,
    /// Round-trip time (ms)
    pub rtt: Option<f64>,
    /// Save data mode enabled
    pub save_data: bool,
}

impl PerformanceEvent {
    /// Convert to analytics event
    pub fn to_event(&self, distinct_id: &str) -> AnalyticsEvent {
        let mut event = AnalyticsEvent::new("$performance", distinct_id, EventCategory::Performance);

        event.properties.insert("$perf_type".to_string(),
            serde_json::json!(format!("{:?}", self.event_type).to_lowercase()));
        event.properties.insert("$current_url".to_string(), serde_json::json!(self.url));
        event.properties.insert("$pathname".to_string(), serde_json::json!(self.path));

        if let Some(ref vitals) = self.vitals {
            if let Some(lcp) = vitals.lcp {
                event.properties.insert("$lcp".to_string(), serde_json::json!(lcp));
                event.properties.insert("$lcp_rating".to_string(),
                    serde_json::json!(format!("{:?}", vitals.lcp_rating().unwrap()).to_lowercase()));
            }
            if let Some(fid) = vitals.fid {
                event.properties.insert("$fid".to_string(), serde_json::json!(fid));
            }
            if let Some(cls) = vitals.cls {
                event.properties.insert("$cls".to_string(), serde_json::json!(cls));
            }
            if let Some(inp) = vitals.inp {
                event.properties.insert("$inp".to_string(), serde_json::json!(inp));
            }
            if let Some(ttfb) = vitals.ttfb {
                event.properties.insert("$ttfb".to_string(), serde_json::json!(ttfb));
            }
            if let Some(fcp) = vitals.fcp {
                event.properties.insert("$fcp".to_string(), serde_json::json!(fcp));
            }

            event.properties.insert("$perf_score".to_string(),
                serde_json::json!(vitals.overall_score()));
        }

        if let Some(ref nav) = self.navigation {
            event.properties.insert("$page_load_time".to_string(),
                serde_json::json!(nav.total_load_time));
            event.properties.insert("$dom_content_loaded".to_string(),
                serde_json::json!(nav.dom_content_loaded));
            event.properties.insert("$transfer_size".to_string(),
                serde_json::json!(nav.transfer_size));
        }

        if let Some(ref device) = self.device {
            event.properties.insert("$is_mobile".to_string(),
                serde_json::json!(device.is_mobile));
            if let Some(memory) = device.memory_gb {
                event.properties.insert("$device_memory".to_string(),
                    serde_json::json!(memory));
            }
        }

        if let Some(ref conn) = self.connection {
            if let Some(ref eff_type) = conn.effective_type {
                event.properties.insert("$connection_type".to_string(),
                    serde_json::json!(eff_type));
            }
        }

        // Custom metrics
        for (key, value) in &self.custom_metrics {
            event.properties.insert(key.clone(), serde_json::json!(value));
        }

        event
    }
}

/// Server-side performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTiming {
    /// Request ID
    pub request_id: String,
    /// Route/endpoint
    pub route: String,
    /// HTTP method
    pub method: String,
    /// Total request duration (ms)
    pub duration_ms: f64,
    /// Database query time (ms)
    pub db_time_ms: Option<f64>,
    /// External API call time (ms)
    pub external_time_ms: Option<f64>,
    /// CPU time (ms)
    pub cpu_time_ms: Option<f64>,
    /// Response size (bytes)
    pub response_size: u64,
    /// HTTP status code
    pub status_code: u16,
    /// Custom timing measurements
    pub timings: HashMap<String, f64>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ServerTiming {
    pub fn new(request_id: String, route: String, method: String) -> Self {
        Self {
            request_id,
            route,
            method,
            duration_ms: 0.0,
            db_time_ms: None,
            external_time_ms: None,
            cpu_time_ms: None,
            response_size: 0,
            status_code: 200,
            timings: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add timing measurement
    pub fn add_timing(&mut self, name: &str, duration_ms: f64) {
        self.timings.insert(name.to_string(), duration_ms);
    }

    /// Convert to Server-Timing header
    pub fn to_header(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("total;dur={:.2}", self.duration_ms));

        if let Some(db) = self.db_time_ms {
            parts.push(format!("db;dur={:.2}", db));
        }

        if let Some(ext) = self.external_time_ms {
            parts.push(format!("external;dur={:.2}", ext));
        }

        for (name, dur) in &self.timings {
            parts.push(format!("{};dur={:.2}", name, dur));
        }

        parts.join(", ")
    }

    /// Convert to analytics event
    pub fn to_event(&self) -> AnalyticsEvent {
        let mut event = AnalyticsEvent::new(
            "$server_timing",
            &self.request_id,
            EventCategory::Performance
        );

        event.properties.insert("$route".to_string(), serde_json::json!(self.route));
        event.properties.insert("$method".to_string(), serde_json::json!(self.method));
        event.properties.insert("$duration_ms".to_string(), serde_json::json!(self.duration_ms));
        event.properties.insert("$status_code".to_string(), serde_json::json!(self.status_code));
        event.properties.insert("$response_size".to_string(), serde_json::json!(self.response_size));

        if let Some(db) = self.db_time_ms {
            event.properties.insert("$db_time_ms".to_string(), serde_json::json!(db));
        }

        if let Some(ext) = self.external_time_ms {
            event.properties.insert("$external_time_ms".to_string(), serde_json::json!(ext));
        }

        for (name, dur) in &self.timings {
            event.properties.insert(format!("$timing_{}", name), serde_json::json!(dur));
        }

        event
    }
}

/// Performance budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBudget {
    /// LCP budget (ms)
    pub lcp_ms: Option<f64>,
    /// FID budget (ms)
    pub fid_ms: Option<f64>,
    /// CLS budget
    pub cls: Option<f64>,
    /// INP budget (ms)
    pub inp_ms: Option<f64>,
    /// Total page load budget (ms)
    pub page_load_ms: Option<f64>,
    /// JavaScript bundle size (bytes)
    pub js_bundle_size: Option<u64>,
    /// Total transfer size (bytes)
    pub total_transfer_size: Option<u64>,
    /// Custom metric budgets
    pub custom: HashMap<String, f64>,
}

impl PerformanceBudget {
    /// Check if performance event exceeds budget
    pub fn check(&self, event: &PerformanceEvent) -> Vec<BudgetViolation> {
        let mut violations = Vec::new();

        if let Some(ref vitals) = event.vitals {
            if let (Some(budget), Some(actual)) = (self.lcp_ms, vitals.lcp) {
                if actual > budget {
                    violations.push(BudgetViolation {
                        metric: "lcp".to_string(),
                        budget,
                        actual,
                        exceeded_by: actual - budget,
                    });
                }
            }

            if let (Some(budget), Some(actual)) = (self.fid_ms, vitals.fid) {
                if actual > budget {
                    violations.push(BudgetViolation {
                        metric: "fid".to_string(),
                        budget,
                        actual,
                        exceeded_by: actual - budget,
                    });
                }
            }

            if let (Some(budget), Some(actual)) = (self.cls, vitals.cls) {
                if actual > budget {
                    violations.push(BudgetViolation {
                        metric: "cls".to_string(),
                        budget,
                        actual,
                        exceeded_by: actual - budget,
                    });
                }
            }

            if let (Some(budget), Some(actual)) = (self.inp_ms, vitals.inp) {
                if actual > budget {
                    violations.push(BudgetViolation {
                        metric: "inp".to_string(),
                        budget,
                        actual,
                        exceeded_by: actual - budget,
                    });
                }
            }
        }

        if let Some(ref nav) = event.navigation {
            if let Some(budget) = self.page_load_ms {
                if nav.total_load_time > budget {
                    violations.push(BudgetViolation {
                        metric: "page_load".to_string(),
                        budget,
                        actual: nav.total_load_time,
                        exceeded_by: nav.total_load_time - budget,
                    });
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub metric: String,
    pub budget: f64,
    pub actual: f64,
    pub exceeded_by: f64,
}

/// Performance aggregator
pub struct PerformanceAggregator {
    /// Percentile calculator
    data: tokio::sync::RwLock<HashMap<String, Vec<f64>>>,
}

impl PerformanceAggregator {
    pub fn new() -> Self {
        Self {
            data: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Record a metric value
    pub async fn record(&self, metric: &str, value: f64) {
        let mut data = self.data.write().await;
        data.entry(metric.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Get percentile for a metric
    pub async fn percentile(&self, metric: &str, p: f64) -> Option<f64> {
        let data = self.data.read().await;
        let values = data.get(metric)?;

        if values.is_empty() {
            return None;
        }

        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[index])
    }

    /// Get summary statistics
    pub async fn summary(&self, metric: &str) -> Option<MetricSummary> {
        let data = self.data.read().await;
        let values = data.get(metric)?;

        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted[0];
        let max = sorted[count - 1];
        let p50 = sorted[count / 2];
        let p75 = sorted[(count as f64 * 0.75) as usize];
        let p95 = sorted[(count as f64 * 0.95) as usize];
        let p99 = sorted[(count as f64 * 0.99) as usize];

        Some(MetricSummary {
            count: count as u64,
            mean,
            min,
            max,
            p50,
            p75,
            p95,
            p99,
        })
    }
}

impl Default for PerformanceAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSummary {
    pub count: u64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
    pub p99: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_vitals_rating() {
        let vitals = WebVitals {
            lcp: Some(2000.0),  // Good
            fid: Some(50.0),   // Good
            cls: Some(0.15),   // Needs improvement
            inp: Some(600.0),  // Poor
            ttfb: Some(200.0),
            fcp: Some(1000.0),
        };

        assert_eq!(vitals.lcp_rating(), Some(VitalRating::Good));
        assert_eq!(vitals.fid_rating(), Some(VitalRating::Good));
        assert_eq!(vitals.cls_rating(), Some(VitalRating::NeedsImprovement));
        assert_eq!(vitals.inp_rating(), Some(VitalRating::Poor));
    }

    #[test]
    fn test_server_timing_header() {
        let mut timing = ServerTiming::new(
            "req-123".to_string(),
            "/api/users".to_string(),
            "GET".to_string(),
        );

        timing.duration_ms = 150.0;
        timing.db_time_ms = Some(45.0);
        timing.add_timing("cache", 5.0);

        let header = timing.to_header();
        assert!(header.contains("total;dur=150.00"));
        assert!(header.contains("db;dur=45.00"));
        assert!(header.contains("cache;dur=5.00"));
    }

    #[test]
    fn test_budget_violations() {
        let budget = PerformanceBudget {
            lcp_ms: Some(2500.0),
            fid_ms: Some(100.0),
            cls: Some(0.1),
            inp_ms: Some(200.0),
            page_load_ms: Some(3000.0),
            js_bundle_size: None,
            total_transfer_size: None,
            custom: HashMap::new(),
        };

        let event = PerformanceEvent {
            event_type: PerformanceEventType::PageLoad,
            url: "https://example.com".to_string(),
            path: "/".to_string(),
            vitals: Some(WebVitals {
                lcp: Some(3000.0),  // Exceeds budget
                fid: Some(50.0),    // Within budget
                cls: Some(0.05),    // Within budget
                inp: Some(100.0),   // Within budget
                ttfb: None,
                fcp: None,
            }),
            navigation: None,
            resources: Vec::new(),
            custom_metrics: HashMap::new(),
            device: None,
            connection: None,
            timestamp: Utc::now(),
        };

        let violations = budget.check(&event);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].metric, "lcp");
        assert_eq!(violations[0].exceeded_by, 500.0);
    }

    #[tokio::test]
    async fn test_performance_aggregator() {
        let agg = PerformanceAggregator::new();

        for i in 1..=100 {
            agg.record("lcp", i as f64).await;
        }

        let p50 = agg.percentile("lcp", 50.0).await.unwrap();
        assert!((p50 - 50.0).abs() < 1.0);

        let summary = agg.summary("lcp").await.unwrap();
        assert_eq!(summary.count, 100);
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 100.0);
    }
}
```

## TypeScript Client

```typescript
// Web Vitals tracking
import { onCLS, onFID, onLCP, onINP, onTTFB, onFCP } from 'web-vitals';

class PerformanceTracker {
  private metrics: Map<string, number> = new Map();

  constructor() {
    this.setupWebVitals();
    this.trackNavigationTiming();
    this.trackLongTasks();
  }

  private setupWebVitals(): void {
    onLCP((metric) => this.reportVital('lcp', metric.value));
    onFID((metric) => this.reportVital('fid', metric.value));
    onCLS((metric) => this.reportVital('cls', metric.value));
    onINP((metric) => this.reportVital('inp', metric.value));
    onTTFB((metric) => this.reportVital('ttfb', metric.value));
    onFCP((metric) => this.reportVital('fcp', metric.value));
  }

  private reportVital(name: string, value: number): void {
    this.metrics.set(name, value);

    analytics.capture('$web_vital', {
      $vital_name: name,
      $vital_value: value,
      $vital_rating: this.rateVital(name, value),
      $pathname: window.location.pathname,
    });
  }

  private rateVital(name: string, value: number): string {
    const thresholds: Record<string, [number, number]> = {
      lcp: [2500, 4000],
      fid: [100, 300],
      cls: [0.1, 0.25],
      inp: [200, 500],
      ttfb: [800, 1800],
      fcp: [1800, 3000],
    };

    const [good, poor] = thresholds[name] || [0, 0];
    if (value <= good) return 'good';
    if (value <= poor) return 'needs_improvement';
    return 'poor';
  }

  private trackNavigationTiming(): void {
    window.addEventListener('load', () => {
      setTimeout(() => {
        const nav = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        if (!nav) return;

        analytics.capture('$page_load', {
          $dns_lookup: nav.domainLookupEnd - nav.domainLookupStart,
          $tcp_connection: nav.connectEnd - nav.connectStart,
          $ttfb: nav.responseStart - nav.requestStart,
          $content_download: nav.responseEnd - nav.responseStart,
          $dom_parsing: nav.domContentLoadedEventStart - nav.responseEnd,
          $dom_content_loaded: nav.domContentLoadedEventStart - nav.fetchStart,
          $load_event: nav.loadEventStart - nav.fetchStart,
          $total_load_time: nav.loadEventEnd - nav.fetchStart,
          $transfer_size: nav.transferSize,
        });
      }, 0);
    });
  }

  private trackLongTasks(): void {
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          if (entry.duration > 50) {
            analytics.capture('$long_task', {
              $duration: entry.duration,
              $start_time: entry.startTime,
            });
          }
        }
      });

      observer.observe({ entryTypes: ['longtask'] });
    }
  }

  // Custom metric tracking
  measure(name: string, startMark: string, endMark: string): void {
    try {
      performance.measure(name, startMark, endMark);
      const measure = performance.getEntriesByName(name, 'measure')[0];
      if (measure) {
        analytics.capture('$custom_metric', {
          $metric_name: name,
          $metric_value: measure.duration,
        });
      }
    } catch (e) {
      console.warn('Performance measure failed:', e);
    }
  }
}
```

## Related Specs

- 419-pageview-tracking.md - Page performance context
- 421-error-tracking.md - Performance-related errors
- 427-dashboard-data.md - Performance dashboards
