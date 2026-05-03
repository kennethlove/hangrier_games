# Gamemaker Event System — Plan 2: Frontend Implementation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface the gamemaker backend (PR1) in the Dioxus frontend: three new hex-map markers (mutt swarm pin, sealed-area shading, convergence point pin), and generic timeline cards for the 11 new gamemaker `MessagePayload` variants. No bespoke per-payload card layouts — every variant routes through one `GamemakerCard` component that uses category icon + stringified payload fields.

**Architecture:** Add a `MessageKind::Gamemaker` variant to keep dispatch clean (the 11 variants don't fit existing categories). All map markers are SVG layers added to `web/src/components/map.rs`, sourced from `gamemaker.active_effects` exposed by the existing `Game` JSON (the `Gamemaker` struct from PR1 already serializes via `serde`). One generic `GamemakerCard` renders all 11 payloads via category-tagged metadata; matches the fallback-card pattern already used by `state_card.rs` for typed `AreaEvent` events. **No** Capitol Feed gauge panel — gauges remain spectator-hidden in v1 per spec.

**Tech Stack:** Dioxus 0.7 (Rust → WASM), Tailwind CSS via the existing build pipeline, `dioxus-query` for fetching, `gloo-storage` for any persisted UI state. All shared types reach the frontend via the `game::` and `shared::` re-imports already in use.

**Spec:** `docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md`
**Backend plan (predecessor):** `docs/superpowers/plans/2026-05-03-gamemaker-event-system-pr1-backend.md`
**Beads issue:** `hangrier_games-5q9`

---

## Pre-flight Notes

- Plan 2 must merge after Plan 1 (PR1 backend) lands. The frontend imports new fields, payload variants, and the `gamemaker.active_effects` data structure that PR1 introduces.
- `web::` already re-imports `game::` types directly — Plan 1's `Gamemaker` struct (with `active_effects: Vec<ActiveIntervention>`) flows through the existing `Game` JSON contract automatically. No DTO bridging required.
- Same applies to `MessagePayload` variants from Plan 1 — they flow through the existing `GameMessage` / `MessagePayload` re-imports already used by the timeline cards.
- The 11 new variants need a routing destination. PR1 leaves `MessagePayload::kind()` returning fallback values. PR2 Task 1 fixes this by adding `MessageKind::Gamemaker` and routing all 11 variants to it. This is a `shared/` change PR2 owns end-to-end.
- Run `just build-css` after any class changes that introduce new Tailwind utility classes (so the JIT pulls them).
- Run `cd web && dx serve` for the live dev loop; smoke tests are manual at the end of each task.
- All commits use the project's jj/git workflow per `AGENTS.md`.

---

## File Structure

**New files:**
- `web/src/components/map_gamemaker_overlay.rs` — three SVG marker layers (mutt swarm pin, sealed-area shading, convergence point pin), rendered when `Gamemaker.active_effects` is non-empty.
- `web/src/components/timeline/cards/gamemaker_card.rs` — single generic timeline card for all 11 new gamemaker payloads.

**Modified files:**
- `shared/src/messages.rs` — add `MessageKind::Gamemaker`; route the 11 new payloads to it in `MessagePayload::kind()`; extend the `MessageKind` round-trip test fixture.
- `web/src/components/mod.rs` — register `map_gamemaker_overlay`.
- `web/src/components/map.rs` — accept `gamemaker_active_effects: Vec<ActiveIntervention>` prop (or read from `Game`); render the overlay layer above the existing affordance overlay.
- `web/src/components/timeline/cards/mod.rs` — re-export `GamemakerCard`.
- `web/src/components/timeline/event_card.rs` — add `MessageKind::Gamemaker => GamemakerCard { ... }` arm.
- `web/src/components/timeline/filters.rs` — add a Gamemaker filter chip if filters are category-based.
- `web/src/components/game_period_page.rs` (or wherever `Map` is rendered) — pass the new `gamemaker_active_effects` prop.

---

## Task Order Rationale

Land the routing wiring first (`MessageKind::Gamemaker`) so subsequent timeline-card work has a real dispatch target. Then build the generic timeline card. Then add map markers, smallest-first (mutt swarm pin → sealed shading → convergence pin). End with WCAG check, visual smoke, self-review, PR.

---

## Task 1: Route 11 new payloads to `MessageKind::Gamemaker`

**Files:**
- Modify: `shared/src/messages.rs`

- [ ] **Step 1: Write failing test** in `shared/src/messages.rs` (append to the existing `#[cfg(test)] mod tests` block):

```rust
#[test]
fn gamemaker_kind_round_trip() {
    let kinds = [
        MessageKind::Death,
        MessageKind::Combat,
        MessageKind::CombatSwing,
        MessageKind::Alliance,
        MessageKind::Movement,
        MessageKind::Item,
        MessageKind::State,
        MessageKind::Gamemaker,
    ];
    for k in kinds {
        let s = serde_json::to_string(&k).unwrap();
        let back: MessageKind = serde_json::from_str(&s).unwrap();
        assert_eq!(k, back);
    }
}

#[test]
fn fireball_strike_routes_to_gamemaker_kind() {
    let p = MessagePayload::FireballStrike {
        area: AreaRef { id: "a1".into(), name: "Forest".into() },
        severity_label: "Major".into(),
        victims: vec![],
        survivors: vec![],
    };
    assert_eq!(p.kind(), MessageKind::Gamemaker);
}

#[test]
fn all_eleven_gamemaker_payloads_route_to_gamemaker_kind() {
    use shared::messages::{Lure, DespawnReason};
    let area = AreaRef { id: "a".into(), name: "A".into() };
    let tref = TributeRef { id: "t".into(), name: "T".into() };
    let cases: Vec<MessagePayload> = vec![
        MessagePayload::FireballStrike { area: area.clone(), severity_label: "Major".into(), victims: vec![], survivors: vec![] },
        MessagePayload::MuttSwarmSpawned { area: area.clone(), kind_label: "Wolf".into(), members: 4 },
        MessagePayload::MuttSwarmAttack { area: area.clone(), kind_label: "Wolf".into(), victim: tref.clone(), damage: 20, killed: false },
        MessagePayload::MuttSwarmDespawned { area: area.clone(), kind_label: "Wolf".into(), reason: DespawnReason::Morning },
        MessagePayload::ForceFieldShifted { closed: vec![area.clone()], opened: vec![], warning_phases: 1 },
        MessagePayload::AreaSealed { area: area.clone(), expires_at_phase: 8 },
        MessagePayload::AreaUnsealed { area: area.clone() },
        MessagePayload::AreaSealEntryDamage { area: area.clone(), tribute: tref.clone(), damage: 10 },
        MessagePayload::ConvergencePointAnnounced { area: area.clone(), lure: Lure::Feast, starts_at_phase: 5 },
        MessagePayload::ConvergencePointExpired { area: area.clone(), lure: Lure::Feast, claimed_by: vec![] },
        MessagePayload::WeatherOverridden { area: area.clone(), weather_label: "HeavyRain".into(), duration_phases: 2 },
    ];
    assert_eq!(cases.len(), 11);
    for p in cases {
        assert_eq!(p.kind(), MessageKind::Gamemaker, "{:?} should route to Gamemaker", p);
    }
}
```

- [ ] **Step 2: Run; expected fail** (compile error: `MessageKind::Gamemaker` doesn't exist).

```bash
cargo test --package shared gamemaker_kind -- --nocapture
```

- [ ] **Step 3: Add the variant.** In `shared/src/messages.rs`, extend `MessageKind`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    Death,
    Combat,
    CombatSwing,
    Alliance,
    Movement,
    Item,
    State,
    Gamemaker,  // <-- add
}
```

- [ ] **Step 4: Wire `kind()` for the 11 payloads.** In the `impl MessagePayload { pub fn kind(&self) -> MessageKind { ... } }` match, add an arm grouping all 11 variants together before the existing `_ => MessageKind::State` fallback:

```rust
| FireballStrike { .. }
| MuttSwarmSpawned { .. }
| MuttSwarmAttack { .. }
| MuttSwarmDespawned { .. }
| ForceFieldShifted { .. }
| AreaSealed { .. }
| AreaUnsealed { .. }
| AreaSealEntryDamage { .. }
| ConvergencePointAnnounced { .. }
| ConvergencePointExpired { .. }
| WeatherOverridden { .. } => MessageKind::Gamemaker,
```

(Read the existing match body in `shared/src/messages.rs` around line 261 first; insert the new arm in the same style as the existing arms.)

- [ ] **Step 5: Update the existing `MessageKind` round-trip test fixture array** (around line 482-490) to include `MessageKind::Gamemaker`. The pre-existing test loops over a fixed list; without the new variant present the test passes but coverage is incomplete.

- [ ] **Step 6: Run shared tests:**

```bash
cargo test --package shared -- --nocapture
```

Expected: all three new tests pass; existing tests unchanged.

- [ ] **Step 7: Workspace check.** The `MessageKind` enum is used in exhaustive matches downstream — adding the variant may break `web/` and `api/`:

```bash
cargo check --workspace
```

If matches break, add `MessageKind::Gamemaker => { /* handled in Task 2 */ }` arms to any non-exhaustive matches. Locate via:

```bash
grep -rn "match.*MessageKind\|MessageKind::" web/ api/
```

Add minimal stub arms — Task 2 will replace any web-side stub with the real `GamemakerCard`.

- [ ] **Step 8: Commit:**

```bash
jj describe -m "feat(shared): add MessageKind::Gamemaker and route 11 new payloads

The gamemaker PR1 backend introduced 11 new MessagePayload variants for
storyteller events (fireball, mutt swarm, force-field shift, area seal,
convergence point, weather override). They were left routed via the
default state-card fallback. Add a dedicated MessageKind::Gamemaker
variant and route all 11 to it so PR2 can dispatch them to a typed
GamemakerCard component.

Refs: hangrier_games-5q9"
jj new
```

---

## Task 2: Generic `GamemakerCard` timeline component

**Files:**
- Create: `web/src/components/timeline/cards/gamemaker_card.rs`
- Modify: `web/src/components/timeline/cards/mod.rs`
- Modify: `web/src/components/timeline/event_card.rs`

- [ ] **Step 1: Read the existing card pattern for style reference**

```bash
cat web/src/components/timeline/cards/state_card.rs
cat web/src/components/timeline/cards/movement_card.rs
```

Confirm the pattern: a `#[component] pub fn FooCard(props: FooCardProps) -> Element` returning an `article` with theme-aware Tailwind classes (`bg-gray-50 theme2:bg-gray-900` etc.) and a leading emoji + body text.

- [ ] **Step 2: Create the card** at `web/src/components/timeline/cards/gamemaker_card.rs`:

```rust
use dioxus::prelude::*;
use shared::messages::{DespawnReason, GameMessage, Lure, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct GamemakerCardProps {
    pub message: GameMessage,
}

/// Single generic card for all 11 gamemaker payload variants.
///
/// Each variant maps to:
/// - A category icon (lethal=🔥, disruptive=⚡, convergence=⭐, atmospheric=☁️)
/// - A border color matching the category
/// - A stringified payload body
#[component]
pub fn GamemakerCard(props: GamemakerCardProps) -> Element {
    let (icon, border, body) = match &props.message.payload {
        // --- Lethal ---
        MessagePayload::FireballStrike { area, severity_label, victims, survivors } => (
            "🔥",
            "border-red-500",
            format!(
                "Fireball ({}) hits {}: {} killed, {} survived",
                severity_label,
                area.name,
                victims.len(),
                survivors.len(),
            ),
        ),
        MessagePayload::MuttSwarmSpawned { area, kind_label, members } => (
            "🐺",
            "border-red-500",
            format!("{} swarm of {} spawns in {}", kind_label, members, area.name),
        ),
        MessagePayload::MuttSwarmAttack { area, kind_label, victim, damage, killed } => {
            let suffix = if *killed { " (KILLED)" } else { "" };
            (
                "🐺",
                "border-red-500",
                format!(
                    "{} swarm in {} attacks {} for {} damage{}",
                    kind_label, area.name, victim.name, damage, suffix
                ),
            )
        }
        MessagePayload::MuttSwarmDespawned { area, kind_label, reason } => {
            let why = match reason {
                DespawnReason::Morning => "morning rollover",
                DespawnReason::NoTargetsNearby => "no targets nearby",
                DespawnReason::NoMembersLeft => "all members killed",
            };
            (
                "🐺",
                "border-stone-500",
                format!("{} swarm in {} despawns ({})", kind_label, area.name, why),
            )
        }
        // --- Disruptive ---
        MessagePayload::ForceFieldShifted { closed, opened, warning_phases } => {
            let closed_names: Vec<String> = closed.iter().map(|a| a.name.clone()).collect();
            let opened_names: Vec<String> = opened.iter().map(|a| a.name.clone()).collect();
            (
                "⚡",
                "border-amber-500",
                format!(
                    "Force field shifts: closing [{}], opening [{}] ({}-phase warning)",
                    closed_names.join(", "),
                    opened_names.join(", "),
                    warning_phases,
                ),
            )
        }
        MessagePayload::AreaSealed { area, expires_at_phase } => (
            "⚡",
            "border-amber-500",
            format!("{} sealed (until phase {})", area.name, expires_at_phase),
        ),
        MessagePayload::AreaUnsealed { area } => (
            "⚡",
            "border-stone-500",
            format!("{} unsealed", area.name),
        ),
        MessagePayload::AreaSealEntryDamage { area, tribute, damage } => (
            "⚡",
            "border-amber-500",
            format!("{} caught in {} seal: -{} HP", tribute.name, area.name, damage),
        ),
        // --- Convergence ---
        MessagePayload::ConvergencePointAnnounced { area, lure, starts_at_phase } => {
            let lure_label = match lure { Lure::Feast => "Feast" };
            (
                "⭐",
                "border-yellow-500",
                format!("{} announced at {} (starts phase {})", lure_label, area.name, starts_at_phase),
            )
        }
        MessagePayload::ConvergencePointExpired { area, lure, claimed_by } => {
            let lure_label = match lure { Lure::Feast => "Feast" };
            let claimants: Vec<String> = claimed_by.iter().map(|t| t.name.clone()).collect();
            let body_str = if claimants.is_empty() {
                format!("{} at {} expires (unclaimed)", lure_label, area.name)
            } else {
                format!(
                    "{} at {} expires; claimed by {}",
                    lure_label, area.name, claimants.join(", ")
                )
            };
            ("⭐", "border-stone-500", body_str)
        }
        // --- Atmospheric ---
        MessagePayload::WeatherOverridden { area, weather_label, duration_phases } => (
            "☁️",
            "border-blue-500",
            format!("{} weather forced to {} for {} phases", area.name, weather_label, duration_phases),
        ),
        // Defensive default — should be unreachable since dispatch in
        // event_card.rs only routes Gamemaker-kind payloads here.
        _ => ("🎬", "border-gray-400", "gamemaker event".to_string()),
    };

    rsx! {
        article {
            class: "rounded border-l-4 {border} bg-gray-50 theme2:bg-gray-900 p-2 text-sm",
            "{icon} {body}"
        }
    }
}
```

- [ ] **Step 3: Re-export in `web/src/components/timeline/cards/mod.rs`:**

Add the line (alphabetic position):

```rust
pub mod gamemaker_card;
```

(Match the existing module-declaration style in the file.)

- [ ] **Step 4: Wire dispatch** in `web/src/components/timeline/event_card.rs`. Add the import:

```rust
use crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, combat_swing_card::CombatSwingCard,
    death_card::DeathCard, gamemaker_card::GamemakerCard, item_card::ItemCard,
    movement_card::MovementCard, state_card::StateCard, survival_card::SurvivalCard,
};
```

Add a new match arm for `MessageKind::Gamemaker` (place it before the closing `}` of the `match kind { ... }` block, alongside the other kind arms):

```rust
MessageKind::Gamemaker => rsx! { GamemakerCard { message: props.message.clone() } },
```

- [ ] **Step 5: Compile**

```bash
cd web && cargo check
```

Expected: clean. If a stub arm was added in Task 1 Step 7 referencing `MessageKind::Gamemaker`, replace it with the dispatch above.

- [ ] **Step 6: Manual smoke**

```bash
cd web && dx serve
```

Open a game far enough into a run that gamemaker events have fired (or seed a test game). Confirm:
- Fireball, mutt-swarm, area-seal, force-field, convergence, and weather-override events render with the correct emoji + border color.
- No "gamemaker event" fallback string appears.

- [ ] **Step 7: Commit:**

```bash
jj describe -m "feat(web): GamemakerCard for 11 storyteller-event payloads

Generic timeline card renders all 11 new gamemaker MessagePayload
variants with category icon (🔥 lethal, ⚡ disruptive, ⭐ convergence,
☁️ atmospheric) and stringified payload bodies. Bespoke per-variant
card layouts are filed as a follow-up bead.

Refs: hangrier_games-5q9"
jj new
```

---

## Task 3: Mutt swarm pin map marker

**Files:**
- Create: `web/src/components/map_gamemaker_overlay.rs`
- Modify: `web/src/components/mod.rs`
- Modify: `web/src/components/map.rs`

- [ ] **Step 1: Read existing affordance overlay for style reference**

```bash
cat web/src/components/map_affordance_overlay.rs
```

Note the SVG-positioning convention (cx, cy, size props) and how glyphs are drawn. Mirror that pattern.

- [ ] **Step 2: Read where `Map` gets its data**

```bash
grep -rn "Map {" web/src/components/
grep -rn "rsx! { Map" web/src/
```

Identify the call site (likely `game_period_page.rs` or similar). Note the parent's access to `game.gamemaker.active_effects` — the parent will pass it down as a new prop.

- [ ] **Step 3: Create the overlay** at `web/src/components/map_gamemaker_overlay.rs` (start with mutt swarm only; add the other two markers in Tasks 4 and 5):

```rust
use dioxus::prelude::*;
use game::gamemaker::ActiveIntervention;

#[derive(Props, PartialEq, Clone)]
pub struct MapGamemakerOverlayProps {
    pub cx: f64,
    pub cy: f64,
    pub size: f64,
    pub area_id: String,
    pub active_effects: Vec<ActiveIntervention>,
}

/// Renders gamemaker active-effect markers on a single hex.
/// Currently supports: mutt swarm pin (Task 3).
/// Tasks 4-5 add: sealed-area shading, convergence-point pin.
#[component]
pub fn MapGamemakerOverlay(props: MapGamemakerOverlayProps) -> Element {
    let mutt = props.active_effects.iter().find_map(|eff| match eff {
        ActiveIntervention::MuttSwarm { area_id, kind, members, .. }
            if area_id == &props.area_id =>
        {
            Some((kind.to_string(), *members))
        }
        _ => None,
    });

    rsx! {
        if let Some((kind_label, members)) = mutt {
            // Pin offset upward and to the right of the hex center.
            {
                let px = props.cx + props.size * 0.35;
                let py = props.cy - props.size * 0.35;
                rsx! {
                    g {
                        // Red claw glyph
                        text {
                            x: "{px}",
                            y: "{py}",
                            text_anchor: "middle",
                            dominant_baseline: "central",
                            class: "fill-red-600 select-none pointer-events-none",
                            font_size: "20",
                            "🦊"
                        }
                        // Member count badge
                        text {
                            x: "{px + 10.0}",
                            y: "{py + 10.0}",
                            text_anchor: "middle",
                            dominant_baseline: "central",
                            class: "fill-white select-none pointer-events-none",
                            font_size: "10",
                            font_weight: "bold",
                            title: "{kind_label} swarm: {members} members",
                            "{members}"
                        }
                    }
                }
            }
        }
    }
}
```

(Note: `🦊` is a placeholder claw glyph. If a closer match exists in the project's icon set — e.g. `web/src/components/icons/` — substitute. Browse the icons directory first; if nothing fits, the emoji is acceptable for v1 and a follow-up bead can swap to a custom SVG.)

- [ ] **Step 4: Register the module** in `web/src/components/mod.rs` (alphabetic position):

```rust
pub mod map_gamemaker_overlay;
```

- [ ] **Step 5: Pass the prop into `Map`** — modify `web/src/components/map.rs` signature:

```rust
#[component]
pub fn Map(
    areas: Vec<AreaDetails>,
    #[props(default)] gamemaker_active_effects: Vec<game::gamemaker::ActiveIntervention>,
) -> Element {
```

Inside the per-hex `g { ... }` block (after the existing affordance overlay rendering), add:

```rust
MapGamemakerOverlay {
    cx: cx,
    cy: cy,
    size: HEX_SIZE,
    area_id: area_name.to_lowercase().replace(' ', "-"),
    active_effects: gamemaker_active_effects.clone(),
}
```

Add the import at the top:

```rust
use crate::components::map_gamemaker_overlay::MapGamemakerOverlay;
```

- [ ] **Step 6: Pass the prop from the Map call site.** Find with:

```bash
grep -rn "rsx! { Map" web/src/
```

Modify each call site to include:

```rust
Map {
    areas: areas.clone(),
    gamemaker_active_effects: game.gamemaker.active_effects.clone(),
}
```

(Adjust to actual field-access path.)

- [ ] **Step 7: Compile**

```bash
cd web && cargo check
```

Expected: clean.

- [ ] **Step 8: Manual smoke**

```bash
just build-css
cd web && dx serve
```

Trigger a mutt-swarm event in a running game (force one via test-fixture seed if needed). Confirm:
- A red glyph + small numeric badge appears on the hex where the swarm is active.
- When the swarm despawns, the marker disappears within one phase tick.

- [ ] **Step 9: Commit:**

```bash
jj describe -m "feat(web): mutt swarm pin marker on hex map

Renders a red glyph + member-count badge on hexes where a MuttSwarm
ActiveIntervention is active. Fed by Game.gamemaker.active_effects
populated by PR1.

Refs: hangrier_games-5q9"
jj new
```

---

## Task 4: Sealed-area shading map marker

**Files:**
- Modify: `web/src/components/map_gamemaker_overlay.rs`

- [ ] **Step 1: Extend `MapGamemakerOverlay`** to render sealed-area shading. Below the mutt-swarm rsx block, add a polygon overlay when an `AreaClosure` matches:

```rust
let is_sealed = props.active_effects.iter().any(|eff| matches!(
    eff,
    ActiveIntervention::AreaClosure { area_id, .. } if area_id == &props.area_id
));

// ... inside the rsx! { } below the mutt block:
if is_sealed {
    // Render hex-shape overlay; reuse hex_corners helper from map.rs by
    // accepting a precomputed `points` string as a prop, or duplicate the
    // small helper inline. Simpler: accept points as an optional prop
    // OR compute the polygon path here.
    {
        // Inline hex corners (matches map.rs hex_corners output).
        let mut pts = String::new();
        for i in 0..6 {
            let angle_deg = 60.0 * (i as f64) + 30.0;
            let a = angle_deg.to_radians();
            let x = props.cx + props.size * a.cos();
            let y = props.cy + props.size * a.sin();
            if i > 0 { pts.push(' '); }
            pts.push_str(&format!("{x:.2},{y:.2}"));
        }
        rsx! {
            polygon {
                points: "{pts}",
                class: "fill-red-500/30 stroke-red-700 pointer-events-none",
                stroke_width: "2",
                stroke_dasharray: "4 2",
            }
        }
    }
}
```

- [ ] **Step 2: Hoist `hex_corners` if duplication bothers you** — move the helper from `web/src/components/map.rs` into a shared module (e.g. `web/src/components/hex_geometry.rs`) and import in both `map.rs` and `map_gamemaker_overlay.rs`. Optional refactor; the inline version is acceptable for v1.

- [ ] **Step 3: Compile**

```bash
cd web && cargo check
```

Expected: clean.

- [ ] **Step 4: Manual smoke**

Trigger an `AreaClosure` event. Confirm:
- The closed hex gets a translucent red overlay with dashed red border.
- When the seal expires (PR1 emits `AreaUnsealed`), the overlay disappears within one phase.
- The overlay is `pointer-events-none` so it doesn't break hex clicks.

- [ ] **Step 5: Commit:**

```bash
jj describe -m "feat(web): sealed-area shading on hex map

Adds translucent red overlay with dashed border to hexes under an
AreaClosure ActiveIntervention. Pointer-events disabled so click-to-
select still works through the overlay.

Refs: hangrier_games-5q9"
jj new
```

---

## Task 5: Convergence point pin map marker

**Files:**
- Modify: `web/src/components/map_gamemaker_overlay.rs`

- [ ] **Step 1: Extend `MapGamemakerOverlay`** to render the convergence pin. Below the sealed-area block, add:

```rust
let convergence = props.active_effects.iter().find_map(|eff| match eff {
    ActiveIntervention::ConvergencePoint { area_id, lure, .. }
        if area_id == &props.area_id =>
    {
        Some(lure.clone())
    }
    _ => None,
});

// ... inside the rsx! { }:
if let Some(lure) = convergence {
    let lure_glyph = match lure {
        shared::messages::Lure::Feast => "🍖",
    };
    {
        let px = props.cx - props.size * 0.35;
        let py = props.cy - props.size * 0.35;
        rsx! {
            g {
                text {
                    x: "{px}",
                    y: "{py}",
                    text_anchor: "middle",
                    dominant_baseline: "central",
                    class: "fill-yellow-500 select-none pointer-events-none",
                    font_size: "20",
                    "⭐"
                }
                text {
                    x: "{px + 12.0}",
                    y: "{py + 12.0}",
                    text_anchor: "middle",
                    dominant_baseline: "central",
                    font_size: "12",
                    class: "select-none pointer-events-none",
                    "{lure_glyph}"
                }
            }
        }
    }
}
```

- [ ] **Step 2: Compile**

```bash
cd web && cargo check
```

Expected: clean.

- [ ] **Step 3: Manual smoke**

Trigger a `ConvergencePointAnnounced` event. Confirm:
- A gold star + lure micro-icon (🍖 for Feast) appears on the announced hex.
- When the convergence expires, the marker disappears within one phase.

- [ ] **Step 4: Commit:**

```bash
jj describe -m "feat(web): convergence point pin on hex map

Gold star + lure-specific glyph (🍖 Feast) on hexes hosting a
ConvergencePoint ActiveIntervention. Future Lure variants extend
the glyph match.

Refs: hangrier_games-5q9"
jj new
```

---

## Task 6: WCAG contrast check + accessibility pass

**Files:** none modified directly; remediation may touch Tasks 3-5 outputs.

- [ ] **Step 1: Visually inspect each new map marker against both terrain themes.**

```bash
just build-css
cd web && dx serve
```

Switch between the available Tailwind themes (default + `theme2:` + `theme3:` if present). For each marker:
- Mutt swarm pin: red glyph against stone-200 / stone-400 hex fills — confirm legibility.
- Sealed-area shading: `fill-red-500/30` over closed-hex `fill-red-500` may stack red-on-red and reduce legibility. Verify; if unreadable on closed hexes specifically, consider `fill-amber-500/40` instead.
- Convergence pin: gold star + lure glyph against stone hexes — confirm visibility (the lure glyph defaults to inherit; force `class: "fill-stone-900"` if needed).

- [ ] **Step 2: Add `<title>` accessibility tooltips** to each marker (matches the `title:` attribute already used on the mutt-swarm member badge). Aim for descriptive text screen readers can announce:

```rust
title { "Mutt swarm: {kind_label}, {members} members" }
title { "Area sealed by gamemaker; entering takes damage" }
title { "Convergence point: {lure_label}" }
```

(Use SVG `<title>` child element, not the HTML `title` attribute, since these are SVG nodes.)

- [ ] **Step 3: Confirm screen-reader announcement** with VoiceOver (macOS) or NVDA (Windows) over a representative hex.

- [ ] **Step 4: Commit any remediation:**

```bash
jj describe -m "fix(web): WCAG remediation for gamemaker map markers

- Adjust sealed-area overlay color to maintain contrast on closed
  (already-red) hexes
- Add SVG <title> tooltips for screen-reader announcement

Refs: hangrier_games-5q9"
jj new
```

(Skip the commit if no remediation needed.)

---

## Task 7: Visual integration tests + filter chip + final pass + PR

**Files:**
- Modify: `web/src/components/timeline/filters.rs` (if filters are category-based)

- [ ] **Step 1: Confirm filter chip wiring**

```bash
cat web/src/components/timeline/filters.rs
```

If the existing filters are keyed off `MessageKind` (likely, given `tribute_filter_chips.rs` exists), add a `Gamemaker` chip alongside the others:

```rust
// In the chip list, alongside Death/Combat/Movement/etc:
FilterChip {
    label: "Gamemaker",
    kind: MessageKind::Gamemaker,
    icon: "🎬",
}
```

(Match the existing chip-render pattern; the snippet above is approximate. Read the file first.)

- [ ] **Step 2: Visual integration tests**

If the project uses `#[component]` testing (Dioxus 0.7's `dioxus_testing` or `dioxus-cli`'s component test runner), add tests in `web/src/components/timeline/cards/gamemaker_card.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shared::messages::{AreaRef, GameMessage, MessagePayload, MessageSource};

    fn msg_with(payload: MessagePayload) -> GameMessage {
        GameMessage {
            source: MessageSource::Game("test".into()),
            payload,
            timestamp: 0,
            emit_index: 0,
            day: 0,
            phase: 0,
        }
        // (Adjust to actual GameMessage shape; read shared/src/messages.rs first.)
    }

    #[test]
    fn fireball_strike_renders_fire_emoji() {
        let m = msg_with(MessagePayload::FireballStrike {
            area: AreaRef { id: "a".into(), name: "Forest".into() },
            severity_label: "Major".into(),
            victims: vec![],
            survivors: vec![],
        });
        // If a render-to-string helper exists in the project, use it; otherwise this
        // test ensures the props at minimum compile with the variant shape.
        let _ = GamemakerCardProps { message: m };
    }
}
```

(If no existing render-test infrastructure exists in `web/`, skip this step and rely on manual smoke from Tasks 2-5. File a follow-up bead to add component-render tests if it's a recurring need.)

- [ ] **Step 3: Format and lint**

```bash
just fmt
just quality
```

Expected: clean — formatter no-op, no clippy warnings, all tests pass.

- [ ] **Step 4: CSS rebuild**

```bash
just build-css
```

Expected: clean.

- [ ] **Step 5: Manual end-to-end smoke**

Run `just dev`. With a fresh game, push it forward several days and confirm:
1. As the gamemaker fires interventions, all 11 payload variants render in the timeline with category-correct icons and borders.
2. Filtering by the Gamemaker chip isolates only those events.
3. Mutt-swarm pins, sealed-area shading, and convergence pins appear on the correct hexes and disappear when the underlying active effect resolves.
4. The hex-click selection (terrain affordance overlay from shelter PR2) still works through gamemaker overlays.
5. Console (browser devtools) shows no errors.

- [ ] **Step 6: Open the PR**

```bash
jj git fetch
jj rebase -d main@origin
bd backup export-git --branch beads-backup
jj bookmark create feat-gamemaker-frontend -r @-
jj git push --bookmark feat-gamemaker-frontend
gh pr create --base main --head feat-gamemaker-frontend \
  --title "feat(web): gamemaker event system frontend (PR2)" \
  --body "$(cat <<'EOF'
## Summary

Frontend slice of the gamemaker event system spec. Pairs with PR1 (backend, already merged).

- Adds `MessageKind::Gamemaker` and routes the 11 new gamemaker payloads to it
- Generic `GamemakerCard` renders all 11 storyteller-event variants with category icon (🔥 lethal, ⚡ disruptive, ⭐ convergence, ☁️ atmospheric) and stringified bodies
- Three new hex-map markers fed by `Game.gamemaker.active_effects`:
  - Mutt swarm pin (claw glyph + member count)
  - Sealed-area shading (translucent red overlay with dashed border)
  - Convergence point pin (gold star + lure glyph)
- WCAG-friendly: SVG `<title>` tooltips on markers; pointer-events disabled on overlays so hex selection still works
- Optional Gamemaker filter chip in the timeline filter strip

## Spec
docs/superpowers/specs/2026-05-03-gamemaker-event-system-design.md

## Verification
- `just quality` — clean
- `just build-css` — clean
- Manual smoke: all 11 payloads render correctly; map markers appear/disappear in sync with active effects; hex selection unaffected

## Follow-ups
- Bespoke per-payload card layouts (file as bead) — current generic card is functional but could be richer
- Custom SVG glyph for mutt swarm (currently 🦊 emoji placeholder)
- Capitol Feed gauge panel (deferred per spec — gauges remain spectator-hidden in v1)
- Extra `Lure` variants (water cache, airdrop cluster, capitol summons) and corresponding glyphs
EOF
)"
```

- [ ] **Step 7: Update beads**

```bash
bd update hangrier_games-5q9 --status closed --notes "Completed: Plan 1 + Plan 2 merged. PR2: <PR URL>"
```

(Now `hangrier_games-5q9` can close — the spec's full v1 scope is shipped.)

---

## Self-Review

**Spec coverage check (frontend section of the spec):**
- Hex map — mutt swarm pin: Task 3 ✓
- Hex map — sealed-area shading: Task 4 ✓
- Hex map — convergence point pin: Task 5 ✓
- Timeline — generic rendering for all 11 new payloads: Tasks 1-2 ✓
- Category icons (lethal=flame, disruptive=forcefield-shimmer, convergence=star, atmospheric=cloud): Task 2 ✓
- Stringified payload fields + standard timeline-card chrome: Task 2 ✓
- No Capitol Feed panel (spec explicitly deferred): respected ✓
- Bespoke per-payload card layouts (spec: filed as follow-up): documented in PR description ✓
- WCAG check: Task 6 ✓

**Placeholder scan:** all RSX blocks and component implementations are concrete; pseudo-shapes (e.g. "adapt to actual loop variable names" in Task 7 Step 1) are clearly labeled where the engineer must read the existing site first. No `TBD` / `TODO` / "implement appropriately" placeholders other than the explicitly-flagged 🦊 emoji glyph (with a follow-up note).

**Type consistency check:**
- `GamemakerCardProps`: `message: GameMessage` — same in Tasks 2, 7.
- `MapGamemakerOverlayProps`: `cx: f64, cy: f64, size: f64, area_id: String, active_effects: Vec<ActiveIntervention>` — same in Tasks 3, 4, 5.
- `Map` prop addition: `gamemaker_active_effects: Vec<ActiveIntervention>` — Task 3.
- `MessageKind::Gamemaker` referenced in Tasks 1, 2, 7 — added in Task 1.
- `MessagePayload` variants used (`FireballStrike`, `MuttSwarmSpawned`, `MuttSwarmAttack`, `MuttSwarmDespawned`, `ForceFieldShifted`, `AreaSealed`, `AreaUnsealed`, `AreaSealEntryDamage`, `ConvergencePointAnnounced`, `ConvergencePointExpired`, `WeatherOverridden`) match exactly the 11 variants PR1 Task 5 introduces.
- `ActiveIntervention` variants used (`MuttSwarm { area_id, kind, members, .. }`, `AreaClosure { area_id, .. }`, `ConvergencePoint { area_id, lure, .. }`) match exactly PR1 Task 9.
- `Lure` variants used (`Feast`) match PR1 Task 5.
- `DespawnReason` variants used (`Morning`, `NoTargetsNearby`, `NoMembersLeft`) match PR1 Task 5.

**Known gaps (filed in PR description as follow-ups):**
1. 🦊 emoji as mutt-swarm glyph placeholder — swap to custom SVG when icon-set work is scheduled.
2. Generic GamemakerCard — bespoke per-payload card layouts deferred.
3. Capitol Feed gauge panel — spectator-hidden in v1 per spec; defer to a future spec.
4. Visual integration tests in Task 7 Step 2 are conditional on existing test infrastructure; if absent, manual smoke + future test-infra bead suffices.
5. `hex_corners` duplication between `map.rs` and `map_gamemaker_overlay.rs` is left optional in Task 4 Step 2.

**Backend coupling sanity check:**
- All `ActiveIntervention` field names (`area_id`, `kind`, `members`, `lure`) match PR1 Task 9 exactly. Verified against PR1 plan lines 2400-2470.
- `Animal::to_string()` is used to derive `kind_label` — relies on `Display` impl on `threats::animals::Animal` (already exists per PR1 Task 5 narrative).
- `Game.gamemaker` field path matches PR1 Task 2 — single source of truth for active-effect rendering.
