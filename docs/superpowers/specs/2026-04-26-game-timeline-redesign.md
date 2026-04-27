# Game Timeline Redesign

**Status:** Draft (revised post-review)
**Date:** 2026-04-26
**Owner:** kennethlove

## 1. Goals & Non-Goals

### Goals

- Replace the long, unstyled `<ul>` text list (`game_day_log.rs`) with a structured, navigable timeline.
- Give each day/phase a dedicated URL and view (`/games/:identifier/day/:day/:phase`).
- Turn `GamePage` (`/games/:identifier`) into a hub: a card grid where each card is one period (Day N + Day/Night).
- Restructure `GameMessage` so events carry typed, structured payloads — enabling per-kind styling, filtering, and (later) richer interactions.
- Guarantee causally correct ordering of events within a period (no "Peeta moves" after "Peeta dies").
- Add a finished-game recap card (winner, day count, duration) above the grid when the game is over.

### Non-Goals (Out of Scope)

The following are explicitly deferred. Each becomes a follow-up beads issue at close-out:

1. **Combat mechanics redesign.** Only refactor *how* combat events are emitted, not how combat resolves.
2. **Mobile polish.** Desktop-first; basic Tailwind responsive only. No swipe nav, touch-target tuning, or mobile-specific layouts.
3. **Inline announcer commentary cards.** `game_day_summary.rs` is deleted; no replacement in this redesign. Future direction is `MessagePayload::AnnouncerCommentary { speaker, text }` interleaved into the timeline.
4. **Per-tribute timeline filter.** Filter chips in this design are by event *category* only.
5. **Sponsor mechanics UI**, real-time WebSocket push, replay/scrubber controls, search, export/share — all out.
6. **Frontend tests.** No web-crate test infra exists; staying out of scope.

## 2. Schema (location: `shared/src/messages.rs`)

> **Crate move (HIGH-1):** `GameMessage`, `MessagePayload`, `MessageKind`, `Phase`, `CombatEngagement`, `CombatOutcome`, and the supporting `*Ref` structs move from `game/src/messages.rs` to `shared/src/messages.rs`. The `game/` crate already depends on `shared/` (per the existing `GameEvent` precedent). `TaggedEvent` stays in `game/` as an internal collection type. The narrative helper functions (`movement_narrative`, `hiding_spot_narrative`, `stamina_narrative`, `terrain_name`) stay in `game/`.

### Changes to `GameMessage`

```rust
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub phase: Phase,                 // NEW
    pub tick: u32,                    // NEW
    pub emit_index: u32,              // NEW (HIGH-4: persisted, not implicit)
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,              // KEEP — human-readable line
    pub payload: MessagePayload,      // REPLACES `kind: Option<MessageKind>`
}
```

> **No `#[serde(default)]` on `payload`** (see CRITICAL-1). Missing/unknown payload tags hard-error during deserialization. This is intentional: dev DB will be wiped (see §6).

### Reference structs (CRITICAL-2)

Tribute / area / item references in payloads are typed structs, not bare strings, so the frontend can build links from `identifier`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TributeRef { pub identifier: String, pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AreaRef { pub identifier: String, pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemRef { pub identifier: String, pub name: String }
```

These are denormalized: `name` is captured at emit time, so renames after the fact don't retroactively rewrite history. `identifier` is the routing key.

### `Phase`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase { Day, Night }
```

`FromStr` and `Display` impls for URL parsing/rendering. `FromStr` accepts `"day"` and `"night"` only (rejects mixed case, whitespace, etc.). No `Default` impl — phase must always be supplied at construction.

### Tick assignment (HIGH-3: explicit table)

The simulator owns a `TickCounter` on `Game` (or per-cycle context). `next_tick()` is called exactly once per **atomic narrative unit**:

| Source | When `next_tick()` fires |
|---|---|
| Tribute action dispatched | Once, before action runs. All `TaggedEvent`s emitted by that action — including knock-on deaths in `lifecycle.rs` triggered by `attacks()` — share that tick. |
| Area event (closure, hazard) | Once per area-event scheduling pass. All events scheduled in one pass share that tick. |
| Sponsor gift | Once per gift dispatch. |
| Phase-boundary side effects (e.g., area closures at night-start) emitted *as messages* | One tick at the very start of the new phase (`tick = 0`), shared across all boundary messages. |
| `GameEvent::DayStarted` / `NightStarted` | These stay in `GameEvent`, are NOT `GameMessage`s, and have no tick. |

`TickCounter` resets to `0` at every phase boundary. First action in a phase gets `tick = 1`. Phase-boundary side-effect messages get `tick = 0`.

> **Note:** No phase-boundary side-effect emit sites exist today. The `tick = 0` slot is reserved for future area-closure-at-phase-start work and is documented here so the sort key is unambiguous when that work lands.

Combined sort key for in-period rendering:

```
(game_day, phase, tick, emit_index)
```

`emit_index` is the position of the message within the per-period emit sequence, persisted as a real field on `GameMessage` (HIGH-4). Filling it: the drain site in `api/src/games.rs` increments a per-period counter as it builds `GameMessage`s from `Vec<TaggedEvent>` and assigns `emit_index` accordingly. No `ORDER BY id ASC` reliance.

### `MessagePayload`

Typed enum, ~22 variants in 6 categories. **No `Other` variant; no `#[serde(other)]`** (CRITICAL-1). Unknown tags hard-error.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    // Lifecycle
    TributeKilled  { victim: TributeRef, killer: Option<TributeRef>, cause: String },
    TributeWounded { victim: TributeRef, attacker: Option<TributeRef>, hp_lost: u32 },

    // Combat
    Combat(CombatEngagement),

    // Alliance
    AllianceFormed     { members: Vec<TributeRef> },
    AllianceProposed   { proposer: TributeRef, target: TributeRef },
    AllianceDissolved  { members: Vec<TributeRef>, reason: String },
    BetrayalTriggered  { betrayer: TributeRef, victim: TributeRef },
    TrustShockBreak    { tribute: TributeRef, partner: TributeRef },

    // Movement / Area
    TributeMoved   { tribute: TributeRef, from: AreaRef, to: AreaRef },
    TributeHidden  { tribute: TributeRef, area: AreaRef },
    AreaClosed     { area: AreaRef },
    AreaEvent      { area: AreaRef, kind: AreaEventKind, description: String },

    // Items
    ItemFound    { tribute: TributeRef, item: ItemRef, area: AreaRef },
    ItemUsed     { tribute: TributeRef, item: ItemRef },
    ItemDropped  { tribute: TributeRef, item: ItemRef, area: AreaRef },
    SponsorGift  { recipient: TributeRef, item: ItemRef, donor: String },

    // Tribute state
    TributeRested      { tribute: TributeRef, hp_restored: u32 },
    TributeStarved     { tribute: TributeRef, hp_lost: u32 },
    TributeDehydrated  { tribute: TributeRef, hp_lost: u32 },
    SanityBreak        { tribute: TributeRef },
}
```

`AreaEvent.kind` is typed (LOW-4):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaEventKind {
    Hazard, Storm, Mutts, Earthquake, Flood, Fire, Other,
}
```

(`Other` here is OK — this is a small enum of known categories, not a deserialize-fallback.)

`SponsorGift` (LOW-3): there is currently no emit site for sponsor gifts. The variant is included for forward compatibility but is **not emitted in PR1**. If we decide it adds friction, drop it; otherwise keep dormant.

### `MessageSource`

`MessageSource` keeps its existing shape (`Game(String)`, `Area(String)`, `Tribute(String)` where the `String` is the source's identifier). Several payloads denormalize tribute identity from `source` into payload fields (e.g., `TributeMoved { tribute, .. }` while `source = MessageSource::Tribute(tribute_id)`). This is intentional (LOW-5): payloads are self-contained for rendering without joining back to `source`. Card components prefer payload fields.

### `CombatEngagement`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEngagement {
    pub attacker: TributeRef,
    pub target: TributeRef,
    pub outcome: CombatOutcome,
    pub detail_lines: Vec<String>,    // per-swing prose, shown when card expanded
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatOutcome {
    Killed, Wounded, TargetFled, AttackerFled, Stalemate,
}
```

`detail_lines` may be empty (e.g., `TargetFled` before any swing). UI handles this (see §5).

One `attacks()` call → one `MessagePayload::Combat(...)` message. Death (if any) is still emitted separately as `TributeKilled` from the lifecycle code, sharing the attacker's tick (per the table above). Result: a killing combat shows two cards (combat detail card + death announcement card) at the same tick. Intentional (MEDIUM-2). `event_count` in `PeriodSummary` counts both; `deaths` counts only `TributeKilled`.

### `MessageKind` (for filter chips)

Small enum, derived from payload via `.kind()`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind { Death, Combat, Alliance, Movement, Item, State }

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        use MessagePayload::*;
        match self {
            TributeKilled { .. } => MessageKind::Death,
            Combat(_) => MessageKind::Combat,
            AllianceFormed { .. } | AllianceProposed { .. } | AllianceDissolved { .. }
                | BetrayalTriggered { .. } | TrustShockBreak { .. } => MessageKind::Alliance,
            TributeMoved { .. } | TributeHidden { .. }
                | AreaClosed { .. } | AreaEvent { .. } => MessageKind::Movement,
            ItemFound { .. } | ItemUsed { .. }
                | ItemDropped { .. } | SponsorGift { .. } => MessageKind::Item,
            TributeWounded { .. } | TributeRested { .. } | TributeStarved { .. }
                | TributeDehydrated { .. } | SanityBreak { .. } => MessageKind::State,
        }
    }
}
```

No `Other` variant (since `MessagePayload` has no `Other`). Single source of truth: payload. No separate `kind` field on `GameMessage`.

### Phase markers

`GameEvent::DayStarted { day }` and `GameEvent::NightStarted { day }` stay in `GameEvent`, not `MessagePayload`. They are emitted at phase boundaries and used for debugging / future use, but do not appear as cards in the timeline.

### Constructor

```rust
impl GameMessage {
    pub fn new(
        source: MessageSource,
        game_day: u32,
        phase: Phase,
        tick: u32,
        emit_index: u32,
        subject: String,
        content: String,
        payload: MessagePayload,
    ) -> Self { /* ... */ }
}
```

`with_kind` is removed. There is no migration path: dev DB is wiped (§6). Unknown payload tags from any persisted legacy data fail to deserialize, surfacing the wipe-required precondition immediately rather than silently producing `Other`.

## 3. Combat Refactor

### Problem

Today, `Tribute::attacks()` in `game/src/tributes/combat.rs` pushes 3-6 `String` lines into `events: &mut Vec<String>`. The drain site at `api/src/games.rs` ~L957 turns each line into a `GameMessage` with `kind=None`. We can't tag combat messages at the drain site because the drain only sees strings.

### Approach: tag at emit

Introduce `TaggedEvent` in `game/src/messages.rs` (or a new `game/src/events.rs`):

```rust
pub struct TaggedEvent {
    pub content: String,
    pub payload: MessagePayload,
}
```

Thread `&mut Vec<TaggedEvent>` (replacing `&mut Vec<String>`) through these known emit sites (LOW-2):

- `game/src/tributes/combat.rs::attacks` and its per-swing helpers
- `game/src/tributes/mod.rs::do_day_action` / `do_night_action`
- `game/src/tributes/lifecycle.rs::take_damage` / death emission helpers
- `game/src/tributes/movement.rs::move_to_area` and movement helpers
- `game/src/tributes/state.rs` (rest, starve, dehydrate, sanity-break helpers)
- `game/src/areas/*` area-event / area-closure emit sites
- `game/src/alliances/*` alliance-lifecycle emit sites

Any further sites the compiler flags during the change get the same treatment.

### `attacks()` after refactor

During the per-swing loop, accumulate prose into a local `detail_lines: Vec<String>` and track outcome. At the end, push **one** `TaggedEvent`:

```rust
let engagement = CombatEngagement {
    attacker: TributeRef { identifier: self.identifier.clone(), name: self.name.clone() },
    target:   TributeRef { identifier: target.identifier.clone(), name: target.name.clone() },
    outcome,
    detail_lines,
};
events.push(TaggedEvent {
    content: summary_line(&engagement),
    payload: MessagePayload::Combat(engagement),
});
```

Death stays a separate `TaggedEvent` emitted from `lifecycle.rs` with `MessagePayload::TributeKilled { victim, killer, cause }`. Both events share the attacker's tick.

### Drain site

`api/src/games.rs` ~L957 takes `Vec<TaggedEvent>` and builds `GameMessage`s, attaching `(game_day, phase, tick, emit_index)` from the cycle context. `emit_index` increments per message within the period. `log_output_kind` is removed; `log_output` is removed entirely (no more untagged emissions).

### Test impact

Existing combat rstest tests in the game crate need updating to assert against payload variants — typically `assert!(matches!(msg.payload, MessagePayload::Combat(CombatEngagement { outcome: CombatOutcome::Killed, .. })))` and length checks on `detail_lines`.

## 4. Routes & API

### Routes (`web/src/routes.rs`)

```rust
#[layout(Navbar)]
    #[route("/")]                                                       Home {},
    #[route("/games/")]                                                 GamesList {},
    #[route("/games/:identifier")]                                      GamePage { identifier: String },
    #[route("/games/:identifier/day/:day/:phase")]                      GamePeriodPage { identifier: String, day: u32, phase: Phase },  // NEW — typed Phase param (MEDIUM-4)
    #[route("/games/:game_identifier/tributes/:tribute_identifier")]    TributeDetail { /* ... */ },
    /* ... unchanged ... */
#[end_layout]
```

`phase: Phase` uses Dioxus router's `FromStr`-based typed param: invalid values fail to match the route and the router falls through to `PageNotFound`. No string-parsing in the component.

### API endpoints

- **`GET /api/games/:id/log/:day`** — KEEP & EXTEND. Still returns `Vec<GameMessage>` for the entire day, now with `phase`, `tick`, `emit_index`, and `payload` fields populated. The period view filters client-side by phase.

- **`GET /api/games/:id/timeline-summary`** — NEW. Returns:

  ```rust
  Vec<PeriodSummary {
      day: u32,
      phase: Phase,
      deaths: u32,
      event_count: u32,
      is_current: bool,    // MEDIUM-6: derived from game.current_day + current_phase
  }>
  ```

  Implemented in `shared/src/messages.rs` (MEDIUM-1) as a pure function:

  ```rust
  pub fn summarize_periods(messages: &[GameMessage], current: (u32, Phase)) -> Vec<PeriodSummary> { ... }
  ```

  The API handler loads all messages for the game (already done elsewhere), calls `summarize_periods`, and serializes. No SurrealDB `GROUP BY` — Rust aggregation keeps the wire format decoupled from DB schema as `MessagePayload` variants evolve. Empty periods (no messages) still appear in the result if the game has reached or passed that period (sourced from `game.current_day`); periods past `current_day` do not appear.

  Auth: public read, mirroring `/log/:day` (MEDIUM-10).

- **`GET /api/games/:id/summarize/:day`** — orphaned by this redesign (used only by the deleted `game_day_summary.rs`). **A beads issue is filed at PR-open time** (MEDIUM-11) to delete the endpoint and its handler in a follow-up. Not deleted in this PR to keep diff scope tight.

- **Recap data** (winner, day count, duration) is already on `DisplayGame`. No new endpoint needed. Today `DisplayGame.winner` is a string (tribute name); RecapCard renders it as plain text. A follow-up beads issue (§7) tracks upgrading `winner` to `Option<TributeRef>` so the name can become a link.

### Cache invalidation (HIGH-5)

dioxus-query cache keys for this redesign:

```rust
pub enum QueryKey {
    GameDayLog(String /*game_id*/, u32 /*day*/, u32 /*generation*/),
    TimelineSummary(String /*game_id*/, u32 /*generation*/),
    /* existing variants */
}
```

A per-game `generation: u32` signal is held in the `PeriodFilters` context (or a sibling `GameCache` context — see §5). It is bumped on `MutationValue::GameAdvanced` (existing handler in `game_detail.rs`).

WebSocket-driven invalidation (bumping `generation` when the WS hook reports `GameEvent::DayStarted` / `NightStarted`) is **out of scope for PR2**. A follow-up beads issue (§7) tracks adding it once the hook surface is verified. Mutation-driven invalidation is sufficient for the user-driven "advance day" flow that exists today.

All readers of `GameDayLog` / `TimelineSummary` interpolate the current generation into their key, so a bump invalidates everything game-scoped without per-key wildcard tracking. Past-day logs are still re-fetched on bump but their content is immutable, so the network cost is bounded and acceptable.

## 5. Frontend Components

### Component tree

```
components/
  game_detail.rs                    (modified — Game info + RecapCard + PeriodGrid)
  recap_card.rs                     (new)
  period_grid.rs                    (new)
  period_grid_empty.rs              (new — empty/error states for the grid; MEDIUM-5)
  period_card.rs                    (new)
  game_period_page.rs               (new — top-level for GamePeriodPage route)
  filter_chips.rs                   (new)
  timeline/
    mod.rs                          (re-exports)
    timeline.rs                     (new — list + empty state)
    event_card.rs                   (new — thin dispatcher, ~30 lines, matches kind, delegates)
    cards/
      death_card.rs                 (new — TributeKilled)
      combat_card.rs                (new — Combat, with local expand Signal)
      alliance_card.rs              (new — 5 alliance variants)
      movement_card.rs              (new — 4 movement/area variants)
      item_card.rs                  (new — 4 item variants)
      state_card.rs                 (new — 5 state variants incl. TributeWounded)
```

(HIGH-8: `event_card.rs` is split.)

### Component behaviors

- **`game_period_page.rs`** — top-level for the `GamePeriodPage` route. Reads `(identifier, day, phase)`. Validates `(day, phase)` against the cached `TimelineSummary` (M5/MEDIUM-5): if the period doesn't exist, renders `PageNotFound`. Otherwise fetches `/api/games/:id/log/:day` via dioxus-query (with current generation), filters returned `Vec<GameMessage>` by `phase`, and renders `<FilterChips />` then `<Timeline />`.

- **`period_card.rs`** — single hub card. Props: `{ game_identifier: String, day: u32, phase: Phase, deaths: u32, event_count: u32, is_current: bool }`. Click navigates to `GamePeriodPage`. `is_current` styles the live period subtly (MEDIUM-6). Tailwind styling matches the existing 3-theme color scheme.

- **`period_grid.rs`** — props: `{ game_identifier: String }`. Fetches `/api/games/:id/timeline-summary`. Renders Tailwind `grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4` of `<PeriodCard />`. Loading state: skeleton placeholder. Error state and empty state delegated to `period_grid_empty.rs`.

- **`period_grid_empty.rs`** — renders one of three states:
  - **Empty** (game exists but has not been simulated yet): "This game hasn't started yet. Click 'Begin' to start." (or matching existing copy from `game_detail.rs` actions).
  - **Loading failure** (HTTP error from `/timeline-summary`): "Couldn't load the timeline." with a retry button that re-runs the query.
  - **Game not yet found** (404 from `/timeline-summary`): defer to parent route — should not happen if reached from a valid `GamePage` instance.

- **`recap_card.rs`** — props: `{ game: DisplayGame }` (MEDIUM-7). Collapsible card pinned above the grid when `game.status == Finished`. Shows winner (or "All tributes died" when `winner.is_none()`), day count, and duration. Collapsed state persisted in `gloo-storage` keyed `recap_collapsed:{game.identifier}`. **Default: expanded on first view** (LOW-7). Winner name is plain text (not a link) for consistency with existing `DisplayGame.winner` shape; if/when `winner` becomes a `TributeRef`, render as link.

- **`filter_chips.rs`** — props: `{ game_identifier: String }`. Reads/writes `PeriodFilters` from context (see State below). Chip row: `[All] [Deaths] [Combat] [Alliances] [Movement] [Items]`. `State` events (TributeWounded/Rested/Starved/Dehydrated/SanityBreak) are always visible when `All` is active and never filtered out by category chips (MEDIUM-8). State of the chip row is rendered from the explicit `FilterMode` enum (HIGH-7):

  ```rust
  pub enum FilterMode {
      All,
      Subset(HashSet<MessageKind>),
  }

  impl FilterMode {
      pub fn matches(&self, kind: MessageKind) -> bool {
          match self {
              FilterMode::All => true,
              FilterMode::Subset(set) => set.contains(&kind) || kind == MessageKind::State,
          }
      }
      pub fn is_all(&self) -> bool { matches!(self, FilterMode::All) }
  }
  ```

  Chip click rules:
  - Click `[All]` → `FilterMode::All`. (No-op if already All.)
  - Click a category while `All` → `FilterMode::Subset({that category})`.
  - Click a category in a Subset → toggle membership.
  - Toggling off the last category in a Subset → snap to `FilterMode::All` and visually flip the `All` chip on (so the transition is explicit, not silent — addresses HIGH-2 UX trap).

  > **No `[State]` chip.** State events (TributeWounded/Rested/Starved/Dehydrated/SanityBreak) are treated as ambient context and always rendered, in both `All` and `Subset` modes. They have no chip representation; this is intentional.

- **`timeline/event_card.rs`** — thin dispatcher (~30 lines). Matches on `payload.kind()` and renders the corresponding card subcomponent from `cards/`. Passes the full `GameMessage` (or relevant payload variant) as a prop.

- **`timeline/cards/*.rs`** — one file per `MessageKind`. Owns its icon, Tailwind classes, and any local UI state. `combat_card.rs` owns a local `use_signal(|| false)` for expand/collapse. If `engagement.detail_lines.is_empty()`, the expand affordance is hidden (MEDIUM-1). Tribute names render as plain links to `/games/:gid/tributes/:tribute_identifier` using `TributeRef.identifier`. Item / area names use their refs' `identifier` similarly (no item/area route today, so render as plain text with `title=identifier` for now; future link work is out of scope).

- **`timeline/timeline.rs`** — vertical list of `<EventCard />`s, sorted by `(tick, emit_index)`, filtered by the active `FilterMode`. Empty state: "Nothing happened this period."

### Modified

- **`game_detail.rs`** — strip the day-by-day rendering (~lines 220-630). Keep header, tribute roster, and map as a "Game info" section above the new `<RecapCard />` (when finished) and `<PeriodGrid />`. Pass the already-fetched `DisplayGame` into both `RecapCard` and `PeriodGrid` props (MEDIUM-7) — no double-fetch even though dioxus-query would dedupe by key.

  **Mutation handlers in `game_detail.rs`:**
  - On `MutationValue::GameAdvanced` — bump per-game `generation` in `PeriodFilters` context (HIGH-5).
  - On `MutationValue::GameDeleted(id, _)` — call `gloo_storage::LocalStorage::delete("recap_collapsed:{id}")` and `gloo_storage::LocalStorage::delete("period_filters:{id}")` (MEDIUM-3).

### Deleted

- `web/src/components/game_day_log.rs`
- `web/src/components/game_day_summary.rs`

### State (HIGH-6: scope resolved)

New context signal, **provided at the `Navbar` layout layer** (top-level under the router). Per-game state is keyed inside the struct, not by component scope:

```rust
pub struct PeriodFilters {
    pub by_game: HashMap<String /*game_id*/, FilterMode>,
    pub generations: HashMap<String /*game_id*/, u32>,
}

impl PeriodFilters {
    pub fn filter_for(&self, game_id: &str) -> FilterMode { /* default: All */ }
    pub fn set_filter(&mut self, game_id: &str, mode: FilterMode) { /* + persist to gloo-storage */ }
    pub fn generation(&self, game_id: &str) -> u32 { /* default: 0 */ }
    pub fn bump(&mut self, game_id: &str) { /* +=1 */ }
}
```

Per-game `FilterMode` is persisted to `gloo-storage` keyed `period_filters:{game_id}`, mirrored into the in-memory `HashMap`. On boot, no preload is needed — entries are lazily hydrated when first read for a given `game_id`. Generations are in-memory only (reset to 0 on full reload, which correctly invalidates everything).

**Filter state is NOT in URL query params** (LOW-1): keeps URLs clean for sharing the period view itself (`/games/:id/day/3/night`), and back/forward already navigate by period URL. Filter state is per-user, per-game, and persists across reloads via gloo-storage.

## 6. Testing & Rollout

### Game crate

- Update existing combat rstest tests to assert `MessagePayload::Combat` variants — match on `outcome` and `detail_lines.len()`.
- Add `MessagePayload::kind()` mapping tests covering every variant.
- Add ordering test for the `(game_day, phase, tick, emit_index)` sort key.
- Add `Phase` enum tests: `FromStr` accepts `"day"` / `"night"` and rejects `"Day"`, `"sideways"`, `""`, whitespace; `Display` round-trip; serde round-trip.
- Add a causal-ordering regression test (LOW-8): construct a scenario where Tribute A kills Tribute B, then assert that no `MessagePayload::TributeMoved { tribute: B, .. }` appears at a later `(tick, emit_index)` than the `TributeKilled { victim: B, .. }` for that period.
- Add `CombatOutcome` coverage: at least one test per variant (`Killed`, `Wounded`, `TargetFled`, `AttackerFled`, `Stalemate`).
- Add `summarize_periods` tests in shared crate: empty input → empty output; mixed days/phases; deaths counted only from `TributeKilled`; current period flagged correctly.
- Unknown-tag round-trip test: serialize a synthetic JSON object with `"type": "FutureKind"` and assert `serde_json::from_str::<MessagePayload>` returns `Err` (confirming hard-error behavior, no silent `Other`).

### API crate

- Update `api/tests/games_tests.rs` for the new `/log/:day` shape (`phase`, `tick`, `emit_index`, `payload`).
- New integration test for `GET /api/games/:id/timeline-summary`:
  - Empty for a game with `current_day == 0` / not started.
  - Returns Day+Night entries for each completed period.
  - Period with zero messages but reached (e.g., `current_day=2` with empty Day 1 Night) still appears with `event_count=0`.
  - `is_current` flag matches `game.current_day` + `current_phase`.
  - 404 (or empty, depending on existing convention) for unknown game id.
- Update existing alliance-lifecycle tests: change assertions from `kind: Some(MessageKind::AllianceFormed)` to matching `MessagePayload::AllianceFormed { members }`.

### Web crate

No tests. No frontend test infrastructure exists — out of scope.

### Rollout

**Stacked PRs (HIGH-2 mitigation):**

- **PR1 — backend + frontend stub.** Schema move (`game/` → `shared/`), `TaggedEvent` introduced, all known emit sites converted from `&mut Vec<String>` to `&mut Vec<TaggedEvent>` with typed payloads, combat refactor, API extensions (`/log/:day` extension + new `/timeline-summary`). Includes deletion of `game_day_log.rs` and `game_day_summary.rs` from the frontend, replaced with a temporary stub component **`web/src/components/game_log_stub.rs`** used by `game_detail.rs` that renders raw `content` lines from `GameMessage` (no styling, no filtering). This is the minimum change to keep `web/` compiling. The stub is deleted in PR2. The deleted-and-stubbed approach is preferred over keeping a serde-shim `kind` field, because it avoids a double-migration of the schema.

- **PR2 — frontend timeline.** Add new components (`recap_card`, `period_grid`, `period_grid_empty`, `period_card`, `game_period_page`, `filter_chips`, `timeline/*`, `cards/*`). Add new route. Replace the PR1 stub in `game_detail.rs` with `RecapCard` + `PeriodGrid`. Delete `game_log_stub.rs`. Wire up cache invalidation and gloo-storage cleanup.

Hard cutover. No feature flag.

### Dev DB

**Wipe `data/` directory required** before PR1 lands (LOW-4 lifted out of "recommended" into hard requirement). The `#[serde(default)]` fallback path is removed, so unknown payload tags or missing required fields hard-error. Wiping is the only safe path. Documented in PR1 description.

### Verification

- `just fmt` and `just quality` clean on each PR.
- `just dev` smoke test after PR2:
  - Hub renders period card grid; current period visually highlighted.
  - Click card → navigate to period view.
  - Filter chips work: `All` → category subset → empty subset snaps back to All; persists across DayNight nav within a game and across full reload (per game).
  - Combat cards expand to show `detail_lines`; Combat with empty `detail_lines` shows no expand control.
  - Tribute names in cards link to tribute detail page using `TributeRef.identifier`.
  - Recap card appears when game is finished, collapses, persists collapsed state across reload, cleared from localStorage on game delete.
  - Browser back/forward across hub  period works.
  - Advancing a day (existing "next step" action) refreshes timeline summary AND the current day's log without manual reload.

## 7. Follow-up beads issues (filed at PR-open)

1. Combat mechanics redesign ("At some point, let's look at changing how combat works"). Includes modeling per-swing combat as `Vec<CombatBeat>` typed structs instead of `Vec<String> detail_lines`.
2. Mobile polish (touch targets, swipe between periods, mobile-first card layouts).
3. Inline announcer commentary cards (`MessagePayload::AnnouncerCommentary { speaker, text }`).
4. Per-tribute timeline filter (filter chips by `TributeRef.identifier`).
5. Delete `/api/games/:id/summarize/:day` endpoint and handler (orphaned by this redesign).
6. Hover-preview cards for tribute / item / area links in the timeline.
7. Item and area detail routes (currently `ItemRef` and `AreaRef` carry identifiers but no route to link to).
8. Filter state in URL query params (`?filter=combat,deaths`) for shareable timeline views.
9. Confirm or drop `MessagePayload::SponsorGift` once sponsor mechanics are designed.
