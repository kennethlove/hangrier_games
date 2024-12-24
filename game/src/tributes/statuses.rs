use std::fmt::Display;
use std::str::FromStr;
use crate::animals::Animal;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
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
            let animal_name = s.split_whitespace().skip(1).map(|s| s.to_string()).collect::<Vec<String>>().join(" ");
            let animal = Animal::from_str(animal_name.as_str());
            if animal.is_ok() {
                return Ok(TributeStatus::Mauled(animal?))
            };
        }
        match s.to_lowercase().as_str() {
            "healthy" => Ok(TributeStatus::Healthy),
            "wounded" => Ok(TributeStatus::Wounded),
            "injured" => Ok(TributeStatus::Wounded),
            "starving" => Ok(TributeStatus::Starving),
            "dehydrated" => Ok(TributeStatus::Dehydrated),
            "sick" => Ok(TributeStatus::Sick),
            "poisoned" => Ok(TributeStatus::Poisoned),
            "recently-dead" => Ok(TributeStatus::RecentlyDead),
            "recently_dead" => Ok(TributeStatus::RecentlyDead),
            "recently dead" => Ok(TributeStatus::RecentlyDead),
            "recentlydead" => Ok(TributeStatus::RecentlyDead),
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
            TributeStatus::Mauled(animal) => write!(f, "mauled {}", animal.to_string()),
        }
    }
}

impl TributeStatus {
    fn as_str(&self) -> &str {
        match self {
            TributeStatus::Healthy => "healthy",
            TributeStatus::Wounded => "wounded",
            TributeStatus::Starving => "starving",
            TributeStatus::Dehydrated => "dehydrated",
            TributeStatus::Sick => "sick",
            TributeStatus::Poisoned => "poisoned",
            TributeStatus::RecentlyDead => "recently dead",
            TributeStatus::Dead => "dead",
            TributeStatus::Electrocuted => "electrocuted",
            TributeStatus::Frozen => "frozen",
            TributeStatus::Overheated => "overheated",
            TributeStatus::Broken => "broken",
            TributeStatus::Infected => "infected",
            TributeStatus::Drowned => "drowned",
            TributeStatus::Burned => "burned",
            TributeStatus::Buried => "buried",
            TributeStatus::Mauled(animal) => format!("mauled {}", animal.to_string()).as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribute_status_to_str() {
        let ts = TributeStatus::Healthy;
        assert_eq!("healthy", ts.as_str());
    }
}
