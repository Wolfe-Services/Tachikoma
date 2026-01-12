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

    #[test]
    fn test_api_key_backend_mapping() {
        // Test known backend mappings
        env::set_var(vars::ANTHROPIC_API_KEY, "test-claude-key");
        env::set_var(vars::OPENAI_API_KEY, "test-openai-key");
        env::set_var(vars::GOOGLE_API_KEY, "test-google-key");

        assert_eq!(ApiKeys::for_backend("claude"), Some("test-claude-key".to_string()));
        assert_eq!(ApiKeys::for_backend("anthropic"), Some("test-claude-key".to_string()));
        assert_eq!(ApiKeys::for_backend("codex"), Some("test-openai-key".to_string()));
        assert_eq!(ApiKeys::for_backend("openai"), Some("test-openai-key".to_string()));
        assert_eq!(ApiKeys::for_backend("gemini"), Some("test-google-key".to_string()));
        assert_eq!(ApiKeys::for_backend("google"), Some("test-google-key".to_string()));
        assert_eq!(ApiKeys::for_backend("unknown"), None);

        // Cleanup
        env::remove_var(vars::ANTHROPIC_API_KEY);
        env::remove_var(vars::OPENAI_API_KEY);
        env::remove_var(vars::GOOGLE_API_KEY);
    }

    #[test]
    fn test_required_validation() {
        // Clean slate
        env::remove_var(vars::ANTHROPIC_API_KEY);
        env::remove_var(vars::OPENAI_API_KEY);

        // Should fail with missing key
        let result = ApiKeys::validate_required(&["claude"]);
        assert!(result.is_err());
        
        // Set key and retry
        env::set_var(vars::ANTHROPIC_API_KEY, "test-key");
        let result = ApiKeys::validate_required(&["claude"]);
        assert!(result.is_ok());

        // Cleanup
        env::remove_var(vars::ANTHROPIC_API_KEY);
    }

    #[test]
    fn test_environment_mode_detection() {
        // Test default (development)
        env::remove_var(vars::NODE_ENV);
        assert!(Environment::is_development());
        assert!(!Environment::is_production());

        // Test explicit development
        env::set_var(vars::NODE_ENV, "development");
        assert!(Environment::is_development());
        assert!(!Environment::is_production());

        // Test production
        env::set_var(vars::NODE_ENV, "production");
        assert!(!Environment::is_development());
        assert!(Environment::is_production());

        // Cleanup
        env::remove_var(vars::NODE_ENV);
    }

    #[test]
    fn test_integer_parsing() {
        env::set_var("TEST_INT", "42");
        let val: Result<Option<i32>, _> = Environment::get_int("TEST_INT");
        assert_eq!(val.unwrap(), Some(42));

        env::set_var("TEST_INT", "invalid");
        let val: Result<Option<i32>, _> = Environment::get_int("TEST_INT");
        assert!(val.is_err());

        env::remove_var("TEST_INT");
        let val: Result<Option<i32>, _> = Environment::get_int("TEST_INT");
        assert_eq!(val.unwrap(), None);

        env::remove_var("TEST_INT");
    }

    #[test]
    fn test_environment_init() {
        // Test that Environment::init() doesn't fail even without .env files
        let result = Environment::init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_variable_names_are_defined() {
        // Ensure all constants are properly defined
        assert!(!vars::ANTHROPIC_API_KEY.is_empty());
        assert!(!vars::OPENAI_API_KEY.is_empty());
        assert!(!vars::GOOGLE_API_KEY.is_empty());
        assert!(!vars::TACHIKOMA_CONFIG_PATH.is_empty());
        assert!(!vars::TACHIKOMA_LOG_LEVEL.is_empty());
        assert!(!vars::TACHIKOMA_DATA_DIR.is_empty());
        assert!(!vars::NODE_ENV.is_empty());
        assert!(!vars::RUST_LOG.is_empty());
        assert!(!vars::RUST_BACKTRACE.is_empty());
    }

    #[test]
    fn test_dotenv_file_loading() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let env_path = dir.path().join(".env");
        
        // Create a test .env file
        fs::write(&env_path, "TEST_TACHIKOMA_VAR=from_dotenv\n").unwrap();
        
        // Save current directory and environment state
        let original_dir = std::env::current_dir().unwrap();
        let original_var = std::env::var("TEST_TACHIKOMA_VAR").ok();
        
        // Clear any existing value
        std::env::remove_var("TEST_TACHIKOMA_VAR");
        
        // Change to the temp directory and load env
        std::env::set_current_dir(dir.path()).unwrap();
        
        // Initialize environment (should load our .env file)
        let _env = Environment::init().unwrap();
        
        // Verify the variable was loaded
        assert_eq!(Environment::get("TEST_TACHIKOMA_VAR"), Some("from_dotenv".to_string()));
        
        // Restore original state
        std::env::set_current_dir(original_dir).unwrap();
        match original_var {
            Some(val) => std::env::set_var("TEST_TACHIKOMA_VAR", val),
            None => std::env::remove_var("TEST_TACHIKOMA_VAR"),
        }
    }
}