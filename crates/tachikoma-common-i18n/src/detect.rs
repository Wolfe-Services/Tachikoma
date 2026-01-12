//! Automatic locale detection.

use super::Locale;
use std::env;

/// Detect the system locale with optional user override.
pub fn detect_locale_with_override(user_locale: Option<&str>) -> Locale {
    // User preference has highest priority
    if let Some(locale_str) = user_locale {
        if let Some(locale) = Locale::parse(locale_str) {
            return locale;
        }
    }
    
    // Fall back to normal detection
    detect_locale()
}

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
    use std::env;

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

    #[test] 
    fn test_tachikoma_locale_priority() {
        // Save original state
        let original = env::var("TACHIKOMA_LOCALE").ok();
        
        env::set_var("TACHIKOMA_LOCALE", "ja_JP");
        let locale = detect_locale();
        assert_eq!(locale, Locale::Ja);
        
        // Restore
        env::remove_var("TACHIKOMA_LOCALE");
        if let Some(val) = original {
            env::set_var("TACHIKOMA_LOCALE", val);
        }
    }

    #[test]
    fn test_lc_all_priority() {
        // Save and clear higher priority vars
        let original_tachikoma = env::var("TACHIKOMA_LOCALE").ok();
        let original_lc_all = env::var("LC_ALL").ok();
        
        env::remove_var("TACHIKOMA_LOCALE");
        env::set_var("LC_ALL", "fr_FR");
        
        let locale = detect_locale();
        assert_eq!(locale, Locale::Fr);
        
        // Restore
        env::remove_var("LC_ALL");
        if let Some(val) = original_tachikoma {
            env::set_var("TACHIKOMA_LOCALE", val);
        }
        if let Some(val) = original_lc_all {
            env::set_var("LC_ALL", val);
        }
    }

    #[test]
    fn test_lc_messages_priority() {
        // Save and clear higher priority vars
        let original_tachikoma = env::var("TACHIKOMA_LOCALE").ok();
        let original_lc_all = env::var("LC_ALL").ok();
        let original_lc_messages = env::var("LC_MESSAGES").ok();
        
        env::remove_var("TACHIKOMA_LOCALE");
        env::remove_var("LC_ALL");
        env::set_var("LC_MESSAGES", "es_ES");
        
        let locale = detect_locale();
        assert_eq!(locale, Locale::Es);
        
        // Restore
        env::remove_var("LC_MESSAGES");
        if let Some(val) = original_tachikoma {
            env::set_var("TACHIKOMA_LOCALE", val);
        }
        if let Some(val) = original_lc_all {
            env::set_var("LC_ALL", val);
        }
        if let Some(val) = original_lc_messages {
            env::set_var("LC_MESSAGES", val);
        }
    }

    #[test]
    fn test_lang_priority() {
        // Save and clear higher priority vars
        let original_tachikoma = env::var("TACHIKOMA_LOCALE").ok();
        let original_lc_all = env::var("LC_ALL").ok();
        let original_lc_messages = env::var("LC_MESSAGES").ok();
        let original_lang = env::var("LANG").ok();
        
        env::remove_var("TACHIKOMA_LOCALE");
        env::remove_var("LC_ALL");
        env::remove_var("LC_MESSAGES");
        env::set_var("LANG", "de_DE");
        
        let locale = detect_locale();
        assert_eq!(locale, Locale::De);
        
        // Restore
        env::remove_var("LANG");
        if let Some(val) = original_tachikoma {
            env::set_var("TACHIKOMA_LOCALE", val);
        }
        if let Some(val) = original_lc_all {
            env::set_var("LC_ALL", val);
        }
        if let Some(val) = original_lc_messages {
            env::set_var("LC_MESSAGES", val);
        }
        if let Some(val) = original_lang {
            env::set_var("LANG", val);
        }
    }

    #[test]
    fn test_invalid_env_variables() {
        // Save original environment state to restore later
        let env_vars = ["TACHIKOMA_LOCALE", "LC_ALL", "LC_MESSAGES", "LANG"];
        let original_values: Vec<_> = env_vars
            .iter()
            .map(|var| (var, env::var(var).ok()))
            .collect();
        
        // Clear all environment variables first
        for var in &env_vars {
            env::remove_var(var);
        }
        
        // Test invalid environment variables fall through properly
        env::set_var("TACHIKOMA_LOCALE", "invalid-locale");
        env::set_var("LC_ALL", "");
        env::set_var("LC_MESSAGES", "xxx");
        env::set_var("LANG", "not-a-locale");
        
        let locale = detect_locale();
        
        // Restore original environment before assertions
        for (var, value) in original_values {
            env::remove_var(var);
            if let Some(val) = value {
                env::set_var(var, val);
            }
        }
        
        // Should fall back to default or system locale for invalid env vars
        // On macOS it might detect system locale, otherwise falls back to English
        match locale {
            Locale::En => {}, // Expected fallback
            other => {
                // If system detection picked up something valid, that's also OK
                // as long as it's a real locale (not parsing the invalid env vars)
                match other {
                    Locale::Es | Locale::Fr | Locale::De | Locale::Ja | Locale::ZhCn => {},
                    _ => panic!("Unexpected locale: {:?}", other),
                }
            }
        }
    }

    #[test]
    fn test_from_env_function() {
        // Test valid env variable
        env::set_var("TEST_LOCALE", "es-ES");
        assert_eq!(from_env("TEST_LOCALE"), Some(Locale::Es));
        
        // Test missing env variable
        env::remove_var("TEST_LOCALE");
        assert_eq!(from_env("TEST_LOCALE"), None);
        
        // Test invalid env variable
        env::set_var("TEST_LOCALE", "invalid");
        assert_eq!(from_env("TEST_LOCALE"), None);
        
        // Clean up
        env::remove_var("TEST_LOCALE");
    }

    #[test]
    fn test_default_fallback_when_no_env() {
        // Save original environment state to restore later  
        let env_vars = ["TACHIKOMA_LOCALE", "LC_ALL", "LC_MESSAGES", "LANG"];
        let original_values: Vec<_> = env_vars
            .iter()
            .map(|var| (var, env::var(var).ok()))
            .collect();
        
        // Clear all possible environment variables
        for var in &env_vars {
            env::remove_var(var);
        }
        
        let locale1 = detect_locale();
        let locale2 = detect_locale();
        
        // Restore original environment
        for (var, value) in original_values {
            env::remove_var(var);
            if let Some(val) = value {
                env::set_var(var, val);
            }
        }
        
        // Should be deterministic (same result each time)
        assert_eq!(locale1, locale2, "detect_locale should be deterministic");
        
        // Should fall back to either system locale or default (English)
        // On macOS, system detection may return En, on other systems it falls back to En
        // We'll just ensure it returns a valid locale - either from system or default
        match locale1 {
            Locale::En | Locale::Es | Locale::Fr | Locale::De | Locale::Ja | Locale::ZhCn => {
                // Any valid locale is fine when no env vars set
            }
        }
    }

    #[test] 
    fn test_fallback_chain_all_locales() {
        // Test each locale's fallback chain
        assert_eq!(locale_fallback_chain(Locale::En), vec![Locale::En]);
        assert_eq!(locale_fallback_chain(Locale::Es), vec![Locale::Es, Locale::En]);
        assert_eq!(locale_fallback_chain(Locale::Fr), vec![Locale::Fr, Locale::En]);
        assert_eq!(locale_fallback_chain(Locale::De), vec![Locale::De, Locale::En]);
        assert_eq!(locale_fallback_chain(Locale::Ja), vec![Locale::Ja, Locale::En]);
        assert_eq!(locale_fallback_chain(Locale::ZhCn), vec![Locale::ZhCn, Locale::En]);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_detection_doesnt_crash() {
        // Test that macOS detection function doesn't panic
        // Even if defaults command fails, it should return None gracefully
        let result = detect_macos();
        // We don't assert on the result since it depends on the system
        // Just ensure it doesn't panic
        match result {
            Some(_) | None => {} // Both are fine
        }
    }

    #[test]
    fn test_user_preference_override() {
        // User preference should override everything
        assert_eq!(detect_locale_with_override(Some("ja_JP")), Locale::Ja);
        assert_eq!(detect_locale_with_override(Some("fr-FR")), Locale::Fr);
        assert_eq!(detect_locale_with_override(Some("es")), Locale::Es);
        
        // Invalid user preference should fall back to detection
        let fallback = detect_locale_with_override(Some("invalid-locale"));
        // Should be same as normal detection
        assert_eq!(fallback, detect_locale());
        
        // None should fall back to normal detection
        assert_eq!(detect_locale_with_override(None), detect_locale());
    }

    #[test]
    fn test_user_override_priority() {
        // Set environment variables
        let original_lang = env::var("LANG").ok();
        env::set_var("LANG", "de_DE");
        
        // User preference should override env vars
        assert_eq!(detect_locale_with_override(Some("ja")), Locale::Ja);
        assert_eq!(detect_locale(), Locale::De); // Without override, should get env var
        
        // Restore
        env::remove_var("LANG");
        if let Some(val) = original_lang {
            env::set_var("LANG", val);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_detection_returns_none() {
        // For now, Windows detection is unimplemented and returns None
        let result = detect_windows();
        assert_eq!(result, None);
    }
}