# 418 - Session Tracking

## Overview

Session management for analytics, tracking user sessions across page views and events with configurable timeout and attribution.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/session.rs

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session timeout duration
    pub timeout: Duration,
    /// Maximum session duration
    pub max_duration: Duration,
    /// Whether to create new session on referrer change
    pub new_session_on_referrer_change: bool,
    /// Whether to create new session on UTM change
    pub new_session_on_utm_change: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::minutes(30),
            max_duration: Duration::hours(24),
            new_session_on_referrer_change: true,
            new_session_on_utm_change: true,
        }
    }
}

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// Distinct user ID
    pub distinct_id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Session end time (if ended)
    pub ended_at: Option<DateTime<Utc>>,
    /// Number of page views
    pub page_views: u32,
    /// Number of events
    pub event_count: u32,
    /// Entry URL
    pub entry_url: Option<String>,
    /// Entry referrer
    pub referrer: Option<String>,
    /// UTM parameters at session start
    pub utm: Option<UtmParams>,
    /// Landing page
    pub landing_page: Option<String>,
    /// Exit page
    pub exit_page: Option<String>,
    /// Device info
    pub device: Option<DeviceInfo>,
    /// Geographic info
    pub geo: Option<GeoInfo>,
    /// Custom session properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Is session currently active
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtmParams {
    pub source: Option<String>,
    pub medium: Option<String>,
    pub campaign: Option<String>,
    pub term: Option<String>,
    pub content: Option<String>,
}

impl UtmParams {
    pub fn from_query_params(params: &HashMap<String, String>) -> Option<Self> {
        let utm = Self {
            source: params.get("utm_source").cloned(),
            medium: params.get("utm_medium").cloned(),
            campaign: params.get("utm_campaign").cloned(),
            term: params.get("utm_term").cloned(),
            content: params.get("utm_content").cloned(),
        };

        if utm.source.is_some() || utm.medium.is_some() || utm.campaign.is_some() {
            Some(utm)
        } else {
            None
        }
    }

    pub fn is_different(&self, other: &UtmParams) -> bool {
        self.source != other.source ||
        self.medium != other.medium ||
        self.campaign != other.campaign
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub browser: Option<String>,
    pub browser_version: Option<String>,
    pub screen_resolution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoInfo {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
}

impl Session {
    pub fn new(distinct_id: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            distinct_id: distinct_id.to_string(),
            started_at: now,
            last_activity: now,
            ended_at: None,
            page_views: 0,
            event_count: 0,
            entry_url: None,
            referrer: None,
            utm: None,
            landing_page: None,
            exit_page: None,
            device: None,
            geo: None,
            properties: HashMap::new(),
            is_active: true,
        }
    }

    /// Session duration in seconds
    pub fn duration_seconds(&self) -> i64 {
        let end = self.ended_at.unwrap_or(self.last_activity);
        (end - self.started_at).num_seconds()
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self, config: &SessionConfig) -> bool {
        Utc::now() - self.last_activity > config.timeout
    }

    /// Check if session has exceeded max duration
    pub fn is_expired(&self, config: &SessionConfig) -> bool {
        Utc::now() - self.started_at > config.max_duration
    }

    /// Record activity
    pub fn record_activity(&mut self) {
        self.last_activity = Utc::now();
        self.event_count += 1;
    }

    /// Record page view
    pub fn record_pageview(&mut self, url: &str) {
        self.last_activity = Utc::now();
        self.page_views += 1;
        self.event_count += 1;

        if self.landing_page.is_none() {
            self.landing_page = Some(url.to_string());
        }
        self.exit_page = Some(url.to_string());
    }

    /// End the session
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        self.is_active = false;
    }
}

/// Session manager
pub struct SessionManager {
    config: SessionConfig,
    /// Active sessions by distinct_id
    sessions: RwLock<HashMap<String, Session>>,
    /// Session storage
    storage: Arc<dyn SessionStorage>,
}

/// Session storage trait
#[async_trait::async_trait]
pub trait SessionStorage: Send + Sync {
    async fn get(&self, session_id: &str) -> Result<Option<Session>, SessionError>;
    async fn get_by_user(&self, distinct_id: &str) -> Result<Option<Session>, SessionError>;
    async fn save(&self, session: &Session) -> Result<(), SessionError>;
    async fn end(&self, session_id: &str) -> Result<(), SessionError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Storage error: {0}")]
    Storage(String),
}

impl SessionManager {
    pub fn new(config: SessionConfig, storage: Arc<dyn SessionStorage>) -> Self {
        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            storage,
        }
    }

    /// Get or create session for user
    pub async fn get_or_create(
        &self,
        distinct_id: &str,
        context: SessionContext,
    ) -> Result<Session, SessionError> {
        // Check cache first
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(distinct_id) {
                if !session.is_timed_out(&self.config) && !session.is_expired(&self.config) {
                    // Check if we should create new session due to attribution change
                    if !self.should_start_new_session(session, &context) {
                        return Ok(session.clone());
                    }
                }
            }
        }

        // Check storage
        if let Some(session) = self.storage.get_by_user(distinct_id).await? {
            if !session.is_timed_out(&self.config) && !session.is_expired(&self.config) {
                if !self.should_start_new_session(&session, &context) {
                    // Cache it
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(distinct_id.to_string(), session.clone());
                    return Ok(session);
                }
            }
        }

        // Create new session
        let mut session = Session::new(distinct_id);
        session.entry_url = context.url.clone();
        session.referrer = context.referrer.clone();
        session.utm = context.utm.clone();
        session.device = context.device.clone();
        session.geo = context.geo.clone();

        // Save to storage
        self.storage.save(&session).await?;

        // Cache it
        let mut sessions = self.sessions.write().await;
        sessions.insert(distinct_id.to_string(), session.clone());

        Ok(session)
    }

    /// Update session activity
    pub async fn update(&self, session_id: &str, event_type: EventType) -> Result<Session, SessionError> {
        let mut sessions = self.sessions.write().await;

        // Find session
        let session = sessions.values_mut()
            .find(|s| s.id == session_id)
            .ok_or(SessionError::NotFound)?;

        match event_type {
            EventType::Pageview(url) => session.record_pageview(&url),
            EventType::Event => session.record_activity(),
        }

        let session = session.clone();
        self.storage.save(&session).await?;

        Ok(session)
    }

    /// End a session
    pub async fn end_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.values_mut().find(|s| s.id == session_id) {
            session.end();
            self.storage.end(&session.id).await?;
        }

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup(&self) {
        let mut sessions = self.sessions.write().await;

        let expired: Vec<_> = sessions.iter()
            .filter(|(_, s)| s.is_timed_out(&self.config) || s.is_expired(&self.config))
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired {
            if let Some(mut session) = sessions.remove(&key) {
                session.end();
                let _ = self.storage.save(&session).await;
            }
        }
    }

    fn should_start_new_session(&self, session: &Session, context: &SessionContext) -> bool {
        // Check referrer change
        if self.config.new_session_on_referrer_change {
            if let (Some(ref current), Some(ref new)) = (&session.referrer, &context.referrer) {
                if self.is_significant_referrer_change(current, new) {
                    return true;
                }
            }
        }

        // Check UTM change
        if self.config.new_session_on_utm_change {
            if let (Some(ref current), Some(ref new)) = (&session.utm, &context.utm) {
                if current.is_different(new) {
                    return true;
                }
            }
        }

        false
    }

    fn is_significant_referrer_change(&self, current: &str, new: &str) -> bool {
        // Extract domains and compare
        let current_domain = extract_domain(current);
        let new_domain = extract_domain(new);

        current_domain != new_domain
    }
}

fn extract_domain(url: &str) -> Option<String> {
    url::Url::parse(url).ok().map(|u| u.host_str().unwrap_or("").to_string())
}

/// Context for session creation/lookup
#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub url: Option<String>,
    pub referrer: Option<String>,
    pub utm: Option<UtmParams>,
    pub device: Option<DeviceInfo>,
    pub geo: Option<GeoInfo>,
}

/// Event type for session updates
pub enum EventType {
    Pageview(String),
    Event,
}

/// In-memory session storage (for development)
pub struct InMemorySessionStorage {
    sessions: RwLock<HashMap<String, Session>>,
}

impl InMemorySessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl SessionStorage for InMemorySessionStorage {
    async fn get(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    async fn get_by_user(&self, distinct_id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values()
            .find(|s| s.distinct_id == distinct_id && s.is_active)
            .cloned())
    }

    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn end(&self, session_id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.end();
        }
        Ok(())
    }
}

/// Session replay recording (optional feature)
pub mod replay {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReplayEvent {
        pub timestamp: DateTime<Utc>,
        pub event_type: ReplayEventType,
        pub data: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ReplayEventType {
        Dom,
        Mouse,
        Scroll,
        Input,
        Viewport,
        Custom,
    }

    pub struct SessionReplay {
        pub session_id: String,
        pub events: Vec<ReplayEvent>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let manager = SessionManager::new(SessionConfig::default(), storage);

        let session = manager.get_or_create("user-123", SessionContext::default()).await.unwrap();

        assert_eq!(session.distinct_id, "user-123");
        assert!(session.is_active);
        assert_eq!(session.page_views, 0);
    }

    #[tokio::test]
    async fn test_session_pageview() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let manager = SessionManager::new(SessionConfig::default(), storage);

        let session = manager.get_or_create("user-123", SessionContext::default()).await.unwrap();
        let updated = manager.update(&session.id, EventType::Pageview("/home".to_string())).await.unwrap();

        assert_eq!(updated.page_views, 1);
        assert_eq!(updated.landing_page, Some("/home".to_string()));
    }

    #[test]
    fn test_utm_parsing() {
        let mut params = HashMap::new();
        params.insert("utm_source".to_string(), "google".to_string());
        params.insert("utm_medium".to_string(), "cpc".to_string());

        let utm = UtmParams::from_query_params(&params).unwrap();
        assert_eq!(utm.source, Some("google".to_string()));
        assert_eq!(utm.medium, Some("cpc".to_string()));
    }
}
```

## Session Properties

| Property | Description |
|----------|-------------|
| $session_id | Unique session identifier |
| $session_duration | Duration in seconds |
| $session_page_views | Number of page views |
| $entry_url | First URL in session |
| $exit_url | Last URL in session |
| $referrer | Entry referrer |

## Related Specs

- 417-user-identification.md - User tracking
- 419-pageview-tracking.md - Page views
- 416-event-aggregation.md - Session aggregation
