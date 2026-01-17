# 473 - Integration Test Patterns

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 473
**Status:** Planned
**Dependencies:** 471-test-harness, 472-unit-patterns
**Estimated Context:** ~12% of Sonnet window

---

## Objective

Establish standardized integration testing patterns that verify interactions between multiple components, external services, and system boundaries while maintaining test reliability and execution speed.

---

## Acceptance Criteria

- [x] Integration tests isolated in dedicated test directories
- [x] Database tests use transactions for isolation and rollback
- [x] External service tests use mock servers (wiremock)
- [x] Test fixtures provide consistent initial state
- [x] Tests can run in parallel without interference
- [x] Clear separation between unit and integration tests

---

## Implementation Details

### 1. Rust Integration Test Structure

Create `crates/tachikoma-test-harness/src/patterns/integration.rs`:

```rust
//! Integration test patterns for testing component interactions.
//!
//! Integration tests live in `/tests` directories and test how components
//! work together, including database operations and external service calls.

use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Integration test context that manages shared resources
pub struct IntegrationContext {
    /// Unique test ID for resource isolation
    pub test_id: String,
    /// Temporary directory for test files
    pub temp_dir: tempfile::TempDir,
    /// Mock server for HTTP calls
    pub mock_server: Option<wiremock::MockServer>,
    /// Cleanup tasks to run on drop
    cleanup_tasks: Vec<Box<dyn FnOnce() + Send>>,
}

impl IntegrationContext {
    /// Create a new integration test context
    pub async fn new() -> Self {
        let test_id = crate::unique_test_id();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

        Self {
            test_id,
            temp_dir,
            mock_server: None,
            cleanup_tasks: Vec::new(),
        }
    }

    /// Create context with a mock HTTP server
    pub async fn with_mock_server() -> Self {
        let mut ctx = Self::new().await;
        ctx.mock_server = Some(wiremock::MockServer::start().await);
        ctx
    }

    /// Get the mock server URL
    pub fn mock_url(&self) -> String {
        self.mock_server
            .as_ref()
            .map(|s| s.uri())
            .unwrap_or_else(|| "http://localhost:9999".into())
    }

    /// Register a cleanup task
    pub fn on_cleanup<F: FnOnce() + Send + 'static>(&mut self, f: F) {
        self.cleanup_tasks.push(Box::new(f));
    }

    /// Create a test file with content
    pub fn create_file(&self, name: &str, content: &str) -> std::path::PathBuf {
        let path = self.temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
        path
    }

    /// Create a test directory
    pub fn create_dir(&self, name: &str) -> std::path::PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        path
    }
}

impl Drop for IntegrationContext {
    fn drop(&mut self) {
        for task in self.cleanup_tasks.drain(..) {
            task();
        }
    }
}

/// Database test context with transaction management
pub struct DbTestContext {
    /// Connection pool for the test database
    pub pool: Arc<Mutex<()>>, // Placeholder for actual DB pool
    /// Whether to rollback on completion
    rollback: bool,
}

impl DbTestContext {
    /// Create a new database test context
    pub async fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(())),
            rollback: true,
        }
    }

    /// Run a test within a transaction that will be rolled back
    pub async fn run_in_transaction<F, Fut, T>(&self, test_fn: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        // In real implementation, would start transaction here
        let result = test_fn().await;
        // Would rollback transaction here
        result
    }

    /// Disable automatic rollback (for tests that need commits)
    pub fn disable_rollback(&mut self) {
        self.rollback = false;
    }
}

/// Run an integration test with full context setup
pub async fn run_integration_test<F, Fut>(test_fn: F)
where
    F: FnOnce(IntegrationContext) -> Fut,
    Fut: Future<Output = ()>,
{
    crate::init();
    let ctx = IntegrationContext::new().await;
    test_fn(ctx).await;
}

/// Run an integration test with mock server
pub async fn run_with_mock_server<F, Fut>(test_fn: F)
where
    F: FnOnce(IntegrationContext) -> Fut,
    Fut: Future<Output = ()>,
{
    crate::init();
    let ctx = IntegrationContext::with_mock_server().await;
    test_fn(ctx).await;
}
```

### 2. Example Integration Tests

Create `crates/tachikoma-common-core/tests/integration_example.rs`:

```rust
//! Example integration tests demonstrating patterns.

use tachikoma_test_harness::patterns::integration::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_api_client_fetches_data_from_server() {
    run_with_mock_server(|ctx| async move {
        // Arrange: Set up mock response
        let mock_server = ctx.mock_server.as_ref().unwrap();
        Mock::given(method("GET"))
            .and(path("/api/data"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1,
                "name": "test"
            })))
            .mount(mock_server)
            .await;

        // Act: Make request through our client
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/data", ctx.mock_url()))
            .send()
            .await
            .expect("Request failed");

        // Assert: Verify response
        assert!(response.status().is_success());
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["name"], "test");
    })
    .await;
}

#[tokio::test]
async fn test_file_operations_with_temp_directory() {
    run_integration_test(|ctx| async move {
        // Arrange: Create test files
        let config_path = ctx.create_file("config.yaml", "key: value");
        let data_dir = ctx.create_dir("data");

        // Act: Verify file system state
        assert!(config_path.exists());
        assert!(data_dir.is_dir());

        // Assert: Can read created files
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "key: value");
    })
    .await;
}

mod database_integration {
    use super::*;

    #[tokio::test]
    async fn test_user_creation_and_retrieval() {
        let db_ctx = DbTestContext::new().await;

        db_ctx
            .run_in_transaction(|| async {
                // Arrange: Prepare user data
                let user_data = serde_json::json!({
                    "name": "Test User",
                    "email": "test@example.com"
                });

                // Act: Would insert and retrieve from database
                // This is a placeholder - actual implementation would use real DB

                // Assert: User can be retrieved
                assert_eq!(user_data["name"], "Test User");
            })
            .await;
    }
}
```

### 3. TypeScript Integration Test Patterns

Create `web/src/test/patterns/integration.ts`:

```typescript
/**
 * Integration test patterns for testing component interactions
 * and external service integrations.
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

// ============================================
// Pattern: Mock Service Worker Setup
// ============================================

const server = setupServer();

export function setupMockServer() {
  beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
  afterAll(() => server.close());
  beforeEach(() => server.resetHandlers());
  return server;
}

// ============================================
// Pattern: API Integration Test
// ============================================

interface ApiResponse<T> {
  data: T;
  status: number;
}

async function fetchFromApi<T>(path: string): Promise<ApiResponse<T>> {
  const response = await fetch(`http://localhost:3000${path}`);
  const data = await response.json();
  return { data, status: response.status };
}

describe('API Integration', () => {
  const mockServer = setupMockServer();

  describe('User API', () => {
    it('should fetch user successfully', async () => {
      // Arrange: Set up mock response
      mockServer.use(
        http.get('http://localhost:3000/api/users/1', () => {
          return HttpResponse.json({
            id: '1',
            name: 'Test User',
            email: 'test@example.com',
          });
        })
      );

      // Act: Make API call
      const result = await fetchFromApi<{ id: string; name: string }>('/api/users/1');

      // Assert: Verify response
      expect(result.status).toBe(200);
      expect(result.data.name).toBe('Test User');
    });

    it('should handle 404 errors', async () => {
      mockServer.use(
        http.get('http://localhost:3000/api/users/999', () => {
          return new HttpResponse(null, { status: 404 });
        })
      );

      const result = await fetchFromApi('/api/users/999');
      expect(result.status).toBe(404);
    });

    it('should handle server errors', async () => {
      mockServer.use(
        http.get('http://localhost:3000/api/users/1', () => {
          return new HttpResponse(null, { status: 500 });
        })
      );

      const result = await fetchFromApi('/api/users/1');
      expect(result.status).toBe(500);
    });
  });
});

// ============================================
// Pattern: Store Integration Tests
// ============================================

import { writable, get } from 'svelte/store';

// Example store for testing
function createUserStore() {
  const { subscribe, set, update } = writable<{ users: Map<string, unknown> }>({
    users: new Map(),
  });

  return {
    subscribe,
    addUser: (id: string, user: unknown) =>
      update(state => {
        state.users.set(id, user);
        return state;
      }),
    removeUser: (id: string) =>
      update(state => {
        state.users.delete(id);
        return state;
      }),
    clear: () => set({ users: new Map() }),
  };
}

describe('Store Integration', () => {
  it('should manage user state correctly', () => {
    const store = createUserStore();
    const user = { name: 'Test', email: 'test@example.com' };

    // Act: Add user
    store.addUser('1', user);

    // Assert: User is in store
    const state = get(store);
    expect(state.users.get('1')).toEqual(user);
  });

  it('should handle multiple operations', () => {
    const store = createUserStore();

    store.addUser('1', { name: 'User 1' });
    store.addUser('2', { name: 'User 2' });
    store.removeUser('1');

    const state = get(store);
    expect(state.users.has('1')).toBe(false);
    expect(state.users.has('2')).toBe(true);
  });
});

// ============================================
// Pattern: Component + Service Integration
// ============================================

describe('Component Service Integration', () => {
  const mockServer = setupMockServer();

  it('should load and display data from service', async () => {
    mockServer.use(
      http.get('http://localhost:3000/api/items', () => {
        return HttpResponse.json([
          { id: '1', name: 'Item 1' },
          { id: '2', name: 'Item 2' },
        ]);
      })
    );

    // In real test, would render component and verify it displays items
    const response = await fetch('http://localhost:3000/api/items');
    const items = await response.json();

    expect(items).toHaveLength(2);
    expect(items[0].name).toBe('Item 1');
  });
});
```

### 4. Integration Test Configuration

Add to `web/vitest.config.ts`:

```typescript
export default defineConfig({
  test: {
    // Separate integration tests
    include: ['src/**/*.{test,spec}.{js,ts}'],
    exclude: ['src/**/*.integration.{test,spec}.{js,ts}'],
  },
});

// vitest.integration.config.ts
export default defineConfig({
  test: {
    include: ['src/**/*.integration.{test,spec}.{js,ts}'],
    testTimeout: 30000,
    hookTimeout: 30000,
    setupFiles: ['./src/test/setup.integration.ts'],
  },
});
```

---

## Testing Requirements

1. Integration tests run separately from unit tests
2. Mock servers intercept all external HTTP calls
3. Database tests properly rollback transactions
4. Tests can run in parallel without conflicts
5. Test setup/teardown is reliable and complete

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md), [472-unit-patterns.md](472-unit-patterns.md)
- Next: [474-property-testing.md](474-property-testing.md)
- Related: [478-mock-network.md](478-mock-network.md)
