fn main() {
    println!("Testing locale detection:");
    
    // Test normal detection
    let locale = tachikoma_common_i18n::detect_locale();
    println!("Detected locale: {} ({})", locale.code(), locale.name());
    
    // Test fallback chain
    let chain = tachikoma_common_i18n::locale_fallback_chain(locale);
    println!("Fallback chain: {:?}", chain.iter().map(|l| l.code()).collect::<Vec<_>>());
    
    // Test user override
    let user_locale = tachikoma_common_i18n::detect_locale_with_override(Some("ja"));
    println!("With user override (ja): {} ({})", user_locale.code(), user_locale.name());
    
    // Test invalid user override
    let fallback = tachikoma_common_i18n::detect_locale_with_override(Some("invalid"));
    println!("With invalid override: {} ({})", fallback.code(), fallback.name());
}
