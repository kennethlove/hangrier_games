# Stamina-as-Combat-Resource — Design

**Beads issue:** `hangrier_games-93m`
**Status:** Spec
**Date:** 2026-05-03
**Predecessors:** Shelter + Hunger/Thirst (`hangrier_games-0yz`), Combat-wire redesign (`hangrier_games-dxp`), Break-mid-swing penalty (commit `ce2c8f1`).

---

## Problem

Combat is currently free. The `stamina` field on `Tribute` (`game/src/tributes/mod.rs:185`) exists with a max of 100 and is drained by movement and most non-combat actions, but the `Action::Attack` branch (`game/src/tributes/mod.rs:548`) calls `attacks()` without subtracting the cost listed in `calculate_stamina_cost` (`mod.rs:852` → `25.0`). The result: tributes can swing infinitely, and the stamina system has no tactical consequence in fights.

Three downstream effects flow from "combat is free":
1. **Survival systems are decorative.** Hunger, thirst, shelter, and rest exist but rarely change a fight's outcome. A starving Exhausted tribute fights identically to a fresh one.
2. **No tactical retreat.** A tribute who is losing has no mechanical reason to flee — the cost of staying is just HP, not stamina.
3. **No visible weakness signaling.** Because there's no fatigue band, a tribute can't tell whether an opponent is gassed. Targeting decisions are blind to the most important "is this fight winnable" signal.

This spec adds combat stamina drain, a Fresh/Winded/Exhausted band system that gates penalties and brain behavior, recovery rules tied to the existing shelter+survival systems, and visible-band predator/prey logic so the bands matter beyond their owner.

## Goals

- Combat actions consume stamina (asymmetric: attacker > target).
- Two visible fatigue bands (Winded, Exhausted) with mechanical penalties.
- Recovery rates that couple cleanly to the existing shelter and hunger/thirst systems.
- Brain override pipeline extended so Winded tributes seek shelter and Exhausted tributes flee.
- Visible bands feed back into other tributes' targeting decisions (predator/prey).
- All combat magic numbers and new stamina constants hoisted to a tunable `CombatTuning` struct.
- Frontend surfaces stamina bar, band pips, and a typed `StaminaBandChanged` timeline event.

## Non-Goals

- Retuning existing combat formulas (`DECISIVE_WIN_MULTIPLIER`, stress contributions). The hoist creates the surface; tuning is a separate post-ship pass (filed as a follow-up bead).
- Property-test coverage of baseline combat behavior. Filed as a separate follow-up bead so it can be specified rigorously without bloating this PR.
- Stamina-aware sponsor gifts (energy drinks, etc.).
- Trait-driven recovery modifiers (Athletic +20%, etc.).
- Hex-map fatigue glow / animation. Revisit during spectator-skin work.

---

## Design

### Constants & tuning struct

A new `CombatTuning` struct in `game/src/tributes/combat.rs` (or a sibling `tuning.rs` file under the tributes module) collects the existing magic numbers plus the new stamina constants. Default values match current behavior so the hoist is mechanically inert.

```rust
pub struct CombatTuning {
    // --- Existing (verbatim from combat.rs:21-26) ---
    pub decisive_win_multiplier: f64,        // 1.5
    pub base_stress_no_engagements: f64,     // 20.0
    pub stress_sanity_normalization: f64,    // 100.0
    pub stress_final_divisor: f64,           // 2.0
    pub kill_stress_contribution: f64,       // 50.0
    pub non_kill_win_stress_contribution: f64, // 20.0

    // --- New: per-swing stamina costs ---
    pub stamina_cost_attacker: u32,          // 25
    pub stamina_cost_target: u32,            // 10

    // --- New: band thresholds (% of max_stamina) ---
    pub band_winded_pct: u8,                 // 50
    pub band_exhausted_pct: u8,              // 20

    // --- New: per-band roll penalties (subtracted from attack/defense rolls) ---
    pub winded_roll_penalty: i32,            // -2
    pub exhausted_roll_penalty: i32,         // -5

    // --- New: per-phase recovery ---
    pub recovery_idle: u32,                  // 5
    pub recovery_resting: u32,               // 30
    pub recovery_sheltered_resting: u32,     // 60
    pub recovery_starving_dehydrated_mult: f64, // 0.5

    // --- New: brain scoring nudges ---
    pub winded_attack_score_penalty: i32,    // -10
    pub fresh_target_visibly_tired_bonus: i32, // +5
}

impl Default for CombatTuning {
    fn default() -> Self { /* values listed above */ }
}
```

`Game` carries one instance: `pub combat_tuning: CombatTuning` (with `#[serde(default)]` so existing saves load unchanged). All combat code reads from this instead of constants.

### Stamina drain on combat

`Action::Attack` resolution in `game/src/tributes/mod.rs:548` is wrapped to deduct attacker cost before calling `attacks()`, and the target's cost is deducted at the call site (or inside `attacks()`, before the contest roll, with a returned-stamina-cost field on `CombatBeat` so the wire stays single-source).

Costs use `saturating_sub` so a tribute who is below cost can still complete a swing already underway (combat is committed once entered) but the attack itself is **gated** at action selection: a tribute below `stamina_cost_attacker` cannot select `Action::Attack`. The brain-side check in `tributes/brains.rs` short-circuits the action-score calculation. (Brain handling for the gated case is in **Brain Integration** below.)

The target cost (`stamina_cost_target`) is paid whether the target acts or not — defending takes work too. A target below their cost still pays via `saturating_sub`, but the additional rule "low stamina applies a roll penalty via band" naturally punishes them.

### Fatigue bands

Three bands derived from `stamina / max_stamina`:

| Band | Threshold (default) | Roll penalty | Visible to others |
|---|---|---|---|
| Fresh | > 50% | 0 | yes |
| Winded | ≤ 50%, > 20% | −2 | yes |
| Exhausted | ≤ 20% | −5 | yes |

A new pure function in `game/src/tributes/survival.rs` (or a new sibling `stamina_band.rs`):

```rust
pub enum StaminaBand { Fresh, Winded, Exhausted }

pub fn stamina_band(stamina: u32, max_stamina: u32, tuning: &CombatTuning) -> StaminaBand {
    let pct = if max_stamina == 0 { 0 } else { (stamina * 100) / max_stamina };
    if pct as u8 > tuning.band_winded_pct { StaminaBand::Fresh }
    else if pct as u8 > tuning.band_exhausted_pct { StaminaBand::Winded }
    else { StaminaBand::Exhausted }
}
```

Roll penalties apply in `combat.rs` to both attacker and defender rolls. The penalty is read from `CombatTuning` and subtracted (not multiplied) so the stat shape stays linear and transparent. Penalties stack with existing modifiers (break-mid-swing forfeit, etc.) — they do not gate them.

### Recovery formula

Recovery runs once per phase per tribute, in `Game::process_turn_phase` (or wherever `restore_stamina` is currently invoked from `Action::Rest`). Formula:

```
base_per_phase =
    if action == Rest && sheltered { tuning.recovery_sheltered_resting }     // 60
    else if action == Rest         { tuning.recovery_resting }                // 30
    else                            { tuning.recovery_idle }                  // 5

multiplier =
    if hunger_band == Starving || thirst_band == Dehydrated {
        tuning.recovery_starving_dehydrated_mult                              // 0.5
    } else { 1.0 }

stamina = (stamina + (base_per_phase as f64 * multiplier).round() as u32).min(max_stamina)
```

`sheltered` here is the boolean from the existing shelter system (`sheltered_until > current_phase`). `hunger_band` / `thirst_band` are the existing pure-function bands from shelter PR1.

The existing `restore_stamina()` (`tributes/lifecycle.rs:179`) becomes either a thin wrapper over the new formula with `action=Rest, sheltered=false, fresh-bands` (so existing tests / call sites still get a meaningful restore) or it is renamed `recover_stamina` and accepts the inputs explicitly. Implementation can pick whichever yields fewer churned tests; leaning toward "rename + plumb arguments" since the old "always full restore" semantics are exactly the over-generous behavior this spec corrects.

### Brain integration

Extends the existing override pipeline in `game/src/tributes/brains.rs`. The order of brain overrides becomes (preserving existing precedence):

1. **Combat preempt** — engaged tributes resolve combat first (unchanged).
2. **Gamemaker overrides** — sealed-area filter, mutt flee, convergence pull (unchanged).
3. **Hunger/thirst overrides** — Eat/Drink when Starving/Dehydrated (unchanged).
4. **Stamina overrides (NEW)** — see below.
5. Standard brain logic.

#### Stamina override rules

- **Fresh:** no override. Standard logic. Apply `fresh_target_visibly_tired_bonus` to attack-action scoring when the candidate target is in a Winded or Exhausted band.
- **Winded:**
  - Apply `winded_attack_score_penalty` to all `Action::Attack` candidate scores. The attack remains *available* — a cornered Winded tribute can still swing — but scoring shifts the brain toward `SeekShelter` and `Rest`.
- **Exhausted:**
  - If a reachable shelter exists (within stamina-affordable hex distance), force `Action::SeekShelter`.
  - Else force `Action::Rest`.
  - **Visible-band flee rule:** if any tribute in the same area or an adjacent area has a *better* visible band than the actor, force `Action::Move` away (toward the highest-cost-distance neighbor that puts more area-edges between actor and the better-banded tribute). This rule overrides the SeekShelter / Rest defaults *only when shelter is not in the actor's own area* — being in shelter beats fleeing.

#### Predator scoring

The Fresh-only `fresh_target_visibly_tired_bonus` adds to the attack-action score when scoring candidate targets. This is a brain-scoring nudge, not a hard gate — a Fresh tribute with a great target choice (high HP, valuable items) still picks their preferred target; the bonus only nudges between roughly-equivalent options. Concretely:

```rust
fn target_attack_score(
    actor: &Tribute, target: &Tribute,
    tuning: &CombatTuning,
) -> i32 {
    let base = /* existing scoring */;
    let actor_band = stamina_band(actor.stamina, actor.max_stamina, tuning);
    let target_band = stamina_band(target.stamina, target.max_stamina, tuning);
    let predator_bonus = if actor_band == StaminaBand::Fresh
        && (target_band == StaminaBand::Winded || target_band == StaminaBand::Exhausted) {
        tuning.fresh_target_visibly_tired_bonus
    } else { 0 };
    base + predator_bonus
}
```

#### Action-gate on insufficient stamina

If `actor.stamina < tuning.stamina_cost_attacker`, the brain treats `Action::Attack` as unavailable (score = `i32::MIN`). This is independent of band — it's a hard mechanic gate. It naturally produces the "Exhausted tribute can't swing" outcome without needing a special-case rule.

### Events

One new typed `MessagePayload` variant in `shared/src/messages.rs`:

```rust
StaminaBandChanged {
    tribute: TributeRef,
    from: StaminaBand,
    to: StaminaBand,
}
```

`StaminaBand` lives in `shared/src/messages.rs` (mirrors `HungerBand` / `ThirstBand` location pattern from shelter PR1):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StaminaBand { Fresh, Winded, Exhausted }
```

`MessagePayload::kind()` routes `StaminaBandChanged` to `MessageKind::State` (alongside `HungerBandChanged` / `ThirstBandChanged` / `TributeStarved` / `TributeDehydrated`). No new `MessageKind` variant needed — this is the same category as the existing survival state events.

### Coexistence with the existing combat code

- `CombatBeat` (the typed swing wire from `hangrier_games-dxp`) gains optional fields `attacker_stamina_cost: u32` and `target_stamina_cost: u32` so the swing's cost is renderable in the timeline / replay if desired. PR1 plumbs the values through; PR2 may or may not surface them on the card body (open question — see below).
- `apply_violence_stress` and the break-mid-swing forfeit logic are unchanged. Stamina penalties are additive to whatever those branches already produce.
- The `restore_stamina` rename is the only breaking change in `lifecycle.rs`. All call sites are local to the game crate; none cross the API/web boundary.

---

## Frontend (PR2 scope)

### Tribute card / detail page

Stamina bar joins HP and sanity bars. Pip strip (`tribute_state_strip`) gets two new pips:

| Pip | Trigger | Glyph |
|---|---|---|
| Winded | `stamina_band == Winded` | 💨 |
| Exhausted | `stamina_band == Exhausted` | 🥵 |

Following the shelter PR2 pattern: pips render with `aria-label`, theme-aware Tailwind classes, and slot into the existing strip's flex layout.

### Timeline card

`StaminaCard` component in `web/src/components/timeline/cards/stamina_card.rs` renders `StaminaBandChanged`:

```
💨 {tribute.name} is winded
🥵 {tribute.name} is exhausted
🌿 {tribute.name} caught their breath  // (recovery: from Winded/Exhausted → Fresh)
```

Routes through `MessageKind::State` (existing dispatch in `event_card.rs`) — no new `MessageKind` arm needed. Border color: amber for Winded transition, red for Exhausted transition, green for recovery.

### Filter chip

If the timeline filter strip is category-based on `MessageKind`, no new chip is needed (covered by existing State filter). If filters are payload-specific, add a "Stamina" chip alongside the existing survival ones.

### Stamina-cost rendering on swing cards (open question)

Should the existing `CombatSwingCard` show the per-swing stamina cost? Two options:
- **A.** Yes, append "(−25 stamina)" to the swing line. Reinforces the new mechanic visually every swing.
- **B.** No, keep the swing card visually unchanged; cost is a backend mechanic, not spectator content.

**Defer.** PR1 carries the cost through `CombatBeat` so the data is available; PR2 picks A or B during implementation based on visual density. File as an inline plan-time decision.

---

## File / Module Layout

**New types in `shared/src/messages.rs`:**
- `StaminaBand` enum
- `MessagePayload::StaminaBandChanged` variant

**New / modified in `game/`:**
- `game/src/tributes/combat.rs` — existing constants moved into `CombatTuning`; per-swing cost deduction; band-penalty application to rolls.
- `game/src/tributes/stamina_band.rs` (NEW) — pure `stamina_band()` function and helpers.
- `game/src/tributes/lifecycle.rs` — `restore_stamina` renamed `recover_stamina`, takes `(action, sheltered, hunger_band, thirst_band, tuning)`.
- `game/src/tributes/brains.rs` — stamina-band override block added to the override pipeline; predator-bonus scoring added to attack-action scoring.
- `game/src/tributes/mod.rs` — `Action::Attack` resolution gated on attacker stamina; cost deduction at the call site (or inside `attacks()`).
- `game/src/games.rs` — `Game.combat_tuning: CombatTuning` field added; per-phase recovery loop wired in `process_turn_phase`; `StaminaBandChanged` emission when band crosses.

**New / modified in `web/`:**
- `web/src/components/tribute_state_strip.rs` — Winded / Exhausted pip rendering.
- `web/src/components/tribute_detail.rs` — stamina bar in the bars block.
- `web/src/components/timeline/cards/stamina_card.rs` (NEW) — `StaminaCard` component.
- `web/src/components/timeline/event_card.rs` — dispatch `StaminaBandChanged` to `StaminaCard` inside the existing `MessageKind::State` arm.

---

## PR Split

Mirrors the shelter and gamemaker spec's PR1/PR2 split.

**PR1 — Backend** (~10 TDD tasks):
1. `CombatTuning` struct + `Default` impl + `Game.combat_tuning` field with `#[serde(default)]`
2. Hoist existing constants to read from `CombatTuning` (no behavior change)
3. `StaminaBand` enum in `shared/`; `stamina_band()` pure function in `game/`
4. `MessagePayload::StaminaBandChanged` variant + routing to `MessageKind::State`
5. Per-swing stamina cost deduction (attacker + target)
6. Band-derived roll penalty in attack/defense rolls
7. `recover_stamina` rename + new formula (idle / resting / sheltered, with starving/dehydrated multiplier)
8. Per-phase band-cross detection + `StaminaBandChanged` emission
9. Brain stamina override block (Winded score nudge, Exhausted SeekShelter/Rest, visible-band flee rule)
10. Brain predator scoring (Fresh attack-action bonus on visibly-tired targets); action-gate on insufficient stamina; integration test in `game/tests/stamina_combat_integration.rs`

**PR2 — Frontend** (~5 TDD tasks):
1. `MessageKind::State` already exists — no shared changes needed beyond PR1
2. Stamina bar in `tribute_detail.rs`
3. Winded / Exhausted pips in `tribute_state_strip.rs`
4. `StaminaCard` component + `event_card.rs` dispatch
5. WCAG check on new pip glyphs and bar colors; manual smoke; PR

---

## Out of Scope (filed as follow-up beads after spec lands)

1. **Property tests pinning current combat behavior baseline** — guards the `CombatTuning` defaults so future tuning passes can verify safety.
2. **Combat formula retuning pass** — re-derive `DECISIVE_WIN_MULTIPLIER`, stress contributions, and stamina constants from playtest data once stamina is shipped.
3. **Stamina-aware sponsor gifts** — energy drink consumable that restores stamina chunk; pairs with sponsorship feature.
4. **Trait-driven recovery modifiers** — Athletic trait gives +20% recovery, Frail gives -20%, etc.
5. **Hex-map fatigue glow** — visual indicator on the hex marker for Winded/Exhausted tributes; revisit during spectator-skin work.
6. **Stamina-cost rendering on swing cards** — A vs B from the open question above; PR2 picks during implementation.
7. **Announcer prompts for stamina events** — extends scope of `hangrier_games-xfi`.

---

## Open Questions (defer to implementation)

- Exact "reachable shelter" range for the Exhausted SeekShelter override — same range as the hunger/thirst SeekShelter override, or a tighter one given the actor is more fragile? Use the same range for v1; adjust if playtest shows Exhausted tributes dying mid-flight to shelter.
- Whether the "visible band" flee rule should consider Winded vs Fresh strictness (current spec: Exhausted flees from Winded *or* Fresh; Winded does not flee). Possibly add a "Winded flees from Fresh if also wounded (HP < 50%)" rule as a v1.x follow-up if Winded tributes feel suicidally aggressive.
- Whether `fresh_target_visibly_tired_bonus` should scale by *which* tired band (Winded = +3, Exhausted = +5) rather than the flat +5. Tunable via `CombatTuning` — extend the struct in PR1 if implementation finds this worth the complexity.
- `restore_stamina` rename vs. wrap: pick whichever generates less test churn at implementation time.

## Risks

- **Combat feels grindy if costs are too high** — Fresh tributes cap out at 4 swings before going Winded (100 ÷ 25). Mitigation: the values are tunable via `CombatTuning`; lean toward this in playtest.
- **Recovery feels too fast and bands rarely surface** — the +60/phase sheltered-resting rate could erase combat fatigue in two phases. Mitigation: same tuning surface; the band-changed events make this measurable in playtest.
- **Brain logic explosion** — five layers of overrides (combat, gamemaker, hunger/thirst, stamina, standard) is a lot. Mitigation: each layer is isolated and testable in `brains.rs`; the new layer follows the same pattern as the previous two and slots into a clear precedence order.
- **Save migration** — `Game.combat_tuning` is `#[serde(default)]` so existing games load with default tuning. New `stamina` field is already on `Tribute`. New `StaminaBand` enum and `StaminaBandChanged` payload are append-only to `MessagePayload` (existing match arms remain exhaustive). No migration script needed.
