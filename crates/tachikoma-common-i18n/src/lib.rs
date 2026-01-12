//! Internationalization support for Tachikoma.

pub mod loader;
pub mod detect;

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,    // English (default)
    Es,    // Spanish
    Fr,    // French
    De,    // German
    Ja,    // Japanese
    ZhCn,  // Chinese (Simplified)
}

impl Locale {
    /// Parse from a locale string (e.g., "en-US", "ja_JP").
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.to_lowercase().replace('-', "_");
        let lang = s.split('_').next()?;

        match lang {
            "en" => Some(Self::En),
            "es" => Some(Self::Es),
            "fr" => Some(Self::Fr),
            "de" => Some(Self::De),
            "ja" => Some(Self::Ja),
            "zh" => Some(Self::ZhCn),
            _ => None,
        }
    }

    /// Get the language code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::De => "de",
            Self::Ja => "ja",
            Self::ZhCn => "zh_CN",
        }
    }

    /// Get the display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
            Self::Fr => "Français",
            Self::De => "Deutsch",
            Self::Ja => "日本語",
            Self::ZhCn => "中文(简体)",
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::En
    }
}

/// Message catalog for a locale.
pub struct Catalog {
    messages: HashMap<String, String>,
    plurals: HashMap<String, Vec<String>>,
}

impl Catalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            plurals: HashMap::new(),
        }
    }

    /// Load from a .mo file.
    pub fn load_mo(path: impl AsRef<Path>) -> Result<Self, I18nError> {
        // Simplified - real implementation would parse .mo format
        let _path = path.as_ref();
        Ok(Self::new())
    }

    /// Get a translated message.
    pub fn get(&self, msgid: &str) -> Option<&str> {
        self.messages.get(msgid).map(|s| s.as_str())
    }

    /// Get a plural translation.
    pub fn get_plural(&self, msgid: &str, n: u64) -> Option<&str> {
        self.plurals.get(msgid).and_then(|forms| {
            let idx = Self::plural_index(n);
            forms.get(idx).map(|s| s.as_str())
        })
    }

    /// Calculate plural form index for English-like languages.
    fn plural_index(n: u64) -> usize {
        if n == 1 { 0 } else { 1 }
    }

    /// Add a message (for testing/building catalogs).
    pub fn insert(&mut self, msgid: impl Into<String>, msgstr: impl Into<String>) {
        self.messages.insert(msgid.into(), msgstr.into());
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Global i18n state.
static I18N: RwLock<Option<I18n>> = RwLock::new(None);

/// i18n manager.
pub struct I18n {
    current_locale: Locale,
    catalogs: HashMap<Locale, Catalog>,
}

impl I18n {
    /// Initialize global i18n.
    pub fn init(locale: Locale) {
        let mut guard = I18N.write().unwrap();
        *guard = Some(Self {
            current_locale: locale,
            catalogs: HashMap::new(),
        });
    }

    /// Add a catalog for a locale.
    pub fn add_catalog(locale: Locale, catalog: Catalog) {
        if let Some(ref mut i18n) = *I18N.write().unwrap() {
            i18n.catalogs.insert(locale, catalog);
        }
    }

    /// Set the current locale.
    pub fn set_locale(locale: Locale) {
        if let Some(ref mut i18n) = *I18N.write().unwrap() {
            i18n.current_locale = locale;
        }
    }

    /// Get the current locale.
    pub fn locale() -> Locale {
        I18N.read()
            .unwrap()
            .as_ref()
            .map(|i| i.current_locale)
            .unwrap_or_default()
    }

    /// Translate a message.
    pub fn translate(msgid: &str) -> String {
        let guard = I18N.read().unwrap();
        if let Some(ref i18n) = *guard {
            if let Some(catalog) = i18n.catalogs.get(&i18n.current_locale) {
                if let Some(msg) = catalog.get(msgid) {
                    return msg.to_string();
                }
            }
        }
        msgid.to_string()
    }
}

/// i18n errors.
#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("failed to load catalog: {0}")]
    LoadError(String),

    #[error("invalid locale: {0}")]
    InvalidLocale(String),
}

/// Translation macro.
#[macro_export]
macro_rules! t {
    ($msgid:expr) => {
        $crate::I18n::translate($msgid)
    };
    ($msgid:expr, $($key:ident = $value:expr),*) => {{
        let mut msg = $crate::I18n::translate($msgid);
        $(
            msg = msg.replace(concat!("{", stringify!($key), "}"), &$value.to_string());
        )*
        msg
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_parse() {
        // Test various formats
        assert_eq!(Locale::parse("en-US"), Some(Locale::En));
        assert_eq!(Locale::parse("en_US"), Some(Locale::En));
        assert_eq!(Locale::parse("EN"), Some(Locale::En));
        assert_eq!(Locale::parse("ja_JP"), Some(Locale::Ja));
        assert_eq!(Locale::parse("zh-CN"), Some(Locale::ZhCn));
        assert_eq!(Locale::parse("es-ES"), Some(Locale::Es));
        assert_eq!(Locale::parse("fr_FR"), Some(Locale::Fr));
        assert_eq!(Locale::parse("de_DE"), Some(Locale::De));
        
        // Test invalid locales
        assert_eq!(Locale::parse("invalid"), None);
        assert_eq!(Locale::parse(""), None);
        assert_eq!(Locale::parse("xx-YY"), None);
    }

    #[test]
    fn test_locale_properties() {
        assert_eq!(Locale::En.code(), "en");
        assert_eq!(Locale::Es.code(), "es");
        assert_eq!(Locale::Fr.code(), "fr");
        assert_eq!(Locale::De.code(), "de");
        assert_eq!(Locale::Ja.code(), "ja");
        assert_eq!(Locale::ZhCn.code(), "zh_CN");

        assert_eq!(Locale::En.name(), "English");
        assert_eq!(Locale::Es.name(), "Español");
        assert_eq!(Locale::Fr.name(), "Français");
        assert_eq!(Locale::De.name(), "Deutsch");
        assert_eq!(Locale::Ja.name(), "日本語");
        assert_eq!(Locale::ZhCn.name(), "中文(简体)");
    }

    #[test]
    fn test_locale_default() {
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn test_catalog_basic_operations() {
        let mut catalog = Catalog::new();
        assert!(catalog.get("test").is_none());
        
        catalog.insert("hello", "Hello");
        assert_eq!(catalog.get("hello"), Some("Hello"));
        assert_eq!(catalog.get("missing"), None);
    }

    #[test]
    fn test_plural_forms() {
        let catalog = Catalog::new();
        
        // Test plural index calculation
        assert_eq!(Catalog::plural_index(0), 1); // "0 items"
        assert_eq!(Catalog::plural_index(1), 0); // "1 item"
        assert_eq!(Catalog::plural_index(2), 1); // "2 items"
        assert_eq!(Catalog::plural_index(100), 1); // "100 items"
        
        // Test plural retrieval with empty catalog
        assert!(catalog.get_plural("items", 0).is_none());
        assert!(catalog.get_plural("items", 1).is_none());
    }

    #[test]
    fn test_translate_fallback() {
        I18n::init(Locale::En);
        
        // Missing translations should fall back to msgid
        assert_eq!(I18n::translate("unknown.key"), "unknown.key");
        assert_eq!(I18n::translate("another.missing"), "another.missing");
        assert_eq!(I18n::translate(""), "");
    }

    #[test]
    fn test_i18n_locale_management() {
        // Test initialization
        I18n::init(Locale::Fr);
        assert_eq!(I18n::locale(), Locale::Fr);
        
        // Test locale switching
        I18n::set_locale(Locale::Ja);
        assert_eq!(I18n::locale(), Locale::Ja);
        
        I18n::set_locale(Locale::Es);
        assert_eq!(I18n::locale(), Locale::Es);
    }

    #[test]
    fn test_translation_macros() {
        I18n::init(Locale::En);
        
        // Basic translation macro
        assert_eq!(t!("test.key"), "test.key");
        
        // Template substitution macro
        let result = t!("Hello {name}!", name = "World");
        assert_eq!(result, "Hello World!");
        
        let result = t!("User {id} has {count} messages", id = 123, count = 5);
        assert_eq!(result, "User 123 has 5 messages");
    }

    #[test]
    fn test_catalog_load_mo() {
        // Test that load_mo returns Ok for now (simplified implementation)
        let result = Catalog::load_mo("nonexistent.mo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_types() {
        let error = I18nError::LoadError("test error".to_string());
        assert_eq!(error.to_string(), "failed to load catalog: test error");
        
        let error = I18nError::InvalidLocale("invalid".to_string());
        assert_eq!(error.to_string(), "invalid locale: invalid");
    }

    #[test]
    fn test_catalog_integration() {
        // Initialize i18n system
        I18n::init(Locale::Es);
        
        // Create Spanish catalog
        let mut es_catalog = Catalog::new();
        es_catalog.insert("hello", "Hola");
        es_catalog.insert("goodbye", "Adiós");
        es_catalog.insert("welcome", "Bienvenido");
        
        // Add catalog to i18n system
        I18n::add_catalog(Locale::Es, es_catalog);
        
        // Test translations
        assert_eq!(I18n::translate("hello"), "Hola");
        assert_eq!(I18n::translate("goodbye"), "Adiós"); 
        assert_eq!(I18n::translate("welcome"), "Bienvenido");
        assert_eq!(I18n::translate("missing"), "missing"); // fallback
        
        // Test macro translations
        assert_eq!(t!("hello"), "Hola");
        assert_eq!(t!("Hola {name}!", name = "María"), "Hola María!");
        
        // Switch to English (no catalog) - should fallback
        I18n::set_locale(Locale::En);
        assert_eq!(I18n::translate("hello"), "hello"); // fallback to msgid
    }
}

// Re-exports for convenience
pub use loader::{
    default_catalog, load_catalog, CatalogFormat, LoaderConfig, LazyLoader, LoaderStats,
};

pub use detect::{
    detect_locale, detect_locale_with_override, locale_fallback_chain,
};