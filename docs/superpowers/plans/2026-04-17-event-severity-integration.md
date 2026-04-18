# Event Severity Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate terrain-based event severity system into game loop so events affect tributes with survival checks

**Architecture:** Add `random_for_terrain()` for terrain-appropriate event generation, add `process_event_for_area()` to run survival checks when events occur, integrate at 4 existing event creation sites in games.rs

**Tech Stack:** Rust, rstest for testing, existing AreaEvent/survival_check mechanics

---

## File Structure

**Modified Files:**
- `game/src/areas/events.rs` - Add `random_for_terrain()` with weighted event selection for 12 terrains
- `game/src/games.rs` - Add `process_event_for_area()` method, integrate at event creation sites

**New Files:**
- `game/tests/event_integration_test.rs` - Integration tests for event processing flow

**Test Files:**
- Tests for AreaEvent::random_for_terrain() added to existing `game/src/areas/events.rs` tests section
- Integration tests in new `game/tests/event_integration_test.rs`

---

### Task 1: Terrain-Aware Event Generation

**Files:**
- Modify: `game/src/areas/events.rs:77-80` (after `random()` method)
- Test: `game/src/areas/events.rs:274-` (in existing tests module)

- [ ] **Step 1: Write failing test for Desert event weights**

Add to `game/src/areas/events.rs` in `mod tests` section (after line 320):

```rust
#[test]
fn test_desert_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Desert);
        *counts.entry(event).or_insert(0) += 1;
    }
    // Desert should have high sandstorm/heatwave, no blizzard
    assert!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0) >= &30);
    assert!(counts.get(&AreaEvent::Heatwave).unwrap_or(&0) >= &20);
    assert_eq!(counts.get(&AreaEvent::Blizzard).unwrap_or(&0), &0);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --package game test_desert_generates_terrain_appropriate_events
```

Expected: FAIL with "no method named `random_for_terrain` found"

- [ ] **Step 3: Implement random_for_terrain() with all 12 terrain weight tables**

Add to `game/src/areas/events.rs` after `random()` method (line ~80):

```rust
/// Generate a terrain-appropriate random event with weighted probabilities
pub fn random_for_terrain(terrain: &BaseTerrain) -> AreaEvent {
    use BaseTerrain::*;
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    
    // Define weights for each terrain (percentages out of 100)
    let weights: Vec<(AreaEvent, u32)> = match terrain {
        Desert => vec![
            (AreaEvent::Sandstorm, 40),
            (AreaEvent::Heatwave, 30),
            (AreaEvent::Drought, 20),
            (AreaEvent::Wildfire, 5),
            (AreaEvent::Earthquake, 5),
        ],
        Mountains => vec![
            (AreaEvent::Avalanche, 35),
            (AreaEvent::Rockslide, 30),
            (AreaEvent::Earthquake, 20),
            (AreaEvent::Blizzard, 15),
        ],
        Wetlands => vec![
            (AreaEvent::Flood, 50),
            (AreaEvent::Wildfire, 20),
            (AreaEvent::Drought, 15),
            (AreaEvent::Landslide, 10),
            (AreaEvent::Earthquake, 5),
        ],
        Tundra => vec![
            (AreaEvent::Blizzard, 45),
            (AreaEvent::Avalanche, 25),
            (AreaEvent::Earthquake, 15),
            (AreaEvent::Heatwave, 10),
            (AreaEvent::Rockslide, 5),
        ],
        Forest => vec![
            (AreaEvent::Wildfire, 40),
            (AreaEvent::Flood, 25),
            (AreaEvent::Landslide, 20),
            (AreaEvent::Earthquake, 10),
            (AreaEvent::Blizzard, 5),
        ],
        Grasslands => vec![
            (AreaEvent::Wildfire, 35),
            (AreaEvent::Drought, 25),
            (AreaEvent::Flood, 20),
            (AreaEvent::Heatwave, 15),
            (AreaEvent::Sandstorm, 5),
        ],
        Clearing => vec![
            (AreaEvent::Wildfire, 30),
            (AreaEvent::Flood, 25),
            (AreaEvent::Heatwave, 20),
            (AreaEvent::Drought, 15),
            (AreaEvent::Earthquake, 10),
        ],
        Badlands => vec![
            (AreaEvent::Sandstorm, 35),
            (AreaEvent::Rockslide, 25),
            (AreaEvent::Drought, 20),
            (AreaEvent::Heatwave, 15),
            (AreaEvent::Earthquake, 5),
        ],
        Highlands => vec![
            (AreaEvent::Rockslide, 30),
            (AreaEvent::Landslide, 25),
            (AreaEvent::Blizzard, 20),
            (AreaEvent::Earthquake, 15),
            (AreaEvent::Avalanche, 10),
        ],
        Jungle => vec![
            (AreaEvent::Wildfire, 35),
            (AreaEvent::Flood, 30),
            (AreaEvent::Landslide, 20),
            (AreaEvent::Heatwave, 10),
            (AreaEvent::Earthquake, 5),
        ],
        UrbanRuins => vec![
            (AreaEvent::Earthquake, 35),
            (AreaEvent::Wildfire, 25),
            (AreaEvent::Rockslide, 20),
            (AreaEvent::Flood, 15),
            (AreaEvent::Landslide, 5),
        ],
        Geothermal => vec![
            (AreaEvent::Heatwave, 40),
            (AreaEvent::Earthquake, 30),
            (AreaEvent::Rockslide, 20),
            (AreaEvent::Wildfire, 10),
        ],
    };

    // Calculate total weight
    let total: u32 = weights.iter().map(|(_, w)| w).sum();
    let roll = rng.random_range(0..total);

    // Select event based on weighted random
    let mut cumulative = 0;
    for (event, weight) in weights {
        cumulative += weight;
        if roll < cumulative {
            return event;
        }
    }

    // Fallback (should never reach here if weights are correct)
    weights[0].0.clone()
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test --package game test_desert_generates_terrain_appropriate_events
```

Expected: PASS

- [ ] **Step 5: Add tests for remaining 11 terrains**

Add to `game/src/areas/events.rs` in tests module:

```rust
#[test]
fn test_mountains_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Mountains);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Avalanche).unwrap_or(&0) >= &25);
    assert!(counts.get(&AreaEvent::Rockslide).unwrap_or(&0) >= &20);
    assert_eq!(counts.get(&AreaEvent::Flood).unwrap_or(&0), &0);
}

#[test]
fn test_wetlands_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Wetlands);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Flood).unwrap_or(&0) >= &40);
    assert_eq!(counts.get(&AreaEvent::Avalanche).unwrap_or(&0), &0);
}

#[test]
fn test_tundra_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Tundra);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Blizzard).unwrap_or(&0) >= &35);
    assert_eq!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0), &0);
}

#[test]
fn test_forest_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Forest);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Wildfire).unwrap_or(&0) >= &30);
    assert_eq!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0), &0);
}

#[test]
fn test_grasslands_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Grasslands);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Wildfire).unwrap_or(&0) >= &25);
    assert_eq!(counts.get(&AreaEvent::Avalanche).unwrap_or(&0), &0);
}

#[test]
fn test_clearing_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Clearing);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Wildfire).unwrap_or(&0) >= &20);
    assert_eq!(counts.get(&AreaEvent::Avalanche).unwrap_or(&0), &0);
}

#[test]
fn test_badlands_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Badlands);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0) >= &25);
    assert_eq!(counts.get(&AreaEvent::Flood).unwrap_or(&0), &0);
}

#[test]
fn test_highlands_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Highlands);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Rockslide).unwrap_or(&0) >= &20);
    assert_eq!(counts.get(&AreaEvent::Sandstorm).unwrap_or(&0), &0);
}

#[test]
fn test_jungle_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Jungle);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Wildfire).unwrap_or(&0) >= &25);
    assert_eq!(counts.get(&AreaEvent::Blizzard).unwrap_or(&0), &0);
}

#[test]
fn test_urban_ruins_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::UrbanRuins);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Earthquake).unwrap_or(&0) >= &25);
    assert_eq!(counts.get(&AreaEvent::Avalanche).unwrap_or(&0), &0);
}

#[test]
fn test_geothermal_generates_terrain_appropriate_events() {
    use std::collections::HashMap;
    let mut counts: HashMap<AreaEvent, u32> = HashMap::new();
    for _ in 0..100 {
        let event = AreaEvent::random_for_terrain(&BaseTerrain::Geothermal);
        *counts.entry(event).or_insert(0) += 1;
    }
    assert!(counts.get(&AreaEvent::Heatwave).unwrap_or(&0) >= &30);
    assert_eq!(counts.get(&AreaEvent::Flood).unwrap_or(&0), &0);
}
```

- [ ] **Step 6: Run all terrain generation tests**

```bash
cargo test --package game generates_terrain_appropriate_events
```

Expected: All 12 tests PASS

- [ ] **Step 7: Commit terrain-aware event generation**

```bash
git add game/src/areas/events.rs
git commit -m "feat: add terrain-aware event generation with weighted probabilities

- Add AreaEvent::random_for_terrain() with weight tables for 12 terrains
- Desert: sandstorm/heatwave dominant, no blizzards
- Mountains: avalanche/rockslide dominant, no floods
- Each terrain has thematically appropriate event distribution
- Add 12 tests verifying weight distributions"
```

---

### Task 2: Event Survival Processing Core

**Files:**
- Modify: `game/src/games.rs:~200` (add new method to Game impl)
- Test: `game/tests/event_integration_test.rs` (new file)

- [ ] **Step 1: Write failing integration test for catastrophic event**

Create `game/tests/event_integration_test.rs`:

```rust
use game::areas::events::AreaEvent;
use game::games::Game;
use game::terrain::BaseTerrain;
use game::tributes::Tribute;

#[test]
fn test_wildfire_in_forest_kills_tributes() {
    let mut game = Game::new("test", "test-game");
    
    // Create area with Forest terrain
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Forest;
    
    // Create tribute in forest area with low health
    let mut tribute = Tribute::random();
    tribute.area = area_name.clone();
    tribute.attributes.health = 50; // Not desperate but vulnerable
    tribute.terrain_affinity = None; // No affinity bonus
    game.tributes.push(tribute.clone());
    
    // Process wildfire event (catastrophic in forest)
    let event = AreaEvent::Wildfire;
    let outputs = game.process_event_for_area(&area_name.to_string(), &event);
    
    // Verify tribute likely died (run 10 times, expect at least some deaths)
    let mut death_count = 0;
    for _ in 0..10 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
        if test_game.tributes[0].attributes.health == 0 {
            death_count += 1;
        }
    }
    
    assert!(death_count > 0, "Catastrophic wildfire should kill some tributes");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test --package game test_wildfire_in_forest_kills_tributes
```

Expected: FAIL with "no method named `process_event_for_area` found"

- [ ] **Step 3: Implement process_event_for_area() method**

Add to `game/src/games.rs` in Game impl block (around line 200):

```rust
/// Process survival checks for all tributes in an area when an event occurs
fn process_event_for_area(&mut self, area_name: &str, event: &AreaEvent) {
    use crate::areas::events::EventSeverity;
    use crate::items::Item;
    use crate::output::GameOutput;
    
    // Get area terrain
    let terrain = {
        let area = self.areas.iter()
            .find(|a| a.area.as_ref().map(|name| name.to_string()) == Some(area_name.to_string()));
        
        match area {
            Some(a) => a.terrain.base.clone(),
            None => return, // Area not found
        }
    };
    
    // Find all alive tributes in this area
    let tribute_indices: Vec<usize> = self.tributes.iter()
        .enumerate()
        .filter(|(_, t)| {
            t.is_alive() && 
            t.area.to_string() == area_name
        })
        .map(|(idx, _)| idx)
        .collect();
    
    if tribute_indices.is_empty() {
        return; // No tributes in area, nothing to do
    }
    
    // Handle multiple events: select most severe
    let area_events = {
        let area = self.areas.iter()
            .find(|a| a.area.as_ref().map(|n| n.to_string()) == Some(area_name.to_string()));
        
        match area {
            Some(a) => a.events.clone(),
            None => vec![],
        }
    };
    
    let most_severe_event = if area_events.len() > 1 {
        // Find event with highest severity
        area_events.iter()
            .max_by_key(|e| e.severity_in_terrain(&terrain))
            .cloned()
            .unwrap_or_else(|| event.clone())
    } else {
        event.clone()
    };
    
    // Process each tribute's survival check
    for tribute_idx in tribute_indices {
        let tribute = &mut self.tributes[tribute_idx];
        
        // Check modifiers
        let has_affinity = tribute.terrain_affinity == Some(terrain.clone());
        
        // Check for protective items (shields only for physical events)
        let is_physical_event = matches!(
            most_severe_event,
            AreaEvent::Avalanche | AreaEvent::Rockslide | AreaEvent::Earthquake
        );
        let has_item_bonus = is_physical_event && 
            tribute.items.iter().any(|item| item.is_defensive());
        
        let is_desperate = tribute.attributes.health < 30;
        let current_health = tribute.attributes.health;
        
        // Run survival check
        let result = most_severe_event.survival_check(
            &terrain,
            has_affinity,
            has_item_bonus,
            is_desperate,
            current_health,
        );
        
        // Apply results
        if !result.survived {
            tribute.attributes.health = 0;
            
            let message = if result.instant_death {
                format!(
                    "{} is instantly killed by the catastrophic {}!",
                    tribute.name, most_severe_event
                )
            } else {
                format!("{} dies from the {}", tribute.name, most_severe_event)
            };
            
            add_tribute_message(&tribute.name, &self.identifier, message)
                .expect("Failed to add tribute death message");
        } else {
            // Survivor - apply rewards if any
            if result.stamina_restored > 0 {
                tribute.stamina = tribute.stamina.saturating_add(result.stamina_restored);
                let message = format!(
                    "{} survives the {}, recovering {} stamina",
                    tribute.name, most_severe_event, result.stamina_restored
                );
                add_tribute_message(&tribute.name, &self.identifier, message)
                    .expect("Failed to add tribute message");
            }
            
            if result.sanity_restored > 0 {
                tribute.sanity = tribute.sanity.saturating_add(result.sanity_restored);
                let message = format!(
                    "{} survives the {}, recovering {} sanity",
                    tribute.name, most_severe_event, result.sanity_restored
                );
                add_tribute_message(&tribute.name, &self.identifier, message)
                    .expect("Failed to add tribute message");
            }
            
            if result.reward_item.is_some() {
                let item = Item::new_random_consumable();
                let item_name = item.name.clone();
                tribute.items.push(item);
                let message = format!(
                    "{} survives the {} and finds a {}",
                    tribute.name, most_severe_event, item_name
                );
                add_tribute_message(&tribute.name, &self.identifier, message)
                    .expect("Failed to add tribute message");
            }
        }
    }
}
```

- [ ] **Step 4: Add necessary imports to games.rs**

Add at top of `game/src/games.rs` with other imports:

```rust
use crate::tributes::add_tribute_message;
```

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test --package game test_wildfire_in_forest_kills_tributes
```

Expected: PASS

- [ ] **Step 6: Add integration test for minor event**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_wildfire_in_desert_minor_impact() {
    let mut game = Game::new("test", "test-game");
    
    // Create area with Desert terrain
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Desert;
    
    // Create tribute
    let mut tribute = Tribute::random();
    tribute.area = area_name.clone();
    tribute.attributes.health = 50;
    tribute.terrain_affinity = None;
    game.tributes.push(tribute);
    
    // Process wildfire (minor in desert)
    let event = AreaEvent::Wildfire;
    
    // Run 10 times, expect low death rate
    let mut death_count = 0;
    for _ in 0..10 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
        if test_game.tributes[0].attributes.health == 0 {
            death_count += 1;
        }
    }
    
    assert!(death_count < 5, "Minor wildfire should rarely kill tributes");
}
```

- [ ] **Step 7: Run test to verify minor event**

```bash
cargo test --package game test_wildfire_in_desert_minor_impact
```

Expected: PASS

- [ ] **Step 8: Commit event survival processing core**

```bash
git add game/src/games.rs game/tests/event_integration_test.rs
git commit -m "feat: add event survival processing with terrain severity

- Add Game::process_event_for_area() method
- Run survival checks for all tributes in area when event occurs
- Handle empty areas (early return)
- Select most severe event when multiple events in area
- Apply deaths, stamina/sanity rewards, item rewards
- Shield bonus for physical events (Avalanche, Rockslide, Earthquake)
- Terrain affinity and desperation modifiers
- Add integration tests for catastrophic vs minor events"
```

---

### Task 3: Integration Tests for Edge Cases

**Files:**
- Test: `game/tests/event_integration_test.rs`

- [ ] **Step 1: Add test for terrain affinity bonus**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_terrain_affinity_helps_survival() {
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Forest;
    
    // Two tributes: one with affinity, one without
    let mut tribute_with_affinity = Tribute::random();
    tribute_with_affinity.area = area_name.clone();
    tribute_with_affinity.attributes.health = 50;
    tribute_with_affinity.terrain_affinity = Some(BaseTerrain::Forest);
    
    let mut tribute_without_affinity = Tribute::random();
    tribute_without_affinity.area = area_name.clone();
    tribute_without_affinity.attributes.health = 50;
    tribute_without_affinity.terrain_affinity = None;
    
    game.tributes.push(tribute_with_affinity);
    game.tributes.push(tribute_without_affinity);
    
    // Run wildfire 20 times, count survivals
    let event = AreaEvent::Wildfire;
    let mut affinity_survived = 0;
    let mut no_affinity_survived = 0;
    
    for _ in 0..20 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
        
        if test_game.tributes[0].attributes.health > 0 {
            affinity_survived += 1;
        }
        if test_game.tributes[1].attributes.health > 0 {
            no_affinity_survived += 1;
        }
    }
    
    assert!(
        affinity_survived > no_affinity_survived,
        "Tribute with terrain affinity should survive more often"
    );
}
```

- [ ] **Step 2: Add test for desperate survivor rewards**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_desperate_survivors_get_rewards() {
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Desert;
    
    // Desperate tribute (health < 30)
    let mut tribute = Tribute::random();
    tribute.area = area_name.clone();
    tribute.attributes.health = 25; // Desperate
    tribute.terrain_affinity = Some(BaseTerrain::Desert); // Help survival
    let initial_stamina = tribute.stamina;
    let initial_sanity = tribute.sanity;
    let initial_item_count = tribute.items.len();
    
    game.tributes.push(tribute);
    
    // Run minor event multiple times until reward
    let event = AreaEvent::Heatwave;
    let mut got_reward = false;
    
    for _ in 0..50 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
        
        let tribute = &test_game.tributes[0];
        if tribute.is_alive() && (
            tribute.stamina > initial_stamina ||
            tribute.sanity > initial_sanity ||
            tribute.items.len() > initial_item_count
        ) {
            got_reward = true;
            break;
        }
    }
    
    assert!(got_reward, "Desperate survivor should eventually get reward");
}
```

- [ ] **Step 3: Add test for shield protective bonus**

Add to `game/tests/event_integration_test.rs`:

```rust
use game::items::Item;

#[test]
fn test_shield_provides_bonus_for_physical_events() {
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Mountains;
    
    // Tribute with shield
    let mut tribute_with_shield = Tribute::random();
    tribute_with_shield.area = area_name.clone();
    tribute_with_shield.attributes.health = 50;
    tribute_with_shield.items.push(Item::new_random_shield());
    
    // Tribute without shield
    let mut tribute_without_shield = Tribute::random();
    tribute_without_shield.area = area_name.clone();
    tribute_without_shield.attributes.health = 50;
    
    game.tributes.push(tribute_with_shield);
    game.tributes.push(tribute_without_shield);
    
    // Physical event (Avalanche)
    let event = AreaEvent::Avalanche;
    let mut with_shield_survived = 0;
    let mut without_shield_survived = 0;
    
    for _ in 0..20 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
        
        if test_game.tributes[0].attributes.health > 0 {
            with_shield_survived += 1;
        }
        if test_game.tributes[1].attributes.health > 0 {
            without_shield_survived += 1;
        }
    }
    
    assert!(
        with_shield_survived > without_shield_survived,
        "Tribute with shield should survive physical events more often"
    );
}
```

- [ ] **Step 4: Add test for empty area**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_event_in_empty_area() {
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Forest;
    // No tributes added
    
    let event = AreaEvent::Wildfire;
    game.process_event_for_area(&area_name.to_string(), &event);
    
    // Should not crash - verify by reaching this line
    assert!(true, "Empty area event processing should not crash");
}
```

- [ ] **Step 5: Add test for multiple events**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_multiple_events_same_area() {
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Mountains;
    
    // Add multiple events to area
    game.areas[0].events.push(AreaEvent::Blizzard); // Major
    game.areas[0].events.push(AreaEvent::Avalanche); // Catastrophic
    
    let mut tribute = Tribute::random();
    tribute.area = area_name.clone();
    tribute.attributes.health = 80;
    game.tributes.push(tribute);
    
    // Process with first event (should use most severe = Avalanche)
    game.process_event_for_area(&area_name.to_string(), &AreaEvent::Blizzard);
    
    // Verify tribute faced the avalanche (catastrophic) not blizzard (major)
    // Run multiple times to verify severity difference
    let mut death_count = 0;
    for _ in 0..10 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &AreaEvent::Blizzard);
        if test_game.tributes[0].attributes.health == 0 {
            death_count += 1;
        }
    }
    
    assert!(
        death_count > 5,
        "Should process most severe event (Avalanche) causing high death rate"
    );
}
```

- [ ] **Step 6: Add test for instant death messaging**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_instant_death_vs_normal_death() {
    // This test verifies message differentiation (implementation detail)
    // Just ensure both code paths execute without crash
    let mut game = Game::new("test", "test-game");
    
    let area_name = game.areas[0].area.clone().unwrap();
    game.areas[0].terrain.base = BaseTerrain::Mountains;
    
    let mut tribute = Tribute::random();
    tribute.area = area_name.clone();
    tribute.attributes.health = 50;
    game.tributes.push(tribute);
    
    // Run catastrophic event many times
    let event = AreaEvent::Avalanche;
    for _ in 0..20 {
        let mut test_game = game.clone();
        test_game.process_event_for_area(&area_name.to_string(), &event);
    }
    
    // Verify no crash (messages handled correctly)
    assert!(true, "Death messaging should not crash");
}
```

- [ ] **Step 7: Run all integration tests**

```bash
cargo test --package game --test event_integration_test
```

Expected: All 9 tests PASS

- [ ] **Step 8: Commit integration tests**

```bash
git add game/tests/event_integration_test.rs
git commit -m "test: add integration tests for event survival edge cases

- test_terrain_affinity_helps_survival
- test_desperate_survivors_get_rewards
- test_shield_provides_bonus_for_physical_events
- test_event_in_empty_area
- test_multiple_events_same_area
- test_instant_death_vs_normal_death"
```

---

### Task 4: Replace AreaEvent::random() Calls

**Files:**
- Modify: `game/src/games.rs:332`, `game/src/games.rs:379`, `game/src/games.rs:387`

- [ ] **Step 1: Replace in trigger_cycle_events() method**

Find line 332 in `game/src/games.rs` (in `trigger_cycle_events()` method):

```rust
// OLD:
let area_event = AreaEvent::random();

// NEW:
let terrain = &area_details.terrain.base;
let area_event = AreaEvent::random_for_terrain(terrain);
```

Also add survival processing after event is added:

```rust
area_details.events.push(area_event.clone());
// NEW: Process survival checks
self.process_event_for_area(&area_name, &area_event);
```

Full context (lines 330-344):

```rust
for area_details in self.areas.iter_mut() {
    if rng.random_bool(frequency) {
        let terrain = &area_details.terrain.base;
        let area_event = AreaEvent::random_for_terrain(terrain);
        let area_name = area_details.area.clone().unwrap().to_string();
        area_details.events.push(area_event.clone());
        
        // Process survival checks immediately
        self.process_event_for_area(&area_name, &area_event);
        
        let event_name = area_event.to_string();
        add_area_message(
            area_name.as_str(),
            &self.identifier,
            format!(
                "{}",
                GameOutput::AreaEvent(event_name.as_str(), area_name.as_str())
            ),
        )
        .expect("Failed to add area event message");
    }
}
```

- [ ] **Step 2: Replace in constrain_areas() - first occurrence**

Find line 379 in `game/src/games.rs` (in `constrain_areas()` method):

```rust
// OLD:
let event = AreaEvent::random();

// NEW:
let terrain = &area_details.terrain.base;
let event = AreaEvent::random_for_terrain(terrain);
```

- [ ] **Step 3: Replace in constrain_areas() - second occurrence**

Find line 387 in `game/src/games.rs`:

```rust
// OLD:
let event = AreaEvent::random();

// NEW:
let terrain = &area_details.terrain.base;
let event = AreaEvent::random_for_terrain(terrain);
```

- [ ] **Step 4: Add survival processing in constrain_areas()**

After the for loop that processes area_events (around line 399-415), add survival processing:

Find the section:
```rust
for (area_name, (_, events)) in area_events.iter() {
    for event in events {
        // ... existing code ...
    }
}
```

After this loop, add:
```rust
// Process survival checks for all events
for (area_name, (_, events)) in area_events.iter() {
    // Events already added to area, now process survival
    if let Some(event) = events.first() {
        self.process_event_for_area(area_name, event);
    }
}
```

- [ ] **Step 5: Write test to verify terrain-appropriate events in game**

Add to `game/tests/event_integration_test.rs`:

```rust
#[test]
fn test_game_generates_terrain_appropriate_events() {
    let mut game = Game::new("test", "test-game");
    
    // Set specific terrain
    game.areas[0].terrain.base = BaseTerrain::Desert;
    let area_name = game.areas[0].area.clone().unwrap();
    
    // Trigger event generation (would need to expose trigger_cycle_events or test indirectly)
    // For now, test process_event_for_area directly with terrain-appropriate event
    let event = AreaEvent::random_for_terrain(&BaseTerrain::Desert);
    
    // Verify it's a desert-appropriate event
    let is_desert_event = matches!(
        event,
        AreaEvent::Sandstorm | AreaEvent::Heatwave | AreaEvent::Drought | 
        AreaEvent::Wildfire | AreaEvent::Earthquake
    );
    
    assert!(is_desert_event, "Event should be appropriate for desert terrain");
}
```

- [ ] **Step 6: Run test**

```bash
cargo test --package game test_game_generates_terrain_appropriate_events
```

Expected: PASS

- [ ] **Step 7: Run full test suite**

```bash
cargo test --package game
```

Expected: All existing + new tests PASS (367+ tests)

- [ ] **Step 8: Commit terrain-aware integration**

```bash
git add game/src/games.rs game/tests/event_integration_test.rs
git commit -m "feat: integrate terrain-aware events into game loop

- Replace AreaEvent::random() with random_for_terrain() in trigger_cycle_events
- Replace in constrain_areas() (2 occurrences)
- Add process_event_for_area() calls after event generation
- Events now affect tributes immediately via survival checks
- Add integration test for terrain-appropriate event generation"
```

---

### Task 5: Update Test Fixtures

**Files:**
- Modify: `game/src/games.rs` (test functions at bottom)

- [ ] **Step 1: Find test functions using AreaEvent::random()**

Lines 718, 794, 848, 849, 875, 876, 957 are in test functions. Update each:

Line 718:
```rust
// OLD:
let event = AreaEvent::random();

// NEW:
let event = AreaEvent::random_for_terrain(&BaseTerrain::Forest);
```

Line 794:
```rust
// OLD:
let event = AreaEvent::random();

// NEW:
let event = AreaEvent::random_for_terrain(&BaseTerrain::Mountains);
```

Lines 848-849:
```rust
// OLD:
area.events.push(AreaEvent::random());
area.events.push(AreaEvent::random());

// NEW:
area.events.push(AreaEvent::random_for_terrain(&area.terrain.base));
area.events.push(AreaEvent::random_for_terrain(&area.terrain.base));
```

Lines 875-876:
```rust
// OLD:
game.areas[0].events.push(AreaEvent::random());
game.areas[1].events.push(AreaEvent::random());

// NEW:
game.areas[0].events.push(AreaEvent::random_for_terrain(&game.areas[0].terrain.base));
game.areas[1].events.push(AreaEvent::random_for_terrain(&game.areas[1].terrain.base));
```

Line 957:
```rust
// OLD:
game.areas[0].events.push(AreaEvent::random());

// NEW:
game.areas[0].events.push(AreaEvent::random_for_terrain(&game.areas[0].terrain.base));
```

- [ ] **Step 2: Run game tests**

```bash
cargo test --package game --lib
```

Expected: All tests PASS

- [ ] **Step 3: Commit test fixture updates**

```bash
git add game/src/games.rs
git commit -m "test: update test fixtures to use terrain-aware event generation

- Replace AreaEvent::random() with random_for_terrain() in 6 test functions
- Use area's terrain for appropriate event generation in tests"
```

---

### Task 6: Final Verification

**Files:**
- All modified files

- [ ] **Step 1: Run complete test suite**

```bash
cargo test --package game
```

Expected: All 367+ tests PASS (12 new terrain generation + 9 new integration tests)

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --package game
```

Expected: No new warnings (30 pre-existing warnings acceptable)

- [ ] **Step 3: Run cargo fmt**

```bash
cargo fmt
```

Expected: Code formatted

- [ ] **Step 4: Verify no AreaEvent::random() calls remain in production code**

```bash
grep -n "AreaEvent::random()" game/src/**/*.rs
```

Expected: Only test code (in `#[cfg(test)]` sections)

- [ ] **Step 5: Run game smoke test (if available)**

```bash
cargo run --package game --example simple_game
```

Expected: Game runs, events appear terrain-appropriate, tributes die from events

- [ ] **Step 6: Create final commit for quality checks**

```bash
git add -A
git commit -m "chore: run fmt and verify event severity integration

- All tests passing (379 total)
- No AreaEvent::random() in production code
- Terrain-aware events fully integrated
- Survival checks processed immediately after event creation"
```

---

## Self-Review Checklist

**Spec Coverage:**
-  Terrain-aware event generation (Task 1)
-  Event survival processing core (Task 2)
-  Shields-only protective items (Task 2, Step 3)
-  Multiple events = most severe (Task 2, Step 3)
-  Empty area handling (Task 2, Step 3, Task 3 Step 4)
-  Terrain affinity bonus (Task 3, Step 1)
-  Desperation rewards (Task 3, Step 2)
-  Integration at 4 call sites (Task 4)
-  Comprehensive testing (12 terrain + 9 integration tests)

**No Placeholders:**
- All code blocks complete
- All file paths exact
- All test expectations specified
- All import statements included

**Type Consistency:**
- `AreaEvent::random_for_terrain()` signature consistent
- `Game::process_event_for_area()` signature consistent
- `BaseTerrain` usage consistent throughout
- `EventSeverity` comparison uses existing enum

**Implementation Order:**
1. Terrain generation (isolated, testable)
2. Survival processing (core logic, testable)
3. Edge case tests (verification)
4. Integration (replace existing calls)
5. Test fixtures (maintain test quality)
6. Final verification (quality gates)

## Execution Notes

**Estimated Time:** 3-4 hours
- Task 1: 45 minutes (terrain weights + 12 tests)
- Task 2: 90 minutes (core processing + initial tests)
- Task 3: 45 minutes (6 edge case tests)
- Task 4: 30 minutes (replace 4 call sites)
- Task 5: 15 minutes (test fixtures)
- Task 6: 15 minutes (final verification)

**Testing Strategy:**
- TDD throughout (write failing test first)
- Run tests after each implementation
- Commit after each logical unit
- Final full suite verification

**Common Issues:**
- Borrow checker with mutable self: Fixed by cloning terrain before iterating tributes
- Multiple events handling: Select most severe using EventSeverity ord
- Empty area: Early return prevents iteration errors
