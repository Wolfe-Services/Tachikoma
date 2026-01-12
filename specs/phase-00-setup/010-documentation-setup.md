# 010 - Documentation Setup

**Phase:** 0 - Setup
**Spec ID:** 010
**Status:** Planned
**Dependencies:** 001-project-structure
**Estimated Context:** ~8% of Sonnet window

---

## Objective

Configure documentation generation for Rust (rustdoc), TypeScript (TypeDoc), and project documentation structure.

---

## Acceptance Criteria

- [ ] Rustdoc configured with workspace settings
- [ ] TypeDoc configured for web/
- [ ] docs/ folder structure created
- [ ] Documentation scripts in package.json
- [ ] README badges and links

---

## Implementation Details

### 1. Rustdoc Configuration

Add to workspace `Cargo.toml`:

```toml
[workspace.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[workspace.lints.rustdoc]
missing_docs = "warn"
```

Add to each crate's `lib.rs`:

```rust
//! Crate documentation goes here.
//!
//! # Examples
//!
//! ```rust
//! use tachikoma_common_core::Error;
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
```

### 2. TypeDoc Configuration (web/typedoc.json)

```json
{
  "entryPoints": ["src/lib"],
  "out": "../docs/api/web",
  "plugin": ["typedoc-plugin-markdown"],
  "readme": "none",
  "exclude": ["**/*.test.ts", "**/*.spec.ts"],
  "excludePrivate": true,
  "excludeInternal": true,
  "categorizeByGroup": true,
  "categoryOrder": ["Stores", "Components", "Utils", "*"]
}
```

### 3. Documentation Structure

```
docs/
├── README.md              # Documentation index
├── getting-started.md     # Quick start guide
├── architecture.md        # System architecture
├── api/
│   ├── rust/              # Generated rustdoc
│   └── web/               # Generated typedoc
├── guides/
│   ├── writing-specs.md
│   ├── using-forge.md
│   └── configuration.md
└── reference/
    ├── cli.md
    └── config.md
```

### 4. Documentation Scripts

Add to root `package.json`:

```json
{
  "scripts": {
    "docs": "npm run docs:rust && npm run docs:web",
    "docs:rust": "cargo doc --workspace --no-deps --document-private-items",
    "docs:web": "cd web && npx typedoc",
    "docs:serve": "npx serve docs"
  }
}
```

### 5. Root README Template

```markdown
# Tachikoma

[![CI](https://github.com/your-org/tachikoma/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/tachikoma/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

> Your squad of tireless AI coders

Tachikoma is an agentic coding platform that turns "big intentions" into
small verified changes using the Ralph Wiggum Loop pattern.

## Quick Start

\`\`\`bash
# Clone and install
git clone https://github.com/your-org/tachikoma
cd tachikoma
npm install

# Run development server
npm run dev

# Build for production
npm run build
\`\`\`

## Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture](docs/architecture.md)
- [CLI Reference](docs/reference/cli.md)
- [API Docs](docs/api/)

## Project Structure

\`\`\`
tachikoma/
├── crates/          # Rust backend crates
├── electron/        # Electron main process
├── web/             # SvelteKit frontend
├── specs/           # Specifications (THE PIN)
└── docs/            # Documentation
\`\`\`

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
```

---

## Testing Requirements

1. `npm run docs:rust` generates rustdoc
2. `npm run docs:web` generates typedoc
3. All public APIs documented
4. No broken links in docs

---

## Related Specs

- Depends on: [001-project-structure.md](001-project-structure.md)
- Related: [511-doc-structure.md](../phase-24-docs/511-doc-structure.md)

---

## Phase 0 Complete

This completes Phase 0: Project Setup. The project now has:

- Complete directory structure
- Rust workspace configured
- Electron shell with security defaults
- SvelteKit frontend
- IPC bridge between processes
- Development tooling
- Build system
- Test infrastructure
- CI/CD pipeline
- Documentation structure

**Next Phase:** [011-common-core-types.md](../phase-01-common/011-common-core-types.md)
