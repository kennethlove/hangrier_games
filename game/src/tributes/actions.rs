use crate::areas::Area;
use crate::items::Item;
use crate::tributes::Tribute;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

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

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Action::None => write!(f, "none"),
            Action::Move(_) => write!(f, "move"),
            Action::Rest => write!(f, "rest"),
            Action::UseItem(_) => write!(f, "use item"),
            Action::Attack => write!(f, "attack"),
            Action::Hide => write!(f, "hide"),
            Action::TakeItem => write!(f, "take item"),
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

#[derive(Debug, PartialEq)]
pub enum AttackResult {
    AttackerWins,
    AttackerWinsDecisively,
    DefenderWins,
    DefenderWinsDecisively,
    Miss,
}

#[derive(Debug, PartialEq)]
pub enum AttackOutcome {
    Kill(Tribute, Tribute),
    Wound(Tribute, Tribute),
    Miss(Tribute, Tribute),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Action::None, "none")]
    #[case(Action::Move(None), "move")]
    #[case(Action::Rest, "rest")]
    #[case(Action::UseItem(None), "use item")]
    #[case(Action::UseItem(Some(Item::new_weapon("lasso"))), "use item")]
    #[case(Action::Attack, "attack")]
    #[case(Action::Hide, "hide")]
    #[case(Action::TakeItem, "take item")]
    fn action_to_string(#[case] action: Action, #[case] expected: &str) {
        assert_eq!(action.to_string(), expected.to_string());
    }

    #[rstest]
    #[case("none", Action::None)]
    #[case("move", Action::Move(None))]
    #[case("rest", Action::Rest)]
    #[case("use item", Action::UseItem(None))]
    #[case("attack", Action::Attack)]
    #[case("hide", Action::Hide)]
    #[case("take item", Action::TakeItem)]
    fn action_from_str(#[case] input: &str, #[case] action: Action) {
        assert_eq!(Action::from_str(input).unwrap(), action);
    }

    #[test]
    fn action_from_str_invalid() {
        assert_eq!(Action::from_str("do a barrel roll"), Err(()));
    }

    #[test]
    fn tribute_action_new() {
        assert_eq!(TributeAction::new(Action::None, None), TributeAction {
            action: Action::None,
            target: None
        });
    }
}
