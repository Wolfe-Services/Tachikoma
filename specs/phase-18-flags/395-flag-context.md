# 395 - Feature Flag Evaluation Context

## Overview

Detailed specification for evaluation context handling, property resolution, and context builders for different platforms.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags/src/context.rs

use crate::types::Properties;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// Full evaluation context with all available information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// User identification
    pub user: Option<UserContext>,
    /// Device/client information
    pub device: Option<DeviceContext>,
    /// Request context (IP, location, etc.)
    pub request: Option<RequestContext>,
    /// Application context
    pub application: Option<ApplicationContext>,
    /// Session context
    pub session: Option<SessionContext>,
    /// Current environment
    pub environment: EnvironmentContext,
    /// Custom properties
    pub custom: Properties,
    /// Context creation timestamp
    pub timestamp: DateTime<Utc>,
}

/// User identification and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// Primary user identifier
    pub id: Option<String>,
    /// Anonymous/device identifier
    pub anonymous_id: Option<String>,
    /// User email
    pub email: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// User's groups/segments
    pub groups: Vec<String>,
    /// Subscription plan
    pub plan: Option<String>,
    /// Account creation date
    pub created_at: Option<DateTime<Utc>>,
    /// Custom user properties
    pub properties: Properties,
}

/// Device and client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceContext {
    /// Device type (desktop, mobile, tablet)
    pub device_type: Option<DeviceType>,
    /// Operating system
    pub os: Option<String>,
    /// OS version
    pub os_version: Option<String>,
    /// Browser name
    pub browser: Option<String>,
    /// Browser version
    pub browser_version: Option<String>,
    /// Screen resolution
    pub screen_resolution: Option<String>,
    /// Device manufacturer
    pub manufacturer: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// Is mobile device
    pub is_mobile: bool,
    /// Device identifier
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Tv,
    Watch,
    Bot,
    Unknown,
}

/// Request/network context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Client IP address
    pub ip: Option<IpAddr>,
    /// Detected country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Detected region/state
    pub region: Option<String>,
    /// Detected city
    pub city: Option<String>,
    /// Detected timezone
    pub timezone: Option<String>,
    /// Request locale
    pub locale: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Referrer URL
    pub referrer: Option<String>,
    /// Request URL
    pub url: Option<String>,
    /// Request path
    pub path: Option<String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
}

/// Application context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationContext {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Build number
    pub build: Option<String>,
    /// SDK name
    pub sdk_name: Option<String>,
    /// SDK version
    pub sdk_version: Option<String>,
}

/// Session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session identifier
    pub id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Is first session for user
    pub is_first_session: bool,
    /// Page views in session
    pub page_views: u32,
    /// Events in session
    pub events_count: u32,
    /// Session duration so far (seconds)
    pub duration_seconds: u64,
}

/// Environment context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// Environment name
    pub name: String,
    /// Is production
    pub is_production: bool,
    /// Deployment region
    pub region: Option<String>,
    /// Server instance ID
    pub instance_id: Option<String>,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            user: None,
            device: None,
            request: None,
            application: None,
            session: None,
            environment: EnvironmentContext {
                name: "development".to_string(),
                is_production: false,
                region: None,
                instance_id: None,
            },
            custom: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the primary identifier for bucketing
    pub fn bucket_key(&self, bucket_by: &str) -> Option<String> {
        match bucket_by {
            "user_id" | "user.id" => self.user.as_ref()?.id.clone()
                .or_else(|| self.user.as_ref()?.anonymous_id.clone()),
            "anonymous_id" | "user.anonymous_id" => self.user.as_ref()?.anonymous_id.clone(),
            "device_id" | "device.device_id" => self.device.as_ref()?.device_id.clone(),
            "session_id" | "session.id" => self.session.as_ref().map(|s| s.id.clone()),
            _ => self.get_property(bucket_by)
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        }
    }

    /// Get a property by path (e.g., "user.plan", "device.os")
    pub fn get_property(&self, path: &str) -> Option<serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "user" => self.get_user_property(&parts[1..]),
            "device" => self.get_device_property(&parts[1..]),
            "request" => self.get_request_property(&parts[1..]),
            "application" | "app" => self.get_app_property(&parts[1..]),
            "session" => self.get_session_property(&parts[1..]),
            "environment" | "env" => self.get_env_property(&parts[1..]),
            "custom" => self.custom.get(parts.get(1).copied().unwrap_or(""))
                .cloned(),
            // Try as direct user property
            _ => self.get_user_property(&parts),
        }
    }

    fn get_user_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        let user = self.user.as_ref()?;

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "id" => user.id.as_ref().map(|s| serde_json::json!(s)),
            "anonymous_id" => user.anonymous_id.as_ref().map(|s| serde_json::json!(s)),
            "email" => user.email.as_ref().map(|s| serde_json::json!(s)),
            "name" => user.name.as_ref().map(|s| serde_json::json!(s)),
            "plan" => user.plan.as_ref().map(|s| serde_json::json!(s)),
            "groups" => Some(serde_json::json!(user.groups)),
            _ => user.properties.get(parts[0]).cloned(),
        }
    }

    fn get_device_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        let device = self.device.as_ref()?;

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "type" | "device_type" => device.device_type.map(|t| serde_json::json!(format!("{:?}", t).to_lowercase())),
            "os" => device.os.as_ref().map(|s| serde_json::json!(s)),
            "os_version" => device.os_version.as_ref().map(|s| serde_json::json!(s)),
            "browser" => device.browser.as_ref().map(|s| serde_json::json!(s)),
            "browser_version" => device.browser_version.as_ref().map(|s| serde_json::json!(s)),
            "is_mobile" => Some(serde_json::json!(device.is_mobile)),
            _ => None,
        }
    }

    fn get_request_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        let request = self.request.as_ref()?;

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "ip" => request.ip.map(|ip| serde_json::json!(ip.to_string())),
            "country" => request.country.as_ref().map(|s| serde_json::json!(s)),
            "region" => request.region.as_ref().map(|s| serde_json::json!(s)),
            "city" => request.city.as_ref().map(|s| serde_json::json!(s)),
            "timezone" => request.timezone.as_ref().map(|s| serde_json::json!(s)),
            "locale" => request.locale.as_ref().map(|s| serde_json::json!(s)),
            "url" => request.url.as_ref().map(|s| serde_json::json!(s)),
            "path" => request.path.as_ref().map(|s| serde_json::json!(s)),
            _ => None,
        }
    }

    fn get_app_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        let app = self.application.as_ref()?;

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "name" => Some(serde_json::json!(app.name)),
            "version" => Some(serde_json::json!(app.version)),
            "build" => app.build.as_ref().map(|s| serde_json::json!(s)),
            _ => None,
        }
    }

    fn get_session_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        let session = self.session.as_ref()?;

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "id" => Some(serde_json::json!(session.id)),
            "is_first_session" => Some(serde_json::json!(session.is_first_session)),
            "page_views" => Some(serde_json::json!(session.page_views)),
            "duration_seconds" => Some(serde_json::json!(session.duration_seconds)),
            _ => None,
        }
    }

    fn get_env_property(&self, parts: &[&str]) -> Option<serde_json::Value> {
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "name" => Some(serde_json::json!(self.environment.name)),
            "is_production" => Some(serde_json::json!(self.environment.is_production)),
            "region" => self.environment.region.as_ref().map(|s| serde_json::json!(s)),
            _ => None,
        }
    }

    /// Get user's groups for group targeting
    pub fn groups(&self) -> Vec<String> {
        self.user.as_ref()
            .map(|u| u.groups.clone())
            .unwrap_or_default()
    }
}

/// Builder for creating evaluation contexts
pub struct ContextBuilder {
    context: EvaluationContext,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: EvaluationContext::default(),
        }
    }

    pub fn user_id(mut self, id: &str) -> Self {
        let user = self.context.user.get_or_insert_with(|| UserContext {
            id: None,
            anonymous_id: None,
            email: None,
            name: None,
            groups: vec![],
            plan: None,
            created_at: None,
            properties: HashMap::new(),
        });
        user.id = Some(id.to_string());
        self
    }

    pub fn anonymous_id(mut self, id: &str) -> Self {
        let user = self.context.user.get_or_insert_with(|| UserContext {
            id: None,
            anonymous_id: None,
            email: None,
            name: None,
            groups: vec![],
            plan: None,
            created_at: None,
            properties: HashMap::new(),
        });
        user.anonymous_id = Some(id.to_string());
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        if let Some(user) = &mut self.context.user {
            user.email = Some(email.to_string());
        }
        self
    }

    pub fn plan(mut self, plan: &str) -> Self {
        if let Some(user) = &mut self.context.user {
            user.plan = Some(plan.to_string());
        }
        self
    }

    pub fn group(mut self, group: &str) -> Self {
        if let Some(user) = &mut self.context.user {
            user.groups.push(group.to_string());
        }
        self
    }

    pub fn user_property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        if let Some(user) = &mut self.context.user {
            user.properties.insert(key.to_string(), value.into());
        }
        self
    }

    pub fn device_type(mut self, device_type: DeviceType) -> Self {
        let device = self.context.device.get_or_insert_with(|| DeviceContext {
            device_type: None,
            os: None,
            os_version: None,
            browser: None,
            browser_version: None,
            screen_resolution: None,
            manufacturer: None,
            model: None,
            is_mobile: false,
            device_id: None,
        });
        device.device_type = Some(device_type);
        device.is_mobile = matches!(device_type, DeviceType::Mobile | DeviceType::Tablet);
        self
    }

    pub fn os(mut self, os: &str, version: Option<&str>) -> Self {
        let device = self.context.device.get_or_insert_with(|| DeviceContext {
            device_type: None,
            os: None,
            os_version: None,
            browser: None,
            browser_version: None,
            screen_resolution: None,
            manufacturer: None,
            model: None,
            is_mobile: false,
            device_id: None,
        });
        device.os = Some(os.to_string());
        device.os_version = version.map(|s| s.to_string());
        self
    }

    pub fn browser(mut self, browser: &str, version: Option<&str>) -> Self {
        let device = self.context.device.get_or_insert_with(|| DeviceContext {
            device_type: None,
            os: None,
            os_version: None,
            browser: None,
            browser_version: None,
            screen_resolution: None,
            manufacturer: None,
            model: None,
            is_mobile: false,
            device_id: None,
        });
        device.browser = Some(browser.to_string());
        device.browser_version = version.map(|s| s.to_string());
        self
    }

    pub fn ip(mut self, ip: IpAddr) -> Self {
        let request = self.context.request.get_or_insert_with(|| RequestContext {
            ip: None,
            country: None,
            region: None,
            city: None,
            timezone: None,
            locale: None,
            user_agent: None,
            referrer: None,
            url: None,
            path: None,
            query_params: HashMap::new(),
        });
        request.ip = Some(ip);
        self
    }

    pub fn country(mut self, country: &str) -> Self {
        if let Some(request) = &mut self.context.request {
            request.country = Some(country.to_string());
        }
        self
    }

    pub fn locale(mut self, locale: &str) -> Self {
        let request = self.context.request.get_or_insert_with(|| RequestContext {
            ip: None,
            country: None,
            region: None,
            city: None,
            timezone: None,
            locale: None,
            user_agent: None,
            referrer: None,
            url: None,
            path: None,
            query_params: HashMap::new(),
        });
        request.locale = Some(locale.to_string());
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        let request = self.context.request.get_or_insert_with(|| RequestContext {
            ip: None,
            country: None,
            region: None,
            city: None,
            timezone: None,
            locale: None,
            user_agent: None,
            referrer: None,
            url: None,
            path: None,
            query_params: HashMap::new(),
        });
        request.url = Some(url.to_string());
        self
    }

    pub fn application(mut self, name: &str, version: &str) -> Self {
        self.context.application = Some(ApplicationContext {
            name: name.to_string(),
            version: version.to_string(),
            build: None,
            sdk_name: None,
            sdk_version: None,
        });
        self
    }

    pub fn environment(mut self, name: &str, is_production: bool) -> Self {
        self.context.environment = EnvironmentContext {
            name: name.to_string(),
            is_production,
            region: None,
            instance_id: None,
        };
        self
    }

    pub fn custom(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.context.custom.insert(key.to_string(), value.into());
        self
    }

    pub fn build(self) -> EvaluationContext {
        self.context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new()
            .user_id("user-123")
            .email("user@example.com")
            .plan("premium")
            .group("beta-testers")
            .device_type(DeviceType::Desktop)
            .browser("Chrome", Some("120.0"))
            .country("US")
            .environment("production", true)
            .build();

        assert_eq!(context.user.as_ref().unwrap().id, Some("user-123".to_string()));
        assert!(context.environment.is_production);
    }

    #[test]
    fn test_property_resolution() {
        let context = ContextBuilder::new()
            .user_id("user-123")
            .plan("premium")
            .user_property("role", "admin")
            .build();

        assert_eq!(
            context.get_property("user.plan"),
            Some(serde_json::json!("premium"))
        );
        assert_eq!(
            context.get_property("user.role"),
            Some(serde_json::json!("admin"))
        );
        // Direct property access (without namespace)
        assert_eq!(
            context.get_property("plan"),
            Some(serde_json::json!("premium"))
        );
    }

    #[test]
    fn test_bucket_key() {
        let context = ContextBuilder::new()
            .user_id("user-123")
            .anonymous_id("anon-456")
            .build();

        assert_eq!(context.bucket_key("user_id"), Some("user-123".to_string()));
        assert_eq!(context.bucket_key("anonymous_id"), Some("anon-456".to_string()));
    }
}
```

## TypeScript Context Builder

```typescript
// packages/flags/src/context.ts

export interface EvaluationContext {
  user?: UserContext;
  device?: DeviceContext;
  request?: RequestContext;
  application?: ApplicationContext;
  session?: SessionContext;
  environment: EnvironmentContext;
  custom: Record<string, unknown>;
  timestamp: Date;
}

export interface UserContext {
  id?: string;
  anonymousId?: string;
  email?: string;
  name?: string;
  groups: string[];
  plan?: string;
  createdAt?: Date;
  properties: Record<string, unknown>;
}

export class ContextBuilder {
  private context: EvaluationContext;

  constructor() {
    this.context = {
      environment: { name: 'development', isProduction: false },
      custom: {},
      timestamp: new Date(),
    };
  }

  userId(id: string): this {
    this.ensureUser();
    this.context.user!.id = id;
    return this;
  }

  anonymousId(id: string): this {
    this.ensureUser();
    this.context.user!.anonymousId = id;
    return this;
  }

  email(email: string): this {
    this.ensureUser();
    this.context.user!.email = email;
    return this;
  }

  plan(plan: string): this {
    this.ensureUser();
    this.context.user!.plan = plan;
    return this;
  }

  group(group: string): this {
    this.ensureUser();
    this.context.user!.groups.push(group);
    return this;
  }

  userProperty(key: string, value: unknown): this {
    this.ensureUser();
    this.context.user!.properties[key] = value;
    return this;
  }

  custom(key: string, value: unknown): this {
    this.context.custom[key] = value;
    return this;
  }

  environment(name: string, isProduction: boolean): this {
    this.context.environment = { name, isProduction };
    return this;
  }

  build(): EvaluationContext {
    return this.context;
  }

  private ensureUser(): void {
    if (!this.context.user) {
      this.context.user = {
        groups: [],
        properties: {},
      };
    }
  }
}
```

## Related Specs

- 394-flag-evaluation.md - Evaluation engine
- 396-percentage-rollout.md - Rollout based on context
- 397-user-targeting.md - User targeting
