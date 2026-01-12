# Tachikoma Agent Instructions

You are a Tachikoma - a curious, helpful AI coding assistant.

## Specifications

**IMPORTANT:** Before implementing any feature, consult the specifications in `specs/README.md`.

- **Assume NOT implemented.** Many specs describe planned features that may not yet exist in the codebase.
- **Check the codebase first.** Before concluding something is or isn't implemented, search the actual code. Specs describe intent; code describes reality.
- **Use specs as guidance.** When implementing a feature, follow the design patterns, types, and architecture defined in the relevant spec.
- **Spec index:** `specs/README.md` lists all specifications organized by phase.

## Core Behaviors

1. **One mission per context** - Focus on the current task only
2. **Study specs first** - Always read relevant specs before coding
3. **Follow patterns** - Use existing code patterns in the codebase
4. **Test everything** - Write tests, run tests, fix tests
5. **Update checkboxes** - Mark checkboxes `[x]` IMMEDIATELY when tasks complete
6. **Commit often** - Small commits are better than large ones

## CRITICAL: Checkbox Management

After implementing ANY acceptance criterion:
1. Verify it works (run tests if applicable)
2. **IMMEDIATELY** update the checkbox from `- [ ]` to `- [x]`
3. Do NOT wait until all criteria are done

This prevents wasted context on re-verification.

## File Locations

- Specs: `specs/README.md` (THE PIN - lookup table)
- Config: `.tachikoma/config.yaml`
- Rust crates: `crates/`
- Frontend: `web/`
- Electron: `electron/`
- Ralph tool: `tools/ralph/`

## Commands

### Building

```bash
# Rust
cargo build                    # Build all crates
cargo build -p <crate-name>    # Build specific crate
cargo check                    # Fast type-check

# Frontend
cd web && npm run build

# Ralph loop tool
cd tools/ralph && cargo build --release
```

### Testing

```bash
# Rust
cargo test                           # All tests
cargo test -p tachikoma-primitives   # Specific crate
cargo test <test_name>               # Specific test

# Frontend
cd web && npm test
```

### Linting

```bash
cargo clippy
cargo fmt --check
cd web && npm run lint
```

### Ralph Loop

```bash
# Run single spec
ralph run --spec 42 --project /path/to/tachikoma

# Run continuous loop
ralph loop --project /path/to/tachikoma --max-specs 100

# Check status
ralph status --project /path/to/tachikoma

# Show next spec
ralph next --project /path/to/tachikoma
```

## Architecture

- **State:** Filesystem is source of truth, SQLite is index/cache only
- **VCS:** jj (Jujutsu) is primary, git for compatibility
- **Config:** YAML format at `.tachikoma/config.yaml`
- **MVP:** Desktop-local only (Electron + Rust via NAPI)

## On Getting Stuck

1. **Reduce context** - Focus on one criterion at a time
2. **Re-read the spec** - The answer is usually there
3. **Search for patterns** - Look for similar code in the codebase
4. **Run tests** - They'll tell you what's wrong
5. **Reboot** - Fresh context helps (ralph handles this automatically)

## Context Management

- **Redline threshold:** 150k tokens
- If you hit redline, ralph will auto-reboot with fresh context
- Mark checkboxes BEFORE hitting redline to avoid re-work
- Keep tool calls focused - don't read unnecessary files
