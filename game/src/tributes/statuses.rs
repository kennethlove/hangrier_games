use crate::threats::animals::Animal;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use strum::EnumIter;

#[derive(Clone, Debug, Default, Deserialize, EnumIter, Eq, PartialEq, Serialize)]
pub enum TributeStatus {
    #[default]
    Healthy,
    RecentlyDead,
    Dead,
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
            "recently dead" => Ok(TributeStatus::RecentlyDead),
            "dead" => Ok(TributeStatus::Dead),
            _ => Err(()),
        }
    }
}

impl Display for TributeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TributeStatus::Healthy => write!(f, "healthy"),
            TributeStatus::RecentlyDead => write!(f, "recently dead"),
            TributeStatus::Dead => write!(f, "dead"),
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
    #[case(TributeStatus::RecentlyDead, "recently dead")]
    #[case(TributeStatus::Dead, "dead")]
    #[case(TributeStatus::Mauled(Animal::Bear), "mauled: bear")]
    fn tribute_status_to_string(#[case] status: TributeStatus, #[case] expected: &str) {
        assert_eq!(status.to_string(), expected.to_string());
    }

    #[rstest]
    #[case("healthy", TributeStatus::Healthy)]
    #[case("recently dead", TributeStatus::RecentlyDead)]
    #[case("dead", TributeStatus::Dead)]
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
