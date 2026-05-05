# web/src/

## Responsibility

The `web/src/` directory implements the **Dioxus WebAssembly frontend** for Hangrier Games. It provides:

- Interactive UI for browsing, creating, and managing games
- Real-time game simulation visualization (tributes, areas, day logs)
- User authentication (login/register)
- Theme switching (3 color schemes)
- Persistent state via browser LocalStorage
- Type-safe routing with nested layouts
- Async data fetching with caching and mutations

**Tech Stack**: Dioxus 0.6 (React-like framework), dioxus-query (async state), reqwest (HTTP client), Tailwind CSS, compiles to WASM for browser execution.

## Design

### Architecture Layers

1. **Entry Point** (`main.rs`): Launches root `App` component
2. **Root Component** (`components/app.rs`): Sets up global state (context providers, query client, persistent storage)
3. **Router** (`routes.rs`): Type-safe routing with nested layouts (Navbar → Games/Accounts → specific pages)
4. **Components** (`components/`): UI components using RSX (JSX-like syntax)
5. **State Management**: Three-tier system:
   - **Context providers**: Global signals (theme, loading state, modal triggers)
   - **Persistent storage**: LocalStorage hook (`use_persistent`) for theme + username (auth lives in HttpOnly cookies set by API)
   - **Query client**: dioxus-query for API data fetching/caching/mutations

### Key Patterns

**Query Pattern** (read operations):
- Define `QueryKey` enum variants (e.g., `AllGames`, `DisplayGame(String)`)
- Async fetch functions return `QueryResult<QueryValue, QueryError>`
- Components use `use_get_query([keys], fetch_fn)` to declaratively fetch data
- Automatic caching, loading states, error handling

**Mutation Pattern** (write operations):
- Async mutation functions return `MutationResult<MutationValue, MutationError>`
- Components use `use_mutation(mutate_fn)` to trigger side effects
- After success, invalidate related queries to refetch fresh data

**Context Providers**:
- Global state accessible via `use_context::<Signal<T>>()`
- Used for theme, loading overlays, modal triggers
- Set once in `App`, read/write anywhere in component tree

**Persistent Storage**:
- `use_persistent("key", default)` hook wraps `Signal<T>` with LocalStorage sync
- Auto-saves on `.set()`, auto-loads on init
- Used for username (display only) and theme preference; the JWT session lives in an HttpOnly `hg_session` cookie

### Build System

**`build.rs`** (codegen):
- Reads `.env` at **build time**
- Generates `src/env.rs` with constants (e.g., `pub const APP_API_HOST: &str = "http://..."`)
- Only processes `APP_*` env vars
- **Critical**: Changing `.env` requires rebuild

**WASM Compilation**:
- Target: `wasm32-unknown-unknown`
- Requires: `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` (for RNG in WASM)
- Dioxus CLI (`dx serve`) handles build, bundling, hot reload
- Config: `Dioxus.toml` (output dir: `dist/`, assets: `assets/`)

## Flow

### Data Read Flow (Queries)

1. **Component mounts** → calls `use_get_query([QueryKey::SomeKey], fetch_fn)`
2. **Query client checks cache** → if miss/stale, calls `fetch_fn`
3. **Fetch function** → makes HTTP request to API with `WithCredentials` (browser attaches `hg_session` cookie), returns `QueryResult<QueryValue, QueryError>`
4. **Component re-renders** → pattern matches on `QueryState::Loading | Settled(Ok/Err)` to render UI
5. **Cache updated** → subsequent reads from same key hit cache

### Data Write Flow (Mutations)

1. **User action** (e.g., "Create Game" button) → calls `mutation.action(data)`
2. **Mutation function** → makes HTTP POST/PATCH/DELETE to API (cookies attached via `with_credentials()`)
3. **Response handled** → returns `MutationResult<MutationValue, MutationError>`
4. **On success** → component invalidates related queries (e.g., `query_client.invalidate_query([QueryKey::AllGames])`)
5. **Query refetch** → cache cleared, components re-fetch fresh data

### Auth Flow

1. **User logs in** → `authenticate_user` mutation hits `/api/users/authenticate`
2. **API sets cookies** → `hg_session` (HttpOnly, 1h) + `hg_refresh` (HttpOnly, 7d, Path=/api/auth)
3. **Username persisted** locally for nav rendering only; cookies are invisible to JS
4. **API calls** → browser attaches `hg_session` automatically when request uses `with_credentials()`
5. **Logout** → POST `/api/auth/logout` with credentials; API clears both cookies, web clears `username`

### Theme Switching

1. **User selects theme** → updates `Signal<Colorscheme>` (via context)
2. **Root div re-renders** → `class="{theme}"` applies `theme1` / `theme2` / `theme3`
3. **Tailwind classes react** → `theme1:bg-red-900`, `theme2:bg-green-800`, etc.
4. **Favicon updates** → conditional `document::Link` based on theme
5. **Persist to storage** → save preference via `use_persistent` hook

### Routing Flow

1. **User navigates** → URL changes (e.g., `/games/abc123`)
2. **Router matches** → `Routes::GamePage { identifier: "abc123" }`
3. **Nested layouts render** → `Navbar` → `Games` → `GamePage`
4. **Component fetches data** → `use_get_query([QueryKey::DisplayGame("abc123")], ...)`
5. **Page renders** → with game details

## Integration

### API Communication

**Base URL**: Read from `env::APP_API_HOST` (generated by `build.rs` from `.env`)

**Common Endpoints**:
- `GET /api/games` → list all games (lightweight `DisplayGame` type)
- `GET /api/games/{id}/display` → single game (display view)
- `GET /api/games/{id}` → full game state (with simulation details)
- `POST /api/games` → create game
- `PATCH /api/games/{id}` → update game (name, status, etc.)
- `DELETE /api/games/{id}` → delete game
- `POST /api/games/{id}/start` → start simulation
- `POST /api/games/{id}/advance` → advance to next day
- `GET /api/games/{id}/tributes` → list tributes
- `GET /api/games/{id}/logs/{day}` → day logs
- `POST /api/users` → register
- `POST /api/users/authenticate` → login
- `GET /api/version` → server version

**Auth**: All protected requests carry the `hg_session` cookie automatically (set when calling `.with_credentials()`); `Authorization: Bearer` is still accepted for non-browser clients

### Shared Types (`shared` crate)

**Imported from `shared/`**:
- `DisplayGame` - Lightweight game view (id, name, status, metadata)
- `GameStatus` - Enum: `Pending`, `InProgress`, `Finished`
- `AuthenticatedUser` - User + JWT response
- `RegistrationUser` - Username + password
- `TributeKey` - Tribute identifier
- `DeleteGame`, `EditGame`, `EditTribute` - Modal trigger types

**Ensures type safety** between frontend and API (both import same structs)

### Game Logic (`game` crate)

**Imported types**:
- `Game` - Full game state (tributes, areas, logs, config)
- `Tribute` - Tribute details (stats, inventory, status effects)
- `AreaDetails` - Arena area metadata
- `GameMessage` - Log entry (events that happened each day)

**Note**: Frontend imports `game` crate for types only (no simulation logic runs in WASM; API handles all game ticks)

### Browser APIs (via `gloo-storage`)

- **LocalStorage**: Persist theme + username (display) across sessions; auth tokens live in HttpOnly cookies
- **Wrappers**: `use_persistent` hook abstracts `LocalStorage::get/set`

### Asset Pipeline

1. **Tailwind CSS**: Built via `npm` in `assets/` → `assets/dist/main.css`
2. **Fonts**: Google Fonts loaded via `<link>` in `App` component
3. **Favicons**: Theme-specific PNG files in `assets/favicons/`
4. **Icons**: Sprite-sheet based system. SVG sources live in `assets/icons/{ui,narrative}/`; `build.rs` generates `src/icons_generated.rs` with `IconName` enum, `SPRITE` const, and per-icon wrapper components. Public API in `src/icons.rs`. Sprite is inlined once at app root (`#icon-sprite`); icons reference it via `<use href="#ui-edit">` etc.

### Development Workflow

- **Start all services**: `just dev` (SurrealDB + API + web frontend)
- **Frontend only**: `dx serve` (or `just web`)
- **Hot reload**: Edit `src/**/*.rs` → Dioxus CLI rebuilds WASM → browser refreshes
- **Build CSS**: `just build-css` (or `npm run build` in `assets/`)
- **Prod build**: `dx build --release` → outputs to `dist/` (WASM + JS glue + assets)

### File Structure Map

```
src/
├── main.rs              # Entry point (launches App)
├── lib.rs               # Crate root, LoadingState enum
├── env.rs               # GENERATED by build.rs (APP_* env vars)
├── routes.rs            # Route definitions (Routable enum)
├── storage.rs           # use_persistent hook, AppState, Colorscheme
├── cache.rs             # Query/mutation types (QueryKey, QueryValue, etc.)
└── components/
    ├── mod.rs           # Public exports
    ├── app.rs           # Root component (context, query client, router)
    ├── navbar.rs        # Top nav
    ├── home.rs          # Landing page
    ├── games.rs         # Games layout (Outlet wrapper)
    ├── games_list.rs    # List games
    ├── game_detail.rs   # Game detail page
    ├── game_edit.rs     # Edit game modal
    ├── game_delete.rs   # Delete game modal
    ├── create_game.rs   # Create game form
    ├── game_areas.rs    # Areas list
    ├── game_tributes.rs # Tributes list
    ├── game_day_log.rs  # Day-by-day log
    ├── tribute_detail.rs # Tribute detail page
    ├── tribute_edit.rs  # Edit tribute modal
    ├── accounts.rs      # Login/register/logout
    ├── button.rs        # Button, ThemedButton
    ├── input.rs         # Themed input
    ├── modal.rs         # Modal wrapper
    ├── loading_modal.rs # Global loading overlay
    ├── server_version.rs # Server version display
    ├── credits.rs       # Credits page
    ├── icons_page.rs    # Icon showcase
    └── ui/icon.rs       # Icon primitive (IconSize, IconTier)
```

### Icon System

- Source SVGs in `web/assets/icons/{ui,narrative}/`
- `build.rs` walks the dirs and emits `web/src/icons_generated.rs` (gitignored)
- Public API: `crate::icons::{Icon, IconSize, IconTier, NarrativeIcon, LoadingIcon, EditIcon, ...}`
- Color via `currentColor` (use `text-*` Tailwind utilities; sprite paths set fill/stroke to currentColor)
- Sprite inlined at app root in `App` via `<div id="icon-sprite" hidden dangerous_inner_html=SPRITE>`

### Dependencies

**Core**:
- `dioxus` 0.6.3 (UI framework, router)
- `dioxus-query` 0.7.0 (async state management)
- `reqwest` 0.12.9 (HTTP client)
- `serde`, `serde_json` (JSON serialization)

**Browser**:
- `gloo-storage` 0.3.0 (LocalStorage wrapper)

**Utilities**:
- `chrono` 0.4.41 (dates/times)
- `dioxus-logger` 0.6.2 (logging)

**Build**:
- `dotenvy` 0.15.7 (`.env` parsing in `build.rs`)

**Local**:
- `game` - Core simulation types
- `shared` - API/frontend shared types
