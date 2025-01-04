use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

#[derive(
    Clone, Debug, Default, Deserialize, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum Animal {
    #[default]
    Squirrel,
    Bear,
    Wolf,
    Cougar,
    Boar,
    Snake,
    Monkey,
    Baboon,
    Hyena,
    Lion,
    Tiger,
    Elephant,
    Rhino,
    Hippo,
    TrackerJacker,
}

impl Animal {
    pub fn as_str(&self) -> &str {
        match self {
            Animal::Squirrel => "squirrel",
            Animal::Bear => "bear",
            Animal::Wolf => "wolf",
            Animal::Cougar => "cougar",
            Animal::Boar => "boar",
            Animal::Snake => "snake",
            Animal::Monkey => "monkey",
            Animal::Baboon => "baboon",
            Animal::Hyena => "hyena",
            Animal::Lion => "lion",
            Animal::Tiger => "tiger",
            Animal::Elephant => "elephant",
            Animal::Rhino => "rhino",
            Animal::Hippo => "hippo",
            Animal::TrackerJacker => "tracker jacker",
        }
    }

    pub fn plural(&self) -> String {
        match self {
            Animal::Wolf => "wolves".to_string(),
            _ => {
                let pluralized = format!("{}s", self.as_str());
                pluralized
            }
        }
    }

    pub fn random() -> Animal {
        let mut rng = rand::thread_rng();
        let animal = Animal::iter().choose(&mut rng).unwrap();
        animal
    }

    pub fn damage(&self) -> i32 {
        match self {
            Animal::Squirrel => 1,
            Animal::Bear => 10,
            Animal::Wolf => 5,
            Animal::Cougar => 5,
            Animal::Boar => 3,
            Animal::Snake => 2,
            Animal::Monkey => 3,
            Animal::Baboon => 5,
            Animal::Hyena => 5,
            Animal::Lion => 10,
            Animal::Tiger => 10,
            Animal::Elephant => 10,
            Animal::Rhino => 10,
            Animal::Hippo => 20,
            Animal::TrackerJacker => 5,
        }
    }
}

impl FromStr for Animal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "squirrel" => Ok(Animal::Squirrel),
            "bear" => Ok(Animal::Bear),
            "wolf" => Ok(Animal::Wolf),
            "cougar" => Ok(Animal::Cougar),
            "boar" => Ok(Animal::Boar),
            "snake" => Ok(Animal::Snake),
            "monkey" => Ok(Animal::Monkey),
            "baboon" => Ok(Animal::Baboon),
            "hyena" => Ok(Animal::Hyena),
            "lion" => Ok(Animal::Lion),
            "tiger" => Ok(Animal::Tiger),
            "elephant" => Ok(Animal::Elephant),
            "rhino" => Ok(Animal::Rhino),
            "hippo" => Ok(Animal::Hippo),
            "tracker jacker" => Ok(Animal::TrackerJacker),
            _ => Err(()),
        }
    }
}

impl Display for Animal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Animal::Squirrel => write!(f, "squirrel"),
            Animal::Bear => write!(f, "bear"),
            Animal::Wolf => write!(f, "wolf"),
            Animal::Cougar => write!(f, "cougar"),
            Animal::Boar => write!(f, "boar"),
            Animal::Snake => write!(f, "snake"),
            Animal::Monkey => write!(f, "monkey"),
            Animal::Baboon => write!(f, "baboon"),
            Animal::Hyena => write!(f, "hyena"),
            Animal::Lion => write!(f, "lion"),
            Animal::Tiger => write!(f, "tiger"),
            Animal::Elephant => write!(f, "elephant"),
            Animal::Rhino => write!(f, "rhino"),
            Animal::Hippo => write!(f, "hippo"),
            Animal::TrackerJacker => write!(f, "tracker jacker"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animal_to_string() {
        let animal = Animal::Squirrel;
        assert_eq!(animal.to_string(), "squirrel".to_string());
    }

    #[test]
    fn animal_from_str() {
        let animal = Animal::Squirrel;
        assert_eq!(animal, Animal::from_str("squirrel").unwrap());
    }

    #[test]
    fn animal_damage() {
        let animal = Animal::Squirrel;
        assert_eq!(animal.damage(), 1);
    }

    #[test]
    fn random_animal() {
        let animal = Animal::random();
        assert_eq!(animal, Animal::from_str(animal.as_str()).unwrap());
    }

    #[test]
    fn animal_plurals() {
        let tracker_jacker = Animal::TrackerJacker;
        let wolf = Animal::Wolf;
        let cougar = Animal::Cougar;
        assert_eq!(tracker_jacker.plural(), "tracker jackers");
        assert_eq!(wolf.plural(), "wolves");
        assert_eq!(cougar.plural(), "cougars");
    }
}
