# Unify Event Systems Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore full per-cycle narration to `Game.messages` by replacing the gutted `try_log_action` no-op with a real event-collection mechanism, and surface previously-lost survival outcomes — without re-introducing I/O into the engine crate.

**Architecture:** Engine functions return `Vec<GameOutput<'_>>` from each behavior site (combat, movement, turn-phase, status death, area events). The cycle loop in `game/src/games.rs` drains those vectors and calls `Game::log` to convert each `GameOutput` into a `GameMessage` (free-form `content: String` via `Display`). `MessageSource::Tribute(actor.identifier)` for tribute-driven events; `Area(area_name)` for area events; `Game(game.identifier)` for cycle bookends and winner. No new types in this plan — `GameOutput` is reused as-is. (Follow-up: `hangrier_games-mqi` will replace it with a serde-friendly `GameEvent`.)

**Tech Stack:** Rust 2024 edition, `game/` crate (pure, no I/O), `api/` crate (axum, drains via existing `save_game`).

**Scope split into three PRs:**
1. **PR1 — Combat narration restoration.** Wire combat call sites (~16) in `tributes/combat.rs` to return events; cycle loop logs them. Restore `WeaponBreak`/`ShieldBreak`/`TributeAttackDied` paths. Status-death narration in `lifecycle.rs:277` also lands here (single site, same shape).
2. **PR2 — Movement and turn-phase narration.** Wire `tributes/movement.rs` (~7) and `tributes/mod.rs::process_turn_phase` (~11). Includes `TributeRest`/`Hide`/`Travel*`/`TakeItem`/`UseItem`/`SponsorGift`/`Suicide`.
3. **PR3 — Survival enrichment + area-event narration.** Enrich `SurvivalResult` with severity/roll/desperation context. Restore `announce_area_events` to emit `AreaEvent`/`AreaClose` per newly-spawned area event. Update placeholder test at `games.rs:1018`.

Each PR ships independently, restores a coherent slice of narration, and is verifiable by running the existing test suite plus new round-trip tests.

---

# PR1 — Combat Narration Restoration

**Branch/bookmark name:** `feat-event-unification-combat-33r`

**Files:**
- Modify: `game/src/tributes/combat.rs` (16 call sites)
- Modify: `game/src/tributes/lifecycle.rs` (status-death site at :277, definition at :286-301)
- Modify: `game/src/tributes/mod.rs` (combat is invoked from `process_turn_phase`; need to drain returned events)
- Modify: `game/src/games.rs` (cycle loop receives drained events; new helper `Game::log_output<D: Display>(source, subject, content)`)
- Modify: `game/src/games.rs` test at `:1004-1019` (placeholder; combat doesn't go through `announce_area_events`, so this stays a no-op for now — leave the comment, just remove `hangrier_games-33r` reference once final PR lands)

## Task 1: Add `Game::log_output` helper that takes a `Display` value

**Files:**
- Modify: `game/src/games.rs:188-198`

- [ ] **Step 1: Read existing `Game::log` signature**

Open `game/src/games.rs:188-198`. Confirm signature is:
```rust
pub fn log(&mut self, source: MessageSource, subject: String, content: String)
```

- [ ] **Step 2: Add the new helper directly below `log`**

Insert after the existing `log` method:

```rust
/// Log a structured game output by rendering its `Display` impl into a `GameMessage`.
///
/// Use this when emitting events from inside the engine: build a `GameOutput<'_>` at
/// the call site and pass it here. Until `hangrier_games-mqi` lands and we have a
/// serializable structured event, this is the bridge between typed events and the
/// stringly-typed `GameMessage.content` field.
pub fn log_output<D: std::fmt::Display>(
    &mut self,
    source: MessageSource,
    subject: String,
    output: D,
) {
    self.log(source, subject, output.to_string());
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --package game`
Expected: clean build, no new warnings.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): add Game::log_output helper for typed event logging"
```

## Task 2: Write failing test for combat event emission

**Files:**
- Test: `game/src/tributes/combat.rs` (add to existing `#[cfg(test)] mod tests` block at the bottom; if none exists, create it)

- [ ] **Step 1: Find or create the combat tests module**

Open `game/src/tributes/combat.rs`. Search for `#[cfg(test)]`. If a `mod tests` block exists, append to it. If not, append a new block at the end of the file.

- [ ] **Step 2: Add the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::GameOutput;
    use crate::tributes::Tribute;

    #[test]
    fn attack_emits_at_least_one_event() {
        let mut attacker = Tribute::new("Attacker".to_string(), Some(1));
        let mut defender = Tribute::new("Defender".to_string(), Some(2));
        attacker.attributes.health = 100;
        defender.attributes.health = 100;

        let events = attacker.attack(&mut defender);

        assert!(
            !events.is_empty(),
            "attack() should return at least one GameOutput describing the outcome"
        );
        // Event should reference one of the two combatants.
        let rendered: Vec<String> = events.iter().map(|e| e.to_string()).collect();
        let any_mentions_combatant = rendered
            .iter()
            .any(|s| s.contains("Attacker") || s.contains("Defender"));
        assert!(
            any_mentions_combatant,
            "Expected combat narration to mention a combatant. Got: {:?}",
            rendered
        );
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test --package game --lib tributes::combat::tests::attack_emits_at_least_one_event`
Expected: compile error — `attack` returns `()` not `Vec<GameOutput>`.

- [ ] **Step 4: Do NOT commit yet** — the failing test stays uncommitted until implementation lands together.

## Task 3: Change `attack` to return `Vec<GameOutput<'_>>`

**Files:**
- Modify: `game/src/tributes/combat.rs` (entire `attack` fn and all 16 `try_log_action` sites)

- [ ] **Step 1: Read `combat.rs` lines 1-50 to confirm the `attack` signature and imports**

Open `game/src/tributes/combat.rs` lines 1-50. Note current imports.

- [ ] **Step 2: Add `GameOutput` import at the top of the file (if not present)**

```rust
use crate::output::GameOutput;
```

- [ ] **Step 3: Change `attack` return type**

Find the `pub fn attack(...)` signature (likely around line 20-30). Change `-> ()` (or implicit unit) to:

```rust
pub fn attack<'a>(&'a mut self, defender: &'a mut Tribute) -> Vec<GameOutput<'a>>
```

(The exact lifetime parameters may need adjustment based on the actual signature; `'a` must outlive both `self` and `defender` for the returned `&str` references in `GameOutput` variants.)

- [ ] **Step 4: Replace each `try_log_action` call with pushing to a local `events` Vec**

At the top of `attack`, add:
```rust
let mut events: Vec<GameOutput<'a>> = Vec::new();
```

Then for each of the 16 call sites (combat.rs:30, 36, 42, 48, 62, 76, 87, 98, 150, 164, 175, 182, 199, 263, 283, 343), replace:

```rust
self.try_log_action(GameOutput::TributeAttackMiss(&self.name, &defender.name), "miss");
```

with:

```rust
events.push(GameOutput::TributeAttackMiss(&self.name, &defender.name));
```

(Drop the second `action_description` argument — it was only used by the gutted `tracing::debug!` call.)

- [ ] **Step 5: Return `events` at every exit path**

Add `events` as the final expression of `attack` and at any early-return points.

- [ ] **Step 6: Run the failing test**

Run: `cargo test --package game --lib tributes::combat::tests::attack_emits_at_least_one_event`
Expected: PASS.

- [ ] **Step 7: Run the full game crate test suite**

Run: `cargo test --package game`
Expected: all tests pass except the pre-existing `test_all_terrains_produce_valid_items` if not yet merged (should be merged from PR #108 — verify with `jj log -r main@origin -l 5`). Specifically watch for any callers of `attack` in other test files that now fail to compile because they expect `()`.

- [ ] **Step 8: Fix any broken callers**

Likely callers: tests in `combat.rs` itself, possibly integration tests. For each caller that does:
```rust
attacker.attack(&mut defender);
```
change to:
```rust
let _ = attacker.attack(&mut defender);
```
(or use the return value if the test should assert on it).

- [ ] **Step 9: Commit**

```bash
jj describe -m "feat(game): combat returns Vec<GameOutput> instead of dropping events

attack() now collects each combat outcome into a Vec<GameOutput<'_>> and returns
it to the caller. Previously these were passed to try_log_action which had been
gutted to a tracing::debug! no-op, losing all 16 combat narration sites."
```

## Task 4: Wire combat events into `process_turn_phase`

**Files:**
- Modify: `game/src/tributes/mod.rs` (around lines 163-340 where `process_turn_phase` is defined)

- [ ] **Step 1: Read `process_turn_phase` to find where `attack` is called**

Open `game/src/tributes/mod.rs`. Search for `.attack(`. Should be inside `process_turn_phase`.

- [ ] **Step 2: Capture the returned events**

Change:
```rust
attacker.attack(target);
```
to:
```rust
let combat_events = attacker.attack(target);
```

- [ ] **Step 3: Make `process_turn_phase` return `Vec<GameOutput<'_>>`**

Change the signature. Add a local `let mut events: Vec<GameOutput<'_>> = Vec::new();` near the top. Push `combat_events` into `events` (use `events.extend(combat_events)`). Return `events` at the end.

For the other 11 `try_log_action` call sites in this file (lines 172, 186, 195, 257, 278, 284, 290, 316, 326, 334, 366), in PR1 leave them as-is — they get migrated in PR2.

- [ ] **Step 4: Compile and fix callers of `process_turn_phase`**

Run: `cargo check --package game`
Expected: errors at every caller of `process_turn_phase` (likely just `Game::run_tribute_cycle` in `games.rs`).

Update the caller to capture the returned `Vec<GameOutput>`.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): process_turn_phase returns combat events to caller"
```

## Task 5: Drain combat events in `run_tribute_cycle`

**Files:**
- Modify: `game/src/games.rs` (around `run_tribute_cycle`, ~line 596-660)

- [ ] **Step 1: Open `run_tribute_cycle` and find the per-tribute loop**

Open `game/src/games.rs` line 596 onward.

- [ ] **Step 2: Capture and log events after `process_turn_phase`**

After the call:
```rust
tribute.process_turn_phase(...);
```

Change to:
```rust
let tribute_events = tribute.process_turn_phase(...);
let actor_subject = format!("tribute:{}", tribute.identifier);
let actor_id = tribute.identifier.clone();
for event in tribute_events {
    self.log_output(
        MessageSource::Tribute(actor_id.clone()),
        actor_subject.clone(),
        event,
    );
}
```

(Borrow checker note: `tribute` is mutably borrowed during `process_turn_phase`, and `self.log_output` takes `&mut self`. Drop the `tribute` borrow before logging by making `process_turn_phase` consume what it needs and return owned data. If lifetime conflicts arise, collect events into owned `String`s via `event.to_string()` before the loop, then push strings via `self.log` directly.)

- [ ] **Step 3: Run the full game test suite**

Run: `cargo test --package game`
Expected: all tests pass; combat narration now reaches `Game.messages`.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): drain combat events into Game.messages per turn"
```

## Task 6: Wire status-death narration

**Files:**
- Modify: `game/src/tributes/lifecycle.rs:277` (status death site) and `:286-301` (try_log_action def)

- [ ] **Step 1: Find the status-death `try_log_action` call**

Open `game/src/tributes/lifecycle.rs:270-284`. Identify the `try_log_action(GameOutput::TributeDiesFromStatus(...), ...)` call.

- [ ] **Step 2: Refactor `process_status` to return `Vec<GameOutput<'_>>`**

Currently `process_status` likely returns `()` and mutates `self`. Change signature to return `Vec<GameOutput<'_>>`. Replace the single `try_log_action` call with `events.push(GameOutput::TributeDiesFromStatus(...))`.

- [ ] **Step 3: Update `process_turn_phase` caller in `tributes/mod.rs`**

`process_turn_phase` calls `process_status`. Capture its returned events and extend the local events vec.

- [ ] **Step 4: Run tests**

Run: `cargo test --package game`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): status-death events surface in Game.messages"
```

## Task 7: Add round-trip integration test

**Files:**
- Create: `game/tests/event_unification_combat_test.rs`

- [ ] **Step 1: Write the test**

```rust
//! Combat narration round-trip: simulate a forced combat exchange and verify
//! the resulting Game.messages contains tribute-sourced combat narration.

use game::games::Game;
use game::messages::MessageSource;

#[test]
fn forced_combat_emits_messages() {
    let mut game = Game::new("Test".to_string(), 12);
    game.start().expect("game should start");

    // Run cycles until at least one combat event happens (cap at 20 day/night
    // pairs to keep the test bounded).
    for _ in 0..40 {
        game.run_day_night_cycle(true);
        if game.messages.iter().any(|m| matches!(m.source, MessageSource::Tribute(_))) {
            break;
        }
        game.run_day_night_cycle(false);
        if game.messages.iter().any(|m| matches!(m.source, MessageSource::Tribute(_))) {
            break;
        }
    }

    let tribute_msgs: Vec<_> = game
        .messages
        .iter()
        .filter(|m| matches!(m.source, MessageSource::Tribute(_)))
        .collect();
    assert!(
        !tribute_msgs.is_empty(),
        "Expected at least one tribute-sourced message after 40 cycles. \
         Got {} total messages, none tribute-sourced.",
        game.messages.len()
    );
}
```

- [ ] **Step 2: Run it**

Run: `cargo test --package game --test event_unification_combat_test`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(game): verify combat narration reaches Game.messages"
```

## Task 8: Quality gates and PR

**Files:** none (verification only)

- [ ] **Step 1: Run all clippy checks**

Run: `cargo clippy --workspace --exclude web --all-targets`
Expected: 0 warnings.

- [ ] **Step 2: Run formatter**

Run: `cargo fmt --check`
Expected: clean (only nightly-only `fn_single_line` warnings are acceptable).

- [ ] **Step 3: Run full game tests**

Run: `cargo test --package game`
Expected: all pass.

- [ ] **Step 4: Check api still builds**

Run: `cargo check --package api`
Expected: clean (no signature changes touched the api boundary).

- [ ] **Step 5: Push bookmark and open PR**

```bash
jj bookmark create feat-event-unification-combat-33r -r @
jj git push --bookmark feat-event-unification-combat-33r
gh pr create --base main --head feat-event-unification-combat-33r \
  --title "feat(game): unify event systems — combat narration (PR1/3, 33r)" \
  --body "$(cat <<'EOF'
## Summary

First of three PRs unifying the event/message systems (issue \`hangrier_games-33r\`). This PR restores **combat** narration to \`Game.messages\` by replacing the gutted \`try_log_action\` no-op with a return-value pattern: combat fns return \`Vec<GameOutput<'_>>\` and the cycle loop drains them into the message buffer.

## Changes

- \`game/src/games.rs\`: add \`Game::log_output<D: Display>\` helper.
- \`game/src/tributes/combat.rs\`: \`attack()\` returns \`Vec<GameOutput<'_>>\`; all 16 combat call sites now collect into the vec instead of being dropped.
- \`game/src/tributes/mod.rs\`: \`process_turn_phase\` returns events; combat events bubble up.
- \`game/src/tributes/lifecycle.rs\`: \`process_status\` returns \`Vec<GameOutput<'_>>\`; status-death narration restored.
- \`game/src/games.rs\`: \`run_tribute_cycle\` drains returned events via \`Game::log_output\` with \`MessageSource::Tribute(actor_id)\`.
- \`game/tests/event_unification_combat_test.rs\`: round-trip test.

## Out of scope (future PRs)

- PR2: movement + turn-phase non-combat narration (rest, hide, travel, items, sponsor gift, suicide).
- PR3: survival_check enrichment + announce_area_events restoration.

## Verification

- \`cargo test --package game\` — all pass
- \`cargo clippy --workspace --exclude web --all-targets\` — 0 warnings
- \`cargo fmt --check\` — clean

## Follow-ups

- \`hangrier_games-33r\` (in progress, 2 PRs remaining)
- \`hangrier_games-mqi\` (P3, replace \`GameOutput\` with serializable \`GameEvent\`)
EOF
)"
```

---

# PR2 — Movement and Turn-Phase Narration Restoration

**Branch/bookmark name:** `feat-event-unification-movement-33r`

**Prereq:** PR1 merged. Sync first: `jj git fetch && jj new main@origin`.

**Files:**
- Modify: `game/src/tributes/movement.rs` (7 call sites: lines 38, 52, 68, 79, 94, 113, 128)
- Modify: `game/src/tributes/mod.rs` (11 remaining call sites: 172, 186, 195, 257, 278, 284, 290, 316, 326, 334, 366)
- Modify: `game/src/games.rs` (no new logic; existing drain in `run_tribute_cycle` already handles the events vec returned by `process_turn_phase`)

## Task 1: Wire movement events

**Files:** `game/src/tributes/movement.rs`

- [ ] **Step 1: Add `GameOutput` import**

```rust
use crate::output::GameOutput;
```

- [ ] **Step 2: Convert each movement fn to return `Vec<GameOutput<'_>>`**

Movement functions are likely `travel`, `hide`, `rest`, `stay`, `follow`. For each fn that currently calls `try_log_action`:

1. Change return type from `()` (or `bool`/whatever it returns) to `(OldReturn, Vec<GameOutput<'_>>)` if it had a meaningful return, else just `Vec<GameOutput<'_>>`.
2. Build a local `events` Vec.
3. Replace each `try_log_action(GameOutput::X(...), "desc")` with `events.push(GameOutput::X(...));`.
4. Return `events` (or `(old, events)`).

- [ ] **Step 3: Update callers in `tributes/mod.rs::process_turn_phase`**

Capture the events vecs from movement calls and `events.extend(...)` into the master vec.

- [ ] **Step 4: Run tests**

Run: `cargo test --package game`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): movement returns Vec<GameOutput> for narration"
```

## Task 2: Wire remaining turn-phase events in `tributes/mod.rs`

**Files:** `game/src/tributes/mod.rs`

- [ ] **Step 1: Identify the 11 remaining `try_log_action` sites**

Run: `rg -n 'try_log_action' game/src/tributes/mod.rs`
Should show 11 lines: 172, 186, 195, 257, 278, 284, 290, 316, 326, 334, 366 (line numbers may shift after PR1 merged).

- [ ] **Step 2: Replace each with `events.push(...)`**

For each site, replace:
```rust
self.try_log_action(GameOutput::Variant(args), "desc");
```
with:
```rust
events.push(GameOutput::Variant(args));
```

(Lifetime gotcha: events that reference `self` fields need `'a` where `'a` outlives `self`. If the `events` Vec is built early in the fn and `self` is mutably re-borrowed later, you may need to push owned `String`s instead. If that happens, use `event.to_string()` at the push site and switch the local Vec type to `Vec<String>`. The drain code in `games.rs` already handles either via `Display`, but `log_output` requires `Display`, so `String` works.)

- [ ] **Step 3: Run tests**

Run: `cargo test --package game`
Expected: all pass; `Game.messages` should now contain rest/hide/travel/item-use narration in addition to combat.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): turn-phase narration (rest, hide, items, sponsor) restored"
```

## Task 3: Now-dead `try_log_action` cleanup

**Files:** `game/src/tributes/lifecycle.rs:286-301`

- [ ] **Step 1: Verify no callers remain**

Run: `rg -n 'try_log_action' game/src/`
Expected: zero matches (PR1 cleared combat + status-death; PR2 cleared movement + turn-phase).

If any matches remain, address them before continuing.

- [ ] **Step 2: Delete the `try_log_action` method**

Open `game/src/tributes/lifecycle.rs:286-301`. Delete the entire fn including its doc comment.

- [ ] **Step 3: Run tests**

Run: `cargo test --package game`
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
jj describe -m "chore(game): remove dead try_log_action method"
```

## Task 4: Add round-trip test for movement

**Files:** Create `game/tests/event_unification_movement_test.rs`

- [ ] **Step 1: Write the test**

```rust
//! Movement narration round-trip.
use game::games::Game;
use game::messages::MessageSource;

#[test]
fn movement_emits_messages() {
    let mut game = Game::new("MoveTest".to_string(), 12);
    game.start().expect("game should start");

    // Run several cycles; movement happens nearly every turn, so 4 cycles
    // should be ample.
    for _ in 0..4 {
        game.run_day_night_cycle(true);
        game.run_day_night_cycle(false);
    }

    let tribute_msgs: Vec<&str> = game
        .messages
        .iter()
        .filter_map(|m| match &m.source {
            MessageSource::Tribute(_) => Some(m.content.as_str()),
            _ => None,
        })
        .collect();

    let move_words = ["travels", "rests", "hides", "Cannot use", "Patron"];
    let any_movement = tribute_msgs
        .iter()
        .any(|c| move_words.iter().any(|w| c.contains(w)));
    assert!(
        any_movement,
        "Expected movement-related narration. Got: {:?}",
        tribute_msgs
    );
}
```

- [ ] **Step 2: Run it**

Run: `cargo test --package game --test event_unification_movement_test`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(game): verify movement narration reaches Game.messages"
```

## Task 5: Quality gates and PR

- [ ] **Step 1: Run quality gates**

Run: `cargo clippy --workspace --exclude web --all-targets && cargo fmt --check && cargo test --package game && cargo check --package api`
Expected: clean.

- [ ] **Step 2: Push and open PR**

```bash
jj bookmark create feat-event-unification-movement-33r -r @
jj git push --bookmark feat-event-unification-movement-33r
gh pr create --base main --head feat-event-unification-movement-33r \
  --title "feat(game): unify event systems — movement and turn-phase (PR2/3, 33r)" \
  --body "$(cat <<'EOF'
## Summary

Second of three PRs for \`hangrier_games-33r\`. Restores movement, rest, hide, item interaction, and sponsor-gift narration to \`Game.messages\`. Removes the now-dead \`try_log_action\` helper.

## Changes

- \`game/src/tributes/movement.rs\`: 7 movement fns return \`Vec<GameOutput<'_>>\`.
- \`game/src/tributes/mod.rs\`: 11 remaining turn-phase narration sites collect events into the same vec returned by \`process_turn_phase\`.
- \`game/src/tributes/lifecycle.rs\`: \`try_log_action\` removed (no callers).
- \`game/tests/event_unification_movement_test.rs\`: round-trip test.

## Verification

- \`cargo clippy --workspace --exclude web --all-targets\` — 0 warnings
- \`cargo test --package game\` — all pass

## Follow-ups

- \`hangrier_games-33r\` (PR3 of 3 next: survival enrichment + area-event narration)
EOF
)"
```

---

# PR3 — Survival Enrichment and Area-Event Narration

**Branch/bookmark name:** `feat-event-unification-survival-33r`

**Prereq:** PR2 merged.

**Files:**
- Modify: `game/src/areas/events.rs:30-37` (`SurvivalResult` struct)
- Modify: `game/src/areas/events.rs:286-385` (`survival_check` impl)
- Modify: `game/src/games.rs:294-438` (`process_event_for_area` consumer)
- Modify: `game/src/games.rs:464-474` (`announce_area_events` — restore narration)
- Modify: `game/src/games.rs:1004-1019` (placeholder test — update assertion to non-zero)
- Modify: `game/src/areas/events.rs` tests (`event_severity_test`)
- Update: `codemap.md` files referencing `GLOBAL_MESSAGES`

## Task 1: Failing test for `SurvivalResult` enrichment

**Files:** `game/src/areas/events.rs` (in-file `#[cfg(test)] mod tests`)

- [ ] **Step 1: Add failing test**

```rust
#[test]
fn survival_result_carries_severity_and_roll() {
    use crate::areas::events::AreaEvent;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    let mut rng = SmallRng::seed_from_u64(42);
    let event = AreaEvent::Wildfire;
    let result = event.survival_check(
        /* tribute_strength */ 5,
        /* tribute_health */ 80,
        /* has_affinity */ false,
        /* has_helpful_item */ false,
        /* is_desperate */ false,
        /* instant_death_enabled */ true,
        /* terrain_severity */ crate::areas::events::EventSeverity::Major,
        /* severity_multiplier */ 1.0,
        /* rng */ &mut rng,
    );

    assert_eq!(result.severity, crate::areas::events::EventSeverity::Major,
        "SurvivalResult should expose the severity used for the DC");
    assert!(result.roll >= 1 && result.roll <= 20,
        "SurvivalResult should expose the d20 roll. Got: {}", result.roll);
}
```

(The exact param order of `survival_check` may differ — read events.rs:286 first and adapt.)

- [ ] **Step 2: Verify it fails to compile**

Run: `cargo test --package game --lib areas::events::tests::survival_result_carries_severity_and_roll`
Expected: FAIL — `severity`, `roll` fields don't exist on `SurvivalResult`.

## Task 2: Enrich `SurvivalResult`

**Files:** `game/src/areas/events.rs:30-37`

- [ ] **Step 1: Add fields**

Change the struct from:

```rust
pub struct SurvivalResult {
    pub survived: bool,
    pub instant_death: bool,
    pub stamina_restored: u32,
    pub sanity_restored: u32,
    pub reward_item: Option<String>,
}
```

to:

```rust
pub struct SurvivalResult {
    pub survived: bool,
    pub instant_death: bool,
    pub stamina_restored: u32,
    pub sanity_restored: u32,
    pub reward_item: Option<String>,
    /// Severity tier used to compute the DC.
    pub severity: EventSeverity,
    /// The raw d20 roll (before modifiers). Useful for narration ("rolls a 1...").
    pub roll: u32,
    /// Sum of modifiers applied (affinity + item + desperation bonuses).
    pub modifier: i32,
    /// True when the desperation reward branch executed (regardless of which
    /// reward was chosen, including the "nothing" 5% slice).
    pub desperation_triggered: bool,
}
```

- [ ] **Step 2: Populate fields in `survival_check`**

In `survival_check` (events.rs:286), at the construction site of the returned `SurvivalResult`, populate the new fields. The `roll` and `modifier` are already local variables; the `severity` is the resolved tier; `desperation_triggered = is_desperate && survived`.

- [ ] **Step 3: Run the failing test**

Run: `cargo test --package game --lib areas::events::tests::survival_result_carries_severity_and_roll`
Expected: PASS.

- [ ] **Step 4: Run full event_severity test file**

Run: `cargo test --package game --test event_severity_test`
Expected: existing tests still pass (new fields are additive).

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): SurvivalResult exposes severity, roll, modifier, desperation flag"
```

## Task 3: Emit narration for non-desperate survival and "nothing" branch

**Files:** `game/src/games.rs:294-438` (`process_event_for_area`)

- [ ] **Step 1: Read current consumer logic**

Open `game/src/games.rs:376-435`. Identify the `if/else if` ladder that decides which message to push.

- [ ] **Step 2: Add an `else` clause for "survived without reward"**

After the existing branches (death, instant_death, stamina, sanity, reward_item), add:

```rust
} else if result.survived {
    // Survived without any reward (non-desperate or 5% nothing branch).
    pending_messages.push(format!(
        "{} weathers the {} (DC {}, rolled {}+{})",
        tribute.name,
        most_severe_event,
        result.severity.dc(),  // Add a helper if not present
        result.roll,
        result.modifier,
    ));
}
```

(Add `dc()` method on `EventSeverity` if it doesn't exist — should map to the same table at events.rs:301-307.)

- [ ] **Step 3: Run tests**

Run: `cargo test --package game`
Expected: all pass; the placeholder test at games.rs:1018 now likely fails because `process_event_for_area` is producing more messages than before.

- [ ] **Step 4: Update the placeholder test**

Open `game/src/games.rs:1004-1019` (`test_announce_area_events`). The comment refers to `hangrier_games-33r`. Update the assertion:

```rust
// announce_area_events emits one AreaEvent message per newly-spawned area event
// (Task 4 below), and process_event_for_area emits one message per affected
// tribute. With no tributes in the test setup, expect 0 from process_event,
// but the AreaEvent narration should now be present.
assert!(
    !game.messages.is_empty(),
    "expected at least the area-event announcement"
);
```

(Adjust based on what the actual test does — read it first to confirm.)

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): emit narration for survived-no-reward area-event outcomes"
```

## Task 4: Restore `announce_area_events`

**Files:** `game/src/games.rs:464-474`

- [ ] **Step 1: Read the current no-op**

Open `game/src/games.rs:464-474`. It currently iterates `area_details.events` with `let _event_name = ...;` bindings.

- [ ] **Step 2: Replace with real emission**

```rust
fn announce_area_events(&mut self) {
    let to_emit: Vec<(String, String)> = self
        .areas
        .iter()
        .flat_map(|(name, details)| {
            details.events.iter().map(move |ev| (name.clone(), ev.to_string()))
        })
        .collect();

    for (area_name, event_name) in to_emit {
        let subject = format!("area:{}", area_name);
        self.log_output(
            MessageSource::Area(area_name.clone()),
            subject,
            crate::output::GameOutput::AreaEvent(&area_name, &event_name),
        );
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --package game`
Expected: pass; the test at games.rs:1004-1019 now sees ≥1 message and passes the new assertion.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): announce_area_events emits AreaEvent narration"
```

## Task 5: Round-trip integration test for survival narration

**Files:** Create `game/tests/event_unification_survival_test.rs`

- [ ] **Step 1: Write the test**

```rust
//! Survival narration round-trip: verify that area events produce both an
//! AreaEvent announcement and per-tribute survival outcomes.

use game::games::Game;
use game::messages::MessageSource;

#[test]
fn area_events_produce_narration() {
    let mut game = Game::new("SurvivalTest".to_string(), 12);
    game.start().expect("game should start");

    // Day 1 announces game start. Day 2-N may trigger area events.
    for _ in 0..6 {
        game.run_day_night_cycle(true);
        game.run_day_night_cycle(false);
    }

    let area_msgs: Vec<&str> = game
        .messages
        .iter()
        .filter_map(|m| match &m.source {
            MessageSource::Area(_) => Some(m.content.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        !area_msgs.is_empty(),
        "Expected at least one area-sourced message after 6 days. \
         Total messages: {}",
        game.messages.len()
    );
}
```

- [ ] **Step 2: Run it**

Run: `cargo test --package game --test event_unification_survival_test`
Expected: PASS (within 6 cycles area events almost certainly fire — if flaky, bump to 12).

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(game): verify area-event narration reaches Game.messages"
```

## Task 6: Update stale codemap docs

**Files:** `codemap.md`, `game/codemap.md`, `game/src/messages/codemap.md` (if exists), `game/src/threats/codemap.md`

- [ ] **Step 1: Find references to GLOBAL_MESSAGES**

Run: `rg -l 'GLOBAL_MESSAGES'`
Expected: matches in 3-4 codemap files.

- [ ] **Step 2: Replace each reference**

In each file, find the section describing `GLOBAL_MESSAGES` and replace with a description of the current architecture:

> **Event flow.** Engine fns return `Vec<GameOutput<'_>>` from each behavior site (combat, movement, turn-phase, status-death, area events, survival outcomes). The cycle loop in `Game::run_day_night_cycle` drains those vectors and calls `Game::log_output` to push each output as a `GameMessage` into `Game.messages` (a per-cycle buffer). The API layer's `save_game` drains the buffer at the end of each cycle, broadcasts via WebSocket, and persists to SurrealDB. See `hangrier_games-mqi` for the planned migration to a serializable `GameEvent` enum.

- [ ] **Step 3: Verify**

Run: `rg 'GLOBAL_MESSAGES'`
Expected: zero matches.

- [ ] **Step 4: Commit**

```bash
jj describe -m "docs: update codemap to describe current event flow (post-33r)"
```

## Task 7: Quality gates and PR

- [ ] **Step 1: Run all quality gates**

Run: `cargo clippy --workspace --exclude web --all-targets && cargo fmt --check && cargo test --package game && cargo check --package api`
Expected: clean.

- [ ] **Step 2: Push and open PR**

```bash
jj bookmark create feat-event-unification-survival-33r -r @
jj git push --bookmark feat-event-unification-survival-33r
gh pr create --base main --head feat-event-unification-survival-33r \
  --title "feat(game): unify event systems — survival and area events (PR3/3, 33r)" \
  --body "$(cat <<'EOF'
## Summary

Final PR for \`hangrier_games-33r\`. Enriches \`SurvivalResult\` with severity/roll/modifier/desperation context, surfaces previously-lost survival outcomes (non-desperate survival, 5%-nothing desperation branch), and restores \`announce_area_events\` to emit \`AreaEvent\` narration.

## Changes

- \`game/src/areas/events.rs\`: \`SurvivalResult\` gains \`severity\`, \`roll\`, \`modifier\`, \`desperation_triggered\` fields.
- \`game/src/games.rs\`: \`process_event_for_area\` emits a survived-no-reward message; \`announce_area_events\` now narrates each newly-spawned area event.
- \`game/src/games.rs:1004-1019\`: placeholder test updated to the now-real behavior.
- \`game/tests/event_unification_survival_test.rs\`: round-trip test.
- \`codemap.md\` (and child files): updated to describe the unified flow.

## Verification

- \`cargo clippy --workspace --exclude web --all-targets\` — 0 warnings
- \`cargo test --package game\` — all pass

## Follow-ups

- Closes \`hangrier_games-33r\`.
- \`hangrier_games-mqi\` (P3, replace \`GameOutput\` with serializable \`GameEvent\`) remains open.
EOF
)"
```

- [ ] **Step 3: After PR3 merges, close the issue**

```bash
bd close hangrier_games-33r --reason "Implemented across PRs (combat, movement, survival)"
```
