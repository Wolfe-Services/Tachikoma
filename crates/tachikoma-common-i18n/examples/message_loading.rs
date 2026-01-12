//! Example demonstrating i18n message loading capabilities.

use tachikoma_common_i18n::{
    default_catalog, load_catalog, CatalogFormat, LoaderConfig, LazyLoader, Locale,
};
use std::path::PathBuf;

fn main() {
    println!("=== Tachikoma i18n Message Loading Demo ===\n");

    // 1. Default embedded messages
    println!("1. Default embedded messages:");
    let default = default_catalog();
    println!("   app.name: {:?}", default.get("app.name"));
    println!("   app.tagline: {:?}", default.get("app.tagline"));
    println!("   mission.start: {:?}", default.get("mission.start"));
    println!();

    // 2. Load .po files directly
    println!("2. Loading .po files directly:");
    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test-data");
    
    let config = LoaderConfig {
        locale_dir: test_data_dir.join("locales"),
        domain: "tachikoma".to_string(),
        format: CatalogFormat::Po,
    };

    // Load Spanish
    match load_catalog(&config, Locale::Es) {
        Ok(catalog) => {
            println!("   Spanish loaded successfully:");
            println!("     app.tagline: {:?}", catalog.get("app.tagline"));
            println!("     mission.start: {:?}", catalog.get("mission.start"));
        }
        Err(e) => println!("   Failed to load Spanish: {}", e),
    }

    // Load French
    match load_catalog(&config, Locale::Fr) {
        Ok(catalog) => {
            println!("   French loaded successfully:");
            println!("     app.tagline: {:?}", catalog.get("app.tagline"));
            println!("     mission.start: {:?}", catalog.get("mission.start"));
        }
        Err(e) => println!("   Failed to load French: {}", e),
    }
    println!();

    // 3. Lazy loading with hot reload
    println!("3. Lazy loading with hot reload:");
    let loader = LazyLoader::new(config).with_hot_reload(true);
    
    println!("   Initial stats: {:?}", loader.stats());
    
    // Preload some locales
    if let Err(e) = loader.preload(&[Locale::Es, Locale::Fr, Locale::De]) {
        println!("   Error preloading: {}", e);
    } else {
        println!("   After preloading: {:?}", loader.stats());
    }
    
    // Clear cache
    loader.clear_cache();
    println!("   After cache clear: {:?}", loader.stats());
    println!();

    // 4. Demonstrate different formats
    println!("4. Configuration options:");
    let po_config = LoaderConfig {
        locale_dir: PathBuf::from("locales"),
        domain: "myapp".to_string(),
        format: CatalogFormat::Po,
    };
    println!("   PO format config: {:?}", po_config);
    
    let mo_config = LoaderConfig {
        locale_dir: PathBuf::from("locales"),
        domain: "myapp".to_string(),
        format: CatalogFormat::Mo,
    };
    println!("   MO format config: {:?}", mo_config);
    println!();

    // 5. Handle missing files gracefully
    println!("5. Graceful handling of missing files:");
    let missing_config = LoaderConfig {
        locale_dir: PathBuf::from("/nonexistent"),
        domain: "missing".to_string(),
        format: CatalogFormat::Po,
    };
    
    match load_catalog(&missing_config, Locale::En) {
        Ok(catalog) => {
            println!("   Missing file handled - empty catalog returned");
            println!("   Catalog has key 'test': {}", catalog.get("test").is_some());
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== Demo complete ===");
}