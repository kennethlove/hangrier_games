# game/src/

## Responsibility
Core game engine implementing the Hunger Games simulation. This directory provides pure Rust business logic with no I/O dependencies - all game state management, turn-based cycle execution, tribute lifecycle, area event handling, and message generation. The engine is deterministic and stateless except for message accumulation, making it suitable for both single-run simulations and stateful API integration.

## Design Patterns

### **Event Sourcing (Partial)**
- `messages.rs` implements a global message queue (`GLOBAL_MESSAGES`) that captures all game events chronologically
- Messages tagged by source (`MessageSource` enum: Game/Area/Tribute) enable event replay and audit trails
- `GameMessage` struct with nanosecond timestamps provides temporal ordering

### **State Machine**
- `Game` struct manages game lifecycle through `GameStatus` enum transitions: `NotStarted -> InProgress -> Finished`
- `Tribute` statuses follow deterministic state transitions: `Healthy -> RecentlyDead -> Dead`
- Turn phases enforce sequential execution: prepare → announce → execute → cleanup

### **Strategy Pattern**
- `Action` enum (in `tributes` module) encapsulates different tribute behaviors
- `ActionSuggestion` allows external influence on AI decisions (e.g., Feast Day bias)
- `EnvironmentContext` and `EncounterContext` provide decision-making inputs

### **Template Method**
- `Game::run_day_night_cycle()` defines the skeletal algorithm:
  1. Check for winner
  2. Prepare cycle (`prepare_cycle`)
  3. Announce start (`announce_cycle_start`)
  4. Execute cycle (`do_a_cycle`)
  5. Announce end (`announce_cycle_end`)
  6. Clean up deaths (`clean_up_recent_deaths`)
- Subphases (area events, tribute processing) vary while overall flow remains constant

### **Builder Pattern (Implicit)**
- `Game::default()` uses `WPGen` to generate random game names
- `Tribute::new()` and `Item::new_random_*()` methods construct entities with sensible defaults

### **Singleton (Anti-Pattern, Pragmatic)**
- `GLOBAL_MESSAGES` static with `Lazy<Mutex<VecDeque<GameMessage>>>` provides thread-safe global state
- Necessary for pure-function game logic to emit events without explicit dependency injection

## Data & Control Flow

### **Input Boundary**
- **Entry Points**: 
  - `Game::new(name)` - Creates game instance
  - `Game::start()` - Initializes simulation
  - `Game::run_day_night_cycle(is_day: bool)` - Advances one half-day cycle
- **Configuration**: Constants define game rules (`LOW_TRIBUTE_THRESHOLD`, `FEAST_WEAPON_COUNT`, etc.)

### **Execution Flow**
```
run_day_night_cycle(day: bool)
  ├─> check_for_winner()
  │     └─> [adds winner/no-winner messages if game over]
  ├─> prepare_cycle(day)
  │     ├─> clear_messages() [if day cycle]
  │     ├─> increment day counter [if day cycle]
  │     └─> clear area events
  ├─> announce_cycle_start(day)
  │     └─> add_game_message(...) [day/night start, special events]
  ├─> do_a_cycle(day)
  │     ├─> announce_area_events() [closed areas and their hazards]
  │     ├─> ensure_open_area() [guarantee at least one safe zone]
  │     ├─> trigger_cycle_events(day, rng)
  │     │     ├─> spawn random AreaEvents [1/4 day, 1/8 night frequency]
  │     │     └─> Feast Day logic [day 3: refill Cornucopia]
  │     ├─> constrain_areas(rng) [close areas if <8 tributes alive]
  │     └─> run_tribute_cycle(day, rng, ...)
  │           ├─> pre-compute ActionSuggestions [day 1: Move, day 3: Cornucopia]
  │           ├─> build area/tribute lookup HashMaps [optimization]
  │           └─> for each tribute:
  │                 ├─> apply random TributeEvent [based on luck]
  │                 ├─> build EnvironmentContext [area details, closed areas]
  │                 ├─> build EncounterContext [nearby tributes, targets]
  │                 └─> tribute.process_turn_phase(...) [delegates to tributes module]
  ├─> announce_cycle_end(day)
  │     ├─> add_game_message(tributes_left)
  │     ├─> announce recently dead tributes
  │     └─> add_game_message(day/night end)
  └─> clean_up_recent_deaths()
        ├─> set day_killed statistics
        ├─> drop tribute items into their area
        └─> transition RecentlyDead -> Dead
```

### **Output Boundary**
- **Message System**: `messages.rs` functions (`add_game_message`, `add_area_message`, `add_tribute_message`) append to `GLOBAL_MESSAGES`
- **Queries**: 
  - `get_all_messages()` - Full event log
  - `get_messages_by_source(source)` - Filtered by source type
  - `get_messages_by_day(day)` - Filtered by game day
- **State Inspection**: `Game::living_tributes()`, `Game::winner()`, public fields (`status`, `day`, `tributes`, `areas`)

### **Random Number Generation**
- `SmallRng` from `rand` crate seeded per cycle (`SmallRng::from_rng(&mut rand::rng())`)
- Used for: area selection, event triggering, tribute shuffling, tribute AI decisions

## Integration Points

### **Consumed By**
- **API Crate** (`api/`): REST endpoints call `Game` methods to advance simulation and query state
- **Announcers Crate** (`announcers/`): Consumes messages from `GLOBAL_MESSAGES` to generate LLM commentary
- **Frontend** (`web/`): Indirectly via API - displays game state and messages

### **Depends On**
- **Modules (within `game/src/`)**:
  - `areas` - `Area` enum, `AreaDetails` struct, `AreaEvent` enum
  - `tributes` - `Tribute` struct, `TributeStatus`/`TributeEvent` enums, `Action` logic
  - `items` - `Item` struct, `OwnsItems` trait
  - `threats` - Animal encounters (referenced in `output.rs`)
  - `witty_phrase_generator` - Random name generation for games
- **External Crates**:
  - `rand` - RNG for procedural generation
  - `serde` - Serialization for API exposure
  - `shared` - Cross-crate types (`GameStatus`)
  - `uuid` - Unique identifiers for games and messages
  - `chrono` - Timestamps for messages
  - `once_cell` - Lazy static initialization of `GLOBAL_MESSAGES`

## Key Files

### **lib.rs** (8 lines)
Module aggregator. Exports all submodules and declares `witty_phrase_generator` as private.

### **games.rs** (863 lines) - **Core Engine**
- **Purpose**: Game state container and cycle orchestration
- **Key Struct**: `Game` 
  - Fields: `identifier`, `name`, `status`, `day`, `areas`, `tributes`, `private`
  - Implements: `Default`, `Display`
- **Game Lifecycle**: `start()`, `end()`, `run_day_night_cycle()`
- **State Queries**: `living_tributes()`, `winner()`, `random_open_area()`
- **Event Logic**:
  - `trigger_cycle_events()` - Spawns area hazards, handles Feast Day
  - `constrain_areas()` - Dynamically closes areas to force tribute encounters
  - `run_tribute_cycle()` - Delegates to `Tribute::process_turn_phase()`
- **Testing**: 18 unit tests covering lifecycle, state transitions, area management

### **messages.rs** (119 lines) - **Event Log System**
- **Purpose**: Global message accumulation and retrieval
- **Key Types**:
  - `MessageSource` enum - Discriminates event origin (Game/Area/Tribute)
  - `GameMessage` struct - Event record with ID, source, day, subject, timestamp, content
- **Global State**: `GLOBAL_MESSAGES` (thread-safe `VecDeque<GameMessage>`)
- **API**:
  - Write: `add_message()`, `add_game_message()`, `add_area_message()`, `add_tribute_message()`
  - Read: `get_all_messages()`, `get_messages_by_source()`, `get_messages_by_day()`
  - Maintenance: `clear_messages()` (called at day start)
- **Thread Safety**: `Mutex` guards ensure concurrent access safety (future-proofing for multi-threaded API)

### **output.rs** (301 lines) - **Presentation Layer**
- **Purpose**: Human-readable message formatting
- **Key Type**: `GameOutput<'a>` enum
  - 79 variants covering all game events (day start, attacks, deaths, area events, etc.)
  - Implements `Display` trait with emoji-rich formatting
- **Formatting Utilities**: Uses `indefinite` crate for article insertion ("a sword", "an axe")
- **Integration**: `games.rs` calls `format!("{}", GameOutput::*)` then passes strings to `add_*_message()`
- **Design Note**: Decouples game logic from presentation - could swap to different languages/styles without touching `games.rs`

### **witty_phrase_generator/mod.rs** (205 lines) - **Name Generator**
- **Purpose**: Procedural game name generation using word combinations
- **Key Struct**: `WPGen`
  - Loads wordlists from embedded text files (`intensifiers.txt`, `adjectives.txt`, `nouns.txt`)
  - Methods: `with_words(n)`, `generic(...)`, `with_phrasewise_alliteration(...)`
- **Algorithm**: Backtracking constraint solver for length/alliteration requirements
- **Usage**: `Game::default()` calls `WPGen::new().with_words(3)` → e.g., "mighty-purple-dragon"
- **Data Files**:
  - `intensifiers.txt` - 194KB, ~4000 entries
  - `adjectives.txt` - 45KB, ~1500 entries  
  - `nouns.txt` - 8KB, ~500 entries

## Notes
- **Pure Logic**: No file I/O, networking, or database access - all side effects are message emissions
- **Testability**: 60+ inline tests across modules using `rstest` parameterized testing
- **Performance**: `run_tribute_cycle()` uses `HashMap` lookups instead of nested loops (O(n²) → O(n))
- **Statefulness**: Only `GLOBAL_MESSAGES` persists between function calls; `Game` struct is fully serializable
- **Future Work**: Consider extracting message system to event bus pattern for pluggable observers
