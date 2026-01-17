# 472 - Unit Test Patterns

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 472
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~10% of Sonnet window

---

## Objective

Define and implement standardized unit testing patterns for Rust and TypeScript codebases, ensuring consistent test structure, clear naming conventions, and maintainable test code across all Tachikoma modules.

---

## Acceptance Criteria

- [x] Consistent test naming conventions documented and enforced
- [x] Arrange-Act-Assert pattern used throughout
- [x] Test helper functions reduce boilerplate
- [x] Test modules properly organized within source files
- [x] Edge cases and error conditions systematically tested
- [x] Test data builders simplify complex object creation

---

## Implementation Details

### 1. Rust Unit Test Patterns

Create `crates/tachikoma-test-harness/src/patterns/unit.rs`:

```rust
//! Unit test patterns and conventions for Rust code.
//!
//! ## Naming Convention
//! - `test_<function>_<scenario>_<expected_outcome>`
//! - Example: `test_parse_config_empty_file_returns_default`
//!
//! ## Structure
//! - Use `mod tests` within each module
//! - Group related tests with nested modules
//! - Use `rstest` for parameterized tests

/// Marker trait for test builders
pub trait TestBuilder {
    type Output;
    fn build(self) -> Self::Output;
}

/// Example test builder pattern
#[derive(Default, Clone)]
pub struct ConfigBuilder {
    pub name: Option<String>,
    pub enabled: bool,
    pub max_retries: u32,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Example domain object for demonstration
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub name: String,
    pub enabled: bool,
    pub max_retries: u32,
}

impl TestBuilder for ConfigBuilder {
    type Output = Config;

    fn build(self) -> Config {
        Config {
            name: self.name.unwrap_or_else(|| "default".into()),
            enabled: self.enabled,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use pretty_assertions::assert_eq;

    // ============================================
    // Pattern: Basic Arrange-Act-Assert
    // ============================================

    #[test]
    fn test_config_builder_default_values_are_sensible() {
        // Arrange
        let builder = ConfigBuilder::new();

        // Act
        let config = builder.build();

        // Assert
        assert_eq!(config.name, "default");
        assert!(!config.enabled);
        assert_eq!(config.max_retries, 0);
    }

    // ============================================
    // Pattern: Parameterized Tests with rstest
    // ============================================

    #[rstest]
    #[case("production", true, 3)]
    #[case("staging", true, 2)]
    #[case("development", false, 0)]
    fn test_config_builder_respects_all_fields(
        #[case] name: &str,
        #[case] enabled: bool,
        #[case] max_retries: u32,
    ) {
        // Arrange & Act
        let config = ConfigBuilder::new()
            .name(name)
            .enabled(enabled)
            .max_retries(max_retries)
            .build();

        // Assert
        assert_eq!(config.name, name);
        assert_eq!(config.enabled, enabled);
        assert_eq!(config.max_retries, max_retries);
    }

    // ============================================
    // Pattern: Grouped Tests by Feature
    // ============================================

    mod name_handling {
        use super::*;

        #[test]
        fn test_name_can_be_set() {
            let config = ConfigBuilder::new().name("custom").build();
            assert_eq!(config.name, "custom");
        }

        #[test]
        fn test_name_accepts_string_types() {
            let string_owned = String::from("owned");
            let config = ConfigBuilder::new().name(string_owned).build();
            assert_eq!(config.name, "owned");
        }
    }

    mod retry_handling {
        use super::*;

        #[test]
        fn test_max_retries_defaults_to_zero() {
            let config = ConfigBuilder::new().build();
            assert_eq!(config.max_retries, 0);
        }

        #[rstest]
        #[case(0)]
        #[case(1)]
        #[case(100)]
        #[case(u32::MAX)]
        fn test_max_retries_accepts_valid_values(#[case] retries: u32) {
            let config = ConfigBuilder::new().max_retries(retries).build();
            assert_eq!(config.max_retries, retries);
        }
    }
}
```

### 2. TypeScript Unit Test Patterns

Create `web/src/test/patterns/unit.ts`:

```typescript
/**
 * Unit test patterns and conventions for TypeScript code.
 *
 * ## Naming Convention
 * - describe: Component or function name
 * - it: "should <expected behavior> when <condition>"
 *
 * ## Structure
 * - Group by component/function
 * - Nest by feature or scenario
 * - Use beforeEach for common setup
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// ============================================
// Pattern: Test Data Builder
// ============================================

export interface User {
  id: string;
  name: string;
  email: string;
  role: 'admin' | 'user' | 'guest';
  createdAt: Date;
}

export class UserBuilder {
  private data: Partial<User> = {};

  static create(): UserBuilder {
    return new UserBuilder();
  }

  withId(id: string): this {
    this.data.id = id;
    return this;
  }

  withName(name: string): this {
    this.data.name = name;
    return this;
  }

  withEmail(email: string): this {
    this.data.email = email;
    return this;
  }

  withRole(role: User['role']): this {
    this.data.role = role;
    return this;
  }

  asAdmin(): this {
    return this.withRole('admin');
  }

  asGuest(): this {
    return this.withRole('guest');
  }

  build(): User {
    return {
      id: this.data.id ?? `user-${Date.now()}`,
      name: this.data.name ?? 'Test User',
      email: this.data.email ?? 'test@example.com',
      role: this.data.role ?? 'user',
      createdAt: new Date(),
    };
  }
}

// ============================================
// Example Tests Using Patterns
// ============================================

describe('UserBuilder', () => {
  describe('default values', () => {
    it('should create user with sensible defaults', () => {
      // Arrange & Act
      const user = UserBuilder.create().build();

      // Assert
      expect(user.name).toBe('Test User');
      expect(user.email).toBe('test@example.com');
      expect(user.role).toBe('user');
    });

    it('should generate unique IDs', () => {
      const user1 = UserBuilder.create().build();
      const user2 = UserBuilder.create().build();

      expect(user1.id).not.toBe(user2.id);
    });
  });

  describe('role handling', () => {
    it.each([
      ['admin', 'admin'],
      ['user', 'user'],
      ['guest', 'guest'],
    ] as const)('should accept %s role', (role, expected) => {
      const user = UserBuilder.create().withRole(role).build();
      expect(user.role).toBe(expected);
    });

    it('should have admin helper', () => {
      const user = UserBuilder.create().asAdmin().build();
      expect(user.role).toBe('admin');
    });

    it('should have guest helper', () => {
      const user = UserBuilder.create().asGuest().build();
      expect(user.role).toBe('guest');
    });
  });

  describe('chaining', () => {
    it('should support method chaining', () => {
      const user = UserBuilder.create()
        .withId('custom-id')
        .withName('Jane Doe')
        .withEmail('jane@example.com')
        .asAdmin()
        .build();

      expect(user).toEqual(
        expect.objectContaining({
          id: 'custom-id',
          name: 'Jane Doe',
          email: 'jane@example.com',
          role: 'admin',
        })
      );
    });
  });
});

// ============================================
// Pattern: Testing Async Functions
// ============================================

async function fetchUserById(id: string): Promise<User | null> {
  // Simulated async operation
  await new Promise(resolve => setTimeout(resolve, 10));
  if (id === 'not-found') return null;
  return UserBuilder.create().withId(id).build();
}

describe('fetchUserById', () => {
  it('should return user when found', async () => {
    const user = await fetchUserById('test-id');

    expect(user).not.toBeNull();
    expect(user?.id).toBe('test-id');
  });

  it('should return null when not found', async () => {
    const user = await fetchUserById('not-found');
    expect(user).toBeNull();
  });
});

// ============================================
// Pattern: Testing with Mocks
// ============================================

interface ApiClient {
  get<T>(url: string): Promise<T>;
}

class UserService {
  constructor(private api: ApiClient) {}

  async getUser(id: string): Promise<User | null> {
    try {
      return await this.api.get<User>(`/users/${id}`);
    } catch {
      return null;
    }
  }
}

describe('UserService', () => {
  let mockApi: ApiClient;
  let service: UserService;

  beforeEach(() => {
    mockApi = {
      get: vi.fn(),
    };
    service = new UserService(mockApi);
  });

  it('should call API with correct URL', async () => {
    const mockUser = UserBuilder.create().withId('123').build();
    vi.mocked(mockApi.get).mockResolvedValue(mockUser);

    await service.getUser('123');

    expect(mockApi.get).toHaveBeenCalledWith('/users/123');
  });

  it('should return user from API', async () => {
    const mockUser = UserBuilder.create().withId('123').build();
    vi.mocked(mockApi.get).mockResolvedValue(mockUser);

    const result = await service.getUser('123');

    expect(result).toEqual(mockUser);
  });

  it('should return null on API error', async () => {
    vi.mocked(mockApi.get).mockRejectedValue(new Error('Network error'));

    const result = await service.getUser('123');

    expect(result).toBeNull();
  });
});
```

### 3. Unit Test Guidelines Document

Create `docs/testing/unit-test-guidelines.md`:

```markdown
# Unit Test Guidelines

## Naming Conventions

### Rust
```
test_<function>_<scenario>_<expected_outcome>
```
Examples:
- `test_parse_config_valid_yaml_returns_config`
- `test_validate_empty_string_returns_error`

### TypeScript
```
describe('<Component/Function>')
  it('should <expected behavior> when <condition>')
```
Examples:
- `it('should return null when user not found')`
- `it('should emit click event when button pressed')`

## Arrange-Act-Assert Pattern

Every test should follow this structure:

```rust
#[test]
fn test_example() {
    // Arrange - Set up test data and dependencies
    let input = create_test_input();
    let service = create_service();

    // Act - Execute the code under test
    let result = service.process(input);

    // Assert - Verify the outcome
    assert_eq!(result, expected_output);
}
```

## Test Data Builders

Use builders for complex test objects:

```rust
let config = ConfigBuilder::new()
    .name("test")
    .enabled(true)
    .build();
```

## What to Test

- Happy path (expected inputs)
- Edge cases (empty, null, boundaries)
- Error conditions
- State transitions

## What NOT to Unit Test

- Private implementation details
- External dependencies (use integration tests)
- Framework/library code
```

---

## Testing Requirements

1. All example tests pass
2. Test patterns are documented and accessible
3. Linting rules enforce naming conventions
4. Test builders reduce test code duplication
5. New code follows established patterns

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Next: [473-integration-patterns.md](473-integration-patterns.md)
- Related: [479-test-fixtures.md](479-test-fixtures.md)
