# 419 - Pageview Tracking

## Overview

Automatic and manual pageview tracking with support for SPAs, virtual pages, and engagement metrics.

## Rust Implementation

```rust
// crates/analytics/src/pageview.rs

use crate::event_types::{AnalyticsEvent, EventCategory, PageviewEvent, UtmParams};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pageview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pageview {
    /// Full URL
    pub url: String,
    /// URL path
    pub path: String,
    /// Page title
    pub title: Option<String>,
    /// Referrer URL
    pub referrer: Option<String>,
    /// Host/domain
    pub host: String,
    /// Query parameters
    pub query_params: HashMap<String, String>,
    /// Hash fragment
    pub hash: Option<String>,
    /// UTM parameters
    pub utm: Option<UtmParams>,
    /// Time spent on page (set on next pageview or exit)
    pub time_on_page_ms: Option<u64>,
    /// Scroll depth (percentage)
    pub scroll_depth: Option<u8>,
    /// Content engagement score
    pub engagement_score: Option<f32>,
    /// Is virtual pageview (SPA navigation)
    pub is_virtual: bool,
    /// Previous page URL
    pub previous_url: Option<String>,
    /// Page load performance
    pub performance: Option<PagePerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagePerformance {
    /// DNS lookup time (ms)
    pub dns_time_ms: Option<u64>,
    /// TCP connection time (ms)
    pub tcp_time_ms: Option<u64>,
    /// TLS handshake time (ms)
    pub tls_time_ms: Option<u64>,
    /// Time to first byte (ms)
    pub ttfb_ms: Option<u64>,
    /// DOM content loaded (ms)
    pub dom_content_loaded_ms: Option<u64>,
    /// Page load complete (ms)
    pub page_load_ms: Option<u64>,
    /// First contentful paint (ms)
    pub fcp_ms: Option<u64>,
    /// Largest contentful paint (ms)
    pub lcp_ms: Option<u64>,
    /// First input delay (ms)
    pub fid_ms: Option<u64>,
    /// Cumulative layout shift
    pub cls: Option<f32>,
}

impl Pageview {
    /// Parse pageview from URL
    pub fn from_url(url: &str) -> Result<Self, PageviewError> {
        let parsed = url::Url::parse(url)
            .map_err(|e| PageviewError::InvalidUrl(e.to_string()))?;

        let query_params: HashMap<String, String> = parsed.query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let utm = UtmParams::from_query_params(&query_params);

        Ok(Self {
            url: url.to_string(),
            path: parsed.path().to_string(),
            title: None,
            referrer: None,
            host: parsed.host_str().unwrap_or("").to_string(),
            query_params,
            hash: parsed.fragment().map(|s| s.to_string()),
            utm,
            time_on_page_ms: None,
            scroll_depth: None,
            engagement_score: None,
            is_virtual: false,
            previous_url: None,
            performance: None,
        })
    }

    /// Convert to analytics event
    pub fn to_event(&self, distinct_id: &str) -> AnalyticsEvent {
        let mut event = AnalyticsEvent::new("$pageview", distinct_id, EventCategory::Pageview);

        event.properties.insert("$current_url".to_string(), serde_json::json!(self.url));
        event.properties.insert("$pathname".to_string(), serde_json::json!(self.path));
        event.properties.insert("$host".to_string(), serde_json::json!(self.host));

        if let Some(ref title) = self.title {
            event.properties.insert("$title".to_string(), serde_json::json!(title));
        }

        if let Some(ref referrer) = self.referrer {
            event.properties.insert("$referrer".to_string(), serde_json::json!(referrer));
            event.properties.insert("$referring_domain".to_string(),
                serde_json::json!(extract_domain(referrer).unwrap_or_default()));
        }

        if let Some(ref utm) = self.utm {
            if let Some(ref source) = utm.source {
                event.properties.insert("utm_source".to_string(), serde_json::json!(source));
            }
            if let Some(ref medium) = utm.medium {
                event.properties.insert("utm_medium".to_string(), serde_json::json!(medium));
            }
            if let Some(ref campaign) = utm.campaign {
                event.properties.insert("utm_campaign".to_string(), serde_json::json!(campaign));
            }
        }

        if let Some(time) = self.time_on_page_ms {
            event.properties.insert("$time_on_page_ms".to_string(), serde_json::json!(time));
        }

        if let Some(scroll) = self.scroll_depth {
            event.properties.insert("$scroll_depth".to_string(), serde_json::json!(scroll));
        }

        if self.is_virtual {
            event.properties.insert("$virtual_pageview".to_string(), serde_json::json!(true));
        }

        if let Some(ref performance) = self.performance {
            if let Some(lcp) = performance.lcp_ms {
                event.properties.insert("$lcp_ms".to_string(), serde_json::json!(lcp));
            }
            if let Some(fcp) = performance.fcp_ms {
                event.properties.insert("$fcp_ms".to_string(), serde_json::json!(fcp));
            }
            if let Some(cls) = performance.cls {
                event.properties.insert("$cls".to_string(), serde_json::json!(cls));
            }
        }

        event
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PageviewError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url).ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
}

/// Pageview tracker for client SDK
pub struct PageviewTracker {
    /// Current page
    current_page: Option<TrackedPage>,
    /// Track history length
    history_length: usize,
    /// Enable scroll tracking
    scroll_tracking: bool,
    /// Enable engagement tracking
    engagement_tracking: bool,
}

#[derive(Debug, Clone)]
struct TrackedPage {
    pageview: Pageview,
    entered_at: DateTime<Utc>,
    max_scroll_depth: u8,
    engagement_time_ms: u64,
}

impl PageviewTracker {
    pub fn new() -> Self {
        Self {
            current_page: None,
            history_length: 0,
            scroll_tracking: true,
            engagement_tracking: true,
        }
    }

    /// Track a new pageview
    pub fn track(&mut self, mut pageview: Pageview) -> Option<Pageview> {
        let now = Utc::now();

        // Finalize previous page if exists
        let previous = self.current_page.take().map(|mut prev| {
            let time_on_page = (now - prev.entered_at).num_milliseconds() as u64;
            prev.pageview.time_on_page_ms = Some(time_on_page);
            prev.pageview.scroll_depth = Some(prev.max_scroll_depth);
            prev.pageview
        });

        // Set previous URL
        if let Some(ref prev) = previous {
            pageview.previous_url = Some(prev.url.clone());
        }

        // Start tracking new page
        self.current_page = Some(TrackedPage {
            pageview,
            entered_at: now,
            max_scroll_depth: 0,
            engagement_time_ms: 0,
        });

        self.history_length += 1;

        previous
    }

    /// Update scroll depth
    pub fn update_scroll(&mut self, depth: u8) {
        if let Some(ref mut page) = self.current_page {
            if depth > page.max_scroll_depth {
                page.max_scroll_depth = depth;
            }
        }
    }

    /// Get current page for exit tracking
    pub fn finalize(&mut self) -> Option<Pageview> {
        self.current_page.take().map(|mut page| {
            let time_on_page = (Utc::now() - page.entered_at).num_milliseconds() as u64;
            page.pageview.time_on_page_ms = Some(time_on_page);
            page.pageview.scroll_depth = Some(page.max_scroll_depth);
            page.pageview
        })
    }
}

impl Default for PageviewTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Autocapture configuration for pageviews
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutocaptureConfig {
    /// Enable automatic pageview tracking
    pub enabled: bool,
    /// Track history push/replace state changes
    pub track_history_changes: bool,
    /// Track hash changes
    pub track_hash_changes: bool,
    /// URL patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Minimum time between pageviews (ms)
    pub debounce_ms: u64,
}

impl Default for AutocaptureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_history_changes: true,
            track_hash_changes: false,
            exclude_patterns: vec![],
            debounce_ms: 500,
        }
    }
}

/// Traffic source classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrafficSource {
    Direct,
    Organic,
    Paid,
    Social,
    Referral,
    Email,
    Affiliate,
    Display,
    Other,
}

impl TrafficSource {
    pub fn classify(referrer: Option<&str>, utm: Option<&UtmParams>) -> Self {
        // Check UTM first
        if let Some(utm) = utm {
            return Self::from_utm(utm);
        }

        // Check referrer
        if let Some(referrer) = referrer {
            return Self::from_referrer(referrer);
        }

        TrafficSource::Direct
    }

    fn from_utm(utm: &UtmParams) -> Self {
        match utm.medium.as_deref() {
            Some("cpc") | Some("ppc") | Some("paid") => TrafficSource::Paid,
            Some("organic") => TrafficSource::Organic,
            Some("social") => TrafficSource::Social,
            Some("email") => TrafficSource::Email,
            Some("affiliate") => TrafficSource::Affiliate,
            Some("display") | Some("banner") => TrafficSource::Display,
            Some("referral") => TrafficSource::Referral,
            _ => {
                // Try to infer from source
                match utm.source.as_deref() {
                    Some(s) if is_search_engine(s) => TrafficSource::Organic,
                    Some(s) if is_social_network(s) => TrafficSource::Social,
                    _ => TrafficSource::Other,
                }
            }
        }
    }

    fn from_referrer(referrer: &str) -> Self {
        if let Some(domain) = extract_domain(referrer) {
            if is_search_engine(&domain) {
                return TrafficSource::Organic;
            }
            if is_social_network(&domain) {
                return TrafficSource::Social;
            }
            return TrafficSource::Referral;
        }

        TrafficSource::Direct
    }
}

fn is_search_engine(domain: &str) -> bool {
    let search_engines = [
        "google", "bing", "yahoo", "duckduckgo", "baidu", "yandex",
        "ecosia", "ask", "aol",
    ];

    let domain_lower = domain.to_lowercase();
    search_engines.iter().any(|se| domain_lower.contains(se))
}

fn is_social_network(domain: &str) -> bool {
    let social_networks = [
        "facebook", "twitter", "linkedin", "instagram", "pinterest",
        "reddit", "tiktok", "youtube", "snapchat", "tumblr",
    ];

    let domain_lower = domain.to_lowercase();
    social_networks.iter().any(|sn| domain_lower.contains(sn))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pageview_parsing() {
        let pageview = Pageview::from_url(
            "https://example.com/page?utm_source=google&utm_medium=cpc&foo=bar"
        ).unwrap();

        assert_eq!(pageview.path, "/page");
        assert_eq!(pageview.host, "example.com");
        assert!(pageview.utm.is_some());

        let utm = pageview.utm.unwrap();
        assert_eq!(utm.source, Some("google".to_string()));
        assert_eq!(utm.medium, Some("cpc".to_string()));
    }

    #[test]
    fn test_traffic_source_classification() {
        // Direct
        assert_eq!(TrafficSource::classify(None, None), TrafficSource::Direct);

        // Organic search
        assert_eq!(
            TrafficSource::classify(Some("https://www.google.com/search?q=test"), None),
            TrafficSource::Organic
        );

        // Social
        assert_eq!(
            TrafficSource::classify(Some("https://twitter.com/user"), None),
            TrafficSource::Social
        );

        // UTM paid
        let utm = UtmParams {
            source: Some("google".to_string()),
            medium: Some("cpc".to_string()),
            campaign: Some("summer".to_string()),
            term: None,
            content: None,
        };
        assert_eq!(TrafficSource::classify(None, Some(&utm)), TrafficSource::Paid);
    }

    #[test]
    fn test_pageview_tracker() {
        let mut tracker = PageviewTracker::new();

        // First pageview
        let pv1 = Pageview::from_url("https://example.com/page1").unwrap();
        let prev = tracker.track(pv1);
        assert!(prev.is_none());

        // Update scroll
        tracker.update_scroll(50);
        tracker.update_scroll(75);

        // Second pageview
        let pv2 = Pageview::from_url("https://example.com/page2").unwrap();
        let prev = tracker.track(pv2);

        assert!(prev.is_some());
        let finalized = prev.unwrap();
        assert_eq!(finalized.scroll_depth, Some(75));
        assert!(finalized.time_on_page_ms.is_some());
    }
}
```

## TypeScript Client

```typescript
// Pageview tracking for browsers
class PageviewTracker {
  private currentPage: TrackedPage | null = null;

  track(url?: string): void {
    const pageview = this.createPageview(url || window.location.href);

    // Finalize previous page
    if (this.currentPage) {
      this.finalizePage();
    }

    // Start tracking new page
    this.currentPage = {
      pageview,
      enteredAt: Date.now(),
      maxScrollDepth: 0,
    };

    // Send pageview event
    analytics.capture('$pageview', pageview);
  }

  private createPageview(url: string): Pageview {
    return {
      url,
      path: new URL(url).pathname,
      title: document.title,
      referrer: document.referrer || undefined,
      host: window.location.host,
    };
  }
}
```

## Related Specs

- 418-session-tracking.md - Session context
- 420-action-tracking.md - User actions
- 422-performance-tracking.md - Page performance
