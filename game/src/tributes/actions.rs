use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::areas::Area;
use crate::items::Item;
use crate::tributes::Tribute;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TributeAction {
    pub action: Action,
    pub target: Option<Tribute>,
}

impl TributeAction {
    pub fn new(action: Action, target: Option<Tribute>) -> TributeAction {
        TributeAction { action, target }
    }
}


#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum Action {
    #[default]
    None,
    Move(Option<Area>),
    Rest,
    UseItem(Option<Item>),
    Attack,
    Hide,
    TakeItem,
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::None => "none",
            Action::Move(_) => "move",
            Action::Rest => "rest",
            Action::UseItem(_) => "use item",
            Action::Attack => "attack",
            Action::Hide => "hide",
            Action::TakeItem => "take item",
        }
    }
}

impl FromStr for Action {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Action::None),
            "move" => Ok(Action::Move(None)),
            "rest" => Ok(Action::Rest),
            "use item" => Ok(Action::UseItem(None)),
            "attack" => Ok(Action::Attack),
            "hide" => Ok(Action::Hide),
            "take item" => Ok(Action::TakeItem),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum AttackResult {
    AttackerWins,
    AttackerWinsDecisively,
    DefenderWins,
    DefenderWinsDecisively,
    Miss,
}

#[derive(Debug)]
pub enum AttackOutcome {
    Kill(Tribute, Tribute),
    Wound(Tribute, Tribute),
    Miss(Tribute, Tribute),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_to_str() {
        assert_eq!(Action::None.as_str(), "none");
    }

    #[test]
    fn action_from_str() {
        assert_eq!(Action::from_str("none").unwrap(), Action::None);
    }

    #[test]
    fn tribute_action_new() {
        assert_eq!(TributeAction::new(Action::None, None), TributeAction { action: Action::None, target: None });
    }
}
