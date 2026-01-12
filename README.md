# Tachikoma

**Agentic Coding Platform**

A powerful development environment that combines AI agents with modern tooling to create an intelligent, collaborative coding experience.

## Overview

Tachikoma is a Rust + Electron + Svelte application that provides:

- **AI-Powered Development**: Intelligent code completion, generation, and review
- **Collaborative Workflows**: Multi-agent coordination for complex tasks
- **Integrated Toolchain**: Built-in testing, building, and deployment
- **Extensible Architecture**: Plugin system for custom functionality

## Architecture

- **Backend**: Rust for performance-critical operations
- **Frontend**: SvelteKit for modern, reactive UI
- **Desktop**: Electron for cross-platform desktop experience
- **AI Integration**: Multiple AI provider support with unified interfaces

## Quick Start

```bash
# Install dependencies
npm install

# Start development environment
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

## Project Structure

```
tachikoma/
├── crates/          # Rust workspace crates
├── electron/        # Electron main process
├── web/             # SvelteKit frontend
├── specs/           # Specification files
├── scripts/         # Build and utility scripts
└── docs/            # Documentation
```

## Development

See [AGENTS.md](AGENTS.md) for AI assistant instructions and development patterns.

## License

MIT License - see [LICENSE](LICENSE) for details.