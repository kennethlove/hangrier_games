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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PsychoticBreakType {
    Berserk,         // Attack anyone nearby
    Paranoid,        // Flee and hide constantly
    Catatonic,       // Skip turns (Action::None)
    SelfDestructive, // Seek danger, ignore health
}

/// Cached personality thresholds generated at tribute creation with individual variance
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonalityThresholds {
    pub low_health: u32,
    pub mid_health: u32,
    pub extreme_low_sanity: u32,
    pub low_sanity: u32,
    pub mid_sanity: u32,
    pub low_movement: u32,
    pub high_intelligence: u32,
    pub low_intelligence: u32,
    pub psychotic_break_threshold: u32, // Sanity level that triggers break
}

impl BrainPersonality {
    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..5) {
            0 => BrainPersonality::Aggressive,
            1 => BrainPersonality::Defensive,
            2 => BrainPersonality::Balanced,
            3 => BrainPersonality::Cautious,
            _ => BrainPersonality::Reckless,
        }
    }

    /// Generate thresholds with ±20% variance for individual differences
    pub fn generate_thresholds(&self, rng: &mut impl Rng) -> PersonalityThresholds {
        fn apply_variance(base: u32, rng: &mut impl Rng) -> u32 {
            let variance = rng.random_range(-0.2..=0.2);
            ((base as f32) * (1.0 + variance)).max(1.0) as u32
        }

        // Base values per personality
        let (base_low_health, base_mid_health) = match self {
            BrainPersonality::Aggressive => (15, 30),
            BrainPersonality::Defensive => (30, 50),
            BrainPersonality::Balanced => (20, 40),
            BrainPersonality::Cautious => (35, 55),
            BrainPersonality::Reckless => (10, 25),
        };

        let (base_extreme_low_sanity, base_low_sanity, base_mid_sanity) = match self {
            BrainPersonality::Aggressive => (8, 15, 25),
            BrainPersonality::Defensive => (15, 25, 45),
            BrainPersonality::Balanced => (10, 20, 35),
            BrainPersonality::Cautious => (18, 30, 50),
            BrainPersonality::Reckless => (5, 10, 20),
        };

        let base_low_movement = match self {
            BrainPersonality::Aggressive => 8,
            BrainPersonality::Defensive => 15,
            BrainPersonality::Balanced => 10,
            BrainPersonality::Cautious => 18,
            BrainPersonality::Reckless => 5,
        };

        let (base_high_intelligence, base_low_intelligence) = match self {
            BrainPersonality::Aggressive => (30, 75),
            BrainPersonality::Defensive => (40, 85),
            BrainPersonality::Balanced => (35, 80),
            BrainPersonality::Cautious => (45, 90),
            BrainPersonality::Reckless => (25, 70),
        };

        // Psychotic break threshold varies by personality
        // Reckless/Aggressive more prone (higher threshold = breaks sooner)
        // Cautious/Defensive more resilient (lower threshold = breaks later)
        let base_break_threshold = match self {
            BrainPersonality::Reckless => 12, // Breaks easily
            BrainPersonality::Aggressive => 10,
            BrainPersonality::Balanced => 8,
            BrainPersonality::Defensive => 6,
            BrainPersonality::Cautious => 5, // Very resilient
        };

        PersonalityThresholds {
            low_health: apply_variance(base_low_health, rng),
            mid_health: apply_variance(base_mid_health, rng),
            extreme_low_sanity: apply_variance(base_extreme_low_sanity, rng),
            low_sanity: apply_variance(base_low_sanity, rng),
            mid_sanity: apply_variance(base_mid_sanity, rng),
            low_movement: apply_variance(base_low_movement, rng),
            high_intelligence: apply_variance(base_high_intelligence, rng),
            low_intelligence: apply_variance(base_low_intelligence, rng),
            psychotic_break_threshold: apply_variance(base_break_threshold, rng),
        }
    }

    // Keep original methods for backward compatibility/reference
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
    pub thresholds: PersonalityThresholds,
    pub psychotic_break: Option<PsychoticBreakType>,
    pub preferred_action: Option<Action>,
    pub preferred_action_percentage: f64,
}

impl Default for Brain {
    fn default() -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0); // Deterministic default
        Self {
            personality: BrainPersonality::Balanced,
            thresholds: BrainPersonality::Balanced.generate_thresholds(&mut rng),
            psychotic_break: None,
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }
}

impl Brain {
    pub fn new_with_random_personality(rng: &mut impl Rng) -> Self {
        let personality = BrainPersonality::random(rng);
        let thresholds = personality.generate_thresholds(rng);
        Self {
            personality,
            thresholds,
            psychotic_break: None,
            preferred_action: None,
            preferred_action_percentage: 0.0,
        }
    }

    /// Check if tribute should have a psychotic break
    ///
    /// Break type is randomly determined, not tied to personality.
    /// This allows for emergent behavior - a cautious tribute going berserk
    /// can be just as dramatic as an aggressive one becoming catatonic.
    pub fn check_psychotic_break(&mut self, current_sanity: u32, rng: &mut impl Rng) {
        // Already broken, stay broken
        if self.psychotic_break.is_some() {
            return;
        }

        // Check if sanity dropped below threshold
        if current_sanity <= self.thresholds.psychotic_break_threshold {
            // Randomly select break type (equal probability)
            let break_type = match rng.random_range(0..4) {
                0 => PsychoticBreakType::Berserk,
                1 => PsychoticBreakType::Paranoid,
                2 => PsychoticBreakType::Catatonic,
                _ => PsychoticBreakType::SelfDestructive,
            };

            self.psychotic_break = Some(break_type);
        }
    }

    /// Can recover from break if sanity recovers significantly
    pub fn check_recovery(&mut self, current_sanity: u32) {
        if self.psychotic_break.is_some() {
            // Need to recover to 20+ sanity above break threshold to recover
            if current_sanity >= self.thresholds.psychotic_break_threshold + 20 {
                self.psychotic_break = None;
            }
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

        // Psychotic break overrides normal behavior
        if let Some(ref break_type) = self.psychotic_break {
            return match break_type {
                PsychoticBreakType::Berserk => {
                    // Attack if enemies nearby, otherwise move to find enemies
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
                PsychoticBreakType::Paranoid => {
                    // Always flee and hide
                    if tribute.attributes.is_hidden {
                        Action::None // Stay hidden
                    } else {
                        Action::Hide
                    }
                }
                PsychoticBreakType::Catatonic => {
                    // Do nothing
                    Action::None
                }
                PsychoticBreakType::SelfDestructive => {
                    // Seek danger: attack if enemies present, move otherwise
                    // Ignore health completely
                    if nearby_tributes > 0 {
                        Action::Attack
                    } else {
                        Action::Move(None)
                    }
                }
            };
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
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

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
        let low_movement = self.thresholds.low_movement;
        let mid_sanity = self.thresholds.mid_sanity;
        let extreme_low_sanity = self.thresholds.extreme_low_sanity;

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
        let high_intelligence = self.thresholds.high_intelligence;
        let low_intelligence = self.thresholds.low_intelligence;

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
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

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
        let low_movement = self.thresholds.low_movement;
        let mid_sanity = self.thresholds.mid_sanity;
        let extreme_low_sanity = self.thresholds.extreme_low_sanity;

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
        let low_health = self.thresholds.low_health;
        let mid_health = self.thresholds.mid_health;
        let low_sanity = self.thresholds.low_sanity;

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
        let high_intelligence = self.thresholds.high_intelligence;
        let low_intelligence = self.thresholds.low_intelligence;

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
    use super::*;
    use crate::items::Item;
    use crate::tributes::Tribute;
    use crate::tributes::actions::Action;
    use rand::prelude::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn tribute() -> Tribute {
        // Build a fully-deterministic Tribute for AI tests:
        //   - Brain::default() = Balanced personality with seed=0 thresholds
        //     (low_health=18, mid_health=38, low_sanity=16, mid_sanity=34,
        //     low_movement=8, high_intelligence=40, low_intelligence=91)
        //   - Attributes::default() = maxed-out attributes
        // Tribute::new() randomizes both, so override after construction.
        let mut tribute = Tribute::new("Katniss".to_string(), None, None);
        tribute.brain = Brain::default();
        tribute.attributes = crate::tributes::Attributes::default();
        tribute
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
        // recklessness = 100 - intelligence - sanity must reach low_intelligence
        // threshold (~91 for Balanced after variance) for the Attack branch.
        tribute.attributes.intelligence = 5;
        tribute.attributes.sanity = 0;
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
        // Regenerate thresholds for the new personality (otherwise stale
        // thresholds from random construction are used). Seed=0 gives
        // Defensive low_health ~28-32, mid_health ~45-55.
        tribute.brain.thresholds =
            BrainPersonality::Defensive.generate_thresholds(&mut SmallRng::seed_from_u64(0));
        tribute.attributes.health = 25; // Below defensive low_health
        // Sanity must be safely above Defensive mid_sanity (base 45, +20%
        // variance ceiling = 54) so the visible/ok-sanity arm of
        // decide_action_few_enemies_low_health returns Move, not Attack.
        tribute.attributes.sanity = 60;

        let action = tribute.brain.decide_action_few_enemies(&tribute);
        // Defensive should prefer moving/hiding at this health
        assert!(matches!(action, Action::Move(_) | Action::Hide));
    }

    #[rstest]
    fn test_threshold_variance_within_range(mut small_rng: SmallRng) {
        let personality = BrainPersonality::Balanced;
        let thresholds = personality.generate_thresholds(&mut small_rng);

        // Base value for Balanced is 20, variance is ±20% = 16-24
        assert!(thresholds.low_health >= 16 && thresholds.low_health <= 24);
        // Base value for Balanced is 40, variance is ±20% = 32-48
        assert!(thresholds.mid_health >= 32 && thresholds.mid_health <= 48);
    }

    #[rstest]
    fn test_threshold_variance_differs_between_tributes(mut small_rng: SmallRng) {
        let personality = BrainPersonality::Balanced;
        let thresholds1 = personality.generate_thresholds(&mut small_rng);
        let thresholds2 = personality.generate_thresholds(&mut small_rng);

        // With high probability, at least one threshold should differ
        // (not guaranteed due to randomness, but very likely)
        let differs = thresholds1.low_health != thresholds2.low_health
            || thresholds1.mid_health != thresholds2.mid_health
            || thresholds1.low_sanity != thresholds2.low_sanity;

        // This test may occasionally fail due to random chance, but is very unlikely
        assert!(differs);
    }

    #[rstest]
    fn test_psychotic_break_triggers_at_low_sanity(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.attributes.sanity = 3; // Below typical break threshold

        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);

        assert!(tribute.brain.psychotic_break.is_some());
    }

    #[rstest]
    fn test_psychotic_break_doesnt_trigger_at_normal_sanity(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.attributes.sanity = 50; // Well above break threshold

        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);

        assert!(tribute.brain.psychotic_break.is_none());
    }

    #[rstest]
    fn test_psychotic_break_recovery(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        // Use deterministic Brain so psychotic_break_threshold is fixed
        // (Balanced base = 7) and the +20 recovery margin is predictable.
        tribute.brain = Brain::default();
        tribute.attributes.sanity = 3;

        // Trigger break
        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);
        assert!(tribute.brain.psychotic_break.is_some());

        // Sanity recovers significantly (needs to be 20+ above threshold)
        tribute.attributes.sanity = 30;
        tribute.brain.check_recovery(tribute.attributes.sanity);

        assert!(tribute.brain.psychotic_break.is_none());
    }

    #[rstest]
    fn test_psychotic_break_no_recovery_insufficient_sanity(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        // Deterministic Brain: Balanced psychotic_break_threshold = 7,
        // recovery requires sanity >= 27. sanity = 15 is below that.
        tribute.brain = Brain::default();
        tribute.attributes.sanity = 3;

        // Trigger break
        tribute
            .brain
            .check_psychotic_break(tribute.attributes.sanity, &mut small_rng);
        assert!(tribute.brain.psychotic_break.is_some());

        // Sanity recovers but not enough
        tribute.attributes.sanity = 15;
        tribute.brain.check_recovery(tribute.attributes.sanity);

        // Should still be broken
        assert!(tribute.brain.psychotic_break.is_some());
    }

    #[rstest]
    fn test_berserk_break_attacks(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Berserk);

        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::Attack);
    }

    #[rstest]
    fn test_paranoid_break_hides(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Paranoid);

        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        assert_eq!(action, Action::Hide);
    }

    #[rstest]
    fn test_catatonic_break_does_nothing(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::Catatonic);

        let action = tribute.brain.act(&tribute.clone(), 0, &[], &mut small_rng);
        assert_eq!(action, Action::None);
    }

    #[rstest]
    fn test_self_destructive_break_attacks(mut small_rng: SmallRng) {
        let mut tribute = Tribute::default();
        tribute.brain.psychotic_break = Some(PsychoticBreakType::SelfDestructive);
        tribute.attributes.health = 5; // Very low health - normally would rest/hide

        let action = tribute.brain.act(&tribute.clone(), 2, &[], &mut small_rng);
        // Self-destructive ignores health and attacks
        assert_eq!(action, Action::Attack);
    }
}
