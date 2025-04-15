use crate::threats::animals::Animal;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::EnumIter;

#[derive(Clone, Debug, Default, Deserialize, EnumIter, Eq, PartialEq, Serialize)]
pub enum TributeStatus {
    #[default]
    Healthy,
    Wounded,
    Starving,
    Dehydrated,
    Sick,
    Poisoned,
    RecentlyDead,
    Dead,
    Electrocuted,
    Frozen,
    Overheated,
    Broken,
    Infected,
    Drowned,
    Burned,
    Buried,
    Mauled(Animal),
}

impl FromStr for TributeStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase().as_str().contains("mauled") {
            if let Some(animal_name) = s.split_once(':').map(|x| x.1) {
                if let Ok(animal) = Animal::from_str(animal_name.trim()) {
                    return Ok(Self::Mauled(animal));
                }
            } else {
                return Err(());
            }
        }
        match s.to_lowercase().as_str() {
            "healthy" => Ok(TributeStatus::Healthy),
            "wounded" => Ok(TributeStatus::Wounded),
            "injured" => Ok(TributeStatus::Wounded),
            "starving" => Ok(TributeStatus::Starving),
            "dehydrated" => Ok(TributeStatus::Dehydrated),
            "sick" => Ok(TributeStatus::Sick),
            "poisoned" => Ok(TributeStatus::Poisoned),
            "recently dead" => Ok(TributeStatus::RecentlyDead),
            "dead" => Ok(TributeStatus::Dead),
            "electrocuted" => Ok(TributeStatus::Electrocuted),
            "frozen" => Ok(TributeStatus::Frozen),
            "overheated" => Ok(TributeStatus::Overheated),
            "broken" => Ok(TributeStatus::Broken),
            "infected" => Ok(TributeStatus::Infected),
            "drowned" => Ok(TributeStatus::Drowned),
            "burned" => Ok(TributeStatus::Burned),
            "buried" => Ok(TributeStatus::Buried),
            _ => Err(()),
        }
    }
}

impl Display for TributeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TributeStatus::Healthy => write!(f, "healthy"),
            TributeStatus::Wounded => write!(f, "wounded"),
            TributeStatus::Starving => write!(f, "starving"),
            TributeStatus::Dehydrated => write!(f, "dehydrated"),
            TributeStatus::Sick => write!(f, "sick"),
            TributeStatus::Poisoned => write!(f, "poisoned"),
            TributeStatus::RecentlyDead => write!(f, "recently dead"),
            TributeStatus::Dead => write!(f, "dead"),
            TributeStatus::Electrocuted => write!(f, "electrocuted"),
            TributeStatus::Frozen => write!(f, "frozen"),
            TributeStatus::Overheated => write!(f, "overheated"),
            TributeStatus::Broken => write!(f, "broken"),
            TributeStatus::Infected => write!(f, "infected"),
            TributeStatus::Drowned => write!(f, "drowned"),
            TributeStatus::Burned => write!(f, "burned"),
            TributeStatus::Buried => write!(f, "buried"),
            TributeStatus::Mauled(animal) => write!(f, "mauled: {}", animal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(TributeStatus::Healthy, "healthy")]
    #[case(TributeStatus::Wounded, "wounded")]
    #[case(TributeStatus::Starving, "starving")]
    #[case(TributeStatus::Dehydrated, "dehydrated")]
    #[case(TributeStatus::Sick, "sick")]
    #[case(TributeStatus::Poisoned, "poisoned")]
    #[case(TributeStatus::RecentlyDead, "recently dead")]
    #[case(TributeStatus::Dead, "dead")]
    #[case(TributeStatus::Electrocuted, "electrocuted")]
    #[case(TributeStatus::Frozen, "frozen")]
    #[case(TributeStatus::Overheated, "overheated")]
    #[case(TributeStatus::Broken, "broken")]
    #[case(TributeStatus::Infected, "infected")]
    #[case(TributeStatus::Drowned, "drowned")]
    #[case(TributeStatus::Burned, "burned")]
    #[case(TributeStatus::Buried, "buried")]
    #[case(TributeStatus::Mauled(Animal::Bear), "mauled: bear")]
    fn tribute_status_to_string(#[case] status: TributeStatus, #[case] expected: &str) {
        assert_eq!(status.to_string(), expected.to_string());
    }

    #[rstest]
    #[case("healthy", TributeStatus::Healthy)]
    #[case("wounded", TributeStatus::Wounded)]
    #[case("starving", TributeStatus::Starving)]
    #[case("dehydrated", TributeStatus::Dehydrated)]
    #[case("sick", TributeStatus::Sick)]
    #[case("poisoned", TributeStatus::Poisoned)]
    #[case("recently dead", TributeStatus::RecentlyDead)]
    #[case("dead", TributeStatus::Dead)]
    #[case("electrocuted", TributeStatus::Electrocuted)]
    #[case("frozen", TributeStatus::Frozen)]
    #[case("overheated", TributeStatus::Overheated)]
    #[case("broken", TributeStatus::Broken)]
    #[case("infected", TributeStatus::Infected)]
    #[case("drowned", TributeStatus::Drowned)]
    #[case("burned", TributeStatus::Burned)]
    #[case("buried", TributeStatus::Buried)]
    #[case("mauled: bear", TributeStatus::Mauled(Animal::Bear))]
    fn tribute_status_from_str(#[case] input: &str, #[case] expected: TributeStatus) {
        assert_eq!(TributeStatus::from_str(input).unwrap(), expected);
    }

    #[test]
    fn tribute_status_from_str_invalid() {
        assert!(TributeStatus::from_str("burping").is_err());
    }

    #[test]
    fn tribute_status_from_str_invalid_animal() {
        assert!(TributeStatus::from_str("mauled: velociraptor").is_err());
    }

    #[test]
    fn tribute_status_from_str_invalid_mauled_str() {
        // Missing the ':'
        assert!(TributeStatus::from_str("mauled velociraptor").is_err());
    }
}
