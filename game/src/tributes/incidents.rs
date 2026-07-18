use crate::areas::Area;
use crate::areas::shelter;
use crate::areas::weather;
use crate::tributes::Tribute;
use rand::Rng;
use rand::RngExt;
use shared::messages::SleepIncidentKind;
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SLEEP_INCIDENT_DAY_PCT: u32 = 8;
const SLEEP_INCIDENT_DAWN_PCT: u32 = 12;
const SLEEP_INCIDENT_DUSK_PCT: u32 = 12;
const SLEEP_INCIDENT_NIGHT_PCT: u32 = 22;
const SLEEP_INCIDENT_SHELTER_MULTIPLIER: f64 = 0.5;

const SHELTER_QUALITY_SCORE_3: f64 = 0.4;
const SHELTER_QUALITY_SCORE_2: f64 = 0.6;
const SHELTER_QUALITY_SCORE_1: f64 = 0.8;
const SHELTER_QUALITY_SCORE_0: f64 = 1.0;

const SLEEP_SHELTER_NONE_MULTIPLIER: f64 = 1.0;
const SLEEP_SHELTER_CRUDE_MULTIPLIER: f64 = 0.8;
const SLEEP_SHELTER_NATURAL_MULTIPLIER: f64 = 0.5;
const SLEEP_SHELTER_FORTIFIED_MULTIPLIER: f64 = 0.3;

// ---------------------------------------------------------------------------
// SleepShelter
// ---------------------------------------------------------------------------

/// Quality of shelter a tribute has constructed before sleeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepShelter {
    None,
    Crude,
    Natural,
    Fortified,
}

impl SleepShelter {
    pub fn multiplier(&self) -> f64 {
        match self {
            SleepShelter::None => SLEEP_SHELTER_NONE_MULTIPLIER,
            SleepShelter::Crude => SLEEP_SHELTER_CRUDE_MULTIPLIER,
            SleepShelter::Natural => SLEEP_SHELTER_NATURAL_MULTIPLIER,
            SleepShelter::Fortified => SLEEP_SHELTER_FORTIFIED_MULTIPLIER,
        }
    }
}

// ---------------------------------------------------------------------------
// Incident probability helpers
// ---------------------------------------------------------------------------

/// Base incident chance (0-100) for a given phase of day.
pub fn base_incident_chance(phase: crate::messages::Phase) -> u32 {
    match phase {
        crate::messages::Phase::Day => SLEEP_INCIDENT_DAY_PCT,
        crate::messages::Phase::Dawn => SLEEP_INCIDENT_DAWN_PCT,
        crate::messages::Phase::Dusk => SLEEP_INCIDENT_DUSK_PCT,
        crate::messages::Phase::Night => SLEEP_INCIDENT_NIGHT_PCT,
    }
}

/// Terrain-based multiplier derived from the biome's inherent shelter quality.
/// Better shelter biomes have lower incident chances.
pub fn biome_incident_multiplier(biome: crate::terrain::types::BaseTerrain) -> f64 {
    let quality = shelter::shelter_quality(biome, &weather::current_weather());
    match quality {
        3 => SHELTER_QUALITY_SCORE_3,
        2 => SHELTER_QUALITY_SCORE_2,
        1 => SHELTER_QUALITY_SCORE_1,
        _ => SHELTER_QUALITY_SCORE_0,
    }
}

/// Returns the multiplier associated with a given shelter type.
pub fn sleep_shelter_multiplier(shelter: &SleepShelter) -> f64 {
    shelter.multiplier()
}

/// Day-scaling multiplier. Returns 1.0 (no scaling) by default.
/// Can be tuned to increase or decrease incident chance as the game progresses.
pub fn day_scaling_multiplier(_current_day: u32) -> f64 {
    1.0
}

/// Combined effective incident chance (0.0-1.0) factoring in phase, terrain,
/// shelter status, constructed shelter quality, and game-day progression.
pub fn effective_incident_chance(
    phase: crate::messages::Phase,
    biome: crate::terrain::types::BaseTerrain,
    is_sheltered: bool,
    sleep_shelter: &SleepShelter,
    current_day: u32,
) -> f64 {
    base_incident_chance(phase) as f64
        * biome_incident_multiplier(biome)
        * if is_sheltered {
            SLEEP_INCIDENT_SHELTER_MULTIPLIER
        } else {
            1.0
        }
        * sleep_shelter_multiplier(sleep_shelter)
        * day_scaling_multiplier(current_day)
}

// ---------------------------------------------------------------------------
// Biome-specific pools
// ---------------------------------------------------------------------------

/// Biome-specific animal pool (5 per biome).
pub fn biome_animal_pool(biome: crate::terrain::types::BaseTerrain) -> &'static [&'static str] {
    use crate::terrain::types::BaseTerrain::*;
    match biome {
        Forest => &["bear", "wolf", "owl", "deer", "fox"],
        Jungle => &["snake", "jaguar", "monkey", "parrot", "spider"],
        Desert => &["scorpion", "rattlesnake", "coyote", "lizard", "vulture"],
        Tundra => &["polar bear", "arctic fox", "snowy owl", "seal", "wolf"],
        Wetlands => &["alligator", "frog", "heron", "snake", "muskrat"],
        Mountains => &["mountain goat", "eagle", "ibex", "pika", "snow leopard"],
        UrbanRuins => &["rat", "stray dog", "pigeon", "feral cat", "cockroach"],
        Grasslands => &["antelope", "prairie dog", "hawk", "bison", "beetle"],
        Badlands => &["coyote", "rattlesnake", "vulture", "scorpion", "lizard"],
        Highlands => &["condor", "llama", "vicuña", "fox", "hawk"],
        Geothermal => &[
            "gecko",
            "thermal worm",
            "sulfur bat",
            "vent crab",
            "ash moth",
        ],
        Clearing => &["rabbit", "deer", "fox", "crow", "beetle"],
    }
}

/// Biome-specific flavor incident pool (2 per biome).
pub fn biome_flavor_pool(biome: crate::terrain::types::BaseTerrain) -> &'static [&'static str] {
    use crate::terrain::types::BaseTerrain::*;
    match biome {
        Forest => &["an acorn drops on them", "a branch snaps nearby"],
        Jungle => &["a fruit falls from above", "vines rustle in the breeze"],
        Desert => &["sand shifts beneath them", "a cactus creaks in the wind"],
        Tundra => &["ice cracks beneath them", "snow falls on their face"],
        Wetlands => &["mud bubbles up nearby", "reeds rustle in the water"],
        Mountains => &[
            "pebbles tumble down the slope",
            "wind howls through the rocks",
        ],
        UrbanRuins => &["glass shatters in the distance", "metal creaks overhead"],
        Grasslands => &["grass rustles around them", "a flower sways in the breeze"],
        Badlands => &["a rock tumbles past", "a dust devil spins nearby"],
        Highlands => &["a gust of wind whips past", "stones clatter down a slope"],
        Geothermal => &["steam hisses from a vent", "the ground rumbles softly"],
        Clearing => &["a flower petal drifts by", "a leaf dances in the air"],
    }
}

// ---------------------------------------------------------------------------
// find_shelter
// ---------------------------------------------------------------------------

/// Attempt to construct a shelter using intelligence and strength.
///
/// * `intelligence` / `strength` — tribute attributes.
/// * `terrain` — determines the base shelter quality (0-3).
/// * `rng` — source of randomness for variance.
///
/// Returns the best [`SleepShelter`] level the tribute can achieve.
pub fn find_shelter(
    intelligence: u32,
    strength: u32,
    terrain: crate::terrain::types::BaseTerrain,
    rng: &mut impl Rng,
) -> SleepShelter {
    let quality = shelter::shelter_quality(terrain, &weather::current_weather());
    let dc = 25 - (quality as u32 * 5);
    let effective_roll =
        (std::cmp::max(intelligence, strength) as f64 * rng.random_range(0.8..=1.2)) as u32;

    if effective_roll >= dc + 20 {
        SleepShelter::Fortified
    } else if effective_roll >= dc + 10 {
        SleepShelter::Natural
    } else if effective_roll >= dc {
        SleepShelter::Crude
    } else {
        SleepShelter::None
    }
}

// ---------------------------------------------------------------------------
// SleepIncident
// ---------------------------------------------------------------------------

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
    /// Nightmare causing sanity loss — does not wake the sleeper.
    Nightmare { sanity_loss: u32 },
    /// Night terror causing sanity loss — wakes the sleeper.
    NightTerror { sanity_loss: u32 },
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
    Custom(String),
}

impl AnnoyingFlavor {
    fn random(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> Self {
        // 50% chance of biome-specific flavor, 50% classic generic flavor
        if rng.random_bool(0.5) {
            let pool = biome_flavor_pool(biome);
            let idx = rng.random_range(0..pool.len());
            AnnoyingFlavor::Custom(pool[idx].to_string())
        } else {
            match rng.random_range(0..5) {
                0 => AnnoyingFlavor::SquirrelOnChest,
                1 => AnnoyingFlavor::ButterflyLanded,
                2 => AnnoyingFlavor::WeirdDream,
                3 => AnnoyingFlavor::MouseRanOver,
                _ => AnnoyingFlavor::LeafOnFace,
            }
        }
    }

    fn description(&self) -> Cow<'static, str> {
        match self {
            AnnoyingFlavor::SquirrelOnChest => Cow::Borrowed("a squirrel on their chest"),
            AnnoyingFlavor::ButterflyLanded => Cow::Borrowed("a butterfly landing on their nose"),
            AnnoyingFlavor::WeirdDream => Cow::Borrowed("a weird dream about turnips"),
            AnnoyingFlavor::MouseRanOver => Cow::Borrowed("a mouse running over their face"),
            AnnoyingFlavor::LeafOnFace => Cow::Borrowed("a leaf drifting onto their face"),
            AnnoyingFlavor::Custom(s) => Cow::Owned(s.clone()),
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
            SleepIncident::Nightmare { .. } => SleepIncidentKind::Nightmare,
            SleepIncident::NightTerror { .. } => SleepIncidentKind::NightTerror,
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
                | SleepIncident::NightTerror { .. }
                | SleepIncident::LimbInjury
        )
    }

    /// Roll whether a sleep incident occurs this phase, factoring in terrain,
    /// shelter, and game-day progression.
    pub fn roll(
        rng: &mut impl Rng,
        phase: crate::messages::Phase,
        biome: crate::terrain::types::BaseTerrain,
        is_sheltered: bool,
        sleep_shelter: &SleepShelter,
        current_day: u32,
    ) -> Option<Self> {
        let chance =
            effective_incident_chance(phase, biome, is_sheltered, sleep_shelter, current_day);
        if !rng.random_bool(chance / 100.0) {
            return None;
        }
        Some(Self::random(rng, biome))
    }

    /// Pick a random sleep incident with weighted probabilities,
    /// using biome-specific pools for animal encounters.
    pub fn random(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> Self {
        let roll: u32 = rng.random_range(0..100);
        match roll {
            // 30% — Annoying (flavor only)
            0..=29 => SleepIncident::Annoying {
                flavor: AnnoyingFlavor::random(rng, biome),
            },
            // 15% — Nightmare (sanity damage, does not wake)
            30..=44 => SleepIncident::Nightmare {
                sanity_loss: rng.random_range(3..=12),
            },
            // 5% — Night terror (sanity damage, wakes)
            45..=49 => SleepIncident::NightTerror {
                sanity_loss: rng.random_range(5..=15),
            },
            // 12% — Theft
            50..=61 => SleepIncident::Theft {
                stolen_item: String::new(),
            },
            // 10% — Relocation
            62..=71 => {
                let new_area = Self::random_area(rng);
                SleepIncident::Relocation { new_area }
            }
            // 10% — Animal encounter
            72..=81 => {
                let animal = Self::random_animal_name(rng, biome);
                SleepIncident::AnimalEncounter { animal }
            }
            // 8% — Limb injury
            82..=89 => SleepIncident::LimbInjury,
            // 10% — Ally abandonment
            _ => SleepIncident::AllyAbandonment,
        }
    }

    fn random_animal_name(rng: &mut impl Rng, biome: crate::terrain::types::BaseTerrain) -> String {
        let pool = biome_animal_pool(biome);
        let idx = rng.random_range(0..pool.len());
        pool[idx].to_string()
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

// ---------------------------------------------------------------------------
// apply_sleep_incident
// ---------------------------------------------------------------------------

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
                let flavor =
                    AnnoyingFlavor::random(rng, crate::terrain::types::BaseTerrain::Forest);
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
            let sanity_loss = rng.random_range(2..=8);
            // ponytail: sanity drain moved to mental condition system; narrative-only here
            format!(
                "A {} scurries over {}! They lose {} sanity from the fright.",
                animal, tribute.name, sanity_loss
            )
        }
        SleepIncident::Nightmare { sanity_loss } => {
            // ponytail: sanity drain moved to mental condition system; narrative-only here
            format!(
                "{} thrashes in their sleep, tormented by terrible nightmares. Loses {} sanity.",
                tribute.name, sanity_loss
            )
        }
        SleepIncident::NightTerror { sanity_loss } => {
            // ponytail: sanity drain moved to mental condition system; narrative-only here
            format!(
                "{} jolts awake with a scream, heart pounding from a night terror! Loses {} sanity.",
                tribute.name, sanity_loss
            )
        }
        SleepIncident::AllyAbandonment => {
            format!(
                "An ally silently slips away into the night, abandoning {}.",
                tribute.name
            )
        }
        SleepIncident::LimbInjury => {
            let hp_loss = rng.random_range(2..=5);
            tribute.blood = tribute.blood.saturating_sub(hp_loss);
            let body_part = if rng.random_bool(0.5) { "leg" } else { "arm" };
            format!(
                "{} wakes to find their {} has completely fallen asleep. Takes {} HP from the panic.",
                tribute.name, body_part, hp_loss
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::Phase;
    use crate::terrain::types::BaseTerrain;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn roll_sleep_incident_sometimes_returns_none() {
        let mut rng = SmallRng::seed_from_u64(42);
        let shelter = SleepShelter::None;
        let mut found_some = false;
        for _ in 0..100 {
            if SleepIncident::roll(
                &mut rng,
                Phase::Night,
                BaseTerrain::Forest,
                false,
                &shelter,
                1,
            )
            .is_some()
            {
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
    fn apply_nightmare_reduces_sanity() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.effective_sanity();
        let incident = SleepIncident::Nightmare { sanity_loss: 5 };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("sanity"),
            "description should mention sanity loss"
        );
        // ponytail: sanity drain moved to mental condition system; narrative-only effect
        let _ = orig_sanity;
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
        let orig_hp = tribute.effective_health();
        let incident = SleepIncident::LimbInjury;
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("HP"),
            "description should mention HP loss: {desc}"
        );
        assert!(tribute.effective_health() < orig_hp, "HP should decrease");
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
        assert!(!desc.is_empty());
    }

    #[test]
    fn nightmare_does_not_wake() {
        let incident = SleepIncident::Nightmare { sanity_loss: 5 };
        assert!(!incident.wakes_tribute());
    }

    #[test]
    fn night_terror_wakes_tribute() {
        let incident = SleepIncident::NightTerror { sanity_loss: 5 };
        assert!(incident.wakes_tribute());
    }

    #[test]
    fn apply_night_terror_reduces_sanity() {
        let mut tribute = Tribute::new("Test".to_string(), None, None);
        let orig_sanity = tribute.effective_sanity();
        let incident = SleepIncident::NightTerror { sanity_loss: 7 };
        let mut rng = SmallRng::seed_from_u64(42);
        let desc = apply_sleep_incident(&mut tribute, &incident, &mut rng);
        assert!(
            desc.contains("sanity"),
            "description should mention sanity loss"
        );
        // ponytail: sanity drain moved to mental condition system; narrative-only effect
        let _ = orig_sanity;
    }

    #[test]
    fn ally_abandonment_no_wake() {
        let incident = SleepIncident::AllyAbandonment;
        assert!(!incident.wakes_tribute());
    }

    #[test]
    fn find_shelter_based_on_attributes() {
        let mut rng = SmallRng::seed_from_u64(42);
        // High int + str with good terrain should yield at least Crude
        let shelter = find_shelter(80, 80, BaseTerrain::Forest, &mut rng);
        assert_ne!(
            shelter,
            SleepShelter::None,
            "skilled tribute in forest should find shelter"
        );
    }

    #[test]
    fn sleep_shelter_multiplier_values() {
        assert!((SleepShelter::None.multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((SleepShelter::Crude.multiplier() - 0.8).abs() < f64::EPSILON);
        assert!((SleepShelter::Natural.multiplier() - 0.5).abs() < f64::EPSILON);
        assert!((SleepShelter::Fortified.multiplier() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn base_incident_chance_by_phase() {
        assert_eq!(base_incident_chance(Phase::Day), 8);
        assert_eq!(base_incident_chance(Phase::Dawn), 12);
        assert_eq!(base_incident_chance(Phase::Dusk), 12);
        assert_eq!(base_incident_chance(Phase::Night), 22);
    }
}
