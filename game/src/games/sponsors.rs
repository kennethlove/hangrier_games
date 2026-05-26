use super::*;
use rand::Rng;

impl Game {
    /// Spawn one sponsor per archetype using the shared catalog.
    /// Loyalist gets a randomly-assigned district (1..=12). Budget is rolled
    /// inside the archetype's budget band. Idempotent: no-op if `self.sponsors`
    /// is already populated.
    pub fn spawn_sponsors(&mut self, rng: &mut impl Rng) {
        use shared::sponsors::{ARCHETYPES, ArchetypeId, Sponsor};
        use std::collections::HashMap;

        if !self.sponsors.is_empty() {
            return;
        }

        for (idx, archetype) in ARCHETYPES.iter().enumerate() {
            let (lo, hi) = archetype.budget_band;
            let budget = rng.random_range(lo..=hi);
            let bound_district = if archetype.id == ArchetypeId::Loyalist {
                Some(rng.random_range(1u8..=12))
            } else {
                None
            };

            self.sponsors.push(Sponsor {
                id: idx as u32,
                archetype: archetype.id,
                budget_remaining: budget,
                bound_district,
                affinity: HashMap::new(),
            });
        }
    }

    /// Test helper: returns `(canonical_name, tribute_identifier, affinity)` triples.
    pub fn sponsor_affinity_snapshot(&self) -> Vec<(&'static str, String, i32)> {
        let mut out = Vec::new();
        for s in &self.sponsors {
            let mut entries: Vec<_> = s.affinity.iter().collect();
            entries.sort_by_key(|(k, _)| (*k).clone());
            for (tribute, value) in entries {
                out.push((s.canonical_name(), tribute.clone(), *value));
            }
        }
        out
    }
}
