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
    pub fn plural(&self) -> String {
        match self {
            Animal::Wolf => "wolves".to_string(),
            _ => {
                format!("{}s", self)
            }
        }
    }

    pub fn random() -> Animal {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Animal::iter().choose(&mut rng).unwrap()
    }

    pub fn damage(&self) -> u32 {
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
    use rstest::rstest;

    #[rstest]
    #[case(Animal::Squirrel, "squirrel")]
    #[case(Animal::Bear, "bear")]
    #[case(Animal::Wolf, "wolf")]
    #[case(Animal::Cougar, "cougar")]
    #[case(Animal::Boar, "boar")]
    #[case(Animal::Snake, "snake")]
    #[case(Animal::Monkey, "monkey")]
    #[case(Animal::Baboon, "baboon")]
    #[case(Animal::Hyena, "hyena")]
    #[case(Animal::Lion, "lion")]
    #[case(Animal::Tiger, "tiger")]
    #[case(Animal::Elephant, "elephant")]
    #[case(Animal::Rhino, "rhino")]
    #[case(Animal::Hippo, "hippo")]
    #[case(Animal::TrackerJacker, "tracker jacker")]
    fn animal_to_string(#[case] animal: Animal, #[case] expected: &str) {
        assert_eq!(animal.to_string(), expected);
    }

    #[rstest]
    #[case("squirrel", Animal::Squirrel)]
    #[case("bear", Animal::Bear)]
    #[case("wolf", Animal::Wolf)]
    #[case("cougar", Animal::Cougar)]
    #[case("boar", Animal::Boar)]
    #[case("snake", Animal::Snake)]
    #[case("monkey", Animal::Monkey)]
    #[case("baboon", Animal::Baboon)]
    #[case("hyena", Animal::Hyena)]
    #[case("lion", Animal::Lion)]
    #[case("tiger", Animal::Tiger)]
    #[case("elephant", Animal::Elephant)]
    #[case("rhino", Animal::Rhino)]
    #[case("hippo", Animal::Hippo)]
    #[case("tracker jacker", Animal::TrackerJacker)]
    fn animal_from_str(#[case] input: &str, #[case] animal: Animal) {
        assert_eq!(animal, Animal::from_str(input).unwrap());
    }

    #[rstest]
    #[case(Animal::Squirrel, 1)]
    #[case(Animal::Bear, 10)]
    #[case(Animal::Wolf, 5)]
    #[case(Animal::Cougar, 5)]
    #[case(Animal::Boar, 3)]
    #[case(Animal::Snake, 2)]
    #[case(Animal::Monkey, 3)]
    #[case(Animal::Baboon, 5)]
    #[case(Animal::Hyena, 5)]
    #[case(Animal::Lion, 10)]
    #[case(Animal::Tiger, 10)]
    #[case(Animal::Elephant, 10)]
    #[case(Animal::Rhino, 10)]
    #[case(Animal::Hippo, 20)]
    #[case(Animal::TrackerJacker, 5)]
    fn animal_damage(#[case] animal: Animal, #[case] damage: u32) {
        assert_eq!(animal.damage(), damage);
    }

    #[test]
    fn random_animal() {
        let animal = Animal::random();
        assert_eq!(animal, Animal::from_str(&animal.to_string()).unwrap());
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
