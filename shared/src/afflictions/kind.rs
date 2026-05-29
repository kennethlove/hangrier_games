use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Categories of afflictions a tribute can carry. Permanent kinds
/// (`MissingArm`, `MissingLeg`, `Blind`, `Deaf`) cannot be cured in v1;
/// reversible kinds progress / heal via the cascade and cure paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhobiaTrigger {
    Fire,
    Water,
    Dark,
    Blood,
    Heights,
    Enclosed,
    Open,
    Animal,
    Tribute,
    TraitGroup,
}

impl fmt::Display for PhobiaTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PhobiaTrigger::Fire => write!(f, "fire"),
            PhobiaTrigger::Water => write!(f, "water"),
            PhobiaTrigger::Dark => write!(f, "dark"),
            PhobiaTrigger::Blood => write!(f, "blood"),
            PhobiaTrigger::Heights => write!(f, "heights"),
            PhobiaTrigger::Enclosed => write!(f, "enclosed"),
            PhobiaTrigger::Open => write!(f, "open"),
            PhobiaTrigger::Animal => write!(f, "animal"),
            PhobiaTrigger::Tribute => write!(f, "tribute"),
            PhobiaTrigger::TraitGroup => write!(f, "trait_group"),
        }
    }
}

impl FromStr for PhobiaTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fire" => Ok(PhobiaTrigger::Fire),
            "water" => Ok(PhobiaTrigger::Water),
            "dark" => Ok(PhobiaTrigger::Dark),
            "blood" => Ok(PhobiaTrigger::Blood),
            "heights" => Ok(PhobiaTrigger::Heights),
            "enclosed" => Ok(PhobiaTrigger::Enclosed),
            "open" => Ok(PhobiaTrigger::Open),
            "animal" => Ok(PhobiaTrigger::Animal),
            "tribute" => Ok(PhobiaTrigger::Tribute),
            "trait_group" => Ok(PhobiaTrigger::TraitGroup),
            other => Err(format!("unknown PhobiaTrigger: {other}")),
        }
    }
}

/// Target of a fixation affliction. A tribute becomes fixated on a specific
/// entity: another tribute, an item, or an area.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum FixationTarget {
    Tribute(String),
    Item(String),
    Area(String),
}

impl fmt::Display for FixationTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixationTarget::Tribute(id) => write!(f, "tribute:{id}"),
            FixationTarget::Item(id) => write!(f, "item:{id}"),
            FixationTarget::Area(name) => write!(f, "area:{name}"),
        }
    }
}

impl FromStr for FixationTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(id) = s.strip_prefix("tribute:") {
            Ok(FixationTarget::Tribute(id.to_string()))
        } else if let Some(id) = s.strip_prefix("item:") {
            Ok(FixationTarget::Item(id.to_string()))
        } else if let Some(name) = s.strip_prefix("area:") {
            Ok(FixationTarget::Area(name.to_string()))
        } else {
            Err(format!("unknown FixationTarget: {s}"))
        }
    }
}

/// Substances a tribute can become addicted to (spec §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Substance {
    /// Adrenaline-based combat stimulants (yayo, go-juice, adrenaline).
    Stimulant,
    /// Opioid-class painkiller, sponsor-gift only.
    Morphling,
    /// Ethanol — sponsor-gift only.
    Alcohol,
    /// Standard painkiller, sponsor-gift only.
    Painkiller,
}

impl Substance {
    /// Icon name for sprite-ui.svg lookups.
    pub fn icon_name(&self) -> &'static str {
        match self {
            Substance::Stimulant => "stimulant",
            Substance::Morphling => "morphling",
            Substance::Alcohol => "alcohol",
            Substance::Painkiller => "painkiller",
        }
    }
}

impl fmt::Display for Substance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Substance::Stimulant => write!(f, "stimulant"),
            Substance::Morphling => write!(f, "morphling"),
            Substance::Alcohol => write!(f, "alcohol"),
            Substance::Painkiller => write!(f, "painkiller"),
        }
    }
}

impl FromStr for Substance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stimulant" => Ok(Substance::Stimulant),
            "morphling" => Ok(Substance::Morphling),
            "alcohol" => Ok(Substance::Alcohol),
            "painkiller" => Ok(Substance::Painkiller),
            other => Err(format!("unknown Substance: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AfflictionKind {
    Wounded,
    Infected,
    MissingArm,
    MissingLeg,
    Blind,
    Deaf,
    BrokenBone,
    Poisoned,
    Starving,
    Dehydrated,
    Frozen,
    Overheated,
    Burned,
    Sick,
    Electrocuted,
    Drowned,
    Buried,
    Trauma,
    Phobia(PhobiaTrigger),
    Fixation(FixationTarget),
    Addiction(Substance),
}

impl fmt::Display for AfflictionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AfflictionKind::Wounded => write!(f, "wounded"),
            AfflictionKind::Infected => write!(f, "infected"),
            AfflictionKind::MissingArm => write!(f, "missing_arm"),
            AfflictionKind::MissingLeg => write!(f, "missing_leg"),
            AfflictionKind::Blind => write!(f, "blind"),
            AfflictionKind::Deaf => write!(f, "deaf"),
            AfflictionKind::BrokenBone => write!(f, "broken_bone"),
            AfflictionKind::Poisoned => write!(f, "poisoned"),
            AfflictionKind::Starving => write!(f, "starving"),
            AfflictionKind::Dehydrated => write!(f, "dehydrated"),
            AfflictionKind::Frozen => write!(f, "frozen"),
            AfflictionKind::Overheated => write!(f, "overheated"),
            AfflictionKind::Burned => write!(f, "burned"),
            AfflictionKind::Sick => write!(f, "sick"),
            AfflictionKind::Electrocuted => write!(f, "electrocuted"),
            AfflictionKind::Drowned => write!(f, "drowned"),
            AfflictionKind::Buried => write!(f, "buried"),
            AfflictionKind::Trauma => write!(f, "trauma"),
            AfflictionKind::Phobia(trigger) => write!(f, "phobia:{trigger}"),
            AfflictionKind::Fixation(target) => write!(f, "fixation:{target}"),
            AfflictionKind::Addiction(sub) => write!(f, "addiction:{sub}"),
        }
    }
}

impl FromStr for AfflictionKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wounded" => Ok(AfflictionKind::Wounded),
            "infected" => Ok(AfflictionKind::Infected),
            "missing_arm" => Ok(AfflictionKind::MissingArm),
            "missing_leg" => Ok(AfflictionKind::MissingLeg),
            "blind" => Ok(AfflictionKind::Blind),
            "deaf" => Ok(AfflictionKind::Deaf),
            "broken_bone" => Ok(AfflictionKind::BrokenBone),
            "poisoned" => Ok(AfflictionKind::Poisoned),
            "starving" => Ok(AfflictionKind::Starving),
            "dehydrated" => Ok(AfflictionKind::Dehydrated),
            "frozen" => Ok(AfflictionKind::Frozen),
            "overheated" => Ok(AfflictionKind::Overheated),
            "burned" => Ok(AfflictionKind::Burned),
            "sick" => Ok(AfflictionKind::Sick),
            "electrocuted" => Ok(AfflictionKind::Electrocuted),
            "drowned" => Ok(AfflictionKind::Drowned),
            "buried" => Ok(AfflictionKind::Buried),
            "trauma" => Ok(AfflictionKind::Trauma),
            rest if rest.starts_with("phobia:") => {
                let trigger_str = rest.strip_prefix("phobia:").unwrap();
                let trigger = PhobiaTrigger::from_str(trigger_str)?;
                Ok(AfflictionKind::Phobia(trigger))
            }
            rest if rest.starts_with("fixation:") => {
                let target_str = rest.strip_prefix("fixation:").unwrap();
                let target = FixationTarget::from_str(target_str)?;
                Ok(AfflictionKind::Fixation(target))
            }
            rest if rest.starts_with("addiction:") => {
                let sub_str = rest.strip_prefix("addiction:").unwrap();
                let sub = Substance::from_str(sub_str)?;
                Ok(AfflictionKind::Addiction(sub))
            }
            other => Err(format!("unknown AfflictionKind: {other}")),
        }
    }
}

/// Anatomical attachment points for body-part-specific afflictions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyPart {
    Arm,
    Leg,
    Eye,
    Ear,
    Skull,
    Rib,
    Hand,
    Foot,
}

impl fmt::Display for BodyPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyPart::Arm => write!(f, "arm"),
            BodyPart::Leg => write!(f, "leg"),
            BodyPart::Eye => write!(f, "eye"),
            BodyPart::Ear => write!(f, "ear"),
            BodyPart::Skull => write!(f, "skull"),
            BodyPart::Rib => write!(f, "rib"),
            BodyPart::Hand => write!(f, "hand"),
            BodyPart::Foot => write!(f, "foot"),
        }
    }
}

impl FromStr for BodyPart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "arm" => Ok(BodyPart::Arm),
            "leg" => Ok(BodyPart::Leg),
            "eye" => Ok(BodyPart::Eye),
            "ear" => Ok(BodyPart::Ear),
            "skull" => Ok(BodyPart::Skull),
            "rib" => Ok(BodyPart::Rib),
            "hand" => Ok(BodyPart::Hand),
            "foot" => Ok(BodyPart::Foot),
            other => Err(format!("unknown BodyPart: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phobia_trigger_display_roundtrip() {
        for trigger in [
            PhobiaTrigger::Fire,
            PhobiaTrigger::Water,
            PhobiaTrigger::Dark,
            PhobiaTrigger::Blood,
            PhobiaTrigger::Heights,
            PhobiaTrigger::Enclosed,
            PhobiaTrigger::Open,
            PhobiaTrigger::Animal,
            PhobiaTrigger::Tribute,
            PhobiaTrigger::TraitGroup,
        ] {
            let s = trigger.to_string();
            let parsed: PhobiaTrigger = s.parse().unwrap();
            assert_eq!(trigger, parsed);
        }
    }

    #[test]
    fn phobia_trigger_from_str_invalid() {
        assert!(PhobiaTrigger::from_str("spiders").is_err());
    }

    #[test]
    fn affliction_kind_phobia_display() {
        let kind = AfflictionKind::Phobia(PhobiaTrigger::Fire);
        assert_eq!(kind.to_string(), "phobia:fire");

        let kind = AfflictionKind::Phobia(PhobiaTrigger::Heights);
        assert_eq!(kind.to_string(), "phobia:heights");
    }

    #[test]
    fn affliction_kind_phobia_from_str() {
        let kind: AfflictionKind = "phobia:fire".parse().unwrap();
        assert_eq!(kind, AfflictionKind::Phobia(PhobiaTrigger::Fire));

        let kind: AfflictionKind = "phobia:dark".parse().unwrap();
        assert_eq!(kind, AfflictionKind::Phobia(PhobiaTrigger::Dark));
    }

    #[test]
    fn affliction_kind_phobia_from_str_invalid() {
        assert!(AfflictionKind::from_str("phobia:unknown").is_err());
    }

    #[test]
    fn fixation_target_display_roundtrip_tribute() {
        let target = FixationTarget::Tribute("uuid-123".to_string());
        let s = target.to_string();
        assert_eq!(s, "tribute:uuid-123");
        let parsed: FixationTarget = s.parse().unwrap();
        assert_eq!(target, parsed);
    }

    #[test]
    fn fixation_target_display_roundtrip_item() {
        let target = FixationTarget::Item("item-456".to_string());
        let s = target.to_string();
        assert_eq!(s, "item:item-456");
        let parsed: FixationTarget = s.parse().unwrap();
        assert_eq!(target, parsed);
    }

    #[test]
    fn fixation_target_display_roundtrip_area() {
        let target = FixationTarget::Area("cornucopia".to_string());
        let s = target.to_string();
        assert_eq!(s, "area:cornucopia");
        let parsed: FixationTarget = s.parse().unwrap();
        assert_eq!(target, parsed);
    }

    #[test]
    fn fixation_target_from_str_invalid() {
        assert!(FixationTarget::from_str("unknown:foo").is_err());
        assert!(FixationTarget::from_str("tribute").is_err());
    }

    #[test]
    fn affliction_kind_fixation_display() {
        let kind = AfflictionKind::Fixation(FixationTarget::Tribute("u-1".to_string()));
        assert_eq!(kind.to_string(), "fixation:tribute:u-1");

        let kind = AfflictionKind::Fixation(FixationTarget::Item("i-1".to_string()));
        assert_eq!(kind.to_string(), "fixation:item:i-1");

        let kind = AfflictionKind::Fixation(FixationTarget::Area("sector1".to_string()));
        assert_eq!(kind.to_string(), "fixation:area:sector1");
    }

    #[test]
    fn affliction_kind_fixation_from_str() {
        let kind: AfflictionKind = "fixation:tribute:u-1".parse().unwrap();
        assert_eq!(
            kind,
            AfflictionKind::Fixation(FixationTarget::Tribute("u-1".to_string()))
        );

        let kind: AfflictionKind = "fixation:item:i-1".parse().unwrap();
        assert_eq!(
            kind,
            AfflictionKind::Fixation(FixationTarget::Item("i-1".to_string()))
        );

        let kind: AfflictionKind = "fixation:area:sector1".parse().unwrap();
        assert_eq!(
            kind,
            AfflictionKind::Fixation(FixationTarget::Area("sector1".to_string()))
        );
    }

    #[test]
    fn affliction_kind_fixation_from_str_invalid() {
        assert!(AfflictionKind::from_str("fixation:unknown:foo").is_err());
    }
}
