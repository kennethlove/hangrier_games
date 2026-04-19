# API Integration Tests

This directory contains integration tests for the Hangrier Games API.

## Test Structure

- `common/mod.rs` - Shared test utilities, database setup, and helpers
- `auth_tests.rs` - Authentication flow tests (signup, signin, refresh, logout)
- `games_tests.rs` - Game CRUD operations and management
- `tributes_tests.rs` - Tribute CRUD operations within games
- `simulation_tests.rs` - Game simulation and advancement

## Running Tests

### Prerequisites

1. Start SurrealDB for testing:
```bash
surreal start --log trace --user root --pass root ws://localhost:8000
```

2. Ensure environment variables are set (`.env` file in project root):
```bash
SURREAL_HOST=ws://localhost:8000
SURREAL_USER=root
SURREAL_PASS=root
```

### Run All Tests

```bash
cargo test --package api
```

### Run Specific Test Files

```bash
cargo test --package api --test auth_tests
cargo test --package api --test games_tests
cargo test --package api --test tributes_tests
cargo test --package api --test simulation_tests
```

### Run Individual Tests

```bash
cargo test --package api test_user_registration
cargo test --package api test_create_game
```

## Test Design

### Database Isolation

Each test creates a unique test database using a UUID-based name to ensure isolation:
- Namespace: `hangry-games-test`
- Database: `test_<uuid>`

Tests clean up after themselves by removing the test database.

### Authentication

Tests use the `TestUser` helper to:
1. Create test users with unique credentials
2. Obtain JWT access and refresh tokens
3. Make authenticated requests

### Test Server

Uses `axum-test` crate for easy HTTP testing:
- Creates a test server with full routing
- Supports all HTTP methods
- Provides assertion helpers

## Test Coverage

### Authentication (7 tests)
-  User registration
-  User authentication
-  Wrong password handling
-  Token refresh
-  Logout
-  Duplicate username validation
-  Session endpoint with authentication

### Games (12 tests)
-  Create game
-  List games with pagination
-  Get specific game
-  Update game
-  Delete game
-  Game display endpoint
-  Game areas endpoint
-  Publish game
-  Unpublish game
-  Unauthorized access prevention

### Tributes (10 tests)
-  Create tribute
-  Get tribute
-  Update tribute
-  Delete tribute
-  Multiple tributes in game
-  Tribute log endpoint
-  Tribute items relationship
-  Validation (missing fields)
-  District validation

### Simulation (8 tests)
-  Advance game
-  Status transitions
-  Game day logs
-  Tribute-specific logs
-  Multiple game cycles
-  Game finishes with winner
-  Advance finished game handling
-  State persistence between cycles

**Total: 37 integration tests**

## Known Limitations

1. **Database cleanup**: Tests attempt to cleanup but may leave databases in edge cases
2. **Timing**: Some simulation tests may timeout if battles take too long
3. **Parallel execution**: Tests create unique databases but share the same SurrealDB instance

## Extending Tests

To add new tests:

1. Create new test file: `tests/your_feature_tests.rs`
2. Import common utilities: `mod common;`
3. Use helpers: `create_authenticated_user()`, `create_test_game()`
4. Follow existing patterns for consistency

Example:
```rust
mod common;

use axum_test::TestServer;
use common::{TestDb, create_test_router};

#[tokio::test]
async fn test_my_feature() {
    let test_db = TestDb::new().await;
    let app_state = test_db.app_state();
    let router = create_test_router(app_state);
    let server = TestServer::new(router);
    
    // Your test code here
    
    test_db.cleanup().await;
}
```

## Troubleshooting

### Tests hang
- Check if SurrealDB is running
- Verify connection string in `.env`
- Check for port conflicts on 8000

### Authentication failures
- Ensure JWT secret matches `schemas/users.surql`
- Check token expiration (1 hour default)
- Verify SurrealDB migrations applied

### Database errors
- Run migrations manually: `cargo run --package api` (starts server and runs migrations)
- Check SurrealDB logs for errors
- Ensure namespace/database permissions

## Future Improvements

- [ ] Add testcontainers for automated SurrealDB lifecycle
- [ ] Implement database pooling for parallel tests
- [ ] Add performance benchmarks
- [ ] Mock external dependencies (Ollama for announcers)
- [ ] Add contract tests for API versioning
- [ ] Implement snapshot testing for complex responses
