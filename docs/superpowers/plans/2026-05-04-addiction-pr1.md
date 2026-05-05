# Addiction PR1 — Types, Storage, `try_acquire_addiction` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the addiction type system, storage on `Affliction` and `Tribute`, the `high_duration` table, and the `try_acquire_addiction` acquisition helper that enforces the probabilistic acquisition curve, substance multipliers, cap-at-2, relapse short-circuit, and single-Addiction-per-substance reinforcement. No use-pipeline integration, no brain layer, no producers, no messages — just the contract surface that PR2/PR3 will build on.

**Architecture:** Adds `AfflictionKind::Addiction(Substance)` variant + `Substance` + `AddictionResistReason` enums to `shared/src/afflictions.rs`. Adds `AddictionMetadata` struct and `Option<AddictionMetadata>` field to `Affliction` (the metadata extension slot — required because addiction carries mutable runtime state, including `cycles_since_last_use`, `high_cycles_remaining`, and observer state). Adds `addiction_use_count: BTreeMap<Substance, u32>` field to `Tribute` (persistent across cure for relapse semantics). Adds `high_duration` table function. Adds a `try_acquire_addiction` method on `Tribute` that wraps `try_acquire_affliction` with the probabilistic curve + multiplier + cap-2 + relapse + reinforcement logic. Pure-type test surface; no producers run yet.

**Tech Stack:** Rust 2024 edition, `serde`, `rstest`, `insta` (yaml), `proptest` (256 cases), SurrealDB schema additions.

---

## Hard prerequisites

- **`hangrier_games-lsis`** (afflictions PR1) MUST be landed and merged before this plan begins. This plan reads/modifies types defined there: `AfflictionKind`, `Severity`, `Affliction`, `AfflictionSource`, `AfflictionDraft`, `AcquireResolution`, `try_acquire_affliction`, `Tribute.afflictions`. If `lsis` has not landed, stop and surface that.
- Verify before starting:
  ```bash
  grep -n "pub enum AfflictionKind" shared/src/afflictions.rs
  grep -n "pub fn try_acquire_affliction" game/src/tributes/mod.rs
  ```
  Each command should print exactly one match.
- **Soft prerequisite (informational):** Trauma PR1 (`hangrier_games-u1fa`) and Phobia PR1 may have already added `trauma_metadata` / `phobia_metadata` `Option` fields to `Affliction` using the same backward-compat pattern this plan uses. If so, mirror their `#[serde(default, skip_serializing_if = "...")]` attribute style exactly. If they have not landed, this plan is the first to introduce the metadata-extension pattern after `lsis`; either order is fine.

## Spec reference

Spec: `docs/superpowers/specs/2026-05-04-addiction-design.md`. This plan implements §4 (types) and §5.1/§5.2/§5.3 (acquisition: probabilistic curve, substance multipliers, cap-at-2, relapse short-circuit, single-substance reinforcement). It does NOT implement: §5 use-pipeline producer hook into `try_use_consumable` (that is PR2), §6 reinforcement/decay tick (that is PR3, sharing the helper extracted by phobia PR3 / trauma PR3 / fixation PR2), §7 effects (High vs Withdrawal), §8 brain layer, §9 visibility, §10 messages, §11 alliance, §12 UI.

## File structure

**Created:**
- `game/src/tributes/afflictions/addiction.rs` — `try_acquire_addiction` helper + `AddictionAcquisition` outcome enum + `acquisition_chance` pure function + `high_duration` table function
- `game/src/tributes/afflictions/addiction_tests.rs` — unit tests (rstest + proptest)
- `game/tests/addiction_acquisition_test.rs` — integration test using `Tribute` end-to-end
- `migrations/definitions/20260504_030000_TributeAfflictions_AddictionMetadata.json` — schema migration

**Modified:**
- `shared/src/afflictions.rs` — adds `Addiction(Substance)` variant to `AfflictionKind`, adds `Substance`, `AddictionResistReason`, `AddictionMetadata`, adds `addiction_metadata: Option<AddictionMetadata>` field to `Affliction`
- `game/src/tributes/mod.rs` — adds `addiction_use_count: BTreeMap<Substance, u32>` field to `Tribute`, adds `try_acquire_addiction` method, adds `addiction_afflictions()` accessor
- `game/src/tributes/afflictions/mod.rs` — re-exports `try_acquire_addiction`, `AddictionAcquisition`, `acquisition_chance`, `high_duration`
- `game/src/tributes/afflictions/anatomy.rs` — extends `can_acquire` to handle the `Addiction(_)` kind (single-substance + cap-2 rules)
- `schemas/tribute.surql` — flexible field already covers it; add a comment documenting addiction metadata + use_count shape

---

## Pre-flight verification

- [ ] **Step 0.1: Confirm afflictions PR1 has landed**

Run:
```bash
grep -n "pub enum AfflictionKind" shared/src/afflictions.rs
grep -n "pub fn try_acquire_affliction" game/src/tributes/mod.rs
grep -n "pub afflictions:" game/src/tributes/mod.rs
```

Expected: each command prints exactly one match. If any print zero matches, stop.

- [ ] **Step 0.2: Note prior metadata-extension fields (informational)**

Run:
```bash
grep -n "trauma_metadata\|phobia_metadata\|fixation_metadata" shared/src/afflictions.rs
```

Record any fields that exist. If they exist, this plan's `addiction_metadata` field MUST follow the identical `#[serde(default, skip_serializing_if = "Option::is_none")]` attribute pattern. If none exist, this plan is establishing the pattern.

- [ ] **Step 0.3: Verify clean working tree**

Run: `jj status`. Working copy should be clean or have a fresh empty change ready.

- [ ] **Step 0.4: Create a working bookmark**

Run:
```bash
jj new main@origin -m "addiction PR1: WIP"
jj bookmark create addiction-pr1 -r @
```

---

## Task 1: Add `Substance` enum to `shared/src/afflictions.rs`

Pure data type referenced by `AfflictionKind::Addiction`, `AddictionMetadata`, and the `addiction_use_count` map. Must land before the variant + metadata + map types because they all embed `Substance`.

**Files:**
- Modify: `shared/src/afflictions.rs` (append below existing types, above `AfflictionKind`)

- [ ] **Step 1.1: Write failing unit tests for `Substance` round-trip + ordering**

In `shared/src/afflictions.rs`, append at the bottom (or extend the existing `#[cfg(test)] mod tests`):

```rust
#[cfg(test)]
mod substance_tests {
    use super::*;

    #[test]
    fn substance_serializes_snake_case_unit() {
        assert_eq!(serde_json::to_string(&Substance::Stimulant).unwrap(), r#""stimulant""#);
        assert_eq!(serde_json::to_string(&Substance::Morphling).unwrap(), r#""morphling""#);
        assert_eq!(serde_json::to_string(&Substance::Alcohol).unwrap(), r#""alcohol""#);
        assert_eq!(serde_json::to_string(&Substance::Painkiller).unwrap(), r#""painkiller""#);
    }

    #[test]
    fn substance_round_trips() {
        for s in [Substance::Stimulant, Substance::Morphling, Substance::Alcohol, Substance::Painkiller] {
            let j = serde_json::to_string(&s).unwrap();
            let back: Substance = serde_json::from_str(&j).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn substance_ord_is_stable() {
        // Used as BTreeMap key; ordering must be deterministic.
        let mut v = vec![Substance::Painkiller, Substance::Stimulant, Substance::Morphling, Substance::Alcohol];
        v.sort();
        // Discriminant order: Stimulant=0, Morphling=1, Alcohol=2, Painkiller=3.
        assert_eq!(v, vec![Substance::Stimulant, Substance::Morphling, Substance::Alcohol, Substance::Painkiller]);
    }

    #[test]
    fn substance_is_copy() {
        let s = Substance::Stimulant;
        let _a = s;
        let _b = s; // Compile error if not Copy.
    }
}
```

Run: `cargo test --package shared substance_tests`. Expected: all four tests fail to compile (`Substance` does not exist).

- [ ] **Step 1.2: Add the `Substance` enum**

In `shared/src/afflictions.rs`, add above `AfflictionKind`:

```rust
/// A substance class that can produce addiction.
///
/// v1 has four classes; only `Stimulant` is reachable through normal play
/// (yayo / go-juice / adrenaline). The other three ship via the sponsorship
/// system (`hangrier_games-dvd`).
///
/// Discriminant order is the canonical sort order for `BTreeMap<Substance, _>`
/// keys. Do not reorder variants without a migration plan for the use-count map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Substance {
    Stimulant,
    Morphling,
    Alcohol,
    Painkiller,
}

impl Substance {
    pub const ALL: [Substance; 4] = [
        Substance::Stimulant,
        Substance::Morphling,
        Substance::Alcohol,
        Substance::Painkiller,
    ];
}
```

Run: `cargo test --package shared substance_tests`. Expected: all four tests pass.

- [ ] **Step 1.3: Verify**

Run: `cargo build --package shared`. Expected: clean build, no warnings.

---

## Task 2: Add `AddictionResistReason` enum to `shared/src/afflictions.rs`

Lightweight enum used by `AddictionAcquisition::Resisted` (Task 7) and the future `MessagePayload::AddictionResisted` (PR2). Lands here so the acquisition outcome enum can reference it.

**Files:**
- Modify: `shared/src/afflictions.rs` (append after `Substance`)

- [ ] **Step 2.1: Write failing test**

Append to `shared/src/afflictions.rs`:

```rust
#[cfg(test)]
mod addiction_resist_reason_tests {
    use super::*;

    #[test]
    fn at_cap_serializes_snake_case() {
        let r = AddictionResistReason::AtCap;
        assert_eq!(serde_json::to_string(&r).unwrap(), r#""at_cap""#);
    }
}
```

Run: `cargo test --package shared addiction_resist_reason_tests`. Expected: fail to compile.

- [ ] **Step 2.2: Add the enum**

```rust
/// Why an addiction acquisition was resisted (state did not change despite
/// the substance being consumed). The substance's immediate effect still
/// applied to the tribute; only the stored Addiction state was suppressed.
///
/// Future variants (left as enum for extensibility): `TraitResistance`,
/// `EquippedTalisman`, `RecentDetox`. Do not reorder variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddictionResistReason {
    AtCap,
}
```

Run: test passes. Run: `cargo build --package shared`. Clean.

---

## Task 3: Add `AddictionMetadata` struct to `shared/src/afflictions.rs`

The runtime-mutable metadata stored on `Affliction` when the kind is `Addiction(_)`. Carries the High/Withdrawal counters, observer state, and the use-count snapshot at acquisition (for relapse messaging).

**Files:**
- Modify: `shared/src/afflictions.rs` (append after `AddictionResistReason`)

- [ ] **Step 3.1: Write failing test**

```rust
#[cfg(test)]
mod addiction_metadata_tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn sample() -> AddictionMetadata {
        AddictionMetadata {
            substance: Substance::Stimulant,
            cycles_since_last_use: 0,
            high_cycles_remaining: 2,
            use_count_at_acquisition: 3,
            observed_by: BTreeSet::new(),
            observer_seen_cycle: BTreeMap::new(),
        }
    }

    #[test]
    fn round_trips() {
        let m = sample();
        let j = serde_json::to_string(&m).unwrap();
        let back: AddictionMetadata = serde_json::from_str(&j).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn empty_observer_state_omitted_in_json() {
        let m = sample();
        let j = serde_json::to_string(&m).unwrap();
        assert!(!j.contains("observed_by"), "expected empty observed_by omitted: {j}");
        assert!(!j.contains("observer_seen_cycle"), "expected empty observer_seen_cycle omitted: {j}");
    }

    #[test]
    fn populated_observer_state_round_trips() {
        let mut m = sample();
        m.observed_by.insert("tributes:cato".into());
        m.observer_seen_cycle.insert("tributes:cato".into(), 12);
        let j = serde_json::to_string(&m).unwrap();
        let back: AddictionMetadata = serde_json::from_str(&j).unwrap();
        assert_eq!(back, m);
    }
}
```

Run: `cargo test --package shared addiction_metadata_tests`. Expected: fail to compile.

- [ ] **Step 3.2: Add the struct**

```rust
/// Runtime-mutable state for an `Addiction(_)` affliction.
///
/// Stored as `Affliction.addiction_metadata` (an `Option`); `None` for all
/// non-Addiction kinds. The `substance` field duplicates the `AfflictionKind`
/// payload for ergonomic access (e.g. `meta.substance` in places that already
/// hold `&AddictionMetadata` without the parent `Affliction`).
///
/// `cycles_since_last_use` drives decay (§6.2 of the addiction spec) and the
/// High → Withdrawal transition. `high_cycles_remaining` counts down each
/// cycle; 0 = Withdrawal mode active.
///
/// `use_count_at_acquisition` is a snapshot used for the `AddictionRelapse`
/// payload (PR2 / PR3) — it lets the message carry "this is your 4th time
/// going through this" without re-deriving from the persistent map.
///
/// `observed_by` and `observer_seen_cycle` mirror trauma's observer pattern;
/// see §9 of the spec. Empty in this PR (no producers run yet).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddictionMetadata {
    pub substance: Substance,
    pub cycles_since_last_use: u32,
    pub high_cycles_remaining: u32,
    pub use_count_at_acquisition: u32,
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub observed_by: std::collections::BTreeSet<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub observer_seen_cycle: std::collections::BTreeMap<String, u32>,
}
```

Note: `BTreeSet<String>` / `BTreeMap<String, u32>` use `String` as the tribute-id type to match the existing convention in `shared/src/afflictions.rs`. If that crate uses a `TributeId` newtype (verify with `grep -n "pub type TributeId\|pub struct TributeId" shared/src/`), substitute that type instead.

Run: tests pass. `cargo build --package shared` clean.

---

## Task 4: Add `Addiction(Substance)` variant to `AfflictionKind`

Extends the discriminated-union enum from `lsis`. Payload-bearing variant — `Substance` is part of the kind identity (one Stimulant addiction is distinct from one Morphling addiction).

**Files:**
- Modify: `shared/src/afflictions.rs` (extend `AfflictionKind`)

- [ ] **Step 4.1: Locate `AfflictionKind` and audit its serialization**

Run: `grep -n "pub enum AfflictionKind" shared/src/afflictions.rs`. Read 30 lines after that line. Confirm the serde tag/content/rename strategy. Match it exactly when adding the new variant.

- [ ] **Step 4.2: Write failing test**

```rust
#[cfg(test)]
mod affliction_kind_addiction_tests {
    use super::*;

    #[test]
    fn addiction_kind_serializes_with_substance_payload() {
        let k = AfflictionKind::Addiction(Substance::Stimulant);
        let j = serde_json::to_string(&k).unwrap();
        // Exact format depends on the existing AfflictionKind serde strategy;
        // adjust this assertion to match (e.g. internally tagged "kind"+"value").
        // Failing assertion below is intentional — replace with the real
        // expected encoding once Step 4.1 has surveyed the existing convention.
        assert!(j.contains("addiction"), "got: {j}");
        assert!(j.contains("stimulant"), "got: {j}");
    }

    #[test]
    fn addiction_kind_round_trips() {
        for s in Substance::ALL {
            let k = AfflictionKind::Addiction(s);
            let j = serde_json::to_string(&k).unwrap();
            let back: AfflictionKind = serde_json::from_str(&j).unwrap();
            assert_eq!(back, k);
        }
    }

    #[test]
    fn addiction_kinds_are_distinct() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for s in Substance::ALL {
            assert!(set.insert(AfflictionKind::Addiction(s)));
        }
        assert_eq!(set.len(), 4);
    }
}
```

Run: tests fail to compile.

- [ ] **Step 4.3: Add the variant**

In `AfflictionKind`, add `Addiction(Substance)` after the existing variants. Follow the surrounding variants' formatting and any inline comments / doc-comments. If `AfflictionKind` derives `Hash`, `Substance` already does too (Task 1). If it derives `Ord`, `Substance` does too. No further changes needed.

Run: tests pass.

- [ ] **Step 4.4: Update any exhaustive matches that the compiler now flags**

Run: `cargo build --package shared` and `cargo build --package game` and `cargo build --package api`.

For each `error[E0004]: non-exhaustive patterns: \`Addiction(_)\` not covered`:
- Add a minimal arm. If the function returns `Option<X>` or `Result<X, E>`, return `None`/error variant. If it computes a stat penalty, return `Default::default()`. Document the stub with `// TODO(addiction PR3): real handling`.
- The full effect tables (High vs Withdrawal stat penalties, suppression rules) land in PR3. PR1's job is only to keep the build green.

Repeat until all crates compile.

- [ ] **Step 4.5: Verify**

Run: `cargo build --workspace`. Clean. Run: `cargo test --package shared affliction_kind_addiction_tests`. Pass.

---

## Task 5: Add `addiction_metadata` field to `Affliction`

Optional metadata extension slot. `None` for all non-Addiction kinds. `Some` for `Addiction(_)`.

**Files:**
- Modify: `shared/src/afflictions.rs` (extend `Affliction` struct)

- [ ] **Step 5.1: Audit existing metadata fields**

Run: `grep -n "trauma_metadata\|phobia_metadata\|fixation_metadata" shared/src/afflictions.rs`. If any exist, copy their attribute style exactly. If none exist, this is the first; use:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub addiction_metadata: Option<AddictionMetadata>,
```

- [ ] **Step 5.2: Write failing test**

```rust
#[cfg(test)]
mod affliction_addiction_metadata_tests {
    use super::*;

    #[test]
    fn affliction_without_addiction_metadata_omits_field_in_json() {
        // Existing afflictions (Wounded, Phobia, etc) must serialize without
        // an `addiction_metadata` key.
        let aff = Affliction {
            // ... fill in with the existing minimal Wounded constructor; replace
            // this stub with the real builder pattern from `lsis`.
            ..Default::default()
        };
        let j = serde_json::to_string(&aff).unwrap();
        assert!(!j.contains("addiction_metadata"), "expected omitted: {j}");
    }

    #[test]
    fn affliction_with_addiction_metadata_round_trips() {
        let meta = AddictionMetadata {
            substance: Substance::Stimulant,
            cycles_since_last_use: 0,
            high_cycles_remaining: 2,
            use_count_at_acquisition: 1,
            observed_by: Default::default(),
            observer_seen_cycle: Default::default(),
        };
        let aff = Affliction {
            // ... real builder
            ..Default::default()
        };
        // After construction, set kind = Addiction(Stimulant) and
        // addiction_metadata = Some(meta) using whatever public API exists.
        // Round-trip and assert.
    }

    #[test]
    fn old_affliction_json_without_field_deserializes_with_none() {
        // Backward-compat: an `Affliction` JSON that lacks the
        // `addiction_metadata` key must deserialize as `None`.
        let json = r#"{"kind":"wounded","severity":"mild"}"#;
        // Adjust to match the real Affliction shape from `lsis`.
        let _aff: Affliction = serde_json::from_str(json).unwrap();
    }
}
```

The assertion stubs above are intentionally rough; refine based on the real `Affliction` constructor surface from `lsis`. The three properties to verify are:
1. `addiction_metadata: None` → field omitted from serialized JSON.
2. `addiction_metadata: Some(_)` → field present and round-trips.
3. JSON missing the field → deserializes as `None` (backward compat).

Run: tests fail to compile.

- [ ] **Step 5.3: Add the field**

In the `Affliction` struct, after the existing metadata fields (or at the end of the struct if none), add:

```rust
/// Runtime-mutable state when `kind` is `AfflictionKind::Addiction(_)`.
/// `None` for all other kinds. See `AddictionMetadata` for field semantics.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub addiction_metadata: Option<AddictionMetadata>,
```

Update the `Default` impl (if `Affliction` has one) to set `addiction_metadata: None`. Update any `Affliction::new` / builder constructors to default the field to `None`.

Run: tests pass. Run: `cargo build --workspace`. Clean.

---

## Task 6: Add `addiction_use_count` field to `Tribute`

Persistent map of how many times this tribute has consumed each substance. Never reset by cure. Drives the acquisition curve (§5.2) and the relapse short-circuit (§5.1 step 5c).

**Files:**
- Modify: `game/src/tributes/mod.rs` (extend `Tribute` struct + `Default` impl)

- [ ] **Step 6.1: Locate `Tribute` struct**

Run: `grep -n "^pub struct Tribute" game/src/tributes/mod.rs`. Read 50 lines after. Note: serde derive presence, field ordering convention, where existing affliction-related fields live.

- [ ] **Step 6.2: Write failing test**

In `game/src/tributes/mod.rs` (or its `tests` submodule):

```rust
#[cfg(test)]
mod addiction_use_count_field_tests {
    use super::*;
    use shared::afflictions::Substance;

    #[test]
    fn default_tribute_has_empty_use_count() {
        let t = Tribute::default();
        assert!(t.addiction_use_count.is_empty());
    }

    #[test]
    fn empty_use_count_omitted_in_json() {
        let t = Tribute::default();
        let j = serde_json::to_string(&t).unwrap();
        assert!(!j.contains("addiction_use_count"), "expected omitted: {j}");
    }

    #[test]
    fn populated_use_count_round_trips() {
        let mut t = Tribute::default();
        t.addiction_use_count.insert(Substance::Stimulant, 3);
        t.addiction_use_count.insert(Substance::Morphling, 1);
        let j = serde_json::to_string(&t).unwrap();
        let back: Tribute = serde_json::from_str(&j).unwrap();
        assert_eq!(back.addiction_use_count, t.addiction_use_count);
    }

    #[test]
    fn old_tribute_json_without_field_deserializes_with_empty() {
        // Use a minimal Tribute JSON — derive from the real existing schema
        // (likely from `lsis` test fixtures). The key property is that JSON
        // lacking `addiction_use_count` must deserialize with an empty map.
        // Replace this with the real minimal encoding.
        // let json = r#"{ ... existing minimal tribute ... }"#;
        // let t: Tribute = serde_json::from_str(json).unwrap();
        // assert!(t.addiction_use_count.is_empty());
    }
}
```

Run: tests fail to compile.

- [ ] **Step 6.3: Add the field**

In the `Tribute` struct, after existing affliction-related fields (typically near `pub afflictions: ...`), add:

```rust
/// Per-substance lifetime use counter. Increments on every successful
/// `try_use_consumable` of an addictive item (PR2). NEVER reset by cure
/// or decay — this persistence is what enables relapse-on-first-use
/// semantics (addiction spec §5.1 step 5c).
#[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
pub addiction_use_count: std::collections::BTreeMap<shared::afflictions::Substance, u32>,
```

Update `impl Default for Tribute` to initialize the field as `BTreeMap::new()`.

- [ ] **Step 6.4: Verify**

Run: `cargo test --package game addiction_use_count_field_tests`. All pass. `cargo build --workspace` clean.

---

## Task 7: Extend `can_acquire` for the single-substance + cap-2 rule

The afflictions PR1 contract gates acquisition through `can_acquire(kind, tribute) -> bool` (or similar; verify with `grep -n "fn can_acquire" game/src/tributes/afflictions/`). For Addiction, two rules combine:

1. **Single-substance**: a tribute already addicted to `Stimulant` cannot acquire a *second* `Stimulant` addiction (that path is reinforcement, handled in `try_acquire_addiction`).
2. **Cap-at-2**: a tribute with 2 distinct active addictions cannot acquire a *third* of any substance.

**Files:**
- Modify: `game/src/tributes/afflictions/anatomy.rs` (or wherever `can_acquire` lives)

- [ ] **Step 7.1: Locate the entry point**

Run: `grep -rn "fn can_acquire" game/src/tributes/afflictions/`. Identify the function that the trauma / phobia plans extended (or that is documented in the `lsis` plan as the extension point).

- [ ] **Step 7.2: Write failing tests**

In `game/src/tributes/afflictions/anatomy.rs` (or a sibling test module):

```rust
#[cfg(test)]
mod addiction_can_acquire_tests {
    use super::*;
    use crate::tributes::Tribute;
    use shared::afflictions::*;

    fn tribute_with_addiction(sub: Substance, severity: Severity) -> Tribute {
        let mut t = Tribute::default();
        // Use the public API to insert an Addiction affliction. The exact
        // builder depends on what `lsis` exposed; replace this stub.
        // E.g.:
        // t.afflictions.insert(
        //     (AfflictionKind::Addiction(sub), None),
        //     Affliction { kind: AfflictionKind::Addiction(sub), severity, ..Default::default() },
        // );
        t
    }

    #[test]
    fn fresh_tribute_can_acquire_any_substance() {
        let t = Tribute::default();
        for s in Substance::ALL {
            assert!(can_acquire(&t, &AfflictionKind::Addiction(s)),
                "fresh tribute should accept {:?}", s);
        }
    }

    #[test]
    fn cannot_acquire_same_substance_twice() {
        let t = tribute_with_addiction(Substance::Stimulant, Severity::Mild);
        assert!(!can_acquire(&t, &AfflictionKind::Addiction(Substance::Stimulant)));
        // Other substances still acquirable.
        assert!(can_acquire(&t, &AfflictionKind::Addiction(Substance::Morphling)));
    }

    #[test]
    fn at_cap_blocks_third_distinct_substance() {
        let mut t = tribute_with_addiction(Substance::Stimulant, Severity::Mild);
        // Add a second.
        // t.afflictions.insert(... Addiction(Alcohol) ...);
        assert!(!can_acquire(&t, &AfflictionKind::Addiction(Substance::Morphling)));
        assert!(!can_acquire(&t, &AfflictionKind::Addiction(Substance::Painkiller)));
        // Still blocks same-substance reinforcement (orthogonally).
        assert!(!can_acquire(&t, &AfflictionKind::Addiction(Substance::Stimulant)));
    }
}
```

Run: tests fail (or compile but assertions fail).

- [ ] **Step 7.3: Implement**

Extend `can_acquire`:

```rust
pub fn can_acquire(tribute: &Tribute, kind: &AfflictionKind) -> bool {
    match kind {
        // ... existing arms (Wounded, Trauma if landed, Phobia if landed, ...)
        AfflictionKind::Addiction(sub) => {
            let active: Vec<Substance> = tribute
                .afflictions
                .keys()
                .filter_map(|(k, _)| match k {
                    AfflictionKind::Addiction(s) => Some(*s),
                    _ => None,
                })
                .collect();
            // Single-substance: already addicted to this exact substance.
            if active.contains(sub) {
                return false;
            }
            // Cap-at-2.
            if active.len() >= 2 {
                return false;
            }
            true
        }
        // ... rest
    }
}
```

(Adjust to the real signature and storage shape.)

Run: tests pass. `cargo build --workspace` clean.

---

## Task 8: Implement `acquisition_chance` and `high_duration` pure functions

Pure, RNG-free helpers consumed by `try_acquire_addiction` (Task 9) and PR3's effect-resolution code. Land here so PR1 has full unit coverage of the curve and the duration table.

**Files:**
- Create: `game/src/tributes/afflictions/addiction.rs` (start the file with these helpers; `try_acquire_addiction` will be appended in Task 9)
- Modify: `game/src/tributes/afflictions/mod.rs` (add `pub mod addiction;` and re-exports)

- [ ] **Step 8.1: Create the file with module wiring**

Create `game/src/tributes/afflictions/addiction.rs`:

```rust
//! Addiction acquisition + High-duration helpers.
//!
//! Spec: docs/superpowers/specs/2026-05-04-addiction-design.md §5.2, §7.2.
//!
//! This module is the contract surface for PR2 (use-pipeline producer hook
//! into `try_use_consumable`) and PR3 (brain layer + effects). It contains
//! NO RNG-driven flow control; `acquisition_chance` is a pure `fn(u32, Substance) -> f64`
//! and `high_duration` is a pure `fn(Substance, Severity) -> u32`. The only
//! RNG-using function in this PR is `try_acquire_addiction` (Task 9) which
//! takes `&mut impl RngCore`.

use shared::afflictions::{Severity, Substance};
```

Add `pub mod addiction;` to `game/src/tributes/afflictions/mod.rs` (and re-export the helpers when added).

- [ ] **Step 8.2: Write failing tests for `acquisition_chance`**

Append to `game/src/tributes/afflictions/addiction.rs`:

```rust
#[cfg(test)]
mod acquisition_chance_tests {
    use super::*;

    #[test]
    fn base_curve_matches_spec_table() {
        // Stimulant has multiplier 1.0, so base == returned for it.
        assert_eq!(acquisition_chance(1, Substance::Stimulant), 0.05);
        assert_eq!(acquisition_chance(2, Substance::Stimulant), 0.15);
        assert_eq!(acquisition_chance(3, Substance::Stimulant), 0.30);
        assert_eq!(acquisition_chance(4, Substance::Stimulant), 0.50);
        assert_eq!(acquisition_chance(5, Substance::Stimulant), 0.75);
        assert_eq!(acquisition_chance(6, Substance::Stimulant), 0.75);
        assert_eq!(acquisition_chance(99, Substance::Stimulant), 0.75);
    }

    #[test]
    fn use_count_zero_treated_as_one() {
        // Defensive: callers should always pass post-increment count >= 1,
        // but if a 0 leaks in, treat as first use.
        assert_eq!(acquisition_chance(0, Substance::Stimulant), 0.05);
    }

    #[test]
    fn morphling_amplifies_curve_with_cap() {
        // Multiplier 1.5, cap at 0.95.
        assert!((acquisition_chance(1, Substance::Morphling) - 0.075).abs() < 1e-9);
        assert!((acquisition_chance(2, Substance::Morphling) - 0.225).abs() < 1e-9);
        assert!((acquisition_chance(3, Substance::Morphling) - 0.45).abs() < 1e-9);
        assert!((acquisition_chance(4, Substance::Morphling) - 0.75).abs() < 1e-9);
        // 0.75 * 1.5 = 1.125, capped to 0.95.
        assert_eq!(acquisition_chance(5, Substance::Morphling), 0.95);
        assert_eq!(acquisition_chance(99, Substance::Morphling), 0.95);
    }

    #[test]
    fn alcohol_dampens_curve() {
        // Multiplier 0.7.
        assert!((acquisition_chance(1, Substance::Alcohol) - 0.035).abs() < 1e-9);
        assert!((acquisition_chance(5, Substance::Alcohol) - 0.525).abs() < 1e-9);
    }

    #[test]
    fn painkiller_uses_unit_multiplier() {
        assert_eq!(acquisition_chance(3, Substance::Painkiller), 0.30);
        assert_eq!(acquisition_chance(5, Substance::Painkiller), 0.75);
    }

    #[test]
    fn all_substances_capped_at_95_percent() {
        for sub in Substance::ALL {
            for n in 1..50 {
                let p = acquisition_chance(n, sub);
                assert!(p >= 0.0 && p <= 0.95, "{:?} use {} -> {} out of range", sub, n, p);
            }
        }
    }
}
```

Run: `cargo test --package game acquisition_chance_tests`. Fail to compile.

- [ ] **Step 8.3: Implement `acquisition_chance`**

Append to `addiction.rs`:

```rust
/// Per-use probability that consuming `substance` acquires (or relapses to)
/// the corresponding Addiction, given the tribute's *post-increment*
/// `addiction_use_count[substance]` value.
///
/// Capped at 0.95 for all substances. Returns 0.0 only if `use_count == 0`
/// AND the curve table starts at use 1 (defensive fallback: 0 is treated
/// as 1).
///
/// Spec: §5.2 (curve `[5, 15, 30, 50, 75]%`, multipliers Morphling 1.5,
/// Alcohol 0.7, Stimulant 1.0, Painkiller 1.0, cap 0.95).
pub fn acquisition_chance(use_count: u32, substance: Substance) -> f64 {
    let n = use_count.max(1);
    let base = match n {
        1 => 0.05,
        2 => 0.15,
        3 => 0.30,
        4 => 0.50,
        _ => 0.75, // 5+
    };
    let multiplier = match substance {
        Substance::Morphling => 1.5,
        Substance::Alcohol => 0.7,
        Substance::Stimulant | Substance::Painkiller => 1.0,
    };
    (base * multiplier).min(0.95)
}
```

Run: tests pass.

- [ ] **Step 8.4: Write failing tests for `high_duration`**

```rust
#[cfg(test)]
mod high_duration_tests {
    use super::*;

    #[test]
    fn duration_table_matches_spec() {
        // (Substance, Severity) -> cycles
        let cases: &[(Substance, Severity, u32)] = &[
            (Substance::Stimulant,  Severity::Mild,     2),
            (Substance::Stimulant,  Severity::Moderate, 1),
            (Substance::Stimulant,  Severity::Severe,   1),
            (Substance::Painkiller, Severity::Mild,     3),
            (Substance::Painkiller, Severity::Moderate, 2),
            (Substance::Painkiller, Severity::Severe,   1),
            (Substance::Morphling,  Severity::Mild,     4),
            (Substance::Morphling,  Severity::Moderate, 2),
            (Substance::Morphling,  Severity::Severe,   1),
            (Substance::Alcohol,    Severity::Mild,     1),
            (Substance::Alcohol,    Severity::Moderate, 1),
            (Substance::Alcohol,    Severity::Severe,   1),
        ];
        for (sub, sev, expected) in cases {
            assert_eq!(
                high_duration(*sub, *sev), *expected,
                "{:?} {:?} expected {}", sub, sev, expected
            );
        }
    }

    #[test]
    fn duration_is_never_zero() {
        // Tribute always gets at least one cycle of High after a successful use.
        for sub in Substance::ALL {
            for sev in [Severity::Mild, Severity::Moderate, Severity::Severe] {
                assert!(high_duration(sub, sev) >= 1);
            }
        }
    }

    #[test]
    fn duration_monotonically_non_increasing_in_severity() {
        // Spec §7.2 invariant: tolerance only ever shortens duration.
        for sub in Substance::ALL {
            let m = high_duration(sub, Severity::Mild);
            let md = high_duration(sub, Severity::Moderate);
            let s = high_duration(sub, Severity::Severe);
            assert!(m >= md, "{:?}: Mild {} < Moderate {}", sub, m, md);
            assert!(md >= s, "{:?}: Moderate {} < Severe {}", sub, md, s);
        }
    }
}
```

Run: fail to compile.

- [ ] **Step 8.5: Implement `high_duration`**

```rust
/// How many cycles of High mode a successful substance use grants.
/// Severity erodes the duration (tolerance, spec §7.2).
///
/// Always returns at least 1.
pub fn high_duration(substance: Substance, severity: Severity) -> u32 {
    use Severity::*;
    use Substance::*;
    match (substance, severity) {
        (Stimulant,  Mild)     => 2,
        (Stimulant,  Moderate) => 1,
        (Stimulant,  Severe)   => 1,
        (Painkiller, Mild)     => 3,
        (Painkiller, Moderate) => 2,
        (Painkiller, Severe)   => 1,
        (Morphling,  Mild)     => 4,
        (Morphling,  Moderate) => 2,
        (Morphling,  Severe)   => 1,
        (Alcohol,    _)        => 1,
    }
}
```

Run: tests pass. `cargo build --workspace` clean.

- [ ] **Step 8.6: Re-export from `mod.rs`**

In `game/src/tributes/afflictions/mod.rs`:

```rust
pub mod addiction;
pub use addiction::{acquisition_chance, high_duration};
```

(`try_acquire_addiction` and `AddictionAcquisition` re-exports added in Task 9.)

---

## Task 9: Implement `try_acquire_addiction` on `Tribute`

The producer-facing entrypoint. PR2 calls this from `try_use_consumable`. Encapsulates the probabilistic roll, cap check, relapse short-circuit, and reinforcement-on-existing branches. Returns a rich `AddictionAcquisition` outcome enum so the eventual PR2 message-emission code knows which payload to fire.

**Files:**
- Modify: `game/src/tributes/afflictions/addiction.rs` (append `AddictionAcquisition` + `try_acquire_addiction`)
- Modify: `game/src/tributes/afflictions/mod.rs` (re-export new symbols)
- Modify: `game/src/tributes/mod.rs` (thin `Tribute::try_acquire_addiction` method delegating to the helper)

- [ ] **Step 9.1: Define the outcome enum**

Append to `addiction.rs`:

```rust
/// Outcome of a `try_acquire_addiction` call. Richer than the generic
/// `AcquireResolution` from `lsis` because callers care about the
/// distinction between fresh acquisition / relapse / reinforcement /
/// resist (and which PR2 message variant to emit).
#[derive(Debug, Clone, PartialEq)]
pub enum AddictionAcquisition {
    /// First-ever acquisition of this substance for this tribute.
    /// `addiction_use_count[substance]` was 0 before the increment that
    /// preceded this call (i.e. this is the post-increment value of 1).
    Acquired {
        substance: Substance,
        severity: Severity,
        use_count: u32,
    },
    /// Auto-acquisition after cure (use_count > 0 from prior cured run).
    /// Bypassed the probabilistic roll. Severity always Mild on relapse.
    Relapse {
        substance: Substance,
        prior_uses: u32,
    },
    /// Tribute already had this addiction; counter reset, source merged
    /// (no source set for addiction — substance IS the source), High
    /// duration refreshed, escalation roll deferred (PR3 owns the
    /// escalation; PR1's reinforce branch only resets the counter and
    /// refreshes High).
    Reinforced {
        substance: Substance,
        from_severity: Severity,
        to_severity: Severity,
    },
    /// Roll succeeded (or relapse fired) but tribute was at the 2-active
    /// cap. State did not change. The substance's immediate effect (handled
    /// by caller) still applies.
    Resisted {
        substance: Substance,
        reason: AddictionResistReason,
    },
    /// Roll failed. Tribute had room for a new addiction but the dice
    /// said no. Most common outcome at low use counts.
    NotAcquired { substance: Substance },
}
```

Add `use shared::afflictions::AddictionResistReason;` near the top.

- [ ] **Step 9.2: Write failing tests**

Create `game/src/tributes/afflictions/addiction_tests.rs` (or append to `addiction.rs`'s existing `#[cfg(test)] mod tests`):

```rust
#[cfg(test)]
mod try_acquire_addiction_tests {
    use crate::tributes::Tribute;
    use crate::tributes::afflictions::addiction::{AddictionAcquisition, try_acquire_addiction};
    use shared::afflictions::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn fresh() -> Tribute { Tribute::default() }

    fn count(t: &Tribute, sub: Substance) -> u32 {
        *t.addiction_use_count.get(&sub).unwrap_or(&0)
    }

    #[test]
    fn first_use_low_chance_usually_not_acquired() {
        // 5% chance; with seed 0 this should produce NotAcquired most of the time.
        // We verify the API contract (Tribute mutated to use_count=1, no addiction
        // added, NotAcquired returned) using a seed known to produce a non-hit.
        let mut rng = StdRng::seed_from_u64(0);
        let mut t = fresh();
        let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
        // Use count incremented regardless of acquisition outcome.
        assert_eq!(count(&t, Substance::Stimulant), 1);
        // Verify NotAcquired or Acquired (depends on seed); either way, no panic.
        match outcome {
            AddictionAcquisition::NotAcquired { substance } => assert_eq!(substance, Substance::Stimulant),
            AddictionAcquisition::Acquired { substance, severity, use_count } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(severity, Severity::Mild);
                assert_eq!(use_count, 1);
            }
            other => panic!("unexpected first-use outcome: {:?}", other),
        }
    }

    #[test]
    fn forced_high_chance_acquires() {
        // After 5 uses, base chance is 75%. Pick a seed that hits.
        let mut rng = StdRng::seed_from_u64(42);
        let mut t = fresh();
        // Pre-seed use count to 4 so this call increments to 5.
        t.addiction_use_count.insert(Substance::Stimulant, 4);
        let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
        assert_eq!(count(&t, Substance::Stimulant), 5);
        match outcome {
            AddictionAcquisition::Acquired { substance, severity, use_count } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(severity, Severity::Mild);
                assert_eq!(use_count, 5);
                // Affliction was actually inserted.
                let aff = t.afflictions.get(&(AfflictionKind::Addiction(Substance::Stimulant), None));
                assert!(aff.is_some());
                let meta = aff.unwrap().addiction_metadata.as_ref().unwrap();
                assert_eq!(meta.substance, Substance::Stimulant);
                assert_eq!(meta.use_count_at_acquisition, 5);
                assert_eq!(meta.cycles_since_last_use, 0);
                // High refreshed to spec value.
                assert_eq!(meta.high_cycles_remaining, 2); // Stimulant Mild
            }
            other => panic!("expected Acquired with seed 42, got {:?}", other),
        }
    }

    #[test]
    fn reinforce_existing_addiction_refreshes_high_and_resets_counter() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut t = fresh();
        // Pre-acquire Mild Stimulant with stale state.
        t.addiction_use_count.insert(Substance::Stimulant, 6);
        // Use the public API to insert a Mild Stimulant Addiction with
        // cycles_since_last_use=4, high_cycles_remaining=0 (Withdrawal).
        // ... insertion code (depends on lsis API) ...

        let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);

        match outcome {
            AddictionAcquisition::Reinforced { substance, from_severity, to_severity } => {
                assert_eq!(substance, Substance::Stimulant);
                assert_eq!(from_severity, Severity::Mild);
                assert_eq!(to_severity, Severity::Mild); // PR1 does not escalate
            }
            other => panic!("expected Reinforced, got {:?}", other),
        }

        let meta = t
            .afflictions
            .get(&(AfflictionKind::Addiction(Substance::Stimulant), None))
            .unwrap()
            .addiction_metadata
            .as_ref()
            .unwrap();
        // Counter reset.
        assert_eq!(meta.cycles_since_last_use, 0);
        // High refreshed (was 0 / Withdrawal; now Mild duration of 2).
        assert_eq!(meta.high_cycles_remaining, 2);
        assert_eq!(count(&t, Substance::Stimulant), 7);
    }

    #[test]
    fn relapse_after_cure_auto_acquires_at_mild() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut t = fresh();
        // Tribute has prior history (cured Stimulant addiction).
        t.addiction_use_count.insert(Substance::Stimulant, 3);
        // No active Stimulant addiction.

        let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);

        match outcome {
            AddictionAcquisition::Relapse { substance, prior_uses } => {
                assert_eq!(substance, Substance::Stimulant);
                // prior_uses is the post-increment count - 1, which equals
                // the count before this use (3). Verify spec contract.
                assert_eq!(prior_uses, 3);
            }
            other => panic!("expected Relapse, got {:?}", other),
        }
        assert_eq!(count(&t, Substance::Stimulant), 4);
        let aff = t.afflictions.get(&(AfflictionKind::Addiction(Substance::Stimulant), None)).unwrap();
        assert_eq!(aff.severity, Severity::Mild);
        assert_eq!(aff.addiction_metadata.as_ref().unwrap().use_count_at_acquisition, 4);
    }

    #[test]
    fn cap_at_two_resists_third_substance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut t = fresh();
        // Pre-insert two distinct active addictions (Stimulant + Alcohol).
        t.addiction_use_count.insert(Substance::Stimulant, 5);
        t.addiction_use_count.insert(Substance::Alcohol, 5);
        // ... insertion of Stimulant + Alcohol Addiction afflictions ...

        let outcome = try_acquire_addiction(&mut t, Substance::Morphling, &mut rng);

        match outcome {
            AddictionAcquisition::Resisted { substance, reason } => {
                assert_eq!(substance, Substance::Morphling);
                assert_eq!(reason, AddictionResistReason::AtCap);
            }
            other => panic!("expected Resisted(AtCap), got {:?}", other),
        }
        // Use count still incremented (the tribute did consume it).
        assert_eq!(count(&t, Substance::Morphling), 1);
        // No third addiction added.
        let morphling = t.afflictions.get(&(AfflictionKind::Addiction(Substance::Morphling), None));
        assert!(morphling.is_none());
        // Existing two unchanged.
        assert_eq!(t.afflictions.iter().filter(|((k, _), _)| matches!(k, AfflictionKind::Addiction(_))).count(), 2);
    }

    #[test]
    fn cap_at_two_does_not_block_reinforcement_of_existing_substance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut t = fresh();
        t.addiction_use_count.insert(Substance::Stimulant, 5);
        t.addiction_use_count.insert(Substance::Alcohol, 5);
        // ... insertion of Stimulant + Alcohol Addiction afflictions ...

        // Use Stimulant again (one of the existing two).
        let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
        assert!(matches!(outcome, AddictionAcquisition::Reinforced { .. }));
    }

    #[test]
    fn use_count_increments_on_every_call_regardless_of_outcome() {
        let mut rng = StdRng::seed_from_u64(0);
        let mut t = fresh();
        for _ in 0..10 {
            try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
        }
        assert_eq!(count(&t, Substance::Stimulant), 10);
    }
}
```

Run: tests fail to compile.

- [ ] **Step 9.3: Implement `try_acquire_addiction`**

Append to `addiction.rs`:

```rust
use crate::tributes::Tribute;
use rand::Rng;

/// Producer-facing entry point. PR2 calls this from `try_use_consumable`
/// after the immediate substance effect has applied. PR3 owns the cycle-tick
/// reinforcement / decay; this function handles only the use-event branch.
///
/// Side effects (in order):
/// 1. Increments `tribute.addiction_use_count[substance]`.
/// 2. Inspects existing Addiction state for `substance`.
/// 3. Branches into one of {Reinforced, Relapse, Acquired, Resisted, NotAcquired}.
/// 4. Returns the rich outcome enum for the caller to translate to a message.
///
/// RNG is consumed only on the acquisition-roll path (no existing addiction,
/// no relapse, cap not full). Reinforcement and Relapse and Resisted paths
/// do NOT touch the RNG.
pub fn try_acquire_addiction(
    tribute: &mut Tribute,
    substance: Substance,
    rng: &mut impl Rng,
) -> AddictionAcquisition {
    // 1. Increment use count.
    let new_count = tribute
        .addiction_use_count
        .entry(substance)
        .and_modify(|n| *n = n.saturating_add(1))
        .or_insert(1);
    let new_count = *new_count;
    let prior_count = new_count - 1;

    // 2. Existing addiction for this substance?
    let key = (AfflictionKind::Addiction(substance), None);
    if let Some(existing) = tribute.afflictions.get_mut(&key) {
        let from_sev = existing.severity;
        let meta = existing
            .addiction_metadata
            .as_mut()
            .expect("Addiction affliction missing addiction_metadata (invariant violation)");
        meta.cycles_since_last_use = 0;
        meta.high_cycles_remaining = high_duration(substance, from_sev);
        return AddictionAcquisition::Reinforced {
            substance,
            from_severity: from_sev,
            to_severity: from_sev, // PR3 owns escalation
        };
    }

    // 3. Cap check (applies to both fresh acquisition and relapse).
    let active_addictions = tribute
        .afflictions
        .keys()
        .filter(|(k, _)| matches!(k, AfflictionKind::Addiction(_)))
        .count();
    if active_addictions >= 2 {
        return AddictionAcquisition::Resisted {
            substance,
            reason: AddictionResistReason::AtCap,
        };
    }

    // 4. Relapse path: prior uses but no active addiction → auto-acquire at Mild.
    if prior_count > 0 {
        insert_fresh_addiction(tribute, substance, Severity::Mild, new_count);
        return AddictionAcquisition::Relapse {
            substance,
            prior_uses: prior_count,
        };
    }

    // 5. Probabilistic acquisition roll (first-ever use of this substance).
    let chance = acquisition_chance(new_count, substance);
    if rng.gen_bool(chance) {
        insert_fresh_addiction(tribute, substance, Severity::Mild, new_count);
        AddictionAcquisition::Acquired {
            substance,
            severity: Severity::Mild,
            use_count: new_count,
        }
    } else {
        AddictionAcquisition::NotAcquired { substance }
    }
}

fn insert_fresh_addiction(
    tribute: &mut Tribute,
    substance: Substance,
    severity: Severity,
    use_count: u32,
) {
    let meta = AddictionMetadata {
        substance,
        cycles_since_last_use: 0,
        high_cycles_remaining: high_duration(substance, severity),
        use_count_at_acquisition: use_count,
        observed_by: Default::default(),
        observer_seen_cycle: Default::default(),
    };
    let aff = Affliction {
        // Replace with the real `lsis` Affliction constructor; key fields:
        //   kind: AfflictionKind::Addiction(substance),
        //   severity,
        //   addiction_metadata: Some(meta),
        //   ... other fields default ...
        ..Default::default()
    };
    tribute.afflictions.insert((AfflictionKind::Addiction(substance), None), aff);
}
```

Add `use shared::afflictions::*;` at the top if not already present.

Run: `cargo test --package game try_acquire_addiction_tests`. Iterate on test seeds (`StdRng::seed_from_u64`) until each test exercises the path it claims to test. Document the chosen seeds inline so future readers know they were picked deliberately.

- [ ] **Step 9.4: Add `Tribute::try_acquire_addiction` thin wrapper**

In `game/src/tributes/mod.rs`:

```rust
impl Tribute {
    /// See `crate::tributes::afflictions::addiction::try_acquire_addiction`.
    pub fn try_acquire_addiction<R: rand::Rng>(
        &mut self,
        substance: shared::afflictions::Substance,
        rng: &mut R,
    ) -> crate::tributes::afflictions::addiction::AddictionAcquisition {
        crate::tributes::afflictions::addiction::try_acquire_addiction(self, substance, rng)
    }

    /// Iterator over active Addiction afflictions on this tribute.
    pub fn addiction_afflictions(&self) -> impl Iterator<Item = (&shared::afflictions::Substance, &shared::afflictions::Affliction)> {
        self.afflictions.iter().filter_map(|((k, _), v)| match k {
            shared::afflictions::AfflictionKind::Addiction(s) => Some((s, v)),
            _ => None,
        })
    }
}
```

Run: `cargo build --workspace` clean.

- [ ] **Step 9.5: Re-export from `mod.rs`**

In `game/src/tributes/afflictions/mod.rs`:

```rust
pub use addiction::{
    acquisition_chance, high_duration, try_acquire_addiction,
    AddictionAcquisition,
};
```

---

## Task 10: Integration test — full `Tribute` round-trip with addiction

End-to-end smoke: instantiate a `Tribute`, call `try_acquire_addiction` enough times to exercise every outcome variant, serialize the tribute, deserialize it, verify state matches.

**Files:**
- Create: `game/tests/addiction_acquisition_test.rs`

- [ ] **Step 10.1: Write the integration test**

```rust
use game::tributes::Tribute;
use game::tributes::afflictions::addiction::{AddictionAcquisition, try_acquire_addiction};
use rand::SeedableRng;
use rand::rngs::StdRng;
use shared::afflictions::*;

fn count(t: &Tribute, sub: Substance) -> u32 {
    *t.addiction_use_count.get(&sub).unwrap_or(&0)
}

#[test]
fn full_lifecycle_acquire_reinforce_resist_round_trip() {
    let mut rng = StdRng::seed_from_u64(0xADD1C71_0_NPR1);
    let mut t = Tribute::default();
    t.id = "tributes:lifecycle".into();

    // 1. Hammer Stimulant until acquired (max 50 attempts; 75% chance from use 5+
    //    means probability of NOT acquiring in 50 attempts is ~3.7e-31).
    let mut acquired_at = None;
    for n in 1..=50 {
        match try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng) {
            AddictionAcquisition::Acquired { use_count, .. } => {
                acquired_at = Some(use_count);
                break;
            }
            AddictionAcquisition::NotAcquired { .. } => continue,
            other => panic!("unexpected first-acquire outcome at n={}: {:?}", n, other),
        }
    }
    let acquired_at = acquired_at.expect("Stimulant should have been acquired within 50 uses");
    assert_eq!(count(&t, Substance::Stimulant), acquired_at);

    // 2. Use again → Reinforced.
    let outcome = try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
    assert!(matches!(outcome, AddictionAcquisition::Reinforced { .. }));

    // 3. Hammer Alcohol until acquired (second active addiction).
    let mut alcohol_acquired = false;
    for _ in 0..100 {
        match try_acquire_addiction(&mut t, Substance::Alcohol, &mut rng) {
            AddictionAcquisition::Acquired { .. } => { alcohol_acquired = true; break; }
            AddictionAcquisition::NotAcquired { .. } => continue,
            other => panic!("unexpected alcohol outcome: {:?}", other),
        }
    }
    assert!(alcohol_acquired, "Alcohol should be acquired within 100 uses");

    // 4. At cap: Morphling resists.
    let outcome = try_acquire_addiction(&mut t, Substance::Morphling, &mut rng);
    assert!(matches!(
        outcome,
        AddictionAcquisition::Resisted { reason: AddictionResistReason::AtCap, .. }
    ));
    assert_eq!(count(&t, Substance::Morphling), 1, "use count still incremented on resist");

    // 5. Round-trip serialization preserves all state.
    let json = serde_json::to_string(&t).expect("Tribute serializes");
    let back: Tribute = serde_json::from_str(&json).expect("Tribute round-trips");

    assert_eq!(back.addiction_use_count, t.addiction_use_count);
    assert_eq!(back.addiction_afflictions().count(), 2);
    for ((kind, slot), aff) in t.afflictions.iter().filter(|((k, _), _)| matches!(k, AfflictionKind::Addiction(_))) {
        let back_aff = back.afflictions.get(&(*kind, slot.clone())).expect("addiction preserved");
        assert_eq!(back_aff.severity, aff.severity);
        assert_eq!(back_aff.addiction_metadata, aff.addiction_metadata);
    }
}
```

Run: `cargo test --package game --test addiction_acquisition_test`. Pass.

---

## Task 11: Proptest invariants

Per spec §13.4. Five core invariants:

1. **Cap invariant** — never more than 2 active Addictions.
2. **Use-count monotonicity** — `addiction_use_count[s]` never decreases.
3. **Relapse determinism** — `prior_count > 0 && no active Addiction(s) && cap not full && use(s) → Relapse`.
4. **Reinforcement-decay exclusivity (degenerate form for PR1)** — every call resets `cycles_since_last_use` to 0 (because PR1 only handles the use-event branch; decay tick is PR3).
5. **Acquisition probability bounds** — `acquisition_chance(n, s) ∈ [0.0, 0.95]` for all `(n, s)`.

**Files:**
- Modify: `game/src/tributes/afflictions/addiction.rs` (append `proptest!` block, gated `#[cfg(test)]`)

- [ ] **Step 11.1: Add proptest dependency check**

Run: `grep -n "proptest" game/Cargo.toml`. Confirm `proptest` is in `dev-dependencies`. If missing, add it (with the version pinned to match the existing `lsis` plan / trauma plan choice — likely `proptest = "1.5"`).

- [ ] **Step 11.2: Write the proptest module**

```rust
#[cfg(test)]
mod proptest_invariants {
    use super::*;
    use crate::tributes::Tribute;
    use proptest::prelude::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn substance_strategy() -> impl Strategy<Value = Substance> {
        prop_oneof![
            Just(Substance::Stimulant),
            Just(Substance::Morphling),
            Just(Substance::Alcohol),
            Just(Substance::Painkiller),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn cap_invariant_holds_across_arbitrary_use_sequences(
            seed in any::<u64>(),
            uses in prop::collection::vec(substance_strategy(), 1..200),
        ) {
            let mut t = Tribute::default();
            let mut rng = StdRng::seed_from_u64(seed);
            for sub in &uses {
                try_acquire_addiction(&mut t, *sub, &mut rng);
                let active = t
                    .afflictions
                    .keys()
                    .filter(|(k, _)| matches!(k, AfflictionKind::Addiction(_)))
                    .count();
                prop_assert!(active <= 2, "active count {} exceeded cap", active);
            }
        }

        #[test]
        fn use_count_monotonic(
            seed in any::<u64>(),
            uses in prop::collection::vec(substance_strategy(), 1..200),
        ) {
            let mut t = Tribute::default();
            let mut rng = StdRng::seed_from_u64(seed);
            let mut prev: std::collections::BTreeMap<Substance, u32> = Default::default();
            for sub in &uses {
                try_acquire_addiction(&mut t, *sub, &mut rng);
                for s in Substance::ALL {
                    let now = *t.addiction_use_count.get(&s).unwrap_or(&0);
                    let was = *prev.get(&s).unwrap_or(&0);
                    prop_assert!(now >= was, "use_count for {:?} regressed: {} -> {}", s, was, now);
                    prev.insert(s, now);
                }
            }
        }

        #[test]
        fn acquisition_chance_in_range(
            n in 0u32..1_000,
            sub in substance_strategy(),
        ) {
            let p = acquisition_chance(n, sub);
            prop_assert!(p >= 0.0 && p <= 0.95, "p={} out of [0, 0.95]", p);
        }

        #[test]
        fn every_call_resets_cycles_since_last_use(
            seed in any::<u64>(),
            uses in prop::collection::vec(substance_strategy(), 1..100),
        ) {
            // After each call, every active Addiction's
            // cycles_since_last_use must be 0 (because PR1 has no decay tick;
            // the only PR1 mutation is the reset on use).
            // Caveat: an addiction that exists *but was not the substance used
            // this call* will retain whatever cycles_since_last_use it had.
            // Since PR1 never increments the counter, the only possible value
            // it can have is 0 (set at creation/relapse/reinforce).
            let mut t = Tribute::default();
            let mut rng = StdRng::seed_from_u64(seed);
            for sub in &uses {
                try_acquire_addiction(&mut t, *sub, &mut rng);
                for ((_k, _), aff) in t.afflictions.iter().filter(|((k, _), _)| matches!(k, AfflictionKind::Addiction(_))) {
                    let meta = aff.addiction_metadata.as_ref().unwrap();
                    prop_assert_eq!(meta.cycles_since_last_use, 0);
                }
            }
        }

        #[test]
        fn relapse_determinism(
            seed in any::<u64>(),
            sub in substance_strategy(),
            prior in 1u32..10,
        ) {
            // Setup: tribute has prior uses but no active addiction for `sub`,
            // cap is not full. Result MUST be Relapse.
            let mut t = Tribute::default();
            t.addiction_use_count.insert(sub, prior);
            let mut rng = StdRng::seed_from_u64(seed);
            let outcome = try_acquire_addiction(&mut t, sub, &mut rng);
            prop_assert!(
                matches!(outcome, AddictionAcquisition::Relapse { substance, prior_uses }
                    if substance == sub && prior_uses == prior),
                "expected Relapse(prior_uses={}), got {:?}", prior, outcome
            );
        }
    }
}
```

Run: `cargo test --package game proptest_invariants -- --nocapture`. All five properties pass with 256 cases each.

- [ ] **Step 11.3: Document any shrunken counterexamples**

If any property fails, do NOT silence it. Read the shrunken counterexample. Either:
- Fix the implementation if the property describes a real spec invariant.
- Refine the property if it overstates the invariant (and document why in the property's doc-comment).

---

## Task 12: Insta snapshot for canonical Addiction state

Per spec §13.3. One YAML snapshot of a representative `Tribute` after a known sequence of calls.

**Files:**
- Create: `game/src/tributes/afflictions/snapshots/` (insta will auto-create on first run)
- Modify: `game/src/tributes/afflictions/addiction.rs` (append snapshot test)

- [ ] **Step 12.1: Write the snapshot test**

```rust
#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use crate::tributes::Tribute;
    use insta::assert_yaml_snapshot;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn canonical_two_addiction_state() {
        let mut rng = StdRng::seed_from_u64(0xADD1C71_5N4P);
        let mut t = Tribute::default();
        t.id = "tributes:snapshot".into();

        // Hammer Stimulant until acquired.
        for _ in 0..50 {
            if matches!(
                try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng),
                AddictionAcquisition::Acquired { .. }
            ) { break; }
        }
        // Use Stimulant once more → Reinforced.
        try_acquire_addiction(&mut t, Substance::Stimulant, &mut rng);
        // Hammer Alcohol.
        for _ in 0..100 {
            if matches!(
                try_acquire_addiction(&mut t, Substance::Alcohol, &mut rng),
                AddictionAcquisition::Acquired { .. }
            ) { break; }
        }
        // Cap-resist Morphling.
        try_acquire_addiction(&mut t, Substance::Morphling, &mut rng);

        // Snapshot the Addiction-relevant state only (not the whole Tribute).
        let snap = serde_json::json!({
            "addiction_use_count": t.addiction_use_count,
            "active_addictions": t.afflictions.iter()
                .filter(|((k, _), _)| matches!(k, AfflictionKind::Addiction(_)))
                .map(|((k, _), v)| (format!("{:?}", k), serde_json::to_value(v).unwrap()))
                .collect::<std::collections::BTreeMap<_, _>>(),
        });
        assert_yaml_snapshot!("addiction_pr1_canonical", snap);
    }
}
```

Run: `cargo test --package game canonical_two_addiction_state`. First run creates `addiction_pr1_canonical.snap.new`. Review with `cargo insta review`. Accept. Re-run; passes.

---

## Task 13: SurrealDB schema migration

Even though the affliction object is already `flexible` (no schema-level changes required), bump the migration version pointer so the migrations crate logs the addition explicitly and downstream tooling can detect the schema change point.

**Files:**
- Create: `migrations/definitions/20260504_030000_TributeAfflictions_AddictionMetadata.json`

- [ ] **Step 13.1: Inspect the most recent migration**

Run: `ls -1 migrations/definitions/ | tail -5`. Read the most recent file (likely a trauma migration if `u1fa` has landed). Match its JSON shape exactly.

- [ ] **Step 13.2: Create the no-op migration**

```json
{
  "name": "TributeAfflictions_AddictionMetadata",
  "description": "PR1 of addiction system. Adds AfflictionKind::Addiction(Substance), AddictionMetadata struct, addiction_metadata field on Affliction, addiction_use_count map on Tribute. Relies on the existing flexible affliction object — no schema change to the SurrealDB tables. This file exists to bump the migration version pointer.",
  "up": [],
  "down": []
}
```

(Adjust to match the surrealdb-migrations format used by neighboring files.)

- [ ] **Step 13.3: Verify migrations apply cleanly**

Run the project's standard migration test (likely a `cargo test` in `api` or `migrations`; run `just quality` if present). All pass.

---

## Task 14: Final verification + format

- [ ] **Step 14.1: Format**

Run: `cargo fmt --all`. Verify with `cargo fmt --all -- --check`.

- [ ] **Step 14.2: Lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`. Address any new lints introduced by this PR. (Pre-existing lints in unrelated code are out of scope.)

- [ ] **Step 14.3: Full test pass**

Run: `cargo test --package shared` then `cargo test --package game --lib --tests`. All green. Run `cargo test --package game --test addiction_acquisition_test`. Green.

- [ ] **Step 14.4: Quality gate**

Run: `just quality` if available. All green.

- [ ] **Step 14.5: Verify spec coverage**

Re-read addiction spec §4, §5.1, §5.2, §5.3, §14. Each numbered item must be implemented or explicitly noted as PR2/PR3 territory in this plan's task list. If anything is missing, add a task.

---

## Task 15: Push, open PR, file follow-ups

- [ ] **Step 15.1: Push the bookmark**

```bash
jj git fetch
jj rebase -d main@origin
jj git push --bookmark addiction-pr1
```

If the rebase produces conflicts, resolve them and re-run.

- [ ] **Step 15.2: Open the PR (from main worktree)**

Per `gh-pr-create-cwd` convention, run from the main worktree:

```bash
gh pr create --base main --head addiction-pr1 \
  --title "feat(addiction): PR1 — types, storage, try_acquire_addiction" \
  --body "$(cat <<'EOF'
## Summary

- Adds `Substance` enum, `AddictionResistReason`, `AddictionMetadata` to `shared/src/afflictions.rs`
- Adds `AfflictionKind::Addiction(Substance)` variant + `addiction_metadata: Option<AddictionMetadata>` field on `Affliction`
- Adds `addiction_use_count: BTreeMap<Substance, u32>` field on `Tribute` (persistent across cure for relapse semantics)
- Adds `acquisition_chance` and `high_duration` pure helper functions
- Adds `try_acquire_addiction` producer-facing entry point with probabilistic curve, substance multiplier, cap-at-2, relapse short-circuit, and single-substance reinforcement
- Adds no-op SurrealDB migration to bump the version pointer

## Changes

- `shared/src/afflictions.rs` — new types
- `game/src/tributes/mod.rs` — `addiction_use_count` field, thin wrapper methods
- `game/src/tributes/afflictions/addiction.rs` — helper module + `try_acquire_addiction`
- `game/src/tributes/afflictions/anatomy.rs` — `can_acquire` extension for the single-substance + cap-2 rules
- `game/tests/addiction_acquisition_test.rs` — end-to-end round-trip
- `migrations/definitions/20260504_030000_TributeAfflictions_AddictionMetadata.json` — version-pointer bump

## Verification

- \`cargo test --package shared\` — green
- \`cargo test --package game --lib --tests\` — green
- \`cargo test --package game --test addiction_acquisition_test\` — green
- \`cargo clippy --workspace --all-targets -- -D warnings\` — green
- \`cargo fmt --all -- --check\` — green
- Insta snapshot accepted: \`addiction_pr1_canonical.snap\`
- Proptest 256 cases green for: cap invariant, use-count monotonicity, acquisition_chance bounds, cycles_since_last_use reset, relapse determinism

## Follow-ups

- \`hangrier_games-<addiction-pr2-id>\` — use-pipeline producer hook into \`try_use_consumable\`, message emission, \`Consumable::substance\` mapping
- \`hangrier_games-<addiction-pr3-id>\` — brain layer, High vs Withdrawal effects, observer state, decay tick, cure paths, alliance integration
- \`hangrier_games-<addiction-pr4-id>\` — frontend (tribute card section, timeline cards, state strip)

Spec: \`docs/superpowers/specs/2026-05-04-addiction-design.md\`

🤖 Implementation plan: \`docs/superpowers/plans/2026-05-04-addiction-pr1.md\`
EOF
)"
```

Substitute the real beads IDs once filed.

- [ ] **Step 15.3: Update beads**

```bash
bd close <addiction-pr1-id> --reason "Landed in PR <PR-URL>"
```

- [ ] **Step 15.4: Hand off**

Provide the PR URL + the next-PR beads ID + a one-line note on what PR2 will build on (the `try_acquire_addiction` entry point and the `Consumable::substance` mapping it will need to add).

---

## Summary

This plan lands the addiction type system, storage extensions on `Affliction` and `Tribute`, the `acquisition_chance` and `high_duration` pure helpers, and the `try_acquire_addiction` producer-facing entry point. It does NOT integrate with the consumable-use pipeline (PR2), the brain pipeline (PR3), or the frontend (PR4). Cap-2, single-substance, relapse-on-first-use, and tolerance-shortened High duration are all enforced and tested.

## Spec coverage check

| Spec section | Covered by | Notes |
|---|---|---|
| §4 Types — `Substance`, `AddictionMetadata`, `AddictionResistReason`, `AfflictionKind::Addiction(_)`, `addiction_metadata` field on `Affliction`, `addiction_use_count` on `Tribute` | Tasks 1, 2, 3, 4, 5, 6 | Full |
| §5.1 Acquisition flow (steps 1, 4-6 of the use-pipeline) | Task 9 | PR1 implements the helper; PR2 wires it into `try_use_consumable` (steps 1, 2 of the flow are PR2) |
| §5.2 Acquisition roll | Tasks 8, 9 | `acquisition_chance` + RNG branch in `try_acquire_addiction` |
| §5.3 Cap-at-2 enforcement | Tasks 7, 9 | `can_acquire` + Resisted branch in `try_acquire_addiction` |
| §6 Reinforcement / decay / cure | Task 9 (partial) | PR1 only handles use-event reinforcement (counter reset, High refresh). PR3 owns the cycle-tick decay, escalation roll, and cure paths. Documented in the task descriptions. |
| §7.2 High duration table | Task 8 | `high_duration` function + tests |
| §7 effects, §8 brain layer, §9 visibility, §10 messages, §11 alliance, §12 UI | — | Out of scope (PR2/PR3/PR4) |
| §14 Migration / rollout | Tasks 5, 6, 13 | Backward-compat field attributes + no-op migration |
| §13.1 Unit tests | Tasks 1, 2, 3, 4, 5, 6, 7, 8, 9 | Each type / helper has dedicated unit coverage |
| §13.2 Integration tests | Task 10 | One end-to-end round-trip in this PR; the per-scenario integration tests land in PR2/PR3 alongside the producer / brain code |
| §13.3 Insta snapshots | Task 12 | Canonical two-addiction state |
| §13.4 Proptest properties | Task 11 | Five PR1-relevant invariants; the rest land in PR2/PR3 |

## Self-review notes

- **Single-substance vs cap-2 ordering in `try_acquire_addiction`**: the helper checks the existing-addiction branch first (reinforcement is allowed even at cap, as long as it's the *same* substance). Then cap check. Then relapse. Then roll. This ordering matches the spec §5.1 step 5 branch tree exactly.
- **`high_cycles_remaining` initialization**: PR1 sets it on every use (acquire / relapse / reinforce) to `high_duration(substance, severity)`. PR3 will add the cycle-tick decrement. Initializing here means the field is always in a valid state from the moment an Addiction exists.
- **`use_count_at_acquisition` semantics**: snapshot at the moment of acquisition (Acquired or Relapse), used for the future `AddictionRelapse { prior_uses }` payload contract. Not updated on reinforcement (it's a one-time snapshot).
- **Reinforcement does NOT escalate severity in PR1**. The `to_severity == from_severity` invariant is intentional. PR3 adds the 12% escalation roll via the shared `apply_traumatic_reinforcement` helper extracted by phobia PR3 / trauma PR3 / fixation PR2 (whichever lands first).
- **RNG discipline**: the only RNG-consuming path in this PR is `gen_bool(chance)` in the fresh-acquisition branch. Reinforcement, Relapse, Resisted, and "no addiction yet but use_count is 0" paths are all deterministic. Tests pin specific seeds and document why.
- **`saturating_add` on use count**: a tribute who has consumed a substance more than `u32::MAX` times is implausible, but the saturating semantic prevents wraparound if it ever happens. No panic.
- **`Affliction` constructor stub**: the test code and `insert_fresh_addiction` helper both reference an `Affliction { ..Default::default() }` builder. The real constructor depends on what `lsis` exposed (likely a `new` or `builder` method). The plan executor should substitute the real API on first compile error.
- **TributeId type**: stubbed as `String` throughout. If `lsis` introduced a `TributeId` newtype, substitute it everywhere on first compile error (mostly in `AddictionMetadata` observer fields).
- **No `MessagePayload` arms added**: PR2's responsibility. The `AddictionAcquisition` outcome enum is the contract that PR2 will translate into messages.
