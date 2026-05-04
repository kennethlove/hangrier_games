# Fixations — v1 Design

**Status:** Approved (brainstorming complete, awaiting implementation plan)
**Author:** klove
**Date:** 2026-05-03
**Builds on:** afflictions v1 (`docs/superpowers/specs/2026-05-03-health-conditions-design.md`), phobias v1 (`docs/superpowers/specs/2026-05-03-phobias-design.md`)

## Purpose

Fixations are appetitive afflictions: an obsessive *pull toward* a target. They are the inverse of phobias (which push *away* from triggers) and reuse the same affliction storage, brain-pipeline override pattern, and observer-aware visibility model. Each fixation drives target selection, loot priority, or movement bias depending on what the tribute is fixated on.

Fixations exist to produce dramatic emergent storylines — vendettas, signature-weapon hoarding, territorial obsession — and to give the trauma spec (`cdu0`) a second affliction kind it can produce alongside phobias.

## Storage

Fixations are stored as `Affliction { kind: AfflictionKind::Fixation(target), severity, origin, observed_by, .. }` in the existing `BTreeMap<AfflictionKey, Affliction>` from the affliction foundation.

```rust
pub enum FixationTarget {
    Tribute(TributeId),
    Item(ItemId),
    Area(AreaId),
}
```

`AfflictionKind` gains a `Fixation(FixationTarget)` variant. The `AfflictionKey` discriminator uses the *variant tag* of `FixationTarget`, not the inner ID — so a second `Fixation(Tribute(_))` collides with the first regardless of which tribute is targeted. This enforces the per-kind cap (see Cap below).

## Severity tiers

Three tiers mirror phobias and base afflictions:

| Tier | Behavior |
|---|---|
| **Mild** | Subtle bias toward target. Brain picks the fixation target when other candidates are equally viable (tiebreaker). |
| **Moderate** | Strong bias. Brain picks the fixation target over candidates that are equal or only slightly better. |
| **Severe** | Compulsion. Brain locks onto the fixation target and will not pick alternatives unless the target is unreachable this cycle. |

The override semantics differ per `FixationTarget` variant:

- `Tribute(_)` overrides combat target selection (and movement-toward when out of range)
- `Item(_)` overrides loot/gather priority
- `Area(_)` overrides movement target

## Origin

```rust
pub enum FixationOrigin {
    Innate,
    Acquired { event_ref: EventRef },
}
```

- **Innate** — permanent. Spawned via the random spawn-time roll (and later, via the district-backstory weighting follow-up). Does not decay.
- **Acquired** — produced by trauma events or item-pickup events. Decays after N contact-free cycles (default N = 5, matches phobia decay). **Contact resets the decay timer AND has a small chance (~10–15%) to escalate severity Mild→Moderate→Severe.**

"Contact" per target kind:
- `Tribute(id)` — same area, line of sight, or combat with `id`
- `Item(id)` — possessing `id`, or having `id` in the same area
- `Area(id)` — being in `id`

The reinforcement rule (timer reset + escalation chance) is **shared with phobias** via spec amendment `hangrier_games-1uhw`. Both systems use the same "traumatic affliction reinforcement" mechanic — phobias sensitize on firing, fixations sensitize on contact.

## Resolution

A fixation ends when any of:

1. **Target loss** — `Tribute` target dies; `Item` target is destroyed or permanently looted by an unreachable rival; `Area` target becomes inaccessible. Emits `FixationThwarted { reason }`.
2. **Decay** — Acquired fixations only: N contact-free cycles elapse. Emits `FixationFaded`.
3. **Consummation** — `Tribute` target killed by the fixated tribute; `Item` target held continuously for N cycles; `Area` target held continuously for N cycles. Emits `FixationConsummated`.

Innate fixations cannot decay but can still be thwarted or consummated.

## Brain pipeline

Insertion point: between `stamina` and `phobia`.

```
stamina → fixation → phobia → affliction → ...rest
```

Fixations take precedence over phobias by design. Rationale: this is a game first, simulation second. The Hollywood moment of charging through a fire to reach a hated rival is more valuable to the storytelling than psychological realism (where fear would dominate). Documented tradeoff.

The phobia layer remains unchanged and behaves identically when no fixation is firing — fixation precedence only matters when both layers want different actions for the same target.

The fixation layer's override:
- Inspects all fixations on the tribute, sorted by severity descending.
- For each fixation, evaluates whether the target is reachable / actionable this cycle.
- The highest-severity actionable fixation produces the override.
- Mild → tiebreaker (only fires if other candidates are equal value).
- Moderate → strong-bias (fires if alternatives are not significantly better).
- Severe → compulsion (always fires unless target unreachable).

## Acquisition

Three producers:

1. **Spawn-time roll** — small probability, produces `Innate` fixation with random target picked from valid candidates (other tributes for `Tribute`, common items for `Item`, districts for `Area`).
2. **Trauma events** (via spec `cdu0`) — produces `Acquired` fixations with `event_ref` pointing to the trauma. Trauma can produce phobias OR fixations from the same event (witnessing a death can produce both fear of the killer AND fixation on revenge).
3. **Item pickup events** — when a tribute picks up an item, small probability to acquire `Acquired Fixation(Item(id))`. Models possession-driven obsession ("found the perfect axe").

Tribute fixations from in-arena events come exclusively through trauma in v1. Rationale: first-contact rolling for fixations would produce noise (every encounter rolling). Pre-arena interactions (training, ceremonies) are rationalized as the source of `Innate Fixation(Tribute(_))` via the spawn-time roll.

## Cap

**2 fixations per tribute, max 1 per kind.** Allowed combinations: Tribute+Item, Tribute+Area, or Item+Area. Disallowed: two Tribute fixations, two Item fixations, two Area fixations.

Enforcement: at insertion time, check both the total cap and the per-kind cap. If at cap, drop the new fixation (do not replace existing).

## Visibility

Severity-gated, mirroring base afflictions:

- **Mild** — hidden from other tributes' brain decisions
- **Moderate** — visible to tributes in the same area
- **Severe** — public (visible to all tributes)

The fixated tribute always "knows" their own fixation (acts on it openly).

`observed_by: BTreeSet<TributeId>` per fixation, with the same observer infrastructure as phobias (5-cycle decay on observer entries when not refreshed).

**Frontend visibility is unconditional.** The player always sees the full fixation list with severity, target, origin, and observer status in the tribute detail view. The visibility tiers above gate only AI brain decisions, not human-facing UI.

## Trait modifiers

- **Resilient** — `-1` effective fixation tier (Severe behaves as Moderate, Moderate as Mild, Mild has no effect).
- **Fragile** — `+1` effective fixation tier (Mild behaves as Moderate, Moderate as Severe, Severe is unchanged).
- **Loyal** — `+1` effective fixation tier *only when target is `Tribute(ally_id)` for an ally tribute*. Models "won't abandon the alliance even when tactically wrong."

No **Reckless** modifier. Reckless tributes already get the dramatic charges-through-danger moments via the pipeline ordering (fixation > phobia); a Reckless modifier on fixations themselves would over-fire.

## Messages

```rust
pub enum MessagePayload {
    // ...existing...
    FixationAcquired { tribute: TributeId, target: FixationTarget, severity: Severity, origin: FixationOrigin },
    FixationEscalated { tribute: TributeId, target: FixationTarget, old_severity: Severity, new_severity: Severity },
    FixationFired { tribute: TributeId, target: FixationTarget, severity: Severity, action: FixationAction },
    FixationConsummated { tribute: TributeId, target: FixationTarget },
    FixationThwarted { tribute: TributeId, target: FixationTarget, reason: ThwartReason },
    FixationFaded { tribute: TributeId, target: FixationTarget },
}

pub enum FixationAction { TargetPick, LootPick, MovePick }

pub enum ThwartReason { TargetLost, TargetUnreachable }
```

`FixationFired` is the high-frequency variant (Severe fixations could fire every cycle). Frontend needs a debounce or "ongoing fixation" condensed card to keep the timeline readable. Filed as a follow-up bead.

## Frontend

Tribute detail view gains a "Fixations" section listing each fixation with target name, severity, origin, observer summary, and consummation/decay progress (where applicable).

Timeline gets a `fixation_card.rs` component handling all six message variants, with `FixationFired` cards condensed when consecutive.

## Cross-spec dependencies

- **Afflictions PR1 (`lsis`)** — provides `Affliction`, `AfflictionKey`, `AfflictionKind`, severity, origin, observer infrastructure. Fixation PR1 blocked by this.
- **Phobia PR2 (`k87i`)** — establishes the brain-pipeline override-layer pattern and `Action` extensions. Fixation PR2 reuses both. Blocked by this.
- **Phobia spec amendment (`1uhw`)** — fixation reinforcement rule must apply to phobias too. Both specs share the rule. The amendment is part of the fixation work; phobia PR3 (`qqqx`) is blocked by it.
- **Trauma spec (`cdu0`)** — canonical producer of `Acquired` fixations. Related, not blocking.

## v2 follow-ups (filed as separate beads)

- District-backstory weighting for `Innate` fixation spawn rolls (mirrors phobia follow-up `nr7x`).
- `FixationFired` debounce / condensed timeline card.
- "Crisis" tier above Severe (self-destructive obsession).
- Sponsor gift to remove or weaken a fixation (mirrors phobia follow-up `luy9`).
- Gamemaker fixation inflict (mirrors phobia follow-up `7m7w`).
- First-contact tribute fixation roll (rejected for v1; reconsider if Tribute fixations are too rare in playtest).

## Out of scope (v1)

- Fixations on actions (e.g., "must perform ritual") — rejected, conflicts with brain-pipeline action override layers.
- Fixations on concepts (e.g., "must win") — too abstract to detect.
- Fixation transfer (one tribute's fixation passing to another via observation) — interesting, but no foundation for cross-tribute affliction inheritance yet.
