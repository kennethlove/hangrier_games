use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug)]
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wildfire" => Ok(AreaEvent::Wildfire),
            "flood" => Ok(AreaEvent::Flood),
            "earthquake" => Ok(AreaEvent::Earthquake),
            "avalanche" => Ok(AreaEvent::Avalanche),
            "blizzard" => Ok(AreaEvent::Blizzard),
            "landslide" => Ok(AreaEvent::Landslide),
            "heatwave" => Ok(AreaEvent::Heatwave),
            _ => Err(()),
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
    pub fn as_str(&self) -> &str {
        match self {
            AreaEvent::Wildfire => "wildfire",
            AreaEvent::Flood => "flood",
            AreaEvent::Earthquake => "earthquake",
            AreaEvent::Avalanche => "avalanche",
            AreaEvent::Blizzard => "blizzard",
            AreaEvent::Landslide => "landslide",
            AreaEvent::Heatwave => "heatwave",
        }
    }

    pub fn random() -> AreaEvent {
        let mut rng = rand::thread_rng();
        let events = vec![
            AreaEvent::Wildfire,
            AreaEvent::Flood,
            AreaEvent::Earthquake,
            AreaEvent::Avalanche,
            AreaEvent::Blizzard,
            AreaEvent::Landslide,
            AreaEvent::Heatwave,
        ];
        let index = rng.gen_range(0..events.len());
        events[index].clone()
    }
}
