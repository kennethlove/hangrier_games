//! Trapped affliction implementation: tuning table, escape mechanic, AreaEvent mapping.
//!
//! See `docs/superpowers/specs/2026-05-04-trapped-afflictions-design.md`.

use crate::areas::events::AreaEvent;
use shared::afflictions::{Severity, TrapKind};

/// Which tribute attribute to use for escape roll computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeStat {
    Strength,
    Intelligence,
}

/// Per-kind tuning for trapped afflictions.
#[derive(Debug, Clone, Copy)]
pub struct TrapKindTuning {
    pub kind: TrapKind,
    /// Per-cycle HP damage indexed by severity (Mild=0, Moderate=1, Severe=2).
    pub hp_damage: [u32; 3],
    /// Per-cycle mental damage indexed by severity.
    pub mental_damage: [u32; 3],
    /// Which stat to use for self-escape roll bonus.
    pub escape_stat: EscapeStat,
    /// Which stat to use for rescuer bonus contribution.
    pub rescue_stat: EscapeStat,
    /// Whether the trap can have a terrain hazard floor (Drowning yes, Buried no).
    pub allows_terrain_floor: bool,
    /// For Drowning: one-time HP loss on acquisition (not per-cycle).
    pub initial_hp_loss: u32,
    /// For Buried: additional HP loss per cycle beyond the base (progressive).
    pub progressive_damage_per_cycle: u32,
}

pub const TRAP_KIND_TABLE: &[TrapKindTuning] = &[
    TrapKindTuning {
        kind: TrapKind::Drowning,
        // Drowning is "washed ashore" — no damage (ordeal damage + recovery)
        // Main consequence is forced relocation + disorientation stat penalty
        hp_damage: [0, 0, 0],
        mental_damage: [0, 0, 0],
        escape_stat: EscapeStat::Intelligence,
        rescue_stat: EscapeStat::Strength,
        allows_terrain_floor: true,
        initial_hp_loss: 0,
        progressive_damage_per_cycle: 0,
    },
    TrapKindTuning {
        kind: TrapKind::Buried,
        // Buried is "cave-in" — no damage; escape is time-based via progress rolls
        hp_damage: [0, 0, 0],
        mental_damage: [0, 0, 0],
        escape_stat: EscapeStat::Strength,
        rescue_stat: EscapeStat::Strength,
        allows_terrain_floor: false,
        initial_hp_loss: 0,
        progressive_damage_per_cycle: 0,
    },
];

pub fn trap_tuning_for(kind: TrapKind) -> &'static TrapKindTuning {
    TRAP_KIND_TABLE
        .iter()
        .find(|t| t.kind == kind)
        .expect("TRAP_KIND_TABLE must have a row for every TrapKind variant")
}

pub fn severity_index(severity: Severity) -> usize {
    match severity {
        Severity::Mild => 0,
        Severity::Moderate => 1,
        Severity::Severe => 2,
    }
}

/// Map an AreaEvent to a Trapped affliction kind and severity.
/// Returns `None` for AreaEvents that don't produce trapped afflictions.
pub fn area_event_to_trap(event: AreaEvent) -> Option<(TrapKind, Severity)> {
    match event {
        AreaEvent::Flood => Some((TrapKind::Drowning, Severity::Severe)),
        AreaEvent::Earthquake => Some((TrapKind::Buried, Severity::Severe)),
        AreaEvent::Avalanche => Some((TrapKind::Buried, Severity::Moderate)),
        AreaEvent::Landslide => Some((TrapKind::Buried, Severity::Moderate)),
        AreaEvent::Rockslide => Some((TrapKind::Buried, Severity::Mild)),
        _ => None,
    }
}

/// Compute the escape roll TARGET probability for a trapped tribute.
/// Returns a value in `[0.0, 1.0]`.
///
/// Called for Buried (dig-out). Drowning has no escape — disorientation
/// recovery is time-based (handled in lifecycle).
///
/// Arguments:
/// - `escape_stat_value`: the tribute's escape stat as a fraction `[0.0, 1.0]`
/// - `severity`: affliction severity
/// - `meta`: TrappedMetadata (cycles_trapped, terrain_hazard_floor)
/// - `rescue_bonus`: sum of rescue contributions this cycle (0.0 if none)
pub fn escape_roll_target(
    escape_stat_value: f32,
    severity: Severity,
    meta: &shared::afflictions::TrappedMetadata,
    rescue_bonus: f32,
) -> f32 {
    use shared::afflictions::{
        CYCLES_DECAY_PER_CYCLE, ESCAPE_ROLL_CAP, ESCAPE_STAT_BONUS_MAX, SEVERITY_BASE_MILD,
        SEVERITY_BASE_MODERATE, SEVERITY_BASE_SEVERE,
    };

    let base = match severity {
        Severity::Mild => SEVERITY_BASE_MILD,
        Severity::Moderate => SEVERITY_BASE_MODERATE,
        Severity::Severe => SEVERITY_BASE_SEVERE,
    };
    let stat_bonus = escape_stat_value.clamp(0.0, 1.0) * ESCAPE_STAT_BONUS_MAX;
    let decay = (meta.cycles_trapped as f32) * CYCLES_DECAY_PER_CYCLE;

    let mut target = (base + stat_bonus + rescue_bonus - decay).clamp(0.0, ESCAPE_ROLL_CAP);

    if let Some(floor) = meta.terrain_hazard_floor {
        target = target.min(floor);
    }

    target
}

/// Extract the escape stat value as a fraction `[0.0, 1.0]` from a Tribute.
pub fn get_escape_stat(tribute: &crate::tributes::Tribute, kind: TrapKind) -> f32 {
    let tuning = trap_tuning_for(kind);
    match tuning.escape_stat {
        EscapeStat::Strength => {
            let val = tribute.attributes.strength as f32;
            let max = crate::config::GameConfig::default().max_strength as f32;
            (val / max).clamp(0.0, 1.0)
        }
        EscapeStat::Intelligence => {
            let val = tribute.attributes.intelligence as f32;
            let max = crate::config::GameConfig::default().max_intelligence as f32;
            (val / max).clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::events::AreaEvent;
    use rstest::rstest;
    use shared::afflictions::{Severity, TrappedMetadata, escape_threshold};

    #[test]
    fn escape_threshold_is_one_for_all_severities() {
        assert_eq!(escape_threshold(Severity::Mild), 1);
        assert_eq!(escape_threshold(Severity::Moderate), 1);
        assert_eq!(escape_threshold(Severity::Severe), 1);
    }

    #[rstest]
    #[case(TrapKind::Drowning)]
    #[case(TrapKind::Buried)]
    fn trap_tuning_for_returns_matching_row(#[case] kind: TrapKind) {
        let t = trap_tuning_for(kind);
        assert_eq!(t.kind, kind);
    }

    #[test]
    fn drowning_uses_intelligence_for_escape() {
        assert_eq!(
            trap_tuning_for(TrapKind::Drowning).escape_stat,
            EscapeStat::Intelligence
        );
    }

    #[test]
    fn buried_uses_strength_for_escape() {
        assert_eq!(
            trap_tuning_for(TrapKind::Buried).escape_stat,
            EscapeStat::Strength
        );
    }

    #[test]
    fn drowning_allows_terrain_floor() {
        assert!(trap_tuning_for(TrapKind::Drowning).allows_terrain_floor);
    }

    #[test]
    fn buried_disallows_terrain_floor() {
        assert!(!trap_tuning_for(TrapKind::Buried).allows_terrain_floor);
    }

    #[rstest]
    #[case(AreaEvent::Flood, Some((TrapKind::Drowning, Severity::Severe)))]
    #[case(AreaEvent::Earthquake, Some((TrapKind::Buried, Severity::Severe)))]
    #[case(AreaEvent::Avalanche, Some((TrapKind::Buried, Severity::Moderate)))]
    #[case(AreaEvent::Landslide, Some((TrapKind::Buried, Severity::Moderate)))]
    #[case(AreaEvent::Rockslide, Some((TrapKind::Buried, Severity::Mild)))]
    #[case(AreaEvent::Wildfire, None)]
    #[case(AreaEvent::Blizzard, None)]
    #[case(AreaEvent::Drought, None)]
    fn area_event_mapping_matches_spec(
        #[case] event: AreaEvent,
        #[case] expected: Option<(TrapKind, Severity)>,
    ) {
        assert_eq!(area_event_to_trap(event), expected);
    }

    // ── escape_roll_target tests ──────────────────────────────────────────

    fn meta(cycles: u8, floor: Option<f32>) -> TrappedMetadata {
        TrappedMetadata {
            cycles_trapped: cycles,
            escape_progress: 0,
            terrain_hazard_floor: floor,
            disorientation_remaining: 0,
        }
    }

    #[test]
    fn escape_target_mild_zero_stat_no_decay_no_rescue() {
        // base 0.50 + 0.0 stat - 0.0 decay = 0.50
        let t = escape_roll_target(0.0, Severity::Mild, &meta(0, None), 0.0);
        assert!((t - 0.50).abs() < 1e-6, "got {t}");
    }

    #[test]
    fn escape_target_severe_max_stat_no_decay_no_rescue() {
        // base 0.15 + 0.30 stat = 0.45
        let t = escape_roll_target(1.0, Severity::Severe, &meta(0, None), 0.0);
        assert!((t - 0.45).abs() < 1e-6, "got {t}");
    }

    #[test]
    fn escape_target_decays_per_cycle() {
        let t0 = escape_roll_target(1.0, Severity::Moderate, &meta(0, None), 0.0);
        let t1 = escape_roll_target(1.0, Severity::Moderate, &meta(1, None), 0.0);
        let t2 = escape_roll_target(1.0, Severity::Moderate, &meta(2, None), 0.0);
        assert!((t0 - t1 - 0.02).abs() < 1e-6);
        assert!((t1 - t2 - 0.02).abs() < 1e-6);
    }

    #[test]
    fn escape_target_capped_at_0_95() {
        let t = escape_roll_target(1.0, Severity::Mild, &meta(0, None), 10.0);
        assert_eq!(t, 0.95);
    }

    #[test]
    fn escape_target_clamped_to_zero() {
        let t = escape_roll_target(0.0, Severity::Severe, &meta(20, None), 0.0);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn escape_target_terrain_floor_caps_below_computed() {
        // Computed would be 0.15 + 0.30 = 0.45, terrain floor is 0.30, result stays 0.30
        let t = escape_roll_target(1.0, Severity::Severe, &meta(0, Some(0.30)), 0.0);
        assert_eq!(t, 0.30);
    }

    #[test]
    fn escape_target_rescue_bonus_contributes() {
        let t_no_rescue = escape_roll_target(0.5, Severity::Severe, &meta(0, None), 0.0);
        let t_rescued = escape_roll_target(0.5, Severity::Severe, &meta(0, None), 0.40);
        assert!((t_rescued - t_no_rescue - 0.40).abs() < 1e-6);
    }
}
