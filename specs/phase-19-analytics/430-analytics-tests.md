# 430 - Analytics Tests

## Overview

Testing utilities for analytics including mock implementations, test helpers, and integration test patterns.

## Rust Implementation

```rust
// crates/analytics/src/testing.rs

use crate::event_types::{AnalyticsEvent, EventCategory};
use crate::session::{Session, SessionStorage, SessionError};
use crate::query::{AnalyticsQuery, QueryResult, QueryError, QueryExecutor};
use crate::webhooks::{WebhookConfig, WebhookDelivery, WebhookStorage, WebhookError, TriggerType};
use crate::pageview::Pageview;
use crate::errors::ErrorEvent;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Test analytics client
pub struct TestAnalytics {
    /// Captured events
    events: Arc<RwLock<Vec<AnalyticsEvent>>>,
    /// Captured identifications
    identifications: Arc<RwLock<Vec<IdentifyCall>>>,
    /// Captured pageviews
    pageviews: Arc<RwLock<Vec<Pageview>>>,
    /// Captured errors
    errors: Arc<RwLock<Vec<ErrorEvent>>>,
    /// Whether to fail operations
    should_fail: Arc<RwLock<bool>>,
    /// Delay before operations complete (ms)
    delay_ms: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
pub struct IdentifyCall {
    pub distinct_id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl TestAnalytics {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            identifications: Arc::new(RwLock::new(Vec::new())),
            pageviews: Arc::new(RwLock::new(Vec::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
            should_fail: Arc::new(RwLock::new(false)),
            delay_ms: Arc::new(RwLock::new(0)),
        }
    }

    /// Capture an event
    pub async fn capture(&self, event: AnalyticsEvent) -> Result<(), String> {
        let delay = *self.delay_ms.read().await;
        if delay > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }

        if *self.should_fail.read().await {
            return Err("Simulated failure".to_string());
        }

        self.events.write().await.push(event);
        Ok(())
    }

    /// Identify a user
    pub async fn identify(
        &self,
        distinct_id: &str,
        properties: HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        if *self.should_fail.read().await {
            return Err("Simulated failure".to_string());
        }

        self.identifications.write().await.push(IdentifyCall {
            distinct_id: distinct_id.to_string(),
            properties,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Track a pageview
    pub async fn page(&self, pageview: Pageview) -> Result<(), String> {
        if *self.should_fail.read().await {
            return Err("Simulated failure".to_string());
        }

        self.pageviews.write().await.push(pageview);
        Ok(())
    }

    /// Track an error
    pub async fn error(&self, error: ErrorEvent) -> Result<(), String> {
        if *self.should_fail.read().await {
            return Err("Simulated failure".to_string());
        }

        self.errors.write().await.push(error);
        Ok(())
    }

    /// Get all captured events
    pub async fn get_events(&self) -> Vec<AnalyticsEvent> {
        self.events.read().await.clone()
    }

    /// Get events by name
    pub async fn get_events_by_name(&self, name: &str) -> Vec<AnalyticsEvent> {
        self.events.read().await
            .iter()
            .filter(|e| e.event == name)
            .cloned()
            .collect()
    }

    /// Get last event
    pub async fn get_last_event(&self) -> Option<AnalyticsEvent> {
        self.events.read().await.last().cloned()
    }

    /// Get event count
    pub async fn event_count(&self) -> usize {
        self.events.read().await.len()
    }

    /// Get all identifications
    pub async fn get_identifications(&self) -> Vec<IdentifyCall> {
        self.identifications.read().await.clone()
    }

    /// Get all pageviews
    pub async fn get_pageviews(&self) -> Vec<Pageview> {
        self.pageviews.read().await.clone()
    }

    /// Get all errors
    pub async fn get_errors(&self) -> Vec<ErrorEvent> {
        self.errors.read().await.clone()
    }

    /// Clear all captured data
    pub async fn reset(&self) {
        self.events.write().await.clear();
        self.identifications.write().await.clear();
        self.pageviews.write().await.clear();
        self.errors.write().await.clear();
    }

    /// Set failure mode
    pub async fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.write().await = should_fail;
    }

    /// Set operation delay
    pub async fn set_delay(&self, delay_ms: u64) {
        *self.delay_ms.write().await = delay_ms;
    }

    /// Assert an event was captured
    pub async fn assert_event_captured(&self, event_name: &str) {
        let events = self.events.read().await;
        assert!(
            events.iter().any(|e| e.event == event_name),
            "Expected event '{}' to be captured. Captured events: {:?}",
            event_name,
            events.iter().map(|e| &e.event).collect::<Vec<_>>()
        );
    }

    /// Assert an event with properties was captured
    pub async fn assert_event_with_properties(
        &self,
        event_name: &str,
        expected_props: &HashMap<String, serde_json::Value>,
    ) {
        let events = self.events.read().await;
        let matching = events.iter().find(|e| {
            if e.event != event_name {
                return false;
            }
            expected_props.iter().all(|(k, v)| {
                e.properties.get(k) == Some(v)
            })
        });

        assert!(
            matching.is_some(),
            "Expected event '{}' with properties {:?}. Captured events: {:?}",
            event_name,
            expected_props,
            events
        );
    }

    /// Assert no events were captured
    pub async fn assert_no_events(&self) {
        let events = self.events.read().await;
        assert!(
            events.is_empty(),
            "Expected no events, but {} were captured: {:?}",
            events.len(),
            events.iter().map(|e| &e.event).collect::<Vec<_>>()
        );
    }

    /// Assert event count
    pub async fn assert_event_count(&self, expected: usize) {
        let count = self.events.read().await.len();
        assert_eq!(
            count, expected,
            "Expected {} events, but {} were captured",
            expected, count
        );
    }
}

impl Default for TestAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock session storage
pub struct MockSessionStorage {
    sessions: RwLock<HashMap<String, Session>>,
    should_fail: RwLock<bool>,
}

impl MockSessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            should_fail: RwLock::new(false),
        }
    }

    pub async fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.write().await = should_fail;
    }
}

impl Default for MockSessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStorage for MockSessionStorage {
    async fn get(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        if *self.should_fail.read().await {
            return Err(SessionError::Storage("Simulated failure".to_string()));
        }
        Ok(self.sessions.read().await.get(session_id).cloned())
    }

    async fn get_by_user(&self, distinct_id: &str) -> Result<Option<Session>, SessionError> {
        if *self.should_fail.read().await {
            return Err(SessionError::Storage("Simulated failure".to_string()));
        }
        let sessions = self.sessions.read().await;
        Ok(sessions.values()
            .find(|s| s.distinct_id == distinct_id && s.is_active)
            .cloned())
    }

    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        if *self.should_fail.read().await {
            return Err(SessionError::Storage("Simulated failure".to_string()));
        }
        self.sessions.write().await.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn end(&self, session_id: &str) -> Result<(), SessionError> {
        if *self.should_fail.read().await {
            return Err(SessionError::Storage("Simulated failure".to_string()));
        }
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.end();
        }
        Ok(())
    }
}

/// Mock query executor
pub struct MockQueryExecutor {
    results: RwLock<HashMap<String, QueryResult>>,
    calls: RwLock<Vec<AnalyticsQuery>>,
}

impl MockQueryExecutor {
    pub fn new() -> Self {
        Self {
            results: RwLock::new(HashMap::new()),
            calls: RwLock::new(Vec::new()),
        }
    }

    /// Set result for a query type
    pub async fn set_result(&self, query_type: &str, result: QueryResult) {
        self.results.write().await.insert(query_type.to_string(), result);
    }

    /// Get query calls
    pub async fn get_calls(&self) -> Vec<AnalyticsQuery> {
        self.calls.read().await.clone()
    }

    /// Clear calls
    pub async fn clear_calls(&self) {
        self.calls.write().await.clear();
    }
}

impl Default for MockQueryExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueryExecutor for MockQueryExecutor {
    async fn execute(&self, query: AnalyticsQuery) -> Result<QueryResult, QueryError> {
        self.calls.write().await.push(query.clone());

        let query_type = match &query {
            AnalyticsQuery::Trends(_) => "trends",
            AnalyticsQuery::Funnel(_) => "funnel",
            AnalyticsQuery::Retention(_) => "retention",
            AnalyticsQuery::Paths(_) => "paths",
            AnalyticsQuery::Stickiness(_) => "stickiness",
            AnalyticsQuery::Lifecycle(_) => "lifecycle",
            AnalyticsQuery::Events(_) => "events",
        };

        self.results.read().await
            .get(query_type)
            .cloned()
            .ok_or_else(|| QueryError::Invalid("No mock result configured".to_string()))
    }
}

/// Mock webhook storage
pub struct MockWebhookStorage {
    webhooks: RwLock<HashMap<String, WebhookConfig>>,
    deliveries: RwLock<Vec<WebhookDelivery>>,
}

impl MockWebhookStorage {
    pub fn new() -> Self {
        Self {
            webhooks: RwLock::new(HashMap::new()),
            deliveries: RwLock::new(Vec::new()),
        }
    }
}

impl Default for MockWebhookStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebhookStorage for MockWebhookStorage {
    async fn get_webhook(&self, id: &str) -> Result<Option<WebhookConfig>, WebhookError> {
        Ok(self.webhooks.read().await.get(id).cloned())
    }

    async fn save_webhook(&self, webhook: &WebhookConfig) -> Result<(), WebhookError> {
        self.webhooks.write().await.insert(webhook.id.clone(), webhook.clone());
        Ok(())
    }

    async fn delete_webhook(&self, id: &str) -> Result<(), WebhookError> {
        self.webhooks.write().await.remove(id);
        Ok(())
    }

    async fn list_webhooks(&self) -> Result<Vec<WebhookConfig>, WebhookError> {
        Ok(self.webhooks.read().await.values().cloned().collect())
    }

    async fn list_by_trigger(&self, trigger_type: TriggerType) -> Result<Vec<WebhookConfig>, WebhookError> {
        Ok(self.webhooks.read().await
            .values()
            .filter(|w| w.triggers.iter().any(|t| t.trigger_type == trigger_type))
            .cloned()
            .collect())
    }

    async fn save_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError> {
        self.deliveries.write().await.push(delivery.clone());
        Ok(())
    }

    async fn update_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError> {
        let mut deliveries = self.deliveries.write().await;
        if let Some(existing) = deliveries.iter_mut().find(|d| d.id == delivery.id) {
            *existing = delivery.clone();
        }
        Ok(())
    }

    async fn get_deliveries(&self, webhook_id: &str, limit: u32) -> Result<Vec<WebhookDelivery>, WebhookError> {
        Ok(self.deliveries.read().await
            .iter()
            .filter(|d| d.webhook_id == webhook_id)
            .take(limit as usize)
            .cloned()
            .collect())
    }

    async fn get_pending_retries(&self) -> Result<Vec<WebhookDelivery>, WebhookError> {
        Ok(self.deliveries.read().await
            .iter()
            .filter(|d| d.status == crate::webhooks::DeliveryStatus::Retrying)
            .cloned()
            .collect())
    }
}

/// Event builder for tests
pub struct EventBuilder {
    event: String,
    distinct_id: String,
    properties: HashMap<String, serde_json::Value>,
    timestamp: DateTime<Utc>,
}

impl EventBuilder {
    pub fn new(event: &str) -> Self {
        Self {
            event: event.to_string(),
            distinct_id: "test-user".to_string(),
            properties: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn distinct_id(mut self, id: &str) -> Self {
        self.distinct_id = id.to_string();
        self
    }

    pub fn property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.properties.insert(key.to_string(), value.into());
        self
    }

    pub fn properties(mut self, props: HashMap<String, serde_json::Value>) -> Self {
        self.properties = props;
        self
    }

    pub fn timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn build(self) -> AnalyticsEvent {
        let mut event = AnalyticsEvent::new(&self.event, &self.distinct_id, EventCategory::Custom);
        event.properties = self.properties;
        event.timestamp = self.timestamp;
        event
    }
}

/// Test fixture for analytics
pub struct AnalyticsTestFixture {
    pub analytics: TestAnalytics,
    pub session_storage: Arc<MockSessionStorage>,
    pub query_executor: Arc<MockQueryExecutor>,
    pub webhook_storage: Arc<MockWebhookStorage>,
}

impl AnalyticsTestFixture {
    pub fn new() -> Self {
        Self {
            analytics: TestAnalytics::new(),
            session_storage: Arc::new(MockSessionStorage::new()),
            query_executor: Arc::new(MockQueryExecutor::new()),
            webhook_storage: Arc::new(MockWebhookStorage::new()),
        }
    }

    /// Reset all test state
    pub async fn reset(&self) {
        self.analytics.reset().await;
        // Reset other components as needed
    }
}

impl Default for AnalyticsTestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration test helpers
pub mod integration {
    use super::*;

    /// Create test events in bulk
    pub fn create_test_events(count: usize, event_name: &str) -> Vec<AnalyticsEvent> {
        (0..count)
            .map(|i| {
                EventBuilder::new(event_name)
                    .distinct_id(&format!("user-{}", i % 10))
                    .property("index", i as i64)
                    .build()
            })
            .collect()
    }

    /// Create test session
    pub fn create_test_session(distinct_id: &str) -> Session {
        Session::new(distinct_id)
    }

    /// Wait for async operations with timeout
    pub async fn wait_for<F, T>(
        timeout_ms: u64,
        poll_interval_ms: u64,
        condition: F,
    ) -> Result<T, &'static str>
    where
        F: Fn() -> Option<T>,
    {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let interval = std::time::Duration::from_millis(poll_interval_ms);

        loop {
            if let Some(result) = condition() {
                return Ok(result);
            }

            if start.elapsed() > timeout {
                return Err("Timeout waiting for condition");
            }

            tokio::time::sleep(interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_analytics_capture() {
        let analytics = TestAnalytics::new();

        let event = EventBuilder::new("test_event")
            .property("key", "value")
            .build();

        analytics.capture(event).await.unwrap();

        assert_eq!(analytics.event_count().await, 1);
        analytics.assert_event_captured("test_event").await;
    }

    #[tokio::test]
    async fn test_test_analytics_failure_mode() {
        let analytics = TestAnalytics::new();
        analytics.set_should_fail(true).await;

        let event = EventBuilder::new("test_event").build();
        let result = analytics.capture(event).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_builder() {
        let event = EventBuilder::new("purchase")
            .distinct_id("user-123")
            .property("amount", 99.99)
            .property("currency", "USD")
            .build();

        assert_eq!(event.event, "purchase");
        assert_eq!(event.distinct_id, "user-123");
        assert_eq!(
            event.properties.get("amount"),
            Some(&serde_json::json!(99.99))
        );
    }

    #[tokio::test]
    async fn test_mock_session_storage() {
        let storage = MockSessionStorage::new();

        let session = Session::new("user-123");
        storage.save(&session).await.unwrap();

        let retrieved = storage.get(&session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().distinct_id, "user-123");
    }

    #[tokio::test]
    async fn test_assert_event_with_properties() {
        let analytics = TestAnalytics::new();

        let event = EventBuilder::new("purchase")
            .property("amount", 100)
            .property("currency", "USD")
            .build();

        analytics.capture(event).await.unwrap();

        let mut expected_props = HashMap::new();
        expected_props.insert("amount".to_string(), serde_json::json!(100));

        analytics.assert_event_with_properties("purchase", &expected_props).await;
    }

    #[test]
    fn test_create_test_events() {
        let events = integration::create_test_events(100, "$pageview");
        assert_eq!(events.len(), 100);
        assert!(events.iter().all(|e| e.event == "$pageview"));
    }

    #[tokio::test]
    async fn test_analytics_fixture() {
        let fixture = AnalyticsTestFixture::new();

        let event = EventBuilder::new("test").build();
        fixture.analytics.capture(event).await.unwrap();

        assert_eq!(fixture.analytics.event_count().await, 1);

        fixture.reset().await;
        assert_eq!(fixture.analytics.event_count().await, 0);
    }
}
```

## TypeScript Testing Utilities

```typescript
// Test utilities for TypeScript SDK
import { Analytics, Event, Identify } from './analytics';

export class MockAnalytics implements Analytics {
  private events: Event[] = [];
  private identifies: Identify[] = [];
  private shouldFail = false;

  capture(event: Event): Promise<void> {
    if (this.shouldFail) {
      return Promise.reject(new Error('Simulated failure'));
    }
    this.events.push(event);
    return Promise.resolve();
  }

  identify(identify: Identify): Promise<void> {
    if (this.shouldFail) {
      return Promise.reject(new Error('Simulated failure'));
    }
    this.identifies.push(identify);
    return Promise.resolve();
  }

  // Test helpers
  getEvents(): Event[] {
    return [...this.events];
  }

  getEventsByName(name: string): Event[] {
    return this.events.filter(e => e.event === name);
  }

  getLastEvent(): Event | undefined {
    return this.events[this.events.length - 1];
  }

  reset(): void {
    this.events = [];
    this.identifies = [];
  }

  setShouldFail(shouldFail: boolean): void {
    this.shouldFail = shouldFail;
  }

  // Assertions
  assertEventCaptured(eventName: string): void {
    const found = this.events.some(e => e.event === eventName);
    if (!found) {
      throw new Error(
        `Expected event '${eventName}' to be captured. ` +
        `Captured: ${this.events.map(e => e.event).join(', ')}`
      );
    }
  }

  assertEventWithProperties(
    eventName: string,
    properties: Record<string, unknown>
  ): void {
    const found = this.events.find(e => {
      if (e.event !== eventName) return false;
      return Object.entries(properties).every(
        ([key, value]) => e.properties?.[key] === value
      );
    });

    if (!found) {
      throw new Error(
        `Expected event '${eventName}' with properties ${JSON.stringify(properties)}`
      );
    }
  }

  assertNoEvents(): void {
    if (this.events.length > 0) {
      throw new Error(
        `Expected no events, but ${this.events.length} were captured`
      );
    }
  }
}

// Jest/Vitest matchers
expect.extend({
  toHaveCapturedEvent(
    received: MockAnalytics,
    eventName: string
  ) {
    const events = received.getEvents();
    const pass = events.some(e => e.event === eventName);

    return {
      pass,
      message: () =>
        pass
          ? `Expected not to have captured event '${eventName}'`
          : `Expected to have captured event '${eventName}'`,
    };
  },

  toHaveCapturedEventWithProperties(
    received: MockAnalytics,
    eventName: string,
    properties: Record<string, unknown>
  ) {
    const events = received.getEventsByName(eventName);
    const pass = events.some(e =>
      Object.entries(properties).every(
        ([key, value]) => e.properties?.[key] === value
      )
    );

    return {
      pass,
      message: () =>
        pass
          ? `Expected not to have captured event '${eventName}' with properties`
          : `Expected to have captured event '${eventName}' with ${JSON.stringify(properties)}`,
    };
  },
});

// React testing utilities
import { render } from '@testing-library/react';
import { AnalyticsProvider } from './analytics-provider';

export function renderWithAnalytics(
  ui: React.ReactElement,
  analytics: MockAnalytics = new MockAnalytics()
) {
  return {
    analytics,
    ...render(
      <AnalyticsProvider client={analytics}>
        {ui}
      </AnalyticsProvider>
    ),
  };
}

// Usage example
describe('Analytics', () => {
  let analytics: MockAnalytics;

  beforeEach(() => {
    analytics = new MockAnalytics();
  });

  it('tracks button clicks', async () => {
    const { getByRole } = renderWithAnalytics(
      <Button onClick={() => analytics.capture({ event: 'button_click' })}>
        Click me
      </Button>,
      analytics
    );

    await userEvent.click(getByRole('button'));

    expect(analytics).toHaveCapturedEvent('button_click');
  });
});
```

## Related Specs

- 411-event-types.md - Event definitions
- 418-session-tracking.md - Session testing
- 410-flag-tests.md - Similar testing patterns
