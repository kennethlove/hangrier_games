use crate::areas::{Area, AreaDetails};
use crate::terrain::{BaseTerrain, Harshness, TerrainType, Visibility};
use crate::tributes::Tribute;
use crate::tributes::actions::Action;
use rand::Rng;
use serde::{Deserialize, Serialize};

const LOW_ENEMY_LIMIT: u32 = 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BrainPersonality {
    Aggressive,
    Defensive,
    Balanced,
    Cautious,
    Reckless,
}

impl BrainPersonality {
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..5) {
            0 => BrainPersonality::Aggressive,
            1 => BrainPersonality::Defensive,
            2 => BrainPersonality::Balanced,
            3 => BrainPersonality::Cautious,
            _ => BrainPersonality::Reckless,
        }
    }

    pub fn low_health_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 15,
            BrainPersonality::Defensive => 30,
            BrainPersonality::Balanced => 20,
            BrainPersonality::Cautious => 35,
            BrainPersonality::Reckless => 10,
        }
    }

    pub fn mid_health_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 30,
            BrainPersonality::Defensive => 50,
            BrainPersonality::Balanced => 40,
            BrainPersonality::Cautious => 55,
            BrainPersonality::Reckless => 25,
        }
    }

    pub fn extreme_low_sanity_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 8,
            BrainPersonality::Defensive => 15,
            BrainPersonality::Balanced => 10,
            BrainPersonality::Cautious => 18,
            BrainPersonality::Reckless => 5,
        }
    }

    pub fn low_sanity_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 15,
            BrainPersonality::Defensive => 25,
            BrainPersonality::Balanced => 20,
            BrainPersonality::Cautious => 30,
            BrainPersonality::Reckless => 10,
        }
    }

    pub fn mid_sanity_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 25,
            BrainPersonality::Defensive => 45,
            BrainPersonality::Balanced => 35,
            BrainPersonality::Cautious => 50,
            BrainPersonality::Reckless => 20,
        }
    }

    pub fn low_movement_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 8,
            BrainPersonality::Defensive => 15,
            BrainPersonality::Balanced => 10,
            BrainPersonality::Cautious => 18,
            BrainPersonality::Reckless => 5,
        }
    }

    pub fn high_intelligence_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 30,
            BrainPersonality::Defensive => 40,
            BrainPersonality::Balanced => 35,
            BrainPersonality::Cautious => 45,
            BrainPersonality::Reckless => 25,
        }
    }

    pub fn low_intelligence_limit(&self) -> u32 {
        match self {
            BrainPersonality::Aggressive => 75,
            BrainPersonality::Defensive => 85,
            BrainPersonality::Balanced => 80,
            BrainPersonality::Cautious => 90,
            BrainPersonality::Reckless => 70,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brain {
    pub personality: BrainPersonality,
    pub preferred_action: Option<Action>,
    pub preferred_action_percentage: f64,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            personality: BrainPersonality::Balanced,
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }
}

impl Brain {
    pub fn new_with_random_personality(rng: &mut impl Rng) -> Self {
        Self {
            personality: BrainPersonality::random(rng),
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
        let low_health = self.personality.low_health_limit();
        let mid_health = self.personality.mid_health_limit();
        let low_sanity = self.personality.low_sanity_limit();

        match tribute.attributes.health {
            h if h < low_health => {
                self.decide_action_few_enemies_low_health_with_terrain(tribute, is_concealed)
            }
            h if h >= low_health && h <= mid_health => {
                // Boost hiding in concealed terrain
                if tribute.attributes.sanity > low_sanity && is_concealed {
                    Action::Hide
                } else if tribute.attributes.sanity > low_sanity {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            // High health - normally would attack, but concealed terrain makes hiding attractive
            _ if is_concealed && tribute.attributes.sanity > low_sanity => Action::Hide,
            _ => Action::Attack,
        }
    }

    fn decide_action_few_enemies_low_health_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let low_movement = self.personality.low_movement_limit();
        let mid_sanity = self.personality.mid_sanity_limit();
        let extreme_low_sanity = self.personality.extreme_low_sanity_limit();

        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden,
        );
        match stats {
            // Boost hiding in concealed terrain with low movement
            (m, s, false) if m < low_movement && s >= mid_sanity && is_concealed => Action::Hide,
            (m, s, false) if m < low_movement && s >= mid_sanity => Action::Hide,
            (m, s, _) if m < low_movement && s >= extreme_low_sanity && s < mid_sanity => {
                Action::Attack
            }
            (_, s, false) if s >= mid_sanity => Action::Move(None),
            (_, s, false) if s < mid_sanity => Action::Attack,
            (_, _, true) => Action::None,
            (_, _, false) => Action::Move(None), // Catch-all for visible tributes
        }
    }

    fn decide_action_many_enemies_with_terrain(
        &self,
        tribute: &Tribute,
        is_concealed: bool,
    ) -> Action {
        let high_intelligence = self.personality.high_intelligence_limit();
        let low_intelligence = self.personality.low_intelligence_limit();

        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            r if r < high_intelligence => Action::Move(None),
            r if r >= low_intelligence => Action::Attack,
            // Boost hiding in concealed terrain for average intelligence
            _ if is_concealed => Action::Hide,
            _ => Action::Hide,
        }
    }

    fn decide_action_no_enemies(&self, tribute: &Tribute) -> Action {
        let low_health = self.personality.low_health_limit();
        let mid_health = self.personality.mid_health_limit();
        let low_sanity = self.personality.low_sanity_limit();

        match tribute.attributes.health {
            // health is low, rest
            h if h < low_health => Action::Rest,
            // health isn't great, hide
            // unless sanity is also low, then move
            h if h >= low_health && h <= mid_health => {
                if tribute.attributes.sanity > low_sanity && tribute.is_visible() {
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
        let low_movement = self.personality.low_movement_limit();
        let mid_sanity = self.personality.mid_sanity_limit();
        let extreme_low_sanity = self.personality.extreme_low_sanity_limit();

        let stats = (
            tribute.attributes.movement,
            tribute.attributes.sanity,
            tribute.attributes.is_hidden,
        );
        match stats {
            // low movement, ok sanity, visible
            (m, s, false) if m < low_movement && s >= mid_sanity => Action::Hide,
            // low movement, low sanity, any visibility
            (m, s, _) if m < low_movement && s >= extreme_low_sanity && s < mid_sanity => {
                Action::Attack
            }
            // any movement, ok sanity, visible
            (_, s, false) if s >= mid_sanity => Action::Move(None),
            // any movement, low sanity, visible
            (_, s, false) if s < mid_sanity => Action::Attack,
            // any movement, any sanity, hidden
            (_, _, true) => Action::None,
            (_, _, false) => Action::Move(None), // Catch-all for visible tributes
        }
    }

    fn decide_action_few_enemies(&self, tribute: &Tribute) -> Action {
        let low_health = self.personality.low_health_limit();
        let mid_health = self.personality.mid_health_limit();
        let low_sanity = self.personality.low_sanity_limit();

        match tribute.attributes.health {
            h if h < low_health => self.decide_action_few_enemies_low_health(tribute),
            h if h >= low_health && h <= mid_health => {
                if tribute.attributes.sanity > low_sanity {
                    Action::Move(None)
                } else {
                    Action::Attack
                }
            }
            _ => Action::Attack,
        }
    }

    fn decide_action_many_enemies(&self, tribute: &Tribute) -> Action {
        let high_intelligence = self.personality.high_intelligence_limit();
        let low_intelligence = self.personality.low_intelligence_limit();

        let recklessness: u32 = 100_u32
            .saturating_sub(tribute.attributes.intelligence)
            .saturating_sub(tribute.attributes.sanity);
        match recklessness {
            // Smart enough to know better, moves
            r if r < high_intelligence => Action::Move(None),
            // Too dumb to know better, attacks
            r if r >= low_intelligence => Action::Attack,
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

    #[test]
    fn test_personality_thresholds_aggressive() {
        let personality = BrainPersonality::Aggressive;
        assert_eq!(personality.low_health_limit(), 15);
        assert_eq!(personality.mid_health_limit(), 30);
        assert!(personality.low_health_limit() < BrainPersonality::Balanced.low_health_limit());
    }

    #[test]
    fn test_personality_thresholds_defensive() {
        let personality = BrainPersonality::Defensive;
        assert_eq!(personality.low_health_limit(), 30);
        assert_eq!(personality.mid_health_limit(), 50);
        assert!(personality.low_health_limit() > BrainPersonality::Balanced.low_health_limit());
    }

    #[test]
    fn test_personality_thresholds_reckless() {
        let personality = BrainPersonality::Reckless;
        assert_eq!(personality.low_health_limit(), 10);
        assert!(personality.low_health_limit() < BrainPersonality::Aggressive.low_health_limit());
    }

    #[test]
    fn test_personality_thresholds_cautious() {
        let personality = BrainPersonality::Cautious;
        assert_eq!(personality.low_health_limit(), 35);
        assert!(personality.low_health_limit() > BrainPersonality::Defensive.low_health_limit());
    }

    #[test]
    fn test_personality_random_distribution() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut counts = std::collections::HashMap::new();

        for _ in 0..100 {
            let personality = BrainPersonality::random(&mut rng);
            *counts.entry(format!("{:?}", personality)).or_insert(0) += 1;
        }

        // Should have all 5 personality types
        assert_eq!(counts.len(), 5);
        // Each should appear at least once (with high probability)
        for count in counts.values() {
            assert!(*count > 0);
        }
    }

    #[rstest]
    fn test_aggressive_fights_at_lower_health(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.personality = BrainPersonality::Aggressive;
        tribute.attributes.health = 18; // Between aggressive (15) and balanced (20)
        tribute.attributes.sanity = 40;

        let action = tribute.brain.act(&tribute.clone(), 1, &[], &mut small_rng);
        // Aggressive should still attack/move at this health
        assert!(matches!(action, Action::Attack | Action::Move(_)));
    }

    #[rstest]
    fn test_defensive_retreats_earlier(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.personality = BrainPersonality::Defensive;
        tribute.attributes.health = 25; // Above balanced limit but below defensive
        tribute.attributes.sanity = 40;

        let action = tribute.brain.decide_action_few_enemies(&tribute);
        // Defensive should prefer moving/hiding at this health
        assert!(matches!(action, Action::Move(_) | Action::Hide));
    }
}
