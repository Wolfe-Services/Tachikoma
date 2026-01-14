# Spec 556: Simple Config Mode

**Phase:** 27 - Ralph Integration  
**Status:** Planned  
**Priority:** P2 - Medium  
**Dependencies:** 553 (Quickstart)
**Inspired By:** Ralph TUI's minimal config.toml

## Overview

Add a "simple mode" configuration with progressive disclosure - minimal defaults for beginners that expand to full power-user options.

## Problem Statement

Current Tachikoma config has many options:
```yaml
backend:
  brain: claude
  brain_model: claude-3-5-sonnet-20241022
  think_tank: o3
  think_tank_model: o3-mini
  forge_participants:
    - claude
    - gemini
    - codex

loop:
  max_iterations: 50
  redline_threshold: 150000
  stop_on:
    - test_fail_streak: 3
    - no_progress: 5
    - redline: true
  attended_by_default: true

policies:
  deploy_requires_tests: true
  auto_commit: true
  allowed_paths:
    - "**/*.rs"
    - "**/*.ts"
  blocked_commands:
    - rm -rf
    - sudo
# ... 50+ more options
```

Ralph TUI:
```toml
[agent]
default = "claude"

[tracker]
type = "json"
```

## Acceptance Criteria

- [ ] Define "simple" config schema with ~10 essential options
- [ ] Auto-expand simple config to full config at runtime
- [ ] Add `config_mode: simple | full` field
- [ ] Create migration from simple → full when user adds advanced options
- [ ] Update `tachikoma init` to generate simple config by default
- [ ] Add `tachikoma config upgrade` to convert simple → full
- [ ] Document both modes

## Simple Config Schema

```yaml
# .tachikoma/config.yaml (simple mode)
config_mode: simple

# The AI that does the coding
agent: claude

# How long to run before stopping
max_iterations: 50

# Watch and approve each step? (true = safer, false = autonomous)
attended: true

# Commit changes automatically?
auto_commit: true
```

That's it! Everything else uses smart defaults.

## Default Expansion

When Tachikoma loads a simple config, it expands to:

| Simple | Expands To |
|--------|------------|
| `agent: claude` | `backend.brain: claude`, `backend.brain_model: claude-sonnet-4-...` |
| `max_iterations: 50` | `loop.max_iterations: 50` |
| `attended: true` | `loop.attended_by_default: true` |
| `auto_commit: true` | `policies.auto_commit: true` |
| (not set) | `loop.redline_threshold: 150000` (default) |
| (not set) | `loop.stop_on: [test_fail_streak:3, ...]` (defaults) |

## Progressive Disclosure

When a user tries to set an advanced option on a simple config:

```bash
$ tachikoma config set loop.redline_threshold 200000

⚠️  'loop.redline_threshold' is an advanced option.

Your config is in simple mode. Would you like to:
  1. Upgrade to full config (recommended for power users)
  2. Keep simple mode and use this setting as an override
  3. Cancel

Choice [1]: 
```

## CLI Commands

```bash
# Show current config (in simple or full format)
tachikoma config show

# Set a value
tachikoma config set agent ollama

# Get a value (works with either mode)
tachikoma config get agent

# Upgrade simple → full
tachikoma config upgrade

# Validate config
tachikoma config validate

# Reset to defaults
tachikoma config reset
```

## Full Config (for power users)

When users run `tachikoma config upgrade`:

```yaml
# .tachikoma/config.yaml (full mode)
config_mode: full

backend:
  brain: claude
  brain_model: claude-sonnet-4-20250514
  think_tank: o3
  think_tank_model: o3-2025-04-16
  
  # For Spec Forge multi-model brainstorming
  forge_participants:
    - claude
    - gemini

loop:
  max_iterations: 50
  redline_threshold: 150000
  
  stop_on:
    - test_fail_streak: 3
    - no_progress: 5
    - cost_limit: 10.00
  
  attended_by_default: true
  pause_on_test_fail: true

policies:
  auto_commit: true
  commit_message_template: "feat({{spec.id}}): {{spec.name}}"
  
  # Security
  allowed_paths:
    - "**/*"
  blocked_commands:
    - "rm -rf /"
    - "sudo"
    - "curl | sh"

# Plugin overrides
plugins:
  agents:
    claude:
      max_tokens: 8192
  templates:
    use: default
```

## Implementation Details

### Files to Modify

1. **`crates/tachikoma-common-config/src/lib.rs`**
   - Add `ConfigMode` enum
   - Add `SimpleConfig` struct
   - Add expansion logic

2. **`crates/tachikoma-common-config/src/simple.rs`** (new)
   ```rust
   pub struct SimpleConfig {
       pub config_mode: ConfigMode,
       pub agent: String,
       pub max_iterations: usize,
       pub attended: bool,
       pub auto_commit: bool,
   }
   
   impl SimpleConfig {
       pub fn expand(&self) -> FullConfig { ... }
   }
   ```

3. **`crates/tachikoma-cli/src/commands/config.rs`**
   - Add upgrade command
   - Add progressive disclosure prompts

## Testing

- Test simple config loads and expands correctly
- Test full config loads unchanged
- Test upgrade preserves all settings
- Test CLI commands work with both modes
- Test invalid simple config shows helpful errors

## References

- Ralph TUI's config.toml
- Current config: `crates/tachikoma-common-config/`
- Existing config tests
