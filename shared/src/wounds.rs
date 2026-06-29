use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WoundType {
    Cut,
    Stab,
    Crush,
    Burn,
    Pierce,
    Tear,
    Amputation,
    Infection,
}

impl fmt::Display for WoundType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WoundType::Cut => write!(f, "cut"),
            WoundType::Stab => write!(f, "stab"),
            WoundType::Crush => write!(f, "crush"),
            WoundType::Burn => write!(f, "burn"),
            WoundType::Pierce => write!(f, "pierce"),
            WoundType::Tear => write!(f, "tear"),
            WoundType::Amputation => write!(f, "amputation"),
            WoundType::Infection => write!(f, "infection"),
        }
    }
}

impl FromStr for WoundType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cut" => Ok(WoundType::Cut),
            "stab" => Ok(WoundType::Stab),
            "crush" => Ok(WoundType::Crush),
            "burn" => Ok(WoundType::Burn),
            "pierce" => Ok(WoundType::Pierce),
            "tear" => Ok(WoundType::Tear),
            "amputation" => Ok(WoundType::Amputation),
            "infection" => Ok(WoundType::Infection),
            _ => Err(format!("unknown wound type: {s}")),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WoundSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

impl WoundSeverity {
    pub fn blood_loss_per_period(&self) -> u32 {
        match self {
            WoundSeverity::Minor => 5,
            WoundSeverity::Moderate => 15,
            WoundSeverity::Severe => 40,
            WoundSeverity::Critical => 80,
        }
    }
}

impl fmt::Display for WoundSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WoundSeverity::Minor => write!(f, "minor"),
            WoundSeverity::Moderate => write!(f, "moderate"),
            WoundSeverity::Severe => write!(f, "severe"),
            WoundSeverity::Critical => write!(f, "critical"),
        }
    }
}

impl FromStr for WoundSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minor" => Ok(WoundSeverity::Minor),
            "moderate" => Ok(WoundSeverity::Moderate),
            "severe" => Ok(WoundSeverity::Severe),
            "critical" => Ok(WoundSeverity::Critical),
            _ => Err(format!("unknown wound severity: {s}")),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BodyPart {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

impl fmt::Display for BodyPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyPart::Head => write!(f, "head"),
            BodyPart::Torso => write!(f, "torso"),
            BodyPart::LeftArm => write!(f, "left arm"),
            BodyPart::RightArm => write!(f, "right arm"),
            BodyPart::LeftLeg => write!(f, "left leg"),
            BodyPart::RightLeg => write!(f, "right leg"),
        }
    }
}

impl FromStr for BodyPart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "head" => Ok(BodyPart::Head),
            "torso" => Ok(BodyPart::Torso),
            "left arm" | "leftarm" => Ok(BodyPart::LeftArm),
            "right arm" | "rightarm" => Ok(BodyPart::RightArm),
            "left leg" | "leftleg" => Ok(BodyPart::LeftLeg),
            "right leg" | "rightleg" => Ok(BodyPart::RightLeg),
            _ => Err(format!("unknown body part: {s}")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Wound {
    pub wound_type: WoundType,
    pub severity: WoundSeverity,
    pub body_part: BodyPart,
    #[serde(default = "default_true")]
    pub bleeding: bool,
    #[serde(default)]
    pub infected: bool,
    #[serde(default)]
    pub created_day: Option<i64>,
}

fn default_true() -> bool {
    true
}

impl Wound {
    pub fn new(wound_type: WoundType, severity: WoundSeverity, body_part: BodyPart) -> Self {
        Self {
            wound_type,
            severity,
            body_part,
            bleeding: true,
            infected: false,
            created_day: None,
        }
    }

    pub fn blood_loss_per_period(&self) -> u32 {
        if self.bleeding {
            self.severity.blood_loss_per_period()
        } else {
            0
        }
    }

    pub fn heals_naturally(&mut self, infection_rng: f64) {
        match self.severity {
            WoundSeverity::Minor => {
                self.bleeding = false;
            }
            WoundSeverity::Moderate => {
                self.bleeding = false;
            }
            WoundSeverity::Severe => {}
            WoundSeverity::Critical => {
                if infection_rng < 0.25 {
                    self.infected = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wound_is_bleeding() {
        let w = Wound::new(WoundType::Cut, WoundSeverity::Minor, BodyPart::Torso);
        assert!(w.bleeding);
        assert!(!w.infected);
        assert_eq!(w.created_day, None);
    }

    #[test]
    fn minor_wound_blood_loss() {
        let w = Wound::new(WoundType::Cut, WoundSeverity::Minor, BodyPart::Torso);
        assert_eq!(w.blood_loss_per_period(), 5);
    }

    #[test]
    fn moderate_wound_blood_loss() {
        let w = Wound::new(WoundType::Stab, WoundSeverity::Moderate, BodyPart::Torso);
        assert_eq!(w.blood_loss_per_period(), 15);
    }

    #[test]
    fn severe_wound_blood_loss() {
        let w = Wound::new(WoundType::Crush, WoundSeverity::Severe, BodyPart::LeftArm);
        assert_eq!(w.blood_loss_per_period(), 40);
    }

    #[test]
    fn critical_wound_blood_loss() {
        let w = Wound::new(WoundType::Pierce, WoundSeverity::Critical, BodyPart::Head);
        assert_eq!(w.blood_loss_per_period(), 80);
    }

    #[test]
    fn stopped_wound_no_blood_loss() {
        let mut w = Wound::new(WoundType::Cut, WoundSeverity::Minor, BodyPart::Torso);
        w.bleeding = false;
        assert_eq!(w.blood_loss_per_period(), 0);
    }

    #[test]
    fn minor_wound_heals_naturally() {
        let mut w = Wound::new(WoundType::Cut, WoundSeverity::Minor, BodyPart::Torso);
        w.heals_naturally(0.0);
        assert!(!w.bleeding);
    }

    #[test]
    fn moderate_wound_heals_naturally() {
        let mut w = Wound::new(WoundType::Stab, WoundSeverity::Moderate, BodyPart::Torso);
        w.heals_naturally(0.0);
        assert!(!w.bleeding);
    }

    #[test]
    fn severe_wound_does_not_heal_naturally() {
        let mut w = Wound::new(WoundType::Crush, WoundSeverity::Severe, BodyPart::Torso);
        w.heals_naturally(0.0);
        assert!(w.bleeding);
    }

    #[test]
    fn critical_wound_infection_chance() {
        let mut w = Wound::new(WoundType::Pierce, WoundSeverity::Critical, BodyPart::Head);
        w.heals_naturally(0.1);
        assert!(w.infected);
        assert!(w.bleeding);
    }

    #[test]
    fn critical_wound_no_infection_on_high_roll() {
        let mut w = Wound::new(WoundType::Pierce, WoundSeverity::Critical, BodyPart::Head);
        w.heals_naturally(0.5);
        assert!(!w.infected);
        assert!(w.bleeding);
    }

    #[test]
    fn wound_type_display() {
        assert_eq!(WoundType::Cut.to_string(), "cut");
        assert_eq!(WoundType::Amputation.to_string(), "amputation");
    }

    #[test]
    fn wound_type_from_str() {
        assert_eq!("stab".parse::<WoundType>(), Ok(WoundType::Stab));
        assert_eq!("Burn".parse::<WoundType>(), Ok(WoundType::Burn));
        assert!("unknown".parse::<WoundType>().is_err());
    }

    #[test]
    fn body_part_display() {
        assert_eq!(BodyPart::LeftArm.to_string(), "left arm");
        assert_eq!(BodyPart::RightLeg.to_string(), "right leg");
    }

    #[test]
    fn body_part_from_str() {
        assert_eq!("left arm".parse::<BodyPart>(), Ok(BodyPart::LeftArm));
        assert_eq!("head".parse::<BodyPart>(), Ok(BodyPart::Head));
        assert!("foot".parse::<BodyPart>().is_err());
    }
}
