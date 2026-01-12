# 025 - i18n Locale Detection

**Phase:** 1 - Core Common Crates
**Spec ID:** 025
**Status:** Planned
**Dependencies:** 023-i18n-core-setup
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Implement automatic locale detection from system settings, environment variables, and user preferences.

---

## Acceptance Criteria

- [x] Detect from LANG/LC_* environment variables
- [x] Detect from system settings (platform-specific)
- [x] User preference override
- [x] Fallback chain handling

---

## Implementation Details

### 1. Locale Detection (crates/tachikoma-common-i18n/src/detect.rs)

```rust
//! Automatic locale detection.

use super::Locale;
use std::env;

/// Detect the system locale.
pub fn detect_locale() -> Locale {
    // Priority: explicit env var > LC_ALL > LC_MESSAGES > LANG > system > default

    if let Some(locale) = from_env("TACHIKOMA_LOCALE") {
        return locale;
    }

    if let Some(locale) = from_env("LC_ALL") {
        return locale;
    }

    if let Some(locale) = from_env("LC_MESSAGES") {
        return locale;
    }

    if let Some(locale) = from_env("LANG") {
        return locale;
    }

    // Platform-specific detection
    #[cfg(target_os = "macos")]
    if let Some(locale) = detect_macos() {
        return locale;
    }

    #[cfg(target_os = "windows")]
    if let Some(locale) = detect_windows() {
        return locale;
    }

    Locale::default()
}

/// Parse locale from environment variable.
fn from_env(var: &str) -> Option<Locale> {
    env::var(var).ok().and_then(|v| Locale::parse(&v))
}

/// Detect locale on macOS using defaults.
#[cfg(target_os = "macos")]
fn detect_macos() -> Option<Locale> {
    use std::process::Command;

    let output = Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;

    if output.status.success() {
        let locale_str = String::from_utf8_lossy(&output.stdout);
        return Locale::parse(locale_str.trim());
    }

    None
}

/// Detect locale on Windows.
#[cfg(target_os = "windows")]
fn detect_windows() -> Option<Locale> {
    // Use GetUserDefaultLocaleName or similar
    // Simplified - real implementation would use Windows API
    None
}

/// Fallback chain for translations.
pub fn locale_fallback_chain(locale: Locale) -> Vec<Locale> {
    let mut chain = vec![locale];

    // Add regional fallbacks
    match locale {
        Locale::ZhCn => {
            // Chinese simplified has no further fallback before English
        }
        _ => {}
    }

    // Always fall back to English
    if locale != Locale::En {
        chain.push(Locale::En);
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_chain() {
        let chain = locale_fallback_chain(Locale::Ja);
        assert_eq!(chain, vec![Locale::Ja, Locale::En]);
    }

    #[test]
    fn test_english_fallback() {
        let chain = locale_fallback_chain(Locale::En);
        assert_eq!(chain, vec![Locale::En]);
    }
}
```

---

## Testing Requirements

1. Environment variables are detected
2. Fallback chain includes English
3. Invalid locales handled gracefully
4. Platform detection doesn't crash

---

## Related Specs

- Depends on: [023-i18n-core-setup.md](023-i18n-core-setup.md)
- Next: [026-logging-infrastructure.md](026-logging-infrastructure.md)
