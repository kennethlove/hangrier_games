# Progressive Display — Design

**Status:** Draft
**Date:** 2026-05-02
**Crate(s) primarily affected:** `web/`
**Related specs:** `2026-04-26-game-event-enum.md`, `2026-04-26-game-timeline-redesign.md`

## Goals

Add a paced event-reveal mode to the existing `Timeline` component so:

1. **Live games feel like sportscasts.** As websocket events arrive during a game in progress, they appear one-by-one with dramatic pacing — not dumped instantly.
2. **Past phases can be replayed on demand.** Static event lists are the default for past days, but a Replay button arms playback controls so users can re-experience the phase as it unfolded.
3. **Users have full transport control during replay.** Phase-jump map, scrubber, transport buttons (back-to-start, prev/next event, play/pause, jump-to-end), and a speed multiplier.
4. **Live navigation is preserved.** During a live game, the user can scrub back to past phases without losing the live stream — new events queue silently and a "Live ●" badge offers one-click return.

Non-goals:

- Replacing the existing `Timeline` rendering of individual `EventCard`s. This spec adds a *delivery* mode on top of the existing card components; it does not change how individual events look (beyond a fade-in entrance).
- Streaming text within a single announcer message (token-level streaming). Reveal pacing is event-level only.
- Server-side change to broadcast pacing. The server emits events as fast as they happen; the client decides reveal cadence.
- Audio cues, screen shake, or stronger per-event-type animation. Future work.

## Two Display Modes

The same `Timeline` component supports two modes via a new prop:

```rust
pub enum TimelineMode {
    Static,    // Show all events immediately. Default for past phases.
    Reveal,    // Drip events one at a time with dwell timing + transport controls.
    Live,      // Drip events as they arrive via websocket; phase chips only (no transport).
}
```

| Context | Default mode | User can switch to |
|---|---|---|
| Past phase, finished game | Static | Reveal (Replay button) |
| Past phase, live game | Static | Reveal (Replay button) |
| Current phase, live game | Live | — (always live) |
| Future phase | n/a | n/a (chip is grayed out) |

### Mode Asymmetry

- **Static** is a no-op rendering pass — same as today.
- **Reveal** renders only the events up to a current cursor index, advancing on a dwell timer and exposing a transport bar.
- **Live** renders events up to the head of an arriving websocket queue, also using the dwell timer, but the transport bar collapses to phase chips only — there's no scrubbing through events that haven't happened yet.

If the user navigates from a current-phase Live view to a past phase, the Live phase keeps queueing events in the background; the Live ● badge surfaces them.

## Pacing Model

Per-event dwell time uses model **D — length-based with a per-card-type minimum floor**:

```
text_dwell  = text_length * 30ms + 500ms
raw_dwell   = max(min_dwell_for(event_card_type), text_dwell)
final_dwell = max(150ms, raw_dwell / speed_multiplier)
```

The 150ms hard floor prevents events from flying past unreadably fast even at 4x with a short text body. Speed multiplier is applied last; per-card minimums scale with multiplier but never below 150ms.

### Per-Card-Type Minimums

A new method (or free function) returns the minimum dwell per event variant:

```rust
fn min_dwell_ms(event: &GameEvent) -> u32;
```

Starter values (final tuning during implementation):

| Event category | Min dwell |
|---|---|
| Death (any cause) | 2500 |
| Combat (kill, decisive win) | 2000 |
| Combat (normal win, wound, miss) | 1200 |
| Alliance formed / broken | 1500 |
| Betrayal | 1800 |
| Item picked up / used | 700 |
| Movement | 300 |
| Hide / Rest | 400 |
| State change (status, weather, area event) | 1000 |
| Emotion label change *(if surfaced as an event card)* | 1200 |

These are **floors** — a long announcer message will use its text-length-based dwell. The floors prevent short messages on dramatic events from blipping by.

### Speed Multiplier

User-controlled speed in replay mode: **0.5x / 1x / 2x / 4x**. Applied to the dwell formula above. Speed changes take effect on the *next* event's dwell, not by accelerating the timer that's already running (avoids jarring mid-event time warps). Live mode also respects the speed multiplier (set via the same selector if surfaced; otherwise defaults to 1x for live).

## Transport Controls (Replay Mode)

```
[ D1  N1  D2  N2  D3  N3  D4 ... ]    Phase chips
[▬▬▬▬●▬▬▬▬▬▬▬▬]  Event 14 of 47       Scrubber
[⏮]  [⏪]  [▶/⏸]  [⏩]  [⏭]   [1x ▾]    Transport + speed
```

### Phase Chips (Top Row — Always Visible)

Compact horizontal row of chips, one per phase across the entire game:

- **Visited phase** (current cursor is past it, or the phase is fully complete): clickable; clicking jumps cursor to event 0 of that phase and switches the timeline view to that phase in Reveal mode.
- **Current phase** (the one being displayed): visually highlighted; pulses softly during Live mode to indicate "this is happening right now."
- **Future phase** (game hasn't reached it): grayed out, not clickable.

Phase chips are the only transport surface visible in Live mode.

### Scrubber (Replay Only)

A draggable progress bar showing event N of M for the current phase. Click to jump to position; drag to scrub. Live mode does not show the scrubber (there's nothing yet-to-scrub-through).

Companion label: `Event 14 of 47 · Day 3 (Day phase)`.

### Transport Buttons (Replay Only)

| Control | Behavior |
|---|---|
| `⏮` Back to start | Reset cursor to event 0 of current phase. Preserves play/pause state. |
| `⏪` Previous event | Step cursor back 1. Always pauses if playing. |
| `▶ / ⏸` Play / Pause | Toggle auto-advance at current speed multiplier. |
| `⏩` Next event | Step cursor forward 1. Always pauses if playing. |
| `⏭` Jump to end | Reveal all remaining events instantly. Pauses at end. |
| Speed `1x ▾` | Dropdown or pill row: 0.5x / 1x / 2x / 4x. |

Stepping (⏪ ⏩) always pauses to avoid the confusing "I clicked next and then it advanced again" double-step on click during play.

### Replay Threshold

Phases with fewer than **3 events** skip the Replay button — not worth the transport ceremony for a single combat or two movement events. Threshold tunable.

## Live Mode

When the user is on the current phase of a game in progress:

- `TimelineMode::Live` is active by default.
- Websocket events from `use_game_websocket` feed directly into the reveal queue.
- Each new event respects the dwell timer — events arriving in a burst (server emits 5 in 100ms) reveal one at a time with proper pacing, not in a flash.
- Phase chips are the only navigation surface visible.
- No scrubber, no transport, no speed selector by default. (Speed selector may be exposed in Live as a "settings" pop-out — out of scope for v1; Live is fixed at 1x.)

### Live Queue Behavior

The reveal pipeline holds an internal queue. When a new websocket event arrives:

1. **If no event is currently dwelling** (queue was empty and reveal is idle): the new event reveals immediately, then its dwell timer starts.
2. **If an event is currently dwelling**: the new event is appended to the queue. When the current dwell completes, the next queued event reveals; the queue drains one event per dwell-tick until empty.
3. **The currently-dwelling event is never preempted** — its full dwell completes regardless of how many new arrivals queue up behind it. (Otherwise a burst of arrivals would visually skip past important events.)

If the queue grows long (e.g., server backlog of 20+ events), reveal pacing alone can't keep up. For Live mode, we accept that — the queue drains at normal pace and the user sees a slightly delayed-but-still-paced stream. (The fast-forward shortcut from "Live ● Backlog Handling" applies only when *returning* to Live after scrubbing away, not to natural Live arrival bursts.)

If the user clicks a past phase chip while in Live mode:

1. Timeline view switches to that past phase in Reveal mode.
2. Live websocket events keep arriving on the underlying signal but are not rendered.
3. The Live ● badge appears in the header.

## Live ● Badge

A persistent badge that surfaces only when the user has scrubbed away from the current live phase.

```
┌─────────────────────────────────────────┐
│ Live ●  Day 4 — 3 new events            │
└─────────────────────────────────────────┘
```

- Renders in the page header (above the phase chips).
- Red dot to signal "live."
- Counter shows queued events accumulated since the user left live ("3 new events"), or omitted if none.
- Click jumps the timeline to the current live phase, switches mode to Live, and flushes the queue (revealed at speed, not all-at-once — same as a normal Live arrival, just with a starting backlog).

The badge is the *only* live signal while scrubbing — no banner, no inline ticker. Single focal point.

### Backlog Handling on Return

When the user clicks the badge, the queued events drain into the reveal pipeline. To avoid a 30-event drain feeling tedious:

- If queue length ≤ 5 events: drain at normal pace.
- If queue length > 5 events: pre-render the first `(queue_len - 5)` events as already-shown (instant), then drain the last 5 with normal pacing. User catches up to "live in the last few seconds" without watching a long replay of the gap.

Implementation tunable; the principle is "don't punish leaving live."

## Reveal Animation

Each event card animates in via fade + slide-up:

- Initial state: `opacity: 0; transform: translateY(8px);`
- Final state: `opacity: 1; transform: translateY(0);`
- Duration: ~200ms with `ease-out`.
- Implemented with Tailwind transition utilities + a transient class swap on mount; no JS animation library.

Cards already in the list don't re-animate when new ones arrive. Only the entering card animates.

## Layout & Scrolling

Append-and-grow vertical list (matches today's `Timeline` shape):

- Newest event always appears at the bottom.
- Page extends as events arrive; user scrolls down to follow.
- **Auto-scroll-to-latest** is engaged by default — when a new event reveals, the page scrolls to keep it in view.
- **Auto-scroll disengages** when the user manually scrolls upward (intent: read something earlier without being yanked back).
- **Auto-scroll re-engages** when the user manually scrolls back to within ~100px of the bottom.

Standard chat-app behavior (Discord, Slack pattern).

## Data Model & Wiring

### Timeline Component (`web/src/components/timeline/timeline.rs`)

New props:

```rust
#[derive(Props, ...)]
pub struct TimelineProps {
    pub events: Vec<GameMessage>,         // Existing
    pub mode: TimelineMode,               // NEW
    pub live_events: Option<Signal<Vec<GameEvent>>>,   // NEW; populated for Live mode only
    pub current_phase: Option<PhaseRef>,  // NEW; needed for chip highlighting & badge
    pub all_phases: Vec<PhaseRef>,        // NEW; drives chip row
    pub on_phase_change: Option<EventHandler<PhaseRef>>,  // NEW; user clicked a chip
}
```

`PhaseRef` is whatever existing identifier already names a (game, day, phase) tuple in the codebase — likely derived from the timeline-summary endpoint's existing fields.

### New Files

- `web/src/components/timeline/reveal.rs` — reveal-mode state machine: cursor index, dwell timer (`use_future` + `gloo-timers::future::TimeoutFuture`), play/pause state, speed multiplier, queue management for live arrivals.
- `web/src/components/timeline/transport.rs` — transport bar component: phase chips, scrubber, buttons, speed selector. Receives callbacks for cursor manipulation.
- `web/src/components/timeline/live_badge.rs` — the badge component with click-to-return.
- `web/src/components/timeline/dwell.rs` — pure functions: `min_dwell_ms(event)`, `compute_dwell(event, speed) -> Duration`.

### Modified Files

- `web/src/components/timeline/timeline.rs` — accept the new props; dispatch to Static / Reveal / Live render paths.
- `web/src/components/game_detail.rs` — when current phase is being viewed during a live game, pass `TimelineMode::Live` and the websocket signal; otherwise `TimelineMode::Static`. Wire phase navigation.
- `web/src/components/game_period_page.rs` — host the Replay button alongside the existing day-log render; on click, switch to `TimelineMode::Reveal`.
- `web/src/hooks/use_game_websocket.rs` — currently caps at `MAX_EVENTS = 200` ring buffer; verify this cap is high enough for the Live ● badge backlog use case, or expose a "since cursor" subscription if needed.

## Integration Points

- **Existing websocket plumbing.** `use_game_websocket` already streams `GameEvent`s into a `Signal<Vec<GameEvent>>`. Live mode subscribes to this signal directly; reveal-mode dwelling sits on top.
- **Existing event ordering.** `GameMessage` has `(tick, emit_index)` — used today by `Timeline` for sorting. The reveal cursor is an index into the *sorted* list; same ordering invariants apply.
- **Existing card components.** `cards/{combat,death,movement,item,alliance,state}_card.rs` are unchanged. They get a thin animation wrapper from the reveal component (CSS transition classes added on mount).
- **Existing timeline-summary endpoint.** Provides per-phase event counts already; this is what powers the phase chips and the "skip Replay if < 3 events" threshold.
- **No server changes** required. Server keeps broadcasting events as fast as they happen; client decides cadence.

## Testing Strategy

Web crate doesn't have heavy test infrastructure today, so testing is mostly pure-function + manual. Where unit tests are practical:

- **`compute_dwell`** — given a known event and speed, returns the documented dwell. Floor honored at 4x for short events. Speed multiplier divides correctly.
- **Reveal cursor logic** — pure state machine functions for next/prev/jump/play-pause produce expected cursor positions and play states.
- **Phase chip state** — given a list of phases and a current cursor, returns correct visited/current/future classification.
- **Backlog drain logic** — given a queue length, produces the right "instant N, drain last 5" split.

Manual / visual verification:

- Live mode: mock a stream of 10 events bursting in 100ms; verify they reveal one-by-one with proper pacing.
- Replay mode: scrub forward and back across phase boundaries.
- Live ● badge: navigate away from live phase, simulate event arrivals, verify badge counter updates and click-to-return drains correctly.
- Auto-scroll: scroll up mid-reveal, verify auto-scroll disengages; scroll back to bottom, verify it re-engages.

## Migration / Backward Compatibility

- **Default mode is Static.** Any existing call site of `Timeline` that doesn't pass the new mode prop continues to work as today (assume `mode: TimelineMode::Static`, `live_events: None`).
- **API DTOs** — no change. All required data already present.
- **Browser compatibility** — `gloo-timers` already in the dependency tree (used elsewhere in the web crate); no new build deps for v1. CSS transitions are universally supported.

## Open Questions for Implementation

These don't block writing the implementation plan but the implementer will resolve them:

- Final per-card-type `min_dwell_ms` values (the table is starting-point, not gospel).
- Whether the speed selector appears in Live mode or stays Replay-only for v1.
- Whether the phase chip row needs to handle very long games (e.g., 30+ phases) with horizontal scroll or wrapping. (Most games likely under 20 phases, but should not break if larger.)
- Exact backlog-drain threshold (`> 5` events trigger fast-forward) — UX-tunable.
- Whether `EmotionLabel` transitions (from emotions spec) and `WeatherChanged` (from weather spec) become rendered event cards. If yes, they need entries in the `min_dwell_ms` table. (Recommendation: yes, both — they're narrative beats — but coordinate with whichever spec lands first.)
- Whether replay state (current cursor, paused/playing) persists across page navigation or resets on each entry. (Recommendation: reset — replay is an explicit user action, not a saved view state.)

## Out of Scope

- Token-level streaming of announcer message text (streaming-LLM-style typewriter effect).
- Audio cues per event type.
- Per-card-type entrance animations beyond the universal fade + slide.
- Saving replay state across browser sessions.
- Sharing a "replay link" with a starting cursor position.
- Speed multiplier preferences saved to user profile / localStorage.
- Mobile-specific transport layout (chips and transport will reflow with Tailwind defaults; bespoke mobile UX is future work).
