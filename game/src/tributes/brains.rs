use crate::tributes::actions::Action;
use crate::tributes::Tribute;
use rand::Rng;
use serde::{Deserialize, Serialize};

const LOW_ENEMY_LIMIT: u32 = 6;
const LOW_HEALTH_LIMIT: u32 = 20;
const MID_HEALTH_LIMIT: u32 = 40;
const EXTREME_LOW_SANITY_LIMIT: u32 = 10;
const LOW_SANITY_LIMIT: u32 = 20;
const MID_SANITY_LIMIT: u32 = 35;
const LOW_MOVEMENT_LIMIT: u32 = 10;
const HIGH_INTELLIGENCE_LIMIT: u32 = 35;
const LOW_INTELLIGENCE_LIMIT: u32 = 80;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    pub preferred_action: Option<Action>,
    pub preferred_action_percentage: f64,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
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

    /// The AI for a tribute. Automatic decisions based on the current state of the tribute.
    pub fn act(&mut self, tribute: &Tribute, nearby_tributes: u32, mut rng: impl Rng) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }

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
            0 => self.decide_action_no_enemies(tribute),
            1..LOW_ENEMY_LIMIT => self.decide_action_few_enemies(tribute),
            _ => self.decide_action_many_enemies(tribute),
        }
    }

    fn decide_action_no_enemies(&self, tribute: &Tribute) -> Action {
        match tribute.attributes.health {
            // health is low, rest
            1..LOW_HEALTH_LIMIT => Action::Rest,
            // health isn't great, hide
            // unless sanity is also low, then move
            LOW_HEALTH_LIMIT..=MID_HEALTH_LIMIT => {
                if tribute.attributes.sanity > LOW_SANITY_LIMIT && tribute.is_visible() {
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

    fn decide_action_few_enemies_low_health(&self, tribute: &Tribute) -> Action {
        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden
        );
        match stats {
            // low movement, ok sanity, visible
            (..LOW_MOVEMENT_LIMIT, MID_SANITY_LIMIT.., false) => { Action::Hide },
            // low movement, low sanity, any visibility
            (..LOW_MOVEMENT_LIMIT, EXTREME_LOW_SANITY_LIMIT..MID_SANITY_LIMIT, _) => { Action::Attack },
            // any movement, ok sanity, visible
            (_, MID_SANITY_LIMIT.., false) => { Action::Move(None) },
            // any movement, low sanity, visible
            (_, ..MID_SANITY_LIMIT, false) => { Action::Attack },
            // any movement, any sanity, hidden
            (_, _, true) => { Action::None },
        }
    }

    fn decide_action_few_enemies(&self, tribute: &Tribute) -> Action {
        match tribute.attributes.health {
            1..LOW_HEALTH_LIMIT => self.decide_action_few_enemies_low_health(tribute),
            LOW_HEALTH_LIMIT..=MID_HEALTH_LIMIT => {
                if tribute.attributes.sanity > LOW_SANITY_LIMIT {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            _ => Action::Attack,
        }
    }

    fn decide_action_many_enemies(&self, tribute: &Tribute) -> Action {
        let recklessness: u32 = 100_u32.saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            // Smart enough to know better, moves
            ..HIGH_INTELLIGENCE_LIMIT => Action::Move(None),
            // Too dumb to know better, attacks
            LOW_INTELLIGENCE_LIMIT.. => Action::Attack,
            // Average intelligence, hides
            _ => Action::Hide,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::items::Item;
    use crate::tributes::actions::Action;
    use crate::tributes::Tribute;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
    use rstest::{fixture, rstest};

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::from_entropy()
    }

    #[rstest]
    fn decide_on_action_default(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If there are no enemies nearby, the tribute should move
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has low health, they should rest
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_no_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no health, they should do nothing
        tribute.attributes.health = 0;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_no_movement_alone(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no movement and is alone, they should rest
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_no_movement_surrounded_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no movement and is not alone, they should hide
        tribute.attributes.movement = 1;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 5, &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_enemies(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, the tribute should attack
        let action = tribute.brain.act(&tribute.clone(), 2, &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_enemies_medium_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, but the tribute is low on health
        // the tribute should hide
        tribute.attributes.health = 20;
        let action = tribute.brain.act(&tribute.clone(), 2, &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_preferred_action(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
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
    fn prefer_to_use_item_if_available(mut tribute: Tribute, mut small_rng: SmallRng) {
        let item = Item::new_random_consumable();
        tribute.items.push(item.clone());
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::UseItem(None));
    }

    #[rstest]
    fn prefer_to_hide_at_mid_health_and_visible(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn prefer_to_move_at_mid_health_and_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_alone_healthy_no_movement(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_movement_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 10;
        tribute.attributes.movement = 0;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 15;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_hidden_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.is_hidden = true;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_surrounded_ok_health_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_normal_sanity_and_intelligence(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.intelligence = 50;
        tribute.attributes.sanity = 50;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_low_sanity_and_intelligence(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.intelligence = 20;
        tribute.attributes.sanity = 20;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_no_sanity_and_intelligence(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.intelligence = 10;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 6, &mut small_rng);
        assert_eq!(action, Action::Attack);
    }
}
