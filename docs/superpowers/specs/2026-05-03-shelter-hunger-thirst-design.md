# Shelter + Hunger/Thirst — Design

**Status:** Draft
**Date:** 2026-05-03
**Crate(s) primarily affected:** `game/`, with shared/api/web echoes
**Related specs:** `2026-05-02-weather-system-design.md`, `2026-05-02-tribute-emotions-design.md`, `2026-04-17-terrain-biome-system-design.md`, `2026-04-25-tribute-alliances-design.md`
**Related beads:** `hangrier_games-0yz` (this spec), `hangrier_games-ex3f` (resource sharing follow-up)

## Goals

Add an interlocking shelter / hunger / thirst layer that:

1. Makes geography (terrain biome) and weather mechanically *expensive* to ignore.
2. Gives the Brain a survival decision branch that can override combat/movement when desperate.
3. Wires the existing-but-unused `TributeStatus::Starving` and `TributeStatus::Dehydrated` into a real lifecycle.
4. Closes the conditional "Hungry / low on supplies" trigger left open in the emotions spec.
5. Replaces the placeholder shelter definition (`Hidden` OR `UrbanRuins`) used in the weather spec with a proper, distinct shelter system.

Non-goals (v1):

- Cooking, food preparation, contamination tracking. *Food poisoning chance* on consumption is in scope (cheap, generates drama). Anything beyond a single roll is not.
- Constructed/persistent shelter, capacity rules, ally co-sheltering bonuses.
- Inter-tribute resource sharing or taking (filed: `hangrier_games-ex3f`).
- Cannibalism / corpse consumption.
- Sponsor pool *content* and economic balance. Data-model affordance only.
- Long-run balance tuning. v1 ships with provisional numbers; expect tuning passes.

## Weight Class

Mid-weight survival loop ("weight B" in the brainstorm). Survival meaningfully shapes play but does not dominate it. Roughly 1.5–2× the implementation footprint of the emotions spec.

## Architecture Overview

Three interlocking subsystems, each modeled as a small pure module wired into the existing tick:

```
                ┌──────────────────────────┐
                │  Area shelter_quality()  │  pure: (terrain, weather) -> u8
                └──────────────────────────┘
                              │
                              ▼
   ┌──────────────────────┐  rolls   ┌──────────────────────────┐
   │  Action: SeekShelter │ ───────► │ Tribute.sheltered_until  │
   └──────────────────────┘          └──────────────────────────┘
                                                │
                                                ▼
       ┌────────────────────────────┐    ┌─────────────────────────┐
       │ Area forage_richness()     │    │ Survival tick()         │
       │ Area water_source()        │    │   hunger += k_h         │
       └────────────────────────────┘    │   thirst += k_t         │
                  │                      │   weather/shelter mods  │
       ┌──────────┴──────────┐           │   band crossings emit   │
       ▼                     ▼           │   drain HP at thresholds│
   Action: Forage      Action: Drink     └─────────────────────────┘
       │                     │                       │
       └─────────┬───────────┘                       │
                 ▼                                   ▼
           Tribute.hunger / Tribute.thirst    TributeKilled(Starvation/Dehydration)
                 ▲
                 │
       Action: Eat / Drink (item)  ◄── ItemType::Food(u8) / Water(u8)
```

The three subsystems share two ground rules:

- **Shelter is per-tribute, not per-area.** An area's `shelter_quality` is the *substrate* a tribute rolls against; the resulting `Sheltered` window lives on the tribute.
- **Hunger and thirst are debt counters**, not 0–100 bars. They tick *up*; eating/drinking ticks them *down*. Bands map directly onto existing `TributeStatus` variants.

## Shelter

### Area shelter quality (derived)

A pure function in a new `game/src/areas/shelter.rs`:

```rust
pub fn shelter_quality(terrain: BaseTerrain, weather: &Weather) -> u8 { ... }
```

Returns 0 (no shelter possible) through 4 (excellent shelter). Pure of any tribute state.

Provisional table covering all current `BaseTerrain` variants (pre-weather):

| Terrain     | Base quality | Notes                                |
|-------------|--------------|--------------------------------------|
| UrbanRuins  | 3            | shelters, structures, cover          |
| Forest      | 2            | dense canopy                         |
| Jungle      | 2            | dense canopy + foliage               |
| Mountains   | 2            | caves, overhangs                     |
| Geothermal  | 2            | warm rock, vents, scattered cover    |
| Wetlands    | 1            | reeds, scattered trees               |
| Highlands   | 1            | rocky outcrops, scattered cover      |
| Clearing    | 1            | scattered trees, low brush           |
| Grasslands  | 1            | rare hedges, low cover               |
| Badlands    | 1            | crags and gullies, sparse            |
| Tundra      | 0            | open frozen ground, no cover         |
| Desert      | 0            | nothing to hide under                |

Note: `Cornucopia` is an `Area`, not a `BaseTerrain`. Its shelter quality follows from whichever terrain the area is assigned (typically `Clearing` or `Grasslands` → 1).

Weather modifiers on top (see Weather subsystem below for the v1 minimal enum):

- `HeavyRain`, `Blizzard`: −1 to all (visibility/exposure penalty for the *roll*, but successful shelter still works — see action below).
- `Heatwave`: UrbanRuins/Mountains/Geothermal keep their full value (stone is cool); Forest/Jungle stay; Tundra/Desert pinned at 0.
- `Clear`: no modifier.

### Action: `SeekShelter`

A tribute spends one action to attempt to shelter. The roll:

```
roll = uniform(0, 4)
modifier = trait_bonus  // see Traits, below
if roll + modifier <= shelter_quality(area.terrain, weather):
    tribute.sheltered_until = now + N_phases
    emit ShelterSought { success: true }
else:
    emit ShelterSought { success: false }
```

`N_phases` (provisional): **3 phases** — long enough to weather a storm and forage; short enough that long-term hiding requires repeated effort.

The action is *available* even at `shelter_quality == 0`, but always fails there (events still log it; useful for "Cato spent the night exposed in the desert" flavor).

### `Sheltered` is its own thing

`Sheltered` is **not** the same as `Hidden`.

- `Hidden` is a stealth state from the existing system — it conceals the tribute from other tributes.
- `Sheltered` is an environmental protection state — it shields from weather exposure ticks and modulates survival ticks.

A tribute can be both, neither, or either independently. `Hidden` no longer participates in weather exposure logic; the weather spec's interim definition of "shelter" (`Hidden` OR `UrbanRuins`) is fully *replaced* by `sheltered_until.is_some()` checked in `tick_survival` and weather exposure pipelines.

### Allies do not co-shelter (v1)

Same-area allies each roll independently. No group bonus, no shared `sheltered_until`. Filed for future consideration if playthroughs show this is too punishing.

## Hunger & Thirst

### Counters and bands

Two `u8` debt counters on the tribute, default 0:

```rust
pub hunger: u8,   // 0 = Sated; ticks up each phase
pub thirst: u8,   // 0 = Sated; ticks up each phase
```

Band thresholds (provisional, tunable):

| Hunger value | Band            | Public event? | Status set?            | Brain effect             |
|--------------|-----------------|---------------|------------------------|--------------------------|
| 0            | Sated           | —             | clear `Starving`       | none                     |
| 1–2          | Peckish         | —             | —                      | flavor only              |
| 3–4          | Hungry          | yes (on entry)| —                      | weighting + emotion trig |
| 5+           | Starving        | yes (on entry)| set `Starving`         | override eligible        |

| Thirst value | Band            | Public event? | Status set?            | Brain effect             |
|--------------|-----------------|---------------|------------------------|--------------------------|
| 0            | Sated           | —             | clear `Dehydrated`     | none                     |
| 1            | Thirsty         | —             | —                      | flavor only              |
| 2            | Parched         | yes (on entry)| —                      | weighting + emotion trig |
| 3+           | Dehydrated      | yes (on entry)| set `Dehydrated`       | override eligible        |

Thirst escalates faster than hunger by design — a body lasts about 3× longer without food than without water; the spec compresses that ratio into game phases.

### Tick rate

Each phase, for each living tribute:

```
hunger += hunger_tick_for(tribute, weather, sheltered)
thirst += thirst_tick_for(tribute, weather, sheltered)
```

Where:

```
hunger_tick_for:
    base = 1
    if tribute.strength is high (≥ threshold):  base += 1   // bigger body, more calories
    if tribute.strength is low (≤ threshold):   base may be 0 this phase (every other phase)
    if not sheltered AND weather is cold (Blizzard/HeavyRain): base += 1
    return base

thirst_tick_for:
    base = 1
    if tribute.stamina is high:  base += 1
    if tribute.stamina is low:   base may be 0 this phase
    if not sheltered AND weather is hot (Heatwave/Clear+Desert): base += 1
    return base
```

Net effect: a tribute with no food/water and no shelter typically reaches `Starving` ≈ day 3 morning, `Dehydrated` ≈ day 2 night. Tunable later.

### Death curve: escalating drain

Once a tribute enters `Starving` or `Dehydrated`, an HP drain begins each phase, *escalating*:

```rust
pub starvation_drain_step: u8,    // resets to 0 on partial recovery (eating)
pub dehydration_drain_step: u8,
```

Each phase in status:

```
starvation_drain_step += 1
hp -= starvation_drain_step       // -1, then -2, then -3, ...
```

(Identical for dehydration.) Both can stack on the same tribute; combined drain in late stages is brutal and intentional.

Eating any food drops the tribute back to the `Hungry` band (or below) and resets `starvation_drain_step` to 0. Drinking does the same for thirst.

If HP reaches 0 while either status is set, the death is routed through `TributeKilled` with the dominant cause (whichever drain landed the killing blow; ties resolved as Dehydration first since it ticks first within a phase).

## Food & Water Items

### `ItemType` extension

Two new variants on `ItemType`:

```rust
pub enum ItemType {
    Consumable,      // existing — "potion-like" non-food consumables
    Weapon,          // existing
    Food(u8),        // new — value = hunger debt removed on consumption
    Water(u8),       // new — value = thirst debt removed on consumption
}
```

Typical food values: `1` (snack), `3` (ration), `5` (feast).
Typical water values: `1` (sip), `2` (canteen), `3` (full waterskin).

### Actions

- **`Eat(item)`** — consume a `Food(n)` item from inventory. `tribute.hunger = saturating_sub(n)`. Resets `starvation_drain_step`. Emits `Ate`. Item is destroyed.
- **`Drink(item)`** — same shape for `Water(n)`. Emits `Drank { source: Item(...) }`.
- **`Drink(area)`** — terrain action at a hex with `water_source(...) > 0`. No item needed. Resolves to `tribute.thirst -= 1` on success. Emits `Drank { source: Terrain(...) }`.
- **`Forage`** — terrain action at a hex with non-zero forage richness. Roll vs. richness. On success, `tribute.hunger -= 1`. Emits `Foraged`.

### Food poisoning

Each `Eat` and each terrain `Drink`/`Forage` rolls for contamination:

```
poisoning_chance = base + weather_modifier - trait_modifier
```

Provisional defaults:

- `base` = 5% for `Eat(item)`, 10% for `Forage`, 15% for terrain `Drink`.
- Weather modifier: `Heatwave` +5%, `Flood`/`HeavyRain` +5% on terrain water.
- Trait modifier: see `ResourcefulForager` in Traits.
- On hit: tribute gains `TributeStatus::Poisoned` for a short window (existing status, existing handling).

The roll is silent on failure (no event); success emits the existing poisoning flow.

### Terrain affordances (forage and water)

Two more pure functions in `game/src/areas/`:

```rust
pub fn forage_richness(terrain: BaseTerrain) -> u8 { ... }     // 0..=4
pub fn water_source(terrain: BaseTerrain, weather: &Weather) -> u8 { ... }  // 0..=3
```

Provisional terrain table covering all current `BaseTerrain` variants:

| Terrain     | forage_richness | base water_source | water in HeavyRain |
|-------------|-----------------|--------------------|--------------------|
| Wetlands    | 3               | 3                  | 3                  |
| Forest      | 2               | 2                  | 3                  |
| Jungle      | 3               | 2                  | 3                  |
| Geothermal  | 1               | 2 (hot springs)   | 2                  |
| Mountains   | 1               | 2 (springs)        | 2                  |
| Highlands   | 1               | 1                  | 2                  |
| Clearing    | 1               | 1                  | 2                  |
| UrbanRuins  | 2 (scavenge)    | 1 (rare)           | 2                  |
| Grasslands  | 1               | 0                  | 2                  |
| Badlands    | 0               | 0                  | 1                  |
| Tundra      | 0               | 1 (snowmelt)       | 1                  |
| Desert      | 0               | 0                  | 1                  |

Note: `Cornucopia` is an `Area`, not a terrain — its forage/water values follow from the terrain it is assigned.

`Heatwave` halves base `water_source` (rounded down).

## Looting on Death (Adjacent Fix, In Scope)

The food/water economy assumes carried items can re-enter the loot pool when a tribute dies. Currently no code transfers `Tribute.items` to `Area.items` on death. This spec adds that step:

- On the `RecentlyDead → Dead` lifecycle transition (or on the kill event, whichever the lifecycle code prefers — see implementation notes), drain `tribute.items` into the area's `items` vector at the tribute's last position.
- The dead tribute's inventory is cleared.
- A `TributeLooted` event (or extension of the existing death event payload) records that N items dropped, for timeline visibility. *(Open implementation question — see below.)*

This change is in scope because:
- It's small and self-contained.
- Without it, food/water spawned via sponsor or item drops becomes a one-shot resource (sticks on the first body).
- The same plumbing is what the future resource-sharing spec (`hangrier_games-ex3f`) will hook into.

## Brain Integration

Mirrors the emotion spec's "override-then-weight" pattern.

### Override states (checked first, before normal Brain scoring)

In order:

1. **`Dehydrated` + at water-source terrain** → `Drink(area)`.
2. **`Dehydrated` + Water item in inventory** → `Drink(item)`.
3. **`Starving` + Food item in inventory** → `Eat(item)`.
4. **`Starving` + at forageable terrain + not in active combat** → `Forage`.

Combat preempts all overrides. A Starving tribute under attack still defends.

### Soft weighting (when no override fires)

Layered onto the existing Brain scoring:

| State                | Weight nudges                                                             |
|----------------------|---------------------------------------------------------------------------|
| Peckish / Thirsty    | small bonus to picking up Food/Water items if encountered                 |
| Hungry / Parched     | move-toward-food-or-water-terrain weight; pick up Food/Water always       |
| Starving / Dehydrated (override miss) | strong move-toward-shelter/water bias; flee combat unless cornered |

`Hungry`/`Parched` band entry also fires the emotion spec's "Hungry / low on supplies" trigger row (which was held conditional on this system existing).

## Traits

Two new traits (see `game/src/tributes/traits.rs`):

| Trait              | Effect                                                                                          |
|--------------------|-------------------------------------------------------------------------------------------------|
| `Builder`          | +1 to `SeekShelter` roll modifier. Successful shelter lasts +1 phase.                            |
| `ResourcefulForager` | +1 to `Forage` roll modifier; halves food poisoning chance on `Forage` and `Eat(item)`.        |

Trait distribution is left to the existing trait roll/assignment system; no special weighting in v1.

## Events

Six new typed events. Public events flow through the existing timeline pipeline; private events are not surfaced in the Action panel by default, only in the per-tribute Inspect drilldown's recent-actions feed.

| Event                  | When                                                              | Payload                                                              | Public? |
|------------------------|-------------------------------------------------------------------|----------------------------------------------------------------------|---------|
| `HungerBandChanged`    | tribute crosses Sated↔Peckish↔Hungry↔Starving                     | `tribute_ref`, `from_band`, `to_band`                                | only Hungry/Starving entries are public |
| `ThirstBandChanged`    | tribute crosses Sated↔Thirsty↔Parched↔Dehydrated                  | `tribute_ref`, `from_band`, `to_band`                                | only Parched/Dehydrated entries are public |
| `ShelterSought`        | `SeekShelter` action ran                                          | `tribute_ref`, `area_ref`, `success: bool`, `roll`                   | private |
| `Foraged`              | `Forage` action ran                                               | `tribute_ref`, `area_ref`, `success: bool`, `debt_recovered: u8`     | private |
| `Drank`                | any drink action                                                  | `tribute_ref`, `source: Terrain(area_ref) \| Item(item_ref)`, `debt_recovered: u8` | private |
| `Ate`                  | `Eat(item)` action                                                | `tribute_ref`, `item_ref`, `debt_recovered: u8`                      | private |

Hunger and thirst band events are kept *separate* (rather than collapsed into one `SurvivalBandChanged`) so the timeline UI can filter and color them independently.

Two new `TributeKilled` causes (extension of the existing death event):

- `Cause::Starvation`
- `Cause::Dehydration`

Provisional in-engine death-line copy (Action panel / timeline):

- "{name} starved." (timeline)
- "{name} died of thirst." (timeline)
- "{name} collapsed from hunger in the {area_name}." (richer log line for Action panel)

## Data Model Changes

### `Tribute` (`game/src/tributes/mod.rs`)

```rust
#[serde(default)] pub hunger: u8,
#[serde(default)] pub thirst: u8,
#[serde(default)] pub sheltered_until: Option<u32>,        // phase index; None = exposed
#[serde(default)] pub starvation_drain_step: u8,
#[serde(default)] pub dehydration_drain_step: u8,
```

`sheltered_until` is stored on the tribute (parallel to the existing `is_hidden: bool`), *not* as a `TributeStatus` variant. Rationale: tributes can be Sheltered *and* Wounded/Sick/etc., which the mutually-exclusive `TributeStatus` enum doesn't model.

`_drain_step` fields are stored (rather than recomputed) so save/load and websocket replay produce identical drain trajectories.

### `TributeStatus`

No new variants. `Starving` and `Dehydrated` already exist; this spec finally drives them.

### `ItemType`

Add `Food(u8)` and `Water(u8)` variants. **Migration risk** — see Open Implementation Questions.

### `Area` / `AreaDetails`

No new persisted fields. `shelter_quality`, `forage_richness`, `water_source` are pure functions of `(terrain, weather)`.

### Traits

Add `Builder` and `ResourcefulForager` to the `Trait` enum.

## Migration / Backward Compatibility

- **Tribute fields:** all new fields use `serde(default)`, so existing JSON game saves deserialize cleanly with all counters at 0 / `None`. SurrealDB schema needs `DEFINE FIELD … DEFAULT …` updates (`hunger: int = 0`, `thirst: int = 0`, `sheltered_until: option<int>`, drain steps `= 0`).
- **In-flight games:** existing games on load become Sated, exposed, no drain. They begin accruing on the next phase. No retroactive penalties.
- **`ItemType` enum bump:** the format used to persist `ItemType` (string tag, full enum, integer discriminant?) determines the migration cost. See open question below.
- **Weather spec interop:** the weather exposure tick must switch from "Hidden OR UrbanRuins" to `tribute.sheltered_until.is_some_and(|p| p > now)` once this spec lands. Hard cutover; no flag.

## Frontend Presentation

### Tribute card — bottom state strip

Extend the emotion-spec state strip with two pips:

- 🍗 hunger band: 4-state (Sated=hidden, Peckish=dim, Hungry=amber, Starving=red pulse).
- 💧 thirst band: 4-state same color logic.

Pips render only when band > Sated. Hover/tap → tooltip with the raw counter and band, e.g. `"Hunger 3 — Hungry"`, and (when applicable) a drain line: `"Starving — losing 3 HP/phase"`.

A house glyph (🏠) renders adjacent to the strip when `sheltered_until.is_some()` and active, with phases-remaining number.

### Tribute Inspect drilldown — Survival panel

Adds a new "Survival" panel to the Inspect view (the same drilldown introduced by the emotions spec).

- Numeric counter and band: `Hunger 3 (Hungry band)` / `Thirst 2 (Parched band)`.
- Optional bar visualization, color-keyed by band.
- Drain step display when relevant: `"Starving — losing 3 HP/phase (next phase: 4)"`.
- Shelter line: `"Sheltered for 2 more phases"` or `"Exposed"`.
- Recent-actions feed (last 3 of: forage / drink / eat / shelter rolls). This is where private events surface.

This panel SHOULD also be visible in any debug/dev tribute view.

### Map panel — terrain affordance hints

When a tribute is selected, surrounding hexes get small overlay glyphs (re-using the layering pattern from the weather spec):

- 💧 droplet — current `water_source(...) > 0` hex.
- 🌿 sprig — `forage_richness(...) > 0` hex.
- 🏠 house — `shelter_quality(...) ≥ 2` hex.

Glyphs respect the weather spec's accessibility convention: shape + glyph carry the meaning, color is decoration only.

### Action panel — event lines

- Public events get standard event-card treatment, color-keyed by severity:
  - "Katniss is now **Hungry**." (amber)
  - "Marvel is now **Starving**." (red, severity bump)
  - "Cato died of thirst in the Desert." (death severity)
- Private events (`ShelterSought`/`Foraged`/`Drank`/`Ate`) are not in the Action panel by default. A debug toggle in dev mode surfaces them. They always appear in the Inspect drilldown's recent-actions feed.

### Sponsor UI affordance (forward-compat)

The sponsor pool will eventually include Food/Water tiers. v1 does not build that UI; the data-model affordance (the new `ItemType` variants) is enough so the future sponsor work doesn't hit a wall.

### Accessibility

- Every band change line carries a text label, not just color.
- Pip elements have `aria-label` (e.g. `"Hunger: Starving"`).
- Map terrain affordance icons use shape + glyph, not color alone.

### Open frontend questions (flagged, deferred)

- Should successful `SeekShelter` get a per-hex transition glyph/animation similar to weather changes?
- Should crossing into Hungry/Parched fire a passive toast notification, or only the Action-panel line?

## Integration Points

- **Weather spec.** Replaces the placeholder shelter check with `tribute.sheltered_until`. Weather exposure ticks gate on shelter, hunger/thirst weather mods gate on shelter. Hard cutover. **At the time this spec lands, the weather subsystem is not yet implemented;** Plan 1 introduces a *minimal* `Weather` enum (`Clear`, `HeavyRain`, `Heatwave`, `Blizzard`) with a stub producer that returns `Clear` everywhere, sufficient to wire the consumer side. The full weather spec implementation later swaps the producer side without touching consumers.
- **Emotions spec.** Activates the "Hungry / low on supplies" trigger row. Brain override list interleaves with the emotion override list in a documented order: emotion overrides first (combat-driven rage etc.), survival overrides second, normal scoring last.
- **Terrain biome spec.** This is the system that finally gives the terrain table teeth — Desert lethality, Wetlands desirability, etc.
- **Alliance spec.** No direct interop in v1. The follow-up resource-sharing work (`hangrier_games-ex3f`) hooks here.
- **Game timeline / Action panel.** Six new event types feed the existing typed-event pipeline.
- **Combat spec / lifecycle.** Looting-on-death fix is a small adjacent change to the `RecentlyDead → Dead` transition.

## Testing Strategy

Convention: rstest inline in `game/`. Total v1 surface ≈ 30 unit + 5 integration cases.

### Unit tests

- `shelter::shelter_quality(terrain, weather)` — every `BaseTerrain` × representative weather. Anchors: Cornucopia in storm = 0; UrbanRuins ≥ 2 in any weather; Desert + Heatwave = 0.
- `areas::forage_richness(terrain)` — full table coverage, bounded.
- `areas::water_source(terrain, weather)` — full table; HeavyRain boosts Plains; Heatwave halves base; Wetlands always > 0.
- `tribute::tick_survival(weather, sheltered)` — pure function:
  - Sated, no weather, exposed → +1 hunger, +1 thirst.
  - Exposed in Heatwave → +2 thirst.
  - Exposed in Blizzard → +2 hunger.
  - Sheltered in same → +1 each (weather modifier suppressed).
  - High strength → hunger ticks every other phase.
  - High stamina → thirst ticks every other phase.
- `band(value)` boundaries — exact thresholds: Sated/Peckish/Hungry/Starving; Sated/Thirsty/Parched/Dehydrated. Off-by-one bait.
- `apply_starvation_drain(tribute)` — escalating: -1, -2, -3 across phases; resets on `Eat`.
- `apply_dehydration_drain(tribute)` — same, independently.
- Brain override branch (stubbed tribute):
  - Starving + Food in inventory → picks `Eat`.
  - Dehydrated + at water-source area → picks `Drink(area)`.
  - Starving + at forageable terrain + no inventory → picks `Forage`.
  - Starving + in active combat → does NOT override.
  - Hungry (not Starving) → no override; falls through to weighting.

### Integration tests

- 7-day deterministic run, 2 tributes, no items: both die of dehydration before day 4. (Confirms thirst-kills-first.)
- Same fixture with one tribute given a `Water(3)` item at start: that tribute survives an extra ~3 phases.
- Sheltered tribute in Heatwave does NOT accrue weather thirst tick (regression guard for shelter↔weather wire).
- Death routing: tribute reaches 0 HP while `Starving` → `TributeKilled { cause: Starvation }` in the timeline event stream.
- Band-crossing visibility: tribute crossing 3→4 hunger emits `HungerBandChanged { to: Hungry }` (public); 1→2 emits a private band change only.

### Out of v1 test scope

- Frontend rendering of pips, gauges, and map overlays. Manual smoke test, like other UI work.
- Sponsor pool food/water spawn balance.
- Long-run balance (this is what playthroughs are for).

## Open Implementation Questions

1. **`ItemType` migration.** What format does the persisted `Item.item_type` field use today — string tag, full enum JSON, integer discriminant? If string-tag with named variants, adding `Food(u8)`/`Water(u8)` requires either a custom serde shape (e.g. `"food:3"` or a struct form) or a tagged enum bump with a fallback for legacy `"Consumable"` rows. Audit before implementation begins.
2. **Looting event shape.** Add a new `TributeLooted { dropped: Vec<ItemRef> }` event, OR extend the existing kill/death event payload with a `dropped` field, OR just rely on `Area`-state change and emit nothing? Pick during implementation; either is acceptable for v1.
3. **Drain ordering within a phase.** "Dehydration ticks first" is asserted for tie-breaking the death cause; confirm this matches the implementation order chosen for `tick_survival`.
4. **Trait roll integration.** `Builder` and `ResourcefulForager` are new entries on the `Trait` enum; whether they slot into the existing trait-assignment weighting unchanged or need explicit weights is an implementation choice.
5. **Shelter on death/movement.** Does `sheltered_until` carry across area moves, or does any move clear it? Recommendation: any successful move clears it (you left the shelter), but the action that *moves into* a sheltering area does not auto-shelter. A fresh `SeekShelter` is required.

## Out of Scope (filed, or to be filed, as beads)

- `hangrier_games-ex3f` — Resource sharing/taking between allied tributes.
- (file if desired) Sponsor pool food/water content + economic balance pass.
- (file if desired) Cooking and food preparation system.
- (file if desired) Constructed/persistent shelter, capacity rules, group sheltering bonuses.
- (file if desired) Cannibalism / corpse consumption.
- (file if desired) Long-run balance tuning for hunger/thirst tick rates and band thresholds.
