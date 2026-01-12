# 008 - Test Infrastructure Setup

**Phase:** 0 - Setup
**Spec ID:** 008
**Status:** Planned
**Dependencies:** 002-rust-workspace, 004-svelte-integration
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Set up testing infrastructure for Rust (cargo test, proptest), TypeScript (Vitest), and end-to-end (Playwright) testing.

---

## Acceptance Criteria

- [ ] Rust unit tests configured
- [ ] Rust property-based tests with proptest
- [ ] Vitest configured for Svelte components
- [ ] Test utilities and helpers created
- [ ] Coverage reporting configured
- [ ] Test scripts in package.json

---

## Implementation Details

### 1. Rust Test Configuration

Add to workspace `Cargo.toml`:

```toml
[workspace.dependencies]
# Testing
proptest = "1.4"
insta = { version = "1.34", features = ["yaml"] }
test-case = "3.3"
mockall = "0.12"
tokio-test = "0.4"
```

### 2. Rust Test Helper Crate

Create `crates/tachikoma-test-utils/Cargo.toml`:

```toml
[package]
name = "tachikoma-test-utils"
version.workspace = true
edition.workspace = true

[dependencies]
tempfile = "3.9"
tokio = { workspace = true, features = ["rt", "macros"] }

[dev-dependencies]
proptest.workspace = true
```

Create `crates/tachikoma-test-utils/src/lib.rs`:

```rust
//! Test utilities for Tachikoma crates.

use std::path::PathBuf;
use tempfile::TempDir;

/// Creates a temporary directory that is cleaned up on drop.
pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Creates a temporary file with given content.
pub fn temp_file(content: &str) -> (TempDir, PathBuf) {
    let dir = temp_dir();
    let path = dir.path().join("test_file");
    std::fs::write(&path, content).expect("Failed to write temp file");
    (dir, path)
}

/// Macro for async tests with tokio runtime.
#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            $body
        }
    };
}

/// Assert that a Result is Ok and return the value.
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a Result is Err.
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
            Err(_) => {}
        }
    };
}
```

### 3. Vitest Configuration (web/vitest.config.ts)

```typescript
import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],

  test: {
    include: ['src/**/*.{test,spec}.{js,ts}'],
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/'
      ]
    }
  }
});
```

### 4. Test Setup (web/src/test/setup.ts)

```typescript
import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock window.tachikoma for tests
vi.stubGlobal('window', {
  tachikoma: {
    platform: 'darwin',
    invoke: vi.fn().mockResolvedValue({}),
    on: vi.fn(),
    off: vi.fn()
  }
});

// Reset mocks between tests
beforeEach(() => {
  vi.clearAllMocks();
});
```

### 5. Test Utilities (web/src/test/utils.ts)

```typescript
import { render, type RenderResult } from '@testing-library/svelte';
import type { ComponentProps, SvelteComponent } from 'svelte';

/**
 * Render a Svelte component with default test providers.
 */
export function renderWithProviders<T extends SvelteComponent>(
  component: new (...args: any[]) => T,
  props?: ComponentProps<T>
): RenderResult<T> {
  return render(component, { props });
}

/**
 * Wait for next tick.
 */
export function tick(): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, 0));
}

/**
 * Mock IPC invoke response.
 */
export function mockIpcInvoke(channel: string, response: unknown): void {
  (window.tachikoma.invoke as jest.Mock).mockImplementation(
    async (ch: string) => {
      if (ch === channel) return response;
      return {};
    }
  );
}
```

### 6. Example Component Test (web/src/lib/components/Button.test.ts)

```typescript
import { describe, it, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Button from './Button.svelte';

describe('Button', () => {
  it('renders with text', () => {
    const { getByRole } = render(Button, { props: { label: 'Click me' } });
    expect(getByRole('button')).toHaveTextContent('Click me');
  });

  it('handles click events', async () => {
    let clicked = false;
    const { getByRole, component } = render(Button, {
      props: { label: 'Click' }
    });

    component.$on('click', () => { clicked = true; });
    await fireEvent.click(getByRole('button'));

    expect(clicked).toBe(true);
  });

  it('can be disabled', () => {
    const { getByRole } = render(Button, {
      props: { label: 'Disabled', disabled: true }
    });
    expect(getByRole('button')).toBeDisabled();
  });
});
```

### 7. Rust Property Test Example

```rust
// In any crate's tests
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_string_roundtrip(s in "\\PC*") {
        // Property: encoding then decoding returns original
        let encoded = encode(&s);
        let decoded = decode(&encoded)?;
        prop_assert_eq!(s, decoded);
    }

    #[test]
    fn test_path_normalization(
        parts in prop::collection::vec("[a-z]+", 1..5)
    ) {
        let path = parts.join("/");
        let normalized = normalize_path(&path);
        // Property: normalized path has no double slashes
        prop_assert!(!normalized.contains("//"));
    }
}
```

---

## Testing Requirements

1. `cargo test --workspace` runs all Rust tests
2. `cd web && npm test` runs all Vitest tests
3. `cd web && npm run test:coverage` generates coverage
4. Property tests generate meaningful test cases

---

## Related Specs

- Depends on: [002-rust-workspace.md](002-rust-workspace.md), [004-svelte-integration.md](004-svelte-integration.md)
- Next: [009-ci-pipeline.md](009-ci-pipeline.md)
- Related: [471-test-harness.md](../phase-22-testing/471-test-harness.md)
