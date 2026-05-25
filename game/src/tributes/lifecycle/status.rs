use crate::areas::AreaDetails;
use crate::areas::events::AreaEvent;
use crate::messages::{MessagePayload, TaggedEvent, TributeRef};
use crate::output::GameOutput;
use crate::tributes::AfflictionDraft;
use crate::tributes::Tribute;
use crate::tributes::statuses::TributeStatus;
use rand::RngExt;
use rand::prelude::*;
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

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
const DROWNED_DAMAGE: u32 = 2;
const DROWNED_MENTAL_DAMAGE: u32 = 2;
const BURNED_DAMAGE: u32 = 5;
const BURIED_DAMAGE: u32 = 5;

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
                    });
                }
                AreaEvent::Flood => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Drowned,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Earthquake => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Buried,
                        body_part: None,
                        severity: Severity::Severe,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Avalanche => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Buried,
                        body_part: None,
                        severity: Severity::Severe,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Blizzard => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Frozen,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Landslide => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Buried,
                        body_part: None,
                        severity: Severity::Severe,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Heatwave => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Overheated,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Sandstorm => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Wounded,
                        body_part: Some(BodyPart::Arm),
                        severity: Severity::Mild,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Drought => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Dehydrated,
                        body_part: None,
                        severity: Severity::Moderate,
                        source: AfflictionSource::Environmental,
                    });
                }
                AreaEvent::Rockslide => {
                    self.try_acquire_affliction(AfflictionDraft {
                        kind: AfflictionKind::Buried,
                        body_part: None,
                        severity: Severity::Severe,
                        source: AfflictionSource::Environmental,
                    });
                }
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
        self.apply_affliction_cycle_effects(rng);

        self.events.clear();

        if self.attributes.health == 0 {
            let killer = self.status.clone();
            let line = GameOutput::TributeDiesFromStatus(self.name.as_str(), &killer.to_string())
                .to_string();
            events.push(TaggedEvent::new(
                line,
                MessagePayload::TributeKilled {
                    victim: TributeRef {
                        identifier: self.identifier.clone(),
                        name: self.name.clone(),
                    },
                    killer: None,
                    cause: killer.to_string(),
                },
            ));
            self.statistics.killed_by = Some(killer.to_string());
            self.status = TributeStatus::RecentlyDead;
        }
    }

    /// Apply per-cycle effects for each affliction the tribute carries.
    /// Replaces the legacy TributeStatus-based damage loop.
    fn apply_affliction_cycle_effects(&mut self, rng: &mut impl Rng) {
        // Collect affliction kinds first to avoid borrow conflicts
        let kinds: Vec<AfflictionKind> = self.afflictions.keys().map(|(k, _)| *k).collect();

        for kind in &kinds {
            match kind {
                AfflictionKind::Wounded => {
                    self.takes_physical_damage(WOUNDED_DAMAGE);
                }
                AfflictionKind::Sick => {
                    self.reduce_strength(SICK_STRENGTH_REDUCTION);
                    self.reduce_movement(SICK_MOVEMENT_REDUCTION);
                }
                AfflictionKind::Electrocuted => {
                    self.takes_physical_damage(ELECTROCUTED_DAMAGE);
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
                    self.takes_mental_damage(POISONED_MENTAL_DAMAGE);
                }
                AfflictionKind::BrokenBone => {
                    let bone = rng.random_range(0..4);
                    match bone {
                        0 => self.reduce_movement(BROKEN_BONE_LEG_MOVEMENT_REDUCTION),
                        1 => self.reduce_strength(BROKEN_BONE_ARM_STRENGTH_REDUCTION),
                        2 => self.reduce_intelligence(BROKEN_BONE_SKULL_INTELLIGENCE_REDUCTION),
                        _ => self.takes_physical_damage(BROKEN_BONE_RIB_DAMAGE),
                    }
                }
                AfflictionKind::Infected => {
                    self.takes_physical_damage(INFECTED_DAMAGE);
                    self.takes_mental_damage(INFECTED_MENTAL_DAMAGE);
                }
                AfflictionKind::Drowned => {
                    self.takes_physical_damage(DROWNED_DAMAGE);
                    self.takes_mental_damage(DROWNED_MENTAL_DAMAGE);
                }
                AfflictionKind::Burned => {
                    self.takes_physical_damage(BURNED_DAMAGE);
                }
                AfflictionKind::Buried => {
                    self.takes_physical_damage(BURIED_DAMAGE);
                }
                AfflictionKind::MissingArm
                | AfflictionKind::MissingLeg
                | AfflictionKind::Blind
                | AfflictionKind::Deaf
                | AfflictionKind::Trauma
                | AfflictionKind::Phobia(_) => {}
            }
        }

        // Mauled status still applies (has Animal data payload)
        if let TributeStatus::Mauled(animal) = &self.status {
            let number_of_animals = rng.random_range(2..=5);
            let damage = animal.damage() * number_of_animals;
            self.takes_physical_damage(damage);
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
    use shared::afflictions::{AfflictionKind, AfflictionSource, Severity};

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
    fn flood_sets_drowned_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("River".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Flood);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Drowned, None))
        );
    }

    #[rstest]
    fn earthquake_sets_buried_affliction(mut tribute: Tribute) {
        let mut area_details =
            AreaDetails::new(Some("Cave".to_string()), crate::areas::Area::Cornucopia);
        area_details.events.push(AreaEvent::Earthquake);

        tribute.apply_area_effects(&area_details);

        assert!(
            tribute
                .afflictions
                .contains_key(&(AfflictionKind::Buried, None))
        );
    }
}
