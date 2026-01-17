# 490 - Test Documentation

**Phase:** 22 - Testing Infrastructure
**Spec ID:** 490
**Status:** Planned
**Dependencies:** 471-test-harness
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Create comprehensive documentation for the testing infrastructure, including guides for writing tests, running test suites, debugging failures, and contributing to test coverage.

---

## Acceptance Criteria

- [x] Testing philosophy and principles documented
- [x] Quick start guide for running tests
- [x] Detailed guides for each test type
- [x] Debugging and troubleshooting guide
- [x] Contributing guide for test coverage
- [x] CI/CD testing documentation

---

## Implementation Details

### 1. Main Testing Documentation

Create `docs/testing/README.md`:

```markdown
# Tachikoma Testing Guide

This guide covers all aspects of testing in the Tachikoma project.

## Quick Start

```bash
# Run all tests
make test

# Run specific test suites
cargo test --workspace          # Rust tests
cd web && npm test              # TypeScript tests
npm run test:e2e                # End-to-end tests

# Run with coverage
cargo llvm-cov --workspace      # Rust coverage
cd web && npm run test:coverage # TypeScript coverage
```

## Testing Philosophy

1. **Test Pyramid**: Prioritize unit tests, supplement with integration and E2E
2. **Fast Feedback**: Tests should run quickly for rapid iteration
3. **Deterministic**: Tests should produce consistent results
4. **Independent**: Tests should not depend on execution order
5. **Documented**: Test names should describe behavior

## Test Types

| Type | Location | Framework | Purpose |
|------|----------|-----------|---------|
| Unit | `src/` | cargo test / vitest | Test isolated functions |
| Integration | `tests/` | cargo test / vitest | Test component interactions |
| Property | `tests/` | proptest / fast-check | Test invariants |
| Snapshot | `snapshots/` | insta / vitest | Test output stability |
| E2E | `e2e/` | Playwright | Test full application |
| Visual | `e2e/visual/` | Playwright | Test UI appearance |
| Benchmark | `benches/` | Criterion / vitest bench | Test performance |
| Load | `load-tests/` | k6 | Test scalability |

## Directory Structure

```
tests/
  fixtures/           # Shared test data
    configs/
    api_responses/
    specs/
  integration/        # Integration tests

crates/
  tachikoma-test-harness/
    src/
      lib.rs          # Test utilities
      mocks/          # Mock implementations
      fixtures/       # Fixture loading
      generators/     # Data generators
    benches/          # Benchmarks

web/
  src/test/
    setup.ts          # Test setup
    mocks/            # Mock services
    fixtures/         # Test fixtures

e2e/
  tests/              # E2E test specs
  pages/              # Page objects
  fixtures/           # E2E fixtures
  visual/             # Visual regression
```

## Configuration Files

- `.config/nextest.toml` - Rust test runner config
- `web/vitest.config.ts` - TypeScript test config
- `e2e/playwright.config.ts` - E2E test config
- `coverage.config.json` - Coverage thresholds

## Further Reading

- [Writing Tests](./writing-tests.md)
- [Test Patterns](./patterns.md)
- [Mocking Guide](./mocking.md)
- [Debugging Tests](./debugging.md)
- [CI Integration](./ci.md)
```

### 2. Writing Tests Guide

Create `docs/testing/writing-tests.md`:

```markdown
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

## What to Test

### DO Test
- Public API behavior
- Edge cases (empty, null, boundaries)
- Error handling paths
- State transitions
- Integration points

### DON'T Test
- Private implementation details
- Framework/library code
- Trivial getters/setters
- External service internals

## Test Data

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
```

### 3. Debugging Tests Guide

Create `docs/testing/debugging.md`:

```markdown
# Debugging Tests

## Rust Tests

### Run Single Test
```bash
cargo test test_name -- --nocapture
```

### Show Output
```bash
cargo test -- --nocapture --test-threads=1
```

### Debug with LLDB
```bash
cargo test test_name -- --nocapture
# Get binary path from output, then:
lldb target/debug/deps/crate_name-hash
```

### Enable Logging
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

## TypeScript Tests

### Run Single Test
```bash
npm test -- --filter "test name"
```

### Debug Mode
```bash
npm test -- --inspect-brk
# Then attach debugger to port 9229
```

### Verbose Output
```bash
npm test -- --reporter verbose
```

## E2E Tests

### Debug Mode
```bash
npx playwright test --debug
```

### UI Mode (Interactive)
```bash
npx playwright test --ui
```

### Generate Trace
```bash
npx playwright test --trace on
# View with: npx playwright show-trace trace.zip
```

### Screenshot on Failure
Screenshots are automatically captured. Find them in:
```
e2e/test-results/
```

## Common Issues

### Test Timeout
Increase timeout or check for:
- Unresolved promises
- Missing async/await
- Infinite loops

### Flaky Tests
- Check for race conditions
- Ensure proper cleanup
- Use explicit waits, not sleeps

### Mock Issues
- Verify mock setup before test
- Clear mocks between tests
- Check mock call order
```

### 4. CI Testing Documentation

Create `docs/testing/ci.md`:

```markdown
# CI Testing

## Workflow Overview

```
Pull Request:
  lint → rust-tests → typescript-tests → e2e-tests → quality-gate
        ↓
     coverage → codecov

Main Branch (Nightly):
  full-test-suite → benchmarks → load-tests → flaky-detection
```

## Running Locally (Same as CI)

```bash
# Full CI suite
make ci

# Individual jobs
make lint
make test-rust
make test-typescript
make test-e2e
make test-coverage
```

## Quality Gates

PRs must pass:
- All lint checks (format, clippy, eslint)
- All unit and integration tests
- Coverage thresholds (70% minimum)
- E2E tests on all platforms

## Coverage Requirements

| Component | Minimum |
|-----------|---------|
| Rust libs | 70% |
| TypeScript | 70% |
| Overall | 65% |

## Artifacts

CI produces:
- Test reports (JUnit XML)
- Coverage reports (LCOV, HTML)
- E2E screenshots/videos
- Benchmark results

## Caching

CI caches:
- Cargo dependencies
- Node modules
- Playwright browsers
- Test fixtures

## Troubleshooting CI

### Test Passes Locally, Fails in CI
- Check for timezone dependencies
- Verify environment variables
- Look for race conditions
- Check file system permissions

### Slow CI
- Review test parallelization
- Check cache effectiveness
- Consider test splitting
```

---

## Testing Requirements

1. Documentation is accurate and up-to-date
2. Quick start works for new developers
3. All test types have clear guides
4. Debugging instructions are practical
5. CI documentation matches actual workflows

---

## Related Specs

- Depends on: [471-test-harness.md](471-test-harness.md)
- Related: All Phase 22 testing specs
- Next Phase: [491-build-overview.md](../phase-23-build/491-build-overview.md)
