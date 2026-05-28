# Contagion affliction system — design

Beads: `h8us`
Related: afflictions epic (`4o8a`), trauma spec (`2026-05-04-trauma-design.md`), health-conditions spec (`2026-05-03-health-conditions-design.md`), four-phase day spec (`2026-05-03-four-phase-day-design.md`), shelter spec (`2026-05-03-shelter-hunger-thirst-design.md`)

## 1. Purpose

Model three supernatural contagion afflictions that spread through the arena via bite, exsanguination, fluid contact, or reanimation. Unlike trauma (durable psychological state) and health conditions (physical injury/recovery), contagions are:

- **transmissible** — acquired via contact with an infected tribute;
- **persistent** — no natural cure path (carried until death or game-end);
- **behavior-changing** — each contagion overrides the brain pipeline with forced actions at specific cycles; and
- **observable** — transformations and unusual behavior let other tributes deduce the condition.

Contagions sit beside health conditions and trauma as a peer `AfflictionKind` family sharing the same storage, severity, and visibility infrastructure. They are the "supernatural threat vector" of the arena — an opt-in difficulty layer that turns tributes into monsters.

## 2. Non-goals (v1)

- Player-controllable infection (tributes cannot deliberately infect others; infection is simulation-driven).
- Cure or sponsor-gift reversal (no antivirals, no holy water, no silver bullets in v1).
- Contagion stacking (a tribute carries at most one contagion at a time; infection attempt on already-infected tribute is a no-op).
- Contagion-specific UI beyond severity badge and source metadata (defer to `n52s`).
- Lycanthropy lunar calendar — "full moon" is a simple flag on the phase for v1; no multi-cycle moon phase tracking.
- Zombie intelligence — zombies have no free will, no strategy, no tools; they are a pure hostile-environment mechanic.
- Vampire daylight damage granularity — flat burn per cycle in open areas during Day/Dawn phases; no partial-shade mitigation.

## 3. Relationship to neighboring systems

| System | Contagion's relationship |
|---|---|
| `Trauma` (`2026-05-04`) | Contagion acquisition (being bitten/turned) is a potent trauma source. Trauma pipeline may co-fire when a tribute is infected by another tribute. |
| `Health conditions` (`2026-05-03`) | Contagions are peer affliction kinds reusing the same `Affliction` storage. Vampire sunlight damage applies the `Burned` affliction; combat wounds from lycanthrope/vampire attacks may cascade to `Infected`. |
| `Four-phase day` (`2026-05-03`) | All three contagions are phase-sensitive: Lycanthropy triggers at `Night`, Vampirism forces shelter at `Dawn`/`Day`, Zombie plague is phase-agnostic but day/night visibility affects zombie targeting. |
| `Shelter` (`2026-05-03`) | Vampirism requires shelter during Day/Dawn phases. Lycanthropy transformation happens regardless of shelter (the curse is celestial, not environmental). Zombies do not seek shelter. |
| `Emotions` (`2026-05-02`) | Contagion acquisition spikes fear and anger. Vampire hunger and lycanthrope bloodlust compose with existing emotion modifiers. |
| Alliance system (`2026-04-25`) | Infected tributes are less desirable allies. Zombies do not form alliances. Vampire and lycanthrope severity is visible to allies sharing an area at transformation time. |
| Combat system | Lycanthropy forces Attack during full moon cycles. Vampire Attack is boosted at Night. Zombie Attack is the only action (no other brain layers consulted). |

## 4. Types

Three new `AfflictionKind` variants:

```rust
// shared/src/afflictions/kind.rs

pub enum AfflictionKind {
    // ... existing variants
    Lycanthropy,
    Vampirism,
    ZombiePlague,
}
```

Three metadata structs, one per contagion kind. Each tracks infection origin, progression cycle counter, and contagion-specific state:

```rust
// shared/src/afflictions/contagion.rs

use super::{AfflictionKind, BodyPart, Severity};
use serde::{Deserialize, Serialize};

/// Metadata for Lycanthropy (werewolf curse).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LycanthropyMetadata {
    /// Who bit/infected this tribute (None if spawned with the curse).
    pub infected_by: Option<String>,
    /// Cycle number of the most recent full-moon transformation.
    pub last_transformation_cycle: u32,
    /// Whether the tribute is currently in wolf form this phase.
    pub is_transformed: bool,
}

/// Severity variant for vampirism: acute (recently turned, weak) or chronic (established, full powers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VampireStage {
    Acute,
    Chronic,
}

/// Metadata for Vampirism (nocturnal predator).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VampirismMetadata {
    /// Who exsanguinated/infected this tribute.
    pub infected_by: Option<String>,
    /// Stage of vampirism (Acute → Chronic progression).
    pub stage: VampireStage,
    /// Cycle number when the tribute last fed (resets day-sleep penalty).
    pub last_fed_cycle: u32,
    /// Cycle number this tribute became chronic (None while acute).
    pub chronic_since_cycle: Option<u32>,
}

/// Severity variant for zombie plague: recently dead (fresh, slower) or rotted (faster, aggressive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZombieStage {
    Fresh,
    Rotted,
}

/// Metadata for Zombie Plague (reanimation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZombieMetadata {
    /// Who killed this tribute (the trigger for reanimation).
    pub killed_by: Option<String>,
    /// Stage of decomposition (Fresh → Rotted progression).
    pub stage: ZombieStage,
    /// Cycle number when the tribute reanimated.
    pub reanimated_cycle: u32,
    /// Number of tributes this zombie has killed since reanimation.
    pub kill_count: u32,
}
```

`Affliction` gains a single optional contagion metadata field:

```rust
// shared/src/afflictions/affliction.rs

pub struct Affliction {
    pub kind: AfflictionKind,
    pub body_part: Option<BodyPart>,
    pub severity: Severity,
    pub source: AfflictionSource,
    pub acquired_cycle: u32,
    pub last_progressed_cycle: u32,
    pub trauma_metadata: Option<TraumaMetadata>,
    pub phobia_metadata: Option<PhobiaMetadata>,
    pub fixation_metadata: Option<FixationMetadata>,
    /// Optional contagion-specific metadata.
    /// Only `Some` for `AfflictionKind::Lycanthropy | Vampirism | ZombiePlague`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contagion_metadata: Option<ContagionMetadata>,
}

/// Enum wrapping all three contagion metadata types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum ContagionMetadata {
    Lycanthropy(LycanthropyMetadata),
    Vampirism(VampirismMetadata),
    Zombie(ZombieMetadata),
}
```

### 4.1 AfflictionSource integration

All three contagions use the existing `AfflictionSource::Combat` when acquired via attack, and a new variant for the infection vector:

```rust
// shared/src/afflictions/source.rs

pub enum AfflictionSource {
    Spawn,
    Combat { attacker_id: String },
    Contagion { from: AfflictionKind, infected_by: String },
    Environmental,
    Cascade { from: AfflictionKey },
    Sponsor,
    Gamemaker,
}
```

`AfflictionSource::Contagion` captures the source affliction kind and the specific tribute who transmitted it. This enables cascade tracking (vampire bite → vampirism) and observer deduction ("X was bitten by a known werewolf").

## 5. Acquisition (infection vectors)

All three contagions are acquired via contact with an infected tribute. The infection pass runs once per cycle, **after** the cycle's combat resolution and **before** the brain pipeline runs. It scans combat events for contact patterns.

### 5.1 Lycanthropy

| Vector | Trigger pattern | Severity on acquisition |
|---|---|---|
| Werewolf bite | Infected werewolf's Attack action succeeds on uninfected tribute; hit rolls in bite damage range | Moderate |
| Fluid contact (rare) | Tribute enters area containing werewolf blood (from previous cycle's combat wound) | Mild |

### 5.2 Vampirism

| Vector | Trigger pattern | Severity on acquisition |
|---|---|---|
| Vampire exsanguination | Infected vampire's Attack action succeeds; tribute's HP drops below 20% from the attack | Moderate (Acute) |
| Vampire bite (non-lethal) | Infected vampire's Attack action succeeds; HP remains above 20% | Mild (Acute) |

### 5.3 Zombie Plague

| Vector | Trigger pattern | Severity on acquisition |
|---|---|---|
| Killed by zombie | Tribute dies in combat where attacker has `AfflictionKind::ZombiePlague` | Severe — tribute dies, reanimates next phase |
| Fluid contact | Tribute enters area containing zombie remains from previous cycle | Mild — slow infection (dies and reanimates after incubation) |

### 5.4 Reanimation (zombie-specific)

When a tribute dies with zombie plague (either Severe acquisition from death-by-zombie or Mild incubation that progresses to death):

1. Tribute is removed from the living tributes set.
2. A new zombie tribute is spawned in the same area at next phase boundary.
3. The zombie carries `AfflictionKind::ZombiePlague` with `ContagionMetadata::Zombie { stage: Fresh, .. }`.
4. The zombie uses a separate brain pipeline (§8.3) that skips all normal layers.
5. Zombies can be permanently destroyed (HP reduced to 0 by any damage source) — this is distinct from the reanimation death.

**Chain reaction potential:** When a zombie kills a tribute, that tribute reanimates as a zombie. If multiple zombies are in an area together, each can kill independently, producing exponential spread until the area is cleared.

### 5.5 Acquisition rules

- **Single-contagion per tribute.** A tribute can carry at most one contagion. Infection attempt on an already-infected tribute is a no-op.
- **Severity floor on transform.** Lycanthropy and vampirism set their lowest severity on acquisition; vampirism always starts `Acute`. Severity can increase via progression (§6).
- **Contagion takes priority over other affliction acquisition.** If a tribute would acquire both a contagion and another affliction from the same combat event, the contagion wins (the supernatural vector supersedes the mundane wound).

## 6. Progression, severity, stage

### 6.1 Lycanthropy severity

| Tier | Name | Effect |
|---|---|---|
| Mild | Moodiness | +1 irritability to emotion rolls, slight sensitivity to loud noises (not implemented in v1). No stat change. |
| Moderate | Partial transformation | +1 Strength, +1 Agility during `Night` phase (non-full-moon nights). Daytime: −1 stamina recovery. |
| Severe | Full werewolf | Full transformation under full moon (§7.1). +3 Strength, +3 Agility, −2 sanity when transformed. Daytime exhaustion: −2 stamina recovery, −1 strength. |

Progression from Moderate → Severe requires 3+ full-moon transformation cycles (the curse deepens each time the wolf emerges). No regression — lycanthropy only progresses forward.

### 6.2 Vampirism stage progression

| Stage | Effect |
|---|---|
| Acute (recently turned) | Day-sleep forced (§7.2). Night buff: +1 Attack, +1 Speed. Water avoidance: forced Rest on water terrain. Sunlight: mild burn (2 damage/cycle). Feeding requirement: must feed every 5 cycles or lose 1 tier of night buff. |
| Chronic (established) | Day-sleep forced. Night buff: +3 Attack, +3 Speed. Water avoidance: forced Rest on water terrain. Sunlight: moderate burn (5 damage/cycle). Feeding: must feed every 10 cycles or lose night buff entirely until fed. |

Acute → Chronic progression: 7 cycles since acquisition. No regression — vampirism only progresses forward.

### 6.3 Zombie stage progression

| Stage | Effect |
|---|---|
| Fresh (recently reanimated) | Speed: −1 (stiff joints). Attack: standard zombie bite. Sanity: 0 (no free will). Target: nearest living thing each cycle. |
| Rotted (decomposed) | Speed: +1 (no pain inhibition). Attack: standard zombie bite +1 bonus. Sanity: 0 (no free will). Target: nearest living thing each cycle (more aggressive — will leave area if no target present). |

Fresh → Rotted progression: 5 cycles since reanimation. No regression — zombies only rot forward.

### 6.4 Cure paths

| Contagion | Cure |
|---|---|
| Lycanthropy | No cure in v1. Tribute carries it until death or game-end. |
| Vampirism | No cure in v1. Tribute carries it until death (sunlight, stakes, HP depletion) or game-end. |
| Zombie Plague | No cure. Zombies are destroyed when HP reaches 0. An undestroyed zombie persists until game-end. |

## 7. Effects and brain overrides

Each contagion produces both always-on stat modifications and phase-gated brain overrides. Overrides insert into the brain pipeline as follows (expanding the unified pipeline from trauma spec §8):

```
[psychotic, preferred, survival, stamina, fixation, phobia, trauma, contagion, affliction, gamemaker, alliance, consumable] → decide_base
```

Contagion slots **between trauma and generic affliction**:

- **Below trauma** because trauma flashback/avoidance should not be suppressed by contagion stat effects. A werewolf with severe trauma can still flashback.
- **Above generic affliction** because forced actions (lycanthropy Attack, zombie Attack) beat generic affliction stat-penalty action selection.

### 7.1 Lycanthropy override

```rust
fn lycanthropy_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    let lycan = tribute.contagion(|c| matches!(c.kind, AfflictionKind::Lycanthropy))?;
    let meta = lycan.contagion_metadata.as_ref().and_then(|c| match c {
        ContagionMetadata::Lycanthropy(m) => Some(m),
        _ => None,
    })?;

    // Full moon check: Night phase AND moon_phase == FullMoon
    if ctx.phase == Phase::Night && ctx.moon_phase == MoonPhase::FullMoon {
        // Force Attack nearest living thing
        if let Some(target) = nearest_living_target(tribute, ctx) {
            return Some(Action::Attack {
                target,
                weapon: None, // natural weapons (claws/teeth)
            });
        }
    }

    // Night buff (non-full-moon): already applied via visible_modifiers
    // Daytime stat penalty: already applied via visible_modifiers

    None
}
```

**Key behaviors:**
- Full-moon transformation overrides all other actions — the werewolf must Attack.
- Stat changes (Strength, Agility, sanity) compose via `tribute.visible_modifiers(ctx)` regardless of override.
- Observers in the same area during a full-moon transformation witness it: the tribute's `is_transformed` flag is visible to co-located tributes.

### 7.2 Vampirism override

```rust
fn vampirism_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    let vamp = tribute.contagion(|c| matches!(c.kind, AfflictionKind::Vampirism))?;
    let meta = vamp.contagion_metadata.as_ref().and_then(|c| match c {
        ContagionMetadata::Vampirism(m) => Some(m),
        _ => None,
    })?;

    // Day-sleep: Dawn or Day phase, not in shelter
    if (ctx.phase == Phase::Dawn || ctx.phase == Phase::Day) && !ctx.area.has_shelter {
        // Sunlight damage (applied as burn affliction)
        // Forced shelter-seeking
        let shelter = nearest_shelter_area(tribute, ctx);
        if let Some(target) = shelter {
            return Some(Action::MoveToArea { target });
        }
        // No shelter reachable: take damage (handled outside this override)
    }

    // Water avoidance
    if ctx.area.terrain == Terrain::Water && meta.stage == VampireStage::Acute {
        return Some(Action::Rest); // forced rest (cannot cross water)
    }
    if ctx.area.terrain == Terrain::Water && meta.stage == VampireStage::Chronic {
        return Some(Action::Rest); // even chronic vampires avoid water
    }

    None
}
```

**Key behaviors:**
- At Dawn/Day phase without shelter: forced shelter-seeking (MoveToArea nearest shelter).
- If no shelter reachable: sunlight damage applied as `AfflictionKind::Burned` (Mild for Acute, Moderate for Chronic per cycle).
- Water terrain: forced Rest (cannot navigate water areas).
- Night buff (+Attack, +Speed) composes via `visible_modifiers`.
- Feeding: if `current_cycle - last_fed_cycle > feeding_threshold`, night buff is suppressed. Feeding happens automatically when Attack action succeeds against a living target in Night phase.

### 7.3 Zombie override

```rust
fn zombie_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    let zombie = tribute.contagion(|c| matches!(c.kind, AfflictionKind::ZombiePlague))?;

    // Zombie has no free will: always attack nearest living thing
    let target = nearest_living_target(tribute, ctx);
    match target {
        Some(t) => Some(Action::Attack {
            target: t,
            weapon: None, // natural weapons (bite)
        }),
        None => {
            // No target in current area
            let meta = zombie.contagion_metadata.as_ref().and_then(|c| match c {
                ContagionMetadata::Zombie(m) => Some(m),
                _ => None,
            })?;
            match meta.stage {
                ZombieStage::Fresh => Some(Action::Rest), // stays put
                ZombieStage::Rotted => {
                    // Roam toward nearest area with living tributes
                    let target_area = nearest_area_with_living(ctx);
                    target_area.map(Action::MoveToArea)
                }
            }
        }
    }
}
```

**Key behaviors:**
- Zombie brain pipeline skips ALL other layers — `zombie_override` runs in place of the normal pipeline when the tribute has `ZombiePlague`.
- No sanity, no emotions, no alliances, no consumable use.
- Attack uses natural weapons (bite) with fixed damage dice.
- Zombies do not use tools, weapons, or items.
- Fresh zombies stay put if no target; Rotted zombies seek out prey.
- Zombie death: when HP reaches 0, the zombie is permanently destroyed (no reanimation). Emit `MessagePayload::ZombieDestroyed`.

## 8. Brain pipeline placement

The unified pipeline (per trauma spec §8) becomes:

```
[psychotic, preferred, survival, stamina, fixation, phobia, trauma, contagion, affliction, gamemaker, alliance, consumable] → decide_base
```

### 8.1 Zombie bypass

When a tribute has `AfflictionKind::ZombiePlague`, the normal brain pipeline is **bypassed entirely**. The `zombie_override` (§7.3) is the only action selection layer. Stat modifiers, emotions, phobias, trauma — none are consulted. The zombie acts as a pure hostile-environment entity.

### 8.2 Lycanthropy placement

Lycanthropy override runs as part of the contagion layer. If the full-moon condition does not fire, the override returns `None` and the pipeline continues to generic affliction modifiers. Non-transformed lycanthropes retain full free will — the curse only compels during the full moon.

### 8.3 Vampirism placement

Vampirism override runs as part of the contagion layer. Day-sleep shelter-seeking and water avoidance are strong overrides that fire whenever their conditions are met. During Night phase without water terrain, the override returns `None` and the vampire acts normally (with night stat buffs).

## 9. Visibility (observer-aware)

Mirrors trauma spec §9. Visibility moments (when other tributes learn about the contagion):

1. **Full-moon transformation** (Lycanthropy Severe) — every co-located tribute at the start of the transforming tribute's turn observes the transformation.
2. **Vampire day-sleep panic** (forced shelter-seeking visible as frantic movement at dawn) — observers can deduce vampirism if they witness a tribute fleeing to shelter at daybreak.
3. **Zombie reanimation** — every co-located tribute sees a dead tribute rise.
4. **Combat with infected** — tribute using supernatural Attack patterns (werewolf claws, vampire bite, zombie bite) reveals the attack style. Observers can deduce the affliction.
5. **Sunlight damage visible** — tributes co-located with a vampire taking sun damage see the burning.

Observer tracking uses the same pattern as trauma spec §9: `observed_by: BTreeSet<TributeId>` on the contagion metadata, with decay at 10 cycles since last observation.

Brain consumer: `tribute.knows_contagion(target_id) -> Option<AfflictionKind>`. Returns the known contagion kind if the observer has witnessed a visibility moment within the decay window.

## 10. Messages

New `MessagePayload` variants:

```rust
MessagePayload::ContagionAcquired   { tribute: TributeRef, kind: AfflictionKind, source: AfflictionSource }
MessagePayload::LycanthropyTransform { tribute: TributeRef, cycle: u32 }
MessagePayload::LycanthropyRevert   { tribute: TributeRef, cycle: u32 }
MessagePayload::VampireSunDamage    { tribute: TributeRef, damage: u32, area: AreaRef }
MessagePayload::VampireFed          { tribute: TributeRef, target: TributeRef, cycle: u32 }
MessagePayload::ZombieReanimation   { tribute: TributeRef, area: AreaRef, cycle: u32 }
MessagePayload::ZombieDestroyed     { tribute: TributeRef, area: AreaRef, killer: Option<TributeRef> }
MessagePayload::ZombieKill          { tribute: TributeRef, victim: TributeRef, area: AreaRef }
MessagePayload::ContagionProgression { tribute: TributeRef, kind: AfflictionKind, from: Severity, to: Severity }
MessagePayload::ContagionObserved   { observer: TributeRef, subject: TributeRef, kind: AfflictionKind }
```

`kind()` and `involves()` exhaustive matches gain these 10 new arms.

## 11. Config gate

All contagion systems are gated on a single config flag:

```rust
// game/src/config.rs

pub struct GameConfig {
    // ... existing fields
    /// Enable contagion afflictions (lycanthropy, vampirism, zombie plague).
    /// Default: false — opt-in for games with supernatural elements.
    pub contagions_enabled: bool,
}
```

Default: `false` (opt-in). When `false`:
- Infection pass does not run.
- Contagion brain overrides are not consulted (pipeline skips the layer).
- No `ContagionAcquired` or related messages are emitted.
- `AfflictionKind::Lycanthropy | Vampirism | ZombiePlague` cannot be assigned to any tribute.

## 12. Alliance integration

Three rules, all gated on `knows_contagion`:

- **Contagion penalty.** Tributes who know about a target's contagion reduce their alliance-affinity score by `contagion_affinity_penalty` (default `−0.25` per known contagion; tunable).
- **Zombie exclusion.** Zombies are not eligible for alliances. Any alliance with a zombie is dissolved when the zombie state is confirmed (reanimation observed).
- **Vampire stability.** Vampire tributes have a reduced alliance stability score: at Chronic stage, the stability reduction increases (the predator drive makes them unreliable).

## 13. Messages (integrations with existing payloads)

Existing message payloads gain contagion awareness:

- `TributeDied { cause: DeathCause::Affliction(AfflictionKind::ZombiePlague) }` when killed by a zombie.
- `TributeDamaged` with `attacker` having a contagion kind — damage source includes "werewolf claws", "vampire bite", "zombie bite" flavor.
- Existing injury/affliction progression payloads are emitted when contagion damage cascades to Burned (sunlight), Infected (bite wounds), etc.

## 14. UI

**Tribute detail (admin view):**

- "Contagion" section after Trauma, when a contagion is present.
- Badge with contagion icon (moon/vial/skull per kind).
- Severity/stage display: "Lycanthropy (Moderate)", "Vampirism (Chronic)", "Zombie Plague (Rotted)".
- Infection source: "Bitten by X" or "Reanimated at cycle N".
- Observer count and list (who has deduced the condition).

**Timeline cards:**

- New `contagion_card.rs` consuming all 10 payloads.
- `ZombieReanimation` is the headline event: green accent for reanimation, red for zombie kill.
- `LycanthropyTransform` / `LycanthropyRevert` are paired cards (transform at dusk, revert at dawn).
- `VampireSunDamage` is a compact damage card (overlay on existing damage display).
- Reuses `CardShell` (`t7g1`).

**Tribute state strip:**

- Distinct icon per contagion kind.
- Transformation indicator (pulsing icon for werewolf during full moon).
- Sun damage indicator for vampires at Dawn/Day.

## 15. Testing strategy

Per `uz80` (proptest) and `yj9u` (snapshot streams).

### 15.1 Unit tests

- Infection vector table: each vector produces the correct contagion kind and starting severity.
- Single-contagion rule: second infection attempt on existing contagion is a no-op.
- Lycanthropy full-moon override: tribute forced to Attack during Night + full moon; free will otherwise.
- Vampire shelter-seeking: Dawn/Day without shelter produces MoveToArea; Night produces no override.
- Vampire sunlight damage: Burned affliction applied correctly per acute/chronic stage.
- Zombie override: both stages Attack nearest target; Fresh rests when no target, Rotted roams.
- Zombie reanimation chain: zombie kills tribute → victim reanimates as Fresh zombie next phase.
- Stage progression: acute→chronic at 7 cycles; fresh→rotted at 5 cycles; no regression.

### 15.2 Integration tests (`game/tests/contagion_*.rs`)

- `werewolf_bite_infects.rs`: werewolf tribute attacks uninfected tribute; bite hits → victim acquires Lycanthropy (Moderate); correct payloads.
- `full_moon_forced_attack.rs`: lycanthrope at Night + full moon forced to Attack; no override on non-full-moon night.
- `vampire_exsanguination_infects.rs`: vampire reduces target to <20% HP → victim acquires Vampirism (Moderate, Acute); correct payloads.
- `vampire_day_shelter_seeking.rs`: vampire at Dawn without shelter → forced MoveToArea to nearest shelter; no shelter reachable → takes Burned damage.
- `vampire_water_avoidance.rs`: vampire in water terrain → forced Rest.
- `zombie_kill_reanimates.rs`: zombie kills tribute → victim reanimates next phase as Fresh zombie; chain reaction test with 3+ zombies.
- `zombie_destroyed.rs`: zombie HP reaches 0 → permanently destroyed; `ZombieDestroyed` emitted.
- `zombie_bypass_normal_brain.rs`: zombie tribute does not consult normal brain layers; always Attacks nearest living thing.
- `single_contagion_noop.rs`: tribute with Lycanthropy attacked by vampire → no second contagion acquired.
- `acute_to_chronic_progression.rs`: vampire survives 7 cycles → becomes Chronic; stat buff upgrades; `ContagionProgression` emitted.
- `fresh_to_rotted_progression.rs`: zombie survives 5 cycles → becomes Rotted; behavior changes; `ContagionProgression` emitted.
- `observers_deduce_contagion.rs`: co-located tribute witnesses full-moon transformation → `knows_contagion` returns correct kind; decay after 10 cycles.
- `contagion_disabled_noop.rs`: `contagions_enabled = false`; infection pass does not run; no messages emitted.

### 15.3 Insta snapshots

- Contagion state on `Tribute` after each integration scenario (metadata incl. infection source, stage, transformation count).
- Ordered `MessagePayload` streams for acquire → transform → damage → reanimate paths.

### 15.4 Proptest properties (per `uz80`)

- **Single-contagion invariant**: across any sequence of infection events, a tribute has at most one contagion affliction from `{Lycanthropy, Vampirism, ZombiePlague}`.
- **No-contagion-on-dead**: dead tributes do not acquire new contagions (except the explicit zombie-reanimation path, which spawns a fresh zombie entity).
- **Contagion-stage monotonic**: stage only progresses forward (Acute→Chronic, Fresh→Rotted), never backward.
- **Zombie kill chain bounded**: a zombie kill produces exactly one new zombie; no auto-compounding within a single phase.
- **Override exclusivity**: contagion override and trauma flashback cannot both fire in the same cycle (contagion wins except when trauma is Severity + no stimulus match).

## 16. Migration / rollout

No data migration required. Three new `AfflictionKind` variants are additive; existing tributes default to no contagion. New `MessagePayload` variants are additive. `AfflictionSource::Contagion` is additive. SurrealDB schema unchanged (affliction object is flexible per health-conditions spec).

Rollout: ship behind `contagions_enabled: bool` config flag on `Game`, default `false`. Infection pass, brain layers, and message emissions all no-op when flag is off.

## 17. PR breakdown

- **PR1** — Types: `AfflictionKind::Lycanthropy | Vampirism | ZombiePlague`, three metadata structs, `ContagionMetadata` enum wrapping them, `contagion_metadata` field on `Affliction`, `AfflictionSource::Contagion` variant. Unit tests for metadata construction, single-contagion rule.
- **PR2** — Infection pipeline. Pass runs after combat resolution each cycle, scanning for infection vectors. Acquires contagion on matching patterns. Reanimation: dead tribute → zombie spawn at next phase boundary. `ContagionAcquired` message emission. Integration tests for all infection vectors.
- **PR3** — Brain layer. `lycanthropy_override`, `vampirism_override`, `zombie_override`. Pipeline placement (between trauma and affliction). Zombie bypass (skips normal pipeline). Stat modifiers via `visible_modifiers`. Stage progression (acute→chronic, fresh→rotted). All 10 message payloads emitted. Integration tests for overrides, forced actions, daylight damage, water avoidance.
- **PR4** — Frontend: tribute-detail "Contagion" section, timeline `contagion_card.rs` consuming all 10 payloads, state-strip contagion icons, spectator-skin integration. Smoke tests.
- **PR5** — Alliance integration: contagion penalty, zombie exclusion, vampire stability reduction. Observer-deduction system (`knows_contagion`).

### Hard prerequisites

- PR1 blocked by **afflictions PR1** (`hangrier_games-lsis`) — needs `AfflictionKind` extensibility + `try_acquire_affliction` + flexible affliction object shape. No other prerequisites.

### Soft dependencies

- **Trauma co-acquisition** (contagion acquisition as trauma source) — gated on trauma PR2 landing. If trauma PR2 has not landed, the co-acquisition is stubbed.
- **Combat flavor integration** (werewolf claws, vampire bite as weapon types) — depends on combat system flexibility. If combat is not extensible, use generic "Attack" flavor.

## 18. Out of scope (filed as follow-ups)

- Cure items (silver bullets, holy water, antivirals) — no sponsor gifts in v1.
- Player-controlled infection / directed transmission.
- Lycanthropy lunar calendar (multi-cycle moon phase tracking).
- Vampire partial-shade sunlight mitigation.
- Zombie intelligence (tools, strategy, items).
- Contagion stacking (multiple contagions on one tribute).
- Contagion-specific sponsor gifts (blood packs for vampires, raw meat for werewolves).
- Zombie horde AI (coordinated movement).

## 19. Spec self-review

- **Placeholders / tunables** (called out for post-observability tuning):
  - `contagions_enabled` = false (opt-in default).
  - Lycanthropy bite damage range for infection threshold.
  - Vampire exsanguination threshold (20% HP).
  - `feeding_threshold`: 5 cycles (Acute), 10 cycles (Chronic).
  - Sunlight damage: 2/cycle (Acute), 5/cycle (Chronic).
  - Stage progression: 7 cycles (acute→chronic), 5 cycles (fresh→rotted).
  - `contagion_affinity_penalty` = −0.25 per known contagion.
  - Observer decay = 10 cycles (matches trauma).
  - Full-moon condition: `MoonPhase::FullMoon` on the cycle context.

- **Internal consistency:**
  - Severity, stage, and metadata conventions match trauma/phobia/fixation specs.
  - Single-contagion-per-tribute consistent throughout (§5.5, §4, §15).
  - Brain pipeline placement (between trauma and affliction) is consistent with trauma spec §8 gradient: phobia → trauma → contagion → affliction is a stable left-to-right gradient from "specific stimulus" → "generalized state" → "supernatural compulsion" → "physical condition".
  - `AfflictionSource::Contagion` extends the existing source enum without breaking existing arms.
  - Three metadata structs use the same `#[serde(default, skip_serializing_if = "Option::is_none")]` pattern as trauma/phobia/fixation metadata on `Affliction`.

- **Scope check:** PR1-PR5 each ~200-400 LOC plus tests, comparable to trauma PRs. Contagion stacking, cure items, and lunar calendar correctly deferred.

- **Ambiguity resolved inline:**
  - "Full moon" defined as `ctx.moon_phase == MoonPhase::FullMoon` at Night phase. No lunar cycle tracking in v1 — full moon is a game-configurable per-cycle flag.
  - "Nearest living thing" for zombie target selection: `ctx.area.tributes` sorted by distance metric (area adjacency if multi-area; co-located tributes if same area).
  - "Shelter" defined per shelter spec: any area with `has_shelter: true`. Vampires seek the nearest such area at Dawn/Day.
  - "Reanimation at next phase boundary" means: tribute dies during current phase processing; the zombie spawn is queued and materializes at the start of the next phase in the same area.

- **Open questions resolved by punt to follow-up:**
  - Whether trait modifiers affect contagion progression (e.g. `StrongWill` slowing vampirism progression) — defer to trait system integration PR.
  - Zombie horde AI (coordinated movement / pack targeting) — defer to future zombie-specific expansion.
  - Whether vampire feeding on a zombie produces any special outcome — defer (likely no, zombies have no blood).
