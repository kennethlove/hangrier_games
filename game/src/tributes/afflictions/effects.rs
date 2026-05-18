//! Stat-effect and brain-bias computation for afflictions.
//!
//! §6 Mechanical Effects — each affliction kind maps to concrete stat
//! penalties and behavioral bias multipliers. Severity tiers scale
//! penalties linearly (Mild = 0.5x, Moderate = 1.0x, Severe = 1.5x).

use shared::afflictions::{Affliction, AfflictionKind, Severity};

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

/// Aggregated stat modifiers produced by a set of afflictions.
///
/// All fields are penalties (negative = worse) except `stamina_move_pct`
/// which is a fractional *increase* in move cost (e.g. 0.75 = +75% cost).
/// `hp_per_cycle` is negative for drain (Infected = -1).
#[derive(Debug, Clone, Copy, Default)]
pub struct StatModifiers {
    pub atk: i32,
    pub def: i32,
    pub forage: i32,
    pub escape: i32,
    pub ambush_detect: i32,
    /// Fractional increase to stamina move cost. 0.75 means +75% cost.
    pub stamina_move_pct: f64,
    /// Change to max stamina (negative = reduced pool).
    pub stamina_max: i32,
    /// HP change per cycle (negative = drain).
    pub hp_per_cycle: i32,
}

/// Compute additive stat modifiers from a tribute's afflictions.
///
/// Penalties are summed across all afflictions, scaled by severity,
/// then clamped so atk/def/forage/escape/ambush_detect never drop
/// below their negative baseline (i.e. the penalty is capped at the
/// magnitude of the raw spec values — no double-Severe overflow).
pub fn compute_stat_modifiers(afflictions: &[Affliction]) -> StatModifiers {
    let mut mods = StatModifiers::default();

    for aff in afflictions {
        let m = severity_multiplier(aff.severity);
        let (atk, def, forage, escape, ambush_detect, stamina_move, stamina_max, hp) =
            base_penalties(aff.kind);

        mods.atk += (atk as f64 * m).round() as i32;
        mods.def += (def as f64 * m).round() as i32;
        mods.forage += (forage as f64 * m).round() as i32;
        mods.escape += (escape as f64 * m).round() as i32;
        mods.ambush_detect += (ambush_detect as f64 * m).round() as i32;
        mods.stamina_move_pct += stamina_move * m;
        mods.stamina_max += (stamina_max as f64 * m).round() as i32;
        mods.hp_per_cycle += (hp as f64 * m).round() as i32;
    }

    // Clamp: penalties should not reduce effective stats below zero.
    // The raw spec values represent the maximum reasonable penalty,
    // so we cap each field at its most-negative spec magnitude.
    mods.atk = mods.atk.max(-12);
    mods.def = mods.def.max(-12);
    mods.forage = mods.forage.max(-8);
    mods.escape = mods.escape.max(-6);
    mods.ambush_detect = mods.ambush_detect.max(-6);
    mods.stamina_move_pct = mods.stamina_move_pct.min(2.0);
    mods.stamina_max = mods.stamina_max.max(-4);
    mods.hp_per_cycle = mods.hp_per_cycle.max(-3);

    mods
}

/// Base (Moderate-tier) penalty values per affliction kind.
/// Values match spec §6 Mechanical Effects table.
///
/// Returns:
/// (atk, def, forage, escape, ambush_detect, stamina_move_pct, stamina_max, hp_per_cycle)
fn base_penalties(kind: AfflictionKind) -> (i32, i32, i32, i32, i32, f64, i32, i32) {
    match kind {
        AfflictionKind::MissingArm => (-2, -2, 0, 0, 0, 0.0, 0, 0),
        AfflictionKind::MissingLeg => (0, 0, 0, -3, 0, 0.75, 0, 0),
        AfflictionKind::Blind => (-6, -4, -2, 0, 0, 0.0, 0, 0),
        AfflictionKind::Deaf => (0, 0, 0, 0, -3, 0.0, 0, 0),
        AfflictionKind::BrokenBone => (-3, -3, 0, 0, 0, 0.5, 0, 0),
        AfflictionKind::Infected => (0, 0, 0, 0, 0, 0.0, -1, -1),
        AfflictionKind::Wounded => (-1, -1, 0, 0, 0, 0.0, 0, 0),
        // Non-table afflictions: no direct stat penalties in v1.
        // Their effects are handled elsewhere (survival bands, etc.).
        AfflictionKind::Poisoned
        | AfflictionKind::Starving
        | AfflictionKind::Dehydrated
        | AfflictionKind::Frozen
        | AfflictionKind::Overheated
        | AfflictionKind::Burned => (0, 0, 0, 0, 0, 0.0, 0, 0),
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

/// Compute brain bias multipliers from a tribute's afflictions.
///
/// Bias weights compose multiplicatively: `final = base * affliction_multiplier`.
/// A tribute with no afflictions gets neutral bias (all 1.0).
pub fn compute_brain_bias(afflictions: &[Affliction]) -> BrainBias {
    let mut bias = BrainBias::neutral();

    for aff in afflictions {
        let m = severity_multiplier(aff.severity);
        let (ca, sp, iso, ws, rp) = base_bias(aff.kind);

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
        // Non-table afflictions: no direct brain bias in v1
        AfflictionKind::Poisoned
        | AfflictionKind::Starving
        | AfflictionKind::Dehydrated
        | AfflictionKind::Frozen
        | AfflictionKind::Overheated
        | AfflictionKind::Burned => (1.0, 1.0, 1.0, 1.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::afflictions::{AfflictionSource, BodyPart};

    fn aff(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: None,
            severity,
            source: AfflictionSource::Combat,
        }
    }

    fn aff_with_part(kind: AfflictionKind, part: BodyPart, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: Some(part),
            severity,
            source: AfflictionSource::Combat,
        }
    }

    // ── Stat modifier tests ────────────────────────────────────────────

    #[test]
    fn empty_afflictions_yields_zero_modifiers() {
        let mods = compute_stat_modifiers(&[]);
        assert_eq!(mods.atk, 0);
        assert_eq!(mods.def, 0);
        assert_eq!(mods.forage, 0);
        assert_eq!(mods.escape, 0);
        assert_eq!(mods.ambush_detect, 0);
        assert_eq!(mods.stamina_move_pct, 0.0);
        assert_eq!(mods.stamina_max, 0);
        assert_eq!(mods.hp_per_cycle, 0);
    }

    #[test]
    fn missing_arm_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert_eq!(mods.atk, -2);
        assert_eq!(mods.def, -2);
    }

    #[test]
    fn missing_arm_severe_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Severe,
        )]);
        assert_eq!(mods.atk, -3);
        assert_eq!(mods.def, -3);
    }

    #[test]
    fn missing_arm_mild_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Mild,
        )]);
        assert_eq!(mods.atk, -1);
        assert_eq!(mods.def, -1);
    }

    #[test]
    fn missing_leg_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingLeg,
            BodyPart::Leg,
            Severity::Moderate,
        )]);
        assert_eq!(mods.escape, -3);
        assert!((mods.stamina_move_pct - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn blind_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::Blind,
            BodyPart::Eye,
            Severity::Moderate,
        )]);
        assert_eq!(mods.atk, -6);
        assert_eq!(mods.def, -4);
        assert_eq!(mods.forage, -2);
    }

    #[test]
    fn deaf_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::Deaf,
            BodyPart::Ear,
            Severity::Moderate,
        )]);
        assert_eq!(mods.ambush_detect, -3);
    }

    #[test]
    fn broken_bone_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::BrokenBone,
            BodyPart::Leg,
            Severity::Moderate,
        )]);
        assert_eq!(mods.atk, -3);
        assert_eq!(mods.def, -3);
        assert!((mods.stamina_move_pct - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn infected_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::Infected,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert_eq!(mods.hp_per_cycle, -1);
        assert_eq!(mods.stamina_max, -1);
    }

    #[test]
    fn wounded_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::Wounded,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        assert_eq!(mods.atk, -1);
        assert_eq!(mods.def, -1);
    }

    #[test]
    fn multiple_afflictions_stack_additively() {
        let afflictions = vec![
            aff_with_part(
                AfflictionKind::MissingArm,
                BodyPart::Arm,
                Severity::Moderate,
            ),
            aff_with_part(AfflictionKind::Wounded, BodyPart::Leg, Severity::Moderate),
        ];
        let mods = compute_stat_modifiers(&afflictions);
        assert_eq!(mods.atk, -3);
        assert_eq!(mods.def, -3);
    }

    #[test]
    fn severity_scaling_missing_arm() {
        let mild = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Mild,
        )]);
        let moderate = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Moderate,
        )]);
        let severe = compute_stat_modifiers(&[aff_with_part(
            AfflictionKind::MissingArm,
            BodyPart::Arm,
            Severity::Severe,
        )]);

        // Higher severity = more negative penalty, so mild > moderate > severe
        // (because -1 > -2 > -3)
        assert!(mild.atk > moderate.atk);
        assert!(moderate.atk > severe.atk);
    }

    #[test]
    fn stat_modifiers_clamp_reasonably() {
        let afflictions = vec![
            aff_with_part(AfflictionKind::Blind, BodyPart::Eye, Severity::Severe),
            aff_with_part(AfflictionKind::MissingArm, BodyPart::Arm, Severity::Severe),
            aff_with_part(AfflictionKind::BrokenBone, BodyPart::Leg, Severity::Severe),
            aff_with_part(AfflictionKind::Wounded, BodyPart::Rib, Severity::Severe),
        ];
        let mods = compute_stat_modifiers(&afflictions);
        // Should be clamped, not arbitrarily negative
        assert!(mods.atk >= -12);
        assert!(mods.def >= -12);
    }

    // ── Brain bias tests ───────────────────────────────────────────────

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
    fn non_table_afflictions_no_effects() {
        let mods = compute_stat_modifiers(&[aff(AfflictionKind::Poisoned, Severity::Severe)]);
        assert_eq!(mods.atk, 0);
        assert_eq!(mods.def, 0);

        let bias = compute_brain_bias(&[aff(AfflictionKind::Starving, Severity::Severe)]);
        assert_eq!(bias.combat_avoid, 1.0);
        assert_eq!(bias.shelter_preference, 1.0);
    }
}
