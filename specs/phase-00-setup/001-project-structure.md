# 001 - Project Structure

**Phase:** 0 - Setup
**Spec ID:** 001
**Status:** Planned
**Dependencies:** None (First spec)
**Estimated Context:** ~15% of Sonnet window

---

## Objective

Initialize the Tachikoma project directory structure with all necessary folders, configuration files, and workspace setup for a Rust + Electron + Svelte application.

---

## Acceptance Criteria

- [x] Root directory contains all required folders
- [x] `.gitignore` configured for Rust, Node, Electron
- [x] `README.md` with project overview
- [x] `LICENSE` file present
- [x] `.editorconfig` for consistent formatting
- [x] `AGENTS.md` (symlinked as `CLAUDE.md`) with agent instructions

---

## Directory Structure

```
tachikoma/
├── .github/                    # GitHub workflows, templates
│   ├── workflows/
│   └── ISSUE_TEMPLATE/
├── .tachikoma/                 # Tachikoma configuration
│   ├── config.yaml
│   └── forge/
├── crates/                     # Rust workspace crates
│   └── .gitkeep
├── electron/                   # Electron main process
│   ├── main/
│   ├── preload/
│   └── package.json
├── web/                        # SvelteKit frontend
│   ├── src/
│   ├── static/
│   └── package.json
├── specs/                      # Specification files (THE PIN)
│   └── README.md
├── scripts/                    # Build and utility scripts
├── docs/                       # Documentation
├── .editorconfig
├── .gitignore
├── AGENTS.md
├── CLAUDE.md -> AGENTS.md      # Symlink
├── Cargo.toml                  # Rust workspace root
├── package.json                # Root package.json for scripts
├── LICENSE
├── README.md
└── prompt.md                   # Ralph loop prompt
```

---

## Implementation Details

### 1. Create Root Files

**`.gitignore`:**
```gitignore
# Rust
/target/
**/*.rs.bk
Cargo.lock

# Node
node_modules/
.npm
*.log

# Electron
dist/
out/
release/

# Build
*.dmg
*.exe
*.AppImage
*.deb

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Env
.env
.env.local
*.pem

# Tachikoma
.tachikoma/forge/sessions/
```

**`.editorconfig`:**
```ini
root = true

[*]
indent_style = space
indent_size = 2
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.rs]
indent_size = 4

[*.md]
trim_trailing_whitespace = false

[Makefile]
indent_style = tab
```

### 2. Create AGENTS.md

```markdown
# Tachikoma Agent Instructions

You are a Tachikoma - a curious, helpful AI coding assistant.

## Core Behaviors

1. **One mission per context** - Focus on the current task only
2. **Study specs first** - Always read relevant specs before coding
3. **Follow patterns** - Use existing code patterns in the codebase
4. **Test everything** - Write tests, run tests, fix tests
5. **Update plans** - Mark checkboxes when tasks complete

## File Locations

- Specs: `specs/README.md` (lookup table)
- Config: `.tachikoma/config.yaml`
- Rust: `crates/`
- Frontend: `web/`
- Electron: `electron/`

## Commands

- Build: `cargo build` (Rust), `npm run build` (frontend)
- Test: `cargo test`, `npm test`
- Lint: `cargo clippy`, `npm run lint`

## On Getting Stuck

1. Reduce context
2. Reduce tools
3. Reboot (fresh context)
4. Re-anchor to the spec
```

### 3. Create Symlink

```bash
ln -s AGENTS.md CLAUDE.md
```

### 4. Root package.json

```json
{
  "name": "tachikoma",
  "version": "0.1.0",
  "private": true,
  "description": "Agentic coding platform",
  "scripts": {
    "dev": "concurrently \"npm run dev:electron\" \"npm run dev:web\"",
    "dev:web": "cd web && npm run dev",
    "dev:electron": "cd electron && npm run dev",
    "build": "npm run build:web && npm run build:electron",
    "build:web": "cd web && npm run build",
    "build:electron": "cd electron && npm run build",
    "test": "npm run test:rust && npm run test:web",
    "test:rust": "cargo test",
    "test:web": "cd web && npm test"
  },
  "devDependencies": {
    "concurrently": "^8.2.0"
  }
}
```

---

## Testing Requirements

1. Verify all directories exist
2. Verify `.gitignore` excludes expected patterns
3. Verify symlink `CLAUDE.md` -> `AGENTS.md` works
4. Verify root `package.json` scripts are valid

---

## Related Specs

- Next: [002-rust-workspace.md](002-rust-workspace.md)
- Depends on this: All subsequent specs

---

## Notes

This is the foundation spec. All other specs assume this structure exists.
