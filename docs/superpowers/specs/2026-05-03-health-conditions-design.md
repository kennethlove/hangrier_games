# Health Conditions (Afflictions) — v1 Design

**Status:** Approved (brainstorming complete, awaiting implementation plan)
**Author:** klove
**Date:** 2026-05-03
**Related:**
- Foundation for: `phobias` spec (next), `fixations` spec (after phobias)
- Pairs with: `2026-05-03-shelter-hunger-thirst-design.md` (shelter recovery payoff)
- Migrates legacy: `GameEvent::TributeBrokenArm/Leg/Infected` → `MessagePayload::AfflictionAcquired` (closes part of `hangrier_games-b67j`)
- Integration beads: `hangrier_games-hbox` (brain pipeline unification — required), `hangrier_games-uz80` (proptest setup), `hangrier_games-yj9u` (snapshot streams), `hangrier_games-dvd` (sponsor — future hook), `hangrier_games-a3pm`/`gj42` (gamemaker — future hook)

## 1. Problem

Tributes today have a single-slot `TributeStatus` enum that conflates lifecycle states (`Healthy`, `Dead`), transient damage (`Wounded`, `Burned`, `Frozen`), and chronic conditions (`Broken`, `Infected`). Setting one value clobbers another, so a tribute can't be both `Wounded` and `Sick`. Legacy `GameEvent::TributeBrokenArm`/`TributeBrokenLeg`/`TributeInfected` exist as log lines but never mutate tribute state — they're scaffolding without machinery.

This spec replaces transient/chronic `TributeStatus` variants with a multi-slot, anatomy-aware **affliction** system that:
- Stores multiple simultaneous conditions per tribute, keyed by anatomy
- Tracks severity tiers (`Mild`/`Moderate`/`Severe`) with cascade progression
- Has cure paths (item, time-in-shelter) and explicit permanence rules
- Filters visibility to brains by severity (Mild hidden, Moderate co-located, Severe public)
- Lays the foundation for phobias and fixations to register as afflictions of a different kind

After v1, `TributeStatus` shrinks to lifecycle-only: `{ Healthy, RecentlyDead, Dead }`. `Drowned`/`Buried` become future trapped-with-death-roll afflictions (separate spec, not v1).

## 2. Conditions in v1

Six new conditions:

| Kind | Body part | Permanence | Acquisition |
|---|---|---|---|
| `MissingLimb` | `ArmLeft`/`ArmRight`/`LegLeft`/`LegRight` | Permanent | Combat critical, env (maul, blast), spawn (rare) |
| `Blind` | `Eyes` | Permanent | Combat (head crit), env (acid/blast), spawn (rare) |
| `Deaf` | `Ears` | Permanent | Combat (concussion crit), env (blast), spawn (rare) |
| `Broken` | bone-bearing parts | Reversible (splint or shelter rest) | Combat, env (fall, trap) |
| `Infected` | wound site | Reversible (antibiotic or shelter rest, may lose to cascade) | Cascade from untreated `Wounded` |
| `Wounded` | any | Reversible (bandage or rest) | Combat (any hit), env (small) |

Migrated from `TributeStatus` (same shape, now multi-slot):

| Kind | Body part | Permanence | Notes |
|---|---|---|---|
| `Sick` | None | Reversible | Currently rare; future: contagion |
| `Poisoned` | None | Reversible | Item/env source |
| `Burned` | site | Reversible | Env/combat |
| `Frozen` | None | Reversible | Env (cold biome) |
| `Overheated` | None | Reversible | Env (hot biome) |
| `Electrocuted` | None | Reversible | Env (storm/trap) |
| `Mauled(Animal)` | site | Reversible | Env (animal threat) |

Out of scope for v1:
- `Drowned`, `Buried` — need their own escape-roll/death-chance design conversation
- Pregnancy — gated behind a game-creation flag, separate spec
- Addiction, trauma — separate specs, planned next after fixations

## 3. Acquisition paths

Five sources, in order of likelihood:

1. **Spawn-time** — low-probability roll on tribute creation. Any affliction kind is eligible (the Reaping doesn't care): a tribute can enter the arena already missing an arm, blind, splinted from an old break, nursing an unhealed infection, or sick. Severity weighted toward `Mild`/`Moderate` for reversible kinds; permanents are always `Severe` since they don't have tiers in practice. District/backstory weighting deferred until those exist.
2. **Combat** — beat outcomes consult an inflict table. Critical hits and `BreakMidSwing` outcomes have higher weights for severe afflictions. Source recorded as `AfflictionSource::Combat { attacker_id }`.
3. **Environmental** — area events (fire, flood, animal, trap, blast) emit afflictions via the existing `AreaEventKind` pipeline. Source: `AfflictionSource::Environmental(AreaEventKind)`.
4. **Cascade** — untreated reversible afflictions worsen over time. The canonical case: `Wounded(Mild)` → `Wounded(Moderate)` → `Wounded(Severe)` → spawn `Infected(Mild)` (new affliction, not relabel). Source: `AfflictionSource::Cascade { from: AfflictionKey }`.
5. **Future hooks (filed, not v1)**:
   - `AfflictionSource::Sponsor` — sponsor gifts can heal (and rare hostile gifts could inflict)
   - `AfflictionSource::Gamemaker` — sadistic intervention can inflict; merciful can heal

`AfflictionSource` is part of v1 even though sponsor/gamemaker variants won't be produced — that way we don't have to revisit the enum when those systems land.

## 4. Stacking rules (anatomy-aware)

Storage uses `AfflictionKey = (AfflictionKind, Option<BodyPart>)` so the same kind can appear on multiple body parts (`(Broken, Some(ArmLeft))` and `(Broken, Some(ArmRight))` are independent), but the same kind on the same part is one slot.

Resolution function `can_acquire(existing, new) -> AcquireResolution` enforces anatomy:

```rust
enum AcquireResolution {
    Insert,                          // No conflict; add new affliction
    Upgrade(AfflictionKey),          // Replace existing with higher severity
    Supersede(Vec<AfflictionKey>),   // Remove subordinate afflictions; insert new
    Reject(RejectReason),            // Acquisition is nonsensical
}
```

Anatomy rules (non-exhaustive examples):

- `MissingLimb(ArmRight)` **supersedes** `Broken(ArmRight)`, `Wounded(ArmRight)`, `Infected(ArmRight)` (all wound state on a missing limb is meaningless)
- `MissingLimb(X)` **rejects** subsequent `Broken(X)` (can't break a missing bone)
- `Infected(X)` **requires** an existing `Wounded(X)` slot (no random whole-body infection in v1; only via cascade)
- Same-part `Wounded` **upgrades** instead of stacking (Mild + Moderate hit becomes Moderate, not two Mild)
- `Blind` is unique (one `Eyes` slot total, no L/R distinction in v1)
- `Deaf` is unique (one `Ears` slot total)

The full table lives in code (`game/src/tributes/afflictions/anatomy.rs`) and is unit-tested exhaustively.

## 5. Severity tiers

Three tiers: `Mild`, `Moderate`, `Severe`.

Cascade ticks happen during the per-tribute cycle phase. For each reversible affliction:

- If tribute is **sheltered** (per shelter spec): chance to step severity *down* by one (heal toward Mild → cure)
- If tribute is **exposed** and untreated: chance to step severity *up* by one
- At `Severe` and exposed: chance to **spawn a successor affliction** (Wounded→Infection cascade) and/or **trigger death roll** (Severe Infected has per-cycle mortality probability)

Step probabilities are tunable via a new `AfflictionTuning` struct (mirrors `CombatTuning` precedent). Defaults are placeholder; balancing happens after the system is observable.

Tier-scaled effects multiply the base penalty:

| Tier | Stat penalty multiplier | Brain bias weight | Cure difficulty |
|---|---|---|---|
| Mild | 0.5× | low | easy (rest, basic item) |
| Moderate | 1.0× | medium | needs medkit/sponsor-class item |
| Severe | 1.5× | high | sponsor/gamemaker only or extended shelter |

## 6. Mechanical effects

Three effect channels per affliction: **stat modifiers**, **brain bias**, **hard gates**.

| Condition | Stat (base, scaled by tier) | Brain bias | Hard gate (any tier) |
|---|---|---|---|
| Missing arm | -2 atk, -2 def | avoid combat with 2H weapons in inventory | can't equip 2-handed weapon |
| Missing leg | +75% stamina cost on move, -3 escape | prefer shelter / stationary actions | can't enter cliff/swamp terrain |
| Blind | -6 atk, -4 def, -2 forage | strong shelter preference, avoid open terrain | no ranged attacks |
| Deaf | -3 ambush detect | slight isolation preference | none |
| Broken (limb) | -3 atk, -3 def, +50% stamina | refuse combat unless cornered | none |
| Infected | -1 HP/cycle, -1 max stamina | seek water + shelter | none |
| Wounded | -1 atk, -1 def per slot | rest preference | none |

Migrated conditions (Sick/Poisoned/Burned/Frozen/Overheated/Electrocuted/Mauled) carry their existing effect logic; this spec moves them into the new storage and tier system but does not redesign them. Their tier defaults to `Moderate` for all current emit sites (preserves current behavior; future tuning pass can vary).

Stat penalties stack additively across afflictions. Brain bias weights compose multiplicatively in the existing utility scoring (so two "shelter-preference" afflictions don't oversaturate). Hard gates compose by union — any active gate blocks the action.

## 7. Cure / recovery paths

Two cure channels: **items** and **shelter time**.

| Affliction | Item cure | Shelter recovery |
|---|---|---|
| Wounded | Bandage → -1 tier | 1 cycle sheltered → -1 tier |
| Broken | Splint → -1 tier (Mild → cured) | 4 cycles sheltered → -1 tier |
| Infected | Antibiotic → -1 tier (Mild → cured) | 3 cycles sheltered → -1 tier |
| Sick / Poisoned / Burned / Frozen / Overheated / Electrocuted / Mauled | (existing item logic) | (existing) |
| Missing limb / Blind / Deaf | none in v1 | none |

Items are an extension of the existing inventory system; new item kinds (`Bandage`, `Splint`, `Antibiotic`) are added or, where they already exist, wired to the new affliction API. Cure logic is a single function `Tribute::apply_cure(affliction_key, item) -> CureOutcome`.

Shelter recovery integrates with the shelter spec (`2026-05-03-shelter-hunger-thirst-design.md`): each cycle the tribute is sheltered, all reversible afflictions roll for tier reduction. Defaults give shelter a meaningful payoff without trivializing damage.

Ally aid (one tribute spends a turn applying first aid to another) is **out of scope for v1**; filed as follow-up bead.

## 8. Visibility (severity-gated)

Brains see a filtered view of other tributes' afflictions:

- `Severe` — always visible (the limp, the missing arm, the obvious fever)
- `Moderate` — visible only if observer is in the same area in the current cycle
- `Mild` — never visible

Implementation: `Tribute::visible_afflictions_to(observer: &Tribute, ctx: &CycleContext) -> impl Iterator<Item = &Affliction>`. Brain pipeline replaces all current "look at target's status" calls with this.

UI scope:

- Spectator timeline — sees all (timeline events are emitted from the omniscient game state)
- Admin tribute-detail — sees all
- *Future:* a "tribute viewport" that shows only what one tribute knows would respect the brain visibility filter; out of scope for v1

## 9. Data shape

```rust
// shared/src/afflictions.rs (new)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum AfflictionKind {
    Wounded, Broken, Infected, MissingLimb, Blind, Deaf,
    Sick, Poisoned, Burned, Frozen, Overheated, Electrocuted,
    Mauled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum BodyPart {
    ArmLeft, ArmRight, LegLeft, LegRight,
    Torso, Head, Eyes, Ears,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Severity { Mild, Moderate, Severe }

pub type AfflictionKey = (AfflictionKind, Option<BodyPart>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AfflictionSource {
    Spawn,
    Combat { attacker_id: TributeId },
    Environmental(AreaEventKind),
    Cascade { from: AfflictionKey },
    Sponsor,    // reserved
    Gamemaker,  // reserved
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affliction {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub acquired_cycle: u32,
    pub last_progressed_cycle: u32,
    pub source: AfflictionSource,
}

impl Affliction {
    pub fn key(&self) -> AfflictionKey { (self.kind, self.body_part) }
    pub fn is_permanent(&self) -> bool { /* per kind table */ }
    pub fn is_reversible(&self) -> bool { !self.is_permanent() }
}
```

Storage on `Tribute`:

```rust
// game/src/tributes/mod.rs
pub afflictions: BTreeMap<AfflictionKey, Affliction>,
```

`BTreeMap` chosen for deterministic serialization (snapshot test stability) and sorted iteration. Serializes to SurrealDB as a sorted array of objects.

Game logic lives in `game/src/tributes/afflictions/` (new module): `mod.rs`, `anatomy.rs` (resolution rules), `cascade.rs` (per-cycle ticks), `cure.rs` (item + shelter), `effects.rs` (stat modifiers + brain bias + gates), `tuning.rs` (`AfflictionTuning` struct).

## 10. Messages

New `MessagePayload` variants (in `shared/src/messages.rs`, replacing the legacy `GameEvent::TributeBrokenArm/Leg/Infected`):

```rust
MessagePayload::AfflictionAcquired {
    tribute: TributeRef,
    kind: AfflictionKind,
    body_part: Option<BodyPart>,
    severity: Severity,
    source: AfflictionSource,
}

MessagePayload::AfflictionProgressed {
    tribute: TributeRef,
    kind: AfflictionKind,
    body_part: Option<BodyPart>,
    from: Severity,
    to: Severity,
}

MessagePayload::AfflictionHealed {
    tribute: TributeRef,
    kind: AfflictionKind,
    body_part: Option<BodyPart>,
    method: HealMethod,  // Item(ItemRef), ShelterRest
}

MessagePayload::AfflictionCascaded {
    tribute: TributeRef,
    from: AfflictionKey,
    to: AfflictionKey,
}
```

The `kind()` and `involves()` exhaustive matches in `messages.rs` get four new arms (this remains the maintenance burden documented in `hangrier_games-i26a`; that bead's derive-macro fix is the long-term answer).

Legacy `GameEvent::TributeBrokenArm`, `TributeBrokenLeg`, `TributeInfected` are deleted (closes the corresponding portion of `hangrier_games-b67j`).

## 11. Brain pipeline integration

Hard requirement: `hangrier_games-hbox` (brain pipeline unification) lands first. Afflictions register as a single new override layer in the unified pipeline:

```
[psychotic, preferred, survival, stamina, affliction, gamemaker, alliance, consumable] → decide_base
```

`affliction_override(tribute, ctx) -> Option<Action>` consults active afflictions for hard gates first (return early-veto wrapper around base decision) and bias weights second (modify scoring inputs to base decision).

## 12. Combat integration

`CombatBeat` extension: `AttackResult` outcomes can carry an optional `inflicts: Vec<AfflictionDraft>` field. `Tribute::apply_attack_result` walks this list, calling `try_acquire_affliction` for each. Inflict tables live in `combat/inflict_table.rs` (new), keyed by (weapon kind, hit severity). Default tables produce realistic distributions:

- Bare-fists Severe hit on torso: 30% Wounded(Mild)
- Bladed Severe hit on limb: 50% Wounded, 5% MissingLimb (very rare)
- Blunt Critical hit on head: 40% Wounded, 10% Broken(Head), 2% Blind, 1% Deaf
- BreakMidSwing follow-through: rolls on attacker's body part (recoil injury)

Numbers are placeholders; tuning pass after observability.

## 13. Survival & shelter integration

- Pregnancy spec (future) will likely add `Pregnant` as an affliction with stamina/calorie effects.
- Shelter recovery (this spec) is the primary cure path for non-item resolutions.
- Hunger/thirst cascade interaction: `Severe` hunger could spawn `Wounded(Mild, Torso)` (starvation damage) — defer to next iteration; v1 keeps survival → death direct.

## 14. Alliance integration

- A tribute with `Severe` afflictions has reduced alliance-affinity (others see them as a liability).
- An ally with afflictions raises the observer's `concern` emotion (existing emotions spec hook).
- Phobias (next spec) of a specific tribute will also gate alliance proposals; afflictions provide the storage shape phobias will reuse.

## 15. Sponsor / gamemaker integration (filed, not v1)

Filed as integration beads against `hangrier_games-dvd` (sponsor) and the gamemaker series:
- Sponsor gift: `Cure { affliction_key }` and `Heal { tribute_id }`
- Gamemaker intervention: `Inflict { tribute_id, kind, severity }` for sadistic events; `MassCure` for merciful events

`AfflictionSource::Sponsor` and `Gamemaker` variants ship in v1 so producers can be added later without enum churn.

## 16. UI

**Web tribute-detail:** new "Afflictions" section after stats, lists active afflictions with severity badges (mild=yellow, moderate=orange, severe=red), body-part labels, and source. Renders nothing if list is empty. Consider integration with the proposed `TributeDetail` decomposition (`hangrier_games-lzfe`).

**Web timeline cards:** new `affliction_card.rs` consuming the four new payload variants. Reuses the proposed `CardShell` (`hangrier_games-t7g1`); accent color depends on severity.

**Tribute state strip** (`TributeStateStrip`): icon for each affliction, similar to stamina band pips. Compact representation; tooltip shows full list.

## 17. Testing strategy (per `uz80`/`yj9u` foundations)

Unit tests (rstest, in module `#[cfg(test)]`):
- `anatomy.rs::can_acquire`: every (existing, new) pair from a representative grid; assert correct `AcquireResolution`
- `cascade.rs::tick`: mild → moderate → severe; sheltered tributes step down; severe-infected death roll
- `cure.rs::apply_item`: each item kind against each affliction; mismatches return `CureOutcome::NoEffect`
- `effects.rs::stat_modifier_sum`: stack two afflictions; assert additive composition
- `effects.rs::hard_gates`: missing-arm gates 2H weapon equip; missing-leg gates cliff terrain; blind gates ranged

Integration tests (`game/tests/afflictions_*.rs`):
- `acquire_from_combat.rs`: seeded combat scenario produces deterministic affliction acquisition
- `cascade_to_death.rs`: untreated wound → infection → death over N cycles
- `shelter_heals.rs`: same scenario but with shelter access produces survival
- `visibility.rs`: brain decisions differ when target's affliction is Mild (hidden) vs Severe (visible)

Insta snapshots (per `yj9u`):
- Affliction state at end of each test scenario above (BTreeMap serializes deterministically)
- Ordered MessagePayload stream from acquire→progress→heal flows

Proptest properties (per `uz80`):
- Severity is monotonic under cascade ticks for exposed tributes (never spontaneously decreases without item/shelter)
- `can_acquire` is deterministic (same inputs → same `AcquireResolution`)
- Anatomy resolution preserves invariants: no `Broken(X)` coexists with `MissingLimb(X)`; no `Infected(X)` without `Wounded(X)` ancestor
- Stat-modifier sum is bounded (no overflow under any combination of legal afflictions)

## 18. Migration plan (TributeStatus → afflictions)

Single PR, ordered:

1. Land `shared::afflictions` types and `Tribute::afflictions: BTreeMap<...>` field (default empty).
2. Migrate `TributeStatus::{Wounded, Broken, Infected, Sick, Poisoned, Burned, Frozen, Overheated, Electrocuted, Mauled}` producers to call `try_acquire_affliction` instead. Effects continue to flow through legacy paths in parallel for one PR.
3. Migrate consumers (combat, brain, web display) to read from `afflictions`. Delete legacy `TributeStatus` consumer code.
4. Delete the migrated `TributeStatus` variants. `TributeStatus` is now `{ Healthy, RecentlyDead, Dead }`.
5. Delete legacy `GameEvent::TributeBrokenArm/Leg/Infected` and replace producers with `MessagePayload::AfflictionAcquired`.

SurrealDB schema migration: new affliction array column on `tribute` table. Existing in-flight games — none are persisted with the migrated states actively populated (the legacy variants were near-unused), so a default-empty migration is safe. Document this in the migration commit.

## 19. Out of scope (filed as follow-ups)

- `Drowned`/`Buried` as trapped-with-death-roll afflictions
- Pregnancy (game-flag-gated)
- Addiction
- Trauma
- Ally aid (turn-spending first aid)
- District/backstory weighting on spawn-time afflictions
- Tribute-viewport UI respecting brain visibility filter
- Hunger/thirst → wound cascade

## 20. Spec self-review

- **Placeholders**: tuning numbers (probabilities, item cure tiers, inflict-table weights) are explicit defaults to be tuned post-observability — flagged in §5/§7/§12.
- **Internal consistency**: severity tiers, cascade direction, cure direction, and visibility gating all use the same `Severity` ordering. Anatomy rules in §4 align with permanence table in §2 and effect table in §6.
- **Scope check**: 6 conditions + ~10 migrated. One PR series with the migration plan in §18 is feasible. Pregnancy/addiction/trauma deliberately excluded.
- **Ambiguity**: "sheltered" defers to the shelter spec for definition. Inflict-table contents are placeholders; spec commits to shape, not numbers.
