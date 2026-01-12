# Tachikoma Architecture

Tachikoma is designed as a multi-process agentic coding platform using the Ralph Wiggum Loop pattern.

## Core Components

### Rust Backend (`crates/`)
- **tachikoma-common-core**: Shared types and utilities
- **tachikoma-test-utils**: Testing infrastructure

### Electron Shell (`electron/`)
- Main process coordination
- Security sandboxing
- Native system integration

### SvelteKit Frontend (`web/`)
- Modern web UI built with SvelteKit
- TypeScript throughout
- Component-based architecture

### IPC Communication
- Type-safe communication between Rust and Electron
- JSON-based message passing
- Async/await patterns

## The Ralph Wiggum Loop

1. **Big Intentions** → **Small Changes**
2. **Verify Everything** → **Iterate Quickly** 
3. **Fail Fast** → **Learn Faster**

## Project Structure

```
tachikoma/
├── crates/          # Rust backend crates
├── electron/        # Electron main process  
├── web/             # SvelteKit frontend
├── specs/           # Specifications (THE PIN)
└── docs/            # Documentation
```