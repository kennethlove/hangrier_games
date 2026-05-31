# Repository Atlas: Hangrier Games

## Project Responsibility

**Hangrier Games** is a browser-based Hunger Games simulation built with Rust. It provides a complete stack for creating, managing, and watching autonomous tributes compete in procedurally-generated arena battles with AI-powered commentary. The project demonstrates a pure Rust backend architecture: REST API (Axum + SurrealDB), stateless game engine, and LLM narration.

## System Architecture

**4-Crate Rust Workspace:**

```
┌─────────────────────────────────────────────────────────────┐
│                API Server (Axum)                            │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  api/ - REST Endpoints                                │  │
│  │  • JWT authentication & authorization                 │  │
│  │  • SurrealDB integration (graph database)             │  │
│  │  • Translation layer (HTTP ↔ Game Engine ↔ DB)       │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────────┘
                       │ calls pure functions
                       ▼
┌─────────────────────────────────────────────────────────────┐
│           Core Game Engine (Pure Rust)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  game/ - Simulation Logic                             │  │
│  │  • Stateless, deterministic, no I/O                   │  │
│  │  • Turn-based AI tributes with d20 combat             │  │
│  │  • Event sourcing via global message queue            │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                       │ emits event log
                       ▼
┌─────────────────────────────────────────────────────────────┐
│          Commentary Pipeline                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  announcers/ - BroadcastPackage → Commentator trait   │  │
│  │  • BroadcastPackageBuilder: typed EventLines          │  │
│  │  • Commentator trait (Ollama default, swappable)      │  │
│  │  • TributeHistories rolling digests                   │  │
│  │  • Background task after each game phase              │  │
│  │  • Persisted to SurrealDB + pushed via SSE/WS         │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│             Cross-Cutting Concerns                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  shared/ - Common Types                               │  │
│  │  • API DTOs (DisplayGame, EditGame, etc.)             │  │
│  │  • Shared enums (GameStatus, GameArea)                │  │
│  │  • Serde-enabled for JSON serialization               │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Data Flow:**
1. Client sends HTTP request (create game, advance day) with JWT
2. API authenticates via SurrealDB, hydrates `Game` from database
3. Game engine runs pure logic (`run_day_night_cycle()`), emits messages
4. API persists updated state to SurrealDB (tributes, items, areas)
5. Announcers optionally consume message log to generate commentary

## Entry Points

### Running the Application

**Start Everything** (recommended):
```bash
just dev  # Starts SurrealDB + API in parallel
          # API at: http://localhost:3000
```

**Individual Services**:
```bash
just db   # SurrealDB only (ws://localhost:8000)
just api  # API server only (http://localhost:3000) - requires SurrealDB running
```

### Code Entry Points

| Entry Point | File | Purpose |
|-------------|------|---------|
| **API Server** | [`api/src/main.rs`](api/src/main.rs) | Axum server setup, CORS, JWT middleware, route mounting |
| **Game Engine** | [`game/src/lib.rs`](game/src/lib.rs) | Module aggregator (exposes `Game`, `Tribute`, `Item`, etc.) |
| **Commentary** | [`announcers/src/lib.rs`](announcers/src/lib.rs) | `generate_commentary()` convenience fn, `Commentator` trait |
| **Shared Types** | [`shared/src/lib.rs`](shared/src/lib.rs) | All shared types in single-file crate |

### Development Workflow

| Command | Description |
|---------|-------------|
| `just` | List all available recipes |
| `just dev` | Full dev environment (DB + API) |
| `just test` | Run game crate tests (60+ unit tests) |
| `just quality` | Run all quality checks (format, check, clippy, test) |
| `just fmt` | Format code (custom edition=2024, fn_single_line=true) |
| `just setup` | Install all dependencies (Ollama model) |

**Justfile Location**: [`justfile`](justfile)

## Directory Map (Aggregated)

| Directory | Responsibility Summary | Detailed Map |
|-----------|------------------------|--------------|
| **`game/`** | Pure Rust simulation engine - stateless game logic with no I/O dependencies | [game/codemap.md](game/codemap.md) |
| **`game/src/`** | Core game engine implementing turn-based Hunger Games simulation with event sourcing | [game/src/codemap.md](game/src/codemap.md) |
| **`game/src/tributes/`** | Autonomous AI tributes with d20 combat, status effects, and context-aware decision-making | [game/src/tributes/codemap.md](game/src/tributes/codemap.md) |
| **`game/src/areas/`** | 5-region arena topology (Cornucopia + cardinals) with item inventory and dynamic closures | [game/src/areas/codemap.md](game/src/areas/codemap.md) |
| **`game/src/items/`** | Procedurally-generated weapons, shields, and consumables with factory pattern creation | [game/src/items/codemap.md](game/src/items/codemap.md) |
| **`game/src/threats/`** | Environmental hazards and animal attacks (bears, wolves, etc.) | [game/src/threats/codemap.md](game/src/threats/codemap.md) |
| **`game/src/witty_phrase_generator/`** | Procedural game name generation using backtracking constraint solver | [game/src/witty_phrase_generator/codemap.md](game/src/witty_phrase_generator/codemap.md) |
| **`api/`** | Axum REST API server translating HTTP ↔ game engine ↔ SurrealDB | [api/codemap.md](api/codemap.md) |
| **`api/src/`** | REST endpoints for game lifecycle, tributes, users, and authentication | [api/src/codemap.md](api/src/codemap.md) |
| **`shared/`** | Shared data types and API contracts between backend and game core | [shared/codemap.md](shared/codemap.md) |
| **`shared/src/`** | Single-file crate with all shared types (DisplayGame, EditGame, GameStatus, etc.) | [shared/src/codemap.md](shared/src/codemap.md) |
| **`announcers/`** | Commentary pipeline: BroadcastPackageBuilder → Commentator trait → persisted CommentarySegments | [announcers/codemap.md](announcers/codemap.md) |
| **`announcers/src/`** | Broadcast package builder, rolling history tracker, severity mappings, Commentator trait + Ollama impl | [announcers/src/codemap.md](announcers/src/codemap.md) |
| **`schemas/`** | SurrealDB database schema definitions with graph relations and custom query functions | [schemas/codemap.md](schemas/codemap.md) |
| **`migrations/`** | Database migration tracking and schema evolution management | [migrations/codemap.md](migrations/codemap.md) |
| **`migrations/definitions/`** | Initial schema state and migration definitions for surrealdb-migrations | [migrations/definitions/codemap.md](migrations/definitions/codemap.md) |

## Technology Stack

### Backend

- **Web Framework**: Axum 0.8.4 (async HTTP server with type-safe routing)
- **Database**: SurrealDB 2.3.2 (graph database with native Rust client)
- **Authentication**: JWT with Argon2 password hashing (HS512, 1-hour expiry)
- **Migrations**: surrealdb-migrations 2.2.2 (schema versioning)
- **Runtime**: Tokio 1.45.0 (async runtime)

### Game Logic

- **Language**: Pure Rust (edition 2024)
- **RNG**: rand crate (SmallRng for procedural generation)
- **Testing**: rstest (parameterized testing, 60+ unit tests)
- **Serialization**: serde + serde_json (API exposure)
- **Messaging**: Global event queue with thread-safe Mutex

### AI/LLM

- **Commentary Pipeline**: `BroadcastPackageBuilder` → `Commentator::generate()` → `CommentarySegment`
- **Default Backend**: Ollama (optional, behind `features = ["ollama"]`)
- **Trait**: `Commentator` — swap Ollama for any LLM backend

### Build Tools

- **Task Runner**: just (Makefile alternative)
- **Package Manager**: Cargo (workspace with 4 crates)
- **Docker**: Multi-stage builds for API
- **Version Manager**: mise (for Rust toolchain)

## Database Schema

**SurrealDB Graph Database** with 8 schema files defining:

- **Tables**: `game`, `tribute`, `area`, `item`, `user`, `message`, `summary`, `script_migration`
- **Relations** (graph edges):
  - `owns` (tribute → item) - Item possession
  - `playing_in` (tribute → game, ENFORCED) - Tribute enrollment
  - `areas` (game → area, ENFORCED) - Arena composition
  - `items` (area → item) - Item locations
  - `summaries` (game → summary) - LLM commentary linkage

**Permission Model**:
- **Public Read, Owner Write**: `game`, `tribute`
- **Authenticated Only**: `area`, `item`, `summary`
- **Owner Only**: `user` (record-level isolation)
- **Immutable Audit**: `message`, `script_migration`

**Custom Functions** (in `schemas/*.surql`):
- `fn::get_full_game($id)` - Complete game state with tributes, areas, items
- `fn::get_display_game($id)` - UI-optimized view with winner/readiness
- `fn::get_full_tribute($id)` - Tribute with items and log
- `fn::get_messages_by_*()` - Various message filtering strategies

**Schema Files**: [schemas/codemap.md](schemas/codemap.md)
**Migration System**: [migrations/codemap.md](migrations/codemap.md)

## Development Workflow

### Initial Setup

```bash
# Clone repository
git clone https://github.com/kennethlove/hangrier_games
cd hangrier_games

# Install all dependencies (Ollama model)
just setup

# Create .env file (if not present)
cp .env.example .env  # Contains SURREAL_HOST, credentials
```

### Daily Development

```bash
# Start full dev environment (DB + API)
just dev

# Make changes to code, hot reload happens automatically

# Run tests before committing
just test

# Format code
just fmt

# Full quality gate before PR
just quality
```

### Database Management

```bash
# Start SurrealDB with trace logging
just db

# Access SurrealDB console
surreal sql --conn ws://localhost:8000 --user root --pass root --ns hangry-games --db games

# Migrations run automatically at API startup
# Schema files: schemas/*.surql
# Initial state: migrations/definitions/_initial.json
```

### LLM Setup

```bash
# Create custom Ollama model for commentary (optional)
# Requires `features = ["ollama"]` on the announcers crate
cd announcers/src
ollama create announcers -f Modelfile

# Verify model exists
ollama list | grep announcers
```

### Building for Production

```bash
# Build everything (optimized release builds)
just build-prod

# Run with Docker Compose
docker-compose up --build
```

## Key Design Patterns

### 1. Pure Functional Core, Imperative Shell

- **Core** (`game/`): Pure Rust, no I/O, deterministic given RNG seed
- **Shell** (`api/`): I/O boundary translating HTTP ↔ pure functions ↔ database

### 2. Event Sourcing (Partial)

- `GLOBAL_MESSAGES` queue captures all game events chronologically
- Messages tagged by source (Game/Area/Tribute) for filtering
- Enables replay, audit trails, and LLM commentary generation

### 3. Translation Layer Pattern

API acts as adapter between three domains:
- **HTTP** (JSON requests/responses) ↔ **Rust types** ↔ **SurrealDB** (graph database)

### 4. Factory Pattern (Item Creation)

Static factory methods for procedural generation:
- `Item::new_random_weapon()`, `Item::new_random_shield()`, `Item::new_random_consumable()`
- Encapsulates RNG logic and ensures valid attribute ranges

### 5. Strategy Pattern (Tribute AI)

`Brain` component uses different decision strategies based on context:
- No enemies → Rest/Hide/Move
- Few enemies → Attack/Move/Hide (health-dependent)
- Many enemies → Move/Hide/Attack (intelligence-dependent)

## Testing Strategy

**Game Crate**: 60+ unit tests using `rstest` for parameterized testing
- Test lifecycle, state transitions, combat mechanics, AI decisions
- Run with: `just test` (WARNING: workspace-wide tests may hang)

**API Crate**: No tests currently (integration tests recommended)

**Recommended Additions**:
- Integration tests against real SurrealDB instance
- Mock `AppState` for API unit tests
- Test auth middleware separately
- Verify transaction rollback on errors

## Security Considerations

**Strengths**:
- JWT authentication on protected routes
- SurrealDB handles password hashing (Argon2)
- SQL injection prevented (parameterized queries)
- Row-level permissions in database schema
- Transaction-based atomic updates

**Weaknesses** (Development Mode):
- JWT tokens checked for expiration but no refresh mechanism
- No rate limiting
- CORS allows any origin (`AllowOrigin::any()`)
- No input validation (relies on Serde deserialization)
- Some endpoints use `.expect()` instead of proper error handling

## Configuration

### Environment Variables (`.env`)

```bash
ENV=development                     # production | development
SURREAL_HOST=ws://localhost:8000     # API → SurrealDB WebSocket
SURREAL_USER=root                    # SurrealDB auth
SURREAL_PASS=root                    # SurrealDB auth
```

### Docker Compose

Services defined in `docker-compose.yml`:
- **api**: REST API (port 3000)

## Performance Considerations

**Optimizations**:
- Concurrent futures for bulk operations (game creation, state saves)
- SurrealDB custom functions reduce round-trips
- Diffing for item updates minimizes database writes
- Game engine uses `HashMap` lookups instead of nested loops (O(n²) → O(n))

**Bottlenecks**:
- `save_game` diffing is O(n) per tribute/area
- No pagination on game lists
- Full game state fetched for every simulation step
- No caching layer between API and database

**Scalability**:
- Stateless API (can horizontally scale)
- No connection pooling (single connection per AppState clone)
- SurrealDB connection via persistent WebSocket

## Future Work / TODOs

1. **messages.rs incomplete** - `save_global_messages_to_db` has `todo!()` placeholder
2. **logging.rs unused** - Custom tracing layer code present but not active
3. **Publish/unpublish bugs** - SQL uses `'$identifier'` instead of `$identifier` binding
4. **No refresh token mechanism** - JWT expiration checked but no rotation
5. **Transaction coverage incomplete** - Not all multi-step operations use transactions
6. **No pagination** - Game/tribute lists unbounded
7. **Tribute Outlook**: TODO placeholder in `tribute_detail.rs`
8. **Batch Tribute Fetching**: Eliminate N+1 in `game_tributes.rs`

## Related Documentation

- **AGENTS.md**: Project-specific instructions for AI agents
- **Cargo.toml**: Workspace configuration and dependency versions
- **docker-compose.yml**: Container orchestration
- **Modelfile**: Ollama model definition for announcers (only needed for `features = ["ollama"]`)

---

**Last Updated**: 2026-05-19
**Purpose**: Definitive entry point for understanding the Hangrier Games repository
**Maintenance**: Update when adding new crates or changing architecture
