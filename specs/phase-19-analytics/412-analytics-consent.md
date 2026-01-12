# Spec 412: Analytics User Consent

## Phase
19 - Analytics/Telemetry

## Spec ID
412

## Status
Planned

## Dependencies
- Spec 407: Analytics Configuration (config management)
- Spec 003: Configuration System (persistent settings)

## Estimated Context
~8%

---

## Objective

Implement a comprehensive user consent system for analytics collection, ensuring compliance with privacy regulations and user expectations while maintaining transparent data collection practices.

---

## Acceptance Criteria

- [ ] Implement consent state management
- [ ] Create first-run consent dialog handling
- [ ] Support granular consent categories
- [ ] Implement consent versioning for updates
- [ ] Create consent revocation mechanisms
- [ ] Persist consent state securely
- [ ] Provide consent status API
- [ ] Support consent export for compliance

---

## Implementation Details

### Consent System

```rust
// src/analytics/consent.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Consent categories that users can independently control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentCategory {
    /// Essential system telemetry (crash reports, critical errors)
    Essential,
    /// Usage analytics (feature usage, session data)
    Usage,
    /// Performance metrics (latency, resource usage)
    Performance,
    /// Business metrics (token usage, costs)
    Business,
    /// Third-party integrations
    ThirdParty,
}

impl ConsentCategory {
    /// Get all consent categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::Essential,
            Self::Usage,
            Self::Performance,
            Self::Business,
            Self::ThirdParty,
        ]
    }

    /// Get categories that can be opted out of
    pub fn optional() -> Vec<Self> {
        vec![
            Self::Usage,
            Self::Performance,
            Self::Business,
            Self::ThirdParty,
        ]
    }

    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            Self::Essential => "Critical error reports and system diagnostics necessary for application stability",
            Self::Usage => "Feature usage patterns and session information to improve the product",
            Self::Performance => "Performance metrics including response times and resource utilization",
            Self::Business => "Token consumption and cost tracking for your billing insights",
            Self::ThirdParty => "Anonymous data shared with third-party analytics services",
        }
    }

    /// Check if this category can be disabled
    pub fn is_optional(&self) -> bool {
        !matches!(self, Self::Essential)
    }
}

/// Consent status for a category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStatus {
    /// User has granted consent
    Granted,
    /// User has denied consent
    Denied,
    /// User has not yet decided
    Pending,
}

impl Default for ConsentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Record of a consent decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Unique identifier for this consent record
    pub id: Uuid,
    /// Category this consent applies to
    pub category: ConsentCategory,
    /// Consent status
    pub status: ConsentStatus,
    /// When this consent was given/revoked
    pub timestamp: DateTime<Utc>,
    /// Version of consent policy
    pub policy_version: String,
    /// Method of consent (first_run, settings, api)
    pub method: ConsentMethod,
    /// Optional reason for decision
    pub reason: Option<String>,
}

impl ConsentRecord {
    pub fn new(
        category: ConsentCategory,
        status: ConsentStatus,
        policy_version: &str,
        method: ConsentMethod,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            status,
            timestamp: Utc::now(),
            policy_version: policy_version.to_string(),
            method,
            reason: None,
        }
    }
}

/// Method by which consent was obtained
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentMethod {
    /// First-run dialog
    FirstRun,
    /// Settings panel
    Settings,
    /// Command-line flag
    CommandLine,
    /// API call
    Api,
    /// Environment variable
    Environment,
    /// Configuration file
    ConfigFile,
    /// Default (no explicit consent)
    Default,
}

/// Complete consent state for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentState {
    /// Unique installation ID
    pub installation_id: Uuid,
    /// Current policy version
    pub policy_version: String,
    /// Consent status per category
    pub categories: HashMap<ConsentCategory, ConsentStatus>,
    /// Full consent history
    pub history: Vec<ConsentRecord>,
    /// When consent was first requested
    pub first_prompt: Option<DateTime<Utc>>,
    /// When consent was last updated
    pub last_updated: DateTime<Utc>,
    /// Whether initial consent flow is complete
    pub initial_consent_complete: bool,
}

impl Default for ConsentState {
    fn default() -> Self {
        let mut categories = HashMap::new();
        for cat in ConsentCategory::all() {
            categories.insert(cat, ConsentStatus::Pending);
        }

        Self {
            installation_id: Uuid::new_v4(),
            policy_version: CURRENT_POLICY_VERSION.to_string(),
            categories,
            history: Vec::new(),
            first_prompt: None,
            last_updated: Utc::now(),
            initial_consent_complete: false,
        }
    }
}

impl ConsentState {
    /// Check if a specific category is consented
    pub fn is_consented(&self, category: ConsentCategory) -> bool {
        self.categories
            .get(&category)
            .map(|s| *s == ConsentStatus::Granted)
            .unwrap_or(false)
    }

    /// Check if any analytics is consented
    pub fn has_any_consent(&self) -> bool {
        self.categories.values().any(|s| *s == ConsentStatus::Granted)
    }

    /// Check if all optional categories are pending
    pub fn needs_consent_prompt(&self) -> bool {
        !self.initial_consent_complete
    }

    /// Get all granted categories
    pub fn granted_categories(&self) -> Vec<ConsentCategory> {
        self.categories
            .iter()
            .filter(|(_, s)| **s == ConsentStatus::Granted)
            .map(|(c, _)| *c)
            .collect()
    }
}

/// Current consent policy version
pub const CURRENT_POLICY_VERSION: &str = "1.0.0";

/// Consent manager
pub struct ConsentManager {
    state: Arc<RwLock<ConsentState>>,
    storage_path: Option<PathBuf>,
}

impl ConsentManager {
    /// Create a new consent manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ConsentState::default())),
            storage_path: None,
        }
    }

    /// Create with persistent storage
    pub fn with_storage(path: PathBuf) -> Result<Self, ConsentError> {
        let state = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| ConsentError::StorageError(e.to_string()))?;
            serde_json::from_str(&content)
                .map_err(|e| ConsentError::DeserializationError(e.to_string()))?
        } else {
            ConsentState::default()
        };

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            storage_path: Some(path),
        })
    }

    /// Get current consent state
    pub async fn get_state(&self) -> ConsentState {
        self.state.read().await.clone()
    }

    /// Check if category is consented
    pub async fn is_consented(&self, category: ConsentCategory) -> bool {
        self.state.read().await.is_consented(category)
    }

    /// Grant consent for a category
    pub async fn grant(
        &self,
        category: ConsentCategory,
        method: ConsentMethod,
    ) -> Result<(), ConsentError> {
        self.update_consent(category, ConsentStatus::Granted, method, None)
            .await
    }

    /// Deny consent for a category
    pub async fn deny(
        &self,
        category: ConsentCategory,
        method: ConsentMethod,
        reason: Option<&str>,
    ) -> Result<(), ConsentError> {
        if !category.is_optional() {
            return Err(ConsentError::CannotDenyEssential);
        }

        self.update_consent(category, ConsentStatus::Denied, method, reason.map(String::from))
            .await
    }

    /// Update consent for a category
    async fn update_consent(
        &self,
        category: ConsentCategory,
        status: ConsentStatus,
        method: ConsentMethod,
        reason: Option<String>,
    ) -> Result<(), ConsentError> {
        let mut state = self.state.write().await;

        // Create consent record
        let mut record = ConsentRecord::new(category, status, &state.policy_version, method);
        record.reason = reason;

        // Update state
        state.categories.insert(category, status);
        state.history.push(record);
        state.last_updated = Utc::now();

        drop(state);

        // Persist
        self.save().await?;

        Ok(())
    }

    /// Grant consent for all optional categories
    pub async fn grant_all(&self, method: ConsentMethod) -> Result<(), ConsentError> {
        for category in ConsentCategory::all() {
            self.grant(category, method).await?;
        }
        self.mark_initial_consent_complete().await
    }

    /// Deny consent for all optional categories
    pub async fn deny_all(&self, method: ConsentMethod) -> Result<(), ConsentError> {
        // Essential is always granted
        self.grant(ConsentCategory::Essential, method).await?;

        for category in ConsentCategory::optional() {
            self.deny(category, method, None).await?;
        }
        self.mark_initial_consent_complete().await
    }

    /// Mark initial consent flow as complete
    pub async fn mark_initial_consent_complete(&self) -> Result<(), ConsentError> {
        let mut state = self.state.write().await;
        state.initial_consent_complete = true;
        if state.first_prompt.is_none() {
            state.first_prompt = Some(Utc::now());
        }
        drop(state);
        self.save().await
    }

    /// Check if consent needs to be re-requested (policy update)
    pub async fn needs_policy_update(&self) -> bool {
        let state = self.state.read().await;
        state.policy_version != CURRENT_POLICY_VERSION
    }

    /// Handle policy version update
    pub async fn handle_policy_update(&self) -> Result<(), ConsentError> {
        let mut state = self.state.write().await;

        if state.policy_version != CURRENT_POLICY_VERSION {
            // Reset to pending for re-consent
            for category in ConsentCategory::optional() {
                state.categories.insert(category, ConsentStatus::Pending);
            }
            state.policy_version = CURRENT_POLICY_VERSION.to_string();
            state.initial_consent_complete = false;
        }

        drop(state);
        self.save().await
    }

    /// Export consent records for compliance
    pub async fn export_consent_records(&self) -> ConsentExport {
        let state = self.state.read().await;

        ConsentExport {
            installation_id: state.installation_id,
            exported_at: Utc::now(),
            policy_version: state.policy_version.clone(),
            current_consent: state.categories.clone(),
            consent_history: state.history.clone(),
        }
    }

    /// Revoke all consent and delete data
    pub async fn revoke_all_and_delete(&self) -> Result<(), ConsentError> {
        let mut state = self.state.write().await;

        // Record revocation
        for category in ConsentCategory::all() {
            let record = ConsentRecord::new(
                category,
                ConsentStatus::Denied,
                &state.policy_version,
                ConsentMethod::Api,
            );
            state.history.push(record);
            state.categories.insert(category, ConsentStatus::Denied);
        }

        state.last_updated = Utc::now();
        drop(state);

        self.save().await?;

        Ok(())
    }

    /// Save consent state to storage
    async fn save(&self) -> Result<(), ConsentError> {
        if let Some(ref path) = self.storage_path {
            let state = self.state.read().await;
            let content = serde_json::to_string_pretty(&*state)
                .map_err(|e| ConsentError::SerializationError(e.to_string()))?;

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ConsentError::StorageError(e.to_string()))?;
            }

            std::fs::write(path, content)
                .map_err(|e| ConsentError::StorageError(e.to_string()))?;
        }
        Ok(())
    }

    /// Load consent from environment variables
    pub async fn load_from_env(&self) -> Result<(), ConsentError> {
        if let Ok(val) = std::env::var("TACHIKOMA_ANALYTICS_CONSENT") {
            match val.to_lowercase().as_str() {
                "all" | "true" | "yes" | "1" => {
                    self.grant_all(ConsentMethod::Environment).await?;
                }
                "none" | "false" | "no" | "0" => {
                    self.deny_all(ConsentMethod::Environment).await?;
                }
                _ => {
                    // Parse comma-separated category list
                    let categories: Vec<&str> = val.split(',').map(|s| s.trim()).collect();
                    for cat_str in categories {
                        let category = match cat_str.to_lowercase().as_str() {
                            "usage" => Some(ConsentCategory::Usage),
                            "performance" => Some(ConsentCategory::Performance),
                            "business" => Some(ConsentCategory::Business),
                            "thirdparty" | "third_party" => Some(ConsentCategory::ThirdParty),
                            _ => None,
                        };
                        if let Some(cat) = category {
                            self.grant(cat, ConsentMethod::Environment).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Consent export for compliance/GDPR requests
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentExport {
    pub installation_id: Uuid,
    pub exported_at: DateTime<Utc>,
    pub policy_version: String,
    pub current_consent: HashMap<ConsentCategory, ConsentStatus>,
    pub consent_history: Vec<ConsentRecord>,
}

/// Consent-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConsentError {
    #[error("Cannot deny essential consent")]
    CannotDenyEssential,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Consent prompt builder for CLI/UI
#[derive(Debug)]
pub struct ConsentPrompt {
    pub title: String,
    pub description: String,
    pub categories: Vec<ConsentPromptCategory>,
    pub accept_all_text: String,
    pub deny_all_text: String,
    pub customize_text: String,
}

#[derive(Debug)]
pub struct ConsentPromptCategory {
    pub category: ConsentCategory,
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_enabled: bool,
}

impl ConsentPrompt {
    pub fn build() -> Self {
        let categories = ConsentCategory::all()
            .into_iter()
            .map(|cat| ConsentPromptCategory {
                category: cat,
                name: format!("{:?}", cat),
                description: cat.description().to_string(),
                required: !cat.is_optional(),
                default_enabled: true,
            })
            .collect();

        Self {
            title: "Analytics & Telemetry Consent".to_string(),
            description: "Tachikoma collects anonymous usage data to improve the product. \
                          You can customize what data is collected.".to_string(),
            categories,
            accept_all_text: "Accept All".to_string(),
            deny_all_text: "Essential Only".to_string(),
            customize_text: "Customize".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consent_manager_basic() {
        let manager = ConsentManager::new();

        assert!(!manager.is_consented(ConsentCategory::Usage).await);

        manager.grant(ConsentCategory::Usage, ConsentMethod::Api).await.unwrap();

        assert!(manager.is_consented(ConsentCategory::Usage).await);
    }

    #[tokio::test]
    async fn test_cannot_deny_essential() {
        let manager = ConsentManager::new();

        let result = manager.deny(
            ConsentCategory::Essential,
            ConsentMethod::Api,
            None,
        ).await;

        assert!(matches!(result, Err(ConsentError::CannotDenyEssential)));
    }

    #[tokio::test]
    async fn test_grant_all() {
        let manager = ConsentManager::new();

        manager.grant_all(ConsentMethod::Api).await.unwrap();

        let state = manager.get_state().await;
        assert!(state.initial_consent_complete);

        for category in ConsentCategory::all() {
            assert!(state.is_consented(category));
        }
    }

    #[tokio::test]
    async fn test_deny_all() {
        let manager = ConsentManager::new();

        manager.deny_all(ConsentMethod::Api).await.unwrap();

        let state = manager.get_state().await;

        // Essential should still be granted
        assert!(state.is_consented(ConsentCategory::Essential));

        // Optional should be denied
        for category in ConsentCategory::optional() {
            assert!(!state.is_consented(category));
        }
    }

    #[tokio::test]
    async fn test_consent_export() {
        let manager = ConsentManager::new();
        manager.grant(ConsentCategory::Usage, ConsentMethod::Api).await.unwrap();
        manager.deny(ConsentCategory::ThirdParty, ConsentMethod::Api, Some("Privacy concerns")).await.unwrap();

        let export = manager.export_consent_records().await;

        assert_eq!(export.consent_history.len(), 2);
        assert_eq!(export.current_consent.get(&ConsentCategory::Usage), Some(&ConsentStatus::Granted));
        assert_eq!(export.current_consent.get(&ConsentCategory::ThirdParty), Some(&ConsentStatus::Denied));
    }

    #[tokio::test]
    async fn test_consent_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("consent.json");

        {
            let manager = ConsentManager::with_storage(path.clone()).unwrap();
            manager.grant(ConsentCategory::Usage, ConsentMethod::Api).await.unwrap();
        }

        // Reload from storage
        let manager = ConsentManager::with_storage(path).unwrap();
        assert!(manager.is_consented(ConsentCategory::Usage).await);
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Consent state management
   - Grant/deny operations
   - Essential consent protection
   - Persistence round-trip

2. **Integration Tests**
   - Consent with analytics system
   - Environment variable loading
   - Policy version updates

3. **Compliance Tests**
   - Consent export completeness
   - Revocation effectiveness

---

## Related Specs

- Spec 407: Analytics Configuration
- Spec 413: Privacy Controls
- Spec 424: Data Retention
