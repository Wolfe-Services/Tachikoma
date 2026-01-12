# 016 - Environment Variables

**Phase:** 1 - Core Common Crates
**Spec ID:** 016
**Status:** Planned
**Dependencies:** 014-config-core-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement environment variable handling with dotenv support, typed access, and secure handling of sensitive values.

---

## Acceptance Criteria

- [ ] Load `.env` files (dev, prod)
- [ ] Typed environment variable access
- [ ] Secure API key retrieval
- [ ] Validation of required variables
- [ ] Documentation of all env vars

---

## Implementation Details

### 1. Environment Module (crates/tachikoma-common-config/src/env.rs)

```rust
//! Environment variable handling.

use std::env;
use thiserror::Error;

/// Environment variable errors.
#[derive(Debug, Error)]
pub enum EnvError {
    #[error("required environment variable not set: {var}")]
    NotSet { var: String },

    #[error("invalid value for {var}: {message}")]
    InvalidValue { var: String, message: String },

    #[error("failed to load .env file: {0}")]
    DotenvError(#[from] dotenvy::Error),
}

/// Environment variable names.
pub mod vars {
    // API Keys
    pub const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
    pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
    pub const GOOGLE_API_KEY: &str = "GOOGLE_API_KEY";

    // Configuration
    pub const TACHIKOMA_CONFIG_PATH: &str = "TACHIKOMA_CONFIG_PATH";
    pub const TACHIKOMA_LOG_LEVEL: &str = "TACHIKOMA_LOG_LEVEL";
    pub const TACHIKOMA_DATA_DIR: &str = "TACHIKOMA_DATA_DIR";

    // Development
    pub const NODE_ENV: &str = "NODE_ENV";
    pub const RUST_LOG: &str = "RUST_LOG";
    pub const RUST_BACKTRACE: &str = "RUST_BACKTRACE";
}

/// Environment configuration.
pub struct Environment {
    _guard: (), // Prevent construction outside module
}

impl Environment {
    /// Initialize environment from .env files.
    pub fn init() -> Result<Self, EnvError> {
        // Load .env files in order (later overrides earlier)
        let _ = dotenvy::from_filename(".env");
        let _ = dotenvy::from_filename(".env.local");

        // Load environment-specific file
        if let Ok(env) = env::var(vars::NODE_ENV) {
            let _ = dotenvy::from_filename(format!(".env.{}", env));
        }

        Ok(Self { _guard: () })
    }

    /// Get a required string variable.
    pub fn require(var: &str) -> Result<String, EnvError> {
        env::var(var).map_err(|_| EnvError::NotSet { var: var.to_string() })
    }

    /// Get an optional string variable.
    pub fn get(var: &str) -> Option<String> {
        env::var(var).ok()
    }

    /// Get a variable with a default value.
    pub fn get_or(var: &str, default: &str) -> String {
        env::var(var).unwrap_or_else(|_| default.to_string())
    }

    /// Get a boolean variable.
    pub fn get_bool(var: &str) -> Option<bool> {
        env::var(var).ok().map(|v| {
            matches!(v.to_lowercase().as_str(), "true" | "1" | "yes")
        })
    }

    /// Get an integer variable.
    pub fn get_int<T: std::str::FromStr>(var: &str) -> Result<Option<T>, EnvError> {
        match env::var(var) {
            Ok(v) => v.parse().map(Some).map_err(|_| EnvError::InvalidValue {
                var: var.to_string(),
                message: "expected integer".to_string(),
            }),
            Err(_) => Ok(None),
        }
    }

    /// Check if running in development mode.
    pub fn is_development() -> bool {
        env::var(vars::NODE_ENV)
            .map(|v| v == "development")
            .unwrap_or(true)
    }

    /// Check if running in production mode.
    pub fn is_production() -> bool {
        env::var(vars::NODE_ENV)
            .map(|v| v == "production")
            .unwrap_or(false)
    }
}

/// API key manager with secure access.
pub struct ApiKeys;

impl ApiKeys {
    /// Get Anthropic API key.
    pub fn anthropic() -> Option<String> {
        Environment::get(vars::ANTHROPIC_API_KEY)
    }

    /// Get OpenAI API key.
    pub fn openai() -> Option<String> {
        Environment::get(vars::OPENAI_API_KEY)
    }

    /// Get Google API key.
    pub fn google() -> Option<String> {
        Environment::get(vars::GOOGLE_API_KEY)
    }

    /// Get API key for a specific backend.
    pub fn for_backend(backend: &str) -> Option<String> {
        match backend {
            "claude" | "anthropic" => Self::anthropic(),
            "codex" | "openai" => Self::openai(),
            "gemini" | "google" => Self::google(),
            _ => None,
        }
    }

    /// Validate all required API keys are set.
    pub fn validate_required(backends: &[&str]) -> Result<(), EnvError> {
        for backend in backends {
            if Self::for_backend(backend).is_none() {
                let var = match *backend {
                    "claude" | "anthropic" => vars::ANTHROPIC_API_KEY,
                    "codex" | "openai" => vars::OPENAI_API_KEY,
                    "gemini" | "google" => vars::GOOGLE_API_KEY,
                    _ => continue,
                };
                return Err(EnvError::NotSet { var: var.to_string() });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_default() {
        let val = Environment::get_or("NONEXISTENT_VAR_12345", "default");
        assert_eq!(val, "default");
    }

    #[test]
    fn test_bool_parsing() {
        env::set_var("TEST_BOOL", "true");
        assert_eq!(Environment::get_bool("TEST_BOOL"), Some(true));
        env::set_var("TEST_BOOL", "1");
        assert_eq!(Environment::get_bool("TEST_BOOL"), Some(true));
        env::set_var("TEST_BOOL", "false");
        assert_eq!(Environment::get_bool("TEST_BOOL"), Some(false));
        env::remove_var("TEST_BOOL");
    }
}
```

### 2. Add Dependencies

```toml
[dependencies]
dotenvy = "0.15"
```

---

## Testing Requirements

1. `.env` file is loaded when present
2. Missing required vars return clear error
3. Bool parsing handles various formats
4. API key lookup by backend name works

---

## Related Specs

- Depends on: [014-config-core-types.md](014-config-core-types.md)
- Next: [017-secret-types.md](017-secret-types.md)
