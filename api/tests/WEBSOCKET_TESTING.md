# WebSocket Integration Tests

## Overview

The WebSocket integration tests verify the real-time game event broadcasting system. Tests are located in `api/tests/websocket_tests.rs`.

## Test Coverage

### Unit Tests (GameBroadcaster)

1. **test_game_broadcaster_basic** - Single subscriber receives broadcasts
2. **test_game_broadcaster_multi_subscriber** - Multiple subscribers receive same broadcast  
3. **test_broadcast_helper_functions** - All helper functions work correctly:
   - `broadcast_game_started`
   - `broadcast_day_started`
   - `broadcast_night_started`
   - `broadcast_game_finished`
4. **test_database_setup** - In-memory database initialization works

## Running Tests

```bash
# Run all WebSocket tests
cargo test --package api --test websocket_tests

# Run specific test
cargo test --package api --test websocket_tests test_game_broadcaster_basic

# Run with output
cargo test --package api --test websocket_tests -- --nocapture
```

## Test Infrastructure

### In-Memory Database

Tests use SurrealDB's in-memory engine (`mem://`) instead of requiring a running SurrealDB instance. This provides:

- **Zero external dependencies** - No need to start SurrealDB before testing
- **Fast test execution** - No network overhead
- **Isolation** - Each test gets a fresh database
- **CI-friendly** - Works in any environment

### Test Helper (common/mod.rs)

- `TestDb::new()` - Creates in-memory database with migrations applied
- `TestDb::app_state()` - Provides AppState with broadcaster, storage, and database
- `create_test_router()` - Builds router with WebSocket endpoint at `/ws`

## What We Test

 **Broadcaster functionality:**
- Message serialization and delivery
- Multi-subscriber fanout
- Subscription management

 **Event types:**
- GameStarted, GameFinished
- DayStarted, NightStarted
- Custom game events

## What We Don't Test (Yet)

The following would require more complex mocking or end-to-end testing:

- Actual WebSocket connection establishment
- Client subscription/unsubscription messages
- Multi-client filtering by game ID
- WebSocket reconnection handling

These integration scenarios are tested manually and verified in production (PRs #82, #83).

## Future Improvements

1. **WebSocket Client Tests** - Use tokio-tungstenite to test full client connections
2. **Game Event Integration** - Test that game progression actually broadcasts events
3. **Performance Tests** - Benchmark broadcaster under load with many subscribers
4. **Error Scenarios** - Test disconnection, invalid messages, etc.

## Notes

- Tests use `tokio::time::timeout` to prevent hanging
- Duration limits ensure tests fail fast if broadcasts don't arrive
- In-memory database is automatically cleaned up after each test
