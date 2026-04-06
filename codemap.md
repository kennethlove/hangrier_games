# Repository Atlas: Hangrier Games

## Project Responsibility

**Hangrier Games** is a browser-based Hunger Games simulation built with Rust. It provides a complete stack for creating, managing, and watching autonomous tributes compete in procedurally-generated arena battles with AI-powered commentary. The project demonstrates a pure Rust web application architecture: backend API (Axum + SurrealDB), frontend (Dioxus WASM), and stateless game engine with LLM narration.

## System Architecture

**5-Crate Rust Workspace:**

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser (WASM)                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  web/ - Dioxus Frontend                               │  │
│  │  • Query-driven state (dioxus-query)                  │  │
│  │  • 3 themeable colorschemes                           │  │
│  │  • LocalStorage persistence (JWT + theme)            │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP/JSON (reqwest)
                       ▼
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
│          AI Commentary Layer (Ollama)                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  announcers/ - LLM Integration                        │  │
│  │  • Transforms game logs → sports commentary          │  │
│  │  • Dual-commentator narrative (Verity & Rex)         │  │
│  │  • Streaming or batch generation                     │  │
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
1. Frontend sends HTTP request (create game, advance day) with JWT
2. API authenticates via SurrealDB, hydrates `Game` from database
3. Game engine runs pure logic (`run_day_night_cycle()`), emits messages
4. API persists updated state to SurrealDB (tributes, items, areas)
5. Announcers optionally consume message log to generate commentary
6. Frontend refetches game state via query invalidation

## Entry Points

### Running the Application

**Start Everything** (recommended):
```bash
just dev  # Starts SurrealDB + API + web frontend in parallel
          # Access at: http://localhost:8080
```

**Individual Services**:
```bash
just db   # SurrealDB only (ws://localhost:8000)
just api  # API server only (http://localhost:3000) - requires SurrealDB running
just web  # Frontend dev server only (http://localhost:8080) - requires API running
```

### Code Entry Points

| Entry Point | File | Purpose |
|-------------|------|---------|
| **API Server** | [`api/src/main.rs`](api/src/main.rs) | Axum server setup, CORS, JWT middleware, route mounting |
| **Web Frontend** | [`web/src/main.rs`](web/src/main.rs) | Dioxus WASM launcher (mounts root `App` component) |
| **Game Engine** | [`game/src/lib.rs`](game/src/lib.rs) | Module aggregator (exposes `Game`, `Tribute`, `Item`, etc.) |
| **LLM Commentary** | [`announcers/src/lib.rs`](announcers/src/lib.rs) | Public API (`summarize`, `summarize_stream`, `prompt`) |
| **Shared Types** | [`shared/src/lib.rs`](shared/src/lib.rs) | All shared types in single-file crate |

### Development Workflow

| Command | Description |
|---------|-------------|
| `just` | List all available recipes |
| `just dev` | Full dev environment (DB + API + frontend) |
| `just build-css` | Build Tailwind CSS for frontend |
| `just test` | Run game crate tests (60+ unit tests) |
| `just quality` | Run all quality checks (format, check, clippy, test) |
| `just fmt` | Format code (custom edition=2024, fn_single_line=true) |
| `just setup` | Install all dependencies (Dioxus CLI, npm packages, Ollama model) |

**Justfile Location**: [`justfile`](justfile) (199 lines, 20+ recipes)

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
| **`api/`** | Axum REST API server translating HTTP  game engine  SurrealDB | [api/codemap.md](api/codemap.md) |
| **`api/src/`** | REST endpoints for game lifecycle, tributes, users, and authentication | [api/src/codemap.md](api/src/codemap.md) |
| **`web/`** | Dioxus WASM frontend with query-driven state management and 3 themeable colorschemes | [web/codemap.md](web/codemap.md) |
| **`web/src/`** | Frontend components, routing, caching, and browser integration | [web/src/codemap.md](web/src/codemap.md) |
| **`web/src/components/`** | 42+ UI components including modals, icons, game displays, and tribute cards | [web/src/components/codemap.md](web/src/components/codemap.md) |
| **`web/src/components/icons/`** | SVG icon components for UI elements and theme switcher | [web/src/components/icons/codemap.md](web/src/components/icons/codemap.md) |
| **`web/src/components/icons/game_icons_net/`** | 50+ game-icons.net SVG components for items and status effects | [web/src/components/icons/game_icons_net/codemap.md](web/src/components/icons/game_icons_net/codemap.md) |
| **`shared/`** | Shared data types and API contracts between frontend, backend, and game core | [shared/codemap.md](shared/codemap.md) |
| **`shared/src/`** | Single-file crate with all shared types (DisplayGame, EditGame, GameStatus, etc.) | [shared/src/codemap.md](shared/src/codemap.md) |
| **`announcers/`** | Ollama LLM integration transforming game logs into Capitol-style sports commentary | [announcers/codemap.md](announcers/codemap.md) |
| **`announcers/src/`** | LLM client, prompt engineering, and streaming generation support | [announcers/src/codemap.md](announcers/src/codemap.md) |
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

### Frontend

- **Framework**: Dioxus 0.6.3 (React-like WASM UI framework)
- **State Management**: dioxus-query 0.7.0 (async query caching and mutations)
- **HTTP Client**: reqwest 0.12.9 (async HTTP with WASM support)
- **Styling**: Tailwind CSS (utility-first CSS framework)
- **Storage**: gloo-storage 0.3.0 (LocalStorage wrapper for JWT + theme)
- **Build**: Dioxus CLI (`dx`) with WASM target

### Game Logic

- **Language**: Pure Rust (edition 2024)
- **RNG**: rand crate (SmallRng for procedural generation)
- **Testing**: rstest (parameterized testing, 60+ unit tests)
- **Serialization**: serde + serde_json (API exposure)
- **Messaging**: Global event queue with thread-safe Mutex

### AI/LLM

- **LLM**: Ollama with custom `announcers` model (based on qwen2.5:1.5b)
- **Client**: ollama-rs (Rust SDK for Ollama API)
- **Streaming**: async_stream for progressive commentary generation

### Build Tools

- **Task Runner**: just (Makefile alternative, 20+ recipes)
- **Package Manager**: Cargo (workspace with 5 crates)
- **Docker**: Multi-stage builds for API and web frontend
- **Version Manager**: mise (for Node.js, Rust toolchain)

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

# Install all dependencies (Dioxus CLI, Node packages, Ollama model)
just setup

# Create .env file (if not present)
cp .env.example .env  # Contains APP_API_HOST, SURREAL_HOST, credentials
```

### Daily Development

```bash
# Start full dev environment (DB + API + web)
just dev

# Make changes to code, hot reload happens automatically

# Run tests before committing
just test

# Format code
just fmt

# Full quality gate before PR
just quality
```

### Frontend-Only Development

```bash
# In one terminal: Start SurrealDB + API
just db &
just api &

# In another terminal: Start frontend with hot reload
just web

# Rebuild Tailwind CSS after changes to classes
just build-css
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
# Create custom Ollama model for commentary
cd announcers/src
ollama create announcers -f Modelfile.qwen

# Verify model exists
ollama list | grep announcers
```

### Building for Production

```bash
# Build everything (optimized release builds)
just build-prod

# Output locations:
# - API: target/release/api
# - Web: web/dist/ (WASM + JS glue + assets)

# Run with Docker Compose
docker-compose up --build
```

## Key Design Patterns

### 1. Pure Functional Core, Imperative Shell

- **Core** (`game/`): Pure Rust, no I/O, deterministic given RNG seed
- **Shell** (`api/`, `web/`): I/O boundary translating HTTP  pure functions  database

### 2. Event Sourcing (Partial)

- `GLOBAL_MESSAGES` queue captures all game events chronologically
- Messages tagged by source (Game/Area/Tribute) for filtering
- Enables replay, audit trails, and LLM commentary generation

### 3. Translation Layer Pattern

API acts as adapter between three domains:
- **HTTP** (JSON requests/responses)  **Rust types**  **SurrealDB** (graph database)

### 4. Query-Driven State Management

Frontend uses `dioxus-query` (React Query pattern):
- Automatic caching by `QueryKey`
- Mutations invalidate cache → triggers refetch
- No manual loading/error state management

### 5. Context-Based Global State

Root app provides 6 context signals:
- `loading_signal`, `theme_signal`, `game_signal`, `delete_game_signal`, `edit_game_signal`, `edit_tribute_signal`
- Consumed anywhere in component tree via `use_context::<Signal<T>>()`

### 6. Factory Pattern (Item Creation)

Static factory methods for procedural generation:
- `Item::new_random_weapon()`, `Item::new_random_shield()`, `Item::new_random_consumable()`
- Encapsulates RNG logic and ensures valid attribute ranges

### 7. Strategy Pattern (Tribute AI)

`Brain` component uses different decision strategies based on context:
- No enemies → Rest/Hide/Move
- Few enemies → Attack/Move/Hide (health-dependent)
- Many enemies → Move/Hide/Attack (intelligence-dependent)

## Testing Strategy

**Game Crate**: 60+ unit tests using `rstest` for parameterized testing
- Test lifecycle, state transitions, combat mechanics, AI decisions
- Run with: `just test` (WARNING: workspace-wide tests may hang)

**API Crate**: No tests currently (integration tests recommended)

**Frontend**: No tests currently (consider Dioxus testing utils)

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
APP_API_HOST=http://127.0.0.1:3000  # Frontend → API base URL
SURREAL_HOST=ws://localhost:8000     # API → SurrealDB WebSocket
SURREAL_USER=root                    # SurrealDB auth
SURREAL_PASS=root                    # SurrealDB auth
```

**Frontend Build Note**: `APP_*` env vars read at **build time** by `web/build.rs` and generated into `web/src/env.rs`. Changing `.env` requires rebuild.

### Docker Compose

Services defined in `docker-compose.yml`:
- **surrealdb**: Database (port 8000)
- **api**: REST API (port 3000)
- **web**: Frontend static files (port 8080)

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
9. **Icon Optimization**: Consider SVG sprite sheets or lazy loading

## Related Documentation

- **AGENTS.md**: Project-specific instructions for AI agents
- **Cargo.toml**: Workspace configuration and dependency versions
- **Dioxus.toml**: Frontend build configuration
- **docker-compose.yml**: Container orchestration
- **Modelfile.qwen**: Ollama model definition for announcers
- **Tailwind config**: `web/assets/tailwind.config.js`

---

**Last Updated**: 2026-04-05 (Cartography Orchestrator)  
**Purpose**: Definitive entry point for understanding the Hangrier Games repository  
**Maintenance**: Update when adding new crates or changing architecture
