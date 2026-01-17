# 474 - Property Testing Setup

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 474
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Implement property-based testing using proptest for Rust and fast-check for TypeScript, enabling automatic generation of test cases that verify invariants hold across a wide range of inputs.

---

## Acceptance Criteria

- [x] proptest configured for all Rust crates
- [x] fast-check configured for TypeScript tests
- [x] Custom generators for domain types
- [x] Shrinking produces minimal failing cases
- [x] Property tests integrated with CI pipeline
- [x] Regression tests saved for previously failing cases

---

## Implementation Details

### 1. Proptest Configuration for Rust

Create `crates/tachikoma-test-harness/src/proptest_config.rs`:

```rust
//! Property testing configuration and utilities using proptest.
//!
//! Property-based testing generates many random test cases to verify
//! that invariants hold. When a test fails, proptest automatically
//! shrinks the input to find the minimal failing case.

use proptest::prelude::*;
use proptest::test_runner::Config;

/// Standard proptest configuration for Tachikoma
pub fn standard_config() -> Config {
    Config {
        // Number of successful tests before passing
        cases: 256,
        // Maximum shrink iterations
        max_shrink_iters: 10_000,
        // Save failing cases for regression testing
        failure_persistence: Some(Box::new(
            proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"),
        )),
        // Timeout per test case
        timeout: 1_000,
        ..Config::default()
    }
}

/// Quick configuration for development (fewer cases)
pub fn quick_config() -> Config {
    Config {
        cases: 32,
        max_shrink_iters: 1_000,
        ..standard_config()
    }
}

/// Thorough configuration for CI (more cases)
pub fn ci_config() -> Config {
    Config {
        cases: 1_024,
        max_shrink_iters: 50_000,
        ..standard_config()
    }
}

/// Get configuration based on environment
pub fn env_config() -> Config {
    match std::env::var("PROPTEST_CASES").ok().as_deref() {
        Some("quick") => quick_config(),
        Some("ci") => ci_config(),
        _ => standard_config(),
    }
}
```

### 2. Custom Proptest Strategies

Create `crates/tachikoma-test-harness/src/strategies.rs`:

```rust
//! Custom proptest strategies for Tachikoma domain types.

use proptest::prelude::*;

/// Strategy for generating valid file paths
pub fn valid_file_path() -> impl Strategy<Value = String> {
    prop::collection::vec("[a-z][a-z0-9_]{0,15}", 1..5)
        .prop_map(|parts| parts.join("/"))
}

/// Strategy for generating valid identifiers
pub fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,31}".prop_map(|s| s)
}

/// Strategy for generating valid YAML content
pub fn valid_yaml_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (valid_identifier(), any::<i64>()),
        1..10,
    )
    .prop_map(|pairs| {
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Strategy for generating valid JSON objects
pub fn valid_json_object() -> impl Strategy<Value = serde_json::Value> {
    prop::collection::btree_map(
        valid_identifier(),
        prop_oneof![
            any::<bool>().prop_map(serde_json::Value::Bool),
            any::<i64>().prop_map(|n| serde_json::Value::Number(n.into())),
            "[a-z ]{0,50}".prop_map(serde_json::Value::String),
        ],
        0..10,
    )
    .prop_map(|map| serde_json::Value::Object(map.into_iter().collect()))
}

/// Strategy for generating backend configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub name: String,
    pub api_key: String,
    pub max_retries: u32,
    pub timeout_ms: u64,
}

pub fn backend_config_strategy() -> impl Strategy<Value = BackendConfig> {
    (
        valid_identifier(),
        "[a-zA-Z0-9]{32,64}",
        0u32..10,
        100u64..30_000,
    )
        .prop_map(|(name, api_key, max_retries, timeout_ms)| BackendConfig {
            name,
            api_key,
            max_retries,
            timeout_ms,
        })
}

/// Strategy for generating tool definitions
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<String>,
}

pub fn tool_definition_strategy() -> impl Strategy<Value = ToolDefinition> {
    (
        valid_identifier(),
        "[A-Za-z ]{10,100}",
        prop::collection::vec(valid_identifier(), 0..5),
    )
        .prop_map(|(name, description, parameters)| ToolDefinition {
            name,
            description,
            parameters,
        })
}

/// Arbitrary implementation for standard types
pub mod arbitrary {
    use super::*;

    impl Arbitrary for BackendConfig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            backend_config_strategy().boxed()
        }
    }

    impl Arbitrary for ToolDefinition {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            tool_definition_strategy().boxed()
        }
    }
}
```

### 3. Property Test Examples

Create `crates/tachikoma-test-harness/src/examples/property_tests.rs`:

```rust
//! Example property tests demonstrating common patterns.

use proptest::prelude::*;
use crate::strategies::*;

// ============================================
// Pattern: Roundtrip Property
// ============================================

fn encode(s: &str) -> String {
    base64::encode(s)
}

fn decode(s: &str) -> Result<String, base64::DecodeError> {
    base64::decode(s).map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

proptest! {
    /// Property: encode then decode returns original string
    #[test]
    fn test_encode_decode_roundtrip(s in "\\PC*") {
        let encoded = encode(&s);
        let decoded = decode(&encoded).expect("decode failed");
        prop_assert_eq!(s, decoded);
    }
}

// ============================================
// Pattern: Invariant Property
// ============================================

fn sort_and_dedupe(mut items: Vec<i32>) -> Vec<i32> {
    items.sort();
    items.dedup();
    items
}

proptest! {
    /// Property: sorted list is always sorted
    #[test]
    fn test_sort_produces_sorted_list(items in prop::collection::vec(any::<i32>(), 0..100)) {
        let result = sort_and_dedupe(items);

        // Check sorted invariant
        for window in result.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }

    /// Property: deduped list has no consecutive duplicates
    #[test]
    fn test_dedupe_removes_duplicates(items in prop::collection::vec(any::<i32>(), 0..100)) {
        let result = sort_and_dedupe(items);

        // Check no consecutive duplicates
        for window in result.windows(2) {
            prop_assert_ne!(window[0], window[1]);
        }
    }
}

// ============================================
// Pattern: Oracle Property (compare implementations)
// ============================================

fn my_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

proptest! {
    /// Property: our max matches std max
    #[test]
    fn test_max_matches_std(a in any::<i32>(), b in any::<i32>()) {
        prop_assert_eq!(my_max(a, b), std::cmp::max(a, b));
    }
}

// ============================================
// Pattern: Metamorphic Property
// ============================================

fn search(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

proptest! {
    /// Metamorphic: if needle found in part, found in whole
    #[test]
    fn test_search_metamorphic(
        prefix in "[a-z]{0,10}",
        needle in "[a-z]{1,5}",
        suffix in "[a-z]{0,10}"
    ) {
        let haystack = format!("{}{}{}", prefix, needle, suffix);
        prop_assert!(search(&haystack, &needle));
    }
}

// ============================================
// Pattern: Domain-Specific Properties
// ============================================

proptest! {
    /// Property: file paths don't contain double slashes after normalization
    #[test]
    fn test_path_normalization(path in valid_file_path()) {
        // Simulate path normalization
        let normalized = path.replace("//", "/");
        prop_assert!(!normalized.contains("//"));
    }

    /// Property: valid identifiers match expected pattern
    #[test]
    fn test_identifier_format(id in valid_identifier()) {
        prop_assert!(id.chars().next().unwrap().is_ascii_lowercase());
        prop_assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        prop_assert!(id.len() <= 32);
    }
}
```

### 4. TypeScript Property Testing with fast-check

Create `web/src/test/property/setup.ts`:

```typescript
/**
 * Property testing setup using fast-check.
 */

import * as fc from 'fast-check';

// ============================================
// Custom Arbitraries
// ============================================

export const validIdentifier = fc
  .tuple(
    fc.constantFrom(...'abcdefghijklmnopqrstuvwxyz'.split('')),
    fc.stringOf(fc.constantFrom(...'abcdefghijklmnopqrstuvwxyz0123456789_'.split('')), {
      minLength: 0,
      maxLength: 31,
    })
  )
  .map(([first, rest]) => first + rest);

export const validEmail = fc
  .tuple(validIdentifier, validIdentifier, fc.constantFrom('com', 'org', 'io', 'dev'))
  .map(([user, domain, tld]) => `${user}@${domain}.${tld}`);

export const validFilePath = fc
  .array(validIdentifier, { minLength: 1, maxLength: 5 })
  .map(parts => parts.join('/'));

export interface UserArbitrary {
  id: string;
  name: string;
  email: string;
  age: number;
}

export const userArbitrary: fc.Arbitrary<UserArbitrary> = fc.record({
  id: fc.uuid(),
  name: fc.string({ minLength: 1, maxLength: 50 }),
  email: validEmail,
  age: fc.integer({ min: 0, max: 150 }),
});

// ============================================
// Property Test Utilities
// ============================================

export const propertyConfig: fc.Parameters<unknown> = {
  numRuns: 100,
  verbose: process.env.VERBOSE === 'true',
  seed: process.env.FC_SEED ? parseInt(process.env.FC_SEED) : undefined,
};

export const ciConfig: fc.Parameters<unknown> = {
  ...propertyConfig,
  numRuns: 1000,
};
```

Create `web/src/test/property/examples.test.ts`:

```typescript
import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';
import { validIdentifier, validEmail, userArbitrary, propertyConfig } from './setup';

describe('Property Tests', () => {
  // ============================================
  // Pattern: Roundtrip Property
  // ============================================

  describe('JSON roundtrip', () => {
    it('should parse stringified JSON correctly', () => {
      fc.assert(
        fc.property(fc.jsonValue(), value => {
          const stringified = JSON.stringify(value);
          const parsed = JSON.parse(stringified);
          expect(parsed).toEqual(value);
        }),
        propertyConfig
      );
    });
  });

  // ============================================
  // Pattern: Invariant Property
  // ============================================

  describe('Array sort', () => {
    it('should produce sorted array', () => {
      fc.assert(
        fc.property(fc.array(fc.integer()), arr => {
          const sorted = [...arr].sort((a, b) => a - b);

          // Invariant: each element <= next element
          for (let i = 0; i < sorted.length - 1; i++) {
            expect(sorted[i]).toBeLessThanOrEqual(sorted[i + 1]);
          }
        }),
        propertyConfig
      );
    });

    it('should preserve array length', () => {
      fc.assert(
        fc.property(fc.array(fc.integer()), arr => {
          const sorted = [...arr].sort((a, b) => a - b);
          expect(sorted.length).toBe(arr.length);
        }),
        propertyConfig
      );
    });
  });

  // ============================================
  // Pattern: Domain-Specific Properties
  // ============================================

  describe('Valid identifier', () => {
    it('should start with lowercase letter', () => {
      fc.assert(
        fc.property(validIdentifier, id => {
          expect(id[0]).toMatch(/[a-z]/);
        }),
        propertyConfig
      );
    });

    it('should contain only valid characters', () => {
      fc.assert(
        fc.property(validIdentifier, id => {
          expect(id).toMatch(/^[a-z][a-z0-9_]*$/);
        }),
        propertyConfig
      );
    });
  });

  describe('Valid email', () => {
    it('should contain @ symbol', () => {
      fc.assert(
        fc.property(validEmail, email => {
          expect(email).toContain('@');
        }),
        propertyConfig
      );
    });

    it('should have valid structure', () => {
      fc.assert(
        fc.property(validEmail, email => {
          const parts = email.split('@');
          expect(parts).toHaveLength(2);
          expect(parts[0].length).toBeGreaterThan(0);
          expect(parts[1]).toContain('.');
        }),
        propertyConfig
      );
    });
  });

  // ============================================
  // Pattern: Metamorphic Testing
  // ============================================

  describe('String operations', () => {
    it('concat then split should preserve parts', () => {
      fc.assert(
        fc.property(fc.array(fc.string()), parts => {
          const separator = '|||';
          const joined = parts.join(separator);
          const split = joined.split(separator);

          // Empty array joins to empty string, splits to ['']
          if (parts.length === 0) {
            expect(split).toEqual(['']);
          } else {
            expect(split).toEqual(parts);
          }
        }),
        propertyConfig
      );
    });
  });

  // ============================================
  // Pattern: Complex Object Properties
  // ============================================

  describe('User object', () => {
    it('should have valid structure', () => {
      fc.assert(
        fc.property(userArbitrary, user => {
          expect(user.id).toBeDefined();
          expect(user.name.length).toBeGreaterThan(0);
          expect(user.email).toContain('@');
          expect(user.age).toBeGreaterThanOrEqual(0);
        }),
        propertyConfig
      );
    });
  });
});
```

---

## Testing Requirements

1. `cargo test` runs proptest tests by default
2. `npm run test:property` runs fast-check tests
3. Shrinking produces minimal failing cases
4. Regression files are committed to repository
5. CI uses higher iteration counts

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [475-snapshot-testing.md](475-snapshot-testing.md)
- Related: [480-test-generators.md](480-test-generators.md)
