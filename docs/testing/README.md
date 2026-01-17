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
  forge/              # Integration tests

crates/
  tachikoma-test-harness/
    src/
      lib.rs          # Test utilities
      mocks/          # Mock implementations
      fixtures.rs     # Fixture loading
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