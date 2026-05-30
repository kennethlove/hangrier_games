# Trap Expansion — Pitfall, Snared, Pinned + Action::SetTrap

**Status:** Draft
**Date:** 2026-05-29
**TrapKind beads:** `eeuz` (Pitfall), `etxv` (Pinned), `v0n2` (Snared)
**Set-Trap bead:** (not yet filed)
**Parent spec:** `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md`
**Hard prereq:** trapped afflictions PR3 (narrative messages, brain rescue, spectator disapproval)

---

## 1. Summary

Add four new `TrapKind` variants — Pitfall, SpikedPitfall, Snared, Pinned — and introduce `Action::SetTrap` so tributes can place traps for other tributes.

Pitfall, Snared, and Pinned follow the existing Trapped affliction model (movement-locked, per-cycle attrition, escape roll, rescue). SpikedPitfall is **instant-kill** — triggers `TributeKilled` instead of applying an affliction.

No trap is a death timer. Non-lethal traps deal symbolic attrition damage; SpikedPitfall bypasses the affliction system entirely.

---

## 2. New TrapKinds

### Pitfall
Concealed hole in ground. Tribute falls in. Immobilized but can still attack/use items from pit bottom. Exposed — defense halved (easier target).

| Property | Value |
|----------|-------|
| Escape stat | Strength (climb out) |
| Rescue stat | Strength (pull out) |
| Can fight while trapped | Yes |
| Defense halved | Yes (exposed in hole) |
| Terrain floor | No |
| HP damage | [2, 4, 7] per cycle |
| Mental damage | [2, 4, 6] per cycle |
| Special | Hidden. Perception check to spot before stepping in |

**Severity mapping:**
- Mild: shallow pit (1-2m). Easy climb, low damage
- Moderate: deep pit (3-4m). Harder climb, higher damage
- Severe: very deep pit (5m+). Requires rope or multiple rescuers

**Setting:** Requires digging tool (shovel, pick) — or bare hands at 3× time. Takes 2 cycles (1 if tool available). Camouflage with foliage for concealment.

### SpikedPitfall
Same as Pitfall but spikes at the bottom. **Instant kill.** No affliction — `TributeKilled` with cause. Never enters the affliction system.

| Property | Value |
|----------|-------|
| Trigger resolution | Kill tribute immediately |
| Setting | Same as Pitfall + requires spikes (sharp sticks, metal, glass) |
| Perception to spot | Yes (same as Pitfall concealment) |

**Severity:** None (death is death). Severity parameter is meaningless.
**Setting:** Must have Pitfall prerequisites (digging tool, time) + sharp objects as spikes. Takes 3 cycles total (2 dig + 1 arm). Can set as environmental hazard via AreaEvents (e.g., pre-existing spiked pit in arena).

**Design note:** SpikedPitfall is a `TrapKind` variant but NOT an affliction. Trigger logic emits `TributeKilled` directly. The affliction system never sees it.

### Snared
Tribute caught in rope/net trap. Entangled — can still fight and use items. Not exposed (no defense penalty).

| Property | Value |
|----------|-------|
| Escape stat | Intelligence (cut/untie) |
| Rescue stat | Strength (tear) or Intelligence (untie) |
| Can fight while trapped | Yes |
| Defense halved | No (just tangled, not exposed) |
| Terrain floor | No |
| HP damage | [1, 2, 3] per cycle |
| Mental damage | [3, 5, 8] per cycle |
| Special | Only trap where tribute stays fully combat-capable |

**Severity mapping:**
- Mild: loose net. Slip out quickly
- Moderate: tight ropes. Need tool or time to free self
- Severe: complex bindings. Need sharp tool or help

**Setting:** Requires rope/vine. Takes 1 cycle. Can rig tripwire or pressure plate trigger.

### Pinned
Heavy debris (log, rock, deadfall) crushing the tribute. Most physically dangerous. Completely immobilized — no attacks, no items.

| Property | Value |
|----------|-------|
| Escape stat | Strength (lift/push) |
| Rescue stat | Strength (lift/push) |
| Can fight while trapped | No |
| Defense halved | N/A (cannot act) |
| Terrain floor | No |
| HP damage | [3, 6, 10] per cycle |
| Mental damage | [4, 7, 12] per cycle |
| Special | Can be set as deadfall. Most HP/Mental intensive |

**Severity mapping:**
- Mild: small branch/log. Manageable alone, low damage
- Moderate: medium log/rock. Needs help or multiple attempts
- Severe: large deadfall/rock slab. Requires multiple rescuers or tools

**Setting:** Requires heavy object + trigger mechanism. Deadfall: prop heavy object on stick, attach tripwire. Takes 2 cycles.

---

## 3. Action::SetTrap

New action variant:

```rust
pub enum Action {
    // ... existing ...
    SetTrap {
        trap_kind: Option<TrapKind>,  // None = brain picks best
        target_area: Option<Area>,     // None = current area
    },
    // ... existing (Rescue, etc.) ...
}
```

**Requirements by trap kind:**

| TrapKind | Tool needed | Time | Notes |
|----------|-------------|------|-------|
| Pitfall | Digging tool (shovel, pick) | 2 cycles (1 with tool) | Can use hands, 3× time |
| SpikedPitfall | Digging tool + sharp objects | 3 cycles (2 dig + 1 arm) | Requires Pitfall + spikes |
| Snared | Rope/vine | 1 cycle | Can improvise from cloth |
| Pinned | Rope + heavy object + trigger stick | 2 cycles | Most setup, highest payload |
| Buried | — | — | Environmental only |
| Drowning | — | — | Environmental only |

**Skill check:** Intelligence check (or Survival if skill exists). Low roll → poor concealment (easier to spot). Critical failure → trap triggers on setter.

**Concealment formula:** Base 10 + Intelligence bonus + tool bonus (camouflage kit +5).

---

## 4. Trigger/Detection

### Mechanics
1. Another tribute **enters area** where trap is placed
2. Roll hidden **Perception check** (tribute passive perception) vs **concealment** (trap concealment DC)
3. **Fail** → trigger
4. **Succeed** → tribute spots trap. Can choose to:
   - Disarm (Intelligence check vs concealment DC. Fail = trigger on disarmer)
   - Step around (trap stays)
   - Mark for allies (trap revealed, future Perception auto-succeeds)

### SpikedPitfall trigger resolution
```rust
match trap_kind {
    TrapKind::SpikedPitfall => {
        events.push(TributeEvent::TributeKilled {
            tribute_id,
            cause: DeathCause::SpikedPitfall,
        });
    }
    other => {
        apply_trapped_affliction(tribute_id, other, severity);
    }
}
```

---

## 5. State: PlacedTrap

Traps persist in `Game::placed_traps` until triggered or disarmed. Outlive their setter.

```rust
pub struct PlacedTrap {
    pub id: String,
    pub area: Area,
    pub kind: TrapKind,
    pub severity: Severity,
    pub set_by: TributeRef,
    pub concealment: u32,
    pub triggered: bool,
}
```

**Cap:** 3 traps max per area (total, not per tribute).

**Persistence:** Traps survive setter's death. Rust Belt rules.

---

## 6. Tuning Table Extension

Existing `TrapKindTuning` struct (game/src/tributes/afflictions/trapped.rs):

```rust
pub struct TrapKindTuning {
    pub kind: TrapKind,
    pub hp_damage: [u32; 3],
    pub mental_damage: [u32; 3],
    pub escape_stat: EscapeStat,
    pub rescue_stat: EscapeStat,
    pub allows_terrain_floor: bool,
    pub initial_hp_loss: u32,
    pub progressive_damage_per_cycle: u32,
}
```

**New rows:**

```rust
TrapKindTuning {
    kind: TrapKind::Pitfall,
    hp_damage: [2, 4, 7],
    mental_damage: [2, 4, 6],
    escape_stat: EscapeStat::Strength,
    rescue_stat: EscapeStat::Strength,
    allows_terrain_floor: false,
    initial_hp_loss: 0,
    progressive_damage_per_cycle: 0,
},
TrapKindTuning {
    kind: TrapKind::SpikedPitfall,
    // Not an affliction — tuning row is a stub for trap_tuning_for() completeness.
    // Trigger logic kills directly; these values are never applied.
    hp_damage: [0, 0, 0],
    mental_damage: [0, 0, 0],
    escape_stat: EscapeStat::Strength,
    rescue_stat: EscapeStat::Strength,
    allows_terrain_floor: false,
    initial_hp_loss: 0,
    progressive_damage_per_cycle: 0,
},
TrapKindTuning {
    kind: TrapKind::Snared,
    hp_damage: [1, 2, 3],
    mental_damage: [3, 5, 8],
    escape_stat: EscapeStat::Intelligence,
    rescue_stat: EscapeStat::Strength,
    allows_terrain_floor: false,
    initial_hp_loss: 0,
    progressive_damage_per_cycle: 0,
},
TrapKindTuning {
    kind: TrapKind::Pinned,
    hp_damage: [3, 6, 10],
    mental_damage: [4, 7, 12],
    escape_stat: EscapeStat::Strength,
    rescue_stat: EscapeStat::Strength,
    allows_terrain_floor: false,
    initial_hp_loss: 0,
    progressive_damage_per_cycle: 0,
},
```

---

## 7. Acquisition Paths

Two paths exist in parallel:

**Environmental** (existing): AreaEvents produce Trapped afflictions deterministically via `area_event_to_trap()`. Expansion: Earthquakes → Pinned, Pre-existing sinkholes → Pitfall, Arena hazards → SpikedPitfall. No structural change.

**Player-set** (new): `Action::SetTrap` → `PlacedTrap` in game state → Perception check on area entry → trigger (affliction or death) or spot.

---

## 8. Brain Layer Integration

### Setting traps
- Strategist: strong bias toward trap setting over direct combat
- Sadist: may set traps in high-traffic areas. SpikedPitfall priority
- Compassionate: avoids setting traps (especially SpikedPitfall)
- Default: sets traps passively (low priority) unless strategic situation

### Triggering traps (victim)
- Pitfall: same as existing trapped override (movement locked, defense halved, gates apply)
- Snared: tribute CAN still attack and use items — but cannot move
- Pinned: full immobilization (no attacks, no items)
- SpikedPitfall: death — no brain override needed

### Spotting traps (potential victim)
- Passive Perception check on area entry (automatic, hidden)
- Future: `Action::Search` for active scanning

---

## 9. Spectator Reactions

| Event | Sadist | Compassionate | Other |
|-------|--------|---------------|-------|
| Tribute sets trap | +affinity | -affinity | 0 |
| Tribute sets SpikedPitfall | ++affinity | --affinity | -affinity |
| Tribute triggers trap | +affinity | -affinity | 0 |
| Attacking trapped victim | +affinity (prey) | --affinity (coward) | -affinity |
| Rescuing trapped victim | -affinity | ++affinity | +affinity |

---

## 10. Resolved Questions

| Question | Answer |
|----------|--------|
| Friendly fire — can setter trigger own trap on re-entry? | **No.** Setter auto-passes own traps. Knows their layout. |
| Trap vs trap — two traps in same area? | **First trigger stops.** First trap triggered consumes the "entry event". Remaining traps stay for next entry. |
| SpikedPitfall — player-set or environmental? | **Environmental only.** Arena designer places them. Not in Action::SetTrap. Not a PlacedTrap kind. |
| Disarm — own action or free on spot? | **Free on spot.** Spotted trap can be disarmed without spending an action. |
| Action::Search — needed? | **Yes, added to PR2.** Active scan for traps (extensible to items, hiding tributes). |

---

## 11. PR Breakdown (Draft)

### PR1 — TrapKind variants + tuning + environmental acquisition
- Add Pitfall, SpikedPitfall, Snared, Pinned to `TrapKind` enum
- Add Display impls for new variants
- Add tuning rows to `TRAP_KIND_TABLE`
- Add area_event_to_trap mappings: Earthquake → Pinned, Sinkhole → Pitfall, etc.
- Wire SpikedPitfall trigger logic in area resolution (instant kill, emit TributeKilled)
- SpikedPitfall is environmental-only (no PlacedTrap kind)
- Tests: damage table correctness, escape roll math, spiked kill trigger

### PR2 — Action::SetTrap + Action::Search + PlacedTrap state
- `Action::SetTrap` variant + Display/FromStr/serde
- `Action::Search` variant + Display/FromStr/serde (scans for traps in current area)
- `Game::placed_traps: Vec<PlacedTrap>`
- Setting mechanics (tool requirements, time cost, Intelligence check, concealment)
- Trigger mechanic (Perception vs concealment on area entry)
- Disarming mechanic (free on spot)
- Area cap enforcement (3 max per area)
- Setter auto-passes own traps (no friendly fire)
- First trigger stops (no chain-traps)
- Tests: set trap, trigger trap, disarm trap, area cap, Search reveals trap

### PR3 — Brain layer + spectator integration
- Archetype biases for setting/avoiding traps
- SpikedPitfall trigger in cycle (skip brain, emit TributeKilled)
- Spectator affinity reactions per table above
- Tests: brain trap-setting priority, spectator event emissions
