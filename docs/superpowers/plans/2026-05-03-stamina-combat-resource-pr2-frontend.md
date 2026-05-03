# Stamina-as-Combat-Resource PR2 — Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface PR1's stamina backend in the Dioxus frontend: a stamina bar in the tribute detail page, Winded / Exhausted pips in the existing `TributeStateStrip`, a `StaminaCard` component for `StaminaBandChanged` timeline events, and dispatch wiring inside the existing `MessageKind::State` arm of `event_card.rs`. No new shared types or routing — PR1 already routed `StaminaBandChanged` through `MessageKind::State`.

**Architecture:** Mirror the shelter PR2 pattern exactly: extend `TributeStateStrip` with two new pip components (`WindedPip`, `ExhaustedPip`); add stamina bar to `tribute_detail.rs` next to HP / sanity; create `StaminaCard` mirroring `SurvivalCard`'s structure; extend the existing `SurvivalCard` dispatch arm in `event_card.rs` (or add `StaminaCard` as a sibling under the same `MessageKind::State` match). All Tailwind classes use existing palette tokens (amber for Winded, red for Exhausted, green for recovery) plus the `theme2:` dark variant.

**Tech Stack:** Dioxus 0.7 (Rust → WASM), Tailwind CSS via the existing build pipeline. New `StaminaBand` enum from PR1 is re-exported through `shared::messages::StaminaBand` and reachable from `web/`.

**Spec:** `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`
**Backend plan (predecessor):** `docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr1-backend.md`
**Beads issue:** `hangrier_games-93m`

---

## Pre-flight Notes

- **PR2 must merge after PR1.** It imports `shared::messages::StaminaBand`, the `StaminaBandChanged` payload variant, and the `attacker_stamina_cost` / `target_stamina_cost` fields on `CombatBeat`.
- **Existing `TributeStateStrip` lives at `web/src/components/tribute_state_strip.rs`** with `HungerPip` / `ThirstPip` / `ShelterPip` already shipped (shelter PR2). PR2 adds `WindedPip` and `ExhaustedPip` next to them — mechanical extension, no refactor needed.
- **`tribute_detail.rs:347+`** has the `TributeAttributes` block listing Health / Sanity as `dl` items. The stamina bar lands in the same block as a row.
- **`event_card.rs`** already routes the band-changed survival events to `SurvivalCard` inside the `MessageKind::State` arm at line ~51-57. PR2 either (a) extends `SurvivalCard` to render `StaminaBandChanged` too, or (b) creates a new `StaminaCard` and adds it as a sibling arm. **Plan picks option (b)** — separate component keeps stamina-specific copy and styling out of the survival card and matches the spec's file structure (`web/src/components/timeline/cards/stamina_card.rs`).
- **Open question from spec section "Stamina-cost rendering on swing cards"**: PR2 picks **option B** (don't render stamina cost on swing cards) for v1. Reasoning: the swing card is already information-dense after the combat-wire redesign; per-swing stamina is a backend mechanic and the band events make the consequence visible. Filed as follow-up bead for revisit if playtest disagrees.
- Run `just build-css` after any class changes that introduce new Tailwind utility classes.
- Run `cd web && dx serve` for the live dev loop; smoke tests are manual at the end of each task.
- All commits use the project's jj/git workflow per `AGENTS.md`. Each task ends with `jj describe -m "..."` then `jj new`.
- `bd update <PR2 bead id> --claim` before starting Task 1.

---

## File Structure

**New files:**
- `web/src/components/timeline/cards/stamina_card.rs` — `StaminaCard` component for `StaminaBandChanged`.

**Modified files:**
- `web/src/components/tribute_state_strip.rs` — `WindedPip` + `ExhaustedPip` pip components; extend the `any_visible` guard; render the new pips inside the `flex` row.
- `web/src/components/tribute_detail.rs` — add a stamina bar in the `TributeAttributes` block (or in a sibling `TributeBars` block depending on existing layout). Show `stamina / max_stamina` plus the visible band label.
- `web/src/components/timeline/cards/mod.rs` — register `pub mod stamina_card;` and re-export.
- `web/src/components/timeline/event_card.rs` — extend the `MessageKind::State` `match payload` to dispatch `MessagePayload::StaminaBandChanged { .. }` to `StaminaCard`.

---

## Task Order Rationale

Stamina bar first (most visible, least controversial). Pips second (depends on the bar's color tokens being settled). Timeline card third. Final task is end-to-end smoke + WCAG check + PR.

---

## Task 1: Stamina bar in `tribute_detail.rs`

**Files:**
- Modify: `web/src/components/tribute_detail.rs`

**Goal:** Add a single-row stamina display showing `stamina / max_stamina` plus the band label, matching the visual treatment of the existing Health / Sanity rows. Color token shifts based on band: green Fresh, amber Winded, red Exhausted.

- [ ] **Step 1: Locate the bars block.** Open `web/src/components/tribute_detail.rs` and find `TributeAttributes` (~line 347). Note the existing `dl class: "grid grid-cols-2 gap-4"` with `Health` / `Sanity` / `Movement` rows.

- [ ] **Step 2: Add stamina row** alongside the others. After the `Sanity` row (line ~354):

```rust
            dt { "Stamina" }
            dd {
                StaminaReadout {
                    current: tribute.stamina,
                    max: tribute.max_stamina,
                }
            }
```

Wait — `TributeAttributes` takes `attributes: Attributes` (not a full `Tribute`). Stamina lives on `Tribute`, not `Attributes`. Two options:

- **Option A (preferred):** add a sibling component `TributeStaminaRow` rendered next to `TributeAttributes` in the parent (search for `TributeAttributes {` to find the call site).
- **Option B:** thread `tribute` through `TributeAttributes` props.

Pick A. Find the parent component (likely `TributeDetail`) that renders `TributeAttributes` and insert the stamina row immediately above or below it.

- [ ] **Step 3: Create `StaminaReadout` component** at the bottom of `tribute_detail.rs`:

```rust
#[component]
fn StaminaReadout(current: u32, max: u32) -> Element {
    use shared::messages::StaminaBand;
    let pct = if max == 0 { 0 } else { (current * 100) / max };
    let band = if pct > 50 {
        StaminaBand::Fresh
    } else if pct > 20 {
        StaminaBand::Winded
    } else {
        StaminaBand::Exhausted
    };
    let (color_cls, label) = match band {
        StaminaBand::Fresh => ("text-emerald-600 theme2:text-emerald-300", "Fresh"),
        StaminaBand::Winded => ("text-amber-600 theme2:text-amber-300", "Winded"),
        StaminaBand::Exhausted => ("text-red-600 theme2:text-red-400 font-semibold", "Exhausted"),
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-2 {color_cls}",
            "aria-label": "Stamina: {current} of {max}, band {label}",
            span { "{current} / {max}" }
            span { class: "text-xs uppercase tracking-wide opacity-80", "({label})" }
        }
    }
}
```

(Mirror the rendering of any existing `HealthReadout` / `SanityReadout` if they exist; search `grep -n "HealthReadout\|SanityReadout" web/src/components/tribute_detail.rs` and copy that shape if present. Otherwise the inline component above is fine.)

- [ ] **Step 4: Build and smoke-check:**

```bash
cd web && just build-css && dx serve
```

Open the tribute detail page for a known tribute. Verify:
- Stamina row appears under Sanity.
- Format reads `100 / 100 (Fresh)` for a fresh tribute.
- Color is green for Fresh, amber for Winded, red for Exhausted.

If you can't quickly trigger Winded / Exhausted in dev, manually edit a test fixture or run a few dev cycles with combat enabled to confirm the band switching renders correctly.

- [ ] **Step 5: Run `cargo check --package web`** — expected: clean.

- [ ] **Step 6: Commit:**

```bash
jj describe -m "feat(web): stamina readout row in tribute detail (hangrier_games-93m)"
jj new
```

---

## Task 2: Winded / Exhausted pips in `TributeStateStrip`

**Files:**
- Modify: `web/src/components/tribute_state_strip.rs`

**Goal:** Add two new pip components (`WindedPip`, `ExhaustedPip`) following the exact pattern of `HungerPip` / `ThirstPip` / `ShelterPip`. Glyphs: 💨 for Winded, 🥵 for Exhausted. The `any_visible` guard extends to include the stamina band so a fully-fresh tribute card still renders nothing.

- [ ] **Step 1: Read the current strip** to confirm the exact pattern (the file is already opened in pre-flight; `pub fn TributeStateStrip` is at line 6).

- [ ] **Step 2: Compute stamina band inside the component.** Replace the existing visibility check with a stamina-aware version:

```rust
use dioxus::prelude::*;
use game::tributes::Tribute;
use game::tributes::survival::{HungerBand, ThirstBand, hunger_band, thirst_band};
use shared::messages::StaminaBand;

fn stamina_band_local(stamina: u32, max_stamina: u32) -> StaminaBand {
    if max_stamina == 0 {
        return StaminaBand::Exhausted;
    }
    let pct = (stamina * 100) / max_stamina;
    if pct > 50 {
        StaminaBand::Fresh
    } else if pct > 20 {
        StaminaBand::Winded
    } else {
        StaminaBand::Exhausted
    }
}

#[component]
pub fn TributeStateStrip(tribute: Tribute, current_phase: Option<u32>) -> Element {
    let h_band = hunger_band(tribute.hunger);
    let t_band = thirst_band(tribute.thirst);
    let s_band = stamina_band_local(tribute.stamina, tribute.max_stamina);
    let sheltered_phases_left = match (tribute.sheltered_until, current_phase) {
        (Some(until), Some(now)) if until > now => Some(until - now),
        _ => None,
    };

    let any_visible = h_band != HungerBand::Sated
        || t_band != ThirstBand::Sated
        || s_band != StaminaBand::Fresh
        || sheltered_phases_left.is_some();

    if !any_visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex flex-row gap-2 items-center text-sm select-none",
            if h_band != HungerBand::Sated {
                HungerPip { band: h_band, raw: tribute.hunger }
            }
            if t_band != ThirstBand::Sated {
                ThirstPip { band: t_band, raw: tribute.thirst }
            }
            if s_band != StaminaBand::Fresh {
                StaminaPip { band: s_band, current: tribute.stamina, max: tribute.max_stamina }
            }
            if let Some(left) = sheltered_phases_left {
                ShelterPip { phases_left: left }
            }
        }
    }
}
```

- [ ] **Step 3: Add the `StaminaPip` component** at the bottom of the file, mirroring `HungerPip`:

```rust
#[component]
fn StaminaPip(band: StaminaBand, current: u32, max: u32) -> Element {
    let (glyph, cls, label) = match band {
        StaminaBand::Winded => ("💨", "text-amber-400", "Winded"),
        StaminaBand::Exhausted => ("🥵", "text-red-500 animate-pulse", "Exhausted"),
        StaminaBand::Fresh => return rsx! {},
    };
    rsx! {
        span {
            class: "inline-flex items-center gap-1 {cls}",
            "aria-label": "Stamina: {label}",
            title: "Stamina {current}/{max} — {label}",
            span { class: "text-base", "{glyph}" }
            span { class: "text-xs uppercase tracking-wide", "{label}" }
        }
    }
}
```

- [ ] **Step 4: Run `cargo check --package web`** — expected: clean.

- [ ] **Step 5: Build CSS + smoke:**

```bash
just build-css
cd web && dx serve
```

Verify on a tribute card list page:
- Fresh tribute: no stamina pip (and if also Sated/Sated/no-shelter, the whole strip is empty).
- Winded tribute: amber 💨 pip with "Winded" label.
- Exhausted tribute: red 🥵 pip with pulsing animation.
- Pip slots in horizontally next to existing hunger/thirst pips with the same `gap-2` spacing.

- [ ] **Step 6: Commit:**

```bash
jj describe -m "feat(web): Winded / Exhausted stamina pips in TributeStateStrip"
jj new
```

---

## Task 3: `StaminaCard` component

**Files:**
- Create: `web/src/components/timeline/cards/stamina_card.rs`
- Modify: `web/src/components/timeline/cards/mod.rs`

**Goal:** New `StaminaCard` component renders `StaminaBandChanged` payloads. Border colors per spec: amber Winded, red Exhausted, green for recovery (any band → Fresh, or Exhausted → Winded). Copy:

| Transition | Glyph | Phrase |
|---|---|---|
| any → Winded | 💨 | "{name} is winded" |
| any → Exhausted | 🥵 | "{name} is exhausted" |
| Winded → Fresh, Exhausted → Fresh, Exhausted → Winded | 🌿 | "{name} caught their breath" |

- [ ] **Step 1: Create the file:**

```rust
//! Timeline card for stamina band-change events
//! (`MessagePayload::StaminaBandChanged`).
//! See spec `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`.

use dioxus::prelude::*;
use shared::messages::{GameMessage, MessagePayload};

#[derive(Props, PartialEq, Clone)]
pub struct StaminaCardProps {
    pub message: GameMessage,
}

#[component]
pub fn StaminaCard(props: StaminaCardProps) -> Element {
    let MessagePayload::StaminaBandChanged { tribute, from, to } = &props.message.payload else {
        return rsx! {};
    };

    let direction = transition_direction(from, to);
    let (border_cls, bg_cls, glyph, phrase) = match (direction, to.as_str()) {
        (Direction::Worsening, "Winded") => (
            "border-amber-400",
            "bg-amber-50 theme2:bg-amber-950/40",
            "💨",
            format!("{} is winded.", tribute.name),
        ),
        (Direction::Worsening, "Exhausted") => (
            "border-red-500",
            "bg-red-50 theme2:bg-red-950/40",
            "🥵",
            format!("{} is exhausted.", tribute.name),
        ),
        (Direction::Recovery, _) => (
            "border-emerald-400",
            "bg-emerald-50 theme2:bg-emerald-950/40",
            "🌿",
            format!("{} caught their breath.", tribute.name),
        ),
        // Defensive: unknown transitions fall through to a neutral style.
        _ => (
            "border-stone-400",
            "bg-stone-50 theme2:bg-stone-900/40",
            "•",
            format!("{}: {} → {}", tribute.name, from, to),
        ),
    };

    rsx! {
        article {
            class: "rounded border-l-4 {border_cls} {bg_cls} p-2 text-sm",
            p {
                class: "text-xs text-stone-700 theme2:text-stone-200",
                "{glyph} {phrase}"
                span { class: "text-[10px] text-stone-500 ml-2", "(was {from})" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Direction { Worsening, Recovery, Unknown }

fn transition_direction(from: &str, to: &str) -> Direction {
    // Worse: Fresh -> Winded, Fresh -> Exhausted, Winded -> Exhausted.
    // Better: Winded -> Fresh, Exhausted -> Fresh, Exhausted -> Winded.
    match (from, to) {
        ("Fresh", "Winded") | ("Fresh", "Exhausted") | ("Winded", "Exhausted") => Direction::Worsening,
        ("Winded", "Fresh") | ("Exhausted", "Fresh") | ("Exhausted", "Winded") => Direction::Recovery,
        _ => Direction::Unknown,
    }
}
```

- [ ] **Step 2: Register in `web/src/components/timeline/cards/mod.rs`.** Add `pub mod stamina_card;` alphabetically with the others (`alliance_card`, `combat_card`, ...).

- [ ] **Step 3: Run `cargo check --package web`** — expected: clean.

- [ ] **Step 4: Commit:**

```bash
jj describe -m "feat(web): StaminaCard component for StaminaBandChanged events"
jj new
```

---

## Task 4: Wire `StaminaCard` into `event_card.rs` dispatch

**Files:**
- Modify: `web/src/components/timeline/event_card.rs`

**Goal:** Extend the `MessageKind::State` arm so `StaminaBandChanged` routes to `StaminaCard` while everything else continues to route to `SurvivalCard` / `CycleCard` / `StateCard` as before.

- [ ] **Step 1: Open `web/src/components/timeline/event_card.rs`** and find the `MessageKind::State` arm at line ~51:

```rust
            MessageKind::State => match payload {
                MessagePayload::HungerBandChanged { .. }
                | MessagePayload::ThirstBandChanged { .. }
                | MessagePayload::ShelterSought { .. }
                | MessagePayload::Foraged { .. }
                | MessagePayload::Drank { .. }
                | MessagePayload::Ate { .. } => rsx! { SurvivalCard { message: props.message.clone() } },
                MessagePayload::CycleStart { .. }
                | MessagePayload::CycleEnd { .. }
                | MessagePayload::GameEnded { .. } => rsx! { CycleCard { message: props.message.clone() } },
                _ => rsx! { StateCard { message: props.message.clone() } },
            },
```

- [ ] **Step 2: Add the new arm above the catch-all** and import `StaminaCard`:

```rust
use crate::components::timeline::cards::{
    alliance_card::AllianceCard, combat_card::CombatCard, combat_swing_card::CombatSwingCard,
    cycle_card::CycleCard, death_card::DeathCard, item_card::ItemCard, movement_card::MovementCard,
    stamina_card::StaminaCard, state_card::StateCard, survival_card::SurvivalCard,
};
```

```rust
            MessageKind::State => match payload {
                MessagePayload::StaminaBandChanged { .. } => {
                    rsx! { StaminaCard { message: props.message.clone() } }
                }
                MessagePayload::HungerBandChanged { .. }
                | MessagePayload::ThirstBandChanged { .. }
                | MessagePayload::ShelterSought { .. }
                | MessagePayload::Foraged { .. }
                | MessagePayload::Drank { .. }
                | MessagePayload::Ate { .. } => rsx! { SurvivalCard { message: props.message.clone() } },
                MessagePayload::CycleStart { .. }
                | MessagePayload::CycleEnd { .. }
                | MessagePayload::GameEnded { .. } => rsx! { CycleCard { message: props.message.clone() } },
                _ => rsx! { StateCard { message: props.message.clone() } },
            },
```

- [ ] **Step 3: Run `cargo check --package web`** — expected: clean.

- [ ] **Step 4: Smoke test.** Run `just build-css && cd web && dx serve`. Find a game in the timeline view that has had stamina-band events. Verify:
- Winded transition cards have amber left border with 💨.
- Exhausted transition cards have red left border with 🥵.
- Recovery cards (Winded→Fresh / Exhausted→Fresh / Exhausted→Winded) have green border with 🌿.
- Cards interleave correctly with hunger/thirst band events in the timeline.

If no live game has stamina events yet, manually inject a test event into a saved game JSON, reload, and confirm.

- [ ] **Step 5: Commit:**

```bash
jj describe -m "feat(web): dispatch StaminaBandChanged to StaminaCard in event_card.rs"
jj new
```

---

## Task 5: Final pass — WCAG, smoke, PR

**Files:** none (verification + PR creation)

**Goal:** Verify accessibility, run all quality gates, hand off PR2 with a clean PR description.

- [ ] **Step 1: WCAG color contrast check.** Tailwind tokens used:
- `text-emerald-600` / `text-emerald-300` (Fresh): contrast OK on white / dark backgrounds (existing palette pattern).
- `text-amber-400` / `text-amber-600` (Winded): borderline at 400 on white; mitigated by the bold "Winded" label and the 💨 glyph providing redundant signal.
- `text-red-500` / `text-red-600` (Exhausted): high contrast.

If a contrast checker (e.g. `pa11y`, browser devtools) flags `text-amber-400` on `bg-amber-50`, swap to `text-amber-700` for `light` and keep `text-amber-300` for `theme2:`. Run the contrast tool against a deployed dev page if available; otherwise manual visual check is acceptable for v1.

- [ ] **Step 2: Manual smoke checklist:**
- [ ] Fresh tribute: detail page shows green stamina readout, no pip in strip.
- [ ] Winded tribute: detail page shows amber stamina readout, 💨 pip in strip.
- [ ] Exhausted tribute: detail page shows red stamina readout, 🥵 pulsing pip in strip.
- [ ] Timeline shows StaminaCard for each band-change event with correct color and copy.
- [ ] Filter by tribute (existing per-tribute timeline filter): stamina events for that tribute appear; for others they don't.
- [ ] Mobile-width viewport: pip strip wraps cleanly without overlap; stamina readout remains legible.
- [ ] Both `light` and `theme2:` variants render correctly.

- [ ] **Step 3: Run quality gates:**

```bash
just fmt
just build-css
cargo check --workspace
cargo clippy --workspace -- -D warnings
just test
```

Expected: all clean.

- [ ] **Step 4: Self-review checklist:**
- [ ] All 5 tasks committed; `jj log -r 'main..@'` shows 5 commits with `feat(web): ...` messages.
- [ ] No backend changes in this PR — `git diff main..@ -- game/ shared/ api/` is empty (PR1 already landed all of those).
- [ ] Imports are alphabetised in `mod.rs` and `event_card.rs`.
- [ ] `StaminaPip` follows the exact `aria-label` and `title` pattern of the existing pips.
- [ ] `StaminaCard` border colors match the survival card palette family.
- [ ] No new Tailwind classes that weren't already in the JIT — if any new ones introduced, `just build-css` was re-run.

- [ ] **Step 5: Push branch + open PR.** From the project root (main repo, not specs worktree):

```bash
jj git fetch
jj rebase -d main@origin
jj bookmark create stamina-pr2-frontend -r @-
jj git push --bookmark stamina-pr2-frontend --allow-new
gh pr create --base main --head stamina-pr2-frontend \
  --title "feat(web): stamina-as-combat-resource frontend (hangrier_games-93m)" \
  --body "$(cat <<'EOF'
## Summary

PR2 of the stamina-as-combat-resource feature (`hangrier_games-93m`). Surfaces PR1's backend (combat stamina drain, fatigue bands, recovery, brain integration) in the Dioxus frontend.

- Adds a stamina readout row to the tribute detail page with band-coloured label.
- Extends `TributeStateStrip` with Winded (💨) and Exhausted (🥵) pips.
- Adds `StaminaCard` for `StaminaBandChanged` timeline events with amber / red / green border treatments.
- Wires dispatch inside the existing `MessageKind::State` arm of `event_card.rs`.

No backend changes — PR1 already added `StaminaBand`, the `StaminaBandChanged` payload variant, and `MessageKind::State` routing.

## Spec / Plan
- Spec: `docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md`
- Plan: `docs/superpowers/plans/2026-05-03-stamina-combat-resource-pr2-frontend.md`

## Verification

```
just fmt
just build-css
cargo check --workspace
cargo clippy --workspace -- -D warnings
just test
```

Manual smoke (light + theme2):
- Fresh tribute: green readout, no pip.
- Winded: amber readout + 💨 pip + amber-bordered timeline card.
- Exhausted: red readout + 🥵 pip + red-bordered timeline card.
- Recovery: green-bordered 🌿 card.

## Follow-ups

- Stamina-cost rendering on swing cards (open question A vs B from spec) — chose B for v1; revisit if playtest disagrees.
- WCAG contrast pass on `text-amber-400` deserves a follow-up audit when the spectator-skin work updates the palette.

## Beads
Closes the PR2 child of `hangrier_games-93m`.
EOF
)"
```

- [ ] **Step 6: Close the PR2 implementation bead** once the PR is approved and merged:

```bash
bd close <PR2-bead-id> --reason "Merged in PR #<n>"
```

If both PR1 and PR2 are merged, also close the parent bead `hangrier_games-93m`:

```bash
bd close hangrier_games-93m --reason "PR1 #<n1> + PR2 #<n2> shipped"
```

- [ ] **Step 7: Hand-off summary** to the user with PR URL and any deferred items (notably: stats reporting follow-up bead, if relevant; stamina-cost-on-swing-card decision filed; WCAG contrast follow-up if applicable).

---

## Open Questions for PR Review

- Stamina-cost rendering on swing cards: PR2 chose B (don't render). Reviewer may overrule.
- The `transition_direction` helper duplicates band-ordering logic that already lives in `StaminaBand` enum. Acceptable, or refactor into a `From<(&str, &str)>` on a shared helper?
- Should the recovery card also distinguish "Exhausted → Winded" (still tired but recovering) from "→ Fresh" (fully recovered)? Current implementation collapses both to a single 🌿 card; a more nuanced phrasing could split them.
- WCAG audit on `text-amber-400`: defer to spectator-skin work or fix in this PR?
