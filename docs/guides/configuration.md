# Configuration Guide

This guide covers how to configure Tachikoma for your environment.

## Environment Variables

Create a `.env` file based on `.env.example`:

```bash
# Development settings
NODE_ENV=development
VITE_DEV_SERVER_PORT=5173

# Rust settings  
RUST_LOG=debug

# Application settings
TACHIKOMA_CONFIG_PATH=./config.toml
```

## Configuration Files

### Rust Configuration
- `Cargo.toml` - Workspace settings
- `clippy.toml` - Linting configuration
- `rustfmt.toml` - Code formatting

### TypeScript Configuration
- `tsconfig.json` - TypeScript compiler settings
- `vite.config.ts` - Build tool configuration
- `.eslintrc.json` - Linting rules

## Development Setup

1. Install recommended VS Code extensions
2. Configure Rust toolchain
3. Set up Node.js environment
4. Configure Git hooks (coming soon)