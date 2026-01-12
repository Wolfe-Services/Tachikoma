# 023 - i18n Core Setup

**Phase:** 1 - Core Common Crates
**Spec ID:** 023
**Status:** Planned
**Dependencies:** 011-common-core-types
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Set up internationalization infrastructure using gettext for Rust with message catalogs, pluralization, and locale management.

---

## Acceptance Criteria

- [x] gettext integration for Rust
- [x] Message catalog structure (.po/.mo)
- [x] Locale type with parsing
- [x] Translation macros
- [x] Fallback handling

---

## Implementation Details

### 1. i18n Module (crates/tachikoma-common-i18n/src/lib.rs)

```rust
//! Internationalization support for Tachikoma.

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
        assert_eq!(Locale::parse("en-US"), Some(Locale::En));
        assert_eq!(Locale::parse("ja_JP"), Some(Locale::Ja));
        assert_eq!(Locale::parse("zh-CN"), Some(Locale::ZhCn));
    }

    #[test]
    fn test_translate_fallback() {
        I18n::init(Locale::En);
        assert_eq!(I18n::translate("unknown.key"), "unknown.key");
    }
}
```

### 2. Crate Setup

```toml
[package]
name = "tachikoma-common-i18n"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror.workspace = true
```

---

## Testing Requirements

1. Locale parsing handles various formats
2. Missing translations fall back to msgid
3. Plural forms select correctly
4. Template substitution works

---

## Related Specs

- Depends on: [011-common-core-types.md](011-common-core-types.md)
- Next: [024-i18n-message-loading.md](024-i18n-message-loading.md)
