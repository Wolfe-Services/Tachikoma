# 425 - Privacy Compliance

## Overview

GDPR, CCPA, and privacy compliance features including consent management, data subject rights, and PII handling.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

```rust
// crates/analytics/src/privacy.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use async_trait::async_trait;

/// Privacy regulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyRegulation {
    Gdpr,
    Ccpa,
    Lgpd,
    Pipeda,
    Custom(u32),
}

/// Consent category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentCategory {
    /// Essential/necessary (always allowed)
    Necessary,
    /// Analytics/statistics
    Analytics,
    /// Functional/preferences
    Functional,
    /// Marketing/advertising
    Marketing,
    /// Third-party
    ThirdParty,
}

/// User consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// User/visitor ID
    pub subject_id: String,
    /// Consent preferences by category
    pub preferences: HashMap<ConsentCategory, ConsentPreference>,
    /// Applicable regulations
    pub regulations: Vec<PrivacyRegulation>,
    /// IP address (hashed)
    pub ip_hash: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Consent collection method
    pub collection_method: ConsentCollectionMethod,
    /// First consent timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Consent version (banner version)
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPreference {
    Granted,
    Denied,
    NotSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentCollectionMethod {
    Banner,
    Preference,
    Api,
    Import,
}

impl ConsentRecord {
    pub fn new(subject_id: &str) -> Self {
        Self {
            subject_id: subject_id.to_string(),
            preferences: HashMap::new(),
            regulations: Vec::new(),
            ip_hash: None,
            user_agent: None,
            collection_method: ConsentCollectionMethod::Banner,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version: "1.0".to_string(),
        }
    }

    /// Check if a category is consented
    pub fn has_consent(&self, category: ConsentCategory) -> bool {
        match category {
            ConsentCategory::Necessary => true, // Always allowed
            _ => self.preferences.get(&category)
                .map(|p| *p == ConsentPreference::Granted)
                .unwrap_or(false),
        }
    }

    /// Set consent for a category
    pub fn set_consent(&mut self, category: ConsentCategory, preference: ConsentPreference) {
        self.preferences.insert(category, preference);
        self.updated_at = Utc::now();
    }

    /// Grant all consents
    pub fn grant_all(&mut self) {
        for category in &[
            ConsentCategory::Analytics,
            ConsentCategory::Functional,
            ConsentCategory::Marketing,
            ConsentCategory::ThirdParty,
        ] {
            self.preferences.insert(*category, ConsentPreference::Granted);
        }
        self.updated_at = Utc::now();
    }

    /// Deny all optional consents
    pub fn deny_all(&mut self) {
        for category in &[
            ConsentCategory::Analytics,
            ConsentCategory::Functional,
            ConsentCategory::Marketing,
            ConsentCategory::ThirdParty,
        ] {
            self.preferences.insert(*category, ConsentPreference::Denied);
        }
        self.updated_at = Utc::now();
    }
}

/// Consent storage trait
#[async_trait]
pub trait ConsentStorage: Send + Sync {
    async fn get(&self, subject_id: &str) -> Result<Option<ConsentRecord>, PrivacyError>;
    async fn save(&self, record: &ConsentRecord) -> Result<(), PrivacyError>;
    async fn delete(&self, subject_id: &str) -> Result<(), PrivacyError>;
    async fn list_by_regulation(&self, regulation: PrivacyRegulation) -> Result<Vec<ConsentRecord>, PrivacyError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PrivacyError {
    #[error("Subject not found")]
    NotFound,
    #[error("Invalid request: {0}")]
    Invalid(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Consent required")]
    ConsentRequired,
}

/// Consent manager
pub struct ConsentManager {
    storage: std::sync::Arc<dyn ConsentStorage>,
    config: ConsentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentConfig {
    /// Default consent (for regions without strict requirements)
    pub default_consent: ConsentPreference,
    /// Regions requiring explicit consent
    pub strict_regions: Vec<String>,
    /// Consent expiry (days)
    pub expiry_days: u32,
    /// Cookie name
    pub cookie_name: String,
    /// Banner version
    pub banner_version: String,
}

impl Default for ConsentConfig {
    fn default() -> Self {
        Self {
            default_consent: ConsentPreference::NotSet,
            strict_regions: vec!["EU".to_string(), "CA".to_string()],
            expiry_days: 365,
            cookie_name: "consent_preferences".to_string(),
            banner_version: "1.0".to_string(),
        }
    }
}

impl ConsentManager {
    pub fn new(storage: std::sync::Arc<dyn ConsentStorage>, config: ConsentConfig) -> Self {
        Self { storage, config }
    }

    /// Get or create consent record
    pub async fn get_or_create(&self, subject_id: &str) -> Result<ConsentRecord, PrivacyError> {
        match self.storage.get(subject_id).await? {
            Some(record) => Ok(record),
            None => {
                let record = ConsentRecord::new(subject_id);
                self.storage.save(&record).await?;
                Ok(record)
            }
        }
    }

    /// Update consent preferences
    pub async fn update_consent(
        &self,
        subject_id: &str,
        preferences: HashMap<ConsentCategory, ConsentPreference>,
    ) -> Result<ConsentRecord, PrivacyError> {
        let mut record = self.get_or_create(subject_id).await?;

        for (category, preference) in preferences {
            record.set_consent(category, preference);
        }

        record.version = self.config.banner_version.clone();
        self.storage.save(&record).await?;

        Ok(record)
    }

    /// Check if tracking is allowed
    pub async fn can_track(
        &self,
        subject_id: &str,
        category: ConsentCategory,
    ) -> Result<bool, PrivacyError> {
        let record = self.storage.get(subject_id).await?;

        match record {
            Some(r) => Ok(r.has_consent(category)),
            None => {
                // No record means no explicit consent
                match self.config.default_consent {
                    ConsentPreference::Granted => Ok(true),
                    _ => Ok(category == ConsentCategory::Necessary),
                }
            }
        }
    }

    /// Withdraw all consent
    pub async fn withdraw_consent(&self, subject_id: &str) -> Result<(), PrivacyError> {
        let mut record = self.get_or_create(subject_id).await?;
        record.deny_all();
        self.storage.save(&record).await
    }
}

/// PII (Personally Identifiable Information) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiConfig {
    /// Properties to hash
    pub hash_properties: Vec<String>,
    /// Properties to mask
    pub mask_properties: Vec<String>,
    /// Properties to remove
    pub remove_properties: Vec<String>,
    /// Properties to encrypt
    pub encrypt_properties: Vec<String>,
    /// Custom patterns to detect
    pub patterns: Vec<PiiPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiPattern {
    pub name: String,
    pub pattern: String,
    pub action: PiiAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PiiAction {
    Hash,
    Mask,
    Remove,
    Encrypt,
}

impl Default for PiiConfig {
    fn default() -> Self {
        Self {
            hash_properties: vec![
                "email".to_string(),
                "$email".to_string(),
                "ip".to_string(),
                "$ip".to_string(),
            ],
            mask_properties: vec![
                "phone".to_string(),
                "credit_card".to_string(),
            ],
            remove_properties: vec![
                "password".to_string(),
                "ssn".to_string(),
                "social_security".to_string(),
            ],
            encrypt_properties: vec![],
            patterns: vec![
                PiiPattern {
                    name: "email".to_string(),
                    pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                    action: PiiAction::Hash,
                },
                PiiPattern {
                    name: "ip_v4".to_string(),
                    pattern: r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b".to_string(),
                    action: PiiAction::Mask,
                },
            ],
        }
    }
}

/// PII processor
pub struct PiiProcessor {
    config: PiiConfig,
    hasher: ring::hmac::Key,
    patterns: Vec<(String, regex::Regex, PiiAction)>,
}

impl PiiProcessor {
    pub fn new(config: PiiConfig, secret: &[u8]) -> Result<Self, PrivacyError> {
        let hasher = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, secret);

        let patterns: Vec<_> = config.patterns.iter()
            .filter_map(|p| {
                regex::Regex::new(&p.pattern)
                    .ok()
                    .map(|re| (p.name.clone(), re, p.action))
            })
            .collect();

        Ok(Self { config, hasher, patterns })
    }

    /// Process event properties
    pub fn process(&self, properties: &mut HashMap<String, serde_json::Value>) {
        let keys: Vec<String> = properties.keys().cloned().collect();

        for key in keys {
            // Check explicit property rules
            if self.config.remove_properties.contains(&key) {
                properties.remove(&key);
                continue;
            }

            if self.config.hash_properties.contains(&key) {
                if let Some(value) = properties.get(&key) {
                    let hashed = self.hash_value(value);
                    properties.insert(key, serde_json::json!(hashed));
                }
                continue;
            }

            if self.config.mask_properties.contains(&key) {
                if let Some(value) = properties.get(&key) {
                    let masked = self.mask_value(value);
                    properties.insert(key, serde_json::json!(masked));
                }
                continue;
            }

            // Check patterns in string values
            if let Some(serde_json::Value::String(s)) = properties.get(&key) {
                let processed = self.process_string(s);
                if processed != *s {
                    properties.insert(key, serde_json::json!(processed));
                }
            }
        }
    }

    fn hash_value(&self, value: &serde_json::Value) -> String {
        let input = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };

        let tag = ring::hmac::sign(&self.hasher, input.as_bytes());
        hex::encode(tag.as_ref())
    }

    fn mask_value(&self, value: &serde_json::Value) -> String {
        let input = match value {
            serde_json::Value::String(s) => s.clone(),
            _ => value.to_string(),
        };

        let len = input.len();
        if len <= 4 {
            "*".repeat(len)
        } else {
            format!("{}***{}", &input[..2], &input[len-2..])
        }
    }

    fn process_string(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (_name, pattern, action) in &self.patterns {
            result = pattern.replace_all(&result, |caps: &regex::Captures| {
                let matched = &caps[0];
                match action {
                    PiiAction::Hash => {
                        let tag = ring::hmac::sign(&self.hasher, matched.as_bytes());
                        hex::encode(&tag.as_ref()[..8])
                    }
                    PiiAction::Mask => self.mask_string(matched),
                    PiiAction::Remove => "[REDACTED]".to_string(),
                    PiiAction::Encrypt => matched.to_string(), // TODO: implement encryption
                }
            }).to_string();
        }

        result
    }

    fn mask_string(&self, input: &str) -> String {
        let len = input.len();
        if len <= 4 {
            "*".repeat(len)
        } else {
            format!("{}***{}", &input[..2], &input[len-2..])
        }
    }
}

/// Data subject request types (GDPR Article 15-22)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSubjectRequestType {
    /// Right of access (Art. 15)
    Access,
    /// Right to rectification (Art. 16)
    Rectification,
    /// Right to erasure / right to be forgotten (Art. 17)
    Erasure,
    /// Right to restriction of processing (Art. 18)
    Restriction,
    /// Right to data portability (Art. 20)
    Portability,
    /// Right to object (Art. 21)
    Objection,
}

/// Data subject request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    /// Request ID
    pub id: String,
    /// Subject identifier
    pub subject_id: String,
    /// Request type
    pub request_type: DataSubjectRequestType,
    /// Additional identifiers (email, phone, etc.)
    pub identifiers: HashMap<String, String>,
    /// Request details
    pub details: Option<String>,
    /// Status
    pub status: RequestStatus,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Deadline (30 days from creation for GDPR)
    pub deadline: DateTime<Utc>,
    /// Result data (for access/portability)
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    InProgress,
    Completed,
    Rejected,
    Expired,
}

impl DataSubjectRequest {
    pub fn new(subject_id: &str, request_type: DataSubjectRequestType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            subject_id: subject_id.to_string(),
            request_type,
            identifiers: HashMap::new(),
            details: None,
            status: RequestStatus::Pending,
            created_at: now,
            completed_at: None,
            deadline: now + chrono::Duration::days(30),
            result: None,
        }
    }
}

/// Data subject request handler
#[async_trait]
pub trait DataSubjectRequestHandler: Send + Sync {
    async fn handle_access(&self, request: &DataSubjectRequest) -> Result<serde_json::Value, PrivacyError>;
    async fn handle_erasure(&self, request: &DataSubjectRequest) -> Result<(), PrivacyError>;
    async fn handle_portability(&self, request: &DataSubjectRequest) -> Result<Vec<u8>, PrivacyError>;
    async fn handle_restriction(&self, request: &DataSubjectRequest) -> Result<(), PrivacyError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_record() {
        let mut record = ConsentRecord::new("user-123");

        // Initially no consent
        assert!(!record.has_consent(ConsentCategory::Analytics));

        // Necessary is always true
        assert!(record.has_consent(ConsentCategory::Necessary));

        // Grant analytics
        record.set_consent(ConsentCategory::Analytics, ConsentPreference::Granted);
        assert!(record.has_consent(ConsentCategory::Analytics));

        // Deny marketing
        record.set_consent(ConsentCategory::Marketing, ConsentPreference::Denied);
        assert!(!record.has_consent(ConsentCategory::Marketing));
    }

    #[test]
    fn test_grant_deny_all() {
        let mut record = ConsentRecord::new("user-123");

        record.grant_all();
        assert!(record.has_consent(ConsentCategory::Analytics));
        assert!(record.has_consent(ConsentCategory::Marketing));

        record.deny_all();
        assert!(!record.has_consent(ConsentCategory::Analytics));
        assert!(!record.has_consent(ConsentCategory::Marketing));
    }

    #[test]
    fn test_pii_processor() {
        let config = PiiConfig::default();
        let processor = PiiProcessor::new(config, b"secret-key").unwrap();

        let mut props = HashMap::new();
        props.insert("email".to_string(), serde_json::json!("test@example.com"));
        props.insert("name".to_string(), serde_json::json!("John Doe"));
        props.insert("password".to_string(), serde_json::json!("secret123"));

        processor.process(&mut props);

        // Email should be hashed
        assert!(props.get("email").unwrap().as_str().unwrap() != "test@example.com");

        // Name should be unchanged
        assert_eq!(props.get("name").unwrap().as_str().unwrap(), "John Doe");

        // Password should be removed
        assert!(!props.contains_key("password"));
    }

    #[test]
    fn test_pii_pattern_detection() {
        let config = PiiConfig::default();
        let processor = PiiProcessor::new(config, b"secret-key").unwrap();

        let mut props = HashMap::new();
        props.insert("message".to_string(),
            serde_json::json!("Contact me at user@email.com or 192.168.1.1"));

        processor.process(&mut props);

        let message = props.get("message").unwrap().as_str().unwrap();
        assert!(!message.contains("user@email.com"));
        assert!(!message.contains("192.168.1.1"));
    }

    #[test]
    fn test_data_subject_request() {
        let request = DataSubjectRequest::new("user-123", DataSubjectRequestType::Erasure);

        assert_eq!(request.status, RequestStatus::Pending);
        assert!(request.deadline > request.created_at);
        assert!((request.deadline - request.created_at).num_days() >= 29);
    }
}
```

## Consent Banner Configuration

```typescript
// TypeScript consent banner
interface ConsentBannerConfig {
  position: 'bottom' | 'top' | 'center';
  theme: 'light' | 'dark' | 'auto';
  categories: CategoryConfig[];
  texts: BannerTexts;
  legal: LegalConfig;
}

interface CategoryConfig {
  id: ConsentCategory;
  label: string;
  description: string;
  required: boolean;
  default: boolean;
}

interface BannerTexts {
  title: string;
  description: string;
  acceptAll: string;
  rejectAll: string;
  customize: string;
  save: string;
  privacyPolicy: string;
}

// Example configuration
const bannerConfig: ConsentBannerConfig = {
  position: 'bottom',
  theme: 'auto',
  categories: [
    {
      id: 'necessary',
      label: 'Necessary',
      description: 'Required for the website to function',
      required: true,
      default: true,
    },
    {
      id: 'analytics',
      label: 'Analytics',
      description: 'Help us understand how visitors use our site',
      required: false,
      default: false,
    },
    {
      id: 'marketing',
      label: 'Marketing',
      description: 'Used for targeted advertising',
      required: false,
      default: false,
    },
  ],
  texts: {
    title: 'We value your privacy',
    description: 'We use cookies to enhance your experience.',
    acceptAll: 'Accept All',
    rejectAll: 'Reject All',
    customize: 'Customize',
    save: 'Save Preferences',
    privacyPolicy: 'Privacy Policy',
  },
  legal: {
    privacyPolicyUrl: '/privacy',
    cookiePolicyUrl: '/cookies',
    dpoEmail: 'privacy@example.com',
  },
};
```

## Related Specs

- 417-user-identification.md - User identity handling
- 424-analytics-export.md - Data export for portability
- 426-data-retention.md - Data retention policies
