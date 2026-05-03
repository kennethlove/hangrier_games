# Shelter + Hunger/Thirst — Plan 2: Frontend Implementation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface the shelter + hunger/thirst backend in the Dioxus frontend: a generic tribute state strip with hunger/thirst pips and a shelter glyph; a Survival section in the tribute detail page; survival event cards in the timeline; and terrain affordance overlays on the hex map when a tribute is selected.

**Architecture:** Build a small reusable `tribute_state_strip.rs` component now, populated only with survival pips for v1 — emotion pips slot in cleanly later without refactor. The Survival section lives directly inside the existing `tribute_detail.rs` page (no separate Inspect drilldown abstraction; YAGNI). New timeline cards plug into the existing `state_card.rs` / `death_card.rs` dispatch. Map gets a per-hex affordance overlay layer that lights up when a tribute is selected; selection state is local to the map for v1.

**Tech Stack:** Dioxus 0.7 (Rust → WASM), Tailwind CSS via the existing build pipeline, `dioxus-query` for fetching, `gloo-storage` for any persisted UI state. All shared types reach the frontend via the `game::` and `shared::` re-imports already in use.

**Spec:** `docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md`
**Backend plan (predecessor):** `docs/superpowers/plans/2026-05-03-shelter-hunger-thirst-pr1-backend.md`
**Beads issue:** `hangrier_games-0yz`

---

## Pre-flight Notes

- Plan 2 must merge after Plan 1 (PR1 backend) lands. The frontend imports new fields and message variants that PR1 introduces.
- `web::` already re-imports `game::tributes::Tribute` directly — Plan 1's new fields (`hunger`, `thirst`, `sheltered_until`, etc.) flow through the existing `TributeQ` JSON contract automatically. No DTO bridging required.
- Same applies to `MessagePayload` variants from Plan 1 — they flow through `GameMessage` / `MessagePayload` re-imports already used by the timeline cards.
- Pure-function calls (`shelter_quality`, `forage_richness`, `water_source`, `hunger_band`, `thirst_band`) live in `game::areas::*` and `game::tributes::survival` — also directly available.
- Run `just build-css` after any class changes that introduce new Tailwind utility classes (so the JIT pulls them).
- Run `cd web && dx serve` for the live dev loop; smoke tests are manual at the end of each task.
- All commits use the project's jj/git workflow per `AGENTS.md`.

---

## File Structure

**New files:**
- `web/src/components/tribute_state_strip.rs` — generic state strip component; v1 renders survival pips only.
- `web/src/components/tribute_survival_section.rs` — Survival section rendered inside `tribute_detail.rs`.
- `web/src/components/timeline/cards/survival_card.rs` — timeline cards for `HungerBandChanged`, `ThirstBandChanged`, `ShelterSought`, `Foraged`, `Drank`, `Ate`.
- `web/src/components/map_affordance_overlay.rs` — per-hex 💧/🌿/🏠 glyph layer rendered when a tribute is selected on the map.

**Modified files:**
- `web/src/components/mod.rs` — register the new components.
- `web/src/components/tribute_detail.rs` — render `TributeStateStrip` near the status block; add `TributeSurvivalSection` to the body.
- `web/src/components/game_tributes.rs` (or wherever the tribute card list renders) — render the `TributeStateStrip` on each card.
- `web/src/components/map.rs` — accept an optional `selected_tribute: Option<TributeRef>` prop; render the affordance overlay layer when set; expose a click handler that toggles selection (selection state can be `Signal`-local or lifted to the page if it's needed elsewhere).
- `web/src/components/timeline/cards/mod.rs` — re-export the new survival card.
- `web/src/components/timeline/event_card.rs` — dispatch the new `MessagePayload` variants to the survival card.
- `web/src/components/timeline/cards/death_card.rs` — render `cause = "starvation"` / `"dehydration"` with appropriate copy and styling.
- `web/src/components/timeline/cards/state_card.rs` — confirm existing `TributeStarved` / `TributeDehydrated` rendering still applies (no change needed unless dispatch routing changes).
- `web/src/components/timeline/filters.rs` — add a survival filter category if filters are category-based.

---

## Task Order Rationale

Build the smallest reusable piece first (state strip), then the page-level use of it (tribute detail Survival section), then timeline cards, then the map overlay (most ambitious, depends on the map's existing component shape). Each task is independently mergeable and visually verifiable.

---

## Task 1: Generic `TributeStateStrip` component (survival pips only)

**Files:**
- Create: `web/src/components/tribute_state_strip.rs`
- Modify: `web/src/components/mod.rs`

- [ ] **Step 1: Read the existing tribute card render to understand the visual context**

Run: `grep -n "TributeStatusIcon" web/src/components/*.rs`

Open the tribute card render site (likely `game_tributes.rs` or inside `tribute_detail.rs`) and note where the status icon currently sits. The state strip will live nearby.

- [ ] **Step 2: Write the component**

Create `web/src/components/tribute_state_strip.rs`:

```rust
use dioxus::prelude::*;
use game::tributes::Tribute;
use game::tributes::survival::{
    hunger_band, thirst_band, HungerBand, ThirstBand,
};

/// Generic state strip for a tribute's at-a-glance state pips. v1 renders
/// only survival (hunger, thirst, shelter) pips; emotion pips slot in here
/// when the emotion frontend lands.
///
/// The strip renders nothing when every pip is in its hidden state (e.g.,
/// Sated hunger, Sated thirst, no shelter), so a healthy tribute card stays
/// visually clean.
#[component]
pub fn TributeStateStrip(tribute: Tribute, current_phase: Option<u32>) -> Element {
    let h_band = hunger_band(tribute.hunger);
    let t_band = thirst_band(tribute.thirst);
    let sheltered_phases_left = match (tribute.sheltered_until, current_phase) {
        (Some(until), Some(now)) if until > now => Some(until - now),
        _ => None,
    };

    let any_visible = h_band != HungerBand::Sated
        || t_band != ThirstBand::Sated
        || sheltered_phases_left.is_some();

    if !any_visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex flex-row gap-2 items-center text-sm select-none",
            // Hunger pip
            if h_band != HungerBand::Sated {
                HungerPip { band: h_band, raw: tribute.hunger }
            }
            // Thirst pip
            if t_band != ThirstBand::Sated {
                ThirstPip { band: t_band, raw: tribute.thirst }
            }
            // Shelter glyph
            if let Some(left) = sheltered_phases_left {
                ShelterPip { phases_left: left }
            }
        }
    }
}

#[component]
fn HungerPip(band: HungerBand, raw: u8) -> Element {
    let (cls, label) = match band {
        HungerBand::Peckish => ("text-amber-300/60", "Peckish"),
        HungerBand::Hungry => ("text-amber-400", "Hungry"),
        HungerBand::Starving => ("text-red-500 animate-pulse", "Starving"),
        HungerBand::Sated => return rsx! {},
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-1 {cls}",
            "aria-label": "Hunger: {label}",
            title: "Hunger {raw} — {label}",
            span { class: "text-base", "🍗" }
            span { class: "text-xs uppercase tracking-wide", "{label}" }
        }
    }
}

#[component]
fn ThirstPip(band: ThirstBand, raw: u8) -> Element {
    let (cls, label) = match band {
        ThirstBand::Thirsty => ("text-sky-300/60", "Thirsty"),
        ThirstBand::Parched => ("text-sky-400", "Parched"),
        ThirstBand::Dehydrated => ("text-red-500 animate-pulse", "Dehydrated"),
        ThirstBand::Sated => return rsx! {},
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-1 {cls}",
            "aria-label": "Thirst: {label}",
            title: "Thirst {raw} — {label}",
            span { class: "text-base", "💧" }
            span { class: "text-xs uppercase tracking-wide", "{label}" }
        }
    }
}

#[component]
fn ShelterPip(phases_left: u32) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center gap-1 text-emerald-300",
            "aria-label": "Sheltered for {phases_left} more phases",
            title: "Sheltered for {phases_left} more phases",
            span { class: "text-base", "🏠" }
            span { class: "text-xs", "{phases_left}" }
        }
    }
}
```

- [ ] **Step 3: Register the module**

Edit `web/src/components/mod.rs`, add:

```rust
pub mod tribute_state_strip;
```

- [ ] **Step 4: Compile**

Run: `cd web && cargo check`
Expected: clean compile.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): generic TributeStateStrip with survival pips

New reusable component renders at-a-glance state pips for a tribute.
v1 renders only survival pips (hunger, thirst, shelter). Future
emotion frontend slots its pips into the same strip.

The strip self-hides when every pip is in its 'hidden' state, so
healthy tribute cards stay visually clean.

Refs: hangrier_games-0yz"
```

---

## Task 2: Wire `TributeStateStrip` into the tribute card list

**Files:**
- Modify: `web/src/components/game_tributes.rs` (or whatever component renders the per-tribute card row on the game page).

- [ ] **Step 1: Locate the card render site**

Run: `grep -n "tribute" web/src/components/game_tributes.rs | head -20`

Find the loop or component that renders a tribute summary card. Identify the cleanest place to insert `TributeStateStrip` (typically after the name + status icon row).

- [ ] **Step 2: Determine `current_phase` source**

The shelter pip needs the current phase index to compute remaining phases. Locate how the existing tribute or game data exposes the current phase (likely on `DisplayGame` or via a query — search for `current_phase` / `day` / `cycle`).

If the page already fetches a `Game` or has `current_day` / `current_phase_index` available, pass it through. If not, pass `None` — the shelter pip will then render with `phases_left = ?` placeholder; in that case render the glyph with no number.

- [ ] **Step 3: Write the failing visual smoke test**

Visual smoke: there is no automated test for this — verify by `cd web && dx serve` and looking at the page after the backend lands.

For *type* safety, run `cd web && cargo check` — if the import / props are wrong it will fail at compile.

- [ ] **Step 4: Insert the component**

In `web/src/components/game_tributes.rs`, import:

```rust
use crate::components::tribute_state_strip::TributeStateStrip;
```

Inside the per-tribute render block, add (adapting the surrounding `rsx!` shape):

```rust
TributeStateStrip {
    tribute: tribute.clone(),
    current_phase: current_phase_signal(),  // adapt to actual source
}
```

If no current-phase source exists yet, pass `None` and update the `ShelterPip` to handle the case (already handled — the strip just doesn't render the shelter pip when `current_phase` is `None`).

- [ ] **Step 5: Build CSS and check**

Run: `just build-css`
Run: `cd web && cargo check`
Expected: clean.

- [ ] **Step 6: Manual smoke**

Run `just dev` (in another terminal). Open a game in the browser. With Plan 1 backend live, kick a game forward enough phases that some tribute crosses into Hungry / Parched (or use the dev DB to bump `tribute.hunger = 5` directly). Confirm the pips appear on the card.

- [ ] **Step 7: Commit**

```bash
jj describe -m "feat(web): show TributeStateStrip on game tributes list

The per-tribute card on the game page now renders survival pips when
a tribute is hungry, thirsty, or sheltered. Healthy tributes show
nothing extra.

Refs: hangrier_games-0yz"
```

---

## Task 3: Survival section on tribute detail page

**Files:**
- Create: `web/src/components/tribute_survival_section.rs`
- Modify: `web/src/components/mod.rs`
- Modify: `web/src/components/tribute_detail.rs`

- [ ] **Step 1: Write the new section component**

Create `web/src/components/tribute_survival_section.rs`:

```rust
use dioxus::prelude::*;
use game::tributes::Tribute;
use game::tributes::survival::{
    hunger_band, thirst_band, HungerBand, ThirstBand,
};

/// Survival panel inside the tribute detail page. Always renders (so the
/// page has a stable shape); shows "Sated" / "Exposed" labels when nothing
/// dramatic is happening.
#[component]
pub fn TributeSurvivalSection(tribute: Tribute, current_phase: Option<u32>) -> Element {
    let h_band = hunger_band(tribute.hunger);
    let t_band = thirst_band(tribute.thirst);
    let sheltered_phases_left = match (tribute.sheltered_until, current_phase) {
        (Some(until), Some(now)) if until > now => Some(until - now),
        _ => None,
    };

    let starvation_drain_line: Option<String> = if h_band == HungerBand::Starving {
        Some(format!(
            "Starving — losing {} HP/phase (next phase: {})",
            tribute.starvation_drain_step,
            tribute.starvation_drain_step.saturating_add(1),
        ))
    } else { None };

    let dehydration_drain_line: Option<String> = if t_band == ThirstBand::Dehydrated {
        Some(format!(
            "Dehydrated — losing {} HP/phase (next phase: {})",
            tribute.dehydration_drain_step,
            tribute.dehydration_drain_step.saturating_add(1),
        ))
    } else { None };

    let h_label = format!("{:?}", h_band);
    let t_label = format!("{:?}", t_band);

    rsx! {
        section {
            class: "rounded-lg border border-stone-700/40 bg-stone-900/30 p-4 mt-4",
            h3 {
                class: "text-lg font-semibold mb-2 text-stone-100",
                "Survival"
            }
            dl {
                class: "grid grid-cols-2 gap-y-1 text-sm",
                dt { class: "text-stone-400", "Hunger" }
                dd { class: "text-stone-100", "{tribute.hunger} ({h_label})" }
                dt { class: "text-stone-400", "Thirst" }
                dd { class: "text-stone-100", "{tribute.thirst} ({t_label})" }
                dt { class: "text-stone-400", "Shelter" }
                dd {
                    class: "text-stone-100",
                    if let Some(left) = sheltered_phases_left {
                        "Sheltered for {left} more phases"
                    } else {
                        "Exposed"
                    }
                }
            }
            if let Some(line) = starvation_drain_line {
                p { class: "mt-2 text-sm text-red-400", "{line}" }
            }
            if let Some(line) = dehydration_drain_line {
                p { class: "mt-1 text-sm text-red-400", "{line}" }
            }
        }
    }
}
```

- [ ] **Step 2: Register the module**

In `web/src/components/mod.rs`, add:

```rust
pub mod tribute_survival_section;
```

- [ ] **Step 3: Insert into tribute detail page**

In `web/src/components/tribute_detail.rs`, import:

```rust
use crate::components::tribute_survival_section::TributeSurvivalSection;
```

Inside the `Settled { res: Ok(tribute), .. }` render block (around line 124+), find the existing block that renders status / attributes and add the survival section after it:

```rust
TributeSurvivalSection {
    tribute: (**tribute).clone(),
    current_phase: None,  // wire to real phase source if available; see Task 2
}
```

- [ ] **Step 4: Compile and smoke**

Run: `cd web && cargo check`
Expected: clean.

Run `just dev`, open a tribute detail page. Confirm the Survival section renders. Bump a tribute to `hunger=5` via the dev DB; confirm the drain line appears in red.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(web): add Survival section to tribute detail page

New TributeSurvivalSection component inside tribute_detail.rs shows
hunger/thirst counters and bands, shelter status with phases remaining,
and an escalating-drain line when the tribute is Starving/Dehydrated.

Refs: hangrier_games-0yz"
```

---

## Task 4: Survival timeline event cards

**Files:**
- Create: `web/src/components/timeline/cards/survival_card.rs`
- Modify: `web/src/components/timeline/cards/mod.rs`
- Modify: `web/src/components/timeline/event_card.rs`

- [ ] **Step 1: Read the existing card-dispatch pattern**

Open `web/src/components/timeline/event_card.rs` and read how it dispatches `MessagePayload` variants to per-card components (e.g. `state_card`, `combat_card`).

- [ ] **Step 2: Write the new card module**

Create `web/src/components/timeline/cards/survival_card.rs`:

```rust
use dioxus::prelude::*;
use shared::messages::MessagePayload;

#[component]
pub fn SurvivalCard(payload: MessagePayload) -> Element {
    match payload {
        MessagePayload::HungerBandChanged { tribute, from, to } => rsx! {
            p {
                class: hunger_class(&to),
                "{tribute.name} is now "
                strong { "{to}" }
                span { class: "text-xs text-stone-500 ml-1", "(was {from})" }
            }
        },
        MessagePayload::ThirstBandChanged { tribute, from, to } => rsx! {
            p {
                class: thirst_class(&to),
                "{tribute.name} is now "
                strong { "{to}" }
                span { class: "text-xs text-stone-500 ml-1", "(was {from})" }
            }
        },
        MessagePayload::ShelterSought { tribute, area, success, roll: _ } => rsx! {
            p {
                class: "text-xs text-stone-400",
                if success {
                    "{tribute.name} found shelter in {area.name}."
                } else {
                    "{tribute.name} failed to find shelter in {area.name}."
                }
            }
        },
        MessagePayload::Foraged { tribute, area, success, debt_recovered } => rsx! {
            p {
                class: "text-xs text-stone-400",
                if success {
                    "{tribute.name} foraged in {area.name} (+{debt_recovered} hunger relief)."
                } else {
                    "{tribute.name} foraged in {area.name} but found nothing."
                }
            }
        },
        MessagePayload::Drank { tribute, source: _, debt_recovered } => rsx! {
            p {
                class: "text-xs text-stone-400",
                "{tribute.name} drank (+{debt_recovered} thirst relief)."
            }
        },
        MessagePayload::Ate { tribute, item: _, debt_recovered } => rsx! {
            p {
                class: "text-xs text-stone-400",
                "{tribute.name} ate (+{debt_recovered} hunger relief)."
            }
        },
        _ => rsx! {},  // not a survival payload
    }
}

fn hunger_class(band: &str) -> &'static str {
    match band {
        "Hungry" => "text-amber-400",
        "Starving" => "text-red-500 font-semibold",
        _ => "text-stone-400",
    }
}

fn thirst_class(band: &str) -> &'static str {
    match band {
        "Parched" => "text-sky-400",
        "Dehydrated" => "text-red-500 font-semibold",
        _ => "text-stone-400",
    }
}
```

- [ ] **Step 3: Register the new card**

Edit `web/src/components/timeline/cards/mod.rs`, add:

```rust
pub mod survival_card;
```

- [ ] **Step 4: Wire dispatch in `event_card.rs`**

In `web/src/components/timeline/event_card.rs`, find the dispatch match and add arms (or extend the existing one) for the survival payloads. Pseudo-shape:

```rust
use crate::components::timeline::cards::survival_card::SurvivalCard;

// In the match on `payload.kind()` or directly on `payload`:
MessagePayload::HungerBandChanged { .. }
| MessagePayload::ThirstBandChanged { .. }
| MessagePayload::ShelterSought { .. }
| MessagePayload::Foraged { .. }
| MessagePayload::Drank { .. }
| MessagePayload::Ate { .. } => rsx! { SurvivalCard { payload: payload.clone() } },
```

- [ ] **Step 5: Death card extension for new causes**

In `web/src/components/timeline/cards/death_card.rs`, locate where `cause` is rendered. If it currently renders the raw string, that already works — `"starvation"` and `"dehydration"` from `shared::messages::CAUSE_STARVATION` / `CAUSE_DEHYDRATION` will display. Optionally style:

```rust
let cause_class = match props.cause.as_str() {
    "starvation" | "dehydration" => "text-amber-400 italic",
    _ => "text-gray-600",
};
// ...render with class
```

- [ ] **Step 6: Compile and smoke**

Run: `cd web && cargo check`
Expected: clean.

Smoke: with Plan 1 live, run a game until band crossings and starvation deaths happen. Confirm the cards render in the timeline.

- [ ] **Step 7: Commit**

```bash
jj describe -m "feat(web): timeline cards for survival events + death causes

New SurvivalCard renders HungerBandChanged, ThirstBandChanged,
ShelterSought, Foraged, Drank, and Ate payloads. Death cards style
starvation/dehydration causes with appropriate emphasis.

Refs: hangrier_games-0yz"
```

---

## Task 5: Optional — survival filter category in the timeline

**Files:**
- Modify: `web/src/components/timeline/filters.rs`

This task is *optional* — file as a follow-up bead and skip if the timeline-filter system isn't trivially extensible (the spec did not require it).

- [ ] **Step 1: Read the existing filter category model**

Run: `grep -nE "enum.*Filter|filter_categories|category" web/src/components/timeline/filters.rs | head -20`

If the filter system is category-driven and survival fits cleanly as a new category, proceed. If it's a more rigid taxonomy, file `bd create "Add 'Survival' filter chip to game timeline"` and stop here.

- [ ] **Step 2: Add a Survival category if applicable**

Pattern depends on the existing filter shape. Wire the survival `MessagePayload` variants to the new category and add a chip to the filter UI matching the existing style.

- [ ] **Step 3: Commit (if implemented)**

```bash
jj describe -m "feat(web): add Survival filter chip to timeline

Refs: hangrier_games-0yz"
```

---

## Task 6: Map terrain affordance overlay

**Files:**
- Create: `web/src/components/map_affordance_overlay.rs`
- Modify: `web/src/components/map.rs`

- [ ] **Step 1: Decide the selection-state model**

Selection is local to the map for v1: a `Signal<Option<Area>>` inside `Map` that toggles when a hex is clicked. (The existing `onclick` already logs; replace the log with a selection toggle.)

When `selected.is_some()`, render the affordance overlay; when `None`, don't.

- [ ] **Step 2: Write the overlay component**

Create `web/src/components/map_affordance_overlay.rs`:

```rust
use dioxus::prelude::*;
use game::areas::AreaDetails;
use game::areas::forage::forage_richness;
use game::areas::shelter::shelter_quality;
use game::areas::water::water_source;
use game::areas::weather::{current_weather, Weather};

/// Renders 💧 / 🌿 / 🏠 glyphs on hexes when an area is selected, hinting at
/// terrain affordances (water source, forage, shelter quality).
///
/// `cx`, `cy`, `size` are the hex centroid + size in SVG units; `area` is the
/// AreaDetails for that hex.
#[component]
pub fn MapAffordanceOverlay(cx: f64, cy: f64, size: f64, area: AreaDetails) -> Element {
    let weather = current_weather();
    let terrain = area.terrain.base;
    let water = water_source(terrain, &weather);
    let forage = forage_richness(terrain);
    let shelter = shelter_quality(terrain, &weather);

    // Glyphs anchor in a horizontal row across the bottom of the hex.
    let glyph_size = size * 0.20;
    let row_y = cy + size * 0.55;
    let mut x_off = -glyph_size;

    rsx! {
        g {
            class: "pointer-events-none",
            if water > 0 {
                {
                    let x = cx + x_off;
                    x_off += glyph_size * 1.2;
                    rsx! {
                        text {
                            x: "{x}",
                            y: "{row_y}",
                            text_anchor: "middle",
                            font_size: "{glyph_size}",
                            "💧"
                        }
                    }
                }
            }
            if forage > 0 {
                {
                    let x = cx + x_off;
                    x_off += glyph_size * 1.2;
                    rsx! {
                        text {
                            x: "{x}",
                            y: "{row_y}",
                            text_anchor: "middle",
                            font_size: "{glyph_size}",
                            "🌿"
                        }
                    }
                }
            }
            if shelter >= 2 {
                {
                    let x = cx + x_off;
                    rsx! {
                        text {
                            x: "{x}",
                            y: "{row_y}",
                            text_anchor: "middle",
                            font_size: "{glyph_size}",
                            "🏠"
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Register the module**

Edit `web/src/components/mod.rs`, add:

```rust
pub mod map_affordance_overlay;
```

- [ ] **Step 4: Wire the overlay into `Map`**

Edit `web/src/components/map.rs`. At the top of the function add a selection signal:

```rust
use crate::components::map_affordance_overlay::MapAffordanceOverlay;
use game::areas::Area;
use std::rc::Rc;

let mut selected: Signal<Option<Area>> = use_signal(|| None);
```

Replace the existing per-hex `on_click` body to toggle selection:

```rust
let area_for_click = *area;
let on_click = move |_| {
    selected.with_mut(|s| {
        if *s == Some(area_for_click) {
            *s = None;
        } else {
            *s = Some(area_for_click);
        }
    });
};
```

Inside the per-hex `g { ... }` block, after the existing `text { ... }`, conditionally render the overlay when this hex matches selection:

```rust
if selected.read().is_some() {
    if let Some(ad) = areas.iter().find(|ad| ad.area == Some(*area)) {
        MapAffordanceOverlay {
            cx: cx,
            cy: cy,
            size: HEX_SIZE,
            area: (*ad).clone(),
        }
    }
}
```

(Render the overlay on *every* hex when *any* hex is selected — matches the spec wording "highlight surrounding hexes" — so all affordances within the player's mental field of view appear at once. If you'd rather only highlight neighbors of the selected hex, file that as a follow-up.)

Add a visible "selected" outline on the polygon when this hex is the selected one:

```rust
let is_selected = selected.read().as_ref() == Some(area);
let stroke_class = if is_selected {
    "stroke-emerald-400"
} else {
    "stroke-stone-700"
};
// apply via class on the polygon
```

- [ ] **Step 5: Build CSS and compile**

Run: `just build-css`
Run: `cd web && cargo check`
Expected: clean.

- [ ] **Step 6: Manual smoke**

Run `just dev`. Open a game's map. Click a hex; confirm:
- Selected hex gets an emerald outline.
- Every hex with non-zero water_source / forage_richness / shelter_quality (≥ 2) shows the corresponding glyph(s).
- Clicking the same hex again clears selection and removes overlays.

- [ ] **Step 7: Commit**

```bash
jj describe -m "feat(web): map terrain affordance overlay on tribute hex select

Clicking a hex on the game map toggles selection. While any hex is
selected, every hex renders 💧 (water_source > 0), 🌿 (forage_richness > 0),
and 🏠 (shelter_quality >= 2) glyphs to make terrain affordances legible.

Refs: hangrier_games-0yz"
```

---

## Task 7: Final quality pass + bookmark + PR

- [ ] **Step 1: Format and lint**

Run: `just fmt`
Run: `just quality`
Expected: clean — formatter no-op, no clippy warnings, all tests pass.

- [ ] **Step 2: CSS rebuild**

Run: `just build-css`
Expected: clean.

- [ ] **Step 3: Manual end-to-end smoke**

Run `just dev`. With a fresh game:
1. Verify game tributes list cards stay clean (no pips) for healthy tributes.
2. Push the game forward several phases. Confirm pips appear on hungry/thirsty tributes.
3. Open a tribute detail page; confirm the Survival section renders correctly.
4. Look at the timeline; confirm `HungerBandChanged` and `ThirstBandChanged` cards appear at band crossings, that starvation/dehydration deaths render with the new cause copy.
5. Click a hex on the map; confirm affordance overlays appear and clear correctly.

- [ ] **Step 4: Open the PR**

```bash
jj git fetch
jj rebase -d main@origin
bd backup export-git --branch beads-backup
jj bookmark create feat-shelter-hunger-frontend -r @-
jj git push --bookmark feat-shelter-hunger-frontend
gh pr create --base main --head feat-shelter-hunger-frontend \
  --title "feat(web): shelter + hunger/thirst frontend (PR2)" \
  --body "$(cat <<'EOF'
## Summary

Frontend slice of the shelter + hunger/thirst spec. Pairs with PR1 (backend, already merged).

- Generic TributeStateStrip component renders survival pips (🍗 hunger, 💧 thirst, 🏠 shelter) on tribute cards; future emotion frontend slots its pips into the same strip
- TributeSurvivalSection on tribute detail page exposes hunger/thirst counters, bands, shelter status, and the escalating-drain line when relevant
- New SurvivalCard handles HungerBandChanged / ThirstBandChanged / ShelterSought / Foraged / Drank / Ate timeline events; death cards style starvation/dehydration causes
- Map terrain affordance overlay: clicking a hex reveals 💧 / 🌿 / 🏠 glyphs across the map so terrain choices are legible at a glance

## Spec
docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md

## Verification
- `just quality` — clean
- `just build-css` — clean
- Manual smoke: pips appear/disappear correctly across band crossings, drain line shows when Starving/Dehydrated, timeline cards render, map overlays toggle on selection

## Follow-ups
- hangrier_games-ex3f (resource sharing between allied tributes)
- hangrier_games-xfi (announcer prompts consume new survival events)
- (consider filing) Survival filter chip in timeline (Task 5 was optional)
- (consider filing) Successful SeekShelter glyph/animation on the hex (spec open question)
EOF
)"
```

- [ ] **Step 5: Update beads**

```bash
bd update hangrier_games-0yz --status closed --notes "Completed: Plan 1 + Plan 2 merged. PR2: <PR URL>"
```

(Now `hangrier_games-0yz` can close — the spec's full v1 scope is shipped.)

---

## Self-Review

**Spec coverage check (frontend section of the spec):**
- Tribute card — bottom state strip with hunger/thirst pips + house glyph: Tasks 1–2 ✓
- Tribute Inspect drilldown — Survival panel: Task 3 (rendered inline in tribute_detail.rs per the agreed YAGNI scoping) ✓
- Map panel — terrain affordance hints (💧 / 🌿 / 🏠): Task 6 ✓
- Action panel — band-change and death event lines: Task 4 ✓
- Sponsor UI affordance — forward-compat only, no v1 work needed ✓
- Accessibility — aria-labels on pips, text label paired with color: covered in Task 1 ✓
- Open frontend questions (seek-shelter glyph, low-supply toast): explicitly deferred to follow-ups in PR description ✓

**Placeholder scan:** all RSX blocks and component implementations are concrete; pseudo-shapes (e.g. "adapt to actual loop variable names") are clearly labeled where the engineer must read the existing site first. No `TBD` / `TODO` / "implement appropriately" placeholders.

**Type consistency check:**
- `TributeStateStrip` props: `tribute: Tribute, current_phase: Option<u32>` — same in Tasks 1, 2.
- `TributeSurvivalSection` props: `tribute: Tribute, current_phase: Option<u32>` — Task 3.
- `MapAffordanceOverlay` props: `cx: f64, cy: f64, size: f64, area: AreaDetails` — Task 6.
- `SurvivalCard` props: `payload: MessagePayload` — Task 4.
- `MessagePayload` variants used (`HungerBandChanged`, `ThirstBandChanged`, `ShelterSought`, `Foraged`, `Drank`, `Ate`) match exactly the variants Plan 1 Task 11 introduces.
- Pure functions used (`hunger_band`, `thirst_band`, `shelter_quality`, `forage_richness`, `water_source`, `current_weather`) match exactly the Plan 1 module paths.
- Tribute fields accessed (`hunger`, `thirst`, `sheltered_until`, `starvation_drain_step`, `dehydration_drain_step`) match exactly Plan 1 Task 4.
