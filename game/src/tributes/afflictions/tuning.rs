//! Placeholder tuning constants for afflictions. Defaults are explicit
//! starting values to be tuned post-observability (spec §5).

/// Tunable knobs for the affliction system. Numbers are placeholders.
/// PR3 wires these into cascade / cure logic; PR1 only defines the shape.
#[derive(Debug, Clone)]
pub struct AfflictionTuning {
    /// Per-cycle probability that an exposed reversible affliction steps up one tier.
    pub progression_chance: f32,
    /// Per-cycle probability that a sheltered reversible affliction steps down one tier.
    pub shelter_recovery_chance: f32,
    /// Per-cycle probability that Severe Wounded spawns Infected.
    pub wound_to_infection_chance: f32,
    /// Per-cycle mortality probability for Severe Infected exposed tributes.
    pub severe_infected_death_chance: f32,
}

impl Default for AfflictionTuning {
    fn default() -> Self {
        Self {
            progression_chance: 0.10,
            shelter_recovery_chance: 0.25,
            wound_to_infection_chance: 0.15,
            severe_infected_death_chance: 0.10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tuning_is_in_unit_range() {
        let t = AfflictionTuning::default();
        for v in [
            t.progression_chance,
            t.shelter_recovery_chance,
            t.wound_to_infection_chance,
            t.severe_infected_death_chance,
        ] {
            assert!((0.0..=1.0).contains(&v), "tuning value {v} out of [0,1]");
        }
    }
}
