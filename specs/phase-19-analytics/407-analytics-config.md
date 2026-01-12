# Spec 407: Analytics Configuration

## Phase
19 - Analytics/Telemetry

## Spec ID
407

## Status
Planned

## Dependencies
- Spec 406: Analytics Types (event definitions)
- Spec 003: Configuration System (config infrastructure)

## Estimated Context
~9%

---

## Objective

Define comprehensive configuration options for the analytics system, enabling fine-grained control over data collection, privacy settings, storage limits, and export behavior.

---

## Acceptance Criteria

- [ ] Define analytics configuration structure
- [ ] Implement per-event-type collection settings
- [ ] Create sampling configuration options
- [ ] Define storage and retention settings
- [ ] Implement privacy level configuration
- [ ] Support environment-based configuration
- [ ] Create configuration validation
- [ ] Implement runtime configuration updates

---

## Implementation Details

### Analytics Configuration

```rust
// src/analytics/config.rs

use crate::analytics::types::{EventCategory, EventPriority, EventType, SamplingConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Master analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalyticsConfig {
    /// Whether analytics is enabled globally
    pub enabled: bool,

    /// Privacy level setting
    pub privacy_level: PrivacyLevel,

    /// Collection settings
    pub collection: CollectionConfig,

    /// Storage settings
    pub storage: StorageConfig,

    /// Export settings
    pub export: ExportConfig,

    /// Retention settings
    pub retention: RetentionConfig,

    /// Per-event-type overrides
    pub event_overrides: HashMap<String, EventOverride>,

    /// Per-category settings
    pub category_settings: HashMap<EventCategory, CategorySettings>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            privacy_level: PrivacyLevel::Balanced,
            collection: CollectionConfig::default(),
            storage: StorageConfig::default(),
            export: ExportConfig::default(),
            retention: RetentionConfig::default(),
            event_overrides: HashMap::new(),
            category_settings: default_category_settings(),
        }
    }
}

fn default_category_settings() -> HashMap<EventCategory, CategorySettings> {
    let mut settings = HashMap::new();

    settings.insert(EventCategory::Usage, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 1.0, min_per_window: 1, window_seconds: 60 },
        min_priority: EventPriority::Low,
    });

    settings.insert(EventCategory::Performance, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 0.1, min_per_window: 10, window_seconds: 60 },
        min_priority: EventPriority::Normal,
    });

    settings.insert(EventCategory::Error, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 1.0, min_per_window: 100, window_seconds: 60 },
        min_priority: EventPriority::Low,
    });

    settings.insert(EventCategory::Business, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 1.0, min_per_window: 1, window_seconds: 60 },
        min_priority: EventPriority::Low,
    });

    settings.insert(EventCategory::Security, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 1.0, min_per_window: 100, window_seconds: 60 },
        min_priority: EventPriority::Low,
    });

    settings.insert(EventCategory::System, CategorySettings {
        enabled: true,
        sampling: SamplingConfig { rate: 1.0, min_per_window: 1, window_seconds: 60 },
        min_priority: EventPriority::Normal,
    });

    settings
}

/// Privacy level controlling data collection detail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// No data collection
    Off,
    /// Minimal anonymous metrics only
    Minimal,
    /// Balanced data collection (default)
    Balanced,
    /// Full data collection for debugging
    Full,
}

impl PrivacyLevel {
    /// Get which data categories are allowed at this privacy level
    pub fn allowed_categories(&self) -> Vec<EventCategory> {
        match self {
            Self::Off => vec![],
            Self::Minimal => vec![EventCategory::Error],
            Self::Balanced => vec![
                EventCategory::Usage,
                EventCategory::Performance,
                EventCategory::Error,
                EventCategory::Business,
                EventCategory::System,
            ],
            Self::Full => vec![
                EventCategory::Usage,
                EventCategory::Performance,
                EventCategory::Error,
                EventCategory::Business,
                EventCategory::Security,
                EventCategory::System,
                EventCategory::Custom,
            ],
        }
    }

    /// Check if stack traces should be included
    pub fn include_stack_traces(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if detailed timing data should be included
    pub fn include_detailed_timing(&self) -> bool {
        matches!(self, Self::Balanced | Self::Full)
    }
}

/// Settings for event collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CollectionConfig {
    /// Maximum events to buffer before flush
    pub buffer_size: usize,

    /// Flush interval in seconds
    pub flush_interval_secs: u64,

    /// Default sampling rate (0.0 to 1.0)
    pub default_sampling_rate: f64,

    /// Whether to collect session data
    pub collect_sessions: bool,

    /// Whether to include environment metadata
    pub include_environment: bool,

    /// Batch size for event processing
    pub batch_size: usize,

    /// Maximum queue depth
    pub max_queue_depth: usize,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            flush_interval_secs: 30,
            default_sampling_rate: 1.0,
            collect_sessions: true,
            include_environment: true,
            batch_size: 100,
            max_queue_depth: 10000,
        }
    }
}

/// Settings for local analytics storage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Path to analytics database
    pub database_path: Option<PathBuf>,

    /// Maximum database size in MB
    pub max_size_mb: u64,

    /// Enable compression
    pub compression: bool,

    /// Encryption key (if encrypting at rest)
    pub encryption_enabled: bool,

    /// Write-ahead log mode
    pub wal_mode: bool,

    /// Sync mode for durability
    pub sync_mode: SyncMode,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: None,
            max_size_mb: 100,
            compression: true,
            encryption_enabled: false,
            wal_mode: true,
            sync_mode: SyncMode::Normal,
        }
    }
}

/// Database sync modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    /// Sync on every write
    Full,
    /// Sync periodically
    Normal,
    /// No explicit sync (fastest, least durable)
    Off,
}

/// Settings for data export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportConfig {
    /// Enable automatic export
    pub auto_export: bool,

    /// Export interval in hours
    pub export_interval_hours: u64,

    /// Export directory
    pub export_path: Option<PathBuf>,

    /// Default export format
    pub default_format: ExportFormat,

    /// Compress exported files
    pub compress_exports: bool,

    /// Maximum export file size in MB
    pub max_file_size_mb: u64,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            auto_export: false,
            export_interval_hours: 24,
            export_path: None,
            default_format: ExportFormat::Json,
            compress_exports: true,
            max_file_size_mb: 50,
        }
    }
}

/// Export file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Csv,
    Parquet,
    Ndjson,
}

/// Settings for data retention
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Default retention period in days
    pub default_retention_days: u32,

    /// Per-category retention periods
    pub category_retention: HashMap<EventCategory, u32>,

    /// Enable automatic cleanup
    pub auto_cleanup: bool,

    /// Cleanup interval in hours
    pub cleanup_interval_hours: u64,

    /// Keep aggregated data longer
    pub aggregated_retention_days: u32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        let mut category_retention = HashMap::new();
        category_retention.insert(EventCategory::Error, 90);
        category_retention.insert(EventCategory::Security, 365);
        category_retention.insert(EventCategory::Business, 365);

        Self {
            default_retention_days: 30,
            category_retention,
            auto_cleanup: true,
            cleanup_interval_hours: 24,
            aggregated_retention_days: 365,
        }
    }
}

/// Override settings for specific event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOverride {
    /// Override enabled status
    pub enabled: Option<bool>,

    /// Override sampling config
    pub sampling: Option<SamplingConfig>,

    /// Override minimum priority
    pub min_priority: Option<EventPriority>,

    /// Override retention days
    pub retention_days: Option<u32>,
}

/// Settings for an event category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySettings {
    /// Whether this category is enabled
    pub enabled: bool,

    /// Sampling configuration
    pub sampling: SamplingConfig,

    /// Minimum priority to collect
    pub min_priority: EventPriority,
}

/// Configuration manager for analytics
pub struct AnalyticsConfigManager {
    config: AnalyticsConfig,
    config_path: Option<PathBuf>,
}

impl AnalyticsConfigManager {
    /// Create a new config manager with defaults
    pub fn new() -> Self {
        Self {
            config: AnalyticsConfig::default(),
            config_path: None,
        }
    }

    /// Load configuration from a file
    pub fn load_from_file(path: PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let config: AnalyticsConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let mut manager = Self {
            config,
            config_path: Some(path),
        };

        manager.validate()?;
        Ok(manager)
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Result<Self, ConfigError> {
        let mut config = AnalyticsConfig::default();

        if let Ok(val) = std::env::var("TACHIKOMA_ANALYTICS_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("TACHIKOMA_ANALYTICS_PRIVACY") {
            config.privacy_level = match val.to_lowercase().as_str() {
                "off" => PrivacyLevel::Off,
                "minimal" => PrivacyLevel::Minimal,
                "balanced" => PrivacyLevel::Balanced,
                "full" => PrivacyLevel::Full,
                _ => PrivacyLevel::Balanced,
            };
        }

        if let Ok(val) = std::env::var("TACHIKOMA_ANALYTICS_DB_PATH") {
            config.storage.database_path = Some(PathBuf::from(val));
        }

        if let Ok(val) = std::env::var("TACHIKOMA_ANALYTICS_RETENTION_DAYS") {
            if let Ok(days) = val.parse() {
                config.retention.default_retention_days = days;
            }
        }

        let mut manager = Self {
            config,
            config_path: None,
        };

        manager.validate()?;
        Ok(manager)
    }

    /// Get the current configuration
    pub fn config(&self) -> &AnalyticsConfig {
        &self.config
    }

    /// Update configuration at runtime
    pub fn update(&mut self, updates: ConfigUpdate) -> Result<(), ConfigError> {
        if let Some(enabled) = updates.enabled {
            self.config.enabled = enabled;
        }

        if let Some(privacy) = updates.privacy_level {
            self.config.privacy_level = privacy;
        }

        if let Some(sampling) = updates.default_sampling_rate {
            if !(0.0..=1.0).contains(&sampling) {
                return Err(ConfigError::ValidationError(
                    "Sampling rate must be between 0.0 and 1.0".to_string(),
                ));
            }
            self.config.collection.default_sampling_rate = sampling;
        }

        self.validate()?;
        Ok(())
    }

    /// Check if an event type should be collected
    pub fn should_collect(&self, event_type: &EventType, priority: EventPriority) -> bool {
        if !self.config.enabled {
            return false;
        }

        let category = event_type.category();

        // Check if category is allowed by privacy level
        if !self.config.privacy_level.allowed_categories().contains(&category) {
            return false;
        }

        // Check event-specific override
        let event_key = format!("{:?}", event_type);
        if let Some(override_config) = self.config.event_overrides.get(&event_key) {
            if let Some(enabled) = override_config.enabled {
                if !enabled {
                    return false;
                }
            }
            if let Some(min_priority) = override_config.min_priority {
                if priority < min_priority {
                    return false;
                }
            }
        }

        // Check category settings
        if let Some(category_config) = self.config.category_settings.get(&category) {
            if !category_config.enabled {
                return false;
            }
            if priority < category_config.min_priority {
                return false;
            }
        }

        true
    }

    /// Get sampling config for an event type
    pub fn get_sampling(&self, event_type: &EventType) -> SamplingConfig {
        // Check event-specific override
        let event_key = format!("{:?}", event_type);
        if let Some(override_config) = self.config.event_overrides.get(&event_key) {
            if let Some(sampling) = &override_config.sampling {
                return sampling.clone();
            }
        }

        // Check category settings
        let category = event_type.category();
        if let Some(category_config) = self.config.category_settings.get(&category) {
            return category_config.sampling.clone();
        }

        // Return default
        SamplingConfig {
            rate: self.config.collection.default_sampling_rate,
            ..Default::default()
        }
    }

    /// Get retention period for an event category
    pub fn get_retention(&self, category: EventCategory) -> Duration {
        let days = self.config.retention.category_retention
            .get(&category)
            .copied()
            .unwrap_or(self.config.retention.default_retention_days);

        Duration::from_secs(days as u64 * 24 * 60 * 60)
    }

    /// Validate the configuration
    fn validate(&self) -> Result<(), ConfigError> {
        let config = &self.config;

        // Validate sampling rate
        if !(0.0..=1.0).contains(&config.collection.default_sampling_rate) {
            return Err(ConfigError::ValidationError(
                "default_sampling_rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate buffer size
        if config.collection.buffer_size == 0 {
            return Err(ConfigError::ValidationError(
                "buffer_size must be greater than 0".to_string(),
            ));
        }

        // Validate retention
        if config.retention.default_retention_days == 0 {
            return Err(ConfigError::ValidationError(
                "retention_days must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| ConfigError::IoError("No config path set".to_string()))?;

        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }
}

impl Default for AnalyticsConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime configuration updates
#[derive(Debug, Default)]
pub struct ConfigUpdate {
    pub enabled: Option<bool>,
    pub privacy_level: Option<PrivacyLevel>,
    pub default_sampling_rate: Option<f64>,
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialize error: {0}")]
    SerializeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyticsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.privacy_level, PrivacyLevel::Balanced);
    }

    #[test]
    fn test_privacy_levels() {
        assert!(PrivacyLevel::Off.allowed_categories().is_empty());
        assert!(PrivacyLevel::Full.allowed_categories().contains(&EventCategory::Security));
        assert!(!PrivacyLevel::Balanced.allowed_categories().contains(&EventCategory::Security));
    }

    #[test]
    fn test_should_collect() {
        let manager = AnalyticsConfigManager::new();

        assert!(manager.should_collect(&EventType::MissionCreated, EventPriority::Normal));
        assert!(manager.should_collect(&EventType::ErrorOccurred, EventPriority::High));
    }

    #[test]
    fn test_config_update() {
        let mut manager = AnalyticsConfigManager::new();

        manager.update(ConfigUpdate {
            enabled: Some(false),
            ..Default::default()
        }).unwrap();

        assert!(!manager.config().enabled);
    }

    #[test]
    fn test_validation() {
        let mut manager = AnalyticsConfigManager::new();

        let result = manager.update(ConfigUpdate {
            default_sampling_rate: Some(1.5),
            ..Default::default()
        });

        assert!(result.is_err());
    }
}
```

---

## Testing Requirements

1. **Unit Tests**
   - Default configuration validity
   - Configuration loading from various sources
   - Validation logic for all fields
   - Privacy level category filtering

2. **Integration Tests**
   - Configuration file loading and saving
   - Environment variable overrides
   - Runtime configuration updates

---

## Related Specs

- Spec 406: Analytics Types
- Spec 408: Analytics Collector
- Spec 412: User Consent
- Spec 413: Privacy Controls
