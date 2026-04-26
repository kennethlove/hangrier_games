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
    /// Set to `true` by the cycle drain when this tribute is the victim of a
    /// betrayal. Consumed at the top of `process_turn_phase` on the victim's
    /// next turn to drive the trust-shock cascade (spec §7.3c1).
    #[serde(default)]
    pub pending_trust_shock: bool,
    /// Per-tribute alliance event buffer, populated during `process_turn_phase`
    /// (e.g. on Treacherous betrayal) and drained by the game cycle into
    /// `Game.alliance_events` between turns. Transient; never persisted.
    #[serde(default, skip)]
    pub alliance_events: Vec<alliances::AllianceEvent>,
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
        let terrain_affinity = if (1..=12).contains(&district) {
            crate::districts::assign_terrain_affinity(district as u8, &mut rng)
        } else {
            vec![]
        };
        let traits = traits::generate_traits(district as u8, &mut rng);
        let brain = Brain::from_traits(&traits, &mut rng);

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
            pending_trust_shock: false,
            alliance_events: Vec::new(),
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

        // Advance per-tribute alliance timers (spec §7.4). Ticks
        // turns_since_last_betrayal so Treacherous tributes can betray on
        // cadence. Skipped for dead tributes via `is_alive` guard above.
        self.tick_alliance_timers();

        // Consume any pending trust-shock from a betrayal recorded last turn
        // (spec §7.3c1). Rolls per ally; broken allies are removed locally.
        self.consume_pending_trust_shock(rng, events);

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
    /// Pick a target tribute from `targets` to attack, given the number of
    /// living tributes in the game.
    ///
    /// Selection rules:
    /// 1. If there are no targets and the tribute is suicidal (very low sanity),
    ///    target self.
    /// 2. Otherwise, filter out current allies — they are off-limits regardless
    ///    of district.
    /// 3. If any non-allies remain, pick one at random.
    /// 4. If only allies are nearby (and we're not the last two alive), pick no
    ///    target. Final confrontation (only two alive) overrides alliance.
    fn pick_target(
        &self,
        mut targets: Vec<Tribute>,
        living_tributes_count: u32,
        events: &mut Vec<String>,
    ) -> Option<Tribute> {
        // If there are no targets, check if the tribute is feeling suicidal.
        if targets.is_empty() {
            return match self.attributes.sanity {
                0..=SANITY_BREAK_LEVEL => {
                    // attempt suicide
                    events.push(GameOutput::TributeSuicide(self.name.as_str()).to_string());
                    Some(self.clone())
                }
                _ => None, // Attack no one
            };
        }

        let enemies: Vec<Tribute> = targets
            .iter()
            .filter(|t| !self.allies.contains(&t.id))
            .cloned()
            .collect();

        if enemies.is_empty() {
            // Only allies in range. Final confrontation overrides loyalty.
            if targets.len() == 1 && living_tributes_count == 2 {
                return Some(targets.pop().unwrap());
            }
            return None;
        }

        let mut rng = SmallRng::from_rng(&mut rand::rng());
        Some(enemies.choose(&mut rng).unwrap().clone())
    }

    /// Drain this tribute's per-turn alliance event buffer. Called by
    /// `Game::run_tribute_cycle` after each tribute's turn so the events
    /// are appended to the game's queue and processed before the next
    /// tribute acts. See spec §7.5.
    pub fn drain_alliance_events(&mut self) -> Vec<alliances::AllianceEvent> {
        std::mem::take(&mut self.alliance_events)
    }

    /// Advance per-tribute alliance bookkeeping for one turn (spec §7.4).
    ///
    /// Ticks `turns_since_last_betrayal`, which gates Treacherous-trait
    /// betrayals to fire at most every
    /// [`alliances::TREACHEROUS_BETRAYAL_INTERVAL`] turns. Saturates at
    /// `u8::MAX` so a long-lived tribute never overflows or wraps back
    /// through the betrayal trigger. Dead tributes are skipped.
    pub fn tick_alliance_timers(&mut self) {
        if !self.is_alive() {
            return;
        }
        self.turns_since_last_betrayal = self.turns_since_last_betrayal.saturating_add(1);
    }

    /// Consume a pending trust-shock flag (set when this tribute was the
    /// victim of a betrayal). For each current ally, roll
    /// [`alliances::trust_shock_roll`]; on success drop that ally from this
    /// tribute's `allies` list and emit a message. The flag is reset
    /// unconditionally so it never carries past the turn it fires.
    ///
    /// Note: this only mutates `self`. The symmetric back-edge on the broken
    /// ally's side is left to the next cycle's processing or to subsequent
    /// alliance events; per Phase 4 of the implementation plan, full
    /// symmetric cleanup is deferred. See spec §7.3c1.
    pub fn consume_pending_trust_shock(
        &mut self,
        rng: &mut impl rand::Rng,
        events: &mut Vec<String>,
    ) {
        if !self.pending_trust_shock {
            return;
        }
        let limit = self.brain.thresholds.extreme_low_sanity;
        let sanity = self.attributes.sanity;
        let mut broken: Vec<Uuid> = Vec::new();
        for ally_id in &self.allies {
            if alliances::trust_shock_roll(sanity, limit, rng) {
                broken.push(*ally_id);
            }
        }
        for ally_id in &broken {
            self.allies.retain(|x| x != ally_id);
            events.push(format!(
                "{} loses faith and breaks ties with ally {}.",
                self.name, ally_id
            ));
        }
        self.pending_trust_shock = false;
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
    fn serde_roundtrip_alliance_fields() {
        use crate::tributes::traits::Trait;
        use uuid::Uuid;

        let mut tribute = Tribute::new("Rue".to_string(), None, None);
        let ally = Uuid::new_v4();
        tribute.allies.push(ally);
        tribute.traits.clear();
        tribute.traits.push(Trait::Loyal);
        tribute.traits.push(Trait::Treacherous);
        tribute.turns_since_last_betrayal = 7;
        tribute.pending_trust_shock = true;

        let json = serde_json::to_string(&tribute).expect("serialize");
        assert!(json.contains("\"allies\""));
        assert!(json.contains("\"traits\""));
        assert!(json.contains("\"Loyal\""));
        assert!(json.contains("\"turns_since_last_betrayal\":7"));
        assert!(json.contains("\"pending_trust_shock\":true"));

        let restored: Tribute = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.allies, vec![ally]);
        assert_eq!(restored.traits, vec![Trait::Loyal, Trait::Treacherous]);
        assert_eq!(restored.turns_since_last_betrayal, 7);
        assert!(restored.pending_trust_shock);
    }

    #[rstest]
    fn serde_defaults_for_missing_alliance_fields() {
        // Persisted tribute records written before the alliance fields existed
        // must still deserialize. Simulate this by serialising a fresh tribute,
        // stripping the new fields, then round-tripping.
        let baseline = Tribute::new("Legacy".to_string(), None, None);
        let mut value: serde_json::Value = serde_json::to_value(&baseline).expect("to_value");
        let obj = value.as_object_mut().expect("object");
        obj.remove("allies");
        obj.remove("traits");
        obj.remove("turns_since_last_betrayal");
        obj.remove("pending_trust_shock");

        let restored: Tribute = serde_json::from_value(value).expect("legacy deserialize");
        assert!(restored.allies.is_empty());
        assert!(restored.traits.is_empty());
        assert_eq!(restored.turns_since_last_betrayal, 0);
        assert!(!restored.pending_trust_shock);
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
    fn new_tribute_has_no_pending_trust_shock() {
        let tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert!(!tribute.pending_trust_shock);
    }

    #[rstest]
    fn tribute_drain_alliance_events_returns_and_clears_buffer() {
        use crate::tributes::alliances::AllianceEvent;
        use uuid::Uuid;
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        let other = Uuid::new_v4();
        tribute
            .alliance_events
            .push(AllianceEvent::BetrayalRecorded {
                betrayer: tribute.id,
                victim: other,
            });
        let drained = tribute.drain_alliance_events();
        assert_eq!(drained.len(), 1);
        assert!(tribute.alliance_events.is_empty());
    }

    #[rstest]
    fn consume_pending_trust_shock_resets_flag_when_not_set() {
        // No flag → no rolls, flag stays false, allies untouched.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        let ally = uuid::Uuid::new_v4();
        tribute.allies.push(ally);
        let mut events: Vec<String> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(53);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);
        assert!(!tribute.pending_trust_shock);
        assert_eq!(tribute.allies, vec![ally]);
        assert!(events.is_empty());
    }

    #[rstest]
    fn consume_pending_trust_shock_breaks_allies_on_success_and_clears_flag() {
        // Force trust_shock to fire deterministically: sanity=0, threshold>0
        // gives p = 0.5 + 0.5 * 1.0 = 1.0 → always true.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.sanity = 0;
        tribute.brain.thresholds.extreme_low_sanity = 50;
        let ally1 = uuid::Uuid::new_v4();
        let ally2 = uuid::Uuid::new_v4();
        tribute.allies.push(ally1);
        tribute.allies.push(ally2);
        tribute.pending_trust_shock = true;

        let mut events: Vec<String> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(211);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);

        assert!(!tribute.pending_trust_shock, "flag must reset");
        assert!(
            tribute.allies.is_empty(),
            "all allies broken on guaranteed success"
        );
        assert_eq!(events.len(), 2, "one message per broken ally");
    }

    #[rstest]
    fn consume_pending_trust_shock_no_break_when_sanity_above_threshold() {
        // Sanity at/above threshold → trust_shock_roll returns false → no break.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.sanity = 100;
        tribute.brain.thresholds.extreme_low_sanity = 50;
        let ally = uuid::Uuid::new_v4();
        tribute.allies.push(ally);
        tribute.pending_trust_shock = true;

        let mut events: Vec<String> = vec![];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(89);
        tribute.consume_pending_trust_shock(&mut rng, &mut events);

        assert!(!tribute.pending_trust_shock, "flag must reset");
        assert_eq!(tribute.allies, vec![ally], "ally retained");
        assert!(events.is_empty());
    }

    #[rstest]
    fn new_tribute_has_traits_for_valid_district() {
        let tribute = Tribute::new("Katniss".to_string(), Some(12), None);
        // generate_traits rolls 2..=6 traits from the district pool.
        assert!((2..=6).contains(&tribute.traits.len()));
    }

    #[rstest]
    fn pick_target_skips_allies() {
        // An ally is in the same area but must not be picked as a target.
        let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
        me.attributes.sanity = 100; // not suicidal
        let ally = Tribute::new("Peeta".to_string(), Some(12), None);
        me.allies.push(ally.id);

        let mut events: Vec<String> = vec![];
        let target = me.pick_target(vec![ally.clone()], 5, &mut events);
        // Only candidate was an ally and we're not in final confrontation.
        assert!(target.is_none());
    }

    #[rstest]
    fn pick_target_allows_same_district_when_not_ally() {
        // Same-district tributes can now be targeted unless they're allies.
        let me = Tribute::new("Katniss".to_string(), Some(12), None);
        let same_district = Tribute::new("Peeta".to_string(), Some(12), None);

        let mut events: Vec<String> = vec![];
        let target = me.pick_target(vec![same_district.clone()], 5, &mut events);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id, same_district.id);
    }

    #[rstest]
    fn pick_target_final_confrontation_overrides_alliance() {
        // When only two tributes remain alive, even an ally is a valid target.
        let mut me = Tribute::new("Katniss".to_string(), Some(12), None);
        me.attributes.sanity = 100;
        let ally = Tribute::new("Peeta".to_string(), Some(12), None);
        me.allies.push(ally.id);

        let mut events: Vec<String> = vec![];
        let target = me.pick_target(vec![ally.clone()], 2, &mut events);
        assert!(target.is_some());
        assert_eq!(target.unwrap().id, ally.id);
    }

    #[rstest]
    fn tick_alliance_timers_increments_betrayal_counter() {
        // Living tribute: counter increments by exactly one per tick.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        assert_eq!(tribute.turns_since_last_betrayal, 0);
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 1);
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 2);
    }

    #[rstest]
    fn tick_alliance_timers_saturates_does_not_overflow() {
        // u8 saturating add: never panics, never wraps to zero.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.turns_since_last_betrayal = u8::MAX;
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, u8::MAX);
    }

    #[rstest]
    fn tick_alliance_timers_skips_dead_tributes() {
        // Dead tributes don't accumulate betrayal cooldown.
        let mut tribute = Tribute::new("Cinna".to_string(), Some(1), None);
        tribute.attributes.health = 0;
        tribute.status = crate::tributes::TributeStatus::RecentlyDead;
        tribute.tick_alliance_timers();
        assert_eq!(tribute.turns_since_last_betrayal, 0);
    }
}
