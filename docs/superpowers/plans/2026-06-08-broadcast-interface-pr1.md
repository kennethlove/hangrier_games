---
status: in-progress
phase: 4
updated: 2026-06-09
issue: bd-e7d6
---

# Broadcast Interface — PR1: CSS Tokens + Layout Restructure + Core Components

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current 3-tab game detail page (Tributes / Areas / Log loaded via HTMX into `#detail-content`) with a single-page broadcast layout: dark control-room theme, 50/50 grid (hex map + tribute roster | event feed), ticker bar, day nav, and broadcast header. The existing Tributes/Areas/Log sub-pages become unnecessary — their data surfaces through the broadcast view.

**Design reference:** `designs/round-broadcast-5.html` — full HTML/CSS/JS prototype of the broadcast interface.

**Architecture:** All CSS tokens and layout classes go in `api/assets/src/main.css` (Tailwind v4 with plain CSS overrides). Templates stay in `api/src/templates/` using maud. A new `broadcast.rs` template module replaces `game_detail.rs`. The broadcast page is server-rendered with HTMX SSE for live updates. All hex map and roster rendering is server-side (maud generates the SVG and HTML). JavaScript interactivity (feed tabs, day nav, map zoom/pan) loads from a static JS file.

**Tech Stack:** Rust 2024, maud, Tailwind v4 (via npm), HTMX 2.0 with SSE extension, vanilla JS for broadcast interactivity.

**Spec:** `designs/round-broadcast-5.html`

**Beads issue:** `bd-819c` (create before starting)

---

## Pre-flight notes

- All CSS changes go in `api/assets/src/main.css`. Do NOT add a separate broadcast CSS file — the design system lives in one place.
- The existing `main.css` has ~936 lines. The broadcast tokens and classes will add ~400 new lines under a `/* ── BROADCAST ── */` section header. Keep existing styles intact.
- The existing `base_layout` in `templates/mod.rs` renders a topnav with `Hangry Games` logo and `Broadcast`/`Tributes`/`Arena`/`Odds` nav links. The new broadcast page needs its own top nav that matches the design (site brand hex emblem, back link, auth buttons) — NOT the existing base_layout top nav. This means the broadcast page either skips `base_layout` or `base_layout` grows an optional broadcast mode.
- The existing 3-tab system (Tributes/Areas/Log) loads content into `#detail-content` via HTMX. The new broadcast page renders everything server-side in one pass: the broadcast header, day nav, ticker, map, roster, and feed all come from a single `broadcast_page()` template function. The HTMX SSE connection for live feed updates remains, but the tab navigation goes away.
- The `game_detail_page` function in `game_detail.rs` becomes `broadcast_page` in a new `broadcast.rs`. The old `game_detail.rs` can be deleted after migration.
- The existing Tributes/Areas/Log routes (`/games/{id}/tributes`, `/games/{id}/areas`, `/games/{id}/log`) can stay as standalone pages linked from elsewhere, but the broadcast page does NOT use them.
- `just dev` runs SurrealDB + API. `just quality` runs full workspace checks.
- Commits use the jj workflow per `AGENTS.md`. Each task ends with `jj describe -m "..."` then `jj new`.

---

## File Structure

**Modified:**
- `api/assets/src/main.css` — add broadcast CSS tokens + all layout/component classes
- `api/src/templates/mod.rs` — add `broadcast` module, add `broadcast_layout` function
- `api/src/routes/games.rs` — update `game_detail_handler` to render broadcast page
- `api/src/games/handlers.rs` — expose game state needed by broadcast (tributes, areas, events, phases, alliances)

**Created:**
- `api/src/templates/broadcast.rs` — main broadcast page template:
  - `broadcast_page()` — top nav + broadcast header + day nav + ticker + main grid
  - `broadcast_header()` — round name, phase badge, alive/fallen/total stats
  - `day_nav()` — day arrows, day select, advance phase button
  - `ticker_bar()` — live dot, scrolling text, event count, clock
  - `map_section()` — hex SVG container + zoom controls + legend
  - `roster_section()` — tribute list with alliance grouping, health bars, status
  - `feed_section()` — event feed with tab filters and 4 card types
  - `event_card()` variants — ACTION (combat), ELIMINATED (death), ARENA EVENT, ANALYSIS (commentary)
  - `tribute_row()` — single tribute row with avatar, health bar, stats
  - `hex_map_svg()` — server-side SVG hex grid rendering
- `api/assets/src/broadcast.js` — client-side JS for feed tabs, day nav, map zoom/pan, roster filtering, countdown clock

**Removed:**
- `api/src/templates/game_detail.rs` — replaced by `broadcast.rs` (after migration)

---

## Task Order Rationale

Tasks build the CSS foundation first (Task 1), then the layout shell (Task 2), then populate the shell top-to-bottom (Tasks 3-5), then add interactivity (Task 6), then wire the route (Task 7). Each task is independently testable by running `just dev` and viewing a game.

---

## Task 1: [COMPLETE] Add Broadcast CSS Tokens + Layout Classes (1-2 hrs)

**Why first:** Every subsequent task depends on CSS variables and layout classes. Land these early so all templates can reference them immediately.

**File:** `api/assets/src/main.css`

**Note:** The existing `main.css` uses OKLCH color functions for its light theme. The broadcast design uses a completely different dark palette with hex color values and CSS custom properties. These new variables live in a separate `:root` block — do NOT modify the existing `:root` block. Browsers cascade the last-defined property, so the broadcast `:root` should come AFTER the existing one, but broadcast components scope their usage to the broadcast page container (`.broadcast-page`) to avoid leaking into existing pages.

**CSS section to add at the end of `main.css`:**

```css
/* ═══════════════════════════════════════════════════════════════════════════
   BROADCAST — dark control-room theme for game detail page
   ═══════════════════════════════════════════════════════════════════════════ */
```

- [x] **Step 1: Add broadcast design tokens under `.broadcast-page` scope**

```css
.broadcast-page {
  --bg:              #0b0e14;
  --surface:         #141a26;
  --surface-alt:     #1c2438;
  --surface-hover:   #222c44;
  --fg:              #e8ecf4;
  --fg-muted:        #7982a0;
  --border:          #283044;
  --border-strong:   #3d4866;
  --accent:          #00b8d9;
  --accent-glow:     rgba(0,184,217,0.15);
  --gold:            #f0b429;
  --gold-glow:       rgba(240,180,41,0.12);
  --danger:          #ef4455;
  --danger-glow:     rgba(239,68,85,0.12);
  --success:         #22c55e;
  --warning:         #f97316;
  --info:            #3b82f6;
  --purple:          #8b5cf6;

  /* Phase colors */
  --phase-dawn:     #f5b342;
  --phase-day:      #f5d742;
  --phase-dusk:     #c97b3a;
  --phase-night:    #4a6fa5;

  /* Terrain colors */
  --terrain-arena:    #5a4a3a;
  --terrain-ruins:    #8b7d6b;
  --terrain-forest:   #2d6a4f;
  --terrain-mountain: #6b7280;
  --terrain-swamp:    #5b4b6b;
  --terrain-plains:   #7c9a5e;
  --terrain-lake:     #3b82c4;

  /* Faction colors */
  --faction-0: #00b8d9;
  --faction-1: #f0b429;
  --faction-2: #ef4455;
  --faction-3: #22c55e;
  --faction-4: #8b5cf6;
  --faction-5: #f97316;
  --faction-6: #ec4899;
  --faction-7: #06b6d4;

  /* Typography */
  --font-display:  'Tahoma', 'Arial Narrow', 'Impact', -apple-system, system-ui, sans-serif;
  --font-body:     -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  --font-mono:     'Courier New', ui-monospace, monospace;
  --font-condensed: 'Arial Narrow', 'Impact', -apple-system, system-ui, sans-serif;

  --fs-hero:  clamp(28px, 3.2vw, 42px);
  --fs-h2:    clamp(18px, 2vw, 26px);
  --fs-h3:    clamp(14px, 1.4vw, 18px);
  --fs-body:  13px;
  --fs-sm:    12px;
  --fs-xs:    10px;
  --fs-xxs:   9px;
}
```

Variables match the design spec exactly. Scope under `.broadcast-page` prevents pollution.

- [x] **Step 2: Add app layout classes** — `#app` wrapper (max-width, flex column), broadcast body reset scoped to `.broadcast-page`

- [x] **Step 3: Add top nav classes** — `.top-nav` (flex row, gradient bg, border), `.nav-left`, `.nav-right`, `.site-brand` (hex emblem, lettering), `.nav-link`, `.back-link`, `.auth-btn`, `.user-badge` with hover states and active variants

- [x] **Step 4: Add broadcast header classes** — `.broadcast-header` (gradient bg, flex layout), `.round-id`, `.location`, `.phase-indicator` (4 phase color variants: dawn/day/dusk/night), `.broadcast-meta`, `.stat` blocks (alive/fallen/total)

- [x] **Step 5: Add day nav classes** — `.day-nav` (monospace, flex row), `.day-arrows`, `.day-arrow`, `.day-select`, `.phase-btn`, `.day-info`

- [x] **Step 6: Add ticker bar classes** — `.ticker-bar` (flex, border), `.live-dot` (pulsing red), `.live-label`, `.ticker-text`, `.ticker-stat`, `.clock`

- [x] **Step 7: Add main grid classes** — `.main-grid` (1fr 1fr grid, gap 10px), `.left-panel` (flex column), `.feed-section` (flex column, min-height)

- [x] **Step 8: Add map section classes** — `.map-section` (fixed height, flex column), `.section-header`, `.map-container` (overflow hidden, grab cursor), `.map-viewport` (transform origin), `.map-zoom-controls`, `.map-legend`

- [x] **Step 9: Add roster section classes** — `.roster-section` (flex column, flex:1), `.roster-scroll` (scrollable), `.pair-card` (alliance pair grid), `.tribute-card` (grid with avatar/info/stats), `.tribute-avatar`, `.health-bar`, `.health-fill` (3 color ranges), `.tribute-status`, `.tribute-alliance badge`, `.tribute-tooltip` (hover card with stats)

- [x] **Step 10: Add event feed classes** — `.feed-scroll` (scrollable), `.feed-tabs`, `.feed-tab`, `.event-card` base + 4 variants:
  - `.event-card.action` — blue left border, combat card with avatar/verb/target/detail
  - `.event-card.death` — red bold border, quote style with death-meta footer
  - `.event-card.event` — purple top border, title/description/affected pills
  - `.event-card.commentary` — gold left border, italic quote with speaker attribution

- [x] **Step 11: Add responsive breakpoints** — `@media(max-width:1100px)` single-column grid, `@media(max-width:720px)` stacked layout. Match design breakpoints exactly.

- [x] **Step 12: Run `just quality` to verify no build breaks from CSS changes**

**Done when:** `just dev` starts without errors; broadcast CSS variables are inspectable in browser devtools on a game detail page.

---

## Task 2: [COMPLETE] Create `broadcast_layout` and `broadcast.rs` Template Module (2-3 hrs)

**Why second:** The layout shell must exist before we can populate it with components. This task creates the template module and the empty broadcast page function.

**Files:**
- Modify: `api/src/templates/mod.rs`
- Create: `api/src/templates/broadcast.rs`

- [x] **Step 1: Add broadcast module to `templates/mod.rs`**

Add `pub mod broadcast;` alongside existing module declarations.

- [x] **Step 2: Create `broadcast_layout()` function**

Unlike `base_layout` (which renders the site topnav with the "Hangry Games" logo and the Broadcast/Tributes/Arena/Odds links and the footer), `broadcast_layout` renders:
- The broadcast-specific top nav (hex emblem, "HANGRIER GAMES" brand, back link, auth buttons)
- NO footer (the broadcast page is full-height, no footer needed)
- The `.broadcast-page` wrapper div that scopes all broadcast CSS variables
- HTMX SSE extension script tag and broadcast.js script tag

```rust
pub fn broadcast_layout(title: &str, auth: AuthState, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Hangry Games Broadcast" }
                link rel="stylesheet" href="/assets/main.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                script src="https://unpkg.com/htmx-ext-sse@2.2.3" {}
                script src="/assets/broadcast.js" {}
            }
            body {
                div class="broadcast-page" {
                    // Top nav
                    div class="top-nav" {
                        div class="nav-left" {
                            div class="site-brand" {
                                div class="emblem" {}
                                span { "HANGRIER " span class="brand-accent" { "GAMES" } }
                            }
                            a href="/games" class="nav-link back-link" { "ALL GAMES" }
                        }
                        div class="nav-right" {
                            // Auth state — logged in vs guest
                            ...
                        }
                    }
                    (content)
                }
            }
        }
    }
}
```

Auth section mirrors pattern from `auth_links` in `mod.rs` but styled for broadcast (user-badge with green dot for logged in, auth-btn for guest).

- [x] **Step 3: Create `broadcast_page()` function skeleton**

```rust
pub fn broadcast_page(
    auth: AuthState,
    game: &DisplayGame,
    tributes: &[Tribute],
    areas: &[Area],
    events: &[GameMessage],
    commentary: &[CommentarySegment],
    current_phase: &str,
    current_day: u32,
) -> maud::Markup {
    broadcast_layout(
        &game.name,
        auth,
        html! {
            div id="app" hx-ext="sse" sse-connect=(format!("/api/games/{}/events", game.identifier)) {
                // Broadcast header
                (broadcast_header(game, current_phase, current_day))
                // Day nav
                (day_nav(current_day, current_phase))
                // Ticker bar
                (ticker_bar(event_count, day_label))
                // Main grid
                div class="main-grid" {
                    // Left panel: map + roster
                    div class="left-panel" {
                        (map_section(areas, tributes))
                        (roster_section(tributes))
                    }
                    // Right panel: event feed
                    (feed_section(events, commentary))
                }
            }
        },
    )
}
```

For now each sub-function returns `maud::Markup` with a placeholder div. Implementation happens in Tasks 3-5.

- [x] **Step 4: Run `cargo check` to verify the module compiles**

**Done when:** `cargo check` succeeds, broadcast page compiles, route can render an empty broadcast page.

---

## Task 3: [COMPLETE] Broadcast Header + Day Nav + Ticker (1-2 hrs)

**Why third:** These are the top-of-page components that give the broadcast its identity. They're structurally simple and immediately visible when the page loads.

**File:** `api/src/templates/broadcast.rs`

- [x] **Step 1: Implement `broadcast_header()`**

Renders:
- Left: Capitol hex emblem + "ROUND N" (accent-colored number) + location name
- Center/right: Phase badge (DAWN/DAY/DUSK/NIGHT) — colored border + dot via phase class
- Right: Stats row — ALIVE count (green), FALLEN count (red), TOTAL count

Phase class is computed from `current_phase` string: `"phase-dawn"`, `"phase-day"`, `"phase-dusk"`, `"phase-night"`.

```rust
fn broadcast_header(game: &DisplayGame, current_phase: &str, _current_day: u32) -> maud::Markup {
    let phase_class = format!("phase-indicator phase-{}", current_phase.to_lowercase());
    html! {
        div class="broadcast-header" {
            div class="brand" {
                div class="capitol-emblem" {}
                span class="round-id" {
                    "ROUND " span class="hl" { (game.day.unwrap_or(0)) }
                }
                span class="location" { (game.arena_name) }
            }
            div class=(phase_class) id="phaseIndicator" {
                span class="phase-dot" {}
                span id="phaseLabel" { (current_phase) }
            }
            div class="broadcast-meta" {
                div class="stat" {
                    span class="num alive" id="aliveCount" { (game.living_count) }
                    span class="stat-label" { "ALIVE" }
                }
                div class="stat" {
                    span class="num fallen" id="fallenCount" { (fallen_count) }
                    span class="stat-label" { "FALLEN" }
                }
                div class="stat" {
                    span class="num" id="totalCount" { (game.tribute_count) }
                    span class="stat-label" { "TOTAL" }
                }
            }
        }
    }
}
```

- [x] **Step 2: Implement `day_nav()`**

Renders:
- Left: "DAY N" label (accent number), prev/next arrow buttons, day select dropdown
- Right: "PHASE" label, "ADVANCE +" button (only for game owner, wired via HTMX)

Day select options render days 1-10 with optional labels for special days (OPENING, FINALE, named days from game data).

- [x] **Step 3: Implement `ticker_bar()`**

Renders:
- Left: Live dot (pulsing red circle via CSS animation), "LIVE" label, divider, scrolling ticker text (first recent event or default broadcast text)
- Right: "EVENTS: N" count, divider, clock span (HH:MM format, populated by JS)

Ticker text truncates with ellipsis via CSS `text-overflow: ellipsis`.

- [x] **Step 4: Verify in browser** — Load a game, confirm header, nav, and ticker render with correct data from the game state

**Done when:** Broadcast header shows real game data (day, location, counts), phase indicator reflects current phase, day nav renders clickable arrows, ticker shows LIVE dot with event count.

---

## Task 4: [COMPLETE] Event Feed with 4 Card Types + Tab Filters (3-4 hrs)

**Why fourth:** The event feed is the most visible part of the right panel and has the most complex rendering logic. Doing it after the structural components means we can see it working in context.

**File:** `api/src/templates/broadcast.rs`

- [x] **Step 1: Implement `feed_section()`**

Renders:
- Section header: "EVENT FEED" with accent "FEED" span, feed tab buttons (ALL/ACTION/DEATHS/EVENTS/COMMS)
- Feed scroll container (`#feedScroll`) with event cards

Current events come from `GameMessage` slice. Filter tabs are client-side JS (hide/show by data attribute).

- [x] **Step 2: Implement ACTION card** — For combat/attack messages

Card class: `event-card action`
- Badge: "COMBAT" on blue background
- Body: actor avatar circle (faction-colored border), action verb (bold, uppercase), target name, detail line (weapon / -damage / location)

- [x] **Step 3: Implement ELIMINATED card** — For death/kill messages

Card class: `event-card death`
- Badge: "ELIMINATED" on red background
- Body: Quote in serif italic (e.g., "Cassia eliminated by Toran after a brutal engagement.")
- Footer: weapon name left, slayer name + kill count right (red accent)

- [x] **Step 4: Implement ARENA EVENT card** — For area events, gamemaker events

Card class: `event-card event`
- Badge: "ARENA EVENT" on purple background
- Body: Title (condensed, uppercase), description (muted, smaller), affected tributes as pills

- [x] **Step 5: Implement ANALYSIS card** — For commentary/state messages

Card class: `event-card commentary`
- Badge: "ANALYSIS" on gold background
- Body: Italic quote with gold left border, speaker row (avatar circle, speaker name + role)

- [x] **Step 6: Map GameMessage kinds to card types**

```rust
fn event_card(msg: &GameMessage) -> maud::Markup {
    use shared::messages::MessageKind::*;
    match msg.payload.kind() {
        Death => death_card(msg),
        Combat | CombatSwing => action_card(msg),
        // Area events, sleep events, gamemaker events -> arena_event_card
        // Commentary segments -> analysis_card
        // Everything else (movement, items, alliances, afflictions) -> action_card or a generic fallback
        _ => action_card(msg), // or mini_card for low-urgency events
    }
}
```

- [x] **Step 7: Add data attributes for feed filtering**

Each event card gets `data-type` attribute: `"action"`, `"death"`, `"event"`, `"commentary"`. The ALL tab shows everything, specific tabs filter by data-type. Filtering is done client-side in JS.

- [x] **Step 8: Verify event cards render** — Load a game with events, confirm all 4 card types appear with correct styling

**Done when:** All event card types render with correct colors, icons, and data. Feed tabs toggle visibility via JS. Commentary cards show speaker name and role.

---

## Task 5: Tribute Roster with Alliance Grouping (2-3 hrs)

**Why fifth:** The roster fills the bottom of the left panel. It's data-heavy but structurally straightforward — a scrollable list of alliance pairs.

**File:** `api/src/templates/broadcast.rs`

- [ ] **Step 1: Group tributes by alliance**

```rust
fn group_by_alliance(tributes: &[Tribute]) -> Vec<Vec<&Tribute>> {
    // Group tributes by their alliance name
    // Each group is a "pair" (typically 2 tributes per alliance)
    // Sort groups: alive groups first, then dead groups
}
```

- [ ] **Step 2: Implement single tribute row** — `fn tribute_row(tribute: &Tribute) -> maud::Markup`

Renders:
- Avatar circle (first letter, faction-colored border)
- Name (condensed, bold, truncated)
- Meta row: Alliance short badge (faction-colored border) + health bar (colored by fill level)
- Stats: kill count + status badge (ALIVE green / INJURED orange / DEAD red)

Health bar logic: `>60` = high (green), `>20` = mid (orange), `>0` = low (red), `0` = empty (transparent).

Dead tributes get a half-opacity card with diagonal stripe overlay pattern.

- [ ] **Step 3: Implement pair card** — `fn pair_card(tributes: &[&Tribute]) -> maud::Markup`

Wraps two tribute rows in a `.pair-card` div with a vertical divider between them and a link icon in the center.

- [ ] **Step 4: Implement `roster_section()`**

Renders:
- Section header: "TRIBUTE ROSTER" title + count (e.g., "12 ALLIANCES / 24 TRIBUTES")
- Scrollable container with pair cards

- [ ] **Step 5: Implement tribute tooltip** — Hover card showing detailed stats

Each `.tribute-card` includes a `.tribute-tooltip` div that appears on hover:
- Large avatar, name, alliance badge
- Divider
- 2x2 stat grid: Health (colored), Status (uppercase), Kills (accent), ID (muted), Alliance (full name)

Tooltip is positioned above the card, centered, with box shadow. CSS `pointer-events: none` to avoid flickering.

- [ ] **Step 6: Verify roster renders** — Load a game, confirm alliance pairs, health bars, status colors, tooltips

**Done when:** Roster displays all tributes grouped by alliance, health bars reflect actual health values, dead tributes are visually distinct, tooltips show detailed stats on hover.

---

## Task 6: Hex Arena Map (Server-Side SVG) (4-6 hrs)

**Why sixth:** The map is the most technically complex component. It goes after the roster because both share the left panel, and the roster is simpler to debug.

**Files:**
- Modify: `api/src/templates/broadcast.rs`
- Create: `api/assets/src/broadcast.js` (add zoom/pan interaction)

- [ ] **Step 1: Implement hex coordinate generation**

```rust
struct HexCoord { q: i32, r: i32 }

fn get_hex_coords() -> Vec<HexCoord> {
    // 7-hex honeycomb: axial coordinates (q, r) where |q + r| <= 1
    // Returns: [(-1,0), (-1,1), (0,-1), (0,0), (0,1), (1,-1), (1,0)]
}
```

- [ ] **Step 2: Implement axial-to-pixel conversion**

```rust
fn axial_to_pixel(q: i32, r: i32, size: f64) -> (f64, f64) {
    let dx = size * 1.5;
    let dy = (3.0_f64).sqrt() * size;
    (dx * q as f64, dy * (r as f64 + q as f64 / 2.0))
}
```

- [ ] **Step 3: Implement `hex_map_svg()`**

Generate a full SVG hex grid server-side:

```rust
fn hex_map_svg(areas: &[Area], tributes: &[Tribute], hex_size: f64) -> maud::Markup {
    // 1. Calculate bounding box from all 7 hexes
    // 2. Generate SVG with viewBox
    // 3. For each hex:
    //    a. Generate polygon points (flat-top hexagon, 6 corners)
    //    b. Fill with terrain color (from area terrain type)
    //    c. Add terrain label text (multi-line, centered)
    // 4. Place tribute dots:
    //    a. Group by alliance, assign each alliance to a hex
    //    b. Position dots within hex with small offset per member
    //    c. Color by faction, stroke by alive/dead
    //    d. Add hover labels (done via JS or SVG title elements)
    // 5. Add resource markers on 3 hexes
}
```

Hex corners (flat-top):
```
for i in 0..6:
    angle = (PI / 3) * i
    x = cx + size * cos(angle)
    y = cy + size * sin(angle)
```

- [ ] **Step 4: Parse area data into terrain types**

Map game areas to the design's 7 terrain types (arena, ruins, forest, plains, mountain, lake, swamp). Each area's `.name` or `.terrain` field determines the hex color and label.

If fewer than 7 areas exist, fill remaining hexes with unused terrain types. If more than 7, assign the 7 closest to the center.

Zone names map from terrain types:
```
arena → "THE ARENA", ruins → "ANCIENT RUINS", forest → "DARK FOREST",
plains → "OPEN PLAINS", mountain → "CRAG MOUNTAIN", lake → "MIRROR LAKE",
swamp → "FEN SWAMP"
```

- [ ] **Step 5: Implement `map_section()`**

Renders:
- Section header: "ARENA SURVEILLANCE" title + filter buttons (PLAYERS / RESOURCES / TERRAIN)
- Map container (`#mapContainer`) with:
  - SVG hex grid
  - Zoom controls (+ / - / reset buttons)
  - Map legend (player dot + resource dot)

- [ ] **Step 6: Add map interaction JS to `broadcast.js`**

- `setupMapInteraction()` — mousedown drag pan, wheel zoom, touch support
- `zoomMap(factor)` — scale transform around cursor center
- `resetMap()` — reset to 1.0 scale, centered
- `filterMap(type, btn)` — toggle visibility of players/resources/terrain labels by data attribute

- [ ] **Step 7: Verify map renders** — Load a game, confirm 7 hexes with correct terrain colors, tribute dots positioned on hexes, zoom/pan works

**Done when:** 7-hex honeycomb renders with terrain colors and zone labels, tribute dots appear on hexes with faction colors, zoom and pan interactions work in browser.

---

## Task 7: JavaScript Interactivity (2-3 hrs)

**Why seventh:** JS ties all components together with client-side behavior. It depends on all template components being in place so the DOM structure is final.

**File:** `api/assets/src/broadcast.js` (can be created incrementally across Tasks 4-6)

- [ ] **Step 1: Feed tab switching**

```javascript
function filterFeed(type, btn) {
  document.querySelectorAll('.feed-tab').forEach(t => t.classList.remove('active'));
  btn.classList.add('active');
  document.querySelectorAll('.event-card').forEach(c => {
    c.style.display = (type === 'all' || c.dataset.type === type) ? '' : 'none';
  });
}
```

Feed tabs: ALL, ACTION (`data-type="action"`), DEATHS (`data-type="death"`), EVENTS (`data-type="event"`), COMMS (`data-type="commentary"`).

- [ ] **Step 2: Day navigation**

```javascript
let currentDay = N;    // Set from server-rendered data attribute
let currentPhase = 0;  // Index into ['DAWN','DAY','DUSK','NIGHT']

function prevDay() { /* decrement, update display, trigger HTMX reload */ }
function nextDay() { /* increment, update display, trigger HTMX reload */ }
function jumpToDay(sel) { /* jump to selected day, update display */ }
function advancePhase() { /* cycle phase, update indicator class */ }
function updateDayDisplay() { /* sync label, select, ticker text */ }
```

Day navigation for previous days triggers an HTMX GET to load that day's archived events. The advance phase button is only functional for the game owner and triggers an HTMX PUT to advance the game state.

- [ ] **Step 3: Countdown clock**

```javascript
function startClock() {
  let seconds = 60 + Math.floor(Math.random() * 120);
  setInterval(() => {
    seconds--;
    if (seconds <= 0) seconds = 60 + Math.floor(Math.random() * 120);
    // Update clock span with MM:SS
  }, 1000);
}
```

- [ ] **Step 4: Wire SSE updates**

The broadcast page connects via `hx-ext="sse"` and `sse-connect`. Listen for new events and update:
- Feed scroll: prepend new event cards (most recent first)
- Alive/fallen counts: update stat spans
- Ticker text: rotate to newest headline
- Roster: update health bars and statuses (or full swap the roster section)

SSE event names match current event types (from `shared::messages::MessagePayload` variants — see the sse_events string in `game_detail.rs`).

- [ ] **Step 5: Map zoom/pan (from Task 6)**

Ensure all map interaction functions are defined:
- `setupMapInteraction()` — drag, wheel zoom, touch pinch-zoom
- `zoomMap(factor)`, `resetMap()`, `updateMapTransform()`
- `filterMap(type, btn)` — toggle filters

- [ ] **Step 6: Roster filtering** (stretch)

Optional: add text search or status filter to the roster section header. Filter tributes by name or status (alive/dead) using a simple JS filter.

- [ ] **Step 7: Build `broadcast.js` to production output**

If using a build step (check if `api/assets/src/` files are compiled to `api/assets/dist/`), ensure broadcast.js is included. Otherwise serve it as a static file.

Check `api/build.rs` and `api/assets/` structure for how static assets are handled.

- [ ] **Step 8: Add initializer call at broadcast page foot**

```html
<script>
  document.addEventListener('DOMContentLoaded', function() {
    startClock();
    setupMapInteraction();
    // Set currentDay from server-rendered data attribute
    currentDay = parseInt(document.getElementById('dayLabel').textContent);
  });
</script>
```

**Done when:** Feed tabs filter cards, day nav cycles days (visual only for archived days), clock counts down, map zoom/pan works in browser, SSE updates arrive and update the feed.

---

## Task 8: Wire Routes and Handlers (1-2 hrs)

**Why last:** Route wiring is the integration layer. All components exist; now we connect them to real data.

**Files:**
- Modify: `api/src/routes/games.rs`
- Modify: `api/src/games/handlers.rs` (if needed to expose data)

- [ ] **Step 1: Update `game_detail_handler` to render broadcast page**

Replace `game_detail::game_detail_page(auth, &game)` with `broadcast::broadcast_page(auth, &game, ...)`.

The handler needs to fetch:
- The game's DisplayGame (already fetched)
- Tributes list (from fn::get_tributes_by_game)
- Areas list (from fn::get_areas_by_game)
- Recent events / game messages (from the game's messages or a separate query)
- Commentary segments (from commentary_segments table)
- Current phase string (from game state)

```rust
pub async fn game_detail_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(game_identifier): Path<uuid::Uuid>,
) -> Response {
    let (auth, csrf) = extract_auth(&headers);
    let identifier = game_identifier.to_string();

    // Fetch game, tributes, areas, messages, commentary in parallel
    let game_fut = state.db.query("SELECT * FROM fn::get_display_game($identifier)")
        .bind(("identifier", identifier.clone()))
        .await;
    // ... similar for tributes, areas, messages, commentary

    match (game_opt, tributes, areas, messages) {
        (Some(game), Ok(tributes), Ok(areas), Ok(events)) => {
            html_with_csrf(
                broadcast::broadcast_page(auth, &game, &tributes, &areas, &events, &[], "Dawn", game.day.unwrap_or(1)),
                &csrf,
            )
        }
        _ => not_found_page(...),
    }
}
```

- [ ] **Step 2: Remove old 3-tab routes or keep for backward compat**

The old `/games/{id}/tributes`, `/games/{id}/areas`, `/games/{id}/log` routes can remain as standalone pages (linked from game list cards). The broadcast page does NOT use them, but they're still accessible for fallback.

The `game_detail.rs` file can be deleted after confirming nothing else imports it.

- [ ] **Step 3: Run `just quality` — full workspace check**

Fix any compilation errors, clippy warnings, or test failures.

- [ ] **Step 4: Manual browser verification**

1. Start dev: `just dev`
2. Create a game and start it
3. Navigate to the game detail page
4. Verify: broadcast header shows correct data, day nav works, ticker shows live dot, 50/50 grid renders, map has 7 hexes, roster shows tributes, feed shows event cards
5. Verify: feed tabs filter cards, day nav cycles, map zooms/pans
6. Verify: play a day and see SSE updates arrive in the feed

**Done when:** Full broadcast page renders with real data from all game tables, all interactions work in browser, no regressions in existing pages (home, games list, auth).

---

## Future Work (PR2+)

Items deferred from this PR:

- **Broadcast-specific websocket/SSE delivery** — The current SSE connection pushes raw GameMessages. PR2 could reformat them into broadcast-style event cards server-side before pushing.
- **Sound effects / audio cues** — The broadcast aesthetic could include arena ambient sounds or event-triggered audio.
- **Sponsor ticker rotation** — The ticker text should cycle through recent events, sponsor messages, and arena updates.
- **Full day archive navigation** — Jumping to a previous day should reload all broadcast components (map state, roster at that day, events for that day) via HTMX.
- **Hex map animation** — Tribute dots could animate between hexes when tributes move.
- **Responsive mobile layout** — The current responsive breakpoints collapse to single column but don't adapt for very small screens (tap targets, font sizes).
- **Tribute detail modal** — Clicking a tribute row or map dot opens a modal with full tribute stats, items, afflictions, and event history.

---

## Completed
- Phase 1 (bd-hda9): 510 lines of broadcast CSS tokens + classes. Closed.
- Phase 2 (bd-uh2f): 50/50 grid layout, handler fetches tributes/messages, tributes_page updated. Closed.
- Phase 3 (bd-opwm): Dynamic phase badge, live ticker, all-day select from message data, SSE updates. Closed.
- Phase 4 (bd-mjoh): Event cards restyled into 4 archetypes (ACTION/DEATH/EVENT/ANALYSIS) with color-coded borders. Closed.
