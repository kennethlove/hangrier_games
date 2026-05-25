# api/src/

## Responsibility

REST API + HTML server built with **Axum** that provides both HTTP API endpoints and server-rendered HTMX pages for the Hangrier Games simulation. The API acts as the bridge between the browser (HTMX + Maud templates) and the game engine (pure Rust simulation logic), persisting game state in **SurrealDB**.

**Core Job**: Serve HTMX-powered HTML pages + REST API, translate HTTP requests → call pure game engine functions → persist results to SurrealDB → return responses.

**Key Services**:
- Game lifecycle management (create, delete, step simulation)
- Tribute CRUD operations
- User authentication & JWT authorization
- Game log/message persistence
- State synchronization between in-memory game engine and relational database

---

## Design

### File Structure

```
api/
├── src/
│   ├── main.rs                # Application entry point, Axum server, routes, middleware
│   ├── lib.rs                 # Shared types (AppState, AppError)
│   ├── auth.rs                # JWT middleware, refresh token logic
│   ├── cookies.rs             # Cookie-based session management
│   ├── email.rs               # Email verification
│   ├── games.rs               # Game lifecycle endpoints (CRUD, step simulation)
│   ├── tributes.rs            # Tribute management endpoints
│   ├── users.rs               # Authentication and user management
│   ├── messages.rs            # Game message/log persistence (incomplete)
│   ├── logging.rs             # Custom tracing layer (currently unused)
│   ├── sse.rs                 # Server-Sent Events for live game log streaming
│   ├── websocket.rs           # WebSocket broadcaster for game events
│   ├── storage.rs             # File upload storage backend
│   ├── cleanup.rs             # Scheduled cleanup of expired refresh tokens
│   ├── routes/
│   │   ├── mod.rs             # Route module aggregator
│   │   ├── auth.rs            # Auth HTML page handlers (login, register, logout)
│   │   ├── games.rs           # Game HTML page handlers (list, detail)
│   │   └── dev.rs             # Dev-only routes
│   └── templates/
│       ├── mod.rs             # Base layout, AuthState enum, icon helpers
│       ├── pages.rs           # Page components: home, game list, 404, 500
│       ├── game_detail.rs     # Game detail page, tribute/area/log cards
│       ├── auth.rs            # Auth page components (login, register forms)
│       └── timeline/
│           ├── mod.rs         # Timeline layout and card rendering
│           └── cards.rs       # Event card components (combat, death, etc.)
├── assets/
│   ├── src/main.css           # Tailwind CSS source with design tokens
│   ├── dist/main.css          # Compiled CSS (Tailwind v4 + LightningCSS)
│   ├── icons/
│   │   ├── sprite-ui.svg      # UI icon sprite (nav, actions)
│   │   └── sprite-narrative.svg  # Narrative icon sprite (events, items)
│   └── package.json           # Node.js deps for CSS build (Tailwind CLI)
└── tests/                     # Integration tests (auth, games, tributes, simulation)
```

### Architecture Patterns

**1. Translation Layer Pattern**

API translates between three domains:
- **HTTP** (JSON requests/responses)  **Rust types**  **SurrealDB** (graph database)

Example flow:
```rust
// HTTP → Rust
PUT /api/games/{id}/next
  ↓
// Rust → Database
let game = get_full_game(id, db).await?;
  ↓
// Pure game logic
game.run_day_night_cycle(true);
  ↓
// Database ← Rust
save_game(&game, db).await?;
  ↓
// HTTP ← Rust
Ok(Json(game))
```

**2. State Synchronization Pattern**

Challenge: Game engine operates on in-memory `Game` struct, DB stores normalized relational data.

Solution: Two-way transformation with diffing:
- **Load**: `get_full_game` assembles game + tributes + items + areas from relations
- **Save**: `save_game` decomposes into tables, uses HashMap diffing to minimize writes

**3. Lazy Static Routers**

Each module exports a `LazyLock<Router>` for modularity:
```rust
pub static GAMES_ROUTER: LazyLock<Router<AppState>> = LazyLock::new(|| {
    Router::new()
        .route("/", get(game_list).post(create_game))
        .route("/{id}", get(game_detail).delete(game_delete))
        // ...
});
```

Mounted in `main.rs`:
```rust
Router::new()
    .nest("/games", GAMES_ROUTER)
    .nest("/users", USERS_ROUTER)
```

**4. SurrealDB Integration Patterns**

**Schema-First Design**: Database schema defined in `schemas/*.surql`, applied via migrations at startup.

**Custom Functions**: Complex queries encapsulated in SurrealQL functions:
```rust
// Defined in schemas/game.surql
DEFINE FUNCTION fn::get_full_game($id: string) { ... }

// Called from Rust
db.query("SELECT * FROM fn::get_full_game($identifier)")
    .bind(("identifier", id))
    .await
```

**Graph Relations**: Uses SurrealDB's graph edges for relationships:
```rust
// Create relation
db.insert("playing_in").relation(
    TributeGameEdge { in: tribute_id, out: game_id }
).await

// Query via graph traversal
"SELECT * FROM <-playing_in<-tribute"  // Inbound tributes
"SELECT * FROM ->areas->area"          // Outbound areas
```

**5. Error Handling Strategy**

Custom `AppError` enum (via `thiserror`) converts to HTTP responses:
```rust
pub enum AppError {
    NotFound(String) → 404,
    InternalServerError(String) → 500,
    BadRequest(String) → 400,
    Unauthorized(String) → 401,
    // ...
}

impl IntoResponse for AppError { ... }
```

Endpoints return `Result<Json<T>, AppError>`, Axum auto-converts errors to HTTP responses.

**6. Concurrency Patterns**

Parallel creation/updates using `futures::join_all`:
```rust
// Create 24 tributes concurrently
let futures = (0..24).map(|i| create_tribute(..., i));
let results = futures::future::join_all(futures).await;

// Save areas in parallel
let results = futures::join_all(
    game.areas.iter().map(|a| async { save_area(a, db) })
).await;
```

**7. Transaction Management**

Critical operations wrapped in SurrealDB transactions:
```rust
db.query("BEGIN TRANSACTION").await;
// ... multiple operations ...
if error {
    db.query("ROLLBACK").await;
    return Err(...);
}
db.query("COMMIT").await;
```

Used in `save_game` to ensure atomic state updates.

**8. Server-Side Rendering Pattern (HTMX + Maud)**

- All HTML is rendered server-side using `maud` (compile-time HTML templates)
- HTMX 2.0 handles client-side interactivity (form submissions, dynamic loading, tab switches)
- SSE for real-time log streaming (game log follows simulation step)
- Static assets (CSS, icons) served via `ServeDir` in Axum's tower service layer
- CSRF tokens protect all mutating HTML form submissions

**9. Design System v1 — CSS Custom Properties + Tailwind v4**

The UI follows a single canonical light theme defined via CSS custom properties in `:root {}`. All component styling uses these tokens — no hardcoded colors in templates.

**Token Categories** (defined in `api/assets/src/main.css`):

| Category | Tokens | Description |
|----------|--------|-------------|
| **Backgrounds** | `--bg`, `--surface` | Page background (light) and card surface (white) |
| **Foregrounds** | `--fg`, `--muted` | Primary text and secondary/meta text |
| **Accent** | `--accent`, `--accent-soft` | Interactive/emphasis color (violet) with 10% mix |
| **Status** | `--running` (green), `--waiting` (amber), `--finished` (gray) | Game status indicators |
| **Borders** | `--border` | Subtle gray border on cards/sections |
| **Typography** | `--font-display` (Newsreader serif), `--font-body` (Inter), `--font-mono` (JetBrains Mono) | Font stack assignments |
| **Type Scale** | `--fs-h1` (clamp 48–84px) → `--fs-xs` (11px) | Fluid/stepped type scale |
| **Spacing** | `--gap-xs` (6px) → `--gap-xl` (48px) | Consistent spacing scale |
| **Layout** | `--container` (1200px), `--gutter` (28px) | Max-width and page padding |
| **Radii** | `--radius-sm` (4px), `--radius` (8px) | Border radius for inputs/cards |

All color values use **OKLCH** color space for perceptual uniformity.

**Component Classes** (naming convention: `.{component}`, `.{component}-{variant}`):

| Group | Classes | Description |
|-------|---------|-------------|
| **Navigation** | `.topnav`, `.logo`, `.auth-links` | Sticky top nav with backdrop blur |
| **Page Structure** | `.page-header`, `.deck`, `.container`, `.detail-header`, `.detail-tabs` | Page layout sections |
| **Game Cards** | `.game-card`, `.game-card.featured`, `.game-card.running`, `.game-info`, `.game-meta`, `.status-pill` | List cards with status color-coded left border |
| **Buttons** | `.btn`, `.btn-primary`, `.btn-ghost`, `.btn-sm`, `.quickstart-btn`, `.reveal-btn` | Primary (accent fill), ghost (border), small variants |
| **Stats** | `.stats-col`, `.summary-card`, `.s-val`, `.s-label` | Dashboard stat tiles |
| **Tribute/Area** | `.tribute-card`, `.area-card`, `.card-grid`, `.card-stats`, `.card-bands`, `.item-tag` | Entity detail cards with stat grids |
| **Filters** | `.filter-pills`, `.filter-pill`, `.filter-pill.active` | Status filter pill buttons |
| **Forms** | `.form-group`, `.tab-bar`, `.tab-btn`, `.tab-panel`, `.auth-card`, `.error-banner` | Auth forms with client-side tab switching |
| **Log** | `.log-container`, `.log-entry`, `.kind-death`, `.kind-combat`, `.kind-alliance`, `.kind-movement`, `.kind-item`, `.kind-state`, `.kind-affliction`, `.kind-phobia`, `.kind-trauma` | Game log entries with payload-type color coding |
| **Severity** | `.severity-mild`, `.severity-moderate`, `.severity-severe` | Affliction/trauma severity badges |
| **Survival** | `.band-good`, `.band-warn`, `.band-danger`, `.band-none` | Tribute survival status indicators |
| **Status** | `.status-not-started`, `.status-in-progress`, `.status-finished` | Game status badges |
| **Animations** | `.live-pulse`, `.progress-bar .fill`, `.htmx-indicator`, `.spinner` | Live pulsing indicator, progress bars, HTMX loading states |
| **Utilities** | `.icon`, `.capitalize`, `.text-center`, `.num`, `.container` | Icon sizing, typography helpers |

**Maud Template Components** (`api/src/templates/`):

| Module | Key Exports | Purpose |
|--------|-------------|---------|
| `mod.rs` | `base_layout()`, `AuthState`, `icon()`, `narrative_icon()` | HTML shell, auth state machine, SVG icon renderers |
| `pages.rs` | `home_page()`, `games_list_page()`, `summary_card()`, `filter_pill()`, `featured_running_card()`, `running_card()`, `waiting_card()`, `finished_card()`, `status_color()`, `not_found_page()`, `server_error_page()` | Page-level layouts and reusable card components |
| `game_detail.rs` | detail page, `kind_color()`, `hunger_color()`, `thirst_color()`, `stamina_color()` | Game detail with tribute/area/log card rendering |
| `auth.rs` | login/register pages | Auth forms with tab switching |
| `timeline/mod.rs` | chronology layout | Timeline card grid |
| `timeline/cards.rs` | event cards | Combat/death/affliction item cards with severity styling |

**SVG Icon Sprites**:
- `sprite-ui.svg` — navigation and action icons (referenced via `#icon_ui_{name}`)
- `sprite-narrative.svg` — game event icons (referenced via `#icon_narrative_{name}`)
- Loaded once in `<body>` via `<svg style="display:none"><use href="/icons/sprite-ui.svg"/></svg>`

**Build Pipeline**:
```bash
# CSS builds with Tailwind v4 + LightningCSS
cd api/assets
npx @tailwindcss/cli -i src/main.css -o dist/main.css
```

## Flow

### Application Bootstrap (main.rs)

```
1. initialize_logging()
   ├─ Configure tracing (stdout, JSON optional)
   └─ Set log level based on PRODUCTION env var

2. Connect to SurrealDB
   ├─ Read SURREAL_HOST from env (ws://localhost:8000)
   ├─ Authenticate as Root (SURREAL_USER/SURREAL_PASS)
   └─ Use namespace: "hangry-games", database: "games"

3. Apply Migrations
   └─ MigrationRunner::new(&db).up().await
       (reads schemas/*.surql, applies changes)

4. Build Router Tree
   ├─ Configure CORS (allow any origin, all methods)
   ├─ Mount /api/games (protected by JWT middleware)
   ├─ Mount /api/users (public)
   └─ Add middleware:
       ├─ Error handling (timeout → 408, else → 500)
       ├─ Timeout (10 seconds)
       ├─ Tracing (HTTP request/response logging)
       └─ CORS

5. Listen on 0.0.0.0:3000
```

### Authentication Flow

```
1. User Registration
   POST /api/users
   ├─ db.signup(Record { username, password })
   ├─ SurrealDB creates user, hashes password
   └─ Return JWT token

2. User Authentication
   POST /api/users/authenticate
   ├─ db.signin(Record { username, password })
   └─ Return JWT token

3. Protected Request
   GET/PUT/POST /api/games/*
   ├─ surreal_jwt middleware intercepts
   ├─ Extract Authorization: Bearer <token>
   ├─ Decode JWT payload, check expiration
   ├─ db.authenticate(Jwt::from(token))
   └─ If valid: continue, else: 401 Unauthorized
```

### Game Creation Flow

```
POST /api/games
  ↓
create_game(payload)
  ├─ Insert game record
  │   └─ db.create(("game", id)).content(payload)
  ├─ Create 24 tributes (parallel)
  │   └─ create_tribute() × 24
  │       ├─ db.create("tribute", ...)
  │       ├─ db.insert("playing_in", edge)  # Link to game
  │       └─ db.insert("owns", item_edge)   # Starting item
  └─ Create 12 areas (parallel)
      └─ create_area() × 12
          ├─ create_game_area_edge()
          │   ├─ db.create("area", ...)
          │   └─ db.insert("areas", edge)   # Link to game
          └─ add_item_to_area() × 3
              ├─ db.create("item", ...)
              └─ db.insert("items", edge)   # Link to area
  ↓
Return created game (JSON)
```

### Simulation Step Flow

```
PUT /api/games/{id}/next
  ↓
get_game_status(db, id)
  ↓
switch status:
  ┌─ NotStarted:
  │   ├─ Update status to InProgress
  │   └─ Return game (no simulation)
  │
  ├─ InProgress:
  │   ├─ get_full_game(id, db)
  │   │   └─ Query fn::get_full_game($id)
  │   │       (assembles game + tributes + items + areas)
  │   ├─ game.run_day_night_cycle(true)   # Day (pure logic)
  │   ├─ game.run_day_night_cycle(false)  # Night (pure logic)
  │   ├─ save_game(game, db)
  │   │   ├─ BEGIN TRANSACTION
  │   │   ├─ Save game logs (from global message queue)
  │   │   ├─ Parallel: update areas
  │   │   │   └─ save_area_items (diff, delete/update/insert)
  │   │   ├─ Parallel: update tributes
  │   │   │   └─ save_tribute_items (diff, delete/update/insert)
  │   │   ├─ Update game record
  │   │   └─ COMMIT (or ROLLBACK on error)
  │   ├─ Check if 24 tributes dead
  │   │   └─ If yes: update status to Finished
  │   └─ Return updated game
  │
  └─ Finished:
      └─ Return None
```

### Item Synchronization Flow (save_area_items / save_tribute_items)

```
1. Fetch existing items from DB
   └─ SELECT * FROM items WHERE in = $owner

2. Build lookups
   ├─ existing_map: HashMap<identifier, Item>
   └─ new_map: HashMap<identifier, Item>

3. Diff
   ├─ items_to_delete = in DB but not in new OR quantity = 0
   └─ items_to_update = in new AND (not in DB OR different)

4. Apply changes
   ├─ DELETE items in items_to_delete
   ├─ UPDATE/INSERT items in items_to_update
   └─ DELETE + INSERT relations (owns/items edges)
```

---

## Integration

### With Game Engine (`game` crate)

**Direction**: API → Game (calls pure functions)

**Integration Points**:
- `game::games::Game` - Main game state struct
- `game::tributes::Tribute` - Tribute entity
- `game::items::Item` - Item entity
- `game::areas::{Area, AreaDetails}` - Area types
- `game::messages::{get_all_messages, GameMessage}` - Log retrieval

**Pattern**: API hydrates `Game` from DB → calls game engine methods → persists updated state.

Example:
```rust
// Load from DB
let mut game = get_full_game(id, db).await?;

// Pure game logic (no I/O)
game.run_day_night_cycle(is_day);

// Save back to DB
save_game(&game, db).await?;
```

### With Shared Types (`shared` crate)

**Direction**: API  Shared  Frontend

**Shared Types**:
- `DisplayGame` - Optimized game view for frontend
- `ListDisplayGame` - Summary for game lists
- `EditGame` - Update payload
- `EditTribute` - Tribute update payload
- `GameStatus` - Enum (NotStarted, InProgress, Finished)
- `GameArea` - Area representation

**Pattern**: API serializes these types to JSON, frontend deserializes.

### With SurrealDB

**Connection**: WebSocket (`ws://localhost:8000`)

**Schema Management**:
- Migrations in `migrations/definitions/`
- Schema files in `schemas/*.surql`
- Applied via `surrealdb-migrations` at startup

**Authentication Layers**:
1. **Root Auth** (API startup): Full database access for migrations/queries
2. **User Auth** (per-request): JWT-based, enforces row-level permissions

**Key Tables**:
- `game` - Game records
- `tribute` - Tribute records
- `area` - Area records
- `item` - Item records
- `user` - User accounts
- `message` - Game logs

**Relation Tables** (graph edges):
- `playing_in` - tribute → game
- `owns` - tribute → item
- `areas` - game → area
- `items` - area → item

**Custom Functions** (defined in schemas):
- `fn::get_full_game($id)` - Full game state
- `fn::get_display_game($id)` - Optimized display view
- `fn::get_list_games()` - All games for user
- `fn::get_full_tribute($id)` - Tribute with items
- `fn::get_tributes_items_by_game($id)` - Cleanup helper
- `fn::get_areas_items_by_game($id)` - Cleanup helper

### With Browser (HTMX + Server-Rendered Pages)

**Protocol**: HTML over HTTP (server-rendered) + JSON API

**Base URL**: `http://localhost:3000` (same server — no CORS needed)

**Authentication**: HttpOnly cookies (`hg_session` JWT + `hg_refresh`), CSRF tokens for form mutations

**Rendering Pipeline**:
```
Browser request  →  Axum handler  →  hydrate data from SurrealDB
                    →  pure game engine functions
                    →  render HTML via maud templates
                    →  return full HTML page  →  HTMX swaps fragments
```

**HTMX-driven Pages**:
| Route | Handler | HTMX Features |
|-------|---------|---------------|
| `GET /games` | `games_list_handler` | hx-get for "Load More" pagination |
| `GET /games/{id}` | `game_detail_handler` | hx-ext="sse" for live log streaming |
| `POST /games/new` | `create_game_post_handler` | Standard form POST |
| `POST /auth/login` | `login_post_handler` | Form POST + hx-target for inline errors |
| `GET /auth` | `auth_handler` | Client-side tab switching (no HTMX) |

**SSE Streaming**:
- `GET /api/games/{game_id}/events` — SSE endpoint for live game log updates
- Connected via `hx-ext="sse"` in the game detail template
- Streams new log entries as the simulation progresses

**Static Assets**:
- `/assets/main.css` — Compiled stylesheet
- `/icons/sprite-ui.svg` / `/icons/sprite-narrative.svg` — SVG icon sprites
- Served via Axum's `ServeDir` tower service (no separate frontend build)

**No CORS needed** — frontend and API are the same server.

---

## API Endpoints Reference

### Games (`/api/games`)

| Method | Path | Handler | Protected | Purpose |
|--------|------|---------|-----------|---------|
| GET | `/` | `game_list` | ✓ | List all games visible to user |
| POST | `/` | `create_game` | ✓ | Create new game with tributes/areas |
| GET | `/{id}` | `game_detail` | ✓ | Full game state (detail view) |
| PUT | `/{id}` | `game_update` | ✓ | Update game name/private flag |
| DELETE | `/{id}` | `game_delete` | ✓ | Delete game and all related data |
| GET | `/{id}/areas` | `game_areas` | ✓ | Get all areas with items |
| GET | `/{id}/display` | `game_display` | ✓ | Optimized display view |
| GET | `/{id}/log/{day}` | `game_day_logs` | ✓ | Logs for specific day |
| GET | `/{id}/log/{day}/{trib}` | `tribute_logs` | ✓ | Logs for tribute on day |
| PUT | `/{id}/next` | `next_step` | ✓ | Run simulation step |
| PUT | `/{id}/publish` | `publish_game` | ✓ | Make game public |
| PUT | `/{id}/unpublish` | `unpublish_game` | ✓ | Make game private |

### Tributes (`/api/games/{game_id}/tributes`)

| Method | Path | Handler | Protected | Purpose |
|--------|------|---------|-----------|---------|
| GET | `/` | `game_tributes` | ✓ | List all tributes in game |
| GET | `/{id}` | `tribute_detail` | ✓ | Full tribute state with items |
| PUT | `/{id}` | `tribute_update` | ✓ | Update tribute name |
| DELETE | `/{id}` | `tribute_delete` | ✓ | Delete tribute |
| GET | `/{id}/log` | `tribute_log` | ✓ | All logs for tribute |

### Users (`/api/users`)

| Method | Path | Handler | Protected | Purpose |
|--------|------|---------|-----------|---------|
| GET | `/` | `session` | ✗ | Debug: show session data |
| POST | `/` | `user_create` | ✗ | Register new user |
| POST | `/authenticate` | `user_authenticate` | ✗ | Login, get JWT |

---

## Key Data Models

### API Types (lib.rs)

```rust
pub struct AppState {
    pub db: Surreal<Any>,  // Cloneable database connection
}

pub enum AppError {
    NotFound(String),           // 404
    InternalServerError(String), // 500
    BadRequest(String),         // 400
    Unauthorized(String),       // 401
    GameFull(String),           // 400 (tribute limit)
    DbError(String),            // 500
    InvalidStatus(String),      // 500
}
```

### Relation Models (games.rs, tributes.rs)

```rust
// game → area relation
pub struct GameAreaEdge {
    in: RecordId,   // game
    out: RecordId,  // area
}

// area → item relation
pub struct AreaItemEdge {
    in: RecordId,   // area
    out: RecordId,  // item
}

// tribute → game relation
struct TributeGameEdge {
    in: RecordId,   // tribute
    out: RecordId,  // game
}

// tribute → item relation
pub struct TributeItemEdge {
    in: RecordId,   // tribute
    out: RecordId,  // item
}
```

### Auth Models (users.rs)

```rust
struct Params {
    username: String,
    password: String,
}

struct JwtResponse {
    jwt: String,  // Insecure token for client
}
```

---

## Dependencies

**Core Framework**:
- `axum` 0.8.4 - HTTP server, routing, extractors
- `tokio` 1.45.0 - Async runtime
- `tower` / `tower-http` - Middleware (CORS, tracing, timeout)
- `serde` / `serde_json` - Serialization

**Database**:
- `surrealdb` 2.3.2 - Database client (platform-specific features)
- `surrealdb-migrations` 2.2.2 - Schema management

**Game Logic** (workspace crates):
- `game` - Pure simulation engine
- `shared` - Shared types
- `announcers` - LLM commentary (not used in API)

**Utilities**:
- `uuid` 1.16.0 - Unique identifiers
- `chrono` 0.4.41 - Timestamps
- `futures` 0.3.31 - Parallel operations
- `thiserror` 2.0.12 - Error types
- `tracing` / `tracing-subscriber` - Logging
- `base64-url` 3.0.0 - JWT decoding
- `time` 0.3.41 - Timestamp handling
- `strum` 0.27.1 - Enum utilities

---

## Security Considerations

**Strengths**:
- JWT authentication on protected routes
- SurrealDB handles password hashing
- SQL injection prevented (parameterized queries)
- Row-level permissions in database schema
- Transaction-based atomic updates

**Weaknesses**:
- JWT tokens checked for expiration but no refresh mechanism
- No rate limiting
- Error messages may expose internal details
- CORS allows any origin (`AllowOrigin::any()`)
- No input validation (relies on Serde deserialization)
- Some endpoints use `.expect()` instead of proper error handling

---

## Performance Notes

**Optimizations**:
- Concurrent futures for bulk operations (game creation, state saves)
- SurrealDB custom functions reduce round-trips
- Diffing for item updates minimizes writes
- Lazy static routers compiled once

**Bottlenecks**:
- `save_game` diffing is O(n) per tribute/area
- No pagination on game lists
- Full game state fetched for every step
- No caching layer

**Scalability**:
- Stateless API (can horizontally scale)
- No connection pooling (single connection per AppState clone)
- SurrealDB connection via websocket (persistent)

---

## Open Issues / TODOs

1. **messages.rs incomplete** - `save_global_messages_to_db` has `todo!()` placeholder
2. **logging.rs unused** - Custom tracing layer code present but not active
3. **Error handling inconsistencies** - Some `.expect()` calls should return `AppError`
4. **Publish/unpublish bugs** - SQL uses `'$identifier'` instead of `$identifier` binding
5. **No refresh token mechanism** - JWT expiration checked but no rotation
6. **Transaction coverage incomplete** - Not all multi-step operations use transactions
7. **No pagination** - Game/tribute lists unbounded

---

## Testing Strategy

**Current State**: No tests in api crate (game crate has ~60 tests).

**Recommended Additions**:
- Integration tests against real SurrealDB instance
- Mock `AppState` for unit tests
- Test auth middleware separately
- Verify transaction rollback on errors
- Test concurrent creation/updates
- Test error conversion to HTTP responses

---

## Related Documentation

- `../game/src/` - Core simulation logic (pure Rust, no I/O)
- `../shared/src/` - Shared types between API and frontend
- `./assets/src/main.css` - Design tokens and component CSS
- `schemas/` - SurrealDB schema definitions
- `migrations/` - Database migration files
- `justfile` - Development commands (`just api`, `just dev`)

---

**Last Updated**: 2026-05-25
