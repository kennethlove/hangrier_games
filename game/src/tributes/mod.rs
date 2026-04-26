pub mod actions;
pub mod alliances;
pub mod brains;
pub mod combat;
pub mod events;
pub mod inventory;
pub mod lifecycle;
pub mod movement;
pub mod statuses;
pub mod traits;

// Re-export key items from sub-modules
pub use combat::{attack_contest, update_stats};
pub use movement::TravelResult;

use crate::areas::{Area, AreaDetails};
use crate::items::{Item, OwnsItems};
use crate::output::GameOutput;
use crate::tributes::events::TributeEvent;
use actions::{Action, AttackOutcome};
use brains::Brain;
use fake::Fake;
use fake::faker::name::raw::*;
use fake::locales::*;
use rand::prelude::*;
use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};
use statuses::TributeStatus;
use uuid::Uuid;

/// Consts
const SANITY_BREAK_LEVEL: u32 = 9;
const LOYALTY_BREAK_LEVEL: f64 = 0.25;

#[derive(Clone, Debug)]
pub struct ActionSuggestion {
    pub action: Action,
    pub probability: Option<f64>,
}

#[derive(Debug)]
pub struct EnvironmentContext<'a> {
    pub is_day: bool,
    pub area_details: &'a mut AreaDetails,
    pub closed_areas: &'a [Area],
    pub available_destinations: Vec<crate::areas::DestinationInfo>,
}

#[derive(Clone, Debug)]
pub struct EncounterContext {
    pub nearby_tributes_count: u32,
    pub potential_targets: Vec<Tribute>,
    pub total_living_tributes: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tribute {
    /// Identifier
    pub identifier: String,
    /// Stable typed UUID. Mirrors `identifier` for callers that want a
    /// non-stringly-typed key (alliance graph, betrayal events).
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    /// Where are they?
    pub area: Area,
    /// What is their current status?
    pub status: TributeStatus,
    /// This is their thinker
    #[serde(skip)]
    pub brain: Brain,
    /// How they present themselves to the real world
    pub avatar: Option<String>,
    /// Who created them in the real world
    #[serde(rename = "player_name")]
    pub human_player_name: Option<String>,
    /// What they like to go by
    pub name: String,
    /// Where they're from
    pub district: u32,
    /// Stats like fights won
    pub statistics: Statistics,
    /// Attributes like health
    pub attributes: Attributes,
    /// Items the tribute owns
    #[serde(default)]
    pub items: Vec<Item>,
    /// Events that have happened to the tribute
    #[serde(default)]
    pub events: Vec<TributeEvent>,
    #[serde(default)]
    pub editable: bool,
    /// Terrain types this tribute is familiar with
    #[serde(default)]
    pub terrain_affinity: Vec<crate::terrain::BaseTerrain>,
    /// Current stamina for actions
    pub stamina: u32,
    /// Maximum stamina capacity
    pub max_stamina: u32,
    /// Personality/behavior trait set. Replaces `BrainPersonality`.
    /// A tribute with zero traits behaves as the old `Balanced` baseline.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<traits::Trait>,
    /// Pair-wise alliance graph. Symmetric: when A allies with B, both
    /// `allies` lists gain the other. Capped at `MAX_ALLIES`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allies: Vec<Uuid>,
    /// Turn counter for the Treacherous betrayal cadence. Reset on betrayal.
    #[serde(default)]
    pub turns_since_last_betrayal: u8,
}

impl Default for Tribute {
    fn default() -> Self {
        Tribute::new("Default Tribute".to_string(), None, None)
    }
}

impl Tribute {
    /// Creates a new Tribute with full health, sanity, and movement.
    pub fn new(name: String, district: Option<u32>, avatar: Option<String>) -> Self {
        let district = district.unwrap_or(0);
        let attributes = Attributes::new();
        let statistics = Statistics::default();

        let id_uuid: Uuid = Uuid::new_v4();
        let id: String = id_uuid.to_string();

        // Assign terrain affinity, traits, and personality based on district
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let brain = Brain::new_with_random_personality(&mut rng);
        let terrain_affinity = if (1..=12).contains(&district) {
            crate::districts::assign_terrain_affinity(district as u8, &mut rng)
        } else {
            vec![]
        };
        let traits = traits::generate_traits(district as u8, &mut rng);

        Self {
            identifier: id,
            id: id_uuid,
            area: Area::Cornucopia,
            name: name.clone(),
            district,
            brain,
            status: TributeStatus::default(),
            avatar,
            human_player_name: None,
            attributes,
            statistics,
            items: vec![],
            events: vec![],
            editable: true,
            terrain_affinity,
            stamina: 100,
            max_stamina: 100,
            traits,
            allies: Vec::new(),
            turns_since_last_betrayal: 0,
        }
    }

    pub fn random() -> Self {
        let name = Name(EN).fake();
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let district = rng.random_range(1..=12);
        Tribute::new(name, Some(district), None)
    }

    pub fn avatar(&self) -> String {
        if self.avatar.is_none() {
            return "https://fallback.pics/api/v1/400x400".to_string();
        }
        format!("assets/{}", self.avatar.clone().unwrap())
    }

    /// Send a tribute through a game cycle.
    /// This is the main function that runs the tribute's actions.
    /// 1. Ignore dead tributes.
    /// 2. Process status effects including area events.
    /// 3. Check for gifts from sponsors.
    /// 4. Check for nighttime effects.
    /// 5. Check for suggested actions.
    /// 6. Get the tribute's action from the brain.
    /// 7. Perform the action.
    /// 8. Log the action.
    pub fn process_turn_phase(
        &mut self,
        action_suggestion: Option<ActionSuggestion>,
        environment_details: &mut EnvironmentContext<'_>,
        encounter_context: EncounterContext,
        rng: &mut impl Rng,
        events: &mut Vec<String>,
    ) {
        // Tribute is already dead, do nothing.
        if !self.is_alive() {
            events.push(GameOutput::TributeAlreadyDead(self.name.as_str()).to_string());
            return;
        }

        let area_details = &mut environment_details.area_details;

        // Update the tribute based on the period's events.
        self.process_status(area_details, rng, events);

        // Tribute died to the period's events.
        if self.status == TributeStatus::RecentlyDead || self.attributes.health == 0 {
            events.push(GameOutput::TributeDead(self.name.as_str()).to_string());
            return;
        }

        // Any generous patrons this round?
        if let Some(gift) = self.receive_patron_gift(&mut *rng) {
            events.push(GameOutput::SponsorGift(self.name.as_str(), &gift).to_string());
            self.add_item(gift);
        }

        // Nighttime terror
        let is_day = environment_details.is_day;
        if !is_day && self.is_alive() {
            self.misses_home();
        }

        // Check for psychotic breaks or recovery (sanity-based mental state changes)
        self.brain
            .check_psychotic_break(self.attributes.sanity, rng);
        self.brain.check_recovery(self.attributes.sanity);

        // Set a preferred action if one is suggested
        if let Some(suggestion) = action_suggestion {
            self.brain.set_preferred_action(
                suggestion.action,
                suggestion.probability.unwrap_or(1.0), // If no probability is set, perform the preferred action.
            );
        }

        // Get tribute action
        let number_of_nearby_tributes = encounter_context.nearby_tributes_count;
        let action = self.brain.act(
            self,
            number_of_nearby_tributes,
            &environment_details.available_destinations,
            rng,
        );

        let closed_areas = environment_details.closed_areas;

        match &action {
            Action::Move(area) => {
                let travel_result = match area {
                    Some(specific_area) => self.travels(closed_areas, Some(*specific_area), events),
                    None => self.travels(closed_areas, None, events),
                };

                match travel_result {
                    TravelResult::Success(destination) => {
                        // Find destination info from available_destinations
                        let dest_info = environment_details
                            .available_destinations
                            .iter()
                            .find(|d| d.area == destination);

                        match dest_info {
                            Some(info) => {
                                // Check if tribute has enough stamina
                                if self.stamina >= info.stamina_cost {
                                    // Move and deduct stamina
                                    self.area = destination;
                                    self.stamina = self.stamina.saturating_sub(info.stamina_cost);
                                } else {
                                    // Insufficient stamina - exhausted
                                    self.short_rests();
                                    events.push(
                                        GameOutput::TributeTravelExhausted(
                                            self.name.as_str(),
                                            &self.area.to_string(),
                                        )
                                        .to_string(),
                                    );
                                }
                            }
                            None => {
                                // Destination not in available_destinations (shouldn't happen)
                                self.short_rests();
                            }
                        }
                    }
                    TravelResult::Failure => {
                        self.short_rests();
                    }
                }
            }
            Action::Rest => {
                events.push(GameOutput::TributeRest(self.name.as_str()).to_string());
                self.long_rests();
            }
            Action::Hide => {
                let hidden = self.hides();
                if hidden {
                    events.push(GameOutput::TributeHide(self.name.as_str()).to_string());
                } else {
                    // Just log as regular hide, game doesn't distinguish failure in output
                    events.push(GameOutput::TributeHide(self.name.as_str()).to_string());
                }
            }
            Action::Attack => {
                let target = self.pick_target(
                    encounter_context.potential_targets,
                    encounter_context.total_living_tributes,
                    events,
                );
                if let Some(mut target) = target {
                    let outcome = self.attacks(&mut target, rng, events);
                    match outcome {
                        AttackOutcome::Kill(_, mut target) => {
                            self.statistics.kills += 1;
                            target.statistics.day_killed =
                                Some(self.statistics.game.parse().unwrap_or(1));
                        }
                        AttackOutcome::Wound(_, _) | AttackOutcome::Miss(_, _) => {}
                    }
                }
                // If no target, no output needed - already logged elsewhere
            }
            Action::TakeItem => {
                if let Some(item) = self.take_nearby_item(area_details) {
                    events.push(
                        GameOutput::TributeTakeItem(self.name.as_str(), &item.name).to_string(),
                    );
                }
                // If no items available, no output
            }
            Action::UseItem(maybe_item) => {
                if let Some(item) = maybe_item {
                    if let Err(error) = self.try_use_consumable(item) {
                        events.push(
                            GameOutput::TributeCannotUseItem(
                                self.name.as_str(),
                                &error.to_string(),
                            )
                            .to_string(),
                        );
                    } else {
                        events
                            .push(GameOutput::TributeUseItem(self.name.as_str(), item).to_string());
                    }
                }
            }
            Action::None => {
                // Tribute does nothing - no output needed
            }
        }
    }

    /// Pick an appropriate target from nearby tributes prioritizing targets as follows:
    /// (for this function, "nearby" means in the same area and "ally" means
    /// from the same district)
    /// 1. If there are enemy tributes nearby, target them.
    /// 2. If there are no enemies and the tribute is feeling suicidal, target self.
    /// 3. If there are no enemies nearby, but they exist elsewhere, target no one.
    /// 4. If there are no enemies nearby and no enemies left in the game:
    ///    a. If loyalty is low, target ally.
    ///    b. Otherwise, target no one.
    fn pick_target(
        &self,
        mut targets: Vec<Tribute>,
        living_tributes_count: u32,
        events: &mut Vec<String>,
    ) -> Option<Tribute> {
        // If there are no targets, check if the tribute is feeling suicidal.
        if targets.is_empty() {
            match self.attributes.sanity {
                0..=SANITY_BREAK_LEVEL => {
                    // attempt suicide
                    events.push(GameOutput::TributeSuicide(self.name.as_str()).to_string());
                    Some(self.clone())
                }
                _ => None, // Attack no one
            }
        } else {
            let enemies: Vec<Tribute> = targets
                .iter()
                .filter(|t| t.district != self.district)
                .cloned()
                .collect();

            match enemies.len() {
                0 => {
                    // No enemies, check for a "friend"
                    // If there are two of us in the area
                    if targets.len() == 1 {
                        let target = targets.pop().unwrap();
                        // And we're the only two left in the game
                        if living_tributes_count == 2 {
                            // Kill the other tribute (final confrontation)
                            Some(target)
                        } else if (self.attributes.loyalty as f64 / 100.0) < LOYALTY_BREAK_LEVEL {
                            // ...or they're unloyal (betrayal)
                            Some(target)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => {
                    // If there are enemies
                    let mut rng = SmallRng::from_rng(&mut rand::rng());
                    Some(enemies.choose(&mut rng).unwrap().clone())
                }
            }
        }
    }
}

/// Calculates the stamina cost for a tribute action based on:
/// - Base action cost
/// - Terrain movement multiplier
/// - Terrain affinity modifier (0.8 if tribute has affinity, 1.0 otherwise)
/// - Desperation multiplier based on health (1.0 + 0.5 * (1.0 - health%))
pub fn calculate_stamina_cost(
    action: &Action,
    terrain: &crate::terrain::TerrainType,
    tribute: &Tribute,
) -> u32 {
    // Base costs for each action type
    let base_cost: f32 = match action {
        Action::Move(_) => 20.0,
        Action::Hide => 15.0,
        Action::TakeItem => 10.0,
        Action::Attack => 25.0,
        Action::Rest | Action::None => 0.0,
        Action::UseItem(_) => 10.0,
    };

    // If base cost is 0, no need to calculate multipliers
    if base_cost == 0.0 {
        return 0;
    }

    // Terrain multiplier from movement_cost
    let terrain_multiplier = terrain.base.movement_cost();

    // Affinity modifier: 0.8 if tribute has affinity for this terrain, else 1.0
    let affinity_modifier = if tribute.terrain_affinity.contains(&terrain.base) {
        0.8
    } else {
        1.0
    };

    // Desperation multiplier: 1.0 + (0.5 × (1.0 - health%))
    let health_percent = tribute.attributes.health as f32 / 100.0;
    let desperation_multiplier = 1.0 + (0.5 * (1.0 - health_percent));

    // Calculate final cost with all multipliers
    let final_cost = base_cost * terrain_multiplier * affinity_modifier * desperation_multiplier;

    // Round to nearest integer
    final_cost.round() as u32
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Statistics {
    /// What day, if any, were they killed?
    pub day_killed: Option<u32>,
    /// Who or what killed them?
    pub killed_by: Option<String>,
    /// How many tributes did they kill?
    pub kills: u32,
    /// How many fights did they win?
    pub wins: u32,
    /// How many fights did they lose?
    pub defeats: u32,
    /// How many fights ended in a draw?
    pub draws: u32,
    /// Which game do these stats relate to?
    pub game: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Attributes {
    /// How much damage can they take?
    pub health: u32,
    /// How much suffering can they handle? Are they still sane?
    pub sanity: u32,
    /// How far can they move before they need a rest?
    pub movement: u32,
    /// How hard do they hit?
    pub strength: u32,
    /// How hard of a hit can they take?
    pub defense: u32,
    /// Will they jump into dangerous situations?
    pub bravery: u32,
    /// Are they a backstabber?
    pub loyalty: u32,
    /// How well do they avoid traps?
    pub intelligence: u32,
    /// Can they talk their way out of, or into, things?
    pub persuasion: u32,
    /// Are they likely to get gifts or come out slightly ahead?
    pub luck: u32,
    /// Can other tributes see them?
    pub is_hidden: bool,
}

impl Default for Attributes {
    /// Provides a maxed-out set of Attributes
    fn default() -> Self {
        Self {
            health: 100,
            sanity: 100,
            movement: 100,
            strength: 50,
            defense: 50,
            bravery: 100,
            loyalty: 100,
            intelligence: 100,
            persuasion: 100,
            luck: 100,
            is_hidden: false,
        }
    }
}

impl Attributes {
    /// Provides a randomized set of Attributes using default config values
    pub fn new() -> Self {
        let mut rng = SmallRng::from_rng(&mut rand::rng());
        let config = crate::config::GameConfig::default();

        Self {
            health: rng.random_range(50..=config.max_health),
            sanity: rng.random_range(50..=config.max_sanity),
            movement: config.max_movement,
            strength: rng.random_range(1..=config.max_strength),
            defense: rng.random_range(1..=config.max_defense),
            bravery: rng.random_range(1..=config.max_bravery),
            loyalty: rng.random_range(1..=config.max_loyalty),
            intelligence: rng.random_range(1..=config.max_intelligence),
            persuasion: rng.random_range(1..=config.max_persuasion),
            luck: rng.random_range(1..=config.max_luck),
            is_hidden: false,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::tributes::Tribute;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rstest::*;

    #[fixture]
    fn tribute() -> Tribute {
        Tribute::new("Katniss".to_string(), None, None)
    }

    #[fixture]
    fn target() -> Tribute {
        Tribute::new("Peeta".to_string(), None, None)
    }

    #[fixture]
    fn small_rng() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    #[rstest]
    fn default() {
        let tribute = Tribute::default();
        assert_eq!(tribute.name, "Default Tribute");
    }

    #[rstest]
    fn new() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        assert_eq!(tribute.name, "Katniss");
        assert_eq!(tribute.district, 12);
        // Attributes::new() randomizes health in 50..=max_health.
        assert!(
            (50..=100).contains(&tribute.attributes.health),
            "health {} out of range",
            tribute.attributes.health
        );
    }

    #[rstest]
    fn random() {
        let tribute = Tribute::random();
        assert!(!tribute.name.is_empty());
        assert!(tribute.district >= 1 && tribute.district <= 12);
    }

    #[rstest]
    fn new_tribute_has_empty_alliance_state() {
        let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert!(tribute.allies.is_empty());
        assert_eq!(tribute.turns_since_last_betrayal, 0);
        // `id` mirrors `identifier`.
        assert_eq!(tribute.id.to_string(), tribute.identifier);
    }

    #[rstest]
    fn new_tribute_has_traits_for_valid_district() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        // generate_traits rolls 2..=6 traits from the district pool.
        assert!((2..=6).contains(&tribute.traits.len()));
    }
}
