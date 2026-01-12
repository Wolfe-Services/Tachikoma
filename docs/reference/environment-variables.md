# Environment Variables

Tachikoma supports configuration through environment variables and `.env` files. This document describes all available environment variables and their usage.

## Loading Order

Environment variables are loaded in the following order (later values override earlier ones):

1. System environment variables
2. `.env` file (if present)
3. `.env.local` file (if present)
4. `.env.{NODE_ENV}` file (e.g., `.env.development`, `.env.production`)

## API Keys

These variables store API keys for various AI backend services.

### ANTHROPIC_API_KEY

- **Required for:** Claude models
- **Format:** `sk-ant-api03-...`
- **Description:** API key for Anthropic's Claude models
- **Example:** `ANTHROPIC_API_KEY=sk-ant-api03-abcd1234...`

### OPENAI_API_KEY

- **Required for:** OpenAI models (GPT, Codex)
- **Format:** `sk-...`
- **Description:** API key for OpenAI models
- **Example:** `OPENAI_API_KEY=sk-abcd1234...`

### GOOGLE_API_KEY

- **Required for:** Google AI models (Gemini)
- **Format:** API key string
- **Description:** API key for Google AI services
- **Example:** `GOOGLE_API_KEY=AIza...`

## Configuration

These variables control Tachikoma's behavior and configuration.

### TACHIKOMA_CONFIG_PATH

- **Required:** No
- **Default:** `.tachikoma/config.yaml`
- **Description:** Path to the Tachikoma configuration file
- **Example:** `TACHIKOMA_CONFIG_PATH=/custom/path/config.yaml`

### TACHIKOMA_LOG_LEVEL

- **Required:** No
- **Default:** `info`
- **Values:** `trace`, `debug`, `info`, `warn`, `error`
- **Description:** Log level for Tachikoma output
- **Example:** `TACHIKOMA_LOG_LEVEL=debug`

### TACHIKOMA_DATA_DIR

- **Required:** No
- **Default:** `.tachikoma/data`
- **Description:** Directory for storing Tachikoma data files
- **Example:** `TACHIKOMA_DATA_DIR=/data/tachikoma`

## Development

These variables are used for development and debugging.

### NODE_ENV

- **Required:** No
- **Default:** `development`
- **Values:** `development`, `production`, `test`
- **Description:** Environment mode (affects .env file loading)
- **Example:** `NODE_ENV=production`

### RUST_LOG

- **Required:** No
- **Default:** `info`
- **Description:** Rust logging configuration (env_logger format)
- **Example:** `RUST_LOG=tachikoma=debug`

### RUST_BACKTRACE

- **Required:** No
- **Default:** `0`
- **Values:** `0`, `1`, `full`
- **Description:** Enable Rust backtraces on panic
- **Example:** `RUST_BACKTRACE=1`

## Web Development

These variables are specific to the web UI components.

### VITE_DEV_SERVER_URL

- **Required:** No (development only)
- **Default:** `http://localhost:5173`
- **Description:** URL for the Vite development server
- **Example:** `VITE_DEV_SERVER_URL=http://localhost:3000`

### ELECTRON_ENABLE_LOGGING

- **Required:** No
- **Default:** `0`
- **Values:** `0`, `1`
- **Description:** Enable verbose Electron logging
- **Example:** `ELECTRON_ENABLE_LOGGING=1`

## Usage Examples

### Basic Development Setup

```bash
# .env.development
NODE_ENV=development
RUST_LOG=tachikoma=debug
RUST_BACKTRACE=1
ANTHROPIC_API_KEY=sk-ant-api03-your-key-here
TACHIKOMA_LOG_LEVEL=debug
```

### Production Setup

```bash
# .env.production
NODE_ENV=production
RUST_LOG=info
ANTHROPIC_API_KEY=sk-ant-api03-your-key-here
OPENAI_API_KEY=sk-your-openai-key-here
TACHIKOMA_CONFIG_PATH=/etc/tachikoma/config.yaml
TACHIKOMA_DATA_DIR=/var/lib/tachikoma
```

### Testing Environment

```bash
# .env.test
NODE_ENV=test
RUST_LOG=warn
TACHIKOMA_DATA_DIR=/tmp/tachikoma-test
# No API keys needed for tests
```

## Security Notes

- **Never commit API keys to version control**
- Use `.env.local` or `.env.{environment}` files for sensitive values
- The `.env.example` file should contain empty values as templates
- In production, consider using a secrets manager instead of files

## Validation

Tachikoma validates required environment variables at startup:

- API keys are checked when the corresponding backend is used
- Invalid values (e.g., malformed integers) will cause startup errors
- Missing required variables will display helpful error messages

## API

The environment variables can be accessed programmatically:

```rust
use tachikoma_common_config::{Environment, ApiKeys, vars};

// Initialize environment (loads .env files)
Environment::init()?;

// Get API keys
let claude_key = ApiKeys::anthropic();
let openai_key = ApiKeys::openai();

// Get configuration values
let config_path = Environment::get_or(vars::TACHIKOMA_CONFIG_PATH, ".tachikoma/config.yaml");
let log_level = Environment::get(vars::TACHIKOMA_LOG_LEVEL);

// Type-safe parsing
let is_dev = Environment::is_development();
let backtrace_enabled = Environment::get_bool(vars::RUST_BACKTRACE);
```