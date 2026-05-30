# announcers/

## Responsibility

Structured commentary pipeline for Hunger Games events. Transforms typed game messages into Capitol broadcast-style commentary between Verity (play-by-play) and Rex (color commentary). Not an LLM-only crate — the `Commentator` trait abstracts over any backend.

## Architecture

```
Phase events (Vec<GameMessage>)
         │
         ▼
BroadcastPackageBuilder::build(header, events, histories)
         │
         ▼
BroadcastPackage { header: GameStateSnapshot, events: Vec<EventLine>, histories: Vec<TributeDigest> }
         │
         ▼
Commentator::generate(package) → CommentarySegment { lines: Vec<CommentaryLine> }
         │
         ▼
Persisted to SurrealDB (commentary_segments table) + pushed via SSE/WS
```

## Module Structure

| Module | File | Responsibility |
|---|---|---|
| `types` | `src/types.rs` | Core types: EventKind, EventLine, GameStateSnapshot, TributeDigest, BroadcastPackage, CommentaryLine, CommentarySegment, CommentaryError |
| `severity` | `src/severity.rs` | Raw-value→narrative-descriptor mappings (damage, injury, hit quality, area activity) |
| `broadcast` | `src/broadcast.rs` | BroadcastPackageBuilder — iterates 55+ MessagePayload variants, produces typed EventLines |
| `history` | `src/history.rs` | TributeHistories — rolling per-tribute digest (status, location, allies, notable events) |
| `llm` | `src/llm/mod.rs` | Commentator trait (`async fn generate(&self, package) -> Result<CommentarySegment>`) |
| `llm/ollama` | `src/llm/ollama.rs` | OllamaCommentator — feature-gated behind `features = ["ollama"]` |

## Key Types

- **`BroadcastPackage`**: Full structured input to the LLM (header + events + histories)
- **`EventLine`**: Hybrid format — typed `EventKind` + prose + optional structured data
- **`CommentaryLine`**: One utterance (`speaker: String`, `text: String`)
- **`CommentarySegment`**: Persisted output with id, game_id, day, phase, lines, timestamp
- **`TributeDigest`**: Rolling per-tribute summary (capped at 8 notable events)

## Integration

**API Trigger** (`api/src/games/mod.rs`):
- After `save_game()` drains phase messages, spawns `tokio::spawn` background task
- Builds `GameStateSnapshot` + `TributeHistories` from current game state
- Calls `announcers::generate_commentary()`
- Persists `CommentarySegment` to SurrealDB (`commentary_segments` table)
- Broadcasts via `WebSocketMessage::Commentary` (relayed through both WebSocket and SSE)

**LLM Backend** (optional):
- Default: `OllamaCommentator` behind `features = ["ollama"]`
- Custom: implement `Commentator` for any LLM (OpenAI, Anthropic, etc.)
- Prompt: system prompt establishing Verity/Rex voices + serialized `BroadcastPackage`

## Key Files
- `src/lib.rs`: Module declarations, re-exports, `generate_commentary()` convenience fn
- `src/types.rs`: All core data types
- `src/broadcast.rs`: MessagePayload → EventLine classification
- `src/history.rs`: Rolling per-tribute digest tracker
- `src/llm/mod.rs`: Commentator trait definition
- `src/llm/ollama.rs`: Ollama-backed implementation
- `Modelfile.qwen`: Ollama model definition (only for `ollama` feature)
