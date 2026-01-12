# Spec 413: Analytics Privacy Controls

## Phase
19 - Analytics/Telemetry

## Spec ID
413

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 412: User Consent (consent system)

## Estimated Context
~9%

---

## Objective

Implement comprehensive privacy controls for analytics data, including PII detection and removal, data anonymization, and privacy-preserving data handling techniques.

---

## Acceptance Criteria

- [ ] Implement PII detection and redaction
- [ ] Create data anonymization utilities
- [ ] Support k-anonymity for exported data
- [ ] Implement differential privacy for aggregates
- [ ] Create privacy audit logging
- [ ] Support data minimization rules
- [ ] Implement secure data handling
- [ ] Create privacy compliance reporting

---

## Implementation Details

### Privacy Controls

```rust
// src/analytics/privacy.rs

use crate::analytics::types::{AnalyticsEvent, EventData, EventMetadata};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Types of personally identifiable information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiType {
    Email,
    PhoneNumber,
    IpAddress,
    CreditCard,
    SocialSecurity,
    ApiKey,
    Password,
    Name,
    Address,
    Custom,
}

impl PiiType {
    /// Get regex pattern for this PII type
    pub fn pattern(&self) -> Option<&str> {
        match self {
            Self::Email => Some(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"),
            Self::PhoneNumber => Some(r"\+?[\d\s\-\(\)]{10,}"),
            Self::IpAddress => Some(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
            Self::CreditCard => Some(r"\b(?:\d{4}[\s\-]?){3}\d{4}\b"),
            Self::SocialSecurity => Some(r"\b\d{3}[\s\-]?\d{2}[\s\-]?\d{4}\b"),
            Self::ApiKey => Some(r"(?:sk|pk|api|key|token)[-_]?[a-zA-Z0-9]{20,}"),
            Self::Password => None, // Detected by field name
            Self::Name => None,     // Detected by field name
            Self::Address => None,  // Complex detection
            Self::Custom => None,
        }
    }

    /// Get replacement text for this PII type
    pub fn replacement(&self) -> &str {
        match self {
            Self::Email => "[EMAIL_REDACTED]",
            Self::PhoneNumber => "[PHONE_REDACTED]",
            Self::IpAddress => "[IP_REDACTED]",
            Self::CreditCard => "[CC_REDACTED]",
            Self::SocialSecurity => "[SSN_REDACTED]",
            Self::ApiKey => "[KEY_REDACTED]",
            Self::Password => "[PASSWORD_REDACTED]",
            Self::Name => "[NAME_REDACTED]",
            Self::Address => "[ADDRESS_REDACTED]",
            Self::Custom => "[REDACTED]",
        }
    }
}

/// PII detection result
#[derive(Debug, Clone)]
pub struct PiiDetection {
    pub pii_type: PiiType,
    pub field: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
}

/// PII detector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiDetectorConfig {
    /// PII types to detect
    pub detect_types: HashSet<PiiType>,
    /// Field names that indicate sensitive data
    pub sensitive_fields: HashSet<String>,
    /// Custom patterns to detect
    pub custom_patterns: Vec<(String, String)>, // (pattern, replacement)
    /// Minimum confidence threshold
    pub confidence_threshold: f32,
}

impl Default for PiiDetectorConfig {
    fn default() -> Self {
        let mut detect_types = HashSet::new();
        detect_types.insert(PiiType::Email);
        detect_types.insert(PiiType::PhoneNumber);
        detect_types.insert(PiiType::IpAddress);
        detect_types.insert(PiiType::CreditCard);
        detect_types.insert(PiiType::ApiKey);

        let sensitive_fields: HashSet<String> = [
            "password",
            "secret",
            "token",
            "api_key",
            "apikey",
            "auth",
            "credential",
            "ssn",
            "credit_card",
            "phone",
            "email",
            "name",
            "address",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            detect_types,
            sensitive_fields,
            custom_patterns: Vec::new(),
            confidence_threshold: 0.8,
        }
    }
}

/// PII detector
pub struct PiiDetector {
    config: PiiDetectorConfig,
    patterns: HashMap<PiiType, Regex>,
    custom_patterns: Vec<(Regex, String)>,
}

impl PiiDetector {
    pub fn new(config: PiiDetectorConfig) -> Self {
        let mut patterns = HashMap::new();

        for pii_type in &config.detect_types {
            if let Some(pattern) = pii_type.pattern() {
                if let Ok(regex) = Regex::new(pattern) {
                    patterns.insert(*pii_type, regex);
                }
            }
        }

        let custom_patterns: Vec<_> = config
            .custom_patterns
            .iter()
            .filter_map(|(pattern, replacement)| {
                Regex::new(pattern)
                    .ok()
                    .map(|r| (r, replacement.clone()))
            })
            .collect();

        Self {
            config,
            patterns,
            custom_patterns,
        }
    }

    /// Detect PII in a string
    pub fn detect(&self, text: &str) -> Vec<PiiDetection> {
        let mut detections = Vec::new();

        for (pii_type, pattern) in &self.patterns {
            for mat in pattern.find_iter(text) {
                detections.push(PiiDetection {
                    pii_type: *pii_type,
                    field: String::new(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 0.9,
                });
            }
        }

        detections
    }

    /// Check if a field name indicates sensitive data
    pub fn is_sensitive_field(&self, field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        self.config
            .sensitive_fields
            .iter()
            .any(|s| lower.contains(s))
    }

    /// Redact PII from a string
    pub fn redact(&self, text: &str) -> String {
        let mut result = text.to_string();

        for (pii_type, pattern) in &self.patterns {
            result = pattern
                .replace_all(&result, pii_type.replacement())
                .to_string();
        }

        for (pattern, replacement) in &self.custom_patterns {
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
        }

        result
    }

    /// Redact PII from JSON value
    pub fn redact_json(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(s) => {
                *s = self.redact(s);
            }
            serde_json::Value::Object(map) => {
                let sensitive_keys: Vec<String> = map
                    .keys()
                    .filter(|k| self.is_sensitive_field(k))
                    .cloned()
                    .collect();

                for key in sensitive_keys {
                    map.insert(key, serde_json::Value::String("[REDACTED]".to_string()));
                }

                for (_, v) in map.iter_mut() {
                    self.redact_json(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.redact_json(item);
                }
            }
            _ => {}
        }
    }
}

impl Default for PiiDetector {
    fn default() -> Self {
        Self::new(PiiDetectorConfig::default())
    }
}

/// Data anonymizer for analytics events
pub struct DataAnonymizer {
    pii_detector: PiiDetector,
    salt: String,
}

impl DataAnonymizer {
    pub fn new(pii_detector: PiiDetector, salt: &str) -> Self {
        Self {
            pii_detector,
            salt: salt.to_string(),
        }
    }

    /// Anonymize an analytics event
    pub fn anonymize_event(&self, mut event: AnalyticsEvent) -> AnalyticsEvent {
        // Hash session ID if present
        if let Some(session_id) = event.session_id {
            event.session_id = Some(self.hash_uuid(&session_id));
        }

        // Anonymize event data
        event.data = self.anonymize_event_data(event.data);

        // Anonymize metadata
        event.metadata = self.anonymize_metadata(event.metadata);

        event
    }

    fn anonymize_event_data(&self, data: EventData) -> EventData {
        match data {
            EventData::KeyValue(mut map) => {
                for (key, value) in map.iter_mut() {
                    if self.pii_detector.is_sensitive_field(key) {
                        *value = serde_json::Value::String("[REDACTED]".to_string());
                    } else if let serde_json::Value::String(s) = value {
                        *value = serde_json::Value::String(self.pii_detector.redact(s));
                    }
                }
                EventData::KeyValue(map)
            }
            EventData::Error(mut error_data) => {
                error_data.message = self.pii_detector.redact(&error_data.message);
                if let Some(ref mut trace) = error_data.stack_trace {
                    *trace = self.pii_detector.redact(trace);
                }
                EventData::Error(error_data)
            }
            EventData::Custom(mut value) => {
                self.pii_detector.redact_json(&mut value);
                EventData::Custom(value)
            }
            other => other,
        }
    }

    fn anonymize_metadata(&self, mut metadata: EventMetadata) -> EventMetadata {
        // Remove or hash potentially identifying metadata
        for (key, value) in metadata.custom.iter_mut() {
            if self.pii_detector.is_sensitive_field(key) {
                *value = serde_json::Value::String("[REDACTED]".to_string());
            }
        }
        metadata
    }

    /// Hash a UUID deterministically
    fn hash_uuid(&self, uuid: &uuid::Uuid) -> uuid::Uuid {
        let mut hasher = Sha256::new();
        hasher.update(uuid.as_bytes());
        hasher.update(self.salt.as_bytes());
        let result = hasher.finalize();

        // Take first 16 bytes for UUID
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);
        uuid::Uuid::from_bytes(bytes)
    }

    /// Hash a string deterministically
    pub fn hash_string(&self, value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        hasher.update(self.salt.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// K-anonymity enforcer for exported data
pub struct KAnonymizer {
    k: usize,
    quasi_identifiers: Vec<String>,
}

impl KAnonymizer {
    pub fn new(k: usize, quasi_identifiers: Vec<String>) -> Self {
        Self {
            k,
            quasi_identifiers,
        }
    }

    /// Check if a dataset satisfies k-anonymity
    pub fn check_k_anonymity(&self, events: &[AnalyticsEvent]) -> bool {
        let groups = self.group_by_quasi_identifiers(events);

        groups.values().all(|group| group.len() >= self.k)
    }

    /// Generalize data to achieve k-anonymity
    pub fn enforce_k_anonymity(
        &self,
        mut events: Vec<AnalyticsEvent>,
    ) -> Vec<AnalyticsEvent> {
        let groups = self.group_by_quasi_identifiers(&events);

        // Find groups that don't meet k requirement
        let small_groups: HashSet<String> = groups
            .iter()
            .filter(|(_, v)| v.len() < self.k)
            .map(|(k, _)| k.clone())
            .collect();

        if small_groups.is_empty() {
            return events;
        }

        // Generalize or suppress events in small groups
        events.retain(|event| {
            let key = self.compute_quasi_identifier_key(event);
            !small_groups.contains(&key)
        });

        events
    }

    fn group_by_quasi_identifiers(
        &self,
        events: &[AnalyticsEvent],
    ) -> HashMap<String, Vec<&AnalyticsEvent>> {
        let mut groups: HashMap<String, Vec<&AnalyticsEvent>> = HashMap::new();

        for event in events {
            let key = self.compute_quasi_identifier_key(event);
            groups.entry(key).or_default().push(event);
        }

        groups
    }

    fn compute_quasi_identifier_key(&self, event: &AnalyticsEvent) -> String {
        let mut parts = Vec::new();

        for qi in &self.quasi_identifiers {
            let value = match qi.as_str() {
                "category" => format!("{:?}", event.category),
                "event_type" => format!("{:?}", event.event_type),
                "hour" => event.timestamp.format("%Y-%m-%d-%H").to_string(),
                "day" => event.timestamp.format("%Y-%m-%d").to_string(),
                _ => String::new(),
            };
            parts.push(value);
        }

        parts.join("|")
    }
}

/// Differential privacy noise adder
pub struct DifferentialPrivacy {
    epsilon: f64,
    sensitivity: f64,
}

impl DifferentialPrivacy {
    pub fn new(epsilon: f64, sensitivity: f64) -> Self {
        Self {
            epsilon,
            sensitivity,
        }
    }

    /// Add Laplace noise to a value
    pub fn add_laplace_noise(&self, value: f64) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let scale = self.sensitivity / self.epsilon;
        let u: f64 = rng.gen::<f64>() - 0.5;

        let noise = -scale * u.signum() * (1.0 - 2.0 * u.abs()).ln();
        value + noise
    }

    /// Add noise to a count
    pub fn add_count_noise(&self, count: u64) -> u64 {
        let noisy = self.add_laplace_noise(count as f64);
        noisy.max(0.0).round() as u64
    }

    /// Add noise to an aggregated metric
    pub fn add_aggregate_noise(&self, aggregate: f64) -> f64 {
        self.add_laplace_noise(aggregate)
    }
}

/// Privacy audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyAuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: PrivacyAction,
    pub details: String,
    pub affected_records: Option<u64>,
}

/// Privacy-related actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyAction {
    PiiDetected,
    PiiRedacted,
    DataAnonymized,
    DataDeleted,
    ConsentUpdated,
    DataExported,
    AccessRequested,
}

/// Privacy audit logger
pub struct PrivacyAuditLog {
    entries: Arc<tokio::sync::RwLock<Vec<PrivacyAuditEntry>>>,
}

impl PrivacyAuditLog {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn log(&self, action: PrivacyAction, details: &str, affected_records: Option<u64>) {
        let entry = PrivacyAuditEntry {
            timestamp: chrono::Utc::now(),
            action,
            details: details.to_string(),
            affected_records,
        };

        let mut entries = self.entries.write().await;
        entries.push(entry);
    }

    pub async fn get_entries(&self) -> Vec<PrivacyAuditEntry> {
        self.entries.read().await.clone()
    }

    pub async fn export(&self) -> String {
        let entries = self.entries.read().await;
        serde_json::to_string_pretty(&*entries).unwrap_or_default()
    }
}

impl Default for PrivacyAuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Privacy controller combining all privacy features
pub struct PrivacyController {
    pii_detector: PiiDetector,
    anonymizer: DataAnonymizer,
    k_anonymizer: Option<KAnonymizer>,
    differential_privacy: Option<DifferentialPrivacy>,
    audit_log: PrivacyAuditLog,
}

impl PrivacyController {
    pub fn new(salt: &str) -> Self {
        let pii_detector = PiiDetector::default();
        let anonymizer = DataAnonymizer::new(PiiDetector::default(), salt);

        Self {
            pii_detector,
            anonymizer,
            k_anonymizer: None,
            differential_privacy: None,
            audit_log: PrivacyAuditLog::new(),
        }
    }

    pub fn with_k_anonymity(mut self, k: usize, quasi_identifiers: Vec<String>) -> Self {
        self.k_anonymizer = Some(KAnonymizer::new(k, quasi_identifiers));
        self
    }

    pub fn with_differential_privacy(mut self, epsilon: f64, sensitivity: f64) -> Self {
        self.differential_privacy = Some(DifferentialPrivacy::new(epsilon, sensitivity));
        self
    }

    /// Process an event through all privacy controls
    pub async fn process_event(&self, event: AnalyticsEvent) -> AnalyticsEvent {
        let anonymized = self.anonymizer.anonymize_event(event);

        self.audit_log
            .log(PrivacyAction::DataAnonymized, "Event anonymized", Some(1))
            .await;

        anonymized
    }

    /// Process a batch of events
    pub async fn process_batch(
        &self,
        events: Vec<AnalyticsEvent>,
    ) -> Vec<AnalyticsEvent> {
        let count = events.len() as u64;

        let anonymized: Vec<_> = events
            .into_iter()
            .map(|e| self.anonymizer.anonymize_event(e))
            .collect();

        let result = if let Some(ref k_anon) = self.k_anonymizer {
            k_anon.enforce_k_anonymity(anonymized)
        } else {
            anonymized
        };

        self.audit_log
            .log(
                PrivacyAction::DataAnonymized,
                "Batch processed",
                Some(count),
            )
            .await;

        result
    }

    /// Apply differential privacy to a count
    pub fn private_count(&self, count: u64) -> u64 {
        if let Some(ref dp) = self.differential_privacy {
            dp.add_count_noise(count)
        } else {
            count
        }
    }

    /// Get audit log
    pub fn audit_log(&self) -> &PrivacyAuditLog {
        &self.audit_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_detection() {
        let detector = PiiDetector::default();

        let text = "Contact me at john@example.com or 555-123-4567";
        let detections = detector.detect(text);

        assert!(detections.iter().any(|d| d.pii_type == PiiType::Email));
        assert!(detections.iter().any(|d| d.pii_type == PiiType::PhoneNumber));
    }

    #[test]
    fn test_pii_redaction() {
        let detector = PiiDetector::default();

        let text = "API key: sk-abc123def456ghi789jkl012mno345pqr678";
        let redacted = detector.redact(text);

        assert!(redacted.contains("[KEY_REDACTED]"));
        assert!(!redacted.contains("sk-abc123"));
    }

    #[test]
    fn test_sensitive_field_detection() {
        let detector = PiiDetector::default();

        assert!(detector.is_sensitive_field("user_password"));
        assert!(detector.is_sensitive_field("api_key"));
        assert!(!detector.is_sensitive_field("event_type"));
    }

    #[test]
    fn test_differential_privacy() {
        let dp = DifferentialPrivacy::new(1.0, 1.0);

        let original = 100.0;
        let noisy = dp.add_laplace_noise(original);

        // Noisy value should be different
        assert_ne!(original, noisy);
    }

    #[test]
    fn test_k_anonymity_check() {
        use crate::analytics::types::EventBuilder;

        let k_anon = KAnonymizer::new(2, vec!["category".to_string()]);

        let events: Vec<_> = (0..10)
            .map(|_| {
                EventBuilder::new(crate::analytics::types::EventType::FeatureUsed)
                    .build()
            })
            .collect();

        // All events have same category, so should pass k=2
        assert!(k_anon.check_k_anonymity(&events));
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - PII pattern detection accuracy
   - Redaction completeness
   - Anonymization consistency
   - K-anonymity verification

2. **Integration Tests**
   - Full privacy pipeline
   - Audit log completeness
   - Performance impact

3. **Compliance Tests**
   - GDPR requirements coverage
   - Data minimization effectiveness

---

## Related Specs

- Spec 406: Analytics Types
- Spec 412: User Consent
- Spec 424: Data Retention
