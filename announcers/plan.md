# Announcers Crate — Rebuild Plan

## Goal

Replace the current skeleton `announcers/` crate with a proper commentary system: abstract `Commentator` trait (swappable LLM backend), broadcast package builder, rolling tribute history tracker, and async API integration that triggers commentary after each game phase and pushes to browsers via SSE.

## Architecture

```
Game cycle completes (per phase)
         │
         ▼
API spawns async background task
         │
         ▼
announcers::broadcast::build_package(game_state, phase_events, tribute_histories)
         │
         ├── Structured events [combat], [death], [allied], etc.
         ├── Prose bullet events (foraging, movement, resting)
         ├── Phase header (alive count, kill leaders, alliances, hot zones)
         └── Tribute histories (rolling digest per tribute, updated each phase)
         │
         ▼
announcers::llm::Commentator::generate(package) → one LLM call
         │
         ▼
Interleaved Verity/Rex script (CommentarySegment)
         │
         ▼
Stored as CommentarySegment (new SurrealDB table)
Pushed to live browsers via SSE
```

### Design decisions

| Decision | Choice | Why |
|---|---|---|
| Voices | One LLM call, interleaved script | Simpler, cheaper, natural banter |
| Event format | Hybrid — structured typed headers for key events, prose bullets for minor | Token-efficient where it counts |
| Raw numbers | Mapped to narrative severity tiers | Sports commentary doesn't say "7dmg" |
| Trigger | Async background task after each phase | Game tick is never blocked |
| Storage | Separate `CommentarySegment` type/table | Clean boundary from game events |
| LLM backend | Abstracted behind `Commentator` trait | Swap Ollama → API model later |

## Module structure

```
announcers/
├── Cargo.toml
├── codemap.md
├── plan.md
└── src/
    ├── lib.rs           # Public API + generate_commentary() convenience fn
    ├── types.rs         # Core types (see below)
    ├── broadcast.rs     # BroadcastPackage builder from GameMessage vec
    ├── history.rs       # Rolling per-tribute digest (append, capped)
    ├── severity.rs      # Raw value → narrative descriptor mappings
    └── llm/
        ├── mod.rs       # Commentator trait
        └── ollama.rs    # OllamaCommentator impl (feature-gated)
```

## Types (`types.rs`)

| Type | Fields / Shape | Notes |
|---|---|---|
| `GameStateSnapshot` | `alive_count`, `kill_leaders: Vec<KillLeader>`, `alliances: Vec<AllianceInfo>`, `hot_zones: Vec<AreaActivity>` | Phase-level context |
| `EventLine` | `kind: EventKind`, `prose: String`, `structured: Option<EventData>` | Hybrid format |
| `EventKind` | enum: `Combat`, `Death`, `Allied`, `Betrayal`, `Hazard`, `Item`, `Movement`, `Sponsor`, `Other` | For typed headers |
| `EventData` | free-form serde_json::Value per variant | Structured sub-fields |
| `TributeDigest` | `name`, `district`, `status`, `injury_level`, `location`, `allies`, `notable_events: Vec<String>` | Rolling history |
| `BroadcastPackage` | `header: GameStateSnapshot`, `events: Vec<EventLine>`, `histories: Vec<TributeDigest>` | Full input to LLM |
| `CommentaryLine` | `speaker: String`, `text: String` | One utterance |
| `CommentarySegment` | `id, game_id, day, phase, lines: Vec<CommentaryLine>, generated_at, model_used` | Persisted output |
| `CommentaryError` | error enum | Crate error type |

## Severity mappings (`severity.rs`)

Raw game values → narrative descriptors (no numbers exposed to LLM):

| Input | Range → Descriptor |
|---|---|
| Damage dealt | 0 = missed, 1-4 = glancing/scraped, 5-9 = solid hit/nasty wound, 10-14 = grievous/devastating, 15+ = near-fatal/crushing |
| Tribute HP % | 100% = unharmed, 75-99% = scraped up, 50-74% = wounded, 25-49% = badly wounded, 1-24% = near death, 0 = deceased |
| Attack roll vs AC | missed by 5+ = "easily dodged", missed by 1-4 = "just barely dodged", hit by 1-4 = "just connected", hit by 5+ = "clean hit" |

## Broadcast package builder (`broadcast.rs`)

Iterates `Vec<GameMessage>`, inspects each `MessagePayload` variant:

- **Combat variants** (`CombatSwing`, `CombatEngagement`) → `EventLine { kind: Combat, structured: {attacker, defender, weapon, result, severity, location, outcome}, prose: "..." }`
- **Death variants** (`TributeKilled`) → `EventLine { kind: Death, ... }`
- **Alliance variants** (`AllianceFormed`, `BetrayalTriggered`) → `EventLine { kind: Allied/Betrayal, ... }`
- **Hazard variants** (`AreaEvent`) → `EventLine { kind: Hazard, ... }`
- **Item variants** (`ItemFound`, `ItemUsed`) → `EventLine { kind: Item, prose: "..." }` if notable, skipped otherwise
- **Movement variants** (`TributeMoved`) → `EventLine { kind: Movement, prose: "..." }`
- **State/tick variants** (`HungerBandChanged`, `TraumaAcquired`, etc.) → skipped or rolled into prose bullets

Calls `severity::describe_damage()`, `severity::describe_injury()`, etc. for narrative values.

## Tribute history (`history.rs`)

```rust
pub struct TributeHistories {
    inner: HashMap<String, TributeDigest>,
}
```

- `new(game_state) -> Self` — initialize from tribute roster
- `update(&mut self, events: &[GameMessage])` — for each event involving a tribute, append to that tribute's `notable_events` (capped at 8, oldest pruned)
- `digests(&self) -> Vec<TributeDigest>` — sorted by tribute name
- Serialize/Deserialize for SurrealDB persistence

Cap of 8 lines is generous — at 1 line per phase, that's 2 full days of history per tribute.

## LLM abstraction (`llm/mod.rs` + `llm/ollama.rs`)

```rust
#[async_trait]
pub trait Commentator: Send + Sync {
    /// Generate a commentary segment from a broadcast package.
    /// Returns structured lines tagged by speaker.
    async fn generate(&self, package: &BroadcastPackage) -> Result<CommentarySegment, CommentaryError>;
}
```

`OllamaCommentator`:
- Holds model name, Ollama client config
- Constructs a prompt from `BroadcastPackage` (phase header → events → histories → instruction to produce Verity/Rex dialogue)
- Calls Ollama generate (batch for now; streaming can be added later via the trait)
- Parses output into `CommentaryLine` segments
- Behind `features = ["ollama"]` — not a required dependency

**Prompt structure (inside OllamaCommentator):**

```
System: You are a live sports broadcast team covering the Hunger Games...
(prompts for Verity play-by-play + Rex color commentary format)

User: [BroadcastPackage serialized as compact text]

Generate an interleaved broadcast script with [VERITY] and [REX] tags.
```

## Public API (`lib.rs`)

```rust
/// Top-level convenience: build package + generate commentary
pub async fn generate_commentary(
    commentator: &dyn Commentator,
    game_state: &GameStateSnapshot,
    phase_events: &[GameMessage],
    histories: &[TributeDigest],
) -> Result<CommentarySegment, CommentaryError>;

// Re-exports
pub use types::*;
pub use llm::Commentator;
#[cfg(feature = "ollama")]
pub use llm::ollama::OllamaCommentator;
```

## Dependency changes (`Cargo.toml`)

| Change | Why |
|---|---|
| Add `async-trait` | Required for `Commentator` trait |
| Add `serde` with `derive` | Serialize types for SurrealDB + prompt construction |
| Add `tokio` | Async trait + background task (already in workspace) |
| Make `ollama-rs` optional (`features = ["ollama"]`) | Backend-agnostic crate |
| Remove `futures` | No longer needed |
| Remove `tokio-stream` | No longer needed |
| Remove `thiserror` | Replace with custom `CommentaryError` or keep |

## API integration

### Trigger (in `api/src/games/mod.rs`)

After `run_game_cycles()` returns and messages are saved to SurrealDB, spawn a background task:

```rust
tokio::spawn(async move {
    // 1. Build GameStateSnapshot from current game state
    // 2. Fetch TributeHistories from SurrealDB (or rebuild from messages)
    // 3. Get phase events from the just-completed cycle
    // 4. Call announcers::generate_commentary(&commentator, state, events, histories)
    // 5. Save CommentarySegment to SurrealDB
    // 6. Push to SSE stream
    // 7. Update TributeHistories in SurrealDB
});
```

### SSE delivery

Extend existing SSE endpoint or add a new stream for commentary.

### SurrealDB schema

```sql
DEFINE TABLE commentary_segments SCHEMAFULL;
DEFINE FIELD game_id ON TABLE commentary_segments TYPE string;
DEFINE FIELD day ON TABLE commentary_segments TYPE int;
DEFINE FIELD phase ON TABLE commentary_segments TYPE string;
DEFINE FIELD lines ON TABLE commentary_segments TYPE array;
DEFINE FIELD generated_at ON TABLE commentary_segments TYPE datetime;
DEFINE FIELD model_used ON TABLE commentary_segments TYPE string;
DEFINE INDEX idx_game_day_phase ON commentary_segments COLUMNS game_id, day, phase UNIQUE;
```

## Implementation order

1. **`types.rs`** — Core types. Everything depends on these, so get them right first.
2. **`severity.rs`** — Narrative descriptor mappings. Self-contained.
3. **`broadcast.rs`** — Package builder. Heavy iteration over MessagePayload variants.
4. **`history.rs`** — Rolling digest. Needs to play well with broadcast.rs input.
5. **`llm/mod.rs` + `llm/ollama.rs`** — Trait + one impl. Feature-gated.
6. **`lib.rs` + `Cargo.toml`** — Wire it all together, remove old code.
7. **API integration** — Async trigger, SSE push, SurrealDB storage + schema.

## Open questions (deferred)

- **SSE infrastructure** — how the existing SSE/WebSocket setup works; extend or add new stream
- **Tribute history persistence** — rebuild from messages each cycle vs store in SurrealDB
- **Prompt details** — iterate separately from code; OllamaCommentator gets a reasonable default
- **Model config** — Ollama stays default; `Commentator` trait makes swapping easy later
