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

# Release tagging
.PHONY: tag tag-signed tag-list tag-delete

tag:
	@read -p "Version: " VERSION && ./scripts/tag-release.sh $$VERSION

tag-signed:
	@read -p "Version: " VERSION && ./scripts/tag-release-signed.sh $$VERSION

tag-list:
	@git tag -l "v*" --sort=-version:refname | head -10

tag-delete:
	@read -p "Tag to delete (e.g., v1.0.0): " TAG && \
		git tag -d $$TAG && \
		echo "Local tag deleted. To delete remote: git push origin :refs/tags/$$TAG"