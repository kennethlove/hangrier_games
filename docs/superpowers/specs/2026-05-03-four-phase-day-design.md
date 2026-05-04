# Four-Phase Day — v1 Design

**Status:** Approved (brainstorming complete, awaiting implementation plan)
**Author:** klove
**Date:** 2026-05-03
**Related:**
- Replaces the current `Phase::{Day, Night}` model in `shared::messages`
- Foundation for: phase-gated brain layers, biome×phase environmental rolls, sleep mechanic
- Touches: `shared::messages::Phase`, `game::games`, `game::events`, `game::tributes::brains`, `game::tributes::lifecycle`, every test that asserts phase sequence

## 1. Problem

The current model has two phases per game-day: `Day` and `Night`. Each runs the full brain pipeline once per living tribute, then advances the day. Two problems:

1. **Pacing is coarse.** A tribute's whole day is one decision; consequences (afflictions, alliances, ambushes) take a full game-day to manifest. The story reads as long flat blocks rather than a daily arc.
2. **Time of day doesn't *mean* anything beyond combat tone.** Dawn, dusk, midday heat, night chill — none exist in the model. The arena feels temporally flat.

Add two transitional phases (`Dawn`, `Dusk`) so each game-day arcs **Dawn → Day → Dusk → Night** with a real beat per phase. Use the new substrate to wire phase-aware mechanics: lighting affects sight, temperature triggers afflictions in the right biomes, daily routine biases brain decisions, sleep becomes a real mechanic.

## 2. The four phases

```rust
// shared/src/messages.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Dawn,
    Day,
    Dusk,
    Night,
}
```

Ordinal order matches narrative order. Phases are equal-weight: each phase runs the full brain pipeline once per living tribute. Day boundary is at end of `Night`: after `Night` resolves, `current_day += 1` and `phase = Phase::Dawn`.

The `Phase::Day` variant retains its name and serialized form (`"day"`) so existing wire data remains backward-compatible. New variants `Dawn`, `Dusk`, `Night` (already exists) round it out.

**Phase ordinals** (used for message ordering by `(day, phase_ord)`):

| Phase | Ord |
|---|---|
| `Dawn` | 0 |
| `Day` | 1 |
| `Dusk` | 2 |
| `Night` | 3 |

## 3. First day handling

Day 1 is special: tributes rise from pedestals, no preceding sleep or dawn. Day 1 starts at `Phase::Day` and runs `Day1 → Dusk1 → Night1`. Then Day 2 begins normally at `Phase::Dawn`.

`FirstDayStart` event keeps its semantic meaning ("the games begin"). The bloodbath is an event during the Day1 phase, not its own phase.

Subsequent days are uniform `Dawn → Day → Dusk → Night`.

## 4. Feast Day handling

Feast Day (currently Day 3, see `FeastDayStart`) uses the standard 4-phase structure. The Feast itself is an event during the `Day3` phase — guaranteed to fire at `Phase::Day` specifically (not Dawn, Dusk, or Night). No structural special case; just a constraint on event scheduling.

Multi-phase event windows (e.g. Feast lasting Day3 + Dusk3, or extended gamemaker storms) are filed as a v2 follow-up — see §13.

## 5. Equal-weight phases

Every phase runs the full brain pipeline (currently: `psychotic, preferred, survival, stamina, affliction, gamemaker, alliance, consumable → decide_base`; PR2 of afflictions adds the `affliction` layer). No "transition" phases with reduced layer sets in v1.

Implication: **roughly 4× the current cycle count.** Optimization is a first-class concern in this spec — see §11.

## 6. Phase-aware mechanics (v1)

Four mechanics ship in v1. Each is small in code; together they make every phase *feel* different.

### 6.1 Lighting / visibility

Phase shifts the *baseline* light level; biome and weather modulate it. Lighting affects:

- Sight range for ambush detection, target acquisition, "tributes I can see" lists for brain visibility-gating
- Whether `AreaEvent::Fog` is a meaningful roll candidate for the phase

Baseline by phase (modulated by biome, weather, and the per-area roll table — see §6.5):

| Phase | Baseline light |
|---|---|
| `Dawn` | low (transition) |
| `Day` | high |
| `Dusk` | low (transition) |
| `Night` | very low (darkness) |

Light level is an `enum LightLevel { Bright, Dim, Dark }` derived from `(phase, biome, weather)` per area each phase, not stored. Brains read it via `Tribute::light_level_in_area(&Area, phase, weather)`.

### 6.2 Temperature & weather afflictions

Each `(phase, biome, weather)` combination has a roll table for environmental afflictions. Examples:

- `Dawn` + tundra biome + clear weather → small chance of `Frozen` for unsheltered tributes
- `Day` + desert biome + clear weather → moderate chance of `Overheated` for unsheltered
- `Day` + jungle biome + storm → moderate chance of `Sick` from exposure
- `Night` + tundra biome + any weather → strong chance of `Frozen` for unsheltered
- `Night` + temperate biome + storm → small chance of `Overheated`'s opposite via cold (mild Frozen)

These hook into the existing affliction system (`Tribute::try_acquire_affliction`). The roll table is a new `game::phases::environment` module containing `pub fn roll_environmental_afflictions(phase, biome, weather, sheltered) -> Vec<AfflictionDraft>`.

**Variability principle:** not every dawn is cold, not every night is cold, not every day is hot. The roll consults biome + weather + day-number-weighting + RNG. Phase only sets the *baseline* probability; the actual outcome varies game-to-game and area-to-area.

### 6.3 Phase-gated brain bias

Phase biases brain layer weights without forcing actions. The unified pipeline already has weighted utility scoring; phase shifts the weights:

| Phase | Bias |
|---|---|
| `Dawn` | +eat/drink (morning routine), +observation refresh |
| `Day` | +act-on-goals (combat, movement, foraging) |
| `Dusk` | +seek-shelter, +alliance check-ins (returning to camp) |
| `Night` | +sleep, +ambush opportunism for hostile tributes |

Bias values are placeholder weights tuned post-observability, identical pattern to `AfflictionTuning`.

### 6.4 Sleep mechanic

Sleep is a **brain decision**, not phase-locked. A tribute can sleep at any phase.

New `Action::Sleep { duration_phases: u8 }`. Brain weighs sleep need based on:

- Hours since last sleep (tracked as `Tribute::cycles_awake: u32`)
- Current stamina band (Exhausted strongly raises sleep weight)
- Local safety (hostile tributes nearby strongly lowers sleep weight)
- Current shelter status (sheltered raises weight)
- Phase (Night raises weight, Day lowers — daylight visibility = vulnerability)

Sleep effects:

- Restores stamina at a fixed rate per phase slept
- Restores small HP if not afflicted by `Wounded`/`Infected`/`Sick`
- Sets a `sleeping: bool` flag making the tribute vulnerable to ambush (combat detection target priority +)
- Sleeping tributes skip brain pipeline (decision is "continue sleeping" until duration elapses or interrupted)

Sleep can be interrupted by ambush, area events (storm, mutts), or alliance summons. Interruption sets `cycles_awake = 0` regardless of how long they actually slept (rude awakening = no rest).

`cycles_awake` increments by 1 each phase the tribute is *not* sleeping. Threshold for "sleep need" begins around 6 phases awake (1.5 game-days); beyond 12 phases (3 days) the weight dominates and the tribute will sleep almost regardless of safety.

### 6.5 Per-area environmental roll

§6.1 and §6.2 share an underlying mechanism: each phase, every area rolls for environmental conditions. Output:

```rust
pub struct AreaPhaseConditions {
    pub light: LightLevel,
    pub weather: Weather,                 // existing or new enum
    pub afflictions_inflicted: Vec<(TributeId, AfflictionDraft)>,
}
```

Stored on the area for the duration of the phase, cleared at phase transition. Tributes read it during their pipeline pass.

A new `Weather` enum may be needed if one doesn't exist; verify before introducing — there's existing code for `AreaEvent::Wildfire`/`Blizzard`/`Heatwave`/`Sandstorm`/`Drought` which is event-based, not state-based. Weather as continuous state (not just events) is a separate question — design decision deferred to PR1 implementation: either reuse an existing weather field or introduce `enum Weather { Clear, Cloudy, Rain, Storm, Snow }` and roll it per-phase per-area.

## 7. Cycle pipeline

Per phase, in order:

1. **Phase-start global events** — emit `PhaseStarted { day, phase }` message; gamemaker considers events
2. **Per-area environmental roll** — populate `AreaPhaseConditions` for each area (light, weather, environmental afflictions)
3. **Per-tribute pipeline pass** — for each living, non-sleeping tribute (in deterministic order), run the full brain pipeline with the current `phase` available in `CycleContext`
4. **Sleeping-tribute updates** — for sleeping tributes, decrement `Action::Sleep.duration_phases`; restore stamina/HP; check interruption conditions
5. **Per-tribute affliction tick** — apply environmental affliction drafts from step 2; apply per-phase tribute state changes (cycles_awake increment, etc.)
6. **Phase-end events** — emit `PhaseEnded { day, phase }`; advance phase or day boundary

Phase advancement: `Phase::Dawn → Day → Dusk → Night → Dawn (next day)`.

## 8. Messages

```rust
// shared/src/messages.rs
MessagePayload::PhaseStarted { day: u32, phase: Phase, weather_summary: Option<String> }
MessagePayload::PhaseEnded { day: u32, phase: Phase }
```

Existing `GameDayStart`, `GameDayEnd`, `GameNightStart`, `GameNightEnd` events are subsumed by `PhaseStarted`/`PhaseEnded`. Migration: deprecate the four old variants in this PR, retain backward-compat translation in the timeline UI for one cycle, then remove in a follow-up. (Note: the existing variants are in `game::events::GameEvent`, which is itself slated for collapse onto `MessagePayload` per `hangrier_games-b67j`. This spec accelerates that collapse for the four phase events.)

`Action::Sleep { duration_phases: u8 }` adds a `MessagePayload::TributeSlept { tribute, phase, restored_stamina, restored_hp }` emitted at sleep onset; `MessagePayload::TributeWoke { tribute, phase, reason: WakeReason }` at sleep end.

```rust
pub enum WakeReason {
    Rested,                                 // duration elapsed
    Interrupted { event: InterruptionKind } // ambush, area event, alliance summons
}

pub enum InterruptionKind {
    Ambush { attacker: TributeRef },
    AreaEvent(AreaEventKind),
    AllianceSummons { ally: TributeRef },
}
```

## 9. Storage

New fields on `Tribute`:

```rust
pub cycles_awake: u32,             // phases since last full sleep
pub sleeping: bool,                // currently sleeping (affects ambush vulnerability)
pub sleep_remaining: u8,           // phases left in current Action::Sleep
```

All default to `cycles_awake: 0, sleeping: false, sleep_remaining: 0` for backward compatibility (`#[serde(default)]`).

New field on `Game` (or the existing game state struct):

```rust
pub current_phase: Phase,           // already exists, but variant set expands
```

No structural schema change for `current_phase` — just the enum's variant set grows. SurrealDB stores it as a string already (`"day"` / `"night"`); new strings (`"dawn"`, `"dusk"`) just appear after migration.

New optional per-area state for the duration of a phase:

```rust
pub struct AreaPhaseConditions { ... }   // see §6.5
```

Stored in-memory on the area, NOT persisted. Cleared each phase transition.

## 10. Migration

Single PR, ordered:

1. Extend `shared::messages::Phase` with `Dawn` and `Dusk` variants. `Day` and `Night` retain their serialized forms; new variants serialize as `"dawn"`/`"dusk"`. Update `FromStr` and `Display` impls.
2. Update `Game::current_phase` initialization to `Phase::Day` for new games (Day 1 starts at Day per §3). Existing in-flight games: their `current_phase` is `"day"` or `"night"` already — both deserialize cleanly. They continue from wherever they left off; the new phases will be reached on subsequent transitions.
3. Update phase-advancement logic (`game::games`, around lines 449/451/496/498/741/743) to walk the 4-phase cycle: `Phase::Dawn → Day → Dusk → Night → (day++) → Dawn`. With first-day exception: if `current_day == 1` and `current_phase == Day`, the previous phase was conceptually the start; after `Night`, advance to `Phase::Dawn` of day 2.
4. Add `MessagePayload::PhaseStarted`/`PhaseEnded` and emit them at phase boundaries. Replace `GameDayStart`/`GameDayEnd`/`GameNightStart`/`GameNightEnd` producers with the new payloads. Update `kind()`/`involves()` exhaustive matches in `messages.rs`.
5. Add `Tribute` fields (`cycles_awake`, `sleeping`, `sleep_remaining`) with serde defaults.
6. Add `Action::Sleep` variant; brain layer scoring for sleep need.
7. Add `game::phases::environment::roll_environmental_afflictions` with placeholder per-(phase, biome, weather) tables.
8. Add `Weather` enum (or reuse if exists); roll per-area per-phase.
9. Add phase-bias to existing brain layers.
10. Update timeline UI to render 4 phases per day.

Steps 1-4 are the substrate. Steps 5-9 are mechanics. Step 10 is frontend. Plan can split into three PRs: (1-4 substrate), (5-9 mechanics), (10 frontend) — confirm scope at writing-plans time.

## 11. Optimization

4× cycle count makes performance a first-class concern. Optimizations to land alongside or shortly after the substrate:

1. **Skip dead tributes early** — `RecentlyDead` and `Dead` tributes don't enter the per-tribute pipeline at all. Existing code may already do this; verify and tighten.
2. **Skip sleeping tributes from full pipeline** — sleeping tributes get a minimal "still sleeping?" check, not the full layer chain (per §7 step 4).
3. **Phase-gate brain layers** — layers that have zero effect at certain phases can early-return. Examples: alliance-formation layer at `Dusk` only; ambush-opportunism layer at `Dusk`/`Night` only. Each layer declares its active phases via `fn active_phases(&self) -> &[Phase]`.
4. **Per-area condition caching** — `AreaPhaseConditions` is computed once per phase per area, read N times by tributes in that area (not recomputed per tribute).
5. **Message coalescing** — repeated `MovedTo(same_area)` events within a phase coalesce into a single emission. Snapshot-band emissions (already done for stamina/hunger) extend to other state where it helps.
6. **Defer message emission** — collect per-phase messages in a buffer, emit at phase boundary in batches rather than streaming each one.

These are filed as separate beads off this spec's epic — substrate lands first, then optimizations are measured and applied. Don't preemptively optimize beyond §11.1 and §11.2 (those are correctness, not performance).

## 12. Testing strategy

Unit tests:
- `Phase::next()` cycles correctly: Dawn → Day → Dusk → Night → Dawn
- `Phase` ord values match §2 table
- `FromStr`/`Display` round-trip for all four variants
- `roll_environmental_afflictions` deterministic given seeded RNG
- Sleep brain-weight scoring: exhausted tributes pick sleep, fresh ones don't; nearby hostile tributes suppress sleep
- `cycles_awake` increments correctly across phases; resets on full sleep; resets on interruption

Integration tests:
- Game loop runs 3 game-days, asserts phase sequence: `Day1 → Dusk1 → Night1 → Dawn2 → Day2 → Dusk2 → Night2 → Dawn3 → Day3 → Dusk3 → Night3`
- Tribute fatigue: tribute with no sleep over many phases eventually picks `Action::Sleep` despite phase bias
- Ambush vulnerability: sleeping tribute is preferred ambush target
- Environmental affliction roll: 100-game seeded run produces variety (not all dawns foggy, not all nights cold)
- Backward compatibility: a game persisted with `current_phase = "day"` or `"night"` resumes correctly under the new model

Snapshot tests (insta):
- Phase-message stream for a 3-day seeded game
- Tribute serialization with new fields populated
- `AreaPhaseConditions` per phase across a 4-phase day

## 13. Out of scope (filed as v2 follow-ups)

- **Multi-phase event windows** — events that span multiple phases (Feast across Day+Dusk; gamemaker storms; mutt swarms with persistence). Q6 D from brainstorming.
- **Ambush window mechanic** — Ambush attempts only viable at Dusk/Night. Trivial to add once substrate exists.
- **Phase-gated environmental events** — `AreaEvent` rolls weighted by phase (storms more likely at Dusk, fires at Day). Different from §6.2; this is for the existing `AreaEvent` system.
- **Sponsor decision windows** — sponsors decide gifts at end-of-Day phase only. Gives sponsors a clear rhythm.
- **Per-phase social events** — alliance proposals only at Dawn/Night, etc.
- **Per-phase tribute energy budget** — decisions cost "action energy" that resets at sleep.
- **Variable phase count per day** — Feast Day or gamemaker-event days get 5-6 phases, normal days 4. Q2 D from brainstorming.
- **Phase-of-day emoji/icons in timeline** — minor UI polish.

## 14. Spec self-review

- **Placeholders:** brain bias weights (§6.3), sleep restore rates (§6.4), environmental roll tables (§6.2/§6.5), `cycles_awake` thresholds (§6.4) are explicit defaults to be tuned post-observability. None are "TBD" gaps; all are concrete starting values.
- **Internal consistency:** Phase ordering (§2) matches first-day handling (§3) and the migration step 3 logic. Sleep being decision-driven (§6.4) is consistent across §7, §8, §9, §11. Equal-weight phases (§5) is consistent with the cycle pipeline (§7) running the full brain pipeline per non-sleeping tribute per phase.
- **Scope check:** Substrate (§§2-4, §10 steps 1-4) + mechanics (§§6-9, §10 steps 5-9) + frontend (§10 step 10) is borderline-large for one plan; the writing-plans step should split into 2-3 PRs.
- **Ambiguity:** The `Weather` enum question (§6.5) is explicitly flagged as a PR1 implementation decision rather than left unresolved. The rest commits to shape, with numbers tunable.
