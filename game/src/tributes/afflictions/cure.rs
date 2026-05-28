//! Item-based cure logic for afflictions.
//!
//! Maps cure items to affliction kinds and steps severity down by one tier.
//! Mild afflictions are removed entirely.
//!
//! See spec §7 Cure.

use shared::afflictions::{Affliction, AfflictionKind, Severity};

/// Outcome of applying a cure item to a tribute's afflictions.
#[derive(Debug, Clone, PartialEq)]
pub enum CureOutcome {
    /// Affliction severity stepped down (Severe→Moderate, Moderate→Mild) or removed (Mild→gone).
    Cured {
        affliction: AfflictionKind,
        from: Severity,
        /// None means the affliction was removed (was Mild).
        to: Option<Severity>,
    },
    /// No matching affliction found or item has no effect.
    NoEffect { reason: String },
}

/// Recovery cycles needed per affliction kind when sheltered.
/// Wounded=1, Infected=3, Broken=4. Other reversible afflictions default to 2.
pub fn recovery_cycles(kind: AfflictionKind) -> u32 {
    match kind {
        AfflictionKind::Wounded => 1,
        AfflictionKind::Infected => 3,
        AfflictionKind::BrokenBone => 4,
        _ => 2,
    }
}

/// Map a cure item name to the affliction it treats.
///
/// Returns `None` if the item is not a cure item.
pub fn cure_item_to_affliction(item_name: &str) -> Option<AfflictionKind> {
    let lower = item_name.to_lowercase();
    if lower.contains("bandage") {
        return Some(AfflictionKind::Wounded);
    }
    if lower.contains("splint") {
        return Some(AfflictionKind::BrokenBone);
    }
    if lower.contains("antibiotic") || lower.contains("antibiotics") {
        return Some(AfflictionKind::Infected);
    }
    None
}

/// Apply a cure item to a tribute's afflictions.
///
/// Finds the matching affliction and steps severity down by one tier.
/// If the affliction was Mild, it is removed entirely.
///
/// Returns `CureOutcome::Cured` on success, `NoEffect` if no matching affliction.
pub fn apply_cure(afflictions: &mut Vec<Affliction>, item_name: &str) -> CureOutcome {
    let target_kind = match cure_item_to_affliction(item_name) {
        Some(k) => k,
        None => {
            return CureOutcome::NoEffect {
                reason: format!("'{item_name}' is not a cure item"),
            };
        }
    };

    // Find the highest-severity matching affliction to treat.
    let mut best_idx: Option<usize> = None;
    let mut best_severity: Option<Severity> = None;

    for (i, aff) in afflictions.iter().enumerate() {
        if aff.kind == target_kind && !aff.is_permanent() {
            let should_pick = match best_severity {
                None => true,
                Some(best) => aff.severity > best,
            };
            if should_pick {
                best_idx = Some(i);
                best_severity = Some(aff.severity);
            }
        }
    }

    let idx = match best_idx {
        Some(i) => i,
        None => {
            return CureOutcome::NoEffect {
                reason: format!("no {target_kind} affliction to cure"),
            };
        }
    };

    let aff = &afflictions[idx];
    let from = aff.severity;

    match from {
        Severity::Mild => {
            // Mild → remove entirely.
            afflictions.remove(idx);
            CureOutcome::Cured {
                affliction: target_kind,
                from,
                to: None,
            }
        }
        Severity::Moderate => {
            afflictions[idx].severity = Severity::Mild;
            CureOutcome::Cured {
                affliction: target_kind,
                from,
                to: Some(Severity::Mild),
            }
        }
        Severity::Severe => {
            afflictions[idx].severity = Severity::Moderate;
            CureOutcome::Cured {
                affliction: target_kind,
                from,
                to: Some(Severity::Moderate),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::afflictions::{AfflictionSource, BodyPart};

    fn make_affliction(kind: AfflictionKind, severity: Severity) -> Affliction {
        Affliction {
            kind,
            body_part: None,
            severity,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
        }
    }

    #[test]
    fn cure_item_mapping_bandage() {
        assert_eq!(
            cure_item_to_affliction("bandage"),
            Some(AfflictionKind::Wounded)
        );
        assert_eq!(
            cure_item_to_affliction("Medical Bandage"),
            Some(AfflictionKind::Wounded)
        );
    }

    #[test]
    fn cure_item_mapping_splint() {
        assert_eq!(
            cure_item_to_affliction("splint"),
            Some(AfflictionKind::BrokenBone)
        );
    }

    #[test]
    fn cure_item_mapping_antibiotic() {
        assert_eq!(
            cure_item_to_affliction("antibiotic"),
            Some(AfflictionKind::Infected)
        );
        assert_eq!(
            cure_item_to_affliction("Antibiotics"),
            Some(AfflictionKind::Infected)
        );
    }

    #[test]
    fn cure_item_mapping_unknown() {
        assert_eq!(cure_item_to_affliction("health kit"), None);
        assert_eq!(cure_item_to_affliction("food"), None);
    }

    #[test]
    fn apply_cure_severe_to_moderate() {
        let mut affs = vec![make_affliction(AfflictionKind::Wounded, Severity::Severe)];
        let result = apply_cure(&mut affs, "bandage");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::Wounded,
                from: Severity::Severe,
                to: Some(Severity::Moderate),
            }
        );
        assert_eq!(affs[0].severity, Severity::Moderate);
    }

    #[test]
    fn apply_cure_moderate_to_mild() {
        let mut affs = vec![make_affliction(AfflictionKind::Wounded, Severity::Moderate)];
        let result = apply_cure(&mut affs, "bandage");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::Wounded,
                from: Severity::Moderate,
                to: Some(Severity::Mild),
            }
        );
        assert_eq!(affs[0].severity, Severity::Mild);
    }

    #[test]
    fn apply_cure_mild_removes_affliction() {
        let mut affs = vec![make_affliction(AfflictionKind::Wounded, Severity::Mild)];
        let result = apply_cure(&mut affs, "bandage");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::Wounded,
                from: Severity::Mild,
                to: None,
            }
        );
        assert!(affs.is_empty());
    }

    #[test]
    fn apply_cure_no_matching_affliction() {
        let mut affs = vec![make_affliction(
            AfflictionKind::BrokenBone,
            Severity::Moderate,
        )];
        let result = apply_cure(&mut affs, "bandage");
        assert!(
            matches!(result, CureOutcome::NoEffect { .. }),
            "expected NoEffect, got {:?}",
            result
        );
    }

    #[test]
    fn apply_cure_non_cure_item() {
        let mut affs = vec![make_affliction(AfflictionKind::Wounded, Severity::Moderate)];
        let result = apply_cure(&mut affs, "health kit");
        assert!(
            matches!(result, CureOutcome::NoEffect { .. }),
            "expected NoEffect, got {:?}",
            result
        );
    }

    #[test]
    fn apply_cure_treats_highest_severity() {
        let mut affs = vec![
            make_affliction(AfflictionKind::Wounded, Severity::Mild),
            make_affliction(AfflictionKind::Wounded, Severity::Severe),
            make_affliction(AfflictionKind::Wounded, Severity::Moderate),
        ];
        let result = apply_cure(&mut affs, "bandage");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::Wounded,
                from: Severity::Severe,
                to: Some(Severity::Moderate),
            }
        );
        // Severe became Moderate, Mild and Moderate remain.
        assert_eq!(affs.len(), 3);
        let severities: Vec<_> = affs.iter().map(|a| a.severity).collect();
        assert!(severities.contains(&Severity::Moderate));
        assert!(severities.contains(&Severity::Mild));
    }

    #[test]
    fn apply_cure_splint_for_broken_bone() {
        let mut affs = vec![make_affliction(
            AfflictionKind::BrokenBone,
            Severity::Severe,
        )];
        let result = apply_cure(&mut affs, "splint");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::BrokenBone,
                from: Severity::Severe,
                to: Some(Severity::Moderate),
            }
        );
    }

    #[test]
    fn apply_cure_antibiotic_for_infected() {
        let mut affs = vec![make_affliction(
            AfflictionKind::Infected,
            Severity::Moderate,
        )];
        let result = apply_cure(&mut affs, "antibiotic");
        assert_eq!(
            result,
            CureOutcome::Cured {
                affliction: AfflictionKind::Infected,
                from: Severity::Moderate,
                to: Some(Severity::Mild),
            }
        );
    }

    #[test]
    fn recovery_cycles_values() {
        assert_eq!(recovery_cycles(AfflictionKind::Wounded), 1);
        assert_eq!(recovery_cycles(AfflictionKind::Infected), 3);
        assert_eq!(recovery_cycles(AfflictionKind::BrokenBone), 4);
        // Default for others.
        assert_eq!(recovery_cycles(AfflictionKind::Burned), 2);
        assert_eq!(recovery_cycles(AfflictionKind::Poisoned), 2);
    }

    #[test]
    fn apply_cure_with_body_part() {
        let mut affs = vec![Affliction {
            kind: AfflictionKind::Wounded,
            body_part: Some(BodyPart::Arm),
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: "tributes:test".into(),
            },
            acquired_cycle: 1,
            last_progressed_cycle: 1,
            trauma_metadata: None,
            phobia_metadata: None,
            fixation_metadata: None,
        }];
        let result = apply_cure(&mut affs, "bandage");
        assert!(matches!(result, CureOutcome::Cured { .. }));
        assert_eq!(affs[0].severity, Severity::Mild);
        assert_eq!(affs[0].body_part, Some(BodyPart::Arm));
    }
}
