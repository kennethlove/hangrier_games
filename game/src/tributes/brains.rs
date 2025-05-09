use crate::tributes::actions::{Action, TributeAction};
use crate::tributes::Tribute;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    pub previous_actions: Vec<TributeAction>,
    pub preferred_action: Option<Action>,
    pub preferred_action_percentage: f64,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            previous_actions: Vec::new(),
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }
}

impl Brain {
    pub fn set_preferred_action(&mut self, action: Action, percentage: f64) {
        self.preferred_action = Some(action);
        self.preferred_action_percentage = percentage;
    }

    pub fn clear_preferred_action(&mut self) {
        self.preferred_action = None;
        self.preferred_action_percentage = 0.0;
    }

    /// Decide on an action for the tribute to take
    /// First weighs any preferred actions, then decides based on current state
    pub fn act(&mut self, tribute: &Tribute, nearby_tributes: usize, rng: impl Rng) -> Action {
        let action = self.decide_on_action(tribute, nearby_tributes, rng);

        // self.previous_actions
        //     .push(TributeAction::new(action.clone(), None));
        //
        action
    }

    /// The AI for a tribute. Automatic decisions based on current state.
    fn decide_on_action(&mut self, tribute: &Tribute, nearby_tributes: usize, mut rng: impl Rng) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }
        // if tribute.attributes.movement <= 0 {
        //     return Action::Rest;
        // }

        // If there is a preferred action, we should take it, assuming a positive roll
        if let Some(preferred_action) = self.preferred_action.clone() {
            if rng.gen_bool(self.preferred_action_percentage) {
                return preferred_action;
            }
        }

        // Does the tribute have items?
        if !tribute.consumables().is_empty() {
            // Use an item
            return Action::UseItem(None);
        }

        match &nearby_tributes {
            0 => {
                match tribute.attributes.health {
                    // health is low, rest
                    1..=20 => Action::Rest,
                    // health isn't great, hide
                    // unless sanity is also low, then move
                    21..=30 => {
                        if tribute.attributes.sanity > 20 && tribute.is_visible() {
                            Action::Hide
                        } else {
                            Action::Move(None)
                        }
                    }
                    // health is good, move
                    _ => {
                        // If the tribute has movement, move
                        match tribute.attributes.movement {
                            0 => Action::Rest,
                            _ => Action::Move(None),
                        }
                    }
                }
            }
            1..6 => {
                // Enemies are nearby, attack depending on health
                match tribute.attributes.health {
                    // health is low, hide
                    1..=15 => {
                        let stats = (
                            tribute.attributes.movement,
                            tribute.attributes.sanity,
                            tribute.attributes.is_hidden
                        );
                        match stats {
                            // low movement, ok sanity, visible
                            (..10, 35.., false) => { Action::Hide },
                            // low movement, low sanity, any visibility
                            (..10, ..35, _) => { Action::Attack },
                            // any movement, ok sanity, visible
                            (_, 35.., false) => { Action::Move(None) },
                            // any movement, low sanity, visible
                            (_, ..35, false) => { Action::Attack },
                            // any movement, any sanity, hidden
                            (_, _, true) => { Action::None },
                        }
                    }
                    // health isn't great, run away
                    16..=35 => {
                        if tribute.attributes.sanity > 20 {
                            Action::Move(None)
                        } else {
                            Action::Attack
                        }
                    }
                    // health is good, attack
                    _ => Action::Attack,
                }
            }
            _ => {
                // More than 5 enemies? Intelligence decides next move
                let sense: i32 = 100_i32
                    .saturating_sub(tribute.attributes.intelligence as i32)
                    .saturating_sub(tribute.attributes.sanity as i32);
                match sense {
                    // Smart enough to know better, moves
                    ..36 => Action::Move(None),
                    // Too dumb to know better, attacks
                    80.. => Action::Attack,
                    // Average intelligence, hides
                    _ => Action::Hide,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use super::*;
    use crate::items::Item;
    use rstest::{fixture, rstest};

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[rstest]
    fn decide_on_action_default(mut tribute: Tribute) {
        // If there are no enemies nearby, the tribute should move
        let mut rng = SmallRng::from_entropy();
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_low_health(mut tribute: Tribute) {
        // If the tribute has low health, they should rest
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_no_health(mut tribute: Tribute) {
        // If the tribute has no health, they should do nothing
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 0;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_no_movement_alone(mut tribute: Tribute) {
        // If the tribute has no movement and is alone, they should rest
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_no_movement_surrounded_low_health(mut tribute: Tribute) {
        // If the tribute has no movement and is not alone, they should hide
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.movement = 1;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 5, &mut rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_enemies(mut tribute: Tribute) {
        // If there are enemies nearby, the tribute should attack
        let mut rng = SmallRng::from_entropy();
        let action = tribute.brain.act(&tribute.clone(), 2, &mut rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_enemies_medium_health(mut tribute: Tribute) {
        // If there are enemies nearby, but the tribute is low on health
        // the tribute should hide
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 20;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_preferred_action(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn clear_preferred_action(mut tribute: Tribute) {
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        assert_eq!(tribute.brain.preferred_action, Some(Action::Rest));
        assert_eq!(tribute.brain.preferred_action_percentage, 1.0);

        tribute.brain.clear_preferred_action();
        assert_eq!(tribute.brain.preferred_action, None);
        assert_eq!(tribute.brain.preferred_action_percentage, 0.0);
    }

    #[rstest]
    fn prefer_to_use_item_if_available(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        let item = Item::new_random_consumable();
        tribute.items.push(item.clone());
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::UseItem(None));
    }

    #[rstest]
    fn prefer_to_hide_at_mid_health_and_visible(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 25;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn prefer_to_move_at_mid_health_and_low_sanity(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_alone_healthy_no_movement(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_movement_low_sanity(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 10;
        tribute.attributes.movement = 0;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_sanity(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 15;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_hidden_low_health(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.is_hidden = true;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_surrounded_ok_health_low_sanity(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_normal_sanity_and_intelligence(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.intelligence = 50;
        tribute.attributes.sanity = 50;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_low_sanity_and_intelligence(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.intelligence = 20;
        tribute.attributes.sanity = 20;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_no_sanity_and_intelligence(mut tribute: Tribute) {
        let mut rng = SmallRng::from_entropy();
        tribute.attributes.intelligence = 10;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut rng);
        assert_eq!(action, Action::Attack);
    }
}
