# Game Timeline PR1 — Backend Schema + Combat Refactor + Frontend Stub

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `GameMessage` and friends to `shared/`, replace its untyped `kind: Option<MessageKind>` with a structured `MessagePayload` enum, add `(phase, tick, emit_index)` causal-ordering fields, refactor combat to emit one structured `CombatEngagement` per fight, and ship a frontend stub that keeps `web/` compiling.

**Architecture:** `GameMessage`/`MessagePayload`/`Phase`/`CombatEngagement`/`*Ref` types move from `game/src/messages.rs` to `shared/src/messages.rs`. A new `TaggedEvent { content, payload }` collection type stays in `game/`. Every `&mut Vec<String>` parameter currently used to accumulate per-action prose becomes `&mut Vec<TaggedEvent>`. The drain site at `game/src/games.rs::do_step` (NOT `api/src/games.rs` as the spec says — confirmed L791-1046) builds `GameMessage`s by attaching `(game_day, phase, tick, emit_index)` from a per-period `TickCounter`. The frontend `game_day_log.rs` and `game_day_summary.rs` are deleted; a temporary `game_log_stub.rs` renders raw `content` lines so `web/` keeps building.

**Tech Stack:** Rust 2024 edition, Axum (api), Dioxus (web), SurrealDB. New shared crate deps: `chrono` (with `serde` feature), `serde_json`. Frontend uses no new deps in PR1.

**Spec corrections discovered during planning:**
1. Spec §3 says drain is in `api/src/games.rs ~L957`. **Wrong.** Actual location is `game/src/games.rs` L1001-L1046 inside `Game::do_step`. Plan uses correct location.
2. Current `GameMessage` has `event: Option<GameEvent>` and `event_id: Option<String>` fields not mentioned in spec. Plan removes them with the rest of the schema reshape.
3. Spec lists `game/src/tributes/state.rs` as a file to modify. **No such file exists.** State events (rest, sponsor gift, etc.) live in `tributes/mod.rs`. Plan handles them there.
4. `game/src/areas/` only contains `mod.rs` + `events.rs`; area events already use the typed `AreaEvent` enum on `area_details.events`, not `Vec<String>`. They are emitted as messages from `do_step`, not from area code. No `&mut Vec<TaggedEvent>` thread needed inside `areas/`.

These corrections are reflected below. A spec-followup task at the end updates the spec doc to match.

---

## Task Ordering Rationale

Tasks are grouped to minimize "broken tree" time. The schema move (Tasks 1-3) breaks the `game` crate temporarily; Tasks 4-9 fix it incrementally per emit site; Task 10 fixes the drain; Tasks 11-12 clean up callers; Tasks 13-14 add the new API endpoint; Task 15 stubs the frontend so `web/` compiles. Quality gates (Task 16) run last. Each task ends with a commit.

> **TDD note:** The schema-move tasks (1-3) cannot start with a failing test — they're pure mechanical relocations. New behavior tasks (kind mapping, summarize_periods, Phase parsing, the timeline-summary endpoint) follow strict red-green-commit.

---

## Task 1: Add chrono and serde_json to shared/

**Files:**
- Modify: `shared/Cargo.toml`

- [ ] **Step 1: Inspect current shared/Cargo.toml**

Run: `cat shared/Cargo.toml`
Expected: shows `serde`, `validator`, `uuid` as deps. No `chrono`, no `serde_json`.

- [ ] **Step 2: Add chrono and serde_json**

Edit `shared/Cargo.toml` `[dependencies]` table. Add:

```toml
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1"
```

(Match existing version pins for these crates if already used elsewhere in the workspace — check `game/Cargo.toml` for the chrono version and use the same.)

- [ ] **Step 3: Verify shared crate still builds**

Run: `cargo check -p shared`
Expected: PASS (no compile errors).

- [ ] **Step 4: Commit**

```bash
jj commit -m "chore(shared): add chrono and serde_json deps for GameMessage move"
```

---

## Task 2: Move schema types from game/src/messages.rs to shared/src/messages.rs (no behavior change)

**Files:**
- Create: `shared/src/messages.rs`
- Modify: `shared/src/lib.rs` (add `pub mod messages;`)
- Modify: `game/src/messages.rs` (will become a thin re-export shim)

This task is pure relocation. No type signatures change yet. Reshape happens in Task 3.

- [ ] **Step 1: Read full current game/src/messages.rs**

Run: `wc -l game/src/messages.rs && cat game/src/messages.rs`
Expected: 667 lines. Note: contains `GameMessage`, `MessageKind` (3 variants), `MessageSource`, `serde_event_via_json` module, narrative helpers (`movement_narrative`, `hiding_spot_narrative`, `stamina_narrative`, `terrain_name`).

- [ ] **Step 2: Create shared/src/messages.rs as copy of the schema-only portions**

Move to `shared/src/messages.rs`:
- `GameMessage` struct (lines ~70-110 in current file)
- `MessageKind` enum (lines ~10-30)
- `MessageSource` enum
- `serde_event_via_json` module (lines 32-67) — KEEP TEMPORARILY in shared so the move compiles; will be deleted in Task 3.
- All `impl` blocks for these types
- Existing tests for these types (relocate from `#[cfg(test)] mod tests` — drop only the narrative-helper tests).

Do NOT move:
- `movement_narrative`, `hiding_spot_narrative`, `stamina_narrative`, `terrain_name` (game-only narrative helpers)
- Anything that references `GameEvent` from `game/` (the `serde_event_via_json` module references it — leave the `Option<GameEvent>` field as `Option<serde_json::Value>` temporarily; we delete it in Task 3)

For the temporary move, in `shared/src/messages.rs` replace the `event: Option<GameEvent>` field with `event: Option<serde_json::Value>` (will be removed in Task 3).

- [ ] **Step 3: Add `pub mod messages;` to shared/src/lib.rs**

Edit `shared/src/lib.rs`. Add the module declaration alongside existing `pub mod` lines.

- [ ] **Step 4: Replace game/src/messages.rs with a thin shim**

Rewrite `game/src/messages.rs` to contain ONLY:
- `pub use shared::messages::{GameMessage, MessageKind, MessageSource};`
- The narrative helper fns (`movement_narrative`, `hiding_spot_narrative`, `stamina_narrative`, `terrain_name`) and their tests
- The `with_kind` constructor moved here as a free fn IF any callers still use it (there are 3: `game/src/games.rs:232, 906, 1187, 1223`); easier: keep `with_kind` on the moved type for now and remove it in Task 3.

- [ ] **Step 5: Build the workspace**

Run: `cargo check --workspace`
Expected: PASS. If `api/` or `web/` reference `game::messages::*` paths, those still work via the shim.

- [ ] **Step 6: Run game crate tests**

Run: `cargo test -p game --lib messages`
Expected: PASS (all relocated tests still green).

- [ ] **Step 7: Commit**

```bash
jj commit -m "refactor(shared): move GameMessage and friends from game/ to shared/ (no behavior change)"
```

---

## Task 3: Reshape GameMessage to typed payload + new ordering fields

**Files:**
- Modify: `shared/src/messages.rs`
- Modify: `game/src/messages.rs` (shim — update re-exports)

This task introduces the breaking change. After this task the workspace WILL NOT BUILD until Tasks 4-15 fix every caller.

- [ ] **Step 1: Replace `GameMessage` struct definition**

In `shared/src/messages.rs`, replace the struct with:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMessage {
    pub identifier: String,
    pub source: MessageSource,
    pub game_day: u32,
    pub phase: Phase,
    pub tick: u32,
    pub emit_index: u32,
    pub subject: String,
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub payload: MessagePayload,
}
```

DELETE: the old `kind: Option<MessageKind>` field, the `event: Option<...>` field, the `event_id: Option<String>` field, and the entire `serde_event_via_json` module (no longer needed).

- [ ] **Step 2: Add `Phase` enum**

In `shared/src/messages.rs`:

```rust
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase { Day, Night }

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Day => write!(f, "day"),
            Phase::Night => write!(f, "night"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePhaseError;

impl std::fmt::Display for ParsePhaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "phase must be 'day' or 'night'")
    }
}

impl std::error::Error for ParsePhaseError {}

impl FromStr for Phase {
    type Err = ParsePhaseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "day" => Ok(Phase::Day),
            "night" => Ok(Phase::Night),
            _ => Err(ParsePhaseError),
        }
    }
}
```

- [ ] **Step 3: Add reference structs**

In `shared/src/messages.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TributeRef { pub identifier: String, pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AreaRef { pub identifier: String, pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemRef { pub identifier: String, pub name: String }
```

- [ ] **Step 4: Add `AreaEventKind` and `CombatEngagement` / `CombatOutcome`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaEventKind {
    Hazard, Storm, Mutts, Earthquake, Flood, Fire, Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEngagement {
    pub attacker: TributeRef,
    pub target: TributeRef,
    pub outcome: CombatOutcome,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatOutcome {
    Killed, Wounded, TargetFled, AttackerFled, Stalemate,
}
```

- [ ] **Step 5: Replace `MessageKind` with the new 6-variant enum + add `MessagePayload`**

DELETE the existing 3-variant `MessageKind` (`AllianceFormed`, `BetrayalTriggered`, `TrustShockBreak`).

ADD:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind { Death, Combat, Alliance, Movement, Item, State }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    TributeKilled  { victim: TributeRef, killer: Option<TributeRef>, cause: String },
    TributeWounded { victim: TributeRef, attacker: Option<TributeRef>, hp_lost: u32 },

    Combat(CombatEngagement),

    AllianceFormed     { members: Vec<TributeRef> },
    AllianceProposed   { proposer: TributeRef, target: TributeRef },
    AllianceDissolved  { members: Vec<TributeRef>, reason: String },
    BetrayalTriggered  { betrayer: TributeRef, victim: TributeRef },
    TrustShockBreak    { tribute: TributeRef, partner: TributeRef },

    TributeMoved   { tribute: TributeRef, from: AreaRef, to: AreaRef },
    TributeHidden  { tribute: TributeRef, area: AreaRef },
    AreaClosed     { area: AreaRef },
    AreaEvent      { area: AreaRef, kind: AreaEventKind, description: String },

    ItemFound    { tribute: TributeRef, item: ItemRef, area: AreaRef },
    ItemUsed     { tribute: TributeRef, item: ItemRef },
    ItemDropped  { tribute: TributeRef, item: ItemRef, area: AreaRef },
    SponsorGift  { recipient: TributeRef, item: ItemRef, donor: String },

    TributeRested      { tribute: TributeRef, hp_restored: u32 },
    TributeStarved     { tribute: TributeRef, hp_lost: u32 },
    TributeDehydrated  { tribute: TributeRef, hp_lost: u32 },
    SanityBreak        { tribute: TributeRef },
}

impl MessagePayload {
    pub fn kind(&self) -> MessageKind {
        use MessagePayload::*;
        match self {
            TributeKilled { .. } => MessageKind::Death,
            Combat(_) => MessageKind::Combat,
            AllianceFormed { .. } | AllianceProposed { .. } | AllianceDissolved { .. }
                | BetrayalTriggered { .. } | TrustShockBreak { .. } => MessageKind::Alliance,
            TributeMoved { .. } | TributeHidden { .. }
                | AreaClosed { .. } | AreaEvent { .. } => MessageKind::Movement,
            ItemFound { .. } | ItemUsed { .. }
                | ItemDropped { .. } | SponsorGift { .. } => MessageKind::Item,
            TributeWounded { .. } | TributeRested { .. } | TributeStarved { .. }
                | TributeDehydrated { .. } | SanityBreak { .. } => MessageKind::State,
        }
    }
}
```

- [ ] **Step 6: Replace `GameMessage::new` constructor; delete `with_kind`**

```rust
impl GameMessage {
    pub fn new(
        source: MessageSource,
        game_day: u32,
        phase: Phase,
        tick: u32,
        emit_index: u32,
        subject: String,
        content: String,
        payload: MessagePayload,
    ) -> Self {
        Self {
            identifier: uuid::Uuid::new_v4().to_string(),
            source,
            game_day,
            phase,
            tick,
            emit_index,
            subject,
            timestamp: chrono::Utc::now(),
            content,
            payload,
        }
    }
}
```

DELETE: `with_kind` method entirely.

- [ ] **Step 7: Update game/src/messages.rs shim re-exports**

```rust
pub use shared::messages::{
    AreaEventKind, AreaRef, CombatEngagement, CombatOutcome, GameMessage, ItemRef,
    MessageKind, MessagePayload, MessageSource, ParsePhaseError, Phase, TributeRef,
};
```

Keep narrative helper fns (`movement_narrative`, etc.) in this file.

- [ ] **Step 8: Add `TaggedEvent` to game/src/messages.rs**

After the re-exports, add:

```rust
/// Per-action accumulator: a typed payload plus its already-formatted prose line.
/// Drain site converts each into a `GameMessage` with `(game_day, phase, tick, emit_index)`.
#[derive(Debug, Clone)]
pub struct TaggedEvent {
    pub content: String,
    pub payload: MessagePayload,
}

impl TaggedEvent {
    pub fn new(content: impl Into<String>, payload: MessagePayload) -> Self {
        Self { content: content.into(), payload }
    }
}
```

- [ ] **Step 9: Verify shared crate compiles**

Run: `cargo check -p shared`
Expected: PASS.

(The full workspace is intentionally broken at this point. Tasks 4-15 fix it.)

- [ ] **Step 10: Commit**

```bash
jj commit -m "feat(shared): reshape GameMessage with typed payload and causal-ordering fields"
```

---

## Task 4: Add MessagePayload::kind() unit tests

**Files:**
- Modify: `shared/src/messages.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing test**

Append to the test module:

```rust
#[test]
fn kind_lifecycle_variants_map_correctly() {
    let t = TributeRef { identifier: "t1".into(), name: "T1".into() };
    let p = MessagePayload::TributeKilled {
        victim: t.clone(), killer: None, cause: "fall".into(),
    };
    assert_eq!(p.kind(), MessageKind::Death);

    let p = MessagePayload::TributeWounded {
        victim: t.clone(), attacker: None, hp_lost: 5,
    };
    assert_eq!(p.kind(), MessageKind::State);
}

#[test]
fn kind_combat_maps_to_combat() {
    let a = TributeRef { identifier: "a".into(), name: "A".into() };
    let b = TributeRef { identifier: "b".into(), name: "B".into() };
    let p = MessagePayload::Combat(CombatEngagement {
        attacker: a, target: b,
        outcome: CombatOutcome::Wounded,
        detail_lines: vec![],
    });
    assert_eq!(p.kind(), MessageKind::Combat);
}

#[test]
fn kind_alliance_variants_all_map_to_alliance() {
    let a = TributeRef { identifier: "a".into(), name: "A".into() };
    let b = TributeRef { identifier: "b".into(), name: "B".into() };
    let cases = vec![
        MessagePayload::AllianceFormed { members: vec![a.clone(), b.clone()] },
        MessagePayload::AllianceProposed { proposer: a.clone(), target: b.clone() },
        MessagePayload::AllianceDissolved { members: vec![a.clone()], reason: "x".into() },
        MessagePayload::BetrayalTriggered { betrayer: a.clone(), victim: b.clone() },
        MessagePayload::TrustShockBreak { tribute: a.clone(), partner: b.clone() },
    ];
    for c in cases {
        assert_eq!(c.kind(), MessageKind::Alliance);
    }
}

#[test]
fn kind_movement_variants_map_to_movement() {
    let t = TributeRef { identifier: "t".into(), name: "T".into() };
    let area = AreaRef { identifier: "a".into(), name: "Ar".into() };
    let cases = vec![
        MessagePayload::TributeMoved { tribute: t.clone(), from: area.clone(), to: area.clone() },
        MessagePayload::TributeHidden { tribute: t.clone(), area: area.clone() },
        MessagePayload::AreaClosed { area: area.clone() },
        MessagePayload::AreaEvent { area: area.clone(), kind: AreaEventKind::Storm, description: "s".into() },
    ];
    for c in cases {
        assert_eq!(c.kind(), MessageKind::Movement);
    }
}

#[test]
fn kind_item_variants_map_to_item() {
    let t = TributeRef { identifier: "t".into(), name: "T".into() };
    let i = ItemRef { identifier: "i".into(), name: "I".into() };
    let area = AreaRef { identifier: "a".into(), name: "Ar".into() };
    let cases = vec![
        MessagePayload::ItemFound { tribute: t.clone(), item: i.clone(), area: area.clone() },
        MessagePayload::ItemUsed { tribute: t.clone(), item: i.clone() },
        MessagePayload::ItemDropped { tribute: t.clone(), item: i.clone(), area: area.clone() },
        MessagePayload::SponsorGift { recipient: t.clone(), item: i.clone(), donor: "d".into() },
    ];
    for c in cases {
        assert_eq!(c.kind(), MessageKind::Item);
    }
}

#[test]
fn kind_state_variants_map_to_state() {
    let t = TributeRef { identifier: "t".into(), name: "T".into() };
    let cases = vec![
        MessagePayload::TributeRested { tribute: t.clone(), hp_restored: 1 },
        MessagePayload::TributeStarved { tribute: t.clone(), hp_lost: 1 },
        MessagePayload::TributeDehydrated { tribute: t.clone(), hp_lost: 1 },
        MessagePayload::SanityBreak { tribute: t.clone() },
    ];
    for c in cases {
        assert_eq!(c.kind(), MessageKind::State);
    }
}

#[test]
fn unknown_payload_tag_hard_errors() {
    let json = r#"{"type":"FutureKindNobodyKnows","extra":"x"}"#;
    let result: Result<MessagePayload, _> = serde_json::from_str(json);
    assert!(result.is_err(), "unknown tag must hard-error, no silent default");
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p shared --lib messages::tests::kind`
Expected: PASS for all 6 `kind_*` tests + the unknown-tag hard-error test.

- [ ] **Step 3: Commit**

```bash
jj commit -m "test(shared): cover MessagePayload::kind() and unknown-tag hard-error"
```

---

## Task 5: Add Phase enum tests

**Files:**
- Modify: `shared/src/messages.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing tests**

Append to the test module:

```rust
#[test]
fn phase_from_str_accepts_lowercase_only() {
    assert_eq!(Phase::from_str("day").unwrap(), Phase::Day);
    assert_eq!(Phase::from_str("night").unwrap(), Phase::Night);
}

#[test]
fn phase_from_str_rejects_mixed_case_and_garbage() {
    assert!(Phase::from_str("Day").is_err());
    assert!(Phase::from_str("NIGHT").is_err());
    assert!(Phase::from_str("sideways").is_err());
    assert!(Phase::from_str("").is_err());
    assert!(Phase::from_str(" day ").is_err());
}

#[test]
fn phase_display_round_trip() {
    for p in [Phase::Day, Phase::Night] {
        let s = p.to_string();
        let back = Phase::from_str(&s).unwrap();
        assert_eq!(p, back);
    }
}

#[test]
fn phase_serde_round_trip() {
    let p = Phase::Day;
    let s = serde_json::to_string(&p).unwrap();
    assert_eq!(s, "\"day\"");
    let back: Phase = serde_json::from_str(&s).unwrap();
    assert_eq!(p, back);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p shared --lib messages::tests::phase`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj commit -m "test(shared): cover Phase FromStr/Display/serde round-trips"
```

---

## Task 6: Refactor combat.rs to emit TaggedEvent (one Combat per fight)

**Files:**
- Modify: `game/src/tributes/combat.rs` (787 lines; `attacks` at L34, `apply_violence_stress` at L195, helpers at L254 and L366)

- [ ] **Step 1: Read the current combat.rs**

Run: `cat game/src/tributes/combat.rs`
Note: 4 fns take `events: &mut Vec<String>`. `attacks()` pushes 3-6 lines per call.

- [ ] **Step 2: Write a failing test for the new contract**

Add to the existing combat tests module:

```rust
#[rstest]
fn attacks_emits_one_combat_taggedevent(/* existing fixtures */) {
    // Construct attacker, target, world per existing test pattern
    let mut events: Vec<TaggedEvent> = Vec::new();
    attacker.attacks(&mut target, &mut events, /* other params */);

    let combat_events: Vec<_> = events.iter()
        .filter(|e| matches!(e.payload, MessagePayload::Combat(_)))
        .collect();
    assert_eq!(combat_events.len(), 1, "exactly one Combat payload per attacks() call");

    if let MessagePayload::Combat(eng) = &combat_events[0].payload {
        assert_eq!(eng.attacker.name, attacker.name);
        assert_eq!(eng.target.name, target.name);
        assert!(matches!(eng.outcome, CombatOutcome::Killed | CombatOutcome::Wounded
            | CombatOutcome::TargetFled | CombatOutcome::AttackerFled | CombatOutcome::Stalemate));
    } else {
        panic!("expected Combat payload");
    }
}
```

(Match the existing `#[rstest]` fixture pattern in the file. If the file already has a working "attacks_kills_target" test or similar, model after that.)

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p game --lib tributes::combat::tests::attacks_emits_one_combat_taggedevent`
Expected: FAIL (the function still takes `&mut Vec<String>`).

- [ ] **Step 4: Change function signatures**

In `game/src/tributes/combat.rs`, change the 4 `&mut Vec<String>` parameters to `&mut Vec<TaggedEvent>`:
- `attacks` (L34)
- `apply_violence_stress` (L195)
- helper at L254
- helper at L366

Add `use crate::messages::{TaggedEvent, MessagePayload, CombatEngagement, CombatOutcome, TributeRef};` at the top.

- [ ] **Step 5: Replace per-line `events.push(string)` with one Combat TaggedEvent at end of attacks()**

Inside `attacks()`, accumulate prose into a local `let mut detail_lines: Vec<String> = Vec::new();` instead of pushing to `events`. Track `outcome: CombatOutcome` based on the existing branches:
- Tribute dies → `CombatOutcome::Killed`
- HP reduced but alive → `CombatOutcome::Wounded`
- Target flees → `CombatOutcome::TargetFled`
- Attacker flees → `CombatOutcome::AttackerFled`
- No one lands a blow → `CombatOutcome::Stalemate`

For lines that today are SELF-HARM / SUICIDE / CRITICAL FUMBLE (combat.rs L38, L55, L80) — these are NOT engagements. Emit them as their own `TaggedEvent` with `MessagePayload::TributeKilled` (suicide/self-harm-fatal) or as state-change payloads, NOT as part of a Combat engagement.

At the end of an actual two-tribute engagement, push one:

```rust
let engagement = CombatEngagement {
    attacker: TributeRef { identifier: self.identifier.clone(), name: self.name.clone() },
    target:   TributeRef { identifier: target.identifier.clone(), name: target.name.clone() },
    outcome,
    detail_lines: detail_lines.clone(),
};
let summary = format!("{} attacks {} ({:?})", self.name, target.name, outcome);
events.push(TaggedEvent::new(summary, MessagePayload::Combat(engagement)));
```

`apply_violence_stress` and the helpers: same treatment — convert each `events.push(string)` to a `TaggedEvent::new(string, MessagePayload::<appropriate variant>)`. For violence-stress-induced sanity breaks, use `MessagePayload::SanityBreak { tribute: TributeRef { .. } }`.

- [ ] **Step 6: Run combat tests**

Run: `cargo test -p game --lib tributes::combat`
Expected: New test PASSES. Existing combat tests fail until they're updated in Step 7.

- [ ] **Step 7: Update existing combat tests**

For every existing combat rstest in this file, change `let mut events: Vec<String> = Vec::new();` to `let mut events: Vec<TaggedEvent> = Vec::new();`. Change assertions on event strings to assertions on payload variants. Example:

```rust
// Before:
assert!(events.iter().any(|e| e.contains("dies")));
// After:
assert!(events.iter().any(|e| matches!(e.payload, MessagePayload::TributeKilled { .. })
    || matches!(&e.payload, MessagePayload::Combat(c) if c.outcome == CombatOutcome::Killed)));
```

- [ ] **Step 8: Run combat tests, all passing**

Run: `cargo test -p game --lib tributes::combat`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
jj commit -m "refactor(game): emit one Combat TaggedEvent per attacks() call"
```

---

## Task 7: Refactor lifecycle.rs and movement.rs to use TaggedEvent

**Files:**
- Modify: `game/src/tributes/lifecycle.rs` (485 lines; `&mut Vec<String>` at L210, push at L282; AreaEvent push at L480)
- Modify: `game/src/tributes/movement.rs` (213 lines; `&mut Vec<String>` at L30, pushes at L39, L53, L69, L80, L95, L114, L129)

- [ ] **Step 1: Update lifecycle.rs**

Change the `events: &mut Vec<String>` parameter (L210, in `take_damage` or equivalent — confirm by reading the fn signature) to `events: &mut Vec<TaggedEvent>`. Replace L282's `events.push(string)` with `events.push(TaggedEvent::new(line, MessagePayload::TributeKilled { victim: ..., killer: ..., cause: ... }))`. Source the `killer` field from the existing damage-source argument; `cause` is a short string like `"combat"`, `"poison"`, `"fall"`, `"starvation"` — pick based on the call context branch.

L480's `area_details.events.push(AreaEvent::Wildfire)` is unchanged — that's an `AreaEvent` enum push, not a string push.

Add `use crate::messages::{TaggedEvent, MessagePayload, TributeRef};` at the top.

- [ ] **Step 2: Update movement.rs**

Change `events: &mut Vec<String>` (L30, in `move_to_area` or equivalent) to `events: &mut Vec<TaggedEvent>`.

Each push site (L39, L53, L69, L80, L95, L114, L129) becomes a `TaggedEvent` with `MessagePayload::TributeMoved { tribute, from, to }` or `MessagePayload::TributeHidden { tribute, area }` based on the branch. Source `from`/`to` from the existing area variables in scope; `tribute` from `self`.

Add `use crate::messages::{TaggedEvent, MessagePayload, TributeRef, AreaRef};` at the top.

- [ ] **Step 3: Verify game crate compiles up to mod.rs callers**

Run: `cargo check -p game --lib 2>&1 | head -50`
Expected: Errors only in `game/src/tributes/mod.rs` (callers not yet updated) and `game/src/games.rs` (drain not yet updated). lifecycle.rs and movement.rs themselves compile clean.

- [ ] **Step 4: Commit**

```bash
jj commit -m "refactor(game): convert lifecycle and movement to TaggedEvent"
```

---

## Task 8: Refactor tributes/mod.rs to use TaggedEvent

**Files:**
- Modify: `game/src/tributes/mod.rs` (1021 lines; `&mut Vec<String>` at L214, L430, L497; pushes at L218, L238, L274, L333, L354, L360, L363, L387, L396, L437, L512; 8 internal Vec<String> declarations between L832-L1000)

> **Note:** `game/src/tributes/alliances.rs` (473 lines) was confirmed to have NO `events.push` and NO `&mut Vec<String>` parameters — it is pure decision logic (gates, rolls, factors, `try_form_alliance`). Alliance message emission happens in `game/src/games.rs::do_step` and is handled by Task 10 Steps 4-5.

- [ ] **Step 1: Update mod.rs three public methods**

For `do_day_action` (L214), `do_night_action` (L430), and the third method (L497), change `events: &mut Vec<String>` to `events: &mut Vec<TaggedEvent>`.

For each push site, choose the payload by branch:
- L218: `TributeAlreadyDead` — this is informational, no useful payload. Use `MessagePayload::TributeWounded { victim, attacker: None, hp_lost: 0 }` is wrong; better: drop this push entirely (skip emit when already dead), as the simulator already short-circuits.
- L238: `TributeDead` — `MessagePayload::TributeKilled { victim, killer: None, cause: "untracked".into() }` (rare branch).
- L274: `SponsorGift` — currently no emit per spec §2 LOW-3, BUT this code already exists. Either keep emitting `MessagePayload::SponsorGift { recipient, item, donor: "Sponsor".into() }` (preferred — the variant exists) or drop the push. **Prefer keeping it; revisit if too noisy.**
- L333, L387, L396: depend on branch context — read surrounding code; most are state-change events. Map to `MessagePayload::SanityBreak { .. }` or similar based on what the line text says.
- L354: `TributeRest` → `MessagePayload::TributeRested { tribute, hp_restored: <local var or 0> }`.
- L360, L363: `TributeHide` → `MessagePayload::TributeHidden { tribute, area }`.
- L437: `TributeSuicide` → `MessagePayload::TributeKilled { victim, killer: None, cause: "suicide".into() }`.
- L512: format-string event — read surrounding code; map by branch.

- [ ] **Step 2: Update internal Vec<String> declarations**

The 8 internal `let mut events: Vec<String> = Vec::new();` (L832, L853, L875, L899, L911, L925, L978, L1000) become `let mut events: Vec<TaggedEvent> = Vec::new();`. They are then drained into the parent context — confirm by reading each. If they're consumed locally to format a final string, leave the type as `Vec<String>` and rename to `lines` to avoid the typename clash. If they're pushed into the outer `events: &mut Vec<TaggedEvent>`, they must be `Vec<TaggedEvent>` and pushed via `events.extend(local_events)`.

- [ ] **Step 3: Verify game crate compiles up to games.rs**

Run: `cargo check -p game --lib 2>&1 | grep -E '^(error|warning)' | head -30`
Expected: Errors only in `game/src/games.rs` (drain not yet updated). Tribute module clean.

- [ ] **Step 4: Commit**

```bash
jj commit -m "refactor(game): convert tributes mod to TaggedEvent"
```

---

## Task 9: Add TickCounter and CycleContext to Game

**Files:**
- Modify: `game/src/games.rs` (struct and impl Game)

- [ ] **Step 1: Add TickCounter struct**

Near the top of `game/src/games.rs` after the existing imports:

```rust
/// Per-period tick counter. Resets to 0 at every phase boundary.
/// Phase-boundary side-effect messages get tick=0.
/// First action in a phase gets tick=1.
#[derive(Debug, Default, Clone, Copy)]
pub struct TickCounter { current: u32 }

impl TickCounter {
    pub fn reset(&mut self) { self.current = 0; }
    pub fn next(&mut self) -> u32 {
        self.current += 1;
        self.current
    }
    pub fn boundary(&self) -> u32 { 0 }
}
```

- [ ] **Step 2: Add a tick_counter field to Game (transient, not persisted)**

In the `Game` struct, add:

```rust
#[serde(skip, default)]
pub tick_counter: TickCounter,
```

- [ ] **Step 3: Verify it compiles standalone**

Run: `cargo check -p game --lib 2>&1 | grep TickCounter`
Expected: No errors mentioning TickCounter.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(game): add TickCounter for per-period causal ordering"
```

---

## Task 10: Refactor games.rs::do_step drain to build typed GameMessages

**Files:**
- Modify: `game/src/games.rs` (L201, L212, L224, L232, L791-L1046 — main drain block, plus tests at L906, L1187, L1223, L2158, L2165, L2208, L2257, L2347)

- [ ] **Step 1: Inspect the drain block**

Run: `sed -n '780,1050p' game/src/games.rs`
Expected: Read the per-tribute action loop. Identify where `Vec<String>` events are drained into `GameMessage`s.

- [ ] **Step 2: Change all internal events accumulators to Vec<TaggedEvent>**

In `do_step`, convert every `let mut events: Vec<String> = Vec::new();` (and any `Vec<String>` passed to tribute methods) to `Vec<TaggedEvent>`.

The drain block (~L1001) currently calls `log_output` / `log_output_kind` per string. Replace with a per-event loop that:

```rust
let phase = if game.is_day { Phase::Day } else { Phase::Night };
let mut emit_index: u32 = 0;
let tick = game.tick_counter.next();

for tagged in events.drain(..) {
    let msg = GameMessage::new(
        MessageSource::Tribute(tribute.identifier.clone()),
        game.day,
        phase,
        tick,
        emit_index,
        tribute.name.clone(),  // subject
        tagged.content,
        tagged.payload,
    );
    game.messages.push(msg);
    emit_index += 1;
}
```

(Adjust `MessageSource` per the actual emit context — `Game(...)`, `Area(...)`, or `Tribute(...)`.)

`emit_index` is per-period, NOT per-tick. Hoist its declaration to the top of the per-period block; reset it on phase boundary in step 6.

- [ ] **Step 3: Delete `log_output` and `log_output_kind`**

Remove the two methods at L212 and L224 entirely. All callers were just rewritten in Step 2.

- [ ] **Step 4: Update the existing alliance-formed emit (L902-L906)**

Replace:
```rust
collected_events.push((
    /* ... */,
    Some(crate::messages::MessageKind::AllianceFormed),
));
```
With code that builds a `MessagePayload::AllianceFormed { members: Vec<TributeRef> }` directly into a `GameMessage::new(...)` push onto `game.messages`. Use `game.tick_counter.next()` for the tick and a per-period `emit_index` (separate counter from the action drain — alliance emits happen outside the action loop in this code path).

- [ ] **Step 5: Update betrayal/trust-shock emit sites (L1187, L1223)**

Similarly: build `MessagePayload::BetrayalTriggered { betrayer, victim }` and `MessagePayload::TrustShockBreak { tribute, partner }` payloads. Construct `TributeRef`s from the in-scope tribute structs.

- [ ] **Step 6: Reset TickCounter on phase boundary**

Find the place in `do_step` (or its phase-advance helper) where `game.is_day` flips. Immediately after the flip, call `game.tick_counter.reset();`.

- [ ] **Step 7: Update tests in games.rs**

The 5 test sites at L2158, L2165, L2208, L2257, L2347 currently assert `m.kind == Some(MessageKind::AllianceFormed)` etc. Change to `matches!(m.payload, MessagePayload::AllianceFormed { .. })` and similar for `BetrayalTriggered`.

- [ ] **Step 8: Run game crate tests**

Run: `cargo test -p game --lib`
Expected: PASS. (Some tests may need additional cleanup if they construct `GameMessage` directly — fix in place.)

- [ ] **Step 9: Verify the workspace compiles minus web/api**

Run: `cargo check -p game`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
jj commit -m "refactor(game): drain TaggedEvents into typed GameMessages with tick/emit_index"
```

---

## Task 11: Add summarize_periods to shared/

**Files:**
- Modify: `shared/src/messages.rs` (add fn + tests)

- [ ] **Step 1: Add the PeriodSummary struct and stub fn**

Append to `shared/src/messages.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeriodSummary {
    pub day: u32,
    pub phase: Phase,
    pub deaths: u32,
    pub event_count: u32,
    pub is_current: bool,
}

/// Aggregate messages into one summary per (day, phase). Includes empty periods
/// up to and including `current` so the hub shows the live period even when
/// nothing has been emitted there yet. Periods past `current` are not emitted.
pub fn summarize_periods(messages: &[GameMessage], current: (u32, Phase)) -> Vec<PeriodSummary> {
    use std::collections::BTreeMap;

    let (current_day, current_phase) = current;
    let mut bucket: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    // key: (day, phase as u32)  value: (deaths, event_count)
    let phase_ord = |p: Phase| match p { Phase::Day => 0, Phase::Night => 1 };

    for m in messages {
        let key = (m.game_day, phase_ord(m.phase));
        let entry = bucket.entry(key).or_insert((0, 0));
        entry.1 += 1;
        if matches!(m.payload, MessagePayload::TributeKilled { .. }) {
            entry.0 += 1;
        }
    }

    // Ensure all reached periods appear, even with zero messages.
    for d in 1..=current_day {
        bucket.entry((d, 0)).or_insert((0, 0));
        if d < current_day || matches!(current_phase, Phase::Night) {
            bucket.entry((d, 1)).or_insert((0, 0));
        }
    }

    bucket.into_iter().map(|((day, p), (deaths, count))| {
        let phase = if p == 0 { Phase::Day } else { Phase::Night };
        PeriodSummary {
            day, phase, deaths, event_count: count,
            is_current: day == current_day && phase == current_phase,
        }
    }).collect()
}
```

- [ ] **Step 2: Write the failing tests**

Append to the test module:

```rust
fn make_msg(day: u32, phase: Phase, payload: MessagePayload) -> GameMessage {
    GameMessage::new(
        MessageSource::Game("g".into()),
        day, phase, 1, 0,
        "subject".into(), "content".into(), payload,
    )
}

#[test]
fn summarize_empty_input_with_current_day_zero() {
    let result = summarize_periods(&[], (0, Phase::Day));
    assert!(result.is_empty(), "no periods reached when current_day=0");
}

#[test]
fn summarize_groups_by_day_and_phase() {
    let t = TributeRef { identifier: "t".into(), name: "T".into() };
    let killed = MessagePayload::TributeKilled { victim: t.clone(), killer: None, cause: "x".into() };
    let moved = MessagePayload::TributeHidden { tribute: t.clone(), area: AreaRef { identifier: "a".into(), name: "A".into() } };

    let msgs = vec![
        make_msg(1, Phase::Day, killed.clone()),
        make_msg(1, Phase::Day, moved.clone()),
        make_msg(1, Phase::Night, moved.clone()),
        make_msg(2, Phase::Day, killed.clone()),
    ];
    let result = summarize_periods(&msgs, (2, Phase::Day));
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], PeriodSummary { day: 1, phase: Phase::Day, deaths: 1, event_count: 2, is_current: false });
    assert_eq!(result[1], PeriodSummary { day: 1, phase: Phase::Night, deaths: 0, event_count: 1, is_current: false });
    assert_eq!(result[2], PeriodSummary { day: 2, phase: Phase::Day, deaths: 1, event_count: 1, is_current: true });
}

#[test]
fn summarize_includes_empty_reached_periods() {
    let result = summarize_periods(&[], (2, Phase::Day));
    // Day 1 (Day + Night) reached, Day 2 Day reached, Day 2 Night not reached
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], PeriodSummary { day: 1, phase: Phase::Day, deaths: 0, event_count: 0, is_current: false });
    assert_eq!(result[1], PeriodSummary { day: 1, phase: Phase::Night, deaths: 0, event_count: 0, is_current: false });
    assert_eq!(result[2], PeriodSummary { day: 2, phase: Phase::Day, deaths: 0, event_count: 0, is_current: true });
}

#[test]
fn summarize_is_current_flag_set_correctly() {
    let t = TributeRef { identifier: "t".into(), name: "T".into() };
    let p = MessagePayload::TributeRested { tribute: t, hp_restored: 1 };
    let msgs = vec![make_msg(2, Phase::Night, p.clone())];
    let result = summarize_periods(&msgs, (2, Phase::Night));
    let current: Vec<_> = result.iter().filter(|s| s.is_current).collect();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].day, 2);
    assert_eq!(current[0].phase, Phase::Night);
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p shared --lib messages::tests::summarize`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(shared): add summarize_periods aggregation for timeline summary"
```

---

## Task 12: Add causal-ordering regression test

**Files:**
- Modify: `game/src/games.rs` (test module)

- [ ] **Step 1: Write the failing test**

Add to the `tests` mod in `games.rs`:

```rust
#[test]
fn dead_tribute_has_no_movement_event_after_death_in_same_period() {
    // Build a small scenario: tribute B killed by tribute A in Day 1.
    // After do_step, no MessagePayload::TributeMoved { tribute: B, .. }
    // should appear at a later (tick, emit_index) than B's TributeKilled.
    let mut game = /* construct game with A and B in same area, A guaranteed to win */;
    game.do_step(&mut rand::thread_rng());

    let b_killed = game.messages.iter()
        .find(|m| matches!(&m.payload,
            MessagePayload::TributeKilled { victim, .. } if victim.name == "B"));
    let b_killed = b_killed.expect("B should have died");

    let later_b_move = game.messages.iter().find(|m| {
        m.game_day == b_killed.game_day
            && m.phase == b_killed.phase
            && (m.tick, m.emit_index) > (b_killed.tick, b_killed.emit_index)
            && matches!(&m.payload,
                MessagePayload::TributeMoved { tribute, .. } if tribute.name == "B")
    });

    assert!(later_b_move.is_none(), "no TributeMoved for B after B dies in same period");
}
```

(If existing test scaffolding doesn't make this scenario easy to construct, mark as `#[ignore]` with a note pointing to the scenario gap; do NOT skip the assertion structure.)

- [ ] **Step 2: Run test**

Run: `cargo test -p game --lib games::tests::dead_tribute_has_no_movement`
Expected: PASS (or `#[ignore]`'d if scenario can't be set up).

- [ ] **Step 3: Commit**

```bash
jj commit -m "test(game): regression test for causal ordering of TributeKilled vs TributeMoved"
```

---

## Task 13: Update API drain and serialization

**Files:**
- Modify: `api/src/games.rs` (any `kind` references for `GameMessage`)
- Modify: `api/tests/games_tests.rs`

- [ ] **Step 1: Find broken sites**

Run: `cargo check -p api 2>&1 | grep -E '^(error|warning)' | head -40`
Expected: Errors at any site referencing `GameMessage.kind` or `MessageKind::AllianceFormed/BetrayalTriggered/TrustShockBreak` (the old 3-variant enum).

- [ ] **Step 2: Update each error site**

For each error, replace `kind: Some(MessageKind::Xxx)` checks with `matches!(payload, MessagePayload::Xxx { .. })`. Replace `MessageKind` use statements with `MessagePayload`.

- [ ] **Step 3: Update api integration tests**

In `api/tests/games_tests.rs`, change assertions on the old `kind` field to assertions on `payload` variants. Add tests for the new `phase`, `tick`, `emit_index` fields appearing in `/log/:day` responses.

- [ ] **Step 4: Verify api compiles**

Run: `cargo check -p api`
Expected: PASS.

- [ ] **Step 5: Run api tests**

Run: `cargo test -p api`
Expected: PASS. (If SurrealDB-dependent tests fail because the runner has no DB, they were already broken — note in commit message and skip.)

- [ ] **Step 6: Commit**

```bash
jj commit -m "refactor(api): adapt to new GameMessage payload shape"
```

---

## Task 14: Add GET /api/games/:id/timeline-summary endpoint

**Files:**
- Modify: `api/src/games.rs` (add handler, register route)
- Modify: `api/tests/games_tests.rs` (integration test)

- [ ] **Step 1: Write the failing integration test**

In `api/tests/games_tests.rs`:

```rust
#[tokio::test]
async fn timeline_summary_empty_for_unstarted_game() {
    let app = test_app().await;
    let game_id = create_game(&app).await;

    let resp = app
        .request(format!("/api/games/{}/timeline-summary", game_id))
        .send().await;
    assert_eq!(resp.status(), 200);

    let body: Vec<shared::messages::PeriodSummary> = resp.json().await;
    assert!(body.is_empty(), "unstarted game has no reached periods");
}

#[tokio::test]
async fn timeline_summary_includes_current_period_even_when_empty() {
    let app = test_app().await;
    let game_id = create_game_with_day(&app, 1).await;  // helper that advances to day 1

    let resp = app.request(format!("/api/games/{}/timeline-summary", game_id)).send().await;
    let body: Vec<shared::messages::PeriodSummary> = resp.json().await;
    assert!(body.iter().any(|s| s.day == 1 && s.is_current));
}
```

(Adapt to the existing `tests/games_tests.rs` style — match how other endpoints are tested.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p api timeline_summary`
Expected: FAIL (route doesn't exist → 404).

- [ ] **Step 3: Add the handler**

In `api/src/games.rs`:

```rust
async fn timeline_summary(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> Result<Json<Vec<shared::messages::PeriodSummary>>, ApiError> {
    let game = state.db.load_game(&game_id).await?;
    let messages = state.db.load_messages_for_game(&game_id).await?;
    let current_phase = if game.is_day { shared::messages::Phase::Day } else { shared::messages::Phase::Night };
    let summaries = shared::messages::summarize_periods(&messages, (game.current_day, current_phase));
    Ok(Json(summaries))
}
```

(Adapt names to actual State and DB-access patterns in the file. `game.current_day` and `game.is_day` may have different names — read the `Game` struct to confirm.)

- [ ] **Step 4: Register the route**

In whatever fn builds the api router (probably near other `/api/games/:id/*` routes):

```rust
.route("/api/games/:id/timeline-summary", get(timeline_summary))
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p api timeline_summary`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(api): add /timeline-summary endpoint backed by summarize_periods"
```

---

## Task 15: Stub the frontend so web/ compiles

**Files:**
- Delete: `web/src/components/game_day_log.rs`
- Delete: `web/src/components/game_day_summary.rs`
- Create: `web/src/components/game_log_stub.rs`
- Modify: `web/src/components/mod.rs` (or wherever components are registered)
- Modify: `web/src/components/game_detail.rs` (replace day-log / day-summary calls with the stub)

- [ ] **Step 1: Find current usages**

Run: `rg 'game_day_log|game_day_summary|GameDayLog|GameDaySummary' web/src/`
Expected: Lists the import sites and component-call sites in `game_detail.rs` and `mod.rs`.

- [ ] **Step 2: Create web/src/components/game_log_stub.rs**

```rust
use dioxus::prelude::*;
use shared::messages::GameMessage;

#[component]
pub fn GameLogStub(messages: Vec<GameMessage>) -> Element {
    rsx! {
        div { class: "border border-yellow-400 bg-yellow-50 p-3 my-2 text-xs",
            p { class: "font-bold mb-2", "[PR1 stub — full timeline lands in PR2]" }
            ul { class: "space-y-1",
                for m in messages.iter() {
                    li {
                        span { class: "text-gray-500", "Day {m.game_day} {m.phase} t{m.tick}.{m.emit_index} — " }
                        "{m.content}"
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 3: Delete the old components**

```bash
rm -f web/src/components/game_day_log.rs web/src/components/game_day_summary.rs
```

- [ ] **Step 4: Update web/src/components/mod.rs**

Remove `pub mod game_day_log;` and `pub mod game_day_summary;` lines. Add `pub mod game_log_stub;`.

- [ ] **Step 5: Update game_detail.rs**

Replace the call sites for `<GameDayLog ... />` and `<GameDaySummary ... />` (lines ~220-630 per spec) with a single `<GameLogStub messages={...} />` call. The `messages` prop comes from the existing `/api/games/:id/log/:day` query — collect across days or keep the existing per-day fetch and pass that day's messages.

To keep this PR small and avoid touching the larger game_detail.rs structure: leave the per-day loops in place but inside each day's section, replace `<GameDayLog />` with `<GameLogStub messages={day_messages.clone()} />`. Strip `<GameDaySummary />` entirely.

- [ ] **Step 6: Build web crate**

Run: `cd web && RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo check --target wasm32-unknown-unknown`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
jj commit -m "refactor(web): stub day log to keep web crate compiling for PR1"
```

---

## Task 16: Quality gates and spec correction

**Files:**
- Modify: `docs/superpowers/specs/2026-04-26-game-timeline-redesign.md` (correct drain location and state.rs reference)

- [ ] **Step 1: Run full quality suite**

Run: `just quality`
Expected: PASS (fmt, check, clippy, test).

- [ ] **Step 2: If any failures, fix in place and re-run**

Iterate until clean. Commit fixes individually with descriptive messages.

- [ ] **Step 3: Correct the spec**

Edit `docs/superpowers/specs/2026-04-26-game-timeline-redesign.md`:
- §3 "Drain site" line: change `api/src/games.rs ~L957` to `game/src/games.rs ~L1001 (inside Game::do_step)`.
- §3 emit-sites bullet list: remove `game/src/tributes/state.rs (rest, starve, dehydrate, sanity-break helpers)` (no such file). Replace with: `game/src/tributes/mod.rs (rest, sponsor, hide, suicide, sanity helpers across do_day_action/do_night_action)`.
- §2 "Changes to GameMessage": add a one-line note that the legacy `event: Option<GameEvent>` and `event_id: Option<String>` fields plus the `serde_event_via_json` helper module are also removed by this PR.

- [ ] **Step 4: Commit spec correction**

```bash
jj commit -m "docs(spec): correct drain location and state.rs reference for timeline redesign"
```

---

## Task 17: Open the PR

- [ ] **Step 1: Sync with remote**

```bash
jj git fetch
jj rebase -d main@origin
```

- [ ] **Step 2: Push beads (if any new issues filed during the work)**

```bash
bd dolt push
```

- [ ] **Step 3: Create the bookmark and push**

```bash
jj bookmark create feat-timeline-pr1-backend -r @-
jj git push --bookmark feat-timeline-pr1-backend
```

- [ ] **Step 4: Open the PR**

```bash
gh pr create --base main --head feat-timeline-pr1-backend \
  --title "feat(game,api,shared): timeline schema reshape + combat refactor (PR1 of 2)" \
  --body "$(cat <<'EOF'
## Summary
PR1 of 2 for the game timeline redesign. Backend schema move + combat refactor + frontend stub. Spec: docs/superpowers/specs/2026-04-26-game-timeline-redesign.md.

## Changes
- Move GameMessage / Phase / CombatEngagement / *Ref types from game/ to shared/
- Replace kind: Option<MessageKind> with structured payload: MessagePayload (22 variants in 6 categories)
- Add (phase, tick, emit_index) for causal ordering; TickCounter on Game
- Refactor combat to emit one CombatEngagement per attacks() call (was 3-6 lines)
- Convert all &mut Vec<String> emit sites in tributes/ to &mut Vec<TaggedEvent>
- New GET /api/games/:id/timeline-summary endpoint
- Delete game_day_log.rs and game_day_summary.rs; add temporary game_log_stub.rs
- Spec correction commit (drain location, state.rs reference)

## Verification
- just quality: PASS
- cargo test -p shared: PASS (+ new MessagePayload, Phase, summarize_periods coverage)
- cargo test -p game: PASS (+ updated combat assertions, causal-ordering regression test)
- cargo test -p api: PASS (+ /timeline-summary integration)
- web/ builds for wasm32

## Breaking
- Dev DB MUST be wiped before deploy. Unknown payload tags hard-error on deserialize.

## Follow-ups
- PR2 (frontend timeline UI): RecapCard, PeriodGrid, FilterChips, timeline cards.
- Beads issues to be filed at PR2 close-out per spec §7.
EOF
)"
```

- [ ] **Step 5: Verify PR URL**

Output the PR URL from the previous command. Hand off to user.
