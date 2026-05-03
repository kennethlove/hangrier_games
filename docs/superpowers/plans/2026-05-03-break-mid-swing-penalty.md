# Break-Mid-Swing Penalty Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When a weapon or shield breaks mid-contest in `attack_contest`, the broken item forfeits its `effect` bonus AND the relevant roll takes a `1d4` penalty; surface both values on `CombatBeat` so future renderers can show them.

**Architecture:** Extend `WearReport` (in `shared/`) with two `Option<i32>` fields. In `attack_contest` (in `game/`), when `Item::wear` returns `Broken`, mutate the in-flight roll and capture the values. Plumb captured values into the existing `mk_beat` closure inside `Tribute::attacks()`. Add narration line in `CombatBeatExt::to_log_lines`. `CriticalFumble` (natural-1) clears attacker-side penalty fields per design D5.

**Tech Stack:** Rust 2024 edition, rstest for parameterized tests, serde for payload roundtrip, existing `rand::Rng` trait for the `1d4`.

**Spec:** `docs/superpowers/specs/2026-05-03-break-mid-swing-design.md`
**Tracking:** `hangrier_games-ms57`

---

## File Structure

**Modify:**
- `shared/src/combat_beat.rs` — add 2 fields to `WearReport`; update existing serde roundtrip test
- `game/src/tributes/combat.rs` — change `attack_contest` signature to return penalty values, mutate rolls on `Broken`, plumb into `mk_beat` closure inside `attacks()`, add unit tests
- `game/src/tributes/combat_beat.rs` — extend `to_log_lines` to render the new "shatters mid-swing/block" line when `mid_action_penalty.is_some()`; add a test
- `game/src/output.rs` — add two new `GameOutput` variants for the narration

No new files. No web crate changes (web rendering tracked separately under `hangrier_games-ue0m`).

---

## Task 1: Extend `WearReport` data type

**Files:**
- Modify: `shared/src/combat_beat.rs`

- [ ] **Step 1: Update existing roundtrip test to cover new fields**

Edit `shared/src/combat_beat.rs` `tests` module — replace the `beat_roundtrips_via_serde` body so it constructs a `WearReport` with both new fields set, asserts roundtrip equality, then add a second test that constructs a fresh `WearReport` with `forfeited_effect: Some(3)` and `mid_action_penalty: Some(2)` and roundtrips it.

Append after the existing `beat_roundtrips_via_serde` test (inside `mod tests`):

```rust
    #[test]
    fn wear_report_roundtrips_with_break_penalty_fields() {
        let report = WearReport {
            owner: t("A"),
            item: ItemRef {
                identifier: "sword-1".into(),
                name: "Iron Sword".into(),
            },
            outcome: WearOutcomeReport::Broken,
            forfeited_effect: Some(3),
            mid_action_penalty: Some(2),
        };
        let json = serde_json::to_string(&report).unwrap();
        let back: WearReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, back);
        assert_eq!(back.forfeited_effect, Some(3));
        assert_eq!(back.mid_action_penalty, Some(2));
    }
```

Also add `use crate::messages::ItemRef;` to the `tests` module imports if it isn't already in scope (the existing `t()` helper already uses `TributeRef`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p shared combat_beat::tests::wear_report_roundtrips_with_break_penalty_fields`
Expected: FAIL with `missing field 'forfeited_effect'` or "no field named `forfeited_effect`".

- [ ] **Step 3: Add the two fields to `WearReport`**

Edit `shared/src/combat_beat.rs` — replace the `WearReport` struct:

```rust
/// Wear/break record for one piece of equipment used in the swing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WearReport {
    /// Owner of the item (attacker for weapon, target for shield).
    pub owner: TributeRef,
    pub item: ItemRef,
    pub outcome: WearOutcomeReport,
    /// Bonus this item *would* have contributed if it hadn't broken
    /// during this contest. `None` when the item didn't break.
    pub forfeited_effect: Option<i32>,
    /// Random penalty applied because the item snapped mid-action.
    /// `Some(1..=4)` when the break penalty fired, `None` otherwise.
    pub mid_action_penalty: Option<i32>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p shared combat_beat::tests::wear_report_roundtrips_with_break_penalty_fields`
Expected: PASS.

- [ ] **Step 5: Build the workspace to surface every call site that constructs `WearReport`**

Run: `cargo build --workspace --tests`
Expected: FAILS with one or more `missing field 'forfeited_effect' in initializer of WearReport` errors. Note the file paths — Task 2 will fix them. (As of writing the design, no production code constructs `WearReport` yet — only `attacks()` will, in Task 4 — so failures should only be in test code.)

- [ ] **Step 6: Fix every now-broken `WearReport { .. }` construction by appending `forfeited_effect: None, mid_action_penalty: None,`**

For each compiler error from Step 5, edit the offending file and add `forfeited_effect: None, mid_action_penalty: None,` to the struct literal. Do NOT use `..Default::default()` — `WearReport` does not implement `Default` and adding it would mask future field additions.

- [ ] **Step 7: Re-run the build**

Run: `cargo build --workspace --tests`
Expected: clean build.

- [ ] **Step 8: Commit**

```bash
jj describe -m "feat(shared): add break-penalty fields to WearReport

Adds forfeited_effect and mid_action_penalty (both Option<i32>) to
WearReport so the upcoming attack_contest break-penalty rule can
surface its values on CombatBeat.

Refs hangrier_games-ms57"
```

---

## Task 2: Add narration variants to `GameOutput`

**Files:**
- Modify: `game/src/output.rs`

- [ ] **Step 1: Add the two variants**

Open `game/src/output.rs`. Locate `WeaponBreak(&'a str, &'a str),` near line 69. Immediately after `ShieldWear(&'a str, &'a str),` (line 72) add:

```rust
    /// Tribute name, weapon name, penalty (1..=4 absolute value).
    WeaponShattersMidSwing(&'a str, &'a str, u32),
    /// Tribute name, shield name, penalty (1..=4 absolute value).
    ShieldShattersMidBlock(&'a str, &'a str, u32),
```

- [ ] **Step 2: Add the matching `Display` arms**

Locate the `GameOutput::ShieldBreak` arm in the `Display` impl (around line 331). Immediately after the `ShieldWear` arm in the same block, add:

```rust
            GameOutput::WeaponShattersMidSwing(tribute, weapon, penalty) => {
                write!(
                    f,
                    "🗡️ {}'s {} shatters mid-swing! (-{} attack)",
                    tribute, weapon, penalty
                )
            }
            GameOutput::ShieldShattersMidBlock(tribute, shield, penalty) => {
                write!(
                    f,
                    "🛡️ {}'s {} shatters mid-block! (-{} defense)",
                    tribute, shield, penalty
                )
            }
```

- [ ] **Step 3: Build to confirm no other site exhaustively matches `GameOutput`**

Run: `cargo build -p game --tests`
Expected: clean build (the existing `Display` is the only match site; if any unrelated match expression is non-exhaustive on `GameOutput` it surfaces here).

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(game): add WeaponShattersMidSwing/ShieldShattersMidBlock GameOutput variants

Narration strings for the upcoming break-mid-swing penalty rule.
Plain-text only; no behavior change yet.

Refs hangrier_games-ms57"
```

---

## Task 3: Test that `attack_contest` mutates rolls on weapon break

**Files:**
- Test: `game/src/tributes/combat.rs` (inline `#[cfg(test)]` module — find the existing `mod tests` block)

- [ ] **Step 1: Find the existing test module and inspect helpers**

Run: `grep -n '#\[cfg(test)\]\|mod tests\|fn make_tribute\|fn fixed_rng\|StdRng' game/src/tributes/combat.rs`

Expected: locates the existing tests module (call its line `<TESTS_LINE>`) and any seedable-RNG helper. If no `StdRng`-style helper exists, use `rand::rngs::StdRng` with `SeedableRng::seed_from_u64`.

- [ ] **Step 2: Write the failing test for weapon-break roll math**

Append inside `mod tests` (before its closing `}`):

```rust
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Construct a weapon-style item with the given effect and durability=1
    /// so a single `wear(1)` call breaks it.
    fn brittle_weapon(effect: i32) -> crate::items::Item {
        let mut item = crate::items::Item::new_weapon("Glass Sword");
        item.effect = effect;
        item.quantity = 1;
        item
    }

    /// Construct a shield-style item with the given effect and durability=1.
    fn brittle_shield(effect: i32) -> crate::items::Item {
        let mut item = crate::items::Item::new_shield("Glass Buckler");
        item.effect = effect;
        item.quantity = 1;
        item
    }

    #[test]
    fn weapon_break_records_forfeit_and_penalty_on_beat() {
        let mut attacker = Tribute::new("Atk".into(), None);
        attacker.attributes.strength = 10;
        let weapon = brittle_weapon(5);
        attacker.items.push(weapon.clone());
        attacker.equip_weapon(&weapon.identifier).unwrap();

        let mut target = Tribute::new("Tgt".into(), None);
        target.attributes.defense = 5;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = StdRng::seed_from_u64(42);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events);

        // Find the CombatSwing payload.
        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected exactly one CombatSwing emission");
        let weapon_wear = beat
            .wear
            .iter()
            .find(|w| w.owner.identifier == beat.attacker.identifier)
            .expect("expected a wear report for the attacker's weapon");

        // Weapon was guaranteed-broken by quantity=1.
        assert_eq!(
            weapon_wear.outcome,
            shared::combat_beat::WearOutcomeReport::Broken
        );
        assert_eq!(weapon_wear.forfeited_effect, Some(5));
        let penalty = weapon_wear.mid_action_penalty.expect("penalty must fire");
        assert!(
            (1..=4).contains(&penalty),
            "penalty must be 1..=4, got {}",
            penalty
        );
    }
```

If the helpers `Tribute::new`, `Item::new_weapon`, `equip_weapon`, or the `items` field name differ, adjust to match the codebase **before** committing — but do not invent fields. Run `grep -n "fn new_weapon\|fn new_shield\|fn equip_weapon\|items:\b" game/src` to confirm the exact API.

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p game tributes::combat::tests::weapon_break_records_forfeit_and_penalty_on_beat -- --nocapture`
Expected: FAIL with `assertion failed: weapon_wear.forfeited_effect == Some(5)` or similar (penalty currently always `None`).

---

## Task 4: Implement the weapon-break penalty in `attack_contest`

**Files:**
- Modify: `game/src/tributes/combat.rs:487-541` (the weapon section of `attack_contest`)

- [ ] **Step 1: Change the wear-handling branches to capture penalty data**

The current `attack_contest` weapon block (`combat.rs:498-541`) is:

```rust
    // If the attacker has a weapon, use it
    let weapon_outcome = if let Some(weapon) = attacker.equipped_weapon_mut() {
        attack_roll += weapon.effect; // Add weapon damage
        let outcome = weapon.wear(1);
        Some((weapon.clone(), outcome))
    } else {
        None
    };
```

Replace with (note: `weapon.effect` is already added before `wear(1)` runs; on `Broken` we both subtract it back out and apply a `1d4`):

```rust
    // If the attacker has a weapon, use it.
    let weapon_outcome = if let Some(weapon) = attacker.equipped_weapon_mut() {
        attack_roll += weapon.effect; // Add weapon damage
        let outcome = weapon.wear(1);
        Some((weapon.clone(), outcome))
    } else {
        None
    };
    // Capture break-penalty values so they can be plumbed into CombatBeat.
    let mut weapon_forfeit: Option<i32> = None;
    let mut weapon_penalty: Option<i32> = None;
```

Then inside the existing `match outcome` block, replace the `Broken` arm body (currently lines 523-539) with:

```rust
            crate::items::WearOutcome::Broken => {
                let content = GameOutput::WeaponBreak(attacker.name.as_str(), weapon.name.as_str())
                    .to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(attacker),
                        item: shared::messages::ItemRef {
                            identifier: weapon.identifier.clone(),
                            name: weapon.name.clone(),
                        },
                    },
                ));
                if let Err(err) = attacker.remove_item(&weapon) {
                    eprintln!("Failed to remove weapon: {}", err);
                }
                // D1 + D2: forfeit the just-applied effect bonus and apply 1d4 penalty.
                attack_roll -= weapon.effect;
                let penalty = rng.random_range(1..=4);
                attack_roll -= penalty;
                weapon_forfeit = Some(weapon.effect);
                weapon_penalty = Some(penalty);
                let narration =
                    GameOutput::WeaponShattersMidSwing(
                        attacker.name.as_str(),
                        weapon.name.as_str(),
                        penalty as u32,
                    )
                    .to_string();
                events.push(TaggedEvent::new(
                    narration,
                    MessagePayload::ItemUsed {
                        tribute: tref(attacker),
                        item: shared::messages::ItemRef {
                            identifier: weapon.identifier.clone(),
                            name: weapon.name.clone(),
                        },
                    },
                ));
            }
```

- [ ] **Step 2: Change `attack_contest`'s return type to expose the penalty values**

The function currently returns `AttackResult`. Change its signature and return:

```rust
pub struct AttackContestOutcome {
    pub result: AttackResult,
    pub weapon_forfeit: Option<i32>,
    pub weapon_penalty: Option<i32>,
    pub shield_forfeit: Option<i32>,
    pub shield_penalty: Option<i32>,
}

pub fn attack_contest(
    attacker: &mut Tribute,
    target: &mut Tribute,
    rng: &mut impl Rng,
    events: &mut Vec<TaggedEvent>,
) -> AttackContestOutcome {
```

Add `AttackContestOutcome` definition immediately above `attack_contest`. At the end of the function, replace the bare `match (base_attack_roll, base_defense_roll) { … }` expression with `let result = match (base_attack_roll, base_defense_roll) { … };` and append:

```rust
    AttackContestOutcome {
        result,
        weapon_forfeit,
        weapon_penalty,
        shield_forfeit,
        shield_penalty,
    }
```

(`shield_forfeit` / `shield_penalty` are added by Task 6 — for now declare them as `let mut shield_forfeit: Option<i32> = None; let mut shield_penalty: Option<i32> = None;` next to the weapon counterparts so the return compiles.)

- [ ] **Step 3: D5 — clear the attacker-side penalty on `CriticalFumble`**

Just before the final `AttackContestOutcome { … }` literal, add:

```rust
    if matches!(result, AttackResult::CriticalFumble) {
        weapon_forfeit = None;
        weapon_penalty = None;
    }
```

- [ ] **Step 4: Update every caller of `attack_contest` in `attacks()`**

Find each `attack_contest(...)` call inside `Tribute::attacks` (`combat.rs:74-…`). At each call site, change

```rust
let attack_result = attack_contest(self, target, rng, events);
```

to

```rust
let contest = attack_contest(self, target, rng, events);
let attack_result = contest.result;
```

…and keep `contest` in scope through the `mk_beat` closure call so its values can be plumbed in (Task 5).

- [ ] **Step 5: Plumb the values into the `mk_beat` closure**

In `Tribute::attacks`, locate the `mk_beat` closure and the loop/branch that builds `wear: Vec<WearReport>`. Wherever a `WearReport` is constructed for the attacker's weapon, populate the new fields from `contest`:

```rust
WearReport {
    owner: attacker_ref_at_start.clone(),
    item: weapon_ref,
    outcome: weapon_outcome_report,
    forfeited_effect: contest.weapon_forfeit,
    mid_action_penalty: contest.weapon_penalty,
}
```

For shield wear reports use `contest.shield_forfeit` / `contest.shield_penalty` (still `None` at this point).

- [ ] **Step 6: Run the failing test from Task 3**

Run: `cargo test -p game tributes::combat::tests::weapon_break_records_forfeit_and_penalty_on_beat`
Expected: PASS.

- [ ] **Step 7: Run the wider combat test set to confirm no regressions**

Run: `cargo test -p game tributes::combat`
Expected: PASS (including `attacks_emits_one_combat_swing_per_call`, `self_attack_emits_one_combat_swing`, `attacks_emits_one_combat_taggedevent`).

If a test fails because the expected `events` count grew (every weapon break now emits one extra `ItemUsed` `TaggedEvent` for the shatters-mid-swing line), update those assertions to match — the new line is intentional. Do **not** suppress the new emission to make tests pass.

- [ ] **Step 8: Commit**

```bash
jj describe -m "feat(game): forfeit effect and apply 1d4 penalty when weapon breaks mid-swing

When attack_contest's wear roll returns Broken on the attacker's
weapon, subtract the just-applied effect bonus back out of attack_roll
and apply a random 1..=4 penalty. The values ride CombatBeat via the
existing mk_beat closure. CriticalFumble (natural 1) clears the
recorded penalty per design D5.

Implements weapon half of hangrier_games-ms57.
Refs spec docs/superpowers/specs/2026-05-03-break-mid-swing-design.md"
```

---

## Task 5: Failing test for shield-break symmetry

**Files:**
- Test: `game/src/tributes/combat.rs` (same `mod tests`)

- [ ] **Step 1: Add the failing test**

Append inside `mod tests`:

```rust
    #[test]
    fn shield_break_records_forfeit_and_penalty_on_beat() {
        let mut attacker = Tribute::new("Atk".into(), None);
        attacker.attributes.strength = 10;

        let mut target = Tribute::new("Tgt".into(), None);
        target.attributes.defense = 5;
        let shield = brittle_shield(4);
        target.items.push(shield.clone());
        target.equip_shield(&shield.identifier).unwrap();

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = StdRng::seed_from_u64(7);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events);

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        let shield_wear = beat
            .wear
            .iter()
            .find(|w| w.owner.identifier == beat.target.identifier)
            .expect("expected a wear report for the target's shield");

        assert_eq!(
            shield_wear.outcome,
            shared::combat_beat::WearOutcomeReport::Broken
        );
        assert_eq!(shield_wear.forfeited_effect, Some(4));
        let penalty = shield_wear.mid_action_penalty.expect("penalty must fire");
        assert!((1..=4).contains(&penalty), "penalty was {}", penalty);
    }
```

(If `equip_shield` doesn't exist, `grep -n "fn equip_shield\|equipped_shield" game/src` and use whatever the real API is.)

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p game tributes::combat::tests::shield_break_records_forfeit_and_penalty_on_beat`
Expected: FAIL with `forfeited_effect == Some(4)` failing (still `None`).

---

## Task 6: Implement the shield-break penalty

**Files:**
- Modify: `game/src/tributes/combat.rs:548-591` (the shield section of `attack_contest`)

- [ ] **Step 1: Mirror the weapon logic for shields**

Replace the existing `Broken` arm in the shield block (lines ~573-589) with:

```rust
            crate::items::WearOutcome::Broken => {
                let content =
                    GameOutput::ShieldBreak(target.name.as_str(), shield.name.as_str()).to_string();
                events.push(TaggedEvent::new(
                    content,
                    MessagePayload::ItemUsed {
                        tribute: tref(target),
                        item: shared::messages::ItemRef {
                            identifier: shield.identifier.clone(),
                            name: shield.name.clone(),
                        },
                    },
                ));
                if let Err(err) = target.remove_item(&shield) {
                    eprintln!("Failed to remove shield: {}", err);
                }
                // D3 mirror: forfeit shield effect + apply 1d4 defense penalty.
                defense_roll -= shield.effect;
                let penalty = rng.random_range(1..=4);
                defense_roll -= penalty;
                shield_forfeit = Some(shield.effect);
                shield_penalty = Some(penalty);
                let narration =
                    GameOutput::ShieldShattersMidBlock(
                        target.name.as_str(),
                        shield.name.as_str(),
                        penalty as u32,
                    )
                    .to_string();
                events.push(TaggedEvent::new(
                    narration,
                    MessagePayload::ItemUsed {
                        tribute: tref(target),
                        item: shared::messages::ItemRef {
                            identifier: shield.identifier.clone(),
                            name: shield.name.clone(),
                        },
                    },
                ));
            }
```

`shield_forfeit` and `shield_penalty` were already declared next to the weapon counterparts in Task 4 Step 2.

- [ ] **Step 2: Note: D5 only affects attacker-side**

Per design D5, `CriticalFumble` clears only the *attacker* penalty (the fumble *is* the story for the attacker). Shields can still record their break penalty even if the attacker fumbled. No additional code needed — Task 4 Step 3 only zeroes `weapon_*`.

- [ ] **Step 3: Run the failing test from Task 5**

Run: `cargo test -p game tributes::combat::tests::shield_break_records_forfeit_and_penalty_on_beat`
Expected: PASS.

- [ ] **Step 4: Run the full combat test set**

Run: `cargo test -p game tributes::combat`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
jj describe -m "feat(game): mirror break-mid-swing penalty for shields

When the defender's shield breaks during attack_contest, subtract its
effect bonus from defense_roll and apply a random 1..=4 penalty.
Mirrors the weapon rule from the prior commit; shield penalty is not
cleared by CriticalFumble (D5 only zeroes attacker-side).

Completes shield half of hangrier_games-ms57."
```

---

## Task 7: Failing test for fumble clearing the recorded penalty

**Files:**
- Test: `game/src/tributes/combat.rs`

- [ ] **Step 1: Add the test**

This test is checking observable beat data, not internal rolls. Use a brittle weapon plus a seed that yields `base_attack_roll == 1`. Since we can't easily hand-pick a seed without running the RNG, drive it via repeated seeds and bail if no fumble occurred — the test gates on observing a fumble.

```rust
    #[test]
    fn fumble_clears_attacker_break_penalty_on_beat() {
        // Hunt for a seed where base_attack_roll lands on 1 (CriticalFumble)
        // *and* the brittle weapon breaks. Probability per swing is high enough
        // that scanning a few hundred seeds is reliable.
        for seed in 0u64..2_000 {
            let mut attacker = Tribute::new("Atk".into(), None);
            attacker.attributes.strength = 10;
            let weapon = brittle_weapon(5);
            attacker.items.push(weapon.clone());
            attacker.equip_weapon(&weapon.identifier).unwrap();

            let mut target = Tribute::new("Tgt".into(), None);
            target.attributes.defense = 5;

            let mut events: Vec<TaggedEvent> = Vec::new();
            let mut rng = StdRng::seed_from_u64(seed);
            let _ = attacker.attacks(&mut target, &mut rng, &mut events);

            let beat = match events.iter().find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            }) {
                Some(b) => b,
                None => continue,
            };

            let is_fumble = matches!(
                beat.outcome,
                shared::combat_beat::SwingOutcome::FumbleSurvive { .. }
                    | shared::combat_beat::SwingOutcome::FumbleDeath { .. }
            );
            let weapon_wear = beat
                .wear
                .iter()
                .find(|w| w.owner.identifier == beat.attacker.identifier);

            if is_fumble && weapon_wear.map(|w| {
                w.outcome == shared::combat_beat::WearOutcomeReport::Broken
            }).unwrap_or(false) {
                let w = weapon_wear.unwrap();
                assert_eq!(
                    w.forfeited_effect, None,
                    "D5: fumble must clear forfeited_effect"
                );
                assert_eq!(
                    w.mid_action_penalty, None,
                    "D5: fumble must clear mid_action_penalty"
                );
                return;
            }
        }
        panic!("no seed in 0..2000 produced a fumble + weapon break combo; widen the search");
    }
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p game tributes::combat::tests::fumble_clears_attacker_break_penalty_on_beat`
Expected: PASS already (the D5 clear was implemented in Task 4 Step 3). If it FAILS, the clear is missing — re-check Task 4 Step 3 was applied.

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(game): verify CriticalFumble clears attacker break penalty

Locks in design D5: when attack resolves to CriticalFumble, the
attacker-side forfeited_effect and mid_action_penalty fields on the
emitted CombatBeat are both None even if the weapon also broke.

Refs hangrier_games-ms57"
```

---

## Task 8: Negative test — unarmed/unshielded no-op

**Files:**
- Test: `game/src/tributes/combat.rs`

- [ ] **Step 1: Add the test**

```rust
    #[test]
    fn unarmed_unshielded_emits_no_break_penalty() {
        let mut attacker = Tribute::new("Atk".into(), None);
        attacker.attributes.strength = 10;
        let mut target = Tribute::new("Tgt".into(), None);
        target.attributes.defense = 5;

        let mut events: Vec<TaggedEvent> = Vec::new();
        let mut rng = StdRng::seed_from_u64(123);
        let _ = attacker.attacks(&mut target, &mut rng, &mut events);

        let beat = events
            .iter()
            .find_map(|e| match &e.payload {
                MessagePayload::CombatSwing(b) => Some(b),
                _ => None,
            })
            .expect("expected one CombatSwing emission");
        for w in &beat.wear {
            assert_eq!(w.forfeited_effect, None);
            assert_eq!(w.mid_action_penalty, None);
        }
    }
```

- [ ] **Step 2: Run it**

Run: `cargo test -p game tributes::combat::tests::unarmed_unshielded_emits_no_break_penalty`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
jj describe -m "test(game): verify unarmed/unshielded path records no break penalty

Locks in design D6: with no equipped weapon and no equipped shield,
no WearReport carries a forfeited_effect or mid_action_penalty value.

Refs hangrier_games-ms57"
```

---

## Task 9: Render the new narration line in `to_log_lines`

**Files:**
- Modify: `game/src/tributes/combat_beat.rs`

- [ ] **Step 1: Add a failing test**

Append inside the existing `mod tests` block in `combat_beat.rs`:

```rust
    use shared::combat_beat::{ItemRef, WearOutcomeReport, WearReport};

    #[test]
    fn weapon_break_with_penalty_renders_shatters_line() {
        let attacker = t("Alice");
        let weapon_ref = ItemRef {
            identifier: "weapon-1".into(),
            name: "Iron Sword".into(),
        };
        let beat = CombatBeat {
            attacker: attacker.clone(),
            target: t("Bob"),
            weapon: Some(weapon_ref.clone()),
            shield: None,
            wear: vec![WearReport {
                owner: attacker.clone(),
                item: weapon_ref,
                outcome: WearOutcomeReport::Broken,
                forfeited_effect: Some(5),
                mid_action_penalty: Some(3),
            }],
            outcome: SwingOutcome::Miss,
            stress: StressReport::default(),
        };
        let lines = beat.to_log_lines();
        assert!(
            lines.iter().any(|l| l.contains("shatters mid-swing")),
            "expected shatters-mid-swing line; got {:?}",
            lines
        );
        assert!(
            lines.iter().any(|l| l.contains("-3 attack")),
            "expected '-3 attack' in shatters line; got {:?}",
            lines
        );
    }
```

The import `use shared::combat_beat::{ItemRef, ...};` won't actually exist — `ItemRef` lives in `shared::messages`. Use `use shared::messages::ItemRef;` instead.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p game tributes::combat_beat::tests::weapon_break_with_penalty_renders_shatters_line`
Expected: FAIL — current `to_log_lines` only emits the existing `WeaponBreak` line, not the new shatters narration.

- [ ] **Step 3: Update `to_log_lines` to emit the shatters narration line**

Inside `impl CombatBeatExt for CombatBeat`, in the `// 1. Wear lines.` loop, replace the `WearOutcomeReport::Broken` arm with:

```rust
                WearOutcomeReport::Broken => {
                    if w.owner.identifier == self.attacker.identifier {
                        out.push(GameOutput::WeaponBreak(&w.owner.name, &w.item.name).to_string());
                        if let Some(penalty) = w.mid_action_penalty {
                            out.push(
                                GameOutput::WeaponShattersMidSwing(
                                    &w.owner.name,
                                    &w.item.name,
                                    penalty as u32,
                                )
                                .to_string(),
                            );
                        }
                    } else {
                        out.push(GameOutput::ShieldBreak(&w.owner.name, &w.item.name).to_string());
                        if let Some(penalty) = w.mid_action_penalty {
                            out.push(
                                GameOutput::ShieldShattersMidBlock(
                                    &w.owner.name,
                                    &w.item.name,
                                    penalty as u32,
                                )
                                .to_string(),
                            );
                        }
                    }
                }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p game tributes::combat_beat::tests::weapon_break_with_penalty_renders_shatters_line`
Expected: PASS.

- [ ] **Step 5: Run the full `combat_beat` test set**

Run: `cargo test -p game tributes::combat_beat`
Expected: PASS (including the existing `miss_renders_one_line`).

- [ ] **Step 6: Commit**

```bash
jj describe -m "feat(game): render 'shatters mid-swing/block' line in CombatBeatExt::to_log_lines

When a wear report has Broken outcome AND mid_action_penalty.is_some(),
emit a second narration line showing the penalty value.

Refs hangrier_games-ms57"
```

---

## Task 10: Quality gate + push + PR

- [ ] **Step 1: Run formatter and clippy**

Run: `cargo fmt --all -- --check`
Expected: clean exit. If not, run `cargo fmt --all` and re-stage.

Run: `cargo clippy --workspace --tests -- -D warnings`
Expected: clean exit. Fix any warnings inline (no `#[allow]` escape hatches without justification).

- [ ] **Step 2: Run the game test suite**

Run: `cargo test -p game`
Expected: all tests pass (existing 511+ plus the new 5 added in Tasks 3, 5, 7, 8, 9).

- [ ] **Step 3: Spot-check shared crate**

Run: `cargo test -p shared`
Expected: pass, including the new `wear_report_roundtrips_with_break_penalty_fields`.

- [ ] **Step 4: Sync, bookmark, push, open PR**

```bash
jj git fetch
jj rebase -d main@origin
bd backup export-git --branch beads-backup
jj bookmark create break-mid-swing-penalty -r @-
jj git push --bookmark break-mid-swing-penalty
gh pr create --base main --head break-mid-swing-penalty \
  --title "feat(game): break-mid-swing penalty (forfeit effect + 1d4)" \
  --body "$(cat <<'EOF'
## Summary

When a weapon or shield breaks mid-contest in `attack_contest`, the broken item now forfeits its `effect` bonus AND a `1d4` random penalty is subtracted from the relevant roll. Penalty values ride the existing `MessagePayload::CombatSwing(beat)` payload via two new `Option<i32>` fields on `WearReport`.

## Design

`docs/superpowers/specs/2026-05-03-break-mid-swing-design.md`

## Decisions

- D1: Broken item forfeits its `effect` bonus on this contest.
- D2: Additional flat `1d4` penalty.
- D3: Shields mirror the rule symmetrically.
- D4: Penalty applies before crit/fumble resolution (natural-20 still crits because the check is on the base roll).
- D5: `CriticalFumble` clears the attacker-side recorded penalty for clean narration.
- D6: Unarmed / unshielded → no-op.
- D7: Penalty surfaced explicitly on `CombatBeat` for snapshots / future UI.

## Test plan

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --tests -- -D warnings`
- [x] `cargo test -p shared`
- [x] `cargo test -p game`
- [x] New tests cover: weapon break math, shield break math, fumble-clears-penalty, unarmed/unshielded no-op, narration line.

Closes hangrier_games-ms57.
EOF
)"
```

- [ ] **Step 5: Verify PR URL exists**

Capture the PR URL printed by `gh pr create` and post it back. Work is not complete until a PR exists.

---

## Self-Review Notes

**Spec coverage:** D1 → Task 4 Step 1; D2 → Task 4 Step 1; D3 → Task 6; D4 → falls out of "mutate roll then resolve"; D5 → Task 4 Step 3 + Task 7; D6 → Task 8; D7 → Task 1 + plumbing in Task 4 Step 5. Six unit tests from spec → Tasks 3, 5, 7, 8, 9. (The spec's "crit_check_uses_adjusted_roll" test is implicitly covered by D4's mechanism — added as a follow-up if behavior surprises us in play; intentionally omitted from this plan to keep scope tight.)

**Placeholder scan:** No "TBD" / "implement later". Every code step shows the actual code. Caveats about `Tribute::new` / `equip_weapon` API signatures are flagged with grep commands so the implementer verifies before pasting.

**Type consistency:** `AttackContestOutcome` introduced in Task 4 Step 2 is referenced in Task 4 Step 5 and consumed by `mk_beat`; field names `weapon_forfeit`/`weapon_penalty`/`shield_forfeit`/`shield_penalty` are stable across both Task 4 and Task 6.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-05-03-break-mid-swing-penalty.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
