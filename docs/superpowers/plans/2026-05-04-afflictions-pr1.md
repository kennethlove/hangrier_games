# Afflictions PR1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the foundational types, storage, anatomy resolution, and parallel-write migration for the afflictions system per spec §9 and §18 steps 1–2. Existing `TributeStatus` producers ALSO write to `Tribute::afflictions`. Legacy effects continue to flow through `TributeStatus` paths unchanged this PR.

**Architecture:** New `shared::afflictions` module defines wire-visible types (`AfflictionKind`, `BodyPart`, `Severity`, `AfflictionKey`, `AfflictionSource`, `Affliction`). New `game::tributes::afflictions` module owns anatomy resolution (`can_acquire`), tuning placeholders, and the `Tribute::try_acquire_affliction` API. Storage is a `BTreeMap<AfflictionKey, Affliction>` field on `Tribute` (default empty, serde-skipped when empty for backward compatibility). Five existing `set_status(...)` producer sites in `game/src/tributes/lifecycle.rs` are augmented with parallel `try_acquire_affliction` calls. SurrealDB schema gains an optional `afflictions` array on the `tribute` table via a new migration definition.

**Tech Stack:** Rust 2024 edition, Cargo workspace, serde, BTreeMap (deterministic iter for snapshot tests), uuid::Uuid (existing tribute id type), rstest (case-table unit tests), proptest (invariant tests), insta (snapshot tests), surrealdb-migrations (schema migration), Tailwind unaffected (PR4).

**Bead:** `hangrier_games-lsis` — afflictions PR1: types, storage, anatomy resolution, parallel-write migration

**Depends on:** none (afflictions PR1 is the entry point of the trilogy)

**Out of PR1 (deferred to later PRs):**
- Cure / cascade / shelter integration (PR3)
- Brain pipeline `affliction_override` layer (PR2)
- `MessagePayload::AfflictionAcquired/Progressed/Healed/Cascaded` (PR2)
- Combat inflict tables (PR2)
- Frontend (PR4)
- Deletion of migrated `TributeStatus` variants (PR2/PR3 after consumers migrate)

---

## File Structure

**New files:**
- `shared/src/afflictions.rs` — wire-visible affliction types
- `game/src/tributes/afflictions/mod.rs` — module entry, `try_acquire_affliction` API, `AfflictionDraft`
- `game/src/tributes/afflictions/anatomy.rs` — `AcquireResolution`, `can_acquire(existing, new)` table
- `game/src/tributes/afflictions/tuning.rs` — `AfflictionTuning` placeholder defaults
- `migrations/definitions/20260504_010000_TributeAfflictions.json` — additive schema migration
- `game/tests/afflictions_storage_test.rs` — integration tests for `try_acquire_affliction`
- `game/src/tributes/snapshots/affliction_btreemap_canonical.snap` — insta baseline (created by test runner)

**Modified files:**
- `shared/src/lib.rs` — `pub mod afflictions;`
- `game/src/tributes/mod.rs` — add `pub mod afflictions;`, add `pub afflictions: BTreeMap<AfflictionKey, Affliction>` field with `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`, initialize in `Tribute::default()` (line 282), wire `try_acquire_affliction` calls into the five `set_status` sites
- `game/src/tributes/lifecycle.rs` — at each `self.set_status(TributeStatus::X)` for migrated kinds (lines 221, 225, 227, 228, 229), follow with `self.try_acquire_affliction(AfflictionDraft { ... })`. The legacy `set_status` call STAYS — this is parallel-write per spec §18 step 2.
- `schemas/tribute.surql` — add `DEFINE FIELD OVERWRITE afflictions ON tribute TYPE option<array<object>>;`

**Files NOT touched in PR1:** `messages.rs`, `combat.rs`, `combat_beat.rs`, `brains.rs`, `web/`, `api/`. Those are PR2/PR3/PR4 territory.

---

## Task 1: shared::afflictions types

**Files:**
- Create: `shared/src/afflictions.rs`
- Modify: `shared/src/lib.rs` (add `pub mod afflictions;` after line 7)
- Test: inline `#[cfg(test)]` in `shared/src/afflictions.rs`

- [ ] **Step 1: Write the failing test**

Create `shared/src/afflictions.rs` with the test module first (TDD — types come from making this compile and pass).

```rust
// shared/src/afflictions.rs
// (full file content in Step 3)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affliction_key_uses_kind_and_body_part() {
        let a = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::ArmLeft),
            severity: Severity::Mild,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            source: AfflictionSource::Spawn,
        };
        assert_eq!(a.key(), (AfflictionKind::Wounded, Some(BodyPart::ArmLeft)));
    }

    #[test]
    fn permanent_kinds_report_permanent() {
        assert!(Affliction::is_kind_permanent(AfflictionKind::MissingLimb));
        assert!(Affliction::is_kind_permanent(AfflictionKind::Blind));
        assert!(Affliction::is_kind_permanent(AfflictionKind::Deaf));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Wounded));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Broken));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Infected));
    }

    #[test]
    fn severity_ordering_is_total() {
        assert!(Severity::Mild < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Severe);
    }

    #[test]
    fn btreemap_serialization_is_deterministic() {
        use std::collections::BTreeMap;
        let mut a: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        a.insert(
            (AfflictionKind::Wounded, Some(BodyPart::Torso)),
            Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(BodyPart::Torso),
                severity: Severity::Mild,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                source: AfflictionSource::Spawn,
            },
        );
        a.insert(
            (AfflictionKind::Broken, Some(BodyPart::ArmLeft)),
            Affliction {
                kind: AfflictionKind::Broken,
                body_part: Some(BodyPart::ArmLeft),
                severity: Severity::Moderate,
                acquired_cycle: 2,
                last_progressed_cycle: 2,
                source: AfflictionSource::Spawn,
            },
        );
        let s1 = serde_json::to_string(&a).unwrap();
        let s2 = serde_json::to_string(&a).unwrap();
        assert_eq!(s1, s2, "BTreeMap serialization must be deterministic");
    }
}
```

- [ ] **Step 2: Run test to verify it fails (compile error)**

Run: `cargo test -p shared afflictions 2>&1 | head -20`
Expected: FAIL with "unresolved import" / "cannot find type" — types don't exist yet.

- [ ] **Step 3: Write the minimal implementation**

Replace the stub `shared/src/afflictions.rs` with full content:

```rust
//! Wire-visible affliction types. Lives in `shared/` because `Tribute::afflictions`
//! is serialized to SurrealDB and broadcast over the WebSocket protocol.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md` §9.

use serde::{Deserialize, Serialize};

use crate::messages::AreaEventKind;

/// Categories of afflictions a tribute can carry. Permanent kinds
/// (`MissingLimb`, `Blind`, `Deaf`) cannot be cured in v1; reversible kinds
/// progress / heal via the cascade and cure paths.
///
/// `AfflictionKey` discriminates by the variant tag, so phobia and fixation
/// extensions added in later specs (e.g. `Phobia(PhobiaTrigger)`) collide on
/// the discriminator regardless of their inner payload.
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
}

/// Anatomical attachment points for body-part-specific afflictions.
/// `Eyes` and `Ears` are unique slots (no L/R split in v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPart {
    ArmLeft,
    ArmRight,
    LegLeft,
    LegRight,
    Torso,
    Head,
    Eyes,
    Ears,
}

/// Severity tier for tier-scaled afflictions. Permanent kinds are always
/// `Severe` in practice; tier ordering is total (Mild < Moderate < Severe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Mild,
    Moderate,
    Severe,
}

/// Storage discriminator. Same kind on different parts is independent;
/// same kind on the same part collapses to one slot.
pub type AfflictionKey = (AfflictionKind, Option<BodyPart>);

/// Origin of an affliction. `Sponsor` and `Gamemaker` variants are reserved
/// for future systems but ship in v1 to avoid enum churn (per spec §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AfflictionSource {
    Spawn,
    Combat { attacker_id: String },
    Environmental { event: AreaEventKind },
    Cascade { from: AfflictionKey },
    Sponsor,
    Gamemaker,
}

/// A single affliction slot on a tribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affliction {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub acquired_cycle: u32,
    pub last_progressed_cycle: u32,
    pub source: AfflictionSource,
}

impl Affliction {
    pub fn key(&self) -> AfflictionKey {
        (self.kind, self.body_part)
    }

    pub fn is_kind_permanent(kind: AfflictionKind) -> bool {
        matches!(
            kind,
            AfflictionKind::MissingLimb | AfflictionKind::Blind | AfflictionKind::Deaf
        )
    }

    pub fn is_permanent(&self) -> bool {
        Self::is_kind_permanent(self.kind)
    }

    pub fn is_reversible(&self) -> bool {
        !self.is_permanent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affliction_key_uses_kind_and_body_part() {
        let a = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::ArmLeft),
            severity: Severity::Mild,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            source: AfflictionSource::Spawn,
        };
        assert_eq!(a.key(), (AfflictionKind::Wounded, Some(BodyPart::ArmLeft)));
    }

    #[test]
    fn permanent_kinds_report_permanent() {
        assert!(Affliction::is_kind_permanent(AfflictionKind::MissingLimb));
        assert!(Affliction::is_kind_permanent(AfflictionKind::Blind));
        assert!(Affliction::is_kind_permanent(AfflictionKind::Deaf));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Wounded));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Broken));
        assert!(!Affliction::is_kind_permanent(AfflictionKind::Infected));
    }

    #[test]
    fn severity_ordering_is_total() {
        assert!(Severity::Mild < Severity::Moderate);
        assert!(Severity::Moderate < Severity::Severe);
    }

    #[test]
    fn btreemap_serialization_is_deterministic() {
        use std::collections::BTreeMap;
        let mut a: BTreeMap<AfflictionKey, Affliction> = BTreeMap::new();
        a.insert(
            (AfflictionKind::Wounded, Some(BodyPart::Torso)),
            Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(BodyPart::Torso),
                severity: Severity::Mild,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                source: AfflictionSource::Spawn,
            },
        );
        a.insert(
            (AfflictionKind::Broken, Some(BodyPart::ArmLeft)),
            Affliction {
                kind: AfflictionKind::Broken,
                body_part: Some(BodyPart::ArmLeft),
                severity: Severity::Moderate,
                acquired_cycle: 2,
                last_progressed_cycle: 2,
                source: AfflictionSource::Spawn,
            },
        );
        let s1 = serde_json::to_string(&a).unwrap();
        let s2 = serde_json::to_string(&a).unwrap();
        assert_eq!(s1, s2, "BTreeMap serialization must be deterministic");
    }
}
```

Modify `shared/src/lib.rs` — add after `pub mod messages;` (line 7):

```rust
pub mod afflictions;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p shared afflictions`
Expected: PASS, 4 tests.

- [ ] **Step 5: Run shared crate clippy**

Run: `cargo clippy -p shared --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(shared): add Affliction types (PR1, lsis)"
```

---

## Task 2: AcquireResolution + can_acquire skeleton

**Files:**
- Create: `game/src/tributes/afflictions/mod.rs`
- Create: `game/src/tributes/afflictions/anatomy.rs`
- Modify: `game/src/tributes/mod.rs` (add `pub mod afflictions;` near other module decls; existing modules listed around line 24-34)
- Test: inline `#[cfg(test)]` in `anatomy.rs`

- [ ] **Step 1: Write the failing test (rstest grid)**

Create `game/src/tributes/afflictions/anatomy.rs`:

```rust
//! Anatomy resolution: how a new affliction interacts with existing slots.
//!
//! See spec §4 (full table) and §17 (testing strategy).

use shared::afflictions::{Affliction, AfflictionKey, AfflictionKind, BodyPart, Severity};
use std::collections::BTreeMap;

/// Outcome of attempting to acquire an affliction given the current tribute state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcquireResolution {
    /// No conflict; insert the new affliction.
    Insert,
    /// Replace an existing slot at the same key with the new (higher) severity.
    Upgrade(AfflictionKey),
    /// Remove subordinate afflictions; insert the new one. Used when
    /// `MissingLimb` arrives and supersedes wound state on that limb.
    Supersede(Vec<AfflictionKey>),
    /// Acquisition is nonsensical (e.g. break a missing bone).
    Reject(RejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// Body part is already missing; the new affliction can't apply.
    LimbAlreadyMissing,
    /// `Infected` requires a `Wounded` ancestor on the same part (no random
    /// whole-body infection in v1; only via cascade).
    InfectedRequiresWoundedAncestor,
    /// New severity is not strictly greater than existing same-key severity.
    NotStrictlyHigherSeverity,
}

/// Decide what happens when `new` is offered to a tribute who already carries
/// `existing` afflictions. Pure function; no mutation. Spec §4.
pub fn can_acquire(existing: &BTreeMap<AfflictionKey, Affliction>, new: &Affliction) -> AcquireResolution {
    let new_key = new.key();

    // Rule: MissingLimb on a part supersedes ALL wound-state slots on that part
    // and rejects subsequent same-part Broken/Wounded/Infected.
    if let Some(part) = new.body_part {
        // 1. Reject if same part is already MissingLimb and new kind is wound-state
        let limb_missing_here = existing.contains_key(&(AfflictionKind::MissingLimb, Some(part)));
        if limb_missing_here
            && matches!(
                new.kind,
                AfflictionKind::Broken | AfflictionKind::Wounded | AfflictionKind::Infected
            )
        {
            return AcquireResolution::Reject(RejectReason::LimbAlreadyMissing);
        }

        // 2. MissingLimb supersedes wound-state on the same part
        if new.kind == AfflictionKind::MissingLimb {
            let supersede: Vec<AfflictionKey> = existing
                .keys()
                .filter(|(k, p)| {
                    p == &Some(part)
                        && matches!(
                            k,
                            AfflictionKind::Broken | AfflictionKind::Wounded | AfflictionKind::Infected
                        )
                })
                .copied()
                .collect();
            if !supersede.is_empty() {
                return AcquireResolution::Supersede(supersede);
            }
            // No conflict; just insert.
            return AcquireResolution::Insert;
        }
    }

    // Rule: Infected requires Wounded ancestor on the same part (or no body part).
    if new.kind == AfflictionKind::Infected
        && !existing.contains_key(&(AfflictionKind::Wounded, new.body_part))
    {
        return AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor);
    }

    // Rule: Same-key collision → upgrade if strictly higher severity.
    if let Some(prev) = existing.get(&new_key) {
        return if new.severity > prev.severity {
            AcquireResolution::Upgrade(new_key)
        } else {
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        };
    }

    // Rule: Blind/Deaf are unique (single slot regardless of body_part).
    // Caller is expected to pass body_part = Some(Eyes) / Some(Ears) for these,
    // so the same-key collision rule above already handles uniqueness.

    AcquireResolution::Insert
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use shared::afflictions::AfflictionSource;

    fn affl(kind: AfflictionKind, part: Option<BodyPart>, sev: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: part,
            severity: sev,
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            source: AfflictionSource::Spawn,
        }
    }

    fn map(items: Vec<Affliction>) -> BTreeMap<AfflictionKey, Affliction> {
        items.into_iter().map(|a| (a.key(), a)).collect()
    }

    #[test]
    fn empty_state_inserts_anything() {
        let r = can_acquire(&map(vec![]), &affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Mild));
        assert_eq!(r, AcquireResolution::Insert);
    }

    #[test]
    fn missing_limb_supersedes_wound_state_on_same_part() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::ArmRight), Severity::Mild),
            affl(AfflictionKind::Broken, Some(BodyPart::ArmRight), Severity::Moderate),
        ]);
        let new = affl(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight), Severity::Severe);
        match can_acquire(&existing, &new) {
            AcquireResolution::Supersede(keys) => {
                assert_eq!(keys.len(), 2);
                assert!(keys.contains(&(AfflictionKind::Wounded, Some(BodyPart::ArmRight))));
                assert!(keys.contains(&(AfflictionKind::Broken, Some(BodyPart::ArmRight))));
            }
            other => panic!("expected Supersede, got {:?}", other),
        }
    }

    #[test]
    fn missing_limb_does_not_affect_other_parts() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::ArmLeft), Severity::Mild),
        ]);
        let new = affl(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight), Severity::Severe);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn breaking_a_missing_bone_is_rejected() {
        let existing = map(vec![
            affl(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight), Severity::Severe),
        ]);
        let new = affl(AfflictionKind::Broken, Some(BodyPart::ArmRight), Severity::Mild);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Reject(RejectReason::LimbAlreadyMissing));
    }

    #[test]
    fn infection_without_wound_ancestor_is_rejected() {
        let new = affl(AfflictionKind::Infected, Some(BodyPart::Torso), Severity::Mild);
        assert_eq!(
            can_acquire(&map(vec![]), &new),
            AcquireResolution::Reject(RejectReason::InfectedRequiresWoundedAncestor)
        );
    }

    #[test]
    fn infection_with_wound_ancestor_inserts() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Severe),
        ]);
        let new = affl(AfflictionKind::Infected, Some(BodyPart::Torso), Severity::Mild);
        assert_eq!(can_acquire(&existing, &new), AcquireResolution::Insert);
    }

    #[test]
    fn same_key_higher_severity_upgrades() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Mild),
        ]);
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Moderate);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Upgrade((AfflictionKind::Wounded, Some(BodyPart::Torso)))
        );
    }

    #[test]
    fn same_key_equal_severity_is_rejected() {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Moderate),
        ]);
        let new = affl(AfflictionKind::Wounded, Some(BodyPart::Torso), Severity::Moderate);
        assert_eq!(
            can_acquire(&existing, &new),
            AcquireResolution::Reject(RejectReason::NotStrictlyHigherSeverity)
        );
    }

    #[rstest]
    #[case(BodyPart::ArmLeft)]
    #[case(BodyPart::ArmRight)]
    #[case(BodyPart::LegLeft)]
    #[case(BodyPart::LegRight)]
    fn missing_limb_works_on_each_limb_part(#[case] part: BodyPart) {
        let existing = map(vec![
            affl(AfflictionKind::Wounded, Some(part), Severity::Mild),
        ]);
        let new = affl(AfflictionKind::MissingLimb, Some(part), Severity::Severe);
        match can_acquire(&existing, &new) {
            AcquireResolution::Supersede(keys) => {
                assert!(keys.contains(&(AfflictionKind::Wounded, Some(part))));
            }
            other => panic!("expected Supersede for {:?}, got {:?}", part, other),
        }
    }
}
```

Create `game/src/tributes/afflictions/mod.rs`:

```rust
//! Game-layer affliction logic: anatomy resolution, acquisition API,
//! tuning. Storage and wire types live in `shared::afflictions`.
//!
//! PR1 ships only the foundation. Cure / cascade / brain-pipeline
//! integration arrive in PR2 and PR3.
//!
//! See `docs/superpowers/specs/2026-05-03-health-conditions-design.md`.

pub mod anatomy;
pub mod tuning;

pub use anatomy::{AcquireResolution, RejectReason, can_acquire};
pub use tuning::AfflictionTuning;
```

Modify `game/src/tributes/mod.rs` — add module declaration. Find the existing module block (around line 24-34, where `pub mod actions;`, `pub mod alliances;`, etc. live) and add alphabetically:

```rust
pub mod afflictions;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p game --lib tributes::afflictions::anatomy 2>&1 | head -30`
Expected: FAIL with "unresolved import `tuning`" — `tuning.rs` doesn't exist yet.

- [ ] **Step 3: Add a minimal tuning stub so the module compiles**

Create `game/src/tributes/afflictions/tuning.rs`:

```rust
//! Placeholder tuning constants for afflictions. Defaults are explicit
//! starting values to be tuned post-observability (spec §5).

/// Tunable knobs for the affliction system. Numbers are placeholders.
/// PR3 wires these into cascade / cure logic; PR1 only defines the shape.
#[derive(Debug, Clone)]
pub struct AfflictionTuning {
    /// Per-cycle probability that an exposed reversible affliction steps up one tier.
    pub progression_chance: f32,
    /// Per-cycle probability that a sheltered reversible affliction steps down one tier.
    pub shelter_recovery_chance: f32,
    /// Per-cycle probability that Severe Wounded spawns Infected.
    pub wound_to_infection_chance: f32,
    /// Per-cycle mortality probability for Severe Infected exposed tributes.
    pub severe_infected_death_chance: f32,
}

impl Default for AfflictionTuning {
    fn default() -> Self {
        Self {
            progression_chance: 0.10,
            shelter_recovery_chance: 0.25,
            wound_to_infection_chance: 0.15,
            severe_infected_death_chance: 0.10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tuning_is_in_unit_range() {
        let t = AfflictionTuning::default();
        for v in [
            t.progression_chance,
            t.shelter_recovery_chance,
            t.wound_to_infection_chance,
            t.severe_infected_death_chance,
        ] {
            assert!((0.0..=1.0).contains(&v), "tuning value {v} out of [0,1]");
        }
    }
}
```

- [ ] **Step 4: Run anatomy tests**

Run: `cargo test -p game --lib tributes::afflictions::anatomy`
Expected: PASS, 9 tests.

- [ ] **Step 5: Run tuning test**

Run: `cargo test -p game --lib tributes::afflictions::tuning`
Expected: PASS, 1 test.

- [ ] **Step 6: Clippy**

Run: `cargo clippy -p game --all-targets -- -D warnings 2>&1 | tail -20`
Expected: clean (or only pre-existing warnings — afflictions module must be clean).

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(game): add affliction anatomy resolution + tuning stub (PR1, lsis)"
```

---

## Task 3: Tribute::afflictions storage field

**Files:**
- Modify: `game/src/tributes/mod.rs` (add field, default init, imports)
- Test: inline `#[cfg(test)]` in `game/src/tributes/mod.rs` (or a new `affliction_storage_tests` submodule)

- [ ] **Step 1: Write the failing test**

Add to `game/src/tributes/mod.rs` test module (find an existing `#[cfg(test)] mod tests {` block; if none nearby, add one at the bottom of the file):

```rust
#[cfg(test)]
mod afflictions_storage_tests {
    use super::*;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, BodyPart, Severity};

    #[test]
    fn default_tribute_has_empty_afflictions() {
        let t = Tribute::default();
        assert!(t.afflictions.is_empty());
    }

    #[test]
    fn empty_afflictions_field_skipped_in_serialization() {
        let t = Tribute::default();
        let s = serde_json::to_string(&t).unwrap();
        assert!(
            !s.contains("\"afflictions\""),
            "empty afflictions should be skipped, got: {s}"
        );
    }

    #[test]
    fn populated_afflictions_field_round_trips() {
        let mut t = Tribute::default();
        let a = Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Torso),
            severity: Severity::Mild,
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            source: AfflictionSource::Spawn,
        };
        t.afflictions.insert(a.key(), a.clone());
        let s = serde_json::to_string(&t).unwrap();
        let back: Tribute = serde_json::from_str(&s).unwrap();
        assert_eq!(back.afflictions.len(), 1);
        assert_eq!(back.afflictions[&a.key()], a);
    }

    #[test]
    fn legacy_tribute_json_without_afflictions_deserializes() {
        // Simulates a row written before this PR — no `afflictions` key at all.
        // Construct via a default tribute serialized with the field stripped.
        let t = Tribute::default();
        let mut v: serde_json::Value = serde_json::to_value(&t).unwrap();
        v.as_object_mut().unwrap().remove("afflictions");
        let back: Tribute = serde_json::from_value(v).unwrap();
        assert!(back.afflictions.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p game --lib tributes::afflictions_storage_tests 2>&1 | head -20`
Expected: FAIL with "no field `afflictions`".

- [ ] **Step 3: Write the minimal implementation**

Modify `game/src/tributes/mod.rs`:

(a) Add import near the top with other `use` lines (look for existing `use std::collections::...` or add near line 30-35):

```rust
use std::collections::BTreeMap;
use shared::afflictions::{Affliction, AfflictionKey};
```

(b) Add field to the `Tribute` struct after the `pub status: TributeStatus,` field (line 163). Match the existing field documentation style:

```rust
    /// Multi-slot, anatomy-aware affliction storage. Each slot is keyed by
    /// `(AfflictionKind, Option<BodyPart>)` so the same kind can appear on
    /// multiple body parts (e.g. `Broken(ArmLeft)` and `Broken(ArmRight)` are
    /// independent slots). `BTreeMap` is used for deterministic iteration
    /// (snapshot-test stability) and serializes as a sorted array of objects.
    ///
    /// Default-empty for backward compatibility with pre-affliction rows;
    /// `skip_serializing_if` keeps unaffected rows compact on the wire.
    ///
    /// See `docs/superpowers/specs/2026-05-03-health-conditions-design.md` §9.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub afflictions: BTreeMap<AfflictionKey, Affliction>,
```

(c) Add field initializer in `Tribute::default()` body (line 282 area). Find the existing struct construction with `status: TributeStatus::default(),` and add:

```rust
            afflictions: BTreeMap::new(),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p game --lib tributes::afflictions_storage_tests`
Expected: PASS, 4 tests.

- [ ] **Step 5: Run all game crate tests to confirm no regressions**

Run: `cargo test -p game --lib 2>&1 | tail -20`
Expected: all existing tests still pass.

- [ ] **Step 6: Commit**

```bash
jj commit -m "feat(game): add Tribute::afflictions BTreeMap storage (PR1, lsis)"
```

---

## Task 4: Tribute::try_acquire_affliction API

**Files:**
- Modify: `game/src/tributes/afflictions/mod.rs` (add `AfflictionDraft`)
- Modify: `game/src/tributes/mod.rs` (add `try_acquire_affliction` method)
- Test: inline tests in `game/src/tributes/afflictions/mod.rs`

- [ ] **Step 1: Write the failing test**

Append to `game/src/tributes/afflictions/mod.rs`:

```rust
/// Caller-supplied request to acquire an affliction. The `acquired_cycle`
/// is set by `Tribute::try_acquire_affliction` from the tribute's current
/// game cycle, not by the caller.
#[derive(Debug, Clone)]
pub struct AfflictionDraft {
    pub kind: shared::afflictions::AfflictionKind,
    pub body_part: Option<shared::afflictions::BodyPart>,
    pub severity: shared::afflictions::Severity,
    pub source: shared::afflictions::AfflictionSource,
}

#[cfg(test)]
mod try_acquire_tests {
    use super::*;
    use crate::tributes::Tribute;
    use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

    fn draft(kind: AfflictionKind, part: Option<BodyPart>, sev: Severity) -> AfflictionDraft {
        AfflictionDraft {
            kind,
            body_part: part,
            severity: sev,
            source: AfflictionSource::Spawn,
        }
    }

    #[test]
    fn try_acquire_inserts_into_empty_storage() {
        let mut t = Tribute::default();
        let r = t.try_acquire_affliction(draft(
            AfflictionKind::Wounded,
            Some(BodyPart::Torso),
            Severity::Mild,
        ));
        assert_eq!(r, AcquireResolution::Insert);
        assert_eq!(t.afflictions.len(), 1);
    }

    #[test]
    fn try_acquire_upgrade_replaces_existing_slot() {
        let mut t = Tribute::default();
        t.try_acquire_affliction(draft(
            AfflictionKind::Wounded,
            Some(BodyPart::Torso),
            Severity::Mild,
        ));
        let r = t.try_acquire_affliction(draft(
            AfflictionKind::Wounded,
            Some(BodyPart::Torso),
            Severity::Severe,
        ));
        assert!(matches!(r, AcquireResolution::Upgrade(_)));
        assert_eq!(t.afflictions.len(), 1);
        let stored = t
            .afflictions
            .get(&(AfflictionKind::Wounded, Some(BodyPart::Torso)))
            .unwrap();
        assert_eq!(stored.severity, Severity::Severe);
    }

    #[test]
    fn try_acquire_supersede_clears_subordinate_slots() {
        let mut t = Tribute::default();
        t.try_acquire_affliction(draft(
            AfflictionKind::Wounded,
            Some(BodyPart::ArmRight),
            Severity::Mild,
        ));
        t.try_acquire_affliction(draft(
            AfflictionKind::Broken,
            Some(BodyPart::ArmRight),
            Severity::Moderate,
        ));
        let r = t.try_acquire_affliction(draft(
            AfflictionKind::MissingLimb,
            Some(BodyPart::ArmRight),
            Severity::Severe,
        ));
        assert!(matches!(r, AcquireResolution::Supersede(_)));
        assert_eq!(t.afflictions.len(), 1);
        assert!(t.afflictions.contains_key(&(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight))));
    }

    #[test]
    fn try_acquire_reject_leaves_storage_unchanged() {
        let mut t = Tribute::default();
        t.try_acquire_affliction(draft(
            AfflictionKind::MissingLimb,
            Some(BodyPart::ArmRight),
            Severity::Severe,
        ));
        let r = t.try_acquire_affliction(draft(
            AfflictionKind::Broken,
            Some(BodyPart::ArmRight),
            Severity::Mild,
        ));
        assert!(matches!(r, AcquireResolution::Reject(RejectReason::LimbAlreadyMissing)));
        assert_eq!(t.afflictions.len(), 1);
        assert!(t.afflictions.contains_key(&(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight))));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p game --lib tributes::afflictions::try_acquire_tests 2>&1 | head -20`
Expected: FAIL with "no method named `try_acquire_affliction`".

- [ ] **Step 3: Write the minimal implementation**

Modify `game/src/tributes/mod.rs` — add the method to the `impl Tribute` block. Find an existing `impl Tribute {` block (the main one starts around line 280) and append (after `Tribute::default()` impl or near other small helpers):

```rust
    /// Attempt to acquire an affliction. Resolves anatomy rules via
    /// `afflictions::can_acquire`, then mutates `self.afflictions`
    /// according to the resolution. The `acquired_cycle` is set from
    /// `self.game_day` (current cycle counter) — callers supply only the
    /// draft, not the timestamp.
    ///
    /// Returns the `AcquireResolution` for caller logging / message emission.
    /// PR1: silent (no messages emitted; PR2 wires `MessagePayload`).
    pub fn try_acquire_affliction(
        &mut self,
        draft: crate::tributes::afflictions::AfflictionDraft,
    ) -> crate::tributes::afflictions::AcquireResolution {
        use shared::afflictions::Affliction as A;
        let cycle = self.game_day.unwrap_or(0) as u32;
        let new = A {
            kind: draft.kind,
            body_part: draft.body_part,
            severity: draft.severity,
            acquired_cycle: cycle,
            last_progressed_cycle: cycle,
            source: draft.source,
        };
        let resolution = crate::tributes::afflictions::can_acquire(&self.afflictions, &new);
        match &resolution {
            crate::tributes::afflictions::AcquireResolution::Insert => {
                self.afflictions.insert(new.key(), new);
            }
            crate::tributes::afflictions::AcquireResolution::Upgrade(key) => {
                self.afflictions.insert(*key, new);
            }
            crate::tributes::afflictions::AcquireResolution::Supersede(keys) => {
                for k in keys {
                    self.afflictions.remove(k);
                }
                self.afflictions.insert(new.key(), new);
            }
            crate::tributes::afflictions::AcquireResolution::Reject(_) => {
                // No mutation.
            }
        }
        resolution
    }
```

**Note:** `self.game_day` field name verification — if the actual field is named differently (e.g. `current_cycle`, `cycle`, etc.), grep first: `grep -n "pub game_day\|pub current_cycle\|pub cycle" game/src/tributes/mod.rs`. If neither exists, default to `0` and add a TODO note for a `cycle: u32` parameter to be threaded by PR2 (replace `self.game_day.unwrap_or(0) as u32` with `0` and add `// TODO(PR2): take cycle parameter`). The test does not depend on the cycle value being meaningful.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p game --lib tributes::afflictions::try_acquire_tests`
Expected: PASS, 4 tests.

- [ ] **Step 5: Run game crate tests**

Run: `cargo test -p game --lib 2>&1 | tail -10`
Expected: no regressions.

- [ ] **Step 6: Clippy**

Run: `cargo clippy -p game --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(game): add Tribute::try_acquire_affliction (PR1, lsis)"
```

---

## Task 5: Parallel-write migration of TributeStatus producers

**Files:**
- Modify: `game/src/tributes/lifecycle.rs` (5 `set_status` sites at lines 221, 225, 227, 228, 229)
- Test: extend `game/src/tributes/lifecycle.rs` test module with parallel-write assertions

The five sites are area-event handlers: Wildfire→Burned, Blizzard→Frozen, Heatwave→Overheated, Sandstorm→Burned, Drought→Overheated. Each gets a parallel `try_acquire_affliction` call with `AfflictionSource::Environmental { event: ... }`.

**Mapping table (TributeStatus → AfflictionKind):**

| TributeStatus | AfflictionKind | BodyPart | Severity (default) |
|---|---|---|---|
| Burned | Burned | None | Moderate |
| Frozen | Frozen | None | Moderate |
| Overheated | Overheated | None | Moderate |
| Wounded | Wounded | None* | Moderate |
| Sick | Sick | None | Moderate |
| Poisoned | Poisoned | None | Moderate |
| Electrocuted | Electrocuted | None | Moderate |
| Broken | Broken | None* | Moderate |
| Infected | Infected | None* | Moderate |
| Mauled | Mauled | None | Moderate |

\* PR1 records body_part as `None` for Wounded/Broken/Infected since the legacy `set_status(TributeStatus::Wounded)` calls don't carry a body part. PR2 (combat integration) will pass actual body parts via attack-result drafts.

Map AreaEvent → AreaEventKind for the source:

| AreaEvent | AreaEventKind | Note |
|---|---|---|
| Wildfire | Fire | direct |
| Blizzard | Storm | extreme cold, no dedicated kind |
| Heatwave | Hazard | no dedicated kind |
| Sandstorm | Storm | wind/sand |
| Drought | Hazard | dehydration |

Verify the actual `AreaEventKind` variants by checking `shared/src/messages.rs:81` (already viewed: `Hazard, Storm, Mutts, Earthquake, Flood, Fire, Other`). Use the closest match.

- [ ] **Step 1: Write the failing test**

Add to `game/src/tributes/lifecycle.rs` test module (find existing `#[cfg(test)] mod tests {` near line 460+ and add):

```rust
    #[test]
    fn area_event_wildfire_writes_to_both_status_and_afflictions() {
        use shared::afflictions::AfflictionKind;
        let mut t = Tribute::default();
        t.handle_area_event(AreaEvent::Wildfire);
        assert_eq!(t.status, TributeStatus::Burned);
        assert!(
            t.afflictions
                .keys()
                .any(|(k, _)| *k == AfflictionKind::Burned),
            "Wildfire should also write to afflictions"
        );
    }

    #[test]
    fn area_event_blizzard_writes_to_both() {
        use shared::afflictions::AfflictionKind;
        let mut t = Tribute::default();
        t.handle_area_event(AreaEvent::Blizzard);
        assert_eq!(t.status, TributeStatus::Frozen);
        assert!(t.afflictions.keys().any(|(k, _)| *k == AfflictionKind::Frozen));
    }

    #[test]
    fn area_event_heatwave_writes_to_both() {
        use shared::afflictions::AfflictionKind;
        let mut t = Tribute::default();
        t.handle_area_event(AreaEvent::Heatwave);
        assert_eq!(t.status, TributeStatus::Overheated);
        assert!(t.afflictions.keys().any(|(k, _)| *k == AfflictionKind::Overheated));
    }

    #[test]
    fn area_event_sandstorm_writes_to_both() {
        use shared::afflictions::AfflictionKind;
        let mut t = Tribute::default();
        t.handle_area_event(AreaEvent::Sandstorm);
        assert_eq!(t.status, TributeStatus::Burned);
        assert!(t.afflictions.keys().any(|(k, _)| *k == AfflictionKind::Burned));
    }

    #[test]
    fn area_event_drought_writes_to_both() {
        use shared::afflictions::AfflictionKind;
        let mut t = Tribute::default();
        t.handle_area_event(AreaEvent::Drought);
        assert_eq!(t.status, TributeStatus::Overheated);
        assert!(t.afflictions.keys().any(|(k, _)| *k == AfflictionKind::Overheated));
    }
```

**Note:** `handle_area_event` is the assumed entry-point method name. Verify with `grep -n "fn handle_area_event\|set_status(TributeStatus::Burned)" game/src/tributes/lifecycle.rs`. The test calls whatever wraps line 221's `self.set_status(TributeStatus::Burned)`. If the method is named differently (e.g. `apply_area_event`, `tick_area_event`), use that.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p game --lib tributes::lifecycle::tests::area_event_wildfire_writes 2>&1 | head -20`
Expected: FAIL — afflictions empty after `set_status(Burned)`.

- [ ] **Step 3: Add parallel writes at the five producer sites**

In `game/src/tributes/lifecycle.rs`, find the `match` block at lines 221-229. The current shape is:

```rust
                AreaEvent::Wildfire => self.set_status(TributeStatus::Burned),
                // ...
                AreaEvent::Blizzard => self.set_status(TributeStatus::Frozen),
                // ...
                AreaEvent::Heatwave => self.set_status(TributeStatus::Overheated),
                AreaEvent::Sandstorm => self.set_status(TributeStatus::Burned),
                AreaEvent::Drought => self.set_status(TributeStatus::Overheated),
```

Convert each arm to a block that does both writes. Replace the entire `match` with:

```rust
                AreaEvent::Wildfire => {
                    self.set_status(TributeStatus::Burned);
                    self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                        kind: shared::afflictions::AfflictionKind::Burned,
                        body_part: None,
                        severity: shared::afflictions::Severity::Moderate,
                        source: shared::afflictions::AfflictionSource::Environmental {
                            event: shared::messages::AreaEventKind::Fire,
                        },
                    });
                }
                AreaEvent::Blizzard => {
                    self.set_status(TributeStatus::Frozen);
                    self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                        kind: shared::afflictions::AfflictionKind::Frozen,
                        body_part: None,
                        severity: shared::afflictions::Severity::Moderate,
                        source: shared::afflictions::AfflictionSource::Environmental {
                            event: shared::messages::AreaEventKind::Storm,
                        },
                    });
                }
                AreaEvent::Heatwave => {
                    self.set_status(TributeStatus::Overheated);
                    self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                        kind: shared::afflictions::AfflictionKind::Overheated,
                        body_part: None,
                        severity: shared::afflictions::Severity::Moderate,
                        source: shared::afflictions::AfflictionSource::Environmental {
                            event: shared::messages::AreaEventKind::Hazard,
                        },
                    });
                }
                AreaEvent::Sandstorm => {
                    self.set_status(TributeStatus::Burned);
                    self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                        kind: shared::afflictions::AfflictionKind::Burned,
                        body_part: None,
                        severity: shared::afflictions::Severity::Moderate,
                        source: shared::afflictions::AfflictionSource::Environmental {
                            event: shared::messages::AreaEventKind::Storm,
                        },
                    });
                }
                AreaEvent::Drought => {
                    self.set_status(TributeStatus::Overheated);
                    self.try_acquire_affliction(crate::tributes::afflictions::AfflictionDraft {
                        kind: shared::afflictions::AfflictionKind::Overheated,
                        body_part: None,
                        severity: shared::afflictions::Severity::Moderate,
                        source: shared::afflictions::AfflictionSource::Environmental {
                            event: shared::messages::AreaEventKind::Hazard,
                        },
                    });
                }
```

Preserve all surrounding `match` arms and other variants exactly as they are — only the five listed lines (221, 225, 227, 228, 229) are converted.

- [ ] **Step 4: Run tests**

Run: `cargo test -p game --lib tributes::lifecycle::tests::area_event 2>&1 | tail -20`
Expected: PASS, 5 new tests; existing lifecycle tests still pass.

- [ ] **Step 5: Run full game crate tests**

Run: `cargo test -p game --lib 2>&1 | tail -10`
Expected: no regressions.

- [ ] **Step 6: Clippy**

Run: `cargo clippy -p game --all-targets -- -D warnings 2>&1 | tail -10`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
jj commit -m "feat(game): parallel-write afflictions from area-event handlers (PR1, lsis)"
```

---

## Task 6: Proptest invariants

**Files:**
- Modify: `game/Cargo.toml` (add `proptest` to `[dev-dependencies]` if not present — check first)
- Create or modify: `game/src/tributes/afflictions/anatomy.rs` (add `proptest!` block at end of `#[cfg(test)] mod tests`)

- [ ] **Step 1: Verify proptest dependency**

Run: `grep -n "proptest" game/Cargo.toml`
Expected: present in `[dev-dependencies]`. If absent, add:

```toml
[dev-dependencies]
proptest = "1"
```

- [ ] **Step 2: Write the failing test (proptest)**

Append to the `#[cfg(test)] mod tests` block in `game/src/tributes/afflictions/anatomy.rs`:

```rust
    use proptest::prelude::*;

    fn arb_kind() -> impl Strategy<Value = AfflictionKind> {
        prop_oneof![
            Just(AfflictionKind::Wounded),
            Just(AfflictionKind::Broken),
            Just(AfflictionKind::Infected),
            Just(AfflictionKind::MissingLimb),
            Just(AfflictionKind::Blind),
            Just(AfflictionKind::Deaf),
            Just(AfflictionKind::Sick),
            Just(AfflictionKind::Poisoned),
            Just(AfflictionKind::Burned),
            Just(AfflictionKind::Frozen),
            Just(AfflictionKind::Overheated),
            Just(AfflictionKind::Electrocuted),
            Just(AfflictionKind::Mauled),
        ]
    }

    fn arb_body_part() -> impl Strategy<Value = Option<BodyPart>> {
        prop_oneof![
            Just(None),
            Just(Some(BodyPart::ArmLeft)),
            Just(Some(BodyPart::ArmRight)),
            Just(Some(BodyPart::LegLeft)),
            Just(Some(BodyPart::LegRight)),
            Just(Some(BodyPart::Torso)),
            Just(Some(BodyPart::Head)),
            Just(Some(BodyPart::Eyes)),
            Just(Some(BodyPart::Ears)),
        ]
    }

    fn arb_severity() -> impl Strategy<Value = Severity> {
        prop_oneof![Just(Severity::Mild), Just(Severity::Moderate), Just(Severity::Severe)]
    }

    fn arb_affliction() -> impl Strategy<Value = Affliction> {
        (arb_kind(), arb_body_part(), arb_severity()).prop_map(|(k, p, s)| affl(k, p, s))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn can_acquire_is_deterministic(
            existing in proptest::collection::vec(arb_affliction(), 0..6),
            new in arb_affliction(),
        ) {
            let map_existing = map(existing);
            let r1 = can_acquire(&map_existing, &new);
            let r2 = can_acquire(&map_existing, &new);
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn missing_limb_and_broken_never_coexist_after_resolution(
            existing in proptest::collection::vec(arb_affliction(), 0..6),
            new in arb_affliction(),
        ) {
            let mut state = map(existing);
            let resolution = can_acquire(&state, &new);
            match resolution {
                AcquireResolution::Insert | AcquireResolution::Upgrade(_) => {
                    state.insert(new.key(), new.clone());
                }
                AcquireResolution::Supersede(keys) => {
                    for k in &keys { state.remove(k); }
                    state.insert(new.key(), new.clone());
                }
                AcquireResolution::Reject(_) => {}
            }
            // Invariant: no part has both MissingLimb and Broken simultaneously.
            for part in [BodyPart::ArmLeft, BodyPart::ArmRight, BodyPart::LegLeft, BodyPart::LegRight] {
                let has_missing = state.contains_key(&(AfflictionKind::MissingLimb, Some(part)));
                let has_broken = state.contains_key(&(AfflictionKind::Broken, Some(part)));
                prop_assert!(
                    !(has_missing && has_broken),
                    "violation: part {:?} has both MissingLimb and Broken",
                    part
                );
            }
        }

        #[test]
        fn missing_limb_and_wound_state_never_coexist_after_resolution(
            existing in proptest::collection::vec(arb_affliction(), 0..6),
            new in arb_affliction(),
        ) {
            let mut state = map(existing);
            let resolution = can_acquire(&state, &new);
            match resolution {
                AcquireResolution::Insert | AcquireResolution::Upgrade(_) => {
                    state.insert(new.key(), new.clone());
                }
                AcquireResolution::Supersede(keys) => {
                    for k in &keys { state.remove(k); }
                    state.insert(new.key(), new.clone());
                }
                AcquireResolution::Reject(_) => {}
            }
            for part in [BodyPart::ArmLeft, BodyPart::ArmRight, BodyPart::LegLeft, BodyPart::LegRight] {
                if state.contains_key(&(AfflictionKind::MissingLimb, Some(part))) {
                    for kind in [AfflictionKind::Wounded, AfflictionKind::Infected, AfflictionKind::Broken] {
                        prop_assert!(
                            !state.contains_key(&(kind, Some(part))),
                            "violation: part {:?} has MissingLimb + {:?}",
                            part, kind
                        );
                    }
                }
            }
        }
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p game --lib tributes::afflictions::anatomy 2>&1 | tail -10`
Expected: PASS — all unit tests + 3 proptest cases × 256 iterations each.

If proptest fails with a counterexample, the resolution table has a bug. Debug by printing the counterexample input and tracing through `can_acquire` — fix the rule, do not loosen the invariant.

- [ ] **Step 4: Commit**

```bash
jj commit -m "test(game): proptest invariants for affliction anatomy resolution (PR1, lsis)"
```

---

## Task 7: Insta snapshot baseline

**Files:**
- Modify: `game/Cargo.toml` (verify `insta` in `[dev-dependencies]`)
- Create: `game/src/tributes/afflictions/snapshot_tests.rs` (or inline submodule)
- Modify: `game/src/tributes/afflictions/mod.rs` (declare `#[cfg(test)] mod snapshot_tests;`)

- [ ] **Step 1: Verify insta dependency**

Run: `grep -n "insta" game/Cargo.toml`
Expected: present. If absent, add:

```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml", "json"] }
```

- [ ] **Step 2: Write the snapshot test**

Create `game/src/tributes/afflictions/snapshot_tests.rs`:

```rust
//! Snapshot test for canonical affliction storage shapes. Locks the
//! BTreeMap serialization layout so future PRs don't change the wire
//! format silently.

use crate::tributes::Tribute;
use crate::tributes::afflictions::AfflictionDraft;
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

#[test]
fn canonical_mixed_afflictions_serialize_stably() {
    let mut t = Tribute::default();
    // Stable name for snapshot redaction. The Tribute::default() id is a
    // random UUID; redact the volatile fields below.
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Torso),
        severity: Severity::Mild,
        source: AfflictionSource::Spawn,
    });
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Broken,
        body_part: Some(BodyPart::ArmLeft),
        severity: Severity::Moderate,
        source: AfflictionSource::Spawn,
    });
    // Insert a Wounded(Torso) ancestor so Infected acquisition succeeds.
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Infected,
        body_part: Some(BodyPart::Torso),
        severity: Severity::Mild,
        source: AfflictionSource::Cascade {
            from: (AfflictionKind::Wounded, Some(BodyPart::Torso)),
        },
    });

    let json = serde_json::to_value(&t.afflictions).unwrap();
    insta::assert_json_snapshot!("canonical_mixed_afflictions", json);
}
```

Add to `game/src/tributes/afflictions/mod.rs` (bottom):

```rust
#[cfg(test)]
mod snapshot_tests;
```

- [ ] **Step 3: Run the snapshot test (creates baseline)**

Run: `cargo test -p game --lib tributes::afflictions::snapshot_tests 2>&1 | tail -10`
Expected: FAIL on first run (snapshot missing), then accept the new snapshot:

```bash
INSTA_UPDATE=auto cargo test -p game --lib tributes::afflictions::snapshot_tests
```

- [ ] **Step 4: Verify snapshot file exists and is sane**

Run: `ls game/src/tributes/snapshots/ 2>&1 | head -20`
Expected: contains `mod__tributes__afflictions__snapshot_tests__canonical_mixed_afflictions.snap` (or similar — exact path depends on insta config).

Open the snapshot and confirm the BTreeMap is sorted by key (Broken < Infected < Wounded alphabetically? — actually by enum discriminant order: Wounded=0, Broken=1, Infected=2, so iteration order is Wounded, Broken, Infected — but `BTreeMap` orders by `Ord` derived on the tuple, so the actual order depends on the variant order in `AfflictionKind`. Verify it's deterministic by running twice).

Run: `cargo test -p game --lib tributes::afflictions::snapshot_tests` (second time)
Expected: PASS (snapshot now matches).

- [ ] **Step 5: Commit (snapshot file + test)**

```bash
jj commit -m "test(game): insta snapshot baseline for affliction storage (PR1, lsis)"
```

---

## Task 8: SurrealDB schema migration

**Files:**
- Modify: `schemas/tribute.surql` (add `afflictions` field DEFINE)
- Create: `migrations/definitions/20260504_010000_TributeAfflictions.json` (additive migration)

The existing pattern: each migration JSON file has the shape of `{ "patches": [ ... ] }`. Check an existing additive migration for the exact shape — `migrations/definitions/20260503_120000_TributeSurvivalFields.json` is the most recent precedent.

- [ ] **Step 1: Inspect the precedent migration**

Run: `cat migrations/definitions/20260503_120000_TributeSurvivalFields.json`

Capture the file shape (will look something like a JSON array/object describing schema patches).

- [ ] **Step 2: Add the schema field**

Modify `schemas/tribute.surql` — add after the `status` field DEFINE (line 7):

```surql
DEFINE FIELD OVERWRITE afflictions ON tribute TYPE option<array<object>>;
```

- [ ] **Step 3: Create the migration definition**

Create `migrations/definitions/20260504_010000_TributeAfflictions.json` matching the shape and patch syntax used by `20260503_120000_TributeSurvivalFields.json`. The migration adds the new optional `afflictions` field to the `tribute` table; default value when absent is the empty array (handled by `#[serde(default)]` on the Rust side, so SurrealDB can leave the field unset and reads will populate `BTreeMap::new()`).

Concrete JSON content depends on the precedent's exact syntax. If the precedent uses raw `.surql` patches as strings, the patch is:

```surql
DEFINE FIELD OVERWRITE afflictions ON tribute TYPE option<array<object>>;
```

If a `_initial.json` regeneration is required (per `hangrier_games-9579`-style precedent), do NOT regenerate it in this PR — that's a separate concern. This PR only adds an additive patch on top of the existing initial.

- [ ] **Step 4: Test the migration locally**

Run: `surreal start --user root --pass root memory &` (or equivalent — check `justfile` for the test DB target)

Then run a quick API smoke test if one exists:

Run: `cargo test -p api --test '*' 2>&1 | tail -10` (skip if API tests are CI-gated per `hangrier_games-yj9u`).

Expected: schema migrates cleanly; no regressions.

If running surrealdb locally is too heavy for plan execution, skip the live migration test and verify only that the JSON file is valid:

Run: `cat migrations/definitions/20260504_010000_TributeAfflictions.json | python3 -m json.tool > /dev/null && echo OK`
Expected: `OK`.

- [ ] **Step 5: Commit**

```bash
jj commit -m "feat(api): add tribute.afflictions schema migration (PR1, lsis)"
```

---

## Task 9: Integration test (game/tests/)

**Files:**
- Create: `game/tests/afflictions_storage_test.rs`

- [ ] **Step 1: Write the failing test**

Create `game/tests/afflictions_storage_test.rs`:

```rust
//! Integration test for PR1 affliction storage and acquisition.
//! Verifies the public API surface from the consumer's perspective.

use game::tributes::Tribute;
use game::tributes::afflictions::{AcquireResolution, AfflictionDraft, RejectReason};
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

#[test]
fn full_acquisition_flow_insert_upgrade_supersede_reject() {
    let mut t = Tribute::default();

    // Insert
    let r = t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::ArmRight),
        severity: Severity::Mild,
        source: AfflictionSource::Spawn,
    });
    assert_eq!(r, AcquireResolution::Insert);

    // Upgrade (same key, higher severity)
    let r = t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::ArmRight),
        severity: Severity::Severe,
        source: AfflictionSource::Spawn,
    });
    assert!(matches!(r, AcquireResolution::Upgrade(_)));
    assert_eq!(t.afflictions.len(), 1);
    assert_eq!(
        t.afflictions[&(AfflictionKind::Wounded, Some(BodyPart::ArmRight))].severity,
        Severity::Severe
    );

    // Supersede (MissingLimb removes wound state on same part)
    let r = t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::MissingLimb,
        body_part: Some(BodyPart::ArmRight),
        severity: Severity::Severe,
        source: AfflictionSource::Spawn,
    });
    assert!(matches!(r, AcquireResolution::Supersede(_)));
    assert_eq!(t.afflictions.len(), 1);
    assert!(t.afflictions.contains_key(&(AfflictionKind::MissingLimb, Some(BodyPart::ArmRight))));

    // Reject (can't break a missing bone)
    let r = t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Broken,
        body_part: Some(BodyPart::ArmRight),
        severity: Severity::Mild,
        source: AfflictionSource::Spawn,
    });
    assert_eq!(r, AcquireResolution::Reject(RejectReason::LimbAlreadyMissing));
    assert_eq!(t.afflictions.len(), 1);
}

#[test]
fn afflictions_round_trip_through_serde() {
    let mut t = Tribute::default();
    t.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Burned,
        body_part: None,
        severity: Severity::Moderate,
        source: AfflictionSource::Environmental {
            event: shared::messages::AreaEventKind::Fire,
        },
    });
    let s = serde_json::to_string(&t).unwrap();
    let back: Tribute = serde_json::from_str(&s).unwrap();
    assert_eq!(back.afflictions.len(), 1);
    assert!(back.afflictions.contains_key(&(AfflictionKind::Burned, None)));
}
```

- [ ] **Step 2: Run the integration test**

Run: `cargo test -p game --test afflictions_storage_test 2>&1 | tail -10`
Expected: PASS, 2 tests.

- [ ] **Step 3: Commit**

```bash
jj commit -m "test(game): integration test for affliction acquisition flow (PR1, lsis)"
```

---

## Task 10: Quality gates + finalize

- [ ] **Step 1: Run the workspace quality suite**

Run: `just quality 2>&1 | tail -30` (this runs format, check, clippy, test per the project's `justfile`)
Expected: clean across the workspace. If `just test` is slow, run `cargo test -p game --lib && cargo test -p shared` instead and note the deferral.

If clippy flags warnings in pre-existing code outside `game/src/tributes/afflictions/` and `shared/src/afflictions.rs`, do NOT fix them in this PR — file a follow-up bead. Only block on warnings inside the new code.

- [ ] **Step 2: Verify nothing in PR2/PR3/PR4 territory was touched**

Run: `jj diff --stat 2>&1 | head -30`
Expected: changes only to:
- `shared/src/lib.rs`, `shared/src/afflictions.rs`
- `game/src/tributes/mod.rs`, `game/src/tributes/lifecycle.rs`
- `game/src/tributes/afflictions/{mod,anatomy,tuning,snapshot_tests}.rs`
- `game/tests/afflictions_storage_test.rs`
- `game/src/tributes/snapshots/*.snap` (new)
- `schemas/tribute.surql`
- `migrations/definitions/20260504_010000_TributeAfflictions.json`
- (possibly `game/Cargo.toml` if proptest/insta were added)

If anything else changed (e.g. `messages.rs`, `combat.rs`, `brains.rs`), back it out — those are PR2 territory.

- [ ] **Step 3: Update beads**

```bash
bd close hangrier_games-lsis
bd export -o .beads/issues.jsonl
```

- [ ] **Step 4: Push the branch and open the PR**

```bash
jj git fetch
jj rebase -d main@origin
jj bookmark create afflictions-pr1 -r @-
jj git push --bookmark afflictions-pr1
gh pr create --base main --head afflictions-pr1 \
  --title "feat(afflictions): PR1 — types, storage, anatomy resolution, parallel-write migration (lsis)" \
  --body "$(cat <<'EOF'
## Summary

Foundation for the afflictions / phobias / fixations trilogy. Lands the wire-visible types, multi-slot storage, anatomy resolution table, and parallel-write migration of five area-event producers per spec §18 steps 1-2.

## Changes

- \`shared::afflictions\` — \`AfflictionKind\`, \`BodyPart\`, \`Severity\`, \`AfflictionKey\`, \`AfflictionSource\`, \`Affliction\`
- \`Tribute::afflictions: BTreeMap<AfflictionKey, Affliction>\` (default-empty, skip-if-empty serde)
- \`game::tributes::afflictions\` — \`can_acquire\`, \`AcquireResolution\`, \`AfflictionDraft\`, \`AfflictionTuning\` placeholders
- \`Tribute::try_acquire_affliction\` API
- Five \`set_status\` sites in \`lifecycle.rs\` now also write to \`afflictions\` (parallel; legacy effects unchanged)
- SurrealDB schema migration: \`tribute.afflictions: option<array<object>>\`

## Out of scope (later PRs)

- Brain pipeline \`affliction_override\` layer (PR2 / \`dyom\`)
- Combat inflict tables (PR2)
- Cure / cascade / shelter logic (PR3 / \`370g\`)
- \`MessagePayload::Affliction*\` variants (PR2)
- Frontend (PR4 / \`kcdl\`)
- Deletion of migrated \`TributeStatus\` variants (deferred until consumers migrate)

## Verification

- \`cargo test -p shared afflictions\` — 4 tests pass
- \`cargo test -p game --lib tributes::afflictions\` — unit + proptest pass
- \`cargo test -p game --test afflictions_storage_test\` — integration pass
- \`cargo test -p game --lib tributes::lifecycle\` — area-event parallel-write pass
- \`just quality\` — workspace clean
EOF
)"
```

- [ ] **Step 5: Verify PR opened**

Confirm a PR URL is in hand. Hand off the PR URL plus a one-line summary for the next session.

---

## Self-Review

Checked the plan against the spec section by section:

**1. Spec coverage:**
- §9 Data shape — Task 1 (shared types), Task 3 (storage field) ✓
- §4 Stacking rules — Task 2 (`can_acquire` table) ✓
- §5 Severity tiers — `Severity` enum in Task 1 ✓ (cascade logic deferred to PR3 per spec §18)
- §17 Testing — Tasks 1, 2, 4, 6, 7, 9 (rstest + proptest + insta) ✓
- §18 Migration plan steps 1-2 — Tasks 3, 5 ✓
- §18 step 3-5 — explicitly out of scope (PR2/PR3) ✓

**2. Placeholder scan:** no TBDs, no "implement later," no vague steps. The `tuning.rs` defaults are explicit numbers (per spec they ARE placeholders awaiting balancing — but they're concrete defaults, not unfilled gaps).

**3. Type consistency:**
- `AfflictionDraft` introduced in Task 4, used in Tasks 5, 7, 9 — same shape (kind, body_part, severity, source) ✓
- `AcquireResolution` variants (Insert, Upgrade(key), Supersede(keys), Reject(reason)) consistent across Tasks 2, 4, 9 ✓
- `try_acquire_affliction` signature `(&mut self, AfflictionDraft) -> AcquireResolution` consistent ✓
- `AfflictionSource::Environmental { event: AreaEventKind }` shape consistent in Tasks 1, 5 ✓

**4. One known unknown:** `Tribute::game_day` field name verification (Task 4 Step 3 includes a fallback branch with grep instructions). The plan handles this gracefully — implementation falls back to `0` with a `TODO(PR2)` comment if the field doesn't exist. This is the only place implementation needs to inspect-and-decide; everything else is fully concrete.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-05-04-afflictions-pr1.md`. Two execution options:

**1. Subagent-Driven (recommended)** — dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — execute tasks in this session using executing-plans, batch execution with checkpoints

Per your "brainstorm only this session" directive: neither runs now. Plan is committed and ready for the next session.
