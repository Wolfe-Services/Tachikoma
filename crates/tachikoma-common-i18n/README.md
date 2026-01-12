# tachikoma-common-i18n

Internationalization (i18n) support for Tachikoma using gettext-style message catalogs.

## Features

- **Locale Management**: Support for multiple locales with automatic parsing
- **Message Catalogs**: .po/.mo file format support for translations
- **Translation Macros**: Convenient `t!()` macro for translations with template substitution
- **Fallback Handling**: Graceful fallback to message IDs when translations are missing
- **Plural Forms**: Support for locale-specific plural form handling

## Supported Locales

- `en` - English (default)
- `es` - Spanish (Español)
- `fr` - French (Français)
- `de` - German (Deutsch)
- `ja` - Japanese (日本語)
- `zh_CN` - Chinese Simplified (中文简体)

## Usage

### Basic Setup

```rust
use tachikoma_common_i18n::{Catalog, I18n, Locale, t};

// Initialize the i18n system
I18n::init(Locale::En);

// Create a message catalog
let mut catalog = Catalog::new();
catalog.insert("app.welcome", "Welcome to Tachikoma!");
catalog.insert("user.greeting", "Hello, {name}!");

// Add the catalog for English
I18n::add_catalog(Locale::En, catalog);
```

### Using Translations

```rust
// Basic translation
let message = t!("app.welcome");

// Translation with template substitution
let greeting = t!("user.greeting", name = "Alice");

// Missing translations fall back to the message ID
let fallback = t!("missing.key"); // Returns "missing.key"
```

### Locale Management

```rust
// Parse locale from string
let locale = Locale::parse("en-US").unwrap(); // Returns Locale::En
let locale = Locale::parse("ja_JP").unwrap(); // Returns Locale::Ja

// Get locale properties
println!("Code: {}", locale.code()); // "en"
println!("Name: {}", locale.name()); // "English"

// Switch locales
I18n::set_locale(Locale::Es);
let current = I18n::locale(); // Returns Locale::Es
```

## Message Catalog Format

The crate supports gettext-style .po files for translations:

```po
# Spanish translation example
msgid "app.welcome"
msgstr "¡Bienvenido a Tachikoma!"

msgid "user.greeting"
msgstr "Hola, {name}!"

# Plural forms
msgid "file.count"
msgid_plural "file.count"
msgstr[0] "{count} archivo"
msgstr[1] "{count} archivos"
```

## Examples

Run the basic example to see i18n in action:

```bash
cargo run --example basic -p tachikoma-common-i18n
```

## Testing

The crate includes comprehensive tests covering:

- Locale parsing from various formats
- Translation fallback behavior
- Plural form selection
- Template substitution in macros
- Message catalog operations

```bash
cargo test -p tachikoma-common-i18n
```