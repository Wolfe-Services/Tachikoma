# Debugging Tests

This guide covers debugging strategies for different types of tests in the Tachikoma project.

## General Debugging Principles

1. **Isolate the problem** - Run the failing test in isolation
2. **Enable verbose output** - Use logging and detailed error messages
3. **Check assumptions** - Verify test setup and mock configurations
4. **Use debuggers** - Step through code when necessary
5. **Reproduce locally** - Ensure the same environment as CI

## Rust Test Debugging

### Running Single Tests

```bash
# Run a specific test
cargo test test_name -- --nocapture

# Run tests matching a pattern
cargo test auth -- --nocapture

# Run tests in a specific module
cargo test auth::login -- --nocapture

# Show all output (including passed tests)
cargo test -- --nocapture --test-threads=1
```

### Enabling Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo test test_name -- --nocapture

# Enable trace logging for specific modules
RUST_LOG=tachikoma_auth=trace cargo test -- --nocapture

# Enable logging for multiple modules
RUST_LOG=tachikoma_auth=debug,tachikoma_db=info cargo test -- --nocapture
```

### Using Nextest for Better Output

```bash
# Run with nextest (better formatting)
cargo nextest run

# Run specific test with detailed output
cargo nextest run test_name --no-capture

# Run with failure summary
cargo nextest run --failure-output immediate-final

# Run with timing information
cargo nextest run --final-status-level all
```

### Debug with Debugger

#### Using LLDB (macOS/Linux)

```bash
# Build test binary with debug symbols
cargo test --no-run test_name

# Find the test binary path (from previous command output)
# Run with LLDB
lldb ./target/debug/deps/my_crate-<hash>

# Set breakpoints and run
(lldb) b test_name
(lldb) run
```

#### Using GDB (Linux)

```bash
# Build test binary
cargo test --no-run test_name

# Run with GDB
gdb ./target/debug/deps/my_crate-<hash>

# Set breakpoints
(gdb) break test_name
(gdb) run
```

### Debugging Async Tests

```rust
#[tokio::test]
async fn test_async_operation() {
    // Enable tokio console for async debugging
    console_subscriber::init();
    
    // Add detailed logging
    tracing::info!("Starting async test");
    
    let result = async_operation().await;
    
    // Use debug assertions
    dbg!(&result);
    
    assert!(result.is_ok());
}
```

### Memory Debugging

```bash
# Run with Valgrind (Linux)
cargo test --target x86_64-unknown-linux-gnu
valgrind --tool=memcheck ./target/x86_64-unknown-linux-gnu/debug/deps/my_crate-<hash>

# Use Address Sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Use Miri for undefined behavior detection
cargo +nightly miri test
```

### Property Test Debugging

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property(input in 0..1000u32) {
        // Enable debugging for failing cases
        println!("Testing with input: {}", input);
        
        let result = my_function(input);
        
        // Add detailed assertions
        prop_assert!(
            result.is_ok(),
            "Function failed with input {}: {:?}",
            input,
            result
        );
    }
}

// To debug a specific failing case:
#[test]
fn test_specific_failure() {
    // Use the exact input that caused the property test to fail
    let input = 42; // From property test output
    let result = my_function(input);
    assert!(result.is_ok());
}
```

## TypeScript Test Debugging

### Running Specific Tests

```bash
# Run specific test file
npm test -- UserService.test.ts

# Run tests matching pattern
npm test -- --grep "should handle errors"

# Run single test suite
npm test -- --grep "UserService"

# Run with reporter for better output
npm test -- --reporter verbose
```

### Debugging with Node Inspector

```bash
# Run tests with inspector
npm test -- --inspect-brk

# Then connect debugger (Chrome DevTools or VS Code)
# Navigate to chrome://inspect in Chrome
```

### Vitest Debugging

```bash
# Run in UI mode (interactive debugging)
npx vitest --ui

# Run with debugging enabled
npx vitest --inspect-brk

# Run specific test with verbose output
npx vitest run UserService.test.ts --reporter=verbose

# Watch mode for continuous debugging
npx vitest watch UserService.test.ts
```

### Console Debugging

```typescript
describe('UserService', () => {
  it('should handle complex scenario', async () => {
    console.log('Starting test with mock setup');
    
    const mockApi = new MockApiService();
    console.log('Mock API created:', mockApi);
    
    mockApi.mockResponse('/users/1', { id: 1, name: 'Test' });
    console.log('Mock response configured');
    
    const service = new UserService(mockApi);
    
    try {
      const result = await service.getUser('1');
      console.log('Service result:', result);
      
      expect(result).toEqual({ id: 1, name: 'Test' });
    } catch (error) {
      console.error('Test failed with error:', error);
      console.error('Mock call history:', mockApi.getCallHistory());
      throw error;
    }
  });
});
```

### Debugging Store Tests

```typescript
import { get } from 'svelte/store';

describe('UserStore', () => {
  it('should update state correctly', () => {
    const store = createUserStore();
    
    // Log initial state
    console.log('Initial state:', get(store));
    
    // Subscribe to changes for debugging
    const unsubscribe = store.subscribe(state => {
      console.log('Store state changed:', state);
    });
    
    // Perform actions
    store.setUser({ id: '1', name: 'Test' });
    
    // Check final state
    const finalState = get(store);
    console.log('Final state:', finalState);
    
    expect(finalState.user).toBeTruthy();
    
    unsubscribe();
  });
});
```

### Mock Debugging

```typescript
describe('Service with complex mocking', () => {
  let mockApi: jest.Mocked<ApiService>;

  beforeEach(() => {
    mockApi = {
      get: jest.fn(),
      post: jest.fn(),
    } as jest.Mocked<ApiService>;
  });

  it('should debug mock interactions', async () => {
    // Set up mock with detailed logging
    mockApi.get.mockImplementation((url) => {
      console.log('Mock API called with URL:', url);
      if (url === '/users/1') {
        return Promise.resolve({ id: 1, name: 'Test' });
      }
      throw new Error(`Unexpected URL: ${url}`);
    });

    const service = new UserService(mockApi);
    
    try {
      await service.processUser('1');
    } catch (error) {
      console.error('Service error:', error);
      console.log('Mock call count:', mockApi.get.mock.calls.length);
      console.log('Mock calls:', mockApi.get.mock.calls);
      console.log('Mock results:', mockApi.get.mock.results);
      throw error;
    }

    // Verify mock interactions
    expect(mockApi.get).toHaveBeenCalledWith('/users/1');
  });
});
```

## E2E Test Debugging

### Playwright Debug Mode

```bash
# Run in debug mode (opens browser)
npx playwright test --debug

# Debug specific test
npx playwright test auth.spec.ts --debug

# Run in headed mode (see browser)
npx playwright test --headed

# Run with slow motion
npx playwright test --headed --slow-mo=1000
```

### Playwright UI Mode

```bash
# Interactive test runner
npx playwright test --ui

# Run specific test in UI mode
npx playwright test auth.spec.ts --ui
```

### Screenshots and Videos

```typescript
test('debugging with screenshots', async ({ page }) => {
  await page.goto('/login');
  
  // Take screenshot for debugging
  await page.screenshot({ path: 'debug-login-page.png' });
  
  await page.fill('#username', 'test@example.com');
  await page.fill('#password', 'password123');
  
  // Screenshot before clicking submit
  await page.screenshot({ path: 'debug-before-submit.png' });
  
  await page.click('#submit');
  
  // Wait and screenshot result
  await page.waitForURL('/dashboard');
  await page.screenshot({ path: 'debug-after-login.png' });
});
```

### Tracing

```typescript
// Enable tracing in playwright.config.ts
use: {
  trace: 'on-first-retry',
  screenshot: 'only-on-failure',
  video: 'retain-on-failure',
}

// Or enable programmatically
test('with manual tracing', async ({ page, context }) => {
  await context.tracing.start({ screenshots: true, snapshots: true });
  
  await page.goto('/complex-page');
  // ... test steps ...
  
  await context.tracing.stop({ path: 'trace.zip' });
});
```

View traces with:
```bash
npx playwright show-trace trace.zip
```

### Console and Network Debugging

```typescript
test('debug network requests', async ({ page }) => {
  // Listen to console messages
  page.on('console', msg => {
    console.log(`Browser console (${msg.type()}): ${msg.text()}`);
  });

  // Listen to network requests
  page.on('request', request => {
    console.log(`Request: ${request.method()} ${request.url()}`);
  });

  page.on('response', response => {
    console.log(`Response: ${response.status()} ${response.url()}`);
  });

  // Listen to page errors
  page.on('pageerror', error => {
    console.error('Page error:', error);
  });

  await page.goto('/app');
  
  // Your test code here
});
```

### Element Debugging

```typescript
test('debug element interactions', async ({ page }) => {
  await page.goto('/form');

  const button = page.locator('[data-testid="submit-button"]');
  
  // Debug element state
  console.log('Button visible:', await button.isVisible());
  console.log('Button enabled:', await button.isEnabled());
  console.log('Button text:', await button.textContent());
  
  // Wait for element to be ready
  await button.waitFor({ state: 'visible' });
  
  // Highlight element for debugging
  await button.highlight();
  
  // Take screenshot of specific element
  await button.screenshot({ path: 'debug-button.png' });
  
  await button.click();
});
```

## Common Debugging Scenarios

### Flaky Tests

```rust
// Add deterministic timing
#[tokio::test]
async fn test_with_retry_logic() {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match run_test_logic().await {
            Ok(result) => {
                println!("Test passed on attempt {}", attempts);
                assert!(result);
                break;
            }
            Err(e) if attempts < max_attempts => {
                println!("Test failed on attempt {} with error: {:?}", attempts, e);
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => {
                panic!("Test failed after {} attempts: {:?}", max_attempts, e);
            }
        }
    }
}
```

### Race Conditions

```typescript
test('handle race conditions', async ({ page }) => {
  await page.goto('/async-page');
  
  // Wait for network to be idle
  await page.waitForLoadState('networkidle');
  
  // Use explicit waits instead of timeouts
  await page.waitForSelector('[data-testid="content"]', { 
    state: 'visible',
    timeout: 10000 
  });
  
  // Wait for specific condition
  await page.waitForFunction(() => {
    const element = document.querySelector('[data-testid="status"]');
    return element && element.textContent === 'Ready';
  });
});
```

### Database Test Issues

```rust
#[tokio::test]
async fn test_with_transaction_rollback() {
    let pool = get_test_db_pool().await;
    
    // Start transaction for isolation
    let mut tx = pool.begin().await.unwrap();
    
    // Run test operations
    let result = create_user(&mut tx, "test@example.com").await;
    
    // Debug database state
    let user_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(&mut tx)
        .await
        .unwrap();
    
    println!("User count after creation: {}", user_count);
    
    assert!(result.is_ok());
    
    // Transaction is automatically rolled back when tx is dropped
}
```

### Mock Configuration Issues

```typescript
describe('Complex mock scenario', () => {
  let mockService: MockService;

  beforeEach(() => {
    mockService = new MockService();
    
    // Debug mock setup
    console.log('Mock service created with methods:', 
      Object.getOwnPropertyNames(Object.getPrototypeOf(mockService))
    );
  });

  it('should handle mock properly', async () => {
    // Verify mock is configured correctly
    expect(typeof mockService.getData).toBe('function');
    
    mockService.getData.mockResolvedValue({ data: 'test' });
    
    // Verify mock setup
    console.log('Mock implementation set:', mockService.getData.getMockImplementation());
    
    const result = await someFunction(mockService);
    
    // Debug mock calls
    console.log('Mock was called with:', mockService.getData.mock.calls);
    
    expect(result).toBeTruthy();
  });
});
```

## Debugging Environment

### VS Code Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Debug Rust Tests",
      "type": "lldb",
      "request": "launch",
      "program": "${workspaceFolder}/target/debug/deps/${input:testBinary}",
      "args": ["${input:testName}", "--nocapture"],
      "cwd": "${workspaceFolder}",
      "preLaunchTask": "rust: cargo test --no-run"
    },
    {
      "name": "Debug TypeScript Tests",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/node_modules/.bin/vitest",
      "args": ["run", "${file}"],
      "cwd": "${workspaceFolder}/web",
      "skipFiles": ["<node_internals>/**"]
    },
    {
      "name": "Debug E2E Tests",
      "type": "node",
      "request": "launch",
      "program": "${workspaceFolder}/node_modules/.bin/playwright",
      "args": ["test", "--debug"],
      "cwd": "${workspaceFolder}/e2e",
      "skipFiles": ["<node_internals>/**"]
    }
  ]
}
```

### Environment Variables for Debugging

```bash
# Create .env.test for test-specific configuration
RUST_LOG=debug
RUST_BACKTRACE=1
TEST_LOG=1
DEBUG=1

# For E2E tests
PLAYWRIGHT_SLOW_MO=1000
PLAYWRIGHT_HEADLESS=false
DEBUG=pw:api
```

### Logging Configuration

```rust
// In test setup
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::test]
async fn test_with_detailed_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Test code with tracing
    tracing::info!("Starting test");
    
    let result = my_function().await;
    
    tracing::debug!("Function result: {:?}", result);
}
```

This comprehensive debugging guide should help identify and resolve test issues quickly and effectively.