# 017 - Secret Types

**Phase:** 1 - Core Common Crates
**Spec ID:** 017
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement secure secret handling types that prevent accidental logging or serialization of sensitive values like API keys and tokens.

---

## Acceptance Criteria

- [ ] Secret wrapper type that redacts on Display/Debug
- [ ] Explicit access required for inner value
- [ ] Zeroize on drop for memory safety
- [ ] Integration with serde (custom serializer)
- [ ] PII detection utilities

---

## Implementation Details

### 1. Secret Types (crates/tachikoma-common-secret/src/lib.rs)

```rust
//! Secure secret handling.
//!
//! This module provides types for handling sensitive values like API keys,
//! tokens, and PII that should never be accidentally logged or serialized.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secret value that is redacted in logs and debug output.
///
/// # Example
///
/// ```rust
/// use tachikoma_common_secret::Secret;
///
/// let api_key = Secret::new("sk-abc123".to_string());
/// println!("{}", api_key); // Prints: [REDACTED]
/// println!("{:?}", api_key); // Prints: Secret([REDACTED])
///
/// // Explicit access required
/// let value = api_key.expose();
/// assert_eq!(value, "sk-abc123");
/// ```
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Create a new secret.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Expose the secret value.
    ///
    /// Use this method sparingly and only when necessary.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Expose the secret value mutably.
    pub fn expose_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Consume and return the inner value.
    pub fn into_inner(self) -> T {
        // Note: Zeroize won't run since we're moving out
        let mut this = std::mem::ManuallyDrop::new(self);
        unsafe { std::ptr::read(&this.0) }
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

impl<T: Zeroize + Default> Default for Secret<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Zeroize + PartialEq> PartialEq for Secret<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// Serde: Deserialize normally, but serialize as redacted
impl<'de, T: Zeroize + Deserialize<'de>> Deserialize<'de> for Secret<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Secret::new)
    }
}

impl<T: Zeroize + Serialize> Serialize for Secret<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize as redacted
        "[REDACTED]".serialize(serializer)
    }
}

/// Type alias for a secret string.
pub type SecretString = Secret<String>;

/// PII detection patterns.
pub struct PiiDetector;

impl PiiDetector {
    /// Check if a string might contain an API key.
    pub fn looks_like_api_key(s: &str) -> bool {
        // Common API key patterns
        let patterns = [
            "sk-",      // OpenAI, Anthropic
            "pk-",      // Public keys
            "api_",     // Generic
            "key-",     // Generic
            "token-",   // Generic
            "bearer ",  // Auth headers
        ];

        let lower = s.to_lowercase();
        patterns.iter().any(|p| lower.contains(p))
            || (s.len() >= 32 && s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
    }

    /// Check if a string might contain an email.
    pub fn looks_like_email(s: &str) -> bool {
        s.contains('@') && s.contains('.')
    }

    /// Redact potential PII from a string.
    pub fn redact(s: &str) -> String {
        let mut result = s.to_string();

        // Redact emails
        let email_re = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .unwrap();
        result = email_re.replace_all(&result, "[EMAIL]").to_string();

        // Redact API keys (simple heuristic)
        let key_re = regex::Regex::new(r"(sk|pk|api|key|token)-[a-zA-Z0-9]{20,}")
            .unwrap();
        result = key_re.replace_all(&result, "[API_KEY]").to_string();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_display_is_redacted() {
        let secret = SecretString::new("my-api-key".to_string());
        assert_eq!(format!("{}", secret), "[REDACTED]");
    }

    #[test]
    fn test_secret_debug_is_redacted() {
        let secret = SecretString::new("my-api-key".to_string());
        assert_eq!(format!("{:?}", secret), "Secret([REDACTED])");
    }

    #[test]
    fn test_secret_expose() {
        let secret = SecretString::new("my-api-key".to_string());
        assert_eq!(secret.expose(), "my-api-key");
    }

    #[test]
    fn test_pii_api_key_detection() {
        assert!(PiiDetector::looks_like_api_key("sk-abc123def456"));
        assert!(PiiDetector::looks_like_api_key("api_key_12345"));
        assert!(!PiiDetector::looks_like_api_key("hello world"));
    }

    #[test]
    fn test_pii_redaction() {
        let input = "Contact user@example.com with key sk-abc123def456789012345678";
        let redacted = PiiDetector::redact(input);
        assert!(!redacted.contains("user@example.com"));
        assert!(!redacted.contains("sk-abc123"));
    }
}
```

### 2. Crate Setup (crates/tachikoma-common-secret/Cargo.toml)

```toml
[package]
name = "tachikoma-common-secret"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Secure secret handling for Tachikoma"

[dependencies]
serde = { workspace = true, features = ["derive"] }
zeroize = { version = "1.7", features = ["derive"] }
regex = "1.10"
```

---

## Testing Requirements

1. Display and Debug never expose value
2. Memory is zeroed on drop
3. PII detection catches common patterns
4. Serialization produces redacted output

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [018-thread-utilities.md](018-thread-utilities.md)
