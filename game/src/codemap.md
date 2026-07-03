# game/src/

## Responsibility
Core game engine implementing the Hunger Games simulation. This directory provides pure Rust business logic with no I/O dependencies — all game state management, turn-based cycle execution, tribute lifecycle, area event handling, and message generation. The engine is deterministic and stateless except for message accumulation, making it suitable for both single-run simulations and stateful API integration.

## Design Patterns

### **Event Sourcing (Partial)**
- `messages.rs` implements a global message queue (`GLOBAL_MESSAGES`) that captures all game events chronologically
- Messages tagged by source (`MessageSource` enum: Game/Area/Tribute) enable event replay and audit trails
- `events/` module provides typed `GameEvent` enum as a structured, serde-friendly counterpart to stringly-typed `GameOutput`

### **State Machine**
- `Game` struct manages game lifecycle through `GameStatus` enum transitions: `NotStarted -> InProgress -> Finished`
- `Tribute` statuses follow deterministic state transitions: `Healthy -> RecentlyDead -> Dead` (with `Mauled` variant for animal kills)
- Turn phases enforce sequential execution: prepare → announce → execute → cleanup

### **Strategy Pattern**
- `Action` enum (in `tributes/actions.rs`) encapsulates different tribute behaviors
- `ActionSuggestion` allows external influence on AI decisions (e.g., Feast Day bias)
- `Brain` module uses different decision strategies based on context (enemy count, health, afflictions)

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
  - `Game::new(name)` — Creates game instance
  - `Game::start()` — Initializes simulation
  - `Game::run_day_night_cycle(is_day: bool)` — Advances one half-day cycle
- **Configuration**: `GameConfig` struct centralizes magic numbers (`low_tribute_threshold`, `feast_weapon_count`, etc.) with runtime-tunable knobs

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
  │     ├─> run_tribute_cycle(day, rng, ...)
  │     │     ├─> pre-compute ActionSuggestions [day 1: Move, day 3: Cornucopia]
  │     │     ├─> build area/tribute lookup HashMaps [optimization]
  │     │     └─> for each tribute:
  │     │           ├─> apply random TributeEvent [based on luck]
  │     │           ├─> build EnvironmentContext [area details, closed areas]
  │     │           ├─> build EncounterContext [nearby tributes, targets]
  │     │           └─> tribute.process_turn_phase(...) [delegates to tributes module]
  │     ├─> process_alliance_events() [betrayal cascades, death sanity breaks]
  │     ├─> run_trauma_producers() [acquire/reinforce trauma afflictions]
  │     └─> spawn_sponsors() [one per archetype, idempotent]
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
  - `get_all_messages()` — Full event log
  - `get_messages_by_source(source)` — Filtered by source type
  - `get_messages_by_day(day)` — Filtered by game day
- **State Inspection**: `Game::living_tributes()`, `Game::winner()`, public fields (`status`, `day`, `tributes`, `areas`)

### **Random Number Generation**
- `SmallRng` from `rand` crate seeded per cycle (`SmallRng::from_rng(&mut rand::rng())`)
- Used for: area selection, event triggering, tribute shuffling, tribute AI decisions

## Integration Points

### **Consumed By**
- **API Crate** (`api/`): REST endpoints call `Game` methods to advance simulation and query state
- **Announcers Crate** (`announcers/`): Consumes messages from `GLOBAL_MESSAGES` to generate LLM commentary
- **Browser** (HTMX): Indirectly via API — API renders Maud templates for HTML display

### **Depends On**
- **Modules (within `game/src/`)**:
  - `areas` — `Area` enum, `AreaDetails` struct, `AreaEvent` enum, hex topology, pathfinding graph
  - `tributes` — `Tribute` struct, `TributeStatus`/`TributeEvent` enums, `Action` logic, combat, afflictions, alliances
  - `items` — `Item` struct, `OwnsItems` trait, procedural generation
  - `threats` — Animal encounters (bears, wolves, etc.)
  - `terrain` — `BaseTerrain` enum, `TerrainDescriptor`, `TerrainType`, biome config
  - `events` — Typed `GameEvent` enum (serde-friendly counterpart to `GameOutput`)
  - `phases` — Per-phase pipeline scaffolding (environmental conditions, light levels)
  - `sponsors` — Sponsor archetypes, budget bands, affinity tracking
  - `config` — `GameConfig` struct with runtime-tunable game constants
  - `districts` — 12 district profiles with industry and terrain affinities
  - `pathfinding` — Generic A* graph pathfinding
  - `witty_phrase_generator` — Random name generation for games
- **External Crates**:
  - `rand` — RNG for procedural generation
  - `serde` — Serialization for API exposure
  - `shared` — Cross-crate types (`GameStatus`, `GameEvent`, `CombatBeat`, `Affliction`, etc.)
  - `uuid` — Unique identifiers for games and messages
  - `chrono` — Timestamps for messages
  - `once_cell` — Lazy static initialization of `GLOBAL_MESSAGES`

## Key Files

### **lib.rs** (18 lines)
Module aggregator. Exports all submodules and declares `witty_phrase_generator` as private. Re-exports key terrain types (`BaseTerrain`, `TerrainDescriptor`, `TerrainType`).

### **games/mod.rs** (980 lines) — **Core Game State**
- **Purpose**: `Game` struct definition, lifecycle methods, state queries
- **Key Struct**: `Game`
  - Fields: `identifier`, `name`, `status`, `day`, `areas`, `tributes`, `private`, `sponsors`, `alliance_events`
  - Implements: `Default`, `Display`
- **Game Lifecycle**: `start()`, `end()`, `run_day_night_cycle()`
- **State Queries**: `living_tributes()`, `winner()`, `random_open_area()`
- **Testing**: Submodules in `games/tests.rs` (1624 lines)

### **games/cycle.rs** (1270 lines) — **Cycle Execution**
- **Purpose**: Core cycle execution logic — building cycle context, executing tribute turns, processing area events
- **Key Functions**: `build_cycle_context()`, `execute_cycle()`, `run_tribute_cycle()`
- **Design**: Pre-computed immutable `CycleContext` split from mutable execution for borrow safety

### **games/alliances.rs** (258 lines) — **Alliance Event Processing**
- **Purpose**: Drains alliance event queue, applies betrayal cascades and death sanity breaks
- **Key Function**: `process_alliance_events()` — called between tribute turns

### **games/cycle_helpers.rs** (182 lines) — **Cycle Helper Methods**
- **Purpose**: Trauma producer invocation, area event announcements, event triggering
- **Key Functions**: `run_trauma_producers()`, `announce_area_events()`, `trigger_cycle_events()`

### **games/messages.rs** (142 lines) — **Message Helpers**
- **Purpose**: Fallback `MessagePayload` construction for legacy emission sites
- **Key Function**: `fallback_payload()` — transitional helper pending full typed payload migration

### **games/sponsors.rs** (48 lines) — **Sponsor Spawning**
- **Purpose**: Spawn one sponsor per archetype with district-loyalist binding
- **Key Functions**: `spawn_sponsors()`, `sponsor_affinity_snapshot()`

### **games/tests.rs** (1624 lines) — **Game Integration Tests**
- **Purpose**: Comprehensive test suite covering lifecycle, state transitions, area management, alliances, sponsors

### **messages.rs** (351 lines) — **Event Log System**
- **Purpose**: Global message accumulation and retrieval with typed `MessagePayload` variants
- **Key Types**:
  - `MessageSource` enum — Discriminates event origin (Game/Area/Tribute)
  - `GameMessage` struct — Event record with ID, source, day, subject, timestamp, content
  - `MessagePayload` enum — 55+ typed variants (e.g., `TributeKilled`, `AreaClosed`, `CombatEngagement`)
- **Global State**: `GLOBAL_MESSAGES` (thread-safe `VecDeque<GameMessage>`)
- **API**:
  - Write: `add_message()`, `add_game_message()`, `add_area_message()`, `add_tribute_message()`
  - Read: `get_all_messages()`, `get_messages_by_source()`, `get_messages_by_day()`
  - Maintenance: `clear_messages()` (called at day start)
- **Thread Safety**: `Mutex` guards ensure concurrent access safety (future-proofing for multi-threaded API)

### **output.rs** (502 lines) — **Presentation Layer**
- **Purpose**: Human-readable message formatting
- **Key Type**: `GameOutput<'a>` enum
  - 79+ variants covering all game events (day start, attacks, deaths, area events, etc.)
  - Implements `Display` trait with emoji-rich formatting
- **Formatting Utilities**: Uses `indefinite` crate for article insertion ("a sword", "an axe")
- **Integration**: `games.rs` calls `format!("{}", GameOutput::*)` then passes strings to `add_*_message()`
- **Design Note**: Decouples game logic from presentation — could swap to different languages/styles without touching `games.rs`

### **events/mod.rs** (654 lines) — **Event Module**
- **Purpose**: Module aggregator for typed `GameEvent` system; contains parity tests ensuring `GameEvent` renders identically to `GameOutput`
- **Key Exports**: `GameEvent` from `types.rs`

### **events/types.rs** (395 lines) — **Typed Game Events**
- **Purpose**: `GameEvent` enum — structured, owned, serde-friendly counterpart to `GameOutput`
- **Design**: Carries typed fields (UUIDs, names, items) so consumers react to *what happened* rather than re-parsing strings
- **Status**: Introduced in mqi.1; emission-site migration (mqi.2) and persistence (mqi.3) pending

### **events/display.rs** (517 lines) — **GameEvent Display**
- **Purpose**: `Display` implementation for `GameEvent` variants, rendering to the same strings as `GameOutput`

### **config.rs** (171 lines) — **Game Configuration**
- **Purpose**: `GameConfig` struct centralizing all game constants and tuning knobs
- **Key Fields**: `low_tribute_threshold`, `feast_*_count`, `day/night_event_frequency`, `trauma_enabled`, `phobias_enabled`, `fixations_enabled`, `addiction_enabled`, `event_severity_multiplier`
- **Design**: Runtime-configurable for difficulty modes and feature toggles

### **districts.rs** (209 lines) — **District Profiles**
- **Purpose**: 12 district profiles mapping number → industry → terrain affinities
- **Key Struct**: `DistrictProfile` (number, industry, primary_affinity, bonus_affinity_pool)
- **Usage**: `tribute.district` maps to profile for trait bonuses and alliance affinity

### **pathfinding.rs** (178 lines) — **Graph Pathfinding**
- **Purpose**: Generic A* pathfinding over weighted directed graphs
- **Key Trait**: `Graph` (nodes, neighbors, heuristic) — implementable for hex grid or sub-tile grid
- **Design**: Reusable at multiple granularities; v1 operates on 7-area hex graph

### **witty_phrase_generator/mod.rs** (260 lines) — **Name Generator**
- **Purpose**: Procedural game name generation using word combinations
- **Key Struct**: `WPGen`
  - Loads wordlists from embedded text files (`intensifiers.txt`, `adjectives.txt`, `nouns.txt`)
  - Methods: `with_words(n)`, `generic(...)`, `with_phrasewise_alliteration(...)`
- **Algorithm**: Backtracking constraint solver for length/alliteration requirements
- **Usage**: `Game::default()` calls `WPGen::new().with_words(3)` → e.g., "mighty-purple-dragon"

## Subdirectories

### **areas/** (1799 lines total) — **Arena Topology**
Hex-graph arena with 7+ areas, item inventories, and dynamic closures.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 394 | `Area` enum, `AreaDetails` struct, area lifecycle |
| `events.rs` | 660 | `AreaEvent` enum, event triggering, hazard spawning |
| `hex.rs` | 272 | Hex-graph topology, adjacency, pathfinding integration |
| `path.rs` | 209 | Area path generation, connection management |
| `water.rs` | 91 | Water source mechanics, dehydration effects |
| `shelter.rs` | 90 | Shelter mechanics, protection from weather |
| `forage.rs` | 39 | Foraging mechanics, resource gathering |
| `weather.rs` | 34 | `Weather` enum (Clear, Rain, Storm, etc.) |

### **tributes/** (12,648 lines total) — **Autonomous AI Tributes**
AI-controlled tributes with d20 combat, status effects, alliances, and context-aware decision-making.

| File/Dir | Lines | Purpose |
|----------|-------|---------|
| `mod.rs` | 2473 | `Tribute` struct, lifecycle methods, process_turn_phase |
| `tests.rs` | 710 | Tribute unit tests |
| `actions.rs` | 320 | `Action` enum, action selection, behavior definitions |
| `alliances.rs` | 523 | Alliance formation, breaks, event queue; MAX_ALLIES=5 |
| `combat_beat.rs` | 568 | Game-side narration for `CombatBeat` (wear, outcomes, stress) |
| `combat_tuning.rs` | 118 | `CombatTuning` — stress, stamina costs, band thresholds |
| `events.rs` | 159 | `TributeEvent` enum, random event generation |
| `helpers.rs` | 183 | Utility functions for tribute calculations |
| `incidents.rs` | 654 | Sleep incidents, shelter-based rest, dormancy processing |
| `inventory.rs` | 400 | Item management, equip/unequip, durability tracking |
| `movement.rs` | 276 | Movement between areas, travel restrictions |
| `rescue.rs` | 428 | Rescue resolution for Trapped afflictions |
| `stamina_band.rs` | 69 | `StaminaBand` derivation from stamina ratio (Fresh/Winded/Exhausted) |
| `statuses.rs` | 87 | `TributeStatus` enum (Healthy/RecentlyDead/Dead/Mauled) |
| `survival.rs` | 327 | Hunger/thirst bands, survival mechanics, dehydration |
| `traits.rs` | 459 | `Trait` enum (25+ personality traits), trait bonuses |
| `traps.rs` | 25 | `PlacedTrap` struct, trap state management |

### **tributes/combat/** (2986 lines total) — **Combat Engine**
D20-based combat system with attack contests, wound infliction, and stress.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 360 | Combat orchestrator, `Tribute::attacks()` |
| `resolve.rs` | 930 | Attack contest resolution, combat results application |
| `inflict_table.rs` | 536 | Wound infliction tables, severity rolls |
| `tests.rs` | 1133 | Combat integration tests |

### **tributes/brains/** (3463 lines total) — **Tribute AI**
Decision-making engine with scoring, override layers for afflictions.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 952 | `Brain` struct, decision orchestration, scoring |
| `tests.rs` | 940 | Brain decision tests |
| `scoring.rs` | 170 | Action scoring heuristics |
| `decisions.rs` | 124 | Decision output types |
| `affliction_override.rs` | 341 | Override actions for active afflictions |
| `phobia_override.rs` | 367 | Override actions for phobia triggers |
| `fixation_override.rs` | 388 | Override actions for fixation processing |
| `addiction_override.rs` | 143 | Override actions for addiction cravings |
| `trauma_override.rs` | 108 | Override actions for trauma responses |

### **tributes/afflictions/** (9523 lines total) — **Affliction System**
Comprehensive health condition system: anatomy, trauma, phobias, fixations, addictions.

| File/Dir | Lines | Purpose |
|----------|-------|---------|
| `mod.rs` | 202 | Module aggregator, acquisition API, tuning |
| `anatomy.rs` | 800 | `AcquireResolution`, body part targeting, wound application |
| `anatomy_tests.rs` | 590 | Anatomy resolution tests |
| `trauma.rs` | 312 | `TraumaAcquisition`, trauma producers |
| `trauma_tests.rs` | 319 | Trauma tests |
| `phobia/mod.rs` | 27 | Phobia module aggregator |
| `phobia/scan.rs` | 677 | Per-cycle phobia scan, trigger evaluation |
| `phobia/reaction.rs` | 475 | Phobia reaction effects, panic responses |
| `phobia/triggers.rs` | 384 | Trigger definitions, fear stimulus matching |
| `phobia/spawn.rs` | 199 | Spawn-time phobia acquisition |
| `phobia/outcomes.rs` | 116 | Phobia outcome resolution |
| `fixation.rs` | 924 | Fixation acquisition, processing, obsessions |
| `addiction.rs` | 369 | Addiction mechanics, craving system, decay |
| `addiction_tests.rs` | 410 | Addiction tests |
| `cascade.rs` | 477 | `CascadeResult`, affliction cascading |
| `cure.rs` | 346 | `CureOutcome`, recovery mechanics, item-to-cure mapping |
| `effects/mod.rs` | 13 | Effects module aggregator |
| `effects/brain_bias.rs` | 341 | `BrainBias` computation for afflictions |
| `effects/stat_modifiers.rs` | 436 | `StatModifiers` computation for afflictions |
| `effects/trauma_effects.rs` | 57 | Trauma-specific effect application |
| `trapped.rs` | 367 | Trapped affliction mechanics |
| `tuning.rs` | 45 | `AfflictionTuning` constants |
| `producers/mod.rs` | 37 | Trauma producer module aggregator |
| `producers/shared.rs` | 143 | Shared producer utilities |
| `producers/survive_betrayal.rs` | 44 | Betrayal survival trauma producer |
| `producers/survive_near_death.rs` | 54 | Near-death survival trauma producer |
| `producers/witness_ally_death.rs` | 66 | Ally death witness trauma producer |
| `producers/witness_mass_casualty.rs` | 64 | Mass casualty witness trauma producer |
| `producers/tests.rs` | 405 | Producer tests |
| `integration_tests.rs` | 1527 | Cross-module integration tests |
| `snapshot_tests.rs` | 93 | Snapshot regression tests |
| `trauma_snapshot_tests.rs` | 55 | Trauma snapshot tests |

### **tributes/lifecycle/** (911 lines total) — **Tribute Lifecycle**
Death, health, stamina, and status management.

| File | Lines | Purpose |
|------|-------|---------|
| `status.rs` | 588 | Status transition logic, death processing |
| `health.rs` | 378 | Health pool management, healing, damage |
| `stamina.rs` | 139 | Stamina pool management, fatigue |
| `death.rs` | 102 | Death resolution, item drops |
| `mod.rs` | 4 | Module aggregator |

### **tributes/snapshots/** and **tributes/afflictions/snapshots/**
Snapshot test data directories (insta snapshot files for regression testing).

### **events/** (1566 lines total) — **Typed Event System**
Structured game events for persistence and analytics.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 654 | Module aggregator, parity tests |
| `types.rs` | 395 | `GameEvent` enum (55+ typed variants) |
| `display.rs` | 517 | `Display` implementation for `GameEvent` |

### **items/** (1083 lines total) — **Item System**
Weapons, shields, and consumables with procedural generation.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 349 | `Item` struct, `OwnsItems` trait, rarity system |
| `tests.rs` | 461 | Item unit tests |
| `generation.rs` | 217 | Procedural item generation |
| `name_generator.rs` | 56 | Item name generation |

### **terrain/** (472 lines total) — **Terrain System**
Biome types, descriptors, and terrain configuration.

| File | Lines | Purpose |
|------|-------|---------|
| `config.rs` | 149 | `Visibility`, `Harshness`, `ItemWeights` per terrain |
| `assignment.rs` | 192 | Terrain-to-area assignment, balance constraints |
| `types.rs` | 121 | `BaseTerrain` enum (12 biomes), `TerrainDescriptor`, `TerrainType` |
| `mod.rs` | 8 | Module aggregator, re-exports |
| `descriptors.rs` | 2 | (Placeholder) |

### **threats/** (193 lines total) — **Environmental Hazards**
Animal encounters and environmental threats.

| File | Lines | Purpose |
|------|-------|---------|
| `animals.rs` | 192 | `Animal` enum, attack mechanics |
| `mod.rs` | 1 | Module aggregator |

### **sponsors/** (601 lines total) — **Sponsor System**
Sponsor archetypes, budgets, and affinity tracking.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 601 | `SponsorContext`, archetype modifiers, sponsorship resolution |

### **phases/** (394 lines total) — **Phase Pipeline**
Per-phase environmental conditions and pipeline scaffolding.

| File | Lines | Purpose |
|------|-------|---------|
| `environment.rs` | 386 | `LightLevel`, `AfflictionDraft`, `AreaPhaseConditions` |
| `mod.rs` | 8 | Module aggregator |

## Notes
- **Pure Logic**: No file I/O, networking, or database access — all side effects are message emissions
- **Testability**: 60+ inline tests across modules using `rstest` parameterized testing; snapshot tests via `insta`
- **Performance**: `run_tribute_cycle()` uses `HashMap` lookups instead of nested loops (O(n²) → O(n))
- **Statefulness**: Only `GLOBAL_MESSAGES` persists between function calls; `Game` struct is fully serializable
- **Total LOC**: ~38,500 lines across 100+ source files
