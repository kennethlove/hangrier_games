use crate::areas::Area;
use crate::tributes::Tribute;
use rand::Rng;
use rand::RngExt;
use shared::messages::SleepIncidentKind;

/// Probability (0-100) that a sleep incident occurs per sleeping phase.
const SLEEP_INCIDENT_CHANCE_PCT: u32 = 18;

/// Full sleep incident data used internally by the game engine.
#[derive(Debug, Clone, PartialEq)]
pub enum SleepIncident {
    /// Annoying but harmless (flavor only).
    Annoying { flavor: AnnoyingFlavor },
    /// A random item was stolen.
    Theft { stolen_item: String },
    /// Area relocation while unconscious.
    Relocation { new_area: Area },
    /// Animal encounter (named).
    AnimalEncounter { animal: String },
    /// Hallucination/dream causing sanity loss.
    Hallucination,
    /// Ally abandoned the sleeper.
    AllyAbandonment,
    /// Comedic limb injury (leg fell asleep, etc.).
    LimbInjury,
}

/// Flavor variants for the `Annoying` tier — no mechanical effect.
#[derive(Debug, Clone, PartialEq)]
pub enum AnnoyingFlavor {
    SquirrelOnChest,
    ButterflyLanded,
    WeirdDream,
    MouseRanOver,
    LeafOnFace,
}

impl AnnoyingFlavor {
    fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..5) {
            0 => AnnoyingFlavor::SquirrelOnChest,
            1 => AnnoyingFlavor::ButterflyLanded,
            2 => AnnoyingFlavor::WeirdDream,
            3 => AnnoyingFlavor::MouseRanOver,
            _ => AnnoyingFlavor::LeafOnFace,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            AnnoyingFlavor::SquirrelOnChest => "a squirrel on their chest",
            AnnoyingFlavor::ButterflyLanded => "a butterfly landing on their nose",
            AnnoyingFlavor::WeirdDream => "a weird dream about turnips",
            AnnoyingFlavor::MouseRanOver => "a mouse running over their face",
            AnnoyingFlavor::LeafOnFace => "a leaf drifting onto their face",
        }
    }
}

impl From<&SleepIncident> for SleepIncidentKind {
    fn from(incident: &SleepIncident) -> Self {
        match incident {
            SleepIncident::Annoying { .. } => SleepIncidentKind::Annoying,
            SleepIncident::Theft { .. } => SleepIncidentKind::Theft,
            SleepIncident::Relocation { .. } => SleepIncidentKind::Relocation,
            SleepIncident::AnimalEncounter { animal } => SleepIncidentKind::AnimalEncounter {
                animal: animal.clone(),
            },
            SleepIncident::Hallucination => SleepIncidentKind::Hallucination,
            SleepIncident::AllyAbandonment => SleepIncidentKind::AllyAbandonment,
            SleepIncident::LimbInjury => SleepIncidentKind::LimbInjury,
        }
    }
}

impl SleepIncident {
    /// Whether this incident wakes the tribute immediately.
    pub fn wakes_tribute(&self) -> bool {
        matches!(
            self,
            SleepIncident::Theft { .. }
                | SleepIncident::Relocation { .. }
                | SleepIncident::AnimalEncounter { .. }
                | SleepIncident::AllyAbandonment
                | SleepIncident::LimbInjury
        )
    }

    /// Roll whether a sleep incident occurs this phase.
    pub fn roll(rng: &mut impl Rng) -> Option<Self> {
        if !rng.random_bool(SLEEP_INCIDENT_CHANCE_PCT as f64 / 100.0) {
            return None;
        }
        Some(Self::random(rng))
    }

    /// Pick a random sleep incident with weighted probabilities.
    pub fn random(rng: &mut impl Rng) -> Self {
        // Weights: annoying is most common, severe incidents are rare.
        let roll: u32 = rng.random_range(0..100);
        match roll {
            0..=30 => SleepIncident::Annoying {
                flavor: AnnoyingFlavor::random(rng),
            },
            31..=45 => SleepIncident::Hallucination,
            46..=55 => {
                let animal = Self::random_animal_name(rng);
                SleepIncident::AnimalEncounter { animal }
            }
            56..=68 => SleepIncident::Theft {
                stolen_item: String::new(), // filled at apply time
            },
            69..=78 => SleepIncident::AllyAbandonment,
            79..=88 => {
                let new_area = Self::random_area(rng);
                SleepIncident::Relocation { new_area }
            }
            _ => SleepIncident::LimbInjury,
        }
    }

    fn random_animal_name(rng: &mut impl Rng) -> String {
        let animals = [
            "squirrel",
            "rabbit",
            "raccoon",
            "possum",
            "feral cat",
            "rat",
            "crow",
            "lizard",
        ];
        let idx = rng.random_range(0..animals.len());
        animals[idx].to_string()
    }

    fn random_area(rng: &mut impl Rng) -> Area {
        let areas = [
            Area::Cornucopia,
            Area::Sector1,
            Area::Sector2,
            Area::Sector3,
            Area::Sector4,
        ];
        let idx = rng.random_range(0..areas.len());
        areas[idx]
    }
}

/// Apply the mechanical effects of a sleep incident to a tribute.
/// Returns the narrative description of what happened.
pub fn apply_sleep_incident(
    tribute: &mut Tribute,
    incident: &SleepIncident,
    rng: &mut impl Rng,
) -> String {
    match incident {
        SleepIncident::Annoying { flavor } => {
            format!(
                "{} stirs as {} settles.",
                tribute.name,
                flavor.description()
            )
        }
        SleepIncident::Theft { .. } => {
            if tribute.items.is_empty() {
                // Nothing to steal — treat as annoying instead
                let flavor = AnnoyingFlavor::random(rng);
                return format!(
                    "{} stirs as {} settles (no items to steal).",
                    tribute.name,
                    flavor.description()
                );
            }
            let idx = rng.random_range(0..tribute.items.len());
            let stolen = tribute.items.remove(idx);
            format!(
                "{}'s {} is stolen while they sleep!",
                tribute.name, stolen.name
            )
        }
        SleepIncident::Relocation { new_area } => {
            let old_area = tribute.area;
            tribute.area = *new_area;
            format!(
                "{} sleepwalks from {} to {}!",
                tribute.name, old_area, new_area
            )
        }
        SleepIncident::AnimalEncounter { animal } => {
            // Minor sanity damage from fright
            let sanity_loss = rng.random_range(2..=8);
            tribute.attributes.sanity = tribute.attributes.sanity.saturating_sub(sanity_loss);
            format!(
                "A {} scurries over {}! They lose {} sanity from the fright.",
                animal, tribute.name, sanity_loss
            )
        }
        SleepIncident::Hallucination => {
            let sanity_loss = rng.random_range(3..=12);
            tribute.attributes.sanity = tribute.attributes.sanity.saturating_sub(sanity_loss);
            format!(
                "{} thrashes in their sleep, tormented by strange visions. Loses {} sanity.",
                tribute.name, sanity_loss
            )
        }
        SleepIncident::AllyAbandonment => {
            // Remove a random ally
            if tribute.allies.is_empty() {
                let flavor = AnnoyingFlavor::random(rng);
                return format!(
                    "{} stirs as {} settles (no allies to abandon).",
                    tribute.name,
                    flavor.description()
                );
            }
            let idx = rng.random_range(0..tribute.allies.len());
            let _abandoned = tribute.allies.remove(idx);
            format!(
                "An ally silently slips away into the night, abandoning {}.",
                tribute.name
            )
        }
        SleepIncident::LimbInjury => {
            // Comedic: leg fell asleep, arm is pins-and-needles
            let hp_loss = rng.random_range(2..=5);
            tribute.attributes.health = tribute.attributes.health.saturating_sub(hp_loss);
            let body_part = if rng.random_bool(0.5) { "leg" } else { "arm" };
            format!(
                "{} wakes to find their {} has completely fallen asleep. Takes {} HP from the panic.",
                tribute.name, body_part, hp_loss
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn roll_sleep_incident_sometimes_returns_none() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut found_some = false;
        for _ in 0..100 {
            if SleepIncident::roll(&mut rng).is_some() {
                found_some = true;
                break;
            }
        }
        assert!(found_some, "should roll at least one incident in 100 tries");
    }

    #[test]
    fn annoying_does_not_wake() {
        let incident = SleepIncident::Annoying {
            flavor: AnnoyingFlavor::SquirrelOnChest,
        };
        assert!(!incident.wakes_tribute());
    }

    #[test]
    fn theft_wakes_tribute() {
        let incident = SleepIncident::Theft {
            stolen_item: "sword".into(),
        };
        assert!(incident.wakes_tribute());
    }

    #[test]
    fn apply_theft_removes_item() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let item = crate::items::Item {
            identifier: "sword-1".into(),
            name: "sword".into(),
            ..Default::default()
        };
        tribute.items.push(item);
        let incident = SleepIncident::Theft {
            stolen_item: String::new(),
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("stolen"),
            "description should mention theft: {desc}"
        );
        assert_eq!(tribute.items.len(), 0, "item should be removed");
    }

    #[test]
    fn apply_relocation_changes_area() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_area = tribute.area;
        let incident = SleepIncident::Relocation {
            new_area: Area::Sector1,
        };
        let mut rng = SmallRng::seed_from_u64(42);
        apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert_ne!(tribute.area, orig_area, "area should change");
        assert_eq!(tribute.area, Area::Sector1);
    }

    #[test]
    fn apply_hallucination_reduces_sanity() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.attributes.sanity;
        let incident = SleepIncident::Hallucination;
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("sanity"),
            "description should mention sanity loss"
        );
        assert!(
            tribute.attributes.sanity < orig_sanity,
            "sanity should decrease"
        );
    }

    #[test]
    fn sleep_incident_round_trips_through_kind() {
        let incident = SleepIncident::AnimalEncounter {
            animal: "raccoon".into(),
        };
        let kind: SleepIncidentKind = (&incident).into();
        assert_eq!(
            kind,
            SleepIncidentKind::AnimalEncounter {
                animal: "raccoon".into()
            }
        );
    }

    #[test]
    fn apply_limb_injury_reduces_hp() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_hp = tribute.attributes.health;
        let incident = SleepIncident::LimbInjury;
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("HP"),
            "description should mention HP loss: {desc}"
        );
        assert!(tribute.attributes.health < orig_hp, "HP should decrease");
    }

    #[test]
    fn theft_with_no_items_falls_back_to_annoying() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        assert!(tribute.items.is_empty());
        let incident = SleepIncident::Theft {
            stolen_item: String::new(),
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        // Should not crash, should produce flavor text
        assert!(!desc.is_empty());
    }
}
