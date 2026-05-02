# Weather System — Design

**Status:** Draft
**Date:** 2026-05-02
**Crate(s) primarily affected:** `game/`, `shared/`
**Related specs:** `2026-04-17-terrain-biome-system-design.md`, `2026-04-17-event-severity-integration.md`, `2026-04-26-game-event-enum.md`, `2026-05-02-tribute-emotions-design.md`
**Related future spec:** Gamemaker Event System (to be filed separately)

## Goals

Add a persistent atmospheric layer to each area that:

1. **Influences combat and movement** through derived per-state modifiers (primary mechanical role).
2. **Drives natural hazards** (formerly `AreaEvent`) by combining with terrain so disasters feel earned, not arbitrary.
3. **Provides ambient narrative texture** the announcer can lean on ("a storm rolls into the Forest as night falls").
4. **Separates natural causation from gamemaker intervention** — the existing `AreaEvent` enum currently mixes "wildfire spreads in dry forest" with "fireballs from the sky." This spec untangles them.

Non-goals:

- Implementing the gamemaker-event side of the split. That gets its own spec; this one only carves the boundary.
- Modeling forecasts, atmospheric fronts, pressure systems, or any meteorological realism beyond per-area Markov transitions.
- Sponsoring weather-aware items, shelter-class items, or gamemaker weather control. Future work.

## Three-Layer Architecture

The current `AreaEvent` enum does two unrelated jobs: natural consequences of conditions (wildfire, flood) and gamemaker interventions (which are not yet implemented but are the obvious next step — fireballs, mutts). This spec splits them and adds the missing weather layer underneath.

| Layer | What it is | Lifecycle | Examples |
|---|---|---|---|
| **Weather** *(new)* | Persistent atmospheric state per area | Evolves slowly per-day-phase | Clear, LightRain, HeavyRain, Storm, Snow, Fog |
| **NaturalHazard** *(refactor of `AreaEvent`)* | Acute event caused by terrain + weather | One-shot, probabilistic | Wildfire, Flood, Avalanche, Landslide, Heatwave, Earthquake |
| **GamemakerEvent** *(new, separate spec)* | Imposed Capitol intervention | One-shot, narratively-driven, weather-independent | Fireballs, Mutts, ForceFieldShift, AreaClosure |

This spec covers Weather and NaturalHazard. GamemakerEvent is filed as a separate brainstorm.

### Carving the Existing `AreaEvent`

Current `AreaEvent` variants and where they go:

- **Stay as NaturalHazard:** Wildfire, Flood, Earthquake, Avalanche, Blizzard, Landslide, Heatwave, Rockslide.
- **Removed (folded into Weather):** Sandstorm — the weather state's modifiers do the damage; no separate hazard variant. Drought — dropped entirely; long-term aridity is a terrain property already.
- **Future GamemakerEvent variants:** none migrate from `AreaEvent` directly. The gamemaker spec defines its own enum.

`Heatwave` is acute, not a sustained weather state — gamemakers cranking the temperature in an already-hot area. It stays a NaturalHazard, weighted toward hot terrain (Desert, Badlands).

## Weather State Space

Discrete enum with derived modifier methods (model C — same pattern as `EmotionLabel` derived from axes in the emotions spec). The variant set is curated and small; consumers read modifier methods, never the variant name directly, so adding a state means adding modifier methods rather than patching match arms across the codebase.

```rust
pub enum Weather {
    Clear,
    Overcast,
    Fog,
    LightRain,
    HeavyRain,
    Storm,
    LightSnow,
    HeavySnow,
    Sandstorm,
}
```

### Derived Modifier Surface

Every variant implements:

```rust
impl Weather {
    pub fn visibility_modifier(&self) -> i8;       // applied to combat/detection rolls
    pub fn movement_modifier(&self) -> i8;         // applied to effective speed
    pub fn temperature_band(&self) -> TemperatureBand;  // Cold/Cool/Mild/Warm/Hot/Scorching
    pub fn hide_modifier(&self) -> i8;             // bonus or penalty to hide rolls
    pub fn exposure_health_tick(&self) -> u8;      // damage per phase if exposed (0 = no tick)
    pub fn exposure_sanity_tick(&self) -> u8;      // sanity damage per phase if exposed
    pub fn hazard_weight_multiplier(&self, hazard: NaturalHazard) -> f32;
    pub fn is_extreme(&self) -> bool;              // Storm, HeavySnow, Sandstorm
    pub fn is_precipitating(&self) -> bool;
}
```

### Starter Modifier Table (per-phase tribute effects)

Final values to be tuned during implementation; these are the design targets.

| Weather | Visibility | Movement | Hide | Health tick (exposed) | Sanity tick (exposed) |
|---|---|---|---|---|---|
| Clear | 0 | 0 | 0 | 0 | 0 |
| Overcast | 0 | 0 | 0 | 0 | 0 |
| Fog | −6 | 0 | **+3** | 0 | 0 |
| LightRain | −1 | −1 | 0 | 0 | 0 |
| HeavyRain | −3 | −2 | 0 | 1 | 0 |
| Storm | −5 | −3 | 0 | 2 | 0 |
| LightSnow | −1 | −2 | **−2** | 0 | 0 |
| HeavySnow | −3 | −4 | **−3** | 2 | 0 |
| Sandstorm | −7 | −4 | 0 | 2 | 1 |

Principles baked in:

- **Visibility hits combat/detection** — added to both `attack_contest` rolls (attacker and defender). Heavy fog and sandstorms make every attack feel like fumbling.
- **Movement modifier reduces effective speed** in `travels`, same path as the existing `BROKEN_BONE_LEG_SPEED_REDUCTION`.
- **Hide modifier** — fog *helps* hiding (low visibility everywhere); snow *hurts* hiding (visible tracks).
- **Exposure ticks fire only if exposed** — i.e., the tribute is not Hidden *and* the tribute is not in a shelter-providing area. For v1, "shelter" is defined as: `TributeStatus::Hidden` is set, OR the tribute's current `Area` has `BaseTerrain::UrbanRuins`. Shelter-class items are out of scope.
- **Sanity tick on Sandstorm only** — most weather is irritating, sandstorms are *psychologically* grinding (per Q5 discussion).

## Per-Area, Per-Phase Lifecycle

Each `Area` (or `AreaDetails`) carries:

```rust
pub struct AreaWeather {
    pub current: Weather,
    pub phases_in_state: u8,    // turns since last transition; capped to avoid overflow
}
```

Each game day has two phases (day, night) per the existing day/night cycle in `process_turn_phase`. At each phase boundary:

1. **Weather transition roll** — Markov chain on current weather, conditioned on terrain.
2. **Hazard roll** — using the (possibly-new) weather + terrain, with a transition bonus if the weather just escalated.
3. **Apply modifiers** — visibility, movement, exposure ticks applied to tributes in the area for the duration of the phase.
4. **Emit `WeatherChanged` event** if the state changed; otherwise silent.

This is twice per game day. Within a phase, weather is constant.

### Starting Weather

At game creation, for each area:

- **Cornucopia** always starts `Clear`. Mirrors the bloodbath staging — gamemakers want maximum visibility for the opening spectacle.
- **All other areas:** sampled from the stationary distribution of the area's terrain transition matrix, biased toward neutral states. Implementation: down-weight `is_extreme()` variants by an additional factor (e.g., ×0.25) when sampling Day 1 weather. This gives Day 1 atmospheric variety without immediate disaster.

## Markov Chain Evolution

Each `BaseTerrain` has its own transition matrix governing what weather is plausible in that terrain. Most cells are `0`; the matrix is sparse.

```rust
fn transition_weights(terrain: BaseTerrain, current: Weather) -> &'static [(Weather, u32)];
```

Returned weights feed a weighted random sample. Sample illustrates the shape (final tables tuned during implementation):

**Forest, current = Clear:** `[(Clear, 60), (Overcast, 25), (Fog, 8), (LightRain, 7)]`
**Forest, current = HeavyRain:** `[(HeavyRain, 30), (Storm, 20), (LightRain, 35), (Overcast, 15)]`
**Desert, current = Clear:** `[(Clear, 70), (Overcast, 15), (Sandstorm, 10), (HeavyRain, 5)]`
**Desert, current = HeavyRain:** `[(HeavyRain, 20), (Storm, 10), (LightRain, 30), (Clear, 40)]` *(rain doesn't last in desert)*
**Tundra, current = Clear:** `[(Clear, 55), (Overcast, 20), (LightSnow, 15), (Fog, 10)]`
**Tundra, current = HeavySnow:** `[(HeavySnow, 40), (LightSnow, 35), (Storm, 10), (Overcast, 15)]`

Transitions to states with `0` weight are impossible (no `Sandstorm` in Tundra, no `HeavySnow` in Desert). Each terrain's table embeds its climate.

The full per-terrain matrix is part of implementation, not this spec.

## Hazard System (Transition-Aware)

Hazard rolls happen *after* the weather transition each phase. Weights are computed from `(weather, terrain, hazard)`, with a bonus if the weather just escalated.

### `NaturalHazard` Enum

```rust
pub enum NaturalHazard {
    Wildfire,
    Flood,
    Earthquake,
    Avalanche,
    Blizzard,
    Landslide,
    Heatwave,
    Rockslide,
}
```

Same set as today's `AreaEvent` minus Sandstorm and Drought. Severity, survival rolls, status-effect application, and emotion triggers (existing "hit by area event" trigger) all carry over from the current `AreaEvent` machinery — this is a rename + scope-narrowing, not a behavioral rewrite of hazards themselves.

### Weight Computation

```rust
fn compute_hazard_weights(
    weather: Weather,
    terrain: BaseTerrain,
    just_escalated: bool,
) -> Vec<(NaturalHazard, u32)>;
```

`just_escalated` is `true` when the weather changed *and* the new state has a strictly higher severity rank than the previous one. Severity ranks (low → high):

```
Clear / Overcast / Fog        = 0  (calm)
LightRain / LightSnow         = 1  (mild)
HeavyRain / HeavySnow         = 2  (heavy)
Storm / Sandstorm             = 3  (extreme)
```

A `Clear → HeavyRain` transition escalates by 2 ranks → `just_escalated = true`. A `HeavyRain → LightRain` transition de-escalates → `just_escalated = false`. A `Clear → Overcast` transition is same-rank → `just_escalated = false`.

For each plausible `(weather, terrain)` combination, a small table maps to hazards:

| Weather | Terrain | Hazard | Base weight | Transition bonus (if escalated) |
|---|---|---|---|---|
| HeavyRain | **Desert / Badlands** | **Flood** | **30** | **+50** |
| HeavyRain | Wetlands | Flood | 12 | +20 |
| HeavyRain | Forest / Grasslands | Flood | 6 | +15 |
| HeavyRain | Mountains / Highlands | Landslide | 15 | +25 |
| Storm | Desert / Badlands | Flood | 35 | +60 |
| Storm | any lowland | Flood | 18 | +30 |
| Storm | Mountains | Landslide | 20 | +30 |
| HeavySnow | Mountains | Avalanche | 20 | +25 |
| HeavySnow | Highlands | Avalanche | 12 | +20 |
| Clear | Desert / Badlands | Heatwave | 4 | n/a |
| (any) | (any) | Earthquake | 1 | n/a *(weather-independent baseline)* |

After weights are summed, roll: pick one hazard from the weighted distribution, or roll `None` (no hazard this phase) using a baseline `no-hazard` weight (e.g., 50). The full table is implementation work.

**Two independent terrain effects are now in play:**

1. **Terrain shapes which weather is common** (the Markov transition table — deserts trend Clear and Sandstorm, tundras trend Snow).
2. **Terrain shapes which hazards a given weather triggers** (the weight table — HeavyRain in a Desert is much more likely to flash-flood than HeavyRain in Wetlands).

Earthquakes and Rockslides remain weather-independent (small constant baseline weight regardless of weather) — they're geological, not atmospheric. They still fit under NaturalHazard because they're caused by the world, not the gamemakers.

### The "Clear → HeavyRain in Desert" Worked Example

1. Phase boundary. Area = Desert, current weather = Clear.
2. Markov roll on Desert/Clear table → `HeavyRain` (5% chance).
3. `just_escalated = true` (Clear=0 → HeavyRain=2, two-rank jump).
4. `compute_hazard_weights(HeavyRain, Desert, true)` → `[(Flood, 30 + 50 = 80), (None, 50)]`.
5. Roll: Flood fires (≈61% probability).
6. Existing `AreaEvent::Flood` survival/damage logic runs. `WeatherChanged { area, Clear → HeavyRain, Day }` event is emitted, plus the existing flood event.

In contrast: same transition in Wetlands would produce `Flood` weight `12 + 20 = 32` against `None` `50`, ≈39% — a coin flip.

## Emotion Integration

Per the emotions spec, exposure to harsh weather produces a new dedicated trigger weaker than the punctual area-event trigger:

```rust
EmotionTrigger::ExposedToHarshWeather  // axis_changes: morale -1, composure -2
```

Fired once per phase per exposed tribute on weather states with `exposure_health_tick() > 0` or `exposure_sanity_tick() > 0`. A tribute caught in a Storm during the day phase, in `Cornucopia` (no shelter) and not Hidden, accrues one trigger that phase.

The existing `EmotionTrigger::HitByAreaEvent` (from emotions spec) continues to fire when a NaturalHazard hits the tribute, separately and additively.

## Events

New event:

```rust
pub struct WeatherChanged {
    pub area: Area,
    pub from: Weather,
    pub to: Weather,
    pub phase: DayPhase,  // Day or Night
}
```

Emitted only when the Markov roll changes the state. Silent on no-op transitions (e.g., `Clear → Clear`). The announcer can lean on these for atmospheric beats ("as night falls, a storm rolls into the Forest").

NaturalHazard events continue to fire from the existing `AreaEvent` plumbing (renamed). No change to event volume from hazards.

## Data Model Changes

### `Area` / `AreaDetails` (`game/src/areas/mod.rs`)

Add a `weather: AreaWeather` field. Migration sets initial values per the starting-weather rules.

### `Weather` enum (new `game/src/areas/weather.rs`)

The enum, its modifier methods, and the Markov transition function live here. Per-terrain transition tables and stationary distributions in a sibling `transition_tables.rs`.

### `NaturalHazard` (refactor of existing `AreaEvent`)

Rename `AreaEvent` → `NaturalHazard` in `game/src/areas/events.rs`. Remove `Sandstorm` and `Drought` variants. All `FromStr` / `Display` / `random_for_terrain` impls update accordingly. Existing `EventSeverity` and `SurvivalResult` types stay as-is.

### Hazard weight table (new `game/src/areas/hazards.rs` or extension of `events.rs`)

The `compute_hazard_weights` function and its underlying table.

### `shared/` DTOs

Add a `weather` field to whatever `DisplayArea` (or equivalent area DTO) the frontend consumes. New addition is non-breaking; existing API consumers ignore unknown fields.

## Integration Points

- **Game loop (`process_turn_phase` and surrounding cycle code)** — at each phase boundary, for each area: roll weather transition → roll hazard → apply modifiers / emit events.
- **Combat (`attack_contest` in `tributes/mod.rs`)** — read current area weather, add `visibility_modifier()` to both attacker and defender rolls.
- **Movement (`travels` in `tributes/mod.rs`)** — apply `movement_modifier()` to effective speed.
- **Hiding (`hides` and related)** — apply `hide_modifier()` to hide success rolls.
- **Status processing (`process_status`)** — if the tribute is exposed, apply `exposure_health_tick()` and `exposure_sanity_tick()`; fire `ExposedToHarshWeather` emotion trigger.
- **Hazard application** — existing `apply_area_effects` keeps its shape; the input is now selected by `compute_hazard_weights` instead of `AreaEvent::random_for_terrain`.
- **Frontend rendering** — current weather per area is surfaced via the area DTO; UI displays it (covered by progressive-display spec).

## Testing Strategy

Following the project's existing rstest pattern:

- **Modifier methods** — every weather variant returns the documented modifiers; `is_extreme()` flags the right variants.
- **Markov transitions** — given a fixed RNG seed, transitions follow the terrain table; impossible transitions (zero-weight cells) never occur; stationary distribution sampling produces plausible Day 1 weather.
- **Hazard weighting** — `(HeavyRain, Desert)` produces near-guaranteed Flood on transition; `(HeavyRain, Wetlands)` produces ≈coin-flip Flood on transition; weather-independent hazards (Earthquake) fire under any weather at low rate.
- **Exposure logic** — Hidden tributes don't take exposure ticks; tributes in `UrbanRuins` don't take exposure ticks; tributes in any other area in extreme weather do.
- **Cornucopia start** — game creation across many seeds always produces `Clear` for Cornucopia.
- **Per-area independence** — different areas in the same phase can have different weather and different hazard outcomes.
- **Emotion trigger emission** — `ExposedToHarshWeather` fires once per phase per exposed tribute on qualifying weather states.
- **Event emission** — `WeatherChanged` fires only on actual state changes; no event on `Clear → Clear`.
- **Refactor regression** — every existing `AreaEvent` test that referenced `Sandstorm` or `Drought` is converted, deleted with rationale, or moved to the weather state's modifier tests.

## Migration / Backward Compatibility

- **`AreaEvent` rename to `NaturalHazard`** — all internal references updated. SurrealDB schema field that stored `AreaEvent` strings continues to work for the kept variants; persisted games that recorded `Sandstorm` or `Drought` need a one-time migration to either drop those records or convert `Sandstorm` records to a `Weather::Sandstorm` state on the affected area at hydration time. Trivial migration entry under `migrations/definitions/`.
- **`Area` schema** — new `weather` field added, defaulted to `Clear` for in-flight games (or rolled fresh per the starting-weather rules on first hydration; implementation choice).
- **API DTOs** — additive only. Removing `Sandstorm`/`Drought` from any `AreaEvent`-shaped DTO is breaking and must be coordinated with the frontend.

## Open Questions for Implementation

These don't block writing the implementation plan but the implementer will resolve them:

- Final modifier values in the per-state table (the `−6`, `−7`, etc. are starting points; Q5 confirmed the shape, not the exact numbers).
- Final per-terrain transition matrices (one matrix per `BaseTerrain` × current weather).
- Final hazard weight table including cells the design didn't enumerate (e.g., HeavySnow in Forest? LightSnow in Desert if it can even occur?).
- Whether `phases_in_state` is read anywhere or is debug-only telemetry. (Could later drive things like "weather has been bad for 5 phases → morale tick.")
- Whether `WeatherChanged` events are emitted before or after `NaturalHazard` events from the same phase. (Design intent: Weather first, then Hazard, so the announcer has the cause before the effect.)
- Whether the per-phase Day 1 starting roll uses a dedicated "neutral-biased" sampler or just calls the standard sampler with a flag.

## Out of Scope

- The GamemakerEvent system. Filed as a separate brainstorm. This spec carves the boundary; gamemaker design is its own conversation.
- Forecasts, weather control items, weather-aware sponsor gifts, or weather-aware tribute traits.
- Cross-area weather coupling (no fronts moving from West → East over time). Each area is independent.
- Visual frontend representation of weather. Covered by the progressive-display spec.
- Announcer prompt updates to use `WeatherChanged` events. Handled when the announcer integration lands.
