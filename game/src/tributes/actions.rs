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
    /// Spend the turn proposing an alliance to a non-allied tribute in the
    /// current area. The proposal succeeds or fails via the existing alliance
    /// roll (`game::tributes::alliances::try_form_alliance`); either way the
    /// turn is consumed. See spec §6.1.
    ProposeAlliance,

    /// Spend the turn looking for shelter in the current area. Brain
    /// override decides when to surface this. Resolution lives in the
    /// action handler (out of scope for this task).
    SeekShelter,
    /// Spend the turn foraging for food in the current area.
    Forage,
    /// Spend the turn drinking from a terrain water source in the current
    /// area (no item consumed).
    DrinkFromTerrain,
    /// Spend the turn eating a Food item from inventory. `None` lets the
    /// action handler pick the best Food item; `Some(item)` selects it.
    Eat(Option<Item>),
    /// Spend the turn drinking a Water item from inventory.
    DrinkItem(Option<Item>),
    /// Sleep for `duration_phases` phases. The tribute is unreachable to the
    /// brain pipeline while sleeping; the engine decrements the counter and
    /// emits `MessagePayload::TributeWoke` on the wake-up phase. Defined here
    /// as the substrate for the four-phase sleep mechanic; the brain does
    /// not yet score this action (see bd-xi0z follow-ups).
    Sleep {
        duration_phases: u8,
    },
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
            Action::ProposeAlliance => write!(f, "propose alliance"),
            Action::SeekShelter => write!(f, "seek shelter"),
            Action::Forage => write!(f, "forage"),
            Action::DrinkFromTerrain => write!(f, "drink from terrain"),
            Action::Eat(_) => write!(f, "eat"),
            Action::DrinkItem(_) => write!(f, "drink item"),
            Action::Sleep { .. } => write!(f, "sleep"),
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
            "propose alliance" => Ok(Action::ProposeAlliance),
            "seek shelter" => Ok(Action::SeekShelter),
            "forage" => Ok(Action::Forage),
            "drink from terrain" => Ok(Action::DrinkFromTerrain),
            "eat" => Ok(Action::Eat(None)),
            "drink item" => Ok(Action::DrinkItem(None)),
            "sleep" => Ok(Action::Sleep { duration_phases: 0 }),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AttackResult {
    AttackerWins,
    AttackerWinsDecisively,
    CriticalHit, // Natural 20 on attack roll - triple damage
    DefenderWins,
    DefenderWinsDecisively,
    PerfectBlock,   // Natural 20 on defense roll - counter attack
    CriticalFumble, // Natural 1 on attack roll - attacker takes damage
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
        assert_eq!(
            TributeAction::new(Action::None, None),
            TributeAction {
                action: Action::None,
                target: None
            }
        );
    }
}

#[cfg(test)]
mod survival_action_tests {
    use super::*;

    #[test]
    fn survival_actions_round_trip_serde() {
        for a in [
            Action::SeekShelter,
            Action::Forage,
            Action::DrinkFromTerrain,
            Action::Eat(None),
            Action::DrinkItem(None),
        ] {
            let json = serde_json::to_string(&a).unwrap();
            let back: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", a), format!("{:?}", back));
        }
    }

    #[test]
    fn survival_actions_display_and_fromstr() {
        assert_eq!(Action::SeekShelter.to_string(), "seek shelter");
        assert_eq!(Action::Forage.to_string(), "forage");
        assert_eq!(Action::DrinkFromTerrain.to_string(), "drink from terrain");
        assert_eq!(Action::Eat(None).to_string(), "eat");
        assert_eq!(Action::DrinkItem(None).to_string(), "drink item");
        assert!(matches!(
            "seek shelter".parse::<Action>(),
            Ok(Action::SeekShelter)
        ));
        assert!(matches!("forage".parse::<Action>(), Ok(Action::Forage)));
        assert!(matches!(
            "drink from terrain".parse::<Action>(),
            Ok(Action::DrinkFromTerrain)
        ));
        assert!(matches!("eat".parse::<Action>(), Ok(Action::Eat(None))));
        assert!(matches!(
            "drink item".parse::<Action>(),
            Ok(Action::DrinkItem(None))
        ));
    }
}
