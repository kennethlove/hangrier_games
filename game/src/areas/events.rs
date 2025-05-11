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
        }
    }
}

impl AreaEvent {
    pub fn random() -> AreaEvent {
        let mut rng = rand::thread_rng();
        Self::iter().choose(&mut rng).unwrap().clone()
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
