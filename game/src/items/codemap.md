# game/src/items/

## Responsibility
Provides the item system for game equipment and consumables. Defines item types (weapons, shields, consumables), their attributes (health, strength, defense, etc.), and procedural generation logic. Implements the `OwnsItems` trait for inventory management polymorphism across tributes and areas. Handles item lifecycle from creation to consumption.

## Design Patterns

### **Factory Pattern**
- Static factory methods for item creation:
  - `new_weapon(name)`, `new_random_weapon()` - Strength-based melee weapons
  - `new_shield(name)`, `new_random_shield()` - Defense-based protective gear
  - `new_consumable(name)`, `new_random_consumable()` - Stat-boosting consumables
  - `new_random(name)` - Meta-factory delegating to specialized factories
- Each factory encapsulates RNG logic for effect values (weapons: 1-5, shields: 1-7, consumables: 1-10)

### **Strategy Pattern (via Attributes)**
- `Attribute` enum defines 7 different item effects: Health, Sanity, Movement, Bravery, Speed, Strength, Defense
- `ConsumableAttribute` trait maps attributes to themed consumable names (Health → "health kit", Bravery → "yayo")
- Item behavior determined by `(ItemType, Attribute)` tuple, not inheritance

### **Trait Object Pattern**
- `OwnsItems` trait provides polymorphic inventory interface
- Implemented by both `Tribute` (in `tributes` module) and `AreaDetails` (in `areas` module)
- Enables generic item management without coupling to specific owner types

### **Value Object**
- `Item` struct is cloneable with UUID-based identity (`identifier: String`)
- Implements `Display`, `PartialEq`, `Serialize`, `Deserialize` for ergonomic usage
- Immutable after creation except for quantity changes during consumption

### **Type State Pattern (Implicit)**
- `is_weapon()`, `is_defensive()`, `is_consumable()` methods encode type-checking logic
- Different item types have different valid attribute ranges (e.g., weapons only have Strength/Defense)

## Data & Control Flow

### **Item Creation Pipeline**
```
new_random(name)
  ├─> ItemType::random() [50/50 Consumable/Weapon]
  ├─> if Weapon:
  │     ├─> random_bool(0.5) [shield vs weapon]
  │     ├─> if weapon:
  │     │     └─> new_weapon(name) OR new_random_weapon()
  │     │           └─> generate_weapon_name() → "sharp sword", "iron dagger", etc.
  │     └─> if shield:
  │           └─> new_shield(name) OR new_random_shield()
  │                 └─> generate_shield_name() → "iron shield", "wooden shield", etc.
  └─> if Consumable:
        └─> new_consumable(name) OR new_random_consumable()
              ├─> Attribute::random() [uniform across 7 attributes]
              └─> attribute.consumable_name() → "health kit", "trail mix", etc.
```

### **Attribute Effect Ranges**
- **Weapons** (Strength): effect ∈ [1, 5] - Adds damage to attacks
- **Shields** (Defense): effect ∈ [1, 7] - Reduces incoming damage
- **Consumables**: effect ∈ [1, 10] - Variable based on attribute:
  - Health/Sanity: restores stat points
  - Movement/Speed: increases movement range/priority
  - Bravery/Strength: combat bonuses
  - Defense: temporary damage reduction

### **Inventory Operations**
```
OwnsItems trait contract:
  add_item(item)     → Append to internal Vec<Item>
  has_item(&item)    → iter().any(|i| i == item)
  use_item(&item)    → swap_remove() if quantity > 0, else ItemNotFound
  remove_item(&item) → find by identifier, remove(), return Ok/ItemNotFound
```

### **Item Lifecycle**
1. **Creation**: `Game::start()` spawns items in Cornucopia via `Item::new_random()`
2. **Acquisition**: Tribute takes item from area → `area.remove_item()` + `tribute.add_item()`
3. **Usage**: Tribute uses consumable → `tribute.use_item()` applies effect, decrements quantity
4. **Death Transfer**: Dead tribute's items → `area.add_item()` for each item in inventory
5. **Persistence**: Items remain in world until consumed or game ends

## Integration Points

### **Consumed By**
- **tributes module**: 
  - `Tribute` implements `OwnsItems` trait
  - `Action::TakeItem(Item)` references items for acquisition
  - `Action::UseItem(Attribute)` triggers consumable usage
- **areas module**:
  - `AreaDetails` implements `OwnsItems` trait
  - Areas store items in `items: Vec<Item>` field
- **games.rs**:
  - Initializes Cornucopia with random items during `Game::start()`
  - Transfers items from dead tributes to areas in `clean_up_recent_deaths()`

### **Depends On**
- **name_generator module**: `generate_weapon_name()`, `generate_shield_name()` for procedural naming
- **External Crates**:
  - `rand` - RNG for effect values and type selection
  - `serde` - Serialization for API exposure
  - `strum` - `EnumIter` for iterating attributes
  - `uuid` - Unique identifiers for items
  - `thiserror` - Custom error types (`ItemError`)

### **Data Structures**
- `Item`: ~120 bytes (String×2 for identifier/name, enums, u32, i32)
- `ItemType`: 1 byte enum (Consumable | Weapon)
- `Attribute`: 1 byte enum (7 variants)
- `ItemError`: Lightweight error enum (3 variants, no data)

## Key Files

### **mod.rs** (454 lines)
- **Purpose**: Core item types, factory methods, and trait definitions
- **Key Types**:
  - `Item` struct - Fields: `identifier`, `name`, `item_type`, `quantity`, `attribute`, `effect`
  - `ItemType` enum - Consumable | Weapon (shields are weapons with Defense attribute)
  - `Attribute` enum - 7 stat categories (Health, Sanity, Movement, Bravery, Speed, Strength, Defense)
  - `ItemError` enum - ItemNotFound | ItemNotUsable | InvalidAttribute
- **Traits**:
  - `OwnsItems` - Inventory management interface (4 methods)
  - `ConsumableAttribute` - Maps attributes to consumable names
- **Factory Methods**:
  - `new(name, type, qty, attr, effect)` - Generic constructor with UUID generation
  - `new_random(name)` - Randomizes type and delegates to specialized factory
  - `new_weapon/shield/consumable(name)` - Type-specific with RNG effects
  - `new_random_*()` - Calls name generator + delegates to named constructor
- **Type Checks**: `is_weapon()`, `is_defensive()`, `is_consumable()` - Predicate methods for item categorization
- **Testing**: 24 unit tests using `rstest` for parameterized validation
- **Design Notes**:
  - Shields are weapons with `Attribute::Defense` (no separate ShieldType)
  - Quantity always initialized to 1 (multi-quantity items not implemented)
  - `use_item()` in trait checks `quantity > 0` but quantity never decrements (dead code path)

### **name_generator.rs** (56 lines)
- **Purpose**: Procedural name generation for weapons and shields
- **Data**: Static string slices embedded in source:
  - `SHIELD_ADJECTIVES`: 7 materials (iron, wooden, brass, bronze, glass, steel, stone)
  - `WEAPON_NOUNS`: 9 weapon types (sword, spear, dagger, knife, net, trident, bow, mace, axe)
  - `WEAPON_ADJECTIVES`: 12 descriptors (sharp, heavy, long, short, + materials)
- **API**:
  - `generate_shield_name() -> String` - Formats as "{adjective} shield"
  - `generate_weapon_name() -> String` - Formats as "{adjective} {noun}"
- **Algorithm**: `rand::SliceRandom::choose()` for uniform random selection
- **Testing**: 2 tests validate name format (contains space, correct word lists)
- **Design Notes**:
  - Simple template-based generation (no Markov chains or LLM)
  - Material adjectives shared between weapons and shields for consistency
  - Could be extended with prefixes/suffixes for rarity tiers

## Notes
- **Quantity Invariant**: All items created with `quantity: 1`, never incremented - stacking not implemented
- **Effect Scaling**: No level/tier system - effect ranges hardcoded per item type
- **Consumable Semantics**: Consumables are one-use (should decrement quantity, but implementation incomplete)
- **Type Safety**: Rust type system prevents invalid attribute/type combos at runtime via factory methods
- **Naming Quirks**: "yayo" (Bravery consumable) is slang for cocaine - humorous tone
- **No Rarity System**: All items equally likely - no legendary/common tiers
- **Thread Safety**: `Item` is `Clone` but not `Sync` (contains non-atomic String)
- **Future Work**: Implement quantity decrementation, add item durability, create rarity/enchantment system
