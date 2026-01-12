//! Simple example demonstrating i18n usage.

use tachikoma_common_i18n::{Catalog, I18n, Locale, t};

fn main() {
    println!("=== Tachikoma i18n Example ===");
    
    // Initialize the i18n system with English
    I18n::init(Locale::En);
    println!("Current locale: {} ({})", I18n::locale().code(), I18n::locale().name());
    
    // Test fallback behavior (no catalogs loaded)
    println!("Fallback: '{}'", t!("app.welcome"));
    
    // Create and load a Spanish catalog
    let mut es_catalog = Catalog::new();
    es_catalog.insert("app.welcome", "¡Bienvenido a Tachikoma!");
    es_catalog.insert("app.goodbye", "¡Hasta luego!");
    es_catalog.insert("user.greeting", "Hola, {name}!");
    
    I18n::add_catalog(Locale::Es, es_catalog);
    
    // Switch to Spanish
    I18n::set_locale(Locale::Es);
    println!("Current locale: {} ({})", I18n::locale().code(), I18n::locale().name());
    
    // Test basic translation
    println!("Translation: '{}'", t!("app.welcome"));
    
    // Test template substitution
    println!("With template: '{}'", t!("user.greeting", name = "María"));
    
    // Test missing key (fallback)
    println!("Missing key: '{}'", t!("missing.key"));
    
    // Create and load a Japanese catalog
    let mut ja_catalog = Catalog::new();
    ja_catalog.insert("app.welcome", "Tachikomaへようこそ！");
    ja_catalog.insert("app.goodbye", "さようなら！");
    ja_catalog.insert("user.greeting", "こんにちは、{name}さん！");
    
    I18n::add_catalog(Locale::Ja, ja_catalog);
    
    // Switch to Japanese
    I18n::set_locale(Locale::Ja);
    println!("Current locale: {} ({})", I18n::locale().code(), I18n::locale().name());
    
    println!("Translation: '{}'", t!("app.welcome"));
    println!("With template: '{}'", t!("user.greeting", name = "田中"));
    
    println!("\n=== Locale Parsing Examples ===");
    
    // Test locale parsing
    let test_locales = vec![
        "en-US", "en_US", "EN",
        "ja-JP", "ja_JP", "JA", 
        "zh-CN", "zh_CN",
        "invalid", "xx-YY"
    ];
    
    for locale_str in test_locales {
        match Locale::parse(locale_str) {
            Some(locale) => println!("'{}' -> {} ({})", locale_str, locale.code(), locale.name()),
            None => println!("'{}' -> Invalid", locale_str),
        }
    }
}