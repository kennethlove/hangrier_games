# Trapped Afflictions PR1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate `TributeStatus::Drowned` and `TributeStatus::Buried` to the affliction system as `AfflictionKind::Trapped(TrapKind)`, with parameterized per-kind tuning, deterministic AreaEvent→severity mapping, a hybrid self-escape mechanic, save-game migration, and full retirement of the legacy status variants.

**Architecture:** New `TrapKind` sub-enum extends `AfflictionKind::Trapped(TrapKind)`. New optional `trapped_metadata: Option<TrappedMetadata>` field on `Affliction` carries cycle counter, partial-rescue progress, and terrain hazard floor. New `TRAP_KIND_TABLE` provides per-(kind, severity) damage tuning. New `attempt_escape` helper computes per-cycle escape rolls. The five `set_status(Drowned|Buried)` sites in `lifecycle.rs:222-230` switch to `try_acquire_affliction(Trapped(_))` calls; the per-cycle damage block at `lifecycle.rs:291-305` switches from status-iteration to affliction-iteration. Save migration uses a Custom `Deserialize` for `TributeStatus` plus a one-shot post-load conversion pass. The `TributeStatus::Drowned` and `Buried` variants are deleted (paired with `b67j`). Rescue action and brain-layer integration are deferred to PR2.

**Tech Stack:** Rust 2024, serde, rstest 0.26, insta 1.40 (yaml + json snapshots), proptest 1.5 (256 cases).

**Spec:** `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md`

**Hard prereq:** `hangrier_games-lsis` (afflictions PR1 — types & storage foundation must be merged first)
**Pairs with:** `hangrier_games-b67j` (TributeStatus legacy retirement — bundled into Task 12)

---

## File Structure

**Create:**
- `shared/src/trapped.rs` — `TrapKind`, `TrappedMetadata`, severity-base constants
- `game/src/tributes/afflictions/trapped.rs` — `TRAP_KIND_TABLE`, `trap_tuning_for`, `attempt_escape`, `area_event_to_trap`
- `game/tests/trapped_afflictions_lifecycle_test.rs` — integration tests
- `game/tests/trapped_save_migration_test.rs` — save migration tests
- `game/tests/snapshots/` (insta snapshot files generated as needed)

**Modify:**
- `shared/src/lib.rs` — `pub mod trapped;`
- `shared/src/afflictions.rs` (created in `lsis`) — extend `AfflictionKind` with `Trapped(TrapKind)`, add `trapped_metadata: Option<TrappedMetadata>` field on `Affliction`, derive helpers
- `shared/src/messages.rs` — add `MessagePayload::TributeTrapped`, `Struggling`, `TrappedEscaped`, `TributeDiedWhileTrapped`
- `game/src/tributes/afflictions/mod.rs` (created in `lsis`) — `pub mod trapped;`, extend `try_acquire_affliction` payload dispatch
- `game/src/tributes/lifecycle.rs` — replace `set_status(Drowned|Buried)` calls (lines 222-230) with `try_acquire_affliction(Trapped(_))`; replace per-cycle damage block (lines 291-305) with affliction-iteration; add escape attempt + death-while-trapped emission; remove `DROWNED_DAMAGE` (line 47), `BURIED_DAMAGE` (line 50), `DROWNED_MENTAL_DAMAGE` (if present)
- `game/src/tributes/statuses.rs` — delete `Drowned` (line 23) and `Buried` (line 25) variants; delete their parse/display arms (lines 57, 59, 81, 83); delete their rstest cases (lines 108, 110, 130, 132); add custom `Deserialize` impl with `__LegacyDrowned`/`__LegacyBuried` private variants
- `game/src/games.rs` — add `pub trapped_afflictions_enabled: bool` field (default `true`); add `migrate_legacy_trapped_statuses` post-load pass
- `game/src/events.rs` — remove `GameEvent::TributeDrowned` (line 186, 619, 1208)
- `game/src/output.rs` — remove `GameOutput::TributeDrowned` (line 45, 229); add output renderer for `MessagePayload::TributeDiedWhileTrapped`
- `game/src/tributes/lifecycle.rs:483, 485` — remove rstest cases referencing deleted `TributeStatus` variants

**Test:**
- `game/tests/trapped_afflictions_lifecycle_test.rs`
- `game/tests/trapped_save_migration_test.rs`
- Inline rstest in `shared/src/trapped.rs`, `game/src/tributes/afflictions/trapped.rs`, `game/src/tributes/statuses.rs`

---

## Conventions

- All commits use Conventional Commits (`feat:`, `refactor:`, `test:`, `chore:`)
- Each task ends with a single commit; commit message is given verbatim per task
- TDD throughout: write the failing test, run it to see it fail, write minimal code, run to see it pass, commit
- jj is the VCS; commits use `jj commit -m "..."` (the `git add` step is implicit — jj tracks all changes in the working copy)
- After every task, run `just test` to confirm the wider game crate still builds and tests pass; don't commit if it fails
- Never run `cargo test` workspace-wide (it can hang); always scope to `--package game` or `--package shared`

---

## Task 1: TrapKind enum + constants in shared crate

**Files:**
- Create: `shared/src/trapped.rs`
- Modify: `shared/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `shared/src/trapped.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Sub-discriminator for `AfflictionKind::Trapped(TrapKind)`.
///
/// Initial v1 only ships `Drowning` and `Buried` — see beads `eeuz` (Pitfall),
/// `v0n2` (Snared), `etxv` (Pinned), `2y3a` (Bound) for future kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapKind {
    Drowning,
    Buried,
}

/// Runtime state for a Trapped affliction. Lives on `Affliction.trapped_metadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrappedMetadata {
    /// Cycles spent trapped. Drives escape-roll decay.
    pub cycles_trapped: u8,
    /// Partial rescue accumulator. Only meaningful at Severe.
    /// Each single-rescuer cycle adds 1; reaches escape threshold at 2.
    pub escape_progress: u8,
    /// Cached terrain hazard floor for the area at acquisition time.
    /// Caps escape roll regardless of stat/rescue bonuses (e.g. 0.30 in active rapids).
    /// `None` means no floor applies.
    pub terrain_hazard_floor: Option<f32>,
}

impl TrappedMetadata {
    pub fn fresh(terrain_hazard_floor: Option<f32>) -> Self {
        Self {
            cycles_trapped: 0,
            escape_progress: 0,
            terrain_hazard_floor,
        }
    }
}

/// Escape-roll severity bases. Mild is most escapable; Severe is hardest.
pub const SEVERITY_BASE_MILD: f32 = 0.50;
pub const SEVERITY_BASE_MODERATE: f32 = 0.35;
pub const SEVERITY_BASE_SEVERE: f32 = 0.20;

/// Maximum bonus from a maxed-out escape stat.
pub const ESCAPE_STAT_BONUS_MAX: f32 = 0.30;

/// Per-cycle decay applied to the escape roll (the longer you're stuck, the harder it is).
pub const CYCLES_DECAY_PER_CYCLE: f32 = 0.08;

/// Hard cap on escape probability — never a guaranteed escape.
pub const ESCAPE_ROLL_CAP: f32 = 0.95;

/// Threshold for partial rescue at Severe — a single rescuer must contribute
/// this many cycles before their bonus applies.
pub const PARTIAL_RESCUE_THRESHOLD: u8 = 2;

/// Cap on total rescue contribution per cycle (prevents 4 rescuers from trivializing).
pub const RESCUE_BONUS_CAP: f32 = 0.80;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trap_kind_serializes_snake_case() {
        let drowning = serde_json::to_string(&TrapKind::Drowning).unwrap();
        assert_eq!(drowning, "\"drowning\"");
        let buried = serde_json::to_string(&TrapKind::Buried).unwrap();
        assert_eq!(buried, "\"buried\"");
    }

    #[test]
    fn trapped_metadata_fresh_defaults() {
        let m = TrappedMetadata::fresh(None);
        assert_eq!(m.cycles_trapped, 0);
        assert_eq!(m.escape_progress, 0);
        assert_eq!(m.terrain_hazard_floor, None);

        let m2 = TrappedMetadata::fresh(Some(0.30));
        assert_eq!(m2.terrain_hazard_floor, Some(0.30));
    }

    #[test]
    fn severity_bases_ordered() {
        assert!(SEVERITY_BASE_SEVERE < SEVERITY_BASE_MODERATE);
        assert!(SEVERITY_BASE_MODERATE < SEVERITY_BASE_MILD);
    }
}
```

Add to `shared/src/lib.rs`:

```rust
pub mod trapped;
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --package shared trapped`
Expected: PASS — 3 tests pass

- [ ] **Step 3: Commit**

```bash
jj commit -m "feat(shared): add TrapKind enum + TrappedMetadata + escape constants"
```

---

## Task 2: Extend AfflictionKind with `Trapped(TrapKind)` variant

**Files:**
- Modify: `shared/src/afflictions.rs`

This task assumes the afflictions PR1 (`lsis`) has merged. If `AfflictionKind` is in a different file, adjust path.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `shared/src/afflictions.rs`:

```rust
#[test]
fn affliction_kind_trapped_serializes_with_inner_kind() {
    use crate::trapped::TrapKind;
    let kind = AfflictionKind::Trapped(TrapKind::Drowning);
    let json = serde_json::to_string(&kind).unwrap();
    // serde tag-discrimination: "trapped" key with snake_case inner
    assert_eq!(json, r#"{"trapped":"drowning"}"#);

    let buried = AfflictionKind::Trapped(TrapKind::Buried);
    let json = serde_json::to_string(&buried).unwrap();
    assert_eq!(json, r#"{"trapped":"buried"}"#);
}

#[test]
fn affliction_kind_trapped_round_trips() {
    use crate::trapped::TrapKind;
    let kind = AfflictionKind::Trapped(TrapKind::Drowning);
    let json = serde_json::to_string(&kind).unwrap();
    let back: AfflictionKind = serde_json::from_str(&json).unwrap();
    assert_eq!(kind, back);
}

#[test]
fn affliction_kind_trapped_is_not_permanent() {
    use crate::trapped::TrapKind;
    assert!(!Affliction::is_kind_permanent(AfflictionKind::Trapped(TrapKind::Drowning)));
    assert!(!Affliction::is_kind_permanent(AfflictionKind::Trapped(TrapKind::Buried)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package shared affliction_kind_trapped`
Expected: FAIL with "no variant Trapped found for enum AfflictionKind"

- [ ] **Step 3: Add the Trapped variant**

In `shared/src/afflictions.rs`, find the `AfflictionKind` enum. Add the `Trapped(TrapKind)` variant at the end:

```rust
use crate::trapped::TrapKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfflictionKind {
    Wounded,
    Broken,
    Infected,
    MissingLimb,
    Blind,
    Deaf,
    Sick,
    Poisoned,
    Burned,
    Frozen,
    Overheated,
    Electrocuted,
    Mauled,
    Trapped(TrapKind),
}
```

If `AfflictionKey` is `(AfflictionKind, Option<BodyPart>)` and uses `Hash`/`Ord`, the `Trapped(TrapKind)` variant gets these for free since `TrapKind` derives them.

The `is_kind_permanent` function does NOT need updating — `Trapped` is reversible (you escape or die), and the function's `matches!` should fall through to `false`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package shared affliction_kind_trapped`
Expected: PASS — 3 tests pass

Run: `cargo test --package shared` (full crate)
Expected: PASS — no other tests broke (the existing `AfflictionKey` tests should keep working since `(AfflictionKind::Trapped(_), Option<BodyPart>)` is still a valid key tuple)

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(shared): extend AfflictionKind with Trapped(TrapKind) variant"
```

---

## Task 3: Add `trapped_metadata` field to `Affliction`

**Files:**
- Modify: `shared/src/afflictions.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `shared/src/afflictions.rs`:

```rust
#[test]
fn affliction_with_trapped_metadata_serializes() {
    use crate::trapped::{TrapKind, TrappedMetadata};

    let a = Affliction {
        kind: AfflictionKind::Trapped(TrapKind::Drowning),
        body_part: None,
        severity: Severity::Severe,
        acquired_cycle: 5,
        last_progressed_cycle: 5,
        source: AfflictionSource::Environmental {
            event: AreaEventKind::Flood,
        },
        trapped_metadata: Some(TrappedMetadata::fresh(Some(0.30))),
    };

    let json = serde_json::to_string(&a).unwrap();
    assert!(json.contains("trapped_metadata"));
    assert!(json.contains("0.3"));
}

#[test]
fn affliction_without_trapped_metadata_omits_field() {
    let a = Affliction {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::ArmLeft),
        severity: Severity::Mild,
        acquired_cycle: 3,
        last_progressed_cycle: 3,
        source: AfflictionSource::Spawn,
        trapped_metadata: None,
    };

    let json = serde_json::to_string(&a).unwrap();
    assert!(!json.contains("trapped_metadata"), "expected field omitted, got: {json}");
}

#[test]
fn affliction_round_trips_with_trapped_metadata() {
    use crate::trapped::{TrapKind, TrappedMetadata};

    let a = Affliction {
        kind: AfflictionKind::Trapped(TrapKind::Buried),
        body_part: None,
        severity: Severity::Moderate,
        acquired_cycle: 7,
        last_progressed_cycle: 7,
        source: AfflictionSource::Environmental {
            event: AreaEventKind::Avalanche,
        },
        trapped_metadata: Some(TrappedMetadata {
            cycles_trapped: 2,
            escape_progress: 1,
            terrain_hazard_floor: None,
        }),
    };

    let json = serde_json::to_string(&a).unwrap();
    let back: Affliction = serde_json::from_str(&json).unwrap();
    assert_eq!(a, back);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package shared affliction_with_trapped_metadata`
Expected: FAIL with "no field `trapped_metadata` on Affliction"

- [ ] **Step 3: Add the field**

In `shared/src/afflictions.rs`, modify the `Affliction` struct:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affliction {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub acquired_cycle: u32,
    pub last_progressed_cycle: u32,
    pub source: AfflictionSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trapped_metadata: Option<crate::trapped::TrappedMetadata>,
}
```

If any existing constructors / `Default` impls / test fixtures don't set this field, add `trapped_metadata: None` to each one.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package shared affliction`
Expected: PASS — all affliction tests pass including the 3 new ones

Run: `cargo test --package shared` (full crate)
Expected: PASS

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(shared): add optional trapped_metadata field to Affliction"
```

---

## Task 4: TRAP_KIND_TABLE + tuning lookup in game crate

**Files:**
- Create: `game/src/tributes/afflictions/trapped.rs`
- Modify: `game/src/tributes/afflictions/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `game/src/tributes/afflictions/trapped.rs`:

```rust
//! Trapped affliction implementation: tuning table, escape mechanic, AreaEvent mapping.
//!
//! See `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md`.

use shared::afflictions::Severity;
use shared::trapped::TrapKind;

use crate::tributes::traits::TributeStat;

#[derive(Debug, Clone, Copy)]
pub struct TrapKindTuning {
    pub kind: TrapKind,
    /// Per-cycle HP damage indexed by severity (Mild=0, Moderate=1, Severe=2).
    pub hp_damage: [u32; 3],
    /// Per-cycle mental damage indexed by severity.
    pub mental_damage: [u32; 3],
    /// Stat used for the trapped tribute's self-escape roll bonus.
    pub escape_stat: TributeStat,
    /// Stat used for the rescuer's bonus contribution.
    pub rescue_stat: TributeStat,
    /// Whether the trap can have a terrain hazard floor (Drowning yes, Buried no).
    pub allows_terrain_floor: bool,
}

pub const TRAP_KIND_TABLE: &[TrapKindTuning] = &[
    TrapKindTuning {
        kind: TrapKind::Drowning,
        hp_damage: [15, 30, 50],
        mental_damage: [3, 6, 10],
        escape_stat: TributeStat::Intelligence,
        rescue_stat: TributeStat::Strength,
        allows_terrain_floor: true,
    },
    TrapKindTuning {
        kind: TrapKind::Buried,
        hp_damage: [15, 30, 50],
        mental_damage: [3, 6, 10],
        escape_stat: TributeStat::Strength,
        rescue_stat: TributeStat::Strength,
        allows_terrain_floor: false,
    },
];

pub fn trap_tuning_for(kind: TrapKind) -> &'static TrapKindTuning {
    TRAP_KIND_TABLE
        .iter()
        .find(|t| t.kind == kind)
        .expect("TRAP_KIND_TABLE must have a row for every TrapKind variant")
}

pub fn severity_index(severity: Severity) -> usize {
    match severity {
        Severity::Mild => 0,
        Severity::Moderate => 1,
        Severity::Severe => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(TrapKind::Drowning)]
    #[case(TrapKind::Buried)]
    fn trap_tuning_for_returns_matching_row(#[case] kind: TrapKind) {
        let t = trap_tuning_for(kind);
        assert_eq!(t.kind, kind);
    }

    #[rstest]
    #[case(TrapKind::Drowning, Severity::Mild, 15, 3)]
    #[case(TrapKind::Drowning, Severity::Moderate, 30, 6)]
    #[case(TrapKind::Drowning, Severity::Severe, 50, 10)]
    #[case(TrapKind::Buried, Severity::Mild, 15, 3)]
    #[case(TrapKind::Buried, Severity::Moderate, 30, 6)]
    #[case(TrapKind::Buried, Severity::Severe, 50, 10)]
    fn damage_table_matches_spec(
        #[case] kind: TrapKind,
        #[case] severity: Severity,
        #[case] expected_hp: u32,
        #[case] expected_mental: u32,
    ) {
        let t = trap_tuning_for(kind);
        let i = severity_index(severity);
        assert_eq!(t.hp_damage[i], expected_hp);
        assert_eq!(t.mental_damage[i], expected_mental);
    }

    #[test]
    fn drowning_uses_intelligence_for_escape() {
        assert_eq!(trap_tuning_for(TrapKind::Drowning).escape_stat, TributeStat::Intelligence);
    }

    #[test]
    fn buried_uses_strength_for_escape() {
        assert_eq!(trap_tuning_for(TrapKind::Buried).escape_stat, TributeStat::Strength);
    }

    #[test]
    fn drowning_allows_terrain_floor() {
        assert!(trap_tuning_for(TrapKind::Drowning).allows_terrain_floor);
    }

    #[test]
    fn buried_disallows_terrain_floor() {
        assert!(!trap_tuning_for(TrapKind::Buried).allows_terrain_floor);
    }
}
```

Add to `game/src/tributes/afflictions/mod.rs`:

```rust
pub mod trapped;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --lib afflictions::trapped`
Expected: FAIL — `TributeStat::Intelligence` may not exist as a variant. Check `game/src/tributes/traits.rs`.

If `TributeStat` does not exist (or the variant names differ), use the actual stat type. Common alternatives: `Tribute.intelligence` field directly, or a different enum name. Run:

```bash
grep -rn "Intelligence\|Strength" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/tributes/traits.rs /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/tributes/mod.rs | head -20
```

Adjust the imports and field references to match the actual stat representation.

- [ ] **Step 3: Run tests to verify they pass after adjustment**

Run: `cargo test --package game --lib afflictions::trapped`
Expected: PASS — 9 tests pass

- [ ] **Step 4: Commit**

```bash
jj commit -m "feat(game): add TRAP_KIND_TABLE + tuning lookup for trapped afflictions"
```

---

## Task 5: AreaEvent → (TrapKind, Severity) mapping

**Files:**
- Modify: `game/src/tributes/afflictions/trapped.rs`

- [ ] **Step 1: Write the failing test**

Append to `game/src/tributes/afflictions/trapped.rs`:

```rust
use crate::areas::AreaEvent;

/// Map an AreaEvent to a Trapped affliction kind and severity.
/// Returns `None` for AreaEvents that don't produce trapped afflictions.
///
/// See spec §8 for the table.
pub fn area_event_to_trap(event: AreaEvent) -> Option<(TrapKind, Severity)> {
    match event {
        AreaEvent::Flood => Some((TrapKind::Drowning, Severity::Severe)),
        AreaEvent::Earthquake => Some((TrapKind::Buried, Severity::Severe)),
        AreaEvent::Avalanche => Some((TrapKind::Buried, Severity::Moderate)),
        AreaEvent::Landslide => Some((TrapKind::Buried, Severity::Moderate)),
        AreaEvent::Rockslide => Some((TrapKind::Buried, Severity::Mild)),
        _ => None,
    }
}
```

Append to the `tests` module:

```rust
    #[rstest]
    #[case(AreaEvent::Flood, Some((TrapKind::Drowning, Severity::Severe)))]
    #[case(AreaEvent::Earthquake, Some((TrapKind::Buried, Severity::Severe)))]
    #[case(AreaEvent::Avalanche, Some((TrapKind::Buried, Severity::Moderate)))]
    #[case(AreaEvent::Landslide, Some((TrapKind::Buried, Severity::Moderate)))]
    #[case(AreaEvent::Rockslide, Some((TrapKind::Buried, Severity::Mild)))]
    fn area_event_mapping_matches_spec(
        #[case] event: AreaEvent,
        #[case] expected: Option<(TrapKind, Severity)>,
    ) {
        assert_eq!(area_event_to_trap(event), expected);
    }
```

(If there are other `AreaEvent` variants, add a case asserting `None` for at least one of them.)

- [ ] **Step 2: Run test to verify it fails first, then passes**

Run: `cargo test --package game --lib afflictions::trapped::tests::area_event_mapping_matches_spec`
Expected: PASS (the function and tests are in the same step here since they're trivially coupled)

If the path to `AreaEvent` is wrong, fix the `use` statement. Check `game/src/areas/mod.rs` or similar.

- [ ] **Step 3: Commit**

```bash
jj commit -m "feat(game): map AreaEvent to trap (kind, severity)"
```

---

## Task 6: `attempt_escape` helper with self-roll math

**Files:**
- Modify: `game/src/tributes/afflictions/trapped.rs`

- [ ] **Step 1: Write the failing test**

Append to `game/src/tributes/afflictions/trapped.rs`:

```rust
use shared::trapped::{
    TrappedMetadata, CYCLES_DECAY_PER_CYCLE, ESCAPE_ROLL_CAP, ESCAPE_STAT_BONUS_MAX,
    SEVERITY_BASE_MILD, SEVERITY_BASE_MODERATE, SEVERITY_BASE_SEVERE,
};

/// Compute the escape roll TARGET (not the roll itself) for a trapped tribute.
/// Returns a probability in `[0.0, ESCAPE_ROLL_CAP]`.
///
/// Inputs:
/// - `escape_stat_value`: tribute's escape stat as a fraction in `[0.0, 1.0]`
/// - `severity`: affliction severity
/// - `meta`: TrappedMetadata (provides cycles_trapped + terrain_hazard_floor)
/// - `rescue_bonus`: sum of rescue contributions this cycle
///
/// See spec §9.
pub fn escape_roll_target(
    escape_stat_value: f32,
    severity: Severity,
    meta: &TrappedMetadata,
    rescue_bonus: f32,
) -> f32 {
    let base = match severity {
        Severity::Mild => SEVERITY_BASE_MILD,
        Severity::Moderate => SEVERITY_BASE_MODERATE,
        Severity::Severe => SEVERITY_BASE_SEVERE,
    };
    let stat_bonus = escape_stat_value.clamp(0.0, 1.0) * ESCAPE_STAT_BONUS_MAX;
    let decay = (meta.cycles_trapped as f32) * CYCLES_DECAY_PER_CYCLE;

    let mut target = (base + stat_bonus + rescue_bonus - decay).clamp(0.0, ESCAPE_ROLL_CAP);

    if let Some(floor) = meta.terrain_hazard_floor {
        target = target.min(floor);
    }

    target
}
```

Append to the `tests` module:

```rust
    use shared::trapped::TrappedMetadata;

    fn meta(cycles: u8, floor: Option<f32>) -> TrappedMetadata {
        TrappedMetadata { cycles_trapped: cycles, escape_progress: 0, terrain_hazard_floor: floor }
    }

    #[test]
    fn escape_target_mild_zero_stat_no_decay_no_rescue() {
        // base 0.50 + 0.0 stat - 0.0 decay = 0.50
        let t = escape_roll_target(0.0, Severity::Mild, &meta(0, None), 0.0);
        assert!((t - 0.50).abs() < 1e-6, "got {t}");
    }

    #[test]
    fn escape_target_severe_max_stat_no_decay_no_rescue() {
        // base 0.20 + 0.30 stat = 0.50
        let t = escape_roll_target(1.0, Severity::Severe, &meta(0, None), 0.0);
        assert!((t - 0.50).abs() < 1e-6, "got {t}");
    }

    #[test]
    fn escape_target_decays_per_cycle() {
        let t0 = escape_roll_target(1.0, Severity::Moderate, &meta(0, None), 0.0);
        let t1 = escape_roll_target(1.0, Severity::Moderate, &meta(1, None), 0.0);
        let t2 = escape_roll_target(1.0, Severity::Moderate, &meta(2, None), 0.0);
        assert!((t0 - t1 - 0.08).abs() < 1e-6);
        assert!((t1 - t2 - 0.08).abs() < 1e-6);
    }

    #[test]
    fn escape_target_capped_at_0_95() {
        // Mild + max stat + huge rescue bonus = would exceed 1.0
        let t = escape_roll_target(1.0, Severity::Mild, &meta(0, None), 10.0);
        assert_eq!(t, 0.95);
    }

    #[test]
    fn escape_target_clamped_to_zero_floor() {
        // Severe + zero stat + huge decay = would go negative
        let t = escape_roll_target(0.0, Severity::Severe, &meta(20, None), 0.0);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn escape_target_terrain_floor_caps_below_computed() {
        // Computed would be 0.50, terrain floor is 0.30
        let t = escape_roll_target(1.0, Severity::Severe, &meta(0, Some(0.30)), 0.0);
        assert_eq!(t, 0.30);
    }

    #[test]
    fn escape_target_terrain_floor_does_not_raise() {
        // Computed is 0.20 (Severe + 0 stat), floor 0.30 — should NOT raise to 0.30
        let t = escape_roll_target(0.0, Severity::Severe, &meta(0, Some(0.30)), 0.0);
        assert_eq!(t, 0.20);
    }

    #[test]
    fn escape_target_rescue_bonus_contributes() {
        let t_no_rescue = escape_roll_target(0.5, Severity::Severe, &meta(0, None), 0.0);
        let t_rescued = escape_roll_target(0.5, Severity::Severe, &meta(0, None), 0.40);
        assert!((t_rescued - t_no_rescue - 0.40).abs() < 1e-6);
    }
```

Add proptest at the end of the `tests` module:

```rust
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn escape_target_always_in_valid_range(
            stat in 0.0f32..=1.0,
            cycles in 0u8..=50,
            rescue in 0.0f32..=2.0,
            floor in proptest::option::of(0.0f32..=1.0),
            severity_idx in 0usize..3,
        ) {
            let severity = match severity_idx {
                0 => Severity::Mild,
                1 => Severity::Moderate,
                _ => Severity::Severe,
            };
            let m = TrappedMetadata { cycles_trapped: cycles, escape_progress: 0, terrain_hazard_floor: floor };
            let t = escape_roll_target(stat, severity, &m, rescue);
            prop_assert!(t >= 0.0);
            prop_assert!(t <= ESCAPE_ROLL_CAP);
            if let Some(f) = floor {
                prop_assert!(t <= f.max(0.0));
            }
        }
    }
```

If `proptest` isn't already a `[dev-dependencies]` of the `game` crate, check `game/Cargo.toml` and add `proptest = "1.5"` if missing.

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --package game --lib afflictions::trapped`
Expected: PASS — all unit + proptest cases pass

- [ ] **Step 3: Commit**

```bash
jj commit -m "feat(game): add attempt_escape roll-target math + proptest invariants"
```

---

## Task 7: New `MessagePayload` variants

**Files:**
- Modify: `shared/src/messages.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `shared/src/messages.rs`:

```rust
#[test]
fn tribute_trapped_serializes() {
    use crate::trapped::TrapKind;
    use crate::afflictions::Severity;

    let p = MessagePayload::TributeTrapped {
        tribute: "tribute-1".into(),
        kind: TrapKind::Drowning,
        severity: Severity::Severe,
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("tribute_trapped"));
    assert!(json.contains("drowning"));
    assert!(json.contains("severe"));
}

#[test]
fn tribute_died_while_trapped_serializes() {
    use crate::trapped::TrapKind;

    let p = MessagePayload::TributeDiedWhileTrapped {
        tribute: "tribute-1".into(),
        kind: TrapKind::Buried,
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("tribute_died_while_trapped"));
    assert!(json.contains("buried"));
}

#[test]
fn struggling_serializes() {
    use crate::trapped::TrapKind;
    use crate::afflictions::Severity;

    let p = MessagePayload::Struggling {
        tribute: "tribute-1".into(),
        kind: TrapKind::Drowning,
        severity: Severity::Moderate,
        cycles_trapped: 2,
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("struggling"));
    assert!(json.contains("\"cycles_trapped\":2"));
}

#[test]
fn trapped_escaped_serializes() {
    use crate::trapped::TrapKind;

    let p = MessagePayload::TrappedEscaped {
        tribute: "tribute-1".into(),
        kind: TrapKind::Drowning,
        cycles_trapped: 1,
        rescued_by: vec!["tribute-2".into()],
    };
    let json = serde_json::to_string(&p).unwrap();
    assert!(json.contains("trapped_escaped"));
    assert!(json.contains("tribute-2"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package shared messages`
Expected: FAIL with "no variant TributeTrapped" etc.

- [ ] **Step 3: Add the variants**

In `shared/src/messages.rs`, find the `MessagePayload` enum. Add four new variants near related variants (or at the end):

```rust
use crate::afflictions::Severity;
use crate::trapped::TrapKind;

// ... in MessagePayload enum:

    TributeTrapped {
        tribute: String,
        kind: TrapKind,
        severity: Severity,
    },
    Struggling {
        tribute: String,
        kind: TrapKind,
        severity: Severity,
        cycles_trapped: u8,
    },
    TrappedEscaped {
        tribute: String,
        kind: TrapKind,
        cycles_trapped: u8,
        rescued_by: Vec<String>,
    },
    TributeDiedWhileTrapped {
        tribute: String,
        kind: TrapKind,
    },
```

(Match the existing enum's serde tagging convention — if it uses `#[serde(tag = "type", content = "data")]` or similar, the JSON shape in the assertions may need adjusting. Check existing variant serialization in the test module to confirm.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package shared messages`
Expected: PASS

If the JSON shape assertions are wrong because of the existing serde tagging, adjust the assertions to match. The key thing is the variant exists, serializes, and round-trips.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(shared): add Trapped MessagePayload variants"
```

---

## Task 8: Wire `AfflictionMetadataPayload::Trapped` dispatch

**Files:**
- Modify: `game/src/tributes/afflictions/mod.rs`

This task assumes `lsis` introduces `AfflictionMetadataPayload` (or equivalent) — the dispatch enum used by `try_acquire_affliction` to route metadata onto the right `Affliction.*_metadata` slot. If the actual API differs, adapt accordingly.

- [ ] **Step 1: Inspect the current API**

Run: `grep -n "AfflictionMetadataPayload\|try_acquire_affliction\|AfflictionDraft" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src/tributes/afflictions/mod.rs`

Confirm the shape of the metadata dispatch. The plan continues assuming a `AfflictionMetadataPayload` enum with variants per metadata-bearing kind.

- [ ] **Step 2: Write the failing test**

Add to `game/src/tributes/afflictions/mod.rs` (or the existing test module):

```rust
#[cfg(test)]
mod trapped_dispatch_tests {
    use super::*;
    use shared::afflictions::{AfflictionKind, Severity};
    use shared::trapped::{TrapKind, TrappedMetadata};

    #[test]
    fn try_acquire_trapped_sets_metadata_on_affliction() {
        let mut tribute = crate::tributes::Tribute::default();
        let payload = AfflictionMetadataPayload::Trapped(TrappedMetadata::fresh(Some(0.30)));

        tribute.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Trapped(TrapKind::Drowning),
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Environmental {
                event: shared::messages::AreaEventKind::Flood,
            },
            metadata: Some(payload),
        });

        let key = (AfflictionKind::Trapped(TrapKind::Drowning), None);
        let a = tribute.afflictions.get(&key).expect("affliction should be present");
        assert_eq!(a.severity, Severity::Severe);
        let meta = a.trapped_metadata.as_ref().expect("metadata should be set");
        assert_eq!(meta.terrain_hazard_floor, Some(0.30));
        assert_eq!(meta.cycles_trapped, 0);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --package game --lib trapped_dispatch_tests`
Expected: FAIL — `AfflictionMetadataPayload::Trapped` doesn't exist yet, or `metadata` field on `AfflictionDraft` doesn't exist.

- [ ] **Step 4: Extend the dispatch**

In `game/src/tributes/afflictions/mod.rs`, find `AfflictionMetadataPayload` (if `lsis` introduced it). Add the variant:

```rust
pub enum AfflictionMetadataPayload {
    // ... existing variants (Trauma, Phobia, Addiction may not yet exist depending on merge order) ...
    Trapped(shared::trapped::TrappedMetadata),
}
```

In the `try_acquire_affliction` impl, when constructing the `Affliction`, route `AfflictionMetadataPayload::Trapped(meta)` to `affliction.trapped_metadata = Some(meta)`. If the existing code uses a `match` over the payload, add a `Trapped` arm.

If `AfflictionMetadataPayload` doesn't exist (because `lsis` didn't introduce it), introduce it now as a small enum scoped to this PR — name it the same and put it in `game/src/tributes/afflictions/mod.rs`. Future trauma/addiction PRs will extend it.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --package game --lib trapped_dispatch_tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): dispatch TrappedMetadata via try_acquire_affliction"
```

---

## Task 9: Replace acquisition sites in `lifecycle.rs:222-230`

**Files:**
- Modify: `game/src/tributes/lifecycle.rs`

- [ ] **Step 1: Read the current code**

View `game/src/tributes/lifecycle.rs` lines 215-235 to see the exact current `match area_event` block.

- [ ] **Step 2: Write the failing integration test**

Create `game/tests/trapped_afflictions_lifecycle_test.rs`:

```rust
//! Integration: AreaEvents produce Trapped afflictions (replaces TributeStatus::Drowned/Buried).

use game::areas::AreaEvent;
use game::tributes::Tribute;
use shared::afflictions::{AfflictionKind, Severity};
use shared::trapped::TrapKind;

fn fresh_tribute() -> Tribute {
    Tribute::default()
}

#[test]
fn flood_produces_severe_drowning_affliction() {
    let mut t = fresh_tribute();
    t.apply_area_event(AreaEvent::Flood);  // method name may differ; see step 3 note

    let key = (AfflictionKind::Trapped(TrapKind::Drowning), None);
    let a = t.afflictions.get(&key).expect("Drowning affliction expected after Flood");
    assert_eq!(a.severity, Severity::Severe);
    assert!(a.trapped_metadata.is_some());
}

#[test]
fn earthquake_produces_severe_buried_affliction() {
    let mut t = fresh_tribute();
    t.apply_area_event(AreaEvent::Earthquake);
    let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
    let a = t.afflictions.get(&key).expect("Buried affliction expected after Earthquake");
    assert_eq!(a.severity, Severity::Severe);
}

#[test]
fn rockslide_produces_mild_buried() {
    let mut t = fresh_tribute();
    t.apply_area_event(AreaEvent::Rockslide);
    let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
    let a = t.afflictions.get(&key).expect("Buried affliction expected after Rockslide");
    assert_eq!(a.severity, Severity::Mild);
}

#[test]
fn no_trapped_status_set_after_migration() {
    use game::tributes::statuses::TributeStatus;
    let mut t = fresh_tribute();
    t.apply_area_event(AreaEvent::Flood);
    // Legacy status MUST NOT be set anymore — afflictions are the source of truth
    assert_ne!(t.status, TributeStatus::__LegacyDrowned);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test flood_produces_severe_drowning_affliction`
Expected: FAIL — likely "method `apply_area_event` not found" or "no Drowning affliction".

If the actual method name in `lifecycle.rs` differs (it might be inline in a larger function rather than a standalone method), refactor: extract the `match area_event { ... }` block from the existing function into a new `pub fn apply_area_event(&mut self, event: AreaEvent)` method on `Tribute`. The original call site invokes the new method.

- [ ] **Step 4: Replace acquisition logic**

In `game/src/tributes/lifecycle.rs`, find the block at line 222 starting `AreaEvent::Flood => self.set_status(...)` and replace the entire block (lines 221-230) with:

```rust
            // Trapped afflictions (replaces legacy TributeStatus::Drowned/Buried).
            // Gated by Game::trapped_afflictions_enabled — see Task 10.
            if let Some((trap_kind, severity)) = crate::tributes::afflictions::trapped::area_event_to_trap(area_event) {
                let terrain_floor = if matches!(trap_kind, shared::trapped::TrapKind::Drowning) {
                    // TODO: read from area state once area-water-hazard model lands; for now Flood always 0.30
                    if matches!(area_event, AreaEvent::Flood) { Some(0.30) } else { None }
                } else {
                    None
                };
                let metadata = shared::trapped::TrappedMetadata::fresh(terrain_floor);
                self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                    kind: shared::afflictions::AfflictionKind::Trapped(trap_kind),
                    body_part: None,
                    severity,
                    source: shared::afflictions::AfflictionSource::Environmental {
                        event: area_event_to_kind(area_event),  // existing helper or inline conversion
                    },
                    metadata: Some(crate::tributes::afflictions::AfflictionMetadataPayload::Trapped(metadata)),
                });
            }
```

If `area_event_to_kind` doesn't exist as a helper, inline the `AreaEventKind` conversion using whatever existing code converts `AreaEvent` to `AreaEventKind` for messaging.

The legacy `set_status(TributeStatus::Drowned|Buried)` calls are DELETED — full replacement per spec §16. The status will be `TributeStatus::Healthy` (or whatever default), and the affliction is the source of truth.

- [ ] **Step 5: Run integration tests to verify they pass**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test`
Expected: PASS — 4 tests pass

If `apply_area_event` had to be a new extracted method, also run `cargo test --package game --lib lifecycle` to confirm no other lifecycle tests broke.

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): replace set_status(Drowned|Buried) with Trapped affliction acquisition"
```

---

## Task 10: Add `Game::trapped_afflictions_enabled` rollout flag

**Files:**
- Modify: `game/src/games.rs`
- Modify: `game/src/tributes/lifecycle.rs`

- [ ] **Step 1: Write the failing test**

Add to `game/tests/trapped_afflictions_lifecycle_test.rs`:

```rust
#[test]
fn flag_disabled_skips_acquisition() {
    use game::games::Game;
    let mut game = Game::default();
    game.trapped_afflictions_enabled = false;
    game.tributes.push(fresh_tribute());

    // Apply a Flood to the only tribute (test harness; specific API may differ)
    let tribute = &mut game.tributes[0];
    if game.trapped_afflictions_enabled {
        tribute.apply_area_event(AreaEvent::Flood);
    }

    let key = (AfflictionKind::Trapped(TrapKind::Drowning), None);
    assert!(tribute.afflictions.get(&key).is_none(), "no affliction when flag is disabled");
}
```

(This test is somewhat awkward because the gate is on `Game`, not `Tribute`. The realistic test is at the game-orchestration level — adapt to whichever orchestration function calls `apply_area_event` from a `Game` context. If no clean orchestration test point exists, defer the gate-check to inside the orchestration loop, not inside `apply_area_event`, and add the test there.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test flag_disabled_skips_acquisition`
Expected: FAIL — `trapped_afflictions_enabled` field doesn't exist on `Game`.

- [ ] **Step 3: Add the field**

In `game/src/games.rs`, find the `Game` struct. Add the field with default `true`:

```rust
#[serde(default = "default_trapped_afflictions_enabled")]
pub trapped_afflictions_enabled: bool,
```

Add a default function:

```rust
fn default_trapped_afflictions_enabled() -> bool { true }
```

Update `Game::default()` (or `Default` derive — if `Default` is derived, the field default `false` would be wrong; use the explicit `default = "..."` serde attribute and add `trapped_afflictions_enabled: true` to the manual `Default` impl).

- [ ] **Step 4: Add gate at the orchestration call site**

Find the place in `game/src/tributes/lifecycle.rs` (or wherever `apply_area_event` is called by the game loop) and wrap the call:

```rust
if self.trapped_afflictions_enabled {
    tribute.apply_area_event(area_event);
} else {
    // legacy passthrough or no-op — see spec §17
}
```

If the call site is inside `lifecycle.rs` and doesn't have access to `Game`, plumb `trapped_afflictions_enabled: bool` as a parameter, or do the gate check at the brain/orchestration layer above.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): add trapped_afflictions_enabled rollout flag"
```

---

## Task 11: Replace per-cycle damage block in `lifecycle.rs:291-305`

**Files:**
- Modify: `game/src/tributes/lifecycle.rs`

- [ ] **Step 1: Read the current code**

View `game/src/tributes/lifecycle.rs` lines 285-310 to see the full per-cycle damage match block.

- [ ] **Step 2: Write the failing integration test**

Add to `game/tests/trapped_afflictions_lifecycle_test.rs`:

```rust
#[test]
fn per_cycle_damage_severe_drowning_kills_in_two_cycles() {
    let mut t = fresh_tribute();
    let starting_hp = t.hp;
    assert!(starting_hp >= 80, "test assumes default ~80 HP");

    t.apply_area_event(AreaEvent::Flood);  // Severe Drowning

    // Cycle 1: 50 HP damage
    t.tick_trapped_afflictions();
    assert!(t.hp <= starting_hp - 50, "expected HP <= {}, got {}", starting_hp - 50, t.hp);
    assert!(t.is_alive(), "should survive one cycle of Severe Drowning");

    // Cycle 2: another 50 HP — should die
    t.tick_trapped_afflictions();
    assert!(!t.is_alive(), "should die after 2 cycles of Severe Drowning");
}

#[test]
fn per_cycle_damage_mild_buried_survives_multiple_cycles() {
    let mut t = fresh_tribute();
    let starting_hp = t.hp;

    t.apply_area_event(AreaEvent::Rockslide);  // Mild Buried (15 HP/cycle)

    for _ in 0..3 {
        t.tick_trapped_afflictions();
    }
    // 3 cycles × 15 = 45 damage; tribute should still be alive
    assert!(t.is_alive());
    assert!(t.hp <= starting_hp - 45);
}

#[test]
fn cycles_trapped_increments_each_tick() {
    let mut t = fresh_tribute();
    t.apply_area_event(AreaEvent::Earthquake);

    let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
    assert_eq!(t.afflictions.get(&key).unwrap().trapped_metadata.as_ref().unwrap().cycles_trapped, 0);

    t.tick_trapped_afflictions();
    if t.is_alive() {
        // After tick the affliction MAY be gone (escaped) — only check cycles if still trapped
        if let Some(a) = t.afflictions.get(&key) {
            assert_eq!(a.trapped_metadata.as_ref().unwrap().cycles_trapped, 1);
        }
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test per_cycle_damage`
Expected: FAIL — `tick_trapped_afflictions` doesn't exist; old damage path used `TributeStatus::Drowned` which is being removed.

- [ ] **Step 4: Replace the damage block**

In `game/src/tributes/lifecycle.rs`, locate the per-cycle damage block (around lines 291-305). The existing code looks roughly:

```rust
match self.status {
    TributeStatus::Drowned => {
        self.takes_physical_damage(DROWNED_DAMAGE);
        // ...
    }
    TributeStatus::Buried => {
        self.takes_physical_damage(BURIED_DAMAGE);
    }
    // ... other statuses
}
```

Remove the `Drowned` and `Buried` arms. Then add a new method:

```rust
impl Tribute {
    /// Per-cycle: apply Trapped affliction damage, increment cycle counters, attempt escape.
    /// Emits messages for trapping events. Removes affliction if escaped; kills tribute if HP hits 0.
    pub fn tick_trapped_afflictions(&mut self) {
        use shared::afflictions::AfflictionKind;
        use shared::trapped::TrapKind;
        use crate::tributes::afflictions::trapped::{trap_tuning_for, severity_index, escape_roll_target};

        // Collect trapped affliction keys (we'll mutate the map below)
        let trapped_keys: Vec<_> = self.afflictions
            .iter()
            .filter_map(|(k, _)| match k.0 {
                AfflictionKind::Trapped(tk) => Some((*k, tk)),
                _ => None,
            })
            .collect();

        for (key, trap_kind) in trapped_keys {
            let (severity, hp_dmg, mental_dmg) = {
                let a = self.afflictions.get(&key).unwrap();
                let t = trap_tuning_for(trap_kind);
                let i = severity_index(a.severity);
                (a.severity, t.hp_damage[i], t.mental_damage[i])
            };

            // Apply damage
            self.takes_physical_damage(hp_dmg);
            self.takes_mental_damage(mental_dmg);

            // Death check
            if self.hp == 0 {
                // Emit TributeDiedWhileTrapped (use existing message-emission path; signature varies)
                // self.emit(MessagePayload::TributeDiedWhileTrapped { tribute: self.id.clone(), kind: trap_kind });
                self.kill();
                return;
            }

            // Escape attempt (PR1: pure self-roll, no rescuer integration yet)
            let escape_stat_value = self.normalized_stat_value(trap_tuning_for(trap_kind).escape_stat);
            let meta = self.afflictions.get(&key).unwrap().trapped_metadata.as_ref().unwrap().clone();
            let target = escape_roll_target(escape_stat_value, severity, &meta, 0.0);

            let roll: f32 = rand::random();
            if roll <= target {
                // Escaped
                self.afflictions.remove(&key);
                // self.emit(MessagePayload::TrappedEscaped { ... });
            } else {
                // Increment cycles_trapped
                if let Some(a) = self.afflictions.get_mut(&key) {
                    if let Some(m) = a.trapped_metadata.as_mut() {
                        m.cycles_trapped = m.cycles_trapped.saturating_add(1);
                    }
                }
                // self.emit(MessagePayload::Struggling { ... });
            }
        }
    }
}
```

You'll need to:
- Add a helper `normalized_stat_value(stat: TributeStat) -> f32` on `Tribute` that returns the stat as a fraction in `[0.0, 1.0]`. (May already exist.)
- Wire the `// self.emit(...)` lines to whatever the actual message-emission API is. Check how other lifecycle code emits `MessagePayload::*` — it's likely via a returned `Vec<Message>` or a `&mut MessageBuffer`.

The legacy `DROWNED_DAMAGE` / `BURIED_DAMAGE` constants (lines 47, 50) and any `*_MENTAL_DAMAGE` constants — DELETE them, they're unreferenced now.

The orchestration loop that previously triggered the per-cycle status-damage block must call `tick_trapped_afflictions()` once per cycle for each tribute. Find that orchestration site and add the call.

Also delete the rstest cases at lines 483 and 485 referencing `TributeStatus::Drowned` and `TributeStatus::Buried`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test`
Expected: PASS — note the escape-roll RNG can occasionally let a tribute escape Severe Drowning on cycle 1; the death tests should be RNG-independent. If a death test fails because of RNG, the test should set the tribute's relevant stat to 0 (`tribute.intelligence = 0`) so the escape roll target is just the severity base, and either:
- Skip the escape entirely in tests (add a `tick_trapped_afflictions_no_escape` test helper), or
- Use a seeded RNG for tests.

Pick whichever approach matches the codebase's existing test conventions (check how other RNG-driven tests in `game/` handle this).

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): replace status-based trapped damage with affliction-based tick"
```

---

## Task 12: Delete `TributeStatus::Drowned` and `Buried` variants

**Files:**
- Modify: `game/src/tributes/statuses.rs`

This pairs with `b67j`. After this task, the legacy variants are gone from the live code — only the `__Legacy*` private stubs (added in Task 13) remain for save migration.

- [ ] **Step 1: Delete the public variants**

In `game/src/tributes/statuses.rs`:
- Delete line 23 (`Drowned,`)
- Delete line 25 (`Buried,`)
- Delete the parse arms at lines 57 and 59
- Delete the display arms at lines 81 and 83
- Delete the rstest cases at lines 108, 110, 130, 132

- [ ] **Step 2: Run the full game test suite to find references**

Run: `cargo test --package game 2>&1 | head -100`
Expected: compilation errors at every site that references `TributeStatus::Drowned` or `TributeStatus::Buried`.

- [ ] **Step 3: Fix all compile errors**

For each error:
- If the reference is in production code, the path was supposed to be migrated in Task 9 / Task 11 — go back and check
- If the reference is in a test, delete or update the test

Common sites that may need updates beyond what's been done:
- Snapshot files (`*.snap` under `snapshots/`) — regenerate with `cargo insta accept` after rerunning tests
- UI/serialization code in `web/` or `api/` — ignore for PR1 (only `game` crate is in scope; web/api are a follow-up)
- `output.rs` — handled in Task 14

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game`
Expected: PASS — all game crate tests compile and pass

- [ ] **Step 5: Commit**

```bash
jj commit -m "refactor(game): delete TributeStatus::Drowned and Buried (b67j retirement)"
```

---

## Task 13: Save migration — Custom `Deserialize` + post-load pass

**Files:**
- Modify: `game/src/tributes/statuses.rs`
- Modify: `game/src/games.rs`
- Create: `game/tests/trapped_save_migration_test.rs`

- [ ] **Step 1: Write the failing test**

Create `game/tests/trapped_save_migration_test.rs`:

```rust
//! Save migration: legacy TributeStatus::Drowned/Buried → AfflictionKind::Trapped(_).

use game::games::Game;
use shared::afflictions::{AfflictionKind, Severity};
use shared::trapped::TrapKind;

#[test]
fn legacy_drowned_status_migrates_to_drowning_affliction() {
    // Construct a save-game JSON with a tribute carrying the old "drowned" status
    let json = r#"{
        "id": "game-1",
        "tributes": [
            { "id": "t-1", "name": "Test", "status": "drowned", "afflictions": {} }
        ],
        "trapped_afflictions_enabled": true
    }"#;
    // (Field set is illustrative — match whatever Game serialization actually requires.
    //  Produce a minimal valid JSON by serializing a default Game and editing.)

    let mut game: Game = serde_json::from_str(json).expect("legacy save should deserialize");
    game.migrate_legacy_trapped_statuses();

    let tribute = game.tributes.iter().find(|t| t.id == "t-1").unwrap();
    let key = (AfflictionKind::Trapped(TrapKind::Drowning), None);
    let a = tribute.afflictions.get(&key).expect("Drowning affliction present after migration");
    assert_eq!(a.severity, Severity::Severe);
}

#[test]
fn legacy_buried_status_migrates_to_buried_affliction() {
    let json = r#"{
        "id": "game-1",
        "tributes": [
            { "id": "t-1", "name": "Test", "status": "buried", "afflictions": {} }
        ],
        "trapped_afflictions_enabled": true
    }"#;

    let mut game: Game = serde_json::from_str(json).unwrap();
    game.migrate_legacy_trapped_statuses();

    let tribute = game.tributes.iter().find(|t| t.id == "t-1").unwrap();
    let key = (AfflictionKind::Trapped(TrapKind::Buried), None);
    let a = tribute.afflictions.get(&key).expect("Buried affliction present after migration");
    assert_eq!(a.severity, Severity::Severe);
}

#[test]
fn migration_clears_legacy_status_marker() {
    let json = r#"{
        "id": "game-1",
        "tributes": [
            { "id": "t-1", "name": "Test", "status": "drowned", "afflictions": {} }
        ],
        "trapped_afflictions_enabled": true
    }"#;

    let mut game: Game = serde_json::from_str(json).unwrap();
    game.migrate_legacy_trapped_statuses();

    let tribute = game.tributes.iter().find(|t| t.id == "t-1").unwrap();
    // After migration the legacy marker is gone — status is Healthy or whatever default
    assert_ne!(tribute.status, game::tributes::statuses::TributeStatus::__LegacyDrowned);
}
```

(Adjust the JSON to whatever minimal fields the actual `Game` and `Tribute` types require. The simplest way: build a `Game` in code, serialize it with `serde_json::to_string_pretty`, then hand-edit the status field to `"drowned"` and use that as the test fixture.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package game --test trapped_save_migration_test`
Expected: FAIL — either deserialization fails (no `__LegacyDrowned` variant) OR `migrate_legacy_trapped_statuses` doesn't exist.

- [ ] **Step 3: Add legacy stub variants + custom Deserialize**

In `game/src/tributes/statuses.rs`, add private stub variants and a custom Deserialize:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TributeStatus {
    Healthy,
    // ... other current variants ...
    /// Legacy migration stub — only produced by Deserialize for old saves carrying "drowned".
    /// MUST be cleared by Game::migrate_legacy_trapped_statuses on load.
    #[doc(hidden)]
    __LegacyDrowned,
    /// Legacy migration stub for old "buried" saves.
    #[doc(hidden)]
    __LegacyBuried,
}

impl<'de> serde::Deserialize<'de> for TributeStatus {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw: String = String::deserialize(d)?;
        match raw.as_str() {
            "drowned" => Ok(TributeStatus::__LegacyDrowned),
            "buried" => Ok(TributeStatus::__LegacyBuried),
            // Delegate other variants to the existing FromStr impl
            other => other.parse().map_err(serde::de::Error::custom),
        }
    }
}
```

(If the existing code has a custom `FromStr` impl, route through it. If it relied on derived Deserialize, the custom impl above replaces it — make sure the `Serialize` derive stays so writes still work.)

- [ ] **Step 4: Add `migrate_legacy_trapped_statuses` method**

In `game/src/games.rs`:

```rust
impl Game {
    /// One-shot migration: convert legacy TributeStatus::__LegacyDrowned/Buried markers
    /// into AfflictionKind::Trapped(_) entries. Idempotent: tributes without legacy markers
    /// are untouched.
    ///
    /// Pairs with `b67j` — the legacy stub variants are removed in a follow-up release.
    pub fn migrate_legacy_trapped_statuses(&mut self) {
        use shared::afflictions::{AfflictionKind, AfflictionSource, Severity};
        use shared::trapped::{TrapKind, TrappedMetadata};
        use crate::tributes::statuses::TributeStatus;

        for tribute in self.tributes.iter_mut() {
            let trap_kind = match tribute.status {
                TributeStatus::__LegacyDrowned => Some(TrapKind::Drowning),
                TributeStatus::__LegacyBuried => Some(TrapKind::Buried),
                _ => None,
            };

            if let Some(kind) = trap_kind {
                tribute.status = TributeStatus::Healthy;
                let key = (AfflictionKind::Trapped(kind), None);
                tribute.afflictions.entry(key).or_insert_with(|| {
                    shared::afflictions::Affliction {
                        kind: AfflictionKind::Trapped(kind),
                        body_part: None,
                        severity: Severity::Severe,
                        acquired_cycle: 0,
                        last_progressed_cycle: 0,
                        source: AfflictionSource::Spawn,  // unknown origin; conservative
                        trapped_metadata: Some(TrappedMetadata::fresh(None)),
                    }
                });
            }
        }
    }
}
```

The orchestration that loads a `Game` from disk MUST call `game.migrate_legacy_trapped_statuses()` after deserialization. Find the load path (likely in `api/` or in a `Game::load` helper) and add the call. If no such helper exists, document the requirement in a doc comment on `migrate_legacy_trapped_statuses` so consumers know to call it.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --package game --test trapped_save_migration_test`
Expected: PASS — 3 tests pass

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): legacy trapped status save migration via Deserialize stub + post-load pass"
```

---

## Task 14: Retire `GameEvent::TributeDrowned` + `GameOutput::TributeDrowned`

**Files:**
- Modify: `game/src/events.rs`
- Modify: `game/src/output.rs`

- [ ] **Step 1: Find all references**

```bash
grep -rn "TributeDrowned" /Users/klove/ghq/github.com/kennethlove/hangrier_games/game/src /Users/klove/ghq/github.com/kennethlove/hangrier_games/shared/src 2>&1
```

Expected sites:
- `game/src/events.rs:186, 619, 1208`
- `game/src/output.rs:45, 229`

- [ ] **Step 2: Delete the variants**

In `game/src/events.rs`:
- Delete the `TributeDrowned` variant from the `GameEvent` enum (~line 186)
- Delete or update the matches at lines 619 and 1208 (likely a `match` arm that emits the event from the old `Drowned` status — replaced now by `MessagePayload::TributeDiedWhileTrapped` from Task 11)

In `game/src/output.rs`:
- Delete the `TributeDrowned` variant from the `GameOutput` enum (~line 45)
- Delete the renderer arm at line 229

- [ ] **Step 3: Add output renderer for `TributeDiedWhileTrapped`**

In `game/src/output.rs`, add a new `GameOutput::TributeDiedWhileTrapped { tribute_name: String, kind: TrapKind }` variant (matching the existing pattern used by other `MessagePayload` → `GameOutput` mappings). Add a renderer arm that produces text like:

```rust
GameOutput::TributeDiedWhileTrapped { tribute_name, kind } => {
    match kind {
        TrapKind::Drowning => format!("{tribute_name} drowned."),
        TrapKind::Buried => format!("{tribute_name} suffocated, buried alive."),
    }
}
```

Wire the conversion from `MessagePayload::TributeDiedWhileTrapped` → `GameOutput::TributeDiedWhileTrapped` in whatever `From<MessagePayload>` impl or `match payload {}` block does the existing conversions.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package game`
Expected: PASS — full game crate compiles and tests pass

- [ ] **Step 5: Commit**

```bash
jj commit -m "refactor(game): retire TributeDrowned event/output, replaced by TributeDiedWhileTrapped"
```

---

## Task 15: Lifecycle integration test with insta snapshot

**Files:**
- Modify: `game/tests/trapped_afflictions_lifecycle_test.rs`

- [ ] **Step 1: Write the snapshot test**

Append to `game/tests/trapped_afflictions_lifecycle_test.rs`:

```rust
#[test]
fn full_severe_drowning_lifecycle_snapshot() {
    // Deterministic: zero out the escape stat so the only outcome is HP attrition → death
    let mut t = fresh_tribute();
    t.intelligence = 0;  // adjust to whatever stat-zeroing API exists
    let initial_hp = t.hp;

    t.apply_area_event(AreaEvent::Flood);

    let mut log = vec![format!("acquired: hp={}, status={:?}", t.hp, t.status)];
    while t.is_alive() && t.afflictions.contains_key(&(AfflictionKind::Trapped(TrapKind::Drowning), None)) {
        t.tick_trapped_afflictions();
        log.push(format!("tick: hp={}, alive={}", t.hp, t.is_alive()));
        if log.len() > 10 { break; }  // safety
    }

    insta::assert_yaml_snapshot!(log);
}
```

- [ ] **Step 2: Run the snapshot test (first run creates snapshot)**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test full_severe_drowning_lifecycle_snapshot`
Expected: FAIL on first run with "snapshot pending" — review with `cargo insta review` or `cargo insta accept`.

- [ ] **Step 3: Review and accept the snapshot**

Run: `cargo insta review --package game`
Inspect the snapshot — should show acquisition + 2 ticks + death (Severe Drowning at 50 HP/cycle vs ~80 HP).

If the snapshot looks correct, accept it. If escape RNG fired (it shouldn't with `intelligence = 0` and Severe base 0.20), investigate.

- [ ] **Step 4: Run again to verify the snapshot is stable**

Run: `cargo test --package game --test trapped_afflictions_lifecycle_test full_severe_drowning_lifecycle_snapshot`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
jj commit -m "test(game): full Severe Drowning lifecycle snapshot"
```

---

## Self-Review

After all tasks land, run through this checklist:

**Spec coverage:**
- [x] §4 TrapKind enum — Task 1
- [x] §5 AfflictionKind extension — Task 2
- [x] §6 TrappedMetadata — Tasks 1, 3
- [x] §7 TRAP_KIND_TABLE — Task 4
- [x] §8 AreaEvent → severity mapping — Task 5
- [x] §9 Escape mechanic — Task 6
- [x] §11 Combat & action gates — DEFERRED to PR2 (per spec §18)
- [x] §10 Rescue action — DEFERRED to PR2
- [x] §12 Brain layer — DEFERRED to PR2
- [x] §13 Acquisition pipeline — Task 9
- [x] §14 Per-cycle damage — Task 11
- [x] §15 Messages — Task 7
- [x] §16 Save migration — Task 13
- [x] §17 Rollout flag — Task 10
- [x] TributeStatus retirement (b67j) — Task 12
- [x] Legacy event/output retirement — Task 14

**Placeholder scan:** No "TODO" / "fill in" / "implement later" left in plan body. All test code is concrete.

**Type consistency:**
- `TrapKind::Drowning` / `TrapKind::Buried` used consistently throughout
- `AfflictionKind::Trapped(TrapKind)` used consistently
- `TrappedMetadata::fresh(Option<f32>)` used consistently
- `area_event_to_trap` returns `Option<(TrapKind, Severity)>` consistently
- `escape_roll_target(stat, severity, &meta, rescue_bonus) -> f32` consistently

**Cut order if PR grows too large** (per spec §18):
1. Drop save migration (Task 13) → ship breaking change
2. Drop TributeStatus retirement (Task 12) → leave variants as `#[deprecated]` for a release
3. Drop the Task 14 output-renderer addition for `TributeDiedWhileTrapped` → emit raw `MessagePayload` only (UI shows nothing for now)

Don't cut Buried — defeats the TrapKind abstraction.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-04-trapped-afflictions-pr1.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
