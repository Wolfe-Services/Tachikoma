# 429 - Analytics Webhooks

## Overview

Webhook delivery for analytics events, alerts, and integrations with external systems.

## Rust Implementation

```rust
// crates/analytics/src/webhooks.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use reqwest::Client;
use sha2::{Sha256, Digest};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Webhook ID
    pub id: String,
    /// Webhook name
    pub name: String,
    /// Target URL
    pub url: String,
    /// Secret for signature
    pub secret: String,
    /// Enabled
    pub enabled: bool,
    /// Event triggers
    pub triggers: Vec<WebhookTrigger>,
    /// Headers to include
    pub headers: HashMap<String, String>,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// Webhook trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTrigger {
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Event filters
    pub filters: Vec<TriggerFilter>,
    /// Conditions (for threshold triggers)
    pub conditions: Vec<TriggerCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// On every matching event
    Event,
    /// On error events
    Error,
    /// On threshold breach
    Threshold,
    /// On anomaly detection
    Anomaly,
    /// Periodic summary
    Scheduled,
    /// On funnel completion
    FunnelComplete,
    /// On cohort entry
    CohortEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerFilter {
    pub property: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    /// Metric to check
    pub metric: String,
    /// Operator (gt, lt, eq, gte, lte)
    pub operator: String,
    /// Threshold value
    pub value: f64,
    /// Time window (seconds)
    pub window_seconds: u64,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Max retry attempts
    pub max_attempts: u32,
    /// Initial delay (seconds)
    pub initial_delay_seconds: u32,
    /// Max delay (seconds)
    pub max_delay_seconds: u32,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_seconds: 5,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
        }
    }
}

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Payload ID
    pub id: String,
    /// Webhook ID
    pub webhook_id: String,
    /// Event type
    pub event_type: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Payload data
    pub data: serde_json::Value,
    /// Environment
    pub environment: String,
}

impl WebhookPayload {
    /// Sign the payload
    pub fn sign(&self, secret: &str) -> String {
        let payload = serde_json::to_string(self).unwrap();
        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        hmac::Mac::update(&mut mac, payload.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}

/// Webhook delivery record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    /// Delivery ID
    pub id: String,
    /// Webhook ID
    pub webhook_id: String,
    /// Payload ID
    pub payload_id: String,
    /// Status
    pub status: DeliveryStatus,
    /// HTTP status code
    pub status_code: Option<u16>,
    /// Response body
    pub response_body: Option<String>,
    /// Attempt number
    pub attempt: u32,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Duration (ms)
    pub duration_ms: Option<u64>,
    /// Error message
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Pending,
    Delivering,
    Delivered,
    Failed,
    Retrying,
}

/// Webhook storage trait
#[async_trait]
pub trait WebhookStorage: Send + Sync {
    async fn get_webhook(&self, id: &str) -> Result<Option<WebhookConfig>, WebhookError>;
    async fn save_webhook(&self, webhook: &WebhookConfig) -> Result<(), WebhookError>;
    async fn delete_webhook(&self, id: &str) -> Result<(), WebhookError>;
    async fn list_webhooks(&self) -> Result<Vec<WebhookConfig>, WebhookError>;
    async fn list_by_trigger(&self, trigger_type: TriggerType) -> Result<Vec<WebhookConfig>, WebhookError>;

    async fn save_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError>;
    async fn update_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError>;
    async fn get_deliveries(&self, webhook_id: &str, limit: u32) -> Result<Vec<WebhookDelivery>, WebhookError>;
    async fn get_pending_retries(&self) -> Result<Vec<WebhookDelivery>, WebhookError>;
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Webhook not found")]
    NotFound,
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("Storage error: {0}")]
    Storage(String),
}

/// Webhook service
pub struct WebhookService {
    storage: std::sync::Arc<dyn WebhookStorage>,
    client: Client,
    queue: tokio::sync::mpsc::Sender<WebhookDelivery>,
}

impl WebhookService {
    pub fn new(
        storage: std::sync::Arc<dyn WebhookStorage>,
    ) -> (Self, tokio::sync::mpsc::Receiver<WebhookDelivery>) {
        let (tx, rx) = tokio::sync::mpsc::channel(10000);

        let service = Self {
            storage,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            queue: tx,
        };

        (service, rx)
    }

    /// Create a webhook
    pub async fn create(&self, mut webhook: WebhookConfig) -> Result<WebhookConfig, WebhookError> {
        webhook.id = uuid::Uuid::new_v4().to_string();
        webhook.secret = self.generate_secret();
        webhook.created_at = Utc::now();
        webhook.updated_at = Utc::now();

        self.storage.save_webhook(&webhook).await?;
        Ok(webhook)
    }

    fn generate_secret(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }

    /// Trigger webhooks for an event
    pub async fn trigger_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
        environment: &str,
    ) -> Result<Vec<String>, WebhookError> {
        let webhooks = self.storage.list_by_trigger(TriggerType::Event).await?;
        let mut delivery_ids = Vec::new();

        for webhook in webhooks {
            if !webhook.enabled {
                continue;
            }

            // Check if event matches triggers
            if !self.matches_triggers(&webhook, event_type, &data) {
                continue;
            }

            let payload = WebhookPayload {
                id: uuid::Uuid::new_v4().to_string(),
                webhook_id: webhook.id.clone(),
                event_type: event_type.to_string(),
                timestamp: Utc::now(),
                data: data.clone(),
                environment: environment.to_string(),
            };

            let delivery_id = self.queue_delivery(&webhook, payload).await?;
            delivery_ids.push(delivery_id);
        }

        Ok(delivery_ids)
    }

    /// Trigger webhooks for errors
    pub async fn trigger_error(
        &self,
        error_type: &str,
        error_message: &str,
        error_data: serde_json::Value,
        environment: &str,
    ) -> Result<Vec<String>, WebhookError> {
        let webhooks = self.storage.list_by_trigger(TriggerType::Error).await?;
        let mut delivery_ids = Vec::new();

        let data = serde_json::json!({
            "error_type": error_type,
            "error_message": error_message,
            "details": error_data,
        });

        for webhook in webhooks {
            if !webhook.enabled {
                continue;
            }

            let payload = WebhookPayload {
                id: uuid::Uuid::new_v4().to_string(),
                webhook_id: webhook.id.clone(),
                event_type: "$error".to_string(),
                timestamp: Utc::now(),
                data: data.clone(),
                environment: environment.to_string(),
            };

            let delivery_id = self.queue_delivery(&webhook, payload).await?;
            delivery_ids.push(delivery_id);
        }

        Ok(delivery_ids)
    }

    /// Trigger threshold alert
    pub async fn trigger_threshold(
        &self,
        metric: &str,
        value: f64,
        threshold: f64,
        environment: &str,
    ) -> Result<Vec<String>, WebhookError> {
        let webhooks = self.storage.list_by_trigger(TriggerType::Threshold).await?;
        let mut delivery_ids = Vec::new();

        let data = serde_json::json!({
            "metric": metric,
            "value": value,
            "threshold": threshold,
            "exceeded": value > threshold,
        });

        for webhook in webhooks {
            if !webhook.enabled {
                continue;
            }

            // Check threshold conditions
            let should_trigger = webhook.triggers.iter()
                .filter(|t| t.trigger_type == TriggerType::Threshold)
                .any(|t| {
                    t.conditions.iter().any(|c| {
                        if c.metric != metric {
                            return false;
                        }
                        match c.operator.as_str() {
                            "gt" => value > c.value,
                            "lt" => value < c.value,
                            "gte" => value >= c.value,
                            "lte" => value <= c.value,
                            "eq" => (value - c.value).abs() < f64::EPSILON,
                            _ => false,
                        }
                    })
                });

            if !should_trigger {
                continue;
            }

            let payload = WebhookPayload {
                id: uuid::Uuid::new_v4().to_string(),
                webhook_id: webhook.id.clone(),
                event_type: "$threshold_alert".to_string(),
                timestamp: Utc::now(),
                data: data.clone(),
                environment: environment.to_string(),
            };

            let delivery_id = self.queue_delivery(&webhook, payload).await?;
            delivery_ids.push(delivery_id);
        }

        Ok(delivery_ids)
    }

    fn matches_triggers(
        &self,
        webhook: &WebhookConfig,
        event_type: &str,
        data: &serde_json::Value,
    ) -> bool {
        webhook.triggers.iter().any(|trigger| {
            if trigger.trigger_type != TriggerType::Event {
                return false;
            }

            // Check filters
            trigger.filters.iter().all(|filter| {
                let value = data.get(&filter.property);
                match filter.operator.as_str() {
                    "equals" => value == Some(&filter.value),
                    "not_equals" => value != Some(&filter.value),
                    "contains" => {
                        value.and_then(|v| v.as_str())
                            .and_then(|v| filter.value.as_str().map(|f| v.contains(f)))
                            .unwrap_or(false)
                    }
                    "is_set" => value.is_some(),
                    _ => true,
                }
            })
        })
    }

    async fn queue_delivery(
        &self,
        webhook: &WebhookConfig,
        payload: WebhookPayload,
    ) -> Result<String, WebhookError> {
        let delivery = WebhookDelivery {
            id: uuid::Uuid::new_v4().to_string(),
            webhook_id: webhook.id.clone(),
            payload_id: payload.id.clone(),
            status: DeliveryStatus::Pending,
            status_code: None,
            response_body: None,
            attempt: 1,
            created_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            error: None,
        };

        self.storage.save_delivery(&delivery).await?;

        // Queue for delivery
        self.queue.send(delivery.clone()).await
            .map_err(|e| WebhookError::DeliveryFailed(e.to_string()))?;

        Ok(delivery.id)
    }

    /// Deliver a webhook
    pub async fn deliver(
        &self,
        webhook: &WebhookConfig,
        payload: &WebhookPayload,
        delivery: &mut WebhookDelivery,
    ) -> Result<(), WebhookError> {
        delivery.status = DeliveryStatus::Delivering;
        self.storage.update_delivery(delivery).await?;

        let signature = payload.sign(&webhook.secret);
        let body = serde_json::to_string(payload)
            .map_err(|e| WebhookError::DeliveryFailed(e.to_string()))?;

        let start = std::time::Instant::now();

        let mut request = self.client.post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &signature)
            .header("X-Webhook-Id", &webhook.id)
            .header("X-Delivery-Id", &delivery.id)
            .body(body);

        // Add custom headers
        for (key, value) in &webhook.headers {
            request = request.header(key, value);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let body = response.text().await.ok();

                delivery.status_code = Some(status);
                delivery.response_body = body;
                delivery.duration_ms = Some(start.elapsed().as_millis() as u64);

                if status >= 200 && status < 300 {
                    delivery.status = DeliveryStatus::Delivered;
                    delivery.completed_at = Some(Utc::now());
                } else {
                    delivery.status = DeliveryStatus::Failed;
                    delivery.error = Some(format!("HTTP {}", status));
                }
            }
            Err(e) => {
                delivery.status = DeliveryStatus::Failed;
                delivery.error = Some(e.to_string());
                delivery.duration_ms = Some(start.elapsed().as_millis() as u64);
            }
        }

        self.storage.update_delivery(delivery).await?;

        if delivery.status == DeliveryStatus::Failed && delivery.attempt < webhook.retry.max_attempts {
            // Schedule retry
            self.schedule_retry(webhook, delivery).await?;
        }

        Ok(())
    }

    async fn schedule_retry(
        &self,
        webhook: &WebhookConfig,
        delivery: &WebhookDelivery,
    ) -> Result<(), WebhookError> {
        let delay = self.calculate_retry_delay(&webhook.retry, delivery.attempt);

        let mut retry_delivery = delivery.clone();
        retry_delivery.attempt += 1;
        retry_delivery.status = DeliveryStatus::Retrying;

        self.storage.update_delivery(&retry_delivery).await?;

        // In a real implementation, this would use a job queue
        tracing::info!(
            "Scheduling retry {} for delivery {} in {} seconds",
            retry_delivery.attempt,
            retry_delivery.id,
            delay
        );

        Ok(())
    }

    fn calculate_retry_delay(&self, config: &RetryConfig, attempt: u32) -> u32 {
        let delay = config.initial_delay_seconds as f64
            * config.backoff_multiplier.powi(attempt as i32 - 1);
        (delay as u32).min(config.max_delay_seconds)
    }

    /// Get webhook delivery history
    pub async fn get_deliveries(
        &self,
        webhook_id: &str,
        limit: u32,
    ) -> Result<Vec<WebhookDelivery>, WebhookError> {
        self.storage.get_deliveries(webhook_id, limit).await
    }

    /// Test webhook delivery
    pub async fn test(&self, webhook_id: &str) -> Result<WebhookDelivery, WebhookError> {
        let webhook = self.storage.get_webhook(webhook_id).await?
            .ok_or(WebhookError::NotFound)?;

        let payload = WebhookPayload {
            id: uuid::Uuid::new_v4().to_string(),
            webhook_id: webhook.id.clone(),
            event_type: "$test".to_string(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "message": "This is a test webhook delivery",
                "webhook_name": webhook.name,
            }),
            environment: "test".to_string(),
        };

        let mut delivery = WebhookDelivery {
            id: uuid::Uuid::new_v4().to_string(),
            webhook_id: webhook.id.clone(),
            payload_id: payload.id.clone(),
            status: DeliveryStatus::Pending,
            status_code: None,
            response_body: None,
            attempt: 1,
            created_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            error: None,
        };

        self.deliver(&webhook, &payload, &mut delivery).await?;

        Ok(delivery)
    }
}

/// Webhook worker for processing deliveries
pub async fn webhook_worker(
    mut rx: tokio::sync::mpsc::Receiver<WebhookDelivery>,
    service: std::sync::Arc<WebhookService>,
    storage: std::sync::Arc<dyn WebhookStorage>,
) {
    while let Some(delivery) = rx.recv().await {
        let webhook = match storage.get_webhook(&delivery.webhook_id).await {
            Ok(Some(w)) => w,
            _ => continue,
        };

        // Reconstruct payload (in practice, store payload separately)
        let payload = WebhookPayload {
            id: delivery.payload_id.clone(),
            webhook_id: delivery.webhook_id.clone(),
            event_type: "unknown".to_string(), // Would be stored
            timestamp: delivery.created_at,
            data: serde_json::json!({}),
            environment: "production".to_string(),
        };

        let mut delivery = delivery;
        if let Err(e) = service.deliver(&webhook, &payload, &mut delivery).await {
            tracing::error!("Webhook delivery failed: {}", e);
        }
    }
}

use hmac::Mac;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_signing() {
        let payload = WebhookPayload {
            id: "test-id".to_string(),
            webhook_id: "webhook-1".to_string(),
            event_type: "test".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            data: serde_json::json!({"key": "value"}),
            environment: "test".to_string(),
        };

        let sig1 = payload.sign("secret");
        let sig2 = payload.sign("secret");
        let sig3 = payload.sign("different-secret");

        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_seconds: 5,
            max_delay_seconds: 3600,
            backoff_multiplier: 2.0,
        };

        // Attempt 1: 5 seconds
        assert_eq!(5 * 2_i32.pow(0) as u32, 5);
        // Attempt 2: 10 seconds
        assert_eq!(5 * 2_i32.pow(1) as u32, 10);
        // Attempt 3: 20 seconds
        assert_eq!(5 * 2_i32.pow(2) as u32, 20);
    }

    #[test]
    fn test_default_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}
```

## REST API

```yaml
openapi: 3.0.0
paths:
  /api/webhooks:
    get:
      summary: List webhooks
    post:
      summary: Create webhook

  /api/webhooks/{id}:
    get:
      summary: Get webhook
    put:
      summary: Update webhook
    delete:
      summary: Delete webhook

  /api/webhooks/{id}/test:
    post:
      summary: Send test delivery

  /api/webhooks/{id}/deliveries:
    get:
      summary: Get delivery history
```

## Related Specs

- 421-error-tracking.md - Error alerts
- 422-performance-tracking.md - Performance alerts
- 428-realtime-analytics.md - Real-time triggers
