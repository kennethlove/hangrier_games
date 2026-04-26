# Tribute Alliances Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `BrainPersonality` with a per-tribute `traits: Vec<Trait>` system and add a pair-wise `allies: Vec<Uuid>` graph with formation/break mechanics.

**Architecture:** Pure-engine work in `game/` (new `traits.rs` + `alliances.rs`, gut `brains.rs`, edit `tributes/mod.rs` and `games.rs`); thin DTO/schema/API extensions; hard cutover with no data migration.

**Tech Stack:** Rust 2024, rstest, SurrealDB schema, Serde, Dioxus (frontend touch only for `loyalty` scrub).

**Spec:** `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md`

**Bead:** `hangrier_games-0ug` (in_progress)

---

## Bead Plan

File these beads BEFORE coding. Each phase = one bead, blocking the next:

```bash
bd create --title="Phase 1: traits.rs module" --description="Trait enum, conflict table, district pools, threshold deltas, alliance affinity, geometric_mean_affinity, refusers. Pure functions, fully unit-tested." --type=task --priority=2 --notes="depends-on=0ug"
bd create --title="Phase 2: alliances.rs module" --description="Formation roll, refuser gate, deciding factor, break triggers (sanity / treacherous / trust-shock cascade), AllianceEvent queue. Pure on &Tribute slices." --type=task --priority=2
bd create --title="Phase 3: Tribute + Brain integration" --description="Add allies/traits/turns_since_last_betrayal to Tribute. Remove BrainPersonality, district!=district filter, loyalty field, LOYALTY_BREAK_LEVEL. Rewrite pick_target. Add Tribute::test_default." --type=task --priority=2
bd create --title="Phase 4: run_day_night_cycle drain" --description="Drain AllianceEvent queue between tribute turns inside game/src/games.rs run_day_night_cycle. Schedule trust-shock and ally-death cascades." --type=task --priority=2
bd create --title="Phase 5: Persistence + DTO + frontend scrub" --description="schemas/tribute.surql adds allies/traits/turns_since_last_betrayal. _initial.json bump. shared/ DTO. api/tributes.rs read/write. web tribute_detail.rs loyalty scrub." --type=task --priority=2
bd create --title="Phase 6: Test migration + alliance test suite" --description="Roll out Tribute::test_default across ~60 game tests. Replace BrainPersonality assertions with trait-set checks. Add bucket-tolerance helper. Write multi-tribute alliance suite + api round-trip test." --type=task --priority=2
bd dep add <p2> <p1>; bd dep add <p3> <p2>; bd dep add <p4> <p3>; bd dep add <p5> <p4>; bd dep add <p6> <p5>
```

`bd update <p1> --claim` before starting Phase 1.

---

## File Map

| Path | Action | Responsibility |
|------|--------|----------------|
| `game/src/tributes/traits.rs` | **Create** | Trait enum, conflict table, district pools, threshold math, affinity, geometric_mean, refusers |
| `game/src/tributes/alliances.rs` | **Create** | Formation roll, gate, deciding factor, break triggers, AllianceEvent enum |
| `game/src/tributes/brains.rs` | **Modify** | Remove `BrainPersonality` enum + 9 lookup tables; `PersonalityThresholds` keeps shape; new `compute_thresholds(traits, rng)` |
| `game/src/tributes/mod.rs` | **Modify** | Add `traits`, `allies`, `turns_since_last_betrayal` fields. Remove `loyalty`, `LOYALTY_BREAK_LEVEL`, district filter. Rewrite `pick_target`. Add `test_default` helper. |
| `game/src/tributes/mod.rs` (sub-add) | **Modify** | Add `pub mod traits; pub mod alliances;` declarations |
| `game/src/games.rs` | **Modify** | `Game.alliance_events: Vec<AllianceEvent>`. Drain between tribute turns inside `run_day_night_cycle`. |
| `game/src/config.rs` | **Modify** | Remove `loyalty_break_level`, `max_loyalty` config fields |
| `schemas/tribute.surql` | **Modify** | Add `allies`, `traits`, `turns_since_last_betrayal` fields |
| `migrations/definitions/_initial.json` | **Modify** | Bump version, add release note |
| `shared/src/lib.rs` | **Modify** | Tribute DTO gains `allies: Vec<Uuid>`, `traits: Vec<Trait>` (re-export Trait) |
| `api/src/tributes.rs` | **Modify** | Persist new fields (likely automatic via Serde, verify) |
| `web/src/components/tribute_detail.rs` | **Modify** | Remove `attributes.loyalty` display line |
| `game/src/tributes/test_helpers.rs` (or test mod) | **Create** | `Tribute::test_default()` + `assert_within_tolerance()` |

---

## Phase 1 — `traits.rs` Module

**Files:**
- Create: `game/src/tributes/traits.rs`
- Modify: `game/src/tributes/mod.rs` (add `pub mod traits;` at top)
- Test: inline `#[cfg(test)] mod tests` at bottom of `traits.rs`

### Task 1.1: Scaffold module + Trait enum

- [ ] **Step 1: Create file with enum + module declaration**

`game/src/tributes/traits.rs`:

```rust
//! Tribute trait system. Replaces `BrainPersonality`. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §5.

use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trait {
    // Combat stance
    Aggressive,
    Defensive,
    Cautious,
    Reckless,
    // Social
    Friendly,
    Loyal,
    Paranoid,
    LoneWolf,
    Treacherous,
    // Mental
    Resilient,
    Fragile,
    Cunning,
    Dim,
    // Physical
    Asthmatic,
    Nearsighted,
    Tough,
}
```

Add `pub mod traits;` at top of `game/src/tributes/mod.rs` (next to existing `pub mod brains;`).

- [ ] **Step 2: Add `label()` method**

```rust
impl Trait {
    pub fn label(&self) -> &'static str {
        match self {
            Trait::Aggressive => "aggressive",
            Trait::Defensive => "defensive",
            Trait::Cautious => "cautious",
            Trait::Reckless => "reckless",
            Trait::Friendly => "friendly",
            Trait::Loyal => "loyal",
            Trait::Paranoid => "paranoid",
            Trait::LoneWolf => "a lone wolf",
            Trait::Treacherous => "treacherous",
            Trait::Resilient => "resilient",
            Trait::Fragile => "fragile",
            Trait::Cunning => "cunning",
            Trait::Dim => "dim",
            Trait::Asthmatic => "asthmatic",
            Trait::Nearsighted => "nearsighted",
            Trait::Tough => "tough",
        }
    }
}
```

- [ ] **Step 3: Run `cargo check -p game`** to confirm compile.

Expected: `Finished` clean.

- [ ] **Step 4: Commit**

```bash
jj describe -m "feat(traits): add Trait enum scaffold"
```

### Task 1.2: Alliance affinity + refuser table

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affinity_known_values() {
        assert_eq!(Trait::Friendly.alliance_affinity(), 1.5);
        assert_eq!(Trait::Loyal.alliance_affinity(), 1.4);
        assert_eq!(Trait::Treacherous.alliance_affinity(), 1.2);
        assert_eq!(Trait::Aggressive.alliance_affinity(), 1.0);
        assert_eq!(Trait::Tough.alliance_affinity(), 1.0);
        assert_eq!(Trait::LoneWolf.alliance_affinity(), 0.6);
        assert_eq!(Trait::Paranoid.alliance_affinity(), 0.5);
    }

    #[test]
    fn refusers_membership() {
        assert!(REFUSERS.contains(&Trait::Paranoid));
        assert!(REFUSERS.contains(&Trait::LoneWolf));
        assert!(!REFUSERS.contains(&Trait::Friendly));
    }
}
```

- [ ] **Step 2: Run** `cargo test -p game tributes::traits::tests::affinity_known_values --no-run` — expect compile fail "no method".

- [ ] **Step 3: Implement**

```rust
pub const REFUSERS: &[Trait] = &[Trait::Paranoid, Trait::LoneWolf];

impl Trait {
    pub fn alliance_affinity(&self) -> f64 {
        match self {
            Trait::Friendly => 1.5,
            Trait::Loyal => 1.4,
            Trait::Treacherous => 1.2,
            Trait::LoneWolf => 0.6,
            Trait::Paranoid => 0.5,
            _ => 1.0,
        }
    }
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): alliance affinity + refusers"`

### Task 1.3: `geometric_mean_affinity` helper

- [ ] **Step 1: Failing test**

```rust
#[test]
fn geometric_mean_empty_is_one() {
    assert_eq!(geometric_mean_affinity(&[]), 1.0);
}

#[test]
fn geometric_mean_single_is_identity() {
    assert!((geometric_mean_affinity(&[Trait::Friendly]) - 1.5).abs() < f64::EPSILON * 10.0);
}

#[test]
fn geometric_mean_two_friendly_one_lonewolf() {
    // (1.5 * 1.5 * 0.6)^(1/3) = 1.35^(1/3) ≈ 1.1051709...
    let g = geometric_mean_affinity(&[Trait::Friendly, Trait::Friendly, Trait::LoneWolf]);
    let expected = (1.5_f64 * 1.5 * 0.6).powf(1.0 / 3.0);
    assert!((g - expected).abs() < f64::EPSILON * 10.0);
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Geometric mean of trait affinity values. Returns 1.0 for empty input.
pub fn geometric_mean_affinity(traits: &[Trait]) -> f64 {
    if traits.is_empty() {
        return 1.0;
    }
    let n = traits.len() as f64;
    let product: f64 = traits.iter().map(|t| t.alliance_affinity()).product();
    product.powf(1.0 / n)
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests::geometric` — expect 3 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): geometric_mean_affinity helper"`

### Task 1.4: Conflict table + `conflicts_with`

- [ ] **Step 1: Failing test**

```rust
#[test]
fn conflict_symmetry() {
    let pairs = [
        (Trait::Friendly, Trait::Paranoid),
        (Trait::Loyal, Trait::Treacherous),
        (Trait::Loyal, Trait::LoneWolf),
        (Trait::Aggressive, Trait::Cautious),
        (Trait::Aggressive, Trait::Defensive),
        (Trait::Reckless, Trait::Cautious),
        (Trait::Resilient, Trait::Fragile),
        (Trait::Cunning, Trait::Dim),
    ];
    for (a, b) in pairs {
        assert!(conflicts_with(a, b), "{a:?} should conflict with {b:?}");
        assert!(conflicts_with(b, a), "{b:?} should conflict with {a:?} (symmetry)");
    }
}

#[test]
fn allowed_combos_do_not_conflict() {
    assert!(!conflicts_with(Trait::Friendly, Trait::Treacherous));
    assert!(!conflicts_with(Trait::Paranoid, Trait::LoneWolf));
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
pub const CONFLICTS: &[(Trait, Trait)] = &[
    (Trait::Friendly, Trait::Paranoid),
    (Trait::Loyal, Trait::Treacherous),
    (Trait::Loyal, Trait::LoneWolf),
    (Trait::Aggressive, Trait::Cautious),
    (Trait::Aggressive, Trait::Defensive),
    (Trait::Reckless, Trait::Cautious),
    (Trait::Resilient, Trait::Fragile),
    (Trait::Cunning, Trait::Dim),
];

pub fn conflicts_with(a: Trait, b: Trait) -> bool {
    CONFLICTS.iter().any(|(x, y)| (*x == a && *y == b) || (*x == b && *y == a))
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests::conflict` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): conflict table"`

### Task 1.5: District pools + `pool_for`

- [ ] **Step 1: Failing test**

```rust
#[test]
fn pool_for_returns_correct_pool_per_district() {
    let p1 = pool_for(1);
    assert!(p1.iter().any(|(t, _)| *t == Trait::Loyal));
    let p12 = pool_for(12);
    assert!(p12.iter().any(|(t, _)| *t == Trait::LoneWolf));
}

#[test]
fn pool_for_unknown_district_falls_back() {
    // Districts outside 1..=12 fall back to district 1's pool (or empty —
    // implementation choice; here we assert non-panic).
    let _ = pool_for(99);
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
pub const DISTRICT_1_POOL: &[(Trait, u8)] = &[
    (Trait::Loyal, 4), (Trait::Aggressive, 4), (Trait::Paranoid, 3), (Trait::Tough, 2),
];
pub const DISTRICT_2_POOL: &[(Trait, u8)] = &[
    (Trait::Aggressive, 4), (Trait::Defensive, 4), (Trait::Loyal, 3), (Trait::Tough, 2),
];
pub const DISTRICT_3_POOL: &[(Trait, u8)] = &[
    (Trait::Cunning, 4), (Trait::Cautious, 3), (Trait::Dim, 2),
    (Trait::Nearsighted, 2), (Trait::Asthmatic, 1),
];
pub const DISTRICT_4_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4), (Trait::Aggressive, 3), (Trait::Loyal, 3), (Trait::Tough, 2),
];
pub const DISTRICT_5_POOL: &[(Trait, u8)] = &[
    (Trait::Cunning, 4), (Trait::Cautious, 3), (Trait::Treacherous, 2),
];
pub const DISTRICT_6_POOL: &[(Trait, u8)] = &[
    (Trait::Fragile, 3), (Trait::Friendly, 3), (Trait::Asthmatic, 2), (Trait::Nearsighted, 2),
];
pub const DISTRICT_7_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4), (Trait::Defensive, 3), (Trait::Tough, 3),
];
pub const DISTRICT_8_POOL: &[(Trait, u8)] = &[
    (Trait::Fragile, 2), (Trait::Friendly, 4), (Trait::Loyal, 3), (Trait::Asthmatic, 2),
];
pub const DISTRICT_9_POOL: &[(Trait, u8)] = &[
    (Trait::Cautious, 3), (Trait::Friendly, 3), (Trait::Asthmatic, 2),
];
pub const DISTRICT_10_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 4), (Trait::Defensive, 3), (Trait::Tough, 3),
];
pub const DISTRICT_11_POOL: &[(Trait, u8)] = &[
    (Trait::Loyal, 3), (Trait::Friendly, 4), (Trait::Resilient, 3), (Trait::Tough, 2),
];
pub const DISTRICT_12_POOL: &[(Trait, u8)] = &[
    (Trait::Resilient, 3), (Trait::LoneWolf, 3), (Trait::Cunning, 3), (Trait::Asthmatic, 2),
];

pub fn pool_for(district: u8) -> &'static [(Trait, u8)] {
    match district {
        1 => DISTRICT_1_POOL,
        2 => DISTRICT_2_POOL,
        3 => DISTRICT_3_POOL,
        4 => DISTRICT_4_POOL,
        5 => DISTRICT_5_POOL,
        6 => DISTRICT_6_POOL,
        7 => DISTRICT_7_POOL,
        8 => DISTRICT_8_POOL,
        9 => DISTRICT_9_POOL,
        10 => DISTRICT_10_POOL,
        11 => DISTRICT_11_POOL,
        12 => DISTRICT_12_POOL,
        _ => DISTRICT_1_POOL,
    }
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests::pool` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): district pools"`

### Task 1.6: Trait generation with conflict rejection

- [ ] **Step 1: Failing test**

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn generate_respects_count_when_pool_supports() {
    let mut rng = StdRng::seed_from_u64(42);
    let traits = generate_traits(1, &mut rng); // district 1 has 4 distinct
    assert!(traits.len() >= 2 && traits.len() <= 6);
    // No conflicts
    for i in 0..traits.len() {
        for j in (i + 1)..traits.len() {
            assert!(!conflicts_with(traits[i], traits[j]));
        }
    }
}

#[test]
fn generate_no_duplicates() {
    let mut rng = StdRng::seed_from_u64(7);
    let traits = generate_traits(2, &mut rng);
    let mut sorted: Vec<_> = traits.clone();
    sorted.sort_by_key(|t| *t as u8);
    sorted.dedup();
    assert_eq!(sorted.len(), traits.len());
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Generate a trait set for a tribute in `district`. Rolls 2–6 uniformly,
/// then draws weighted picks from the district pool, rejecting conflicts and
/// duplicates. Stops early if the pool cannot satisfy the count; never spins.
pub fn generate_traits(district: u8, rng: &mut impl Rng) -> Vec<Trait> {
    let pool = pool_for(district);
    let target_count = rng.random_range(2..=6);
    let mut chosen: Vec<Trait> = Vec::with_capacity(target_count);

    // Build a working list of (trait, weight) we may still draw.
    let mut remaining: Vec<(Trait, u8)> = pool.to_vec();

    while chosen.len() < target_count && !remaining.is_empty() {
        // Weighted pick from remaining.
        let total: u32 = remaining.iter().map(|(_, w)| *w as u32).sum();
        if total == 0 { break; }
        let mut roll = rng.random_range(0..total);
        let mut picked_idx: Option<usize> = None;
        for (i, (_, w)) in remaining.iter().enumerate() {
            if roll < *w as u32 {
                picked_idx = Some(i);
                break;
            }
            roll -= *w as u32;
        }
        let idx = picked_idx.expect("weighted pick must succeed when total > 0");
        let (candidate, _) = remaining.remove(idx);

        // Reject if conflicts with anything already chosen.
        if chosen.iter().any(|t| conflicts_with(*t, candidate)) {
            continue;
        }
        chosen.push(candidate);
    }

    chosen
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests::generate` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): generate_traits with conflict rejection"`

### Task 1.7: `ThresholdDelta` + `threshold_modifiers`

Spec §5.5: deltas are framed but not tuned. Pick conservative starter values; balance later.

- [ ] **Step 1: Failing test**

```rust
#[test]
fn threshold_delta_aggressive_lowers_health_threshold() {
    let d = Trait::Aggressive.threshold_modifiers();
    // Aggressive should bias toward fighting → lower health-flee threshold
    assert!(d.low_health_limit < 0);
}

#[test]
fn threshold_delta_zero_traits_is_identity() {
    let total: ThresholdDelta = [].iter().map(|t: &Trait| t.threshold_modifiers()).sum();
    assert_eq!(total, ThresholdDelta::default());
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Additive deltas applied to `PersonalityThresholds`. `i32` so deltas can be
/// signed; final values clamp to u32 ranges in `compute_thresholds`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ThresholdDelta {
    pub low_health_limit: i32,
    pub mid_health_limit: i32,
    pub low_sanity_limit: i32,
    pub mid_sanity_limit: i32,
    pub high_sanity_limit: i32,
    pub movement_limit: i32,
    pub low_intelligence_limit: i32,
    pub high_intelligence_limit: i32,
    pub psychotic_break_threshold: i32,
}

impl std::ops::Add for ThresholdDelta {
    type Output = ThresholdDelta;
    fn add(self, rhs: Self) -> Self {
        ThresholdDelta {
            low_health_limit: self.low_health_limit + rhs.low_health_limit,
            mid_health_limit: self.mid_health_limit + rhs.mid_health_limit,
            low_sanity_limit: self.low_sanity_limit + rhs.low_sanity_limit,
            mid_sanity_limit: self.mid_sanity_limit + rhs.mid_sanity_limit,
            high_sanity_limit: self.high_sanity_limit + rhs.high_sanity_limit,
            movement_limit: self.movement_limit + rhs.movement_limit,
            low_intelligence_limit: self.low_intelligence_limit + rhs.low_intelligence_limit,
            high_intelligence_limit: self.high_intelligence_limit + rhs.high_intelligence_limit,
            psychotic_break_threshold: self.psychotic_break_threshold + rhs.psychotic_break_threshold,
        }
    }
}

impl std::iter::Sum for ThresholdDelta {
    fn sum<I: Iterator<Item = ThresholdDelta>>(iter: I) -> Self {
        iter.fold(ThresholdDelta::default(), |a, b| a + b)
    }
}

impl Trait {
    pub fn threshold_modifiers(&self) -> ThresholdDelta {
        match self {
            Trait::Aggressive => ThresholdDelta {
                low_health_limit: -5, mid_health_limit: -10,
                low_sanity_limit: -2, psychotic_break_threshold: 2,
                ..Default::default()
            },
            Trait::Defensive => ThresholdDelta {
                low_health_limit: 10, mid_health_limit: 10,
                psychotic_break_threshold: -2,
                ..Default::default()
            },
            Trait::Cautious => ThresholdDelta {
                low_health_limit: 15, mid_health_limit: 15,
                low_sanity_limit: 10, mid_sanity_limit: 10,
                psychotic_break_threshold: -3,
                ..Default::default()
            },
            Trait::Reckless => ThresholdDelta {
                low_health_limit: -10, low_sanity_limit: -10,
                psychotic_break_threshold: 4,
                ..Default::default()
            },
            Trait::Resilient => ThresholdDelta {
                psychotic_break_threshold: -3,
                low_sanity_limit: -3,
                ..Default::default()
            },
            Trait::Fragile => ThresholdDelta {
                psychotic_break_threshold: 3,
                low_sanity_limit: 5,
                ..Default::default()
            },
            Trait::Cunning => ThresholdDelta {
                low_intelligence_limit: -5, high_intelligence_limit: -5,
                ..Default::default()
            },
            Trait::Dim => ThresholdDelta {
                low_intelligence_limit: 10, high_intelligence_limit: 5,
                ..Default::default()
            },
            // Social and physical traits leave thresholds untouched.
            _ => ThresholdDelta::default(),
        }
    }
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::traits::tests::threshold` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(traits): threshold deltas"`

### Task 1.8: Phase 1 self-check

- [ ] Run `cargo clippy -p game --no-deps -- -D warnings`. Expect clean.
- [ ] Run `cargo test -p game tributes::traits::tests`. Expect all passing.
- [ ] `bd update <p1> --status=closed --reason="traits.rs landed"` then `bd update <p2> --claim`.

---

## Phase 2 — `alliances.rs` Module

**Files:**
- Create: `game/src/tributes/alliances.rs`
- Modify: `game/src/tributes/mod.rs` (add `pub mod alliances;`)
- Test: inline `#[cfg(test)] mod tests`

This phase writes pure functions only. They take `&Tribute`/`&[Tribute]`, return decisions + new event variants. Mutation is wired in Phase 3.

### Task 2.1: `AllianceEvent` enum + scaffold

- [ ] **Step 1: Create `game/src/tributes/alliances.rs`**

```rust
//! Tribute alliance formation, breaks, and event queue. See spec
//! `docs/superpowers/specs/2026-04-25-tribute-alliances-design.md` §6–§7.

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllianceEvent {
    BetrayalRecorded { betrayer: Uuid, victim: Uuid },
    DeathRecorded { deceased: Uuid, killer: Option<Uuid> },
}

/// Per-tribute hard cap on direct alliances.
pub const MAX_ALLIES: usize = 5;
/// Base chance per encounter that two tributes form an alliance.
pub const BASE_ALLIANCE_CHANCE: f64 = 0.20;
/// Treacherous betrayal cadence in turns.
pub const TREACHEROUS_BETRAYAL_INTERVAL: u8 = 5;
```

- [ ] **Step 2: Add `pub mod alliances;` to `game/src/tributes/mod.rs`**

- [ ] **Step 3: Run** `cargo check -p game`. Expect clean.

- [ ] **Step 4: Commit** `jj describe -m "feat(alliances): scaffold + AllianceEvent"`

### Task 2.2: `passes_gate` refuser logic

- [ ] **Step 1: Failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tributes::traits::Trait;

    fn t(traits: Vec<Trait>) -> Vec<Trait> { traits }

    #[test]
    fn paranoid_vs_paranoid_blocked() {
        assert!(!passes_gate(&t(vec![Trait::Paranoid]), &t(vec![Trait::Paranoid])));
    }

    #[test]
    fn lonewolf_vs_friendly_blocked_when_only_lonewolf_has_no_positive() {
        // LoneWolf has affinity 0.6 (no positive), Friendly has 1.5.
        // (positive AND positive) = false; (no_refuser AND no_refuser) = false.
        assert!(!passes_gate(&t(vec![Trait::LoneWolf]), &t(vec![Trait::Friendly])));
    }

    #[test]
    fn snake_in_grass_passes_gate() {
        // [Friendly, Paranoid] paired with [Loyal]: both have positives.
        assert!(passes_gate(
            &t(vec![Trait::Friendly, Trait::Paranoid]),
            &t(vec![Trait::Loyal]),
        ));
    }

    #[test]
    fn empty_traits_pass_gate() {
        assert!(passes_gate(&t(vec![]), &t(vec![])));
    }
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
use crate::tributes::traits::{REFUSERS, Trait};

pub fn passes_gate(self_traits: &[Trait], target_traits: &[Trait]) -> bool {
    let has_positive = |ts: &[Trait]| ts.iter().any(|x| x.alliance_affinity() >= 1.0);
    let has_refuser = |ts: &[Trait]| ts.iter().any(|x| REFUSERS.contains(x));
    (has_positive(self_traits) && has_positive(target_traits))
        || (!has_refuser(self_traits) && !has_refuser(target_traits))
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::alliances::tests::passes_gate` and the four `*_gate*` tests. Expect 4 passing (zero-trait test passes — empty has no refusers).

- [ ] **Step 5: Commit** `jj describe -m "feat(alliances): passes_gate refuser logic"`

### Task 2.3: `roll_chance` formula

- [ ] **Step 1: Failing test**

```rust
#[test]
fn cap_pen_zero_when_full() {
    let chance = roll_chance(
        &[Trait::Friendly], &[Trait::Friendly],
        true, // same district
        MAX_ALLIES, 0, // self at cap
    );
    assert_eq!(chance, 0.0);
}

#[test]
fn fully_neutral_pair_at_base() {
    // Both empty traits, different district, both 0 allies.
    let chance = roll_chance(&[], &[], false, 0, 0);
    // base 0.20 * 1.0 * 1.0 * 1.0 * 1.0 * 1.0 = 0.20
    assert!((chance - 0.20).abs() < 1e-9);
}

#[test]
fn clamps_at_cap() {
    // Stack everything: Friendly+Friendly, same district, both 0 allies.
    let chance = roll_chance(&[Trait::Friendly], &[Trait::Friendly], true, 0, 0);
    // 0.20 * 1.5 * 1.5 * 1.5 * 1.0 * 1.0 = 0.675
    assert!(chance > 0.6 && chance <= 0.95);
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
use crate::tributes::traits::geometric_mean_affinity;

/// Roll chance per spec §6.2. `self_allies_len` and `target_allies_len` are
/// current `Vec::len()` of each tribute's `allies` list.
pub fn roll_chance(
    self_traits: &[Trait],
    target_traits: &[Trait],
    same_district: bool,
    self_allies_len: usize,
    target_allies_len: usize,
) -> f64 {
    let trait_factor = geometric_mean_affinity(self_traits);
    let target_factor = geometric_mean_affinity(target_traits);
    let district_bonus = if same_district { 1.5 } else { 1.0 };
    let self_cap_pen = (MAX_ALLIES as f64 - self_allies_len as f64) / MAX_ALLIES as f64;
    let target_cap_pen = (MAX_ALLIES as f64 - target_allies_len as f64) / MAX_ALLIES as f64;
    let raw = BASE_ALLIANCE_CHANCE
        * trait_factor
        * target_factor
        * district_bonus
        * self_cap_pen.max(0.0)
        * target_cap_pen.max(0.0);
    raw.clamp(0.0, 0.95)
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::alliances::tests` — expect new tests passing.

- [ ] **Step 5: Commit** `jj describe -m "feat(alliances): roll_chance formula"`

### Task 2.4: `deciding_factor` event-text helper

- [ ] **Step 1: Failing test**

```rust
#[test]
fn deciding_factor_picks_largest_above_one() {
    let f = deciding_factor(&[Trait::Friendly], &[Trait::Loyal], true);
    // district 1.5 > Friendly 1.5 tie → Friendly wins by trait label sort
    // ("friendly" < "same district"); but for this test just assert non-empty.
    assert!(f.is_some());
}

#[test]
fn deciding_factor_none_when_nothing_exceeds_one() {
    let f = deciding_factor(&[], &[], false);
    assert!(f.is_none());
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Returns the human-readable deciding factor for a successful alliance roll,
/// or `None` if no factor exceeded 1.0. Caller formats this into the event
/// string (e.g. "Deciding factor: Peeta is friendly.").
pub fn deciding_factor(
    self_traits: &[Trait],
    target_traits: &[Trait],
    same_district: bool,
) -> Option<DecidingFactor> {
    let mut candidates: Vec<(f64, DecidingFactor)> = Vec::new();
    if same_district {
        candidates.push((1.5, DecidingFactor::SameDistrict));
    }
    for t in self_traits {
        let a = t.alliance_affinity();
        if a > 1.0 {
            candidates.push((a, DecidingFactor::TraitOnSelf(*t)));
        }
    }
    for t in target_traits {
        let a = t.alliance_affinity();
        if a > 1.0 {
            candidates.push((a, DecidingFactor::TraitOnTarget(*t)));
        }
    }
    candidates.sort_by(|(a, df_a), (b, df_b)| {
        b.partial_cmp(a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| df_a.label().cmp(df_b.label()))
    });
    candidates.into_iter().next().map(|(_, df)| df)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecidingFactor {
    SameDistrict,
    TraitOnSelf(Trait),
    TraitOnTarget(Trait),
}

impl DecidingFactor {
    pub fn label(&self) -> &'static str {
        match self {
            DecidingFactor::SameDistrict => "same district",
            DecidingFactor::TraitOnSelf(t) | DecidingFactor::TraitOnTarget(t) => t.label(),
        }
    }
}
```

- [ ] **Step 4: Run** `cargo test -p game tributes::alliances::tests::deciding_factor` — expect 2 passed.

- [ ] **Step 5: Commit** `jj describe -m "feat(alliances): deciding_factor"`

### Task 2.5: `sanity_break_roll` per-ally check

- [ ] **Step 1: Failing test**

```rust
#[test]
fn sanity_break_above_limit_no_break() {
    let mut rng = StdRng::seed_from_u64(1);
    // sanity 50 with limit 20: deficit_ratio 0 → never breaks.
    let breaks = sanity_break_roll(50, 20, &mut rng);
    assert!(!breaks);
}

#[test]
fn sanity_break_far_below_always_breaks() {
    let mut rng = StdRng::seed_from_u64(1);
    // sanity 0 with limit 20: deficit_ratio 1.0 → always breaks.
    let breaks = sanity_break_roll(0, 20, &mut rng);
    assert!(breaks);
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Per-ally sanity-break roll (spec §7.3a). Returns `true` if the symmetric
/// pair should be removed.
pub fn sanity_break_roll(current_sanity: u32, low_sanity_limit: u32, rng: &mut impl rand::Rng) -> bool {
    if current_sanity >= low_sanity_limit {
        return false;
    }
    let deficit_ratio = (low_sanity_limit.saturating_sub(current_sanity) as f64)
        / (low_sanity_limit.max(1) as f64);
    let p = deficit_ratio.clamp(0.0, 1.0);
    rng.random_bool(p)
}
```

- [ ] **Step 4: Run** the 2 new tests — expect passing.

- [ ] **Step 5: Commit** `jj describe -m "feat(alliances): sanity_break_roll"`

### Task 2.6: `trust_shock_roll` (betrayal cascade) helper

- [ ] **Step 1: Failing test**

```rust
#[test]
fn trust_shock_baseline_50pct_at_full_sanity() {
    // Exhaustive seed trial: with deficit 0, p = 0.5; expect mix.
    let mut rng = StdRng::seed_from_u64(1);
    let mut breaks = 0;
    for _ in 0..200 {
        if trust_shock_roll(50, 20, &mut rng) { breaks += 1; }
    }
    // Should be sanity >= limit → return false always.
    assert_eq!(breaks, 0);
}

#[test]
fn trust_shock_below_limit_high_baseline() {
    let mut rng = StdRng::seed_from_u64(7);
    let mut breaks = 0;
    for _ in 0..200 {
        if trust_shock_roll(10, 20, &mut rng) { breaks += 1; }
    }
    // p = 0.5 + 0.5*0.5 = 0.75; ~150 expected, allow wide band.
    assert!(breaks > 100, "expected most to break, got {breaks}");
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Implement**

```rust
/// Trust-shock roll for a betrayal victim (§7.3c1). Same threshold as sanity
/// break, higher baseline (`0.5 + 0.5 * deficit_ratio`).
pub fn trust_shock_roll(current_sanity: u32, low_sanity_limit: u32, rng: &mut impl rand::Rng) -> bool {
    if current_sanity >= low_sanity_limit {
        return false;
    }
    let deficit_ratio = (low_sanity_limit.saturating_sub(current_sanity) as f64)
        / (low_sanity_limit.max(1) as f64);
    let p = (0.5 + 0.5 * deficit_ratio).clamp(0.0, 1.0);
    rng.random_bool(p)
}
```

- [ ] **Step 4: Run** 2 new tests — expect passing.

- [ ] **Step 5: Commit** `jj describe -m "feat(alliances): trust_shock_roll"`

### Task 2.7: Phase 2 self-check

- [ ] `cargo clippy -p game --no-deps -- -D warnings`. Clean.
- [ ] `cargo test -p game tributes::alliances`. All passing.
- [ ] `bd update <p2> --status=closed --reason="alliances.rs pure module landed"`; claim p3.

---

## Phase 3 — Tribute + Brain Integration

**Files:**
- Modify: `game/src/tributes/mod.rs` (add fields, rewrite `pick_target`, remove loyalty + district filter, add `test_default`)
- Modify: `game/src/tributes/brains.rs` (remove `BrainPersonality` enum and 9 lookup tables; add `compute_thresholds` over a trait set)
- Modify: `game/src/config.rs` (remove `loyalty_break_level`, `max_loyalty`)

This is the destructive phase. Cutover, no migration. After this phase the engine compiles only with traits driving thresholds and `allies` driving target filtering.

### Task 3.1: Add fields to `Tribute`

- [ ] **Step 1: Failing test in `game/src/tributes/mod.rs`** (or tests file)

```rust
#[test]
fn tribute_default_has_empty_allies_and_traits() {
    let t = Tribute::test_default();
    assert!(t.allies.is_empty());
    assert!(t.traits.is_empty());
    assert_eq!(t.turns_since_last_betrayal, 0);
}
```

- [ ] **Step 2: Run** `cargo test -p game tributes::tribute_default_has_empty` — expect fail.

- [ ] **Step 3: Add fields to `Tribute` struct**

In `game/src/tributes/mod.rs`, on `pub struct Tribute`:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub allies: Vec<uuid::Uuid>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub traits: Vec<crate::tributes::traits::Trait>,

#[serde(default)]
pub turns_since_last_betrayal: u8,
```

Add `Tribute::test_default()`:

```rust
impl Tribute {
    /// Construct a zero-trait tribute with deterministic fields suitable for
    /// unit tests. Equivalent to the old `BrainPersonality::Balanced`
    /// behavior because zero traits = base thresholds.
    #[cfg(test)]
    pub fn test_default() -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut t = Tribute::default();
        t.id = uuid::Uuid::new_v4();
        t.brain = Brain::default();
        t.allies = Vec::new();
        t.traits = Vec::new();
        t.turns_since_last_betrayal = 0;
        // ... existing default-ish field population, mirroring Tribute::new
        // minus randomized trait generation.
        let _ = rng;
        t
    }
}
```

(The exact body depends on existing `Tribute::default`/`Tribute::new`. Match whatever scaffolding the surrounding tests already rely on.)

- [ ] **Step 4: Run** `cargo test -p game tributes::tribute_default_has_empty` — expect pass.

- [ ] **Step 5: Commit** `jj describe -m "feat(tribute): add allies, traits, turns_since_last_betrayal"`

### Task 3.2: Wire `generate_traits` into `Tribute::new`

- [ ] **Step 1: Read** `game/src/tributes/mod.rs` lines around 110 (`Brain::new_with_random_personality(&mut rng)`).

- [ ] **Step 2: Replace personality call with trait generation:**

```rust
// OLD
let brain = Brain::new_with_random_personality(&mut rng);

// NEW
let traits = crate::tributes::traits::generate_traits(district, &mut rng);
let brain = Brain::default();
```

Then where `Tribute` is constructed, set `traits`, leave `allies = Vec::new()` and `turns_since_last_betrayal = 0`.

- [ ] **Step 3: Run** `cargo check -p game` — expect compile errors next.

- [ ] **Step 4: Commit** `jj describe -m "feat(tribute): generate traits on creation"`

### Task 3.3: Gut `BrainPersonality`

- [ ] **Step 1: Delete** the entire `pub enum BrainPersonality { … }` and all its `impl` blocks in `game/src/tributes/brains.rs`.

- [ ] **Step 2: Delete** `Brain::personality` field and `Brain::new_with_random_personality`.

- [ ] **Step 3: Replace** `BrainPersonality::Balanced.generate_thresholds(rng)` callers with a free function:

```rust
// In brains.rs
use crate::tributes::traits::{Trait, ThresholdDelta};

impl PersonalityThresholds {
    /// Compute thresholds from a trait set with ±20% per-tribute variance.
    pub fn from_traits(traits: &[Trait], rng: &mut impl rand::Rng) -> Self {
        // Base = old Balanced numbers.
        let base = PersonalityThresholds {
            low_health_limit: 20,
            mid_health_limit: 40,
            low_sanity_limit: 10,
            mid_sanity_limit: 20,
            high_sanity_limit: 35,
            movement_limit: 10,
            low_intelligence_limit: 35,
            high_intelligence_limit: 80,
            psychotic_break_threshold: 8,
        };
        let delta: ThresholdDelta = traits.iter().map(|t| t.threshold_modifiers()).sum();
        let apply_var = |val: i32, rng: &mut dyn rand::RngCore| -> u32 {
            let varied = val as f64 * rng.random_range(0.8..=1.2);
            varied.clamp(0.0, 100.0) as u32
        };
        let apply_break = |val: i32, rng: &mut dyn rand::RngCore| -> u32 {
            let varied = val as f64 * rng.random_range(0.8..=1.2);
            varied.clamp(0.0, 20.0) as u32
        };
        PersonalityThresholds {
            low_health_limit: apply_var(base.low_health_limit as i32 + delta.low_health_limit, rng),
            mid_health_limit: apply_var(base.mid_health_limit as i32 + delta.mid_health_limit, rng),
            low_sanity_limit: apply_var(base.low_sanity_limit as i32 + delta.low_sanity_limit, rng),
            mid_sanity_limit: apply_var(base.mid_sanity_limit as i32 + delta.mid_sanity_limit, rng),
            high_sanity_limit: apply_var(base.high_sanity_limit as i32 + delta.high_sanity_limit, rng),
            movement_limit: apply_var(base.movement_limit as i32 + delta.movement_limit, rng),
            low_intelligence_limit: apply_var(base.low_intelligence_limit as i32 + delta.low_intelligence_limit, rng),
            high_intelligence_limit: apply_var(base.high_intelligence_limit as i32 + delta.high_intelligence_limit, rng),
            psychotic_break_threshold: apply_break(base.psychotic_break_threshold as i32 + delta.psychotic_break_threshold, rng),
        }
    }
}
```

- [ ] **Step 4:** Update every caller in `brains.rs` and elsewhere that relied on `BrainPersonality::*` lookup tables (low/mid/high sanity, movement, intelligence, psychotic break) to instead read off the cached `PersonalityThresholds` already on `Brain` (or re-compute from `tribute.traits`).

- [ ] **Step 5:** `cargo check -p game` — fix compile errors iteratively. Expect 6–10 call sites in `brains.rs` to need adjustment.

- [ ] **Step 6: Commit** `jj describe -m "refactor(brains): replace BrainPersonality with trait-derived thresholds"`

### Task 3.4: Remove `loyalty` field + `LOYALTY_BREAK_LEVEL`

- [ ] **Step 1:** In `game/src/tributes/mod.rs`:
  - Delete line 31: `const LOYALTY_BREAK_LEVEL: f64 = 0.25;`
  - Delete line 472: `pub loyalty: u32,` from the attributes struct.
  - Delete loyalty initialization at lines 493 and 515.

- [ ] **Step 2:** In `game/src/config.rs`:
  - Delete `loyalty_break_level` (line 49) and `max_loyalty` (line 58).
  - Delete their default initializers (lines 91 and 100).

- [ ] **Step 3:** Run `cargo check -p game` — fix any remaining `attributes.loyalty` references.

- [ ] **Step 4: Commit** `jj describe -m "refactor: remove loyalty + LOYALTY_BREAK_LEVEL"`

### Task 3.5: Rewrite `pick_target` ally filter

- [ ] **Step 1: Locate** `pick_target` in `game/src/tributes/mod.rs` line 340.

- [ ] **Step 2: Failing test**

```rust
#[test]
fn pick_target_excludes_allies() {
    let mut a = Tribute::test_default();
    let mut b = Tribute::test_default();
    a.allies.push(b.id);
    b.allies.push(a.id);
    // Both in same area, both alive. pick_target should return None or some
    // other tribute, never `b`.
    let targets = vec![b.clone()];
    let pick = a.pick_target(/* construct args matching real signature */);
    // Adjust to actual signature; assertion is "result is not b.id".
    assert!(pick.map(|t| t.id != b.id).unwrap_or(true));
}
```

(Adjust call shape to match `pick_target`'s real signature in `mod.rs:340`.)

- [ ] **Step 3:** Inside `pick_target`, replace the `district != self.district` filter at line 359 with an ally filter:

```rust
// OLD
.filter(|t| t.district != self.district)

// NEW
.filter(|t| !self.allies.contains(&t.id))
```

Delete the loyalty branch at line 373 entirely (the `else if (self.attributes.loyalty as f64 / 100.0) < LOYALTY_BREAK_LEVEL { … }` block).

- [ ] **Step 4: Run** `cargo test -p game tributes::pick_target_excludes_allies`. Expect pass.

- [ ] **Step 5: Commit** `jj describe -m "refactor(pick_target): ally filter replaces district filter and loyalty branch"`

### Task 3.6: Phase 3 self-check

- [ ] `cargo build -p game`. Clean.
- [ ] `cargo clippy -p game --no-deps -- -D warnings`. Clean.
- [ ] Many existing tests will fail (they reference `BrainPersonality` or `loyalty`). That is expected — Phase 6 fixes them. Run `cargo test -p game --no-fail-fast 2>&1 | grep -c FAILED` and write the count into the bead notes for tracking.
- [ ] `bd update <p3> --status=closed --reason="Tribute integration done; existing tests broken pending Phase 6"`; claim p4.

---

## Phase 4 — Cycle Drain in `run_day_night_cycle`

**Files:**
- Modify: `game/src/games.rs` (around line 716 `for tribute in self.tributes.iter_mut()`)
- Modify: `game/src/games.rs` (struct: add `alliance_events: Vec<AllianceEvent>` field at top of `Game`)

### Task 4.1: Add event queue field on `Game`

- [ ] **Step 1: Failing test** in `game/src/games.rs` tests:

```rust
#[test]
fn game_has_empty_alliance_event_queue_on_new() {
    let g = Game::default();
    assert!(g.alliance_events.is_empty());
}
```

- [ ] **Step 2: Run, expect compile fail.**

- [ ] **Step 3: Add field** to `Game`:

```rust
#[serde(default, skip_serializing)]
pub alliance_events: Vec<crate::tributes::alliances::AllianceEvent>,
```

`skip_serializing` because the queue is transient — it lives only inside one `run_day_night_cycle` and is drained before save.

- [ ] **Step 4: Run test** — expect pass.

- [ ] **Step 5: Commit** `jj describe -m "feat(game): add alliance_events queue"`

### Task 4.2: Drain queue between tribute turns

- [ ] **Step 1:** Locate `for tribute in self.tributes.iter_mut()` at `game/src/games.rs:716`.

- [ ] **Step 2:** Refactor that loop body so each iteration ends with:

```rust
// After tribute.act(...):
self.alliance_events.append(&mut tribute.drain_alliance_events());
```

Then between iterations of the outer cycle (or at the end of each tribute's turn before mutable borrow releases — the cleanest place is once per outer step), call:

```rust
self.process_alliance_events(rng);
```

Where `process_alliance_events`:

```rust
fn process_alliance_events(&mut self, rng: &mut impl Rng) {
    use crate::tributes::alliances::{AllianceEvent, trust_shock_roll, sanity_break_roll};

    for ev in self.alliance_events.drain(..).collect::<Vec<_>>() {
        match ev {
            AllianceEvent::BetrayalRecorded { betrayer, victim } => {
                // 1. Ensure the symmetric pair is removed on victim's side.
                if let Some(v) = self.tributes.iter_mut().find(|t| t.id == victim) {
                    v.allies.retain(|x| *x != betrayer);
                    // 2. Schedule trust-shock on victim's NEXT turn.
                    //    Encode as a flag on the tribute (e.g. `pending_trust_shock: bool`),
                    //    consumed at top of victim's next act().
                    v.pending_trust_shock = true;
                }
                // Betrayer is NOT scheduled for trust-shock (spec §7.5).
            }
            AllianceEvent::DeathRecorded { deceased, killer: _ } => {
                // Snapshot allies of the deceased before mutation.
                let allies_of_deceased: Vec<uuid::Uuid> = self
                    .tributes
                    .iter()
                    .find(|t| t.id == deceased)
                    .map(|d| d.allies.clone())
                    .unwrap_or_default();

                for ally_id in allies_of_deceased {
                    if let Some(ally) = self.tributes.iter_mut().find(|t| t.id == ally_id) {
                        let limit = ally.brain.thresholds.low_sanity_limit;
                        let sanity = ally.attributes.sanity;
                        if sanity_break_roll(sanity, limit, rng) {
                            ally.allies.retain(|x| *x != deceased);
                            self.messages.push(GameMessage::ally_death_break(
                                &ally.name,
                                deceased,
                            ));
                        }
                    }
                }
                // Also remove the deceased from everyone's lists (cleanup).
                for t in self.tributes.iter_mut() {
                    t.allies.retain(|x| *x != deceased);
                }
            }
        }
    }
}
```

(Adjust event-message API to match existing `messages` shape on `Game`; `ally_death_break` is a placeholder constructor.)

- [ ] **Step 3:** Add `pending_trust_shock: bool` field to `Tribute` (`#[serde(default)]`, no `skip_serializing_if`). At top of `Tribute::act`, if `pending_trust_shock`, run `trust_shock_roll(...)` against current allies and break each pair on success. Reset flag.

- [ ] **Step 4:** `cargo check -p game` — fix compile.

- [ ] **Step 5: Commit** `jj describe -m "feat(game): drain alliance event queue between turns"`

### Task 4.3: Phase 4 self-check

- [ ] `cargo build -p game`. Clean.
- [ ] `cargo test -p game tributes::alliances tributes::traits`. Phase-1/2 tests still passing.
- [ ] `bd update <p4> --status=closed`; claim p5.

---

## Phase 5 — Persistence + DTO + Frontend Scrub

### Task 5.1: SurrealDB schema additions

- [ ] **Step 1:** Edit `schemas/tribute.surql`. After the existing `attributes` line, insert:

```surql
DEFINE FIELD OVERWRITE allies ON tribute TYPE array<uuid> DEFAULT [];
DEFINE FIELD OVERWRITE traits ON tribute TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE turns_since_last_betrayal ON tribute TYPE int DEFAULT 0;
```

- [ ] **Step 2:** Edit `migrations/definitions/_initial.json`. Bump version to next integer (read existing first). Add release note in description: `"0ug: traits replace BrainPersonality; allies + traits + turns_since_last_betrayal added on tribute. Reset your dev DB."`

- [ ] **Step 3:** Restart SurrealDB locally; run `just dev`; verify migration applies cleanly.

- [ ] **Step 4: Commit** `jj describe -m "feat(schema): tribute alliances + traits"`

### Task 5.2: `shared/` DTO additions

- [ ] **Step 1:** In `shared/src/lib.rs`, locate the tribute DTO struct (used by API responses). Add:

```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub allies: Vec<uuid::Uuid>,

#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub traits: Vec<game::tributes::traits::Trait>, // re-export path may differ
```

(If `shared/` cannot depend on `game/`, copy the `Trait` enum into `shared/` and add a `From<game::Trait>` conversion in the API layer. The simpler v1 path: serialize as `Vec<String>` matching Serde unit-variant repr.)

- [ ] **Step 2:** `cargo build -p shared -p api -p web` — fix imports.

- [ ] **Step 3: Commit** `jj describe -m "feat(shared): tribute DTO gains allies + traits"`

### Task 5.3: API persistence path

- [ ] **Step 1:** Open `api/src/tributes.rs`. Find the SELECT/UPDATE shape.

- [ ] **Step 2:** Verify Serde already round-trips the new fields (most likely yes since the struct uses `#[serde(default)]`). Run an integration test if one exists; otherwise add a smoke test:

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn tribute_persists_allies_and_traits() {
    let app = spawn_test_app().await;
    let game = app.create_game().await;
    let tribute = &game.tributes[0];
    // Verify response contains new fields.
    let body = app.get_tribute(&tribute.identifier).await;
    assert!(body.get("traits").is_some());
    assert!(body.get("allies").is_some());
}
```

- [ ] **Step 3:** Run `cargo test -p api tribute_persists_allies_and_traits -- --test-threads=1`. Expect pass.

- [ ] **Step 4: Commit** `jj describe -m "test(api): tribute persistence round-trip for allies + traits"`

### Task 5.4: Frontend `loyalty` scrub

- [ ] **Step 1:** Open `web/src/components/tribute_detail.rs:316`.

- [ ] **Step 2:** Delete the `dd { "{attributes.loyalty}" }` line and any sibling `dt` label that introduces it.

- [ ] **Step 3:** Run `just web` locally; verify the tribute detail page renders without compile error or visual breakage.

- [ ] **Step 4: Commit** `jj describe -m "refactor(web): remove loyalty display from tribute detail"`

### Task 5.5: Phase 5 self-check

- [ ] `cargo build --workspace`. Clean.
- [ ] `just dev` boots, migration applies, frontend renders tribute detail.
- [ ] `bd update <p5> --status=closed`; claim p6.

---

## Phase 6 — Test Migration + Alliance Test Suite

This is the catch-up phase. Existing ~60 game-crate tests reference `BrainPersonality` and `loyalty`; they will not compile after Phase 3. Fix in batches.

### Task 6.1: Bulk-replace `BrainPersonality::Balanced` constructions

- [ ] **Step 1:** `rg "BrainPersonality::" game/ --files-with-matches` — list affected files.

- [ ] **Step 2:** For each call site, replace pattern:

```rust
// OLD
let mut tribute = Tribute::new(...);
tribute.brain.personality = BrainPersonality::Balanced;
```

```rust
// NEW
let tribute = Tribute::test_default();
```

- [ ] **Step 3:** For tests asserting on personality, replace with trait-set assertions:

```rust
// OLD
assert_eq!(tribute.brain.personality, BrainPersonality::Aggressive);

// NEW
assert!(tribute.traits.contains(&Trait::Aggressive));
```

- [ ] **Step 4:** `cargo test -p game --no-run` and iterate until clean compile.

- [ ] **Step 5: Commit** `jj describe -m "test: migrate BrainPersonality references to traits"` (multiple commits OK as you batch by file).

### Task 6.2: Bucket-tolerance helper

- [ ] **Step 1:** In `game/src/tributes/traits.rs` test module, add:

```rust
#[cfg(test)]
fn assert_within_tolerance(observed: u32, expected: u32, pct: f64) {
    let tol = (expected as f64 * pct).max(1.0);
    let diff = (observed as f64 - expected as f64).abs();
    assert!(
        diff <= tol,
        "observed {observed} not within ±{}% of expected {expected} (diff {diff})",
        pct * 100.0
    );
}
```

- [ ] **Step 2: Failing test for district pool weighting**

```rust
#[test]
fn district_pool_weighting_within_tolerance() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut counts: std::collections::HashMap<Trait, u32> = Default::default();
    for _ in 0..10_000 {
        let traits = generate_traits(1, &mut rng);
        for t in traits {
            *counts.entry(t).or_insert(0) += 1;
        }
    }
    // Pool: Loyal 4, Aggressive 4, Paranoid 3, Tough 2 (total 13).
    // Expected counts depend on draw count per tribute (avg 4 traits).
    // Loose assertion: each pool member observed; relative ratios within ±15%.
    let loyal = *counts.get(&Trait::Loyal).unwrap_or(&0);
    let tough = *counts.get(&Trait::Tough).unwrap_or(&0);
    // Loyal:Tough weight ratio is 4:2 = 2:1. So loyal ≈ 2*tough ±15%.
    assert_within_tolerance(loyal, tough * 2, 0.15);
}
```

- [ ] **Step 3: Run** test, iterate on tolerance/seed if flaky — but flakiness with a fixed seed indicates a real generator bug.

- [ ] **Step 4: Commit** `jj describe -m "test(traits): bucket-tolerance helper + district weighting test"`

### Task 6.3: Multi-tribute alliance integration tests

In `game/src/tributes/alliances.rs` test module, add scenarios:

- [ ] **Test:** Paranoid×Paranoid never allies over 1000 trials.
- [ ] **Test:** Friendly×Friendly same-district allies ≥ 60% over 1000 trials with fixed seed.
- [ ] **Test:** Tribute at `allies.len() == 5` always refuses (`roll_chance` returns 0).
- [ ] **Test:** Sanity drop below limit triggers `sanity_break_roll` true with high probability when deficit large.
- [ ] **Test:** Treacherous timer increments and resets correctly across turns. (May require a small harness driving a `Game` through cycles.)
- [ ] **Test:** Trust-shock cascade: betrayed survivor with 3 other allies removes all 3 pairs on roll success.
- [ ] **Test:** Ally-death cascade: when tribute X dies, X's direct allies roll independently; those rolling true remove the pair with X.
- [ ] **Test:** Shared perception: build two same-area allied tributes, verify ally's perceived items unioned into self's `EnvironmentContext`.

Each test follows the failing-first → implement-or-validate → pass pattern. Most exercise existing pure functions from Phase 2; if a behavior is missing (e.g. shared perception helper), add the helper in `alliances.rs` first and update Phase 2 task list retroactively (this is the only allowed back-edit).

- [ ] **Commit** as you batch: `jj describe -m "test(alliances): formation gate / cap penalty / sanity break / treacherous / cascade / perception"`

### Task 6.4: API integration round-trip test

- [ ] **Test:** in `api/tests/`, create `tribute_alliances_test.rs` with a single test that:
  1. Creates a game (auto-spawns 24 tributes).
  2. Asserts each tribute has `traits` non-empty and `allies` empty initially.
  3. Runs `/api/games/{id}/run-day` (or whatever endpoint drives a cycle).
  4. Asserts at least one tribute has gained or lost an ally entry, OR that the queue mechanism emitted alliance events.

- [ ] **Run** `cargo test -p api tribute_alliances_test -- --test-threads=1`. Expect pass.

- [ ] **Commit** `jj describe -m "test(api): tribute alliances round-trip"`

### Task 6.5: Phase 6 self-check + close `0ug`

- [ ] `cargo test --workspace --no-fail-fast 2>&1 | tail -20`. All passing.
- [ ] `cargo clippy --workspace --no-deps -- -D warnings`. Clean.
- [ ] `just fmt`. Clean.
- [ ] `cargo build --workspace`. Clean.
- [ ] Update spec status from "Draft v2" to "Implemented" in the spec front-matter.
- [ ] File the §12 follow-up beads:

```bash
bd create --title="SeekAlly action" --description="Tribute spends a turn explicitly looking for allies." --type=feature --priority=3
bd create --title="Cross-area ally pings" --description="Tributes know rough location of distant allies." --type=feature --priority=3
bd create --title="Duel feature" --description="Explicit 1v1 combat that bypasses ally filters." --type=feature --priority=3
bd create --title="Trait categories + per-category caps" --description="Limit traits per category (e.g. one combat-stance)." --type=feature --priority=3
bd create --title="Expanded physical/medical traits" --description="More physical and medical trait options (e.g. Charming, Allergic, Limp)." --type=feature --priority=3
bd create --title="Alliance SurrealDB entity" --description="First-class alliance table with history (ties to 5wt/wxn)." --type=feature --priority=3
bd create --title="Player UI: alliance accept/reject" --description="UI for player to accept or reject alliance offers." --type=feature --priority=3
bd create --title="Frontend ally grouping" --description="Visual grouping of allies in tribute list (clique-derived)." --type=feature --priority=3
bd create --title="Group-style narrative events" --description="Clique detection on the alliance graph for group-style story beats." --type=feature --priority=3
```

- [ ] `bd update <p6> --status=closed`; `bd close hangrier_games-0ug --reason="Tribute alliances v1 implemented per spec 2026-04-25."`

- [ ] Open PR per AGENTS.md session-completion protocol.

---

## Self-Review Checklist

Before handing off:

- [ ] Every spec section §1–§12 has at least one task implementing it (or is explicitly out of scope).
- [ ] No "TBD" / "implement appropriate error handling" / "similar to above" placeholders.
- [ ] Every code block compiles in isolation (no undefined types or methods).
- [ ] File paths match real repo paths.
- [ ] `Trait` enum, `AllianceEvent` enum, `MAX_ALLIES` constant referenced consistently across phases.
- [ ] Phase ordering matches dependency: traits → alliances → tribute → cycle → persistence → tests.
- [ ] `szl` (Brain `serde(skip)` bug) is acknowledged but **out of scope** for this plan; spec §8.3 explicitly puts the persisted runtime fields on `Tribute`, sidestepping it.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-04-25-tribute-alliances-implementation.md`. Two execution options:

1. **Subagent-Driven (recommended)** — Dispatch a fresh subagent per phase. Review between phases. Fast iteration.
2. **Inline Execution** — Execute phases in this session using `executing-plans`, batch with checkpoints.

Which approach?
