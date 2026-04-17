# Terrain/Biome System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a terrain/biome system that makes each arena region feel distinct through movement costs, event probabilities, item distributions, AI behavior, and rich narrative descriptions.

**Architecture:** Pure functional core in `game/` crate with terrain logic isolated in new `game/src/terrain/` module. API layer (`api/`) handles database persistence and user-facing customization. Frontend (`web/`) displays terrain information and customization options.

**Tech Stack:** Rust (game engine), Axum (API), Dioxus (frontend), SurrealDB (database)

---

## File Structure

### New Files (Create)

**Game crate (`game/src/`):**
- `terrain/mod.rs` - Module root, re-exports
- `terrain/types.rs` - TerrainType, BaseTerrain, TerrainDescriptor enums
- `terrain/config.rs` - Const terrain configurations (movement costs, visibility, harshness, item weights)
- `terrain/assignment.rs` - Random terrain generation with balance constraints
- `terrain/descriptors.rs` - Descriptor compatibility validation
- `districts.rs` - District profiles with terrain affinities

**Test files:**
- `game/tests/terrain_compatibility_test.rs` - Descriptor validation tests
- `game/tests/stamina_edge_cases_test.rs` - Zero stamina, cost > pool tests
- `game/tests/event_severity_test.rs` - Event-terrain severity calculation tests
- `game/tests/district_affinity_test.rs` - Affinity assignment tests
- `game/tests/desperation_test.rs` - Desperation mechanics tests
- `api/tests/game_customization_test.rs` - CreateGame DTO integration tests

### Modified Files

**Game crate:**
- `game/src/areas/mod.rs` - Add terrain field to Area struct
- `game/src/events/mod.rs` - Add new AreaEvent variants, severity calculation
- `game/src/tributes/mod.rs` - Add stamina, max_stamina, terrain_affinity fields
- `game/src/tributes/actions.rs` - Replace movement with stamina-based system
- `game/src/tributes/brain.rs` - Add terrain-aware AI decision-making
- `game/src/items/mod.rs` - Add terrain-based item spawning logic
- `game/src/messages.rs` - Add rich terrain-aware narrative generation

**Shared crate:**
- `shared/src/lib.rs` - Extend CreateGame DTO with customization options

**API crate:**
- `api/src/games.rs` - Apply customization options, assign terrains
- `api/src/areas.rs` - Persist terrain data
- `api/src/tributes.rs` - Persist stamina and affinity data

**Database:**
- `schemas/areas.surql` - Add base_terrain, terrain_descriptors fields
- `schemas/tributes.surql` - Add terrain_affinity, stamina, max_stamina fields

**Frontend:**
- `web/src/components/game/create_game_modal.rs` - Add customization dropdowns
- `web/src/components/game/area_card.rs` - Display terrain type
- `web/src/components/tributes/tribute_card.rs` - Display affinity badges

---

## Task 1: Core Terrain Data Structures

**Files:**
- Create: `game/src/terrain/mod.rs`
- Create: `game/src/terrain/types.rs`
- Create: `game/src/terrain/descriptors.rs`
- Test: `game/tests/terrain_compatibility_test.rs`

- [ ] **Step 1: Create terrain module root**

```rust
// game/src/terrain/mod.rs
pub mod types;
pub mod config;
pub mod assignment;
pub mod descriptors;

pub use types::{BaseTerrain, TerrainDescriptor, TerrainType};
pub use config::{Visibility, Harshness};
```

- [ ] **Step 2: Define base terrain and descriptor enums**

```rust
// game/src/terrain/types.rs
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter)]
pub enum BaseTerrain {
    Clearing,
    Forest,
    Desert,
    Tundra,
    Wetlands,
    Mountains,
    UrbanRuins,
    Jungle,
    Grasslands,
    Badlands,
    Highlands,
    Geothermal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainDescriptor {
    // Temperature
    Hot,
    Cold,
    Temperate,
    // Density/Structure
    Dense,
    Sparse,
    Open,
    // Moisture
    Wet,
    Dry,
    // Altitude
    HighAltitude,
    Lowland,
    // Condition
    Rocky,
    Sandy,
    Frozen,
    Overgrown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainType {
    pub base: BaseTerrain,
    pub descriptors: Vec<TerrainDescriptor>,
}

impl TerrainType {
    pub fn new(base: BaseTerrain, descriptors: Vec<TerrainDescriptor>) -> Result<Self, String> {
        // Validate descriptor compatibility
        for descriptor in &descriptors {
            if !Self::is_compatible(&base, descriptor) {
                return Err(format!(
                    "{:?} cannot have {:?} descriptor",
                    base, descriptor
                ));
            }
        }
        
        Ok(TerrainType { base, descriptors })
    }
    
    fn is_compatible(base: &BaseTerrain, descriptor: &TerrainDescriptor) -> bool {
        use BaseTerrain::*;
        use TerrainDescriptor::*;
        
        match (base, descriptor) {
            // Desert cannot be Wet (except during temporary Flood events)
            (Desert, Wet) => false,
            // Tundra must be Cold or Frozen
            (Tundra, Hot) => false,
            (Tundra, Temperate) => false,
            // Geothermal must be Hot
            (Geothermal, Cold) => false,
            (Geothermal, Frozen) => false,
            // Otherwise compatible
            _ => true,
        }
    }
}

impl BaseTerrain {
    pub fn descriptive_name(&self) -> &'static str {
        match self {
            BaseTerrain::Clearing => "clearing",
            BaseTerrain::Forest => "forest",
            BaseTerrain::Desert => "desert",
            BaseTerrain::Tundra => "tundra",
            BaseTerrain::Wetlands => "wetlands",
            BaseTerrain::Mountains => "mountains",
            BaseTerrain::UrbanRuins => "urban ruins",
            BaseTerrain::Jungle => "jungle",
            BaseTerrain::Grasslands => "grasslands",
            BaseTerrain::Badlands => "badlands",
            BaseTerrain::Highlands => "highlands",
            BaseTerrain::Geothermal => "geothermal area",
        }
    }
}

impl TerrainDescriptor {
    pub fn as_adjective(&self) -> &'static str {
        match self {
            TerrainDescriptor::Hot => "hot",
            TerrainDescriptor::Cold => "cold",
            TerrainDescriptor::Temperate => "temperate",
            TerrainDescriptor::Dense => "dense",
            TerrainDescriptor::Sparse => "sparse",
            TerrainDescriptor::Open => "open",
            TerrainDescriptor::Wet => "wet",
            TerrainDescriptor::Dry => "dry",
            TerrainDescriptor::HighAltitude => "high-altitude",
            TerrainDescriptor::Lowland => "lowland",
            TerrainDescriptor::Rocky => "rocky",
            TerrainDescriptor::Sandy => "sandy",
            TerrainDescriptor::Frozen => "frozen",
            TerrainDescriptor::Overgrown => "overgrown",
        }
    }
}
```

- [ ] **Step 3: Write descriptor validation tests**

```rust
// game/tests/terrain_compatibility_test.rs
use game::terrain::{BaseTerrain, TerrainDescriptor, TerrainType};

#[test]
fn test_desert_cannot_be_wet() {
    let result = TerrainType::new(BaseTerrain::Desert, vec![TerrainDescriptor::Wet]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot have Wet descriptor"));
}

#[test]
fn test_tundra_cannot_be_hot() {
    let result = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Hot]);
    assert!(result.is_err());
}

#[test]
fn test_tundra_must_be_cold_or_frozen() {
    let cold_tundra = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Cold]);
    assert!(cold_tundra.is_ok());
    
    let frozen_tundra = TerrainType::new(BaseTerrain::Tundra, vec![TerrainDescriptor::Frozen]);
    assert!(frozen_tundra.is_ok());
}

#[test]
fn test_geothermal_must_be_hot() {
    let cold_geo = TerrainType::new(BaseTerrain::Geothermal, vec![TerrainDescriptor::Cold]);
    assert!(cold_geo.is_err());
    
    let hot_geo = TerrainType::new(BaseTerrain::Geothermal, vec![TerrainDescriptor::Hot]);
    assert!(hot_geo.is_ok());
}

#[test]
fn test_forest_can_be_wet() {
    let rainforest = TerrainType::new(
        BaseTerrain::Forest,
        vec![TerrainDescriptor::Wet, TerrainDescriptor::Dense],
    );
    assert!(rainforest.is_ok());
}

#[test]
fn test_empty_descriptors_valid() {
    let plain_clearing = TerrainType::new(BaseTerrain::Clearing, vec![]);
    assert!(plain_clearing.is_ok());
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game terrain_compatibility`

Expected: All tests PASS

- [ ] **Step 5: Add terrain module to game crate**

```rust
// game/src/lib.rs
pub mod terrain;  // Add this line with other module declarations

// Re-export key terrain types
pub use terrain::{BaseTerrain, TerrainDescriptor, TerrainType};
```

- [ ] **Step 6: Commit**

```bash
git add game/src/terrain/ game/tests/terrain_compatibility_test.rs game/src/lib.rs
git commit -m "feat(game): add core terrain data structures with validation"
```

---

## Task 2: Terrain Configuration Constants

**Files:**
- Create: `game/src/terrain/config.rs`
- Test: `game/tests/terrain_config_test.rs`

- [ ] **Step 1: Define visibility and harshness enums**

```rust
// game/src/terrain/config.rs
use serde::{Deserialize, Serialize};
use crate::terrain::BaseTerrain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    Exposed,    // Tundra, Desert, Grasslands - hard to hide
    Moderate,   // Clearing, Highlands, Wetlands
    Concealed,  // Forest, Jungle, UrbanRuins - easy to hide
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Harshness {
    Mild,       // Clearing, Grasslands
    Moderate,   // Forest, Jungle, UrbanRuins, Wetlands, Highlands, Geothermal
    Harsh,      // Desert, Tundra, Mountains, Badlands
}

#[derive(Debug, Clone, Copy)]
pub struct ItemWeights {
    pub weapons: f32,
    pub shields: f32,
    pub consumables: f32,
}

impl BaseTerrain {
    pub const fn movement_cost(&self) -> f32 {
        match self {
            BaseTerrain::Clearing => 1.0,
            BaseTerrain::Grasslands => 0.9,
            BaseTerrain::UrbanRuins => 1.2,
            BaseTerrain::Forest => 1.3,
            BaseTerrain::Jungle => 1.4,
            BaseTerrain::Geothermal => 1.4,
            BaseTerrain::Wetlands => 1.5,
            BaseTerrain::Highlands => 1.6,
            BaseTerrain::Badlands => 1.7,
            BaseTerrain::Mountains => 1.8,
            BaseTerrain::Desert => 2.0,
            BaseTerrain::Tundra => 2.0,
        }
    }
    
    pub const fn visibility(&self) -> Visibility {
        match self {
            BaseTerrain::Forest | BaseTerrain::Jungle | BaseTerrain::UrbanRuins => {
                Visibility::Concealed
            }
            BaseTerrain::Desert | BaseTerrain::Tundra | BaseTerrain::Grasslands | BaseTerrain::Badlands => {
                Visibility::Exposed
            }
            _ => Visibility::Moderate,
        }
    }
    
    pub const fn harshness(&self) -> Harshness {
        match self {
            BaseTerrain::Clearing | BaseTerrain::Grasslands => Harshness::Mild,
            BaseTerrain::Desert | BaseTerrain::Tundra | BaseTerrain::Mountains | BaseTerrain::Badlands => {
                Harshness::Harsh
            }
            _ => Harshness::Moderate,
        }
    }
    
    pub const fn item_spawn_modifier(&self) -> f32 {
        match self {
            BaseTerrain::Clearing => 1.0,
            BaseTerrain::Jungle => 1.0,
            BaseTerrain::Forest => 1.1,
            BaseTerrain::Grasslands => 1.1,
            BaseTerrain::UrbanRuins => 1.2,
            BaseTerrain::Wetlands => 0.9,
            BaseTerrain::Highlands => 0.8,
            BaseTerrain::Geothermal => 0.8,
            BaseTerrain::Mountains => 0.7,
            BaseTerrain::Badlands => 0.7,
            BaseTerrain::Desert => 0.6,
            BaseTerrain::Tundra => 0.6,
        }
    }
    
    pub const fn item_weights(&self) -> ItemWeights {
        match self {
            BaseTerrain::Desert => ItemWeights {
                weapons: 0.2,
                shields: 0.2,
                consumables: 0.6,
            },
            BaseTerrain::Tundra => ItemWeights {
                weapons: 0.3,
                shields: 0.4,
                consumables: 0.3,
            },
            BaseTerrain::UrbanRuins => ItemWeights {
                weapons: 0.5,
                shields: 0.3,
                consumables: 0.2,
            },
            BaseTerrain::Forest => ItemWeights {
                weapons: 0.3,
                shields: 0.2,
                consumables: 0.5,
            },
            BaseTerrain::Mountains => ItemWeights {
                weapons: 0.4,
                shields: 0.4,
                consumables: 0.2,
            },
            BaseTerrain::Wetlands => ItemWeights {
                weapons: 0.25,
                shields: 0.25,
                consumables: 0.5,
            },
            BaseTerrain::Jungle => ItemWeights {
                weapons: 0.2,
                shields: 0.3,
                consumables: 0.5,
            },
            BaseTerrain::Clearing => ItemWeights {
                weapons: 0.33,
                shields: 0.33,
                consumables: 0.34,
            },
            BaseTerrain::Grasslands => ItemWeights {
                weapons: 0.30,
                shields: 0.30,
                consumables: 0.40,
            },
            BaseTerrain::Badlands => ItemWeights {
                weapons: 0.35,
                shields: 0.30,
                consumables: 0.35,
            },
            BaseTerrain::Highlands => ItemWeights {
                weapons: 0.30,
                shields: 0.35,
                consumables: 0.35,
            },
            BaseTerrain::Geothermal => ItemWeights {
                weapons: 0.30,
                shields: 0.30,
                consumables: 0.40,
            },
        }
    }
}
```

- [ ] **Step 2: Write configuration tests**

```rust
// game/tests/terrain_config_test.rs
use game::terrain::{BaseTerrain, Visibility, Harshness};

#[test]
fn test_movement_costs_within_range() {
    for terrain in BaseTerrain::iter() {
        let cost = terrain.movement_cost();
        assert!(cost >= 0.5 && cost <= 3.0, "Movement cost out of range for {:?}", terrain);
    }
}

#[test]
fn test_clearing_is_mild() {
    assert_eq!(BaseTerrain::Clearing.harshness(), Harshness::Mild);
}

#[test]
fn test_tundra_is_harsh() {
    assert_eq!(BaseTerrain::Tundra.harshness(), Harshness::Harsh);
}

#[test]
fn test_forest_is_concealed() {
    assert_eq!(BaseTerrain::Forest.visibility(), Visibility::Concealed);
}

#[test]
fn test_desert_is_exposed() {
    assert_eq!(BaseTerrain::Desert.visibility(), Visibility::Exposed);
}

#[test]
fn test_item_weights_sum_to_one() {
    for terrain in BaseTerrain::iter() {
        let weights = terrain.item_weights();
        let sum = weights.weapons + weights.shields + weights.consumables;
        assert!((sum - 1.0).abs() < 0.01, "Item weights don't sum to 1.0 for {:?}: {}", terrain, sum);
    }
}

#[test]
fn test_item_spawn_modifiers_reasonable() {
    for terrain in BaseTerrain::iter() {
        let modifier = terrain.item_spawn_modifier();
        assert!(modifier >= 0.5 && modifier <= 1.5, "Item modifier out of range for {:?}", terrain);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --package game terrain_config`

Expected: All tests PASS

- [ ] **Step 4: Commit**

```bash
git add game/src/terrain/config.rs game/tests/terrain_config_test.rs
git commit -m "feat(game): add terrain configuration constants with const fn lookups"
```

---

## Task 3: Random Terrain Assignment with Balance Constraints

**Files:**
- Create: `game/src/terrain/assignment.rs`
- Create: `game/src/terrain/descriptors.rs`
- Test: `game/tests/terrain_assignment_test.rs`

**Summary:** Implement terrain generation that assigns random terrains to areas with safety constraints (no all-Deadly games, ensure variety). Generate compatible descriptors based on terrain type.

**Key steps:**
- Write `assign_random_terrain()` function
- Add descriptor generation with compatibility checks
- Implement balance constraint: max 3 Harsh terrains per game
- Write tests for constraint enforcement
- Commit

---

## Task 4: District Profiles & Tribute Affinity

**Files:**
- Create: `game/src/districts.rs`
- Modify: `game/src/tributes/mod.rs` (add terrain_affinity field)
- Test: `game/tests/district_affinity_test.rs`

**Summary:** Define district profiles with primary and bonus terrain affinities. Assign terrain affinity to tributes based on district with 40% bonus chance.

**Key steps:**
- Create DistrictProfile struct with all 12 districts
- Add terrain_affinity: Vec<BaseTerrain> to Tribute struct
- Implement `assign_terrain_affinity()` using district profiles
- Write tests for affinity assignment (primary + bonus pools)
- Commit

---

## Task 5: Event System Extensions

**Files:**
- Modify: `game/src/events/mod.rs`
- Test: `game/tests/event_severity_test.rs`

**Summary:** Add new event types (Sandstorm, Blizzard, Drought, etc.), implement terrain-based severity calculation, add survival check modifiers.

**Key steps:**
- Add 5 new AreaEvent variants (Sandstorm, Blizzard, Drought, Heatwave, Rockslide)
- Implement `severity_in_terrain()` method (Catastrophic/Major/Moderate/Minor)
- Add `survival_check()` with affinity bonus (+3), item bonuses, desperation bonus (+5)
- Add desperation success rewards (42.5% stamina, 42.5% sanity, 10% item, 5% nothing)
- Reduce Catastrophic instant-death to 5% (was 10%)
- Write tests for severity calculation across terrain types
- Commit

---

## Task 6: Stamina-Based Movement System

**Files:**
- Modify: `game/src/tributes/mod.rs` (add stamina/max_stamina fields)
- Modify: `game/src/tributes/actions.rs` (replace movement with stamina costs)
- Test: `game/tests/stamina_edge_cases_test.rs`

**Summary:** Replace old movement system with stamina pool (100 base). Calculate action costs based on terrain, affinity, and desperation.

**Key steps:**
- Add stamina: u32, max_stamina: u32 to Tribute struct
- Implement `stamina_cost()` for all TributeAction types
- Calculate movement cost: base_cost × terrain_multiplier × affinity_modifier × desperation_multiplier
- Add desperation multiplier (1.0-1.5x based on health %)
- Add stamina regeneration (full restore each turn)
- Write edge case tests (stamina=0, cost>pool, negative checks)
- Commit

---

## Task 7: AI Behavior Modifications

**Files:**
- Modify: `game/src/tributes/brain.rs`
- Test: `game/tests/ai_terrain_behavior_test.rs`

**Summary:** Make AI terrain-aware in destination selection, action choice, and resource seeking.

**Key steps:**
- Update `choose_destination()` to score areas by: affinity (+20), harshness penalty (-10 per tier), visibility, item availability
- Modify action weights: boost Search in resource-scarce terrain (Desert 2.0x), boost Hide in Concealed terrain (1.5x)
- Add desperate behavior: flee to affinity terrain when health < 30
- Write tests for AI decision patterns in different terrains
- Commit

---

## Task 8: Item System Changes

**Files:**
- Modify: `game/src/items/mod.rs`
- Modify: `api/src/areas.rs` (use terrain weights for spawning)
- Test: `game/tests/item_distribution_test.rs`

**Summary:** Apply terrain-based item weights and spawn modifiers when creating items.

**Key steps:**
- Update `add_item_to_area()` to use terrain.item_weights() for type selection
- Apply terrain.item_spawn_modifier() to item quantity
- Add terrain-specific variants (Cactus Water in Desert, Thermal Gear in Tundra)
- Write tests for item distribution across terrain types
- Commit

---

## Task 9: Game Customization (API + DTO)

**Files:**
- Modify: `shared/src/lib.rs` (extend CreateGame DTO)
- Modify: `api/src/games.rs` (apply customization)
- Test: `api/tests/game_customization_test.rs`

**Summary:** Add customization options to CreateGame DTO (item quantity, event frequency, starting health) and apply them during game creation.

**Key steps:**
- Add fields to CreateGame: item_quantity, event_frequency, starting_health_range
- Define enums: ItemQuantity (Sparse/Common/Abundant), EventFrequency (Calm/Standard/Chaotic)
- Update `create_game()` to apply customization (modify item counts, event probabilities)
- Write integration tests with real SurrealDB
- Commit

---

## Task 10: Rich Descriptive Text

**Files:**
- Modify: `game/src/messages.rs`
- Test: `game/tests/narrative_test.rs`

**Summary:** Add terrain-aware narrative generation for movement, hiding, events, and actions.

**Key steps:**
- Update movement messages: "struggles through the scorching desert" vs "moves to North"
- Add terrain-specific hiding spots: "behind dense foliage" vs "in a rocky crevice"
- Enrich event descriptions with terrain context
- Add stamina-level descriptors (Energetic/Steady/Weary/Exhausted)
- Write tests comparing message quality
- Commit

---

## Task 11: Database Schema Changes

**Files:**
- Modify: `schemas/areas.surql`
- Modify: `schemas/tributes.surql`
- Modify: `api/src/areas.rs` (persist terrain)
- Modify: `api/src/tributes.rs` (persist stamina/affinity)

**Summary:** Add terrain fields to database schema and update persistence code.

**Key steps:**
- Add base_terrain, terrain_descriptors to area table
- Add terrain_affinity, stamina, max_stamina to tribute table
- Update create_area() to persist terrain data
- Update create_tribute() to persist affinity and stamina
- Update serialization/deserialization
- Test with real database
- Commit

---

## Task 12: Frontend UI Updates

**Files:**
- Modify: `web/src/components/game/create_game_modal.rs`
- Modify: `web/src/components/game/area_card.rs`
- Modify: `web/src/components/tributes/tribute_card.rs`

**Summary:** Display terrain information and add customization controls to game creation UI.

**Key steps:**
- Add customization dropdowns to CreateGame modal (item quantity, event frequency, health range)
- Display terrain type and descriptors in area cards with icons
- Show tribute affinity badges with terrain icons
- Add terrain tooltips explaining effects
- Test UI rendering
- Commit

---

## Implementation Order

**Phase 1 (Foundation):** Tasks 1-4 (terrain types, config, assignment, districts)
**Phase 2 (Core Mechanics):** Tasks 5-6 (events, stamina)
**Phase 3 (Behavior):** Tasks 7-8 (AI, items)
**Phase 4 (Integration):** Tasks 9-11 (customization, narrative, database)
**Phase 5 (Frontend):** Task 12 (UI)

**Estimated effort:** 3-4 weeks for full implementation

---

## Self-Review Checklist

- [ ] All spec requirements covered (12 terrains, stamina system, events, AI, items, customization)
- [ ] No placeholders or TODOs
- [ ] Type consistency across tasks
- [ ] Test coverage for edge cases
- [ ] Database schema changes included
- [ ] Frontend updates included
