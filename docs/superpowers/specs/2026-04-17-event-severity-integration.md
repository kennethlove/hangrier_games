# Event Severity Integration Design

**Date:** 2026-04-17  
**Status:** Draft  
**Related:** terrain-biome-system-design.md (PR #74), terrain-integration (PR #75, #76)

## Overview

Integrate the existing terrain-based event severity system into the game loop. Currently, `AreaEvent` has sophisticated survival mechanics (`severity_in_terrain()`, `survival_check()`) but events are purely cosmetic. This design makes events affect tributes based on terrain, using d20 survival checks with modifiers for terrain affinity, protective items, and desperation.

## Current State

### What Exists
- **AreaEvent enum** (game/src/areas/events.rs): 10 event types (Wildfire, Flood, Earthquake, etc.)
- **EventSeverity** levels: Minor, Moderate, Major, Catastrophic
- **severity_in_terrain()**: Maps (event, terrain) → severity (e.g., Wildfire in Forest = Catastrophic)
- **survival_check()**: d20 roll against DC based on severity, with modifiers:
  - Terrain affinity: +3
  - Protective item: +2
  - Desperation (health < 30%): +5
  - Catastrophic events: 5% instant death chance
  - Desperate survivors get rewards: stamina/sanity restore (42.5% of missing health) or item (10%)

### What's Missing
- Events are generated with `AreaEvent::random()` (no terrain consideration)
- No survival checks happen when events occur
- Events are announced but don't affect tributes
- No terrain-appropriate event generation

## Design

### Event Generation

Replace `AreaEvent::random()` with terrain-aware generation:

```rust
impl AreaEvent {
    pub fn random_for_terrain(terrain: &BaseTerrain) -> AreaEvent {
        // Weighted random based on terrain
        // Each terrain has event probabilities that match theme
    }
}
```

**Event Weights by Terrain (sample - full table in implementation):**

Each terrain gets weights for thematically appropriate events. Examples:

- **Desert**: Sandstorm (40%), Heatwave (30%), Drought (20%), other (10%)
- **Mountains**: Avalanche (35%), Rockslide (30%), Earthquake (20%), Blizzard (15%)
- **Wetlands**: Flood (50%), Wildfire (20%), Drought (15%), other (15%)
- **Tundra**: Blizzard (45%), Avalanche (25%), Earthquake (15%), other (15%)
- **Forest**: Wildfire (40%), Flood (25%), Landslide (20%), other (15%)

*(Full weights for all 12 terrains: Desert, Mountains, Wetlands, Tundra, Forest, Grasslands, Clearing, Badlands, Highlands, Jungle, UrbanRuins, Geothermal - defined in implementation)*

This ensures:
- Deserts have sandstorms/heatwaves, not blizzards
- Mountains have avalanches/rockslides, not floods
- Each biome feels distinct in its dangers

### Event Processing Flow

When an AreaEvent is created and added to an area:

1. **Get terrain** from the area
2. **Generate terrain-appropriate event** using `random_for_terrain()`
3. **Add event to area** (area becomes closed)
4. **Find all alive tributes** in the area
5. **If area empty**, return (no survival checks needed)
6. **If multiple events in area**, process only most severe (by `severity_in_terrain()`)
7. **Collect survival check results** for each tribute
8. **Apply all effects atomically** (deaths, rewards, stat changes)
9. **Generate output messages** for all affected tributes

**Key Principles:** 
- All tributes in the area face the event simultaneously
- All effects apply together before any other game state changes
- Event processing happens immediately after event creation, before movement phase
- Multiple events in same area: tributes face only the most severe event (prevents excessive lethality)

### Implementation Structure

**New Method: `Game::process_event_for_area()`**

```rust
fn process_event_for_area(
    &mut self, 
    area_name: &str, 
    event: &AreaEvent
) -> Vec<GameOutput>
```

This method:
- Gets the area's terrain from `self.areas` (access via `area_details.terrain.base`)
- Finds all alive tributes in the area
- Returns early if no tributes present (no survival checks needed)
- If multiple events in area, selects most severe (by `severity_in_terrain()`)
- For each tribute:
  - Check if has terrain affinity (`tribute.terrain_affinity == Some(area.terrain.base)`)
  - Check if has protective item (shields only - see Implementation Decisions)
  - Check if desperate (`tribute.attributes.health < 30` - absolute value, not percentage)
  - Call `event.survival_check(terrain, has_affinity, has_item_bonus, is_desperate, current_health)`
  - Store result
- Apply all results together:
  - Mark failed tributes as dead (`tribute.attributes.health = 0`)
  - Restore stamina/sanity for desperate survivors
  - Award items for lucky survivors (via `Item::new_random_consumable()`)
- Generate output messages for each affected tribute (using existing `add_tribute_message()`)

**Integration Points:**

Modify existing event creation in `game/src/games.rs`:

1. **`prepare_cycle()`** - Area closing events
2. **Area event generation during low tribute count**

After creating event:
```rust
let area_details = // get area
let terrain = &area_details.terrain.base; // NOT .as_ref() - terrain is TerrainType not Option
let event = AreaEvent::random_for_terrain(terrain);
area_details.events.push(event.clone());

// NEW: Process survival checks immediately
let outputs = self.process_event_for_area(&area_name, &event);
// Add outputs to game results
```

## Implementation Decisions

**Protective Items (This PR):**
- Only shields provide bonuses for this PR
- Check via `tribute.items.iter().any(|i| i.is_defensive())`
- Applies to physical events: Avalanche, Rockslide, Earthquake
- Other events: `has_item_bonus = false`
- Future PR will add clothing consumables for weather/terrain events

**Desperation Threshold:**
- Use absolute health value: `tribute.attributes.health < 30`
- NOT percentage (MAX_HEALTH is 100, so threshold is 30)

**Multiple Events Per Area:**
- Tributes face only the most severe event in their area
- Severity determined by `event.severity_in_terrain(&terrain)`
- Prevents compound lethality when `constrain_areas()` adds multiple events

**Reward Items:**
- Generate via `Item::new_random_consumable()`
- Add to `tribute.items` immediately
- Consumables created from terrain-appropriate pool

**Output Strategy:**
- Reuse existing `TributeDiesFromAreaEvent(tribute, event)` for deaths
- Use `add_tribute_message()` for survival/rewards (no new GameOutput variants)
- Differentiate instant death vs. normal death in messaging:
  - Instant death: "{tribute} is instantly killed by the catastrophic {event}!"
  - Normal death: "{tribute} dies from the {event}"
  - Survival with reward: "{tribute} survives the {event}, recovering {X} stamina"

## Event Processing Order (Within Cycle)

1. **prepare_cycle():** Events generated via `random_for_terrain()`, areas closed (events added to `area.events`)
2. **process_event_for_area():** Survival checks run immediately after event added to area
3. **Tribute deaths applied** atomically before movement phase
4. **run_tribute_cycle():** Living tributes act (dead tributes skipped automatically)

### Data Structures

**SurvivalResult** (already exists in game/src/areas/events.rs):
```rust
pub struct SurvivalResult {
    pub survived: bool,
    pub instant_death: bool,
    pub stamina_restored: u32,
    pub sanity_restored: u32,
    pub reward_item: Option<String>,
}
```

**GameOutput Usage:**
- Use existing `TributeDiesFromAreaEvent(tribute, event)` variant (already exists in output.rs)
- No new variants needed
- Survival/rewards use descriptive game messages via `add_tribute_message()`

### Protective Items

For the `has_item_bonus` modifier in survival checks:

**This PR (Shields Only):**
- Physical events get bonus if tribute has defensive item
- Events: Avalanche, Rockslide, Earthquake
- Check: `tribute.items.iter().any(|i| i.is_defensive())`
- All other events: `has_item_bonus = false`

**Future Enhancement:**
- Add clothing consumables for weather events (Blizzard, Heatwave, Sandstorm)
- Add terrain gear for terrain events (Flood, Landslide)
- Requires new ItemType variants and spawning logic

## Testing Strategy

### Unit Tests

**Event Generation:**
```rust
#[test]
fn test_desert_generates_appropriate_events() {
    let mut counts = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Desert);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0) >= &30); // At least 30%
    assert!(counts.get(&AreaEvent::Heatwave).unwrap_or(&0) >= &20);  // At least 20%
    assert!(counts.get(&AreaEvent::Blizzard).unwrap_or(&0) == &0);    // Never in desert
}
```
- Similar tests for all 12 terrains
- Verify weight distributions match spec

**Survival Checks (already tested in events.rs):**
- Existing tests cover survival_check() mechanics
- No new unit tests needed for survival logic

### Integration Tests

**Event Processing:**
- `test_wildfire_in_forest_kills_tributes()` - Catastrophic event, verify deaths
- `test_wildfire_in_desert_minor_impact()` - Minor event, verify few/no deaths
- `test_terrain_affinity_helps_survival()` - Tribute with affinity survives better
- `test_desperate_survivors_get_rewards()` - Low health survivors get stamina/sanity
- `test_all_tributes_processed_atomically()` - Multiple tributes, all face event together
- `test_desperate_survivor_item_reward()` - Verify actual Item added to tribute.items
- `test_multiple_events_same_area()` - Only most severe event processed
- `test_event_in_empty_area()` - No crash when no tributes present
- `test_event_kills_last_tribute()` - Game end condition triggered correctly
- `test_terrain_affinity_uses_tribute_field()` - Verify `tribute.terrain_affinity` checked
- `test_instant_death_vs_normal_death()` - Different messages for instant/normal death
- `test_shield_provides_bonus_for_physical_events()` - Defensive items work for Avalanche/Rockslide/Earthquake

## Migration Path

**Phase 1: Event Generation**
- Add `random_for_terrain()` to AreaEvent
- Define event weights for all 12 terrains
- Replace `AreaEvent::random()` calls with terrain-aware version

**Phase 2: Survival Processing**
- Add `process_event_for_area()` to Game
- Integrate at event creation sites
- Add GameOutput variants

**Phase 3: Protective Items**
- Implement shields-only logic for physical events
- Check `tribute.items.iter().any(|i| i.is_defensive())`
- Apply bonus to Avalanche, Rockslide, Earthquake
- Add tests for protective item bonuses

## Future Enhancements

**Not in this spec:**
- Persistent events (events last multiple cycles)
- Tribute awareness of events in adjacent areas (AI decision-making)
- Event-specific visual effects (frontend)
- TributeEvent integration (personal disasters like dysentery)
- Clothing/boots consumables for weather/terrain event protection

## Success Criteria

- Events are terrain-appropriate (Desert has sandstorms, Mountains have avalanches)
- Tributes die from events based on terrain severity
- Terrain affinity provides +3 survival bonus
- Shields provide +2 bonus for physical events (Avalanche, Rockslide, Earthquake)
- Desperate tributes (health < 30) get +5 bonus and rewards when they survive
- All tributes in area face event simultaneously (atomic processing)
- Multiple events in same area: only most severe event is processed
- Empty areas don't crash when events occur
- All tests pass (12 unit tests for terrain weights + 12 integration tests)
- No regression in existing game mechanics

## Dependencies

- PR #74: Terrain system (merged)
- PR #75: Terrain integration (merged)
- PR #76: Movement adjacency (merged)
- Existing: `Item::new_random_consumable()` for reward items
- Existing: `Item::is_defensive()` for shield detection
- Existing: `TributeDiesFromAreaEvent` GameOutput variant

## Estimated Complexity

**Medium** - Uses existing survival_check() logic, but requires:
- Event weight tables for 12 terrains (simple but tedious)
- Integration with game loop at 4 call sites in games.rs
- Shield detection logic for protective items
- Comprehensive testing (12 terrain generation tests + 12 integration tests)

**Files Modified:**
- `game/src/areas/events.rs` - Add `random_for_terrain()` with weight tables for 12 terrains (~70 lines)
- `game/src/games.rs` - Add `process_event_for_area()` method, integrate at 4 event creation sites (~120 lines)
- `game/tests/event_integration_test.rs` - **New file** for integration tests (~80 lines)

**Lines Changed:** ~270 lines total
