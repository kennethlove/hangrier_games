//! Combat inflict tables: maps (weapon kind, hit severity) to affliction drafts.
//!
//! Per spec §12, critical hits and BreakMidSwing outcomes have higher weights
//! for severe afflictions. Placeholder weights; tuned later with data.

use crate::tributes::AfflictionDraft;
use rand::RngExt;
use rand::prelude::*;
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

/// A single entry in an inflict table row.
#[derive(Debug, Clone)]
pub struct InflictEntry {
    pub kind: AfflictionKind,
    /// Relative weight for weighted random selection.
    pub base_weight: f64,
}

/// Which "severity band" the attack result falls into for inflict purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSeverity {
    /// Normal hit (AttackerWins, DefenderWins).
    Normal,
    /// Heavy hit (AttackerWinsDecisively, DefenderWinsDecisively, PerfectBlock counter).
    Heavy,
    /// Critical hit (CriticalHit — triple damage).
    Critical,
}

/// Which weapon category was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Unarmed,
    Bladed,
    Blunt,
    Ranged,
}

/// Look up affliction inflicts for a given weapon + severity combination.
///
/// Returns a Vec of 0–2 AfflictionDrafts selected via weighted random.
/// The RNG seed should come from the combat RNG so results are reproducible.
pub fn lookup_inflicts(
    weapon: WeaponKind,
    severity: HitSeverity,
    _attacker_id: &str,
    rng: &mut impl Rng,
) -> Vec<AfflictionDraft> {
    let table = inflict_table(weapon, severity);
    if table.is_empty() {
        return Vec::new();
    }

    // Number of inflicts: 1 for Normal, 1-2 for Heavy, 2 for Critical.
    let count = match severity {
        HitSeverity::Normal => 1,
        HitSeverity::Heavy => {
            if rng.random_bool(0.5) {
                2
            } else {
                1
            }
        }
        HitSeverity::Critical => 2,
    };

    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        if let Some(entry) = weighted_select(&table, rng) {
            let body_part = select_body_part(entry.kind.clone(), rng);
            let sev = select_severity(&entry, severity, rng);
            result.push(AfflictionDraft {
                kind: entry.kind.clone(),
                body_part,
                severity: sev,
                source: AfflictionSource::Combat {
                    attacker_id: String::new(),
                },
            });
        }
    }
    result
}

/// BreakMidSwing recoil inflict: when a weapon shatters mid-swing, the
/// attacker suffers a recoil injury. Always produces exactly one draft.
pub fn lookup_break_mid_swing_inflict(
    weapon: WeaponKind,
    _attacker_id: &str,
    rng: &mut impl Rng,
) -> Option<AfflictionDraft> {
    let table = break_mid_swing_table(weapon);
    weighted_select(&table, rng).map(|entry| {
        let body_part = select_body_part(entry.kind.clone(), rng);
        AfflictionDraft {
            kind: entry.kind.clone(),
            body_part,
            severity: Severity::Moderate,
            source: AfflictionSource::Combat {
                attacker_id: String::new(),
            },
        }
    })
}

/// Select a body part appropriate for the affliction kind.
fn select_body_part(kind: AfflictionKind, rng: &mut impl Rng) -> Option<BodyPart> {
    match kind {
        AfflictionKind::Wounded | AfflictionKind::BrokenBone | AfflictionKind::Infected => {
            // Combat wounds attach to a body part.
            let parts = [
                BodyPart::Arm,
                BodyPart::Leg,
                BodyPart::Rib,
                BodyPart::Hand,
                BodyPart::Foot,
            ];
            Some(*parts.choose(rng).unwrap())
        }
        AfflictionKind::MissingArm => Some(BodyPart::Arm),
        AfflictionKind::MissingLeg => Some(BodyPart::Leg),
        AfflictionKind::Blind => Some(BodyPart::Eye),
        AfflictionKind::Deaf => Some(BodyPart::Ear),
        // Non-body afflictions.
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
        | AfflictionKind::Trauma
        | AfflictionKind::Phobia(_)
        | AfflictionKind::Fixation(_) => None,
    }
}

/// Select severity based on the entry weight and hit severity band.
fn select_severity(_entry: &InflictEntry, hit: HitSeverity, rng: &mut impl Rng) -> Severity {
    // Higher hit severity shifts probability toward Severe.
    let severe_chance = match hit {
        HitSeverity::Normal => 0.1,
        HitSeverity::Heavy => 0.3,
        HitSeverity::Critical => 0.6,
    };
    let moderate_chance = match hit {
        HitSeverity::Normal => 0.5,
        HitSeverity::Heavy => 0.5,
        HitSeverity::Critical => 0.3,
    };
    // Remainder is Mild.

    let roll: f64 = rng.random();
    if roll < severe_chance {
        Severity::Severe
    } else if roll < severe_chance + moderate_chance {
        Severity::Moderate
    } else {
        Severity::Mild
    }
}

/// Weighted random selection from a list of entries.
fn weighted_select(table: &[InflictEntry], rng: &mut impl Rng) -> Option<InflictEntry> {
    if table.is_empty() {
        return None;
    }
    let total: f64 = table.iter().map(|e| e.base_weight).sum();
    if total <= 0.0 {
        return None;
    }
    let mut roll: f64 = rng.random::<f64>() * total;
    for entry in table {
        roll -= entry.base_weight;
        if roll <= 0.0 {
            return Some(entry.clone());
        }
    }
    Some(table.last().unwrap().clone())
}

/// The full inflict table keyed by (WeaponKind, HitSeverity).
fn inflict_table(weapon: WeaponKind, severity: HitSeverity) -> Vec<InflictEntry> {
    use AfflictionKind as K;
    match (weapon, severity) {
        // ── Unarmed ─────────────────────────────────────────────────────
        (WeaponKind::Unarmed, HitSeverity::Normal) => vec![InflictEntry {
            kind: K::Wounded,
            base_weight: 10.0,
        }],
        (WeaponKind::Unarmed, HitSeverity::Heavy) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 8.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 2.0,
            },
        ],
        (WeaponKind::Unarmed, HitSeverity::Critical) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 5.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 4.0,
            },
            InflictEntry {
                kind: K::Blind,
                base_weight: 1.0,
            },
        ],

        // ── Bladed ──────────────────────────────────────────────────────
        (WeaponKind::Bladed, HitSeverity::Normal) => vec![InflictEntry {
            kind: K::Wounded,
            base_weight: 10.0,
        }],
        (WeaponKind::Bladed, HitSeverity::Heavy) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 7.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 2.0,
            },
            InflictEntry {
                kind: K::MissingArm,
                base_weight: 0.5,
            },
            InflictEntry {
                kind: K::MissingLeg,
                base_weight: 0.5,
            },
        ],
        (WeaponKind::Bladed, HitSeverity::Critical) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 4.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 3.0,
            },
            InflictEntry {
                kind: K::MissingArm,
                base_weight: 2.0,
            },
            InflictEntry {
                kind: K::MissingLeg,
                base_weight: 2.0,
            },
            InflictEntry {
                kind: K::Blind,
                base_weight: 1.0,
            },
        ],

        // ── Blunt ───────────────────────────────────────────────────────
        (WeaponKind::Blunt, HitSeverity::Normal) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 8.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 2.0,
            },
        ],
        (WeaponKind::Blunt, HitSeverity::Heavy) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 5.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 5.0,
            },
        ],
        (WeaponKind::Blunt, HitSeverity::Critical) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 3.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 5.0,
            },
            InflictEntry {
                kind: K::Blind,
                base_weight: 1.0,
            },
            InflictEntry {
                kind: K::Deaf,
                base_weight: 1.0,
            },
        ],

        // ── Ranged ──────────────────────────────────────────────────────
        (WeaponKind::Ranged, HitSeverity::Normal) => vec![InflictEntry {
            kind: K::Wounded,
            base_weight: 10.0,
        }],
        (WeaponKind::Ranged, HitSeverity::Heavy) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 6.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 3.0,
            },
            InflictEntry {
                kind: K::Blind,
                base_weight: 1.0,
            },
        ],
        (WeaponKind::Ranged, HitSeverity::Critical) => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 3.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 4.0,
            },
            InflictEntry {
                kind: K::Blind,
                base_weight: 2.0,
            },
            InflictEntry {
                kind: K::Deaf,
                base_weight: 1.0,
            },
        ],
    }
}

/// BreakMidSwing recoil table: what affliction the attacker gets when their
/// weapon shatters. Always moderate severity.
fn break_mid_swing_table(weapon: WeaponKind) -> Vec<InflictEntry> {
    use AfflictionKind as K;
    match weapon {
        WeaponKind::Unarmed => vec![], // No recoil for unarmed
        WeaponKind::Bladed => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 6.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 3.0,
            },
            InflictEntry {
                kind: K::Burned,
                base_weight: 1.0,
            }, // shard cuts
        ],
        WeaponKind::Blunt => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 5.0,
            },
            InflictEntry {
                kind: K::BrokenBone,
                base_weight: 5.0,
            },
        ],
        WeaponKind::Ranged => vec![
            InflictEntry {
                kind: K::Wounded,
                base_weight: 7.0,
            },
            InflictEntry {
                kind: K::Burned,
                base_weight: 3.0,
            }, // bow snap / explosion
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn inflict_table_non_empty_for_all_combos() {
        let weapons = [
            WeaponKind::Unarmed,
            WeaponKind::Bladed,
            WeaponKind::Blunt,
            WeaponKind::Ranged,
        ];
        let severities = [
            HitSeverity::Normal,
            HitSeverity::Heavy,
            HitSeverity::Critical,
        ];
        for w in &weapons {
            for s in &severities {
                let table = inflict_table(*w, *s);
                assert!(
                    !table.is_empty(),
                    "inflict_table({:?}, {:?}) is empty",
                    w,
                    s
                );
            }
        }
    }

    #[test]
    fn lookup_inflicts_returns_correct_count() {
        let mut rng = SmallRng::seed_from_u64(42);
        let drafts = lookup_inflicts(
            WeaponKind::Bladed,
            HitSeverity::Normal,
            "attacker-1",
            &mut rng,
        );
        assert_eq!(drafts.len(), 1);

        let drafts = lookup_inflicts(
            WeaponKind::Bladed,
            HitSeverity::Critical,
            "attacker-1",
            &mut rng,
        );
        assert_eq!(drafts.len(), 2);
    }

    #[test]
    fn lookup_inflicts_produces_valid_drafts() {
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..20 {
            let drafts = lookup_inflicts(
                WeaponKind::Blunt,
                HitSeverity::Heavy,
                "attacker-1",
                &mut rng,
            );
            for d in &drafts {
                assert!(matches!(
                    d.kind,
                    AfflictionKind::Wounded
                        | AfflictionKind::BrokenBone
                        | AfflictionKind::Blind
                        | AfflictionKind::Deaf
                ));
                assert!(matches!(
                    d.severity,
                    Severity::Mild | Severity::Moderate | Severity::Severe
                ));
            }
        }
    }

    #[test]
    fn break_mid_swing_unarmed_returns_none() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = lookup_break_mid_swing_inflict(WeaponKind::Unarmed, "attacker-1", &mut rng);
        assert!(result.is_none());
    }

    #[test]
    fn break_mid_swing_bladed_returns_affliction() {
        let mut rng = SmallRng::seed_from_u64(42);
        let result = lookup_break_mid_swing_inflict(WeaponKind::Bladed, "attacker-1", &mut rng);
        assert!(result.is_some());
        let d = result.unwrap();
        assert!(matches!(
            d.kind,
            AfflictionKind::Wounded | AfflictionKind::BrokenBone | AfflictionKind::Burned
        ));
        assert_eq!(d.severity, Severity::Moderate);
    }

    #[test]
    fn weighted_select_respects_weights() {
        let mut rng = SmallRng::seed_from_u64(0);
        let table = vec![
            InflictEntry {
                kind: AfflictionKind::Wounded,
                base_weight: 90.0,
            },
            InflictEntry {
                kind: AfflictionKind::Blind,
                base_weight: 10.0,
            },
        ];
        let mut wounded_count = 0;
        for _ in 0..100 {
            if let Some(e) = weighted_select(&table, &mut rng)
                && e.kind == AfflictionKind::Wounded
            {
                wounded_count += 1;
            }
        }
        // Should be roughly 90% wounded
        assert!(wounded_count > 70);
    }

    #[test]
    fn body_part_assignment_correct() {
        let mut rng = SmallRng::seed_from_u64(42);
        assert_eq!(
            select_body_part(AfflictionKind::MissingArm, &mut rng),
            Some(BodyPart::Arm)
        );
        assert_eq!(
            select_body_part(AfflictionKind::MissingLeg, &mut rng),
            Some(BodyPart::Leg)
        );
        assert_eq!(
            select_body_part(AfflictionKind::Blind, &mut rng),
            Some(BodyPart::Eye)
        );
        assert_eq!(
            select_body_part(AfflictionKind::Deaf, &mut rng),
            Some(BodyPart::Ear)
        );
        assert_eq!(select_body_part(AfflictionKind::Poisoned, &mut rng), None);
        // Wounded should get a body part
        assert!(select_body_part(AfflictionKind::Wounded, &mut rng).is_some());
    }
}
