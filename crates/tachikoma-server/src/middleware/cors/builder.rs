//! CORS configuration builder.

use super::config::{AllowedHeaders, AllowedOrigins, CorsConfig};
use std::collections::HashSet;
use std::time::Duration;

/// Builder for CORS configuration.
pub struct CorsBuilder {
    config: CorsConfig,
}

impl CorsBuilder {
    pub fn new() -> Self {
        Self {
            config: CorsConfig::default(),
        }
    }

    /// Allow any origin.
    pub fn allow_any_origin(mut self) -> Self {
        self.config.allowed_origins = AllowedOrigins::Any;
        self
    }

    /// Allow specific origins.
    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_origins = AllowedOrigins::List(
            origins.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Allow origins matching regex patterns.
    pub fn allow_origin_regex<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let regexes: Vec<regex::Regex> = patterns
            .into_iter()
            .filter_map(|p| regex::Regex::new(p.as_ref()).ok())
            .collect();
        self.config.allowed_origins = AllowedOrigins::Regex(regexes);
        self
    }

    /// Set allowed methods.
    pub fn allow_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_methods = methods.into_iter().map(Into::into).collect();
        self
    }

    /// Allow any headers.
    pub fn allow_any_header(mut self) -> Self {
        self.config.allowed_headers = AllowedHeaders::Any;
        self
    }

    /// Allow specific headers.
    pub fn allow_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.allowed_headers = AllowedHeaders::List(
            headers.into_iter().map(Into::into).collect(),
        );
        self
    }

    /// Set exposed headers.
    pub fn expose_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.exposed_headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Allow credentials.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.config.allow_credentials = allow;
        self
    }

    /// Set max age for preflight cache.
    pub fn max_age(mut self, duration: Duration) -> Self {
        self.config.max_age = Some(duration);
        self
    }

    /// Build the CORS configuration.
    pub fn build(self) -> CorsConfig {
        self.config
    }
}

impl Default for CorsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = CorsBuilder::new()
            .allow_origins(["https://example.com", "https://app.example.com"])
            .allow_methods(["GET", "POST"])
            .allow_credentials(true)
            .max_age(Duration::from_secs(3600))
            .build();

        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("https://other.com"));
        assert!(config.allow_credentials);
    }
}