# game/src/threats/

## Responsibility
Defines hostile wildlife encounters for the game simulation. Provides a taxonomy of 15 animal types with damage values and display logic. Animals serve as environmental hazards triggered by tribute events, adding unpredictable danger beyond tribute-vs-tribute combat.

## Design Patterns

### **Value Object**
- `Animal` enum is a pure data type with no mutable state
- Implements `Clone`, `Copy`, `Eq`, `Ord`, `PartialEq`, `PartialOrd` for value semantics
- Used as discriminator in game events, not as stateful entities

### **Strategy Pattern (via Enum Methods)**
- `damage() -> u32` method encodes different threat levels per animal type
- Replaces inheritance hierarchy with match-based dispatch
- Damage values range 1-20 (Squirrel to Hippo)

### **Data-Driven Design**
- Animal behavior defined purely through data (damage values, display names)
- No complex AI or stateful behavior - animals are event triggers, not actors
- Game logic in `tributes` module interprets animal encounters, not `threats` module

### **Flyweight Pattern (Implicit)**
- Enum variants have no per-instance data (zero-sized types)
- `SmallRng` created temporarily in `random()`, not stored
- Memory footprint: 1 byte per animal reference (enum discriminant)

## Data & Control Flow

### **Animal Selection**
```
Animal::random()
  └─> SmallRng::from_rng(&mut rand::rng())
        └─> Animal::iter() [strum::EnumIter]
              └─> choose(&mut rng) [rand::SliceRandom]
                    └─> Returns one of 15 variants uniformly
```

### **Damage Tiers**
```
Low Threat (1-3 damage):
  ├─> Squirrel (1)
  ├─> Snake (2)
  └─> Boar, Monkey (3)

Medium Threat (5 damage):
  ├─> Wolf, Cougar, Baboon, Hyena, TrackerJacker

High Threat (10 damage):
  ├─> Bear, Lion, Tiger, Elephant, Rhino

Critical Threat (20 damage):
  └─> Hippo [most dangerous]
```

### **Event Integration**
```
Game cycle (in tributes module):
  ├─> TributeEvent::random() spawned based on tribute luck
  ├─> if TributeEvent::AnimalAttack:
  │     ├─> Animal::random() selects species
  │     ├─> animal.damage() calculates harm
  │     └─> tribute.health -= damage (minus defense modifiers)
  └─> GameOutput::AnimalAttack(tribute, animal) formats message
```

### **Pluralization Logic**
- `plural() -> String` handles irregular forms:
  - "wolf" → "wolves" (special case)
  - Everything else: "{animal}s" (e.g., "bears", "tracker jackers")
- Used in message formatting for multi-animal encounters (future feature?)

## Integration Points

### **Consumed By**
- **tributes module**: 
  - `TributeEvent` enum likely includes `AnimalAttack(Animal)` variant
  - Tribute processing applies `animal.damage()` to health stat
- **output.rs**:
  - `GameOutput` enum formats animal encounters for messages
  - Uses `animal.to_string()` and `animal.plural()` in display logic
- **games.rs** (indirectly):
  - Animal attacks triggered during `run_tribute_cycle()`
  - Messages added to `GLOBAL_MESSAGES` via `add_tribute_message()`

### **Depends On**
- **External Crates**:
  - `rand` - Random animal selection (`SmallRng`, `choose()`)
  - `serde` - Serialization for API exposure
  - `strum` - `EnumIter` for iterating all animal types

### **Data Structures**
- `Animal` enum: 1 byte (discriminant only, no variant data)
- Default: `Squirrel` (least threatening, logical fallback)
- Total variants: 15 (0-14 discriminant values)

## Key Files

### **mod.rs** (1 line)
- **Purpose**: Module declaration
- **Content**: `pub(crate) mod animals;`
- **Visibility**: `pub(crate)` restricts animals to `game` crate only
- **Design Note**: Could export `Animal` directly here instead of forcing `use threats::animals::Animal`

### **animals.rs** (192 lines)
- **Purpose**: Animal type definitions and behavior
- **Key Type**: `Animal` enum
  - 15 variants: Squirrel, Bear, Wolf, Cougar, Boar, Snake, Monkey, Baboon, Hyena, Lion, Tiger, Elephant, Rhino, Hippo, TrackerJacker
  - Derives: `Clone`, `Debug`, `Default`, `Deserialize`, `EnumIter`, `Eq`, `Ord`, `PartialEq`, `PartialOrd`, `Serialize`
- **API**:
  - `plural() -> String` - Handles irregular plurals (wolf → wolves)
  - `random() -> Animal` - Uniform random selection
  - `damage() -> u32` - Threat level (1-20)
- **String Conversion**:
  - `FromStr` - Case-insensitive parsing ("BEAR" → `Animal::Bear`)
  - `Display` - Lowercase output ("bear", not "Bear")
  - `TrackerJacker` special case: "tracker jacker" (two words)
- **Testing**: 5 tests using `rstest`:
  - 15×3 parameterized tests for to_string/from_str/damage validation
  - Random animal test (ensures valid variant)
  - Plural forms test (wolf edge case + general case)
- **Design Notes**:
  - TrackerJacker is a Hunger Games reference (genetically engineered wasps)
  - Damage values not scientifically accurate (hippos > elephants is correct though)
  - No speed/aggression stats - only damage matters

## Notes
- **Minimal Module**: Only 1 file (animals.rs) and 1 type (Animal enum) - intentionally simple
- **No Stateful Encounters**: Animals don't persist between events - each attack spawns fresh instance
- **Uniform Probability**: All 15 animals equally likely - no biome/area-specific fauna
- **Future Extensibility**: Could add animal packs (multiple animals in one encounter), animal items (pelts/meat), or area-specific spawning
- **Naming Collision**: `animals.rs` uses same name as module - could rename to `types.rs` for clarity
- **Missing Features**: No evasion/dodge mechanics, no animal loot drops, no taming/befriending
- **Thread Safety**: `Animal` is `Sync + Send` (Copy type with no interior mutability)
- **Future Work**: Add animal behaviors (aggressive vs defensive), biome-specific fauna (bears in North, snakes in South), animal packs with combinatorial damage
