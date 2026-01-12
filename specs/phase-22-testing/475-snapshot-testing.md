# 475 - Snapshot Testing

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 475
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Implement snapshot testing using insta for Rust and vitest snapshots for TypeScript, enabling automatic verification of complex output structures against stored baseline snapshots.

---

## Acceptance Criteria

- [ ] insta configured for all Rust crates with YAML/JSON support
- [ ] Vitest snapshots configured for TypeScript tests
- [ ] Snapshot review workflow documented
- [ ] Inline snapshots supported where appropriate
- [ ] Snapshot naming conventions established
- [ ] CI fails on pending snapshot reviews

---

## Implementation Details

### 1. Insta Configuration for Rust

Create `crates/tachikoma-test-harness/src/snapshot.rs`:

```rust
//! Snapshot testing utilities using insta.
//!
//! Snapshot tests capture complex output and compare against stored baselines.
//! Use `cargo insta review` to interactively accept/reject changes.

use insta::{assert_json_snapshot, assert_yaml_snapshot, assert_debug_snapshot};
use serde::Serialize;

/// Configuration for snapshot behavior
pub struct SnapshotConfig {
    /// Snapshot directory relative to crate root
    pub snapshot_dir: &'static str,
    /// Whether to sort keys in JSON/YAML output
    pub sort_keys: bool,
    /// Redactions to apply (hide dynamic values)
    pub redactions: Vec<(&'static str, &'static str)>,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            snapshot_dir: "snapshots",
            sort_keys: true,
            redactions: vec![
                ("[].id", "[id]"),
                ("[].created_at", "[timestamp]"),
                ("[].updated_at", "[timestamp]"),
            ],
        }
    }
}

/// Helper to create consistent snapshot settings
#[macro_export]
macro_rules! snapshot_settings {
    () => {{
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path("snapshots");
        settings.set_prepend_module_to_snapshot(false);
        settings.set_sort_maps(true);
        settings
    }};
}

/// Assert a JSON snapshot with standard settings
#[macro_export]
macro_rules! assert_json {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_json_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_json_snapshot!($name, $value);
        });
    }};
}

/// Assert a YAML snapshot with standard settings
#[macro_export]
macro_rules! assert_yaml {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_yaml_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_yaml_snapshot!($name, $value);
        });
    }};
}

/// Assert a debug snapshot with standard settings
#[macro_export]
macro_rules! assert_debug {
    ($value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_debug_snapshot!($value);
        });
    }};
    ($name:expr, $value:expr) => {{
        let settings = $crate::snapshot_settings!();
        settings.bind(|| {
            insta::assert_debug_snapshot!($name, $value);
        });
    }};
}

/// Redact dynamic fields in snapshots
pub fn with_redactions<F, R>(redactions: &[(&str, &str)], f: F) -> R
where
    F: FnOnce() -> R,
{
    let mut settings = insta::Settings::clone_current();
    for (selector, placeholder) in redactions {
        settings.add_redaction(selector, placeholder);
    }
    settings.bind(f)
}
```

### 2. Example Rust Snapshot Tests

Create `crates/tachikoma-common-core/tests/snapshot_tests.rs`:

```rust
//! Snapshot tests for core types.

use serde::{Deserialize, Serialize};
use tachikoma_test_harness::{assert_json, assert_yaml, assert_debug, with_redactions};

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    data: ResponseData,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseData {
    items: Vec<Item>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    version: String,
    timestamp: String,
}

fn create_test_response() -> ApiResponse {
    ApiResponse {
        status: "success".into(),
        data: ResponseData {
            items: vec![
                Item { id: "1".into(), name: "First".into(), value: 100 },
                Item { id: "2".into(), name: "Second".into(), value: 200 },
            ],
            total: 2,
        },
        metadata: Metadata {
            version: "1.0.0".into(),
            timestamp: "2024-01-15T10:30:00Z".into(),
        },
    }
}

#[test]
fn test_api_response_json_snapshot() {
    let response = create_test_response();
    assert_json!(response);
}

#[test]
fn test_api_response_yaml_snapshot() {
    let response = create_test_response();
    assert_yaml!(response);
}

#[test]
fn test_api_response_with_redactions() {
    let response = create_test_response();

    with_redactions(&[
        (".metadata.timestamp", "[timestamp]"),
        (".data.items[].id", "[id]"),
    ], || {
        insta::assert_json_snapshot!("api_response_redacted", response);
    });
}

#[test]
fn test_named_snapshots() {
    let items = vec![
        Item { id: "a".into(), name: "Alpha".into(), value: 1 },
        Item { id: "b".into(), name: "Beta".into(), value: 2 },
    ];

    assert_json!("item_list", items);
}

#[test]
fn test_inline_snapshot() {
    let simple = serde_json::json!({
        "key": "value",
        "number": 42
    });

    insta::assert_json_snapshot!(simple, @r###"
    {
      "key": "value",
      "number": 42
    }
    "###);
}

mod error_snapshots {
    use super::*;

    #[derive(Debug, Serialize)]
    struct ErrorResponse {
        code: String,
        message: String,
        details: Option<String>,
    }

    #[test]
    fn test_error_not_found() {
        let error = ErrorResponse {
            code: "NOT_FOUND".into(),
            message: "Resource not found".into(),
            details: Some("The requested item does not exist".into()),
        };
        assert_json!("error_not_found", error);
    }

    #[test]
    fn test_error_validation() {
        let error = ErrorResponse {
            code: "VALIDATION_ERROR".into(),
            message: "Invalid input".into(),
            details: None,
        };
        assert_json!("error_validation", error);
    }
}
```

### 3. TypeScript Snapshot Testing

Create `web/src/test/snapshot/setup.ts`:

```typescript
/**
 * Snapshot testing utilities for TypeScript/Vitest.
 */

import { expect } from 'vitest';

/**
 * Custom serializer for Svelte components
 */
export const svelteSerializer = {
  test: (val: unknown) => val && typeof val === 'object' && 'component' in val,
  serialize: (val: { component: unknown; props: unknown }) => {
    return `<${(val.component as { name?: string }).name || 'Component'} ${JSON.stringify(val.props, null, 2)} />`;
  },
};

/**
 * Redact dynamic values in snapshots
 */
export function redact<T extends object>(obj: T, paths: string[]): T {
  const clone = JSON.parse(JSON.stringify(obj));

  for (const path of paths) {
    const parts = path.split('.');
    let current = clone;

    for (let i = 0; i < parts.length - 1; i++) {
      if (current[parts[i]] === undefined) break;
      current = current[parts[i]];
    }

    const lastPart = parts[parts.length - 1];
    if (current && lastPart in current) {
      current[lastPart] = `[${lastPart}]`;
    }
  }

  return clone;
}

/**
 * Create a stable snapshot by sorting keys
 */
export function stableSnapshot<T>(obj: T): string {
  return JSON.stringify(obj, Object.keys(obj as object).sort(), 2);
}
```

Create `web/src/test/snapshot/examples.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import { redact, stableSnapshot } from './setup';

describe('Snapshot Tests', () => {
  // ============================================
  // Pattern: Basic Object Snapshots
  // ============================================

  describe('API Response Snapshots', () => {
    it('should match success response', () => {
      const response = {
        status: 'success',
        data: {
          users: [
            { id: '1', name: 'Alice', role: 'admin' },
            { id: '2', name: 'Bob', role: 'user' },
          ],
          total: 2,
        },
      };

      expect(response).toMatchSnapshot();
    });

    it('should match error response', () => {
      const error = {
        status: 'error',
        code: 'VALIDATION_ERROR',
        message: 'Invalid input provided',
        fields: ['email', 'password'],
      };

      expect(error).toMatchSnapshot();
    });
  });

  // ============================================
  // Pattern: Inline Snapshots
  // ============================================

  describe('Inline Snapshots', () => {
    it('should match inline snapshot', () => {
      const config = {
        theme: 'dark',
        language: 'en',
        notifications: true,
      };

      expect(config).toMatchInlineSnapshot(`
        {
          "language": "en",
          "notifications": true,
          "theme": "dark",
        }
      `);
    });
  });

  // ============================================
  // Pattern: Redacted Snapshots
  // ============================================

  describe('Redacted Snapshots', () => {
    it('should redact dynamic values', () => {
      const response = {
        id: 'uuid-12345-abcde',
        createdAt: new Date().toISOString(),
        name: 'Test Item',
        version: 1,
      };

      const redacted = redact(response, ['id', 'createdAt']);

      expect(redacted).toMatchSnapshot();
    });
  });

  // ============================================
  // Pattern: File/Template Snapshots
  // ============================================

  describe('Template Snapshots', () => {
    function generateTemplate(data: { title: string; items: string[] }): string {
      return `
# ${data.title}

${data.items.map((item, i) => `${i + 1}. ${item}`).join('\n')}
      `.trim();
    }

    it('should match generated template', () => {
      const template = generateTemplate({
        title: 'Shopping List',
        items: ['Apples', 'Bread', 'Milk'],
      });

      expect(template).toMatchSnapshot();
    });
  });

  // ============================================
  // Pattern: HTML/Component Snapshots
  // ============================================

  describe('HTML Snapshots', () => {
    function renderCard(props: { title: string; content: string }): string {
      return `
<div class="card">
  <h2 class="card-title">${props.title}</h2>
  <p class="card-content">${props.content}</p>
</div>
      `.trim();
    }

    it('should match rendered HTML', () => {
      const html = renderCard({
        title: 'Welcome',
        content: 'This is a test card',
      });

      expect(html).toMatchSnapshot();
    });
  });

  // ============================================
  // Pattern: Array/Collection Snapshots
  // ============================================

  describe('Collection Snapshots', () => {
    it('should match sorted array', () => {
      const items = [
        { name: 'Charlie', score: 85 },
        { name: 'Alice', score: 92 },
        { name: 'Bob', score: 78 },
      ].sort((a, b) => a.name.localeCompare(b.name));

      expect(items).toMatchSnapshot();
    });
  });
});
```

### 4. Snapshot Review Workflow

Create `docs/testing/snapshot-workflow.md`:

```markdown
# Snapshot Testing Workflow

## Rust (insta)

### Reviewing Snapshots

```bash
# Review all pending snapshots interactively
cargo insta review

# Accept all pending snapshots
cargo insta accept

# Reject all pending snapshots
cargo insta reject
```

### Updating Snapshots

```bash
# Update snapshots during test run
cargo test -- --update-snapshots

# Or use insta's update mode
INSTA_UPDATE=always cargo test
```

### CI Configuration

```yaml
- name: Check for pending snapshots
  run: cargo insta test --check
```

## TypeScript (Vitest)

### Updating Snapshots

```bash
# Update all snapshots
npm run test -- --update

# Update specific test file's snapshots
npm run test -- path/to/test.ts --update
```

### CI Configuration

```yaml
- name: Run tests (fail on snapshot mismatch)
  run: npm test -- --run
```

## Best Practices

1. **Review carefully**: Always review snapshot changes before accepting
2. **Use redactions**: Redact timestamps, IDs, and other dynamic values
3. **Name snapshots**: Use descriptive names for clarity
4. **Keep small**: Prefer multiple small snapshots over one large one
5. **Commit snapshots**: Snapshot files should be committed to version control
```

---

## Testing Requirements

1. `cargo insta test` passes with no pending snapshots
2. `npm test` passes with no snapshot mismatches
3. Snapshot files are properly organized in `snapshots/` directories
4. Redactions hide dynamic values consistently
5. CI fails when snapshots need review

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [476-mock-backends.md](476-mock-backends.md)
- Related: [482-test-reporting.md](482-test-reporting.md)
