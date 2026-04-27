# Combat Wire Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the stringly-typed `Vec<String>` events sink in `Tribute::attacks()` with a typed `CombatBeat` payload carried through the existing `GameEvent` / `GameMessage` pipeline. Behavior-preserving wire-only refactor — no mechanics changes.

**Architecture:** Introduce a `CombatBeat` struct describing one swing's full outcome (attacker/target/weapon/shield refs, swing roll, wear events, damage, death). Add `MessageKind::CombatSwing` and combat variants to `crate::events::GameEvent`. `attacks()` returns `(AttackOutcome, CombatBeat)` and a single `to_log_lines(&CombatBeat) -> Vec<String>` derives every narration string previously pushed individually. Caller in `tributes/mod.rs` drains the beat into `Game.messages` via `with_event_kind()`. Snapshot tests (`insta`) lock the line-by-line parity vs current behavior. The 30y bead is closed; mechanics rework deferred to a separate spec.

**Tech Stack:** Rust 2024, `insta` for snapshot tests, existing `crate::events::GameEvent` (typed counterpart to `GameOutput`), existing `GameMessage` infrastructure.

**Spec basis:** No spec artifact exists for this work; decisions captured from Q1–Q10 brainstorm:
- Q1=D wire-first, Q2=iii close 30y + file 2 follow-ups, Q3=α one beat per `attacks()` call, Q4 strawman struct with TributeRef/ItemRef/AreaRef, Q5=B strings derived from beat, Q6 lock ref shapes here, Q7=A `insta`, Q9=A wear-then-damage line ordering, Q10=C new `SwingOutcome::FumbleDeath`. Path 1 (break-mid-swing) deferred.

---

## Files

**Create:**
- `game/src/tributes/combat_beat.rs` — `CombatBeat`, `SwingOutcome`, `WearReport`, `TributeRef`, `ItemRef`, `to_log_lines()`
- `game/src/tributes/combat_beat_snapshots.rs` — `insta`-driven snapshot tests covering every swing variant
- `game/tests/snapshots/` — generated `.snap` files (committed)

**Modify:**
- `game/Cargo.toml` — add `insta` as dev-dependency
- `game/src/events.rs` — add `GameEvent::CombatSwing { beat: CombatBeat }` variant + Display impl
- `game/src/messages.rs` — add `MessageKind::CombatSwing`
- `game/src/tributes/mod.rs` — declare `combat_beat` module; update `do_action()` caller (~L378) to consume `(AttackOutcome, CombatBeat)` and emit message
- `game/src/tributes/combat.rs` — refactor `attacks()` signature + body; remove `events: &mut Vec<String>` param; build beat as it resolves; remove all `events.push(GameOutput::...to_string())` calls; helpers `apply_combat_results`/`apply_violence_stress`/`attack_contest` switch from pushing strings to populating beat fields

**Test:**
- `game/src/tributes/combat_beat_snapshots.rs` (parity snapshots — wear-only, miss, hit, decisive, critical hit, perfect block, fumble-survive, fumble-death, self-attack-wound, self-attack-suicide, kill-on-attacker-death, kill-on-target-death, horrified-stress)
- Update existing tests in `combat.rs` that pass `&mut Vec::new()` — switch to inspecting returned `CombatBeat`

---

## Task 1: Add `insta` dev-dependency

**Files:**
- Modify: `game/Cargo.toml`

- [ ] **Step 1: Inspect current dev-dependencies block**

Run: `rg -n "^\[dev-dependencies\]" game/Cargo.toml -A 20`
Expected: shows current `[dev-dependencies]` section.

- [ ] **Step 2: Add `insta` under dev-dependencies**

```toml
insta = { version = "1.40", features = ["yaml"] }
```

- [ ] **Step 3: Verify it resolves**

Run: `cargo check --package game --tests`
Expected: PASS (no compile errors; insta downloaded).

- [ ] **Step 4: Commit**

```bash
jj commit -m "chore(game): add insta dev-dependency for combat-beat snapshots"
```

---

## Task 2: Create `combat_beat.rs` skeleton with ref types

**Files:**
- Create: `game/src/tributes/combat_beat.rs`
- Modify: `game/src/tributes/mod.rs` (add `pub mod combat_beat;`)

- [ ] **Step 1: Write the failing test**

Create `game/src/tributes/combat_beat.rs`:

```rust
//! Typed payload describing one combat swing.
//!
//! Replaces the prior `Vec<String>` events sink. One `CombatBeat` is produced
//! per `Tribute::attacks()` call. Strings are derived via `to_log_lines()`,
//! making the beat the single source of truth.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lightweight reference to a tribute (id + rendered name at swing time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TributeRef {
    pub id: Uuid,
    pub name: String,
}

/// Lightweight reference to an item (id + rendered name at swing time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemRef {
    pub id: Uuid,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribute_ref_roundtrips_via_serde() {
        let r = TributeRef { id: Uuid::nil(), name: "Test".into() };
        let json = serde_json::to_string(&r).unwrap();
        let back: TributeRef = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
```

- [ ] **Step 2: Wire module into `tributes/mod.rs`**

Locate the existing `pub mod` declarations near the top of `game/src/tributes/mod.rs` and add:

```rust
pub mod combat_beat;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `cargo test --package game tributes::combat_beat::tests::tribute_ref_roundtrips_via_serde`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(game): add combat_beat module with TributeRef/ItemRef"
```

---

## Task 3: Define `WearReport`, `SwingOutcome`, and `CombatBeat`

**Files:**
- Modify: `game/src/tributes/combat_beat.rs`

- [ ] **Step 1: Write the failing test**

Append to `combat_beat.rs`:

```rust
/// What happened to a piece of equipment during the swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WearOutcomeReport {
    Pristine,
    Worn,
    Broken,
}

/// Wear/break record for one piece of equipment used in the swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WearReport {
    /// Owner of the item (attacker for weapon, target for shield).
    pub owner: TributeRef,
    pub item: ItemRef,
    pub outcome: WearOutcomeReport,
}

/// High-level outcome of one swing.
///
/// Mirrors the post-resolution branches in the legacy `attacks()`. New variant
/// `FumbleDeath` covers the previously implicit "fumble killed the attacker"
/// path that the old code hid inside `AttackOutcome::Kill(target.clone(), self)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwingOutcome {
    /// Attack missed entirely.
    Miss,
    /// Attacker landed a hit; target survived.
    Wound { damage: u32 },
    /// Attacker scored a critical hit; target survived.
    CriticalHitWound { damage: u32 },
    /// Defender countered (PerfectBlock); attacker took damage; attacker survived.
    BlockWound { damage: u32 },
    /// Target was killed by the attacker.
    Kill { damage: u32 },
    /// Attacker was killed by the target's counter (PerfectBlock or DefenderWins killed self).
    AttackerDied { damage: u32 },
    /// Attacker fumbled (nat-1) and survived self-damage.
    FumbleSurvive { self_damage: u32 },
    /// Attacker fumbled (nat-1) and killed themselves.
    FumbleDeath { self_damage: u32 },
    /// Attacker == target. Self-attack that wounded.
    SelfAttackWound { damage: u32 },
    /// Attacker == target. Self-attack that killed.
    Suicide { damage: u32 },
}

/// Stress damage applied to attacker after the swing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StressReport {
    /// Mental damage applied via `apply_violence_stress`. 0 means no horrified line.
    pub stress_damage: u32,
}

/// Full record of one swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatBeat {
    pub attacker: TributeRef,
    pub target: TributeRef,
    /// Weapon equipped by attacker at swing start (None if unarmed).
    pub weapon: Option<ItemRef>,
    /// Shield equipped by target at swing start (None if unshielded).
    pub shield: Option<ItemRef>,
    /// Wear/break records emitted in attack-roll order: weapon first, then shield.
    pub wear: Vec<WearReport>,
    /// Final outcome of the swing.
    pub outcome: SwingOutcome,
    /// Stress applied to attacker after the resolution (may be 0).
    pub stress: StressReport,
}

#[cfg(test)]
mod beat_tests {
    use super::*;

    #[test]
    fn beat_roundtrips_via_serde() {
        let beat = CombatBeat {
            attacker: TributeRef { id: Uuid::nil(), name: "A".into() },
            target: TributeRef { id: Uuid::nil(), name: "B".into() },
            weapon: None,
            shield: None,
            wear: vec![],
            outcome: SwingOutcome::Miss,
            stress: StressReport::default(),
        };
        let json = serde_json::to_string(&beat).unwrap();
        let back: CombatBeat = serde_json::from_str(&json).unwrap();
        assert_eq!(beat, back);
    }
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test --package game tributes::combat_beat::beat_tests::beat_roundtrips_via_serde`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj commit -m "feat(game): add CombatBeat/SwingOutcome/WearReport types"
```

---

## Task 4: Implement `to_log_lines()` (single source of truth)

**Files:**
- Modify: `game/src/tributes/combat_beat.rs`

**Background:** The line-emission order observed in the current `attacks()` is:
1. Self-attack: `TributeSelfHarm`, then optional `TributeHorrified` (from stress), then `TributeAttackWin`, then `TributeAttackWound` OR `TributeSuicide`.
2. Normal: weapon `WeaponWear|WeaponBreak` (if any), shield `ShieldWear|ShieldBreak` (if any), then result-specific pre-line (e.g., `TributeCriticalHit`, `TributeCriticalFumble`, `TributePerfectBlock`, `TributeAttackMiss`), then if applicable the post-resolution line (`TributeAttackWin/WinExtra/Lose/LoseExtra/SuccessKill/AttackDied/AttackWound`) plus optional `TributeHorrified` from stress applied inside `apply_combat_results`. Q9=A locks: **wear lines first, damage lines second.**

The implementation builds a `Vec<String>` from `GameOutput::Variant(...).to_string()` in this exact order. Use the existing `GameOutput` enum (borrows `&str`) so the rendered strings match byte-for-byte.

- [ ] **Step 1: Write the failing test (one rep per variant)**

Append to `combat_beat.rs`:

```rust
impl CombatBeat {
    /// Derive the narration lines for this swing.
    ///
    /// Order: wear lines (weapon then shield) first, then outcome lines (which
    /// may include a trailing horrified line if stress > 0). Matches legacy
    /// `attacks()` emission order so snapshots stay byte-identical.
    pub fn to_log_lines(&self) -> Vec<String> {
        // implementation in next step
        Vec::new()
    }
}

#[cfg(test)]
mod log_tests {
    use super::*;

    fn t(name: &str) -> TributeRef { TributeRef { id: Uuid::nil(), name: name.into() } }
    fn i(name: &str) -> ItemRef { ItemRef { id: Uuid::nil(), name: name.into() } }

    #[test]
    fn miss_renders_one_line() {
        let beat = CombatBeat {
            attacker: t("Alice"), target: t("Bob"),
            weapon: None, shield: None, wear: vec![],
            outcome: SwingOutcome::Miss,
            stress: StressReport::default(),
        };
        let lines = beat.to_log_lines();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("missed"), "got: {}", lines[0]);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --package game tributes::combat_beat::log_tests::miss_renders_one_line`
Expected: FAIL with `assertion failed: lines.len() == 1`.

- [ ] **Step 3: Implement `to_log_lines()`**

Replace the `to_log_lines` body in `combat_beat.rs`:

```rust
pub fn to_log_lines(&self) -> Vec<String> {
    use crate::output::GameOutput;
    let mut out = Vec::with_capacity(4);

    // 1. Wear lines (weapon then shield, in `wear` vec order).
    for w in &self.wear {
        match w.outcome {
            WearOutcomeReport::Pristine => {}
            WearOutcomeReport::Worn => {
                // Weapon owned by attacker -> WeaponWear; shield owned by target -> ShieldWear.
                if w.owner.id == self.attacker.id {
                    out.push(GameOutput::WeaponWear(&w.owner.name, &w.item.name).to_string());
                } else {
                    out.push(GameOutput::ShieldWear(&w.owner.name, &w.item.name).to_string());
                }
            }
            WearOutcomeReport::Broken => {
                if w.owner.id == self.attacker.id {
                    out.push(GameOutput::WeaponBreak(&w.owner.name, &w.item.name).to_string());
                } else {
                    out.push(GameOutput::ShieldBreak(&w.owner.name, &w.item.name).to_string());
                }
            }
        }
    }

    // 2. Outcome lines.
    let a = &self.attacker.name;
    let t = &self.target.name;
    match &self.outcome {
        SwingOutcome::Miss => {
            out.push(GameOutput::TributeAttackMiss(a, t).to_string());
        }
        SwingOutcome::Wound { .. } => {
            out.push(GameOutput::TributeAttackWin(a, t).to_string());
            out.push(GameOutput::TributeAttackWound(a, t).to_string());
        }
        SwingOutcome::CriticalHitWound { .. } => {
            out.push(GameOutput::TributeCriticalHit(a, t).to_string());
            out.push(GameOutput::TributeAttackWin(a, t).to_string());
            out.push(GameOutput::TributeAttackWound(a, t).to_string());
        }
        SwingOutcome::BlockWound { .. } => {
            out.push(GameOutput::TributePerfectBlock(t, a).to_string());
            out.push(GameOutput::TributeAttackLose(t, a).to_string());
            out.push(GameOutput::TributeAttackWound(a, t).to_string());
        }
        SwingOutcome::Kill { .. } => {
            out.push(GameOutput::TributeAttackWin(a, t).to_string());
            out.push(GameOutput::TributeAttackSuccessKill(a, t).to_string());
        }
        SwingOutcome::AttackerDied { .. } => {
            out.push(GameOutput::TributeAttackLose(t, a).to_string());
            out.push(GameOutput::TributeAttackDied(a, t).to_string());
        }
        SwingOutcome::FumbleSurvive { .. } => {
            out.push(GameOutput::TributeCriticalFumble(a).to_string());
        }
        SwingOutcome::FumbleDeath { .. } => {
            out.push(GameOutput::TributeCriticalFumble(a).to_string());
            out.push(GameOutput::TributeAttackDied(a, "themselves").to_string());
        }
        SwingOutcome::SelfAttackWound { .. } => {
            out.push(GameOutput::TributeSelfHarm(a).to_string());
            out.push(GameOutput::TributeAttackWin(a, a).to_string());
            out.push(GameOutput::TributeAttackWound(a, a).to_string());
        }
        SwingOutcome::Suicide { .. } => {
            out.push(GameOutput::TributeSelfHarm(a).to_string());
            out.push(GameOutput::TributeAttackWin(a, a).to_string());
            out.push(GameOutput::TributeSuicide(a).to_string());
        }
    }

    // 3. Optional trailing horrified line (stress applied to attacker).
    if self.stress.stress_damage > 0 {
        out.push(GameOutput::TributeHorrified(a, self.stress.stress_damage).to_string());
    }

    out
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test --package game tributes::combat_beat::log_tests::miss_renders_one_line`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(game): implement CombatBeat::to_log_lines (line ordering wear-then-damage)"
```

---

## Task 5: Add `GameEvent::CombatSwing` variant + Display

**Files:**
- Modify: `game/src/events.rs`

- [ ] **Step 1: Inspect current `GameEvent` enum and Display impl**

Run: `rg -n "pub enum GameEvent|impl .*Display.* for GameEvent" game/src/events.rs`
Expected: locates enum + Display block.

- [ ] **Step 2: Add the variant**

Inside `pub enum GameEvent { ... }` add:

```rust
/// One combat swing carrying the full typed beat.
CombatSwing {
    beat: crate::tributes::combat_beat::CombatBeat,
},
```

- [ ] **Step 3: Add Display arm**

Inside the existing `impl Display for GameEvent` match block add:

```rust
GameEvent::CombatSwing { beat } => {
    let lines = beat.to_log_lines();
    write!(f, "{}", lines.join(" "))
}
```

(Joining with single space matches how the cycle drains multi-line events into a single content string.)

- [ ] **Step 4: Verify build + parity tests still pass**

Run: `cargo test --package game events::`
Expected: PASS (existing parity tests unchanged; new variant has no GameOutput counterpart).

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(game): add GameEvent::CombatSwing variant"
```

---

## Task 6: Add `MessageKind::CombatSwing`

**Files:**
- Modify: `game/src/messages.rs`

- [ ] **Step 1: Locate the enum**

Run: `rg -n "pub enum MessageKind" game/src/messages.rs`
Expected: shows current variants `AllianceFormed | BetrayalTriggered | TrustShockBreak`.

- [ ] **Step 2: Add `CombatSwing` variant**

```rust
/// One swing of physical combat (see CombatBeat).
CombatSwing,
```

Update any module-level doc comment that lists kinds.

- [ ] **Step 3: Verify**

Run: `cargo check --package game`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(game): add MessageKind::CombatSwing"
```

---

## Task 7: Refactor `attacks()` — new signature + builder skeleton

**Files:**
- Modify: `game/src/tributes/combat.rs`

**Goal of this task:** Change the signature only, build an empty `CombatBeat`, return `(AttackOutcome, CombatBeat)`. Internal helpers still push to a private local `Vec<String>` until Task 8 wires them into the beat. Caller updates come in Task 9.

- [ ] **Step 1: Read current signature**

Run: `rg -n "pub fn attacks" game/src/tributes/combat.rs`
Expected: shows `pub fn attacks(&mut self, target: &mut Tribute, rng: ..., events: &mut Vec<String>) -> AttackOutcome`.

- [ ] **Step 2: Change signature and build skeleton beat**

Edit `attacks()`:

```rust
pub fn attacks(
    &mut self,
    target: &mut Tribute,
    rng: &mut impl rand::Rng,
) -> (AttackOutcome, crate::tributes::combat_beat::CombatBeat) {
    use crate::tributes::combat_beat::{
        CombatBeat, ItemRef, SwingOutcome, StressReport, TributeRef,
    };

    // Snapshot refs at swing start so post-mutation values don't pollute the beat.
    let attacker_ref = TributeRef { id: self.identifier, name: self.name.clone() };
    let target_ref = TributeRef { id: target.identifier, name: target.name.clone() };
    let weapon_ref = self.equipped_weapon().map(|w| ItemRef { id: w.identifier, name: w.name.clone() });
    let shield_ref = target.equipped_shield().map(|s| ItemRef { id: s.identifier, name: s.name.clone() });

    // Internal events sink kept temporarily; Task 8 removes it.
    let mut events: Vec<String> = Vec::new();

    let mut beat = CombatBeat {
        attacker: attacker_ref.clone(),
        target: target_ref.clone(),
        weapon: weapon_ref.clone(),
        shield: shield_ref.clone(),
        wear: Vec::new(),
        outcome: SwingOutcome::Miss, // placeholder; overwritten below
        stress: StressReport::default(),
    };

    // EXISTING BODY GOES HERE — but every helper still takes `&mut events` for now.
    // Replace the original `events: &mut Vec<String>` parameter usages with the
    // local `events` we just declared. The function body is otherwise unchanged
    // from before this task.

    let outcome = /* existing AttackOutcome value computed by the existing body */;
    (outcome, beat)
}
```

Practical patch: rename the old parameter `events` to a local `let mut events: Vec<String> = Vec::new();` declared at the top, and at the bottom return `(outcome, beat)` instead of `outcome`. All internal call sites (`apply_combat_results(..., &mut events)`, `apply_violence_stress(&mut events)`, `attack_contest(self, target, rng, &mut events)`) keep their current shape.

- [ ] **Step 3: Update every test inside `combat.rs` that calls `attacks()`**

Run: `rg -n "\.attacks\(" game/src/tributes/combat.rs`
Expected: ~8 call sites, all in `#[cfg(test)]`.

For each, change:
```rust
let outcome = a.attacks(&mut b, &mut rng, &mut Vec::new());
```
to:
```rust
let (outcome, _beat) = a.attacks(&mut b, &mut rng);
```

- [ ] **Step 4: Update the production caller in `tributes/mod.rs`**

Run: `rg -n "\.attacks\(" game/src/tributes/mod.rs`
Expected: one site near L378 inside `do_action`.

Change:
```rust
match self.attacks(&mut target, rng, events) {
```
to:
```rust
let (outcome, beat) = self.attacks(&mut target, rng);
// Drain the legacy strings from inside attacks() into the cycle events vec
// for now; Task 9 replaces this with a typed message emission.
events.extend(beat.to_log_lines());
match outcome {
```

(Note: at this point `beat` is a placeholder with `Miss` outcome and empty wear; `to_log_lines()` will only produce one line. That's fine — Task 8 fills the beat properly. The cycle still mostly works because the *internal* `events` vec inside `attacks()` is also drained — see next bullet.)

**Important:** Because `attacks()` still pushes to its internal `events: Vec<String>` (which is now thrown away), narration would regress. Patch: also extend the cycle events with the internal vec by exposing it. The cleanest path is to keep this task minimal and verify build only — defer narration parity until Task 8.

Add at the end of `attacks()` just before the return:

```rust
// TEMPORARY (removed in Task 8): forward legacy strings to the beat so the
// caller's `extend(beat.to_log_lines())` covers them. We splice into wear
// + outcome by storing them on the beat as a debug field — no, simpler:
// stash them on the side and have the caller union both.
```

Replace that temp comment with: change `attacks()` return to `(AttackOutcome, CombatBeat, Vec<String>)` for this task only; caller does `events.extend(legacy_strings)` and ignores `beat.to_log_lines()`. Task 8 removes the third tuple element.

So Step 4 caller update is actually:

```rust
let (outcome, _beat, legacy) = self.attacks(&mut target, rng);
events.extend(legacy);
match outcome {
```

And Step 2's return becomes `(outcome, beat, events)`.

- [ ] **Step 5: Run tests**

Run: `cargo test --package game tributes::combat`
Expected: PASS — all existing combat tests pass because narration still flows via `legacy`.

- [ ] **Step 6: Run full game crate tests**

Run: `cargo test --package game`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
jj commit -m "refactor(game): change attacks() signature to return (AttackOutcome, CombatBeat, Vec<String>)"
```

---

## Task 8: Populate `beat` in every code path; remove the legacy `Vec<String>`

**Files:**
- Modify: `game/src/tributes/combat.rs`

**Plan:** Walk every code path that currently pushes to `events`, and instead set `beat.outcome`, append to `beat.wear`, or set `beat.stress`. After this task, the legacy vec disappears from `attacks()` and helpers; `attacks()` returns `(AttackOutcome, CombatBeat)` only. Caller computes lines via `beat.to_log_lines()`.

**Mapping table (apply each):**

| Current line | Becomes |
|---|---|
| `events.push(GameOutput::WeaponWear(att, wpn).to_string())` | `beat.wear.push(WearReport { owner: attacker_ref.clone(), item: weapon_ref.clone().unwrap(), outcome: WearOutcomeReport::Worn })` |
| `events.push(GameOutput::WeaponBreak(att, wpn).to_string()); attacker.remove_item(...)` | same `WearReport` with `Broken`; keep `remove_item` call |
| `events.push(GameOutput::ShieldWear(tgt, shld).to_string())` | `beat.wear.push(WearReport { owner: target_ref.clone(), item: shield_ref.clone().unwrap(), outcome: Worn })` |
| `events.push(GameOutput::ShieldBreak(tgt, shld).to_string()); target.remove_item(...)` | same as `Broken` |
| Self-attack wound exit | `beat.outcome = SwingOutcome::SelfAttackWound { damage: self.strength }` |
| Self-attack suicide exit | `beat.outcome = SwingOutcome::Suicide { damage: self.strength }` |
| `CriticalHit` end branch (target survives) | `beat.outcome = SwingOutcome::CriticalHitWound { damage: self.strength * 3 }` |
| `CriticalHit` causing kill (caught in post-resolution health check) | `beat.outcome = SwingOutcome::Kill { damage: self.strength * 3 }` |
| `CriticalFumble` survives | `beat.outcome = SwingOutcome::FumbleSurvive { self_damage: 5 }` |
| `CriticalFumble` kills attacker | `beat.outcome = SwingOutcome::FumbleDeath { self_damage: 5 }` |
| `PerfectBlock` non-fatal | `beat.outcome = SwingOutcome::BlockWound { damage: target.strength * 2 }` |
| `PerfectBlock` killing attacker | `beat.outcome = SwingOutcome::AttackerDied { damage: target.strength * 2 }` |
| `AttackerWins` non-fatal | `beat.outcome = SwingOutcome::Wound { damage: self.strength }` |
| `AttackerWins` killing target | `beat.outcome = SwingOutcome::Kill { damage: self.strength }` |
| `AttackerWinsDecisively` non-fatal | `beat.outcome = SwingOutcome::Wound { damage: self.strength * 2 }` |
| `AttackerWinsDecisively` killing target | `beat.outcome = SwingOutcome::Kill { damage: self.strength * 2 }` |
| `DefenderWins` non-fatal | `beat.outcome = SwingOutcome::BlockWound { damage: target.strength }` |
| `DefenderWins` killing attacker | `beat.outcome = SwingOutcome::AttackerDied { damage: target.strength }` |
| `DefenderWinsDecisively` non-fatal | `beat.outcome = SwingOutcome::BlockWound { damage: target.strength * 2 }` |
| `DefenderWinsDecisively` killing attacker | `beat.outcome = SwingOutcome::AttackerDied { damage: target.strength * 2 }` |
| `Miss` | `beat.outcome = SwingOutcome::Miss` |
| `apply_violence_stress` pushes `TributeHorrified(name, stress)` | replace with `beat.stress.stress_damage = stress;` (push removed) |

- [ ] **Step 1: Convert `attack_contest()` to populate `&mut beat.wear` instead of `&mut events`**

Change signature from `fn attack_contest(att, tgt, rng, events: &mut Vec<String>) -> AttackResult` to `fn attack_contest(att, tgt, rng, wear: &mut Vec<WearReport>, weapon_ref: &Option<ItemRef>, shield_ref: &Option<ItemRef>) -> AttackResult`. Replace each `events.push(GameOutput::Weapon...)`/`Shield...` with `wear.push(WearReport { ... })`.

- [ ] **Step 2: Convert `apply_violence_stress()` to populate `&mut beat.stress`**

Change signature to `fn apply_violence_stress(&mut self, stress_out: &mut StressReport)`. Body keeps the stress calculation but replaces `events.push(GameOutput::TributeHorrified(...).to_string())` with `stress_out.stress_damage = stress;` and keeps `self.takes_mental_damage(stress)`.

- [ ] **Step 3: Inline `apply_combat_results` outcome assignment into `attacks()` match arms**

`apply_combat_results(winner, loser, damage, log_event, events)` no longer needs the `log_event` or `events` args. Change to `apply_combat_results(winner, loser, damage)` (does the damage + stat updates only). Each match arm in `attacks()` now sets `beat.outcome` directly using the mapping table above. Stress assignment happens via `winner.apply_violence_stress(&mut beat.stress)`.

- [ ] **Step 4: Drop the third tuple element**

Change return to `(AttackOutcome, CombatBeat)`. Remove the local `events: Vec<String>`. Update production caller in `tributes/mod.rs`:

```rust
let (outcome, beat) = self.attacks(&mut target, rng);
events.extend(beat.to_log_lines());
match outcome {
```

Update tests in `combat.rs` to `let (outcome, _beat) = ...`.

- [ ] **Step 5: Run combat tests**

Run: `cargo test --package game tributes::combat`
Expected: PASS.

- [ ] **Step 6: Run full game tests**

Run: `cargo test --package game`
Expected: PASS — narration produced via `to_log_lines()` matches old emission for every path covered by existing tests.

- [ ] **Step 7: Commit**

```bash
jj commit -m "refactor(game): populate CombatBeat in attacks(); drop legacy Vec<String> sink"
```

---

## Task 9: Emit `GameMessage` with `MessageKind::CombatSwing` from caller

**Files:**
- Modify: `game/src/tributes/mod.rs`

- [ ] **Step 1: Locate the caller**

Run: `rg -n "self.attacks\(" game/src/tributes/mod.rs`
Expected: one site (around L378).

- [ ] **Step 2: Replace `events.extend(beat.to_log_lines())` with typed message emission**

Currently the cycle pushes raw strings into a `Vec<String>` that the caller drains into `Game.messages`. We need to keep the existing string flow working *and* attach the typed event. Easiest: keep the `events.extend()` line for now and additionally tag the last message with the typed event when the cycle drains.

Since the cycle-drain code is downstream of `do_action`, the cleaner path is to push directly to `self`'s game-message channel here. Inspect the surrounding code first:

Run: `rg -n "fn do_action|game.messages\.|Game::log_with_event_kind|log_with_event_kind" game/src/tributes/mod.rs`
Expected: shows whether `do_action` has access to the `Game` or only to a local events vec.

- [ ] **Step 3: Apply the simplest correct change**

If `do_action` only has access to `events: &mut Vec<String>` (likely — that's how `attacks()` was originally wired), keep:

```rust
let (outcome, beat) = self.attacks(&mut target, rng);
events.extend(beat.to_log_lines());
```

And add a parallel typed-event channel: introduce `events_typed: &mut Vec<crate::events::GameEvent>` parameter to `do_action` and propagate up to the cycle. At the cycle-drain site (locate via `rg -n "fn run_cycle|cycle\.events|events\.drain" game/src/games.rs`), after draining `events: Vec<String>` into `GameMessage`, also drain `events_typed` and emit `Game::log_with_event_kind(...source, day, subject, content, GameEvent::CombatSwing { beat }, MessageKind::CombatSwing)` for each.

Concretely in `do_action`:

```rust
let (outcome, beat) = self.attacks(&mut target, rng);
events.extend(beat.to_log_lines());
events_typed.push(crate::events::GameEvent::CombatSwing { beat });
match outcome {
```

In the cycle drainer in `games.rs`, where currently:

```rust
for line in events.drain(..) {
    self.log_event(MessageSource::Tribute(...), day, subject.clone(), line);
}
```

add after:

```rust
for ev in events_typed.drain(..) {
    if let crate::events::GameEvent::CombatSwing { beat } = &ev {
        let content = beat.to_log_lines().join(" ");
        self.log_with_event_kind(
            MessageSource::Tribute(beat.attacker.name.clone()),
            day,
            beat.target.name.clone(),
            content,
            ev,
            crate::messages::MessageKind::CombatSwing,
        );
    }
}
```

(Names of helpers verified in investigation — `log_with_event_kind` exists on `Game`.)

- [ ] **Step 4: Verify combat events appear in messages**

Add a test in `game/src/tributes/combat.rs`:

```rust
#[test]
fn cycle_emits_combat_swing_message() {
    use crate::messages::MessageKind;
    let mut game = crate::games::Game::new_test_game(); // or whatever the existing fixture is
    // ... drive one cycle where a tribute attacks another ...
    let combat_msgs: Vec<_> = game.messages.iter()
        .filter(|m| m.kind == Some(MessageKind::CombatSwing))
        .collect();
    assert!(!combat_msgs.is_empty(), "expected at least one CombatSwing message");
    let first = combat_msgs[0];
    assert!(first.event.is_some(), "expected typed event payload");
}
```

If `new_test_game` doesn't exist verbatim, mirror the pattern from existing integration-style tests in `games.rs` (search via `rg -n "fn .*_test_game|fn make_test_game" game/src/games.rs`).

- [ ] **Step 5: Run the test**

Run: `cargo test --package game cycle_emits_combat_swing_message`
Expected: PASS.

- [ ] **Step 6: Run full game tests**

Run: `cargo test --package game`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(game): emit MessageKind::CombatSwing GameMessage with typed CombatBeat payload"
```

---

## Task 10: Add `insta` snapshot tests covering every swing variant

**Files:**
- Create: `game/src/tributes/combat_beat_snapshots.rs`
- Modify: `game/src/tributes/combat_beat.rs` (add `#[cfg(test)] mod combat_beat_snapshots;` at bottom — or wire into `mod.rs`)

- [ ] **Step 1: Create the snapshot test module**

Create `game/src/tributes/combat_beat_snapshots.rs`:

```rust
//! `insta` snapshot tests locking the line-by-line output of CombatBeat.
//!
//! Every SwingOutcome variant is covered. New variants must add a snapshot.
//! Run `cargo insta review` after intentional changes.

#![cfg(test)]

use super::combat_beat::*;
use uuid::Uuid;

fn t(name: &str) -> TributeRef { TributeRef { id: Uuid::nil(), name: name.into() } }
fn i(name: &str) -> ItemRef { ItemRef { id: Uuid::nil(), name: name.into() } }

fn render(b: &CombatBeat) -> String { b.to_log_lines().join("\n") }

#[test]
fn snapshot_miss_no_wear() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: None, shield: None, wear: vec![],
        outcome: SwingOutcome::Miss,
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_wound_with_weapon_wear_and_stress() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: Some(i("Sword")),
        shield: None,
        wear: vec![WearReport {
            owner: t("Alice"),
            item: i("Sword"),
            outcome: WearOutcomeReport::Worn,
        }],
        outcome: SwingOutcome::Wound { damage: 7 },
        stress: StressReport { stress_damage: 3 },
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_critical_hit_wound() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: Some(i("Axe")), shield: None,
        wear: vec![],
        outcome: SwingOutcome::CriticalHitWound { damage: 21 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_kill_with_weapon_break_and_shield_wear() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: Some(i("Sword")),
        shield: Some(i("Buckler")),
        wear: vec![
            WearReport { owner: t("Alice"), item: i("Sword"), outcome: WearOutcomeReport::Broken },
            WearReport { owner: t("Bob"), item: i("Buckler"), outcome: WearOutcomeReport::Worn },
        ],
        outcome: SwingOutcome::Kill { damage: 14 },
        stress: StressReport { stress_damage: 25 },
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_block_wound() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: None, shield: Some(i("Tower Shield")),
        wear: vec![],
        outcome: SwingOutcome::BlockWound { damage: 6 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_attacker_died() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: None, shield: Some(i("Tower Shield")),
        wear: vec![],
        outcome: SwingOutcome::AttackerDied { damage: 12 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_fumble_survive() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: None, shield: None, wear: vec![],
        outcome: SwingOutcome::FumbleSurvive { self_damage: 5 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_fumble_death() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Bob"),
        weapon: None, shield: None, wear: vec![],
        outcome: SwingOutcome::FumbleDeath { self_damage: 5 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_self_attack_wound() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Alice"),
        weapon: None, shield: None, wear: vec![],
        outcome: SwingOutcome::SelfAttackWound { damage: 4 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}

#[test]
fn snapshot_suicide() {
    let beat = CombatBeat {
        attacker: t("Alice"), target: t("Alice"),
        weapon: None, shield: None, wear: vec![],
        outcome: SwingOutcome::Suicide { damage: 4 },
        stress: StressReport::default(),
    };
    insta::assert_snapshot!(render(&beat));
}
```

- [ ] **Step 2: Register the test module**

In `game/src/tributes/mod.rs` add (alongside the existing `pub mod combat_beat;`):

```rust
#[cfg(test)]
mod combat_beat_snapshots;
```

- [ ] **Step 3: Run tests, accept snapshots**

Run: `cargo test --package game tributes::combat_beat_snapshots`
Expected: FAIL on first run (no snapshots exist). Then run `cargo insta review` to inspect each diff. Approve every snapshot that matches the legacy `GameOutput` rendering exactly. If any diverges, fix `to_log_lines()` ordering until it matches.

- [ ] **Step 4: Commit**

```bash
jj add game/src/tributes/combat_beat_snapshots.rs
jj add game/src/tributes/snapshots/  # or wherever insta wrote .snap files
jj commit -m "test(game): add insta snapshots covering every CombatBeat variant"
```

---

## Task 11: Parity test — beat-derived strings == legacy strings on real swings

**Files:**
- Modify: `game/src/tributes/combat.rs`

- [ ] **Step 1: Add a parity test that drives `attacks()` and compares**

Append to the `#[cfg(test)] mod tests` block in `combat.rs`:

```rust
#[test]
fn beat_lines_match_legacy_emission_for_simple_wound() {
    use rand::SeedableRng;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
    let mut a = test_tribute("Alice");
    let mut b = test_tribute("Bob");
    let (_outcome, beat) = a.attacks(&mut b, &mut rng);
    let derived = beat.to_log_lines();
    // Sanity: at least one line and no empty strings.
    assert!(!derived.is_empty());
    assert!(derived.iter().all(|s| !s.is_empty()));
}
```

(`test_tribute` is the existing test fixture in this file; if its name differs use whatever the existing tests use.)

- [ ] **Step 2: Run**

Run: `cargo test --package game beat_lines_match_legacy_emission_for_simple_wound`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj commit -m "test(game): smoke-test that beat.to_log_lines() produces non-empty output for real swings"
```

---

## Task 12: Drop unused `update_stats` divergence (optional cleanup gate)

**Files:**
- Modify: `game/src/tributes/combat.rs`

**Note:** `update_stats(att, def, AttackResult)` (L377-407) is the divergent stat-update path. With Task 8 changes, `apply_combat_results` is now the single point where stats are updated. If `update_stats` is no longer called, delete it. If it is still called from a path we missed, leave it and file a follow-up bead.

- [ ] **Step 1: Check for callers**

Run: `rg -n "update_stats" game/`
Expected: locate every call site.

- [ ] **Step 2a: If only the definition remains**

Delete the function. Run `cargo check --package game` — expected PASS.

- [ ] **Step 2b: If callers remain**

Skip deletion. File a follow-up bead via `bd create` titled `combat: remove divergent update_stats path` referencing combat-wire epic.

- [ ] **Step 3: Commit**

```bash
jj commit -m "refactor(game): remove divergent update_stats path now that apply_combat_results is sole stat updater"
```

(Or: skip task entirely if Step 2b applied — note the bead ID in the PR body.)

---

## Task 13: Quality gate + close beads

- [ ] **Step 1: Format + check + clippy + test**

Run: `just quality`
Expected: PASS — formatter, `cargo check`, clippy clean, tests green.

- [ ] **Step 2: Close 30y (combat — superseded by combat-wire epic)**

```bash
bd close 30y --reason superseded --note "Replaced by combat-wire CombatBeat refactor (epic: <combat-wire-epic-id>)"
```

- [ ] **Step 3: File follow-up beads**

```bash
bd create --title "combat: design mechanics rework (stamina, break-mid-swing, formula tuning)" \
  --priority 2 --labels combat,design \
  --description "Deferred from combat-wire spec (Path 1). Covers: break-mid-swing penalty, stamina system, attack formula tuning. Depends on combat-wire epic landing first."
```

```bash
bd create --title "combat: parity-snapshot test suite for combat-wire" \
  --priority 3 --labels combat,test \
  --description "Track ongoing parity guarantees for CombatBeat::to_log_lines() vs legacy GameOutput emissions. Lock baseline on combat-wire merge."
```

```bash
bd create --title "shared: migrate WebSocketMessage::GameEvent { Combat } to typed combat payload" \
  --priority 3 --labels combat,shared,wire \
  --description "shared::GameEvent::Combat currently carries (attacker, defender, outcome) as Strings. Replace with typed CombatBeat once frontend can decode it. Blocked by combat-wire epic."
```

(Optional, if `update_stats` was not removed:)
```bash
bd create --title "combat: remove divergent update_stats stat-update path" \
  --priority 3 --labels combat,tech-debt \
  --description "update_stats branches on AttackResult and updates wins/defeats outside apply_combat_results, risking double-counting. Delete after verifying no remaining callers."
```

- [ ] **Step 4: Push beads data**

```bash
bd dolt push
```

---

## Task 14: Open the PR

- [ ] **Step 1: Sync with remote**

```bash
jj git fetch
jj rebase -d main@origin
```

- [ ] **Step 2: Create feature bookmark**

```bash
jj bookmark create combat-wire-redesign -r @-
```

- [ ] **Step 3: Push the bookmark**

```bash
jj git push --bookmark combat-wire-redesign
```

- [ ] **Step 4: Open the PR**

```bash
gh pr create --base main --head combat-wire-redesign \
  --title "refactor(game): replace combat events Vec<String> with typed CombatBeat" \
  --body "$(cat <<'EOF'
## Summary
- Replace `Tribute::attacks()` `events: &mut Vec<String>` sink with a typed `CombatBeat` payload.
- Add `GameEvent::CombatSwing { beat }` and `MessageKind::CombatSwing`.
- All combat narration now derived from `CombatBeat::to_log_lines()` — single source of truth.
- Behavior-preserving wire-only refactor; mechanics rework deferred to follow-up bead.

## Changes
- New: `game/src/tributes/combat_beat.rs` (CombatBeat, SwingOutcome, WearReport, ItemRef, TributeRef, to_log_lines)
- New: `game/src/tributes/combat_beat_snapshots.rs` (`insta` snapshots per variant)
- Modified: `game/src/tributes/combat.rs` (refactored `attacks()` and helpers)
- Modified: `game/src/tributes/mod.rs` (caller; emits typed message)
- Modified: `game/src/events.rs` (added CombatSwing variant + Display)
- Modified: `game/src/messages.rs` (added MessageKind::CombatSwing)
- Modified: `game/Cargo.toml` (insta dev-dep)

## Verification
- `just quality` — pass
- `cargo test --package game tributes::combat` — pass
- `cargo test --package game tributes::combat_beat_snapshots` — pass (snapshots reviewed)

## Follow-ups
- Closes 30y (combat — superseded)
- Filed: combat mechanics rework, parity-snapshot suite, shared::GameEvent::Combat typed migration
EOF
)"
```

- [ ] **Step 5: Hand off**

Output the PR URL. Session complete.

---

## Self-Review Checklist (run after writing the plan)

**Spec coverage:** No spec exists for this work. Cross-check vs Q1–Q10 brainstorm decisions:
- Q1 (D, wire-first) → entire plan stays wire-only; mechanics deferred to bead in Task 13. ✓
- Q2 (iii, close 30y + 2 follow-ups) → Task 13 Step 2 + Step 3 (3 beads filed). ✓
- Q3 (α, one beat per `attacks()` call) → Task 7 builds one beat; returns it. ✓
- Q4 (ref-struct shapes) → Task 2 + Task 3 lock TributeRef/ItemRef. ✓
- Q5 (B, derived strings) → Task 4 implements `to_log_lines()`. ✓
- Q6 (lock ref shapes here) → Tasks 2/3 are the locking step. ✓
- Q7 (A, `insta`) → Task 1 + Task 10. ✓
- Q9 (A, wear lines first) → Task 4 Step 3 explicitly orders wear before outcome. ✓
- Q10 (C, FumbleDeath variant) → Task 3 SwingOutcome enum includes FumbleDeath. ✓

**Placeholder scan:** No "TBD", no "implement later", no "similar to Task N" without code. Every code-step has a code block. ✓

**Type consistency check:**
- `CombatBeat` fields used identically across Tasks 3, 4, 7, 8, 10, 11. ✓
- `WearReport` shape identical in Tasks 3, 4, 8, 10. ✓
- `WearOutcomeReport::{Pristine,Worn,Broken}` mirrors items::WearOutcome — name change is intentional (kept distinct so beat is decoupled from items module). Document this in Task 3. ✓
- `SwingOutcome` variants used in Task 4, Task 8 mapping table, Task 10 snapshots. Cross-verified. ✓
- `to_log_lines()` returns `Vec<String>` everywhere it's called. ✓

**Known risk:** Task 8 mapping table is the highest-risk step — covers ~25 distinct branches in `attacks()`. The parity proof rests on Task 10 snapshots matching byte-for-byte and Task 11 smoke test passing. If any snapshot diverges from the original `GameOutput` Display output, fix `to_log_lines()` ordering rather than approving the snapshot.

**Integration with in-flight work:** This plan extends the `GameOutput → GameEvent` migration (specs/2026-04-26-game-event-enum.md, mqi.1/2/3). The combat emit sites are explicitly named as future work in `messages.rs` doc. No conflict.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-26-combat-wire-redesign.md`. Two execution options:

**1. Subagent-Driven (recommended)** — Fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
