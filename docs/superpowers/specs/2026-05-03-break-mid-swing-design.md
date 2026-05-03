# Break-mid-swing penalty

**Status**: design
**Created**: 2026-05-03
**Author**: combat-tuning brainstorm
**Beads**: hangrier_games-93m (subset — penalty only; stamina + formula tuning split out)

## Problem

When a weapon or shield breaks during `attack_contest` (`game/src/tributes/combat.rs:487`),
its `effect` bonus has already been added to the in-flight `attack_roll` /
`defense_roll`. The break is logged as flavor only — mechanically the broken gear
still "works" for the swing in which it shatters. There is no in-game penalty
for the moment of failure, so equipment durability has weaker tactical weight
than it should.

## Goal

A weapon or shield that breaks mid-contest should make that specific contest
**strictly worse** than swinging/blocking without it, so durability becomes a
real resource the simulator must respect.

Non-goals: stamina system, broader combat formula retuning, weapon-class
balancing. Tracked separately under hangrier_games-93m's other branches.

## Decisions

| ID  | Decision                                                                                                |
| --- | ------------------------------------------------------------------------------------------------------- |
| D1  | On `WearOutcome::Broken`, the just-broken item **forfeits its `effect` bonus** for this contest.        |
| D2  | An additional **flat random penalty of `1d4`** is subtracted from the relevant roll (attack or defense).|
| D3  | Shields mirror the rule symmetrically: lose `effect` bonus on the defense roll + `−1d4` defense.        |
| D4  | Penalty is applied **before** the natural-roll critical/fumble check; a break can downgrade a crit.     |
| D5  | Penalty does **not** stack with the fumble path — `CriticalFumble` (natural 1) bypasses break penalty.  |
| D6  | Unarmed / unshielded → no `effect` bonus to forfeit and no break can fire; rule is a no-op.             |
| D7  | The penalty is surfaced explicitly in `CombatBeat` so timeline/snapshots can render it.                 |

Rationale notes:

- D2 (`1d4` over flat `−2`): combat already rolls dice everywhere; a small random
  tax keeps the "the moment your sword shatters" feeling unpredictable without
  the predictability of a flat constant. Worst case (`−4` plus loss of `effect`)
  is recoverable; best case (`−1`) still bites.
- D4 (penalty before crit check): the whole point of the rule is that a
  mid-swing break hurts. Exempting natural 20s would make breaks effectively
  free on the swings that matter most.
- D5 (no stacking with fumble): natural 1 already triggers `FumbleSurvive` /
  `FumbleDeath`, which carry their own catastrophic consequences. Piling break
  penalty on top tangles two systems and double-punishes the same roll.

## Mechanics

For one call to `attack_contest`:

1. Roll `base_attack_roll` and `base_defense_roll` as today.
2. Add strength to attack roll, defense to defense roll.
3. If attacker has weapon: add `weapon.effect`, then `weapon.wear(1)`.
   - If `WearOutcome::Broken`:
     - Subtract `weapon.effect` back out of `attack_roll` (D1).
     - Roll `swing_penalty = rng.random_range(1..=4)` and subtract from `attack_roll` (D2).
     - Record `swing_penalty` and the forfeited effect on the beat.
4. If defender has shield: add `shield.effect`, then `shield.wear(1)`.
   - If `WearOutcome::Broken`:
     - Subtract `shield.effect` back out of `defense_roll` (D3 ⇒ D1 mirror).
     - Roll `block_penalty = rng.random_range(1..=4)` and subtract from `defense_roll` (D3).
     - Record `block_penalty` and the forfeited effect on the beat.
5. Existing critical/fumble/decisive/normal resolution uses the **adjusted**
   rolls (D4).
6. `CriticalFumble` branch (natural 1) is already detected by `base_attack_roll`
   pre-modifier; the break penalty applied to `attack_roll` is harmless because
   the fumble path doesn't compare rolls. Per D5, when the result is
   `CriticalFumble` the recorded penalty on `CombatBeat` is set back to `None`
   for narration purposes (the fumble outcome is the story, not the snapped
   weapon).

## Data model changes

`shared/src/combat_beat.rs` — extend the `WearReport` struct (not the
`WearOutcomeReport` enum):

```rust
pub struct WearReport {
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

`CombatBeat.wear` is `Vec<WearReport>`; weapon and shield reports each carry
their own pair of optional fields, so attacker and defender breaks are
addressed independently.

No new `MessagePayload` variant is needed — the existing
`MessagePayload::CombatSwing(beat)` carries the data.

## Narration

`CombatBeatExt::to_log_lines` (in `game/src/tributes/combat_beat.rs`) gains a
post-wear line when `mid_action_penalty.is_some()`:

- Weapon: `"<attacker>'s <weapon> shatters mid-swing! (-<n> attack)"`
- Shield: `"<defender>'s <shield> shatters mid-block! (-<n> defense)"`

Existing `GameOutput::WeaponBreak` / `ShieldBreak` flavor lines stay
unchanged — the new line is purely additive and tagged so timeline filters can
hide it if desired.

## Edge cases

| Case                                              | Behavior                                           |
| ------------------------------------------------- | -------------------------------------------------- |
| Unarmed attacker, no break possible               | Rule no-ops (D6).                                  |
| Unshielded defender, no break possible            | Rule no-ops (D6).                                  |
| Weapon breaks AND shield breaks same contest      | Both penalties fire independently.                 |
| Natural 20 attack + weapon breaks                 | Natural 20 still triggers `CriticalHit` branch (it's checked from `base_attack_roll`), but the per-branch damage formula uses the adjusted `attack_roll` if/when it gates on it. Net effect: still a crit, but if any future tuning ties damage to roll-margin, the break is felt. |
| Natural 1 attack + weapon breaks                  | `CriticalFumble` path runs as today; D5 sets `mid_action_penalty = None` on the beat for clean narration. |
| Effect = 0 weapon (rare/special)                  | Forfeit is `0`, `1d4` still rolls — penalty is just the `1d4`. |
| Self-attack path (`attacker_id == target_id`)     | `attack_contest` not invoked; rule does not fire.  |

## Testing

Unit tests in `game/src/tributes/combat.rs` (rstest where seeds matter):

1. `weapon_break_forfeits_effect_and_applies_penalty` — craft a weapon
   guaranteed to break (durability=1), seed RNG, assert
   `attack_roll_after - attack_roll_before == -(effect + penalty)`.
2. `shield_break_forfeits_effect_and_applies_penalty` — mirror for shield.
3. `weapon_break_penalty_recorded_on_beat` — inspect emitted
   `MessagePayload::CombatSwing` and assert
   `wear.weapon.unwrap().mid_action_penalty == Some(_)` and
   `forfeited_effect == Some(weapon.effect)`.
4. `fumble_clears_mid_action_penalty` — natural 1 + breaking weapon →
   `mid_action_penalty == None` on the beat.
5. `unarmed_unshielded_no_op` — confirm no panic, no break, no penalty.
6. `crit_check_uses_adjusted_roll` — high-effect weapon breaks; assert that a
   roll that would crit pre-penalty still crits (since natural 20 is from the
   base roll), and that a non-natural-20 swing whose adjusted roll falls below
   the decisive threshold downgrades correctly.

Property test (proptest, optional): for any `(weapon_effect, durability,
strength, defense)`, the roll difference introduced by a break is always in
`-(effect + 4) ..= -(effect + 1)`.

Snapshot: extend the parity work tracked by `hangrier_games-2qky` so the new
narration line is captured.

## Implementation outline (one PR)

1. Extend `WearOutcomeReport` in `shared/src/combat_beat.rs` with the two new
   `Option<i32>` fields. Default-construct to `None` in all current call sites.
2. In `attack_contest`:
   - When `WearOutcome::Broken` fires for the weapon: subtract `effect`,
     subtract `1d4`, capture both values for the beat.
   - Mirror for shield.
   - After resolution, if final `AttackResult::CriticalFumble`, clear the
     attacker-side penalty fields (set `mid_action_penalty = None`,
     `forfeited_effect = None`).
3. Plumb the captured values into the `mk_beat` closure already in `attacks()`
   so they appear on the emitted `CombatSwing`.
4. Extend `CombatBeatExt::to_log_lines` with the new narration lines.
5. Add the unit tests listed above.
6. Verify legacy `Combat` / `TributeWounded` / `TributeKilled` events are
   unchanged in count and ordering (existing tests cover this).

## Risks

- **Difficulty creep**: weakest tributes already lose most fights; this rule
  marginally widens the gap between equipped and unequipped fighters at the
  *moment of breakage*, but actually *narrows* the long-game gap because
  durable gear becomes a more meaningful investment. Net: probably neutral,
  monitor via simulation telemetry once we have it.
- **Snapshot churn**: any existing insta snapshot that observed the affected
  rolls will need re-baselining. None exist yet (`hangrier_games-2qky`
  pending), so cost is zero today.
- **Save-game compat**: new fields default to `None` and serde'd structs are
  forward-compat; old payloads decode fine.

## Out of scope (deferred)

- Stamina system from hangrier_games-93m.
- Broader retuning of `DECISIVE_WIN_MULTIPLIER`, strength weighting, base
  damage formulas.
- UI/web rendering of the new beat line — tracked under `hangrier_games-ue0m`.
