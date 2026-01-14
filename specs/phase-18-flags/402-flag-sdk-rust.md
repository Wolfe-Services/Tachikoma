# 402 - Feature Flag SDK for Rust

## Overview

Rust SDK for evaluating feature flags with local caching, streaming updates, and offline support.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/flags-sdk/src/lib.rs

pub mod client;
pub mod cache;
pub mod streaming;
pub mod offline;

pub use client::{FlagClient, FlagClientConfig};
pub use cache::FlagCache;

// crates/flags-sdk/src/client.rs

use crate::cache::LocalCache;
use crate::streaming::StreamingClient;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum FlagError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Flag not found: {0}")]
    NotFound(String),
    #[error("Evaluation error: {0}")]
    EvaluationError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
}

/// Configuration for the flag client
#[derive(Debug, Clone)]
pub struct FlagClientConfig {
    /// API base URL
    pub api_url: String,
    /// SDK key for authentication
    pub sdk_key: String,
    /// Current environment
    pub environment: String,
    /// Enable streaming updates
    pub streaming_enabled: bool,
    /// Cache TTL for flag definitions
    pub cache_ttl: Duration,
    /// Enable offline mode
    pub offline_mode: bool,
    /// Request timeout
    pub timeout: Duration,
    /// Max retries for failed requests
    pub max_retries: u32,
}

impl Default for FlagClientConfig {
    fn default() -> Self {
        Self {
            api_url: "https://flags.example.com".to_string(),
            sdk_key: String::new(),
            environment: "production".to_string(),
            streaming_enabled: true,
            cache_ttl: Duration::from_secs(300),
            offline_mode: false,
            timeout: Duration::from_secs(10),
            max_retries: 3,
        }
    }
}

/// Evaluation context for flag evaluation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    /// User identifier
    pub user_id: Option<String>,
    /// Anonymous identifier
    pub anonymous_id: Option<String>,
    /// User's groups
    pub groups: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
    /// Current environment
    #[serde(skip)]
    pub environment: Option<String>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_anonymous(mut self, id: &str) -> Self {
        self.anonymous_id = Some(id.to_string());
        self
    }

    pub fn with_group(mut self, group: &str) -> Self {
        self.groups.push(group.to_string());
        self
    }

    pub fn with_property(mut self, key: &str, value: impl Into<serde_json::Value>) -> Self {
        self.properties.insert(key.to_string(), value.into());
        self
    }

    /// Get identifier for bucketing
    pub fn key(&self) -> Option<&str> {
        self.user_id.as_deref()
            .or(self.anonymous_id.as_deref())
    }
}

/// Feature flag client
pub struct FlagClient {
    config: FlagClientConfig,
    http_client: HttpClient,
    cache: Arc<LocalCache>,
    streaming: Option<StreamingClient>,
    ready: Arc<RwLock<bool>>,
}

impl FlagClient {
    /// Create a new flag client
    pub async fn new(config: FlagClientConfig) -> Result<Self, FlagError> {
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .build()?;

        let cache = Arc::new(LocalCache::new(config.cache_ttl));

        let mut client = Self {
            config: config.clone(),
            http_client,
            cache: cache.clone(),
            streaming: None,
            ready: Arc::new(RwLock::new(false)),
        };

        // Initial flag fetch
        client.fetch_all_flags().await?;

        // Start streaming if enabled
        if config.streaming_enabled {
            let streaming = StreamingClient::new(
                &config.api_url,
                &config.sdk_key,
                cache.clone(),
            );
            client.streaming = Some(streaming);
        }

        *client.ready.write().await = true;

        Ok(client)
    }

    /// Get boolean flag value
    pub async fn get_bool(&self, flag_key: &str, context: &Context, default: bool) -> bool {
        match self.evaluate(flag_key, context).await {
            Ok(value) => value.as_bool().unwrap_or(default),
            Err(_) => default,
        }
    }

    /// Get string flag value
    pub async fn get_string(&self, flag_key: &str, context: &Context, default: &str) -> String {
        match self.evaluate(flag_key, context).await {
            Ok(value) => value.as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| default.to_string()),
            Err(_) => default.to_string(),
        }
    }

    /// Get numeric flag value
    pub async fn get_number(&self, flag_key: &str, context: &Context, default: f64) -> f64 {
        match self.evaluate(flag_key, context).await {
            Ok(value) => value.as_f64().unwrap_or(default),
            Err(_) => default,
        }
    }

    /// Get JSON flag value
    pub async fn get_json(&self, flag_key: &str, context: &Context) -> Option<serde_json::Value> {
        self.evaluate(flag_key, context).await.ok()
    }

    /// Evaluate a flag with full result
    pub async fn evaluate(&self, flag_key: &str, context: &Context) -> Result<serde_json::Value, FlagError> {
        // Try cache first
        if let Some(result) = self.cache.get_evaluation(flag_key, context).await {
            return Ok(result);
        }

        // Check if we have the flag definition cached
        let flag = self.cache.get_flag(flag_key).await
            .ok_or_else(|| FlagError::NotFound(flag_key.to_string()))?;

        // Evaluate locally
        let result = self.evaluate_locally(&flag, context)?;

        // Cache the result
        self.cache.set_evaluation(flag_key, context, result.clone()).await;

        Ok(result)
    }

    /// Evaluate flag locally using cached definition
    fn evaluate_locally(&self, flag: &FlagDefinition, context: &Context) -> Result<serde_json::Value, FlagError> {
        // Check if flag is active
        if flag.status != "active" {
            return Ok(flag.default_value.clone());
        }

        // Check user overrides
        if let Some(user_id) = &context.user_id {
            if let Some(value) = flag.user_overrides.get(user_id) {
                return Ok(value.clone());
            }
        }

        // Check group overrides
        for group in &context.groups {
            if let Some(value) = flag.group_overrides.get(group) {
                return Ok(value.clone());
            }
        }

        // Evaluate rules
        for rule in &flag.rules {
            if self.evaluate_rule(rule, context) {
                return Ok(rule.value.clone());
            }
        }

        // Check rollout
        if let Some(rollout) = &flag.rollout {
            if let Some(key) = context.key() {
                let bucket = self.hash_to_bucket(flag_key, key);
                if bucket <= rollout.percentage {
                    // For boolean flags, return true when in rollout
                    if flag.default_value.is_boolean() {
                        return Ok(serde_json::json!(true));
                    }
                }
            }
        }

        // Check experiment
        if let Some(experiment) = &flag.experiment {
            if let Some(key) = context.key() {
                let variant = self.select_variant(flag_key, key, &experiment.variants);
                return Ok(variant.value.clone());
            }
        }

        Ok(flag.default_value.clone())
    }

    fn evaluate_rule(&self, rule: &Rule, context: &Context) -> bool {
        if !rule.enabled {
            return false;
        }

        for condition in &rule.conditions {
            let property_value = context.properties.get(&condition.property);
            if !self.evaluate_condition(condition, property_value) {
                return false;
            }
        }

        true
    }

    fn evaluate_condition(&self, condition: &Condition, value: Option<&serde_json::Value>) -> bool {
        match &condition.operator[..] {
            "equals" => value.map(|v| v == &condition.value).unwrap_or(false),
            "not_equals" => value.map(|v| v != &condition.value).unwrap_or(true),
            "contains" => {
                value.and_then(|v| v.as_str())
                    .map(|s| s.contains(condition.value.as_str().unwrap_or("")))
                    .unwrap_or(false)
            }
            "in" => {
                if let Some(list) = condition.value.as_array() {
                    value.map(|v| list.contains(v)).unwrap_or(false)
                } else {
                    false
                }
            }
            "exists" => value.is_some(),
            "not_exists" => value.is_none(),
            _ => false,
        }
    }

    fn hash_to_bucket(&self, flag_key: &str, user_key: &str) -> f64 {
        use sha2::{Sha256, Digest};

        let input = format!("{}:{}", flag_key, user_key);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        let bytes: [u8; 8] = result[..8].try_into().unwrap();
        let hash_int = u64::from_le_bytes(bytes);
        (hash_int as f64 / u64::MAX as f64) * 100.0
    }

    fn select_variant<'a>(&self, flag_key: &str, user_key: &str, variants: &'a [Variant]) -> &'a Variant {
        let bucket = self.hash_to_bucket(flag_key, user_key);
        let mut cumulative = 0.0;

        for variant in variants {
            cumulative += variant.weight;
            if bucket <= cumulative {
                return variant;
            }
        }

        variants.last().unwrap()
    }

    /// Fetch all flag definitions from server
    async fn fetch_all_flags(&self) -> Result<(), FlagError> {
        let url = format!("{}/sdk/flags", self.config.api_url);

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.sdk_key))
            .header("X-Environment", &self.config.environment)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(FlagError::Http(response.error_for_status().unwrap_err()));
        }

        let flags: Vec<FlagDefinition> = response.json().await?;

        for flag in flags {
            self.cache.set_flag(&flag.id, flag).await;
        }

        Ok(())
    }

    /// Track flag evaluation for analytics
    pub async fn track(&self, flag_key: &str, context: &Context, value: &serde_json::Value) {
        // Fire and forget tracking
        let url = format!("{}/sdk/track", self.config.api_url);
        let sdk_key = self.config.sdk_key.clone();
        let event = TrackEvent {
            flag_key: flag_key.to_string(),
            user_id: context.user_id.clone(),
            anonymous_id: context.anonymous_id.clone(),
            value: value.clone(),
            timestamp: chrono::Utc::now(),
        };

        let client = self.http_client.clone();
        tokio::spawn(async move {
            let _ = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", sdk_key))
                .json(&event)
                .send()
                .await;
        });
    }

    /// Check if the client is ready
    pub async fn is_ready(&self) -> bool {
        *self.ready.read().await
    }

    /// Force refresh all flags
    pub async fn refresh(&self) -> Result<(), FlagError> {
        self.fetch_all_flags().await
    }

    /// Close the client and cleanup resources
    pub async fn close(&self) {
        if let Some(streaming) = &self.streaming {
            streaming.close().await;
        }
        self.cache.clear().await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlagDefinition {
    id: String,
    name: String,
    status: String,
    default_value: serde_json::Value,
    rules: Vec<Rule>,
    rollout: Option<Rollout>,
    experiment: Option<Experiment>,
    user_overrides: HashMap<String, serde_json::Value>,
    group_overrides: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rule {
    id: String,
    enabled: bool,
    conditions: Vec<Condition>,
    value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Condition {
    property: String,
    operator: String,
    value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rollout {
    percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Experiment {
    variants: Vec<Variant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Variant {
    key: String,
    weight: f64,
    value: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TrackEvent {
    flag_key: String,
    user_id: Option<String>,
    anonymous_id: Option<String>,
    value: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Macro for easy flag evaluation
#[macro_export]
macro_rules! flag {
    ($client:expr, $key:expr, $context:expr, bool, $default:expr) => {
        $client.get_bool($key, $context, $default).await
    };
    ($client:expr, $key:expr, $context:expr, str, $default:expr) => {
        $client.get_string($key, $context, $default).await
    };
    ($client:expr, $key:expr, $context:expr, num, $default:expr) => {
        $client.get_number($key, $context, $default).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let context = Context::new()
            .with_user("user-123")
            .with_group("beta")
            .with_property("plan", "enterprise");

        assert_eq!(context.user_id, Some("user-123".to_string()));
        assert!(context.groups.contains(&"beta".to_string()));
        assert_eq!(context.properties.get("plan"), Some(&serde_json::json!("enterprise")));
    }
}
```

## Usage Example

```rust
use flags_sdk::{FlagClient, FlagClientConfig, Context};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let config = FlagClientConfig {
        api_url: "https://flags.example.com".to_string(),
        sdk_key: std::env::var("FLAGS_SDK_KEY")?,
        environment: "production".to_string(),
        ..Default::default()
    };

    let client = FlagClient::new(config).await?;

    // Create context
    let context = Context::new()
        .with_user("user-123")
        .with_property("plan", "enterprise");

    // Evaluate flags
    let enabled = client.get_bool("new-feature", &context, false).await;
    if enabled {
        println!("New feature is enabled!");
    }

    // Get variant for A/B test
    let variant = client.get_string("checkout-flow", &context, "control").await;
    println!("User is in variant: {}", variant);

    Ok(())
}
```

## Related Specs

- 394-flag-evaluation.md - Evaluation logic
- 403-flag-sdk-ts.md - TypeScript SDK
- 404-flag-sync.md - Synchronization
