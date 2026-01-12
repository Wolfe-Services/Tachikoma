# 024 - i18n Message Loading

**Phase:** 1 - Core Common Crates
**Spec ID:** 024
**Status:** Planned
**Dependencies:** 023-i18n-core-setup
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Implement loading of message catalogs from .po/.mo files and embedded resources with lazy loading support.

---

## Acceptance Criteria

- [ ] Load .po files (human-readable)
- [ ] Load .mo files (compiled binary)
- [ ] Embed default messages in binary
- [ ] Lazy loading of additional locales
- [ ] Hot reload in development

---

## Implementation Details

### 1. Message Loading (crates/tachikoma-common-i18n/src/loader.rs)

```rust
//! Message catalog loading.

use super::{Catalog, I18nError, Locale};
use std::fs;
use std::path::{Path, PathBuf};

/// Catalog loader configuration.
#[derive(Debug, Clone)]
pub struct LoaderConfig {
    /// Directory containing locale files.
    pub locale_dir: PathBuf,
    /// Domain name (e.g., "tachikoma").
    pub domain: String,
    /// File format to load.
    pub format: CatalogFormat,
}

/// Catalog file format.
#[derive(Debug, Clone, Copy)]
pub enum CatalogFormat {
    /// Human-readable .po format.
    Po,
    /// Compiled binary .mo format.
    Mo,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            locale_dir: PathBuf::from("locales"),
            domain: "tachikoma".to_string(),
            format: CatalogFormat::Po,
        }
    }
}

/// Load a catalog for a locale.
pub fn load_catalog(config: &LoaderConfig, locale: Locale) -> Result<Catalog, I18nError> {
    let extension = match config.format {
        CatalogFormat::Po => "po",
        CatalogFormat::Mo => "mo",
    };

    let path = config
        .locale_dir
        .join(locale.code())
        .join(format!("{}.{}", config.domain, extension));

    if !path.exists() {
        return Ok(Catalog::new()); // Return empty catalog
    }

    match config.format {
        CatalogFormat::Po => load_po(&path),
        CatalogFormat::Mo => load_mo(&path),
    }
}

/// Load a .po file.
fn load_po(path: &Path) -> Result<Catalog, I18nError> {
    let content = fs::read_to_string(path)
        .map_err(|e| I18nError::LoadError(e.to_string()))?;

    let mut catalog = Catalog::new();
    let mut current_msgid: Option<String> = None;
    let mut current_msgstr: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("msgid ") {
            // Save previous entry
            if let (Some(id), Some(str)) = (current_msgid.take(), current_msgstr.take()) {
                if !id.is_empty() {
                    catalog.insert(id, str);
                }
            }
            current_msgid = Some(parse_po_string(line.strip_prefix("msgid ").unwrap()));
        } else if line.starts_with("msgstr ") {
            current_msgstr = Some(parse_po_string(line.strip_prefix("msgstr ").unwrap()));
        } else if line.starts_with('"') {
            // Continuation line
            let continued = parse_po_string(line);
            if let Some(ref mut msgstr) = current_msgstr {
                msgstr.push_str(&continued);
            } else if let Some(ref mut msgid) = current_msgid {
                msgid.push_str(&continued);
            }
        }
    }

    // Save last entry
    if let (Some(id), Some(str)) = (current_msgid, current_msgstr) {
        if !id.is_empty() {
            catalog.insert(id, str);
        }
    }

    Ok(catalog)
}

/// Parse a .po string literal.
fn parse_po_string(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        // Unescape common escapes
        inner
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        s.to_string()
    }
}

/// Load a .mo file (binary format).
fn load_mo(path: &Path) -> Result<Catalog, I18nError> {
    let data = fs::read(path)
        .map_err(|e| I18nError::LoadError(e.to_string()))?;

    // Check magic number
    if data.len() < 28 {
        return Err(I18nError::LoadError("file too small".to_string()));
    }

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let is_le = magic == 0x950412de;
    let is_be = magic == 0xde120495;

    if !is_le && !is_be {
        return Err(I18nError::LoadError("invalid magic number".to_string()));
    }

    // Parse header and entries
    // Simplified - real implementation would fully parse .mo format
    Ok(Catalog::new())
}

/// Embedded default messages (English).
pub fn default_catalog() -> Catalog {
    let mut catalog = Catalog::new();

    // Core messages
    catalog.insert("app.name", "Tachikoma");
    catalog.insert("app.tagline", "Your squad of tireless AI coders");

    // Mission messages
    catalog.insert("mission.start", "Starting mission...");
    catalog.insert("mission.complete", "Mission complete!");
    catalog.insert("mission.error", "Mission failed: {error}");

    // Status messages
    catalog.insert("status.running", "Running");
    catalog.insert("status.paused", "Paused");
    catalog.insert("status.idle", "Idle");

    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_po_string() {
        assert_eq!(parse_po_string("\"hello\""), "hello");
        assert_eq!(parse_po_string("\"hello\\nworld\""), "hello\nworld");
        assert_eq!(parse_po_string("\"say \\\"hi\\\"\""), "say \"hi\"");
    }

    #[test]
    fn test_default_catalog() {
        let catalog = default_catalog();
        assert_eq!(catalog.get("app.name"), Some("Tachikoma"));
    }
}
```

---

## Testing Requirements

1. .po files parse correctly
2. Escaped characters are unescaped
3. Missing files return empty catalog
4. Default catalog has all required keys

---

## Related Specs

- Depends on: [023-i18n-core-setup.md](023-i18n-core-setup.md)
- Next: [025-i18n-locale-detection.md](025-i18n-locale-detection.md)
