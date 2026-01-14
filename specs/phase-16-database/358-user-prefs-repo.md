# Spec 358: User Preferences Repository

## Overview
Implement the repository pattern for user preferences management with caching, validation, and bulk updates.


## Acceptance Criteria
- [x] Implementation complete per spec

## Rust Implementation

### User Preferences Repository
```rust
// src/database/repository/user_prefs.rs

use crate::database::schema::user_prefs::*;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum UserPrefsRepoError {
    #[error("User preferences not found for user: {0}")]
    NotFound(String),

    #[error("Invalid preference value: {0}")]
    InvalidValue(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Update for a single preference
#[derive(Debug, Clone)]
pub struct PreferenceUpdate {
    pub key: String,
    pub value: serde_json::Value,
}

pub struct UserPrefsRepository {
    pool: SqlitePool,
    cache: Arc<RwLock<HashMap<String, UserPreferences>>>,
}

impl UserPrefsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ==================== Core Preferences Methods ====================

    /// Get preferences for a user, creating defaults if not exists
    #[instrument(skip(self))]
    pub async fn get_or_create(&self, user_id: &str) -> Result<UserPreferences, UserPrefsRepoError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(prefs) = cache.get(user_id) {
                return Ok(prefs.clone());
            }
        }

        // Try to fetch from database
        let prefs = sqlx::query_as::<_, UserPreferences>(
            "SELECT * FROM user_preferences WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let prefs = match prefs {
            Some(p) => p,
            None => {
                // Create default preferences
                let default_prefs = UserPreferences::default_for_user(user_id);
                self.create(&default_prefs).await?;
                default_prefs
            }
        };

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(user_id.to_string(), prefs.clone());
        }

        Ok(prefs)
    }

    /// Create preferences
    async fn create(&self, prefs: &UserPreferences) -> Result<(), UserPrefsRepoError> {
        sqlx::query(r#"
            INSERT INTO user_preferences (
                id, user_id, theme, language, timezone, date_format, time_format,
                sidebar_collapsed, compact_mode, font_size,
                email_notifications, push_notifications, notification_frequency,
                notify_mission_updates, notify_spec_comments, notify_mentions, notify_deadlines,
                quiet_hours_start, quiet_hours_end,
                editor_theme, editor_font_family, editor_font_size, editor_tab_size,
                editor_word_wrap, editor_line_numbers, editor_minimap, editor_auto_save, editor_auto_save_delay,
                default_dashboard, dashboard_widgets, keyboard_shortcuts,
                default_mission_view, items_per_page, show_completed_items, auto_refresh_interval,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&prefs.id)
        .bind(&prefs.user_id)
        .bind(prefs.theme)
        .bind(&prefs.language)
        .bind(&prefs.timezone)
        .bind(&prefs.date_format)
        .bind(&prefs.time_format)
        .bind(prefs.sidebar_collapsed as i32)
        .bind(prefs.compact_mode as i32)
        .bind(&prefs.font_size)
        .bind(prefs.email_notifications as i32)
        .bind(prefs.push_notifications as i32)
        .bind(prefs.notification_frequency)
        .bind(prefs.notify_mission_updates as i32)
        .bind(prefs.notify_spec_comments as i32)
        .bind(prefs.notify_mentions as i32)
        .bind(prefs.notify_deadlines as i32)
        .bind(&prefs.quiet_hours_start)
        .bind(&prefs.quiet_hours_end)
        .bind(&prefs.editor_theme)
        .bind(&prefs.editor_font_family)
        .bind(prefs.editor_font_size)
        .bind(prefs.editor_tab_size)
        .bind(prefs.editor_word_wrap as i32)
        .bind(prefs.editor_line_numbers as i32)
        .bind(prefs.editor_minimap as i32)
        .bind(prefs.editor_auto_save as i32)
        .bind(prefs.editor_auto_save_delay)
        .bind(&prefs.default_dashboard)
        .bind(&prefs.dashboard_widgets)
        .bind(&prefs.keyboard_shortcuts)
        .bind(&prefs.default_mission_view)
        .bind(prefs.items_per_page)
        .bind(prefs.show_completed_items as i32)
        .bind(prefs.auto_refresh_interval)
        .bind(prefs.created_at)
        .bind(prefs.updated_at)
        .execute(&self.pool)
        .await?;

        debug!("Created default preferences for user: {}", prefs.user_id);
        Ok(())
    }

    /// Update a single preference
    #[instrument(skip(self))]
    pub async fn update_preference(
        &self,
        user_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<UserPreferences, UserPrefsRepoError> {
        // Validate the key and value
        self.validate_preference(key, &value)?;

        // Build and execute update query
        let sql = format!("UPDATE user_preferences SET {} = ? WHERE user_id = ?", key);

        let value_str = match &value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Bool(b) => (*b as i32).to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
        };

        sqlx::query(&sql)
            .bind(&value_str)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Invalidate cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(user_id);
        }

        debug!("Updated preference {} for user {}", key, user_id);
        self.get_or_create(user_id).await
    }

    /// Update multiple preferences at once
    #[instrument(skip(self, updates))]
    pub async fn update_preferences(
        &self,
        user_id: &str,
        updates: Vec<PreferenceUpdate>,
    ) -> Result<UserPreferences, UserPrefsRepoError> {
        // Validate all updates first
        for update in &updates {
            self.validate_preference(&update.key, &update.value)?;
        }

        // Build bulk update query
        let set_clauses: Vec<String> = updates
            .iter()
            .map(|u| format!("{} = ?", u.key))
            .collect();

        let sql = format!(
            "UPDATE user_preferences SET {} WHERE user_id = ?",
            set_clauses.join(", ")
        );

        let mut query = sqlx::query(&sql);

        for update in &updates {
            let value_str = match &update.value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => (*b as i32).to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => update.value.to_string(),
            };
            query = query.bind(value_str);
        }

        query = query.bind(user_id);
        query.execute(&self.pool).await?;

        // Invalidate cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(user_id);
        }

        debug!("Updated {} preferences for user {}", updates.len(), user_id);
        self.get_or_create(user_id).await
    }

    /// Validate preference key and value
    fn validate_preference(&self, key: &str, value: &serde_json::Value) -> Result<(), UserPrefsRepoError> {
        // Validate known keys
        match key {
            "theme" => {
                let valid = ["system", "light", "dark"];
                if let Some(s) = value.as_str() {
                    if !valid.contains(&s) {
                        return Err(UserPrefsRepoError::InvalidValue(
                            format!("Theme must be one of: {:?}", valid)
                        ));
                    }
                }
            }
            "notification_frequency" => {
                let valid = ["instant", "hourly", "daily", "weekly", "never"];
                if let Some(s) = value.as_str() {
                    if !valid.contains(&s) {
                        return Err(UserPrefsRepoError::InvalidValue(
                            format!("Notification frequency must be one of: {:?}", valid)
                        ));
                    }
                }
            }
            "items_per_page" => {
                if let Some(n) = value.as_i64() {
                    if !(10..=100).contains(&n) {
                        return Err(UserPrefsRepoError::InvalidValue(
                            "Items per page must be between 10 and 100".to_string()
                        ));
                    }
                }
            }
            "editor_font_size" => {
                if let Some(n) = value.as_i64() {
                    if !(8..=32).contains(&n) {
                        return Err(UserPrefsRepoError::InvalidValue(
                            "Editor font size must be between 8 and 32".to_string()
                        ));
                    }
                }
            }
            _ => {} // Allow other keys
        }

        Ok(())
    }

    /// Reset preferences to defaults
    pub async fn reset_to_defaults(&self, user_id: &str) -> Result<UserPreferences, UserPrefsRepoError> {
        sqlx::query("DELETE FROM user_preferences WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        // Invalidate cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(user_id);
        }

        self.get_or_create(user_id).await
    }

    // ==================== Workspace Layout Methods ====================

    /// Get workspace layouts for user
    pub async fn get_layouts(&self, user_id: &str) -> Result<Vec<WorkspaceLayout>, UserPrefsRepoError> {
        let layouts = sqlx::query_as::<_, WorkspaceLayout>(
            "SELECT * FROM workspace_layouts WHERE user_id = ? ORDER BY name"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(layouts)
    }

    /// Save workspace layout
    pub async fn save_layout(
        &self,
        user_id: &str,
        name: &str,
        layout_data: serde_json::Value,
        is_default: bool,
    ) -> Result<WorkspaceLayout, UserPrefsRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // If setting as default, unset other defaults
        if is_default {
            sqlx::query("UPDATE workspace_layouts SET is_default = 0 WHERE user_id = ?")
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(r#"
            INSERT OR REPLACE INTO workspace_layouts (id, user_id, name, is_default, layout_data, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(is_default as i32)
        .bind(layout_data.to_string())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let layout = sqlx::query_as::<_, WorkspaceLayout>(
            "SELECT * FROM workspace_layouts WHERE user_id = ? AND name = ?"
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(layout)
    }

    /// Delete workspace layout
    pub async fn delete_layout(&self, user_id: &str, name: &str) -> Result<bool, UserPrefsRepoError> {
        let result = sqlx::query("DELETE FROM workspace_layouts WHERE user_id = ? AND name = ?")
            .bind(user_id)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== Recent Items Methods ====================

    /// Add recent item
    pub async fn add_recent_item(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
        item_title: &str,
    ) -> Result<(), UserPrefsRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(r#"
            INSERT OR REPLACE INTO recent_items (id, user_id, item_type, item_id, item_title, accessed_at)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(user_id)
        .bind(item_type)
        .bind(item_id)
        .bind(item_title)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Keep only last 50 recent items
        sqlx::query(r#"
            DELETE FROM recent_items
            WHERE user_id = ? AND id NOT IN (
                SELECT id FROM recent_items
                WHERE user_id = ?
                ORDER BY accessed_at DESC
                LIMIT 50
            )
        "#)
        .bind(user_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent items
    pub async fn get_recent_items(
        &self,
        user_id: &str,
        limit: i64,
        item_type: Option<&str>,
    ) -> Result<Vec<RecentItem>, UserPrefsRepoError> {
        let items = if let Some(t) = item_type {
            sqlx::query_as::<_, RecentItem>(r#"
                SELECT * FROM recent_items
                WHERE user_id = ? AND item_type = ?
                ORDER BY accessed_at DESC
                LIMIT ?
            "#)
            .bind(user_id)
            .bind(t)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, RecentItem>(r#"
                SELECT * FROM recent_items
                WHERE user_id = ?
                ORDER BY accessed_at DESC
                LIMIT ?
            "#)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(items)
    }

    // ==================== Saved Filters Methods ====================

    /// Save a filter
    pub async fn save_filter(
        &self,
        user_id: &str,
        name: &str,
        filter_type: &str,
        filter_data: serde_json::Value,
        is_default: bool,
    ) -> Result<SavedFilter, UserPrefsRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        if is_default {
            sqlx::query("UPDATE saved_filters SET is_default = 0 WHERE user_id = ? AND filter_type = ?")
                .bind(user_id)
                .bind(filter_type)
                .execute(&self.pool)
                .await?;
        }

        sqlx::query(r#"
            INSERT OR REPLACE INTO saved_filters (id, user_id, name, filter_type, filter_data, is_default, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(user_id)
        .bind(name)
        .bind(filter_type)
        .bind(filter_data.to_string())
        .bind(is_default as i32)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let filter = sqlx::query_as::<_, SavedFilter>(
            "SELECT * FROM saved_filters WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await?;

        Ok(filter)
    }

    /// Get saved filters
    pub async fn get_saved_filters(
        &self,
        user_id: &str,
        filter_type: Option<&str>,
    ) -> Result<Vec<SavedFilter>, UserPrefsRepoError> {
        let filters = if let Some(t) = filter_type {
            sqlx::query_as::<_, SavedFilter>(
                "SELECT * FROM saved_filters WHERE user_id = ? AND filter_type = ? ORDER BY name"
            )
            .bind(user_id)
            .bind(t)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, SavedFilter>(
                "SELECT * FROM saved_filters WHERE user_id = ? ORDER BY filter_type, name"
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(filters)
    }

    // ==================== Favorites Methods ====================

    /// Add favorite
    pub async fn add_favorite(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<Favorite, UserPrefsRepoError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Get max order
        let (max_order,): (Option<i32>,) = sqlx::query_as(
            "SELECT MAX(display_order) FROM favorites WHERE user_id = ?"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let order = max_order.unwrap_or(0) + 1;

        sqlx::query(r#"
            INSERT OR IGNORE INTO favorites (id, user_id, item_type, item_id, display_order, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(user_id)
        .bind(item_type)
        .bind(item_id)
        .bind(order)
        .bind(now)
        .execute(&self.pool)
        .await?;

        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE user_id = ? AND item_type = ? AND item_id = ?"
        )
        .bind(user_id)
        .bind(item_type)
        .bind(item_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(favorite)
    }

    /// Remove favorite
    pub async fn remove_favorite(
        &self,
        user_id: &str,
        item_type: &str,
        item_id: &str,
    ) -> Result<bool, UserPrefsRepoError> {
        let result = sqlx::query(
            "DELETE FROM favorites WHERE user_id = ? AND item_type = ? AND item_id = ?"
        )
        .bind(user_id)
        .bind(item_type)
        .bind(item_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get favorites
    pub async fn get_favorites(&self, user_id: &str) -> Result<Vec<Favorite>, UserPrefsRepoError> {
        let favorites = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE user_id = ? ORDER BY display_order"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(favorites)
    }

    /// Clear cache for testing
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/user_prefs.rs` - User preferences repository
