# Spec 357: User Preferences Database Schema

## Overview
Define the SQLite schema for storing user preferences, including UI settings, notification preferences, and personalization options.

## Rust Implementation

### Schema Models
```rust
// src/database/schema/user_prefs.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

/// Notification frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum NotificationFrequency {
    #[default]
    Instant,
    Hourly,
    Daily,
    Weekly,
    Never,
}

/// User preferences
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserPreferences {
    pub id: String,
    pub user_id: String,

    // UI Preferences
    pub theme: Theme,
    pub language: String,
    pub timezone: String,
    pub date_format: String,
    pub time_format: String,
    pub sidebar_collapsed: bool,
    pub compact_mode: bool,
    pub font_size: String,

    // Notification Preferences
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub notification_frequency: NotificationFrequency,
    pub notify_mission_updates: bool,
    pub notify_spec_comments: bool,
    pub notify_mentions: bool,
    pub notify_deadlines: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,

    // Editor Preferences
    pub editor_theme: String,
    pub editor_font_family: String,
    pub editor_font_size: i32,
    pub editor_tab_size: i32,
    pub editor_word_wrap: bool,
    pub editor_line_numbers: bool,
    pub editor_minimap: bool,
    pub editor_auto_save: bool,
    pub editor_auto_save_delay: i32,

    // Dashboard Preferences
    pub default_dashboard: Option<String>,
    pub dashboard_widgets: Option<String>,  // JSON array

    // Keyboard Shortcuts
    pub keyboard_shortcuts: Option<String>,  // JSON object

    // Other Preferences
    pub default_mission_view: String,
    pub items_per_page: i32,
    pub show_completed_items: bool,
    pub auto_refresh_interval: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User workspace layout
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub is_default: bool,
    pub layout_data: String,  // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recent items for quick access
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RecentItem {
    pub id: String,
    pub user_id: String,
    pub item_type: String,
    pub item_id: String,
    pub item_title: String,
    pub accessed_at: DateTime<Utc>,
}

/// Saved search/filter
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SavedFilter {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub filter_type: String,  // missions, specs, forge
    pub filter_data: String,  // JSON
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

/// User-specific shortcuts/favorites
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Favorite {
    pub id: String,
    pub user_id: String,
    pub item_type: String,
    pub item_id: String,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}
```

### Migration SQL
```rust
// src/database/migrations/007_create_user_prefs.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000007,
        "create_user_prefs",
        r#"
-- User preferences
CREATE TABLE IF NOT EXISTS user_preferences (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL UNIQUE,

    -- UI Preferences
    theme TEXT NOT NULL DEFAULT 'system' CHECK (theme IN ('system', 'light', 'dark')),
    language TEXT NOT NULL DEFAULT 'en',
    timezone TEXT NOT NULL DEFAULT 'UTC',
    date_format TEXT NOT NULL DEFAULT 'YYYY-MM-DD',
    time_format TEXT NOT NULL DEFAULT 'HH:mm',
    sidebar_collapsed INTEGER NOT NULL DEFAULT 0,
    compact_mode INTEGER NOT NULL DEFAULT 0,
    font_size TEXT NOT NULL DEFAULT 'medium',

    -- Notification Preferences
    email_notifications INTEGER NOT NULL DEFAULT 1,
    push_notifications INTEGER NOT NULL DEFAULT 1,
    notification_frequency TEXT NOT NULL DEFAULT 'instant'
        CHECK (notification_frequency IN ('instant', 'hourly', 'daily', 'weekly', 'never')),
    notify_mission_updates INTEGER NOT NULL DEFAULT 1,
    notify_spec_comments INTEGER NOT NULL DEFAULT 1,
    notify_mentions INTEGER NOT NULL DEFAULT 1,
    notify_deadlines INTEGER NOT NULL DEFAULT 1,
    quiet_hours_start TEXT,
    quiet_hours_end TEXT,

    -- Editor Preferences
    editor_theme TEXT NOT NULL DEFAULT 'vs-dark',
    editor_font_family TEXT NOT NULL DEFAULT 'monospace',
    editor_font_size INTEGER NOT NULL DEFAULT 14,
    editor_tab_size INTEGER NOT NULL DEFAULT 4,
    editor_word_wrap INTEGER NOT NULL DEFAULT 1,
    editor_line_numbers INTEGER NOT NULL DEFAULT 1,
    editor_minimap INTEGER NOT NULL DEFAULT 0,
    editor_auto_save INTEGER NOT NULL DEFAULT 1,
    editor_auto_save_delay INTEGER NOT NULL DEFAULT 1000,

    -- Dashboard Preferences
    default_dashboard TEXT,
    dashboard_widgets TEXT,

    -- Keyboard Shortcuts
    keyboard_shortcuts TEXT,

    -- Other Preferences
    default_mission_view TEXT NOT NULL DEFAULT 'list',
    items_per_page INTEGER NOT NULL DEFAULT 25,
    show_completed_items INTEGER NOT NULL DEFAULT 1,
    auto_refresh_interval INTEGER NOT NULL DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_prefs_user ON user_preferences(user_id);

-- Workspace layouts
CREATE TABLE IF NOT EXISTS workspace_layouts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    layout_data TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_workspace_layouts_user ON workspace_layouts(user_id);

-- Recent items
CREATE TABLE IF NOT EXISTS recent_items (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, item_type, item_id)
);

CREATE INDEX IF NOT EXISTS idx_recent_items_user ON recent_items(user_id);
CREATE INDEX IF NOT EXISTS idx_recent_items_accessed ON recent_items(user_id, accessed_at DESC);

-- Saved filters
CREATE TABLE IF NOT EXISTS saved_filters (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    filter_type TEXT NOT NULL,
    filter_data TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, name, filter_type)
);

CREATE INDEX IF NOT EXISTS idx_saved_filters_user ON saved_filters(user_id);

-- Favorites
CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, item_type, item_id)
);

CREATE INDEX IF NOT EXISTS idx_favorites_user ON favorites(user_id);

-- Update timestamp trigger
CREATE TRIGGER IF NOT EXISTS update_user_prefs_timestamp
AFTER UPDATE ON user_preferences
BEGIN
    UPDATE user_preferences SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_workspace_layouts_timestamp
AFTER UPDATE ON workspace_layouts
BEGIN
    UPDATE workspace_layouts SET updated_at = datetime('now') WHERE id = NEW.id;
END;
"#
    ).with_down(r#"
DROP TRIGGER IF EXISTS update_workspace_layouts_timestamp;
DROP TRIGGER IF EXISTS update_user_prefs_timestamp;
DROP TABLE IF EXISTS favorites;
DROP TABLE IF EXISTS saved_filters;
DROP TABLE IF EXISTS recent_items;
DROP TABLE IF EXISTS workspace_layouts;
DROP TABLE IF EXISTS user_preferences;
"#)
}
```

### Default Preferences
```rust
// src/database/schema/user_prefs_defaults.rs

use super::user_prefs::*;
use chrono::Utc;
use uuid::Uuid;

impl UserPreferences {
    /// Create default preferences for a user
    pub fn default_for_user(user_id: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),

            theme: Theme::System,
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            sidebar_collapsed: false,
            compact_mode: false,
            font_size: "medium".to_string(),

            email_notifications: true,
            push_notifications: true,
            notification_frequency: NotificationFrequency::Instant,
            notify_mission_updates: true,
            notify_spec_comments: true,
            notify_mentions: true,
            notify_deadlines: true,
            quiet_hours_start: None,
            quiet_hours_end: None,

            editor_theme: "vs-dark".to_string(),
            editor_font_family: "monospace".to_string(),
            editor_font_size: 14,
            editor_tab_size: 4,
            editor_word_wrap: true,
            editor_line_numbers: true,
            editor_minimap: false,
            editor_auto_save: true,
            editor_auto_save_delay: 1000,

            default_dashboard: None,
            dashboard_widgets: None,
            keyboard_shortcuts: None,

            default_mission_view: "list".to_string(),
            items_per_page: 25,
            show_completed_items: true,
            auto_refresh_interval: 0,

            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Check if notifications are enabled for a type
    pub fn should_notify(&self, notification_type: &str) -> bool {
        if !self.email_notifications && !self.push_notifications {
            return false;
        }

        if self.notification_frequency == NotificationFrequency::Never {
            return false;
        }

        // Check quiet hours
        if self.is_quiet_hours() {
            return false;
        }

        match notification_type {
            "mission_update" => self.notify_mission_updates,
            "spec_comment" => self.notify_spec_comments,
            "mention" => self.notify_mentions,
            "deadline" => self.notify_deadlines,
            _ => true,
        }
    }

    /// Check if current time is in quiet hours
    fn is_quiet_hours(&self) -> bool {
        match (&self.quiet_hours_start, &self.quiet_hours_end) {
            (Some(start), Some(end)) => {
                let now = chrono::Local::now().format("%H:%M").to_string();
                if start <= end {
                    now >= *start && now <= *end
                } else {
                    // Overnight quiet hours (e.g., 22:00 to 07:00)
                    now >= *start || now <= *end
                }
            }
            _ => false,
        }
    }

    /// Get keyboard shortcut for action
    pub fn get_shortcut(&self, action: &str) -> Option<String> {
        let shortcuts: serde_json::Value = self.keyboard_shortcuts
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        shortcuts.get(action)
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

/// Default keyboard shortcuts
pub fn default_keyboard_shortcuts() -> serde_json::Value {
    serde_json::json!({
        "save": "Ctrl+S",
        "search": "Ctrl+K",
        "new_mission": "Ctrl+Shift+M",
        "new_spec": "Ctrl+Shift+S",
        "toggle_sidebar": "Ctrl+B",
        "quick_switch": "Ctrl+P",
        "command_palette": "Ctrl+Shift+P",
        "help": "F1",
        "close_tab": "Ctrl+W",
        "next_tab": "Ctrl+Tab",
        "prev_tab": "Ctrl+Shift+Tab"
    })
}

/// Default dashboard widgets
pub fn default_dashboard_widgets() -> serde_json::Value {
    serde_json::json!([
        { "id": "active-missions", "type": "mission-list", "position": {"x": 0, "y": 0, "w": 6, "h": 4} },
        { "id": "recent-specs", "type": "spec-list", "position": {"x": 6, "y": 0, "w": 6, "h": 4} },
        { "id": "progress-chart", "type": "chart", "position": {"x": 0, "y": 4, "w": 12, "h": 3} },
        { "id": "activity-feed", "type": "activity", "position": {"x": 0, "y": 7, "w": 6, "h": 3} },
        { "id": "deadlines", "type": "deadline-list", "position": {"x": 6, "y": 7, "w": 6, "h": 3} }
    ])
}
```

## Schema Design Decisions

1. **Single Preferences Row**: One row per user for core preferences
2. **JSON for Complex Data**: Widgets, shortcuts stored as JSON
3. **Sensible Defaults**: All fields have reasonable defaults
4. **Recent Items**: Track recent access for quick navigation
5. **Saved Filters**: Persist frequently used search filters

## Files to Create
- `src/database/schema/user_prefs.rs` - User preferences models
- `src/database/schema/user_prefs_defaults.rs` - Default values
- `src/database/migrations/007_create_user_prefs.rs` - Migration
