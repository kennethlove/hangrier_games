use std::str::FromStr;

use crate::tributes::Tribute;


#[derive(Clone, Debug, Default, PartialEq)]
pub enum TributeAction {
    #[default]
    None,
    Move(Option<String>),
    Rest,
    UseItem(Option<String>),
    Attack,
    Hide,
    TakeItem,
}

impl TributeAction {
    pub fn as_str(&self) -> &str {
        match self {
            TributeAction::None => "None",
            TributeAction::Move(_) => "Move",
            TributeAction::Rest => "Rest",
            TributeAction::UseItem(_) => "Use Item",
            TributeAction::Attack => "Attack",
            TributeAction::Hide => "Hide",
            TributeAction::TakeItem => "Take Item",
        }
    }
}

impl FromStr for TributeAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(TributeAction::None),
            "move" => Ok(TributeAction::Move(None)),
            "rest" => Ok(TributeAction::Rest),
            "use item" => Ok(TributeAction::UseItem(None)),
            "attack" => Ok(TributeAction::Attack),
            "hide" => Ok(TributeAction::Hide),
            "take item" => Ok(TributeAction::TakeItem),
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
