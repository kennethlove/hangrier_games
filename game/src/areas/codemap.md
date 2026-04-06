# game/src/areas/

## Responsibility
Defines the spatial structure of the game arena and manages area-specific state. Provides the 5-region topology (Cornucopia + 4 cardinal directions), item inventory management per area, and dynamic area closure through environmental hazards. Areas act as containers for items and tributes, with neighbor relationships enforcing movement constraints.

## Design Patterns

### **Flyweight Pattern**
- `Area` enum instances (5 variants) are lightweight value types copied throughout the system
- `AreaDetails` struct holds per-instance state (items, events) while `Area` enum provides shared behavior (neighbors, display)

### **Trait-Based Inventory**
- `OwnsItems` trait (from `items` module) implemented on `AreaDetails`
- Provides polymorphic item management alongside `Tribute` (both can hold items)
- Methods: `add_item()`, `has_item()`, `use_item()`, `remove_item()`

### **State Pattern (via Events)**
- `is_open()` method checks if `events` vec is empty
- Non-empty `events` (Wildfire, Flood, etc.) implicitly closes the area
- Binary state (open/closed) determined by presence/absence of events

### **Value Object**
- `Area` enum is immutable with structural equality (`Eq`, `Ord`, `Hash`)
- Used as HashMap keys in `games.rs` for O(1) area lookups
- Implements `FromStr`, `Display`, `Default`, `EnumIter` for ergonomic usage

## Data & Control Flow

### **Area Topology**
```
Area::neighbors() defines static graph:
    Cornucopia (center)
       /  |  \  \
      /   |   \  \
   North East South West
     |  X   X  |
     | / \ / \ |
     |/   X   \|
  (each cardinal connects to Cornucopia + 2 adjacent cardinals)
```

### **Event Processing**
```
Game cycle:
  ├─> trigger_cycle_events() [in games.rs]
  │     └─> area.events.push(AreaEvent::random())
  ├─> announce_area_events()
  │     └─> if !area.is_open() → add message
  ├─> ensure_open_area()
  │     └─> if all closed → force-clear one area
  └─> run_tribute_cycle()
        └─> tribute.choose_action(EnvironmentContext { closed_areas, ... })
              └─> filters out closed areas from movement options
```

### **Item Lifecycle**
```
Item flow through areas:
  1. Initialization: Game::start() populates Cornucopia with random items
  2. Acquisition: Tribute::TakeItem action → area.remove_item() → tribute.add_item()
  3. Death cleanup: Tribute dies → items dropped into area via area.add_item()
  4. Consumption: Not managed by areas (tributes use items directly)
```

### **Event Closure Mechanism**
- `AreaEvent::random()` generates one of 7 hazards via `strum::IntoEnumIterator` + `rand::choose()`
- Events spawned in `trigger_cycle_events()` with 25% day/12.5% night probability
- `constrain_areas()` manually adds events when <8 tributes alive to force confrontations
- No decay/expiration - events persist until cleared at cycle start (`prepare_cycle()`)

## Integration Points

### **Consumed By**
- **games.rs**: Manages `Vec<AreaDetails>` in `Game` struct
  - `random_open_area()` filters by `is_open()`
  - `run_tribute_cycle()` builds `HashMap<&Area, &AreaDetails>` for fast lookups
- **tributes module**: Uses `Area` enum in `EnvironmentContext` for AI decision-making
  - Tributes have `current_area: Area` field
  - `Action::Move(Area)` specifies destination
- **output.rs**: Formats area names in messages (`GameOutput::DayStart`, `GameOutput::AreaClosed`)

### **Depends On**
- **items module**: `Item` struct, `OwnsItems` trait, `ItemError` enum
- **External Crates**:
  - `serde` - Serialization for API exposure
  - `strum` - `EnumIter` for iterating all areas
  - `uuid` - Unique identifiers for `AreaDetails`

### **Data Structures**
- `Area`: 40 bytes (enum discriminant + padding) - Copy type
- `AreaDetails`: ~144+ bytes (String×2, Option<Area>, Vec<Item>, Vec<AreaEvent>)
- Game holds `Vec<AreaDetails>` with exactly 5 elements (one per Area enum variant)

## Key Files

### **mod.rs** (198 lines)
- **Purpose**: Core area types and inventory management
- **Key Types**:
  - `Area` enum - 5 variants (Cornucopia, North, East, South, West)
  - `AreaDetails` struct - Stateful container with `identifier`, `name`, `area`, `items`, `events`
- **Area API**:
  - `neighbors() -> Vec<Area>` - Returns 3-4 adjacent areas based on static topology
  - `new(name, area)` - Constructor with UUID generation
  - `is_open() -> bool` - Area accessibility check (true if no events)
- **Trait Implementation**: `OwnsItems` for `AreaDetails`
  - `use_item()` uses `swap_remove()` for O(1) removal (order doesn't matter)
  - `remove_item()` uses `iter().position()` + `remove()` for exact matches
- **Testing**: 8 unit tests covering serialization, item management, event effects
- **Design Notes**:
  - `PartialEq<&Area> for Area` enables `area == &area` comparisons (ergonomic for refs)
  - `Default` trait returns `Cornucopia` (logical center/starting point)

### **events.rs** (97 lines)
- **Purpose**: Environmental hazard types
- **Key Type**: `AreaEvent` enum
  - 7 variants: Wildfire, Flood, Earthquake, Avalanche, Blizzard, Landslide, Heatwave
  - Implements `Clone`, `Debug`, `PartialEq`, `Serialize`, `Deserialize`, `EnumIter`
- **API**:
  - `random() -> AreaEvent` - Weighted random selection via `strum::IntoEnumIterator`
  - `FromStr`/`Display` for case-insensitive string conversion
- **Testing**: 4 tests using `rstest` for parameterized to_string/from_str validation
- **Design Notes**:
  - Events are pure data (no behavior) - game logic interprets them as area closures
  - No severity/duration fields - all events treated equally (binary open/closed)
  - Display uses lowercase ("wildfire", not "Wildfire") for consistent messaging

## Notes
- **Topology Hardcoded**: `neighbors()` uses match statements, not a graph data structure - simple but inflexible for dynamic arenas
- **Event Semantics**: Events block entry but don't damage tributes inside - they must leave voluntarily
- **Item Ownership**: Areas don't track item quantity changes (consumables managed by tributes)
- **No Multi-Area Events**: Events affect single areas, not regions (e.g., no "forest fire spreads to adjacent areas")
- **Thread Safety**: `AreaDetails` is not `Sync` (items vec not shareable) but entire game runs single-threaded
- **Future Work**: Consider graph-based topology for modular arena designs, event TTL/decay system
