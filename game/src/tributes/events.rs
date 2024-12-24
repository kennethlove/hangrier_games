use rand::Rng;
use std::fmt::Display;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::animals::Animal;

#[derive(Clone, Debug, PartialOrd, PartialEq, Serialize, Deserialize)]
pub enum TributeEvent {
    AnimalAttack(Animal),
    Dysentery,
    LightningStrike,
    Hypothermia,
    HeatStroke,
    Dehydration,
    Starvation,
    Poisoning,
    BrokenBone,
    Infection,
    Drowning,
    Burn,
}

impl FromStr for TributeEvent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("animal attack") {
            let animal_name = s.split_whitespace().skip(2).map(|s| s.to_string()).collect::<Vec<String>>().join(" ");

            let animal = Animal::from_str(animal_name.as_str());
            if animal.is_ok() {
                return Ok(TributeEvent::AnimalAttack(animal?))
            };
        }
        match s {
            "dysentery" => Ok(TributeEvent::Dysentery),
            "lightning strike" => Ok(TributeEvent::LightningStrike),
            "hypothermia" => Ok(TributeEvent::Hypothermia),
            "heat stroke" => Ok(TributeEvent::HeatStroke),
            "dehydration" => Ok(TributeEvent::Dehydration),
            "starvation" => Ok(TributeEvent::Starvation),
            "poisoning" => Ok(TributeEvent::Poisoning),
            "broken bone" => Ok(TributeEvent::BrokenBone),
            "infection" => Ok(TributeEvent::Infection),
            "drowning" => Ok(TributeEvent::Drowning),
            "burn" => Ok(TributeEvent::Burn),
            _ => Err(())
        }
    }
}

impl Display for TributeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TributeEvent::AnimalAttack(animal) => write!(f, "animal attack {}", animal),
            TributeEvent::Dysentery => write!(f, "dysentery"),
            TributeEvent::LightningStrike => write!(f, "lightning strike"),
            TributeEvent::Hypothermia => write!(f, "hypothermia"),
            TributeEvent::HeatStroke => write!(f, "heat stroke"),
            TributeEvent::Dehydration => write!(f, "dehydration"),
            TributeEvent::Starvation => write!(f, "starvation"),
            TributeEvent::Poisoning => write!(f, "poisoning"),
            TributeEvent::BrokenBone => write!(f, "broken bone"),
            TributeEvent::Infection => write!(f, "infection"),
            TributeEvent::Drowning => write!(f, "drowning"),
            TributeEvent::Burn => write!(f, "burn"),
        }
    }
}

impl TributeEvent {
    pub fn as_str(&self) -> &str {
        match self {
            TributeEvent::AnimalAttack(animal) => {
                let s = format!("animal attack {}", animal.as_str());
                Box::leak(s.into_boxed_str())
            },
            TributeEvent::Dysentery => "dysentery",
            TributeEvent::LightningStrike =>"lightning strike",
            TributeEvent::Hypothermia => "hypothermia",
            TributeEvent::HeatStroke => "heat stroke",
            TributeEvent::Dehydration => "dehydration",
            TributeEvent::Starvation => "starvation",
            TributeEvent::Poisoning => "poisoning",
            TributeEvent::BrokenBone => "broken bone",
            TributeEvent::Infection => "infection",
            TributeEvent::Drowning => "drowning",
            TributeEvent::Burn => "burn",
        }
    }

    pub fn random() -> TributeEvent {
        let mut rng = rand::thread_rng();
        let animal = Animal::random();
        let events = vec![
            TributeEvent::AnimalAttack(animal),
            TributeEvent::Dysentery,
            TributeEvent::LightningStrike,
            TributeEvent::Hypothermia,
            TributeEvent::HeatStroke,
            TributeEvent::Dehydration,
            TributeEvent::Starvation,
            TributeEvent::Poisoning,
            TributeEvent::BrokenBone,
            TributeEvent::Infection,
            TributeEvent::Drowning,
            TributeEvent::Burn,
        ];
        let index = rng.gen_range(0..events.len());
        events[index].clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::animals::Animal::Wolf;
    use super::*;

    #[test]
    fn tribute_event_to_str() {
        let te = TributeEvent::AnimalAttack(Animal::Squirrel);
        assert_eq!(te.as_str(), "animal attack squirrel");
    }

    #[test]
    fn tribute_event_from_str() {
        assert_eq!(TributeEvent::from_str("animal attack wolf").unwrap(), TributeEvent::AnimalAttack(Wolf));
    }

    #[test]
    fn random_tribute_event() {
        let te = TributeEvent::random();
        assert_eq!(TributeEvent::from_str(te.as_str()).unwrap(), te);
    }
}
