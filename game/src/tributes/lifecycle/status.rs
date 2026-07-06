use crate::areas::AreaDetails;
use crate::areas::events::AreaEvent;
use crate::messages::{MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::AfflictionDraft;
use crate::tributes::Tribute;
use crate::tributes::afflictions::trapped::{
    area_event_to_trap, escape_roll_target, get_escape_stat, severity_index, trap_tuning_for,
};
use crate::tributes::statuses::TributeStatus;
use rand::RngExt;
use rand::prelude::*;
use shared::afflictions::{
    AfflictionKind, AfflictionSource, BodyPart, Severity, TrapKind, TrappedMetadata,
    escape_threshold,
};

/// Status effect damage constants
const WOUNDED_DAMAGE: u32 = 1;
const SICK_STRENGTH_REDUCTION: u32 = 1;
const SICK_MOVEMENT_REDUCTION: u32 = 1;
const ELECTROCUTED_DAMAGE: u32 = 20;
const FROZEN_MOVEMENT_REDUCTION: u32 = 1;
const OVERHEATED_MOVEMENT_REDUCTION: u32 = 1;
const DEHYDRATED_STRENGTH_REDUCTION: u32 = 1;
const STARVING_STRENGTH_REDUCTION: u32 = 1;
const POISONED_MENTAL_DAMAGE: u32 = 5;
const BROKEN_BONE_LEG_MOVEMENT_REDUCTION: u32 = 10;
const BROKEN_BONE_ARM_STRENGTH_REDUCTION: u32 = 5;
const BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION: u32 = 5;
const BROKEN_BONE_RIB_DAMAGE: u32 = 5;
const INFECTED_DAMAGE: u32 = 2;
const INFECTED_MENTAL_DAMAGE: u32 = 5;
const BURNED_DAMAGE: u32 = 5;

impl Tribute {
    /// Set the tribute's status
    pub fn set_status(&mut self, status: TributeStatus) {
        self.status = status;
    }

    /// Applies statuses to the tribute based on events in the current area.
    pub(crate) fn apply_area_effects(&mut self, area_details: &AreaDetails) {
        for event in &area_details.events {
            match event {
                AreaEvent::Wildfire => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Burned,
                        body_part: None,
                        severity: Severity::Severe,
                        source: AfflictionSource::Environmental,
                        trapped_metadata: None,
                    });
                }
                AreaEvent::Flood
                | AreaEvent::Earthquake
                | AreaEvent::Avalanche
                | AreaEvent::Landslide
                | AreaEvent::Rockslide => {
                    if let Some((trap_kind, severity)) = area_event_to_trap(event.clone()) {
                        self.try_acquire_affliction(AfflictionDraft {
                            kind: AfflictionKind::Trapped(trap_kind),
                            body_part: None,
                            severity,
                            source: AfflictionSource::Environmental,
                            trapped_metadata: Some(TrappedMetadata::fresh_for(trap_kind, None)),
                        });
                    }
                }
                AreaEvent::Blizzard => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Frozen,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                        trapped_metadata: None,
                    });
                }
                AreaEvent::Heatwave => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Overheated,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                        trapped_metadata: None,
                    });
                }
                AreaEvent::Sandstorm => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Wounded,
                        body_part: Some(BodyPart::Arm),
                        severity: Severity::Mild,
                        source: AfflictionSource::Environmental,
                        trapped_metadata: None,
                    });
                }
                AreaEvent::Drought => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Dehydrated,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                        trapped_metadata: None,
                    });
                }
                // Sinkhole is handled as instant-death in process_event_for_area.
                // This arm is unreachable for alive tributes but required for exhaustive match.
                AreaEvent::Sinkhole => { /* no-op — instant death handled upstream */ }
            }
        }
    }

    /// Applies any effects from elsewhere in the game to the tribute.
    /// This may result in status or attribute changes.
    pub(crate) fn process_status(
        &mut self,
        area_details: &AreaDetails,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        // First, apply any area events for the current area
        self.apply_area_effects(area_details);

        // Apply per-cycle affliction effects
        self.apply_affliction_cycle_effects(rng, events);

        self.events.clear();

        if self.attributes.health == 0 {
            let killer = self.status.clone();
            let line = GameOutput::TributeDiesFromStatus(self.name.as_str(), &killer.to_string())
                .to_string();
            let cause = match &killer {
                TributeStatus::Mauled(animal) => {
                    let beast = match animal {
                        crate::threats::animals::Animal::Wolf
                        | crate::threats::animals::Animal::Hyena => {
                            shared::afflictions::BeastKind::Wolf
                        }
                        crate::threats::animals::Animal::Bear => {
                            shared::afflictions::BeastKind::Bear
                        }
                        crate::threats::animals::Animal::Snake => {
                            shared::afflictions::BeastKind::Snake
                        }
                        _ => shared::afflictions::BeastKind::Other,
                    };
                    shared::afflictions::DeathCause::Beast(beast)
                }
                _ => shared::afflictions::DeathCause::Unknown,
            };
            events.push(TaggedEvent::new(
                line,
                MessagePayload::TributeKilled {
                    victim: TributeRef {
                        identifier: self.identifier.clone().into(),
                        name: self.name.clone(),
                    },
                    killer: None,
                    cause,
                },
            ));
            self.statistics.killed_by = Some(killer.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    /// Apply per-cycle effects for each affliction the tribute carries.
    /// Replaces the legacy TributeStatus-based damage loop.
    fn apply_affliction_cycle_effects(
        &mut self,
        rng: &mut impl Rng,
        events: &mut Vec<TaggedEvent>,
    ) {
        // Collect affliction kinds first to avoid borrow conflicts
        let kinds: Vec<AfflictionKind> = self.afflictions.keys().map(|(k, _)| k.clone()).collect();

        for kind in &kinds {
            match kind {
                AfflictionKind::Wounded => {
                    self.attributes.health = self.attributes.health.saturating_sub(WOUNDED_DAMAGE);
                }
                AfflictionKind::Sick => {
                    self.reduce_strength(SICK_STRENGTH_REDUCTION);
                    self.reduce_movement(SICK_MOVEMENT_REDUCTION);
                }
                AfflictionKind::Electrocuted => {
                    self.attributes.health =
                        self.attributes.health.saturating_sub(ELECTROCUTED_DAMAGE);
                }
                AfflictionKind::Frozen => {
                    self.reduce_movement(FROZEN_MOVEMENT_REDUCTION);
                }
                AfflictionKind::Overheated => {
                    self.reduce_movement(OVERHEATED_MOVEMENT_REDUCTION);
                }
                AfflictionKind::Dehydrated => {
                    self.reduce_strength(DEHYDRATED_STRENGTH_REDUCTION);
                }
                AfflictionKind::Starving => {
                    self.reduce_strength(STARVING_STRENGTH_REDUCTION);
                }
                AfflictionKind::Poisoned => {
                    self.attributes.sanity = self
                        .attributes
                        .sanity
                        .saturating_sub(POISONED_MENTAL_DAMAGE);
                }
                AfflictionKind::BrokenBone => {
                    let bone = rng.random_range(0..4);
                    match bone {
                        0 => self.reduce_movement(BROKEN_BONE_LEG_MOVEMENT_REDUCTION),
                        1 => self.reduce_strength(BROKEN_BONE_ARM_STRENGTH_REDUCTION),
                        2 => self.reduce_intelligence(BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION),
                        _ => {
                            self.attributes.health = self
                                .attributes
                                .health
                                .saturating_sub(BROKEN_BONE_RIB_DAMAGE)
                        }
                    }
                }
                AfflictionKind::Infected => {
                    self.attributes.health = self.attributes.health.saturating_sub(INFECTED_DAMAGE);
                    self.attributes.sanity = self
                        .attributes
                        .sanity
                        .saturating_sub(INFECTED_MENTAL_DAMAGE);
                }
                AfflictionKind::Burned => {
                    self.attributes.health = self.attributes.health.saturating_sub(BURNED_DAMAGE);
                }
                AfflictionKind::Trapped(kind) => {
                    let tuning = trap_tuning_for(*kind);
                    let sev_idx = severity_index(
                        self.afflictions
                            .get(&(AfflictionKind::Trapped(*kind), None))
                            .map(|a| a.severity)
                            .unwrap_or(Severity::Mild),
                    );
                    // Extract all data before mutating to avoid borrow conflicts
                    let (cycles_trapped, _disorientation_remaining) = self
                        .afflictions
                        .get(&(AfflictionKind::Trapped(*kind), None))
                        .and_then(|a| a.trapped_metadata.as_ref())
                        .map(|m| (m.cycles_trapped, m.disorientation_remaining))
                        .unwrap_or((0, 0));
                    let buried_severity = self
                        .afflictions
                        .get(&(AfflictionKind::Trapped(*kind), None))
                        .map(|a| a.severity);
                    // Clone identifiers for events before mutable borrow
                    let tribute_name = self.name.clone();
                    let tribute_id = self.identifier.clone();
                    let kind_copy = *kind;
                    let escape_stat = get_escape_stat(self, *kind);

                    match kind {
                        TrapKind::Drowning => {
                            self.attributes.sanity = self
                                .attributes
                                .sanity
                                .saturating_sub(tuning.mental_damage[sev_idx]);
                        }
                        TrapKind::Buried => {
                            let progressive =
                                tuning.progressive_damage_per_cycle * cycles_trapped as u32;
                            self.attributes.health = self
                                .attributes
                                .health
                                .saturating_sub(tuning.hp_damage[sev_idx] + progressive);
                            self.attributes.sanity = self
                                .attributes
                                .sanity
                                .saturating_sub(tuning.mental_damage[sev_idx]);
                        }
                        // Pitfall: HP + mental damage per cycle
                        TrapKind::Pitfall => {
                            self.attributes.health = self
                                .attributes
                                .health
                                .saturating_sub(tuning.hp_damage[sev_idx]);
                            self.attributes.sanity = self
                                .attributes
                                .sanity
                                .saturating_sub(tuning.mental_damage[sev_idx]);
                        }
                        // SpikedPitfall: stub — instant death handled upstream
                        TrapKind::SpikedPitfall => { /* no-op */ }
                        // Snared: HP + mental damage per cycle
                        TrapKind::Snared => {
                            self.attributes.health = self
                                .attributes
                                .health
                                .saturating_sub(tuning.hp_damage[sev_idx]);
                            self.attributes.sanity = self
                                .attributes
                                .sanity
                                .saturating_sub(tuning.mental_damage[sev_idx]);
                        }
                        // Pinned: HP + mental damage per cycle
                        TrapKind::Pinned => {
                            self.attributes.health = self
                                .attributes
                                .health
                                .saturating_sub(tuning.hp_damage[sev_idx]);
                            self.attributes.sanity = self
                                .attributes
                                .sanity
                                .saturating_sub(tuning.mental_damage[sev_idx]);
                        }
                    }

                    // Update metadata (separate mutable borrow)
                    if let Some(meta) = self
                        .afflictions
                        .get_mut(&(AfflictionKind::Trapped(*kind), None))
                        .and_then(|a| a.trapped_metadata.as_mut())
                    {
                        match kind {
                            TrapKind::Drowning => {
                                if meta.disorientation_remaining > 0 {
                                    meta.disorientation_remaining -= 1;
                                }
                            }
                            TrapKind::Buried => {
                                let target = escape_roll_target(
                                    escape_stat,
                                    buried_severity.unwrap_or(Severity::Mild),
                                    meta,
                                    meta.rescue_bonus_accumulated,
                                );
                                if rng.random_bool(target as f64) {
                                    meta.escape_progress += 1;
                                }
                            }
                            TrapKind::Pitfall
                            | TrapKind::SpikedPitfall
                            | TrapKind::Snared
                            | TrapKind::Pinned => { /* no per-cycle metadata update */ }
                        }
                        meta.cycles_trapped += 1;
                        meta.rescue_bonus_accumulated = 0.0;
                    }
                    // Emit Struggling message per cycle
                    events.push(TaggedEvent::new(
                        format!("{} is still trapped", tribute_name),
                        MessagePayload::Struggling {
                            tribute: tribute_id.to_string(),
                            kind: kind_copy,
                            severity: buried_severity.unwrap_or(Severity::Mild),
                            cycles_trapped: cycles_trapped + 1,
                        },
                    ));
                }
                AfflictionKind::MissingArm
                | AfflictionKind::MissingLeg
                | AfflictionKind::Blind
                | AfflictionKind::Deaf
                | AfflictionKind::Trauma
                | AfflictionKind::Phobia(_)
                | AfflictionKind::Fixation(_)
                | AfflictionKind::Addiction(_) => {}
            }
        }

        // Mauled status still applies (has Animal data payload)
        if let TributeStatus::Mauled(animal) = &self.status {
            let number_of_animals = rng.random_range(2..=5);
            let damage = animal.damage() * number_of_animals;
            self.attributes.health = self.attributes.health.saturating_sub(damage);
        }

        // Check for escaped Buried afflictions — remove them
        let escaped: Vec<(AfflictionKind, Option<BodyPart>, u8)> = self
            .afflictions
            .iter()
            .filter_map(|((kind, bp), aff)| {
                if let (AfflictionKind::Trapped(TrapKind::Buried), Some(meta)) =
                    (kind, &aff.trapped_metadata)
                    && meta.escape_progress >= escape_threshold(aff.severity)
                {
                    return Some((kind.clone(), *bp, meta.cycles_trapped));
                }
                None
            })
            .collect();
        for (kind, bp, cycles) in escaped {
            let key = (kind.clone(), bp);
            if let AfflictionKind::Trapped(trap_kind) = &kind {
                events.push(TaggedEvent::new(
                    format!("{} escaped!", self.name),
                    MessagePayload::TrappedEscaped {
                        tribute: self.identifier.to_string(),
                        kind: *trap_kind,
                        cycles_trapped: cycles,
                        rescued_by: Vec::new(),
                    },
                ));
            }
            self.afflictions.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::Area;
    use crate::areas::events::AreaEvent;
    use crate::threats::animals::Animal;
    use crate::tributes::AfflictionDraft;
    use crate::tributes::Tribute;
    use crate::tributes::statuses::TributeStatus;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::*;
    use shared::afflictions::{AfflictionKind, AfflictionSource, Severity, TrapKind};

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[rstest]
    fn process_status_mauled(mut tribute: Tribute, mut small_rng: SmallRng) {
        let health = tribute.attributes.health;
        tribute.status = TributeStatus::Mauled(Animal::Bear);
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert!(tribute.attributes.health < health);
    }

    #[rstest]
    #[case(TributeStatus::Mauled(Animal::Bear))]
    fn process_status_no_effect(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
        #[case] status: TributeStatus,
    ) {
        tribute.status = status.clone();
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert!(tribute.is_alive());
    }

    #[rstest]
    fn process_status_dies(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 1;
        tribute.try_acquire_affliction(AfflictionDraft {
            kind: AfflictionKind::Electrocuted,
            body_part: None,
            severity: Severity::Severe,
            source: AfflictionSource::Environmental,
            trapped_metadata: None,
        });
        let area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert_eq!(tribute.attributes.health, 0);
        assert_eq!(tribute.status, TributeStatus::RecentlyDead);
    }

    #[rstest]
    fn process_status_from_area_event(mut tribute: Tribute, mut small_rng: SmallRng) {
        let mut area_details = AreaDetails::new(Some("Forest".to_string()), Area::Cornucopia);
        area_details.events.push(AreaEvent::Wildfire);

        tribute.process_status(&area_details, &mut small_rng, &mut Vec::new());
        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Burned, None))
        );
    }

    #[rstest]
    fn wildfire_sets_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Forest".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Wildfire);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Burned, None))
        );
    }

    #[rstest]
    fn blizzard_sets_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Tundra".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Blizzard);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Frozen, None))
        );
    }

    #[rstest]
    fn heatwave_sets_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Desert".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Heatwave);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Overheated, None))
        );
    }

    #[rstest]
    fn sandstorm_sets_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Desert".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Sandstorm);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Wounded, Some(BodyPart::Arm)))
        );
    }

    #[rstest]
    fn drought_sets_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Desert".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Drought);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Dehydrated, None))
        );
    }

    #[rstest]
    fn flood_sets_trapped_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("River".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Flood);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Trapped(TrapKind::Drowning), None))
        );
    }

    #[rstest]
    fn earthquake_sets_trapped_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Cave".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Earthquake);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Trapped(TrapKind::Buried), None))
        );
    }
}
