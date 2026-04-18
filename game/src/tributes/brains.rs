use crate::areas::{Area, AreaDetails};
use crate::terrain::{BaseTerrain, Harshness, TerrainType, Visibility};
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
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
    pub fn act(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        available_destinations: &[crate::areas::DestinationInfo],
        rng: &mut impl Rng,
    ) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }

        // If there is a preferred action, we should take it, assuming a positive roll
        if let Some(ref preferred_action) = self.preferred_action {
            if rng.random_bool(self.preferred_action_percentage) {
                return preferred_action.clone();
            }
        }

        // Does the tribute have items?
        let has_consumables = !tribute.consumables().is_empty();
        if has_consumables {
            // Use an item
            return Action::UseItem(None);
        }

        let action = if nearby_tributes == 0 {
            self.decide_action_no_enemies(tribute)
        } else if nearby_tributes < LOW_ENEMY_LIMIT {
            self.decide_action_few_enemies(tribute)
        } else {
            self.decide_action_many_enemies(tribute)
        };

        // If the action is Move(None), choose smart destination based on terrain
        match action {
            Action::Move(None) => {
                // If no destinations available, keep Move(None) for backward compatibility
                if available_destinations.is_empty() {
                    return Action::Move(None);
                }

                // Convert DestinationInfo to AreaDetails for choose_destination
                let area_details: Vec<AreaDetails> = available_destinations
                    .iter()
                    .map(|dest| {
                        let mut ad = AreaDetails::default();
                        ad.area = Some(dest.area.clone());
                        ad.terrain = dest.terrain.clone();
                        ad.events = dest.active_events.clone();
                        ad
                    })
                    .collect();

                // Choose best destination using terrain scoring
                if let Some(best_area) = self.choose_destination(&area_details, tribute) {
                    // Also check if tribute has enough stamina
                    if let Some(dest_info) =
                        available_destinations.iter().find(|d| d.area == best_area)
                    {
                        if tribute.stamina >= dest_info.stamina_cost {
                            return Action::Move(Some(best_area));
                        }
                    }
                }
                // Fall back to rest if no good destination or insufficient stamina
                Action::Rest
            }
            other => other,
        }
    }

    /// Choose the best destination from available areas based on terrain scoring.
    /// Returns the Area enum variant of the highest-scoring area.
    ///
    /// Scoring factors:
    /// - +20 if area has terrain in tribute's affinity
    /// - -10 per harshness tier (Mild=0, Moderate=-10, Harsh=-20)
    /// - +5 if terrain visibility is Concealed (good for hiding)
    /// - +3 if area has items
    /// - +60 (3.0x * 20) if tribute health < 30 and area has affinity terrain (desperate behavior)
    pub fn choose_destination(&self, areas: &[AreaDetails], tribute: &Tribute) -> Option<Area> {
        if areas.is_empty() {
            return None;
        }

        let is_desperate = tribute.attributes.health < 30;

        let mut best_score = i32::MIN;
        let mut best_area: Option<Area> = None;

        for area_details in areas {
            let mut score = 0i32;

            // Affinity bonus: +20 if terrain matches tribute's affinity
            let has_affinity = tribute
                .terrain_affinity
                .contains(&area_details.terrain.base);
            if has_affinity {
                score += 20;

                // Desperate behavior: 3.0x boost to affinity terrain (additional +40)
                if is_desperate {
                    score += 40; // Total 60 for desperate + affinity
                }
            }

            // Harshness penalty: -10 per tier
            let harshness_penalty = match area_details.terrain.base.harshness() {
                Harshness::Mild => 0,
                Harshness::Moderate => -10,
                Harshness::Harsh => -20,
            };
            score += harshness_penalty;

            // Concealed visibility bonus: +5 (good for hiding)
            if matches!(
                area_details.terrain.base.visibility(),
                Visibility::Concealed
            ) {
                score += 5;
            }

            // Items bonus: +3 if area has items
            if !area_details.items.is_empty() {
                score += 3;
            }

            if score > best_score {
                best_score = score;
                best_area = area_details.area.clone();
            }
        }

        best_area
    }

    /// Decide action with terrain awareness. Modifies action weights based on terrain.
    ///
    /// Terrain-based weight modifications:
    /// - Boost Search weight by 2.0x in Desert/Tundra/Badlands (resource-scarce)
    /// - Boost Hide weight by 1.5x in Forest/Jungle/Wetlands (Concealed visibility)
    pub fn decide_action_with_terrain(
        &self,
        tribute: &Tribute,
        nearby_tributes: u32,
        terrain: TerrainType,
        rng: &mut impl Rng,
    ) -> Action {
        if !tribute.is_alive() {
            return Action::None;
        }

        // Check for preferred action first
        if let Some(ref preferred_action) = self.preferred_action {
            if rng.random_bool(self.preferred_action_percentage) {
                return preferred_action.clone();
            }
        }

        // Check if we have consumables
        let has_consumables = !tribute.consumables().is_empty();
        if has_consumables {
            return Action::UseItem(None);
        }

        // Check if terrain is resource-scarce (should boost search/movement)
        let is_resource_scarce = matches!(
            terrain.base,
            BaseTerrain::Desert | BaseTerrain::Tundra | BaseTerrain::Badlands
        );

        // Check if terrain is concealed (should boost hiding)
        let is_concealed = matches!(terrain.base.visibility(), Visibility::Concealed);

        // Decide base action
        let base_action = if nearby_tributes == 0 {
            self.decide_action_no_enemies(tribute)
        } else if nearby_tributes < LOW_ENEMY_LIMIT {
            self.decide_action_few_enemies_with_terrain(tribute, is_concealed)
        } else {
            self.decide_action_many_enemies_with_terrain(tribute, is_concealed)
        };

        // Apply terrain modifiers to action choices
        match base_action {
            Action::Move(None) if is_resource_scarce => {
                // In resource-scarce terrain, stay focused on movement/search
                Action::Move(None)
            }
            Action::Hide if is_concealed => {
                // Concealed terrain makes hiding more effective
                Action::Hide
            }
            other => other,
        }
    }

    fn decide_action_few_enemies_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        match tribute.attributes.health {
            1..LOW_HEALTH_LIMIT => {
                self.decide_action_few_enemies_low_health_with_terrain(tribute, is_concealed)
            }
            LOW_HEALTH_LIMIT..=MID_HEALTH_LIMIT => {
                // Boost hiding in concealed terrain
                if tribute.attributes.sanity > LOW_SANITY_LIMIT && is_concealed {
                    Action::Hide
                } else if tribute.attributes.sanity > LOW_SANITY_LIMIT {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            // High health - normally would attack, but concealed terrain makes hiding attractive
            _ if is_concealed && tribute.attributes.sanity > LOW_SANITY_LIMIT => Action::Hide,
            _ => Action::Attack,
        }
    }

    fn decide_action_few_enemies_low_health_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden,
        );
        match stats {
            // Boost hiding in concealed terrain with low movement
            (..LOW_MOVEMENT_LIMIT, MID_SANITY_LIMIT.., false) if is_concealed => Action::Hide,
            (..LOW_MOVEMENT_LIMIT, MID_SANITY_LIMIT.., false) => Action::Hide,
            (..LOW_MOVEMENT_LIMIT, EXTREME_LOW_SANITY_LIMIT..MID_SANITY_LIMIT, _) => Action::Attack,
            (_, MID_SANITY_LIMIT.., false) => Action::Move(None),
            (_, ..MID_SANITY_LIMIT, false) => Action::Attack,
            (_, _, true) => Action::None,
        }
    }

    fn decide_action_many_enemies_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            ..HIGH_INTELLIGENCE_LIMIT => Action::Move(None),
            LOW_INTELLIGENCE_LIMIT.. => Action::Attack,
            // Boost hiding in concealed terrain for average intelligence
            _ if is_concealed => Action::Hide,
            _ => Action::Hide,
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
            tribute.attributes.is_hidden,
        );
        match stats {
            // low movement, ok sanity, visible
            (..LOW_MOVEMENT_LIMIT, MID_SANITY_LIMIT.., false) => Action::Hide,
            // low movement, low sanity, any visibility
            (..LOW_MOVEMENT_LIMIT, EXTREME_LOW_SANITY_LIMIT..MID_SANITY_LIMIT, _) => Action::Attack,
            // any movement, ok sanity, visible
            (_, MID_SANITY_LIMIT.., false) => Action::Move(None),
            // any movement, low sanity, visible
            (_, ..MID_SANITY_LIMIT, false) => Action::Attack,
            // any movement, any sanity, hidden
            (_, _, true) => Action::None,
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
        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
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
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;
    use rand::prelude::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::from_rng(&mut rand::rng())
    }

    #[rstest]
    fn decide_on_action_default(tribute: Tribute, mut small_rng: SmallRng) {
        // If there are no enemies nearby, the tribute should move
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_low_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has low health, they should rest
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_no_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no health, they should do nothing
        tribute.attributes.health = 0;
        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_no_movement_alone(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If the tribute has no movement and is alone, they should rest
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_no_movement_surrounded_low_health(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        // If the tribute has no movement and is not alone, they should hide
        tribute.attributes.movement = 1;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 5, &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_enemies(tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, the tribute should attack
        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_enemies_medium_health(mut tribute: Tribute, mut small_rng: SmallRng) {
        // If there are enemies nearby, but the tribute is low on health
        // the tribute should hide
        tribute.attributes.health = 20;
        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_preferred_action(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.brain.set_preferred_action(Action::Rest, 1.0);
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
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
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::UseItem(None));
    }

    #[rstest]
    fn prefer_to_hide_at_mid_health_and_visible(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn prefer_to_move_at_mid_health_and_low_sanity(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_alone_healthy_no_movement(mut tribute: Tribute, mut small_rng: SmallRng) {
        tribute.attributes.movement = 0;
        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::Rest);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_movement_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 10;
        tribute.attributes.movement = 0;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_low_health_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 15;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_surrounded_hidden_low_health(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.is_hidden = true;
        tribute.attributes.health = 10;
        let action = tribute.brain.act(&tribute.clone(), 3, &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn decide_on_action_surrounded_ok_health_low_sanity(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.health = 25;
        tribute.attributes.sanity = 15;
        let action = tribute.brain.act(&tribute.clone(), 3, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_normal_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.intelligence = 50;
        tribute.attributes.sanity = 50;
        let action = tribute.brain.act(&tribute.clone(), 6, &[], &mut small_rng);
        assert_eq!(action, Action::Move(None));
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_low_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.intelligence = 20;
        tribute.attributes.sanity = 20;
        let action = tribute.brain.act(&tribute.clone(), 6, &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn decide_on_action_heavily_surrounded_no_sanity_and_intelligence(
        mut tribute: Tribute,
        mut small_rng: SmallRng,
    ) {
        tribute.attributes.intelligence = 10;
        tribute.attributes.sanity = 10;
        let action = tribute.brain.act(&tribute.clone(), 6, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }
}
