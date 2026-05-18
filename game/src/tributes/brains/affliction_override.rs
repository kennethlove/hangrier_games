//! Affliction override layer for the brain pipeline.
//!
//! Runs after stamina_override, before preferred_action. Provides:
//! 1. Hard gates — actions physically impossible due to afflictions are blocked
//! 2. Brain bias — utility scores modified by affliction-derived behavioral weights
//!
//! Pipeline order: [..., survival, stamina, **affliction**, preferred, alliance, consumable]
//!
//! Hard gates (spec §11):
//! - MissingArm (Moderate+) → cannot equip/use 2H weapons
//! - MissingLeg (Moderate+) → cannot enter cliff/swamp-equivalent terrain
//! - Blind (Moderate+) → no ranged attacks
//!
//! Since the Action enum does not yet carry weapon-type or terrain-target
//! details, some gates are enforced at action-execution time rather than
//! decision time. See `Tribute::affliction_action_gate`.

use crate::terrain::BaseTerrain;
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use shared::afflictions::{AfflictionKind, Severity};

/// Check if a tribute has a specific affliction at or above the given severity.
pub fn tribute_has_affliction(
    tribute: &Tribute,
    kind: AfflictionKind,
    min_severity: Severity,
) -> bool {
    tribute
        .afflictions
        .values()
        .any(|a| a.kind == kind && a.severity >= min_severity)
}

/// Hard-gate check on an action given the destination area's terrain.
///
/// Returns `Some(fallback)` if the action is blocked by an affliction.
/// The `destination_terrain` parameter lets us gate MissingLeg terrain
/// restrictions; pass `None` when terrain is unknown (gate is skipped).
///
/// Gates:
/// - MissingLeg (Moderate+) + cliff/swamp destination → blocked (→ Rest)
/// - Blind (Moderate+) + ranged attack → blocked (→ Move(None))
///   (Action::Attack covers both melee and ranged; when the enum gains
///   a distinct ranged variant, this gate will fire on it.)
/// - MissingArm (Moderate+) + 2H weapon use → blocked
///   (Deferred to action-execution time since Action does not carry
///   weapon info yet.)
pub fn hard_gates_with_terrain(
    tribute: &Tribute,
    action: &Action,
    destination_terrain: Option<BaseTerrain>,
) -> Option<Action> {
    let has_missing_leg =
        tribute_has_affliction(tribute, AfflictionKind::MissingLeg, Severity::Moderate);
    let has_blind = tribute_has_affliction(tribute, AfflictionKind::Blind, Severity::Moderate);

    match action {
        // Blind tributes cannot perform ranged attacks.
        // Action::Attack is currently melee+ranged combined; gate fires
        // only when a distinct ranged variant exists.
        _ if has_blind && is_ranged_action(action) => Some(Action::Move(None)),

        // Missing leg: cannot enter cliff/swamp terrain.
        Action::Move(_) if has_missing_leg => {
            if let Some(terrain) = destination_terrain
                && is_forbidden_terrain(terrain)
            {
                return Some(Action::Rest);
            }
            None
        }

        _ => None,
    }
}

/// Returns true if the action is a ranged attack.
/// Currently always false since Action::Attack covers both melee and ranged.
/// When a distinct `Action::RangedAttack` variant is added, update this.
fn is_ranged_action(_action: &Action) -> bool {
    false
}

/// Returns true if the terrain is forbidden for tributes with MissingLeg.
/// Cliff-equivalent: Mountains, Highlands. Swamp-equivalent: Wetlands.
fn is_forbidden_terrain(terrain: BaseTerrain) -> bool {
    matches!(
        terrain,
        BaseTerrain::Mountains | BaseTerrain::Highlands | BaseTerrain::Wetlands
    )
}

/// Apply brain bias from afflictions to influence decision-making.
///
/// Returns a `BrainBias` computed from the tribute's current afflictions.
/// Callers should use this to weight action preferences:
/// - High combat_avoid → prefer Move/Hide over Attack
/// - High shelter_preference → prefer SeekShelter/Rest
/// - High isolation → avoid ProposeAlliance
/// - High water_seek → prefer DrinkFromTerrain/Move toward water
/// - High rest_preference → prefer Rest
pub fn affliction_bias(tribute: &Tribute) -> crate::tributes::afflictions::BrainBias {
    let afflictions: Vec<_> = tribute.afflictions.values().cloned().collect();
    crate::tributes::afflictions::compute_brain_bias(&afflictions)
}

/// Affliction override layer entry point for the pre-decision pipeline.
///
/// Returns `Some(action)` to short-circuit the brain pipeline, or `None`
/// to fall through to preferred_action.
///
/// Terrain is not available at this stage in the legacy `act` entry point,
/// so terrain-dependent gates (MissingLeg) are deferred. The terrain-aware
/// entry point (`decide_action_with_terrain`) should call
/// `hard_gates_with_terrain` separately after deciding a base action.
pub fn affliction_override(tribute: &Tribute, _action: &Action) -> Option<Action> {
    if tribute.afflictions.is_empty() {
        return None;
    }

    // Hard gates that don't require terrain info:
    // - Blind ranged attack (not yet distinguishable from melee)
    // - MissingArm 2H weapon (weapon info not in Action yet)
    // These are deferred to action-execution time. See
    // `Tribute::affliction_action_gate`.

    // Brain bias is computed for callers to apply at the scoring level.
    // The bias itself does not produce an override action — it modifies
    // weights in decide_base. For the pre-decision pipeline, we return
    // None to fall through.

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::tributes::Tribute;
    use shared::afflictions::{Affliction, AfflictionSource, BodyPart};

    fn make_affliction(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: Some(match kind {
                AfflictionKind::MissingArm => BodyPart::Arm,
                AfflictionKind::MissingLeg => BodyPart::Leg,
                AfflictionKind::Blind => BodyPart::Eye,
                AfflictionKind::Deaf => BodyPart::Ear,
                _ => BodyPart::Rib,
            }),
            severity,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
        }
    }

    #[test]
    fn no_afflictions_no_override() {
        let tribute = Tribute::new("Test".to_string(), None, None);
        assert!(affliction_override(&tribute, &Action::Attack).is_none());
    }

    #[test]
    fn mild_affliction_no_hard_gate() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingArm, Severity::Mild);
        tribute.afflictions.insert(aff.key(), aff);
        assert!(!tribute_has_affliction(
            &tribute,
            AfflictionKind::MissingArm,
            Severity::Moderate
        ));
    }

    #[test]
    fn moderate_missing_leg_blocks_mountains() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(BaseTerrain::Mountains),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn moderate_missing_leg_blocks_highlands() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(BaseTerrain::Highlands),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn moderate_missing_leg_blocks_wetlands() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(BaseTerrain::Wetlands),
        );
        assert_eq!(result, Some(Action::Rest));
    }

    #[test]
    fn moderate_missing_leg_allows_forest() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(BaseTerrain::Forest),
        );
        assert!(result.is_none());
    }

    #[test]
    fn missing_leg_no_terrain_info_no_gate() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(&tribute, &Action::Move(Some(Area::Sector1)), None);
        assert!(result.is_none());
    }

    #[test]
    fn affliction_bias_computed_from_afflictions() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::BrokenBone, Severity::Moderate);
        tribute.afflictions.insert(aff.key(), aff);
        let bias = affliction_bias(&tribute);
        assert!(bias.combat_avoid > 1.0);
        assert!(bias.rest_preference > 1.0);
    }

    #[test]
    fn affliction_bias_neutral_without_afflictions() {
        let tribute = Tribute::new("Test".to_string(), None, None);
        let bias = affliction_bias(&tribute);
        assert_eq!(bias.combat_avoid, 1.0);
        assert_eq!(bias.rest_preference, 1.0);
    }

    #[test]
    fn severe_affliction_triggers_gate() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let aff = make_affliction(AfflictionKind::MissingLeg, Severity::Severe);
        tribute.afflictions.insert(aff.key(), aff);
        let result = hard_gates_with_terrain(
            &tribute,
            &Action::Move(Some(Area::Sector1)),
            Some(BaseTerrain::Mountains),
        );
        assert_eq!(result, Some(Action::Rest));
    }
}
