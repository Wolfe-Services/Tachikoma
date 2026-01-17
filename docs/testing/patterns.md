# Test Patterns

This guide covers common testing patterns and best practices used in the Tachikoma project.

## Rust Test Patterns

### Builder Pattern for Test Data
```rust
pub struct ConfigBuilder {
    name: Option<String>,
    enabled: bool,
    settings: HashMap<String, Value>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            enabled: false,
            settings: HashMap::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn setting(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.settings.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Config {
        Config {
            name: self.name.unwrap_or_else(|| "default".to_string()),
            enabled: self.enabled,
            settings: self.settings,
        }
    }
}
```

### Test Context Pattern
```rust
use tachikoma_test_harness::prelude::*;

pub struct TestContext {
    pub temp_dir: TempDir,
    pub db: Database,
    pub server: MockServer,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let db = Database::in_memory().await?;
        let server = MockServer::start().await;

        Ok(Self { temp_dir, db, server })
    }

    pub fn builder() -> TestContextBuilder {
        TestContextBuilder::new()
    }
}

#[tokio::test]
async fn test_with_context() {
    let ctx = TestContext::new().await.unwrap();
    
    // Use context for testing
    let result = process_data(&ctx.db, &ctx.temp_dir).await;
    
    assert!(result.is_ok());
}
```

### Property-Based Testing
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_serialization_roundtrip(
        config in arb_config()
    ) {
        let serialized = serde_json::to_string(&config)?;
        let deserialized: Config = serde_json::from_str(&serialized)?;
        prop_assert_eq!(config, deserialized);
    }
}

fn arb_config() -> impl Strategy<Value = Config> {
    (
        "[a-zA-Z0-9_]{1,20}",
        any::<bool>(),
        prop::collection::hash_map("[a-z]+", any::<i32>(), 0..10),
    ).prop_map(|(name, enabled, settings)| {
        Config {
            name,
            enabled,
            settings: settings.into_iter()
                .map(|(k, v)| (k, Value::Number(v.into())))
                .collect(),
        }
    })
}
```

### Snapshot Testing
```rust
use insta::assert_snapshot;

#[test]
fn test_render_output() {
    let config = Config::default();
    let output = render_config(&config);
    
    assert_snapshot!(output);
}

#[test]
fn test_debug_output() {
    let data = ComplexData::generate();
    assert_debug_snapshot!(data);
}
```

### Async Test Patterns
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    let handles = (0..10).map(|i| {
        tokio::spawn(async move {
            perform_operation(i).await
        })
    }).collect::<Vec<_>>();

    let results = futures::future::try_join_all(handles).await.unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results.into_iter().all(|r| r.is_ok()));
}
```

## TypeScript Test Patterns

### Service Mocking
```typescript
interface ApiService {
  get(url: string): Promise<any>;
  post(url: string, data: any): Promise<any>;
}

class MockApiService implements ApiService {
  private responses = new Map<string, any>();

  mockResponse(url: string, response: any) {
    this.responses.set(url, response);
  }

  async get(url: string): Promise<any> {
    const response = this.responses.get(url);
    if (!response) throw new Error(`No mock response for ${url}`);
    return response;
  }

  async post(url: string, data: any): Promise<any> {
    // Mock implementation
    return { success: true, data };
  }
}
```

### Component Testing with Context
```typescript
import { render, screen } from '@testing-library/svelte';
import { writable } from 'svelte/store';
import { setContext } from 'svelte';

const createTestWrapper = (component: any, props: any, context: any = {}) => {
  return render(component, {
    props,
    context: new Map(Object.entries(context))
  });
};

test('should use context correctly', () => {
  const mockStore = writable({ user: { name: 'Test' } });
  
  createTestWrapper(UserProfile, {}, {
    userStore: mockStore
  });

  expect(screen.getByText('Test')).toBeInTheDocument();
});
```

### Store Testing
```typescript
import { get } from 'svelte/store';
import { createUserStore } from './userStore';

describe('UserStore', () => {
  let store: ReturnType<typeof createUserStore>;

  beforeEach(() => {
    store = createUserStore();
  });

  it('should update user data', () => {
    const user = { id: '1', name: 'Test' };
    
    store.setUser(user);
    
    expect(get(store.user)).toEqual(user);
  });

  it('should handle async operations', async () => {
    const promise = store.loadUser('1');
    
    expect(get(store.loading)).toBe(true);
    
    await promise;
    
    expect(get(store.loading)).toBe(false);
    expect(get(store.user)).toBeDefined();
  });
});
```

### Page Object Model (E2E)
```typescript
export class BasePage {
  constructor(protected page: Page) {}

  async navigate(path: string) {
    await this.page.goto(path);
  }

  async waitForLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  async clickElement(selector: string) {
    await this.page.click(selector);
  }

  async fillInput(selector: string, value: string) {
    await this.page.fill(selector, value);
  }

  async getText(selector: string): Promise<string> {
    return await this.page.textContent(selector) || '';
  }
}

export class LoginPage extends BasePage {
  async login(username: string, password: string) {
    await this.fillInput('[data-testid="username"]', username);
    await this.fillInput('[data-testid="password"]', password);
    await this.clickElement('[data-testid="login-button"]');
  }

  async isErrorVisible(): Promise<boolean> {
    return await this.page.isVisible('[data-testid="error-message"]');
  }
}
```

## Test Organization Patterns

### Feature-Based Structure
```
tests/
  auth/
    login.test.ts
    logout.test.ts
    permissions.test.ts
  user/
    profile.test.ts
    settings.test.ts
  workflow/
    creation.test.ts
    execution.test.ts
```

### Test Suites
```typescript
describe('Authentication System', () => {
  describe('Login', () => {
    it('should accept valid credentials', () => {});
    it('should reject invalid credentials', () => {});
  });

  describe('Logout', () => {
    it('should clear session', () => {});
    it('should redirect to login', () => {});
  });

  describe('Session Management', () => {
    it('should refresh expired tokens', () => {});
    it('should handle concurrent requests', () => {});
  });
});
```

## Mock Patterns

### Dependency Injection
```rust
trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

struct MockEmailService {
    sent_emails: Vec<(String, String, String)>,
}

impl EmailService for MockEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        self.sent_emails.push((to.to_string(), subject.to_string(), body.to_string()));
        Ok(())
    }
}

#[test]
fn test_user_registration_sends_welcome_email() {
    let mock_email = MockEmailService::new();
    let user_service = UserService::new(Box::new(mock_email));
    
    user_service.register_user("test@example.com", "password")?;
    
    assert_eq!(mock_email.sent_emails.len(), 1);
    assert_eq!(mock_email.sent_emails[0].0, "test@example.com");
}
```

### HTTP Mock Server
```rust
use wiremock::{MockServer, Mock, ResponseTemplate};

#[tokio::test]
async fn test_api_client() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/users/1"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(&json!({"id": 1, "name": "Test"})))
        .mount(&mock_server)
        .await;
    
    let client = ApiClient::new(&mock_server.uri());
    let user = client.get_user(1).await?;
    
    assert_eq!(user.name, "Test");
}
```

## Error Testing Patterns

### Error Scenarios
```rust
#[test]
fn test_file_not_found_error() {
    let result = read_config("nonexistent.toml");
    
    match result {
        Err(ConfigError::FileNotFound(path)) => {
            assert_eq!(path, "nonexistent.toml");
        }
        _ => panic!("Expected FileNotFound error"),
    }
}
```

### Should Panic Tests
```rust
#[test]
#[should_panic(expected = "Division by zero")]
fn test_divide_by_zero_panics() {
    divide(10, 0);
}
```

## Performance Testing Patterns

### Benchmarking
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse_config(c: &mut Criterion) {
    let config_str = include_str!("../fixtures/large_config.toml");
    
    c.bench_function("parse_config", |b| {
        b.iter(|| parse_config(black_box(config_str)))
    });
}

criterion_group!(benches, benchmark_parse_config);
criterion_main!(benches);
```

### Load Testing
```typescript
import { check } from 'k6';
import http from 'k6/http';

export let options = {
  stages: [
    { duration: '30s', target: 10 },
    { duration: '1m', target: 50 },
    { duration: '30s', target: 0 },
  ],
};

export default function () {
  const response = http.get('http://localhost:3000/api/users');
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
}
```

## Test Data Management

### Factories
```rust
pub struct UserFactory;

impl UserFactory {
    pub fn create() -> User {
        User {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            active: true,
            created_at: Utc::now(),
        }
    }

    pub fn with_name(name: impl Into<String>) -> User {
        User {
            name: name.into(),
            ..Self::create()
        }
    }

    pub fn inactive() -> User {
        User {
            active: false,
            ..Self::create()
        }
    }
}
```

### Fixture Loading
```rust
use std::fs;
use serde_json::Value;

pub fn load_fixture(name: &str) -> Result<Value> {
    let path = format!("tests/fixtures/{}", name);
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

#[test]
fn test_with_fixture_data() {
    let users = load_fixture("users.json").unwrap();
    let user_list: Vec<User> = serde_json::from_value(users).unwrap();
    
    assert!(!user_list.is_empty());
}
```

## Visual Testing Patterns

### Screenshot Comparison
```typescript
test('should match visual baseline', async ({ page }) => {
  await page.goto('/dashboard');
  
  await expect(page).toHaveScreenshot('dashboard.png', {
    fullPage: true,
    threshold: 0.2,
  });
});
```

### Component Visual Testing
```typescript
test('should render button variants correctly', async ({ page }) => {
  await page.goto('/storybook/button');
  
  const button = page.locator('[data-testid="primary-button"]');
  await expect(button).toHaveScreenshot('primary-button.png');
  
  const disabledButton = page.locator('[data-testid="disabled-button"]');
  await expect(disabledButton).toHaveScreenshot('disabled-button.png');
});
```

These patterns provide a solid foundation for writing maintainable and effective tests across the Tachikoma codebase.