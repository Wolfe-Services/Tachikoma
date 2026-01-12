# Spec 516: Configuration Reference

## Overview
Complete reference documentation for all Tachikoma configuration options, file formats, environment variables, and precedence rules.

## Requirements

### Configuration File Reference
- YAML/TOML/JSON format support
- Complete option listing with types
- Default values for each option
- Valid value ranges/enums
- Example configurations

### File Locations
- System-wide: /etc/tachikoma/
- User-level: ~/.config/tachikoma/
- Project-level: .tachikoma/
- Environment override: TACHIKOMA_CONFIG
- Precedence order documentation

### Configuration Sections
```yaml
# Core settings
core:
  log_level: info
  data_dir: ~/.tachikoma/data

# Agent settings
agent:
  max_concurrent: 5
  timeout: 30m
  retry_count: 3

# Server settings
server:
  listen_addr: 127.0.0.1:8080
  tls_enabled: true

# Storage settings
storage:
  backend: sqlite
  path: ~/.tachikoma/tachikoma.db

# Telemetry settings
telemetry:
  enabled: true
  endpoint: ""
```

### Environment Variables
- TACHIKOMA_LOG_LEVEL
- TACHIKOMA_DATA_DIR
- TACHIKOMA_CONFIG
- TACHIKOMA_API_KEY
- Complete mapping to config keys

### Secret Management
- Secret file references
- Environment variable secrets
- Vault integration config
- Encryption at rest settings

### Validation
- Config validation command
- Schema validation errors
- Migration between versions
- Deprecated option warnings

## Generated Artifacts
```
docs/reference/configuration/
├── index.md
├── core.md
├── agent.md
├── server.md
├── storage.md
├── telemetry.md
├── environment.md
├── secrets.md
└── schema.json
```

## Dependencies
- Spec 511: Documentation Structure

## Verification
- [ ] All options documented
- [ ] Defaults are accurate
- [ ] Examples validate
- [ ] Schema is complete
- [ ] Env vars mapped correctly
