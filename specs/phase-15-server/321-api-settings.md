# Spec 321: Settings API

## Phase
15 - Server/API Layer

## Spec ID
321

## Status
Planned

## Dependencies
- Spec 311: Server Setup
- Spec 312: Server Configuration

## Estimated Context
~8%

---

## Objective

Implement the Settings API for Tachikoma, providing endpoints to manage application settings, user preferences, and system configuration with proper validation and access control.

---

## Acceptance Criteria

- [ ] Get and update application settings
- [ ] User preference management
- [ ] Feature flag management
- [ ] Export/import settings
- [ ] Settings validation
- [ ] Settings change auditing
- [ ] Default settings reset

---

## Implementation Details

### Request/Response Types

```rust
// src/api/types/settings.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use validator::Validate;

/// Settings categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SettingsCategory {
    General,
    Editor,
    Execution,
    Appearance,
    Notifications,
    Privacy,
    Advanced,
}

/// Settings response
#[derive(Debug, Clone, Serialize)]
pub struct SettingsResponse {
    pub general: GeneralSettings,
    pub editor: EditorSettings,
    pub execution: ExecutionSettings,
    pub appearance: AppearanceSettings,
    pub notifications: NotificationSettings,
    pub privacy: PrivacySettings,
    pub advanced: AdvancedSettings,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GeneralSettings {
    /// Application language
    pub language: String,

    /// Timezone
    pub timezone: String,

    /// Date format
    pub date_format: String,

    /// Auto-save interval in seconds (0 = disabled)
    #[validate(range(min = 0, max = 300))]
    pub auto_save_interval: u32,

    /// Default mission template
    pub default_template_id: Option<uuid::Uuid>,

    /// Startup behavior
    pub startup_behavior: StartupBehavior,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupBehavior {
    LastMission,
    Dashboard,
    NewMission,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EditorSettings {
    /// Editor theme
    pub theme: String,

    /// Font family
    pub font_family: String,

    /// Font size
    #[validate(range(min = 8, max = 32))]
    pub font_size: u32,

    /// Tab size
    #[validate(range(min = 1, max = 8))]
    pub tab_size: u32,

    /// Use spaces instead of tabs
    pub use_spaces: bool,

    /// Word wrap
    pub word_wrap: bool,

    /// Line numbers
    pub line_numbers: bool,

    /// Minimap
    pub minimap: bool,

    /// Auto-complete
    pub auto_complete: bool,

    /// Syntax highlighting
    pub syntax_highlighting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExecutionSettings {
    /// Default backend ID
    pub default_backend_id: Option<uuid::Uuid>,

    /// Maximum concurrent executions
    #[validate(range(min = 1, max = 10))]
    pub max_concurrent: u32,

    /// Auto-apply file changes
    pub auto_apply_changes: bool,

    /// Show change diffs before applying
    pub show_diffs: bool,

    /// Confirm before execution
    pub confirm_before_execute: bool,

    /// Default temperature
    #[validate(range(min = 0.0, max = 2.0))]
    pub default_temperature: f32,

    /// Default max tokens
    #[validate(range(min = 100, max = 100000))]
    pub default_max_tokens: u32,

    /// Stream responses
    pub stream_responses: bool,

    /// Retry on failure
    pub retry_on_failure: bool,

    /// Max retries
    #[validate(range(min = 0, max = 5))]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// UI theme (light, dark, system)
    pub theme: AppearanceTheme,

    /// Accent color
    pub accent_color: String,

    /// Compact mode
    pub compact_mode: bool,

    /// Sidebar position
    pub sidebar_position: SidebarPosition,

    /// Show status bar
    pub show_status_bar: bool,

    /// Animation enabled
    pub animations: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceTheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Enable notifications
    pub enabled: bool,

    /// Notification types
    pub execution_complete: bool,
    pub execution_failed: bool,
    pub spec_status_change: bool,
    pub mission_complete: bool,

    /// Sound enabled
    pub sound: bool,

    /// Email notifications
    pub email_notifications: bool,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Send anonymous usage stats
    pub analytics: bool,

    /// Send crash reports
    pub crash_reports: bool,

    /// Store conversation history
    pub store_history: bool,

    /// History retention days (0 = forever)
    pub history_retention_days: u32,

    /// Redact sensitive data in logs
    pub redact_logs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdvancedSettings {
    /// Enable debug mode
    pub debug_mode: bool,

    /// Log level
    pub log_level: LogLevel,

    /// Enable experimental features
    pub experimental_features: bool,

    /// Custom API endpoints
    pub custom_endpoints: HashMap<String, String>,

    /// Request timeout in seconds
    #[validate(range(min = 5, max = 600))]
    pub request_timeout: u32,

    /// Cache size in MB
    #[validate(range(min = 10, max = 1000))]
    pub cache_size_mb: u32,

    /// Enable performance metrics
    pub performance_metrics: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Request to update settings
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSettingsRequest {
    pub general: Option<GeneralSettings>,
    pub editor: Option<EditorSettings>,
    pub execution: Option<ExecutionSettings>,
    pub appearance: Option<AppearanceSettings>,
    pub notifications: Option<NotificationSettings>,
    pub privacy: Option<PrivacySettings>,
    pub advanced: Option<AdvancedSettings>,
}

/// Feature flags
#[derive(Debug, Clone, Serialize)]
pub struct FeatureFlagsResponse {
    pub flags: HashMap<String, FeatureFlag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureFlag {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub default: bool,
    pub experimental: bool,
}

/// Settings export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub settings: SettingsResponse,
    pub feature_flags: HashMap<String, bool>,
}
```

### Settings Handlers

```rust
// src/server/handlers/settings.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use validator::Validate;

use crate::api::types::settings::*;
use crate::server::error::{ApiError, ApiResult};
use crate::server::state::AppState;

/// Get all settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> ApiResult<Json<SettingsResponse>> {
    let storage = state.storage();

    let settings = storage.settings().get_all().await?;

    Ok(Json(settings.into()))
}

/// Update settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateSettingsRequest>,
) -> ApiResult<Json<SettingsResponse>> {
    let storage = state.storage();

    // Get current settings
    let mut settings = storage.settings().get_all().await?;

    // Validate and apply updates
    if let Some(general) = request.general {
        general.validate().map_err(|e| {
            ApiError::Validation {
                errors: validation_errors_to_field_errors(e),
            }
        })?;
        settings.general = general;
    }

    if let Some(editor) = request.editor {
        editor.validate().map_err(|e| {
            ApiError::Validation {
                errors: validation_errors_to_field_errors(e),
            }
        })?;
        settings.editor = editor;
    }

    if let Some(execution) = request.execution {
        execution.validate().map_err(|e| {
            ApiError::Validation {
                errors: validation_errors_to_field_errors(e),
            }
        })?;

        // Validate backend exists if specified
        if let Some(backend_id) = execution.default_backend_id {
            if state.backend_manager().get(backend_id).is_none() {
                return Err(ApiError::bad_request(format!(
                    "Backend {} not found",
                    backend_id
                )));
            }
        }

        settings.execution = execution;
    }

    if let Some(appearance) = request.appearance {
        settings.appearance = appearance;
    }

    if let Some(notifications) = request.notifications {
        // Validate email if notifications enabled
        if notifications.email_notifications && notifications.email.is_none() {
            return Err(ApiError::bad_request(
                "Email required when email notifications are enabled",
            ));
        }
        settings.notifications = notifications;
    }

    if let Some(privacy) = request.privacy {
        settings.privacy = privacy;
    }

    if let Some(advanced) = request.advanced {
        advanced.validate().map_err(|e| {
            ApiError::Validation {
                errors: validation_errors_to_field_errors(e),
            }
        })?;
        settings.advanced = advanced;
    }

    settings.updated_at = Utc::now();

    // Save settings
    storage.settings().save_all(&settings).await?;

    // Log settings change
    tracing::info!(
        categories = ?changed_categories(&request),
        "Settings updated"
    );

    Ok(Json(settings.into()))
}

/// Get settings for a specific category
pub async fn get_category_settings(
    State(state): State<AppState>,
    Path(category): Path<SettingsCategory>,
) -> ApiResult<Json<serde_json::Value>> {
    let storage = state.storage();
    let settings = storage.settings().get_all().await?;

    let category_settings = match category {
        SettingsCategory::General => serde_json::to_value(&settings.general)?,
        SettingsCategory::Editor => serde_json::to_value(&settings.editor)?,
        SettingsCategory::Execution => serde_json::to_value(&settings.execution)?,
        SettingsCategory::Appearance => serde_json::to_value(&settings.appearance)?,
        SettingsCategory::Notifications => serde_json::to_value(&settings.notifications)?,
        SettingsCategory::Privacy => serde_json::to_value(&settings.privacy)?,
        SettingsCategory::Advanced => serde_json::to_value(&settings.advanced)?,
    };

    Ok(Json(category_settings))
}

/// Reset settings to defaults
pub async fn reset_settings(
    State(state): State<AppState>,
    Path(category): Path<Option<SettingsCategory>>,
) -> ApiResult<Json<SettingsResponse>> {
    let storage = state.storage();

    if let Some(cat) = category {
        // Reset specific category
        let mut settings = storage.settings().get_all().await?;

        match cat {
            SettingsCategory::General => settings.general = GeneralSettings::default(),
            SettingsCategory::Editor => settings.editor = EditorSettings::default(),
            SettingsCategory::Execution => settings.execution = ExecutionSettings::default(),
            SettingsCategory::Appearance => settings.appearance = AppearanceSettings::default(),
            SettingsCategory::Notifications => settings.notifications = NotificationSettings::default(),
            SettingsCategory::Privacy => settings.privacy = PrivacySettings::default(),
            SettingsCategory::Advanced => settings.advanced = AdvancedSettings::default(),
        }

        settings.updated_at = Utc::now();
        storage.settings().save_all(&settings).await?;

        Ok(Json(settings.into()))
    } else {
        // Reset all settings
        let settings = Settings::default();
        storage.settings().save_all(&settings).await?;

        Ok(Json(settings.into()))
    }
}

/// Get feature flags
pub async fn get_feature_flags(
    State(state): State<AppState>,
) -> ApiResult<Json<FeatureFlagsResponse>> {
    let storage = state.storage();
    let flags = storage.feature_flags().get_all().await?;

    Ok(Json(FeatureFlagsResponse {
        flags: flags.into_iter().map(|f| (f.name.clone(), f.into())).collect(),
    }))
}

/// Update a feature flag
pub async fn update_feature_flag(
    State(state): State<AppState>,
    Path(flag_name): Path<String>,
    Json(request): Json<UpdateFeatureFlagRequest>,
) -> ApiResult<Json<FeatureFlag>> {
    let storage = state.storage();

    let mut flag = storage
        .feature_flags()
        .get(&flag_name)
        .await?
        .ok_or_else(|| ApiError::not_found_with_id("FeatureFlag", flag_name.clone()))?;

    flag.enabled = request.enabled;

    storage.feature_flags().save(&flag).await?;

    tracing::info!(
        flag = %flag_name,
        enabled = %request.enabled,
        "Feature flag updated"
    );

    Ok(Json(flag.into()))
}

/// Export settings
pub async fn export_settings(
    State(state): State<AppState>,
) -> ApiResult<Json<SettingsExport>> {
    let storage = state.storage();

    let settings = storage.settings().get_all().await?;
    let flags = storage.feature_flags().get_all().await?;

    Ok(Json(SettingsExport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: Utc::now(),
        settings: settings.into(),
        feature_flags: flags.into_iter().map(|f| (f.name, f.enabled)).collect(),
    }))
}

/// Import settings
pub async fn import_settings(
    State(state): State<AppState>,
    Json(import): Json<SettingsExport>,
) -> ApiResult<Json<SettingsResponse>> {
    let storage = state.storage();

    // Validate version compatibility
    // (In production, add proper version checking)

    // Import settings
    let settings = import.settings;
    storage.settings().save_all(&settings.clone().into()).await?;

    // Import feature flags
    for (name, enabled) in import.feature_flags {
        if let Some(mut flag) = storage.feature_flags().get(&name).await? {
            flag.enabled = enabled;
            storage.feature_flags().save(&flag).await?;
        }
    }

    tracing::info!("Settings imported");

    Ok(Json(settings))
}

// Helper functions

fn changed_categories(request: &UpdateSettingsRequest) -> Vec<&'static str> {
    let mut categories = Vec::new();
    if request.general.is_some() { categories.push("general"); }
    if request.editor.is_some() { categories.push("editor"); }
    if request.execution.is_some() { categories.push("execution"); }
    if request.appearance.is_some() { categories.push("appearance"); }
    if request.notifications.is_some() { categories.push("notifications"); }
    if request.privacy.is_some() { categories.push("privacy"); }
    if request.advanced.is_some() { categories.push("advanced"); }
    categories
}
```

### Default Implementations

```rust
// src/api/types/settings.rs (defaults)

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            auto_save_interval: 30,
            default_template_id: None,
            startup_behavior: StartupBehavior::Dashboard,
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            theme: "vs-dark".to_string(),
            font_family: "JetBrains Mono, monospace".to_string(),
            font_size: 14,
            tab_size: 4,
            use_spaces: true,
            word_wrap: true,
            line_numbers: true,
            minimap: true,
            auto_complete: true,
            syntax_highlighting: true,
        }
    }
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            default_backend_id: None,
            max_concurrent: 3,
            auto_apply_changes: false,
            show_diffs: true,
            confirm_before_execute: true,
            default_temperature: 0.7,
            default_max_tokens: 4096,
            stream_responses: true,
            retry_on_failure: true,
            max_retries: 2,
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: AppearanceTheme::System,
            accent_color: "#6366f1".to_string(),
            compact_mode: false,
            sidebar_position: SidebarPosition::Left,
            show_status_bar: true,
            animations: true,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            execution_complete: true,
            execution_failed: true,
            spec_status_change: true,
            mission_complete: true,
            sound: true,
            email_notifications: false,
            email: None,
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            analytics: false,
            crash_reports: true,
            store_history: true,
            history_retention_days: 90,
            redact_logs: true,
        }
    }
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            debug_mode: false,
            log_level: LogLevel::Info,
            experimental_features: false,
            custom_endpoints: HashMap::new(),
            request_timeout: 120,
            cache_size_mb: 100,
            performance_metrics: false,
        }
    }
}
```

---

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.general.language, "en");
        assert_eq!(settings.editor.font_size, 14);
        assert!(settings.execution.confirm_before_execute);
    }

    #[test]
    fn test_settings_validation() {
        let mut editor = EditorSettings::default();
        editor.font_size = 100; // Invalid

        assert!(editor.validate().is_err());
    }

    #[tokio::test]
    async fn test_settings_round_trip() {
        let state = create_test_state().await;

        let original = get_settings(State(state.clone())).await.unwrap().0;

        let update = UpdateSettingsRequest {
            general: Some(GeneralSettings {
                language: "es".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let updated = update_settings(State(state.clone()), Json(update)).await.unwrap().0;

        assert_eq!(updated.general.language, "es");
    }
}
```

---

## Related Specs

- **Spec 312**: Server Configuration
- **Spec 320**: Backends API
