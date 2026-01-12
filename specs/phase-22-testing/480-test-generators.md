# 480 - Test Data Generators

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 480
**Status:** Planned
**Dependencies:** 471-test-harness, 474-property-testing
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Create test data generators using fake-rs for Rust and faker.js for TypeScript that produce realistic, randomized test data for various domain objects while maintaining consistency and reproducibility.

---

## Acceptance Criteria

- [ ] Generators for all major domain types
- [ ] Seeded generation for reproducible tests
- [ ] Locale-aware data (names, addresses)
- [ ] Domain-specific generators (API keys, tokens, paths)
- [ ] Batch generation for performance testing
- [ ] Integration with property testing frameworks

---

## Implementation Details

### 1. Rust Test Data Generators

Create `crates/tachikoma-test-harness/src/generators/mod.rs`:

```rust
//! Test data generators for creating realistic test data.

use fake::{Fake, Faker};
use fake::faker::internet::en::*;
use fake::faker::name::en::*;
use fake::faker::lorem::en::*;
use fake::faker::filesystem::en::*;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub mod domain;
pub mod api;
pub mod config;

/// Generator context with optional seed for reproducibility
pub struct GeneratorContext {
    rng: StdRng,
}

impl GeneratorContext {
    /// Create a new generator context with random seed
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a generator context with specific seed for reproducibility
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Get mutable reference to RNG
    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }
}

impl Default for GeneratorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random email address
pub fn email() -> String {
    FreeEmail().fake()
}

/// Generate a random username
pub fn username() -> String {
    Username().fake()
}

/// Generate a random full name
pub fn full_name() -> String {
    Name().fake()
}

/// Generate a random first name
pub fn first_name() -> String {
    FirstName().fake()
}

/// Generate a random last name
pub fn last_name() -> String {
    LastName().fake()
}

/// Generate random words
pub fn words(count: usize) -> Vec<String> {
    Words(count..count + 1).fake()
}

/// Generate a random sentence
pub fn sentence() -> String {
    Sentence(3..10).fake()
}

/// Generate a random paragraph
pub fn paragraph() -> String {
    Paragraph(3..7).fake()
}

/// Generate a random file path
pub fn file_path() -> String {
    FilePath().fake()
}

/// Generate a random file name
pub fn file_name() -> String {
    FileName().fake()
}

/// Generate a random directory path
pub fn dir_path() -> String {
    DirPath().fake()
}

/// Generate a random hex string
pub fn hex_string(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| format!("{:x}", rng.gen::<u8>() % 16))
        .collect()
}

/// Generate a random alphanumeric string
pub fn alphanumeric(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a UUID v4
pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a timestamp within a range
pub fn timestamp_between(start: i64, end: i64) -> i64 {
    use rand::Rng;
    rand::thread_rng().gen_range(start..end)
}

/// Generate a random boolean with probability
pub fn bool_with_probability(probability: f64) -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool(probability)
}
```

### 2. Domain-Specific Generators

Create `crates/tachikoma-test-harness/src/generators/domain.rs`:

```rust
//! Domain-specific generators for Tachikoma types.

use super::*;

/// Generate a mock API key
pub fn api_key(prefix: &str) -> String {
    format!("{}-{}", prefix, alphanumeric(48))
}

/// Generate a Claude API key
pub fn claude_api_key() -> String {
    api_key("sk-ant")
}

/// Generate an OpenAI API key
pub fn openai_api_key() -> String {
    api_key("sk")
}

/// Generate a session ID
pub fn session_id() -> String {
    format!("session_{}", uuid())
}

/// Generate a mission ID
pub fn mission_id() -> String {
    format!("mission_{}", hex_string(16))
}

/// Generate a spec ID
pub fn spec_id() -> String {
    use rand::Rng;
    let num: u32 = rand::thread_rng().gen_range(1..999);
    format!("{:03}", num)
}

/// User generator
#[derive(Debug, Clone)]
pub struct UserGenerator {
    role: Option<String>,
}

impl UserGenerator {
    pub fn new() -> Self {
        Self { role: None }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn admin(self) -> Self {
        self.with_role("admin")
    }

    pub fn guest(self) -> Self {
        self.with_role("guest")
    }

    pub fn generate(&self) -> GeneratedUser {
        GeneratedUser {
            id: uuid(),
            name: full_name(),
            email: email(),
            username: username(),
            role: self.role.clone().unwrap_or_else(|| "user".into()),
        }
    }

    pub fn generate_many(&self, count: usize) -> Vec<GeneratedUser> {
        (0..count).map(|_| self.generate()).collect()
    }
}

impl Default for UserGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role: String,
}

/// Message generator for chat messages
pub struct MessageGenerator {
    role: String,
}

impl MessageGenerator {
    pub fn user() -> Self {
        Self { role: "user".into() }
    }

    pub fn assistant() -> Self {
        Self { role: "assistant".into() }
    }

    pub fn system() -> Self {
        Self { role: "system".into() }
    }

    pub fn generate(&self) -> GeneratedMessage {
        GeneratedMessage {
            role: self.role.clone(),
            content: paragraph(),
        }
    }

    pub fn generate_conversation(turns: usize) -> Vec<GeneratedMessage> {
        let mut messages = Vec::new();

        // Start with system message
        messages.push(Self::system().generate());

        for i in 0..turns {
            if i % 2 == 0 {
                messages.push(Self::user().generate());
            } else {
                messages.push(Self::assistant().generate());
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedMessage {
    pub role: String,
    pub content: String,
}

/// Tool call generator
pub struct ToolCallGenerator;

impl ToolCallGenerator {
    pub fn read_file() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "read_file".into(),
            arguments: serde_json::json!({
                "path": file_path()
            }),
        }
    }

    pub fn write_file() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "edit_file".into(),
            arguments: serde_json::json!({
                "path": file_path(),
                "old_string": sentence(),
                "new_string": sentence()
            }),
        }
    }

    pub fn bash() -> GeneratedToolCall {
        let commands = ["ls -la", "pwd", "cat file.txt", "git status", "npm test"];
        use rand::Rng;
        let cmd = commands[rand::thread_rng().gen_range(0..commands.len())];

        GeneratedToolCall {
            name: "bash".into(),
            arguments: serde_json::json!({
                "command": cmd
            }),
        }
    }

    pub fn code_search() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "code_search".into(),
            arguments: serde_json::json!({
                "pattern": words(1)[0],
                "path": dir_path()
            }),
        }
    }

    pub fn random() -> GeneratedToolCall {
        use rand::Rng;
        match rand::thread_rng().gen_range(0..4) {
            0 => Self::read_file(),
            1 => Self::write_file(),
            2 => Self::bash(),
            _ => Self::code_search(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Spec generator
pub struct SpecGenerator;

impl SpecGenerator {
    pub fn generate() -> GeneratedSpec {
        let id = spec_id();
        let title = words(3).join(" ");

        GeneratedSpec {
            id: id.clone(),
            title: title.clone(),
            content: format!(
                "# {} - {}\n\n## Objective\n\n{}\n\n## Acceptance Criteria\n\n- [ ] {}\n- [ ] {}\n- [ ] {}\n",
                id,
                title,
                paragraph(),
                sentence(),
                sentence(),
                sentence()
            ),
            phase: format!("{:02}", rand::thread_rng().gen_range(0..25)),
            status: "Planned".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedSpec {
    pub id: String,
    pub title: String,
    pub content: String,
    pub phase: String,
    pub status: String,
}
```

### 3. TypeScript Data Generators

Create `web/src/test/generators/index.ts`:

```typescript
/**
 * Test data generators for TypeScript tests.
 */

import { faker } from '@faker-js/faker';

/**
 * Set seed for reproducible generation
 */
export function setSeed(seed: number): void {
  faker.seed(seed);
}

/**
 * Reset to random seed
 */
export function resetSeed(): void {
  faker.seed();
}

// ============================================
// Basic Generators
// ============================================

export const generators = {
  email: () => faker.internet.email(),
  username: () => faker.internet.userName(),
  fullName: () => faker.person.fullName(),
  firstName: () => faker.person.firstName(),
  lastName: () => faker.person.lastName(),
  sentence: () => faker.lorem.sentence(),
  paragraph: () => faker.lorem.paragraph(),
  words: (count: number) => faker.lorem.words(count),
  uuid: () => faker.string.uuid(),
  hexString: (length: number) => faker.string.hexadecimal({ length, casing: 'lower' }).slice(2),
  alphanumeric: (length: number) => faker.string.alphanumeric(length),
  number: (min: number, max: number) => faker.number.int({ min, max }),
  boolean: (probability = 0.5) => faker.datatype.boolean(probability),
  date: () => faker.date.recent(),
  pastDate: () => faker.date.past(),
  futureDate: () => faker.date.future(),
  filePath: () => faker.system.filePath(),
  fileName: () => faker.system.fileName(),
  dirPath: () => faker.system.directoryPath(),
};

// ============================================
// Domain Generators
// ============================================

export const domainGenerators = {
  apiKey: (prefix: string) => `${prefix}-${generators.alphanumeric(48)}`,
  claudeApiKey: () => domainGenerators.apiKey('sk-ant'),
  openaiApiKey: () => domainGenerators.apiKey('sk'),
  sessionId: () => `session_${generators.uuid()}`,
  missionId: () => `mission_${generators.hexString(16)}`,
  specId: () => String(generators.number(1, 999)).padStart(3, '0'),
};

// ============================================
// Entity Generators
// ============================================

export interface GeneratedUser {
  id: string;
  name: string;
  email: string;
  username: string;
  role: 'admin' | 'user' | 'guest';
}

export function generateUser(overrides?: Partial<GeneratedUser>): GeneratedUser {
  return {
    id: generators.uuid(),
    name: generators.fullName(),
    email: generators.email(),
    username: generators.username(),
    role: 'user',
    ...overrides,
  };
}

export function generateUsers(count: number, overrides?: Partial<GeneratedUser>): GeneratedUser[] {
  return Array.from({ length: count }, () => generateUser(overrides));
}

export interface GeneratedMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export function generateMessage(role: GeneratedMessage['role'] = 'user'): GeneratedMessage {
  return {
    role,
    content: generators.paragraph(),
  };
}

export function generateConversation(turns: number): GeneratedMessage[] {
  const messages: GeneratedMessage[] = [generateMessage('system')];

  for (let i = 0; i < turns; i++) {
    messages.push(generateMessage(i % 2 === 0 ? 'user' : 'assistant'));
  }

  return messages;
}

export interface GeneratedToolCall {
  name: string;
  arguments: Record<string, unknown>;
}

export const toolCallGenerators = {
  readFile: (): GeneratedToolCall => ({
    name: 'read_file',
    arguments: { path: generators.filePath() },
  }),

  writeFile: (): GeneratedToolCall => ({
    name: 'edit_file',
    arguments: {
      path: generators.filePath(),
      old_string: generators.sentence(),
      new_string: generators.sentence(),
    },
  }),

  bash: (): GeneratedToolCall => ({
    name: 'bash',
    arguments: {
      command: faker.helpers.arrayElement(['ls -la', 'pwd', 'cat file.txt', 'git status', 'npm test']),
    },
  }),

  codeSearch: (): GeneratedToolCall => ({
    name: 'code_search',
    arguments: {
      pattern: generators.words(1),
      path: generators.dirPath(),
    },
  }),

  random: (): GeneratedToolCall => {
    return faker.helpers.arrayElement([
      toolCallGenerators.readFile,
      toolCallGenerators.writeFile,
      toolCallGenerators.bash,
      toolCallGenerators.codeSearch,
    ])();
  },
};

export interface GeneratedSpec {
  id: string;
  title: string;
  content: string;
  phase: string;
  status: 'Planned' | 'In Progress' | 'Complete';
}

export function generateSpec(overrides?: Partial<GeneratedSpec>): GeneratedSpec {
  const id = domainGenerators.specId();
  const title = generators.words(3);

  return {
    id,
    title,
    content: `# ${id} - ${title}

## Objective

${generators.paragraph()}

## Acceptance Criteria

- [ ] ${generators.sentence()}
- [ ] ${generators.sentence()}
- [ ] ${generators.sentence()}
`,
    phase: String(generators.number(0, 24)).padStart(2, '0'),
    status: 'Planned',
    ...overrides,
  };
}

// ============================================
// Batch Generation
// ============================================

export function batch<T>(generator: () => T, count: number): T[] {
  return Array.from({ length: count }, generator);
}

// ============================================
// Reproducible Generation Context
// ============================================

export class GeneratorContext {
  constructor(seed?: number) {
    if (seed !== undefined) {
      faker.seed(seed);
    }
  }

  reset(): void {
    faker.seed();
  }

  user = generateUser;
  users = generateUsers;
  message = generateMessage;
  conversation = generateConversation;
  spec = generateSpec;
  toolCall = toolCallGenerators;

  batch<T>(generator: () => T, count: number): T[] {
    return batch(generator, count);
  }
}

export const gen = new GeneratorContext();
```

---

## Testing Requirements

1. Generators produce valid, realistic data
2. Seeded generation produces identical results
3. Batch generation performs efficiently
4. Domain generators create valid domain objects
5. Integration with property testing works correctly

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [474-property-testing.md](474-property-testing.md)
- Next: [481-test-coverage.md](481-test-coverage.md)
- Related: [479-test-fixtures.md](479-test-fixtures.md)
