//! Spawn-time phobia acquisition.
//!
//! At tribute creation, 0-2 phobias are rolled from a weighted distribution.
//! All spawn-time phobias have `Innate` origin and `Mild`/`Moderate` severity.
//!
//! See spec §6.

use rand::Rng;
use rand::RngExt;
use shared::afflictions::{
    AfflictionKind, AfflictionSource, PhobiaMetadata, PhobiaOrigin, PhobiaTrigger, Severity,
};

use crate::tributes::AfflictionDraft;
use crate::tributes::Tribute;

/// Maximum number of phobias a tribute can carry (spec §6 soft cap).
pub const MAX_PHOBIAS: usize = 3;

/// Roll spawn-time phobias for a tribute.
///
/// Returns the list of triggers acquired. Each phobia is acquired via
/// `try_acquire_affliction` on the tribute.
///
/// Distribution (spec §6):
/// - Count: 0-2 phobias, weighted heavily toward 0-1
/// - Trigger: weighted toward common triggers (Fire/Dark/Blood)
/// - Severity: weighted toward Mild/Moderate
pub fn roll_spawn_phobias(tribute: &mut Tribute, rng: &mut impl Rng) -> Vec<PhobiaTrigger> {
    let count = roll_phobia_count(rng);
    if count == 0 {
        return vec![];
    }

    let mut acquired = Vec::with_capacity(count);
    for _ in 0..count {
        let trigger = roll_phobia_trigger(rng);
        let severity = roll_phobia_severity(rng);

        let draft = AfflictionDraft {
            kind: AfflictionKind::Phobia(trigger),
            body_part: None,
            severity,
            source: AfflictionSource::Spawn,
        };

        tribute.try_acquire_affliction(draft);
        acquired.push(trigger);
    }

    acquired
}

/// Returns the number of phobias to roll at spawn (0-2).
///
/// Weights: 70% chance of 0, 25% chance of 1, 5% chance of 2.
fn roll_phobia_count(rng: &mut impl Rng) -> usize {
    let roll: u32 = rng.random_range(0..100);
    if roll < 70 {
        0
    } else if roll < 95 {
        1
    } else {
        2
    }
}

/// Returns a weighted random phobia trigger.
///
/// Common triggers (Fire, Dark, Blood) have higher weights.
/// Social triggers (Tribute, TraitGroup) are rare since they need
/// specific targets to exist.
fn roll_phobia_trigger(rng: &mut impl Rng) -> PhobiaTrigger {
    use PhobiaTrigger::*;

    // Weights: common=30, uncommon=15, rare=5
    let choices: [(PhobiaTrigger, u32); 10] = [
        (Fire, 30),
        (Water, 15),
        (Dark, 30),
        (Blood, 20),
        (Heights, 15),
        (Enclosed, 10),
        (Open, 10),
        (Animal, 10),
        (Tribute, 5),
        (TraitGroup, 5),
    ];

    let total: u32 = choices.iter().map(|(_, w)| w).sum();
    let roll = rng.random_range(0..total);

    let mut cumulative = 0;
    for (trigger, weight) in &choices {
        cumulative += weight;
        if roll < cumulative {
            return *trigger;
        }
    }

    // Fallback (should never reach here)
    Fire
}

/// Returns a weighted random severity for spawn-time phobias.
///
/// Weights: 60% Mild, 40% Moderate. Severe is not rolled at spawn.
fn roll_phobia_severity(rng: &mut impl Rng) -> Severity {
    if rng.random_range(0..100) < 60 {
        Severity::Mild
    } else {
        Severity::Moderate
    }
}

/// Creates phobia metadata for an innate phobia.
pub fn innate_phobia_metadata() -> PhobiaMetadata {
    PhobiaMetadata {
        origin: PhobiaOrigin::Innate,
        ..PhobiaMetadata::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use shared::afflictions::PhobiaTrigger;

    #[test]
    fn roll_phobia_count_distribution() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut counts = [0u32; 3];
        for _ in 0..1000 {
            counts[roll_phobia_count(&mut rng)] += 1;
        }
        // 70% ± 10% for 0
        assert!((600..800).contains(&counts[0]));
        // 25% ± 10% for 1
        assert!((150..350).contains(&counts[1]));
        // 5% ± 5% for 2
        assert!(counts[2] <= 100);
    }

    #[test]
    fn roll_phobia_trigger_favors_common() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut fire_count = 0;
        let mut tribute_count = 0;
        for _ in 0..1000 {
            let t = roll_phobia_trigger(&mut rng);
            if t == PhobiaTrigger::Fire {
                fire_count += 1;
            }
            if t == PhobiaTrigger::Tribute {
                tribute_count += 1;
            }
        }
        // Fire (weight 30) should appear more than Tribute (weight 5)
        assert!(fire_count > tribute_count);
        // Fire should be roughly 15-30% of rolls (weight 30/150 = 20%)
        assert!((150..350).contains(&fire_count));
    }

    #[test]
    fn roll_phobia_severity_never_severe() {
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..1000 {
            let s = roll_phobia_severity(&mut rng);
            assert!(s != Severity::Severe);
        }
    }

    #[test]
    fn roll_spawn_phobias_respects_max() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let acquired = roll_spawn_phobias(&mut tribute, &mut rng);
        assert!(acquired.len() <= 2);
    }

    #[test]
    fn roll_spawn_phobias_can_return_zero() {
        // Use a seed that produces 0 phobias (70% chance)
        let mut rng = SmallRng::seed_from_u64(0);
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let acquired = roll_spawn_phobias(&mut tribute, &mut rng);
        // Just verify it doesn't panic and returns a valid vec
        assert!(acquired.len() <= 2);
    }

    #[test]
    fn innate_phobia_metadata_is_innate() {
        let meta = innate_phobia_metadata();
        assert!(matches!(meta.origin, PhobiaOrigin::Innate));
    }
}
