//! Brain bias computation: maps affliction kinds to behavioral bias
//! multipliers scaled by severity tier.

use shared::afflictions::{Affliction, AfflictionKind, FixationTarget, Severity};

/// Multiplier per severity tier. Permanent afflictions (MissingArm,
/// MissingLeg, Blind, Deaf) are always Severe in practice but still
/// pass through the same scaling pipeline for uniformity.
fn severity_multiplier(severity: Severity) -> f64 {
    match severity {
        Severity::Mild => 0.5,
        Severity::Moderate => 1.0,
        Severity::Severe => 1.5,
    }
}

/// Behavioral bias weights that influence the Brain's decision-making.
///
/// Each field is a multiplicative factor applied to the brain's base
/// preference. Values > 1.0 increase the tendency; < 1.0 decrease it.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrainBias {
    /// Multiplier for combat avoidance. > 1.0 = avoid combat more.
    pub combat_avoid: f64,
    /// Multiplier for shelter-seeking behavior. > 1.0 = prefer shelter.
    pub shelter_preference: f64,
    /// Multiplier for isolation tendency. > 1.0 = prefer being alone.
    pub isolation: f64,
    /// Multiplier for water-seeking behavior. > 1.0 = seek water.
    pub water_seek: f64,
    /// Multiplier for rest preference. > 1.0 = prefer resting.
    pub rest_preference: f64,
}

impl BrainBias {
    /// Neutral bias: all factors at 1.0 (no modification).
    pub fn neutral() -> Self {
        Self {
            combat_avoid: 1.0,
            shelter_preference: 1.0,
            isolation: 1.0,
            water_seek: 1.0,
            rest_preference: 1.0,
        }
    }
}

/// Base (Moderate-tier) bias multipliers per affliction kind.
///
/// Returns: (combat_avoid, shelter_preference, isolation, water_seek, rest_preference)
fn base_bias(kind: AfflictionKind) -> (f64, f64, f64, f64, f64) {
    match kind {
        // Missing arm: avoid 2H combat → moderate combat avoidance
        AfflictionKind::MissingArm => (1.4, 1.0, 1.0, 1.0, 1.0),
        // Missing leg: prefer shelter/stationary
        AfflictionKind::MissingLeg => (1.0, 1.5, 1.0, 1.0, 1.3),
        // Blind: strong shelter preference
        AfflictionKind::Blind => (1.3, 1.8, 1.0, 1.0, 1.2),
        // Deaf: slight isolation
        AfflictionKind::Deaf => (1.0, 1.0, 1.3, 1.0, 1.0),
        // Broken: refuse combat unless cornered
        AfflictionKind::BrokenBone => (1.5, 1.2, 1.0, 1.0, 1.4),
        // Infected: seek water + shelter
        AfflictionKind::Infected => (1.0, 1.4, 1.0, 1.6, 1.3),
        // Wounded: rest preference
        AfflictionKind::Wounded => (1.0, 1.0, 1.0, 1.0, 1.3),
        // Trauma: avoidance behavior from psychological distress.
        // Bias values align with spec: combat_avoid=1.3 (moderate avoidance),
        // shelter_preference=1.2, isolation=1.2, rest_preference=1.2.
        // Severity scales via the composition formula in compute_brain_bias.
        AfflictionKind::Trauma => (1.3, 1.2, 1.2, 1.0, 1.2),
        // Non-table afflictions: no direct brain bias in v1
        AfflictionKind::Poisoned
        | AfflictionKind::Starving
        | AfflictionKind::Dehydrated
        | AfflictionKind::Frozen
        | AfflictionKind::Overheated
        | AfflictionKind::Burned
        | AfflictionKind::Sick
        | AfflictionKind::Electrocuted
        | AfflictionKind::Drowned
        | AfflictionKind::Buried
        | AfflictionKind::Phobia(_)
        | AfflictionKind::Addiction(_)
        | AfflictionKind::Trapped(_) => (1.0, 1.0, 1.0, 1.0, 1.0),
        // Fixation: push toward target
        // Tribute fixation → reduced combat_avoid (want to engage)
        AfflictionKind::Fixation(FixationTarget::Tribute(_)) => (0.7, 1.0, 1.0, 1.0, 1.0),
        // Item fixation → reduced rest_preference (want to explore/loot)
        AfflictionKind::Fixation(FixationTarget::Item(_)) => (1.0, 1.0, 1.0, 1.0, 0.7),
        // Area fixation → reduced isolation (want to travel)
        AfflictionKind::Fixation(FixationTarget::Area(_)) => (1.0, 1.0, 0.7, 1.0, 1.0),
    }
}

/// Compute brain bias multipliers from a tribute's afflictions.
///
/// Bias weights compose multiplicatively: `final = base * affliction_multiplier`.
/// A tribute with no afflictions gets neutral bias (all 1.0).
pub fn compute_brain_bias(afflictions: &[Affliction]) -> BrainBias {
    let mut bias = BrainBias::neutral();

    for aff in afflictions {
        let m = severity_multiplier(aff.severity);
        let (ca, sp, iso, ws, rp) = base_bias(aff.kind.clone());

        bias.combat_avoid *= 1.0 + (ca - 1.0) * m;
        bias.shelter_preference *= 1.0 + (sp - 1.0) * m;
        bias.isolation *= 1.0 + (iso - 1.0) * m;
        bias.water_seek *= 1.0 + (ws - 1.0) * m;
        bias.rest_preference *= 1.0 + (rp - 1.0) * m;
    }

    // Clamp to reasonable bounds
    bias.combat_avoid = bias.combat_avoid.clamp(0.5, 3.0);
    bias.shelter_preference = bias.shelter_preference.clamp(0.5, 3.0);
    bias.isolation = bias.isolation.clamp(0.5, 2.0);
    bias.water_seek = bias.water_seek.clamp(0.5, 3.0);
    bias.rest_preference = bias.rest_preference.clamp(0.5, 3.0);

    bias
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::afflictions::{AfflictionSource, BodyPart, Severity};

    fn aff(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: None,
            severity,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        }
    }

    fn aff_with_part(kind: AfflictionKind, part: BodyPart, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: Some(part),
            severity,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
            acquired_cycle: 0,
            last_progressed_cycle: 0,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
            addiction_metadata: None,
            trapped_metadata: None,
        }
    }

    #[test]
    fn empty_afflictions_yields_neutral_bias() {
        let bias = compute_brain_bias(&[]);
        assert_eq!(bias.combat_avoid, 1.0);
        assert_eq!(bias.shelter_preference, 1.0);
        assert_eq!(bias.isolation, 1.0);
        assert_eq!(bias.water_seek, 1.0);
        assert_eq!(bias.rest_preference, 1.0);
    }

    #[test]
    fn missing_arm_increases_combat_avoid() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert!(bias.combat_avoid > 1.0);
    }

    #[test]
    fn blind_increases_shelter_preference() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Blind,
            BodyPart::Eye,
            Severity::Moderate,
        )]);
        assert!(bias.shelter_preference > 1.0);
    }

    #[test]
    fn deaf_increases_isolation() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Deaf,
            BodyPart::Ear,
            Severity::Moderate,
        )]);
        assert!(bias.isolation > 1.0);
    }

    #[test]
    fn infected_increases_water_seek() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Infected,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert!(bias.water_seek > 1.0);
    }

    #[test]
    fn broken_bone_increases_combat_avoid_and_rest() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::BrokenBone,
            BodyPart::Leg,
            Severity::Moderate,
        )]);
        assert!(bias.combat_avoid > 1.0);
        assert!(bias.rest_preference > 1.0);
    }

    #[test]
    fn wounded_increases_rest_preference() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Wounded,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert!(bias.rest_preference > 1.0);
    }

    #[test]
    fn missing_leg_increases_shelter_and_rest() {
        let bias = compute_brain_bias(&[aff_with_part(
            AfflictionKind::MissingLeg,
            BodyPart::Leg,
            Severity::Moderate,
        )]);
        assert!(bias.shelter_preference > 1.0);
        assert!(bias.rest_preference > 1.0);
    }

    #[test]
    fn multiple_afflictions_compose_multiplicatively() {
        let bias = compute_brain_bias(&[
            aff_with_part(AfflictionKind::Blind, BodyPart::Eye, Severity::Moderate),
            aff_with_part(AfflictionKind::Infected, BodyPart::Arm, Severity::Moderate),
        ]);
        // Both increase shelter_preference, so composite should be higher
        // than either alone
        let blind_only = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Blind,
            BodyPart::Eye,
            Severity::Moderate,
        )]);
        let infected_only = compute_brain_bias(&[aff_with_part(
            AfflictionKind::Infected,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert!(bias.shelter_preference > blind_only.shelter_preference);
        assert!(bias.shelter_preference > infected_only.shelter_preference);
    }

    #[test]
    fn bias_clamped_to_reasonable_bounds() {
        let afflictions = vec![
            aff_with_part(AfflictionKind::Blind, BodyPart::Eye, Severity::Severe),
            aff_with_part(AfflictionKind::BrokenBone, BodyPart::Leg, Severity::Severe),
            aff_with_part(AfflictionKind::MissingArm, BodyPart::Arm, Severity::Severe),
            aff_with_part(AfflictionKind::MissingLeg, BodyPart::Leg, Severity::Severe),
            aff_with_part(AfflictionKind::Infected, BodyPart::Arm, Severity::Severe),
        ];
        let bias = compute_brain_bias(&afflictions);
        assert!(bias.combat_avoid <= 3.0);
        assert!(bias.shelter_preference <= 3.0);
        assert!(bias.isolation <= 2.0);
        assert!(bias.water_seek <= 3.0);
        assert!(bias.rest_preference <= 3.0);
    }

    #[test]
    fn severity_scaling_bias() {
        let mild = compute_brain_bias(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Mild,
        )]);
        let severe = compute_brain_bias(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Severe,
        )]);
        assert!(severe.combat_avoid > mild.combat_avoid);
    }

    #[test]
    fn non_table_afflictions_no_bias_effects() {
        let bias = compute_brain_bias(&[aff(AfflictionKind::Starving, Severity::Severe)]);
        assert_eq!(bias.combat_avoid, 1.0);
        assert_eq!(bias.shelter_preference, 1.0);
    }

    #[test]
    fn fixation_tribute_reduces_combat_avoid() {
        let bias = compute_brain_bias(&[aff(
            AfflictionKind::Fixation(FixationTarget::Tribute("u-1".into())),
            Severity::Moderate,
        )]);
        assert!(
            bias.combat_avoid < 1.0,
            "tribute fixation should reduce combat_avoid"
        );
    }

    #[test]
    fn fixation_item_reduces_rest_preference() {
        let bias = compute_brain_bias(&[aff(
            AfflictionKind::Fixation(FixationTarget::Item("i-1".into())),
            Severity::Moderate,
        )]);
        assert!(
            bias.rest_preference < 1.0,
            "item fixation should reduce rest_preference"
        );
    }

    #[test]
    fn fixation_area_reduces_isolation() {
        let bias = compute_brain_bias(&[aff(
            AfflictionKind::Fixation(FixationTarget::Area("sector1".into())),
            Severity::Moderate,
        )]);
        assert!(
            bias.isolation < 1.0,
            "area fixation should reduce isolation"
        );
    }
}
