# Spec 352: Configuration Repository

## Overview
Implement the repository pattern for configuration management with scoping, feature flags, and environment support.

## Rust Implementation

### Config Repository
```rust
// src/database/repository/config.rs

use crate::database::schema::config::*;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use tracing::{debug, instrument, warn};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum ConfigRepoError {
    #[error("Configuration not found: {0}")]
    NotFound(String),

    #[error("Configuration key already exists: {0}")]
    AlreadyExists(String),

    #[error("Configuration is read-only: {0}")]
    ReadOnly(String),

    #[error("Invalid value for type {0:?}: {1}")]
    InvalidValue(ConfigValueType, String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Input for setting configuration
#[derive(Debug, Clone)]
pub struct SetConfig {
    pub key: String,
    pub value: String,
    pub value_type: Option<ConfigValueType>,
    pub scope: ConfigScope,
    pub scope_id: Option<String>,
    pub description: Option<String>,
    pub is_sensitive: Option<bool>,
    pub is_readonly: Option<bool>,
    pub changed_by: Option<String>,
    pub change_reason: Option<String>,
}

/// Input for creating a feature flag
#[derive(Debug, Clone)]
pub struct CreateFeatureFlag {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rollout_percentage: Option<i32>,
    pub conditions: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub struct ConfigRepository {
    pool: SqlitePool,
}

impl ConfigRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== Configuration Methods ====================

    /// Get a configuration value by key
    #[instrument(skip(self))]
    pub async fn get(&self, key: &str, scope: ConfigScope, scope_id: Option<&str>) -> Result<Option<ConfigEntry>, ConfigRepoError> {
        let config = sqlx::query_as::<_, ConfigEntry>(r#"
            SELECT * FROM config_entries
            WHERE key = ? AND scope = ? AND (scope_id = ? OR (scope_id IS NULL AND ? IS NULL))
        "#)
        .bind(key)
        .bind(scope)
        .bind(scope_id)
        .bind(scope_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(config)
    }

    /// Get configuration with fallback through scopes
    #[instrument(skip(self))]
    pub async fn get_with_fallback(
        &self,
        key: &str,
        user_id: Option<&str>,
        project_id: Option<&str>,
        env: Option<&str>,
    ) -> Result<Option<ConfigEntry>, ConfigRepoError> {
        // Try user scope first
        if let Some(uid) = user_id {
            if let Some(config) = self.get(key, ConfigScope::User, Some(uid)).await? {
                return Ok(Some(config));
            }
        }

        // Try project scope
        if let Some(pid) = project_id {
            if let Some(config) = self.get(key, ConfigScope::Project, Some(pid)).await? {
                return Ok(Some(config));
            }
        }

        // Try environment scope
        if let Some(e) = env {
            if let Some(config) = self.get(key, ConfigScope::Environment, Some(e)).await? {
                return Ok(Some(config));
            }
        }

        // Fall back to global
        self.get(key, ConfigScope::Global, None).await
    }

    /// Set a configuration value
    #[instrument(skip(self, input))]
    pub async fn set(&self, input: SetConfig) -> Result<ConfigEntry, ConfigRepoError> {
        // Check if exists
        if let Some(existing) = self.get(&input.key, input.scope, input.scope_id.as_deref()).await? {
            if existing.is_readonly {
                return Err(ConfigRepoError::ReadOnly(input.key));
            }
            return self.update_config(&existing.id, &input).await;
        }

        // Create new
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let value_type = input.value_type.unwrap_or_default();

        sqlx::query(r#"
            INSERT INTO config_entries (
                id, key, value, value_type, scope, scope_id,
                description, is_sensitive, is_readonly,
                created_at, updated_at, created_by, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&input.key)
        .bind(&input.value)
        .bind(value_type)
        .bind(input.scope)
        .bind(&input.scope_id)
        .bind(&input.description)
        .bind(input.is_sensitive.unwrap_or(false) as i32)
        .bind(input.is_readonly.unwrap_or(false) as i32)
        .bind(now)
        .bind(now)
        .bind(&input.changed_by)
        .bind(&input.changed_by)
        .execute(&self.pool)
        .await?;

        debug!("Created config entry: {}", input.key);

        self.get(&input.key, input.scope, input.scope_id.as_deref()).await?
            .ok_or(ConfigRepoError::NotFound(input.key))
    }

    /// Update existing configuration
    async fn update_config(&self, id: &str, input: &SetConfig) -> Result<ConfigEntry, ConfigRepoError> {
        sqlx::query(r#"
            UPDATE config_entries
            SET value = ?, updated_by = ?, description = COALESCE(?, description)
            WHERE id = ?
        "#)
        .bind(&input.value)
        .bind(&input.changed_by)
        .bind(&input.description)
        .bind(id)
        .execute(&self.pool)
        .await?;

        // Record reason if provided
        if let Some(reason) = &input.change_reason {
            sqlx::query(r#"
                UPDATE config_history
                SET change_reason = ?
                WHERE config_id = ?
                ORDER BY changed_at DESC
                LIMIT 1
            "#)
            .bind(reason)
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        debug!("Updated config entry: {}", input.key);

        self.get(&input.key, input.scope, input.scope_id.as_deref()).await?
            .ok_or(ConfigRepoError::NotFound(input.key.clone()))
    }

    /// Delete a configuration entry
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str, scope: ConfigScope, scope_id: Option<&str>) -> Result<bool, ConfigRepoError> {
        // Check if readonly
        if let Some(config) = self.get(key, scope, scope_id).await? {
            if config.is_readonly {
                return Err(ConfigRepoError::ReadOnly(key.to_string()));
            }
        }

        let result = sqlx::query(r#"
            DELETE FROM config_entries
            WHERE key = ? AND scope = ? AND (scope_id = ? OR (scope_id IS NULL AND ? IS NULL))
        "#)
        .bind(key)
        .bind(scope)
        .bind(scope_id)
        .bind(scope_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List all configurations for a scope
    pub async fn list_by_scope(
        &self,
        scope: ConfigScope,
        scope_id: Option<&str>,
    ) -> Result<Vec<ConfigEntry>, ConfigRepoError> {
        let configs = sqlx::query_as::<_, ConfigEntry>(r#"
            SELECT * FROM config_entries
            WHERE scope = ? AND (scope_id = ? OR (scope_id IS NULL AND ? IS NULL))
            ORDER BY key
        "#)
        .bind(scope)
        .bind(scope_id)
        .bind(scope_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(configs)
    }

    /// Get all configurations as a map
    pub async fn get_all_as_map(
        &self,
        user_id: Option<&str>,
        project_id: Option<&str>,
        env: Option<&str>,
    ) -> Result<HashMap<String, String>, ConfigRepoError> {
        let mut map = HashMap::new();

        // Get global configs
        for config in self.list_by_scope(ConfigScope::Global, None).await? {
            map.insert(config.key, config.value);
        }

        // Override with environment configs
        if let Some(e) = env {
            for config in self.list_by_scope(ConfigScope::Environment, Some(e)).await? {
                map.insert(config.key, config.value);
            }
        }

        // Override with project configs
        if let Some(pid) = project_id {
            for config in self.list_by_scope(ConfigScope::Project, Some(pid)).await? {
                map.insert(config.key, config.value);
            }
        }

        // Override with user configs
        if let Some(uid) = user_id {
            for config in self.list_by_scope(ConfigScope::User, Some(uid)).await? {
                map.insert(config.key, config.value);
            }
        }

        Ok(map)
    }

    /// Get configuration history
    pub async fn get_history(&self, config_id: &str, limit: i64) -> Result<Vec<ConfigHistory>, ConfigRepoError> {
        let history = sqlx::query_as::<_, ConfigHistory>(r#"
            SELECT * FROM config_history
            WHERE config_id = ?
            ORDER BY changed_at DESC
            LIMIT ?
        "#)
        .bind(config_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    // ==================== Feature Flag Methods ====================

    /// Get a feature flag by name
    #[instrument(skip(self))]
    pub async fn get_flag(&self, name: &str) -> Result<Option<FeatureFlag>, ConfigRepoError> {
        let flag = sqlx::query_as::<_, FeatureFlag>(
            "SELECT * FROM feature_flags WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(flag)
    }

    /// Create a feature flag
    #[instrument(skip(self, input))]
    pub async fn create_flag(&self, input: CreateFeatureFlag) -> Result<FeatureFlag, ConfigRepoError> {
        // Check if exists
        if self.get_flag(&input.name).await?.is_some() {
            return Err(ConfigRepoError::AlreadyExists(input.name));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let conditions = input.conditions.map(|c| c.to_string());

        sqlx::query(r#"
            INSERT INTO feature_flags (
                id, name, description, enabled, rollout_percentage,
                conditions, created_at, updated_at, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.enabled as i32)
        .bind(input.rollout_percentage.unwrap_or(100))
        .bind(&conditions)
        .bind(now)
        .bind(now)
        .bind(input.expires_at)
        .execute(&self.pool)
        .await?;

        debug!("Created feature flag: {}", input.name);

        self.get_flag(&input.name).await?.ok_or(ConfigRepoError::NotFound(input.name))
    }

    /// Update a feature flag
    #[instrument(skip(self))]
    pub async fn update_flag(
        &self,
        name: &str,
        enabled: Option<bool>,
        rollout_percentage: Option<i32>,
        conditions: Option<serde_json::Value>,
    ) -> Result<FeatureFlag, ConfigRepoError> {
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(e) = enabled {
            updates.push("enabled = ?");
            bindings.push((e as i32).to_string());
        }

        if let Some(pct) = rollout_percentage {
            updates.push("rollout_percentage = ?");
            bindings.push(pct.to_string());
        }

        if let Some(cond) = conditions {
            updates.push("conditions = ?");
            bindings.push(cond.to_string());
        }

        if updates.is_empty() {
            return self.get_flag(name).await?.ok_or(ConfigRepoError::NotFound(name.to_string()));
        }

        let sql = format!(
            "UPDATE feature_flags SET {} WHERE name = ?",
            updates.join(", ")
        );

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(name);

        query.execute(&self.pool).await?;

        self.get_flag(name).await?.ok_or(ConfigRepoError::NotFound(name.to_string()))
    }

    /// Check if a feature flag is enabled
    #[instrument(skip(self, properties))]
    pub async fn is_flag_enabled(
        &self,
        name: &str,
        user_id: Option<&str>,
        properties: Option<&serde_json::Value>,
    ) -> Result<bool, ConfigRepoError> {
        let flag = match self.get_flag(name).await? {
            Some(f) => f,
            None => {
                warn!("Feature flag not found: {}", name);
                return Ok(false);
            }
        };

        // Check for override
        if let Some(uid) = user_id {
            if let Some(override_enabled) = self.get_flag_override(&flag.id, "user", uid).await? {
                return Ok(override_enabled);
            }
        }

        Ok(flag.is_enabled_for(user_id, properties))
    }

    /// Get flag override
    async fn get_flag_override(
        &self,
        flag_id: &str,
        override_type: &str,
        override_id: &str,
    ) -> Result<Option<bool>, ConfigRepoError> {
        let row: Option<(i32,)> = sqlx::query_as(r#"
            SELECT enabled FROM feature_flag_overrides
            WHERE flag_id = ? AND override_type = ? AND override_id = ?
        "#)
        .bind(flag_id)
        .bind(override_type)
        .bind(override_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.0 != 0))
    }

    /// Set flag override for user/project
    pub async fn set_flag_override(
        &self,
        flag_name: &str,
        override_type: &str,
        override_id: &str,
        enabled: bool,
    ) -> Result<(), ConfigRepoError> {
        let flag = self.get_flag(flag_name).await?
            .ok_or_else(|| ConfigRepoError::NotFound(flag_name.to_string()))?;

        let id = Uuid::new_v4().to_string();

        sqlx::query(r#"
            INSERT OR REPLACE INTO feature_flag_overrides (id, flag_id, override_type, override_id, enabled)
            VALUES (?, ?, ?, ?, ?)
        "#)
        .bind(&id)
        .bind(&flag.id)
        .bind(override_type)
        .bind(override_id)
        .bind(enabled as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// List all feature flags
    pub async fn list_flags(&self) -> Result<Vec<FeatureFlag>, ConfigRepoError> {
        let flags = sqlx::query_as::<_, FeatureFlag>(
            "SELECT * FROM feature_flags ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(flags)
    }

    /// Delete a feature flag
    pub async fn delete_flag(&self, name: &str) -> Result<bool, ConfigRepoError> {
        let result = sqlx::query("DELETE FROM feature_flags WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ==================== Environment Methods ====================

    /// Get environment by name
    pub async fn get_environment(&self, name: &str) -> Result<Option<Environment>, ConfigRepoError> {
        let env = sqlx::query_as::<_, Environment>(
            "SELECT * FROM environments WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(env)
    }

    /// List all environments
    pub async fn list_environments(&self) -> Result<Vec<Environment>, ConfigRepoError> {
        let envs = sqlx::query_as::<_, Environment>(
            "SELECT * FROM environments ORDER BY is_production, name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(envs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
```

## Files to Create
- `src/database/repository/config.rs` - Configuration repository
