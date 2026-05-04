# Phobias — v1 Design

**Status:** Approved (brainstorming complete, awaiting implementation plan)
**Author:** klove
**Date:** 2026-05-03
**Related:**
- Built on: `2026-05-03-health-conditions-design.md` (afflictions foundation, hard prereq)
- Pairs with: trauma spec `hangrier_games-cdu0` (becomes producer of Traumatic phobias)
- Brain prereq: `hangrier_games-hbox` (unified pipeline, **landed** in #218)
- Useful prereq: `hangrier_games-xp4x` (CycleContext for stimulus inputs)
- Future hooks: `hangrier_games-dvd` (sponsor Therapy gift), trait system, alliance system

## 1. Problem

Tributes today have no model for fear-of-stimulus. The emotions spec (`2026-05-02-tribute-emotions-design.md`) covers volatile mood; the trait system covers personality; but neither produces the *frozen-at-the-sight-of-fire* genre beat. A canonical Hunger Games moment — a tribute paralyzed by their lifelong fear at exactly the wrong instant — has no plumbing.

This spec adds **phobias** as triggered, durable, observer-aware fears. A phobia sits dormant inside the tribute's affliction set; when its stimulus is detected during the per-cycle scan, it *fires*, producing severity-tiered reactions ranging from stat penalty (Mild) to forced freeze (Severe). Other tributes learn each other's phobias only by observation, opening exploitative play that mirrors the genre.

Phobias reuse the affliction infrastructure entirely (storage, severity, message family, brain pipeline registration). The new machinery is just: trigger taxonomy, per-cycle detection scan, observer state, and the dedicated brain-pipeline layer that fires before generic affliction handling.

## 2. Phobias as a kind of affliction

```rust
// shared/src/afflictions.rs (extension to existing AfflictionKind from spec #1)
pub enum AfflictionKind {
    // ... existing variants from health conditions spec
    Phobia(PhobiaTrigger),
}
```

Storage and uniqueness come free: `AfflictionKey = (AfflictionKind, Option<BodyPart>)` already discriminates on the full `AfflictionKind` value, so two phobias with different `PhobiaTrigger`s have different keys naturally. No struct change to the affliction key.

The existing severity model applies: Mild / Moderate / Severe. Cure paths and visibility get phobia-specific overrides documented in §6 and §7.

## 3. Trigger taxonomy

```rust
// shared/src/afflictions.rs
pub enum PhobiaTrigger {
    // Environmental (7)
    Fire,
    Water,
    Dark,
    Blood,
    Heights,
    Enclosed,
    Open,
    // Creature (parameterized)
    Animal(Animal),
    // Social (parameterized)
    Tribute(TributeId),
    TraitGroup(Trait),
}
```

The trigger space is open-ended through parameterization (any `Animal` variant, any `TributeId`, any `Trait`). Each variant has a hand-written `is_present` rule (see §4). Out of v1: weapon-specific phobias, sponsor-gift phobias, sound/cannon phobias — filed as v2 follow-up.

`PhobiaTrigger: Eq + Hash + Ord` so it composes inside `AfflictionKind` without breaking the `BTreeMap` storage.

## 4. Trigger detection (per-cycle scan)

At the start of each tribute's turn, the phobia layer walks the tribute's phobia afflictions and checks each:

```rust
// game/src/tributes/afflictions/phobia/triggers.rs
pub fn is_present(trigger: &PhobiaTrigger, tribute: &Tribute, ctx: &CycleContext) -> bool {
    match trigger {
        Fire => ctx.area.has_event(AreaEventKind::Fire) || ctx.weather.is_lightning(),
        Water => ctx.area.terrain.is_water_dominant() || ctx.area.has_event(AreaEventKind::Flood),
        Dark => ctx.is_night() && !tribute.has_light_source(),
        Blood => ctx.recent_events.contains_kind(MessageKind::CombatKilled)
              || tribute.recent_combat_in_area(ctx),
        Heights => matches!(ctx.area.terrain, Terrain::Cliffs | Terrain::Mountains),
        Enclosed => matches!(ctx.area.terrain, Terrain::Cave | Terrain::Bunker),
        Open => matches!(ctx.area.terrain, Terrain::Plains | Terrain::Desert),
        Animal(a) => ctx.area.threats.iter().any(|t| matches!(t, Threat::Animal(x) if x == a)),
        Tribute(id) => ctx.area.tributes.iter().any(|t| t.id == *id && t.is_alive()),
        TraitGroup(tr) => ctx.area.tributes.iter().any(|t| t.id != tribute.id && t.traits.contains(tr)),
    }
}
```

Detection is **boolean**. Severity drives reaction strength, not detection intensity. The world's variation (a campfire vs an inferno) shows up in the underlying stimulus model — they're different `AreaEventKind`s — not in `is_present`.

`CycleContext` (`hangrier_games-xp4x`, planned) provides the stimulus inputs. Until it lands, phobias use a thinner ad-hoc context built inline in the run-tribute-cycle path; spec #1's `affliction` layer already pays this cost so it's pre-allocated.

## 5. Reactions (severity-tiered)

When `is_present` returns true for a phobia, that phobia *fires* this cycle. Reactions:

| Severity | Stat penalty (while triggered) | Brain bias | Action override |
|---|---|---|---|
| Mild | -2 atk/def | none | none |
| Moderate | -4 atk/def | strong flee preference | none |
| Severe | -6 atk/def | overwhelming flee preference | 25% chance freeze (no action); auto-flee otherwise if escape route exists |

Trait modifiers compose by adjusting *effective* severity for reaction computation:

| Trait | Effective severity adjustment |
|---|---|
| `Resilient` | -1 tier (Mild becomes no reaction) |
| `Fragile` | +1 tier (Mild reacts as Moderate; Severe stays Severe) |
| `Reckless` | ignores freeze chance entirely; still takes stat penalty + flee bias |
| `Cautious` | none (no special interaction; the trait already biases away from danger) |

Multiple firing phobias compose by stacking stat penalties additively (capped at -10 total to avoid degenerate cases) and taking the *strongest* override (Severe freeze beats Moderate flee bias). If multiple Severe phobias roll freeze, one freeze fires (it's a single turn).

## 6. Acquisition & origin

```rust
// shared/src/afflictions.rs
pub enum PhobiaOrigin {
    Innate,
    Traumatic { event_ref: MessageId },
}
```

Stored alongside the affliction (extension to `Affliction` for `Phobia` kinds, or as a side-table — implementation detail; recommend a `phobia_metadata: Option<PhobiaMetadata>` field on `Affliction` that's `Some` only for Phobia kinds, defaulting to `None` for all others).

Acquisition paths:

1. **Spawn-time** — 0-2 phobias rolled, weighted heavily toward 0-1. Trigger drawn from a weighted distribution (Fire/Dark/Blood common; Tribute/TraitGroup rare since they need targets that exist; Heights/Enclosed/Open weighted by district backstory if available, otherwise uniform). Origin: `Innate`. Severity weighted toward `Mild`/`Moderate`.
2. **Traumatic** — produced by trauma events when the trauma spec (`cdu0`) lands. The canonical case: ally killed by fire in same area → tribute acquires `Phobia(Fire)` with `Origin: Traumatic`. Trauma-induced phobias spawn at `Mild` and can escalate two ways: (a) the affliction-cascade rules from spec #1 (external trauma stacking), and (b) **reinforcement on firing** — see §7. The two pathways are mutually exclusive per cycle: a phobia that escalates by firing this cycle does not also roll cascade escalation in the same cycle (the firing-escalation roll is checked first; on success, cascade escalation is suppressed for that phobia for the cycle). This avoids double-tier jumps from a single triggering event.
3. **Future hooks** (filed): sponsor-induced (rare hostile gift), gamemaker-induced (sadistic intervention).

Soft cap of 3 phobias per tribute. `try_acquire_phobia` calls into the existing `try_acquire_affliction` from spec #1; at cap, new acquisitions only succeed if severity exceeds the weakest existing phobia (which is then `Supersede`d). This keeps the per-tribute psychology bounded without hard-rejecting every late acquisition.

## 7. Reinforcement, decay, and cure

Traumatic phobias follow a **sensitization-aware** model that mirrors the fixation reinforcement loop (`fazp` epic). Innate phobias are static: they neither decay nor escalate from firing.

### 7.1 Per-cycle reinforcement (Traumatic only)

Every cycle in which `is_present` returns true for a Traumatic phobia (i.e. the phobia *fires*):

1. **Decay timer reset.** `cycles_since_last_fire` is set to `0`.
2. **Sensitization roll.** Roll a `firing_escalation_chance` (default **12%**, tunable, in the `[10%, 15%]` window). On success, severity steps up one tier: `Mild → Moderate → Severe`. A phobia already at `Severe` ignores the result (capped). On a successful escalation, emit `PhobiaEscalated` (see §10) and suppress cascade escalation from spec #1 for this phobia for the rest of the cycle (see §6).

The roll happens once per phobia per cycle, regardless of how many tributes/triggers were observed in the area. Multiple firing phobias on one tribute each roll independently.

### 7.2 Decay (Traumatic only)

A phobia that has not fired for **5 trigger-free cycles** decays one tier: `Severe → Moderate → Mild → cured`. The counter is the same `cycles_since_last_fire: u32` on the phobia metadata, ticked each cycle in the cascade pass from spec #1, and reset to `0` whenever §7.1 fires.

Decay and reinforcement are exclusive in a given cycle (a cycle that fires cannot also be a decay cycle — the counter is reset before the decay check).

### 7.3 Cure paths

| Origin | Cure path |
|---|---|
| `Innate` | Permanent in v1. (Future: `Therapy` sponsor gift via `dvd`. Filed.) |
| `Traumatic` | **Habituation** via §7.2 decay: 5 trigger-free cycles steps tier down; off the bottom of `Mild` cures and removes the phobia. |
| Either | **Sedative item**: suppresses the next firing of one specified phobia (does not remove the phobia, just one cycle of reaction; also blocks that cycle's §7.1 reinforcement roll since the phobia did not fire from the brain's perspective). |
| Either | **Therapy sponsor gift**: removes one phobia entirely. (Filed for `dvd`.) |

Shelter rest does **not** cure phobias (a sheltered tribute who isn't around fire isn't being cured of fire-fear; they're just not encountering the trigger — habituation captures this naturally).

### 7.4 Rationale

Real phobias *sensitize* under repeated uncontrolled exposure as often as they habituate; pure monotonic decay produced an artificial asymmetry vs the fixation reinforcement spec (`fazp`). Unifying both systems on a single "traumatic affliction reinforcement" mechanic — fire resets timer, fire rolls escalation, absence ticks decay — keeps the codebase honest and the psychology truthful. Innate phobias are intentionally exempt: they model lifelong dispositions, not learned responses.

## 8. Visibility (observer-aware)

Phobias are hidden state. The default affliction visibility rule from spec #1 (severity-gated) does not apply. Instead:

```rust
// stored on phobia metadata
pub observed_by: BTreeSet<TributeId>,
pub observer_seen_cycle: BTreeMap<TributeId, u32>,  // last cycle observer saw it fire
```

Rules:

- **Hidden until observed firing.** No observer until the phobia fires in the presence of a witness.
- **Observation moment.** When the phobia fires *and* the reaction is mechanically visible (Moderate flee bias produces a visible move, Severe freeze produces a `Frozen` event), every other tribute in the same area at that moment is added to `observed_by` and their `observer_seen_cycle` is updated to the current cycle.
- **Mild firings are not observable.** A -2 stat penalty produces no visible behavior; observers learn nothing.
- **Decay.** Each cycle, for each observer in `observed_by`, if `current_cycle - observer_seen_cycle[id] > 5`, the observer is removed. They've forgotten.

Brain decisions filter trigger awareness through `tribute.knows_phobia(target_id, trigger) -> bool`. A predator brain layer can exploit phobias only of targets it has personally observed.

UI scope:
- **Spectator timeline + admin tribute-detail**: see all phobias and the full observer graph (the "who knows what" structure is interesting to watch).
- **Future tribute-viewport** (`n52s`): respects observer state — viewer only sees phobias they've personally observed.

## 9. Brain pipeline placement

The unified pipeline (landed in `hbox` / #218):

```
[psychotic, preferred, survival, stamina, affliction, gamemaker, alliance, consumable] → decide_base
```

This spec inserts a dedicated **`phobia` layer between `stamina` and `affliction`**:

```
[psychotic, preferred, survival, stamina, phobia, affliction, gamemaker, alliance, consumable] → decide_base
```

Why between stamina and affliction:
- **Below survival/stamina**: a tribute *will* drink water even while triggered. Phobia doesn't override "I'm dying of thirst."
- **Above generic affliction**: phobia freeze/auto-flee should win against generic affliction stat penalties. Spec #1's `Wounded(Severe) refuses combat` should not block the much stronger `Phobia(Tribute) Severe freeze` reaction.

Composition rules in the phobia layer:

```rust
fn phobia_override(tribute: &Tribute, ctx: &CycleContext) -> Option<Action> {
    let firing = tribute.firing_phobias(ctx);  // walks phobia afflictions, calls is_present
    if firing.is_empty() { return None; }

    let strongest = firing.iter().max_by_key(|p| p.effective_severity(tribute.traits)).unwrap();
    match strongest.reaction(tribute.traits) {
        Reaction::Freeze => Some(Action::Frozen),
        Reaction::AutoFlee => flee_action(tribute, ctx, strongest.trigger),
        Reaction::Penalty => None,  // pass through; stat penalties applied via visible_modifiers
    }
}
```

Stat penalties from firing phobias compose into `tribute.visible_modifiers(ctx)` regardless of whether the layer returns `Some` or `None`, so they always apply. Only the *action override* is gated on the layer returning `Some`.

`Action::Frozen` is a new action variant: tribute does nothing this cycle, no message except the `PhobiaTriggered { effect: Freeze }` payload.

## 10. Messages

New `MessagePayload` variants:

```rust
MessagePayload::PhobiaAcquired {
    tribute: TributeRef,
    trigger: PhobiaTrigger,
    severity: Severity,
    origin: PhobiaOrigin,
}

MessagePayload::PhobiaTriggered {
    tribute: TributeRef,
    trigger: PhobiaTrigger,
    severity: Severity,
    effect: PhobiaEffect,  // Penalty | Flee | Freeze
}

MessagePayload::PhobiaObserved {
    observer: TributeRef,
    subject: TributeRef,
    trigger: PhobiaTrigger,
}

MessagePayload::PhobiaHabituated {
    tribute: TributeRef,
    trigger: PhobiaTrigger,
    from: Severity,
    to: Option<Severity>,  // None = cured (off the bottom)
}

MessagePayload::PhobiaEscalated {
    tribute: TributeRef,
    trigger: PhobiaTrigger,
    from: Severity,
    to: Severity,  // strictly greater than from
}

MessagePayload::PhobiaForgotten {
    observer: TributeRef,
    subject: TributeRef,
    trigger: PhobiaTrigger,
}
```

`kind()` and `involves()` exhaustive matches gain six new arms (per the maintenance burden documented in `i26a`).

## 11. Trauma spec integration

The trauma spec (`cdu0`) becomes the canonical producer of Traumatic phobias. Trauma events (witness ally death, betrayal survived, near-death from a specific source) call:

```rust
tribute.try_acquire_affliction(AfflictionDraft {
    kind: AfflictionKind::Phobia(trigger_derived_from_event),
    severity: Severity::Mild,
    source: AfflictionSource::Cascade { from: trauma_event_key },
    phobia_metadata: Some(PhobiaMetadata {
        origin: PhobiaOrigin::Traumatic { event_ref: event_id },
        ..PhobiaMetadata::default()
    }),
})
```

This spec defines the storage and reaction; trauma spec defines the event-to-trigger mapping. The two specs land in either order; phobias-first lets innate phobias play immediately and trauma-acquired arrives as a pure addition.

## 12. Alliance integration

A phobia of a specific tribute (`Phobia(Tribute(id))`) fires when that tribute is in the same area. Alliance-affinity scoring already exists; phobia presence imposes a hard veto: a tribute cannot propose alliance to a tribute they have a phobia of, and proposals from such a tribute are auto-refused with a special `BondReason::Phobia` recorded.

A phobia of a `TraitGroup(tr)` softly reduces alliance affinity for any tribute carrying that trait, multiplied by phobia severity. Composes with existing alliance affinity calculation.

Phobias of allied tributes (acquired traumatically while allied) trigger `BondShock` immediately on acquisition — the existing alliance-shock mechanic from `2026-04-25-tribute-alliances-design.md` applies.

## 13. UI

**Tribute detail (admin view):**
- "Fears" section after Afflictions (decomposed via `lzfe`).
- Each phobia: trigger label, severity badge, origin icon (Innate / Traumatic), observer count.
- Expandable observer list (who knows about this fear and how recently).

**Timeline cards:**
- New `phobia_card.rs` consuming `PhobiaAcquired`, `PhobiaTriggered`, `PhobiaObserved`, `PhobiaHabituated`, `PhobiaForgotten`.
- `PhobiaTriggered` is the headline event — uses red accent at Severe (freeze), orange at Moderate, yellow at Mild.
- Reuses `CardShell` (`t7g1`).

**Tribute state strip:**
- Compact icon when a phobia is firing this cycle. Distinct from generic affliction icon.
- Tooltip lists active fears.

**No tribute-viewport in v1.** Filed (`n52s`) and would respect observer state when built.

## 14. Testing strategy (per `uz80`/`yj9u`)

Unit tests:

- `triggers::is_present` truth table — every `PhobiaTrigger` variant against representative `CycleContext` states (positive and negative cases).
- `Phobia::reaction(severity, traits)` matrix — every (severity × trait) pair → expected `Reaction`.
- Observer add: firing in presence of N observers → all added with current cycle stamp.
- Observer decay: tick N cycles past threshold → removed.
- Habituation: counter resets on fire; ticks down severity at threshold (Traumatic only; Innate never decays).
- Reinforcement: Traumatic phobia firing reset counter AND rolls escalation at configured rate; Innate firing rolls neither.
- Reinforcement at cap: Severe Traumatic phobia firing rolls escalation but stays at Severe (no `PhobiaEscalated` emitted).
- Sedative item suppresses both the firing and the reinforcement roll for that cycle.
- Trait modifier composition: Resilient + Mild = no reaction; Fragile + Mild = Moderate reaction; Reckless + Severe = penalty + flee but no freeze chance.
- Cap-at-3: 4th acquisition succeeds only if stronger; replaces weakest.

Integration tests (`game/tests/phobias_*.rs`):

- `freeze_to_death.rs`: tribute with `Severe Phobia(Fire), Innate` enters fire area; Cato attacks; tribute freezes; tribute dies; full message stream snapshotted.
- `flee_from_predator.rs`: tribute with `Severe Phobia(Tribute(Cato))`, Cato in same area; tribute auto-flees; safe in next area; phobia fires once.
- `habituation_in_safety.rs`: tribute with `Mild Phobia(Fire), Traumatic`; 5 cycles in non-fire area; phobia cured; `PhobiaHabituated { to: None }` emitted.
- `observer_learns_then_exploits.rs`: target has hidden `Severe Phobia(Tribute(Cato))`; Cato in same area; target freezes, observer Clove witnesses; next cycle Clove's brain knows the phobia (verify via brain decision audit log).
- `forgotten_after_decay.rs`: same setup as previous; 6 cycles separated; observer's `knows_phobia` returns false.
- `reinforcement_escalates.rs`: tribute with `Mild Phobia(Fire), Traumatic`; seeded RNG forces escalation roll to succeed on first firing; phobia becomes `Moderate`; `PhobiaEscalated { from: Mild, to: Moderate }` emitted; counter reset to 0.
- `innate_phobia_static.rs`: tribute with `Mild Phobia(Fire), Innate`; 100 cycles of mixed exposure; severity unchanged, no `PhobiaEscalated` or `PhobiaHabituated` emitted.

Insta snapshots:

- Phobia state on `Tribute` after each integration scenario (BTreeMap of phobias with metadata).
- Ordered MessagePayload streams for acquire→fire→observe→habituate or →freeze→die paths.

Proptest properties (per `uz80`):

- **Observer monotonicity within cycle**: observer set only grows during a single cycle (additions never reverse mid-cycle); decay only happens at cycle boundary.
- **Habituation reset on fire**: any cycle where `is_present` returns true → `cycles_since_last_fire` becomes 0 (Traumatic origin; Innate has no counter).
- **Reinforcement bounded**: over N seeded trials of Traumatic firings at sub-Severe severity, escalation rate hits `firing_escalation_chance` ±tolerance.
- **Reinforcement and decay mutually exclusive**: in any cycle, a Traumatic phobia either reinforces (counter→0, possible escalation) or decays (counter++, possible tier-down at threshold), never both.
- **Innate immutability**: for any sequence of cycles, an Innate phobia's severity is constant.
- **Cap improvement**: post-acquisition, the multiset of phobia severities is lexicographically ≥ pre-acquisition multiset (acquisitions strictly improve total severity).
- **Freeze probability**: over N seeded trials at Severe severity, freeze rate hits 25% ±tolerance.
- **Trait modifier symmetry**: Resilient + Fragile cancels (effective severity equals base).

## 15. Migration / rollout

No data migration required. New affliction kind variant is additive; existing tributes default to no phobias; the new fields on phobia metadata are `Default`-able.

SurrealDB schema: phobia metadata serializes as part of the existing affliction object (added optional field). No schema change required if the affliction array uses a flexible object shape (verify in PR1 of this spec).

Rollout: ship behind a `phobias_enabled: bool` config flag in `Game` for one PR cycle so balance-testing is easy. Default `true` once tuning settles. Spawn-time roll respects the flag.

## 16. PR breakdown

- **PR1**: Types (`PhobiaTrigger`, `PhobiaOrigin`, `PhobiaMetadata`, extension to `AfflictionKind`). Storage (metadata field on `Affliction`). Spawn-time acquisition. Per-cycle scan skeleton (returns nothing). Unit tests for `is_present` truth table. Cap-at-3 logic.
- **PR2**: Phobia layer in brain pipeline. Reactions (`Reaction::{Penalty, AutoFlee, Freeze}`, `Action::Frozen`). Trait modifier composition. Stat penalty composition into `visible_modifiers`. Integration tests for freeze and auto-flee scenarios.
- **PR3**: Observer state (`observed_by`, `observer_seen_cycle`). Visibility-on-firing logic. Decay ticks. `knows_phobia` predicate consumed by alliance and brain decisions. Habituation counter for Traumatic origin. **Reinforcement-on-firing**: counter reset + sensitization escalation roll per §7.1, with cascade-suppression interlock per §6. `PhobiaObserved`, `PhobiaHabituated`, `PhobiaForgotten`, `PhobiaEscalated` payloads. (Note: this PR may land coupled with fixation PR2 (`wss4`) since both implement the shared traumatic-reinforcement rule; see `wss4` for the coupling rationale.)
- **PR4**: Web tribute-detail "Fears" section, timeline phobia card, state strip indicator. Smoke tests; WCAG check on severity colors.

PR1 has no hard prereqs beyond afflictions PR1 (`lsis`). PR2 requires afflictions PR2 (`dyom`) for the phobia layer to slot into the unified pipeline cleanly. PR3 is independent of PR2 and can land in parallel. PR4 lands last.

## 17. Out of scope (filed as follow-ups)

- Weapon/concept/sound phobias (v2 trigger taxonomy expansion)
- Therapy sponsor gift (depends on `dvd`)
- Gamemaker-induced phobia (depends on gamemaker series)
- Tribute-viewport observer-respecting UI (`n52s`)
- District-backstory weighting on innate phobia distribution
- Phobia of one's own actions (e.g. "phobia of killing" after first kill — psychologically interesting but tangential to v1)

## 18. Spec self-review

- **Placeholders**: spawn-roll weights, freeze chance (25%), habituation cycles (5), observer decay cycles (5), `firing_escalation_chance` (12%, target window 10-15%), trait-modifier table — explicit defaults to be tuned post-observability.
- **Internal consistency**: severity ordering, reaction tier table, and visibility-firing rules use the same `Severity` ordering. Observer state is consistent with brain `knows_phobia` consumer. Cap-at-3 composes with the existing affliction `Supersede` resolution.
- **Scope check**: PR1-PR4 are each ~300-500 LOC plus tests. Trauma integration deferred to trauma spec (`cdu0`); v1 lands with Innate phobias only producing meaningful play.
- **Ambiguity**: "in the same area at that moment" defined as `ctx.area.tributes` at the start of the firing tribute's turn — observer set computed once per fire, not continuously.
