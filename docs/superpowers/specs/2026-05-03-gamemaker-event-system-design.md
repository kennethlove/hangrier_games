# Gamemaker Event System — Design

**Status:** Draft
**Date:** 2026-05-03
**Crate(s) primarily affected:** `game/`, `shared/`, `web/`
**Related specs:** `2026-05-02-weather-system-design.md` (carved this out), `2026-04-17-event-severity-integration.md`, `2026-04-26-game-event-enum.md`, `2026-05-02-tribute-emotions-design.md`, `2026-05-03-shelter-hunger-thirst-design.md`
**Related issue:** `hangrier_games-5q9`

## Goals

Add a Capitol-intervention layer to the simulation that:

1. **Imposes authored drama on the arena** — the gamemaker decides *when* and *where* something happens, separate from natural causation (weather, hazards, combat).
2. **Bypasses natural-cover rules at will** — gamemaker interventions are explicitly *force majeure*; shelter and hide get partial respect at best, never full immunity. They're the answer to "the tribute did everything right, why did they still die?"
3. **Models the gamemaker as an actor with internal state** — RimWorld-storyteller style: gauges drive decisions, profiles tune personality. Single source of truth for "what does the Capitol want right now?"
4. **Slots cleanly under the timeline + announcer** — every intervention emits typed `MessagePayload` variants the existing UI and announcer LLM can consume.

Non-goals (deferred — see "Out of Scope" section):

- Multiple storyteller profiles in v1 (one default ships).
- Tribute-targeted interventions ("Capitol decides X dies").
- Human-controlled gamemaker mode.
- Scripted day-N set-pieces (cornucopia rerun, etc.) — generalized via `ConvergencePoint::Lure` instead.
- Bespoke timeline card layouts per intervention (generic rendering in v1).
- Capitol Feed panel exposing gauges to spectators.
- Sponsor-pressure feedback into gauges.

## Architectural Position

Per the weather spec's three-layer split:

| Layer | What it is | Lifecycle | This spec? |
|---|---|---|---|
| **Weather** | Persistent atmospheric state per area | Evolves slowly per-day-phase | No (weather spec) |
| **NaturalHazard** | Acute event caused by terrain + weather | One-shot, probabilistic | No (weather spec) |
| **GamemakerEvent** | Imposed Capitol intervention | One-shot or persistent active-effect | **Yes** |

The three layers are orthogonal; gamemaker decisions never *cause* weather transitions or natural hazards (with one explicit exception: `Fireball` may bias the next phase's wildfire roll in its target area — see Q7 / "Interactions" below).

### Coexistence with current `AreaEvent` (independent migration)

Current code reality:
- `AreaEvent` enum (10 variants in `game/src/areas/events.rs`) is implemented and live, mixed-purpose.
- `Weather` enum doesn't exist (shelter PR1 wedges a stub: `Clear, HeavyRain, Heatwave, Blizzard`).
- `NaturalHazard` enum doesn't exist.
- `Gamemaker*` doesn't exist.

This spec ships **independently** of the weather/hazard refactor. Gamemaker code lives at `game/src/gamemaker/`, defines its own enums, and does **not** carve `AreaEvent`. When the weather spec lands later it carves `AreaEvent` → `NaturalHazard`; this gamemaker work is unaffected.

**Weather stub coordination:** if shelter PR1 has shipped, gamemaker re-uses its `Weather` enum (single source of truth). If shelter hasn't shipped, gamemaker introduces the same minimal stub (`Weather::{Clear, HeavyRain, Heatwave, Blizzard}` plus `current_weather() -> Weather` returning `Clear`). Whichever ships first owns the wedge; the other re-imports.

## The Gamemaker Actor

A new `Gamemaker` struct on `Game` carries all gamemaker state:

```rust
pub struct Gamemaker {
    pub profile: GamemakerProfile,
    pub gauges: Gauges,
    pub recent_interventions: VecDeque<RecentIntervention>,  // capped at 6 entries
    pub interventions_today: u8,                              // resets each day
    pub active_effects: Vec<ActiveIntervention>,              // ongoing things on the map
}

pub struct Game {
    // ... existing fields
    pub gamemaker: Gamemaker,
}
```

Cost: ~50 bytes plus active_effects vec. Persists via existing serialization.

### Gauges

Six `u8`s, each 0–100, recomputed at each phase boundary:

| Gauge | Rises when | Decays when | Drives |
|---|---|---|---|
| `drama_pressure` | Quiet phases (no kills, no hazards, no major moves) | Combat / hazards / deaths happen, intervention fires | Whether to intervene at all |
| `audience_attention` | Spectacle moments, named-tribute deaths, alliance breaks | Slowly each phase regardless | Magnitude — low attention biases toward flashier picks |
| `bloodthirst` | Phases without a tribute death | Each tribute death (proportional to "fame") | Bias toward lethal vs. nuisance interventions |
| `chaos` | Predictable patterns (same tribute winning, sectors quiet) | Already-chaotic states (active combat, multiple hazards live) | Bias toward disruptive picks |
| `patience` | Each phase since last intervention (cooldown timer) | Fires intervention → resets to 0 | Hard gate — no intervention until `patience >= profile.patience_threshold` |
| `body_count_debt` | Daily expected_deaths > actual_deaths (clamped 0..=100) | Deaths happen | Late-game escalation; low priority early |

```rust
pub struct Gauges {
    pub drama_pressure: u8,
    pub audience_attention: u8,
    pub bloodthirst: u8,
    pub chaos: u8,
    pub patience: u8,
    pub body_count_debt: u8,
}

impl Gauges {
    pub const STARTING: Self = Self {
        drama_pressure: 0,
        audience_attention: 80,
        bloodthirst: 20,
        chaos: 10,
        patience: 0,
        body_count_debt: 0,
    };
}
```

### Profile (storyteller personality)

```rust
pub struct GamemakerProfile {
    pub name: &'static str,
    pub pressure_decay_rate: u8,        // pressure rises this much per quiet phase
    pub attention_decay_rate: u8,       // attention drains this much per phase
    pub bloodthirst_weight: u8,         // multiplier into variant scoring (100 = baseline)
    pub chaos_weight: u8,               // multiplier into variant scoring
    pub patience_threshold: u8,         // gauge minimum to be eligible to intervene
    pub max_per_day: u8,                // hard cap on interventions per game-day
    pub max_concurrent_events: u8,      // cap on simultaneously-active effects
    pub late_game_multiplier: f32,      // pressure scaling once ≤ 8 tributes alive
    pub recent_penalty: u32,            // each occurrence in window subtracts this from variant score
}
```

V1 ships **one profile**, `Cassandra` (rising tension, paced escalation):

```rust
pub const CASSANDRA: GamemakerProfile = GamemakerProfile {
    name: "Cassandra",
    pressure_decay_rate: 8,
    attention_decay_rate: 3,
    bloodthirst_weight: 100,
    chaos_weight: 100,
    patience_threshold: 30,
    max_per_day: 2,
    max_concurrent_events: 2,
    late_game_multiplier: 1.5,
    recent_penalty: 25,
};
```

Numbers are starting points; tune during implementation playtest.

Additional profiles (Phoebe, Randy, custom) are filed as a follow-up bead.

### Trigger thresholds

Gauges drive a `should_intervene(profile, gauges, game_state) -> bool` decision. After patience gates pass, fire if **any** of:

- `drama_pressure >= 60`
- `bloodthirst >= 70`
- `chaos >= 70`
- `body_count_debt >= 10`

If `interventions_today >= profile.max_per_day` → no-op regardless of gauges. If `active_effects.len() >= profile.max_concurrent_events` → no-op (let the field clear first).

## The Catalog (6 variants)

```rust
pub enum InterventionKind {
    Fireball,
    MuttPack,
    ForceFieldShift,
    AreaClosure,
    ConvergencePoint,
    WeatherOverride,
}
```

Grouped by flavor:

### Lethal

**`Fireball { area, severity }`**
Area-targeted instant damage. Tributes in the area roll a survival check (`d20 + dodge_mod` vs. severity DC). Bypasses shelter completely. Bypasses hide completely. May bias next phase's wildfire roll in target area (see Interactions).

**`MuttPack { area, kind: Animal, members: u8, hp: u32, decay_phases: u8 }`**
Spawns a `MuttSwarm` — single aggregate entity with HP pool + member count, **not** individual NPCs. Reuses existing `threats::animals::Animal` enum for the kind (Wolf, Bear, Cougar, Hyena, etc.). Pursues nearest tribute; combat reduces swarm HP; killing each chunk reduces `members`. Sheltered tributes get **+5 evade roll**, hidden tributes get **+2 evade roll** (additive, max +7 if both apply). Despawns conditionally — at next morning OR after N consecutive phases with no tributes in adjacent areas.

### Disruptive

**`ForceFieldShift { close: Vec<Area>, open: Vec<Area>, warning_phases: u8 }`**
Closes some sectors, opens others. Tributes inside closing sectors get a one-phase warning (announced via `ForceFieldShifted` event with `warning_phases`); after warning expires, attempting to remain takes damage equivalent to `AreaClosure` entry damage. Topology-only intervention; no direct damage on resolution.

**`AreaClosure { area, duration_phases: u8 }`**
Temporary lockdown. Entering or remaining in the area takes damage per phase. Pressures tributes toward each other. No bypass via shelter/hide (the seal damage is "Capitol force field"; sheltering inside doesn't help).

### Convergence

**`ConvergencePoint { area, lure: Lure, duration_phases: u8, payload: Vec<Item> }`**
Drops a marked area on the map. Brain logic gets a "go to convergence point" pull — adds to `choose_destination` scoring proportional to lure strength. Items in payload are claimable by whoever reaches the area before duration expires. Generic — feast is one lure flavor, future cousins (water cache, sponsor airdrop carnival, capitol summons) plug in as additional `Lure` variants without new top-level intervention.

```rust
pub enum Lure {
    Feast,
    // Future: WaterCache, AirdropCluster, CapitolSummons, ...
}
```

### Atmospheric

**`WeatherOverride { area, weather: Weather, duration_phases: u8 }`**
Forces a weather state in the target area regardless of natural transition. Replaces (does not stack with) current weather for the duration; reverts to natural transition rolls afterward. Cheapest intervention — the gamemaker's "I need to do *something* but the field's not ready for spectacle" valve.

## Decision Flow (per phase)

At each phase boundary in `process_turn_phase`:

1. **Tick gauges** — apply per-phase rises and decays (see "Gauge tick rules" below). Apply `late_game_multiplier` to `drama_pressure` rise if `alive_tributes <= 8`.
2. **Tick active_effects** — decrement remaining phases on `MuttSwarm`/`AreaClosure`/`ConvergencePoint`; emit despawn/expire/unseal events; remove expired entries.
3. **Resolve pending damage from active_effects** — `AreaClosure` entry damage to anyone inside, `MuttSwarm` attacks against tributes in pack's area, etc. Emits `AreaSealEntryDamage`, `MuttSwarmAttack` events.
4. **Eligibility gate** — `should_intervene(profile, gauges, game_state)`:
   - If `interventions_today >= profile.max_per_day` → skip
   - If `active_effects.len() >= profile.max_concurrent_events` → skip
   - If `patience < profile.patience_threshold` → skip
   - If no trigger threshold met → skip
5. **Pick variant** — for each `InterventionKind`, compute `score(profile, gauges, game_state)`. Subtract `profile.recent_penalty * occurrences_in_recent_interventions` per variant. Highest score wins; ties → rng.
6. **Pick target** — call selected variant's `target_pref(game_state) -> Option<TargetSpec>`. If `None`, drop this variant from the candidate set and re-pick (loop max 3 attempts; if all fail, no intervention this phase).
7. **Resolve immediately or register active effect:**
   - `Fireball` / `ForceFieldShift` / `WeatherOverride` → resolve now, emit events, no `active_effects` entry.
   - `MuttPack` / `AreaClosure` / `ConvergencePoint` → push `ActiveIntervention` entry, emit announcement event, resolution happens in step 3 of subsequent phases.
8. **Update bookkeeping** — push to `recent_interventions` (cap at 6), increment `interventions_today`, reset `patience` to 0, decay gauges per intervention rules.
9. **Day rollover** — at the day-night-day transition, reset `interventions_today` to 0.

### Gauge tick rules (per phase)

Per-phase background tick (before any intervention):

```text
drama_pressure   += profile.pressure_decay_rate (clamped 0..=100)
                    × (alive_tributes <= 8 ? profile.late_game_multiplier : 1.0)
audience_attention -= profile.attention_decay_rate (clamped 0..=100)
patience         += 1 (clamped 0..=255)
```

Per-event reactions (subtract these *after* each event):

| Trigger | drama_pressure | audience_attention | bloodthirst | chaos | body_count_debt |
|---|---|---|---|---|---|
| Tribute killed (any cause) | −15 | +12 | −20 | −5 | −10 |
| Hazard fires (no kills) | −8 | +5 | −2 | −10 | 0 |
| Hazard fires (with kill) | −15 | +12 | −20 | −15 | −10 |
| Combat round (no kill) | −3 | +2 | +2 | −2 | 0 |
| Day boundary, expected_deaths_today > actual | 0 | 0 | +10 | 0 | +(expected − actual) × 5 |
| Day boundary, no deaths all day | +20 | 0 | +25 | 0 | +5 |

Per-intervention reactions (subtract *after* this gamemaker's own intervention):

| Intervention | drama_pressure | audience_attention | bloodthirst | chaos | patience |
|---|---|---|---|---|---|
| Fireball / MuttPack with kills | −30 | +20 | −30 | −10 | reset to 0 |
| Fireball / MuttPack with no kills | −15 | +5 | −10 | −5 | reset to 0 |
| ForceFieldShift / AreaClosure | −10 | +5 | 0 | −20 | reset to 0 |
| ConvergencePoint announce | −5 | +3 | 0 | 0 | reset to 0 |
| ConvergencePoint resolve (combat fires there) | use combat reactions | — | — | — | — |
| WeatherOverride | −5 | 0 | 0 | −5 | reset to 0 |

Numbers are starting points; tune during implementation.

## Variant Scoring & Targeting (sketch)

Each variant exposes two pure functions:

```rust
trait InterventionLogic {
    fn score(&self, profile: &GamemakerProfile, gauges: &Gauges, state: &GameState) -> u32;
    fn target_pref(&self, state: &GameState) -> Option<TargetSpec>;
}
```

Sketches (final tuning during implementation):

- **Fireball**: `score = bloodthirst·2 + drama_pressure − patience_penalty`. Target = area with highest `tribute_count + alliance_co_location_bonus`.
- **MuttPack**: `score = bloodthirst + body_count_debt·2 + chaos`. Target = area with `tribute_count > 0` and at least one adjacent occupied area.
- **ForceFieldShift**: `score = chaos·3 + drama_pressure − (alive_tributes_below_threshold ? 50 : 0)`. Target = pick 1–3 low-tribute areas to close, opening previously-closed adjacent areas; refuses if doing so would leave < 3 areas open.
- **AreaClosure**: `score = chaos·2 + drama_pressure`. Target = empty-or-low area positioned between two clusters (forces movement around it).
- **ConvergencePoint**: `score = drama_pressure·2 + (audience_attention < 40 ? 30 : 0) + (alive_tributes >= 6 ? 15 : 0)`. Target = central area (high adjacency count), low current tribute count.
- **WeatherOverride**: `score = drama_pressure + chaos − bloodthirst`. Target = any area; slight preference for area with tributes present.

Each `score` then has `profile.recent_penalty * occurrences_in_recent_interventions` subtracted before comparison.

`TargetSpec` is variant-specific:
```rust
pub enum TargetSpec {
    SingleArea(Area),
    AreaSet { close: Vec<Area>, open: Vec<Area> },  // ForceFieldShift
}
```

## Active Effects

```rust
pub enum ActiveIntervention {
    MuttSwarm {
        area: Area,
        kind: Animal,
        members: u8,
        hp: u32,
        max_hp_per_member: u32,
        phases_since_combat: u8,
        despawn_at_morning: bool,
    },
    AreaClosure {
        area: Area,
        expires_at_phase: u32,
        damage_per_phase: u32,
    },
    ConvergencePoint {
        area: Area,
        lure: Lure,
        expires_at_phase: u32,
        payload: Vec<Item>,
    },
}
```

Variants not listed (`Fireball`, `ForceFieldShift`, `WeatherOverride`) resolve immediately and don't appear in `active_effects`. (`WeatherOverride`'s persistence lives in `AreaWeather` once the weather spec lands; for the v1 wedge, it's a single phase override applied at resolution time.)

## Interactions With Other Systems

| Intervention | Respects shelter? | Respects hide? | Respects weather modifiers? | Triggers natural hazard? |
|---|---|---|---|---|
| Fireball | **No** (bypass) | No | No | Yes — sets +20% wildfire roll bonus in target area for the next phase |
| MuttPack | Partial — sheltered tributes get +5 evade roll; not immune | Partial — hidden tributes get +2 evade roll (stacks with shelter, max +7) | No | No |
| ForceFieldShift | N/A (topology) | N/A | No | No |
| AreaClosure | N/A (entry damage only) | N/A | No | No |
| ConvergencePoint (Feast) | N/A (lure) | N/A | N/A | No |
| WeatherOverride | N/A — *is* weather | N/A | Replaces current weather for area | Per weather spec — yes |

**Brain integration:**

Each tribute's `Brain::act` gets one new override pass before existing logic:

1. **Convergence pull (only when not in combat / not starving / not dehydrated):**
   add `convergence_pull(area)` term to `choose_destination` scoring for each active `ConvergencePoint`.
2. **Mutt avoidance (always when not in combat):**
   any active `MuttSwarm` in the tribute's current area triggers `Travel(adjacent_area)` if any adjacent area lacks one. Higher priority than Forage/SeekShelter.
3. **Sealed-area avoidance:**
   active `AreaClosure` areas filtered out of `choose_destination` candidate set entirely.

Combat preempts all of the above (existing rule).

## Events / Messages

11 new `MessagePayload` variants in `shared/src/messages.rs`. `MuttSwarmDespawned` covers all swarm endings (members exhausted, morning rollover, no nearby targets) via `DespawnReason`; `ConvergencePointActive` is omitted since active state is queryable from `gamemaker.active_effects`.

```rust
pub enum DespawnReason {
    Morning,
    NoTargetsNearby,
    NoMembersLeft,
}

// Lethal
MessagePayload::FireballStrike {
    area: AreaRef,
    severity: EventSeverity,
    victims: Vec<TributeRef>,
    survivors: Vec<TributeRef>,
}
MessagePayload::MuttSwarmSpawned {
    area: AreaRef,
    kind: Animal,
    members: u8,
}
MessagePayload::MuttSwarmAttack {
    area: AreaRef,
    kind: Animal,
    victim: TributeRef,
    damage: u32,
    killed: bool,
}
MessagePayload::MuttSwarmDespawned {
    area: AreaRef,
    kind: Animal,
    reason: DespawnReason,
}

// Disruptive
MessagePayload::ForceFieldShifted {
    closed: Vec<AreaRef>,
    opened: Vec<AreaRef>,
    warning_phases: u8,
}
MessagePayload::AreaSealed {
    area: AreaRef,
    expires_at_phase: u32,
}
MessagePayload::AreaUnsealed {
    area: AreaRef,
}
MessagePayload::AreaSealEntryDamage {
    area: AreaRef,
    tribute: TributeRef,
    damage: u32,
}

// Convergence
MessagePayload::ConvergencePointAnnounced {
    area: AreaRef,
    lure: Lure,
    starts_at_phase: u32,
}
MessagePayload::ConvergencePointExpired {
    area: AreaRef,
    lure: Lure,
    claimed_by: Vec<TributeRef>,
}

// Atmospheric
MessagePayload::WeatherOverridden {
    area: AreaRef,
    weather: Weather,
    duration_phases: u8,
}
```

Final v1 emission set (11):

1. `FireballStrike`
2. `MuttSwarmSpawned`
3. `MuttSwarmAttack`
4. `MuttSwarmDespawned`
5. `ForceFieldShifted`
6. `AreaSealed`
7. `AreaUnsealed`
8. `AreaSealEntryDamage`
9. `ConvergencePointAnnounced`
10. `ConvergencePointExpired`
11. `WeatherOverridden`

**Death cause constants** (added to existing `MessagePayload::TributeKilled.cause: String`):

```rust
pub const CAUSE_FIREBALL: &str = "fireball";
pub const CAUSE_MUTT_PACK: &str = "mutt_pack";
pub const CAUSE_AREA_SEAL: &str = "area_seal";
```

Mutt-killed tributes emit `TributeKilled { cause: CAUSE_MUTT_PACK, ... }` — the specific animal kind is recoverable from the preceding `MuttSwarmAttack` event with the same victim.

## Frontend (PR2 scope)

**Hex map** (`web/src/components/map.rs`) gains three new marker types:

| Marker | Visual | Source data |
|---|---|---|
| Mutt swarm pin | Red claw glyph + member count badge | `gamemaker.active_effects` filtered to `MuttSwarm` |
| Sealed-area shading | Red translucent overlay on hex tile | `gamemaker.active_effects` filtered to `AreaClosure` |
| Convergence point pin | Gold star + lure-specific micro-icon | `gamemaker.active_effects` filtered to `ConvergencePoint` |

**Timeline cards:** generic rendering for all 11 new `MessagePayload` variants. Pattern matches the existing fallback used by `xamw` (typed AreaEvent fallback). Each variant gets:
- A category icon (lethal = flame, disruptive = forcefield-shimmer, convergence = star, atmospheric = cloud)
- Stringified payload fields
- The standard timeline-card chrome (timestamp, source, severity badge if applicable)

Bespoke per-variant card layouts are filed as a follow-up bead.

**No** Capitol Feed panel — gauges are not exposed to spectators in v1.

## File / Module Layout

New module: `game/src/gamemaker/`

```
game/src/gamemaker/
  mod.rs               // pub use; Gamemaker struct
  gauges.rs            // Gauges struct + tick + reaction tables
  profile.rs           // GamemakerProfile struct + CASSANDRA const
  interventions/
    mod.rs             // InterventionKind + InterventionLogic trait
    fireball.rs
    mutt_pack.rs
    force_field.rs
    area_closure.rs
    convergence.rs
    weather_override.rs
  active.rs            // ActiveIntervention enum + per-phase tick/resolve
  decision.rs          // should_intervene + variant selection + targeting fallback loop
  weather_stub.rs      // (only if shelter PR1's Weather isn't merged yet)
```

`shared/src/messages.rs` gains the 11 new `MessagePayload` variants + `DespawnReason` enum + `Lure` enum + 3 cause constants.

`game/src/games.rs` `process_turn_phase` integrates the per-phase decision flow described above.

## PR Split

Mirrors the shelter spec's PR1/PR2 split:

**PR1 — Backend** (~15 TDD tasks, ~equivalent to shelter PR1 footprint):
- `Gauges`, `GamemakerProfile`, `Gamemaker` structs
- `Cassandra` profile constant
- 6 intervention variants with `score()` + `target_pref()` + resolution logic
- `ActiveIntervention` lifecycle (tick, resolve, despawn)
- 11 new `MessagePayload` variants + `DespawnReason` + `Lure` + cause constants
- `should_intervene` + variant selection + targeting fallback loop
- Brain integration (convergence pull, mutt avoidance, sealed-area avoidance)
- `process_turn_phase` integration
- Per-event gauge reaction wiring
- Day-rollover bookkeeping (`interventions_today` reset)
- Weather stub coordination (or import from shelter PR1)
- All unit tests + integration tests

**PR2 — Frontend** (~7 TDD tasks):
- Mutt swarm pin marker on hex map
- Sealed-area overlay on hex map
- Convergence point pin on hex map
- Generic timeline card for each of 11 new payloads
- WCAG check on new map overlay colors
- Visual integration tests
- Self-review

## Out of Scope (filed as follow-up beads after spec lands)

1. **Multiple storyteller profiles** (Phoebe, Randy, others) — single Cassandra in v1
2. **Gamemaker selection at game creation** — UI affordance for choosing storyteller
3. **Capitol Feed panel** — gauge visibility for spectators (toggleable dev view)
4. **Bespoke timeline cards** for gamemaker variants (per-payload card layouts)
5. **Tribute-targeted interventions** ("Capitol decides X dies tonight")
6. **Human-controlled gamemaker mode** (spectator-as-director)
7. **Scripted set-pieces** (fixed-day-N events) — generalized via `Lure` instead, but no scripted-day machinery
8. **Sponsor-pressure feedback into gauges** (sponsor activity affecting `audience_attention`)
9. **Announcer prompt updates for gamemaker events** — extends scope of `hangrier_games-xfi`
10. **Fireball→wildfire chain** — flagged as droppable during PR1 implementation if it complicates testing
11. **Per-game custom profile tuning** (knobs in game-creation UI)
12. **Replay/spectator-mode integration** — existing replay system (`hangrier_games-5wt`) wraps gamemaker state regardless

## Open Questions (defer to implementation)

- Exact damage formulas for `Fireball.severity` per `EventSeverity` band (use existing `survival_check` infrastructure as a model).
- Mutt swarm `members → hp` mapping — does each member's death proportionally reduce HP, or is HP a single pool drained until threshold per-member?
- Recent-interventions window size — spec says 6, but window in *phases* vs. *count* may matter once tuned.
- Whether `WeatherOverride` should require shelter PR1's `Weather` enum to be merged first, or always carry its own stub. (Cleanest is "whichever spec lands first owns the Weather wedge.")

## Risks

- **Cadence feels off in playtest** — the 6-gauge model has many knobs; tuning is non-trivial. Mitigation: ship Cassandra with conservative numbers, expect a tuning pass.
- **Active effect persistence across game-state mutations** — mutt swarms surviving SurrealDB round-trips need careful serialization. Mitigation: standard `serde` derives; covered by integration tests.
- **Brain override interaction with hunger/thirst overrides** — gamemaker overrides land at the *front* of the brain decision pipeline; combat preempts both. Hunger/thirst overrides (per shelter spec) sit between gamemaker overrides and the standard brain logic. Order documented above.
- **Independent migration with weather pending** — same wedge approach as shelter PR1 has been validated; low risk.

## Acceptance Criteria

- [ ] All 6 intervention variants implementable and emit correct events
- [ ] Cassandra profile produces dramatically-paced interventions (≤ 2/day, patience-gated, recent-penalty-resistant)
- [ ] Mutt swarm aggregate combat resolves correctly against tributes (hidden +2, sheltered +5, both stack max +7)
- [ ] Sealed areas damage tributes inside per phase; brain avoids them
- [ ] Convergence points pull tribute brain `choose_destination`; expire correctly with item claim list
- [ ] Weather override replaces area weather for declared duration; reverts to natural rolls afterward
- [ ] All 11 new `MessagePayload` variants serialize/deserialize round-trip
- [ ] Gamemaker state persists across SurrealDB game save/load
- [ ] Frontend renders 3 hex-map marker types correctly on integration test
- [ ] Generic timeline cards render all 11 new payloads without crashing
