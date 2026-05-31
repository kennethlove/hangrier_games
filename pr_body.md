## Summary

Complete rebuild of the announcers crate — transforms structured game events into Capitol broadcast commentary between Verity (play-by-play), Rex (color), and Flash (technical analyst).

## Changes

### Core pipeline (announcers/ crate)
- Commentator trait — abstract LLM backend with batch generate() and streaming generate_stream()
- BroadcastPackageBuilder — classifies 55+ MessagePayload variants into typed EventLines
- TributeHistories — rolling per-tribute digest with permanent highlights (kills, betrayals, alliances)
- OllamaCommentator — reqwest-based, passes think:false for qwen3.5 compatibility
- CloudflareCommentator — Cloudflare Workers AI backend via chat API (feature-gated)

### Game features
- Kill leader tracking per phase
- Killing sprees with tier milestones (heating up -> on fire -> dominating -> unstoppable)
- Spree-break events when a streak is ended
- Hot zones — areas with the most activity
- Permanent highlights survive the rolling 30-event cap

### API integration
- Background task spawns after save_game() each cycle
- Persists CommentarySegment to commentary_segments table
- Persists TributeHistories to tribute_histories table
- Delivers via WebSocketMessage::Commentary on existing SSE/WS channels
- Retry logic with exponential backoff for DB persistence failures
- Empty segment filtering — skips save/broadcast when LLM returns nothing

### Prompt engineering
- Structured format with sections: PHASE CONTEXT, HOT STREAKS, HOT ZONES, KILL LEADERS, PHASE EVENTS, TRIBUTE HISTORIES
- Three-announcer rotation (Verity, Rex, Flash)
- No stage directions — clean dialogue only
- Repeat penalty + temperature tuning to reduce loops

### Testing
- 32 unit tests + 10 integration tests + 2 API tests = 44 total
- Smoke-tested locally with qwen3:1.7b and Cloudflare Llama 3.2 3B

## Usage

Local (Ollama):
  ollama pull qwen3:1.7b
  ollama create announcers -f announcers/src/Modelfile
  cargo run --package api --features ollama

Cloudflare:
  Set CLOUDFLARE_API_TOKEN + CLOUDFLARE_ACCOUNT_ID
  cargo run --package api --features cloudflare

## Verification
- cargo test --package announcers — 42 tests
- cargo test --package api --test commentary_tests — 2 tests
- cargo check --workspace
