# Mocking Guide

This guide covers mocking strategies and patterns used in Tachikoma testing.

## Philosophy

- Mock external dependencies, not your own code
- Prefer real implementations for internal logic
- Use mocks to simulate error conditions and edge cases
- Keep mocks simple and focused

## Rust Mocking

### Trait-Based Mocking

```rust
// Define trait for mockable behavior
trait DatabaseService {
    async fn get_user(&self, id: UserId) -> Result<Option<User>>;
    async fn create_user(&self, user: NewUser) -> Result<User>;
}

// Production implementation
struct PostgresDatabase {
    pool: PgPool,
}

impl DatabaseService for PostgresDatabase {
    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        // Real implementation
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn create_user(&self, user: NewUser) -> Result<User> {
        // Real implementation
        sqlx::query_as!(
            User,
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
            user.name,
            user.email
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}

// Mock implementation
pub struct MockDatabase {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    fail_operations: Arc<Mutex<HashSet<String>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            fail_operations: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn add_user(&self, user: User) {
        self.users.lock().unwrap().insert(user.id, user);
    }

    pub fn fail_operation(&self, operation: &str) {
        self.fail_operations.lock().unwrap().insert(operation.to_string());
    }

    pub fn clear_failures(&self) {
        self.fail_operations.lock().unwrap().clear();
    }
}

impl DatabaseService for MockDatabase {
    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        if self.fail_operations.lock().unwrap().contains("get_user") {
            return Err(anyhow::anyhow!("Mock database failure"));
        }

        Ok(self.users.lock().unwrap().get(&id).cloned())
    }

    async fn create_user(&self, user: NewUser) -> Result<User> {
        if self.fail_operations.lock().unwrap().contains("create_user") {
            return Err(anyhow::anyhow!("Mock database failure"));
        }

        let new_user = User {
            id: UserId::new(),
            name: user.name,
            email: user.email,
            created_at: Utc::now(),
        };

        self.users.lock().unwrap().insert(new_user.id, new_user.clone());
        Ok(new_user)
    }
}
```

### Using Mockall for Automatic Mocking

```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
trait ExternalApi {
    async fn fetch_data(&self, id: &str) -> Result<ApiResponse>;
    async fn send_notification(&self, message: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_with_mock_api() {
        let mut mock_api = MockExternalApi::new();
        
        // Set up expectations
        mock_api
            .expect_fetch_data()
            .with(eq("test-id"))
            .times(1)
            .returning(|_| Ok(ApiResponse::default()));

        mock_api
            .expect_send_notification()
            .with(eq("Test message"))
            .times(1)
            .returning(|_| Ok(()));

        // Test the service
        let service = MyService::new(Box::new(mock_api));
        let result = service.process_data("test-id").await;

        assert!(result.is_ok());
    }
}
```

### HTTP Client Mocking with Wiremock

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header, body_json_schema};

#[tokio::test]
async fn test_api_client_handles_error_responses() {
    let mock_server = MockServer::start().await;

    // Mock successful response
    Mock::given(method("GET"))
        .and(path("/api/users/1"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(&json!({
                "id": 1,
                "name": "John Doe",
                "email": "john@example.com"
            })))
        .mount(&mock_server)
        .await;

    // Mock error response
    Mock::given(method("GET"))
        .and(path("/api/users/404"))
        .respond_with(ResponseTemplate::new(404)
            .set_body_json(&json!({
                "error": "User not found"
            })))
        .mount(&mock_server)
        .await;

    // Mock server error
    Mock::given(method("POST"))
        .and(path("/api/users"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(500)
            .set_delay(std::time::Duration::from_millis(100)))
        .mount(&mock_server)
        .await;

    let client = ApiClient::new(&mock_server.uri());

    // Test successful request
    let user = client.get_user(1).await.unwrap();
    assert_eq!(user.name, "John Doe");

    // Test 404 error
    let result = client.get_user(404).await;
    assert!(matches!(result, Err(ApiError::NotFound)));

    // Test server error
    let result = client.create_user(NewUser {
        name: "Jane".to_string(),
        email: "jane@example.com".to_string(),
    }).await;
    assert!(matches!(result, Err(ApiError::ServerError(_))));
}
```

### File System Mocking

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct MockFileSystem {
    files: HashMap<PathBuf, Vec<u8>>,
    directories: HashSet<PathBuf>,
    fail_operations: HashSet<PathBuf>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            directories: HashSet::new(),
            fail_operations: HashSet::new(),
        }
    }

    pub fn create_file(&mut self, path: impl Into<PathBuf>, content: impl Into<Vec<u8>>) {
        self.files.insert(path.into(), content.into());
    }

    pub fn create_dir(&mut self, path: impl Into<PathBuf>) {
        self.directories.insert(path.into());
    }

    pub fn fail_operation(&mut self, path: impl Into<PathBuf>) {
        self.fail_operations.insert(path.into());
    }
}

trait FileSystemOps {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, path: &Path, content: &[u8]) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
}

impl FileSystemOps for MockFileSystem {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        if self.fail_operations.contains(path) {
            return Err(anyhow::anyhow!("Mock filesystem failure"));
        }

        self.files.get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found"))
    }

    fn write_file(&self, path: &Path, content: &[u8]) -> Result<()> {
        if self.fail_operations.contains(path) {
            return Err(anyhow::anyhow!("Mock filesystem failure"));
        }

        // In real mock, would update internal state
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path) || self.directories.contains(path)
    }
}
```

## TypeScript Mocking

### Jest Mock Functions

```typescript
// Basic mocking
const mockFetch = jest.fn();
global.fetch = mockFetch;

// Mock implementation
mockFetch.mockImplementation((url: string) => {
  if (url.includes('/api/users')) {
    return Promise.resolve({
      json: () => Promise.resolve([{ id: 1, name: 'Test User' }])
    });
  }
  throw new Error('Unexpected URL');
});

// Mock return values
mockFetch
  .mockResolvedValueOnce({ json: () => Promise.resolve({ id: 1 }) })
  .mockResolvedValueOnce({ json: () => Promise.resolve({ id: 2 }) })
  .mockRejectedValueOnce(new Error('Network error'));
```

### Service Mocking

```typescript
interface UserService {
  getUser(id: string): Promise<User | null>;
  createUser(userData: CreateUserData): Promise<User>;
}

class MockUserService implements UserService {
  private users = new Map<string, User>();
  private shouldFail = new Set<string>();

  addUser(user: User) {
    this.users.set(user.id, user);
  }

  failOperation(operation: string) {
    this.shouldFail.add(operation);
  }

  clearFailures() {
    this.shouldFail.clear();
  }

  async getUser(id: string): Promise<User | null> {
    if (this.shouldFail.has('getUser')) {
      throw new Error('Mock service failure');
    }
    return this.users.get(id) || null;
  }

  async createUser(userData: CreateUserData): Promise<User> {
    if (this.shouldFail.has('createUser')) {
      throw new Error('Mock service failure');
    }

    const user: User = {
      id: Math.random().toString(),
      ...userData,
      createdAt: new Date()
    };

    this.users.set(user.id, user);
    return user;
  }
}

// Usage in tests
describe('UserManager', () => {
  let mockUserService: MockUserService;
  let userManager: UserManager;

  beforeEach(() => {
    mockUserService = new MockUserService();
    userManager = new UserManager(mockUserService);
  });

  it('should handle service failures gracefully', async () => {
    mockUserService.failOperation('getUser');
    
    const result = await userManager.getUserSafely('123');
    
    expect(result).toBeNull();
  });
});
```

### Vitest Mock Utilities

```typescript
import { vi, describe, it, expect, beforeEach } from 'vitest';

// Mock modules
vi.mock('../services/apiService', () => ({
  ApiService: vi.fn(() => ({
    get: vi.fn(),
    post: vi.fn()
  }))
}));

// Mock timers
vi.useFakeTimers();

describe('Timer functionality', () => {
  it('should execute callback after delay', () => {
    const callback = vi.fn();
    
    delayedExecute(callback, 1000);
    
    // Fast-forward time
    vi.advanceTimersByTime(1000);
    
    expect(callback).toHaveBeenCalledOnce();
  });
});

// Mock browser APIs
Object.defineProperty(window, 'localStorage', {
  value: {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn(),
    clear: vi.fn()
  }
});

// Mock fetch with MSW (Mock Service Worker)
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

const server = setupServer(
  http.get('/api/users', () => {
    return HttpResponse.json([
      { id: '1', name: 'John Doe' },
      { id: '2', name: 'Jane Smith' }
    ]);
  }),
  
  http.post('/api/users', async ({ request }) => {
    const user = await request.json();
    return HttpResponse.json(
      { id: '3', ...user },
      { status: 201 }
    );
  }),

  http.get('/api/users/:id', ({ params }) => {
    const { id } = params;
    if (id === '404') {
      return new HttpResponse(null, { status: 404 });
    }
    return HttpResponse.json({ id, name: `User ${id}` });
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### Svelte Component Mocking

```typescript
import { render, screen } from '@testing-library/svelte';
import { vi } from 'vitest';
import UserProfile from './UserProfile.svelte';

// Mock store
const mockUserStore = {
  subscribe: vi.fn(),
  set: vi.fn(),
  update: vi.fn()
};

vi.mock('$lib/stores/user', () => ({
  userStore: mockUserStore
}));

describe('UserProfile', () => {
  it('should display user information', () => {
    const mockUser = { id: '1', name: 'John Doe', email: 'john@example.com' };
    
    // Mock store subscription
    mockUserStore.subscribe.mockImplementation((callback) => {
      callback(mockUser);
      return () => {}; // unsubscribe function
    });

    render(UserProfile);

    expect(screen.getByText('John Doe')).toBeInTheDocument();
    expect(screen.getByText('john@example.com')).toBeInTheDocument();
  });
});

// Mock context
import { setContext } from 'svelte';

const renderWithContext = (component: any, props: any, context: Record<string, any>) => {
  return render(component, {
    props,
    context: new Map(Object.entries(context))
  });
};

it('should use injected services', () => {
  const mockApiService = {
    getUser: vi.fn().mockResolvedValue({ id: '1', name: 'Test' })
  };

  renderWithContext(UserProfile, { userId: '1' }, {
    apiService: mockApiService
  });

  expect(mockApiService.getUser).toHaveBeenCalledWith('1');
});
```

## E2E Mocking with Playwright

### API Route Mocking

```typescript
import { test, expect } from '@playwright/test';

test('should handle API responses', async ({ page }) => {
  // Mock API responses
  await page.route('/api/users', async route => {
    const json = [
      { id: 1, name: 'John Doe' },
      { id: 2, name: 'Jane Smith' }
    ];
    await route.fulfill({ json });
  });

  // Mock error responses
  await page.route('/api/users/error', async route => {
    await route.fulfill({
      status: 500,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'Internal Server Error' })
    });
  });

  await page.goto('/users');
  
  await expect(page.locator('[data-testid="user-list"]')).toContainText('John Doe');
});

// Mock with conditions
test('should handle different scenarios', async ({ page }) => {
  await page.route('/api/data', async (route, request) => {
    const url = new URL(request.url());
    const filter = url.searchParams.get('filter');
    
    if (filter === 'active') {
      await route.fulfill({
        json: { items: [{ id: 1, status: 'active' }] }
      });
    } else {
      await route.fulfill({
        json: { items: [] }
      });
    }
  });

  await page.goto('/dashboard?filter=active');
  await expect(page.locator('.item')).toHaveCount(1);
});
```

### Network Conditions

```typescript
test('should handle slow network', async ({ page, context }) => {
  // Simulate slow network
  await context.route('/api/slow-endpoint', async route => {
    await new Promise(resolve => setTimeout(resolve, 5000));
    await route.fulfill({ json: { data: 'slow response' } });
  });

  await page.goto('/slow-page');
  
  // Verify loading state
  await expect(page.locator('[data-testid="loading"]')).toBeVisible();
  
  // Wait for slow response
  await expect(page.locator('[data-testid="content"]')).toBeVisible({ timeout: 10000 });
});

// Simulate network failure
test('should handle network errors', async ({ page }) => {
  await page.route('/api/critical-data', route => route.abort('failed'));

  await page.goto('/dashboard');
  
  await expect(page.locator('[data-testid="error-message"]')).toBeVisible();
});
```

## Mock Data Strategies

### Factory Pattern for Test Data

```rust
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn user() -> User {
        User {
            id: UserId::new(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            active: true,
            created_at: Utc::now(),
        }
    }

    pub fn admin_user() -> User {
        User {
            role: Role::Admin,
            ..Self::user()
        }
    }

    pub fn inactive_user() -> User {
        User {
            active: false,
            ..Self::user()
        }
    }

    pub fn config() -> Config {
        Config {
            database_url: "mock://database".to_string(),
            api_key: "test-key".to_string(),
            debug: true,
            ..Default::default()
        }
    }
}
```

### Fixture Management

```typescript
export class TestDataFixtures {
  static user(overrides: Partial<User> = {}): User {
    return {
      id: '1',
      name: 'Test User',
      email: 'test@example.com',
      active: true,
      createdAt: new Date('2024-01-01'),
      ...overrides
    };
  }

  static users(count: number): User[] {
    return Array.from({ length: count }, (_, i) => 
      this.user({ id: (i + 1).toString(), name: `User ${i + 1}` })
    );
  }

  static apiResponse<T>(data: T, overrides: Partial<ApiResponse<T>> = {}): ApiResponse<T> {
    return {
      data,
      status: 'success',
      timestamp: new Date().toISOString(),
      ...overrides
    };
  }
}
```

## Best Practices

### Mock Verification

```rust
#[test]
async fn test_service_calls_dependencies_correctly() {
    let mut mock_db = MockDatabase::new();
    let mut mock_api = MockExternalApi::new();

    // Set up expectations
    mock_db
        .expect_get_user()
        .with(eq(UserId::from("123")))
        .times(1)
        .returning(|_| Ok(Some(TestDataFactory::user())));

    mock_api
        .expect_send_notification()
        .times(1)
        .returning(|_| Ok(()));

    let service = MyService::new(mock_db, mock_api);
    service.process_user("123").await.unwrap();

    // Mocks are automatically verified on drop
}
```

### Mock Lifecycle Management

```typescript
describe('Service Tests', () => {
  let mockService: MockApiService;

  beforeEach(() => {
    mockService = new MockApiService();
    // Reset all mocks to clean state
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Verify no unexpected calls were made
    expect(mockService.unexpectedCalls()).toEqual([]);
  });
});
```

### Avoiding Over-Mocking

```rust
// DON'T mock everything
#[test]
fn bad_test_with_too_many_mocks() {
    let mock_string_parser = MockStringParser::new();
    let mock_number_formatter = MockNumberFormatter::new();
    let mock_date_validator = MockDateValidator::new();
    // ... too many mocks make test unclear
}

// DO focus on external boundaries
#[test]
async fn good_test_with_focused_mocking() {
    let mock_database = MockDatabase::new(); // External dependency
    
    // Use real implementations for internal logic
    let parser = RealStringParser::new();
    let formatter = RealNumberFormatter::new();
    
    let service = MyService::new(mock_database, parser, formatter);
    // Test focuses on behavior, not implementation
}
```

These patterns help create reliable, maintainable mocks that accurately represent external dependencies while keeping tests focused and fast.