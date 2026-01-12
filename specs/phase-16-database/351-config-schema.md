# Spec 351: Configuration Database Schema

## Overview
Define the SQLite schema for storing application configuration, including settings, feature flags, and environment-specific values.

## Rust Implementation

### Schema Models
```rust
// src/database/schema/config.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Configuration value type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum ConfigValueType {
    String,
    Integer,
    Float,
    Boolean,
    Json,
    Secret,
}

impl Default for ConfigValueType {
    fn default() -> Self {
        Self::String
    }
}

/// Configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum ConfigScope {
    Global,
    User,
    Project,
    Environment,
}

impl Default for ConfigScope {
    fn default() -> Self {
        Self::Global
    }
}

/// Configuration entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub id: String,
    pub key: String,
    pub value: String,
    pub value_type: ConfigValueType,
    pub scope: ConfigScope,
    pub scope_id: Option<String>,  // user_id, project_id, or env name
    pub description: Option<String>,
    pub is_sensitive: bool,
    pub is_readonly: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
}

/// Feature flag
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub rollout_percentage: i32,  // 0-100
    pub conditions: Option<String>,  // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Configuration history
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ConfigHistory {
    pub id: String,
    pub config_id: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub change_reason: Option<String>,
}

/// Environment configuration
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_production: bool,
    pub parent_env: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

### Migration SQL
```rust
// src/database/migrations/004_create_config.rs

use crate::database::migration::Migration;

pub fn migration() -> Migration {
    Migration::new(
        20240101000004,
        "create_config",
        r#"
-- Configuration entries
CREATE TABLE IF NOT EXISTS config_entries (
    id TEXT PRIMARY KEY NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    value_type TEXT NOT NULL DEFAULT 'string'
        CHECK (value_type IN ('string', 'integer', 'float', 'boolean', 'json', 'secret')),
    scope TEXT NOT NULL DEFAULT 'global'
        CHECK (scope IN ('global', 'user', 'project', 'environment')),
    scope_id TEXT,
    description TEXT,
    is_sensitive INTEGER NOT NULL DEFAULT 0,
    is_readonly INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_by TEXT,
    updated_by TEXT,
    UNIQUE(key, scope, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_config_key ON config_entries(key);
CREATE INDEX IF NOT EXISTS idx_config_scope ON config_entries(scope, scope_id);

-- Feature flags
CREATE TABLE IF NOT EXISTS feature_flags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    rollout_percentage INTEGER NOT NULL DEFAULT 100 CHECK (rollout_percentage >= 0 AND rollout_percentage <= 100),
    conditions TEXT,  -- JSON
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_feature_flags_name ON feature_flags(name);
CREATE INDEX IF NOT EXISTS idx_feature_flags_enabled ON feature_flags(enabled);

-- Feature flag overrides per user/project
CREATE TABLE IF NOT EXISTS feature_flag_overrides (
    id TEXT PRIMARY KEY NOT NULL,
    flag_id TEXT NOT NULL REFERENCES feature_flags(id) ON DELETE CASCADE,
    override_type TEXT NOT NULL CHECK (override_type IN ('user', 'project', 'group')),
    override_id TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(flag_id, override_type, override_id)
);

CREATE INDEX IF NOT EXISTS idx_flag_overrides_flag ON feature_flag_overrides(flag_id);

-- Configuration history
CREATE TABLE IF NOT EXISTS config_history (
    id TEXT PRIMARY KEY NOT NULL,
    config_id TEXT NOT NULL REFERENCES config_entries(id) ON DELETE CASCADE,
    old_value TEXT,
    new_value TEXT NOT NULL,
    changed_by TEXT,
    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    change_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_config_history_config ON config_history(config_id);
CREATE INDEX IF NOT EXISTS idx_config_history_changed_at ON config_history(changed_at);

-- Environments
CREATE TABLE IF NOT EXISTS environments (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    is_production INTEGER NOT NULL DEFAULT 0,
    parent_env TEXT REFERENCES environments(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Default environments
INSERT OR IGNORE INTO environments (id, name, display_name, is_production) VALUES
    ('env-dev', 'development', 'Development', 0),
    ('env-staging', 'staging', 'Staging', 0),
    ('env-prod', 'production', 'Production', 1);

-- Update timestamp trigger
CREATE TRIGGER IF NOT EXISTS update_config_timestamp
AFTER UPDATE ON config_entries
BEGIN
    UPDATE config_entries SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- History trigger
CREATE TRIGGER IF NOT EXISTS config_history_trigger
AFTER UPDATE OF value ON config_entries
BEGIN
    INSERT INTO config_history (id, config_id, old_value, new_value, changed_by)
    VALUES (
        lower(hex(randomblob(16))),
        NEW.id,
        OLD.value,
        NEW.value,
        NEW.updated_by
    );
END;

-- Feature flag update timestamp
CREATE TRIGGER IF NOT EXISTS update_feature_flag_timestamp
AFTER UPDATE ON feature_flags
BEGIN
    UPDATE feature_flags SET updated_at = datetime('now') WHERE id = NEW.id;
END;
"#
    ).with_down(r#"
DROP TRIGGER IF EXISTS update_feature_flag_timestamp;
DROP TRIGGER IF EXISTS config_history_trigger;
DROP TRIGGER IF EXISTS update_config_timestamp;
DROP TABLE IF EXISTS environments;
DROP TABLE IF EXISTS config_history;
DROP TABLE IF EXISTS feature_flag_overrides;
DROP TABLE IF EXISTS feature_flags;
DROP TABLE IF EXISTS config_entries;
"#)
}
```

### Config Value Helpers
```rust
// src/database/schema/config_helpers.rs

use super::config::*;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigValueError {
    #[error("Type mismatch: expected {expected:?}, got {actual:?}")]
    TypeMismatch { expected: ConfigValueType, actual: ConfigValueType },

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl ConfigEntry {
    /// Get value as string
    pub fn as_string(&self) -> Result<&str, ConfigValueError> {
        Ok(&self.value)
    }

    /// Get value as integer
    pub fn as_int(&self) -> Result<i64, ConfigValueError> {
        self.value.parse().map_err(|_| {
            ConfigValueError::ParseError(format!("Cannot parse '{}' as integer", self.value))
        })
    }

    /// Get value as float
    pub fn as_float(&self) -> Result<f64, ConfigValueError> {
        self.value.parse().map_err(|_| {
            ConfigValueError::ParseError(format!("Cannot parse '{}' as float", self.value))
        })
    }

    /// Get value as boolean
    pub fn as_bool(&self) -> Result<bool, ConfigValueError> {
        match self.value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ConfigValueError::ParseError(
                format!("Cannot parse '{}' as boolean", self.value)
            )),
        }
    }

    /// Get value as JSON
    pub fn as_json(&self) -> Result<Value, ConfigValueError> {
        serde_json::from_str(&self.value).map_err(|e| {
            ConfigValueError::ParseError(format!("Invalid JSON: {}", e))
        })
    }

    /// Create a typed config value
    pub fn typed_value<T: serde::de::DeserializeOwned>(&self) -> Result<T, ConfigValueError> {
        serde_json::from_str(&self.value).map_err(|e| {
            ConfigValueError::ParseError(format!("Deserialization error: {}", e))
        })
    }

    /// Check if value should be masked in logs
    pub fn should_mask(&self) -> bool {
        self.is_sensitive || self.value_type == ConfigValueType::Secret
    }

    /// Get masked value for display
    pub fn masked_value(&self) -> String {
        if self.should_mask() {
            "********".to_string()
        } else {
            self.value.clone()
        }
    }
}

impl FeatureFlag {
    /// Check if flag is enabled for a given user/context
    pub fn is_enabled_for(&self, user_id: Option<&str>, properties: Option<&Value>) -> bool {
        if !self.enabled {
            return false;
        }

        // Check expiration
        if let Some(expires) = &self.expires_at {
            if expires < &chrono::Utc::now() {
                return false;
            }
        }

        // Check rollout percentage
        if self.rollout_percentage < 100 {
            if let Some(uid) = user_id {
                // Use consistent hashing for rollout
                let hash = Self::hash_user(uid, &self.name);
                if (hash % 100) >= self.rollout_percentage as u64 {
                    return false;
                }
            } else {
                // No user context, use percentage as probability
                return rand::random::<u8>() as i32 <= self.rollout_percentage;
            }
        }

        // Check conditions
        if let Some(conditions) = &self.conditions {
            if let Ok(cond) = serde_json::from_str::<Value>(conditions) {
                return Self::evaluate_conditions(&cond, properties);
            }
        }

        true
    }

    fn hash_user(user_id: &str, flag_name: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        flag_name.hash(&mut hasher);
        hasher.finish()
    }

    fn evaluate_conditions(conditions: &Value, properties: Option<&Value>) -> bool {
        // Simple condition evaluation
        // Could be extended to support complex rules
        if let Some(props) = properties {
            if let Some(obj) = conditions.as_object() {
                for (key, expected) in obj {
                    if let Some(actual) = props.get(key) {
                        if actual != expected {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_config(value: &str, value_type: ConfigValueType) -> ConfigEntry {
        ConfigEntry {
            id: "test".to_string(),
            key: "test.key".to_string(),
            value: value.to_string(),
            value_type,
            scope: ConfigScope::Global,
            scope_id: None,
            description: None,
            is_sensitive: false,
            is_readonly: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            updated_by: None,
        }
    }

    #[test]
    fn test_as_int() {
        let config = test_config("42", ConfigValueType::Integer);
        assert_eq!(config.as_int().unwrap(), 42);
    }

    #[test]
    fn test_as_bool() {
        let config = test_config("true", ConfigValueType::Boolean);
        assert!(config.as_bool().unwrap());

        let config = test_config("0", ConfigValueType::Boolean);
        assert!(!config.as_bool().unwrap());
    }

    #[test]
    fn test_masked_value() {
        let mut config = test_config("secret123", ConfigValueType::Secret);
        config.is_sensitive = true;
        assert_eq!(config.masked_value(), "********");
    }
}
```

## Schema Design Decisions

1. **Scoped Configuration**: Support global, user, project, and environment scopes
2. **Type Safety**: Store value type for proper parsing
3. **Sensitive Data**: Mark sensitive values for masking
4. **Feature Flags**: Rollout percentages and conditions
5. **Audit Trail**: Full history of configuration changes

## Files to Create
- `src/database/schema/config.rs` - Configuration models
- `src/database/schema/config_helpers.rs` - Helper methods
- `src/database/migrations/004_create_config.rs` - Migration
