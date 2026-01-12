.PHONY: all dev build clean test lint
SHELL := /bin/bash

# Default target
all: build

# Development
dev:
	npm run dev

# Build all
build: build-rust build-web build-electron

build-rust:
	source ~/.cargo/env 2>/dev/null || true && cargo build --release --workspace

build-web:
	cd web && npm run build

build-electron:
	cd electron && npm run build

# Clean all
clean:
	source ~/.cargo/env 2>/dev/null || true && cargo clean
	rm -rf web/dist
	rm -rf electron/dist
	rm -rf electron/out

# Test all
test: test-rust test-web

test-rust:
	source ~/.cargo/env 2>/dev/null || true && cargo test --workspace

test-web:
	cd web && npm test

# Lint all
lint: lint-rust lint-web

lint-rust:
	source ~/.cargo/env 2>/dev/null || true && cargo clippy --workspace -- -D warnings
	source ~/.cargo/env 2>/dev/null || true && cargo fmt --all -- --check

lint-web:
	cd web && npm run lint
	cd web && npm run check

# Package for distribution
package:
	cd electron && npm run package

# Install dependencies
install:
	npm install
	cd web && npm install
	cd electron && npm install