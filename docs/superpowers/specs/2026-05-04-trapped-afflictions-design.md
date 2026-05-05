# Trapped Afflictions — Design Spec

**Status:** Draft
**Date:** 2026-05-04
**Bead:** `hangrier_games-zzjv`
**Discovered from:** `hangrier_games-4o8a` (afflictions epic)
**Hard prereq:** `hangrier_games-lsis` (afflictions PR1 — types & storage foundation)
**Pairs with:** `hangrier_games-b67j` (TributeStatus legacy retirement — Drowned/Buried variants)
**Soft dep:** `hangrier_games-hbox` (brain pipeline unification, PR2 only)
**Future trap kinds tracked separately:** `eeuz` (Pitfall), `v0n2` (Snared), `etxv` (Pinned), `2y3a` (Bound)

---

## 1. Summary

Replace the existing `TributeStatus::Drowned` and `TributeStatus::Buried` markers with a unified `AfflictionKind::Trapped(TrapKind)` affliction. Trapped tributes are movement-locked, take per-cycle attrition damage, and must escape via a hybrid self-roll + ally-rescue mechanic before HP runs out. The design uses a `TrapKind` sub-enum so future trap types (Pitfall, Snared, Pinned, Bound) plug in without refactoring the brain layer or escape mechanic.

This is the third afflicition family to land (after trauma and addiction), and it pairs with the legacy `TributeStatus` retirement work — the Drowned/Buried variants are deleted as part of PR1.

---

## 2. Goals & Non-Goals

### Goals

- Migrate Drowned/Buried from `TributeStatus` to the affliction system (paired with `b67j`)
- Establish a `TrapKind` sub-enum extension point for future trap types
- Add a hybrid escape mechanic: self-roll (Intelligence-scaled, decays with cycles trapped) + ally rescue (any co-located tribute, Strength-scaled, full-turn cost)
- Add `Action::Rescue` to the action vocabulary
- Gate combat & actions appropriately: movement-locked, defenseless×½, can self-medicate with held consumables
- Stay deterministic at acquisition (AreaEvent magnitude → severity), static during entrapment

### Non-Goals

- **Other trap kinds** (Pitfall, Snared, Pinned, Bound) — separate beads, separate brainstorms
- **PvP capture mechanic** (Bound trap) — needs its own design session
- **Sponsorship disapproval for attacking trapped tributes** — TODO comment + follow-up bead, wired to `dvd` when sponsorship lands
- **Gamemaker-set traps** — `phvn` four-phase substrate work, future
- **Tribute-set traps** (`Action::SetTrap`) — future
- **Reinforcement-on-firing semantic** — being trapped doesn't escalate trauma; this is a transient affliction with a binary escape outcome per cycle, not a chronic one

---

## 3. Conceptual Model

A **Trapped affliction** represents a tribute caught in an environmental hazard from which they may escape, be rescued, or die. Unlike chronic afflictions (trauma, addiction, phobia) which persist across the campaign and modulate decisions, Trapped is acute and time-bounded: it resolves in one of three ways within a few cycles.

**Lifecycle:**

```
AreaEvent (Flood, Earthquake, etc.)
    │
    ▼
try_acquire_affliction(Trapped(kind), severity)
    │
    ▼
[Trapped state — per-cycle loop]
    ├─ Apply per-cycle HP & mental damage (TRAP_KIND_TABLE × severity)
    ├─ Brain layer: skip all action choice; only escape attempt allowed
    ├─ Combat gate: movement-locked, defense halved, self-medicate-only
    ├─ Co-located tributes may take Action::Rescue
    ├─ Roll escape (self-roll + accumulated rescue bonus, capped 0.95)
    │
    ├─► Escape succeeds → remove affliction, emit TrappedEscaped
    ├─► HP hits 0 → emit TributeDiedWhileTrapped, kill tribute
    └─► Otherwise → cycles_trapped += 1, continue
```

**Key properties:**

- **Static severity** — assigned at acquisition by AreaEvent magnitude, never changes until escape/death
- **Cycles_trapped counter** — drives escape-roll decay (harder over time)
- **Escape_progress field** — only used for Severe entrapment with single-rescuer-over-multiple-cycles partial rescues
- **TrapKind-parameterized tuning** — damage rates, terrain hazard floor, escape stat, rescue stat per-kind via `TRAP_KIND_TABLE`

---

## 4. TrapKind Enum

Initial enumeration:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrapKind {
    Drowning,
    Buried,
    // Future: Pitfall, Snared, Pinned, Bound (filed as separate beads)
}
```

**Decision: enumerate only what ships now.** Premature enumeration of Pitfall/Snared/Pinned/Bound creates dead match arms and `unreachable!()` traps. Each future variant lands with its own brainstorm + spec + tuning row.

**TrapKind extension contract** (for future PRs):

To add a new TrapKind:

1. Add the variant to `TrapKind`
2. Add a tuning row to `TRAP_KIND_TABLE`
3. Verify acquisition path (which AreaEvent or Action produces it?)
4. Verify the brain layer's `affliction` slot handles it (it should, parameterically — no per-kind brain code)
5. Verify UI rendering (one card variant per kind)

No changes to the escape mechanic, rescue logic, or combat gates should be required.

---

## 5. AfflictionKind Extension

```rust
pub enum AfflictionKind {
    // ... existing variants (Trauma, Phobia, Addiction kinds, ...)
    Trapped(TrapKind),
}
```

The `Trapped` variant carries its `TrapKind` directly in the enum (small Copy type, ~1 byte). No nested `TrapMetadata` enum branching needed for kind-discrimination.

---

## 6. TrappedMetadata

Runtime state attached to the Affliction via the existing optional metadata extension pattern:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
```

Added to the existing `Affliction` struct (already extended in trauma/addiction work):

```rust
pub struct Affliction {
    pub kind: AfflictionKind,
    pub severity: Severity,
    // ... existing fields ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trapped_metadata: Option<TrappedMetadata>,
}
```

**Affliction metadata-slot proliferation** — this is the fourth optional metadata field on `Affliction` (trauma_metadata, phobia_metadata, addiction_metadata, now trapped_metadata). A retro-extraction to a unified `metadata: Option<AfflictionMetadata>` enum is a known follow-up; tracked outside this spec.

---

## 7. TRAP_KIND_TABLE (per-kind tuning)

```rust
pub struct TrapKindTuning {
    pub kind: TrapKind,
    /// Per-cycle HP damage by severity (Mild, Moderate, Severe).
    pub hp_damage: [u32; 3],
    /// Per-cycle mental damage by severity (Mild, Moderate, Severe).
    pub mental_damage: [u32; 3],
    /// Stat used for the self-escape roll bonus.
    pub escape_stat: TributeStat,  // Intelligence for Drowning, Strength for Buried
    /// Stat used for the rescuer's bonus contribution.
    pub rescue_stat: TributeStat,  // Strength for both, but extension point for future kinds
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
```

**Tuning rationale:**

- **Severe = ~2 cycles survival** for an avg 80-HP tribute (50 HP/cycle × 2 = death by cycle 3 at latest)
- **Moderate = ~3 cycles** (30 × 3 = 90)
- **Mild = ~5 cycles** (15 × 5 = 75) — survivable with luck
- **Mental damage** is small compared to HP but compounds: a tribute who barely survives a Severe trap still has lasting mental scars
- **Drowning escape uses Intelligence** (panic management, holding breath strategically); **Buried escape uses Strength** (digging out, lifting debris)
- **Drowning has terrain floor** (active rapids cap escape at 0.30 until water recedes); Buried doesn't (debris is static once settled)

---

## 8. AreaEvent → Severity Mapping

Hardcoded, deterministic table. Lives alongside the existing `lifecycle.rs:222-230` mapping:

| AreaEvent      | TrapKind | Severity  |
|----------------|----------|-----------|
| `Flood`        | Drowning | Severe    |
| `Earthquake`   | Buried   | Severe    |
| `Avalanche`    | Buried   | Moderate  |
| `Landslide`    | Buried   | Moderate  |
| `Rockslide`    | Buried   | Mild      |

Future AreaEvents (`HeavyRain`, `Rapids`, etc.) extend the table when they ship.

**No RNG at acquisition.** AreaEvent variety is the natural source of severity variance.

**Terrain hazard floor lookup** — for Drowning at acquisition, check the area's current hazard state (active rapids? receding flood?) and cache the floor in `TrappedMetadata::terrain_hazard_floor`. Floor values:

| Area state             | Floor |
|------------------------|-------|
| Active rapids          | 0.30  |
| Receding flood         | 0.50  |
| Standing water         | None  |

(Initial implementation: just `Some(0.30)` for active Flood AreaEvents, `None` otherwise. Refine as Area state model grows.)

---

## 9. Escape Mechanic

Per cycle, after damage application, attempt escape:

```rust
pub fn attempt_escape(
    tribute: &Tribute,
    affliction: &Affliction,
    rescue_bonus: f32,  // sum of all rescuers' contributions this cycle
) -> bool {
    let meta = affliction.trapped_metadata.as_ref().expect("Trapped affliction missing metadata");
    let tuning = trap_tuning_for(affliction.kind.trap_kind());

    let stat_value = tribute.stat(tuning.escape_stat) as f32 / MAX_STAT as f32;
    let base = severity_base(affliction.severity);  // Mild 0.50, Moderate 0.35, Severe 0.20
    let stat_bonus = stat_value * 0.30;             // up to +0.30 from a maxed escape stat
    let decay = (meta.cycles_trapped as f32) * CYCLES_DECAY_PER_CYCLE;  // 0.08/cycle

    let mut roll_target = (base + stat_bonus + rescue_bonus - decay).clamp(0.0, 0.95);

    // Apply terrain hazard floor if present
    if let Some(floor) = meta.terrain_hazard_floor {
        roll_target = roll_target.min(floor);
    }

    let roll: f32 = rng().random_range(0.0..1.0);
    roll <= roll_target
}
```

**Severe + single-rescuer-over-multiple-cycles partial rescue:** if the affliction is Severe and there is exactly one rescuer this cycle, the rescue bonus does NOT apply directly to this cycle's roll. Instead, increment `escape_progress`. When `escape_progress >= 2`, the next cycle's rescue bonus from any single rescuer applies fully (modeling "we've been digging together for two cycles, now they're free"). This makes Severe genuinely require either two simultaneous rescuers or two consecutive cycles of help.

**Constants** (tunable, listed once here for grep-ability):

```rust
const SEVERITY_BASE_MILD: f32 = 0.50;
const SEVERITY_BASE_MODERATE: f32 = 0.35;
const SEVERITY_BASE_SEVERE: f32 = 0.20;
const ESCAPE_STAT_BONUS_MAX: f32 = 0.30;
const CYCLES_DECAY_PER_CYCLE: f32 = 0.08;
const ESCAPE_ROLL_CAP: f32 = 0.95;
```

---

## 10. Rescue Action

New action variant:

```rust
pub enum Action {
    // ... existing ...
    Rescue { target: TributeId },
}
```

**Rescue resolution:**

1. Validate target is co-located with rescuer
2. Validate target has a Trapped affliction
3. Compute rescuer's bonus: `0.25 + (rescuer_strength / MAX_STAT) * 0.30` → range +0.25 to +0.55
4. If target's affliction is Severe and this is the only rescuer this cycle:
   - Increment `escape_progress`; do NOT apply the bonus this cycle
   - Emit `MessagePayload::PartialRescueProgress { rescuer, target, progress, threshold: 2 }`
5. Otherwise:
   - Add bonus to the cycle's accumulated `rescue_bonus` for this target
   - Cap total rescue contribution at +0.80 (prevents 4 maxed-Strength rescuers from trivializing)
   - Emit `MessagePayload::RescueAttempted { rescuer, target, bonus }`
6. Rescuer's turn is consumed (no other action this cycle)

**Rescuer eligibility:** any tribute co-located with the trapped tribute. No alliance gate. Brain decides whether to attempt based on affinity, strategy, and self-preservation (rescuing in active hazards is risky).

**Brain layer integration** (PR2): the affliction layer of the brain pipeline checks for trapped co-located tributes during action selection. Affinity-positive tributes (allies, romantic interests) preferentially rescue; affinity-neutral tributes may rescue based on a "compassion roll" (low base chance, modulated by personality traits if/when those exist); affinity-negative tributes won't rescue (they may attack instead per §11).

---

## 11. Combat & Action Gates

While trapped:

| Capability                          | Allowed? |
|-------------------------------------|----------|
| Move to another area                | No       |
| Initiate combat                     | No       |
| Defend against incoming attacks     | Yes (defense stat halved) |
| Pick up new items                   | No       |
| Swap equipped weapon                | No       |
| Use consumables already in inventory (food, water, health kit, antidote) | Yes |
| Attempt escape (implicit, per cycle) | Yes     |

**Defense halving:** when a trapped tribute is attacked, their effective defense for the resolution is `defense / 2`. Attackers attack normally; the trapped tribute is at a significant disadvantage but not entirely defenseless.

**Spectator disapproval (deferred):** attacking a trapped tribute should incur a sponsorship affinity penalty (audience hates a coward). This is wired in once `dvd` (sponsorship) lands. **TODO marker:**

```rust
// TODO(dvd): apply sponsor_affinity_penalty(attacker, SPONSOR_PENALTY_ATTACK_TRAPPED)
//            when attacking a tribute with any AfflictionKind::Trapped(_)
```

A follow-up bead is filed alongside the PR2 work to track wiring this once sponsorship exists.

---

## 12. Brain Layer Integration

Added to the existing pipeline at the `affliction` slot:

```
[psychotic, preferred, survival, stamina, fixation, phobia, trauma, addiction, AFFLICTION, gamemaker, alliance, consumable]
```

**Trapped-affliction layer behavior:**

- **For the trapped tribute:** override all action choice. Force `Action::Idle` (the escape attempt happens automatically post-action-resolution, not as a chosen action). Emit `MessagePayload::Struggling { tribute, kind, severity, cycles_trapped }` for narration.
- **For co-located non-trapped tributes:** evaluate rescue opportunity. Affinity-positive → high priority `Action::Rescue`. Affinity-neutral → compassion-roll for `Action::Rescue` (default 30% base chance, modulated by personality if available). Affinity-negative → may select `Action::Attack` against the trapped target instead (and incur the deferred spectator penalty when wired).

**Hard ordering note:** Trapped overrides every later layer (gamemaker, alliance, consumable). A trapped tribute cannot be commanded by Gamemakers, cannot honor alliance directives, cannot use a consumable they're not already holding-and-eligible-to-use.

---

## 13. Acquisition Pipeline

Replaces the existing `lifecycle.rs:222-230` `set_status` calls:

**Before** (current code):
```rust
match area_event {
    AreaEvent::Flood => tribute.set_status(TributeStatus::Drowned),
    AreaEvent::Earthquake | AreaEvent::Avalanche | AreaEvent::Landslide | AreaEvent::Rockslide => {
        tribute.set_status(TributeStatus::Buried)
    }
    _ => {}
}
```

**After:**
```rust
let (kind, severity) = match area_event {
    AreaEvent::Flood => (TrapKind::Drowning, Severity::Severe),
    AreaEvent::Earthquake => (TrapKind::Buried, Severity::Severe),
    AreaEvent::Avalanche | AreaEvent::Landslide => (TrapKind::Buried, Severity::Moderate),
    AreaEvent::Rockslide => (TrapKind::Buried, Severity::Mild),
    _ => return,
};

let terrain_floor = if matches!(kind, TrapKind::Drowning) {
    area.active_water_hazard_floor()  // returns Option<f32>
} else {
    None
};

let metadata = TrappedMetadata {
    cycles_trapped: 0,
    escape_progress: 0,
    terrain_hazard_floor: terrain_floor,
};

tribute.try_acquire_affliction(
    AfflictionKind::Trapped(kind),
    severity,
    Some(AfflictionMetadataPayload::Trapped(metadata)),
);
```

(`AfflictionMetadataPayload` is the existing dispatch enum from the affliction foundation; the trapped variant is added in PR1.)

---

## 14. Per-Cycle Damage Application

Replaces the existing `lifecycle.rs:291-305` damage block:

**Before** (current code, simplified):
```rust
if tribute.status == TributeStatus::Drowned {
    tribute.take_damage(DROWNED_DAMAGE);
    tribute.take_mental_damage(DROWNED_MENTAL_DAMAGE);
}
if tribute.status == TributeStatus::Buried {
    tribute.take_damage(BURIED_DAMAGE);
}
// ... death check emits TributeDrowned / TributeKilled
```

**After:**
```rust
for affliction in tribute.afflictions.iter() {
    let AfflictionKind::Trapped(trap_kind) = affliction.kind else { continue };
    let tuning = trap_tuning_for(trap_kind);
    let severity_idx = affliction.severity as usize;
    tribute.take_damage(tuning.hp_damage[severity_idx]);
    tribute.take_mental_damage(tuning.mental_damage[severity_idx]);
}

// Death check (after damage, before escape attempt):
if tribute.hp == 0 && tribute.has_affliction_kind(AfflictionKindDiscriminant::Trapped) {
    let trap_kind = tribute.first_trapped_kind();  // for narration
    emit(MessagePayload::TributeDiedWhileTrapped { tribute: tribute.id, trap_kind });
    tribute.kill();
    continue;  // skip escape attempt
}
```

**Escape attempt** runs after death check — survivors only.

---

## 15. Messages / Events

New `MessagePayload` variants:

```rust
pub enum MessagePayload {
    // ... existing ...

    /// Tribute became trapped this cycle.
    TributeTrapped {
        tribute: TributeId,
        kind: TrapKind,
        severity: Severity,
    },

    /// Per-cycle struggling narration (for trapped tribute).
    Struggling {
        tribute: TributeId,
        kind: TrapKind,
        severity: Severity,
        cycles_trapped: u8,
    },

    /// Self-escape or rescue-assisted escape succeeded.
    TrappedEscaped {
        tribute: TributeId,
        kind: TrapKind,
        cycles_trapped: u8,
        rescued_by: Vec<TributeId>,  // empty if pure self-escape
    },

    /// Single-rescuer partial progress at Severe.
    PartialRescueProgress {
        rescuer: TributeId,
        target: TributeId,
        progress: u8,
        threshold: u8,
    },

    /// A rescue attempt this cycle (whether successful or not).
    RescueAttempted {
        rescuer: TributeId,
        target: TributeId,
        bonus: f32,
    },

    /// Tribute died while trapped (HP attrition).
    TributeDiedWhileTrapped {
        tribute: TributeId,
        kind: TrapKind,
    },
}
```

**Retired:**

- `GameEvent::TributeDrowned` (paired with `b67j`) — replaced by `TributeDiedWhileTrapped { kind: Drowning }`
- `GameOutput::TributeDrowned` — replaced by output-renderer mapping `TributeDiedWhileTrapped` → appropriate text

---

## 16. Save Migration

**Custom Deserialize for `TributeStatus`** (PR1 work):

```rust
impl<'de> Deserialize<'de> for TributeStatus {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw: String = String::deserialize(d)?;
        match raw.as_str() {
            "drowned" => Ok(TributeStatus::__LegacyDrowned),
            "buried"  => Ok(TributeStatus::__LegacyBuried),
            other => other.parse().map_err(serde::de::Error::custom),
        }
    }
}
```

`__LegacyDrowned` / `__LegacyBuried` are private stub variants used only during migration. After save load completes, a one-shot pass walks tributes:

```rust
fn migrate_legacy_trapped_statuses(game: &mut Game) {
    for tribute in game.tributes.iter_mut() {
        let legacy_kind = match tribute.status {
            TributeStatus::__LegacyDrowned => Some(TrapKind::Drowning),
            TributeStatus::__LegacyBuried  => Some(TrapKind::Buried),
            _ => None,
        };

        if let Some(kind) = legacy_kind {
            tribute.status = TributeStatus::Healthy;  // clear legacy marker
            let meta = TrappedMetadata {
                cycles_trapped: 0,  // can't recover from old saves; reset
                escape_progress: 0,
                terrain_hazard_floor: None,  // conservative
            };
            tribute.afflictions.push(Affliction {
                kind: AfflictionKind::Trapped(kind),
                severity: Severity::Severe,  // conservative — matches old "until death" behavior
                trapped_metadata: Some(meta),
                // ... other existing affliction fields default-initialized
            });
        }
    }
}
```

After PR1 lands and a deprecation cycle completes, the legacy variants can be removed entirely (paired with `b67j`'s broader status retirement).

**Test plan for migration:**

- snapshot test: a saved game with `TributeStatus::Drowned` deserializes + migrates to a tribute with `AfflictionKind::Trapped(TrapKind::Drowning)`, severity Severe, fresh metadata
- same for Buried
- integration test: load → migrate → run one cycle → tribute takes Severe Drowning damage (50 HP) and either escapes or dies as expected

---

## 17. Rollout Flag

```rust
pub struct Game {
    // ... existing ...
    pub trapped_afflictions_enabled: bool,  // default true
}
```

When `false`:
- Acquisition pipeline (§13) is a no-op — no Trapped afflictions are produced
- Brain layer Trapped handling is skipped
- Existing trapped afflictions (loaded from saves where the flag was true) still tick down via the damage and escape pipelines (we don't strand tributes mid-trap when the flag flips)

---

## 18. PR Breakdown

### PR1 — Foundation + Migration (~1000-1400 LOC)

**Bead title:** `spec+impl: trapped afflictions PR1 — types, acquisition, escape, migration`

Scope:
- `TrapKind` enum (Drowning, Buried only)
- `AfflictionKind::Trapped(TrapKind)` extension
- `TrappedMetadata` + `Affliction.trapped_metadata` field
- `TRAP_KIND_TABLE` + `TrapKindTuning`
- Acquisition migration (`lifecycle.rs:222-230`) — replace `set_status` with `try_acquire_affliction`
- Per-cycle damage application (`lifecycle.rs:291-305`) — replace status-check with affliction-iteration
- `attempt_escape` helper (no rescue bonus integration yet — pure self-roll path)
- Save migration (custom Deserialize + post-load pass)
- `TributeStatus::Drowned` / `Buried` removal (paired with `b67j`)
- `GameEvent::TributeDrowned` retirement → `MessagePayload::TributeDiedWhileTrapped`
- `MessagePayload::TributeTrapped`, `Struggling`, `TrappedEscaped`, `TributeDiedWhileTrapped`
- `Game::trapped_afflictions_enabled` flag + gating
- Tests:
  - rstest unit: severity → damage table correctness for both kinds
  - rstest unit: `attempt_escape` self-roll math (Mild/Moderate/Severe × varying stats × varying cycles_trapped)
  - rstest unit: terrain hazard floor caps escape roll
  - proptest: escape roll always in [0, 0.95]; never panics
  - insta yaml: AreaEvent → Trapped affliction snapshots for all 5 mappings (Flood, Earthquake, Avalanche, Landslide, Rockslide)
  - insta yaml: save migration snapshot (legacy save → migrated game)
  - integration: full lifecycle (acquire → cycles of damage → death) with insta snapshot of message stream

**Hard prereq:** `lsis` (afflictions PR1)

### PR2 — Brain Integration + Rescue Action + UI (~800-1200 LOC)

**Bead title:** `spec+impl: trapped afflictions PR2 — brain layer, rescue action, combat gates, UI`

Scope:
- `Action::Rescue { target: TributeId }`
- Rescue resolution logic (rescuer bonus computation, partial-rescue accumulator at Severe, cap)
- Brain pipeline `affliction` layer:
  - Trapped tribute → force `Action::Idle`
  - Co-located non-trapped tributes → evaluate `Action::Rescue` priority via affinity
  - Co-located negative-affinity tributes → may select `Action::Attack` against trapped target
- Combat gate:
  - Movement-locked enforcement
  - Defense halving when target is trapped
  - Self-medicate-only consumable gating
- `MessagePayload::PartialRescueProgress`, `RescueAttempted`
- Spectator-disapproval TODO comment (wired in `dvd` follow-up bead)
- UI cards:
  - Trapped affliction badge on tribute view (with TrapKind icon, severity color, cycles_trapped)
  - Struggling/Rescued/Escaped narration in event log
  - Action affordance: "Rescue" button visible on co-located trapped tributes (when manual control exists; for now, brain-only)
- Tests:
  - rstest unit: rescuer bonus math (strength scaling, cap)
  - rstest unit: Severe + 1 rescuer → escape_progress increment, no bonus this cycle
  - rstest unit: Severe + 2 rescuers same cycle → bonus applies, escape_progress unchanged
  - rstest unit: defense halving on trapped target
  - rstest unit: held-consumable use allowed; new-item pickup denied
  - integration: trapped tribute + co-located ally → rescue success path snapshot
  - integration: trapped tribute + negative-affinity attacker → attack-while-trapped damage path snapshot

**Hard prereq:** `lsis` (afflictions PR1), this spec's PR1
**Soft dep:** `hbox` (brain pipeline unification)

### Cut order if PR1 grows too large

1. Drop save migration → ship breaking change (cheap if no production saves)
2. Drop `TributeStatus` retirement → leave variants `#[deprecated]` for a release (defers `b67j`)
3. Drop spectator-disapproval TODO comment → just don't mention `dvd` in spec at all

Do NOT drop Buried (defeats the TrapKind abstraction proof).

---

## 19. Open Questions / Deferred

- **Affliction metadata-slot retro-extraction** — fourth optional metadata field on `Affliction`; track follow-up bead for unification
- **Sponsorship disapproval wiring** — depends on `dvd`; TODO comment + follow-up bead
- **Compassion-roll for affinity-neutral rescues** — base chance 30%; revisit if behavior is too rescue-happy or too cruel
- **Personality trait modulation** — if/when personality system exists, it should bias rescue compassion-roll and attack-while-trapped propensity
- **Manual rescue UI** — for now brain-only; manual player turn UI (`hq6`) lands the affordance later
- **Future TrapKinds** — `eeuz` (Pitfall), `v0n2` (Snared), `etxv` (Pinned), `2y3a` (Bound, P4)

---

## 20. Self-Review Checklist

- [x] No placeholders / unresolved TODOs in spec body (deferred items explicitly tagged §19)
- [x] All numeric constants named and listed in one place per concern (escape constants §9, damage table §7)
- [x] Lifecycle diagram covers all three exit paths (escape, death, continue)
- [x] Brain pipeline ordering documented with explicit slot
- [x] Save migration has explicit test plan
- [x] PR breakdown matches recommended split with cut order
- [x] Hard prereqs and soft deps explicit per PR
- [x] Future extension contract (TrapKind addition) documented §4
- [x] Sponsorship integration deferred with explicit TODO marker text §11
- [x] Internal consistency: severity tuning §7 matches Q2 survival math; escape constants §9 match Q1 hybrid mechanic; rescue bonuses §10 match Q3 strength-scaled+capped rule; combat gates §11 match Q4 halved-defense+self-medicate rule
- [x] No unfounded claims about systems not yet built (`dvd`, personality, manual UI all explicitly deferred)
