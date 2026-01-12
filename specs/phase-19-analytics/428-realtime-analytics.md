# 428 - Realtime Analytics

## Overview

Real-time event streaming and live analytics with WebSocket connections for dashboards and monitoring.

## Rust Implementation

```rust
// crates/analytics/src/realtime.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use futures::{SinkExt, StreamExt};

/// Real-time event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeEvent {
    /// Event ID
    pub id: String,
    /// Event name
    pub event: String,
    /// Distinct ID
    pub distinct_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Session ID
    pub session_id: Option<String>,
    /// Environment
    pub environment: String,
}

/// Real-time subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription ID
    pub id: String,
    /// Events to subscribe to (empty = all)
    pub events: Vec<String>,
    /// Property filters
    pub filters: Vec<SubscriptionFilter>,
    /// Include person data
    pub include_person: bool,
    /// Environment filter
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    pub property: String,
    pub operator: String,
    pub value: serde_json::Value,
}

impl Subscription {
    pub fn matches(&self, event: &RealtimeEvent) -> bool {
        // Check event name
        if !self.events.is_empty() && !self.events.contains(&event.event) {
            return false;
        }

        // Check environment
        if let Some(ref env) = self.environment {
            if &event.environment != env {
                return false;
            }
        }

        // Check filters
        for filter in &self.filters {
            if !self.check_filter(filter, event) {
                return false;
            }
        }

        true
    }

    fn check_filter(&self, filter: &SubscriptionFilter, event: &RealtimeEvent) -> bool {
        let value = match event.properties.get(&filter.property) {
            Some(v) => v,
            None => return filter.operator == "is_not_set",
        };

        match filter.operator.as_str() {
            "equals" => value == &filter.value,
            "not_equals" => value != &filter.value,
            "contains" => {
                value.as_str()
                    .and_then(|v| filter.value.as_str().map(|f| v.contains(f)))
                    .unwrap_or(false)
            }
            "is_set" => true,
            "is_not_set" => false,
            _ => true,
        }
    }
}

/// Real-time hub for managing connections
pub struct RealtimeHub {
    /// Event broadcast channel
    tx: broadcast::Sender<RealtimeEvent>,
    /// Active connections
    connections: RwLock<HashMap<String, ConnectionInfo>>,
    /// Metrics
    metrics: RwLock<RealtimeMetrics>,
    /// Buffer for recent events
    event_buffer: RwLock<Vec<RealtimeEvent>>,
    /// Buffer size
    buffer_size: usize,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    pub id: String,
    pub subscription: Subscription,
    pub connected_at: DateTime<Utc>,
    pub events_sent: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RealtimeMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub events_per_second: f64,
    pub events_total: u64,
}

impl RealtimeHub {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(10000);

        Self {
            tx,
            connections: RwLock::new(HashMap::new()),
            metrics: RwLock::new(RealtimeMetrics::default()),
            event_buffer: RwLock::new(Vec::new()),
            buffer_size,
        }
    }

    /// Publish an event to all subscribers
    pub async fn publish(&self, event: RealtimeEvent) {
        // Add to buffer
        {
            let mut buffer = self.event_buffer.write().await;
            buffer.push(event.clone());
            if buffer.len() > self.buffer_size {
                buffer.remove(0);
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.events_total += 1;
        }

        // Broadcast to subscribers
        let _ = self.tx.send(event);
    }

    /// Subscribe to events
    pub async fn subscribe(&self, subscription: Subscription) -> RealtimeSubscriber {
        let id = subscription.id.clone();
        let rx = self.tx.subscribe();

        // Register connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(id.clone(), ConnectionInfo {
                id: id.clone(),
                subscription: subscription.clone(),
                connected_at: Utc::now(),
                events_sent: 0,
            });

            let mut metrics = self.metrics.write().await;
            metrics.total_connections += 1;
            metrics.active_connections += 1;
        }

        RealtimeSubscriber {
            id,
            subscription,
            rx,
            hub: self,
        }
    }

    /// Get recent events
    pub async fn get_recent_events(&self, count: usize) -> Vec<RealtimeEvent> {
        let buffer = self.event_buffer.read().await;
        buffer.iter().rev().take(count).cloned().collect()
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> RealtimeMetrics {
        self.metrics.read().await.clone()
    }

    /// Unsubscribe
    async fn unsubscribe(&self, id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(id);

        let mut metrics = self.metrics.write().await;
        metrics.active_connections = metrics.active_connections.saturating_sub(1);
    }
}

/// Real-time subscriber
pub struct RealtimeSubscriber<'a> {
    id: String,
    subscription: Subscription,
    rx: broadcast::Receiver<RealtimeEvent>,
    hub: &'a RealtimeHub,
}

impl<'a> RealtimeSubscriber<'a> {
    /// Receive next matching event
    pub async fn recv(&mut self) -> Option<RealtimeEvent> {
        loop {
            match self.rx.recv().await {
                Ok(event) => {
                    if self.subscription.matches(&event) {
                        return Some(event);
                    }
                }
                Err(broadcast::error::RecvError::Closed) => return None,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }
}

impl<'a> Drop for RealtimeSubscriber<'a> {
    fn drop(&mut self) {
        // Note: This is sync, we can't await here
        // In practice, use a separate cleanup task
    }
}

/// Live user count tracker
pub struct LiveUserTracker {
    /// Active users by time bucket
    users: RwLock<HashMap<String, UserActivity>>,
    /// Activity timeout (seconds)
    timeout_seconds: u64,
}

#[derive(Debug, Clone)]
struct UserActivity {
    pub distinct_id: String,
    pub last_seen: DateTime<Utc>,
    pub session_id: Option<String>,
    pub current_url: Option<String>,
}

impl LiveUserTracker {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            timeout_seconds,
        }
    }

    /// Record user activity
    pub async fn record_activity(
        &self,
        distinct_id: &str,
        session_id: Option<String>,
        current_url: Option<String>,
    ) {
        let mut users = self.users.write().await;
        users.insert(distinct_id.to_string(), UserActivity {
            distinct_id: distinct_id.to_string(),
            last_seen: Utc::now(),
            session_id,
            current_url,
        });
    }

    /// Get live user count
    pub async fn get_count(&self) -> u64 {
        let users = self.users.read().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.timeout_seconds as i64);

        users.values()
            .filter(|u| u.last_seen > cutoff)
            .count() as u64
    }

    /// Get live users with details
    pub async fn get_users(&self) -> Vec<LiveUser> {
        let users = self.users.read().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.timeout_seconds as i64);

        users.values()
            .filter(|u| u.last_seen > cutoff)
            .map(|u| LiveUser {
                distinct_id: u.distinct_id.clone(),
                session_id: u.session_id.clone(),
                current_url: u.current_url.clone(),
                last_seen: u.last_seen,
            })
            .collect()
    }

    /// Cleanup inactive users
    pub async fn cleanup(&self) {
        let mut users = self.users.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.timeout_seconds as i64 * 2);

        users.retain(|_, u| u.last_seen > cutoff);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveUser {
    pub distinct_id: String,
    pub session_id: Option<String>,
    pub current_url: Option<String>,
    pub last_seen: DateTime<Utc>,
}

/// Real-time aggregation
pub struct RealtimeAggregator {
    /// Aggregation windows
    windows: RwLock<HashMap<String, AggregationWindow>>,
    /// Window duration (seconds)
    window_seconds: u64,
}

#[derive(Debug, Clone)]
struct AggregationWindow {
    pub start: DateTime<Utc>,
    pub counts: HashMap<String, u64>,
    pub unique_users: std::collections::HashSet<String>,
}

impl RealtimeAggregator {
    pub fn new(window_seconds: u64) -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
            window_seconds,
        }
    }

    /// Record an event
    pub async fn record(&self, event: &str, distinct_id: &str) {
        let window_key = self.current_window_key();

        let mut windows = self.windows.write().await;
        let window = windows.entry(window_key).or_insert_with(|| AggregationWindow {
            start: Utc::now(),
            counts: HashMap::new(),
            unique_users: std::collections::HashSet::new(),
        });

        *window.counts.entry(event.to_string()).or_insert(0) += 1;
        window.unique_users.insert(distinct_id.to_string());
    }

    /// Get current counts
    pub async fn get_counts(&self) -> HashMap<String, u64> {
        let windows = self.windows.read().await;
        let current_key = self.current_window_key();

        windows.get(&current_key)
            .map(|w| w.counts.clone())
            .unwrap_or_default()
    }

    /// Get events per second
    pub async fn get_rate(&self, event: &str) -> f64 {
        let windows = self.windows.read().await;
        let current_key = self.current_window_key();

        windows.get(&current_key)
            .and_then(|w| w.counts.get(event))
            .map(|count| *count as f64 / self.window_seconds as f64)
            .unwrap_or(0.0)
    }

    fn current_window_key(&self) -> String {
        let now = Utc::now().timestamp();
        let window_start = now - (now % self.window_seconds as i64);
        window_start.to_string()
    }

    /// Cleanup old windows
    pub async fn cleanup(&self) {
        let mut windows = self.windows.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.window_seconds as i64 * 10);

        windows.retain(|_, w| w.start > cutoff);
    }
}

/// WebSocket handler for real-time events
pub async fn websocket_handler(
    ws: axum::extract::ws::WebSocket,
    hub: Arc<RealtimeHub>,
    user_tracker: Arc<LiveUserTracker>,
) {
    use axum::extract::ws::Message;

    let (mut sender, mut receiver) = ws.split();

    // Handle incoming messages
    let hub_clone = hub.clone();
    let send_task = tokio::spawn(async move {
        // Default subscription (all events)
        let subscription = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            events: vec![],
            filters: vec![],
            include_person: false,
            environment: None,
        };

        let mut subscriber = hub_clone.subscribe(subscription).await;

        while let Some(event) = subscriber.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle control messages
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Handle subscription updates
                    if let Ok(cmd) = serde_json::from_str::<WebSocketCommand>(&text) {
                        match cmd {
                            WebSocketCommand::Subscribe { events, filters } => {
                                // Update subscription
                                tracing::info!("Updated subscription: {:?}", events);
                            }
                            WebSocketCommand::Ping => {
                                // Keepalive
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WebSocketCommand {
    Subscribe {
        events: Vec<String>,
        filters: Vec<SubscriptionFilter>,
    },
    Ping,
}

/// Live dashboard data endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDashboardData {
    /// Live user count
    pub live_users: u64,
    /// Events in last minute
    pub events_per_minute: HashMap<String, u64>,
    /// Recent events
    pub recent_events: Vec<RealtimeEvent>,
    /// Top pages (live)
    pub top_pages: Vec<PageCount>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageCount {
    pub path: String,
    pub count: u64,
}

/// Server-Sent Events handler
pub async fn sse_handler(
    hub: Arc<RealtimeHub>,
    subscription: Subscription,
) -> impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>> {
    let mut subscriber = hub.subscribe(subscription).await;

    async_stream::stream! {
        while let Some(event) = subscriber.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            yield Ok(axum::response::sse::Event::default().data(json));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_matching() {
        let subscription = Subscription {
            id: "test".to_string(),
            events: vec!["$pageview".to_string()],
            filters: vec![],
            include_person: false,
            environment: Some("production".to_string()),
        };

        let event = RealtimeEvent {
            id: "1".to_string(),
            event: "$pageview".to_string(),
            distinct_id: "user-1".to_string(),
            timestamp: Utc::now(),
            properties: HashMap::new(),
            session_id: None,
            environment: "production".to_string(),
        };

        assert!(subscription.matches(&event));

        let wrong_event = RealtimeEvent {
            event: "$click".to_string(),
            ..event.clone()
        };
        assert!(!subscription.matches(&wrong_event));

        let wrong_env = RealtimeEvent {
            environment: "staging".to_string(),
            ..event.clone()
        };
        assert!(!subscription.matches(&wrong_env));
    }

    #[tokio::test]
    async fn test_realtime_hub() {
        let hub = RealtimeHub::new(100);

        let subscription = Subscription {
            id: uuid::Uuid::new_v4().to_string(),
            events: vec![],
            filters: vec![],
            include_person: false,
            environment: None,
        };

        let mut subscriber = hub.subscribe(subscription).await;

        // Publish event
        let event = RealtimeEvent {
            id: "1".to_string(),
            event: "test".to_string(),
            distinct_id: "user-1".to_string(),
            timestamp: Utc::now(),
            properties: HashMap::new(),
            session_id: None,
            environment: "test".to_string(),
        };

        hub.publish(event.clone()).await;

        // Should receive event
        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            subscriber.recv()
        ).await;

        assert!(received.is_ok());
    }

    #[tokio::test]
    async fn test_live_user_tracker() {
        let tracker = LiveUserTracker::new(30);

        tracker.record_activity("user-1", None, Some("/home".to_string())).await;
        tracker.record_activity("user-2", None, Some("/about".to_string())).await;

        assert_eq!(tracker.get_count().await, 2);

        let users = tracker.get_users().await;
        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    async fn test_realtime_aggregator() {
        let aggregator = RealtimeAggregator::new(60);

        aggregator.record("$pageview", "user-1").await;
        aggregator.record("$pageview", "user-2").await;
        aggregator.record("$click", "user-1").await;

        let counts = aggregator.get_counts().await;
        assert_eq!(counts.get("$pageview"), Some(&2));
        assert_eq!(counts.get("$click"), Some(&1));
    }
}
```

## TypeScript Client

```typescript
// Real-time analytics client
class RealtimeAnalytics {
  private ws: WebSocket | null = null;
  private eventSource: EventSource | null = null;
  private listeners: Map<string, Set<(event: RealtimeEvent) => void>> = new Map();

  constructor(private baseUrl: string) {}

  // WebSocket connection
  connectWebSocket(subscription?: Subscription): void {
    const url = `${this.baseUrl.replace('http', 'ws')}/api/realtime/ws`;
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      if (subscription) {
        this.ws?.send(JSON.stringify({
          type: 'subscribe',
          ...subscription,
        }));
      }
    };

    this.ws.onmessage = (msg) => {
      const event = JSON.parse(msg.data) as RealtimeEvent;
      this.dispatch(event);
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    this.ws.onclose = () => {
      // Reconnect after delay
      setTimeout(() => this.connectWebSocket(subscription), 5000);
    };
  }

  // Server-Sent Events connection
  connectSSE(subscription?: Subscription): void {
    const params = new URLSearchParams();
    if (subscription?.events) {
      params.set('events', subscription.events.join(','));
    }

    const url = `${this.baseUrl}/api/realtime/sse?${params}`;
    this.eventSource = new EventSource(url);

    this.eventSource.onmessage = (msg) => {
      const event = JSON.parse(msg.data) as RealtimeEvent;
      this.dispatch(event);
    };

    this.eventSource.onerror = () => {
      this.eventSource?.close();
      setTimeout(() => this.connectSSE(subscription), 5000);
    };
  }

  // Subscribe to specific events
  on(event: string, callback: (event: RealtimeEvent) => void): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);

    return () => {
      this.listeners.get(event)?.delete(callback);
    };
  }

  private dispatch(event: RealtimeEvent): void {
    // Event-specific listeners
    this.listeners.get(event.event)?.forEach(cb => cb(event));
    // Wildcard listeners
    this.listeners.get('*')?.forEach(cb => cb(event));
  }

  disconnect(): void {
    this.ws?.close();
    this.eventSource?.close();
  }
}

// React hook for real-time events
function useRealtimeEvents(events: string[]): RealtimeEvent[] {
  const [recentEvents, setRecentEvents] = useState<RealtimeEvent[]>([]);
  const realtime = useContext(RealtimeContext);

  useEffect(() => {
    const unsubscribes = events.map(event =>
      realtime.on(event, (e) => {
        setRecentEvents(prev => [e, ...prev.slice(0, 99)]);
      })
    );

    return () => unsubscribes.forEach(unsub => unsub());
  }, [events]);

  return recentEvents;
}
```

## Related Specs

- 413-event-capture.md - Event ingestion
- 427-dashboard-data.md - Dashboard updates
- 404-flag-sync.md - Similar streaming pattern
