use shared::wounds::{BodyPart, WoundSeverity};

/// Maximum blood pool for a tribute.
pub const MAX_BLOOD: u32 = 1000;

/// Blood level at which a tribute dies.
pub const DEATH_THRESHOLD: u32 = 0;

/// Blood restored per period by natural clotting (applied to all wounds that
/// are still bleeding).
pub const NATURAL_CLOT_RATE: u32 = 5;

/// Infection chance per period for Critical wounds (0.0-1.0).
pub const CRITICAL_INFECTION_CHANCE: f64 = 0.25;

/// Blood restored per period from food/rest.
pub const REST_BLOOD_RESTORE: u32 = 20;

/// --- Stat penalties per wound severity ---
/// Penalty to effective strength per wound severity tier.
pub fn strength_penalty(severity: WoundSeverity) -> i32 {
    match severity {
        WoundSeverity::Minor => -1,
        WoundSeverity::Moderate => -3,
        WoundSeverity::Severe => -6,
        WoundSeverity::Critical => -10,
    }
}

/// Penalty to effective movement per wound severity tier.
pub fn movement_penalty(severity: WoundSeverity) -> i32 {
    match severity {
        WoundSeverity::Minor => -1,
        WoundSeverity::Moderate => -2,
        WoundSeverity::Severe => -5,
        WoundSeverity::Critical => -10,
    }
}

/// Penalty to effective defense per wound severity tier.
pub fn defense_penalty(severity: WoundSeverity) -> i32 {
    match severity {
        WoundSeverity::Minor => 0,
        WoundSeverity::Moderate => -1,
        WoundSeverity::Severe => -3,
        WoundSeverity::Critical => -5,
    }
}

/// Penalty to effective bravery per wound severity tier.
pub fn bravery_penalty(severity: WoundSeverity) -> i32 {
    match severity {
        WoundSeverity::Minor => 0,
        WoundSeverity::Moderate => -1,
        WoundSeverity::Severe => -3,
        WoundSeverity::Critical => -5,
    }
}

/// Additional penalty multiplier for Head wounds (applied on top of severity).
pub const HEAD_WOUND_MULTIPLIER: f64 = 1.5;

/// Additional penalty multiplier for Torso wounds.
pub const TORSO_WOUND_MULTIPLIER: f64 = 1.2;

/// Limb that triggers amputation when Severe+ wound is on it.
pub const AMPUTATION_SEVERITY_THRESHOLD: WoundSeverity = WoundSeverity::Severe;

pub fn body_part_penalty_multiplier(part: BodyPart) -> f64 {
    match part {
        BodyPart::Head => HEAD_WOUND_MULTIPLIER,
        BodyPart::Torso => TORSO_WOUND_MULTIPLIER,
        BodyPart::LeftArm | BodyPart::RightArm | BodyPart::LeftLeg | BodyPart::RightLeg => 1.0,
    }
}

/// Heroism trigger: when blood drops below this percentage, tribute gets a
/// temporary bravery boost.
pub const HEROISM_BLOOD_THRESHOLD: f64 = 0.25;

/// Bravery boost applied when heroism triggers.
pub const HEROISM_BRAVERY_BOOST: u32 = 20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn penalty_increases_with_severity() {
        let minor = strength_penalty(WoundSeverity::Minor);
        let moderate = strength_penalty(WoundSeverity::Moderate);
        let severe = strength_penalty(WoundSeverity::Severe);
        let critical = strength_penalty(WoundSeverity::Critical);
        assert!(minor > moderate);
        assert!(moderate > severe);
        assert!(severe > critical);
    }

    #[test]
    fn head_wound_has_highest_multiplier() {
        let head = body_part_penalty_multiplier(BodyPart::Head);
        let torso = body_part_penalty_multiplier(BodyPart::Torso);
        let arm = body_part_penalty_multiplier(BodyPart::LeftArm);
        assert!(head > torso);
        assert!(torso >= arm);
    }

    #[test]
    fn zero_severity_movement_penalty() {
        assert_eq!(movement_penalty(WoundSeverity::Minor), -1);
    }
}
