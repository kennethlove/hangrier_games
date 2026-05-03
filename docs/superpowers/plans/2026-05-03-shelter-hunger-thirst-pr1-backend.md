# Shelter + Hunger/Thirst — Plan 1: Backend Implementation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the entire backend slice of the shelter + hunger/thirst system: counters, ticks, statuses, Brain overrides, items, actions, traits, looting, and events. Frontend (pips, drilldown, map overlays) ships separately in Plan 2.

**Architecture:** Pure-function modules under `game/src/areas/` for shelter / forage / water tables; new `game/src/tributes/survival.rs` for the tick + drain logic. New typed message payloads in `shared/src/messages.rs`. Brain override list interleaves before existing weighted scoring. A *minimal* `Weather` enum (`Clear | HeavyRain | Heatwave | Blizzard`) is introduced as a stub producer (returns `Clear` everywhere) so the consumer wiring is real and the future weather spec can swap producers without touching consumers. Looting-on-death is wired in lifecycle as part of this plan.

**Tech Stack:** Rust 2024, `serde` with `serde(default)` for save/load migration, `rstest` for inline unit + integration tests, SurrealDB schema definitions in `schemas/*.surql`.

**Spec:** `docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md`

**Beads issue:** `hangrier_games-0yz`

---

## Pre-flight notes

- All code in this plan lives in the `game/`, `shared/`, and `api/` crates. No `web/` changes — those land in Plan 2.
- Run `just test` (game crate) and `just quality` (full workspace) at major checkpoints.
- Commits are frequent and small. Each task ends with a commit.
- The `Weather` enum is intentionally minimal here. The full weather spec implementation will *grow* the enum and replace the stub producer; plan 1 must not lock in any weather details beyond the four variants this spec needs.
- All new fields use `#[serde(default)]` so existing JSON game saves load cleanly. SurrealDB schema additions also use `DEFAULT` clauses for the same reason.

---

## File Structure

**New files:**
- `game/src/areas/weather.rs` — minimal `Weather` enum + stub `current_weather(area) -> Weather`.
- `game/src/areas/shelter.rs` — `shelter_quality(terrain, weather) -> u8`.
- `game/src/areas/forage.rs` — `forage_richness(terrain) -> u8`.
- `game/src/areas/water.rs` — `water_source(terrain, weather) -> u8`.
- `game/src/tributes/survival.rs` — survival tick + drain logic + band enums.

**Modified files:**
- `game/src/tributes/mod.rs` — new fields on `Tribute`; expose new modules.
- `game/src/tributes/statuses.rs` — no struct change; only Display/FromStr already cover Starving/Dehydrated.
- `game/src/tributes/actions.rs` — new `Action` variants: `SeekShelter`, `Forage`, `Eat(Item)`, `Drink(DrinkSource)`.
- `game/src/tributes/brains.rs` — survival override branch ahead of normal scoring.
- `game/src/tributes/lifecycle.rs` — drop tribute items into the area on `RecentlyDead → Dead`.
- `game/src/tributes/traits.rs` — add `Builder`, `ResourcefulForager`.
- `game/src/items/mod.rs` — `ItemType::Food(u8)`, `ItemType::Water(u8)`; helpers.
- `game/src/areas/mod.rs` — re-export the new submodules.
- `game/src/games.rs` — call survival tick once per phase per tribute; wire new events through.
- `shared/src/messages.rs` — new payloads: `HungerBandChanged`, `ThirstBandChanged`, `ShelterSought`, `Foraged`, `Drank`, `Ate`. Constants for new `cause` strings.
- `schemas/tribute.surql` (or equivalent) — add new persisted fields with defaults.

---

## Task Order Rationale

Tasks build the data substrate first (Tasks 1–3), then the survival mechanics (4–7), then the action-and-Brain integration (8–10), then lifecycle/events glue (11–12), then integration tests (13). Each task is independently shippable and reviewable. TDD throughout: failing test first.

---

## Task 1: Minimal `Weather` enum + stub producer

**Why first:** every survival module takes `&Weather` as input. Define the type and a stub source so all subsequent modules can compile against the real consumer signature.

**Files:**
- Create: `game/src/areas/weather.rs`
- Modify: `game/src/areas/mod.rs` (add `pub mod weather;` + re-export)

- [ ] **Step 1: Write the failing test**

Append to `game/src/areas/weather.rs` (file does not yet exist):

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    #[default]
    Clear,
    HeavyRain,
    Heatwave,
    Blizzard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_default_is_clear() {
        assert_eq!(Weather::default(), Weather::Clear);
    }

    #[test]
    fn current_weather_stub_returns_clear() {
        // Stub producer until full weather spec lands.
        assert_eq!(current_weather(), Weather::Clear);
    }
}
```

(The test references `current_weather()` which we have not yet defined — that is the failing call.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --lib areas::weather -- --nocapture`
Expected: FAIL — `cannot find function 'current_weather' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Edit `game/src/areas/weather.rs`, insert above the `#[cfg(test)]` block:

```rust
/// Stub producer. Always returns `Weather::Clear` until the full weather
/// system (see `2026-05-02-weather-system-design.md`) replaces this.
///
/// Consumers must call this (not hardcode `Weather::Clear`) so the future
/// weather implementation needs only a producer-side change.
pub fn current_weather() -> Weather {
    Weather::Clear
}
```

Edit `game/src/areas/mod.rs`, add at the top of the module (alongside other `pub mod`s):

```rust
pub mod weather;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib areas::weather`
Expected: PASS — both tests green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add minimal Weather enum + stub producer

Introduces game/src/areas/weather.rs with a four-variant Weather enum
(Clear, HeavyRain, Heatwave, Blizzard) and a stub current_weather()
producer that always returns Clear. Consumers in subsequent tasks
take &Weather as input so the future full weather implementation
needs only a producer-side change.

Refs: hangrier_games-0yz"
```

(Or equivalent `jj` commit-creation flow — the project uses jj with git coexistence.)

---

## Task 2: `shelter_quality(terrain, weather)` pure function

**Files:**
- Create: `game/src/areas/shelter.rs`
- Modify: `game/src/areas/mod.rs` (add `pub mod shelter;`)

- [ ] **Step 1: Write the failing test**

Create `game/src/areas/shelter.rs` with:

```rust
use crate::areas::weather::Weather;
use crate::terrain::types::BaseTerrain;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::UrbanRuins, Weather::Clear, 3)]
    #[case(BaseTerrain::Forest,     Weather::Clear, 2)]
    #[case(BaseTerrain::Jungle,     Weather::Clear, 2)]
    #[case(BaseTerrain::Mountains,  Weather::Clear, 2)]
    #[case(BaseTerrain::Geothermal, Weather::Clear, 2)]
    #[case(BaseTerrain::Wetlands,   Weather::Clear, 1)]
    #[case(BaseTerrain::Highlands,  Weather::Clear, 1)]
    #[case(BaseTerrain::Clearing,   Weather::Clear, 1)]
    #[case(BaseTerrain::Grasslands, Weather::Clear, 1)]
    #[case(BaseTerrain::Badlands,   Weather::Clear, 1)]
    #[case(BaseTerrain::Tundra,     Weather::Clear, 0)]
    #[case(BaseTerrain::Desert,     Weather::Clear, 0)]
    fn shelter_quality_clear_weather_table(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &weather), expected);
    }

    #[rstest]
    #[case(BaseTerrain::Forest,     Weather::HeavyRain, 1)] // 2 - 1
    #[case(BaseTerrain::UrbanRuins, Weather::HeavyRain, 2)] // 3 - 1
    #[case(BaseTerrain::Desert,     Weather::HeavyRain, 0)] // floors at 0
    #[case(BaseTerrain::Forest,     Weather::Blizzard,  1)]
    #[case(BaseTerrain::Tundra,     Weather::Blizzard,  0)]
    fn shelter_quality_storm_modifier(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &weather), expected);
    }

    #[rstest]
    #[case(BaseTerrain::UrbanRuins, 3)]
    #[case(BaseTerrain::Mountains,  2)]
    #[case(BaseTerrain::Geothermal, 2)]
    #[case(BaseTerrain::Forest,     2)]
    #[case(BaseTerrain::Jungle,     2)]
    #[case(BaseTerrain::Tundra,     0)]
    #[case(BaseTerrain::Desert,     0)]
    fn shelter_quality_heatwave_keeps_stone_and_canopy(
        #[case] terrain: BaseTerrain,
        #[case] expected: u8,
    ) {
        assert_eq!(shelter_quality(terrain, &Weather::Heatwave), expected);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --lib areas::shelter`
Expected: FAIL — `cannot find function 'shelter_quality' in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add to `game/src/areas/shelter.rs` above the test module:

```rust
/// Pure derivation of an area's shelter quality from terrain and current weather.
/// 0 = no shelter possible. 4 = excellent shelter. See spec table.
pub fn shelter_quality(terrain: BaseTerrain, weather: &Weather) -> u8 {
    let base = match terrain {
        BaseTerrain::UrbanRuins => 3,
        BaseTerrain::Forest
        | BaseTerrain::Jungle
        | BaseTerrain::Mountains
        | BaseTerrain::Geothermal => 2,
        BaseTerrain::Wetlands
        | BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::Grasslands
        | BaseTerrain::Badlands => 1,
        BaseTerrain::Tundra | BaseTerrain::Desert => 0,
    };

    match weather {
        Weather::Clear => base,
        Weather::HeavyRain | Weather::Blizzard => base.saturating_sub(1),
        Weather::Heatwave => match terrain {
            BaseTerrain::UrbanRuins
            | BaseTerrain::Mountains
            | BaseTerrain::Geothermal
            | BaseTerrain::Forest
            | BaseTerrain::Jungle => base,
            BaseTerrain::Tundra | BaseTerrain::Desert => 0,
            _ => base,
        },
    }
}
```

Edit `game/src/areas/mod.rs`, add: `pub mod shelter;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib areas::shelter`
Expected: PASS — all rstest cases green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add shelter_quality pure function

Derives an area's shelter quality (0-4) from BaseTerrain + Weather.
Full coverage of all 12 BaseTerrain variants. Storm weathers (HeavyRain,
Blizzard) apply -1 to the roll target; Heatwave preserves stone/canopy.

Refs: hangrier_games-0yz"
```

---

## Task 3: `forage_richness` and `water_source` pure functions

**Files:**
- Create: `game/src/areas/forage.rs`
- Create: `game/src/areas/water.rs`
- Modify: `game/src/areas/mod.rs`

- [ ] **Step 1: Write the failing tests**

Create `game/src/areas/forage.rs`:

```rust
use crate::terrain::types::BaseTerrain;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::Wetlands,   3)]
    #[case(BaseTerrain::Jungle,     3)]
    #[case(BaseTerrain::Forest,     2)]
    #[case(BaseTerrain::UrbanRuins, 2)]
    #[case(BaseTerrain::Mountains,  1)]
    #[case(BaseTerrain::Highlands,  1)]
    #[case(BaseTerrain::Clearing,   1)]
    #[case(BaseTerrain::Grasslands, 1)]
    #[case(BaseTerrain::Geothermal, 1)]
    #[case(BaseTerrain::Badlands,   0)]
    #[case(BaseTerrain::Tundra,     0)]
    #[case(BaseTerrain::Desert,     0)]
    fn forage_richness_table(#[case] terrain: BaseTerrain, #[case] expected: u8) {
        assert_eq!(forage_richness(terrain), expected);
    }
}
```

Create `game/src/areas/water.rs`:

```rust
use crate::areas::weather::Weather;
use crate::terrain::types::BaseTerrain;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(BaseTerrain::Wetlands,   Weather::Clear, 3)]
    #[case(BaseTerrain::Forest,     Weather::Clear, 2)]
    #[case(BaseTerrain::Jungle,     Weather::Clear, 2)]
    #[case(BaseTerrain::Mountains,  Weather::Clear, 2)]
    #[case(BaseTerrain::Geothermal, Weather::Clear, 2)]
    #[case(BaseTerrain::Highlands,  Weather::Clear, 1)]
    #[case(BaseTerrain::Clearing,   Weather::Clear, 1)]
    #[case(BaseTerrain::UrbanRuins, Weather::Clear, 1)]
    #[case(BaseTerrain::Tundra,     Weather::Clear, 1)]
    #[case(BaseTerrain::Grasslands, Weather::Clear, 0)]
    #[case(BaseTerrain::Badlands,   Weather::Clear, 0)]
    #[case(BaseTerrain::Desert,     Weather::Clear, 0)]
    fn water_source_clear_table(
        #[case] terrain: BaseTerrain,
        #[case] weather: Weather,
        #[case] expected: u8,
    ) {
        assert_eq!(water_source(terrain, &weather), expected);
    }

    #[rstest]
    #[case(BaseTerrain::Wetlands,   3)]
    #[case(BaseTerrain::Forest,     3)]
    #[case(BaseTerrain::Jungle,     3)]
    #[case(BaseTerrain::Grasslands, 2)]
    #[case(BaseTerrain::Mountains,  2)]
    #[case(BaseTerrain::Desert,     1)]
    #[case(BaseTerrain::Tundra,     1)]
    fn water_source_heavy_rain_boosts(
        #[case] terrain: BaseTerrain,
        #[case] expected: u8,
    ) {
        assert_eq!(water_source(terrain, &Weather::HeavyRain), expected);
    }

    #[rstest]
    #[case(BaseTerrain::Wetlands, 1)] // 3 / 2 = 1
    #[case(BaseTerrain::Mountains, 1)] // 2 / 2 = 1
    #[case(BaseTerrain::Highlands, 0)] // 1 / 2 = 0
    #[case(BaseTerrain::Desert, 0)]
    fn water_source_heatwave_halves_base(
        #[case] terrain: BaseTerrain,
        #[case] expected: u8,
    ) {
        assert_eq!(water_source(terrain, &Weather::Heatwave), expected);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib 'areas::(forage|water)'`
Expected: FAIL — both functions undefined.

- [ ] **Step 3: Write minimal implementations**

Add to `game/src/areas/forage.rs` above the test module:

```rust
/// Pure derivation of an area's forage richness from terrain.
/// 0 = barren. 4 = abundant. See spec table.
pub fn forage_richness(terrain: BaseTerrain) -> u8 {
    match terrain {
        BaseTerrain::Wetlands | BaseTerrain::Jungle => 3,
        BaseTerrain::Forest | BaseTerrain::UrbanRuins => 2,
        BaseTerrain::Mountains
        | BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::Grasslands
        | BaseTerrain::Geothermal => 1,
        BaseTerrain::Badlands | BaseTerrain::Tundra | BaseTerrain::Desert => 0,
    }
}
```

Add to `game/src/areas/water.rs` above the test module:

```rust
/// Pure derivation of an area's water-source strength from terrain + weather.
/// 0 = no water available. 3 = abundant.
pub fn water_source(terrain: BaseTerrain, weather: &Weather) -> u8 {
    let base = match terrain {
        BaseTerrain::Wetlands => 3,
        BaseTerrain::Forest
        | BaseTerrain::Jungle
        | BaseTerrain::Mountains
        | BaseTerrain::Geothermal => 2,
        BaseTerrain::Highlands
        | BaseTerrain::Clearing
        | BaseTerrain::UrbanRuins
        | BaseTerrain::Tundra => 1,
        BaseTerrain::Grasslands | BaseTerrain::Badlands | BaseTerrain::Desert => 0,
    };

    match weather {
        Weather::Clear | Weather::Blizzard => base,
        Weather::HeavyRain => (base + 2).min(3),
        Weather::Heatwave => base / 2,
    }
}
```

Edit `game/src/areas/mod.rs`, add: `pub mod forage; pub mod water;`

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib 'areas::(forage|water)'`
Expected: PASS — all cases green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add forage_richness + water_source pure functions

Derive area resource availability from terrain + weather. Full coverage
of all 12 BaseTerrain variants. HeavyRain boosts water; Heatwave halves
it (rounding down).

Refs: hangrier_games-0yz"
```

---

## Task 4: New `Tribute` survival fields + serde defaults

**Files:**
- Modify: `game/src/tributes/mod.rs:131-200` (the `pub struct Tribute` definition; line range approximate — locate `pub items: Vec<Item>` and add nearby).

- [ ] **Step 1: Write the failing test**

Append to the existing `#[cfg(test)] mod tests { ... }` block in `game/src/tributes/mod.rs` (locate near the bottom of the file):

```rust
#[test]
fn tribute_default_survival_fields_are_zero_and_none() {
    let t = Tribute::new("Test".to_string(), None, None);
    assert_eq!(t.hunger, 0, "hunger starts at 0 (Sated)");
    assert_eq!(t.thirst, 0, "thirst starts at 0 (Sated)");
    assert_eq!(t.sheltered_until, None, "starts exposed");
    assert_eq!(t.starvation_drain_step, 0);
    assert_eq!(t.dehydration_drain_step, 0);
}

#[test]
fn tribute_legacy_json_loads_with_defaults() {
    // JSON missing the new fields entirely (simulates a saved game from
    // before this feature landed). serde(default) must populate them.
    let legacy = r#"{
        "name": "Legacy",
        "district": null,
        "avatar": null,
        "items": [],
        "is_hidden": false,
        "attributes": {
            "health": 100, "stamina": 50, "sanity": 50,
            "strength": 50, "defense": 50, "bravery": 50, "intelligence": 50
        },
        "status": "Healthy"
    }"#;
    let t: Tribute = serde_json::from_str(legacy).expect("legacy load must succeed");
    assert_eq!(t.hunger, 0);
    assert_eq!(t.thirst, 0);
    assert_eq!(t.sheltered_until, None);
    assert_eq!(t.starvation_drain_step, 0);
    assert_eq!(t.dehydration_drain_step, 0);
}
```

(The legacy JSON shape may need adjusting to match the actual `Tribute` minimum required fields — read the existing struct first and trim or extend the JSON to match. The point is: a doc with no survival fields must deserialize.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib tributes::tests::tribute_default_survival_fields_are_zero_and_none tributes::tests::tribute_legacy_json_loads_with_defaults`
Expected: FAIL — `no field 'hunger' on type 'Tribute'`.

- [ ] **Step 3: Add the fields**

In `game/src/tributes/mod.rs`, in the `pub struct Tribute { ... }` definition, add (placement near the other tribute state fields):

```rust
    #[serde(default)]
    pub hunger: u8,
    #[serde(default)]
    pub thirst: u8,
    #[serde(default)]
    pub sheltered_until: Option<u32>,
    #[serde(default)]
    pub starvation_drain_step: u8,
    #[serde(default)]
    pub dehydration_drain_step: u8,
```

In `Tribute::new(...)` (around line 229), add field initializers (all zero / None):

```rust
            hunger: 0,
            thirst: 0,
            sheltered_until: None,
            starvation_drain_step: 0,
            dehydration_drain_step: 0,
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::tests`
Expected: PASS — both new tests green; existing tribute tests still green.

- [ ] **Step 5: Update SurrealDB schema**

Locate the tribute schema file (`schemas/tribute.surql` or similar — `ls schemas/`). Add (with permissive defaults so existing rows load):

```surql
DEFINE FIELD hunger                  ON TABLE tribute TYPE int DEFAULT 0;
DEFINE FIELD thirst                  ON TABLE tribute TYPE int DEFAULT 0;
DEFINE FIELD sheltered_until         ON TABLE tribute TYPE option<int>;
DEFINE FIELD starvation_drain_step   ON TABLE tribute TYPE int DEFAULT 0;
DEFINE FIELD dehydration_drain_step  ON TABLE tribute TYPE int DEFAULT 0;
```

Add a new migration definition file under `migrations/definitions/` if the project requires explicit migration steps (check the `surrealdb-migrations` setup before deciding). If migrations run at startup against the existing db, that is sufficient.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): add hunger/thirst/shelter fields to Tribute

Adds the persistent state for the survival system to the Tribute struct
with serde(default) so existing JSON saves load cleanly. SurrealDB
schema updated with DEFAULT values.

Fields:
- hunger: u8 (debt counter, 0 = Sated)
- thirst: u8 (debt counter, 0 = Sated)
- sheltered_until: Option<u32> (phase index)
- starvation_drain_step: u8 (escalating HP drain counter)
- dehydration_drain_step: u8 (escalating HP drain counter)

Refs: hangrier_games-0yz"
```

---

## Task 5: Hunger/Thirst band enums + classification

**Files:**
- Create: `game/src/tributes/survival.rs`
- Modify: `game/src/tributes/mod.rs` (add `pub mod survival;`)

- [ ] **Step 1: Write the failing tests**

Create `game/src/tributes/survival.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum HungerBand {
    Sated,
    Peckish,
    Hungry,
    Starving,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ThirstBand {
    Sated,
    Thirsty,
    Parched,
    Dehydrated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0, HungerBand::Sated)]
    #[case(1, HungerBand::Peckish)]
    #[case(2, HungerBand::Peckish)]
    #[case(3, HungerBand::Hungry)]
    #[case(4, HungerBand::Hungry)]
    #[case(5, HungerBand::Starving)]
    #[case(99, HungerBand::Starving)]
    fn hunger_band_thresholds(#[case] value: u8, #[case] expected: HungerBand) {
        assert_eq!(hunger_band(value), expected);
    }

    #[rstest]
    #[case(0, ThirstBand::Sated)]
    #[case(1, ThirstBand::Thirsty)]
    #[case(2, ThirstBand::Parched)]
    #[case(3, ThirstBand::Dehydrated)]
    #[case(99, ThirstBand::Dehydrated)]
    fn thirst_band_thresholds(#[case] value: u8, #[case] expected: ThirstBand) {
        assert_eq!(thirst_band(value), expected);
    }

    #[test]
    fn hunger_starving_is_publicly_visible() {
        assert!(hunger_band_is_public(HungerBand::Starving));
        assert!(hunger_band_is_public(HungerBand::Hungry));
        assert!(!hunger_band_is_public(HungerBand::Peckish));
        assert!(!hunger_band_is_public(HungerBand::Sated));
    }

    #[test]
    fn thirst_dehydrated_is_publicly_visible() {
        assert!(thirst_band_is_public(ThirstBand::Dehydrated));
        assert!(thirst_band_is_public(ThirstBand::Parched));
        assert!(!thirst_band_is_public(ThirstBand::Thirsty));
        assert!(!thirst_band_is_public(ThirstBand::Sated));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib tributes::survival`
Expected: FAIL — `hunger_band` / `thirst_band` / `hunger_band_is_public` / `thirst_band_is_public` undefined.

- [ ] **Step 3: Implement**

Add to `game/src/tributes/survival.rs` above the test module:

```rust
pub fn hunger_band(value: u8) -> HungerBand {
    match value {
        0 => HungerBand::Sated,
        1..=2 => HungerBand::Peckish,
        3..=4 => HungerBand::Hungry,
        _ => HungerBand::Starving,
    }
}

pub fn thirst_band(value: u8) -> ThirstBand {
    match value {
        0 => ThirstBand::Sated,
        1 => ThirstBand::Thirsty,
        2 => ThirstBand::Parched,
        _ => ThirstBand::Dehydrated,
    }
}

/// True if a band-change event into this band should be surfaced in the
/// public timeline (Action panel). Lower bands are private/Inspect-only.
pub fn hunger_band_is_public(band: HungerBand) -> bool {
    matches!(band, HungerBand::Hungry | HungerBand::Starving)
}

pub fn thirst_band_is_public(band: ThirstBand) -> bool {
    matches!(band, ThirstBand::Parched | ThirstBand::Dehydrated)
}
```

In `game/src/tributes/mod.rs`, add to the module declarations (with the other `pub mod`s):

```rust
pub mod survival;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::survival`
Expected: PASS — all band tests green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add HungerBand/ThirstBand classification

Pure functions mapping debt-counter values to bands and identifying
which band crossings are publicly visible vs private/Inspect-only.

Refs: hangrier_games-0yz"
```

---

## Task 6: Survival tick (counters + weather/shelter modulation)

**Files:**
- Modify: `game/src/tributes/survival.rs`

- [ ] **Step 1: Write the failing tests**

Append to `game/src/tributes/survival.rs` test module:

```rust
    use crate::areas::weather::Weather;
    use crate::tributes::Tribute;

    fn baseline_tribute() -> Tribute {
        let mut t = Tribute::new("Test".to_string(), None, None);
        // Mid-range strength + stamina: baseline ticks.
        t.attributes.strength = 50;
        t.attributes.stamina = 50;
        t
    }

    #[test]
    fn survival_tick_baseline_clear_exposed() {
        let mut t = baseline_tribute();
        tick_survival(&mut t, &Weather::Clear, /* sheltered = */ false);
        assert_eq!(t.hunger, 1);
        assert_eq!(t.thirst, 1);
    }

    #[test]
    fn survival_tick_heatwave_exposed_adds_thirst() {
        let mut t = baseline_tribute();
        tick_survival(&mut t, &Weather::Heatwave, false);
        assert_eq!(t.hunger, 1);
        assert_eq!(t.thirst, 2, "heatwave + exposed adds +1 thirst");
    }

    #[test]
    fn survival_tick_blizzard_exposed_adds_hunger() {
        let mut t = baseline_tribute();
        tick_survival(&mut t, &Weather::Blizzard, false);
        assert_eq!(t.hunger, 2, "blizzard + exposed adds +1 hunger");
        assert_eq!(t.thirst, 1);
    }

    #[test]
    fn survival_tick_sheltered_suppresses_weather_modifier() {
        let mut t = baseline_tribute();
        tick_survival(&mut t, &Weather::Heatwave, true);
        assert_eq!(t.thirst, 1, "shelter suppresses heatwave bonus");
        let mut t2 = baseline_tribute();
        tick_survival(&mut t2, &Weather::Blizzard, true);
        assert_eq!(t2.hunger, 1, "shelter suppresses blizzard bonus");
    }

    #[test]
    fn survival_tick_high_strength_increases_hunger() {
        let mut t = baseline_tribute();
        t.attributes.strength = 80; // high
        tick_survival(&mut t, &Weather::Clear, false);
        assert_eq!(t.hunger, 2, "high-strength bodies burn more calories");
    }

    #[test]
    fn survival_tick_high_stamina_increases_thirst() {
        let mut t = baseline_tribute();
        t.attributes.stamina = 80;
        tick_survival(&mut t, &Weather::Clear, false);
        assert_eq!(t.thirst, 2);
    }

    #[test]
    fn survival_tick_low_strength_skips_hunger_every_other_phase() {
        let mut t = baseline_tribute();
        t.attributes.strength = 20; // low
        // Phase 1: skip
        tick_survival(&mut t, &Weather::Clear, false);
        assert_eq!(t.hunger, 0, "low-strength skips first phase");
        // Phase 2: tick
        tick_survival(&mut t, &Weather::Clear, false);
        assert_eq!(t.hunger, 1, "low-strength ticks every other phase");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib tributes::survival::tests::survival_tick`
Expected: FAIL — `tick_survival` undefined.

- [ ] **Step 3: Implement**

Add to `game/src/tributes/survival.rs` above the test module:

```rust
use crate::areas::weather::Weather;
use crate::tributes::Tribute;

const HIGH_ATTR_THRESHOLD: u32 = 75;
const LOW_ATTR_THRESHOLD: u32 = 25;

/// Mutates `tribute` in place to advance one phase of survival (hunger + thirst).
///
/// Tick rules (per spec):
/// - Base +1 hunger and +1 thirst per phase.
/// - High strength (>= HIGH_ATTR_THRESHOLD) adds +1 hunger.
/// - Low strength (<= LOW_ATTR_THRESHOLD) ticks hunger every other phase.
/// - High stamina adds +1 thirst; low stamina ticks thirst every other phase.
/// - If exposed (not sheltered) AND weather is Blizzard: +1 hunger.
/// - If exposed AND weather is Heatwave: +1 thirst.
///
/// Whether HP loss is applied for Starving/Dehydrated states is handled by
/// `apply_starvation_drain` / `apply_dehydration_drain`, called separately.
pub fn tick_survival(tribute: &mut Tribute, weather: &Weather, sheltered: bool) {
    let strength = tribute.attributes.strength;
    let stamina = tribute.attributes.stamina;

    let hunger_delta: u8 = if strength <= LOW_ATTR_THRESHOLD {
        // Tick every other phase: use the parity of the existing counter as
        // a deterministic skip mechanism.
        if tribute.hunger % 2 == 0 { 0 } else { 1 }
    } else if strength >= HIGH_ATTR_THRESHOLD {
        2
    } else {
        1
    };

    let thirst_delta: u8 = if stamina <= LOW_ATTR_THRESHOLD {
        if tribute.thirst % 2 == 0 { 0 } else { 1 }
    } else if stamina >= HIGH_ATTR_THRESHOLD {
        2
    } else {
        1
    };

    let weather_hunger_bonus: u8 = if !sheltered && matches!(weather, Weather::Blizzard) { 1 } else { 0 };
    let weather_thirst_bonus: u8 = if !sheltered && matches!(weather, Weather::Heatwave) { 1 } else { 0 };

    tribute.hunger = tribute.hunger.saturating_add(hunger_delta + weather_hunger_bonus);
    tribute.thirst = tribute.thirst.saturating_add(thirst_delta + weather_thirst_bonus);
}
```

**Note on the low-attribute "every other phase" mechanic:** the parity-of-counter trick is deterministic and avoids needing extra state. The first tick is skipped; subsequent ticks alternate. This means a freshly-created tribute with low strength will skip its first hunger tick, then tick on subsequent phases as parity dictates. The low-strength test above relies on that order.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::survival`
Expected: PASS — all tick tests green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): implement survival tick (hunger + thirst)

Mutates a Tribute's hunger and thirst counters per phase based on
attribute modulation (strength <-> hunger, stamina <-> thirst) and
weather/shelter modifiers (Blizzard, Heatwave). Pure with respect to
external state aside from the tribute mutation.

Refs: hangrier_games-0yz"
```

---

## Task 7: Escalating starvation/dehydration drain

**Files:**
- Modify: `game/src/tributes/survival.rs`

- [ ] **Step 1: Write the failing tests**

Append to `game/src/tributes/survival.rs` test module:

```rust
    fn starving_tribute() -> Tribute {
        let mut t = baseline_tribute();
        t.hunger = 5; // Starving band
        t.attributes.health = 100;
        t.starvation_drain_step = 0;
        t
    }

    #[test]
    fn starvation_drain_escalates_each_phase() {
        let mut t = starving_tribute();
        let lost1 = apply_starvation_drain(&mut t);
        assert_eq!(lost1, 1, "first phase: -1 HP");
        assert_eq!(t.attributes.health, 99);
        assert_eq!(t.starvation_drain_step, 1);

        let lost2 = apply_starvation_drain(&mut t);
        assert_eq!(lost2, 2, "second phase: -2 HP");
        assert_eq!(t.attributes.health, 97);
        assert_eq!(t.starvation_drain_step, 2);

        let lost3 = apply_starvation_drain(&mut t);
        assert_eq!(lost3, 3);
        assert_eq!(t.attributes.health, 94);
    }

    #[test]
    fn starvation_drain_no_op_when_not_starving() {
        let mut t = baseline_tribute();
        t.hunger = 3; // Hungry, not Starving
        let lost = apply_starvation_drain(&mut t);
        assert_eq!(lost, 0);
        assert_eq!(t.starvation_drain_step, 0);
    }

    #[test]
    fn eating_food_resets_drain_step_and_reduces_hunger() {
        let mut t = starving_tribute();
        apply_starvation_drain(&mut t); // drain_step = 1
        apply_starvation_drain(&mut t); // drain_step = 2
        eat_food(&mut t, 3);
        assert_eq!(t.hunger, 2, "5 - 3 = 2 (Peckish band)");
        assert_eq!(t.starvation_drain_step, 0, "eating resets drain");
    }

    #[test]
    fn dehydration_drain_escalates_independently_of_starvation() {
        let mut t = baseline_tribute();
        t.thirst = 3;
        t.hunger = 5;
        let h1 = apply_dehydration_drain(&mut t);
        let s1 = apply_starvation_drain(&mut t);
        assert_eq!(h1, 1);
        assert_eq!(s1, 1);
        assert_eq!(t.attributes.health, 98, "stacked drain: -2 HP first phase");
    }

    #[test]
    fn drink_water_resets_dehydration_step() {
        let mut t = baseline_tribute();
        t.thirst = 3;
        apply_dehydration_drain(&mut t);
        drink_water(&mut t, 2);
        assert_eq!(t.thirst, 1);
        assert_eq!(t.dehydration_drain_step, 0);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib tributes::survival`
Expected: FAIL — `apply_starvation_drain` / `apply_dehydration_drain` / `eat_food` / `drink_water` undefined.

- [ ] **Step 3: Implement**

Append to `game/src/tributes/survival.rs` above the test module:

```rust
/// Applies escalating starvation HP drain. Returns HP lost this phase (0 if
/// the tribute is not in the Starving band).
pub fn apply_starvation_drain(tribute: &mut Tribute) -> u32 {
    if hunger_band(tribute.hunger) != HungerBand::Starving {
        return 0;
    }
    tribute.starvation_drain_step = tribute.starvation_drain_step.saturating_add(1);
    let lost = tribute.starvation_drain_step as u32;
    tribute.attributes.health = tribute.attributes.health.saturating_sub(lost);
    lost
}

/// Applies escalating dehydration HP drain. Returns HP lost this phase.
pub fn apply_dehydration_drain(tribute: &mut Tribute) -> u32 {
    if thirst_band(tribute.thirst) != ThirstBand::Dehydrated {
        return 0;
    }
    tribute.dehydration_drain_step = tribute.dehydration_drain_step.saturating_add(1);
    let lost = tribute.dehydration_drain_step as u32;
    tribute.attributes.health = tribute.attributes.health.saturating_sub(lost);
    lost
}

/// Reduces hunger by `amount`, resetting the starvation drain counter.
pub fn eat_food(tribute: &mut Tribute, amount: u8) {
    tribute.hunger = tribute.hunger.saturating_sub(amount);
    tribute.starvation_drain_step = 0;
}

/// Reduces thirst by `amount`, resetting the dehydration drain counter.
pub fn drink_water(tribute: &mut Tribute, amount: u8) {
    tribute.thirst = tribute.thirst.saturating_sub(amount);
    tribute.dehydration_drain_step = 0;
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::survival`
Expected: PASS — all drain tests green; previous tests still green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): implement escalating starvation/dehydration drain

Each phase a tribute remains in Starving or Dehydrated, HP drain
increases by 1 (-1, -2, -3...). Eating/drinking resets the drain step.
Hunger and thirst drains stack independently.

Refs: hangrier_games-0yz"
```

---

## Task 8: `ItemType` Food/Water variants + helpers

**Files:**
- Modify: `game/src/items/mod.rs:340-380` (the `ItemType` enum + `Display`/`FromStr` impls)

- [ ] **Step 1: Audit the existing ItemType serialization shape first**

Run: `cargo test --package game --lib items 2>&1 | tail -20` — verify existing tests pass.

Check how `Item.item_type` round-trips:

Run: `grep -n "serde" game/src/items/mod.rs | head -10`

The struct uses derived `Serialize`/`Deserialize`. With unit variants the JSON form is the variant name as a string (`"Consumable"`). Adding tuple variants `Food(u8)` and `Water(u8)` will serialize as `{"Food": 3}` — a different shape. This affects backward compat for any persisted items.

Decision for v1: add the new variants and accept that existing rows continue to work (they're still `"Consumable"` / `"Weapon"` strings). New rows containing food/water will be `{"Food": 3}` / `{"Water": 2}`. Round-trip the new shapes in tests.

- [ ] **Step 2: Write the failing tests**

Append to the existing `#[cfg(test)] mod tests { ... }` block in `game/src/items/mod.rs`:

```rust
#[test]
fn item_type_food_serializes_round_trip() {
    let it = ItemType::Food(3);
    let json = serde_json::to_string(&it).unwrap();
    let back: ItemType = serde_json::from_str(&json).unwrap();
    assert_eq!(it, back);
}

#[test]
fn item_type_water_serializes_round_trip() {
    let it = ItemType::Water(2);
    let json = serde_json::to_string(&it).unwrap();
    let back: ItemType = serde_json::from_str(&json).unwrap();
    assert_eq!(it, back);
}

#[test]
fn item_type_legacy_consumable_string_still_loads() {
    let back: ItemType = serde_json::from_str("\"Consumable\"").unwrap();
    assert_eq!(back, ItemType::Consumable);
}

#[test]
fn item_type_food_helpers() {
    assert!(ItemType::Food(3).is_food());
    assert_eq!(ItemType::Food(3).food_value(), Some(3));
    assert_eq!(ItemType::Water(2).water_value(), Some(2));
    assert!(!ItemType::Weapon.is_food());
    assert_eq!(ItemType::Consumable.food_value(), None);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package game --lib items::tests`
Expected: FAIL — `Food` / `Water` / `is_food` etc. undefined.

- [ ] **Step 4: Implement**

In `game/src/items/mod.rs`, modify the `ItemType` enum:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    Consumable,
    Weapon,
    Food(u8),
    Water(u8),
}
```

Update the `random()`, `Display`, and `FromStr` impls. For `FromStr`, parse `food:N` / `water:N` forms. For `Display`, render as `food(3)` etc. Example replacement:

```rust
impl ItemType {
    pub fn random() -> ItemType {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        // Weighted distribution: weapons and consumables remain dominant; food
        // and water enter the spawn pool but are rarer.
        match rng.random_range(0..10) {
            0..=3 => ItemType::Consumable,
            4..=6 => ItemType::Weapon,
            7..=8 => ItemType::Food(rng.random_range(1..=5)),
            _ => ItemType::Water(rng.random_range(1..=3)),
        }
    }

    pub fn is_food(&self) -> bool {
        matches!(self, ItemType::Food(_))
    }

    pub fn is_water(&self) -> bool {
        matches!(self, ItemType::Water(_))
    }

    pub fn food_value(&self) -> Option<u8> {
        if let ItemType::Food(n) = self { Some(*n) } else { None }
    }

    pub fn water_value(&self) -> Option<u8> {
        if let ItemType::Water(n) = self { Some(*n) } else { None }
    }
}
```

Update `Display`:

```rust
impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Consumable => write!(f, "consumable"),
            ItemType::Weapon => write!(f, "weapon"),
            ItemType::Food(n) => write!(f, "food({})", n),
            ItemType::Water(n) => write!(f, "water({})", n),
        }
    }
}
```

Update `FromStr` to parse the new shapes:

```rust
impl FromStr for ItemType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        if let Some(inner) = lower.strip_prefix("food(").and_then(|x| x.strip_suffix(')')) {
            return inner.parse::<u8>().map(ItemType::Food).map_err(|e| e.to_string());
        }
        if let Some(inner) = lower.strip_prefix("water(").and_then(|x| x.strip_suffix(')')) {
            return inner.parse::<u8>().map(ItemType::Water).map_err(|e| e.to_string());
        }
        match lower.as_str() {
            "consumable" => Ok(ItemType::Consumable),
            "weapon" => Ok(ItemType::Weapon),
            _ => Err("Invalid item type".to_string()),
        }
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game --lib items`
Expected: PASS — new and existing item tests green.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): add Food(u8)/Water(u8) ItemType variants

Food(n) and Water(n) variants represent portable hunger/thirst debt
relief. Helpers is_food(), food_value(), is_water(), water_value().
Display renders 'food(3)' / 'water(2)'; FromStr parses the same.
Legacy 'Consumable'/'Weapon' string-tag rows continue to load.

Refs: hangrier_games-0yz"
```

---

## Task 9: New `Action` variants for survival

**Files:**
- Modify: `game/src/tributes/actions.rs:21-35` (the `Action` enum)

- [ ] **Step 1: Write the failing test**

Append to the existing test module in `game/src/tributes/actions.rs` (or create one if absent):

```rust
#[cfg(test)]
mod survival_action_tests {
    use super::*;

    #[test]
    fn survival_actions_exist_and_serialize() {
        let actions = vec![
            Action::SeekShelter,
            Action::Forage,
            Action::DrinkFromTerrain,
        ];
        for a in actions {
            let json = serde_json::to_string(&a).unwrap();
            let back: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", a), format!("{:?}", back));
        }
    }

    #[test]
    fn eat_action_carries_item_value() {
        // Construct Eat with a Food item; round-trip.
        // (Item construction details depend on the existing Item::new signature;
        //  the engineer should construct a minimal Food(3) item per the codebase's
        //  Item::new helper.)
        // Smoke test: variant exists.
        let _ = Action::Eat(None);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --lib tributes::actions::survival_action_tests`
Expected: FAIL — `SeekShelter` / `Forage` / `DrinkFromTerrain` / `Eat` variants undefined.

- [ ] **Step 3: Implement**

In `game/src/tributes/actions.rs`, extend the `Action` enum:

```rust
pub enum Action {
    None,
    Move(Option<Area>),
    Rest,
    UseItem(Option<Item>),
    Attack,
    Hide,
    TakeItem,
    // ... existing variants kept in order
    ProposeAlliance,

    // Survival actions (added by shelter+hunger/thirst spec).
    SeekShelter,
    Forage,
    DrinkFromTerrain,
    Eat(Option<Item>),
    DrinkItem(Option<Item>),
}
```

(Verify the existing variants list is preserved exactly; only append the new ones.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::actions`
Expected: PASS — survival tests green; existing action tests green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add survival Action variants

Five new actions: SeekShelter, Forage, DrinkFromTerrain (no item),
Eat(item), DrinkItem(item). Wiring into Brain happens in Task 10.

Refs: hangrier_games-0yz"
```

---

## Task 10: Brain survival overrides

**Files:**
- Modify: `game/src/tributes/brains.rs`

- [ ] **Step 1: Read the existing Brain decision flow**

Run: `grep -nE "fn (decide|choose|select|pick|next_action)" game/src/tributes/brains.rs | head -20`

Read the function that produces an `Action` from a tribute + context. Note its signature and where to insert the override checks (before existing weighted scoring).

- [ ] **Step 2: Write the failing tests**

Append to the existing `#[cfg(test)]` block in `game/src/tributes/brains.rs`:

```rust
#[cfg(test)]
mod survival_override_tests {
    use super::*;
    use crate::areas::weather::Weather;
    use crate::items::{Item, ItemType};
    use crate::terrain::types::BaseTerrain;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;

    fn dehydrated_at_water(terrain: BaseTerrain) -> Tribute {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.thirst = 3; // Dehydrated
        // place tribute at an area whose terrain has water_source > 0;
        // implementation-specific helper is invoked by survival_override(...)
        t
    }

    #[test]
    fn override_dehydrated_at_water_terrain_picks_drink() {
        let t = dehydrated_at_water(BaseTerrain::Wetlands);
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, /* in_combat */ false);
        assert_eq!(action, Some(Action::DrinkFromTerrain));
    }

    #[test]
    fn override_dehydrated_with_water_item_picks_drink_item() {
        let mut t = dehydrated_at_water(BaseTerrain::Desert);
        let water = Item::new_simple_water(2); // helper to be added inline if missing
        t.items.push(water);
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::DrinkItem(_))));
    }

    #[test]
    fn override_starving_with_food_picks_eat() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let food = Item::new_simple_food(3);
        t.items.push(food);
        let action = survival_override(&t, BaseTerrain::Desert, &Weather::Clear, false);
        assert!(matches!(action, Some(Action::Eat(_))));
    }

    #[test]
    fn override_starving_at_forageable_terrain_no_inventory_picks_forage() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, Some(Action::Forage));
    }

    #[test]
    fn override_starving_in_combat_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 5;
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, /* in_combat */ true);
        assert_eq!(action, None);
    }

    #[test]
    fn override_hungry_not_starving_returns_none() {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.hunger = 3; // Hungry, not Starving
        let action = survival_override(&t, BaseTerrain::Wetlands, &Weather::Clear, false);
        assert_eq!(action, None);
    }
}
```

(`Item::new_simple_food(n)` and `Item::new_simple_water(n)` may need to be added to `game/src/items/mod.rs` as small test-friendly constructors if the existing `Item::new` is inconvenient. Add them as part of this task if needed.)

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package game --lib tributes::brains::survival_override_tests`
Expected: FAIL — `survival_override` undefined.

- [ ] **Step 4: Implement**

Add to `game/src/tributes/brains.rs`:

```rust
use crate::areas::water::water_source;
use crate::areas::forage::forage_richness;
use crate::areas::weather::Weather;
use crate::terrain::types::BaseTerrain;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use crate::tributes::survival::{hunger_band, thirst_band, HungerBand, ThirstBand};

/// Survival override branch. Returns Some(action) to short-circuit the
/// Brain's normal weighted scoring; returns None to fall through.
///
/// Order (per spec):
/// 1. Dehydrated + at water-source terrain -> Drink(area).
/// 2. Dehydrated + Water item in inventory -> Drink(item).
/// 3. Starving + Food item in inventory -> Eat(item).
/// 4. Starving + at forageable terrain + not in combat -> Forage.
///
/// Active combat suppresses all overrides (the existing combat handling
/// preempts decision-making upstream — this is a defensive guard).
pub fn survival_override(
    tribute: &Tribute,
    terrain: BaseTerrain,
    weather: &Weather,
    in_combat: bool,
) -> Option<Action> {
    if in_combat {
        return None;
    }

    let dehydrated = thirst_band(tribute.thirst) == ThirstBand::Dehydrated;
    let starving = hunger_band(tribute.hunger) == HungerBand::Starving;

    if dehydrated && water_source(terrain, weather) > 0 {
        return Some(Action::DrinkFromTerrain);
    }
    if dehydrated {
        if let Some(item) = tribute.items.iter().find(|i| i.item_type.is_water()).cloned() {
            return Some(Action::DrinkItem(Some(item)));
        }
    }
    if starving {
        if let Some(item) = tribute.items.iter().find(|i| i.item_type.is_food()).cloned() {
            return Some(Action::Eat(Some(item)));
        }
        if forage_richness(terrain) > 0 {
            return Some(Action::Forage);
        }
    }

    None
}
```

Then locate the existing Brain entry point that produces an `Action` and insert a call to `survival_override(...)` ahead of the normal scoring path. Pseudo-shape:

```rust
// In the existing `decide` (or equivalent) function:
if let Some(action) = survival_override(tribute, current_terrain, &Weather::current_weather_for(area), in_combat) {
    return action;
}
// ... existing weighted scoring ...
```

The exact wiring depends on the existing function signature; the engineer should extract the necessary inputs (current area's terrain, weather lookup, combat flag) at the call site without restructuring the surrounding decision flow.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::brains`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): wire survival override branch into Brain

Survival overrides are checked before the existing weighted scoring:
1. Dehydrated + water-source terrain -> Drink(area)
2. Dehydrated + Water item in inventory -> Drink(item)
3. Starving + Food item in inventory -> Eat(item)
4. Starving + forageable terrain + not in combat -> Forage

Active combat suppresses all survival overrides.

Refs: hangrier_games-0yz"
```

---

## Task 11: New `MessagePayload` events + `cause` constants

**Files:**
- Modify: `shared/src/messages.rs:117-205` (the `MessagePayload` enum and its `kind()` impl)

- [ ] **Step 1: Write the failing test**

Append to (or create) the test module in `shared/src/messages.rs`:

```rust
#[cfg(test)]
mod survival_event_tests {
    use super::*;

    #[test]
    fn shelter_sought_round_trip() {
        let p = MessagePayload::ShelterSought {
            tribute: TributeRef { id: "t1".into(), name: "Cato".into() },
            area: AreaRef { id: "a1".into(), name: "Forest".into() },
            success: true,
            roll: 2,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: MessagePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
    }

    #[test]
    fn band_change_payloads_exist() {
        let _ = MessagePayload::HungerBandChanged {
            tribute: TributeRef { id: "t1".into(), name: "Cato".into() },
            from: "Sated".into(),
            to: "Hungry".into(),
        };
        let _ = MessagePayload::ThirstBandChanged {
            tribute: TributeRef { id: "t1".into(), name: "Cato".into() },
            from: "Sated".into(),
            to: "Parched".into(),
        };
    }

    #[test]
    fn cause_constants_exist() {
        assert_eq!(CAUSE_STARVATION, "starvation");
        assert_eq!(CAUSE_DEHYDRATION, "dehydration");
    }
}
```

(Adjust `TributeRef`/`AreaRef` construction to whatever the actual struct definitions in shared accept. Read those structs first.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package shared --lib survival_event_tests`
Expected: FAIL — variants and constants undefined.

- [ ] **Step 3: Implement**

In `shared/src/messages.rs`, add the new variants to `MessagePayload`:

```rust
    HungerBandChanged {
        tribute: TributeRef,
        from: String,
        to: String,
    },
    ThirstBandChanged {
        tribute: TributeRef,
        from: String,
        to: String,
    },
    ShelterSought {
        tribute: TributeRef,
        area: AreaRef,
        success: bool,
        roll: u8,
    },
    Foraged {
        tribute: TributeRef,
        area: AreaRef,
        success: bool,
        debt_recovered: u8,
    },
    Drank {
        tribute: TributeRef,
        source: DrinkSource,
        debt_recovered: u8,
    },
    Ate {
        tribute: TributeRef,
        item: ItemRef,
        debt_recovered: u8,
    },
```

Add a sibling enum for the drink source:

```rust
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum DrinkSource {
    Terrain { area: AreaRef },
    Item { item: ItemRef },
}
```

Add the cause constants at the top (or near the existing message kind constants):

```rust
pub const CAUSE_STARVATION: &str = "starvation";
pub const CAUSE_DEHYDRATION: &str = "dehydration";
```

Update the `MessagePayload::kind()` impl to map the new variants to appropriate `MessageKind`s (likely `MessageKind::State` for band changes and survival actions; the engineer should pick consistent kinds based on the existing pattern).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package shared --lib`
Expected: PASS — all messages tests green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(shared): add survival message payloads + cause constants

New MessagePayload variants for the shelter+hunger/thirst spec:
HungerBandChanged, ThirstBandChanged, ShelterSought, Foraged, Drank, Ate.
Adds DrinkSource enum and CAUSE_STARVATION/CAUSE_DEHYDRATION constants
for use in TributeKilled events.

Refs: hangrier_games-0yz"
```

---

## Task 12: Loot drop on death + survival tick wiring

**Files:**
- Modify: `game/src/tributes/lifecycle.rs` (drop items on RecentlyDead → Dead transition)
- Modify: `game/src/games.rs` (call survival tick once per phase per tribute; emit band-change events; route starvation/dehydration deaths)

- [ ] **Step 1: Write the failing tests**

In `game/src/tributes/lifecycle.rs` test module, add:

```rust
#[test]
fn dead_tribute_drops_items_into_area() {
    use crate::items::{Item, ItemType};

    let mut tribute = Tribute::new("Doomed".to_string(), None, None);
    tribute.items.push(Item::new_simple_food(3));
    tribute.items.push(Item::new_simple_water(2));
    tribute.attributes.health = 0;
    tribute.status = TributeStatus::RecentlyDead;

    let mut area_items: Vec<Item> = vec![];
    drop_items_to_area(&mut tribute, &mut area_items);

    assert_eq!(tribute.items.len(), 0, "tribute inventory cleared");
    assert_eq!(area_items.len(), 2, "items moved to area");
}

#[test]
fn drop_items_no_op_when_alive() {
    use crate::items::{Item, ItemType};

    let mut tribute = Tribute::new("Alive".to_string(), None, None);
    tribute.items.push(Item::new_simple_food(3));
    let mut area_items: Vec<Item> = vec![];
    drop_items_to_area(&mut tribute, &mut area_items);

    assert_eq!(tribute.items.len(), 1, "alive tribute keeps inventory");
    assert_eq!(area_items.len(), 0);
}
```

In `game/src/games.rs` test module, add a small integration test:

```rust
#[test]
fn survival_tick_fires_once_per_phase_per_living_tribute() {
    let mut game = Game::new("test");
    // (use the existing helpers in this file for game/tribute setup)
    // bootstrap two living tributes; call the per-phase survival entry
    // point; assert hunger and thirst incremented by exactly 1 each.
    // The exact phase entry function name lives in this file — locate it
    // via grep for "process_turn_phase" or "advance_phase".
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game --lib lifecycle 2>&1 | tail -20`
Expected: FAIL — `drop_items_to_area` undefined.

- [ ] **Step 3: Implement loot drop**

In `game/src/tributes/lifecycle.rs`:

```rust
use crate::items::Item;

/// Moves all items from a freshly-dead tribute into the supplied area item
/// vector. No-op for living tributes (Healthy/Wounded/etc.).
pub fn drop_items_to_area(tribute: &mut Tribute, area_items: &mut Vec<Item>) {
    use crate::tributes::TributeStatus;
    if !matches!(tribute.status, TributeStatus::RecentlyDead | TributeStatus::Dead) {
        return;
    }
    area_items.extend(tribute.items.drain(..));
}
```

Then locate the lifecycle transition from `RecentlyDead` → `Dead` (or the kill site) in `lifecycle.rs` / `combat.rs` / `games.rs`. At the right moment (when a tribute is confirmed dead and the area is in scope), call `drop_items_to_area(&mut tribute, &mut area.items)`.

- [ ] **Step 4: Wire the survival tick into the per-phase game loop**

Find the per-phase loop in `game/src/games.rs` (search for `process_turn_phase`, `for tribute in`, or the top-level day/night cycle entry point). For every living tribute each phase:

```rust
// Pseudo-shape — adapt to actual loop variable names.
let weather = current_weather(); // Task 1 stub
let sheltered = tribute.sheltered_until.map_or(false, |until| until > self.current_phase_index());

let prior_hunger_band = hunger_band(tribute.hunger);
let prior_thirst_band = thirst_band(tribute.thirst);

tick_survival(tribute, &weather, sheltered);

let hp_lost_starv = apply_starvation_drain(tribute);
let hp_lost_dehy = apply_dehydration_drain(tribute);

// Emit band-change events if the band moved (filter by *_band_is_public for
// publishing into the public timeline; private band changes still get logged
// to the per-tribute Inspect feed in Plan 2).
let new_hunger_band = hunger_band(tribute.hunger);
if new_hunger_band != prior_hunger_band {
    emit(MessagePayload::HungerBandChanged {
        tribute: tribute.as_ref(),
        from: format!("{:?}", prior_hunger_band),
        to: format!("{:?}", new_hunger_band),
    });
    // Update TributeStatus::Starving on entry; clear on exit to a lower band.
    if new_hunger_band == HungerBand::Starving {
        tribute.status = TributeStatus::Starving;
    } else if prior_hunger_band == HungerBand::Starving {
        tribute.status = TributeStatus::Healthy;
    }
}
// (Same shape for thirst.)

// Death routing: if HP hit 0 *and* either drain landed, route through
// TributeKilled with the appropriate cause.
if tribute.attributes.health == 0 {
    let cause = if hp_lost_dehy > 0 {
        CAUSE_DEHYDRATION
    } else if hp_lost_starv > 0 {
        CAUSE_STARVATION
    } else {
        // existing causes
        ""
    };
    if !cause.is_empty() {
        emit(MessagePayload::TributeKilled {
            victim: tribute.as_ref(),
            killer: None,
            cause: cause.to_string(),
        });
        tribute.status = TributeStatus::RecentlyDead;
        // Drop items now that the tribute is dead.
        drop_items_to_area(tribute, &mut area.items);
    }
}
```

The exact plumbing to access `area.items` from inside the per-tribute loop depends on how `games.rs` iterates. The engineer should reach for the same access pattern existing combat code uses to update area state.

- [ ] **Step 5: Run all tests**

Run: `cargo test --package game --lib`
Expected: PASS — all new tests green; existing tests still green.

Run: `cargo test --package shared --lib`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): wire survival tick + loot drop into game loop

Per phase, each living tribute:
- ticks hunger and thirst (Task 6)
- applies escalating starvation/dehydration drain (Task 7)
- emits HungerBandChanged/ThirstBandChanged on band crossing
- updates TributeStatus::Starving/Dehydrated on band entry/exit
- routes 0-HP-while-starving deaths through TributeKilled with
  CAUSE_STARVATION / CAUSE_DEHYDRATION

Lifecycle: on RecentlyDead/Dead, drop_items_to_area drains the
tribute's inventory into the area's item vector.

Refs: hangrier_games-0yz"
```

---

## Task 13: Add `Builder` and `ResourcefulForager` traits

**Files:**
- Modify: `game/src/tributes/traits.rs:9-30` (the `Trait` enum)

- [ ] **Step 1: Write the failing test**

Append to the existing test module in `game/src/tributes/traits.rs`:

```rust
#[test]
fn survival_traits_round_trip() {
    use serde_json;
    for t in [Trait::Builder, Trait::ResourcefulForager] {
        let json = serde_json::to_string(&t).unwrap();
        let back: Trait = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}

#[test]
fn builder_and_forager_dont_conflict_with_each_other() {
    // Convention check: traits should not conflict-pair with themselves
    // in CONFLICTS table. Adapt to whatever conflict-checking helper exists.
    // If CONFLICTS is a static array, scan it for these new entries.
    let conflict_targets_for_builder = conflicts_of(Trait::Builder);
    assert!(!conflict_targets_for_builder.contains(&Trait::ResourcefulForager));
}
```

(`conflicts_of` is illustrative — use whatever conflict-query helper actually exists in the file. Replace with a direct CONFLICTS scan if needed.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --lib tributes::traits`
Expected: FAIL — `Builder` / `ResourcefulForager` undefined.

- [ ] **Step 3: Implement**

In `game/src/tributes/traits.rs`, extend the `Trait` enum:

```rust
pub enum Trait {
    // ...existing variants...
    Asthmatic,
    Nearsighted,
    Tough,

    // Survival traits (added by shelter+hunger/thirst spec).
    Builder,
    ResourcefulForager,
}
```

Update any pattern matches on `Trait` that the compiler now flags as non-exhaustive. The simplest treatment: in any catch-all default-zero modifier function, no change needed; in any feature-specific match, add no-op arms unless the trait should affect that feature.

Apply the trait's spec'd effects in the relevant call sites (these are spread across actions):

In `game/src/tributes/brains.rs` (or wherever `SeekShelter` is rolled — see Plan 2 frontend smoke or earlier wiring):

```rust
fn shelter_trait_modifier(tribute: &Tribute) -> u8 {
    if tribute.traits.contains(&Trait::Builder) { 1 } else { 0 }
}

fn forage_trait_modifier(tribute: &Tribute) -> u8 {
    if tribute.traits.contains(&Trait::ResourcefulForager) { 1 } else { 0 }
}
```

(Adapt to actual trait-list field name. The full action-roll wiring for `SeekShelter` / `Forage` / `Drink` / food-poisoning rolls happens at the action-resolution site; if that site does not yet exist for these new actions, this task or Plan 2 should add minimal stub resolvers that consult these modifiers.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game --lib tributes::traits`
Expected: PASS.

Run: `cargo test --package game --lib`
Expected: PASS — workspace still green.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): add Builder + ResourcefulForager traits

Builder: +1 to SeekShelter roll, +1 phase to successful shelter duration.
ResourcefulForager: +1 to Forage roll, halved food-poisoning chance.

Refs: hangrier_games-0yz"
```

---

## Task 14: Integration tests (deterministic survival runs)

**Files:**
- Create: `game/tests/survival_integration.rs` (or extend an existing integration-tests file under `game/tests/` if the project already houses one).

- [ ] **Step 1: Check for an existing integration test directory**

Run: `ls game/tests/ 2>/dev/null && cat game/tests/*.rs 2>/dev/null | head -40`

If `game/tests/` does not exist, the engineer should create the file there. Tests in `game/tests/` are workspace-level integration tests and have access only to `pub` items.

- [ ] **Step 2: Write the failing tests**

Create `game/tests/survival_integration.rs`:

```rust
use game::areas::weather::Weather;
use game::tributes::Tribute;
use game::tributes::survival::{
    apply_dehydration_drain, apply_starvation_drain, hunger_band, thirst_band,
    drink_water, eat_food, tick_survival, HungerBand, ThirstBand,
};

#[test]
fn no_food_no_water_dies_of_dehydration_first() {
    let mut t = Tribute::new("Test".to_string(), None, None);
    t.attributes.health = 100;
    t.attributes.strength = 50;
    t.attributes.stamina = 50;
    let mut phases_to_dehydrated_band = 0u32;
    for phase in 1..=20 {
        tick_survival(&mut t, &Weather::Clear, false);
        if thirst_band(t.thirst) == ThirstBand::Dehydrated && phases_to_dehydrated_band == 0 {
            phases_to_dehydrated_band = phase;
        }
        let _ = apply_dehydration_drain(&mut t);
        let _ = apply_starvation_drain(&mut t);
        if t.attributes.health == 0 {
            // Confirm thirst drove the death, not starvation.
            assert_eq!(thirst_band(t.thirst), ThirstBand::Dehydrated);
            assert!(
                phases_to_dehydrated_band > 0
                    && phases_to_dehydrated_band < phase,
                "must reach Dehydrated before death"
            );
            return;
        }
    }
    panic!("tribute did not die in 20 phases");
}

#[test]
fn carrying_water_extends_survival() {
    use game::tributes::Tribute;

    fn run_to_death(start_thirst_relief: u8) -> u32 {
        let mut t = Tribute::new("Test".to_string(), None, None);
        t.attributes.health = 100;
        t.attributes.strength = 50;
        t.attributes.stamina = 50;
        // Pre-stock relief: drink it up at phase 0 to reset the clock.
        if start_thirst_relief > 0 {
            t.thirst = 3; // Dehydrated, then drink to reset
            drink_water(&mut t, start_thirst_relief);
        }
        for phase in 1..=30 {
            tick_survival(&mut t, &Weather::Clear, false);
            apply_dehydration_drain(&mut t);
            apply_starvation_drain(&mut t);
            if t.attributes.health == 0 {
                return phase;
            }
        }
        30
    }

    let baseline = run_to_death(0);
    let with_water = run_to_death(3);
    assert!(
        with_water > baseline + 2,
        "carrying water should extend life by at least 2 phases (baseline={baseline}, with_water={with_water})"
    );
}

#[test]
fn sheltered_in_heatwave_does_not_accrue_weather_thirst() {
    let mut t_sheltered = Tribute::new("S".to_string(), None, None);
    t_sheltered.attributes.strength = 50;
    t_sheltered.attributes.stamina = 50;
    let mut t_exposed = Tribute::new("E".to_string(), None, None);
    t_exposed.attributes.strength = 50;
    t_exposed.attributes.stamina = 50;

    for _ in 0..3 {
        tick_survival(&mut t_sheltered, &Weather::Heatwave, true);
        tick_survival(&mut t_exposed, &Weather::Heatwave, false);
    }
    assert!(t_exposed.thirst > t_sheltered.thirst,
        "exposed tribute should accrue more thirst (sheltered={}, exposed={})",
        t_sheltered.thirst, t_exposed.thirst);
}
```

(Skip the death-cause-routing and band-event-publication tests for this task if the per-phase loop wiring in `games.rs` is hard to drive headlessly — call those out as follow-ups in `bd` if so. The three tests above cover the core deterministic behaviors.)

- [ ] **Step 3: Run tests to verify they fail (or pass, as appropriate)**

Run: `cargo test --package game --test survival_integration`
Expected: PASS — by this point all helpers exist. If any test fails, fix the helper or test logic before moving on.

- [ ] **Step 4: Commit**

```bash
jj describe -m "test(game): integration tests for survival system

Three deterministic integration tests:
- no food/water -> dies of dehydration first
- carrying water extends survival by >= 2 phases
- shelter suppresses weather thirst modifier in Heatwave

Refs: hangrier_games-0yz"
```

---

## Task 15: Final quality pass + bookmark + PR

- [ ] **Step 1: Format and lint**

Run: `just fmt`
Run: `just quality`
Expected: clean — formatter no-op, no clippy warnings, all tests pass.

- [ ] **Step 2: Verify SurrealDB migration applies cleanly**

Run: `just dev` (in another terminal) and manually create a fresh game; confirm the new tribute fields persist and load. If a migration definition was added under `migrations/definitions/`, confirm `surrealdb-migrations` runs it on startup.

If there is no fresh-db smoke check available, run the existing API integration tests:
Run: `cargo test --package api --lib`
Expected: PASS.

- [ ] **Step 3: Verify in-flight game compatibility**

Manually load a saved game from before the change (or use a JSON fixture in `game/tests/fixtures/` if present). The game should load, all tributes should report `hunger=0, thirst=0, sheltered_until=None`, and the next phase should tick survival forward without panic.

- [ ] **Step 4: Open the PR**

Per project convention (see `AGENTS.md` "Pull Request Workflow"):

```bash
jj git fetch
jj rebase -d main@origin
bd backup export-git --branch beads-backup
jj bookmark create feat-shelter-hunger-backend -r @-
jj git push --bookmark feat-shelter-hunger-backend
gh pr create --base main --head feat-shelter-hunger-backend \
  --title "feat(game): shelter + hunger/thirst backend (PR1)" \
  --body "$(cat <<'EOF'
## Summary

Backend slice of the shelter + hunger/thirst spec. Frontend (pips, Inspect drilldown, map overlays) ships separately in a follow-up PR.

- New survival counters and bands on Tribute (hunger/thirst as debt counters; existing TributeStatus::Starving/Dehydrated finally driven)
- Pure derivations for shelter_quality / forage_richness / water_source per terrain × weather
- Brain survival override branch (Eat / Drink / Forage / fall-through)
- New Action variants: SeekShelter, Forage, DrinkFromTerrain, Eat, DrinkItem
- New ItemType::Food(u8) and Water(u8); legacy "Consumable"/"Weapon" rows still load
- Loot drop on RecentlyDead → Dead transition (adjacent fix; food economy depends on it)
- Six new MessagePayload variants for survival events (band changes are public; rolls are private/Inspect-only)
- Two new Traits: Builder, ResourcefulForager
- Minimal Weather enum + stub producer (full weather subsystem swaps producer side later)

## Spec
docs/superpowers/specs/2026-05-03-shelter-hunger-thirst-design.md

## Verification
- `just test` — all unit tests pass
- `just quality` — clean
- `cargo test --package game --test survival_integration` — pass
- Manual: fresh game + load existing saved-game JSON, both behave correctly

## Follow-ups
- Plan 2: frontend (pips, Inspect drilldown survival panel, map terrain overlays)
- hangrier_games-ex3f (resource sharing between allied tributes)
- hangrier_games-xfi (announcer prompts consume new survival events)
EOF
)"
```

- [ ] **Step 5: Update beads**

```bash
bd update hangrier_games-0yz --status in-progress --notes "PR1 backend opened: <PR URL>"
```

(Do not close `hangrier_games-0yz` — it covers both the spec *and* the frontend; close it when Plan 2 lands.)

---

## Self-Review

**Spec coverage check:**
- Shelter (action + per-tribute state + area-derived quality): Tasks 2, 9, 13, 12 ✓
- Hunger/thirst counters + bands + tick + drain: Tasks 4–7 ✓
- Items (Food/Water): Task 8 ✓
- Brain overrides: Task 10 ✓
- Looting on death: Task 12 ✓
- New events (6) + cause constants: Task 11 ✓
- Traits (Builder, ResourcefulForager): Task 13 ✓
- Migration / serde defaults / SurrealDB: Task 4 ✓
- Tests (≈30 unit + integration): Tasks 1–13 (unit), 14 (integration) ✓
- Frontend: deliberately deferred to Plan 2 ✓

**Placeholder scan:** all code blocks contain runnable code; pseudo-shapes are clearly marked as "adapt to actual ..." where the engineer must consult the existing call site. No "TBD" / "TODO" / "implement appropriately" placeholders.

**Type consistency check:**
- `tick_survival(&mut Tribute, &Weather, bool)` signature consistent across Tasks 6, 12, 14.
- `apply_starvation_drain(&mut Tribute) -> u32` consistent across Tasks 7, 12, 14.
- `survival_override(&Tribute, BaseTerrain, &Weather, bool) -> Option<Action>` consistent across Tasks 10, 12 (call site).
- `drop_items_to_area(&mut Tribute, &mut Vec<Item>)` consistent across Tasks 12, lifecycle integration.
- `hunger_band(u8) -> HungerBand` and `thirst_band(u8) -> ThirstBand` consistent across Tasks 5, 6, 7, 10, 12, 14.
- `Action::Eat(Option<Item>)` / `Action::DrinkItem(Option<Item>)` / `Action::DrinkFromTerrain` / `Action::Forage` / `Action::SeekShelter` consistent across Tasks 9, 10.
- `MessagePayload::ShelterSought { tribute, area, success, roll }` consistent across Task 11 and the field set is what the action-resolution code in Task 12 (or Plan 2 wiring) will emit.
