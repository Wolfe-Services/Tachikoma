# 479 - Test Fixtures

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 479
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create a comprehensive test fixture system for loading, managing, and sharing test data across Rust and TypeScript test suites, enabling consistent test data management and reducing test setup boilerplate.

---

## Acceptance Criteria

- [x] Fixture files stored in organized directory structure
- [x] JSON, YAML, and text fixtures supported
- [x] Fixture loading utilities for both Rust and TypeScript
- [x] Parameterized fixtures for data-driven testing
- [x] Fixture templating with variable substitution
- [x] Fixtures cached for performance

---

## Implementation Details

### 1. Fixture Directory Structure

```
tests/
  fixtures/
    configs/
      valid_config.yaml
      minimal_config.yaml
      full_config.yaml
    api_responses/
      claude/
        success.json
        error_rate_limit.json
        streaming_chunks.json
      codex/
        success.json
    specs/
      simple_spec.md
      complex_spec.md
    users/
      admin.json
      guest.json
    templates/
      prompt_template.txt
```

### 2. Rust Fixture Loader

Create `crates/tachikoma-test-harness/src/fixtures/mod.rs`:

```rust
//! Test fixture loading and management.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use serde::de::DeserializeOwned;

/// Global fixture cache
static FIXTURE_CACHE: OnceLock<HashMap<PathBuf, String>> = OnceLock::new();

/// Get the fixtures directory path
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".into());

    // Try multiple possible locations
    let candidates = [
        PathBuf::from(&manifest_dir).join("tests/fixtures"),
        PathBuf::from(&manifest_dir).join("../tests/fixtures"),
        PathBuf::from(&manifest_dir).join("../../tests/fixtures"),
        PathBuf::from("tests/fixtures"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Default to first candidate
    candidates[0].clone()
}

/// Load a fixture as a string
pub fn load_fixture(path: impl AsRef<Path>) -> String {
    let full_path = fixtures_dir().join(path.as_ref());
    std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {:?}: {}", full_path, e))
}

/// Load a fixture as bytes
pub fn load_fixture_bytes(path: impl AsRef<Path>) -> Vec<u8> {
    let full_path = fixtures_dir().join(path.as_ref());
    std::fs::read(&full_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {:?}: {}", full_path, e))
}

/// Load a JSON fixture and deserialize
pub fn load_json_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let content = load_fixture(path);
    serde_json::from_str(&content).expect("Failed to parse JSON fixture")
}

/// Load a YAML fixture and deserialize
pub fn load_yaml_fixture<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    let content = load_fixture(path);
    serde_yaml::from_str(&content).expect("Failed to parse YAML fixture")
}

/// Fixture with template variables
pub struct TemplatedFixture {
    template: String,
    variables: HashMap<String, String>,
}

impl TemplatedFixture {
    /// Load a template fixture
    pub fn load(path: impl AsRef<Path>) -> Self {
        Self {
            template: load_fixture(path),
            variables: HashMap::new(),
        }
    }

    /// Set a variable value
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set multiple variables
    pub fn vars(mut self, vars: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        for (k, v) in vars {
            self.variables.insert(k.into(), v.into());
        }
        self
    }

    /// Render the template with substitutions
    pub fn render(&self) -> String {
        let mut result = self.template.clone();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Render and parse as JSON
    pub fn render_json<T: DeserializeOwned>(&self) -> T {
        let rendered = self.render();
        serde_json::from_str(&rendered).expect("Failed to parse rendered template as JSON")
    }
}

/// Fixture set for parameterized tests
pub struct FixtureSet<T> {
    fixtures: Vec<(String, T)>,
}

impl<T: DeserializeOwned> FixtureSet<T> {
    /// Load all fixtures from a directory
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        let full_dir = fixtures_dir().join(dir.as_ref());
        let mut fixtures = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&full_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let content = std::fs::read_to_string(&path)
                        .expect("Failed to read fixture");
                    let data: T = serde_json::from_str(&content)
                        .expect("Failed to parse fixture");
                    fixtures.push((name, data));
                }
            }
        }

        Self { fixtures }
    }

    /// Get iterator over fixtures
    pub fn iter(&self) -> impl Iterator<Item = &(String, T)> {
        self.fixtures.iter()
    }

    /// Get fixture by name
    pub fn get(&self, name: &str) -> Option<&T> {
        self.fixtures.iter()
            .find(|(n, _)| n == name)
            .map(|(_, data)| data)
    }
}

/// Macro for inline fixture definitions
#[macro_export]
macro_rules! fixture {
    // JSON fixture
    (json: $($json:tt)+) => {
        serde_json::json!($($json)+)
    };

    // YAML fixture (string)
    (yaml: $yaml:expr) => {
        serde_yaml::from_str::<serde_yaml::Value>($yaml).expect("Invalid YAML")
    };

    // File fixture
    (file: $path:expr) => {
        $crate::fixtures::load_fixture($path)
    };
}
```

### 3. Common Fixture Types

Create `crates/tachikoma-test-harness/src/fixtures/common.rs`:

```rust
//! Common fixture types and helpers.

use serde::{Deserialize, Serialize};

/// API response fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponseFixture {
    pub status: u16,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: serde_json::Value,
}

/// User fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFixture {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Config fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFixture {
    pub backends: Vec<BackendConfigFixture>,
    pub loop_settings: LoopSettingsFixture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfigFixture {
    pub name: String,
    pub api_key_env: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopSettingsFixture {
    pub max_iterations: u32,
    pub context_threshold: f32,
}

/// Fixture builders for common types
pub mod builders {
    use super::*;

    pub fn user() -> UserFixtureBuilder {
        UserFixtureBuilder::default()
    }

    #[derive(Default)]
    pub struct UserFixtureBuilder {
        id: Option<String>,
        name: Option<String>,
        email: Option<String>,
        role: Option<String>,
    }

    impl UserFixtureBuilder {
        pub fn id(mut self, id: impl Into<String>) -> Self {
            self.id = Some(id.into());
            self
        }

        pub fn name(mut self, name: impl Into<String>) -> Self {
            self.name = Some(name.into());
            self
        }

        pub fn email(mut self, email: impl Into<String>) -> Self {
            self.email = Some(email.into());
            self
        }

        pub fn role(mut self, role: impl Into<String>) -> Self {
            self.role = Some(role.into());
            self
        }

        pub fn admin(self) -> Self {
            self.role("admin")
        }

        pub fn guest(self) -> Self {
            self.role("guest")
        }

        pub fn build(self) -> UserFixture {
            UserFixture {
                id: self.id.unwrap_or_else(|| format!("user-{}", uuid::Uuid::new_v4())),
                name: self.name.unwrap_or_else(|| "Test User".into()),
                email: self.email.unwrap_or_else(|| "test@example.com".into()),
                role: self.role.unwrap_or_else(|| "user".into()),
            }
        }
    }
}
```

### 4. TypeScript Fixture System

Create `web/src/test/fixtures/index.ts`:

```typescript
/**
 * Test fixture loading and management for TypeScript tests.
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, resolve } from 'path';

// Fixture base directory
const FIXTURES_DIR = resolve(__dirname, '../../fixtures');

/**
 * Load a fixture file as string
 */
export function loadFixture(path: string): string {
  const fullPath = join(FIXTURES_DIR, path);
  return readFileSync(fullPath, 'utf-8');
}

/**
 * Load a JSON fixture
 */
export function loadJsonFixture<T>(path: string): T {
  const content = loadFixture(path);
  return JSON.parse(content) as T;
}

/**
 * Load a YAML fixture (requires yaml package)
 */
export function loadYamlFixture<T>(path: string): T {
  const yaml = require('yaml');
  const content = loadFixture(path);
  return yaml.parse(content) as T;
}

/**
 * Templated fixture with variable substitution
 */
export class TemplatedFixture {
  private template: string;
  private variables: Map<string, string> = new Map();

  constructor(path: string) {
    this.template = loadFixture(path);
  }

  static load(path: string): TemplatedFixture {
    return new TemplatedFixture(path);
  }

  var(key: string, value: string): this {
    this.variables.set(key, value);
    return this;
  }

  vars(vars: Record<string, string>): this {
    for (const [key, value] of Object.entries(vars)) {
      this.variables.set(key, value);
    }
    return this;
  }

  render(): string {
    let result = this.template;
    for (const [key, value] of this.variables) {
      const placeholder = `{{${key}}}`;
      result = result.replace(new RegExp(placeholder, 'g'), value);
    }
    return result;
  }

  renderJson<T>(): T {
    return JSON.parse(this.render()) as T;
  }
}

/**
 * Load all fixtures from a directory
 */
export function loadFixtureSet<T>(dir: string): Array<{ name: string; data: T }> {
  const fullDir = join(FIXTURES_DIR, dir);
  if (!existsSync(fullDir)) {
    return [];
  }

  return readdirSync(fullDir)
    .filter(file => file.endsWith('.json'))
    .map(file => ({
      name: file.replace('.json', ''),
      data: loadJsonFixture<T>(join(dir, file)),
    }));
}

/**
 * Fixture builder utilities
 */
export const fixtureBuilders = {
  user: () => new UserFixtureBuilder(),
  config: () => new ConfigFixtureBuilder(),
};

class UserFixtureBuilder {
  private data: Partial<UserFixture> = {};

  id(id: string): this {
    this.data.id = id;
    return this;
  }

  name(name: string): this {
    this.data.name = name;
    return this;
  }

  email(email: string): this {
    this.data.email = email;
    return this;
  }

  role(role: 'admin' | 'user' | 'guest'): this {
    this.data.role = role;
    return this;
  }

  admin(): this {
    return this.role('admin');
  }

  guest(): this {
    return this.role('guest');
  }

  build(): UserFixture {
    return {
      id: this.data.id ?? `user-${Date.now()}`,
      name: this.data.name ?? 'Test User',
      email: this.data.email ?? 'test@example.com',
      role: this.data.role ?? 'user',
    };
  }
}

class ConfigFixtureBuilder {
  private data: Partial<ConfigFixture> = {};

  backend(name: string, model: string): this {
    if (!this.data.backends) {
      this.data.backends = [];
    }
    this.data.backends.push({ name, apiKeyEnv: `${name.toUpperCase()}_API_KEY`, model });
    return this;
  }

  maxIterations(max: number): this {
    if (!this.data.loopSettings) {
      this.data.loopSettings = { maxIterations: 100, contextThreshold: 0.8 };
    }
    this.data.loopSettings.maxIterations = max;
    return this;
  }

  build(): ConfigFixture {
    return {
      backends: this.data.backends ?? [],
      loopSettings: this.data.loopSettings ?? { maxIterations: 100, contextThreshold: 0.8 },
    };
  }
}

// Type definitions
export interface UserFixture {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
}

export interface ConfigFixture {
  backends: Array<{ name: string; apiKeyEnv: string; model: string }>;
  loopSettings: { maxIterations: number; contextThreshold: number };
}
```

### 5. Example Fixture Files

Create `tests/fixtures/api_responses/claude/success.json`:

```json
{
  "id": "msg_123",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Hello! I'm Claude, an AI assistant."
    }
  ],
  "model": "claude-3-opus-20240229",
  "stop_reason": "end_turn",
  "usage": {
    "input_tokens": 10,
    "output_tokens": 15
  }
}
```

Create `tests/fixtures/configs/valid_config.yaml`:

```yaml
backends:
  - name: claude
    api_key_env: ANTHROPIC_API_KEY
    model: claude-3-opus-20240229
    role: brain

loop:
  max_iterations: 100
  context_threshold: 0.8
  auto_reboot: true

stop_conditions:
  - type: test_failures
    threshold: 3
  - type: no_progress
    iterations: 5
```

---

## Testing Requirements

1. Fixtures load correctly from all supported formats
2. Templated fixtures substitute variables correctly
3. Fixture sets iterate over all files in directory
4. Fixture builders create valid test data
5. Missing fixtures produce clear error messages

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [480-test-generators.md](480-test-generators.md)
- Related: [474-property-testing.md](474-property-testing.md)
