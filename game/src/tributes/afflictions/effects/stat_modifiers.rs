//! Stat modifier computation: maps affliction kinds to stat penalties
//! scaled by severity tier.

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
        // Trauma: stat penalties for psychological distress.
        // Moderate-tier base: forage=-2 (distracted), escape=-2 (slower reactions),
        // atk=-1 (combat penalty), def=-1 (less aware).
        // Severity scaling: Mild → -1 forage, -1 escape; Moderate → full base;
        // Severe → -2 atk/def, -3 forage/escape (clamped elsewhere).
        AfflictionKind::Trauma => (-1, -1, -2, -2, 0, 0.0, 0, 0),
        // Non-table afflictions: no direct stat penalties in v1.
        // Their effects are handled elsewhere (survival bands, etc.).
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
        | AfflictionKind::Phobia(_) => (0, 0, 0, 0, 0, 0.0, 0, 0),
    }
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
        }
    }

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

    #[test]
    fn non_table_afflictions_no_stat_effects() {
        let mods = compute_stat_modifiers(&[aff(AfflictionKind::Poisoned, Severity::Severe)]);
        assert_eq!(mods.atk, 0);
        assert_eq!(mods.def, 0);
    }

    #[test]
    fn trauma_mild_penalties() {
        let mods = compute_stat_modifiers(&[aff(AfflictionKind::Trauma, Severity::Mild)]);
        assert!(mods.atk >= -1);
        assert!(mods.def >= -1);
        assert!(mods.forage <= -1);
        assert!(mods.escape <= -1);
    }

    #[test]
    fn trauma_moderate_penalties() {
        let mods = compute_stat_modifiers(&[aff(AfflictionKind::Trauma, Severity::Moderate)]);
        assert_eq!(mods.atk, -1);
        assert_eq!(mods.def, -1);
    }

    #[test]
    fn trauma_severe_penalties_heavier() {
        let mild = compute_stat_modifiers(&[aff(AfflictionKind::Trauma, Severity::Mild)]);
        let severe = compute_stat_modifiers(&[aff(AfflictionKind::Trauma, Severity::Severe)]);
        assert!(severe.atk <= mild.atk);
        assert!(severe.escape <= mild.escape);
    }
}
