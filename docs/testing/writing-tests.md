# Writing Tests

## Rust Unit Tests

### Naming Convention
```
test_<function>_<scenario>_<expected_outcome>
```

### Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_valid_yaml_returns_config() {
        // Arrange
        let yaml = "key: value";

        // Act
        let result = parse_config(yaml);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_config_invalid_yaml_returns_error() {
        let yaml = "invalid: [";
        let result = parse_config(yaml);
        assert!(result.is_err());
    }
}
```

### Property Tests
Use proptest for testing invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_then_serialize_roundtrip(input in "[a-zA-Z0-9_]+") {
        let parsed = parse(&input)?;
        let serialized = serialize(&parsed)?;
        prop_assert_eq!(input, serialized);
    }
}
```

### Integration Tests
```rust
// In tests/integration/mod.rs
use tachikoma_test_harness::prelude::*;

#[tokio::test]
async fn test_full_workflow_integration() {
    let ctx = TestContext::builder()
        .with_temp_dir()
        .with_mock_server()
        .build()
        .await;

    // Test implementation
    let result = run_workflow(&ctx).await;
    
    assert!(result.is_ok());
}
```

## TypeScript Unit Tests

### Naming Convention
```typescript
describe('ComponentName', () => {
  it('should <expected behavior> when <condition>', () => {
    // test body
  });
});
```

### Example
```typescript
describe('UserService', () => {
  it('should return user when id exists', async () => {
    // Arrange
    const service = new UserService(mockApi);
    mockApi.get.mockResolvedValue({ id: '1', name: 'Test' });

    // Act
    const user = await service.getUser('1');

    // Assert
    expect(user).toEqual({ id: '1', name: 'Test' });
  });

  it('should return null when id not found', async () => {
    mockApi.get.mockResolvedValue(null);
    const user = await service.getUser('999');
    expect(user).toBeNull();
  });
});
```

### Svelte Component Tests
```typescript
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import MyComponent from './MyComponent.svelte';

describe('MyComponent', () => {
  it('should handle user interaction', async () => {
    const user = userEvent.setup();
    render(MyComponent, { props: { initialValue: 'test' } });

    const button = screen.getByRole('button', { name: /click me/i });
    await user.click(button);

    expect(screen.getByText('Clicked')).toBeInTheDocument();
  });
});
```

## E2E Tests

### Page Object Pattern
```typescript
// e2e/pages/HomePage.ts
export class HomePage {
  constructor(private page: Page) {}

  async navigate() {
    await this.page.goto('/');
  }

  async clickLogin() {
    await this.page.click('[data-testid="login-button"]');
  }

  async isLoggedIn() {
    return await this.page.isVisible('[data-testid="user-menu"]');
  }
}
```

### Test Example
```typescript
import { test, expect } from '@playwright/test';
import { HomePage } from '../pages/HomePage';

test.describe('Login Flow', () => {
  test('should login successfully with valid credentials', async ({ page }) => {
    const homePage = new HomePage(page);
    
    await homePage.navigate();
    await homePage.clickLogin();
    
    await page.fill('[data-testid="username"]', 'test@example.com');
    await page.fill('[data-testid="password"]', 'password123');
    await page.click('[data-testid="submit"]');
    
    expect(await homePage.isLoggedIn()).toBe(true);
  });
});
```

## What to Test

### DO Test
- Public API behavior
- Edge cases (empty, null, boundaries)
- Error handling paths
- State transitions
- Integration points
- User workflows (E2E)

### DON'T Test
- Private implementation details
- Framework/library code
- Trivial getters/setters
- External service internals

## Test Data

### Builders
Use builders for complex objects:

```rust
let config = ConfigBuilder::new()
    .name("test")
    .enabled(true)
    .build();
```

```typescript
const user = UserBuilder.create()
  .withName('Test')
  .asAdmin()
  .build();
```

### Fixtures
Load test data from files:

```rust
let fixture = load_fixture("user_data.json")?;
let users: Vec<User> = serde_json::from_str(&fixture)?;
```

```typescript
import userFixture from '../fixtures/users.json';
```

### Generators
Use property-based test generators:

```rust
use proptest::prelude::*;

prop_compose! {
    fn arb_user()(
        name in "[a-zA-Z ]{1,50}",
        age in 18u8..120,
        active in any::<bool>()
    ) -> User {
        User { name, age, active }
    }
}
```

## Test Organization

### Group Related Tests
```rust
mod auth_tests {
    use super::*;

    mod login {
        use super::*;
        
        #[test]
        fn test_valid_credentials() { /* ... */ }
        
        #[test]
        fn test_invalid_credentials() { /* ... */ }
    }
    
    mod logout {
        use super::*;
        
        #[test]
        fn test_successful_logout() { /* ... */ }
    }
}
```

### Use Descriptive Names
- `test_user_service_get_user_existing_id_returns_user()`
- `test_config_parser_invalid_yaml_returns_parse_error()`
- `test_workflow_engine_timeout_cancels_execution()`

## Common Patterns

### Setup and Teardown
```rust
struct TestFixture {
    temp_dir: TempDir,
    db: Database,
}

impl TestFixture {
    fn new() -> Self {
        // Setup logic
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup logic
    }
}
```

### Async Testing
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Parameterized Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(1, 2, 3)]
    #[case(0, 0, 0)]
    #[case(-1, 1, 0)]
    fn test_add(#[case] a: i32, #[case] b: i32, #[case] expected: i32) {
        assert_eq!(add(a, b), expected);
    }
}
```