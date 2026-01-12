# Tachikoma

[![CI](https://github.com/your-org/tachikoma/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/tachikoma/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

> Your squad of tireless AI coders

Tachikoma is an agentic coding platform that turns "big intentions" into
small verified changes using the Ralph Wiggum Loop pattern.

## Quick Start

```bash
# Clone and install
git clone https://github.com/your-org/tachikoma
cd tachikoma
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture](docs/architecture.md)
- [CLI Reference](docs/reference/cli.md)
- [API Docs](docs/api/)

## Project Structure

```
tachikoma/
├── crates/          # Rust backend crates
├── electron/        # Electron main process
├── web/             # SvelteKit frontend
├── specs/           # Specifications (THE PIN)
└── docs/            # Documentation
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.