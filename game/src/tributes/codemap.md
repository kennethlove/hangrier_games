# game/src/tributes/

## Responsibility

The `tributes` module implements the core game characters and their behavior in the Hunger Games simulation. It manages:

- **Tribute entities**: Character state, attributes, and statistics
- **AI decision-making**: Context-aware action selection based on health, sanity, enemies, and inventory
- **Combat mechanics**: Attack resolution using d20-style dice rolls with equipment modifiers
- **Status effects**: Environmental and combat-induced conditions (wounded, poisoned, frozen, etc.)
- **Action execution**: Turn-based processing including movement, combat, item usage, hiding, and resting
- **Survival mechanics**: Health/sanity management, sponsor gifts, and environmental hazards

This is the behavioral heart of the simulation—tributes are autonomous agents that react to their environment, fight for survival, and ultimately determine game outcomes.

## Design

### Design Patterns

**1. Entity-Component Pattern (Tribute Structure)**
```rust
pub struct Tribute {
    identifier: String,
    area: Area,
    status: TributeStatus,
    brain: Brain,              // AI decision-maker
    attributes: Attributes,    // Health, sanity, strength, etc.
    statistics: Statistics,    // Combat history
    items: Vec<Item>,          // Inventory
    events: Vec<TributeEvent>, // Event history
}
```

Tributes are composed of distinct components (brain, attributes, items) rather than using inheritance. This allows flexible composition and clear separation of concerns.

**2. Strategy Pattern (Brain AI)**
```rust
impl Brain {
    pub fn act(&self, tribute: &Tribute, nearby_tributes: u32, rng: &mut impl Rng) -> Action {
        // Preferred action override
        if let Some(ref preferred_action) = self.preferred_action {
            if rng.random_bool(self.preferred_action_percentage) {
                return preferred_action.clone();
            }
        }
        
        // Context-based decision strategies
        if nearby_tributes == 0 {
            self.decide_action_no_enemies(tribute)
        } else if nearby_tributes < LOW_ENEMY_LIMIT {
            self.decide_action_few_enemies(tribute)
        } else {
            self.decide_action_many_enemies(tribute)
        }
    }
}
```

The `Brain` uses different decision strategies based on context (alone, few enemies, many enemies), with hierarchical fallbacks and state-based rules.

**3. State Machine (Status Processing)**
```rust
fn process_status(&mut self, area_details: &AreaDetails, rng: &mut impl Rng) {
    match &self.status {
        TributeStatus::Wounded => { self.takes_physical_damage(WOUNDED_DAMAGE); }
        TributeStatus::Sick => {
            self.reduce_strength(SICK_STRENGTH_REDUCTION);
            self.reduce_speed(SICK_SPEED_REDUCTION);
        }
        TributeStatus::Broken => {
            // Random bone break location affects different stats
            let bone = rng.random_range(0..4);
            match bone {
                0 => self.reduce_speed(BROKEN_BONE_LEG_SPEED_REDUCTION),
                1 => self.reduce_strength(BROKEN_BONE_ARM_STRENGTH_REDUCTION),
                2 => self.reduce_intelligence(BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION),
                _ => self.reduce_dexterity(BROKEN_BONE_RIB_DEXTERITY_REDUCTION),
            }
        }
        // ... other statuses
    }
}
```

Status effects are processed as state transitions with status-specific damage calculations.

**4. Command Pattern (Actions)**
```rust
pub enum Action {
    None,
    Move(Option<Area>),
    Rest,
    UseItem(Option<Item>),
    Attack,
    Hide,
    TakeItem,
}
```

Actions are first-class values that can be queued, suggested, overridden, and executed independently.

**5. Transaction Pattern (Combat Resolution)**
```rust
fn attacks(&mut self, target: &mut Tribute, rng: &mut impl Rng) -> AttackOutcome {
    // 1. Determine contest result
    match attack_contest(self, target, rng) {
        AttackResult::AttackerWins => { /* apply damage */ }
        AttackResult::DefenderWins => { /* counter-attack */ }
        // ...
    }
    
    // 2. Check for death
    if self.attributes.health == 0 {
        // Attacker died
        AttackOutcome::Kill(target.clone(), self.clone())
    } else if target.attributes.health == 0 {
        // Target died
        AttackOutcome::Kill(self.clone(), target.clone())
    } else {
        // Both survived
        AttackOutcome::Wound(self.clone(), target.clone())
    }
}
```

Combat is a transaction: roll dice, apply effects, determine outcome. Results are categorized as Kill/Wound/Miss.

**6. Trait-Based Composition (OwnsItems)**
```rust
impl OwnsItems for Tribute {
    fn add_item(&mut self, item: Item) { /* ... */ }
    fn use_item(&mut self, item: &Item) -> Result<(), ItemError> { /* ... */ }
    fn remove_item(&mut self, item: &Item) -> Result<(), ItemError> { /* ... */ }
}
```

Tributes implement shared `OwnsItems` trait for inventory management, allowing reuse across `Tribute` and `AreaDetails`.

### Key Architecture Decisions

**Pure Simulation Design**: The module is pure Rust with no I/O dependencies. All effects are deterministic given an RNG seed, enabling replay, fast testing (60+ unit tests), and headless batch processing.

**Separation of Concerns**:
- **Brain**: Pure decision logic (no side effects)
- **Tribute**: State management and action execution
- **Actions/Statuses/Events**: Data types only (no behavior)
- **Logging**: Fire-and-forget via `try_log_action()` (failures don't affect gameplay)

**Attribute Clamping**: All modifiers use saturating arithmetic with min/max bounds:
```rust
self.attributes.health.saturating_sub(damage);  // Never underflow
self.attributes.strength.saturating_add(amount).min(MAX_STRENGTH);  // Respect max
self.attributes.speed.saturating_sub(amount).max(1);  // Never zero
```

**Combat Philosophy**:
- **d20 system**: Base rolls ensure unpredictability
- **Multiplicative scaling**: Decisive wins deal 2x damage
- **Equipment degradation**: Weapons/shields lose durability on use
- **Desensitization**: More combat experience reduces stress per encounter
- **Sanity death spiral**: Low sanity reduces future stress (already broken)

**Serialization Strategy**: `#[serde(skip)]` on `brain` field prevents AI state serialization. Brains are reconstructed as `Brain::default()` on deserialization, ensuring clean AI state between sessions.

## Flow

### Turn Processing Pipeline (Primary Flow)

```
process_turn_phase()
    │
    ├─> 1. Check if alive → return early if dead
    │
    ├─> 2. process_status()
    │       ├─> apply_area_effects() (floods, wildfires, etc.)
    │       └─> Apply status damage (wounded, frozen, sick, etc.)
    │           └─> Check for death from status
    │
    ├─> 3. receive_patron_gift() → add item to inventory
    │
    ├─> 4. Nighttime effects → misses_home() (sanity damage)
    │
    ├─> 5. brain.act() → determine action
    │       ├─> Check preferred action (e.g., forced by game master)
    │       ├─> Prioritize item usage if consumables available
    │       └─> Context-based decision:
    │           ├─> No enemies → Rest/Hide/Move
    │           ├─> Few enemies → Attack/Move/Hide (health-dependent)
    │           └─> Many enemies → Move/Hide/Attack (intelligence-dependent)
    │
    └─> 6. Execute action:
        ├─> Action::Move → travels() → update area
        ├─> Action::Hide → hides() → set is_hidden
        ├─> Action::Rest → long_rests() → restore health/sanity/movement
        ├─> Action::Attack → pick_target() + attacks()
        ├─> Action::TakeItem → take_nearby_item()
        └─> Action::UseItem → try_use_consumable() → apply attribute boost
```

### Combat Resolution Flow

```
attacks(target)
    │
    ├─> 1. Check for self-harm (sanity break)
    │
    ├─> 2. attack_contest()
    │       ├─> Attacker: d20 + strength + weapon effect
    │       ├─> Defender: d20 + defense + shield effect
    │       ├─> Consume weapon/shield durability
    │       └─> Determine winner (normal or decisive)
    │
    ├─> 3. apply_combat_results()
    │       ├─> Apply damage (2x for decisive wins)
    │       ├─> Update statistics (wins/defeats/draws)
    │       ├─> Apply violence stress to winner
    │       └─> Log event
    │
    └─> 4. Return AttackOutcome
        ├─> Kill(winner, loser) → set killed_by, RecentlyDead status
        ├─> Wound(attacker, defender) → both survive
        └─> Miss(attacker, defender) → draw
```

### Violence Stress Calculation

```
apply_violence_stress()
    │
    └─> calculate_violence_stress(kills, wins, sanity)
        ├─> If no wins → BASE_STRESS_NO_ENGAGEMENTS (20.0)
        └─> If has wins:
            ├─> raw_stress = (kills * 50.0) + (non_kill_wins * 20.0)
            ├─> desensitized_stress = raw_stress / total_wins
            └─> final_stress = desensitized_stress * (sanity / 100) / 2
                └─> More wins = less stress per encounter (desensitization)
                └─> Lower sanity = less additional stress (already broken)
```

### Target Selection Logic

```
pick_target(targets, living_tributes_count)
    │
    ├─> No targets available?
    │   └─> Sanity <= 9? → Attack self (suicide)
    │
    ├─> Filter enemies (different district)
    │
    ├─> No enemies in area?
    │   ├─> Only 2 tributes left alive? → Betray ally
    │   └─> Loyalty < 0.25? → Betray ally
    │
    └─> Multiple enemies? → Random selection
```

### AI Decision Tree (Brain)

```
Brain::act()
    │
    ├─> 1. Preferred action set? → Roll probability → Execute if passes
    │
    ├─> 2. Has consumables? → UseItem(None)
    │
    └─> 3. Context-based decision:
        │
        ├─> No enemies nearby:
        │   ├─> Health < 20 → Rest
        │   ├─> Health 20-40 → Hide (if sanity > 20) else Move
        │   └─> Health > 40 → Move (if movement > 0) else Rest
        │
        ├─> Few enemies (< 6):
        │   ├─> Health < 20 → Complex decision based on movement/sanity/visibility
        │   ├─> Health 20-40 → Move (if sanity > 20) else Attack
        │   └─> Health > 40 → Attack
        │
        └─> Many enemies (>= 6):
            ├─> Recklessness = 100 - intelligence - sanity
            ├─> Recklessness < 35 (smart) → Move
            ├─> Recklessness > 80 (dumb) → Attack
            └─> Recklessness 35-80 (average) → Hide
```

## Integration

### Module Dependencies

**Areas Module** (`crate::areas`):
- `Area`: Enum of game locations (Cornucopia, North, South, East, West)
- `AreaDetails`: Contains area-specific items and events
- `AreaEvent`: Environmental hazards (Wildfire, Flood, Earthquake, Avalanche, Blizzard, Landslide, Heatwave)
- **Flow**: `process_status()` → `apply_area_effects()` → sets tribute status based on area events

**Items Module** (`crate::items`):
- `Item`: Weapons, shields, and consumables with effect values
- `Attribute`: Enum of attributes items can affect (Health, Sanity, Movement, etc.)
- `OwnsItems` trait: Shared inventory management
- **Flow**:
  - `receive_patron_gift()` → creates random consumable
  - `take_nearby_item()` → transfers item from area to tribute
  - `try_use_consumable()` → applies item effect and removes from inventory

**Messages Module** (`crate::messages`):
- `add_tribute_message()`: Logs game events to persistent storage
- **Flow**: `try_log_action()` helper wraps every significant action/outcome for game narrative

**Output Module** (`crate::output`):
- `GameOutput`: Enum of formatted game events (attack messages, travel, deaths, etc.)
- **Flow**: All logged events use `GameOutput` variants for consistent formatting

**Threats Module** (`crate::threats::animals`):
- `Animal`: Enum of animals that can maul tributes (Bear, Wolf, etc.)
- **Flow**:
  - `TributeEvent::AnimalAttack(Animal)` sets `TributeStatus::Mauled(Animal)`
  - Damage calculated as `animal.damage() * number_of_animals`

### External Dependencies

- `rand`: RNG for all probabilistic decisions (SmallRng from thread rng)
- `fake`: Generates random tribute names (English locale)
- `uuid`: Creates unique tribute identifiers
- `serde`: Serialization for persistence (skips `brain` field)

### Game Engine Integration (Inbound)

`process_turn_phase()` is called by the game loop with:
- `ActionSuggestion`: Optional forced action with probability
- `EnvironmentContext`: Day/night flag, area details, closed areas
- `EncounterContext`: Nearby tributes, potential targets, total living count

### Key Files

**`mod.rs`** (1,300+ lines) - Core tribute logic
- `Tribute` struct with 60+ methods
- `process_turn_phase()`: Main turn execution (lines 681-835)
- `attacks()`: Combat resolution (lines 336-450)
- `attack_contest()`: D20 dice rolling with modifiers (lines 1116-1178)
- `calculate_violence_stress()`: Stress from combat (lines 1079-1109)
- `travels()`: Movement logic with area validation (lines 457-548)
- `process_status()`: Apply status effect damage (lines 552-616)
- `pick_target()`: Target selection with loyalty/betrayal (lines 949-1018)
- 26 damage/reduction constants (lines 24-57)
- 12 max attribute values (lines 60-71)
- 60+ test functions

**`brains.rs`** (321 lines) - AI decision engine
- `Brain` struct with preferred action override
- `act()`: Main decision function with 3-tier context strategy
- `decide_action_no_enemies()`, `decide_action_few_enemies()`, `decide_action_many_enemies()`
- 8 threshold constants for health/sanity/intelligence
- 20+ test functions covering all decision branches

**`actions.rs`** (121 lines) - Action definitions
- `Action` enum: Move, Rest, UseItem, Attack, Hide, TakeItem, None
- `AttackResult` enum: Combat dice outcomes
- `AttackOutcome` enum: Combat final results (Kill/Wound/Miss)
- Display and FromStr implementations

**`statuses.rs`** (153 lines) - Tribute status effects
- `TributeStatus` enum: 16 status types including Healthy, Wounded, Dead, Mauled(Animal)
- Complex string parsing for "mauled: <animal>" format
- Display and FromStr implementations
- EnumIter support

**`events.rs`** (154 lines) - Tribute events
- `TributeEvent` enum: 12 event types
- `random()`: Generate random event
- Complex string parsing for "animal attack: <animal>" format
- Display and FromStr implementations
