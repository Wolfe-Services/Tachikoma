//! Argument validators.

use std::path::Path;

/// Validate that a path exists
pub fn validate_path_exists(path: &str) -> Result<(), String> {
    if Path::new(path).exists() {
        Ok(())
    } else {
        Err(format!("Path does not exist: {path}"))
    }
}

/// Validate that a path is a directory
pub fn validate_is_directory(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_dir() {
        Ok(())
    } else if p.exists() {
        Err(format!("Path is not a directory: {path}"))
    } else {
        Err(format!("Directory does not exist: {path}"))
    }
}

/// Validate that a path is a file
pub fn validate_is_file(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_file() {
        Ok(())
    } else if p.exists() {
        Err(format!("Path is not a file: {path}"))
    } else {
        Err(format!("File does not exist: {path}"))
    }
}

/// Validate a port number
pub fn validate_port(s: &str) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| format!("Invalid port: {s}"))?;
    if port == 0 {
        Err("Port cannot be 0".to_string())
    } else {
        Ok(port)
    }
}

/// Validate an identifier (alphanumeric + underscore + hyphen)
pub fn validate_identifier(s: &str) -> Result<String, String> {
    if s.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if !s.chars().next().unwrap().is_alphabetic() {
        return Err("Identifier must start with a letter".to_string());
    }

    if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        Ok(s.to_string())
    } else {
        Err("Identifier can only contain letters, numbers, underscores, and hyphens".to_string())
    }
}

/// Validate a semantic version string
pub fn validate_semver(s: &str) -> Result<String, String> {
    semver::Version::parse(s)
        .map(|_| s.to_string())
        .map_err(|e| format!("Invalid semantic version: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port() {
        assert_eq!(validate_port("8080").unwrap(), 8080);
        assert_eq!(validate_port("80").unwrap(), 80);
        assert_eq!(validate_port("65535").unwrap(), 65535);
        assert!(validate_port("0").is_err());
        assert!(validate_port("65536").is_err());
        assert!(validate_port("invalid").is_err());
    }

    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("valid_name").is_ok());
        assert!(validate_identifier("valid-name").is_ok());
        assert!(validate_identifier("ValidName123").is_ok());
        assert!(validate_identifier("123invalid").is_err());
        assert!(validate_identifier("").is_err());
        assert!(validate_identifier("invalid name").is_err());
    }

    #[test]
    fn test_validate_semver() {
        assert!(validate_semver("1.0.0").is_ok());
        assert!(validate_semver("1.2.3-alpha").is_ok());
        assert!(validate_semver("2.0.0-beta.1").is_ok());
        assert!(validate_semver("invalid").is_err());
        assert!(validate_semver("1.0").is_err());
    }
}