use crate::threats::animals::Animal;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

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
        if s.to_lowercase().contains("animal attack") {
            return if let Some(animal_name) = s.split_once(':').map(|x| x.1) {
                if let Ok(animal) = Animal::from_str(animal_name.trim()) {
                    Ok(TributeEvent::AnimalAttack(animal))
                } else { Err(()) }
            } else { Err(()) }
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
            _ => Err(()),
        }
    }
}

impl Display for TributeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TributeEvent::AnimalAttack(animal) => write!(f, "animal attack: {}", animal),
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
    pub fn random() -> TributeEvent {
        let mut rng = rand::thread_rng();
        let animal = Animal::random();
        let events = [
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
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(TributeEvent::AnimalAttack(Animal::Wolf), "animal attack: wolf")]
    #[case(TributeEvent::Dysentery, "dysentery")]
    #[case(TributeEvent::LightningStrike, "lightning strike")]
    #[case(TributeEvent::Hypothermia, "hypothermia")]
    #[case(TributeEvent::HeatStroke, "heat stroke")]
    #[case(TributeEvent::Dehydration, "dehydration")]
    #[case(TributeEvent::Starvation, "starvation")]
    #[case(TributeEvent::Poisoning, "poisoning")]
    #[case(TributeEvent::BrokenBone, "broken bone")]
    #[case(TributeEvent::Infection, "infection")]
    #[case(TributeEvent::Drowning, "drowning")]
    #[case(TributeEvent::Burn, "burn")]
    fn tribute_event_to_string(#[case] event: TributeEvent, #[case] expected: &str) {
        assert_eq!(event.to_string(), expected);
    }

    #[rstest]
    #[case("animal attack: wolf", TributeEvent::AnimalAttack(Animal::Wolf))]
    #[case("dysentery", TributeEvent::Dysentery)]
    #[case("lightning strike", TributeEvent::LightningStrike)]
    #[case("hypothermia", TributeEvent::Hypothermia)]
    #[case("heat stroke", TributeEvent::HeatStroke)]
    #[case("dehydration", TributeEvent::Dehydration)]
    #[case("starvation", TributeEvent::Starvation)]
    #[case("poisoning", TributeEvent::Poisoning)]
    #[case("broken bone", TributeEvent::BrokenBone)]
    #[case("infection", TributeEvent::Infection)]
    #[case("drowning", TributeEvent::Drowning)]
    #[case("burn", TributeEvent::Burn)]
    fn tribute_event_from_str(#[case] input: &str, #[case] event: TributeEvent) {
        assert_eq!(TributeEvent::from_str(input).unwrap(), event);
    }

    #[test]
    fn random_tribute_event() {
        let te = TributeEvent::random();
        assert_eq!(TributeEvent::from_str(&te.to_string()).unwrap(), te);
    }

    #[test]
    fn tribute_event_from_str_invalid() {
        assert!(TributeEvent::from_str("resurrection").is_err());
    }

    #[test]
    fn tribute_event_from_str_invalid_animal() {
        assert!(TributeEvent::from_str("animal attack: dragon").is_err());
    }

    #[test]
    fn tribute_event_from_str_missing_separator() {
        assert!(TributeEvent::from_str("animal attack coyote").is_err());
    }
}
