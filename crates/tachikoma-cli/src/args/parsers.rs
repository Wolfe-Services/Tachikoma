//! Custom value parsers for CLI arguments.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

/// Parse a duration from seconds string
pub fn parse_duration_secs(s: &str) -> Result<Duration, String> {
    let secs: u64 = s
        .parse()
        .map_err(|_| format!("Invalid duration: {s}"))?;
    Ok(Duration::from_secs(secs))
}

/// Parse a duration with unit suffix (e.g., "30s", "5m", "1h")
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();

    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let (num, unit) = if s.ends_with("ms") {
        (&s[..s.len() - 2], "ms")
    } else if s.ends_with('s') {
        (&s[..s.len() - 1], "s")
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], "m")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else if s.ends_with('d') {
        (&s[..s.len() - 1], "d")
    } else {
        (s, "s") // Default to seconds
    };

    let value: u64 = num
        .parse()
        .map_err(|_| format!("Invalid number: {num}"))?;

    let millis = match unit {
        "ms" => value,
        "s" => value * 1000,
        "m" => value * 60 * 1000,
        "h" => value * 60 * 60 * 1000,
        "d" => value * 24 * 60 * 60 * 1000,
        _ => return Err(format!("Unknown unit: {unit}")),
    };

    Ok(Duration::from_millis(millis))
}

/// Parse a key=value pair
pub fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key=value pair: {s}"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// Parse a URL with validation
pub fn parse_url(s: &str) -> Result<url::Url, String> {
    url::Url::parse(s).map_err(|e| format!("Invalid URL: {e}"))
}

/// Parse a semantic version
pub fn parse_semver(s: &str) -> Result<semver::Version, String> {
    semver::Version::parse(s).map_err(|e| format!("Invalid version: {e}"))
}

/// Parse a glob pattern
pub fn parse_glob(s: &str) -> Result<glob::Pattern, String> {
    glob::Pattern::new(s).map_err(|e| format!("Invalid glob pattern: {e}"))
}

/// Parse a size with unit suffix (e.g., "100KB", "1MB", "1GB")
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num, multiplier) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1)
    } else {
        (s.as_str(), 1)
    };

    let value: u64 = num
        .trim()
        .parse()
        .map_err(|_| format!("Invalid size: {s}"))?;

    Ok(value * multiplier)
}

/// Parse a list of items separated by comma
pub fn parse_comma_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parser for model identifiers
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelId {
    pub provider: String,
    pub model: String,
}

impl FromStr for ModelId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('/') {
            Some((provider, model)) => Ok(ModelId {
                provider: provider.to_string(),
                model: model.to_string(),
            }),
            None => Ok(ModelId {
                provider: "default".to_string(),
                model: s.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.provider, self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
    }

    #[test]
    fn test_parse_key_value() {
        let (k, v) = parse_key_value("key=value").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "value");

        let (k, v) = parse_key_value("key=val=ue").unwrap();
        assert_eq!(k, "key");
        assert_eq!(v, "val=ue");

        assert!(parse_key_value("noequals").is_err());
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("100B").unwrap(), 100);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_model_id_parsing() {
        let id: ModelId = "anthropic/claude-3".parse().unwrap();
        assert_eq!(id.provider, "anthropic");
        assert_eq!(id.model, "claude-3");

        let id: ModelId = "gpt-4".parse().unwrap();
        assert_eq!(id.provider, "default");
        assert_eq!(id.model, "gpt-4");
    }

    #[test]
    fn test_parse_comma_list() {
        assert_eq!(parse_comma_list("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(parse_comma_list("a, b, c "), vec!["a", "b", "c"]);
        assert_eq!(parse_comma_list(""), Vec::<String>::new());
        assert_eq!(parse_comma_list("single"), vec!["single"]);
    }

    #[test]
    fn test_parse_url() {
        assert!(parse_url("https://example.com").is_ok());
        assert!(parse_url("http://localhost:8080").is_ok());
        assert!(parse_url("invalid-url").is_err());
    }

    #[test]
    fn test_parse_semver() {
        assert!(parse_semver("1.0.0").is_ok());
        assert!(parse_semver("1.2.3-alpha").is_ok());
        assert!(parse_semver("invalid").is_err());
    }

    #[test]
    fn test_parse_glob() {
        assert!(parse_glob("*.rs").is_ok());
        assert!(parse_glob("**/*.toml").is_ok());
        assert!(parse_glob("[invalid").is_err());
    }
}