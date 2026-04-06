# announcers/

## Responsibility

Provides AI-generated sports commentary for Hunger Games events using local LLM (Ollama). Transforms raw game log entries into engaging broadcast-style dialogue between two commentators (Verity and Rex) following Capitol entertainment show conventions.

## Design

**LLM Integration Pattern:**
- Uses `ollama-rs` client to interact with local Ollama service
- Custom model `announcers` based on `qwen2.5:1.5b` (defined in `Modelfile.qwen`)
- Two generation modes: batch (`summarize`) and streaming (`summarize_stream`)

**Prompt Engineering:**
- `ANNOUNCER_PROMPT`: System prompt defining Verity (play-by-play) and Rex (color commentary) personas
- `prompt()`: Combines system prompt with game log to create complete generation request
- Prompt instructs model to produce markdown script with banter between commentators

**Error Handling:**
- Custom `AnnouncerError::FailedToGenerateResponse` for generation failures
- Stream implementation yields `Result<String, String>` for gradual error handling

## Flow

**Batch Generation:**
```
Game log → prompt(log) → Ollama.generate() → String response
```

**Streaming Generation:**
```
Game log → prompt(log) → Ollama.generate_stream() → 
  async_stream → Pin<Box<dyn Stream<Item = Result<String, String>>>>
```

Stream yields tokens as they're generated, allowing for progressive UI updates.

## Integration

**Consumers:**
- `api/` crate (declared dependency, not yet actively used in routes)
- Intended for future real-time commentary endpoint

**External Dependencies:**
- Ollama server (expected on default host/port via `Ollama::default()`)
- Custom model installation required: `ollama create announcers -f Modelfile.qwen`

**Key Files:**
- `lib.rs`: Public API (`summarize`, `summarize_stream`, `prompt`)
- `main.rs`: Example usage (currently commented out)
- `Modelfile.qwen`: Ollama model definition with system prompt and parameters
