# Game Timeline PR2 — Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the PR1 stub on `GamePage` with a structured day/phase timeline UI: a hub of period cards, a per-period view with category filter chips and typed event cards, and a finished-game recap.

**Architecture:** Dioxus 0.7 components compiled to WASM. Period hub fetches `/api/games/:id/timeline-summary`. Per-period view fetches `/api/games/:id/log/:day` and filters by `phase`. Filter state lives in a `PeriodFilters` context provided at the `Navbar` layout layer, persisted per-game to `gloo-storage`. Generation counter (in-memory) bumped by mutation handlers triggers dioxus-query re-fetches. Event card dispatcher routes each `MessagePayload` variant to a typed sub-card.

**Tech Stack:** Dioxus 0.7 + dioxus-router, dioxus-query (already in use), Tailwind CSS (3-theme system), gloo-storage 0.3.0 (already in use), `shared::messages` types from PR1.

**Spec:** `docs/superpowers/specs/2026-04-26-game-timeline-redesign.md` §5 (components), §6 (rollout), §7 (follow-ups).

**Pre-condition:** PR1 is merged. `shared::messages` exposes `GameMessage`, `MessagePayload`, `MessageKind`, `Phase`, `TributeRef`, `AreaRef`, `ItemRef`, `CombatOutcome`, `PeriodSummary`, `TimelineSummary`. The temporary `web/src/components/game_log_stub.rs` exists and is wired into `game_detail.rs`. `web/src/cache.rs` has `MutationValue::GameDeleted(String, String)` already.

---

## Task 1: Add PeriodFilters context type and provider

**Files:**
- Create: `web/src/components/timeline/filters.rs`
- Modify: `web/src/components/timeline/mod.rs` (created in PR1; add `pub mod filters; pub use filters::*;`)
- Modify: `web/src/components/navbar.rs` — provide context inside `Navbar`

- [ ] **Step 1: Define FilterMode and PeriodFilters in `web/src/components/timeline/filters.rs`**

```rust
use shared::messages::MessageKind;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FilterMode {
    All,
    Subset(HashSet<MessageKind>),
}

impl Default for FilterMode {
    fn default() -> Self { FilterMode::All }
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

#[derive(Clone, PartialEq, Default, Debug)]
pub struct PeriodFilters {
    pub by_game: HashMap<String, FilterMode>,
    pub generations: HashMap<String, u32>,
}

impl PeriodFilters {
    pub fn filter_for(&self, game_id: &str) -> FilterMode {
        self.by_game.get(game_id).cloned().unwrap_or_default()
    }

    pub fn set_filter(&mut self, game_id: &str, mode: FilterMode) {
        self.by_game.insert(game_id.to_string(), mode.clone());
        let key = format!("period_filters:{game_id}");
        // best-effort persist; ignore failure
        let _ = gloo_storage::LocalStorage::set(&key, &SerializableFilter::from(&mode));
    }

    pub fn hydrate(&mut self, game_id: &str) {
        if self.by_game.contains_key(game_id) { return; }
        let key = format!("period_filters:{game_id}");
        if let Ok(saved) = gloo_storage::LocalStorage::get::<SerializableFilter>(&key) {
            self.by_game.insert(game_id.to_string(), saved.into());
        }
    }

    pub fn generation(&self, game_id: &str) -> u32 {
        self.generations.get(game_id).copied().unwrap_or(0)
    }

    pub fn bump(&mut self, game_id: &str) {
        let entry = self.generations.entry(game_id.to_string()).or_insert(0);
        *entry += 1;
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableFilter {
    mode: String,
    kinds: Vec<MessageKind>,
}

impl From<&FilterMode> for SerializableFilter {
    fn from(m: &FilterMode) -> Self {
        match m {
            FilterMode::All => Self { mode: "all".into(), kinds: vec![] },
            FilterMode::Subset(s) => Self { mode: "subset".into(), kinds: s.iter().copied().collect() },
        }
    }
}

impl From<SerializableFilter> for FilterMode {
    fn from(s: SerializableFilter) -> Self {
        if s.mode == "subset" {
            FilterMode::Subset(s.kinds.into_iter().collect())
        } else {
            FilterMode::All
        }
    }
}
```

- [ ] **Step 2: Add module exports to `web/src/components/timeline/mod.rs`**

Append:

```rust
pub mod filters;
pub use filters::{FilterMode, PeriodFilters};
```

- [ ] **Step 3: Provide context in `Navbar`**

Modify `web/src/components/navbar.rs`. Inside `pub fn Navbar() -> Element`, near other `use_context_provider` / `provide_context` calls (or right before the `rsx!` block), add:

```rust
use_context_provider(|| Signal::new(crate::components::timeline::PeriodFilters::default()));
```

- [ ] **Step 4: Build to verify it compiles**

Run: `cargo check --package web --target wasm32-unknown-unknown` (or `just web-check` if defined).
Expected: compiles cleanly.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): add PeriodFilters context for timeline UI"
```

---

## Task 2: Add timeline-summary query key and fetcher

**Files:**
- Modify: `web/src/cache.rs` — add `TimelineSummary(String)` key and `TimelineSummary(shared::messages::TimelineSummary)` value variant
- Create: `web/src/hooks/use_timeline_summary.rs`
- Modify: `web/src/hooks/mod.rs` (or wherever hooks are re-exported)

- [ ] **Step 1: Extend `QueryKey` enum in `web/src/cache.rs`**

Add variant:

```rust
TimelineSummary(String), // Game identifier
```

Add to `QueryValue`:

```rust
TimelineSummary(shared::messages::TimelineSummary),
```

- [ ] **Step 2: Create `web/src/hooks/use_timeline_summary.rs`**

```rust
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::env::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::TimelineSummary;

async fn fetch_timeline_summary(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    let Some(QueryKey::TimelineSummary(id)) = keys.first() else {
        return Err(QueryError::Unknown).into();
    };
    let url = format!("{API_HOST}/api/games/{id}/timeline-summary");
    match reqwest::get(&url).await {
        Ok(resp) => match resp.status() {
            StatusCode::OK => match resp.json::<TimelineSummary>().await {
                Ok(s) => Ok(QueryValue::TimelineSummary(s)).into(),
                Err(_) => Err(QueryError::BadJson).into(),
            },
            StatusCode::NOT_FOUND => Err(QueryError::GameNotFound(id.clone())).into(),
            _ => Err(QueryError::Unknown).into(),
        },
        Err(_) => Err(QueryError::ServerNotFound).into(),
    }
}

pub fn use_timeline_summary(game_id: String) -> UseQuery<QueryValue, QueryError, QueryKey> {
    use_get_query([QueryKey::TimelineSummary(game_id)], fetch_timeline_summary)
}
```

- [ ] **Step 3: Wire into `web/src/hooks/mod.rs`**

Add `pub mod use_timeline_summary;` and `pub use use_timeline_summary::use_timeline_summary;`.

- [ ] **Step 4: Verify it compiles**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: clean.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): add use_timeline_summary query hook"
```

---

## Task 3: PeriodCard component

**Files:**
- Create: `web/src/components/period_card.rs`
- Modify: `web/src/components/mod.rs` — `mod period_card; pub use period_card::PeriodCard;`

- [ ] **Step 1: Implement `PeriodCard`**

```rust
use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::Phase;

#[derive(Props, PartialEq, Clone)]
pub struct PeriodCardProps {
    pub game_identifier: String,
    pub day: u32,
    pub phase: Phase,
    pub deaths: u32,
    pub event_count: u32,
    pub is_current: bool,
}

#[component]
pub fn PeriodCard(props: PeriodCardProps) -> Element {
    let phase_label = match props.phase {
        Phase::Day => "Day",
        Phase::Night => "Night",
    };
    let current_class = if props.is_current {
        "ring-2 ring-amber-400 theme2:ring-green-400 theme3:ring-purple-400"
    } else { "" };
    let route = Routes::GamePeriodPage {
        identifier: props.game_identifier.clone(),
        day: props.day,
        phase: props.phase,
    };
    rsx! {
        Link {
            to: route,
            class: "block rounded-lg border p-4 hover:shadow-lg transition theme1:bg-amber-50 theme1:border-amber-200 theme2:bg-slate-800 theme2:border-green-700 theme3:bg-purple-900 theme3:border-purple-600 {current_class}",
            div { class: "flex items-baseline justify-between",
                h3 { class: "text-lg font-semibold", "Day {props.day} — {phase_label}" }
                if props.is_current { span { class: "text-xs uppercase tracking-wide", "live" } }
            }
            div { class: "mt-2 text-sm",
                span { class: "mr-3", "💀 {props.deaths} deaths" }
                span { "📜 {props.event_count} events" }
            }
        }
    }
}
```

- [ ] **Step 2: Register in `mod.rs`**

Add `mod period_card;` and `pub use period_card::PeriodCard;`.

- [ ] **Step 3: Build**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: compiles. Note: `Routes::GamePeriodPage` does not exist yet — this step will FAIL. Defer the `Link { to: route, … }` line by stubbing with `to: Routes::Home {}` temporarily, or skip this build step until Task 8 adds the route.

> **Author note:** Use a placeholder route in Step 1 (`Routes::Home {}`) and fix it up in Task 8 Step 3. Or accept that this task's build only passes after Task 8.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(web): add PeriodCard component"
```

---

## Task 4: PeriodGrid + PeriodGridEmpty components

**Files:**
- Create: `web/src/components/period_grid.rs`
- Create: `web/src/components/period_grid_empty.rs`
- Modify: `web/src/components/mod.rs`

- [ ] **Step 1: Implement `PeriodGridEmpty` in `web/src/components/period_grid_empty.rs`**

```rust
use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum EmptyKind { NotStarted, LoadFailed, NotFound }

#[derive(Props, PartialEq, Clone)]
pub struct PeriodGridEmptyProps {
    pub kind: EmptyKind,
    pub on_retry: Option<EventHandler<()>>,
}

#[component]
pub fn PeriodGridEmpty(props: PeriodGridEmptyProps) -> Element {
    let copy = match props.kind {
        EmptyKind::NotStarted => "This game hasn't started yet. Click Begin to start.",
        EmptyKind::LoadFailed => "Couldn't load the timeline.",
        EmptyKind::NotFound  => "Game not found.",
    };
    rsx! {
        div { class: "rounded-lg border border-dashed p-8 text-center text-sm",
            p { "{copy}" }
            if matches!(props.kind, EmptyKind::LoadFailed) {
                if let Some(retry) = props.on_retry {
                    button {
                        class: "mt-4 rounded bg-amber-500 px-3 py-1 text-amber-50 hover:bg-amber-600",
                        onclick: move |_| retry.call(()),
                        "Retry"
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Implement `PeriodGrid` in `web/src/components/period_grid.rs`**

```rust
use crate::cache::{QueryError, QueryValue};
use crate::components::period_card::PeriodCard;
use crate::components::period_grid_empty::{EmptyKind, PeriodGridEmpty};
use crate::hooks::use_timeline_summary::use_timeline_summary;
use dioxus::prelude::*;
use dioxus_query::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct PeriodGridProps {
    pub game_identifier: String,
}

#[component]
pub fn PeriodGrid(props: PeriodGridProps) -> Element {
    let query = use_timeline_summary(props.game_identifier.clone());
    rsx! {
        match query.result().value() {
            QueryResult::Ok(QueryValue::TimelineSummary(s)) => {
                if s.periods.is_empty() {
                    rsx!{ PeriodGridEmpty { kind: EmptyKind::NotStarted, on_retry: None } }
                } else {
                    rsx!{
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            for p in s.periods.iter() {
                                PeriodCard {
                                    key: "{p.day}-{p.phase:?}",
                                    game_identifier: props.game_identifier.clone(),
                                    day: p.day,
                                    phase: p.phase,
                                    deaths: p.deaths,
                                    event_count: p.event_count,
                                    is_current: p.is_current,
                                }
                            }
                        }
                    }
                }
            }
            QueryResult::Err(_) => rsx!{ PeriodGridEmpty { kind: EmptyKind::LoadFailed, on_retry: None } },
            _ => rsx!{ div { class: "animate-pulse h-32 rounded-lg bg-gray-200" } },
        }
    }
}
```

- [ ] **Step 3: Register in `mod.rs`**

```rust
mod period_grid;
mod period_grid_empty;
pub use period_grid::PeriodGrid;
```

- [ ] **Step 4: Build**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: compiles after Task 8 lands the route. Until then, `PeriodCard` uses placeholder route from Task 3.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): add PeriodGrid + PeriodGridEmpty components"
```

---

## Task 5: RecapCard component

**Files:**
- Create: `web/src/components/recap_card.rs`
- Modify: `web/src/components/mod.rs`

- [ ] **Step 1: Implement `RecapCard`**

```rust
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use shared::DisplayGame;

#[derive(Props, PartialEq, Clone)]
pub struct RecapCardProps { pub game: DisplayGame }

#[component]
pub fn RecapCard(props: RecapCardProps) -> Element {
    let key = format!("recap_collapsed:{}", props.game.identifier);
    let initial: bool = LocalStorage::get(&key).unwrap_or(false);
    let mut collapsed = use_signal(|| initial);

    let toggle = {
        let key = key.clone();
        move |_| {
            let new = !collapsed();
            collapsed.set(new);
            let _ = LocalStorage::set(&key, &new);
        }
    };

    let winner_line = match props.game.winner.as_deref() {
        Some(w) if !w.is_empty() => format!("🏆 Winner: {w}"),
        _ => "All tributes died".to_string(),
    };

    rsx! {
        section { class: "rounded-lg border bg-amber-50 theme2:bg-slate-800 theme3:bg-purple-900 p-4 mb-4",
            header { class: "flex items-center justify-between cursor-pointer", onclick: toggle,
                h2 { class: "text-xl font-semibold", "Game Recap" }
                span { if collapsed() { "▸" } else { "▾" } }
            }
            if !collapsed() {
                div { class: "mt-3 space-y-1 text-sm",
                    p { "{winner_line}" }
                    p { "Days played: {props.game.day}" }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Register in `mod.rs`**

```rust
mod recap_card;
pub use recap_card::RecapCard;
```

- [ ] **Step 3: Build & commit**

```bash
cargo check --package web --target wasm32-unknown-unknown
jj describe -m "feat(web): add RecapCard for finished games"
```

---

## Task 6: FilterChips component

**Files:**
- Create: `web/src/components/filter_chips.rs`
- Modify: `web/src/components/mod.rs`

- [ ] **Step 1: Implement `FilterChips`**

```rust
use crate::components::timeline::{FilterMode, PeriodFilters};
use dioxus::prelude::*;
use shared::messages::MessageKind;
use std::collections::HashSet;

#[derive(Props, PartialEq, Clone)]
pub struct FilterChipsProps { pub game_identifier: String }

const CATEGORIES: &[(MessageKind, &str)] = &[
    (MessageKind::TributeKilled, "Deaths"),
    (MessageKind::Combat,        "Combat"),
    (MessageKind::AllianceFormed, "Alliances"),
    (MessageKind::TributeMoved,   "Movement"),
    (MessageKind::ItemFound,      "Items"),
];

#[component]
pub fn FilterChips(props: FilterChipsProps) -> Element {
    let mut filters: Signal<PeriodFilters> = use_context();
    let game_id = props.game_identifier.clone();
    {
        let mut f = filters.write();
        f.hydrate(&game_id);
    }
    let current = filters.read().filter_for(&game_id);

    let chip_class = |active: bool| -> &'static str {
        if active {
            "rounded-full px-3 py-1 text-sm bg-amber-500 text-amber-50 theme2:bg-green-600 theme3:bg-purple-500"
        } else {
            "rounded-full px-3 py-1 text-sm border border-amber-400 text-amber-700 theme2:border-green-600 theme2:text-green-300 theme3:border-purple-400 theme3:text-purple-200"
        }
    };

    let on_all = {
        let game_id = game_id.clone();
        move |_| filters.write().set_filter(&game_id, FilterMode::All)
    };

    rsx! {
        div { class: "flex flex-wrap gap-2 mb-4",
            button { class: chip_class(current.is_all()), onclick: on_all, "All" }
            for (kind, label) in CATEGORIES.iter().copied() {
                {
                    let game_id = game_id.clone();
                    let current = current.clone();
                    let active = match &current {
                        FilterMode::All => false,
                        FilterMode::Subset(s) => s.contains(&kind),
                    };
                    rsx!{
                        button {
                            key: "{label}",
                            class: chip_class(active),
                            onclick: move |_| {
                                let mut f = filters.write();
                                let next = match f.filter_for(&game_id) {
                                    FilterMode::All => {
                                        let mut s = HashSet::new();
                                        s.insert(kind);
                                        FilterMode::Subset(s)
                                    }
                                    FilterMode::Subset(mut s) => {
                                        if s.contains(&kind) { s.remove(&kind); } else { s.insert(kind); }
                                        if s.is_empty() { FilterMode::All } else { FilterMode::Subset(s) }
                                    }
                                };
                                f.set_filter(&game_id, next);
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Register in `mod.rs`**

```rust
mod filter_chips;
pub use filter_chips::FilterChips;
```

- [ ] **Step 3: Build & commit**

```bash
cargo check --package web --target wasm32-unknown-unknown
jj describe -m "feat(web): add FilterChips with All/Subset toggle behavior"
```

---

## Task 7: Timeline event card subcomponents

**Files:**
- Create: `web/src/components/timeline/cards/death_card.rs`
- Create: `web/src/components/timeline/cards/combat_card.rs`
- Create: `web/src/components/timeline/cards/alliance_card.rs`
- Create: `web/src/components/timeline/cards/movement_card.rs`
- Create: `web/src/components/timeline/cards/item_card.rs`
- Create: `web/src/components/timeline/cards/state_card.rs`
- Create: `web/src/components/timeline/cards/mod.rs`
- Create: `web/src/components/timeline/event_card.rs`
- Create: `web/src/components/timeline/timeline.rs`
- Modify: `web/src/components/timeline/mod.rs`

- [ ] **Step 1: Create `cards/mod.rs`**

```rust
pub mod death_card;
pub mod combat_card;
pub mod alliance_card;
pub mod movement_card;
pub mod item_card;
pub mod state_card;
```

- [ ] **Step 2: Implement `cards/death_card.rs`**

```rust
use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::TributeRef;

#[derive(Props, PartialEq, Clone)]
pub struct DeathCardProps {
    pub game_identifier: String,
    pub victim: TributeRef,
    pub killer: Option<TributeRef>,
    pub cause: String,
}

#[component]
pub fn DeathCard(props: DeathCardProps) -> Element {
    let victim_route = Routes::TributeDetail {
        game_identifier: props.game_identifier.clone(),
        tribute_identifier: props.victim.identifier.clone(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-red-500 bg-red-50 theme2:bg-red-950 p-3",
            header { class: "font-semibold",
                "💀 "
                Link { to: victim_route, class: "underline", "{props.victim.name}" }
                " killed"
            }
            if let Some(k) = props.killer.as_ref() {
                p { class: "text-sm",
                    "by "
                    Link {
                        to: Routes::TributeDetail {
                            game_identifier: props.game_identifier.clone(),
                            tribute_identifier: k.identifier.clone(),
                        },
                        class: "underline",
                        "{k.name}"
                    }
                }
            }
            p { class: "text-xs text-gray-600", "{props.cause}" }
        }
    }
}
```

- [ ] **Step 3: Implement `cards/combat_card.rs`**

```rust
use crate::routes::Routes;
use dioxus::prelude::*;
use shared::messages::{CombatOutcome, TributeRef};

#[derive(Props, PartialEq, Clone)]
pub struct CombatCardProps {
    pub game_identifier: String,
    pub attacker: TributeRef,
    pub defender: TributeRef,
    pub outcome: CombatOutcome,
    pub detail_lines: Vec<String>,
}

#[component]
pub fn CombatCard(props: CombatCardProps) -> Element {
    let mut expanded = use_signal(|| false);
    let outcome_label = match props.outcome {
        CombatOutcome::Killed       => "killed",
        CombatOutcome::Wounded      => "wounded",
        CombatOutcome::TargetFled   => "drove off",
        CombatOutcome::AttackerFled => "fled from",
        CombatOutcome::Stalemate    => "fought to a stalemate with",
    };
    let has_details = !props.detail_lines.is_empty();
    rsx! {
        article { class: "rounded border-l-4 border-orange-500 bg-orange-50 theme2:bg-orange-950 p-3",
            header { class: "font-semibold",
                "⚔️ "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: props.attacker.identifier.clone(),
                    },
                    class: "underline",
                    "{props.attacker.name}"
                }
                " {outcome_label} "
                Link {
                    to: Routes::TributeDetail {
                        game_identifier: props.game_identifier.clone(),
                        tribute_identifier: props.defender.identifier.clone(),
                    },
                    class: "underline",
                    "{props.defender.name}"
                }
            }
            if has_details {
                button {
                    class: "mt-1 text-xs underline",
                    onclick: move |_| expanded.set(!expanded()),
                    if expanded() { "hide details" } else { "show details" }
                }
                if expanded() {
                    ul { class: "mt-2 list-disc pl-5 text-sm",
                        for line in props.detail_lines.iter() { li { "{line}" } }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4: Implement `cards/alliance_card.rs`**

```rust
use dioxus::prelude::*;
use shared::messages::{MessagePayload, TributeRef};

#[derive(Props, PartialEq, Clone)]
pub struct AllianceCardProps { pub payload: MessagePayload }

#[component]
pub fn AllianceCard(props: AllianceCardProps) -> Element {
    let (icon, body) = match &props.payload {
        MessagePayload::AllianceFormed { members } => ("🤝", names(members, "formed an alliance")),
        MessagePayload::AllianceDisbanded { members, reason } =>
            ("💔", format!("{} ({reason})", names(members, "disbanded"))),
        MessagePayload::BetrayalTriggered { betrayer, victims } =>
            ("🗡️", format!("{} betrayed {}", betrayer.name, joined(victims))),
        MessagePayload::TrustShockBreak { tribute, source } =>
            ("⚡", format!("{} broke trust with {}", tribute.name, source.name)),
        MessagePayload::AllianceJoined { joiner, members } =>
            ("➕", format!("{} joined {}", joiner.name, joined(members))),
        _ => ("🤝", "alliance event".to_string()),
    };
    rsx! {
        article { class: "rounded border-l-4 border-emerald-500 bg-emerald-50 theme2:bg-emerald-950 p-3",
            header { class: "font-semibold", "{icon} {body}" }
        }
    }
}

fn names(members: &[TributeRef], verb: &str) -> String {
    format!("{} {verb}", joined(members))
}

fn joined(members: &[TributeRef]) -> String {
    members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
}
```

- [ ] **Step 5: Implement `cards/movement_card.rs`**

```rust
use dioxus::prelude::*;
use shared::messages::MessagePayload;

#[derive(Props, PartialEq, Clone)]
pub struct MovementCardProps { pub payload: MessagePayload }

#[component]
pub fn MovementCard(props: MovementCardProps) -> Element {
    let body = match &props.payload {
        MessagePayload::TributeMoved { tribute, from, to } =>
            format!("{} moved from {} to {}", tribute.name, from.name, to.name),
        MessagePayload::TributeFled { tribute, area } =>
            format!("{} fled from {}", tribute.name, area.name),
        MessagePayload::AreaClosed { area } =>
            format!("Area closed: {}", area.name),
        MessagePayload::AreaReopened { area } =>
            format!("Area reopened: {}", area.name),
        _ => "movement event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-sky-500 bg-sky-50 theme2:bg-sky-950 p-3",
            header { class: "font-semibold", "🧭 {body}" }
        }
    }
}
```

- [ ] **Step 6: Implement `cards/item_card.rs`**

```rust
use dioxus::prelude::*;
use shared::messages::MessagePayload;

#[derive(Props, PartialEq, Clone)]
pub struct ItemCardProps { pub payload: MessagePayload }

#[component]
pub fn ItemCard(props: ItemCardProps) -> Element {
    let body = match &props.payload {
        MessagePayload::ItemFound { tribute, item } =>
            format!("{} found {}", tribute.name, item.name),
        MessagePayload::ItemUsed { tribute, item } =>
            format!("{} used {}", tribute.name, item.name),
        MessagePayload::ItemBroken { tribute, item } =>
            format!("{}'s {} broke", tribute.name, item.name),
        MessagePayload::ItemDropped { tribute, item } =>
            format!("{} dropped {}", tribute.name, item.name),
        _ => "item event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-yellow-500 bg-yellow-50 theme2:bg-yellow-950 p-3",
            header { class: "font-semibold", "🎒 {body}" }
        }
    }
}
```

- [ ] **Step 7: Implement `cards/state_card.rs`**

```rust
use dioxus::prelude::*;
use shared::messages::MessagePayload;

#[derive(Props, PartialEq, Clone)]
pub struct StateCardProps { pub payload: MessagePayload }

#[component]
pub fn StateCard(props: StateCardProps) -> Element {
    let body = match &props.payload {
        MessagePayload::TributeWounded { tribute, source } =>
            format!("{} wounded by {}", tribute.name, source),
        MessagePayload::TributeRested { tribute } =>
            format!("{} rested", tribute.name),
        MessagePayload::TributeStarved { tribute } =>
            format!("{} is starving", tribute.name),
        MessagePayload::TributeDehydrated { tribute } =>
            format!("{} is dehydrated", tribute.name),
        MessagePayload::SanityBreak { tribute } =>
            format!("{} suffered a sanity break", tribute.name),
        _ => "state event".to_string(),
    };
    rsx! {
        article { class: "rounded border-l-4 border-gray-400 bg-gray-50 theme2:bg-gray-900 p-2 text-sm",
            "🌫️ {body}"
        }
    }
}
```

- [ ] **Step 8: Implement `event_card.rs` dispatcher**

```rust
use crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, death_card::DeathCard,
    item_card::ItemCard, movement_card::MovementCard, state_card::StateCard,
};
use dioxus::prelude::*;
use shared::messages::{GameMessage, MessageKind, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct EventCardProps {
    pub game_identifier: String,
    pub message: GameMessage,
}

#[component]
pub fn EventCard(props: EventCardProps) -> Element {
    let kind = props.message.payload.kind();
    let payload = props.message.payload.clone();
    rsx! {
        match kind {
            MessageKind::TributeKilled => {
                if let MessagePayload::TributeKilled { victim, killer, cause } = payload {
                    rsx!{ DeathCard {
                        game_identifier: props.game_identifier.clone(),
                        victim, killer, cause,
                    } }
                } else { rsx!{} }
            }
            MessageKind::Combat => {
                if let MessagePayload::Combat { attacker, defender, outcome, detail_lines } = payload {
                    rsx!{ CombatCard {
                        game_identifier: props.game_identifier.clone(),
                        attacker, defender, outcome, detail_lines,
                    } }
                } else { rsx!{} }
            }
            MessageKind::AllianceFormed
            | MessageKind::AllianceDisbanded
            | MessageKind::AllianceJoined
            | MessageKind::BetrayalTriggered
            | MessageKind::TrustShockBreak => rsx!{ AllianceCard { payload } },
            MessageKind::TributeMoved
            | MessageKind::TributeFled
            | MessageKind::AreaClosed
            | MessageKind::AreaReopened => rsx!{ MovementCard { payload } },
            MessageKind::ItemFound
            | MessageKind::ItemUsed
            | MessageKind::ItemBroken
            | MessageKind::ItemDropped => rsx!{ ItemCard { payload } },
            MessageKind::State
            | MessageKind::TributeWounded
            | MessageKind::TributeRested
            | MessageKind::TributeStarved
            | MessageKind::TributeDehydrated
            | MessageKind::SanityBreak => rsx!{ StateCard { payload } },
        }
    }
}
```

> **Note:** The exact `MessageKind` variants must match what PR1's `MessagePayload::kind()` returns. If PR1 chose different variant names, adjust the match arms here.

- [ ] **Step 9: Implement `timeline.rs`**

```rust
use crate::components::timeline::event_card::EventCard;
use crate::components::timeline::FilterMode;
use dioxus::prelude::*;
use shared::messages::GameMessage;

#[derive(Props, PartialEq, Clone)]
pub struct TimelineProps {
    pub game_identifier: String,
    pub messages: Vec<GameMessage>,
    pub filter: FilterMode,
}

#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let mut sorted: Vec<GameMessage> = props.messages
        .into_iter()
        .filter(|m| props.filter.matches(m.payload.kind()))
        .collect();
    sorted.sort_by_key(|m| (m.tick, m.emit_index));
    rsx! {
        if sorted.is_empty() {
            div { class: "rounded border border-dashed p-6 text-center text-sm",
                "Nothing happened this period."
            }
        } else {
            div { class: "space-y-2",
                for (i, msg) in sorted.into_iter().enumerate() {
                    EventCard {
                        key: "{i}",
                        game_identifier: props.game_identifier.clone(),
                        message: msg,
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 10: Update `web/src/components/timeline/mod.rs`**

```rust
pub mod cards;
pub mod event_card;
pub mod filters;
pub mod timeline;

pub use event_card::EventCard;
pub use filters::{FilterMode, PeriodFilters};
pub use timeline::Timeline;
```

- [ ] **Step 11: Build**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: compiles. Variants in match arms must match PR1's actual enum exactly — fix any mismatches.

- [ ] **Step 12: Commit**

```bash
jj describe -m "feat(web): add timeline event cards (death/combat/alliance/movement/item/state)"
```

---

## Task 8: GamePeriodPage component + route

**Files:**
- Create: `web/src/components/game_period_page.rs`
- Modify: `web/src/components/mod.rs`
- Modify: `web/src/routes.rs`

- [ ] **Step 1: Implement `GamePeriodPage` in `web/src/components/game_period_page.rs`**

```rust
use crate::cache::{QueryError, QueryKey, QueryValue};
use crate::components::filter_chips::FilterChips;
use crate::components::timeline::{PeriodFilters, Timeline};
use crate::env::API_HOST;
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use reqwest::StatusCode;
use shared::messages::{GameMessage, Phase, TimelineSummary};

async fn fetch_day_log(keys: Vec<QueryKey>) -> QueryResult<QueryValue, QueryError> {
    let Some(QueryKey::GameDayLog(id, day)) = keys.first() else {
        return Err(QueryError::Unknown).into();
    };
    let url = format!("{API_HOST}/api/games/{id}/log/{day}");
    match reqwest::get(&url).await {
        Ok(resp) if resp.status() == StatusCode::OK => {
            match resp.json::<Vec<GameMessage>>().await {
                Ok(v) => Ok(QueryValue::Logs(v)).into(),
                Err(_) => Err(QueryError::BadJson).into(),
            }
        }
        Ok(_) => Err(QueryError::GameNotFound(id.clone())).into(),
        Err(_) => Err(QueryError::ServerNotFound).into(),
    }
}

#[component]
pub fn GamePeriodPage(identifier: String, day: u32, phase: Phase) -> Element {
    let filters: Signal<PeriodFilters> = use_context();
    let filter = filters.read().filter_for(&identifier);

    // validate (day, phase) against TimelineSummary
    let summary_q = use_get_query(
        [QueryKey::TimelineSummary(identifier.clone())],
        crate::hooks::use_timeline_summary::fetch_timeline_summary,
    );

    let valid = match summary_q.result().value() {
        QueryResult::Ok(QueryValue::TimelineSummary(s)) => {
            s.periods.iter().any(|p| p.day == day && p.phase == phase)
        }
        QueryResult::Err(_) => false,
        _ => true, // assume valid while loading
    };

    if !valid {
        return rsx! {
            h1 { "Period not found" }
            p { "Day {day} ({phase:?}) doesn't exist for this game." }
        };
    }

    let log_q = use_get_query([QueryKey::GameDayLog(identifier.clone(), day)], fetch_day_log);

    rsx! {
        div { class: "space-y-4",
            h1 { class: "text-2xl font-semibold", "Day {day} — {phase:?}" }
            FilterChips { game_identifier: identifier.clone() }
            match log_q.result().value() {
                QueryResult::Ok(QueryValue::Logs(msgs)) => {
                    let filtered: Vec<GameMessage> = msgs.iter()
                        .filter(|m| m.phase == phase)
                        .cloned()
                        .collect();
                    rsx!{ Timeline {
                        game_identifier: identifier.clone(),
                        messages: filtered,
                        filter,
                    } }
                }
                QueryResult::Err(_) => rsx!{ p { "Failed to load events." } },
                _ => rsx!{ div { class: "animate-pulse h-32 rounded bg-gray-200" } },
            }
        }
    }
}
```

> **Note:** `fetch_timeline_summary` must be `pub` in `web/src/hooks/use_timeline_summary.rs` for re-use here. Update Task 2 Step 2 if needed.

- [ ] **Step 2: Register in `mod.rs`**

```rust
mod game_period_page;
pub use game_period_page::GamePeriodPage;
```

- [ ] **Step 3: Add route in `web/src/routes.rs`**

Inside the `Games` layout block (right after `GamePage`), add:

```rust
                #[route("/:identifier/day/:day/:phase")]
                GamePeriodPage { identifier: String, day: u32, phase: shared::messages::Phase },
```

Update the imports at the top of `routes.rs`:

```rust
use crate::components::{
    Accounts, AccountsPage, Credits, GamePage, GamePeriodPage, Games, GamesList, Home,
    IconsPage, Navbar, TributeDetail,
};
```

> **Note:** `shared::messages::Phase` must implement `FromStr`, `Display`, `Clone`, and `PartialEq` for the dioxus-router macro. PR1 already requires this. If routing complains, the path is the issue — verify with `dx serve` and the dioxus-router error.

- [ ] **Step 4: Update `PeriodCard` in Task 3 to use the real route**

In `web/src/components/period_card.rs`, change `Routes::Home {}` (placeholder) back to:

```rust
let route = Routes::GamePeriodPage {
    identifier: props.game_identifier.clone(),
    day: props.day,
    phase: props.phase,
};
```

(If you didn't use a placeholder, this step is a no-op verification.)

- [ ] **Step 5: Build**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: compiles end-to-end.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(web): add GamePeriodPage route and component"
```

---

## Task 9: Replace stub in game_detail.rs

**Files:**
- Modify: `web/src/components/game_detail.rs` — remove `GameLogStub` usage; insert `RecapCard` (when finished) + `PeriodGrid`; wire mutation handlers
- Delete: `web/src/components/game_log_stub.rs`
- Modify: `web/src/components/mod.rs` — drop `mod game_log_stub;`

- [ ] **Step 1: In `game_detail.rs`, replace the stub render block**

Find the place where `GameLogStub { ... }` (or whatever PR1 named it) is rendered. Replace with:

```rust
if game.status == shared::GameStatus::Finished {
    rsx!{ RecapCard { game: game.clone() } }
}
PeriodGrid { game_identifier: game.identifier.clone() }
```

Add imports:

```rust
use crate::components::{PeriodGrid, RecapCard};
```

- [ ] **Step 2: Wire mutation handlers**

Find the `next_step` mutation success branch. Add a side effect to bump generation on `MutationValue::GameAdvanced`:

```rust
MutationValue::GameAdvanced(game_identifier) => {
    let mut filters: Signal<crate::components::timeline::PeriodFilters> = use_context();
    filters.write().bump(&game_identifier);
    // (existing query-invalidation code)
    ...
}
```

Find the game-delete mutation handler (or add one if absent). On `MutationValue::GameDeleted(id, _)`:

```rust
MutationValue::GameDeleted(id, _) => {
    let _ = gloo_storage::LocalStorage::delete(&format!("recap_collapsed:{id}"));
    let _ = gloo_storage::LocalStorage::delete(&format!("period_filters:{id}"));
    // (existing navigate-away / list-refresh code)
}
```

> **Note:** `use_context::<Signal<PeriodFilters>>()` must be called at component scope, not inside the mutation closure. Move the `let mut filters = use_context::<…>();` to the top of the `GamePage` function and capture it by `move`.

- [ ] **Step 3: Delete `game_log_stub.rs`**

```bash
rm -f web/src/components/game_log_stub.rs
```

Remove `mod game_log_stub;` from `web/src/components/mod.rs`.

- [ ] **Step 4: Build**

Run: `cargo check --package web --target wasm32-unknown-unknown`.
Expected: compiles. Fix any leftover references to the stub or `MessageKind` enum mismatches.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): replace timeline stub with RecapCard + PeriodGrid"
```

---

## Task 10: Quality gates and smoke test

**Files:** none modified — verification only.

- [ ] **Step 1: Format**

```bash
just fmt
```

- [ ] **Step 2: Quality checks**

```bash
just quality
```

Fix anything that isn't clean.

- [ ] **Step 3: Build CSS**

```bash
just build-css
```

- [ ] **Step 4: Smoke test with `just dev`**

Start the stack:

```bash
just dev
```

Walk through the spec §6 verification list:
1. Open a game's page → period card grid appears; current period has highlight ring.
2. Click a card → navigates to `/games/:id/day/N/day` (or night).
3. On the period page: chip row shows `[All] [Deaths] [Combat] [Alliances] [Movement] [Items]`. State events still appear with `All`.
4. Click `[Combat]` → only Combat (and ambient State) cards remain.
5. Click `[Combat]` again → snaps back to `All`, `All` chip visually active.
6. Refresh the page → filter persists for that game.
7. Click a Combat card with `detail_lines` → expands. Combat with empty `detail_lines` shows no expand button.
8. Click a tribute name in any card → navigates to tribute detail.
9. Finish a game (advance through to end). Recap card appears. Collapse it. Reload — stays collapsed.
10. Delete the game → navigate to a fresh page → both `recap_collapsed:{id}` and `period_filters:{id}` are gone from `LocalStorage` (verify in DevTools).
11. Browser back/forward across hub  period works.
12. Click "Begin / Next Step" → both `/timeline-summary` and `/log/N` re-fetch (generation bump).

- [ ] **Step 5: Commit any final fixes**

```bash
jj describe -m "chore(web): cleanup after smoke test"
```

---

## Task 11: Open the PR

**Files:** none.

- [ ] **Step 1: Sync with main**

```bash
jj git fetch
jj rebase -d main@origin
```

- [ ] **Step 2: Push beads data**

```bash
bd dolt push
```

- [ ] **Step 3: Create the bookmark**

```bash
jj bookmark create feat-timeline-pr2-frontend -r @-
```

- [ ] **Step 4: Push the bookmark**

```bash
jj git push --bookmark feat-timeline-pr2-frontend
```

- [ ] **Step 5: Open the PR**

```bash
gh pr create --base main --head feat-timeline-pr2-frontend \
  --title "feat(web): structured day/phase timeline UI" \
  --body "$(cat <<'EOF'
## Summary
Replaces the unstyled day-log on `GamePage` with a structured timeline UI:
- Hub of period cards (Day/Night per simulated day) with deaths + event counts
- Per-period view at `/games/:id/day/:day/:phase` with filter chips and typed event cards
- Finished-game `RecapCard`, collapsible, persisted per-game

Builds on PR1's typed `MessagePayload` schema and `/api/games/:id/timeline-summary` endpoint.

## Changes
- New components: `PeriodGrid`, `PeriodGridEmpty`, `PeriodCard`, `GamePeriodPage`, `RecapCard`, `FilterChips`, `Timeline`, `EventCard`, and 6 typed sub-cards.
- New context: `PeriodFilters` provided in `Navbar`, persisted to `gloo-storage`.
- Mutation handlers in `game_detail.rs` bump per-game generation on advance, clear localStorage on delete.
- Deleted: `game_log_stub.rs` (PR1's placeholder).

## Verification
- `just fmt && just quality` clean.
- `just dev` smoke test passed (see plan Task 10 Step 4 list).

## Follow-ups
See spec §7. Beads issues filed: combat redesign, mobile polish, announcer cards, per-tribute filter, orphaned summarize endpoint, hover-preview, item/area routes, URL filter params, sponsor gift confirmation, winner→TributeRef upgrade, websocket cache invalidation.
EOF
)"
```

- [ ] **Step 6: Verify**

The PR URL is printed. Open it in a browser, confirm CI starts.

- [ ] **Step 7: Hand off**

Report the PR URL to the user.

---

## Self-review notes

**Spec coverage check:**
- §5 component tree → Tasks 3, 4, 5, 6, 7, 8 cover every file in the spec's tree.
- §5 state (`PeriodFilters` at Navbar layer, `gloo-storage` per game) → Task 1.
- §5 mutation handlers (generation bump, gloo cleanup) → Task 9 Step 2.
- §6 testing — frontend has no test infrastructure; smoke test in Task 10 substitutes per spec.
- §6 rollout PR2 description matches Task 9 + 11.
- §7 follow-ups → Task 11 Step 5 references all of them in the PR body.

**Placeholder scan:** No TBDs. Two "Note:" callouts flag dependencies (route variants matching PR1's `MessageKind`, `fetch_timeline_summary` visibility) — these are explicit verify-and-adjust points, not placeholders.

**Type consistency:** `PeriodFilters`, `FilterMode`, `MessageKind`, `MessagePayload`, `Phase`, `TributeRef`, `CombatOutcome`, `TimelineSummary`, `PeriodSummary` are all types defined by PR1 in `shared::messages`. `QueryKey::GameDayLog(String, u32)` and `QueryKey::TimelineSummary(String)` consistent across Tasks 2 and 8.
