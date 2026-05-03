# web/src/components/

**Framework**: Dioxus (Rust → WASM)  
**Total Lines**: ~5,032 across 42+ files  
**Purpose**: Frontend UI components for Hangrier Games simulation

---

## Responsibility

This directory contains all frontend UI components for the Hangrier Games web application. It provides:

1. **Page-level components** for routing (Home, GamesList, GameDetail, TributeDetail)
2. **UI primitives** (Button, Input, Modal, InfoDetail)
3. **Game management** (create, edit, delete, advance games)
4. **Tribute visualization** (status icons, inventories, attributes, logs)
5. **Area display** (map, items, events)
6. **Icon mapping** (50+ SVG components for items/statuses)
7. **Theme system** (3 colorschemes with typography + color variants)

Components are **pure presentation** - all game logic lives in `game/` crate. API communication uses `dioxus-query` for caching and invalidation.

---

## Design

### Architecture Patterns

#### 1. Query-Driven State Management

Uses `dioxus-query` for API state (similar to React Query):

```rust
// Define fetch function
async fn fetch_games(keys: Vec<QueryKey>, token: String) 
    -> QueryResult<QueryValue, QueryError>

// Use in component
let query = use_get_query(
    [QueryKey::AllGames, QueryKey::Games],
    move |keys| fetch_games(keys, token.clone())
);

// Render based on state
match query.result().value() {
    QueryState::Settled(Ok(QueryValue::DisplayGames(games))) => { /* success */ },
    QueryState::Loading(_) => { /* spinner */ },
    QueryState::Settled(Err(e)) => { /* error */ },
}
```

**Benefits**:
- Automatic caching by `QueryKey`
- Background refetching
- No manual loading/error state management

**Mutations** (POST/PUT/DELETE) invalidate caches:
```rust
client.invalidate_queries(&[QueryKey::Games]); // Triggers refetch
```

#### 2. Context-Based Global State

Root app provides 6 context signals:

| Signal                | Type                      | Purpose                         |
|-----------------------|---------------------------|---------------------------------|
| `loading_signal`      | `Signal<LoadingState>`    | Full-screen loading modal       |
| `theme_signal`        | `Signal<Colorscheme>`     | theme1/theme2/theme3            |
| `game_signal`         | `Signal<Option<Game>>`    | Current game (mostly unused)    |
| `delete_game_signal`  | `Signal<Option<DeleteGame>>` | Delete modal trigger         |
| `edit_game_signal`    | `Signal<Option<EditGame>>`   | Edit game modal trigger      |
| `edit_tribute_signal` | `Signal<Option<EditTribute>>` | Tribute edit modal trigger  |

Components consume via:
```rust
let mut edit_game_signal = use_context::<Signal<Option<EditGame>>>();
edit_game_signal.set(Some(EditGame(id, name, private))); // Opens modal
```

#### 3. Modal System

**Pattern**: Signal-triggered overlays

1. Button component sets context signal to `Some(data)`
2. Modal reads signal to determine `open: bool`
3. Form submits mutation → invalidates cache → sets signal to `None`

**Base Modal** (`modal.rs`):
- Fixed overlay with `backdrop-blur-sm`
- Themed background/border per colorscheme
- Accepts `title`, `open`, `children`

**Modals**:
- `EditGameModal` - Game name/privacy
- `EditTributeModal` - Tribute name/avatar
- `DeleteGameModal` - Confirm deletion
- `LoadingModal` - Full-screen spinner

#### 4. Icon Mapping Components

**Pattern**: Exhaustive match on domain types → SVG components

**ItemIcon** (`item_icon.rs`):
```rust
match item.item_type {
    ItemType::Consumable => match item.attribute {
        Attribute::Health => rsx!(HealthPotionIcon { class }),
        Attribute::Sanity => rsx!(SpinningTopIcon { class }),
        // ...
    },
    ItemType::Weapon => match weapon_name {
        "sword" => rsx!(PointySwordIcon { class }),
        "dagger" => rsx!(PlainDaggerIcon { class }),
        // ...
    }
}
```

**TributeStatusIcon** (`tribute_status_icon.rs`):
- Maps 14 status variants to game-icons.net SVGs
- Examples: `Healthy → HeartsIcon`, `Dead → DeadIcon`, `Poisoned → PoisonBottleIcon`

#### 5. Theme System

**Three Colorschemes** (persisted in localStorage):

| Theme   | Colors           | Font              | Visual Style           |
|---------|------------------|-------------------|------------------------|
| theme1  | Red/amber/stone  | Cinzel (serif)    | Fire/brutality         |
| theme2  | Green/teal       | Playfair (serif)  | Ocean waves, organic   |
| theme3  | Gold/stone       | Orbitron (sans)   | Luxury metallic        |

**CSS Pattern** (Tailwind variants):
```rust
class: r#"
    theme1:bg-red-900 theme1:text-amber-300 theme1:font-[Cinzel]
    theme2:bg-green-800 theme2:text-green-200 theme2:font-[Playfair_Display]
    theme3:bg-stone-50 theme3:text-stone-700 theme3:font-[Orbitron]
"#
```

**Theme Switcher** (navbar.rs):
- Focus-based dropdown with radio buttons
- Each theme has custom mockingjay icon variant
- Updates `theme_signal` + localStorage on selection

#### 6. Collapsible Sections (InfoDetail)

**Pattern**: Themed `<details>` with animated chevron

```rust
InfoDetail {
    title: "Tributes",
    open: false,
    GameTributes { game }
}
```

**Features**:
- CSS `group-open:` variants for expanded state
- 180deg chevron rotation
- Used for: Areas, Tributes, Day Log, Inventory, Attributes

---

## Flow

### Component Hierarchy

```
App (root)
├── Navbar (header + theme switcher)
│   └── Router<Routes>
│       ├── Home (landing page)
│       ├── GamesList
│       │   ├── CreateGameButton / CreateGameForm
│       │   ├── GameListMember (per game)
│       │   │   ├── GameEdit (pencil icon)
│       │   │   └── GameDelete (trash icon)
│       │   └── RefreshButton
│       ├── GamePage
│       │   ├── GameState (header + play button)
│       │   ├── GameStats (day/status/tributes)
│       │   └── GameDetails
│       │       ├── InfoDetail: GameAreaList
│       │       │   └── Map (SVG visualization)
│       │       ├── InfoDetail: GameTributes
│       │       │   └── GameTributeListMember (per tribute)
│       │       │       ├── ItemIcon (inventory)
│       │       │       ├── TributeStatusIcon
│       │       │       └── TributeEdit (if not started)
│       │       └── InfoDetail: GameDayLog
│       └── TributeDetail
│           ├── InfoDetail: Overview (avatar + stats)
│           ├── InfoDetail: Inventory (ItemIcon list)
│           ├── InfoDetail: Attributes (12 stats)
│           └── InfoDetail: TributeLog (day messages)
│
├── EditGameModal (global)
├── EditTributeModal (global)
└── LoadingModal (global)
```

### Data Flow Sequence

#### Creating a Game

1. User clicks "Quickstart" or submits name form
2. `CreateGameButton/Form` sets `loading_signal` → `LoadingModal` shows
3. Mutation: `POST /api/games` → Returns `Game`
4. Invalidates `QueryKey::Games`
5. `GamesList` query refetches automatically
6. `loading_signal` → `LoadingState::Loaded` → Modal hides

#### Playing a Game Day

1. User clicks "Play day N" button (`GameState`)
2. `handle_next_step()` spawns async mutation
3. `LoadingModal` shows via `loading_signal`
4. Mutation: `PUT /api/games/{id}/next`
5. Response codes:
   - `201 CREATED` → Game started
   - `200 OK` → Day advanced
   - `204 NO_CONTENT` → Game finished
6. Invalidates `QueryKey::DisplayGame(id)` + `QueryKey::Games`
7. All 3 detail components (`GameState`, `GameStats`, `GameDetails`) refetch
8. UI updates: day increments, tributes move, log appears

#### Editing a Tribute

1. User clicks pencil icon on tribute card (`TributeEdit`)
2. Sets `edit_tribute_signal` → `Some(EditTribute(id, name, avatar, game_id))`
3. `EditTributeModal` renders (reads signal for `open: true`)
4. User edits form, clicks "Update"
5. Mutation: `PUT /api/games/{game_id}/tributes/{id}`
6. Invalidates `QueryKey::Tribute(game_id, id)`
7. Tribute card refetches, shows new name
8. Modal dismisses (`edit_tribute_signal.set(None)`)

### Query Invalidation Map

| Action              | Mutation              | Invalidates Keys                          |
|---------------------|-----------------------|-------------------------------------------|
| Create game         | POST /api/games       | `Games`                                   |
| Edit game           | PUT /api/games/{id}   | `DisplayGame(id)`, `Games`                |
| Delete game         | DELETE /api/games/{id}| `Games`                                   |
| Advance game        | PUT /api/games/{id}/next | `DisplayGame(id)`, `Games`             |
| Edit tribute        | PUT /api/games/{gid}/tributes/{tid} | `Tribute(gid, tid)`        |

---

## Integration

### With Other Crates

#### `game/` Crate (Core Logic)
- **Types Used**: `Game`, `Tribute`, `Item`, `TributeStatus`, `Attributes`, `AreaDetails`, `GameMessage`
- **Pattern**: Components map domain enums to UI (never modify game state)
- **Example**: `TributeStatusIcon` uses `game::tributes::statuses::TributeStatus`

#### `shared/` Crate (API DTOs)
- **Types Used**: `DisplayGame`, `EditGame`, `EditTribute`, `DeleteGame`, `GameStatus`
- **Purpose**: Lightweight types for API responses (avoid sending full game state)
- **Example**: `GamesList` uses `DisplayGame` (has `living_count`, `day`, `is_mine`)

#### `api/` Crate (Backend)
- **Integration**: All API calls via `reqwest` to `APP_API_HOST` (env var)
- **Authentication**: HttpOnly `hg_session` cookie attached automatically via `WithCredentials` shim
- **Endpoints**: 15+ REST endpoints (see API Integration table in codemap)

### With Web Infrastructure

#### `web/src/cache.rs`
Defines query/mutation types:
```rust
enum QueryKey { AllGames, DisplayGame(String), Tributes(String), ... }
enum QueryValue { DisplayGames(Vec<DisplayGame>), Game(Box<Game>), ... }
enum QueryError { NoGames, GameNotFound(String), Unauthorized, ... }
enum MutationValue { NewGame(Game), GameStarted(String), ... }
```

#### `web/src/storage.rs`
Wraps `dioxus-sdk` localStorage:
```rust
struct AppState {
    username: Option<String>,
    colorscheme: Colorscheme,
}
use_persistent("hangry-games", AppState::default);
```

#### `web/src/routes.rs`
Dioxus router enum:
```rust
#[derive(Routable)]
enum Routes {
    #[route("/")]
    Home {},
    
    #[route("/games")]
    GamesList {},
    
    #[route("/games/:identifier")]
    GamePage { identifier: String },
    
    #[route("/games/:game_identifier/tributes/:tribute_identifier")]
    TributeDetail { game_identifier: String, tribute_identifier: String },
}
```

#### `web/src/env.rs`
Build-time codegen (from `build.rs`):
```rust
pub const APP_API_HOST: &str = "http://127.0.0.1:3000";
```

### External Dependencies

#### Dioxus Framework
- **RSX Syntax**: HTML-like macros (`rsx! { div { ... } }`)
- **Signals**: Reactive state (`use_signal`, `use_context`)
- **Routing**: `Router`, `Link`, `Outlet`

#### Dioxus Query
- **Hooks**: `use_get_query`, `use_mutation`, `use_query_client`
- **States**: `QueryState`, `MutationState`
- **Caching**: Automatic by key, manual invalidation

#### Reqwest
- **HTTP Client**: All API calls (async/await)
- **Cookie Auth**: `.with_credentials()` (from `crate::http::WithCredentials`) on every authed request; browser sends `hg_session` automatically

#### Tailwind CSS
- **Utility Classes**: Built at compile time (`npm run build:css`)
- **Custom Variants**: `theme1:`, `theme2:`, `theme3:`
- **Responsive**: `sm:`, `md:`, `lg:`, `xl:`

---

## Directory Structure

```
components/
├── mod.rs              - Public exports for parent modules
├── app.rs              - Root component with global state
│
├── Core Pages (Routing)
│   ├── home.rs
│   ├── games_list.rs
│   ├── game_detail.rs
│   └── tribute_detail.rs
│
├── Game Management
│   ├── create_game.rs
│   ├── game_edit.rs
│   ├── game_delete.rs
│   └── games.rs
│
├── Game Display
│   ├── game_tributes.rs
│   ├── game_areas.rs
│   ├── game_day_log.rs
│   ├── game_day_summary.rs
│   └── map.rs
│
├── Tribute Management
│   ├── tribute_edit.rs
│   └── tribute_delete.rs
│
├── UI Primitives
│   ├── button.rs
│   ├── input.rs
│   ├── modal.rs
│   ├── info_detail.rs
│   ├── loading_modal.rs
│   └── navbar.rs
│
├── Icon Systems
│   ├── item_icon.rs
│   ├── tribute_status_icon.rs
│   └── icons/
│       ├── mod.rs
│       ├── edit.rs, delete.rs, uturn.rs
│       ├── eye_open.rs, eye_closed.rs
│       ├── lock_open.rs, lock_closed.rs
│       ├── map_pin.rs, loading.rs
│       ├── mockingjay.rs, mockingjay_arrow.rs, mockingjay_flight.rs
│       └── game_icons_net/
│           ├── mod.rs
│           └── 37 game icons (weapons, consumables, statuses)
│
└── Other
    ├── accounts.rs
    ├── credits.rs
    ├── icons_page.rs
    └── server_version.rs
```

**Total**: 42+ files, ~5,032 lines

---

## Key Patterns

### 1. Async Spawn for Mutations
```rust
spawn(async move {
    mutate.mutate_async((args, token)).await;
    if let MutationState::Settled(Ok(_)) = mutate.result() {
        client.invalidate_queries(&[key]);
    }
});
```

### 2. Conditional Rendering
```rust
if game.is_mine {
    GameEdit { ... }
} else {
    p { "By {creator}" }
}
```

### 3. Data Attributes for Theming
```rust
"data-alive": tribute.is_alive(),
class: "data-[alive=false]:border-red-500"
```

### 4. Focus-Based Dropdowns
```rust
input { id: "theme-switcher", class: "peer sr-only" }
label { r#for: "theme-switcher" }
div { class: "peer-focus:visible peer-focus:opacity-100" }
```

### 5. Authenticated Request Pattern
```rust
use crate::http::WithCredentials;
let req = client.get(url).with_credentials();
```

---

## Performance Notes

### Query Caching
- **Good**: Prevents refetch on re-renders
- **Issue**: `game_tributes.rs` has N+1 problem (fetches each tribute individually)

### WASM Size
- 50+ SVG icons embedded → ~25KB overhead
- **Optimization**: Consider sprite sheets or lazy loading

### Rendering
- Dioxus reactive diffing → Only changed subtrees re-render
- Signals prevent unnecessary prop drilling

---

## Accessibility

- **Screen Readers**: `sr-only` class, `aria_label` attributes
- **Keyboard Nav**: Focus states on buttons/links
- **Semantic HTML**: `<nav>`, `<details>`, `<dialog>`, `<dl>`
- **Alt Text**: Icon `title` attributes

---

## Responsive Design

### Breakpoints
- `sm:` (640px) - Tablet layout
- `md:` (768px) - Desktop text sizes
- `lg:` (1024px) - Wide grids
- `xl:` (1280px) - 3/4 column layouts

### Adaptive Layouts
- `GameDetails`: 1-col → 2-col → 3-col grid
- `GameTributes`: District grouping adjusts per screen size

---

## Future Improvements

1. **Tribute Outlook**: TODO placeholder in `tribute_detail.rs` (line 186)
2. **Batch Tribute Fetching**: Eliminate N+1 in `game_tributes.rs`
3. **Icon Optimization**: SVG sprite sheets
4. **Error Recovery**: Retry buttons for failed queries
5. **Skeleton Screens**: Replace "Loading..." text
6. **Offline Mode**: Cache responses for offline viewing

---

*Generated*: 2026-04-05  
*Explorer*: Cartography Skill Agent
