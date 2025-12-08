# Testing Guide

This document explains how to run and maintain tests for the ERP Backend system.

## Overview

The test suite is organized into:

- **Unit Tests**: Located in `src/` with `#[cfg(test)]` modules
- **Integration Tests**: Located in `tests/` directory
- **Test Utilities**: Common helpers in `tests/common/`

## Prerequisites

Before running tests, ensure you have:

1. **PostgreSQL** running on `localhost:5432`
2. **Redis** running on `localhost:6379`
3. **Rust toolchain** installed
4. **Database migrations** ready

## Quick Start

### Run All Tests

```bash
./scripts/run_tests.sh
```

This script will:

1. Set up the test database
2. Run migrations
3. Execute all unit and integration tests
4. Display results

### Run Specific Tests

```bash
# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run specific test module
cargo test channel_tests

# Run a specific test
cargo test test_create_channel_success

# Run tests with output
cargo test -- --nocapture
```

## Test Organization

### Directory Structure

```
tests/
├── common/
│   ├── mod.rs          # Common test setup utilities
│   ├── fixtures.rs     # Test data fixtures
│   └── helpers.rs      # Helper functions
├── integration/
│   ├── mod.rs          # Integration test module
│   └── channel_tests.rs # Channel service integration tests
└── integration_tests.rs # Test entry point
```

### Test Files

#### Common Utilities (`tests/common/mod.rs`)

Provides:

- `setup_test_app_state()`: Create test AppState
- `setup_test_db()`: Get database connection
- `cleanup_test_db()`: Clean up after tests
- Assertion macros: `assert_ok!`, `assert_err!`

#### Fixtures (`tests/common/fixtures.rs`)

Pre-configured test data for:

- Channels
- Departments
- Positions
- Programs
- Program Slots

#### Helpers (`tests/common/helpers.rs`)

Utility functions:

- JWT token generation
- User claims creation
- Error assertion helpers
- Async condition waiting

## Writing Tests

### Integration Test Example

```rust
#[tokio::test]
async fn test_create_channel_success() {
    // Setup
    let state = setup_test_app_state()
        .await
        .expect("Failed to setup test app state");
    let tx = state.db.begin().await.expect("Failed to begin transaction");

    // Prepare test data
    let command = fixtures::create_test_channel_command();

    // Execute
    let result = state.channel_service.create_channel(&tx, &command).await;

    // Assert
    assert!(result.is_ok(), "Failed to create channel: {:?}", result.err());
    let channel = result.unwrap();
    assert_eq!(channel.channel_name, "VTV1");

    // Cleanup
    tx.rollback().await.expect("Failed to rollback transaction");
}
```

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_validation() {
        let command = CreateChannelCommand {
            channel_name: "VTV1".to_string(),
            channel_code: "VTV1".to_string(),
            description: None,
            is_active: Some(true),
        };

        // Test validation logic
        assert!(command.channel_name.len() > 0);
    }
}
```

## Test Coverage

### Channel Service Tests

The channel integration tests cover:

✅ **Create Operations**

- `test_create_channel_success` - Happy path
- `test_create_channel_with_duplicate_code_should_fail` - Validation

✅ **Read Operations**

- `test_get_channel_by_id` - Fetch by ID
- `test_get_channel_by_nonexistent_id_should_fail` - Error handling
- `test_list_channels` - List all
- `test_list_channels_with_search` - Search functionality
- `test_list_channels_with_pagination` - Pagination

✅ **Update Operations**

- `test_update_channel` - Happy path
- `test_update_nonexistent_channel_should_fail` - Error handling

✅ **Delete Operations**

- `test_delete_channel` - Soft delete
- `test_delete_nonexistent_channel_should_fail` - Error handling

✅ **Cache Operations**

- `test_channel_cache_invalidation` - Redis cache invalidation

## Test Database

### Setup

The test database is automatically created and migrated by `setup_test_db.sh`:

```bash
./scripts/setup_test_db.sh
```

### Manual Setup

```bash
# Create database
psql -U postgres -c "CREATE DATABASE erp_test;"

# Run migrations
cd user-migration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/erp_test cargo run -- up
```

### Cleanup

Each test uses transactions and rolls back changes:

```rust
let tx = state.db.begin().await?;
// ... test operations ...
tx.rollback().await?; // Automatic cleanup
```

## Environment Configuration

### Test Environment (`.env.test`)

```bash
TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/erp_test
TEST_REDIS_URL=redis://localhost:6379/1
KAFKA_BROKERS=localhost:9092
JWT_SECRET=test_secret_key_for_testing_only_do_not_use_in_production
RUST_LOG=debug
```

### Test Settings (`settings/test.toml`)

Configuration file for test AppState initialization.

## Continuous Integration

### GitHub Actions

Tests run automatically on:

- Push to `main`, `staging`, `develop`
- Pull requests

See `.github/workflows/test.yml` for CI configuration.

### Local CI Simulation

```bash
# Run tests as CI would
./scripts/run_tests.sh --verbose
```

## Coverage Reports

### Generate Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
./scripts/run_tests.sh --coverage
```

Coverage report will be in `coverage/index.html`.

## Best Practices

### ✅ DO

- Use transactions and rollback for cleanup
- Use fixtures for consistent test data
- Test both happy and error paths
- Use descriptive test names
- Keep tests independent
- Mock external dependencies
- Test edge cases

### ❌ DON'T

- Commit test data to database
- Depend on test execution order
- Use production credentials
- Skip cleanup/rollback
- Test multiple things in one test
- Use hardcoded IDs from production

## Troubleshooting

### Tests Failing to Connect to Database

```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check database exists
psql -U postgres -l | grep erp_test
```

### Tests Failing to Connect to Redis

```bash
# Check Redis is running
redis-cli -h localhost -p 6379 ping
```

### Migration Errors

```bash
# Reset test database
./scripts/setup_test_db.sh
```

### Slow Tests

```bash
# Run tests in parallel (careful with database tests)
cargo test -- --test-threads=4

# Run only specific tests
cargo test channel_tests
```

## Adding New Tests

### 1. Create Test Module

```rust
// tests/integration/your_module_tests.rs
#[cfg(test)]
mod your_module_tests {
    use crate::common::*;

    #[tokio::test]
    async fn test_your_feature() {
        // Test implementation
    }
}
```

### 2. Add Fixtures

```rust
// tests/common/fixtures.rs
pub fn create_test_your_entity_command() -> CreateYourEntityCommand {
    CreateYourEntityCommand {
        // fields
    }
}
```

### 3. Register Module

```rust
// tests/integration/mod.rs
pub mod your_module_tests;
```

### 4. Run Tests

```bash
cargo test your_module_tests
```

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [SeaORM Testing](https://www.sea-ql.org/SeaORM/docs/write-test/testing/)
- [Project Architecture Guide](../README.md)

## Support

For questions or issues with tests:

1. Check this guide
2. Review existing test examples
3. Ask the team in #dev-backend

---

**Happy Testing! 🧪**
