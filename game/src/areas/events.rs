use crate::terrain::BaseTerrain;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumIter)]
pub enum AreaEvent {
    Wildfire,
    Flood,
    Earthquake,
    Avalanche,
    Blizzard,
    Landslide,
    Heatwave,
    Sandstorm,
    Drought,
    Rockslide,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Minor,
    Moderate,
    Major,
    Catastrophic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurvivalResult {
    pub survived: bool,
    pub instant_death: bool,
    pub stamina_restored: u32,
    pub sanity_restored: u32,
    pub reward_item: Option<String>,
}

impl FromStr for AreaEvent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wildfire" => Ok(AreaEvent::Wildfire),
            "flood" => Ok(AreaEvent::Flood),
            "earthquake" => Ok(AreaEvent::Earthquake),
            "avalanche" => Ok(AreaEvent::Avalanche),
            "blizzard" => Ok(AreaEvent::Blizzard),
            "landslide" => Ok(AreaEvent::Landslide),
            "heatwave" => Ok(AreaEvent::Heatwave),
            "sandstorm" => Ok(AreaEvent::Sandstorm),
            "drought" => Ok(AreaEvent::Drought),
            "rockslide" => Ok(AreaEvent::Rockslide),
            _ => Err("Invalid area event".to_string()),
        }
    }
}

impl Display for AreaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AreaEvent::Wildfire => write!(f, "wildfire"),
            AreaEvent::Flood => write!(f, "flood"),
            AreaEvent::Earthquake => write!(f, "earthquake"),
            AreaEvent::Avalanche => write!(f, "avalanche"),
            AreaEvent::Blizzard => write!(f, "blizzard"),
            AreaEvent::Landslide => write!(f, "landslide"),
            AreaEvent::Heatwave => write!(f, "heatwave"),
            AreaEvent::Sandstorm => write!(f, "sandstorm"),
            AreaEvent::Drought => write!(f, "drought"),
            AreaEvent::Rockslide => write!(f, "rockslide"),
        }
    }
}

impl AreaEvent {
    pub fn random() -> AreaEvent {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Self::iter().choose(&mut rng).unwrap().clone()
    }

    /// Calculate event severity based on terrain type
    pub fn severity_in_terrain(&self, terrain: &BaseTerrain) -> EventSeverity {
        use BaseTerrain::*;
        use EventSeverity::*;

        match (self, terrain) {
            // Wildfire
            (AreaEvent::Wildfire, Forest | Jungle) => Catastrophic,
            (AreaEvent::Wildfire, Grasslands | Badlands) => Major,
            (AreaEvent::Wildfire, Clearing | UrbanRuins | Highlands) => Moderate,
            (AreaEvent::Wildfire, Desert | Tundra | Wetlands | Mountains | Geothermal) => Minor,

            // Blizzard
            (AreaEvent::Blizzard, Mountains | Tundra) => Catastrophic,
            (AreaEvent::Blizzard, Highlands) => Major,
            (AreaEvent::Blizzard, Forest | Clearing | Badlands | Grasslands) => Moderate,
            (AreaEvent::Blizzard, Desert | Jungle | Wetlands | UrbanRuins | Geothermal) => Minor,

            // Sandstorm
            (AreaEvent::Sandstorm, Desert | Badlands) => Catastrophic,
            (AreaEvent::Sandstorm, Grasslands) => Major,
            (AreaEvent::Sandstorm, Clearing | Highlands) => Moderate,
            (
                AreaEvent::Sandstorm,
                Forest | Wetlands | Jungle | Mountains | Tundra | UrbanRuins | Geothermal,
            ) => Minor,

            // Flood
            (AreaEvent::Flood, Wetlands) => Catastrophic,
            (AreaEvent::Flood, Jungle | Forest) => Major,
            (AreaEvent::Flood, Grasslands | Clearing | Badlands) => Moderate,
            (
                AreaEvent::Flood,
                Mountains | Highlands | Desert | Tundra | UrbanRuins | Geothermal,
            ) => Minor,

            // Earthquake
            (AreaEvent::Earthquake, Mountains | UrbanRuins) => Catastrophic,
            (AreaEvent::Earthquake, Highlands | Geothermal) => Major,
            (AreaEvent::Earthquake, Forest | Jungle | Badlands) => Moderate,
            (AreaEvent::Earthquake, Grasslands | Clearing | Desert | Tundra | Wetlands) => Minor,

            // Avalanche
            (AreaEvent::Avalanche, Mountains) => Catastrophic,
            (AreaEvent::Avalanche, Highlands | Tundra) => Major,
            (AreaEvent::Avalanche, Forest | Jungle) => Moderate,
            (
                AreaEvent::Avalanche,
                Grasslands | Clearing | Desert | Wetlands | UrbanRuins | Badlands | Geothermal,
            ) => Minor,

            // Landslide
            (AreaEvent::Landslide, Mountains | Highlands | Jungle) => Major,
            (AreaEvent::Landslide, Forest | Badlands) => Moderate,
            (
                AreaEvent::Landslide,
                Grasslands | Clearing | Desert | Tundra | Wetlands | UrbanRuins | Geothermal,
            ) => Minor,

            // Heatwave
            (AreaEvent::Heatwave, Desert) => Catastrophic,
            (AreaEvent::Heatwave, Badlands | Geothermal) => Major,
            (AreaEvent::Heatwave, Grasslands | Forest | Clearing | Jungle | UrbanRuins) => Moderate,
            (AreaEvent::Heatwave, Tundra | Mountains | Highlands | Wetlands) => Minor,

            // Drought
            (AreaEvent::Drought, Desert) => Catastrophic,
            (AreaEvent::Drought, Grasslands | Badlands) => Major,
            (AreaEvent::Drought, Forest | Clearing | Highlands | UrbanRuins | Geothermal) => {
                Moderate
            }
            (AreaEvent::Drought, Wetlands | Jungle | Mountains | Tundra) => Minor,

            // Rockslide
            (AreaEvent::Rockslide, Mountains) => Catastrophic,
            (AreaEvent::Rockslide, Badlands | Highlands) => Major,
            (AreaEvent::Rockslide, UrbanRuins | Geothermal) => Moderate,
            (
                AreaEvent::Rockslide,
                Grasslands | Clearing | Desert | Tundra | Wetlands | Forest | Jungle,
            ) => Minor,
        }
    }

    /// Perform survival check with modifiers
    ///
    /// # Arguments
    /// * `terrain` - The terrain where the event occurs
    /// * `has_affinity` - Whether tribute has terrain affinity (+3 bonus)
    /// * `has_item_bonus` - Whether tribute has relevant protective item (+2 bonus)
    /// * `is_desperate` - Whether tribute is in desperate state (health < 30%, +5 bonus)
    /// * `current_health` - Tribute's current health for desperation rewards
    ///
    /// # Returns
    /// SurvivalResult containing survival status and any rewards
    pub fn survival_check(
        &self,
        terrain: &BaseTerrain,
        has_affinity: bool,
        has_item_bonus: bool,
        is_desperate: bool,
        current_health: u32,
    ) -> SurvivalResult {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let severity = self.severity_in_terrain(terrain);

        // Base survival DC by severity
        let base_dc = match severity {
            EventSeverity::Minor => 5,
            EventSeverity::Moderate => 10,
            EventSeverity::Major => 15,
            EventSeverity::Catastrophic => 20,
        };

        // Calculate modifiers
        let mut modifier = 0;
        if has_affinity {
            modifier += 3;
        }
        if has_item_bonus {
            modifier += 2;
        }
        if is_desperate {
            modifier += 5; // Desperation bonus
        }

        // Roll d20
        let roll = rng.random_range(1..=20);
        let total = roll + modifier;

        // Check for catastrophic instant death (5% chance)
        let instant_death = if severity == EventSeverity::Catastrophic {
            rng.random_range(0..100) < 5 // 5% instant death
        } else {
            false
        };

        if instant_death {
            return SurvivalResult {
                survived: false,
                instant_death: true,
                stamina_restored: 0,
                sanity_restored: 0,
                reward_item: None,
            };
        }

        // Check survival
        let survived = total >= base_dc;

        if !survived {
            return SurvivalResult {
                survived: false,
                instant_death: false,
                stamina_restored: 0,
                sanity_restored: 0,
                reward_item: None,
            };
        }

        // Desperation success rewards (if desperate and survived)
        let (stamina_restored, sanity_restored, reward_item) = if is_desperate {
            let reward_roll = rng.random_range(0..100);
            if reward_roll < 42 {
                // 42% chance: stamina restore (42.5% stamina)
                let stamina = ((100 - current_health) as f32 * 0.425) as u32;
                (stamina, 0, None)
            } else if reward_roll < 85 {
                // 43% chance (42.5%): sanity boost
                let sanity = ((100 - current_health) as f32 * 0.425) as u32;
                (0, sanity, None)
            } else if reward_roll < 95 {
                // 10% chance: item reward
                (0, 0, Some("scavenged_item".to_string()))
            } else {
                // 5% chance: nothing
                (0, 0, None)
            }
        } else {
            (0, 0, None)
        };

        SurvivalResult {
            survived: true,
            instant_death: false,
            stamina_restored,
            sanity_restored,
            reward_item,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn random_area_event() {
        let random_event = AreaEvent::random();
        assert!(AreaEvent::iter().position(|a| a == random_event).is_some());
    }

    #[rstest]
    #[case(AreaEvent::Wildfire, "wildfire")]
    #[case(AreaEvent::Flood, "flood")]
    #[case(AreaEvent::Earthquake, "earthquake")]
    #[case(AreaEvent::Avalanche, "avalanche")]
    #[case(AreaEvent::Blizzard, "blizzard")]
    #[case(AreaEvent::Landslide, "landslide")]
    #[case(AreaEvent::Heatwave, "heatwave")]
    #[case(AreaEvent::Sandstorm, "sandstorm")]
    #[case(AreaEvent::Drought, "drought")]
    #[case(AreaEvent::Rockslide, "rockslide")]
    fn area_event_to_string(#[case] event: AreaEvent, #[case] expected: &str) {
        assert_eq!(event.to_string(), expected.to_string());
    }

    #[rstest]
    #[case("wildfire", AreaEvent::Wildfire)]
    #[case("flood", AreaEvent::Flood)]
    #[case("earthquake", AreaEvent::Earthquake)]
    #[case("avalanche", AreaEvent::Avalanche)]
    #[case("blizzard", AreaEvent::Blizzard)]
    #[case("landslide", AreaEvent::Landslide)]
    #[case("heatwave", AreaEvent::Heatwave)]
    #[case("sandstorm", AreaEvent::Sandstorm)]
    #[case("drought", AreaEvent::Drought)]
    #[case("rockslide", AreaEvent::Rockslide)]
    fn area_event_from_str(#[case] input: &str, #[case] event: AreaEvent) {
        let area_event = AreaEvent::from_str(input).unwrap();
        assert_eq!(area_event, event);
    }

    #[test]
    fn area_event_from_str_invalid() {
        let area_event = AreaEvent::from_str("alien invasion");
        assert!(area_event.is_err());
    }
}
