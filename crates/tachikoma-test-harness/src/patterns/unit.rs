//! Unit test patterns and conventions for Rust code.
//!
//! ## Naming Convention
//! - `test_<function>_<scenario>_<expected_outcome>`
//! - Example: `test_parse_config_empty_file_returns_default`
//!
//! ## Structure
//! - Use `mod tests` within each module
//! - Group related tests with nested modules
//! - Use `rstest` for parameterized tests

/// Marker trait for test builders
pub trait TestBuilder {
    type Output;
    fn build(self) -> Self::Output;
}

/// Example test builder pattern
#[derive(Default, Clone)]
pub struct ConfigBuilder {
    pub name: Option<String>,
    pub enabled: bool,
    pub max_retries: u32,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Example domain object for demonstration
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub name: String,
    pub enabled: bool,
    pub max_retries: u32,
}

impl TestBuilder for ConfigBuilder {
    type Output = Config;

    fn build(self) -> Config {
        Config {
            name: self.name.unwrap_or_else(|| "default".into()),
            enabled: self.enabled,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use pretty_assertions::assert_eq;

    // ============================================
    // Pattern: Basic Arrange-Act-Assert
    // ============================================

    #[test]
    fn test_config_builder_default_values_are_sensible() {
        // Arrange
        let builder = ConfigBuilder::new();

        // Act
        let config = builder.build();

        // Assert
        assert_eq!(config.name, "default");
        assert!(!config.enabled);
        assert_eq!(config.max_retries, 0);
    }

    // ============================================
    // Pattern: Parameterized Tests with rstest
    // ============================================

    #[rstest]
    #[case("production", true, 3)]
    #[case("staging", true, 2)]
    #[case("development", false, 0)]
    fn test_config_builder_respects_all_fields(
        #[case] name: &str,
        #[case] enabled: bool,
        #[case] max_retries: u32,
    ) {
        // Arrange & Act
        let config = ConfigBuilder::new()
            .name(name)
            .enabled(enabled)
            .max_retries(max_retries)
            .build();

        // Assert
        assert_eq!(config.name, name);
        assert_eq!(config.enabled, enabled);
        assert_eq!(config.max_retries, max_retries);
    }

    // ============================================
    // Pattern: Grouped Tests by Feature
    // ============================================

    mod name_handling {
        use super::*;

        #[test]
        fn test_name_can_be_set() {
            let config = ConfigBuilder::new().name("custom").build();
            assert_eq!(config.name, "custom");
        }

        #[test]
        fn test_name_accepts_string_types() {
            let string_owned = String::from("owned");
            let config = ConfigBuilder::new().name(string_owned).build();
            assert_eq!(config.name, "owned");
        }
    }

    mod retry_handling {
        use super::*;

        #[test]
        fn test_max_retries_defaults_to_zero() {
            let config = ConfigBuilder::new().build();
            assert_eq!(config.max_retries, 0);
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(100)]
        #[case(u32::MAX)]
        fn test_max_retries_accepts_valid_values(#[case] retries: u32) {
            let config = ConfigBuilder::new().max_retries(retries).build();
            assert_eq!(config.max_retries, retries);
        }
    }
}