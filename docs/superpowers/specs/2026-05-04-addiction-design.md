# Addiction affliction system — design

Beads: `hangrier_games-opzj`
Related: afflictions epic (`4o8a`), phobias spec (`2026-05-03-phobias-design.md`), fixations spec (`2026-05-03-fixations-design.md`), trauma spec (`2026-05-04-trauma-design.md`), sponsorship epic (`hangrier_games-dvd`)

## 1. Purpose

Model substance dependence as a stored, durable affliction acquired through repeated consumable use. Addiction sits as a peer to trauma and phobia in the affliction system: it has its own producer pipeline (the consumable-use hook), its own metadata struct, its own brain layer, and the same observer-aware visibility infrastructure. Unlike trauma (event-triggered) and phobia (stimulus-keyed reaction), addiction is **use-triggered** — consuming an addictive substance probabilistically acquires the affliction, and once acquired the tribute oscillates between **High** (substance recently consumed) and **Withdrawal** (no recent dose) effect modes.

Addiction is both:

- a **stored affliction** (`AfflictionKind::Addiction(Substance)`) carried on the tribute, with severity, decay, observer state, and reinforcement; and
- a **producer pipeline** that hooks into `try_use_consumable` to roll acquisition + dispatch the substance's immediate High effect + reinforce existing addictions.

Cap of 2 active addictions per tribute. Use counts persist across cure (relapse-on-first-use bypasses the acquisition roll).

## 2. Non-goals (v1)

- Cross-substance interaction (alcohol+morphling synergy, stimulant+painkiller stacking).
- Trait modifiers for addiction sensitivity (`Resilient`/`Fragile` susceptibility curves).
- District backstory weighting on initial susceptibility.
- Dedicated addiction-viewport UI (covered by `n52s`; lives in tribute card only).
- Withdrawal-driven theft / PvP escalation beyond the existing brain layers.
- Substance overdose mechanic (use-count-triggered death roll).
- Player-facing tolerance gauge UI beyond the severity badge.
- Addictive food / water (non-consumable items are explicitly non-addictive).

## 3. Relationship to neighboring systems

| System | Addiction's relationship |
|---|---|
| Health-conditions afflictions (`2026-05-03`) | Addiction is a peer affliction kind reusing the same storage, severity, cure, and visibility infrastructure. |
| `Phobias` (`2026-05-03`) | Independent. An Alcohol High temporarily suppresses phobia triggers (§7) — the only direct cross-system override. |
| `Trauma` (`2026-05-04`) | Independent. A Morphling High suppresses all affliction stat penalties including trauma stat penalty (§7). |
| `Fixations` (`2026-05-03`) | Independent in v1 (filed as follow-up: substance-target fixation). |
| Alliance system (`2026-04-25`) | Known addictions reduce alliance-acceptance affinity (§11). Observed Craving creates a small sympathetic bond increase. |
| Consumable / inventory (`game/src/tributes/inventory.rs`) | The `try_use_consumable` hook is the producer trigger. The addiction layer also drives the inventory bias and the substance-search action. |
| Sponsor system (`dvd`, filed) | Reserved Substance variants (Alcohol, Painkiller, Morphling) ship via sponsor gifts; Detox and Therapy gifts cure addictions. |

## 4. Types

```rust
// shared/src/afflictions.rs

pub enum AfflictionKind {
    // ... existing variants from health-conditions, phobias, trauma specs
    Addiction(Substance),
}

pub enum Substance {
    Stimulant,   // yayo, go-juice, adrenaline (v1 reachable)
    Morphling,   // sponsor-gift only (dvd)
    Alcohol,     // sponsor-gift only (dvd)
    Painkiller,  // sponsor-gift only (dvd)
}
```

`Substance` derives `Copy + Clone + Eq + Ord + Hash + Serialize + Deserialize`. The mapping from `ItemType` / consumable definition to `Substance` lives in `game/src/items/mod.rs::Consumable::substance(&self) -> Option<Substance>` (added in PR2). Items returning `None` are non-addictive (health kit, memento, trail mix, food, water).

`Affliction` already carries severity per the health-conditions spec. Addiction gets a metadata side-struct stored on `Affliction` when the kind is `Addiction(_)`:

```rust
pub struct AddictionMetadata {
    pub substance: Substance,                          // duplicates kind payload for ergonomic access
    pub cycles_since_last_use: u32,                    // reset on use, ticked otherwise; drives decay + Withdrawal mode
    pub high_cycles_remaining: u32,                    // counts down each cycle; 0 = Withdrawal mode
    pub use_count_at_acquisition: u32,                 // snapshot for relapse messaging
    pub observed_by: BTreeSet<TributeId>,
    pub observer_seen_cycle: BTreeMap<TributeId, u32>,
}
```

`Affliction` gains `pub addiction_metadata: Option<AddictionMetadata>` (mirrors trauma spec's `trauma_metadata` and phobia spec's `phobia_metadata` extension pattern). `None` for all non-Addiction kinds. See §14 for the migration note that this field is `skip_serializing_if = "Option::is_none"`.

`Tribute` gains a persistent use counter:

```rust
pub struct Tribute {
    // ... existing
    pub addiction_use_count: BTreeMap<Substance, u32>,    // monotonic, never reset
    // ...
}
```

`addiction_use_count` is **never reset by cure**. This is what enables relapse-on-first-use and the use-count-driven acquisition curve. Serialized with `skip_serializing_if = "BTreeMap::is_empty"`.

## 5. Acquisition (producer)

The addiction producer runs **inside `try_use_consumable`**, after the consumable's immediate effect resolves successfully and before the use-event is emitted. There is exactly one producer (the use hook), unlike trauma's five.

### 5.1 Acquisition flow

```
try_use_consumable(tribute, item):
  1. Apply item's immediate effect (existing logic).
  2. Emit MessagePayload::SubstanceUsed { tribute, item, substance } if substance.is_some().
  3. If substance.is_none(): done.
  4. Increment tribute.addiction_use_count[substance].
  5. Branch on tribute's existing Addiction(substance):
     a. Has it AND cured-then-relapse path: NOT possible if it has it; skip to (5b).
     b. Has it: reinforce (§6.1). Refresh High mode (§7). Emit AddictionReinforced.
     c. Does not have it AND addiction_use_count[substance] > 0 from prior cured run:
        -> auto-acquire at Mild bypassing roll. Emit AddictionRelapse { substance, prior_uses }.
     d. Does not have it AND no prior uses (use_count == 1):
        -> roll acquisition (§5.2).
     e. Does not have it AND prior uses but never acquired:
        -> roll acquisition (§5.2).  // not a relapse, just a slow burn
     f. On acquisition + active addiction count == 2:
        -> resist. Emit AddictionResisted { substance, reason: AtCap }. Item effect still applies. No state change.
  6. Refresh High mode in all "use" branches: high_cycles_remaining = high_duration(substance, severity).
```

The relapse short-circuit at step (5c) is the one path that bypasses the roll entirely — once a tribute has *ever* been addicted to a substance and has been cured, the next use auto-acquires at Mild.

### 5.2 Acquisition roll

Per-use probability is a function of `addiction_use_count[substance]` after step (5.4) increment:

| Use # | Base chance |
|---|---|
| 1 | 5% |
| 2 | 15% |
| 3 | 30% |
| 4 | 50% |
| 5+ | 75% |

Substance multiplier (applied multiplicatively, then capped at 95%):

| Substance | Multiplier | Cap |
|---|---|---|
| `Morphling` | 1.5 | 0.95 |
| `Alcohol` | 0.7 | 0.95 |
| `Stimulant` | 1.0 | 0.95 |
| `Painkiller` | 1.0 | 0.95 |

`p_acquire = min(0.95, base_chance × substance_multiplier)`. A single `rng.gen_bool(p_acquire)` decides.

On success: create `Affliction { kind: Addiction(substance), severity: Mild, addiction_metadata: Some(AddictionMetadata { ... }) }`. Emit `AddictionAcquired { tribute, substance, severity: Mild, use_count }`.

On failure: no state change; the immediate substance effect already applied. No payload emitted (failed roll is silent to keep the message stream clean — observability via the SubstanceUsed payload is sufficient).

### 5.3 Cap-at-2 enforcement

If `tribute.active_addictions().count() >= 2` and the roll succeeds (or relapse fires), no addiction is created. Emit `AddictionResisted { substance, reason: AtCap }`. The substance's immediate effect still applies. The use count still increments. Future uses can still acquire (when an existing addiction decays out, the cap frees up).

If the same substance is one of the active two: that path goes through reinforcement (5b), not cap-resist.

## 6. Reinforcement, decay, cure

Addiction uses the **shared traumatic-affliction reinforcement rule** introduced in the phobia spec amendment (`hangrier_games-1uhw`) and applied to fixations (`hangrier_games-wss4`) and trauma (`cdu0`). The single helper lives at `shared/src/afflictions.rs::apply_traumatic_reinforcement` (extracted by whichever PR lands first; addiction PR3 calls it).

### 6.1 Per-cycle reinforcement (use)

When `try_use_consumable` calls into an existing Addiction:

1. **Counter reset.** `cycles_since_last_use = 0`.
2. **High refresh.** `high_cycles_remaining = high_duration(substance, severity)` (see §7 for the tolerance-driven duration table).
3. **Sensitization roll.** Roll `firing_escalation_chance` (default **12%**, tunable in `[10%, 15%]`). On success, severity steps up one tier (`Mild → Moderate → Severe`). At `Severe`, the roll is performed but no change is recorded and no `AddictionEscalated` is emitted.
4. **Emit `AddictionReinforced`** unconditionally; emit `AddictionEscalated { from, to, substance }` on tier change.

### 6.2 Decay

A cycle in which the tribute does **not** consume the addiction's substance:

1. Decrement `high_cycles_remaining` toward 0 (Withdrawal mode begins when it hits 0; this happens inside the High → Withdrawal transition the same cycle the counter expires).
2. Increment `cycles_since_last_use`.
3. At threshold (default **15 cycles**) → severity steps down one tier; counter resets to 0.
4. Off the bottom of `Mild` → cure: remove the Addiction affliction and emit `AddictionHabituated { from: Mild, to: None, substance }`.
5. On every other tier-down → emit `AddictionHabituated { from, to: Some(new), substance }`.

`addiction_use_count[substance]` is **not** reset by decay or cure. This persistence is what enables relapse semantics (§5.1 step 5c).

Decay and reinforcement are mutually exclusive in a given cycle: the tribute either used the substance this cycle (reinforcement) or did not (decay tick). Cold-turkey acceleration (e.g. Severe addiction decaying faster) is explicitly **not** modeled in v1.

### 6.3 Cure paths

| Path | Effect |
|---|---|
| Decay (15 trigger-free cycles per tier-step; 45 cycles total worst case from Severe) | -1 tier; off Mild = cured |
| **Sponsor gift: Detox** (filed `dvd`) | Removes Addiction entirely; clears `high_cycles_remaining`; does **not** clear `addiction_use_count` (relapse semantics preserved) |
| **Sponsor gift: Therapy** (filed `dvd`; shared with trauma) | Removes Addiction entirely; same persistence semantics as Detox |
| **Shelter rest** | Counts as trigger-free toward decay (advances `cycles_since_last_use` normally); **plus** Withdrawal stat penalties halved during the rest cycle |
| **Ally aid** (`hangrier_games-e2cf`, palliative) | Co-located ally spends a turn → Withdrawal stat penalties halved for one cycle (does not affect severity or decay counter) |

## 7. Effects — High vs Withdrawal

Addiction has two effect modes determined by `high_cycles_remaining`:

- `high_cycles_remaining > 0` → **High mode** (substance-specific effect; severity-uniform).
- `high_cycles_remaining == 0` → **Withdrawal mode** (severity-tiered penalty + brain-layer behavior).

### 7.1 High mode effects (severity-uniform, substance-specific)

Effects apply for the full High duration, composing into `tribute.visible_modifiers(ctx)` and the brain-layer override pipeline.

| Substance | Effect |
|---|---|
| `Stimulant` | +2 strength, +2 speed, -1 intelligence |
| `Painkiller` | Suppress stat penalties from `Wounded`/`Broken`/`Mauled`/`Burned` for this cycle (the conditions persist; only their stat penalty is masked) |
| `Morphling` | Suppress all affliction stat penalties for this cycle (including trauma, phobia, addiction-withdrawal of *other* substances); +1 sanity stat |
| `Alcohol` | -1 intelligence, -1 speed, immune to Phobia triggers this cycle (no phobia-driven action override or stat penalty) |

### 7.2 High duration table (tolerance-driven)

Each subsequent severity tier shortens the High duration. This creates the "rock bottom" loop: as addiction worsens, the tribute needs to use more often to stay High, which feeds reinforcement, which escalates further.

| Substance | Mild (full) | Moderate (⌈N/2⌉) | Severe (1 cycle) |
|---|---|---|---|
| `Stimulant` | 2 cycles | 1 cycle | 1 cycle |
| `Painkiller` | 3 cycles | 2 cycles | 1 cycle |
| `Morphling` | 4 cycles | 2 cycles | 1 cycle |
| `Alcohol` | 1 cycle | 1 cycle | 1 cycle |

`fn high_duration(substance: Substance, severity: Severity) -> u32` lives at `shared/src/afflictions.rs` next to the table.

### 7.3 Withdrawal mode effects (severity-tiered)

Withdrawal applies when `high_cycles_remaining == 0`. The substance-stat targeted by withdrawal:

| Substance | Substance-stat |
|---|---|
| `Stimulant` | strength |
| `Painkiller` | sanity |
| `Morphling` | sanity |
| `Alcohol` | intelligence + sanity (both) |

| Effect | Mild | Moderate | Severe |
|---|---|---|---|
| **Stat penalty** (always-on, composes into `tribute.visible_modifiers`) | -1 substance-stat | -2 substance-stat, -1 sanity | -3 substance-stat, -2 sanity, -1 intelligence |
| **Inventory bias** (brain pipeline) | weak preference for inventory items matching the addicted substance | strong preference; **substance-search action** enabled (see §8) | compulsion override (substance-search beats most other actions) |
| **Craving action chance** | 0% baseline | 0% baseline (search action is the manifestation) | **5%/cycle baseline** even with no proximity stimulus |
| **Sleep penalty (shelter rest)** | none | shelter rest restores 75% of normal stamina/sanity recovery | shelter rest restores 50% (substance is on the brain) |

### 7.4 Substance-search action

```rust
pub enum Action {
    // ... existing
    SearchForSubstance { substance: Substance },
    // ... Flashback/Avoidance from trauma spec, Frozen from phobia spec, etc
}
```

`SearchForSubstance`:

- Tribute moves toward the nearest known cache / spawn-likely area / co-located ally with the substance.
- If a co-located ally has the substance and the tribute's bond is high enough, the tribute may **request** it (existing item-trade flow); otherwise the tribute searches the area.
- If the substance is found in the area's loot table, picks it up and consumes immediately (re-entering High mode the same cycle).
- Emits `MessagePayload::AddictionCraving { tribute, substance, severity }`.
- **Counts as a visibility moment** (§9): co-located tributes are added to `observed_by`.

### 7.5 Severe craving compulsion

When Severe withdrawal triggers the 5%/cycle baseline craving roll:

- The brain pipeline's addiction layer overrides whatever action the lower-priority layers chose.
- The tribute emits `SearchForSubstance` regardless of survival pressures (does not override `psychotic` or `preferred` brain layers — see §8 for ordering).
- Repeated Severe craving without finding substance does NOT escalate beyond Severe (the escalation path is already capped); but the long Withdrawal will eventually hit the 15-cycle decay threshold and downshift to Moderate.

## 8. Brain pipeline placement

The unified pipeline (per phobia spec §9, extended by trauma spec §8) becomes:

```
[psychotic, preferred, survival, stamina, fixation, phobia, trauma, addiction, affliction, gamemaker, alliance, consumable] → decide_base
```

Addiction slots **between trauma and generic affliction**:

- **Below trauma** because trauma's flashback / avoidance is a sharper emotional override — trauma flashback should beat a Withdrawal craving when both fire (the tribute is reliving the worst moment of their life; substance search waits a cycle).
- **Above generic affliction** because Withdrawal compulsion is a stronger brain override than generic affliction stat-penalty action selection. A Severe craving beats a Wounded(Severe) "rest" preference.

```rust
fn addiction_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    for addiction in tribute.addiction_afflictions() {
        let meta = addiction.addiction_metadata.as_ref()?;
        if meta.high_cycles_remaining > 0 {
            continue;  // High mode produces stat effects, not action overrides
        }
        // Withdrawal mode
        match addiction.severity {
            Severity::Severe => {
                if ctx.rng.gen_bool(0.05) {
                    return Some(Action::SearchForSubstance { substance: meta.substance });
                }
                // Compulsion bias also applied in scoring pass
            }
            Severity::Moderate => {
                // Search-action enabled but not forced; surfaced via weight_modifier
            }
            Severity::Mild => {}  // bias only
        }
    }
    None
}
```

Stat penalties always compose via `tribute.visible_modifiers(ctx)` regardless of whether the override returns `Some`. Inventory bias / search-preference is a `weight_modifier` applied to candidate actions in the brain's scoring pass, not an override.

## 9. Visibility (observer-aware)

Mirrors trauma spec §9 with the same 15-cycle observer decay (matches addiction's own decay).

Visibility moments (when other tributes learn about the addiction):

1. Any **substance use** in the presence of others (`AddictionReinforced` for an existing addiction OR `AddictionAcquired`/`AddictionRelapse` on first observed use) — every co-located tribute at the start of the using tribute's turn is added to `observed_by`; their `observer_seen_cycle` is set to current cycle.
2. **Craving action** (Mild bias does NOT trigger; Moderate/Severe `SearchForSubstance` action emission) — same observation set update.
3. **Cap-resist `AddictionResisted`** — does NOT trigger visibility (the resist is internal; the use is still observed via the `SubstanceUsed` payload).
4. The always-on stat penalty is **not** a visibility moment.
5. Mild inventory bias (weight modifier) is **not** a visibility moment.

Observer decay: each cycle, for each observer in `observed_by`, if `current_cycle - observer_seen_cycle[id] > 15`, observer is removed and `AddictionForgotten` is emitted.

Brain consumer: `tribute.knows_addiction(target_id, substance) -> bool`. Used by alliance scoring (§11) and the predator brain layer.

## 10. Messages

Ten new `MessagePayload` variants:

```rust
MessagePayload::SubstanceUsed       { tribute: TributeRef, item: ItemRef, substance: Substance }
MessagePayload::AddictionAcquired   { tribute: TributeRef, substance: Substance, severity: Severity, use_count: u32 }
MessagePayload::AddictionReinforced { tribute: TributeRef, substance: Substance, severity: Severity }
MessagePayload::AddictionEscalated  { tribute: TributeRef, substance: Substance, from: Severity, to: Severity }
MessagePayload::AddictionResisted   { tribute: TributeRef, substance: Substance, reason: AddictionResistReason }
MessagePayload::AddictionCraving    { tribute: TributeRef, substance: Substance, severity: Severity }
MessagePayload::AddictionRelapse    { tribute: TributeRef, substance: Substance, prior_uses: u32 }
MessagePayload::AddictionObserved   { observer: TributeRef, subject: TributeRef, substance: Substance }
MessagePayload::AddictionForgotten  { observer: TributeRef, subject: TributeRef, substance: Substance }
MessagePayload::AddictionHabituated { tribute: TributeRef, substance: Substance, from: Severity, to: Option<Severity> }

pub enum AddictionResistReason {
    AtCap,         // 2 active addictions already
    // Future: TraitResistance, EquippedTalisman, etc — left as enum for extensibility
}
```

`SubstanceUsed` is emitted **only when the consumable maps to `Some(Substance)`** (i.e. the item is one of the four addictive classes). Non-addictive consumables (health kit, memento, trail mix, food, water) continue to use the existing `ItemUsed`/`ConsumableUsed` payload (whatever the inventory module already emits). This keeps the `SubstanceUsed` stream signal-rich and makes "every `SubstanceUsed` is a candidate for an addiction state change" a useful invariant for downstream consumers.

`kind()` and `involves()` exhaustive matches gain 10 new arms (per the maintenance burden documented in `i26a`).

## 11. Alliance integration

Two rules, both gated on `knows_addiction`:

- **Soft acceptance penalty.** Tributes who know about a target's Addiction reduce their alliance-affinity score for that target by `severity_weight × alliance_addiction_penalty` per known addiction (default `alliance_addiction_penalty = -0.10` per tier; tunable). Multiple known addictions stack additively. No hard veto: addicts can still ally; they're just less attractive partners. This is half-magnitude vs trauma's -0.15 because addiction is a "you're a liability" judgment rather than trauma's "you're broken" judgment, and the stacking across substances already amplifies it.
- **Sympathetic bond on observed Craving.** When an ally witnesses a `SearchForSubstance` action (visibility moment §9 case 2), the witness's bond-affinity toward the searching tribute is *increased* by `addiction_sympathetic_bond_increment` (default `+0.05`). Half of trauma's `+0.10` because addiction is less universally sympathetic — but still positive because pity/co-dependence is a real bond-strengthener.

No hard alliance veto for addiction. Even Severe Stimulant-addicted tributes can be in alliances.

## 12. UI

**Tribute detail (admin view):**

- "Addictions" section after Trauma (decomposed via `lzfe`).
- One row per Addiction (0-2 per tribute): substance icon + severity badge, mode badge (`HIGH N` or `WITHDRAWAL`), `cycles_since_last_use` countdown, observer count.
- Expandable observer list (who knows and how recently).
- Persistent "use count" sub-panel: `BTreeMap<Substance, u32>` rendered as a small bar chart (visualizes relapse risk).

**Timeline cards:**

- New `addiction_card.rs` consuming all 10 payloads.
- `AddictionCraving` is the headline event: red accent at Severe, orange at Moderate.
- `AddictionRelapse` is a distinct compact card (highlights the "back on it" moment).
- `SubstanceUsed` is rendered inline as a compact event (does not need a full card unless paired with an Acquire/Reinforce/Escalate that cycle).
- `AddictionReinforced` collapses with `AddictionEscalated` in the visual stack when both fire on the same cycle (escalation supersedes; reinforcement-only renders as a small "deepens" indicator).
- Reuses `CardShell` (`t7g1`).

**Tribute state strip:**

- Compact icon when in Withdrawal at Moderate or Severe (Mild Withdrawal is too quiet).
- Distinct subtle indicator when in High mode (so the player can read "this tribute is currently affected by a substance").
- Tooltip lists substance + mode + severity.

**Spectator skin:**

- Addiction counts as "physical/psychological" content; respects existing severity-color WCAG audit (`hangrier_games-3yb`).

**No tribute-viewport in v1.** Filed (`n52s`) and would respect observer state when built.

## 13. Testing strategy

Per `uz80` (proptest) and `yj9u` (snapshot streams).

### 13.1 Unit tests

- `Substance` mapping: each v1 consumable item → expected `Option<Substance>`.
- Acquisition curve table: use count 1..=6 → expected base chance.
- Substance multiplier composition: `Morphling` cap-at-95% verified at use #5+.
- Single-substance Addiction: second use of same substance reinforces, does not create second Addiction.
- Cap-at-2: third distinct substance at cap → `AddictionResisted { reason: AtCap }`; existing addictions unchanged; substance immediate effect still applied.
- Reinforcement: counter reset on use, escalation rolls at configured rate (seeded RNG).
- Decay: counter increments each use-free cycle, tier-down at threshold (15), cure off Mild.
- `addiction_use_count` persists across cure: cured tribute's next use of same substance auto-acquires at Mild via relapse path; emits `AddictionRelapse`.
- High → Withdrawal transition: `high_cycles_remaining` decrements; effects switch when it hits 0.
- High duration table: each (substance, severity) → expected duration; tolerance-driven shortening verified.
- Withdrawal stat penalty composition: each substance → correct substance-stat penalty per tier.
- Morphling High suppresses other-addiction Withdrawal stat penalties (cross-substance suppression).
- Alcohol High suppresses phobia trigger.
- Painkiller High suppresses Wounded/Broken/Mauled/Burned stat penalties.
- Substance-search action: emitted at Severe craving roll; emitted at Moderate when no other action scores higher.
- Severe baseline craving (5%/cycle no stimulus): seeded RNG verifies rate over N trials.

### 13.2 Integration tests (`game/tests/addiction_*.rs`)

- `first_use_low_acquisition.rs`: tribute uses Stimulant once; seeded RNG below 5% → no acquisition; use count incremented; `SubstanceUsed` emitted.
- `repeated_use_acquires.rs`: tribute uses Stimulant 5×; seeded RNG path through curve; eventual Mild Addiction acquired with correct payload.
- `morphling_high_acquisition_chance.rs`: tribute uses Morphling once; seeded RNG → 7.5% effective chance; verify multiplier path.
- `cap_at_two_addictions.rs`: tribute already addicted to Stimulant + Alcohol; uses Morphling and rolls hit; `AddictionResisted { AtCap }` emitted; substance effect still applied (sanity +1 from Morphling High).
- `relapse_after_cure.rs`: tribute Mild-addicted to Stimulant, decays out via 15 trigger-free cycles, then uses Stimulant; auto-acquires at Mild; `AddictionRelapse { prior_uses }` emitted.
- `high_to_withdrawal_transition.rs`: Stimulant Mild addiction; use sets `high_cycles_remaining = 2`; cycles 1-2 in High; cycle 3 begins Withdrawal; `-1 strength` stat penalty applies.
- `tolerance_shortens_high_duration.rs`: Stimulant Severe addiction; use sets `high_cycles_remaining = 1` (vs Mild's 2); next cycle is Withdrawal.
- `severe_craving_compulsion.rs`: tribute with Severe Stimulant Withdrawal; seeded RNG hits 5% baseline craving; emits `SearchForSubstance` regardless of survival-layer preference (verify ordering).
- `morphling_high_suppresses_trauma_stat.rs`: tribute with Severe Trauma + Morphling Mild Addiction in High mode; verify Morphling High suppresses trauma stat penalty for the cycle.
- `alcohol_high_suppresses_phobia_trigger.rs`: tribute with Severe Phobia + Alcohol Mild Addiction in High mode; phobia stimulus present; phobia override does not fire.
- `painkiller_high_suppresses_wound_penalty.rs`: tribute Wounded(Severe) + Painkiller Mild Addiction in High mode; Wounded stat penalty suppressed for the cycle (condition persists).
- `withdrawal_to_decay_cure.rs`: Mild Addiction; 15 trigger-free cycles; cured; `AddictionHabituated { to: None }` emitted; use_count preserved.
- `detox_cures_addiction.rs`: Detox sponsor gift (using trapdoor for `dvd`-blocked test); Addiction removed; use_count preserved; subsequent use triggers relapse.
- `observer_learns_then_alliance_penalty.rs`: target has Severe Addiction; uses substance co-located with potential ally; ally observes; subsequent alliance proposal scored with reduced affinity (-0.10 × 3 = -0.30 for one Severe addiction).
- `sympathetic_bond_on_craving.rs`: ally witnesses tribute's `SearchForSubstance`; bond-affinity increases by `+0.05`.
- `forgotten_after_decay.rs`: 16 cycles separated; observer's `knows_addiction` returns false; `AddictionForgotten` emitted.

### 13.3 Insta snapshots

- Addiction state on `Tribute` after each integration scenario (BTreeMap of Addiction metadata + use_count map).
- Ordered `MessagePayload` streams for use → acquire → reinforce → escalate → high → withdrawal → craving → habituate → relapse paths.
- `addiction_use_count` BTreeMap snapshot across cure/relapse cycles to verify persistence.

### 13.4 Proptest properties (per `uz80`)

- **Cap invariant**: across any sequence of substance uses (any substance, any RNG), a tribute has at most 2 Addiction afflictions at any time.
- **Use-count monotonicity**: `addiction_use_count[s]` is monotonically non-decreasing for all `s` across all events including cure.
- **Relapse determinism**: if `addiction_use_count[s] > 0` and tribute has no `Addiction(s)` and uses substance `s`, the next state has `Addiction(s)` at Mild (assuming cap not full).
- **Reinforcement-decay exclusivity**: in any cycle, each Addiction has either `cycles_since_last_use` reset to 0 (used) or incremented (not used), never both.
- **High-Withdrawal coverage**: `high_cycles_remaining` is always in `[0, max_high_duration]` for the addiction's substance + severity.
- **Reinforcement bounded**: over N seeded trials of substance use at sub-Severe severity, escalation rate hits `firing_escalation_chance` ±tolerance.
- **Decay reaches cure**: from any starting severity, `15 × tier_count` use-free cycles cures the addiction (assuming no use events).
- **Severity floor monotonic during use bursts**: severity never decreases as a result of a use event (it can only stay or rise via escalation roll).
- **Observer monotonicity within cycle**: observer set only grows during a single cycle's use/craving emissions; decay only at cycle boundary.
- **Acquisition probability bounds**: `p_acquire ∈ [0.0, 0.95]` for all (use_count, substance) pairs.

## 14. Migration / rollout

No data migration required.

- New `AfflictionKind::Addiction(Substance)` variant is additive; existing tributes default to no addiction.
- New `addiction_metadata: Option<AddictionMetadata>` field on `Affliction` is `#[serde(default, skip_serializing_if = "Option::is_none")]` so existing serialized afflictions deserialize cleanly with `None`. (Same backward-compat pattern as trauma's `trauma_metadata` and phobia's `phobia_metadata`.)
- New `addiction_use_count: BTreeMap<Substance, u32>` field on `Tribute` is `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`; existing tributes deserialize with an empty map.
- New `MessagePayload` variants are additive (10 new arms).
- SurrealDB schema unchanged (affliction object is flexible per health-conditions spec; verify in PR1 by running existing migration tests).
- New no-op migration file added in PR1 to bump the migration version pointer.

Rollout: ship behind `addiction_enabled: bool` config flag on `Game`, default `true` once tuning settles. Producer hook in `try_use_consumable` and addiction brain layer both no-op when flag is off. Mirrors phobia + fixation + trauma rollout pattern.

Existing saves load with empty `addiction_use_count` and no `Addiction` afflictions; the system retro-applies as soon as substance use begins post-rollout.

## 15. PR breakdown

- **PR1** — Types & storage: `Substance` enum, `AddictionMetadata`, `AddictionResistReason`, `AfflictionKind::Addiction(Substance)` variant, `addiction_metadata: Option<AddictionMetadata>` on `Affliction`, `addiction_use_count: BTreeMap<Substance, u32>` on `Tribute`. `try_acquire_addiction(tribute, substance, rng)` helper enforcing probabilistic curve + substance multiplier + cap-at-2 + relapse short-circuit. `high_duration` table function. No-op migration for version bump. Unit tests for acquisition curve, substance multiplier, cap-at-2, relapse path, single-instance reinforcement, severity floor.
- **PR2** — Use pipeline & producers: `Consumable::substance(&self) -> Option<Substance>` mapping for v1 items (Stimulant only, with mock items used to test the other three substance paths). Hook into `try_use_consumable` → emit `SubstanceUsed` → call `try_acquire_addiction` → emit `AddictionAcquired`/`AddictionReinforced`/`AddictionEscalated`/`AddictionResisted`/`AddictionRelapse`. Co-acquired immediate High effect dispatched here (refresh `high_cycles_remaining`). Reinforcement decrement of decay counter on subsequent use. Integration tests for first-use, repeated-use, cap-at-2, relapse-after-cure, Morphling multiplier path.
- **PR3** — Brain layer (`addiction_override`), `Action::SearchForSubstance`, severity-tiered Withdrawal effects table (stat penalty, inventory bias, craving rolls, sleep penalty), substance-specific High effects (cross-system suppression: Morphling-suppress-affliction-stat, Alcohol-suppress-phobia-trigger, Painkiller-suppress-wound-penalty), alliance integration (acceptance penalty, sympathetic bond), observer state, decay tick, reinforcement-on-use (the shared rule), all 10 message payloads emitted. Integration tests for High→Withdrawal transition, tolerance, Severe craving compulsion, cross-substance suppression, alliance penalty, sympathetic bond, observer learning, decay-to-cure, Detox.
- **PR4** — Frontend: tribute-detail "Addictions" section after Trauma, use-count bar chart, timeline `addiction_card.rs` consuming all 10 payloads, state-strip High/Withdrawal indicators, spectator-skin integration. Smoke tests + WCAG check on substance icons.

### Hard prerequisites

- PR1 blocked by **afflictions PR1** (`hangrier_games-lsis`) — needs `AfflictionKind` extensibility + `try_acquire_affliction` + flexible affliction object shape.
- PR3 blocked by **brain pipeline unification** (`hangrier_games-hbox`) — same reason phobia PR2 / trauma PR3 was blocked; the addiction layer slots into the unified pipeline.
- PR3 also depends on the **shared traumatic-affliction reinforcement helper** at `shared/src/afflictions.rs::apply_traumatic_reinforcement`, extracted by whichever of phobia PR3 / fixation PR2 / trauma PR3 lands first.
- PR4 lands last, decoupled from PR3.

### Soft dependencies

- **Sponsor gift Detox / Therapy** delivery is filed under `dvd` and not implemented in this spec; PR3 provides the cure-removal logic so a future Detox/Therapy gift can call into it without further changes here.
- **Sponsor gift substance delivery** for Morphling, Alcohol, Painkiller is filed under `dvd`. PR2 producer code is exercised against mock items in tests; the production path lights up when `dvd` lands.
- **Ally aid** palliative cure path is filed (`hangrier_games-e2cf`); PR3 provides the half-Withdrawal-penalty logic so the aid action can call into it.
- **Tribute-viewport UI** (`n52s`) will respect addiction observer state when built.

## 16. Out of scope (filed as follow-ups)

- Cross-substance interaction (alcohol+morphling synergy, stimulant+painkiller stacking) — file as `addiction: cross-substance interaction effects`.
- Trait modifiers for addiction sensitivity (`Resilient`/`Fragile` susceptibility) — file as `addiction: trait modifiers on acquisition curve`.
- District backstory weighting on initial susceptibility (Career districts more resistant? Morphling district pre-disposed?) — file as `addiction: district backstory weighting`.
- Substance overdose (use-count-triggered death roll) — file as `addiction: overdose mechanic`.
- Withdrawal-driven theft / PvP escalation beyond brain layers (Severe Withdrawal mugs nearest non-ally for inventory) — file as `addiction: withdrawal-driven aggression escalation`.
- Substance-target fixation (pull toward known supplier ally) — file as `addiction: substance-target fixation integration`.
- Cold-turkey decay acceleration — file as `addiction: severity-tiered decay rates`.
- Player-facing tolerance gauge UI beyond severity badge — file as `addiction: tolerance visualization`.
- Addictive food / water — N/A; explicitly non-addictive in v1.
- Dedicated addiction-viewport UI — covered by `n52s`.

## 17. Spec self-review

- **Placeholders / tunables** (called out for post-observability tuning):
  - Acquisition curve: `[0.05, 0.15, 0.30, 0.50, 0.75]` for use counts 1..=5+.
  - Substance multipliers: Morphling 1.5 (cap 0.95), Alcohol 0.7, Stimulant 1.0, Painkiller 1.0.
  - `firing_escalation_chance` = 12% (window `[10%, 15%]`, matches phobia + trauma + fixation).
  - Decay threshold = 15 cycles (intentionally longer than trauma's 10; addiction is the stickiest affliction).
  - Observer decay = 15 cycles (matches addiction decay).
  - Severe baseline craving = 5%/cycle.
  - High duration table per (substance, severity) — 12 cells, all tunable.
  - High effects per substance (stat magnitudes, suppression scope) — tunable.
  - Withdrawal stat penalties per (substance, severity) — tunable.
  - `alliance_addiction_penalty` = -0.10 per tier per known addiction.
  - `addiction_sympathetic_bond_increment` = +0.05.
  - Active addiction cap = 2.
  - Sleep penalty: 75% (Moderate) / 50% (Severe) of normal recovery.

- **Internal consistency:**
  - Severity ordering, reaction-tier table, and visibility-firing rules use the same `Severity` ordering as health-conditions, phobias, trauma.
  - Cap-of-2 invariant is consistent throughout (acquisition §5.1/§5.3, storage §4, UI §12, proptest §13.4).
  - Reinforcement rule is the same shared helper as phobia PR3, trauma PR3, fixation PR2 (cross-referenced explicitly).
  - Brain pipeline placement (between trauma and affliction) is consistent with phobia → trauma → addiction → affliction left-to-right gradient from "specific stimulus" → "generalized state" → "use-driven dependence" → "physical condition".
  - Persistent `addiction_use_count` is consistent across cure (§5.1 step 5c, §6.2, §6.3, §13 tests, §13.4 proptest).

- **Scope check:** PR1-PR4 each ~400-600 LOC plus tests, slightly larger than trauma due to substance-specific effect tables but comparable. v1-reachable substance is just Stimulant; the other three are tested via mock items and light up when `dvd` ships.

- **Ambiguity resolved inline:**
  - "Co-located" defined as `ctx.area.tributes` at the start of the using tribute's turn (consistent with phobia / trauma specs).
  - "Successfully use" excludes failed-use cases (no inventory, no charges left, etc); only successful consumption increments use_count.
  - `SubstanceUsed` emission scope clarified in §10: only when `Consumable::substance(&self)` returns `Some`. Non-addictive consumables continue to use existing `ItemUsed`-style payload.
  - Withdrawal mode begins the cycle `high_cycles_remaining` reaches 0 (not the cycle after); decrements happen at start-of-cycle.
  - Cap-resist still applies the substance's immediate effect (the tribute *consumed* the item; only the stored Addiction state is suppressed).

- **Open questions resolved by punt to follow-up:**
  - Trait modifiers (`Resilient`/`Fragile` etc) on addiction acquisition rate — listed in §16; defer to follow-up after observability data settles the base rates.
  - Whether `Stimulant` should subdivide (yayo vs go-juice vs adrenaline have different real flavors) — deferred; one Substance variant covers all three for v1 to keep the Substance enum small. If subdivided later, migration is straightforward (split variant + remap historical use_count).
  - Whether `Morphling` should also suppress addiction Withdrawal stat penalties for *itself* (in addition to other afflictions) — yes, by construction: in High mode there is no Withdrawal mode for that addiction. The cross-substance suppression in §7.1 covers the more interesting case.
