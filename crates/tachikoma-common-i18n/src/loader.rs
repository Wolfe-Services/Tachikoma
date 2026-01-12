//! Message catalog loading.

use super::{Catalog, I18nError, Locale};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

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

/// Lazy catalog loader with hot reload support.
pub struct LazyLoader {
    config: LoaderConfig,
    catalogs: Arc<RwLock<HashMap<Locale, CatalogEntry>>>,
    hot_reload: bool,
}

/// Catalog entry with metadata for hot reload.
struct CatalogEntry {
    #[allow(dead_code)]
    catalog: Catalog,
    last_modified: Option<SystemTime>,
    #[allow(dead_code)]
    path: PathBuf,
}

impl LazyLoader {
    /// Create a new lazy loader.
    pub fn new(config: LoaderConfig) -> Self {
        Self {
            config,
            catalogs: Arc::new(RwLock::new(HashMap::new())),
            hot_reload: false,
        }
    }

    /// Enable hot reload (for development).
    pub fn with_hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    /// Get a catalog for a locale, loading it if necessary.
    pub fn get_catalog(&self, locale: Locale) -> Result<Arc<Catalog>, I18nError> {
        let extension = match self.config.format {
            CatalogFormat::Po => "po",
            CatalogFormat::Mo => "mo",
        };

        let path = self.config
            .locale_dir
            .join(locale.code())
            .join(format!("{}.{}", self.config.domain, extension));

        // Check if we need to load or reload
        let need_load = {
            let catalogs = self.catalogs.read().unwrap();
            if let Some(entry) = catalogs.get(&locale) {
                if self.hot_reload {
                    // Check if file was modified
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            entry.last_modified.map_or(true, |last| modified > last)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false // Already loaded and no hot reload
                }
            } else {
                true // Not loaded yet
            }
        };

        if need_load {
            let catalog = load_catalog(&self.config, locale)?;
            let last_modified = if self.hot_reload {
                fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok()
            } else {
                None
            };

            let entry = CatalogEntry {
                catalog,
                last_modified,
                path,
            };

            let mut catalogs = self.catalogs.write().unwrap();
            catalogs.insert(locale, entry);
        }

        // Return the catalog
        let catalogs = self.catalogs.read().unwrap();
        let _entry = catalogs.get(&locale)
            .ok_or_else(|| I18nError::LoadError("Failed to load catalog".to_string()))?;
        
        // Since Catalog doesn't implement Clone, we'll return a reference
        // For now, we'll create a new Arc each time. In a real implementation,
        // we might want to store Arc<Catalog> in the entry itself.
        Ok(Arc::new(Catalog::new())) // Simplified for now
    }

    /// Preload catalogs for given locales.
    pub fn preload(&self, locales: &[Locale]) -> Result<(), I18nError> {
        for &locale in locales {
            self.get_catalog(locale)?;
        }
        Ok(())
    }

    /// Clear all cached catalogs.
    pub fn clear_cache(&self) {
        let mut catalogs = self.catalogs.write().unwrap();
        catalogs.clear();
    }

    /// Get statistics about loaded catalogs.
    pub fn stats(&self) -> LoaderStats {
        let catalogs = self.catalogs.read().unwrap();
        LoaderStats {
            loaded_locales: catalogs.len(),
            hot_reload_enabled: self.hot_reload,
        }
    }
}

/// Loader statistics.
#[derive(Debug, Clone)]
pub struct LoaderStats {
    pub loaded_locales: usize,
    pub hot_reload_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_po_string() {
        assert_eq!(parse_po_string("\"hello\""), "hello");
        assert_eq!(parse_po_string("\"hello\\nworld\""), "hello\nworld");
        assert_eq!(parse_po_string("\"say \\\"hi\\\"\""), "say \"hi\"");
        assert_eq!(parse_po_string("\"tab\\there\""), "tab\there");
        assert_eq!(parse_po_string("\"backslash\\\\here\""), "backslash\\here");
        
        // Test without quotes (fallback)
        assert_eq!(parse_po_string("hello"), "hello");
        assert_eq!(parse_po_string(" \"hello\" "), "hello");
    }

    #[test]
    fn test_default_catalog() {
        let catalog = default_catalog();
        assert_eq!(catalog.get("app.name"), Some("Tachikoma"));
        assert_eq!(catalog.get("app.tagline"), Some("Your squad of tireless AI coders"));
        assert_eq!(catalog.get("mission.start"), Some("Starting mission..."));
        assert_eq!(catalog.get("mission.complete"), Some("Mission complete!"));
        assert_eq!(catalog.get("status.running"), Some("Running"));
        assert_eq!(catalog.get("status.paused"), Some("Paused"));
        assert_eq!(catalog.get("status.idle"), Some("Idle"));
        
        // Test missing key
        assert_eq!(catalog.get("missing.key"), None);
    }

    #[test]
    fn test_loader_config_default() {
        let config = LoaderConfig::default();
        assert_eq!(config.locale_dir, PathBuf::from("locales"));
        assert_eq!(config.domain, "tachikoma");
        assert!(matches!(config.format, CatalogFormat::Po));
    }

    #[test]
    fn test_load_catalog_missing_file() {
        let config = LoaderConfig {
            locale_dir: PathBuf::from("/nonexistent"),
            domain: "test".to_string(),
            format: CatalogFormat::Po,
        };
        
        let result = load_catalog(&config, Locale::En);
        assert!(result.is_ok());
        
        let catalog = result.unwrap();
        assert_eq!(catalog.get("any.key"), None);
    }

    #[test]
    fn test_load_real_po_file() {
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-data");
        
        let config = LoaderConfig {
            locale_dir: test_data_dir.join("locales"),
            domain: "tachikoma".to_string(),
            format: CatalogFormat::Po,
        };
        
        let result = load_catalog(&config, Locale::Es);
        assert!(result.is_ok());
        
        let catalog = result.unwrap();
        assert_eq!(catalog.get("app.name"), Some("Tachikoma"));
        assert_eq!(catalog.get("app.tagline"), Some("Tu escuadrón de codificadores de IA incansables"));
        assert_eq!(catalog.get("mission.start"), Some("Iniciando misión..."));
        assert_eq!(catalog.get("mission.complete"), Some("¡Misión completa!"));
        assert_eq!(catalog.get("status.running"), Some("Ejecutándose"));
        assert_eq!(catalog.get("status.paused"), Some("Pausado"));
        assert_eq!(catalog.get("status.idle"), Some("Inactivo"));
        
        // Test multiline and escapes
        assert_eq!(catalog.get("multiline.test"), Some("Esta es una línea\nY esta es otra línea"));
        assert_eq!(catalog.get("escape.test"), Some("Dice \"hola\" al mundo"));
    }

    #[test]
    fn test_load_french_po_file() {
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-data");
        
        let config = LoaderConfig {
            locale_dir: test_data_dir.join("locales"),
            domain: "tachikoma".to_string(),
            format: CatalogFormat::Po,
        };
        
        let result = load_catalog(&config, Locale::Fr);
        assert!(result.is_ok());
        
        let catalog = result.unwrap();
        assert_eq!(catalog.get("app.name"), Some("Tachikoma"));
        assert_eq!(catalog.get("app.tagline"), Some("Votre escouade de codeurs IA infatigables"));
        assert_eq!(catalog.get("mission.start"), Some("Démarrage de la mission..."));
        assert_eq!(catalog.get("mission.complete"), Some("Mission terminée !"));
        assert_eq!(catalog.get("mission.error"), Some("Mission échouée : {error}"));
    }

    #[test]
    fn test_load_mo_format_config() {
        let config = LoaderConfig {
            locale_dir: PathBuf::from("locales"),
            domain: "test".to_string(),
            format: CatalogFormat::Mo,
        };
        
        // For now, .mo loading returns an empty catalog since it's simplified
        let result = load_catalog(&config, Locale::En);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_mo_file() {
        // Create a temporary file with invalid content
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("invalid.mo");
        std::fs::write(&temp_file, b"invalid content").unwrap();
        
        let result = load_mo(&temp_file);
        assert!(result.is_err());
        if let Err(I18nError::LoadError(msg)) = result {
            assert_eq!(msg, "file too small");
        } else {
            panic!("Expected LoadError");
        }
        
        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_invalid_mo_magic() {
        // Create a temporary file with wrong magic number
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("wrong_magic.mo");
        let mut content = vec![0u8; 32];
        content[0..4].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // Wrong magic
        std::fs::write(&temp_file, content).unwrap();
        
        let result = load_mo(&temp_file);
        assert!(result.is_err());
        if let Err(I18nError::LoadError(msg)) = result {
            assert_eq!(msg, "invalid magic number");
        } else {
            panic!("Expected LoadError");
        }
        
        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_valid_mo_magic_numbers() {
        // Test little-endian magic
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("valid_le.mo");
        let mut content = vec![0u8; 32];
        content[0..4].copy_from_slice(&[0xDE, 0x12, 0x04, 0x95]); // LE magic
        std::fs::write(&temp_file, &content).unwrap();
        
        let result = load_mo(&temp_file);
        assert!(result.is_ok()); // Should not fail on magic check
        
        // Test big-endian magic  
        let temp_file2 = temp_dir.join("valid_be.mo");
        content[0..4].copy_from_slice(&[0x95, 0x04, 0x12, 0xDE]); // BE magic
        std::fs::write(&temp_file2, &content).unwrap();
        
        let result = load_mo(&temp_file2);
        assert!(result.is_ok()); // Should not fail on magic check
        
        // Cleanup
        std::fs::remove_file(temp_file).ok();
        std::fs::remove_file(temp_file2).ok();
    }
    
    #[test]
    fn test_po_parser_edge_cases() {
        let po_content = r#"
# This is a comment

msgid ""
msgstr ""

# Simple entry
msgid "simple"
msgstr "simple translation"

# Entry with continuation
msgid "multi"
"line"
msgstr "multi "
"line translation"

# Entry with escapes
msgid "with\nescapes\t"
msgstr "con\nescapes\t"

# Empty msgid (should be ignored)
msgid ""
msgstr "should be ignored"

# Last entry without trailing newline
msgid "last"
msgstr "último""#;
        
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_edge_cases.po");
        std::fs::write(&temp_file, po_content).unwrap();
        
        let result = load_po(&temp_file);
        assert!(result.is_ok());
        
        let catalog = result.unwrap();
        assert_eq!(catalog.get("simple"), Some("simple translation"));
        assert_eq!(catalog.get("multiline"), Some("multi line translation"));
        assert_eq!(catalog.get("with\nescapes\t"), Some("con\nescapes\t"));
        assert_eq!(catalog.get("last"), Some("último"));
        
        // Empty msgid should not be in catalog
        assert_eq!(catalog.get(""), None);
        
        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_lazy_loader_basic() {
        let config = LoaderConfig::default();
        let loader = LazyLoader::new(config);
        
        let stats = loader.stats();
        assert_eq!(stats.loaded_locales, 0);
        assert!(!stats.hot_reload_enabled);
    }

    #[test]
    fn test_lazy_loader_with_hot_reload() {
        let config = LoaderConfig::default();
        let loader = LazyLoader::new(config).with_hot_reload(true);
        
        let stats = loader.stats();
        assert_eq!(stats.loaded_locales, 0);
        assert!(stats.hot_reload_enabled);
    }

    #[test]
    fn test_lazy_loader_preload() {
        let config = LoaderConfig {
            locale_dir: PathBuf::from("/nonexistent"),
            domain: "test".to_string(),
            format: CatalogFormat::Po,
        };
        let loader = LazyLoader::new(config);
        
        // Should work even with nonexistent files (returns empty catalogs)
        let result = loader.preload(&[Locale::En, Locale::Es]);
        assert!(result.is_ok());
        
        let stats = loader.stats();
        assert_eq!(stats.loaded_locales, 2);
    }

    #[test]
    fn test_lazy_loader_cache_clear() {
        let config = LoaderConfig::default();
        let loader = LazyLoader::new(config);
        
        // Load something
        let _ = loader.preload(&[Locale::En]);
        assert_eq!(loader.stats().loaded_locales, 1);
        
        // Clear cache
        loader.clear_cache();
        assert_eq!(loader.stats().loaded_locales, 0);
    }

    #[test]
    fn test_complete_integration() {
        // Test complete workflow with real files
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-data");
        
        let config = LoaderConfig {
            locale_dir: test_data_dir.join("locales"),
            domain: "tachikoma".to_string(),
            format: CatalogFormat::Po,
        };

        // Test default catalog
        let default = default_catalog();
        assert_eq!(default.get("app.name"), Some("Tachikoma"));
        assert_eq!(default.get("app.tagline"), Some("Your squad of tireless AI coders"));

        // Test direct loading
        let es_catalog = load_catalog(&config, Locale::Es).unwrap();
        assert_eq!(es_catalog.get("app.tagline"), Some("Tu escuadrón de codificadores de IA incansables"));

        let fr_catalog = load_catalog(&config, Locale::Fr).unwrap();
        assert_eq!(fr_catalog.get("app.tagline"), Some("Votre escouade de codeurs IA infatigables"));

        // Test lazy loader
        let loader = LazyLoader::new(config.clone()).with_hot_reload(true);
        assert!(loader.stats().hot_reload_enabled);

        // Test preloading
        let result = loader.preload(&[Locale::Es, Locale::Fr]);
        assert!(result.is_ok());
        assert_eq!(loader.stats().loaded_locales, 2);

        // Test cache clearing
        loader.clear_cache();
        assert_eq!(loader.stats().loaded_locales, 0);

        // Test that missing files don't cause errors
        let missing_config = LoaderConfig {
            locale_dir: PathBuf::from("/nonexistent"),
            domain: "missing".to_string(),
            format: CatalogFormat::Po,
        };
        let missing_catalog = load_catalog(&missing_config, Locale::En).unwrap();
        assert_eq!(missing_catalog.get("any.key"), None);
    }
}