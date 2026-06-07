# Sleep Incident System — PR1: Backend Implementation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the entire backend slice of the phase-aware, biome-aware, shelter-aware sleep incident system: rename `Hallucination` → `Nightmare`, add `NightTerror`, add `SleepShelter` enum with `find_shelter()` roll, phase-base + day-scaling + biome/shelter-multiplier effective-chance calculation, biome-specific animal/flavor pools, generalised ally abandonment (handled in cycle, not `apply_sleep_incident`), and `InterruptionKind::NightTerror`. Frontend (incident display, shelter indicator) ships separately in PR2.

**Architecture:** Pure-function additions to `game/src/tributes/incidents.rs`; one transient field on `Tribute` (`sleep_shelter: Option<SleepShelter>`); integration wiring in `game/src/games/cycle.rs`; one new variant in `shared/src/messages/mod.rs` (`InterruptionKind::NightTerror`). No new files — all changes are modifications to existing files.

**Tech Stack:** Rust 2024, `serde` with `#[serde(skip)]` for transient field, `rstest` for parametric tests, `rand::SmallRng` for determinism.

**Spec:** `docs/superpowers/specs/2026-06-07-sleep-incident-system-design.md`

**Beads issue:** `hangrier_games-pending` (to be created before starting)

---

## Pre-flight notes

- All code lives in the `game/` and `shared/` crates. No `api/` or `web/` changes — those land in PR2.
- Run `just test` (game crate) and `just quality` (full workspace) at major checkpoints.
- The `shelter_quality()` function already exists at `game/src/areas/shelter.rs:6`. It takes `BaseTerrain` + `&Weather`. The incident system uses it via a pure `biome_incident_multiplier()` wrapper.
- The shelter system (`sheltered_until` on `Tribute`) may or may not have landed. If not, default `is_sheltered` to `false` — incident rates will be slightly higher but correct.
- `phase`, `current_day`, `area_details_map`, and `all_areas_snapshot` are all destructured from `CycleContext` in `execute_cycle` and available at the sleep incident roll call site.
- `AllyAbandonment` is removed from `apply_sleep_incident` — the cycle handles it directly with full game-state access.
- `sleep_shelter: Option<SleepShelter>` is a transient field with `#[serde(skip)]` — never persisted.
- Commits use the project's jj workflow per `AGENTS.md`. Each task ends with `jj describe -m "..."` then `jj new`.

---

## File Structure

**Modified:**
- `game/src/tributes/incidents.rs` — core changes: enum variants, constants, roll/random signatures, biome pools, `SleepShelter`, `find_shelter()`, `apply_sleep_incident` changes, Nightmare/NightTear apply.
- `game/src/tributes/mod.rs` — add transient `sleep_shelter: Option<SleepShelter>` field.
- `game/src/games/cycle.rs` — shelter roll at sleep start, updated incident roll call, AllyAbandonment cycle integration, NightTerror wake emission, shelter clear on wake.
- `shared/src/messages/mod.rs` — add `InterruptionKind::NightTerror`, add `SleepIncidentKind::Nightmare` + `SleepIncidentKind::NightTerror`, remove `SleepIncidentKind::Hallucination`.

---

## Task Order Rationale

Tasks build the data types first (Tasks 1–3), then the mechanical roll/shelter functions (4–5), then the apply effects (6), then cycle integration (7), then shared/messages alignment (8), then tests (9). Each task is independently shippable and reviewable. TDD throughout: failing test first.

---

## Task 1: Rename `Hallucination` → `Nightmare`, add `NightTerror` + `SleepShelter` enums

**Why first:** Every subsequent task references these types. Land them early so the compiler drives the rest of the work.

**Files:**
- Modify: `game/src/tributes/incidents.rs` (enum definitions, `wakes_tribute`, `From<&SleepIncident>` for `SleepIncidentKind`)
- Modify: `shared/src/messages/mod.rs` (mirror renames in `SleepIncidentKind` once Task 8 handles the shared side — for now the `From` impl in incidents.rs will temporarily reference removed enum variants, so Task 1 keeps the old `shared` variants alive until Task 8.)

**Note:** The `From<&SleepIncident> for SleepIncidentKind` impl (`incidents.rs:61`) maps `SleepIncident::Hallucination` → `SleepIncidentKind::Hallucination`. Rename the game-side variant first, update the `From` impl to map `Nightmare` → `SleepIncidentKind::Nightmare` (and `NightTerror` → `SleepIncidentKind::NightTerror`). The shared enum gets updated in Task 8. The compiler will warn about dead variants in shared until Task 8 — suppress with an `#[allow(unused)]` or accept the short-lived warning.

- [ ] **Step 1: Write failing tests**

Append to the test module in `incidents.rs`:

```rust
#[test]
fn nightmare_does_not_wake() {
    assert!(!SleepIncident::Nightmare.wakes_tribute());
}

#[test]
fn night_terror_wakes_tribute() {
    assert!(SleepIncident::NightTerror.wakes_tribute());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game tributes::incidents::tests::nightmare_does_not_wake tributes::incidents::tests::night_terror_wakes_tribute`
Expected: FAIL — `Nightmare` and `NightTerror` not found.

- [ ] **Step 3: Rename and add variants in `incidents.rs`**

In `SleepIncident` enum (line 12), replace:
```rust
    /// Hallucination/dream causing sanity loss.
    Hallucination,
```
with:
```rust
    /// Bad dream causing sanity loss. Sleeper does not wake.
    Nightmare { sanity_loss: u32 },
```

Add after `LimbInjury`:
```rust
    /// Intense phobia-driven night terror. Wakes the tribute.
    NightTerror { sanity_loss: u32 },
```

Update `wakes_tribute()` (line 79):
```rust
    pub fn wakes_tribute(&self) -> bool {
        matches!(
            self,
            SleepIncident::Theft { .. }
                | SleepIncident::Relocation { .. }
                | SleepIncident::AnimalEncounter { .. }
                | SleepIncident::NightTerror
                | SleepIncident::LimbInjury
        )
    }
```

Remove `SleepIncident::AllyAbandonment` from `wakes_tribute()` — ally abandonment no longer wakes per spec §2.6.

Update `From<&SleepIncident> for SleepIncidentKind` (line 61):
```rust
            SleepIncident::Hallucination => SleepIncidentKind::Hallucination,
```
becomes:
```rust
            SleepIncident::Nightmare { .. } => SleepIncidentKind::Nightmare,
```

Add the `NightTerror` arm:
```rust
            SleepIncident::NightTerror { .. } => SleepIncidentKind::NightTerror,
```

- [ ] **Step 4: Add `SleepShelter` enum + multiplier method**

After the `SleepIncident` enum definition (before `AnnoyingFlavor` at line 29), insert:

```rust
/// Shelter quality a tribute can actively create/find for a sleep session.
/// Higher tiers provide better protection against sleep incidents.
/// Transient — set per-sleep-session, cleared on wake or phase change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepShelter {
    /// Sleeping in the open. No protection.
    None,
    /// Leaves, brush, shallow depression.
    Crude,
    /// Cave, hollow log, dense thicket.
    Natural,
    /// Reinforced position. Rare.
    Fortified,
}

impl SleepShelter {
    /// Incident probability multiplier for this shelter tier.
    /// None=1.0, Crude=0.8, Natural=0.5, Fortified=0.3
    pub fn multiplier(&self) -> f64 {
        match self {
            SleepShelter::None => 1.0,
            SleepShelter::Crude => 0.8,
            SleepShelter::Natural => 0.5,
            SleepShelter::Fortified => 0.3,
        }
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game tributes::incidents::tests`
Expected: PASS (the two new tests + all existing tests still green).

- [ ] **Step 6: Run `cargo check --workspace`**

Expected: warnings about `SleepIncidentKind::Hallucination` referring to a variant that maps to a removed `SleepIncident` variant in `cycle.rs:568`. That's expected — Task 2 fixes the random/roll, Task 8 aligns shared. Accept interim warnings.

- [ ] **Step 7: Commit**

```bash
jj describe -m "feat(game): rename Hallucination -> Nightmare, add NightTerror + SleepShelter

- Hallucination variant becomes Nightmare { sanity_loss } — no wake
- New NightTerror { sanity_loss } variant — wakes tribute, phobia-trigger
- AllyAbandonment removed from wakes_tribute() per spec §2.6
- SleepShelter enum (None/Crude/Natural/Fortified) with multiplier()
- From<&SleepIncident> impl updated for both new variants

Refs: hangrier_games-pending"
```

---

## Task 2: Phase-aware base constants + biome/shelter/day multiplier functions

**Why second:** The new `effective_incident_chance()` calculation replaces the flat `SLEEP_INCIDENT_CHANCE_PCT` constant. Both `roll()` and the cycle call site depend on it.

**Files:**
- Modify: `game/src/tributes/incidents.rs`

- [ ] **Step 1: Write failing tests**

Append to the test module in `incidents.rs`:

```rust
    use crate::areas::weather::Weather;
    use crate::terrain::types::BaseTerrain;

    #[rstest]
    #[case(Phase::Day, 8)]
    #[case(Phase::Dawn, 12)]
    #[case(Phase::Dusk, 12)]
    #[case(Phase::Night, 22)]
    fn base_incident_chance_by_phase(#[case] phase: Phase, #[case] expected: u32) {
        assert_eq!(base_incident_chance(phase), expected);
    }

    #[rstest]
    #[case(BaseTerrain::UrbanRuins, 0.4)]
    #[case(BaseTerrain::Forest, 0.6)]
    #[case(BaseTerrain::Desert, 1.0)]
    #[case(BaseTerrain::Tundra, 1.0)]
    fn biome_incident_multiplier_values(#[case] biome: BaseTerrain, #[case] expected: f64) {
        let got = biome_incident_multiplier(biome);
        assert!((got - expected).abs() < f64::EPSILON * 10.0);
    }

    #[rstest]
    #[case(SleepShelter::None, 1.0)]
    #[case(SleepShelter::Crude, 0.8)]
    #[case(SleepShelter::Natural, 0.5)]
    #[case(SleepShelter::Fortified, 0.3)]
    fn sleep_shelter_multiplier_values(#[case] shelter: SleepShelter, #[case] expected: f64) {
        let got = sleep_shelter_multiplier(&shelter);
        assert!((got - expected).abs() < f64::EPSILON * 10.0);
    }

    #[rstest]
    #[case(0, 1.0)]
    #[case(1, 1.0)]
    #[case(2, 1.2)]
    #[case(4, 1.5)]
    #[case(6, 2.0)]
    #[case(99, 2.0)]
    fn day_scaling_multiplier_values(#[case] day: u32, #[case] expected: f64) {
        let got = day_scaling_multiplier(day);
        assert!((got - expected).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn effective_chance_night_desert_no_shelter() {
        // Night (22%) * Desert (1.0) * day 1 (1.0) = 22%
        let chance = effective_incident_chance(
            Phase::Night,
            BaseTerrain::Desert,
            false,
            &SleepShelter::None,
            1,
        );
        assert!((chance - 22.0).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn effective_chance_caps_at_100() {
        // Night (22%) * Tundra (1.0) * day 6 (2.0) = 44%... not near 100.
        // Use extreme: Night (22%) * None shelter multiplier (1.0) on purpose.
        // Actually 22 * 2.0 = 44%. Not near cap. For cap test use extreme params.
        let chance = effective_incident_chance(
            Phase::Night,
            BaseTerrain::Desert,
            false,
            &SleepShelter::None,
            99, // day 99 → 2.0x
        );
        assert!(chance <= 100.0);
    }

    #[test]
    fn effective_chance_sheltered_reduces() {
        // Night (22%) * built shelter (0.5) * day 1 (1.0) = 11%
        let chance = effective_incident_chance(
            Phase::Night,
            BaseTerrain::Desert,
            true,  // is_sheltered
            &SleepShelter::None,  // ignored when is_sheltered
            1,
        );
        assert!((chance - 11.0).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn effective_chance_sleep_shelter_takes_priority() {
        // Night (22%) * sleep shelter Fortified (0.3) * day 1 (1.0) = 6.6%
        let chance = effective_incident_chance(
            Phase::Night,
            BaseTerrain::Desert,
            false,  // not in built shelter
            &SleepShelter::Fortified,
            1,
        );
        assert!((chance - 6.6).abs() < 0.01);
    }
```

- [ ] **Step 2: Add imports + `use` for `Phase` and `BaseTerrain` at top of test module

Add at top of test module (line 239):
```rust
    use crate::messages::Phase;
    use crate::areas::shelter::shelter_quality;
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --package game tributes::incidents::tests::base_incident_chance_by_phase`
Expected: FAIL — `base_incident_chance` undefined.

- [ ] **Step 4: Implement the constants and functions**

After the `SleepShelter` impl block (around line 85), add:

```rust
// ── Phase base rates (spec §2.1) ──
const SLEEP_INCIDENT_DAY_PCT: u32 = 8;
const SLEEP_INCIDENT_DAWN_PCT: u32 = 12;
const SLEEP_INCIDENT_DUSK_PCT: u32 = 12;
const SLEEP_INCIDENT_NIGHT_PCT: u32 = 22;

// ── Shelter multiplier (constructed shelter, spec §2.2) ──
const SLEEP_INCIDENT_SHELTER_MULTIPLIER: f64 = 0.5;

// ── Shelter quality multipliers (per biome score, spec §2.3) ──
const SHELTER_QUALITY_SCORE_3: f64 = 0.4; // UrbanRuins
const SHELTER_QUALITY_SCORE_2: f64 = 0.6; // Forest, Jungle, Mountains, Geothermal
const SHELTER_QUALITY_SCORE_1: f64 = 0.8; // Wetlands, Highlands, Clearing, Grasslands, Badlands
const SHELTER_QUALITY_SCORE_0: f64 = 1.0; // Tundra, Desert

// ── Sleep shelter multipliers (per tier, spec §2.4) ──
const SLEEP_SHELTER_NONE_MULTIPLIER: f64 = 1.0;
const SLEEP_SHELTER_CRUDE_MULTIPLIER: f64 = 0.8;
const SLEEP_SHELTER_NATURAL_MULTIPLIER: f64 = 0.5;
const SLEEP_SHELTER_FORTIFIED_MULTIPLIER: f64 = 0.3;
```

Then add the public functions after the constants block:

```rust
/// Base incident probability for a given phase (spec §2.1).
pub fn base_incident_chance(phase: crate::messages::Phase) -> u32 {
    use crate::messages::Phase;
    match phase {
        Phase::Day => SLEEP_INCIDENT_DAY_PCT,
        Phase::Dawn | Phase::Dusk => SLEEP_INCIDENT_DAWN_PCT,
        Phase::Night => SLEEP_INCIDENT_NIGHT_PCT,
    }
}

/// Incident probability multiplier derived from biome shelter_quality (spec §2.3).
pub fn biome_incident_multiplier(biome: crate::terrain::types::BaseTerrain) -> f64 {
    match crate::areas::shelter::shelter_quality(biome, &crate::areas::weather::current_weather()) {
        3 => SHELTER_QUALITY_SCORE_3,
        2 => SHELTER_QUALITY_SCORE_2,
        1 => SHELTER_QUALITY_SCORE_1,
        _ => SHELTER_QUALITY_SCORE_0,
    }
}

/// Incident probability multiplier for a SleepShelter tier (spec §2.4).
pub fn sleep_shelter_multiplier(shelter: &SleepShelter) -> f64 {
    match shelter {
        SleepShelter::None => SLEEP_SHELTER_NONE_MULTIPLIER,
        SleepShelter::Crude => SLEEP_SHELTER_CRUDE_MULTIPLIER,
        SleepShelter::Natural => SLEEP_SHELTER_NATURAL_MULTIPLIER,
        SleepShelter::Fortified => SLEEP_SHELTER_FORTIFIED_MULTIPLIER,
    }
}

/// Day-based frequency scaling factor (spec §2.8).
pub fn day_scaling_multiplier(current_day: u32) -> f64 {
    match current_day {
        0..=1 => 1.0,
        2..=3 => 1.2,
        4..=5 => 1.5,
        _ => 2.0,
    }
}

/// Compute the effective incident probability for a given context (spec §5.1).
///
/// Priority order:
/// 1. Constructed shelter (`is_sheltered` = true) -> 0.5x
/// 2. Sleep shelter -> tier multiplier
/// 3. Biome shelter_quality -> fallback
pub fn effective_incident_chance(
    phase: crate::messages::Phase,
    biome: crate::terrain::types::BaseTerrain,
    is_sheltered: bool,
    sleep_shelter: &SleepShelter,
    current_day: u32,
) -> f64 {
    let base = base_incident_chance(phase) as f64;

    let factor = if is_sheltered {
        SLEEP_INCIDENT_SHELTER_MULTIPLIER
    } else if *sleep_shelter != SleepShelter::None {
        sleep_shelter_multiplier(sleep_shelter)
    } else {
        biome_incident_multiplier(biome)
    };

    let day_scale = day_scaling_multiplier(current_day);

    (base * factor * day_scale).min(100.0)
}
```

- [ ] **Step 5: Delete the old `SLEEP_INCIDENT_CHANCE_PCT` constant** (line 8)

- [ ] **Step 6: Add the needed `use` imports at the top of the file**

Add to the imports at top:
```rust
use crate::areas::shelter;
use crate::areas::weather;
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test --package game tributes::incidents::tests`
Expected: PASS — all new param tests green.

- [ ] **Step 8: Commit**

```bash
jj describe -m "feat(game): phase-aware base rates + biome/shelter/day multiplier functions

Replaces flat SLEEP_INCIDENT_CHANCE_PCT (18%) with:
- base_incident_chance(phase) -> Day 8, Dawn/Dusk 12, Night 22
- biome_incident_multiplier(biome) -> 0.4-1.0 via shelter_quality()
- sleep_shelter_multiplier(shelter) -> 1.0/0.8/0.5/0.3
- day_scaling_multiplier(day) -> 1.0-2.0 scaling curve
- effective_incident_chance(...) -> clamped composite

All constants tunable post-observability.

Refs: hangrier_games-pending"
```

---

## Task 3: Biome-specific animal pools + flavor pools

**Why third:** The `random()` method expands to accept `biome: BaseTerrain`. The animal/flavor lookup tables must exist first.

**Files:**
- Modify: `game/src/tributes/incidents.rs`

- [ ] **Step 1: Write failing tests**

Append to the test module:

```rust
    #[rstest]
    fn biome_animals_are_non_empty(#[values(
        BaseTerrain::Desert, BaseTerrain::Forest, BaseTerrain::Jungle,
        BaseTerrain::Wetlands, BaseTerrain::Tundra, BaseTerrain::Grasslands,
        BaseTerrain::Mountains, BaseTerrain::Badlands, BaseTerrain::Highlands,
        BaseTerrain::Geothermal, BaseTerrain::UrbanRuins, BaseTerrain::Clearing,
    )] biome: BaseTerrain) {
        let animals = biome_animal_pool(biome);
        assert!(!animals.is_empty(), "biome {:?} should have animals", biome);
    }

    #[test]
    fn desert_animals_are_desert_appropriate() {
        let animals = biome_animal_pool(BaseTerrain::Desert);
        for a in &["scorpion", "rattlesnake", "coyote", "gila monster", "tarantula"] {
            assert!(animals.contains(a), "desert should have {a}");
        }
    }

    #[test]
    fn forest_animals_are_forest_appropriate() {
        let animals = biome_animal_pool(BaseTerrain::Forest);
        for a in &["bear", "wolf", "wild boar", "fox", "owl"] {
            assert!(animals.contains(a), "forest should have {a}");
        }
    }

    #[rstest]
    fn biome_flavors_are_non_empty(#[values(
        BaseTerrain::Desert, BaseTerrain::Forest, BaseTerrain::Jungle,
        BaseTerrain::Wetlands, BaseTerrain::Tundra, BaseTerrain::Grasslands,
        BaseTerrain::Mountains, BaseTerrain::Badlands, BaseTerrain::Highlands,
        BaseTerrain::Geothermal, BaseTerrain::UrbanRuins, BaseTerrain::Clearing,
    )] biome: BaseTerrain) {
        let flavors = biome_flavor_pool(biome);
        assert!(!flavors.is_empty(), "biome {:?} should have flavors", biome);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --package game tributes::incidents::tests::biome_animals_are_non_empty`
Expected: FAIL — `biome_animal_pool` undefined.

- [ ] **Step 3: Implement the pool functions**

Before the `impl SleepIncident` block (around line 89), add:

```rust
// ── Biome-specific animal encounter pools (spec §2.5) ──

/// Returns the animal pool for a given biome. Used by `AnimalEncounter` selection.
pub fn biome_animal_pool(biome: crate::terrain::types::BaseTerrain) -> &'static [&'static str] {
    use crate::terrain::types::BaseTerrain;
    match biome {
        BaseTerrain::Desert => &["scorpion", "rattlesnake", "coyote", "gila monster", "tarantula"],
        BaseTerrain::Forest => &["bear", "wolf", "wild boar", "fox", "owl"],
        BaseTerrain::Jungle => &["jaguar", "python", "poison dart frog", "spider", "howler monkey"],
        BaseTerrain::Wetlands => &["alligator", "snapping turtle", "leeches", "cottonmouth snake", "bullfrog"],
        BaseTerrain::Tundra => &["polar bear", "wolf pack", "snowy owl", "arctic fox", "musk ox"],
        BaseTerrain::Grasslands => &["cougar", "rattlesnake", "wild dog", "hawk", "bison"],
        BaseTerrain::Mountains => &["mountain lion", "goat", "golden eagle", "marmot", "wolverine"],
        BaseTerrain::Badlands => &["coyote", "rattlesnake", "vulture", "scorpion", "roadrunner"],
        BaseTerrain::Highlands => &["wolf", "golden eagle", "deer", "fox", "wildcat"],
        BaseTerrain::Geothermal => &["lynx", "hawk", "fox", "salamander", "badger"],
        BaseTerrain::UrbanRuins => &["feral dog", "crow swarm", "rat swarm", "stray cat", "raccoon"],
        BaseTerrain::Clearing => &["fox", "hawk", "deer", "snake", "rabbit"],
    }
}

/// Returns the environmental flavor pool for a given biome. Used by `Annoying` selection.
pub fn biome_flavor_pool(biome: crate::terrain::types::BaseTerrain) -> &'static [&'static str] {
    use crate::terrain::types::BaseTerrain;
    match biome {
        BaseTerrain::Desert => &[
            "a sandstorm whips grit across their face",
            "the cold desert wind howls",
        ],
        BaseTerrain::Forest => &[
            "a branch falls nearby",
            "pine needles drift down with the breeze",
        ],
        BaseTerrain::Jungle => &[
            "heavy rain drips through the canopy",
            "a howler monkey screeches nearby",
        ],
        BaseTerrain::Wetlands => &[
            "a chorus of frogs swells around them",
            "something large splashes in the murk",
        ],
        BaseTerrain::Tundra => &[
            "the wind screams across the frozen plain",
            "ice crystals form on their eyelashes",
        ],
        BaseTerrain::Grasslands => &[
            "wind rustles through dry grass",
            "a distant prairie fire glows on the horizon",
        ],
        BaseTerrain::Mountains => &[
            "a rockslide echoes from the peak above",
            "thin air makes each breath a labour",
        ],
        BaseTerrain::Badlands => &[
            "wind whistles through the canyons",
            "a rockfall tumbles nearby",
        ],
        BaseTerrain::Highlands => &[
            "a thick mist rolls in",
            "the wind whips across the moor",
        ],
        BaseTerrain::Geothermal => &[
            "a steam vent hisses in the dark",
            "the ground rumbles softly",
        ],
        BaseTerrain::UrbanRuins => &[
            "debris shifts in an abandoned building",
            "the wind moans through broken windows",
        ],
        BaseTerrain::Clearing => &[
            "a gentle breeze rustles the meadow",
            "a firefly lands on their hand",
        ],
    }
}
```

- [ ] **Step 4: Update `random_animal_name` to become `biome_animal_name`**

Replace the existing `fn random_animal_name` (line 123) with a version that takes `biome`:

```rust
    fn random_biome_animal(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> String {
        let pool = biome_animal_pool(biome);
        let idx = rng.random_range(0..pool.len());
        pool[idx].to_string()
    }
```

Also add a `random_biome_flavor` helper:
```rust
    fn random_biome_flavor(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> String {
        let pool = biome_flavor_pool(biome);
        let idx = rng.random_range(0..pool.len());
        pool[idx].to_string()
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game tributes::incidents::tests::biome_animals_are_non_empty tributes::incidents::tests::biome_flavors_are_non_empty`
Expected: PASS.

- [ ] **Step 6: Update `AnnoyingFlavor` to use biome flavors**

The `AnnoyingFlavor::random()` currently picks from a flat 5-option enum. Replace the `AnnoyingFlavor` system with the biome-specific flavor pool. Two approaches:
- **Option A (recommended for v1):** Keep `AnnoyingFlavor` but change its `random()` to delegate to the biome pool, adding a `biome` parameter. The flavor's `description()` returns the selected string rather than matching a variant.
- **Option B:** Replace `AnnoyingFlavor` entirely with a `String` payload.

Go with **Option A** to minimise churn. Change `AnnoyingFlavor::random(rng)` to `AnnoyingFlavor::random(rng, biome)`:

```rust
impl AnnoyingFlavor {
    fn random(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> Self {
        let pool = biome_flavor_pool(biome);
        let idx = rng.random_range(0..pool.len());
        // Store the selected flavor text directly for simplicity.
        // The enum variants remain for backward compat but the
        // `description()` will just return the stored text.
        AnnoyingFlavor::Custom(pool[idx].to_string())
    }
}
```

Add a `Custom(String)` variant to `AnnoyingFlavor`. Update `description()` to match:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum AnnoyingFlavor {
    SquirrelOnChest,
    ButterflyLanded,
    WeirdDream,
    MouseRanOver,
    LeafOnFace,
    /// Biome-specific flavor text.
    Custom(String),
}

impl AnnoyingFlavor {
    fn description(&self) -> &str {
        match self {
            AnnoyingFlavor::SquirrelOnChest => "a squirrel on their chest",
            AnnoyingFlavor::ButterflyLanded => "a butterfly landing on their nose",
            AnnoyingFlavor::WeirdDream => "a weird dream about turnips",
            AnnoyingFlavor::MouseRanOver => "a mouse running over their face",
            AnnoyingFlavor::LeafOnFace => "a leaf drifting onto their face",
            AnnoyingFlavor::Custom(text) => text.as_str(),
        }
    }
}
```

- [ ] **Step 7: Run full test suite**

Run: `cargo test --package game tributes::incidents`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
jj describe -m "feat(game): biome-specific animal + flavor incident pools

Animal encounters now draw from per-biome pools (5 animals each)
instead of the flat 8-animal list. Annoying flavor incidents draw
from biome-specific environmental flavor pools.

Refs: hangrier_games-pending"
```

---

## Task 4: Update `roll()` and `random()` signatures — phase-aware, biome-aware

**Why fourth:** The core entry points `roll()` and `random()` now accept the context parameters needed by the cycle.

**Files:**
- Modify: `game/src/tributes/incidents.rs`

- [ ] **Step 1: Write failing tests**

```rust
    #[test]
    fn roll_with_zero_effective_chance_returns_none() {
        let mut rng = SmallRng::seed_from_u64(42);
        // Day (8%) * Fortified shelter (0.3) * day 1 (1.0) = 2.4% — still not zero.
        // Use a phase with base 0... there is none (minimum is 8). So test
        // that the chance is respected: run many iterations and verify that
        // roll returns None at least sometimes.
        let mut incident_count = 0;
        for _ in 0..1000 {
            if SleepIncident::roll(
                &mut rng,
                crate::messages::Phase::Day,
                crate::terrain::types::BaseTerrain::UrbanRuins,
                true, // sheltered — 0.5x
                1,    // day 1
            )
            .is_some()
            {
                incident_count += 1;
            }
        }
        // Expected: 8 * 0.4 * 0.5 = 1.6% → ~16 per 1000. Allow 0-50 for noise.
        assert!(
            incident_count < 100,
            "expected < 100 incidents in 1000 rolls at 1.6% chance, got {incident_count}"
        );
    }
```

- [ ] **Step 2: Update `roll()` signature and body**

Replace the existing `roll` method (line 91):

```rust
    /// Roll whether a sleep incident occurs this phase. Now phase-aware,
    /// biome-aware, shelter-aware, and day-aware.
    pub fn roll(
        rng: &mut impl Rng,
        phase: crate::messages::Phase,
        biome: crate::terrain::types::BaseTerrain,
        is_sheltered: bool,
        current_day: u32,
    ) -> Option<Self> {
        let chance = effective_incident_chance(phase, biome, is_sheltered, &SleepShelter::None, current_day);
        // Note: sleep_shelter factor is separately applied in the cycle;
        // the roll() function receives the composite effective chance with
        // sleep_shelter already factored in via is_sheltered + biome.
        // Actually no — sleep_shelter is set AFTER the sleep phase starts.
        // We pass SleepShelter::None here and the cycle pre-computes the
        // effective chance with the actual sleep_shelter value.
        // REVISED: roll() takes all params, including sleep_shelter.
        // We add sleep_shelter to the signature in step 3.
        if !rng.random_bool(chance / 100.0) {
            return None;
        }
        Some(Self::random(rng, biome))
    }
```

Wait, I need to be more careful. The spec says roll() takes `is_sheltered` (constructed shelter bool) and internally uses `SleepShelter`. But actually, `SleepShelter` is determined each sleep session. The function should take the `sleep_shelter: &SleepShelter` parameter too.

Let me re-read the spec call signature from §4.1:

```rust
pub fn roll(
    rng: &mut impl Rng,
    phase: Phase,
    biome: BaseTerrain,
    is_sheltered: bool,
    current_day: u32,
) -> Option<Self>
```

The spec shows `is_sheltered` (constructed shelter bool) not `sleep_shelter`. The sleep_shelter modifier is handled by the priority order in `effective_incident_chance`: if `is_sheltered` is true, use 0.5x. Otherwise the cycle passes `SleepShelter::None` for the sleep_shelter value (since it's rolled per-sleep-session and stored on the tribute).

But wait, let me re-read spec §5.1 which shows `sleep_shelter` as a separate parameter. And the spec says `roll()` signature only has `is_sheltered: bool`, not `sleep_shelter`.

Looking at the priority order again:
1. Constructed shelter (is_sheltered) → 0.5x
2. Sleep shelter → tier multiplier
3. Biome shelter_quality → fallback

So the cycle should:
1. Determine `is_sheltered` from `tribute.sheltered_until`
2. If not in constructed shelter, call `find_shelter(tribute, terrain)` and store result in `tribute.sleep_shelter`
3. Pass both `is_sheltered` and the sleep_shelter to `effective_incident_chance()`

But the `roll()` signature in the spec only takes `is_sheltered`, not `sleep_shelter`. Hmm, this is a simplification — the cycle would pre-compute the effective chance using sleep_shelter and then pass the final effective chance... No, the spec clearly has `is_sheltered` not `sleep_shelter` in the roll signature.

Actually, re-reading spec §4.1 more carefully — the signature it shows might be a simplification. The `effective_incident_chance` function (spec §5.1) definitely takes `sleep_shelter`. 

I think the right approach is: `roll()` takes all parameters, including `sleep_shelter`. The cycle determines the sleep_shelter by calling `find_shelter()` before the roll. Let me adjust.

So the signature becomes:
```rust
pub fn roll(
    rng: &mut impl Rng,
    phase: Phase,
    biome: BaseTerrain,
    is_sheltered: bool,
    sleep_shelter: &SleepShelter,
    current_day: u32,
) -> Option<Self>
```

And `random()` becomes:
```rust
pub fn random(rng: &mut impl Rng, biome: BaseTerrain) -> Self
```

- [ ] **Step 2: Update `roll()` and `random()` signatures**

Replace `roll()` with:

```rust
    /// Roll whether a sleep incident occurs this phase. Phase-aware, biome-aware,
    /// shelter-aware (both constructed shelter and sleep shelter), day-aware.
    pub fn roll(
        rng: &mut impl Rng,
        phase: crate::messages::Phase,
        biome: crate::terrain::types::BaseTerrain,
        is_sheltered: bool,
        sleep_shelter: &SleepShelter,
        current_day: u32,
    ) -> Option<Self> {
        let chance = effective_incident_chance(phase, biome, is_sheltered, sleep_shelter, current_day);
        if !rng.random_bool(chance / 100.0) {
            return None;
        }
        Some(Self::random(rng, biome))
    }
```

Replace `random()` with:

```rust
    /// Pick a random sleep incident with weighted probabilities (spec §3.2).
    pub fn random(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> Self {
        // Weights: Annoying 30%, Nightmare 15%, NightTerror 5% (phobia gate),
        // Theft 12%, Relocation 10%, AnimalEncounter 10%, LimbInjury 8%,
        // AllyAbandonment 10% (ally gate).
        let roll: u32 = rng.random_range(0..100);
        match roll {
            0..=29 => SleepIncident::Annoying {
                flavor: AnnoyingFlavor::random(rng, biome),
            },
            30..=44 => SleepIncident::Nightmare {
                sanity_loss: rng.random_range(2..=6),
            },
            45..=49 => {
                // NightTerror: gated on phobia existence. If no phobia,
                // fall back to Nightmare. The phobia check is done at
                // apply time (or the cycle kills the NightTerror roll).
                // For the RNG selection, we pick it; the caller can
                // downgrade to Nightmare if no phobia.
                SleepIncident::NightTerror {
                    sanity_loss: rng.random_range(5..=12),
                }
            }
            50..=61 => SleepIncident::Theft {
                stolen_item: String::new(),
            },
            62..=71 => {
                // AllyAbandonment: gated on qualified ally existing.
                // Cycle handles the gate. If no qualified ally, cycle
                // rerolls or falls back to Annoying.
                SleepIncident::AllyAbandonment
            }
            72..=81 => {
                let animal = Self::random_biome_animal(rng, biome);
                SleepIncident::AnimalEncounter { animal }
            }
            82..=89 => SleepIncident::LimbInjury,
            _ => SleepIncident::Relocation {
                new_area: Self::random_area(rng),
            },
        }
    }
```

Note the weight table changed per spec §3.2:
- Annoying: 30% (was ~30%)
- Nightmare: 15% (was Hallucination ~15%)
- NightTerror: 5% (new)
- Theft: 12% (was ~13%)
- AllyAbandonment: 10% (was ~11%)
- AnimalEncounter: 10% (was ~10%)
- Relocation: 10% (was ~10%) — moved up because total must sum to 100
- LimbInjury: 8% (was ~12%)

Wait, let me recalculate: 30 + 15 + 5 + 12 + 10 + 10 + 8 = 90. Need 10 more → Relocation gets 10.

Fixed: Annoying 30, Nightmare 15, NightTerror 5, Theft 12, AllyAbandonment 10, AnimalEncounter 10, LimbInjury 8, Relocation 10. That's 30+15+5+12+10+10+8+10 = 100. ✓

The spec says `90-99` for AllyAbandonment. Let me match that:
- 0-29 → Annoying (30%)
- 30-44 → Nightmare (15%)
- 45-49 → NightTerror (5%)
- 50-61 → Theft (12%)
- 62-71 → Relocation (10%)
- 72-81 → AnimalEncounter (10%)
- 82-89 → LimbInjury (8%)
- 90-99 → AllyAbandonment (10%)

= 100. ✓

- [ ] **Step 3: Update test `roll_sleep_incident_sometimes_returns_none`**

Update the old test at line 244 to use the new signature:

```rust
    #[test]
    fn roll_sleep_incident_sometimes_returns_none() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut found_some = false;
        for _ in 0..100 {
            if SleepIncident::roll(
                &mut rng,
                crate::messages::Phase::Night,
                crate::terrain::types::BaseTerrain::Forest,
                false,
                &SleepShelter::None,
                1,
            )
            .is_some()
            {
                found_some = true;
                break;
            }
        }
        assert!(found_some, "should roll at least one incident in 100 tries");
    }
```

- [ ] **Step 4: Run tests to verify they compile and pass**

Run: `cargo test --package game tributes::incidents`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): phase/biome/shelter/day-aware roll() + random() signatures

roll() signature expanded to accept phase, biome, is_sheltered,
sleep_shelter, current_day. random() accepts biome for biome-specific
animal/flavor selection. Weight table updated to spec §3.2:
Annoying 30%, Nightmare 15%, NightTerror 5%, Theft 12%,
Relocation 10%, AnimalEncounter 10%, LimbInjury 8%,
AllyAbandonment 10%.

Refs: hangrier_games-pending"
```

---

## Task 5: `find_shelter()` function

**Why fifth:** The cycle needs to call `find_shelter()` at sleep start to determine the tribute's `sleep_shelter`. This is a new pure function.

**Files:**
- Modify: `game/src/tributes/incidents.rs`

- [ ] **Step 1: Write failing tests**

```rust
    #[test]
    fn find_shelter_urban_ruins_high_int_no_shelter_always_fortified() {
        // UrbanRuins has shelter_quality 3 (easiest DC).
        // Intelligence 100 should always succeed at finding Fortified shelter.
        let mut rng = SmallRng::seed_from_u64(42);
        let result = find_shelter(100, 50, crate::terrain::types::BaseTerrain::UrbanRuins, &mut rng);
        // Highest possible (Int 100, best terrain). Expect Natural or Fortified.
        assert!(
            result == SleepShelter::Natural || result == SleepShelter::Fortified,
            "expected Natural or Fortified, got {:?}",
            result
        );
    }

    #[test]
    fn find_shelter_desert_low_int_always_none_or_crude() {
        // Desert has shelter_quality 0 (hardest DC). Low stats.
        let mut rng = SmallRng::seed_from_u64(42);
        let result = find_shelter(10, 10, crate::terrain::types::BaseTerrain::Desert, &mut rng);
        assert!(
            result == SleepShelter::None || result == SleepShelter::Crude,
            "expected None or Crude, got {:?}",
            result
        );
    }

    #[test]
    fn find_shelter_mid_terrain_mid_stats_variable() {
        // Forest (shelter_quality 2). Mid Int/Str.
        let mut rng = SmallRng::seed_from_u64(42);
        let mut found_natural = false;
        for _ in 0..50 {
            let result = find_shelter(50, 50, crate::terrain::types::BaseTerrain::Forest, &mut rng);
            if result == SleepShelter::Natural {
                found_natural = true;
                break;
            }
        }
        assert!(found_natural, "mid stats in forest should occasionally get Natural");
    }
```

- [ ] **Step 2: Implement `find_shelter()`**

Before `impl SleepIncident` (or after the pool functions), add:

```rust
/// Roll for a tribute to find or build shelter for the sleep session.
/// Uses the higher of Intelligence (finding) or Strength (building) with
/// a random factor. The terrain's `shelter_quality` (0-3) sets the DC.
///
/// Returns a `SleepShelter` tier:
/// - High roll relative to DC -> Fortified or Natural
/// - Mid roll -> Crude
/// - Low roll -> None
///
/// Spec §2.4
pub fn find_shelter(
    intelligence: u32,
    strength: u32,
    terrain: crate::terrain::types::BaseTerrain,
    rng: &mut impl Rng,
) -> SleepShelter {
    use crate::terrain::types::BaseTerrain;

    let score = crate::areas::shelter::shelter_quality(terrain, &crate::areas::weather::current_weather());
    // Base DC: higher shelter_quality = easier.
    // Map: score 0→DC 25, 1→DC 20, 2→DC 15, 3→DC 10
    let dc = 25 - (score as u32 * 5);

    // Use the higher of Int or Str, with ±20% random variance.
    let stat = intelligence.max(strength);
    let variance = rng.random_range(80..=120);
    let effective_roll = (stat * variance) / 100; // cent-based variance

    // Compare effective_roll to DC and thresholds for shelter tiers.
    if effective_roll >= dc + 20 {
        SleepShelter::Fortified
    } else if effective_roll >= dc + 10 {
        SleepShelter::Natural
    } else if effective_roll >= dc {
        SleepShelter::Crude
    } else {
        SleepShelter::None
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --package game tributes::incidents::tests::find_shelter_urban_ruins_high_int`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): add find_shelter() shelter-roll function

Tribute rolls Intelligence or Strength (whichever is higher) against
a terrain-derived DC. Result tiers: Fortified, Natural, Crude, or None.
Used by the cycle at sleep start.

Refs: hangrier_games-pending"
```

---

## Task 6: Update `apply_sleep_incident` — Nightmare, NightTerror, remove AllyAbandonment

**Why sixth:** The apply function needs to handle the two new variants and remove the AllyAbandonment branch (moved to cycle).

**Files:**
- Modify: `game/src/tributes/incidents.rs`

- [ ] **Step 1: Write failing tests**

```rust
    #[test]
    fn apply_nightmare_reduces_sanity_does_not_wake() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.attributes.sanity;
        let incident = SleepIncident::Nightmare { sanity_loss: 4 };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(desc.contains("sanity"), "description should mention sanity");
        assert_eq!(
            tribute.attributes.sanity,
            orig_sanity - 4,
            "sanity should decrease by 4"
        );
        assert!(
            !incident.wakes_tribute(),
            "Nightmare should not wake tribute"
        );
    }

    #[test]
    fn apply_night_terror_reduces_sanity() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.attributes.sanity;
        let incident = SleepIncident::NightTerror { sanity_loss: 8 };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(desc.contains("sanity"), "description should mention sanity");
        assert_eq!(
            tribute.attributes.sanity,
            orig_sanity - 8,
            "sanity should decrease by 8"
        );
    }

    #[test]
    fn ally_abandonment_still_applies_flavor_text() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        // Even without allies, should produce flavor text (the cycle
        // gating ensures this only fires when allies exist).
        let incident = SleepIncident::AllyAbandonment;
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(!desc.is_empty(), "should not panic without allies");
        // The ally removal is now handled by the cycle, not apply.
        // The apply function produces flavor text only.
    }
```

- [ ] **Step 2: Update `apply_sleep_incident`**

Replace the `SleepIncident::Hallucination` branch with `Nightmare`:

```rust
        SleepIncident::Nightmare { sanity_loss } => {
            tribute.attributes.sanity = tribute
                .attributes
                .sanity
                .saturating_sub(*sanity_loss);
            format!(
                "{} thrashes in their sleep, tormented by dark visions. Loses {} sanity.",
                tribute.name, sanity_loss
            )
        }
```

Add `NightTerror` branch:

```rust
        SleepIncident::NightTerror { sanity_loss } => {
            tribute.attributes.sanity = tribute
                .attributes
                .sanity
                .saturating_sub(*sanity_loss);
            // Phobia trigger content is handled by the cycle (wake emission);
            // the apply function handles the mechanical effects.
            format!(
                "{} bolts upright with a scream, heart pounding from a night terror! Loses {} sanity.",
                tribute.name, sanity_loss
            )
        }
```

Remove the `AllyAbandonment` branch entirely. Replace it with a stub that produces flavor text and does NO ally removal (the cycle handles removal):

```rust
        SleepIncident::AllyAbandonment => {
            // Ally removal is handled by the cycle with full game-state
            // access. The apply function only provides flavor text.
            format!(
                "An ally silently slips away into the night, abandoning {}.",
                tribute.name
            )
        }
```

- [ ] **Step 3: Update `apply_hallucination_reduces_sanity` test**

Rename to `apply_nightmare_reduces_sanity` and update the incident variant:

```rust
    #[test]
    fn legacy_hallucination_test_mapped_to_nightmare() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.attributes.sanity;
        let incident = SleepIncident::Nightmare { sanity_loss: 5 };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(desc.contains("sanity"));
        assert!(tribute.attributes.sanity < orig_sanity);
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test --package game tributes::incidents`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): update apply_sleep_incident for Nightmare/NightTerror, remove ally removal

- Nightmare branch: applies sanity_loss (no wake)
- NightTerror branch: applies sanity_loss (wake emitted by cycle)
- AllyAbandonment branch: flavor text only — removal moved to cycle
- Old Hallucination test updated to Nightmare

Refs: hangrier_games-pending"
```

---

## Task 7: `sleep_shelter` field on `Tribute`

**Why seventh:** The cycle stores the shelter roll result here. Must land before cycle integration.

**Files:**
- Modify: `game/src/tributes/mod.rs`

- [ ] **Step 1: Write failing test**

Append to the existing tests in `tributes/mod.rs` test module:

```rust
#[test]
fn tribute_sleep_shelter_defaults_to_none() {
    let t = Tribute::new("Test".to_string(), None, None);
    assert_eq!(t.sleep_shelter, None);
}
```

Or use the existing `tribute_default_fields` test (around line 1868) style:

```rust
#[test]
fn tribute_sleep_shelter_serde_skips_field() {
    let t = Tribute::new("Test".to_string(), None, None);
    let json = serde_json::to_string(&t).unwrap();
    assert!(!json.contains("sleep_shelter"), "sleep_shelter should not serialize");
}
```

- [ ] **Step 2: Add field to `Tribute` struct**

In the `Tribute` struct definition (around line 286, near `sleep_remaining`), add:

```rust
    /// Sleep shelter rolled at start of sleep session (transient).
    /// Cleared on wake or phase change. Not persisted.
    #[serde(skip)]
    pub sleep_shelter: Option<crate::tributes::incidents::SleepShelter>,
```

- [ ] **Step 3: Initialize in `Tribute::new`**

In `Tribute::new` (around line 384), add:

```rust
            sleep_shelter: None,
```

Also add near any `sleeping = false; sleep_remaining = 0;` blocks to clear it on wake:

Actually, that clearing happens in the cycle. For the constructor, just set to None.

- [ ] **Step 4: Run tests**

Run: `cargo test --package game tributes::tests::tribute_sleep_shelter_defaults_to_none`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(tribute): add transient sleep_shelter field

sleep_shelter: Option<SleepShelter> with #[serde(skip)] — never
persisted. Set by cycle at sleep start via find_shelter(), cleared
on wake or phase change.

Refs: hangrier_games-pending"
```

---

## Task 8: `InterruptionKind::NightTerror` + `SleepIncidentKind` alignment in shared

**Why eighth:** The cycle needs `InterruptionKind::NightTerror` for the wake emission path. Also align `SleepIncidentKind` with the renamed/added variants.

**Files:**
- Modify: `shared/src/messages/mod.rs`

- [ ] **Step 1: Write failing tests**

Append to the existing test module in `shared/src/messages/tests.rs`:

```rust
#[test]
fn night_terror_interruption_round_trips() {
    let kind = InterruptionKind::NightTerror;
    let json = serde_json::to_string(&kind).unwrap();
    let back: InterruptionKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

#[test]
fn sleep_incident_kind_nightmare_round_trips() {
    let kind = SleepIncidentKind::Nightmare;
    let json = serde_json::to_string(&kind).unwrap();
    let back: SleepIncidentKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}
```

- [ ] **Step 2: Add `NightTerror` variant to `InterruptionKind`**

After `Incident { kind: SleepIncidentKind }` (line 290), add:

```rust
    /// The tribute woke from a night terror (phobia-driven nightmare).
    NightTerror,
```

- [ ] **Step 3: Update `SleepIncidentKind` enum**

Rename `Hallucination` → `Nightmare`:

```rust
    /// Nightmare — sanity damage, no wake.
    Nightmare,
```

Add `NightTerror`:

```rust
    /// Intense night terror — sanity damage, wakes, phobia trigger.
    NightTerror,
```

- [ ] **Step 4: Update any exhaustive match arms on `SleepIncidentKind` or `InterruptionKind`**

Search for exhaustive matches:

```bash
grep -rn "SleepIncidentKind::" shared/ game/ web/ api/ | grep -v ".test"
```

Common locations:
- `cycle.rs:568` — `SleepIncidentKind::Hallucination =>`
- `incidents.rs:64` — via `From<&SleepIncident>` impl (already updated in Task 1)
- `shared/src/messages/tests.rs` — test assertions

In `cycle.rs:568` (the natural wake incident_suffix block), update:

```rust
                            SleepIncidentKind::Nightmare => {
                                " — still shaken by dark dreams".to_string()
                            }
                            SleepIncidentKind::NightTerror => {
                                " — haunted by a night terror".to_string()
                            }
```

Add a catch-all or handle both.

- [ ] **Step 5: Run shared tests**

Run: `cargo test --package shared`
Expected: PASS.

- [ ] **Step 6: Update cycle.rs reference to `SleepIncidentKind::Hallucination`**

The cycle.rs line 568 references `SleepIncidentKind::Hallucination`. Change it to `SleepIncidentKind::Nightmare` and add `SleepIncidentKind::NightTerror`:

```rust
                        .map(|kind| match kind {
                            SleepIncidentKind::Nightmare => {
                                " — still shaken by dark dreams".to_string()
                            }
                            SleepIncidentKind::NightTerror => {
                                " — haunted by a night terror".to_string()
                            }
                            _ => " — though their sleep was restless".to_string(),
                        })
```

- [ ] **Step 7: Run `cargo check --workspace`**

Expected: clean (all exhaustive matches satisfied).

- [ ] **Step 8: Commit**

```bash
jj describe -m "feat(shared): add NightTerror variant to InterruptionKind + SleepIncidentKind

- InterruptionKind::NightTerror variant for wake emission
- SleepIncidentKind::Hallucination renamed to Nightmare
- SleepIncidentKind::NightTerror variant added
- cycle.rs natural wake suffixes updated for both variants

Refs: hangrier_games-pending"
```

---

## Task 9: Cycle integration — shelter roll, incident roll update, AllyAbandonment, NightTerror wake, shelter clear

**Why ninth:** The core integration. All the pieces from Tasks 1-8 wire together in the sleep section of `cycle.rs`.

**Files:**
- Modify: `game/src/games/cycle.rs`

- [ ] **Step 1: Locate and understand the sleep section**

The sleep section starts at line 418 with `if tribute.sleeping {`. Within it:
- Lines 459-523: Incident roll + apply + wake/non-wake branching
- Lines 564-573: Natural wake suffix using pending_sleep_incident

- [ ] **Step 2: Add shelter roll at sleep start**

Before the incident roll (around line 453, after the area event check and before the incident roll comment), add:

```rust
                // ── Sleep shelter roll (spec §2.4) ──
                // On the first sleeping phase, roll for shelter. Clear any
                // previous shelter (from interrupted sleep).
                // Shelter persists for the duration of this sleep session.
                if tribute.sleep_shelter.is_none() {
                    let terrain = area_details_map
                        .get(&tribute.area)
                        .and_then(|&idx| self.areas.get(idx))
                        .map(|a| a.terrain.base);
                    if let Some(base_terrain) = terrain {
                        tribute.sleep_shelter = Some(crate::tributes::incidents::find_shelter(
                            tribute.attributes.intelligence,
                            tribute.attributes.strength,
                            base_terrain,
                            rng,
                        ));
                    } else {
                        tribute.sleep_shelter = Some(crate::tributes::incidents::SleepShelter::None);
                    }
                }
```

- [ ] **Step 3: Update the incident roll call site (line 459)**

Replace:
```rust
                if let Some(incident) = SleepIncident::roll(rng) {
```
with:
```rust
                // Determine biome, shelter status, and sleep_shelter for
                // the phase-aware, biome-aware, shelter-aware roll.
                let terrain = area_details_map
                    .get(&tribute.area)
                    .and_then(|&idx| self.areas.get(idx))
                    .map(|a| a.terrain.base)
                    .unwrap_or(crate::terrain::types::BaseTerrain::Forest); // safe fallback
                let is_sheltered = tribute
                    .sheltered_until
                    .is_some_and(|p| p > phase_index);
                let sleep_shelter = tribute
                    .sleep_shelter
                    .as_ref()
                    .unwrap_or(&crate::tributes::incidents::SleepShelter::None);

                if let Some(incident) = SleepIncident::roll(
                    rng,
                    phase,
                    terrain,
                    is_sheltered,
                    sleep_shelter,
                    current_day,
                ) {
```

Note: `phase_index` is computed at line 163: `let phase_index: u32 = self.day.unwrap_or(1) * 2 + u32::from(!day);`. Confirm it's in scope at line 459. If not, compute it inline: `let phase_index = self.day.unwrap_or(1) * 4 + phase.ord() as u32;`

- [ ] **Step 4: Add NightTerror wake emission path**

In the `if incident.wakes_tribute()` block (line 463), add a special case for NightTerror:

After the existing wake emission (around line 501, before `tribute.sleeping = false`), insert:

```rust
                        // Special handling for NightTerror: emit with
                        // InterruptionKind::NightTerror for the wire format.
                        if matches!(incident, SleepIncident::NightTerror { .. }) {
                            // Replace the last-incident-based wake reason
                            // with the specific NightTerror interruption.
                            // We need to pop the previous TributeWoke event
                            // and replace it. Practical approach: don't push
                            // the generic one — handle NightTerror separately.
                        }
```

Actually, this is cleaner: restructure the wake emission block at lines 478-501 to branch on NightTerror vs other incidents:

```rust
                    if incident.wakes_tribute() {
                        let flavor_line = crate::output::GameOutput::TributeSleepFlavor(
                            tribute.name.as_str(),
                            &description,
                        )
                        .to_string();
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            flavor_line,
                            None,
                            None,
                        ));

                        // NightTerror gets a specific InterruptionKind variant.
                        let interrupt_kind = if matches!(incident, SleepIncident::NightTerror { .. }) {
                            shared::messages::InterruptionKind::NightTerror
                        } else {
                            shared::messages::InterruptionKind::Incident {
                                kind: incident_kind.clone(),
                            }
                        };

                        let incident_msg = crate::output::GameOutput::TributeWakesFromIncident(
                            tribute.name.as_str(),
                            &description,
                        )
                        .to_string();
                        collected_events.push((
                            tribute.identifier.clone(),
                            tribute.name.clone(),
                            incident_msg,
                            Some(MessagePayload::TributeWoke {
                                tribute: TributeRef {
                                    identifier: tribute.identifier.clone(),
                                    name: tribute.name.clone(),
                                },
                                phase,
                                reason: shared::messages::WakeReason::Interrupted {
                                    event: interrupt_kind,
                                },
                            }),
                            None,
                        ));
                        tribute.sleeping = false;
                        tribute.sleep_remaining = 0;
                        tribute.cycles_awake = 0;
                        tribute.pending_sleep_incident = None;
                        tribute.sleep_shelter = None; // clear shelter
                        continue;
                    } else {
```

Note: `incident_kind` is a clone — ensure the `let incident_kind` at line 461 is changed to `let incident_kind = shared::messages::SleepIncidentKind::from(&incident);` and used as a clone if needed. If it's moved into the `InterruptionKind::Incident { kind: incident_kind }` constructor, clone it first.

- [ ] **Stage 5: Implement AllyAbandonment in the cycle**

After the incident has been rolled and applied, handle AllyAbandonment specifically (it no longer wakes but the cycle must perform the ally removal):

Inside the `else` branch (non-waking incidents, line 507), add after the flavor event push:

```rust
                        // AllyAbandonment: the cycle handles removal with
                        // full game-state access (spec §2.6).
                        if matches!(incident, SleepIncident::AllyAbandonment) {
                            // Find a qualified ally (living, awake, same area).
                            let qualified: Vec<usize> = self
                                .tributes
                                .iter()
                                .enumerate()
                                .filter(|(_, t)| {
                                    tribute.allies.contains(&t.id)
                                        && t.is_alive()
                                        && !t.sleeping
                                        && t.area == tribute.area
                                })
                                .map(|(i, _)| i)
                                .collect();

                            if let Some(&idx) = qualified.first() {
                                let abandoning_ally = &self.tributes[idx];
                                // Remove from sleeper's ally list.
                                tribute
                                    .allies
                                    .retain(|id| *id != abandoning_ally.id);
                                // Remove from abandoner's ally list.
                                // We need a mutable reference — this works
                                // because we're inside a block that borrows
                                // self.tributes immutably above; the removal
                                // from tribute (mutable) works since we use
                                // separate borrows.
                                // SAFETY: the qualified ally is a different
                                // element than tribute (borrow check).
                                if let Some(abandoner) = self
                                    .tributes
                                    .iter_mut()
                                    .find(|t| t.id == abandoning_ally.id)
                                {
                                    abandoner.allies.retain(|id| *id != tribute.id);
                                }

                                // Mental effects on sleeper (sanity loss).
                                tribute.attributes.sanity = tribute
                                    .attributes
                                    .sanity
                                    .saturating_sub(3); // betrayal trauma
                            }
                        }
```

Wait, this won't work — we can't borrow `self.tributes` mutably while we already have a `&mut` borrow on `tribute` (from the outer `iter_mut()` loop). Need a two-phase approach: collect the IDs first, then remove in a second pass.

Let me restructure:

```rust
                        // AllyAbandonment: the cycle handles removal with
                        // full game-state access (spec §2.6).
                        // Two-phase: collect IDs first, then mutate.
                        if matches!(incident, SleepIncident::AllyAbandonment) {
                            // Find the first qualified ally (living, awake, same area).
                            let abandoning_id: Option<uuid::Uuid> = self
                                .tributes
                                .iter()
                                .find(|t| {
                                    tribute.allies.contains(&t.id)
                                        && t.is_alive()
                                        && !t.sleeping
                                        && t.area == tribute.area
                                })
                                .map(|t| t.id);

                            if let Some(abandoner_id) = abandoning_id {
                                // Remove from sleeper's ally list.
                                tribute.allies.retain(|id| *id != abandoner_id);
                                // Mental effects on sleeper.
                                tribute.attributes.sanity = tribute
                                    .attributes
                                    .sanity
                                    .saturating_sub(3);

                                // Remove from abandoner's ally list (second pass
                                // via self.tributes — safe because tribute is a
                                // separate element).
                                for t in self.tributes.iter_mut() {
                                    if t.id == abandoner_id {
                                        t.allies.retain(|id| *id != tribute.id);
                                        break;
                                    }
                                }
                            }
                        }
```

This is the correct approach — the first pass is an immutable borrow of `self.tributes` to find the ID, then we mutate `tribute` (already mutably borrowed from the outer `iter_mut()`) and do a second pass to mutate the abandoner.

- [ ] **Step 6: Clear `sleep_shelter` on wake or phase change**

In the natural wake block (line 557-580), add `tribute.sleep_shelter = None;` after `tribute.sleeping = false;`:

```rust
                    tribute.sleeping = false;
                    tribute.cycles_awake = 0;
                    tribute.sleep_shelter = None; // clear shelter
```

Also in the wake-causing incident block (line 502-506), we already added it in Step 4.

- [ ] **Step 7: Update the wake suffix block for Nightmare/NightTerror**

The natural wake suffix (line 564-573) already uses `SleepIncidentKind::Nightmare` and `SleepIncidentKind::NightTerror` from Task 8.

- [ ] **Step 8: Run tests**

Run: `cargo test --package game`
Expected: PASS (existing cycle tests should still pass; the new shelter/incident logic operates on additional parameters with defaults).

- [ ] **Step 9: Run `cargo check --workspace`**

Expected: clean.

- [ ] **Step 10: Commit**

```bash
jj describe -m "feat(game): integrate sleep shelter, phase/biome-aware roll, AllyAbandonment, NightTerror wake

- Shelter roll at sleep start via find_shelter() stored in tribute.sleep_shelter
- Incident roll passes phase, terrain, shelter status, sleep_shelter, day
- NightTerror uses InterruptionKind::NightTerror for wake emission
- AllyAbandonment handled in cycle with full ally validation + mental effects
- sleep_shelter cleared on wake or phase change
- Natural wake suffixes aligned with Nightmare/NightTerror variants

Refs: hangrier_games-pending"
```

---

## Task 10: Integration tests for new functionality

**Why tenth:** Tests for the new functions and the updated cycle behavior. Includes tests for phase-aware roll, biome animal selection, find_shelter, shelter modifiers, and the Nightmare/NightTerror apply effects.

**Files:**
- Modify: `game/src/tributes/incidents.rs` (add tests to existing test module)

- [ ] **Step 1: Phase-aware roll distribution test**

```rust
    #[rstest]
    fn night_phase_more_incidents_than_day_phase() {
        // With identical seed and biome, Night phase should produce more
        // incidents than Day phase over a large sample.
        let mut day_rng = SmallRng::seed_from_u64(42);
        let mut night_rng = SmallRng::seed_from_u64(42);
        let mut day_count = 0;
        let mut night_count = 0;
        let samples = 500;
        for _ in 0..samples {
            if SleepIncident::roll(
                &mut day_rng,
                crate::messages::Phase::Day,
                crate::terrain::types::BaseTerrain::Desert,
                false,
                &SleepShelter::None,
                1,
            )
            .is_some()
            {
                day_count += 1;
            }
            if SleepIncident::roll(
                &mut night_rng,
                crate::messages::Phase::Night,
                crate::terrain::types::BaseTerrain::Desert,
                false,
                &SleepShelter::None,
                1,
            )
            .is_some()
            {
                night_count += 1;
            }
        }
        assert!(
            night_count > day_count,
            "expected night ({night_count}) > day ({day_count}) incidents"
        );
    }
```

- [ ] **Step 2: Biome animal selection test**

```rust
    #[test]
    fn animal_encounter_draws_from_biome_pool() {
        let mut rng = SmallRng::seed_from_u64(42);
        let incident = SleepIncident::random(&mut rng, crate::terrain::types::BaseTerrain::Desert);
        if let SleepIncident::AnimalEncounter { animal } = &incident {
            let pool = biome_animal_pool(crate::terrain::types::BaseTerrain::Desert);
            assert!(
                pool.contains(&animal.as_str()),
                "desert animal '{animal}' not in desert pool"
            );
        }
    }
```

- [ ] **Step 3: Shelter modifier effective chance tests**

```rust
    #[rstest]
    #[case(SleepShelter::None, 22.0)]   // Night * 1.0 * 1.0
    #[case(SleepShelter::Crude, 17.6)]  // Night * 0.8 * 1.0
    #[case(SleepShelter::Natural, 11.0)] // Night * 0.5 * 1.0
    #[case(SleepShelter::Fortified, 6.6)] // Night * 0.3 * 1.0
    fn effective_chance_with_sleep_shelter(#[case] shelter: SleepShelter, #[case] expected: f64) {
        let chance = effective_incident_chance(
            crate::messages::Phase::Night,
            crate::terrain::types::BaseTerrain::Desert,
            false,
            &shelter,
            1,
        );
        assert!((chance - expected).abs() < 0.1, "expected {expected}, got {chance}");
    }
```

- [ ] **Step 4: find_shelter distribution test**

```rust
    #[test]
    fn find_shelter_distribution_over_many_rolls() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut counts = [0usize; 4]; // None, Crude, Natural, Fortified
        for _ in 0..1000 {
            let result = find_shelter(50, 50, crate::terrain::types::BaseTerrain::Forest, &mut rng);
            match result {
                SleepShelter::None => counts[0] += 1,
                SleepShelter::Crude => counts[1] += 1,
                SleepShelter::Natural => counts[2] += 1,
                SleepShelter::Fortified => counts[3] += 1,
            }
        }
        // With mid stats in Forest (shelter_quality 2), expect some of each.
        // None of the four should be zero.
        for (i, count) in counts.iter().enumerate() {
            let name = ["None", "Crude", "Natural", "Fortified"][i];
            assert!(*count > 0, "{name} should appear at least once in 1000 rolls");
        }
    }
```

- [ ] **Step 5: Nightmare sanity loss range test**

```rust
    #[test]
    fn nightmare_sanity_loss_in_range() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut losses = Vec::new();
        for _ in 0..100 {
            if let SleepIncident::Nightmare { sanity_loss } =
                SleepIncident::random(&mut rng, crate::terrain::types::BaseTerrain::Forest)
            {
                losses.push(sanity_loss);
            }
        }
        for loss in losses {
            assert!(
                (2..=6).contains(&loss),
                "Nightmare sanity loss {loss} out of range 2-6"
            );
        }
    }
```

- [ ] **Step 6: Run all incident tests**

Run: `cargo test --package game tributes::incidents`
Expected: PASS — all ~20+ tests green.

- [ ] **Step 7: Run full workspace check + test**

Run: `just fmt && cargo check --workspace && cargo test --package game`
Expected: All green.

- [ ] **Step 8: Commit**

```bash
jj describe -m "test(game): add integration tests for phase-aware roll, biome pools, find_shelter, shelter modifiers, Nightmare/NightTerror

- Phase distribution: Night > Day incident rate
- Biome animal pool membership
- SleepShelter multiplier effective chance table
- find_shelter distribution across all tiers
- Nightmare sanity loss range (2-6)

Refs: hangrier_games-pending"
```

---

## Session Completion

When all tasks above are done, follow the standard session-completion protocol:

1. **File issues for remaining work** — create beads issues for PR2 (frontend), brain scoring for AllyAbandonment, post-observability tuning.
2. **Run quality gates** — `just fmt && cargo check --workspace && cargo clippy --workspace -- -D warnings && cargo test --package game`
3. **Update issue status** — close the working bead issue.
4. **Open PR** — create feature bookmark, push, and open PR on GitHub.
5. **Verify** — PR URL in hand, all CI passes locally.
