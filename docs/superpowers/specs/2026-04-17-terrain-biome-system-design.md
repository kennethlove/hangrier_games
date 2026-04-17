# Terrain/Biome System & Game Customization Design

**Date:** 2026-04-17  
**Status:** Design Approved  
**Issues:** hangrier_games-n1g (Game Customization), hangrier_games-eum (Terrain/Biome System)

## Overview

Add a terrain/biome system to the Hangrier Games that makes each region feel distinct and affects gameplay through movement costs, event probabilities, item distributions, and AI behavior. Pair this with user-facing game customization options for items, tribute attributes, and event frequency.

## Goals

1. **Rich terrain variety:** Each of the 5 arena regions can have different terrain types (Forest, Desert, Tundra, etc.) with modular descriptors (hot, dense, frozen)
2. **Meaningful gameplay impact:** Terrain affects movement costs, hiding success, event types/severity, item spawns, and tribute AI decisions
3. **District identity:** Tributes have terrain affinities based on district industry (District 7 lumber → Forest affinity)
4. **User-friendly customization:** Players can configure item quantities, distributions, tribute stats, and event frequency using descriptive labels (sparse/common/abundant) instead of exact numbers
5. **Narrative richness:** Terrain-aware descriptions replace generic text ("struggles through the scorching desert" vs "moves to North")

## Non-Goals

- Player-controlled terrain selection (future: advanced option)
- Preset arena themes (future enhancement)
- Fully embedded terrain configuration files (data-driven design, but implemented in code initially)
- Changes to tribute count (stays at 24)
- Changes to 5-region arena topology (Cornucopia + N/E/S/W remains)

## Architecture

### Core Data Structures

#### TerrainType
Combines base terrain with optional descriptors.

```rust
pub struct TerrainType {
    pub base: BaseTerrain,
    pub descriptors: Vec<TerrainDescriptor>,
}

pub enum BaseTerrain {
    Clearing,      // Neutral
    Forest,
    Desert,
    Tundra,
    Swamp,
    Mountains,
    UrbanRuins,
    Jungle,
}

pub enum TerrainDescriptor {
    // Temperature
    Hot, Cold, Temperate,
    // Density/Structure
    Dense, Sparse, Open,
    // Moisture
    Wet, Dry,
    // Altitude
    HighAltitude, Lowland,
    // Condition
    Rocky, Sandy, Frozen, Overgrown,
}
```

**Descriptor compatibility rules:**
- Desert cannot be Wet (except during Flood events)
- Tundra must be Cold or Frozen
- Forest can be Wet (rainforest), Dense, Sparse, Temperate, etc.
- System assigns descriptors randomly at game creation, respecting compatibility

#### TerrainConfig
Gameplay properties per terrain.

```rust
pub struct TerrainConfig {
    pub movement_cost: f32,          // 0.5-3.0x stamina multiplier
    pub hiding_modifier: f32,        // -0.5 to +0.5 hide success modifier
    pub visibility: Visibility,      // Exposure level
    pub harshness: Harshness,       // Survival difficulty
    pub item_spawn_modifier: f32,   // 0.5-1.5x item quantity (harsh = fewer)
    pub event_weights: HashMap<AreaEvent, f32>,
    pub item_weights: ItemWeights,
}

pub enum Visibility {
    Exposed,    // Tundra, Desert - hard to hide
    Moderate,   // Clearing, Meadow
    Concealed,  // Forest, Jungle, UrbanRuins - easy to hide
}

pub enum Harshness {
    Mild,       // Clearing, temperate Forest
    Moderate,   // Most terrains
    Harsh,      // Desert, Tundra, Mountains
    Deadly,     // Extreme combos (frozen tundra, scorching desert)
}
```

**Example terrain configs:**

| Terrain | Movement Cost | Visibility | Harshness | Item Modifier |
|---------|--------------|------------|-----------|---------------|
| Clearing | 1.0 | Moderate | Mild | 1.0 |
| Dense Forest | 1.3 | Concealed | Moderate | 1.1 |
| Hot Desert | 2.0 | Exposed | Harsh | 0.6 |
| Frozen Tundra | 2.5 | Exposed | Deadly | 0.5 |
| Urban Ruins | 1.2 | Concealed | Moderate | 1.2 |
| Mountains | 1.8 | Moderate | Harsh | 0.7 |
| Swamp | 1.6 | Moderate | Moderate | 0.9 |

### Event System

#### New AreaEvent Variants

Expand existing events with terrain-specific hazards:

```rust
pub enum AreaEvent {
    // Existing (can happen anywhere)
    Wildfire,
    Flood,
    Earthquake,
    Avalanche,
    Blizzard,
    Landslide,
    Heatwave,
    
    // New terrain-specific
    Sandstorm,      // Desert
    Quicksand,      // Swamp, wet Desert
    Collapse,       // Urban Ruins
    Monsoon,        // Jungle, wet Forest
    Whiteout,       // Tundra (severe blizzard)
    Rockslide,      // Mountains, rocky terrain
}
```

#### Event Severity & Survival

Events have different severity based on terrain:

```rust
pub enum EventSeverity {
    Minor,          // 10-30 damage
    Major,          // 40-70 damage + status effect
    Catastrophic,   // 90+ damage (near-certain death)
}

impl AreaEvent {
    pub fn severity_in_terrain(&self, terrain: &TerrainType) -> EventSeverity {
        match (self, terrain.base) {
            // Terrain-appropriate events are deadlier
            (Wildfire, Forest) => Catastrophic,
            (Wildfire, Desert) => Minor,  // Little fuel
            (Sandstorm, Desert) => Major,
            (Sandstorm, Forest) => Minor,  // Gamemaker chaos
            (Blizzard, Tundra) => Catastrophic,
            (Blizzard, Desert) => Minor,   // Freak weather
            // ... etc
        }
    }
    
    pub fn survival_check(
        &self,
        tribute: &Tribute,
        terrain: &TerrainType,
        rng: &mut Rng
    ) -> EventOutcome {
        let severity = self.severity_in_terrain(terrain);
        let base_damage = match severity {
            Minor => rng.random_range(10..=30),
            Major => rng.random_range(40..=70),
            Catastrophic => rng.random_range(90..=120),
        };
        
        // Modifiers
        let mut roll = rng.roll_d20();
        
        // Terrain affinity helps
        if tribute.has_affinity(terrain.base) {
            roll += 3;
        }
        
        // Relevant items (shelter, water) help
        if self.requires_water() && tribute.has_water() {
            roll += 5;
        }
        
        // Attributes matter
        roll += (tribute.attributes.survival / 10) as i32;
        
        // Determine outcome
        match (severity, roll) {
            (_, r) if r >= 20 => EventOutcome::Unscathed,
            (Catastrophic, r) if r < 10 => EventOutcome::Dead(self.death_type()),
            (Major, r) if r < 8 => EventOutcome::Dead(self.death_type()),
            (_, r) if r >= 15 => EventOutcome::Wounded(base_damage / 2),
            _ => EventOutcome::StatusEffect(self.status_effect()),
        }
    }
}

pub enum EventOutcome {
    Unscathed,
    Wounded(u32),                    // Health damage
    StatusEffect(TributeStatus),     // Burned, Frozen, Poisoned
    Dead(TributeStatus),
}
```

#### Event Selection with Weighted Chaos

95% weighted by terrain, 5% completely random (Gamemaker unpredictability):

```rust
impl TerrainType {
    pub fn select_event(&self, rng: &mut Rng) -> AreaEvent {
        if rng.random_bool(0.05) {
            // 5% chaos - ANY event can happen
            AreaEvent::random()
        } else {
            // 95% terrain-weighted
            let weights = self.event_weights();
            weighted_choice(weights, rng)
        }
    }
}
```

**Example event weights:**

| Event | Forest | Desert | Tundra | Urban Ruins |
|-------|--------|--------|--------|-------------|
| Wildfire | 0.40 | 0.05 | 0.05 | 0.15 |
| Sandstorm | 0.05 | 0.50 | 0.10 | 0.10 |
| Blizzard | 0.05 | 0.05 | 0.50 | 0.05 |
| Collapse | 0.05 | 0.05 | 0.05 | 0.40 |
| Flood | 0.20 | 0.10 | 0.10 | 0.15 |
| Earthquake | 0.15 | 0.15 | 0.15 | 0.10 |

### Stamina-Based Movement System

Replace current movement attribute with stamina pool for all actions.

#### Tribute Stamina

```rust
pub struct Tribute {
    // ... existing fields ...
    pub stamina: u32,              // Current stamina (0-100)
    pub max_stamina: u32,          // Base 100, modified by attributes
    pub terrain_affinity: Vec<BaseTerrain>,
}

pub enum TributeAction {
    Move(Area),        // 20-60 stamina (terrain-dependent)
    Hide,              // 10-25 stamina (visibility-dependent)
    Search,            // 10 stamina
    Attack(Target),    // 25-40 stamina (weapon-dependent)
    Rest,              // 0 stamina, recovers 20
}
```

#### Stamina Cost Calculation

```rust
impl TributeAction {
    pub fn stamina_cost(
        &self,
        tribute: &Tribute,
        current_terrain: &TerrainType,
        context: &ActionContext
    ) -> u32 {
        let base_cost = match self {
            Move(dest) => {
                let target_terrain = context.terrain_at(dest);
                let base = 20;
                let terrain_mult = target_terrain.movement_cost;
                let affinity_mult = if tribute.has_affinity(target_terrain.base) {
                    0.8  // 20% discount
                } else {
                    1.0
                };
                (base as f32 * terrain_mult * affinity_mult) as u32
            }
            Hide => match current_terrain.visibility {
                Visibility::Exposed => 25,     // Hard to hide
                Visibility::Moderate => 15,
                Visibility::Concealed => 10,   // Easy to hide
            }
            Search => 10,
            Attack(_) => 25,  // Base, weapon modifies
            Rest => 0,
        };
        
        // Desperation multiplier
        let desperation = tribute.desperation_level(context);
        (base_cost as f32 * desperation.stamina_multiplier) as u32
    }
}
```

#### Desperation Mechanics

When tributes are desperate (low health, outmatched, cornered), they burn more stamina BUT have better success chances:

```rust
pub struct DesperationBonus {
    pub stamina_multiplier: f32,  // 1.0-1.8x cost
    pub success_bonus: i32,       // +1 to +5 on d20 rolls
}

impl Tribute {
    pub fn desperation_level(&self, context: &ActionContext) -> DesperationBonus {
        let mut multiplier = 1.0;
        let mut bonus = 0;
        
        // Low health = desperate
        if self.health < 30 {
            multiplier += 0.3;
            bonus += 2;
        }
        
        // Outmatched in combat
        if let ActionContext::Combat { opponent } = context {
            let power_gap = opponent.combat_power() - self.combat_power();
            if power_gap > 5 {
                multiplier += (power_gap as f32 * 0.1).min(0.5);
                bonus += 3;
            }
        }
        
        // Last tribute standing (high pressure)
        if context.living_tributes <= 3 {
            multiplier += 0.2;
            bonus += 1;
        }
        
        DesperationBonus {
            stamina_multiplier: multiplier.min(1.8),
            success_bonus: bonus.min(5),
        }
    }
}
```

**Narrative example:**
> "Tribute 5, bleeding and exhausted, summons their last reserves of strength and lands a devastating blow against Tribute 12."

#### Turn Flow

1. Tribute starts turn with full stamina (100)
2. Each action consumes stamina based on terrain/context
3. Can take multiple actions until stamina depleted
4. Stamina resets next turn (modified by wounds/exhaustion)

**Example turn:**
- Tribute in Hot Desert (2.0x movement) with no affinity
- Move to adjacent area: 40 stamina (20 × 2.0)
- Move again: 40 stamina (total 80)
- Hide: 25 stamina (exposed terrain)
- Total: 105 stamina needed, only 100 available → can only do 2 moves OR 1 move + hide

### Tribute Terrain Affinity

#### District-Based Affinities

Each district has a primary terrain affinity based on industry, plus a pool for random bonus affinity:

```rust
pub struct DistrictProfile {
    pub number: u32,
    pub industry: &'static str,
    pub primary_affinity: BaseTerrain,
    pub bonus_affinity_pool: Vec<BaseTerrain>,
}

const DISTRICT_PROFILES: [DistrictProfile; 12] = [
    DistrictProfile {
        number: 1,
        industry: "Luxury",
        primary_affinity: UrbanRuins,
        bonus_affinity_pool: vec![Clearing, Forest],
    },
    DistrictProfile {
        number: 4,
        industry: "Fishing",
        primary_affinity: Swamp,
        bonus_affinity_pool: vec![Jungle, Forest],
    },
    DistrictProfile {
        number: 7,
        industry: "Lumber",
        primary_affinity: Forest,
        bonus_affinity_pool: vec![Jungle, Mountains],
    },
    DistrictProfile {
        number: 10,
        industry: "Livestock",
        primary_affinity: Clearing,
        bonus_affinity_pool: vec![Forest, Desert],
    },
    DistrictProfile {
        number: 11,
        industry: "Agriculture",
        primary_affinity: Clearing,
        bonus_affinity_pool: vec![Forest, Jungle],
    },
    DistrictProfile {
        number: 12,
        industry: "Mining",
        primary_affinity: Mountains,
        bonus_affinity_pool: vec![UrbanRuins, Tundra],
    },
    // ... complete all 12 districts
];
```

#### Affinity Assignment

At tribute creation:

```rust
impl Tribute {
    pub fn assign_terrain_affinity(&mut self, rng: &mut Rng) {
        let profile = &DISTRICT_PROFILES[self.district as usize];
        
        // Always get primary affinity
        self.terrain_affinity.push(profile.primary_affinity);
        
        // 40% chance for random bonus affinity
        if rng.random_bool(0.4) {
            if let Some(bonus) = profile.bonus_affinity_pool.choose(rng) {
                self.terrain_affinity.push(*bonus);
            }
        }
    }
}
```

**Example:** District 7 tribute always gets Forest affinity, 40% chance to also get Jungle or Mountains.

#### Affinity Benefits

- **Movement:** 20% stamina reduction in affinity terrain
- **Hiding:** +2 bonus to hide rolls
- **Search:** +2 bonus to search/scavenge rolls
- **Event survival:** +3 bonus to survival checks
- **AI behavior:** Prefers staying in/moving to affinity terrain

**Future enhancement:** Descriptor-specific affinities (District 7 comfortable in dense forests, less so in sparse)

### AI Behavior Modifications

Terrain significantly affects tribute decision-making:

```rust
impl Tribute {
    pub fn choose_action(&mut self, game_state: &Game, rng: &mut Rng) -> TributeAction {
        let current_terrain = self.current_area_terrain(game_state);
        let mut weights = HashMap::new();
        
        // Start with personality-based weights (existing bravery/aggression)
        self.calculate_base_weights(&mut weights);
        
        // Visibility affects hide/attack likelihood
        match current_terrain.visibility {
            Visibility::Concealed => {
                weights[Hide] *= 1.5;   // Forest = good hiding
                weights[Attack] *= 0.8; // Less aggressive
            }
            Visibility::Exposed => {
                weights[Hide] *= 0.5;   // Tundra = poor hiding
                weights[Move] *= 1.3;   // Get to better terrain
            }
            _ => {}
        }
        
        // Harshness prioritizes survival
        if matches!(current_terrain.harshness, Harshness::Harsh | Harshness::Deadly) {
            weights[Search] *= 1.8;  // Need supplies
            weights[Rest] *= 1.2;    // Conserve energy
            weights[Attack] *= 0.6;  // Survival > combat
        }
        
        // Affinity increases confidence
        if self.has_affinity(current_terrain.base) {
            weights[Hide] *= 1.3;    // Confident hiding
            weights[Attack] *= 1.2;  // Confident fighting
            weights[Move] *= 0.8;    // Less urgency to leave
        } else {
            weights[Move] *= 1.4;    // Want better terrain
        }
        
        // Resource-specific priorities
        if current_terrain.base == Desert && !self.has_water() {
            weights[Search] *= 2.0;  // Water critical
        }
        
        weighted_choice(weights, rng)
    }
    
    pub fn choose_destination(&self, game_state: &Game, rng: &mut Rng) -> Option<Area> {
        let neighbors = self.current_area().neighbors();
        let mut scores = HashMap::new();
        
        for area in neighbors {
            let terrain = game_state.terrain_at(area);
            let mut score = 0.0;
            
            // Strong preference for affinity terrain
            if self.has_affinity(terrain.base) {
                score += 50.0;
            }
            
            // Avoid harsh terrain
            score -= match terrain.harshness {
                Harshness::Mild => 0.0,
                Harshness::Moderate => 10.0,
                Harshness::Harsh => 25.0,
                Harshness::Deadly => 40.0,
            };
            
            // Avoid closed/dangerous areas
            if game_state.area_has_events(area) {
                score -= 100.0;
            }
            
            // Seek items if needed
            if self.needs_supplies() && game_state.area_has_items(area) {
                score += 30.0;
            }
            
            // Avoid crowded areas if not brave
            let tribute_count = game_state.tributes_in_area(area).len();
            if tribute_count > 2 && self.attributes.bravery < 50 {
                score -= tribute_count as f32 * 10.0;
            }
            
            scores.insert(area, score);
        }
        
        scores.into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(area, _)| area)
    }
}
```

**Key behaviors:**
- Forest tributes seek forests, avoid harsh terrain
- Desperate tributes prioritize terrain-appropriate resources
- Hiding is strategic in concealed terrain
- Movement avoids hostile terrain when possible
- Brave tributes still fight, but terrain affects engagement decisions

### Item System Changes

#### Terrain-Based Item Distribution

Each terrain has weighted item pools:

```rust
pub struct ItemWeights {
    pub weapons: f32,
    pub shields: f32,
    pub consumables: f32,
}

impl TerrainType {
    pub fn default_item_weights(&self) -> ItemWeights {
        match self.base {
            Desert => ItemWeights {
                weapons: 0.2,
                shields: 0.2,
                consumables: 0.6,  // Water critical
            },
            Tundra => ItemWeights {
                weapons: 0.3,
                shields: 0.4,      // Shelter important
                consumables: 0.3,
            },
            UrbanRuins => ItemWeights {
                weapons: 0.5,      // Scavenged weapons
                shields: 0.3,
                consumables: 0.2,
            },
            Forest => ItemWeights {
                weapons: 0.3,
                shields: 0.2,
                consumables: 0.5,  // Natural resources
            },
            Mountains => ItemWeights {
                weapons: 0.4,
                shields: 0.4,
                consumables: 0.2,
            },
            Swamp => ItemWeights {
                weapons: 0.25,
                shields: 0.25,
                consumables: 0.5,
            },
            Jungle => ItemWeights {
                weapons: 0.2,
                shields: 0.3,
                consumables: 0.5,
            },
            Clearing => ItemWeights {
                weapons: 0.33,
                shields: 0.33,
                consumables: 0.34,  // Balanced
            },
        }
    }
}
```

#### Terrain-Specific Item Variants

Items get thematic names based on terrain (same mechanics, different flavor):

```rust
impl TerrainType {
    fn terrain_specific_consumable(&self, rng: &mut Rng) -> Item {
        match self.base {
            Desert => Item::new_consumable("Cactus Water", healing: 20),
            Forest => Item::new_consumable("Wild Berries", healing: 15),
            Tundra => Item::new_consumable("Preserved Rations", healing: 25),
            Jungle => Item::new_consumable("Tropical Fruit", healing: 18),
            Swamp => Item::new_consumable("Purified Swamp Water", healing: 12),
            UrbanRuins => Item::new_consumable("Canned Food", healing: 22),
            Mountains => Item::new_consumable("Trail Mix", healing: 16),
            Clearing => Item::new_consumable("Bread", healing: 15),
        }
    }
    
    fn terrain_specific_weapon(&self, rng: &mut Rng) -> Item {
        match self.base {
            UrbanRuins => Item::new_weapon("Scavenged Pipe", damage: 15),
            Forest => Item::new_weapon("Wooden Spear", damage: 12),
            // ... falls back to generic if no specific variant
            _ => Item::new_random_weapon(),
        }
    }
}
```

#### Item Spawning

```rust
impl TerrainType {
    pub fn spawn_items(&self, base_count: u32, rng: &mut Rng) -> Vec<Item> {
        let weights = self.default_item_weights();
        
        // Adjust quantity by harshness
        let quantity_modifier = match self.harshness {
            Harshness::Mild => 1.1,
            Harshness::Moderate => 1.0,
            Harshness::Harsh => 0.7,
            Harshness::Deadly => 0.5,
        };
        
        let count = (base_count as f32 * quantity_modifier) as u32;
        
        (0..count).map(|_| {
            let roll = rng.random_range(0.0..1.0);
            if roll < weights.weapons {
                self.terrain_specific_weapon(rng)
            } else if roll < weights.weapons + weights.shields {
                self.terrain_specific_shield(rng)
            } else {
                self.terrain_specific_consumable(rng)
            }
        }).collect()
    }
}
```

**Sponsor items bypass terrain restrictions** - can send any item to any terrain.

### Game Customization Options

User-facing customization through CreateGame DTO (user-friendly labels, no raw numbers):

```rust
#[derive(Serialize, Deserialize, Validate)]
pub struct CreateGame {
    pub name: Option<String>,
    
    // Item configuration
    pub item_quantity: Option<ItemQuantity>,
    pub item_distribution: Option<ItemDistribution>,
    
    // Tribute configuration
    pub starting_health: Option<HealthLevel>,
    pub starting_sanity: Option<SanityLevel>,
    
    // Event frequency
    pub event_frequency: Option<EventFrequency>,
    
    // Future: terrain override (advanced users)
    pub terrain_override: Option<HashMap<Area, TerrainType>>,
}

pub enum ItemQuantity {
    Sparse,    // 1-2 items per area
    Common,    // 3-4 items (default)
    Abundant,  // 5-7 items
}

pub enum ItemDistribution {
    Balanced,         // Respects terrain weights (default)
    WeaponHeavy,      // +40% weapons, -20% shields/consumables
    ConsumableHeavy,  // +40% consumables, -20% weapons/shields
    ShieldHeavy,      // +40% shields, -20% weapons/consumables
}

pub enum HealthLevel {
    Low,       // 30-60 starting health
    Standard,  // 50-100 (default)
    High,      // 70-100
}

pub enum SanityLevel {
    FrequentBreaks,  // 30-60 (tributes panic more)
    Standard,        // 50-100 (default)
    RareBreaks,      // 70-100 (tributes stay calm)
}

pub enum EventFrequency {
    Calm,      // 10% day / 5% night
    Standard,  // 25% day / 12.5% night (default)
    Chaotic,   // 50% day / 25% night
}
```

#### API Implementation

```rust
// api/src/games.rs
pub async fn create_game(
    state: State<AppState>,
    Json(payload): Json<CreateGame>,
) -> Result<Json<Game>, AppError> {
    // ... existing game creation ...
    
    // Apply customization
    let item_count_range = match payload.item_quantity.unwrap_or(ItemQuantity::Common) {
        ItemQuantity::Sparse => 1..=2,
        ItemQuantity::Common => 3..=4,
        ItemQuantity::Abundant => 5..=7,
    };
    
    let health_range = match payload.starting_health.unwrap_or(HealthLevel::Standard) {
        HealthLevel::Low => 30..=60,
        HealthLevel::Standard => 50..=100,
        HealthLevel::High => 70..=100,
    };
    
    // ... apply to tribute/area creation ...
}
```

### Rich Descriptive Text

Replace generic descriptions with terrain-aware narrative:

```rust
impl GameOutput {
    pub fn tribute_moves(
        tribute: &Tribute,
        from: &AreaDetails,
        to: &AreaDetails,
        stamina_used: u32,
    ) -> String {
        let to_terrain = to.terrain_type();
        
        // Select verb based on terrain difficulty
        let verb = match to_terrain.movement_cost {
            c if c >= 2.0 => ["struggles through", "trudges across", "battles through"],
            c if c >= 1.5 => ["pushes through", "navigates", "moves carefully through"],
            _ => ["moves to", "heads toward", "travels to"],
        }.choose(rng);
        
        // Add contextual detail
        let detail = match (tribute.stamina_percent(), to_terrain.harshness) {
            (s, Harshness::Deadly) if s < 0.3 => ", exhausted and desperate",
            (s, Harshness::Harsh) if s < 0.5 => ", struggling against the harsh conditions",
            (s, _) if s < 0.2 => ", nearly spent",
            _ => "",
        };
        
        format!(
            "{} {} the {} to the {}{}",
            tribute.name,
            verb,
            to_terrain.descriptive_name(),  // "scorching desert"
            to.name,  // "Northern Dunes"
            detail
        )
    }
    
    pub fn tribute_hides(tribute: &Tribute, area: &AreaDetails) -> String {
        let terrain = area.terrain_type();
        let hiding_spot = match terrain.base {
            Forest => ["dense underbrush", "hollow tree", "thick foliage"],
            UrbanRuins => ["crumbling building", "collapsed subway", "burned-out vehicle"],
            Mountains => ["rocky outcrop", "cave entrance", "boulder cluster"],
            Desert => ["sand dune", "rocky crevice", "dried riverbed"],
            Tundra => ["snow drift", "ice formation", "frozen outcrop"],
            Swamp => ["tangled roots", "murky water", "dense reeds"],
            Jungle => ["dense canopy", "hanging vines", "fallen tree"],
            Clearing => ["tall grass", "brush pile", "small grove"],
        }.choose(rng);
        
        format!("{} takes cover in the {}", tribute.name, hiding_spot)
    }
    
    pub fn area_event(
        event: &AreaEvent,
        area: &AreaDetails,
        severity: EventSeverity,
    ) -> String {
        let terrain = area.terrain_type();
        let intensity = match severity {
            EventSeverity::Minor => "A mild",
            EventSeverity::Major => "A dangerous",
            EventSeverity::Catastrophic => "A catastrophic",
        };
        
        let event_name = event.terrain_appropriate_name(terrain);
        
        format!("{} {} sweeps through the {}", intensity, event_name, area.name)
    }
}

impl TerrainType {
    pub fn descriptive_name(&self) -> String {
        let descriptor = self.descriptors.first()
            .map(|d| format!("{} ", d.to_string().to_lowercase()))
            .unwrap_or_default();
        
        format!("{}{}", descriptor, self.base.to_string().to_lowercase())
        // Examples: "hot desert", "dense forest", "frozen tundra"
    }
}
```

**Example outputs:**

| Before | After |
|--------|-------|
| "Tribute 5 moves from Cornucopia to North" | "Tribute 5 struggles through the scorching desert to the Northern Dunes, exhausted and desperate" |
| "Tribute 3 hides" | "Tribute 3 takes cover in the dense underbrush" |
| "wildfire in North" | "A catastrophic wildfire sweeps through the Northern Forest" |

### Terrain Assignment at Game Creation

**Default behavior:**
- Each region gets random terrain from available pool
- Cornucopia heavily weighted toward neutral/safe (Clearing, Meadow)
- Descriptors assigned based on compatibility rules
- Ensures variety (no all-Desert games by accident)

```rust
impl Game {
    pub fn assign_terrains(&mut self, rng: &mut Rng) {
        // Cornucopia: 70% neutral, 30% any
        self.areas[0].terrain = if rng.random_bool(0.7) {
            TerrainType::new_safe(rng)  // Clearing, Meadow variants
        } else {
            TerrainType::random(rng)
        };
        
        // Other regions: random
        for area in &mut self.areas[1..] {
            area.terrain = TerrainType::random(rng);
        }
    }
}

impl TerrainType {
    pub fn random(rng: &mut Rng) -> Self {
        let base = BaseTerrain::iter().choose(rng).unwrap();
        let descriptors = Self::compatible_descriptors(base, rng);
        
        TerrainType { base, descriptors }
    }
    
    fn compatible_descriptors(base: BaseTerrain, rng: &mut Rng) -> Vec<TerrainDescriptor> {
        let compatible = match base {
            Desert => vec![Hot, Cold, Rocky, Sandy, HighAltitude],
            Tundra => vec![Cold, Frozen, Rocky, HighAltitude],
            Forest => vec![Dense, Sparse, Wet, Dry, Temperate],
            // ... etc
        };
        
        // Pick 0-2 descriptors randomly
        let count = rng.random_range(0..=2);
        compatible.choose_multiple(rng, count).cloned().collect()
    }
}
```

**Future enhancement:** Player can manually select terrain per region (advanced option in CreateGame DTO).

## Database Schema Changes

### Area Table

Add terrain fields:

```surql
DEFINE TABLE area SCHEMAFULL;

DEFINE FIELD identifier ON area TYPE string;
DEFINE FIELD name ON area TYPE string;
DEFINE FIELD area ON area TYPE option<string>;  // North, South, etc.
DEFINE FIELD base_terrain ON area TYPE string;  // Forest, Desert, etc.
DEFINE FIELD terrain_descriptors ON area TYPE array<string>;  // ["hot", "rocky"]
DEFINE FIELD items ON area TYPE array;
DEFINE FIELD events ON area TYPE array;
```

**Migration:** Existing areas default to `Clearing` with no descriptors.

### Tribute Table

Add affinity fields:

```surql
DEFINE FIELD terrain_affinity ON tribute TYPE array<string>;  // ["forest", "mountains"]
DEFINE FIELD stamina ON tribute TYPE int DEFAULT 100;
DEFINE FIELD max_stamina ON tribute TYPE int DEFAULT 100;
```

**Migration:** Assign affinities to existing tributes based on district.

## Frontend Changes

### Game Creation UI

Extend CreateGameModal with customization options:

```typescript
<select name="itemQuantity">
  <option value="sparse">Sparse (1-2 items per area)</option>
  <option value="common" selected>Common (3-4 items) - Default</option>
  <option value="abundant">Abundant (5-7 items)</option>
</select>

<select name="itemDistribution">
  <option value="balanced" selected>Balanced - Default</option>
  <option value="weaponHeavy">Weapon-Heavy</option>
  <option value="consumableHeavy">Consumable-Heavy</option>
  <option value="shieldHeavy">Shield-Heavy</option>
</select>

<select name="startingHealth">
  <option value="low">Low Health (30-60)</option>
  <option value="standard" selected>Standard Health (50-100) - Default</option>
  <option value="high">High Health (70-100)</option>
</select>

<select name="eventFrequency">
  <option value="calm">Calm (10% events)</option>
  <option value="standard" selected>Standard (25% events) - Default</option>
  <option value="chaotic">Chaotic (50% events)</option>
</select>
```

### Game Display

Show terrain information:

- Area cards display terrain type with icon/color
- Tribute cards show terrain affinity badges
- Event log uses rich terrain descriptions
- Game summary shows arena terrain composition

## Testing Strategy

### Unit Tests

1. **Terrain compatibility validation**
   - Desert cannot be Wet
   - Tundra must be Cold/Frozen
   - Descriptor assignment respects rules

2. **Event severity calculation**
   - Wildfire in Forest = Catastrophic
   - Wildfire in Desert = Minor
   - Weighted selection with 5% chaos

3. **Stamina cost calculation**
   - Movement cost varies by terrain
   - Affinity reduces cost by 20%
   - Desperation multiplies cost appropriately

4. **District affinity assignment**
   - District 7 always gets Forest
   - 40% chance for bonus affinity
   - Bonus comes from correct pool

### Integration Tests

1. **Game creation with customization**
   - Sparse items creates 1-2 per area
   - Low health tributes start with 30-60
   - Event frequency affects cycle outcomes

2. **Terrain-aware AI behavior**
   - Forest tributes seek forests
   - Desperate tributes prioritize resources
   - Hide likelihood increases in concealed terrain

3. **Event survival checks**
   - Affinity bonuses apply correctly
   - Severity affects outcomes
   - Items influence survival

4. **Item distribution**
   - Desert areas have more consumables
   - Urban areas have more weapons
   - Harshness reduces total items

## Implementation Phases

### Phase 1: Core Terrain System (Week 1)
- Add TerrainType, BaseTerrain, TerrainDescriptor enums
- Implement compatibility validation
- Add terrain fields to Area/AreaDetails
- Random terrain assignment at game creation
- Database migration

### Phase 2: Event System (Week 1-2)
- Add new AreaEvent variants
- Implement severity calculation
- Event selection with weighted chaos
- Survival check mechanics
- Update event handling in game cycle

### Phase 3: Stamina & Movement (Week 2)
- Add stamina fields to Tribute
- Implement stamina cost calculation
- Desperation mechanics
- Update turn flow to use stamina
- Remove old movement system

### Phase 4: Tribute Affinity (Week 2-3)
- Define district profiles
- Implement affinity assignment
- Add affinity bonuses to actions
- Update tribute creation flow

### Phase 5: AI Behavior (Week 3)
- Terrain-aware action selection
- Smart destination choosing
- Resource prioritization
- Test AI decision-making

### Phase 6: Item System (Week 3)
- Terrain-specific item weights
- Item variant naming
- Quantity modifiers by harshness
- Update area item spawning

### Phase 7: Game Customization (Week 4)
- Extend CreateGame DTO
- Add validation
- Implement API changes
- Frontend UI updates

### Phase 8: Rich Descriptions (Week 4)
- Terrain-aware GameOutput methods
- Descriptive name generation
- Contextual detail selection
- Update all message generation

### Phase 9: Testing & Polish (Week 4)
- Comprehensive unit tests
- Integration test suite
- Balance tuning
- Bug fixes

**Total estimate:** 4 weeks

## Success Criteria

1. **Terrain variety:** Games generate with diverse, appropriate terrain combinations
2. **Gameplay impact:** Terrain visibly affects movement, hiding, events, and AI behavior
3. **District identity:** Tributes show preference for affinity terrain and perform better there
4. **User customization:** Players can easily configure games with descriptive options
5. **Narrative quality:** Game logs are rich, varied, and terrain-appropriate
6. **Balance:** No terrain is dominant; each has strengths and weaknesses
7. **Performance:** Terrain calculations don't slow down simulation
8. **Backward compatibility:** Existing games migrate cleanly to new system

## Future Enhancements

- **Player terrain selection:** Advanced option to manually configure each region
- **Arena themes:** Preset terrain bundles ("Desert Arena", "Frozen Wasteland")
- **Descriptor-specific affinity:** District 7 comfortable in dense forests, less so in sparse
- **Weather systems:** Dynamic weather affecting multiple regions
- **Terrain evolution:** Events permanently alter terrain (wildfire → burned forest)
- **Verticality:** Height/depth mechanics for Mountains/Caves
- **Water terrain:** Lakes, rivers as distinct region types
